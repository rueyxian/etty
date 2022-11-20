use libc::c_ushort;
use libc::ioctl;
use libc::termios;
use libc::STDOUT_FILENO;
use libc::TIOCGWINSZ;

pub(crate) fn get_term_attr() -> std::io::Result<Termios> {
    unsafe {
        let mut termios = std::mem::zeroed();
        let res = libc::tcgetattr(libc::STDOUT_FILENO, &mut termios);
        if res.is_negative() {
            return Err(std::io::Error::last_os_error());
        }
        Ok(Termios(termios))
    }
}

#[derive(Clone)]
pub(crate) struct Termios(termios);

impl Termios {
    pub(crate) fn into_raw(mut self) -> Termios {
        unsafe {
            libc::cfmakeraw(&mut self.0);
        }
        self
    }

    pub(crate) fn set_attr(&mut self) -> std::io::Result<()> {
        let res = unsafe { libc::tcsetattr(libc::STDOUT_FILENO, libc::TCSANOW, &self.0) };
        if res.is_negative() {
            return Err(std::io::Error::last_os_error());
        }
        Ok(())
    }
}

pub(crate) fn get_tty_file() -> std::io::Result<std::fs::File> {
    std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open("/dev/tty")
}

#[repr(C)]
pub(crate) struct TermSize {
    pub(crate) row: c_ushort,
    pub(crate) col: c_ushort,
    pub(crate) x: c_ushort,
    pub(crate) y: c_ushort,
}
pub(crate) fn get_term_size() -> std::io::Result<TermSize> {
    unsafe {
        let mut term_size: TermSize = std::mem::zeroed();
        let res = ioctl(STDOUT_FILENO, TIOCGWINSZ, &mut term_size as *mut _);
        if res.is_negative() {
            return Err(std::io::Error::last_os_error());
        }
        Ok(term_size)
    }
}
