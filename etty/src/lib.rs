mod unix;

mod term;
pub use term::raw_mode;
pub use term::term_mode;
pub use term::term_size;
pub use term::term_size_px;
pub use term::TermMode;

mod input;

mod output;
pub use output::*;

pub mod event;
pub use event::event_stream;

mod csi;
pub use csi::*;

pub mod c0;
pub use c0::C0;

// re-export from `etty_macro` crate
pub use etty_macro::sgr;
pub use etty_macro::write_fmt;
