//! Convenience wrapper for [`stdout`][mod-stdout].
//!
//! This module provides convenience wrapper for [`Write`][mod-write] and [`stdout`][mod-stdout].
//! ```
//! use etty::StdoutWrite;
//! fn main() {
//!     etty::ers_all().out();
//!     etty::cusr_goto(15, 10).out();
//!     etty::sgr!(etty::STY_BOLD_SET, etty::FG_YEL).out();
//!     "hello world".out();
//!     etty::sgr_rst().out();
//!     etty::flush();
//! }
//! ```
//! [mod-stdout]: std::io::stdout
//! [mod-write]: std::io::Write

#![allow(clippy::explicit_write)]

use std::io::Write;

pub trait StdoutWrite: std::fmt::Display {
    fn out(&self) {
        std::write!(std::io::stdout(), "{}", self).unwrap();
    }

    fn outln(&self) {
        std::write!(std::io::stdout(), "{}\n", self).unwrap();
    }

    fn print(&self) {
        self.out();
        flush();
    }

    fn println(&self) {
        self.outln();
        flush();
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
