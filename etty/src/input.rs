#![allow(clippy::type_complexity)]

use std::io::Read;

use crate::unix::get_tty_file;

pub(crate) fn async_stdin() -> AsyncStdin {
    AsyncStdin {
        timeout: None,
        until: None,
    }
}

pub(crate) struct AsyncStdin {
    timeout: Option<std::time::Duration>,
    until: Option<Box<dyn Fn(&Result<&u8, &std::io::Error>) -> bool + Send + Sync + 'static>>,
}

impl AsyncStdin {
    pub(crate) fn timeout(&mut self, timeout: std::time::Duration) -> &mut Self {
        self.timeout = Some(timeout);
        self
    }
    pub(crate) fn until<F>(&mut self, f: F) -> &mut Self
    where
        F: Fn(&Result<&u8, &std::io::Error>) -> bool + Send + Sync + 'static,
    {
        self.until = Some(Box::new(f));
        self
    }
    pub(crate) fn init(&mut self) -> (AsyncReader, std::thread::JoinHandle<()>) {
        let (tx, rx) = crossbeam::channel::unbounded::<std::io::Result<u8>>();
        let until = self.until.take();
        let jh = std::thread::spawn(move || {
            let stdin = get_tty_file().unwrap();
            // let stdin = std::io::stdin();
            for b in stdin.bytes() {
                let is_eos = until.as_ref().map(|f| f(&b.as_ref())).unwrap_or(false);
                let is_err = tx.send(b).is_err();
                if is_eos || is_err {
                    return;
                }
            }
        });
        let reader = AsyncReader {
            rx,
            timeout: self.timeout,
        };
        (reader, jh)
    }
}

pub(crate) struct AsyncReader {
    rx: crossbeam::channel::Receiver<std::io::Result<u8>>,
    timeout: Option<std::time::Duration>,
}

impl std::io::Read for AsyncReader {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        use crossbeam::channel::RecvTimeoutError;
        let recv = || -> Result<Result<u8, std::io::Error>, RecvTimeoutError> {
            if let Some(timeout) = self.timeout {
                self.rx.recv_timeout(timeout)
            } else {
                self.rx.recv().map_err(|_| RecvTimeoutError::Disconnected)
            }
        };
        let mut idx = 0;
        while idx < buf.len() {
            let b = match recv() {
                Err(RecvTimeoutError::Disconnected) => break,
                Err(RecvTimeoutError::Timeout) => {
                    let err = std::io::Error::new(
                        std::io::ErrorKind::TimedOut,
                        "`etty::input::AsyncReader::read()` timeout",
                    );
                    return Err(err);
                }
                Ok(msg) => msg?,
            };
            buf[idx] = b;
            idx += 1;
        }
        Ok(idx)
    }
}
