//! An easy tty library.
//!
//! This library aim to be easy to use without being overly abstracted. Key components:
//! * ANSI [CSI][wiki-csi] builder.
//! * Convenience wrapper and macros for [`stdout`][mod-stdout].
//! * Event handler.
//!
//! [wiki-csi]: https://en.wikipedia.org/wiki/ANSI_escape_code#CSI_(Control_Sequence_Introducer)_sequences
//! [mod-output]: crate::output
//! [mod-stdout]: std::io::stdout

pub mod macros {
    //! Convenience macros for [`stdout`][mod-stdout].
    //!
    //! Re-export from [`etty_macros`][mod-etty-macro].
    //!
    //! [mod-stdout]: std::io::stdout
    //! [mod-etty-macro]: etty_macros
    pub use etty_macros::*;
}
#[doc(hidden)]
pub use macros::*;

mod util;
pub(crate) use crate::util::*;

mod unix;

pub mod term;
#[doc(hidden)]
pub use term::*;

mod input;

pub mod output;
#[doc(hidden)]
pub use output::*;

pub mod evt;
#[doc(hidden)]
pub use evt::event_stream;

pub mod csi;
#[doc(hidden)]
pub use csi::*;

pub mod sgr_const;
#[doc(hidden)]
pub use sgr_const::*;

pub mod c0;
#[doc(hidden)]
pub use c0::C0;

// pub mod macros;
