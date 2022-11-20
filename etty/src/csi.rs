use std::io::Read;
use std::io::Write;

use crate::input;

pub fn read_cusr_pos() -> (u16, u16) {
    let (mut stdin, jh) = input::async_stdin()
        .timeout(std::time::Duration::from_millis(100))
        .until(|b| b.map(|&b| b == b'R').unwrap_or(false))
        .init();

    cusr_pos();
    std::io::stdout().flush().unwrap();

    let mut buf = Vec::with_capacity(8); // at least 6 bytes are needed, but make it 8 up front
    stdin.read_to_end(&mut buf).unwrap();

    jh.join().unwrap();

    assert_eq!(buf[0], b'\x1b');
    assert_eq!(buf[1], b'[');
    assert_eq!(buf.last().unwrap(), &b'R');
    let s = String::from_utf8(buf).unwrap();
    let mut yx = {
        let trim = &s[2..s.len() - 1];
        trim.split(';')
    };
    let y = yx.next().unwrap().parse::<u16>().unwrap();
    let x = yx.next().unwrap().parse::<u16>().unwrap();
    (x, y)
}

// navigations
etty_macro::gen_csi! {
    cusr_pos => "6n";

    pub cusr_up => "{n}A", n;
    pub cusr_dn => "{n}B", n;
    pub cusr_rgt => "{n}C", n;
    pub cusr_lft => "{n}D", n;

    pub cusr_next_ln => "{n}E", n;
    pub cusr_prev_ln => "{n}F", n;
    pub cusr_goto_x => "{x}G", x;

    pub cusr_home => "H";
    pub cusr_goto => "{y};{x}H", x, y;

    pub cusr_save => " 7";
    pub cusr_load => " 8";

    pub cusr_show => "?25h";
    pub cusr_hide => "?25l";

    pub scrl_up => "{n}S", n;
    pub scrl_dn => "{n}T", n;
}

// clear
etty_macro::gen_csi! {
    pub erse_bfr_cusr => "0J";
    pub erse_aft_cusr => "1J";
    pub erse_all => "2J";
    pub erse_all_and_saved => "3J";
    pub erse_ln_bfr_cusr => "0K";
    pub erse_ln_aft_cusr => "1K";
    pub erse_ln => "2K";
}

// private modes
etty_macro::gen_csi! {
    pub scrn_save => "?47h";
    pub scrn_load => "?47l";
    pub alt_buf_set => "?1049h";
    pub alt_buf_rst => "?1049l";
}

pub const SGR_RST: u8 = 0;

etty_macro::gen_style_const! {
    1
    BOLD,
    DIM,
    ITALIC,
    UNDERLINE,
    BLINKING,
}

etty_macro::gen_style_const! {
    7
    INVERSE,
    HIDDEN,
    STRIKETHRU,
}

etty_macro::gen_color_const! {
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

etty_macro::gen_color_const! {
    39 =>
    CLR_DEFAULT,
}

etty_macro::gen_color_const! {
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

etty_macro::gen_csi! {
    pub sgr_rst => "0m";

    pub sty_bold_set => "1m";
    pub sty_bold_rst => "21m";

    pub sty_dim_set => "2m";
    pub sty_dim_rst => "22m";

    pub sty_italic_set => "3m";
    pub sty_italic_rst => "23m";

    pub sty_underline_set => "4m";
    pub sty_underline_rst => "24m";

    pub sty_blinking_set => "5m";
    pub sty_blinking_rst => "25m";

    pub sty_inverse_set => "7m";
    pub sty_inverse_rst => "27m";

    pub sty_hidden_set => "8m";
    pub sty_hidden_rst => "28m";

    pub sty_strikethru_set => "9m";
    pub sty_strikethru_rst => "29m";

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

etty_macro::gen_csi! {
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

// https://invisible-island.net/xterm/ctlseqs/ctlseqs.html#h2-Mouse-Tracking
etty_macro::gen_csi! {
    pub evt_mouse_set => "?1000h";
    pub evt_mouse_ext_set => "?1006h";
    pub evt_mouse_drag_set => "?1002h";
    pub evt_mouse_motion_set => "?1003h";
    pub evt_window_focus_set => "?1004h";

    pub evt_mouse_rst => "?1000l";
    pub evt_mouse_ext_rst => "?1006l";
    pub evt_mouse_drag_rst => "?1002l";
    pub evt_mouse_motion_rst => "?1003l";
    pub evt_window_focus_rst => "?1004l";
}
