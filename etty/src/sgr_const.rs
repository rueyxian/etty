//! Constants for building [Select Graphic Rendition][wiki-sgr] (SGR).
//!
//! These `u8` constants represent [SGR parameters][wiki-sgr]. It is expected to be used in conjunction with [etty_macros::sgr][mod-sgr] macro.
//!
//! ```rust
//! fn main() {
//!     // consider this
//!     let sgr1 = etty::sgr!(etty::STY_BOLD_SET, etty::FG_RED, etty::BG_BRGT_YEL);
//!     assert_eq!(sgr1, "\x1b[1;31;103m");
//!
//!     // than this
//!     let sgr2 = format!("{}{}{}", etty::sty_bold_set(), etty::fg_red(), etty::bg_brgt_yel());
//!     assert_eq!(sgr2, "\x1b[1m\x1b[31m\x1b[103m");
//! }
//! ```
//!
//! [wiki-sgr]: https://en.wikipedia.org/wiki/ANSI_escape_code#SGR_(Select_Graphic_Rendition)_parameters
//! [mod-sgr]: etty_macros::sgr

pub const SGR_RST: u8 = 0;

pub const STY_BOLD_SET: u8 = 1;
pub const STY_BOLD_RST: u8 = 21;

etty_macros::gen_sty_const! {
    2
    DIM,
    ITALIC,
    UNDERLN,
    BLINK,
}

etty_macros::gen_sty_const! {
    7
    INVRS,
    HIDE,
    STRKTHRU,
}

etty_macros::gen_clr_const! {
    30 =>
    BLK,
    RED,
    GRN,
    YEL,
    BLU,
    MAG,
    CYN,
    WHT,
}

etty_macros::gen_clr_const! {
    39 =>
    RST,
}

etty_macros::gen_clr_const! {
    90 =>
    BRGT_BLK,
    BRGT_RED,
    BRGT_GRN,
    BRGT_YEL,
    BRGT_BLU,
    BRGT_MAG,
    BRGT_CYN,
    BRGT_WHT,
}
