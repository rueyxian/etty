//! Convenience wrapper for [`stdout`][mod-stdout].
//!
//! This module provides convenience wrapper for [`Write`][mod-write] and [`stdout`][mod-stdout].
//! ```
//! use etty::StdoutWrite;
//! fn main() {
//!     etty::ers_all().outw();
//!     etty::cusr_goto(15, 10).outw();
//!     etty::sgr!(etty::STY_BOLD_SET, etty::FG_YEL).outw();
//!     "hello world".outw();
//!     etty::sgr_rst().outw();
//!     etty::flush();
//! }
//! ```
//! [mod-stdout]: std::io::stdout
//! [mod-write]: std::io::Write

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
