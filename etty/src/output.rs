#![allow(clippy::explicit_write)]

use std::io::Write;

pub trait StdoutWrite: std::fmt::Display {
    fn outw(&self) {
        std::write!(std::io::stdout(), "{}", self).unwrap();
    }

    fn outwln(&self) {
        std::write!(std::io::stdout(), "{}\n", self).unwrap();
    }
}

impl<T> StdoutWrite for T where T: std::fmt::Display {}

pub fn flush() {
    std::io::stdout().flush().unwrap()
}

pub fn stdout_lock<'a>() -> std::io::StdoutLock<'a> {
    std::io::stdout().lock()
}

pub fn outw(b: &[u8]) -> usize {
    std::io::stdout().write(b).unwrap()
}
