//! Event handler.
//!
//! TODO
//!
//! [wiki-c0]: https://en.wikipedia.org/wiki/C0_and_C1_control_codes
//! [mod-evt]: etty_macros::evt

use std::io::Read;

#[derive(Debug)]
pub struct EventAndRaw {
    pub event: Event,
    pub raw: Vec<u8>,
}

impl EventAndRaw {
    fn new(event: Event, raw: Vec<u8>) -> Self {
        Self { event, raw }
    }
}

#[derive(Debug)]
pub enum Event {
    Key(Key),
    Window(Window),
    Mouse(Mouse),
    Undefined,
}

#[derive(Debug)]
pub enum Window {
    FocusIn,
    FocusOut,
}

#[derive(Debug)]
pub enum Mouse {
    Motion(u16, u16),

    RlseBtn(u16, u16),

    RlseLft(u16, u16),
    RlseMid(u16, u16),
    RlseRgt(u16, u16),

    PrssLft(u16, u16),
    PrssMid(u16, u16),
    PrssRgt(u16, u16),

    DragLft(u16, u16),
    DragMid(u16, u16),
    DragRgt(u16, u16),

    PrssLftAlt(u16, u16),
    PrssMidAlt(u16, u16),
    PrssRgtAlt(u16, u16),

    RlseLftAlt(u16, u16),
    RlseMidAlt(u16, u16),
    RlseRgtAlt(u16, u16),

    DragLftAlt(u16, u16),
    DragMidAlt(u16, u16),
    DragRgtAlt(u16, u16),

    PrssLftCtrl(u16, u16),
    PrssMidCtrl(u16, u16),
    PrssRgtCtrl(u16, u16),

    RlseLftCtrl(u16, u16),
    RlseMidCtrl(u16, u16),
    RlseRgtCtrl(u16, u16),

    DragLftCtrl(u16, u16),
    DragMidCtrl(u16, u16),
    DragRgtCtrl(u16, u16),

    WheelUp(u16, u16),
    WheelDn(u16, u16),

    WheelUpShift(u16, u16),
    WheelDnShift(u16, u16),

    WheelUpAlt(u16, u16),
    WheelDnAlt(u16, u16),

    WheelUpCtrl(u16, u16),
    WheelDnCtrl(u16, u16),

    UNIMPLEMENTED(u16, u16, u16),
}

#[derive(Debug, Clone, Copy)]
pub enum Nav {
    Up,
    Dn,
    Rgt,
    Lft,

    Ins,
    Del,
    Home,
    End,
    PgUp,
    PgDn,

    BTab,
}

#[derive(Debug, Clone, Copy)]
pub enum Key {
    C0(crate::C0),
    Nav(Nav),
    F(u8),
    Alt(char),
    Utf8(char),
}

pub fn event_stream() -> EventAndRawStream {
    event_and_raw_stream(std::io::stdin())
}

pub struct EventOnlyStream {
    stream: EventAndRawStream,
}

impl Iterator for EventOnlyStream {
    type Item = std::io::Result<Event>;
    fn next(&mut self) -> Option<Self::Item> {
        self.stream.next().map(|res| res.map(|e| e.event))
    }
}

fn event_and_raw_stream<R>(reader: R) -> EventAndRawStream
where
    R: std::io::Read + Send + 'static,
{
    let (tx, rx) = crossbeam::channel::unbounded::<std::io::Result<EventAndRaw>>();
    let _ = std::thread::spawn(move || {
        let mut parse = parser(reader);
        while let Some(res) = parse() {
            tx.send(res).unwrap()
        }
    });
    EventAndRawStream { rx }
}

pub struct EventAndRawStream {
    rx: crossbeam::channel::Receiver<std::io::Result<EventAndRaw>>,
}

impl EventAndRawStream {
    pub fn event_only(self) -> EventOnlyStream {
        EventOnlyStream { stream: self }
    }
}

impl Iterator for EventAndRawStream {
    type Item = std::io::Result<EventAndRaw>;
    fn next(&mut self) -> Option<Self::Item> {
        self.rx.recv().ok()
    }
}

fn parser<R>(mut reader: R) -> impl FnMut() -> Option<std::io::Result<EventAndRaw>>
where
    R: std::io::Read,
{
    let mut frag: Option<u8> = None;
    let mut buf = [0_u8; 2];
    move || -> Option<std::io::Result<EventAndRaw>> {
        if let Some(frag) = frag.take() {
            let msg = parse(frag, &mut (&mut reader).bytes());
            return Some(msg);
        }
        let res = match reader.read(&mut buf) {
            Ok(0) => return None,
            Ok(1) => {
                if buf[0] == b'\x1b' {
                    let event = Event::Key(Key::C0(crate::C0::from(buf[0])));
                    Ok(EventAndRaw::new(event, vec![buf[0]]))
                } else {
                    parse(buf[0], &mut (&mut reader).bytes())
                }
            }
            Ok(2) => {
                let mut sec_b = [buf[1]].into_iter();
                let mut stream = (&mut sec_b).map(Ok).chain((&mut reader).bytes());
                let res = parse(buf[0], &mut stream);
                frag = sec_b.next();
                assert!(sec_b.next().is_none());
                res
            }
            Ok(_) => unreachable!(),
            Err(err) => Err(err),
        };
        Some(res)
    }
}

fn parse<I>(lead: u8, iter: &mut I) -> std::io::Result<EventAndRaw>
where
    I: Iterator<Item = std::io::Result<u8>>,
{
    match lead {
        b'\x1b' => parse_esc_seq(iter),
        0..=26 | 28..=32 | 127 => {
            let event = Event::Key(Key::C0(crate::C0::from(lead)));
            Ok(EventAndRaw::new(event, vec![lead]))
        }
        _ => parse_utf8(lead, iter),
    }
}

fn parse_esc_seq<I>(iter: &mut I) -> std::io::Result<EventAndRaw>
where
    I: Iterator<Item = std::io::Result<u8>>,
{
    let Some(Ok(next)) = iter.next() else {
        unreachable!("the next byte should be `Some` as it has read before yanking it back to the `iter`");
    };
    match next {
        b'[' => parse_csi(iter),
        b'O' => {
            let Some(b) = iter.next() else {
                return Ok(EventAndRaw::new(Event::Undefined, vec![b'\x1b', b'O']));
            };
            let event = match b? {
                b @ 80..=83 => {
                    EventAndRaw::new(Event::Key(Key::F(b - b'O')), vec![b'\x1b', b'O', b])
                } // F1, F2, F3, F4
                b => EventAndRaw::new(Event::Undefined, vec![b'\x1b', b'O', b]),
            };
            Ok(event)
        }

        _ => {
            let (parse, raw) = {
                // let EventAndRaw { event, raw } = parse(next, iter)?;

                let EventAndRaw { event, raw } = parse_utf8(next, iter)?;
                let Event::Key(Key::Utf8(c)) = event else {
                    unreachable!();
                };
                let raw = [b'\x1b']
                    .into_iter()
                    .chain(raw.into_iter())
                    .collect::<Vec<_>>();
                (Event::Key(Key::Alt(c)), raw)
            };
            let event = EventAndRaw::new(parse, raw);
            Ok(event)
        }
    }
}

fn parse_csi<I>(iter: &mut I) -> std::io::Result<EventAndRaw>
where
    I: Iterator<Item = std::io::Result<u8>>,
{
    let Some(b) = iter.next() else {
        return Ok(EventAndRaw::new(Event::Undefined, vec![b'\x1b', b'[']));
    };

    let event = match b {
        Ok(digit @ b'1'..=b'9') => {
            let mut buf = Vec::new();

            loop {
                let Some(res) = iter.next() else {
                    let raw = [b'\x1b', b'[', digit].into_iter().chain(buf.into_iter()).collect::<Vec<u8>>();
                    return Ok(EventAndRaw::new(Event::Undefined, raw));
                };
                let b = res?;
                if b == b'~' {
                    break;
                }
                buf.push(b);
            }

            'not: {
                if !buf.is_empty() {
                    break 'not;
                }
                let nav = match digit {
                    b'1' => Nav::Home,
                    b'2' => Nav::Ins,
                    b'3' => Nav::Del,
                    b'4' => Nav::End,
                    b'5' => Nav::PgUp,
                    b'6' => Nav::PgDn,
                    _ => break 'not,
                };
                return Ok(EventAndRaw::new(
                    Event::Key(Key::Nav(nav)),
                    vec![b'\x1b', b'[', digit],
                ));
            }

            'not: {
                if buf.len() != 1 {
                    break 'not;
                }
                let fnum = match buf[0] {
                    b @ b'0'..=b'1' => b - 39, // F9, F10
                    b @ b'4' => b - 40,        // F12
                    b @ b'5' => b - 48,        // F5
                    b @ b'7'..=b'9' => b - 49, // F6, F7, F8
                    _ => break 'not,
                };
                let event = EventAndRaw::new(
                    Event::Key(Key::F(fnum)),
                    vec![b'\x1b', b'[', digit, buf[0], b'~'],
                );
                return Ok(event);
            }

            let raw = [b'\x1b', b'[', digit]
                .into_iter()
                .chain(buf.into_iter())
                .collect::<Vec<u8>>();
            return Ok(EventAndRaw::new(Event::Undefined, raw));
        }

        Ok(b @ b'A') => EventAndRaw::new(Event::Key(Key::Nav(Nav::Up)), vec![b'\x1b', b'[', b]),
        Ok(b @ b'B') => EventAndRaw::new(Event::Key(Key::Nav(Nav::Dn)), vec![b'\x1b', b'[', b]),
        Ok(b @ b'C') => EventAndRaw::new(Event::Key(Key::Nav(Nav::Rgt)), vec![b'\x1b', b'[', b]),
        Ok(b @ b'D') => EventAndRaw::new(Event::Key(Key::Nav(Nav::Lft)), vec![b'\x1b', b'[', b]),
        Ok(b @ b'F') => EventAndRaw::new(Event::Key(Key::Nav(Nav::End)), vec![b'\x1b', b'[', b]),
        Ok(b @ b'H') => EventAndRaw::new(Event::Key(Key::Nav(Nav::Home)), vec![b'\x1b', b'[', b]),
        Ok(b @ b'Z') => EventAndRaw::new(Event::Key(Key::Nav(Nav::BTab)), vec![b'\x1b', b'[', b]),
        Ok(b @ b'I') => EventAndRaw::new(Event::Window(Window::FocusIn), vec![b'\x1b', b'[', b]),
        Ok(b @ b'O') => EventAndRaw::new(Event::Window(Window::FocusOut), vec![b'\x1b', b'[', b]),

        Ok(b'M') => parse_mouse_x10(iter),
        Ok(b'<') => parse_mouse_sgr(iter),
        Ok(b) => EventAndRaw::new(Event::Undefined, vec![b'\x1b', b'[', b]),
        Err(err) => return Err(err),
    };
    Ok(event)
}

fn parse_utf8<I>(lead: u8, iter: &mut I) -> std::io::Result<EventAndRaw>
where
    I: Iterator<Item = std::io::Result<u8>>,
{
    if lead.is_ascii() {
        let event = EventAndRaw::new(Event::Key(Key::Utf8(lead as char)), vec![lead]);
        return Ok(event);
    }
    let mut buf = {
        let mut v = Vec::with_capacity(4);
        v.push(lead);
        v
    };
    #[allow(clippy::never_loop)] // it does loop idk why clippy yell at it.
    for next_b in iter.by_ref() {
        buf.push(next_b?);
        let Ok(s) = std::str::from_utf8(buf.as_slice()) else {
            if buf.len() <= 4 {
                continue;
            } else {
                break;
            }
        };
        let parse = {
            let mut chars = s.chars();
            let c = chars.next().unwrap();
            assert!(chars.next().is_none());
            Event::Key(Key::Utf8(c))
        };
        buf.shrink_to_fit();
        let event = EventAndRaw::new(parse, buf);
        return Ok(event);
    }
    let event = EventAndRaw::new(Event::Undefined, buf);
    Ok(event)
}

fn parse_mouse_x10<I>(iter: &mut I) -> EventAndRaw
where
    I: Iterator<Item = std::io::Result<u8>>,
{
    let mut raw = {
        let mut v = Vec::with_capacity(6);
        v.push(b'\x1b');
        v.push(b'[');
        v.push(b'M');
        v
    };
    let mut next = || {
        let b = iter.next().unwrap().unwrap();
        raw.push(b);
        b.saturating_sub(32_u8) as u16
    };
    let (cb, cx, cy) = (next(), next(), next());
    let mouse = match cb {
        0 => Mouse::PrssLft(cx, cy),
        1 => Mouse::PrssMid(cx, cy),
        2 => Mouse::PrssRgt(cx, cy),

        3 => Mouse::RlseBtn(cx, cy),

        8 => Mouse::RlseLftAlt(cx, cy),
        9 => Mouse::RlseMidAlt(cx, cy),
        10 => Mouse::RlseRgtAlt(cx, cy),

        16 => Mouse::RlseLftCtrl(cx, cy),
        17 => Mouse::RlseMidCtrl(cx, cy),
        18 => Mouse::RlseRgtCtrl(cx, cy),

        32 => Mouse::DragLft(cx, cy),
        33 => Mouse::DragMid(cx, cy),
        34 => Mouse::DragRgt(cx, cy),

        40 => Mouse::DragLftAlt(cx, cy),
        41 => Mouse::DragMidAlt(cx, cy),
        42 => Mouse::DragRgtAlt(cx, cy),

        48 => Mouse::DragLftCtrl(cx, cy),
        49 => Mouse::DragMidCtrl(cx, cy),
        50 => Mouse::DragRgtCtrl(cx, cy),

        64 => Mouse::WheelDn(cx, cy),
        65 => Mouse::WheelUp(cx, cy),

        70 => Mouse::WheelDnShift(cx, cy),
        71 => Mouse::WheelUpShift(cx, cy),

        72 => Mouse::WheelDnAlt(cx, cy),
        73 => Mouse::WheelUpAlt(cx, cy),

        80 => Mouse::WheelDnCtrl(cx, cy),
        81 => Mouse::WheelUpCtrl(cx, cy),

        _ => Mouse::UNIMPLEMENTED(cb, cx, cy),
    };
    let parse = Event::Mouse(mouse);
    EventAndRaw::new(parse, raw)
}

/// https://invisible-island.net/xterm/ctlseqs/ctlseqs.html#h3-Extended-coordinates
fn parse_mouse_sgr<I>(iter: &mut I) -> EventAndRaw
where
    I: Iterator<Item = std::io::Result<u8>>,
{
    // static OFF_SET: u8 = 48;
    let mut raw = Vec::with_capacity(16);
    let (b_m, bytes) = {
        let mut m: Option<u8> = None;
        let bytes = iter
            .map_while(|b| {
                //
                match b.unwrap() {
                    b_m @ (b'm' | b'M') => {
                        m = Some(b_m);
                        None
                    }
                    b => Some(b),
                }
            })
            .collect::<Vec<u8>>();
        (m.unwrap(), bytes)
    };
    raw.push(b_m);

    let mut vals = bytes.rsplit(|b| *b == b';').map(|bytes| {
        let num = crate::bytes_to_uint::<u16>(&bytes).unwrap();
        // let mut xten: u16 = 1;
        // let num = (0..bytes.len()).rev().fold(0_u16, |mut acc, i| {
        //     let byte = bytes[i];
        //     raw.push(byte);
        //     acc += (byte - OFF_SET) as u16 * xten;
        //     xten *= 10;
        //     acc
        // });
        raw.push(b';');
        num
    });

    let (cy, cx, cb) = (
        vals.next().unwrap(),
        vals.next().unwrap(),
        vals.next().unwrap(),
    );
    assert!(vals.next().is_none());

    let parse = {
        let mouse = match (cb, b_m) {
            (35, b'M') => Mouse::Motion(cx, cy),

            (0, b'm') => Mouse::RlseLft(cx, cy),
            (1, b'm') => Mouse::RlseMid(cx, cy),
            (2, b'm') => Mouse::RlseRgt(cx, cy),

            (8, b'm') => Mouse::RlseLftAlt(cx, cy),
            (9, b'm') => Mouse::RlseMidAlt(cx, cy),
            (10, b'm') => Mouse::RlseRgtAlt(cx, cy),

            (16, b'm') => Mouse::RlseLftCtrl(cx, cy),
            (17, b'm') => Mouse::RlseMidCtrl(cx, cy),
            (18, b'm') => Mouse::RlseRgtCtrl(cx, cy),

            (0, b'M') => Mouse::PrssLft(cx, cy),
            (1, b'M') => Mouse::PrssMid(cx, cy),
            (2, b'M') => Mouse::PrssRgt(cx, cy),

            (8, b'M') => Mouse::PrssLftAlt(cx, cy),
            (9, b'M') => Mouse::PrssMidAlt(cx, cy),
            (10, b'M') => Mouse::PrssRgtAlt(cx, cy),

            (16, b'M') => Mouse::PrssLftCtrl(cx, cy),
            (17, b'M') => Mouse::PrssMidCtrl(cx, cy),
            (18, b'M') => Mouse::PrssRgtCtrl(cx, cy),

            (32, b'M') => Mouse::DragLft(cx, cy),
            (33, b'M') => Mouse::DragMid(cx, cy),
            (34, b'M') => Mouse::DragRgt(cx, cy),

            (40, b'M') => Mouse::DragLftAlt(cx, cy),
            (41, b'M') => Mouse::DragMidAlt(cx, cy),
            (42, b'M') => Mouse::DragLftAlt(cx, cy),

            (48, b'M') => Mouse::DragLftCtrl(cx, cy),
            (49, b'M') => Mouse::DragMidCtrl(cx, cy),
            (50, b'M') => Mouse::DragLftCtrl(cx, cy),

            (64, b'M') => Mouse::WheelDn(cx, cy),
            (65, b'M') => Mouse::WheelUp(cx, cy),

            (70, b'M') => Mouse::WheelDnShift(cx, cy),
            (71, b'M') => Mouse::WheelUpShift(cx, cy),

            (72, b'M') => Mouse::WheelDnAlt(cx, cy),
            (73, b'M') => Mouse::WheelUpAlt(cx, cy),

            (80, b'M') => Mouse::WheelDnCtrl(cx, cy),
            (81, b'M') => Mouse::WheelUpCtrl(cx, cy),

            _ => Mouse::UNIMPLEMENTED(cb, cx, cy),
        };
        Event::Mouse(mouse)
    };
    raw.push(b'<');
    raw.push(b'[');
    raw.push(b'\x1b');
    raw.shrink_to_fit();
    raw.reverse();
    EventAndRaw::new(parse, raw)
}
