#![allow(clippy::explicit_write)]

use std::io::Write;

pub fn flush() {
    std::io::stdout().flush().unwrap()
}

pub fn stdout_lock<'a>() -> std::io::StdoutLock<'a> {
    std::io::stdout().lock()
}

pub fn write(b: &[u8]) -> usize {
    std::io::stdout().write(b).unwrap()
}

pub fn write_char(c: char) {
    write!(std::io::stdout(), "{}", c).unwrap();
}

pub fn write_str(s: &str) {
    write!(std::io::stdout(), "{}", s).unwrap();
}
