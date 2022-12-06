//! Functions for building [Control Sequence Introducer][wiki] (CSI).
//!  
//! ```rust
//! assert_eq!(etty::ers_all().to_string(), "\x1b[2J");
//! assert_eq!(etty::ers_char(3).to_string(), "\x1b[3J");
//! assert_eq!(etty::cus_goto(5, 15).to_string(), "\x1b[15;5H");
//! assert_eq!(etty::sty_blink_rst().to_string(), "\x1b[25m");
//! assert_eq!(etty::fg_rgb(42, 99, 123).to_string(), "\x1b[38;2;42;99;123m");
//! assert_eq!(etty::evt_mouse_set().to_string(), "\x1b[?1000h");
//! ```
//!
//! To learn more about ANSI CSI:
//! * [wikipedia](https://en.wikipedia.org/wiki/ANSI_escape_code#CSI_(Control_Sequence_Introducer)_sequences)
//! * [gist/github.com/fnky](https://gist.github.com/fnky/458719343aabd01cfb17a3a4f7296797)
//! * [invisible-island.net](https://invisible-island.net/xterm/manpage/xterm.html)
//! * [vt100.net](https://vt100.net/docs/vt510-rm/chapter4.html)
//!
//! [wiki]: https://en.wikipedia.org/wiki/ANSI_escape_code#CSI_(Control_Sequence_Introducer)_sequences

#![allow(clippy::explicit_write)]

use std::borrow::Cow;
use std::io::Read;
use std::io::Write;

use crate::input;

/// Representation CSI sequence.
///
/// `Csi` provides convenience methods for writing into [`std::io::Stdout`](std::io::Stdout).
/// It just a string wrapper, [`std::borrow::Cow<str>`](std::borrow::Cow) specifically.
/// Created by functions in [`etty::csi`](etty::csi) module.
pub struct Csi<'a>(Cow<'a, str>);

impl<'a> std::fmt::Display for Csi<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl<'a> Csi<'a> {
    /// Writes into [`std::io::Stdout`](std::io::Stdout).
    pub fn out(&self) {
        std::write!(std::io::stdout(), "{}", self).unwrap();
    }
    /// Same with `Csi::out` but with newline.
    pub fn outln(&self) {
        std::writeln!(std::io::stdout(), "{}", self).unwrap();
    }
    /// Same with `Csi::out` but perform [`std::io::Stdout::flush`](std::io::Stdout::flush) immediately.
    pub fn outf(&self) {
        self.out();
        std::io::stdout().flush().unwrap();
    }
}

pub fn read_cus_pos() -> (u16, u16) {
    let (mut stdin, jh) = {
        let (stdin, jh) = input::async_stdin()
            .timeout(std::time::Duration::from_millis(100))
            .until(|b| b.map(|&b| b == b'R').unwrap_or(false))
            .init();
        (stdin.bytes(), jh)
    };
    let mut next = || stdin.next().unwrap().unwrap();

    cus_pos_rpt().out();
    std::io::stdout().flush().unwrap();

    while next() != b'\x1b' {}
    assert_eq!(next(), b'[');
    let mut f = |til: u8| -> u16 {
        let mut buf = [0_u8; 4];
        let mut i = 0;
        loop {
            match next() {
                b if b == til => break,
                b @ b'0'..=b'9' => {
                    buf[i] = b;
                    i += 1;
                }
                _ => unreachable!(),
            }
        }
        crate::bytes_to_uint::<u16>(&buf[0..i]).unwrap()
    };
    jh.join().unwrap();
    let (y, x) = (f(b';'), f(b'R'));
    (x, y)
}

// cursor
etty_macros::gen_csi! {
    // pub mod cus;
    pub cus_pos_rpt => "6n";

    pub cus_up => "{n}A", n;
    pub cus_dn => "{n}B", n;
    pub cus_rgt => "{n}C", n;
    pub cus_lft => "{n}D", n;

    pub cus_next_ln => "{n}E", n;
    pub cus_prev_ln => "{n}F", n;
    pub cus_goto_x => "{x}G", x;
    pub cus_goto_y => "{y}d", y;

    pub cus_home => "H";
    pub cus_goto => "{y};{x}H", x, y;

    pub cus_save => " 7";
    pub cus_load => " 8";

    pub cus_show => "?25h";
    pub cus_hide => "?25l";
}

// // c0
// etty_macros::gen_csi! {
//     mod c0;
//     pub c0_show => "3h";
//     pub c0_intp => "3L";
// }

// // term
// etty_macros::gen_csi! {
//     mod term;
//     pub term_soft_rst => "!p";
// }

// erase
etty_macros::gen_csi! {
    // mod ers;
    pub ers_aft_cus => "0J";
    pub ers_bfr_cus => "1J";
    pub ers_all => "2J";
    pub ers_all_and_saved => "3J";
    pub ers_ln_aft_cus => "0K";
    pub ers_ln_bfr_cus => "1K";
    pub ers_ln => "2K";
    pub ers_char => "{n}X", n;
}

// // selective erse
// etty_macros::gen_csi! {
//     mod sel_ers;
//     pub sel_ers_set => "0\"q";
//     pub sel_ers_rst => "1\"q";

//     pub sel_ers_aft_cus => "?0J";
//     pub sel_ers_bfr_cus => "?1J";
//     pub sel_ers_all => "?2J";
//     pub sel_ers_all_and_saved => "?3J";
//     pub sel_ers_ln_aft_cus => "?0K";
//     pub sel_ers_ln_bfr_cus => "?1K";
//     pub sel_ers_ln => "?2K";
// }

// delete
etty_macros::gen_csi! {
    // mod del;
    pub del_char => "{n}P", n;
    pub del_col => "{n}'~", n; // TODO idk why it kenot werks
    pub del_ln => "{n}M", n;
}

// insert
etty_macros::gen_csi! {
    // mod ins;
    pub ins_char => "{n}@", n;
    pub ins_col => "{n}'}}", n; // double '}' for escaping
    pub ins_ln => "{n}L", n;

    pub ins_rpl_set => "4h";
    pub ins_rpl_rst => "4l";
}

// scroll
etty_macros::gen_csi! {
    // mod scrl;
    pub scrl_up => "{n}S", n;
    pub scrl_dn => "{n}T", n;
}

// private modes
etty_macros::gen_csi! {
    // mod private;
    pub scrn_save => "?47h";
    pub scrn_load => "?47l";
    pub alt_buf_set => "?1049h";
    pub alt_buf_rst => "?1049l";
}

etty_macros::gen_csi! {
    // mod sgr;
    pub sgr_rst => "0m";

    pub sty_bold_set => "1m";
    pub sty_bold_rst => "21m";

    pub sty_dim_set => "2m";
    pub sty_dim_rst => "22m";

    pub sty_italic_set => "3m";
    pub sty_italic_rst => "23m";

    pub sty_underln_set => "4m";
    pub sty_underln_rst => "24m";

    pub sty_blink_set => "5m";
    pub sty_blink_rst => "25m";

    pub sty_invrs_set => "7m";
    pub sty_invrs_rst => "27m";

    pub sty_hide_set => "8m";
    pub sty_hide_rst => "28m";

    pub sty_strkthru_set => "9m";
    pub sty_strkthru_rst => "29m";

    // color default
    pub fg_rst => "39m";
    pub bg_rst => "49m";

    // color
    pub fg_blk => "30m";
    pub fg_red => "31m";
    pub fg_grn => "32m";
    pub fg_yel => "33m";
    pub fg_blu => "34m";
    pub fg_mag => "35m";
    pub fg_cyn => "36m";
    pub fg_wht => "37m";
    pub bg_blk => "40m";
    pub bg_red => "41m";
    pub bg_grn => "42m";
    pub bg_yel => "43m";
    pub bg_blu => "44m";
    pub bg_mag => "45m";
    pub bg_cyn => "46m";
    pub bg_wht => "47m";

    // color bright
    pub fg_brgt_blk => "90m";
    pub fg_brgt_red => "91m";
    pub fg_brgt_grn => "92m";
    pub fg_brgt_yel => "93m";
    pub fg_brgt_blu => "94m";
    pub fg_brgt_mag => "95m";
    pub fg_brgt_cyn => "96m";
    pub fg_brgt_wht => "97m";
    pub bg_brgt_blk => "100m";
    pub bg_brgt_red => "101m";
    pub bg_brgt_grn => "102m";
    pub bg_brgt_yel => "103m";
    pub bg_brgt_blu => "104m";
    pub bg_brgt_mag => "105m";
    pub bg_brgt_cyn => "106m";
    pub bg_brgt_wht => "107m";

    // color extended
    pub fg_256color => "38;5;{val}m", val:u8;
    pub bg_256color => "48;5;{val}m", val:u8;
    pub fg_rgb => "38;2;{r};{g};{b}m", r:u8, g:u8, b:u8;
    pub bg_rgb => "48;2;{r};{g};{b}m", r:u8, g:u8, b:u8;
}

etty_macros::gen_csi! {
    // mod scrn;
    pub scrn_mono_40x25_set => "=0h";
    pub scrn_mono_40x25_rst => "=0l";

    pub scrn_clr_40x25_set => "=1h";
    pub scrn_clr_40x25_rst => "=1l";

    pub scrn_mono_80x25_set => "=2h";
    pub scrn_mono_80x25_rst => "=2l";

    pub scrn_clr_80x25_set => "=3h";
    pub scrn_clr_80x25_rst => "=3l";

    pub scrn_4clr_320x200_set => "=4h";
    pub scrn_4clr_320x200_rst => "=4l";

    pub scrn_mono_320x200_set => "=5h";
    pub scrn_mono_320x200_rst => "=5l";

    pub scrn_mono_640x200_set => "=6h";
    pub scrn_mono_640x200_rst => "=6l";

    pub scrn_ln_wrap_set => "=7h";
    pub scrn_ln_wrap_rst => "=7L";

    pub scrn_clr_320x200_set => "=13h";
    pub scrn_clr_320x200_rst => "=13l";

    pub scrn_16clr_640x200_set => "=14h";
    pub scrn_16clr_640x200_rst => "=14l";

    pub scrn_mono_640x350_set => "=15h";
    pub scrn_mono_640x350_rst => "=15l";

    pub scrn_16clr_640x350_set => "=16h";
    pub scrn_16clr_640x350_rst => "=16l";

    pub scrn_mono_640x480_set => "=17h";
    pub scrn_mono_640x480_rst => "=17l";

    pub scrn_16clr_640x480_set => "=18h";
    pub scrn_16clr_640x480_rst => "=18l";

    pub scrn_256clr_320x200_set => "=19h";
    pub scrn_256clr_320x200_rst => "=19l";
}

etty_macros::gen_csi! {
    // mod evt;
    pub evt_mouse_set => "?1000h";
    pub evt_mouse_ext_set => "?1006h";
    pub evt_mouse_drag_set => "?1002h";
    pub evt_mouse_motion_set => "?1003h";
    pub evt_win_focus_set => "?1004h";

    pub evt_mouse_rst => "?1000l";
    pub evt_mouse_ext_rst => "?1006l";
    pub evt_mouse_drag_rst => "?1002l";
    pub evt_mouse_motion_rst => "?1003l";
    pub evt_win_focus_rst => "?1004l";
}
