//! Convenience wrapper for [`stdout`][mod-stdout].
//!
//! This module provides convenience wrapper for [`stdout`][mod-stdout].
//!
//! [mod-stdout]: std::io::stdout

// #![allow(clippy::explicit_write)]

use std::io::Write;

// pub trait StdoutWrite: std::fmt::Display {
//     fn out(&self) {
//         std::write!(std::io::stdout(), "{}", self).unwrap();
//     }

//     fn outln(&self) {
//         std::writeln!(std::io::stdout(), "{}", self).unwrap();
//     }

//     fn outf(&self) {
//         self.out();
//         std::io::stdout().flush().unwrap();
//     }
// }

// impl<T> StdoutWrite for T where T: std::fmt::Display {}

pub fn flush() {
    std::io::stdout().flush().unwrap()
}

pub fn outlock<'a>() -> std::io::StdoutLock<'a> {
    std::io::stdout().lock()
}
