//! Enum for [C0 control codes][wiki-c0].
//!
//! `C0` is produced by [`etty::evt::event_stream`][mod-evt]
//!
//! [wiki-c0]: https://en.wikipedia.org/wiki/C0_and_C1_control_codes
//! [mod-evt]: crate::evt::event_stream

// https://en.wikipedia.org/wiki/C0_and_C1_control_codes
#[derive(Debug, Eq, PartialEq, Clone, Copy, num_derive::FromPrimitive)]
pub enum C0 {
    Nul = 0,
    Soh = 1,
    Stx = 2,
    Etx = 3,
    Eot = 4,
    Enq = 5,
    Ack = 6,
    Bel = 7,
    Bs = 8,
    Ht = 9,
    Lf = 10,
    Vt = 11,
    Ff = 12,
    Cr = 13,
    So = 14,
    Si = 15,
    Dle = 16,
    Dc1 = 17,
    Dc2 = 18,
    Dc3 = 19,
    Dc4 = 20,
    Nak = 21,
    Syn = 22,
    Etb = 23,
    Can = 24,
    Em = 25,
    Sub = 26,
    Esc = 27,
    Fs = 28,
    Gs = 29,
    Rs = 30,
    Us = 31,
    Sp = 32,
    Del = 127,
}

impl C0 {
    pub fn as_idiom(&self) -> Idiom {
        let b = *self as u8;
        match b {
            b'\0' | b'\t' | b'\n' | b'\r' | b' ' => Idiom::Char(b as char),
            1..=8 | 11..=12 | 14..=26 | 28..=32 => Idiom::Ctrl((b + b'a' - 1) as char),
            // 8 => Idiom::Bs,
            27 => Idiom::Esc,
            127 => Idiom::Del,
            _ => unreachable!(),
        }
    }
}

impl From<u8> for C0 {
    fn from(i: u8) -> Self {
        <crate::C0 as num_traits::FromPrimitive>::from_u8(i).unwrap()
    }
}

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum Idiom {
    // Bs,
    Esc,
    Del,
    Char(char),
    Ctrl(char),
}
