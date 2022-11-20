use crate::unix;
use crate::unix::Termios;

pub fn raw_mode() -> TermMode {
    let mut tm = TermMode::new();
    tm.raw();
    tm
}

pub fn term_mode() -> TermMode {
    TermMode::new()
}

pub struct TermMode(Option<Termios>);

impl TermMode {
    fn new() -> Self {
        Self(None)
    }
    pub fn raw(&mut self) {
        self.0
            .as_ref()
            .cloned()
            .unwrap_or_else(|| {
                let termios = unix::get_term_attr().unwrap();
                self.0 = Some(termios.clone());
                termios
            })
            .into_raw()
            .set_attr()
            .unwrap();
    }
    pub fn revert(&mut self) {
        if let Some(termios) = self.0.as_mut() {
            termios.set_attr().unwrap();
        }
    }
}

impl Drop for TermMode {
    fn drop(&mut self) {
        self.revert();
    }
}

pub fn term_size() -> (u16, u16) {
    let size = unix::get_term_size().unwrap();
    (size.col as u16, size.row as u16)
}

pub fn term_size_px() -> (u16, u16) {
    let size = unix::get_term_size().unwrap();
    (size.x as u16, size.y as u16)
}
