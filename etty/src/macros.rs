//! Convenience macros for [`stdout`][mod-stdout].
//!
//! Re-export from [`etty_macros`][mod-etty-macro].
//!
//! [mod-stdout]: std::io::stdout
//! [mod-etty-macro]: etty_macros

pub use etty_macros::out;

pub use etty_macros::outf;

pub use etty_macros::outln;

pub use etty_macros::sgr;
