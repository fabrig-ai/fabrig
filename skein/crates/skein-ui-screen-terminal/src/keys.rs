//! Bounded byte decoding (skein/docs/design/ui/screen.md, 6.1).
use alloc::boxed::Box;
use skein_lib::{List, Queue, bytes};
use skein_ui_screen::Rgb;

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Key {
    Char(char),
    Paste(Box<[u8]>),
    Enter,
    Tab,
    BackTab,
    Backspace,
    Delete,
    Up,
    Down,
    Left,
    Right,
    Home,
    End,
    PageUp,
    PageDown,
    Escape,
    Control(u8),
    Alt(char),
}

#[derive(Clone, Copy, Debug)]
enum State {
    Text,
    Escape,
    Sequence,
    Character { alt: bool },
    Paste,
    Answer { escape: bool },
}

#[derive(Debug)]
#[expect(clippy::option_option, reason = "the decoder contract distinguishes no answer, unreadable and RGB")]
pub struct Decoder {
    state: State,
    partial: List<u8>,
    overlong: bool,
    paste: List<u8>,
    ending: usize,
    answers: bool,
    waiting: bool,
    background: Option<Option<Rgb>>,
}
const PASTE_END: &[u8] = b"\x1b[201~";
const SEQUENCE: u32 = 16;
const ANSWER: u32 = 64;

impl Decoder {
    /// From now on OSC answers, including late ones, are never keys.
    pub fn answers(&mut self) {
        self.answers = true;
        self.waiting = true;
    }

    /// Ends the background wait, giving any held ESC as Escape (skein/docs/design/ui/screen.md, 6.1).
    pub fn end_background_wait(&mut self, keys: &mut Queue<Key>) {
        self.waiting = false;
        match self.state {
            State::Escape => {
                keys.push(Key::Escape);
                self.partial.clear();
                self.state = State::Text;
            }
            State::Text | State::Sequence | State::Character { .. } | State::Paste | State::Answer { .. } => {}
        }
    }

    /// The first OSC 11 answer, readable or not.
    #[must_use]
    pub const fn background(&self) -> Option<Option<Rgb>> {
        self.background
    }

    #[must_use]
    pub fn new(paste: u32) -> Decoder {
        Decoder {
            state: State::Text,
            partial: List::with_capacity(ANSWER),
            overlong: false,
            paste: List::with_capacity(paste),
            ending: 0,
            answers: false,
            waiting: false,
            background: None,
        }
    }

    /// Outside the background wait, a lone ESC is delivered at this read's end.
    pub fn read(&mut self, bytes: &[u8], keys: &mut Queue<Key>) {
        for byte in bytes {
            match self.state {
                State::Text => self.text(*byte, false, keys),
                State::Escape => {
                    if *byte == b'[' || *byte == b'O' {
                        self.partial.push(*byte).expect("sequence introducer fits");
                        self.state = State::Sequence;
                    } else if *byte == b']' && self.answers {
                        self.partial.push(*byte).expect("answer introducer fits");
                        self.state = State::Answer { escape: false };
                    } else {
                        self.partial.clear();
                        self.text(*byte, true, keys);
                    }
                }
                State::Sequence => self.sequence(*byte, keys),
                State::Character { alt } => self.character(*byte, alt, keys),
                State::Paste => self.pasted(*byte, keys),
                State::Answer { escape } => self.answer(*byte, escape),
            }
        }
        match self.state {
            State::Escape => {
                if !self.waiting {
                    keys.push(Key::Escape);
                    self.partial.clear();
                    self.state = State::Text;
                }
            }
            State::Text | State::Sequence | State::Character { .. } | State::Paste | State::Answer { .. } => {}
        }
    }

    fn text(&mut self, byte: u8, alt: bool, keys: &mut Queue<Key>) {
        self.state = State::Text;
        if alt {
            if byte >= b' ' && byte != 0x7f {
                self.character(byte, true, keys);
            }
            return;
        }
        let key = match byte {
            b'\r' | b'\n' => Key::Enter,
            b'\t' => Key::Tab,
            0x08 | 0x7f => Key::Backspace,
            0x01..=0x1a => Key::Control(b'a'.saturating_add(byte.saturating_sub(1))),
            0x1b => {
                self.partial.push(byte).expect("escape fits");
                self.state = State::Escape;
                return;
            }
            0x00 | 0x1c..=0x1f => return,
            0x20..=0x7e | 0x80..=0xff => {
                self.character(byte, false, keys);
                return;
            }
        };
        keys.push(key);
    }

    #[expect(clippy::disallowed_methods, reason = "the terminal decoder validates incoming UTF-8")]
    fn character(&mut self, byte: u8, alt: bool, keys: &mut Queue<Key>) {
        if self.partial.push(byte).is_err() {
            self.partial.clear();
            self.state = State::Text;
            return;
        }
        match core::str::from_utf8(self.partial.as_slice()) {
            Ok(text) => {
                if let Some(ch) = text.chars().next() {
                    keys.push(if alt { Key::Alt(ch) } else { Key::Char(ch) });
                }
                self.partial.clear();
                self.state = State::Text;
            }
            Err(error) => {
                if error.error_len().is_some() || self.partial.len() >= 4 {
                    self.partial.clear();
                    self.state = State::Text;
                } else {
                    self.state = State::Character { alt };
                }
            }
        }
    }

    fn sequence(&mut self, byte: u8, keys: &mut Queue<Key>) {
        if self.partial.len() == SEQUENCE {
            self.overlong = true;
        } else {
            self.partial.push(byte).expect("sequence has room");
        }
        if !(0x40..=0x7e).contains(&byte) {
            return;
        }
        if !self.overlong {
            let key = match self.partial.as_slice() {
                b"\x1b[A" | b"\x1bOA" => Some(Key::Up),
                b"\x1b[B" | b"\x1bOB" => Some(Key::Down),
                b"\x1b[C" | b"\x1bOC" => Some(Key::Right),
                b"\x1b[D" | b"\x1bOD" => Some(Key::Left),
                b"\x1b[H" | b"\x1b[1~" | b"\x1bOH" => Some(Key::Home),
                b"\x1b[F" | b"\x1b[4~" | b"\x1bOF" => Some(Key::End),
                b"\x1b[Z" => Some(Key::BackTab),
                b"\x1b[3~" => Some(Key::Delete),
                b"\x1b[5~" => Some(Key::PageUp),
                b"\x1b[6~" => Some(Key::PageDown),
                b"\x1b[200~" => {
                    self.state = State::Paste;
                    self.partial.clear();
                    return;
                }
                _ => None,
            };
            if let Some(key) = key {
                keys.push(key);
            }
        }
        self.partial.clear();
        self.overlong = false;
        self.state = State::Text;
    }

    fn answer(&mut self, byte: u8, escape: bool) {
        if self.partial.push(byte).is_err() {
            self.overlong = true;
        }
        if byte != 0x07 && !(escape && byte == b'\\') {
            self.state = State::Answer { escape: byte == 0x1b };
            return;
        }
        if !self.overlong && self.background.is_none() {
            let bytes = self.partial.as_slice();
            let payload = if byte == 0x07 { bytes.strip_suffix(b"\x07") } else { bytes.strip_suffix(b"\x1b\\") };
            if let Some(payload) = payload {
                self.background = background(payload);
            }
        }
        self.partial.clear();
        self.overlong = false;
        self.state = State::Text;
    }

    #[expect(clippy::disallowed_methods, reason = "a truncated terminal paste ends at a UTF-8 boundary")]
    fn pasted(&mut self, byte: u8, keys: &mut Queue<Key>) {
        if PASTE_END.get(self.ending) == Some(&byte) {
            self.ending = self.ending.saturating_add(1);
            if self.ending == PASTE_END.len() {
                if let Err(error) = core::str::from_utf8(self.paste.as_slice())
                    && error.error_len().is_none()
                {
                    self.paste.truncate(u32::try_from(error.valid_up_to()).expect("paste length fits u32"));
                }
                keys.push(Key::Paste(bytes::copy_of(self.paste.as_slice())));
                self.paste.clear();
                self.ending = 0;
                self.state = State::Text;
            }
            return;
        }
        for prefix in PASTE_END.get(..self.ending).expect("partial paste ending fits") {
            self.keep(*prefix);
        }
        self.ending = 0;
        if byte == 0x1b {
            self.ending = 1;
        } else {
            self.keep(byte);
        }
    }

    fn keep(&mut self, byte: u8) {
        if self.paste.room() > 0 {
            self.paste.push(byte).expect("paste has room");
        }
    }
}

#[expect(clippy::option_option, reason = "the decoder contract distinguishes no answer, unreadable and RGB")]
fn background(answer: &[u8]) -> Option<Option<Rgb>> {
    let tail = answer.strip_prefix(b"\x1b]11")?;
    if tail.is_empty() {
        return Some(None);
    }
    let payload = tail.strip_prefix(b";")?;
    Some(rgb(payload))
}

fn rgb(payload: &[u8]) -> Option<Rgb> {
    let payload = payload.strip_prefix(b"rgb:")?;
    let end = bytes::find(payload, b"/")?;
    let red = component(payload.get(..end)?)?;
    let payload = payload.get(end.checked_add(1)?..)?;
    let end = bytes::find(payload, b"/")?;
    let green = component(payload.get(..end)?)?;
    let blue = component(payload.get(end.checked_add(1)?..)?)?;
    Some(Rgb { red, green, blue })
}

fn component(hex: &[u8]) -> Option<u8> {
    if hex.is_empty() || hex.len() > 4 {
        return None;
    }
    let mut value = 0_u8;
    for (position, byte) in hex.iter().enumerate() {
        let digit = match byte {
            b'0'..=b'9' => byte.checked_sub(b'0')?,
            b'a'..=b'f' => byte.checked_sub(b'a')?.checked_add(10)?,
            b'A'..=b'F' => byte.checked_sub(b'A')?.checked_add(10)?,
            0..=b'/' | b':'..=b'@' | b'G'..=b'`' | b'g'..=u8::MAX => return None,
        };
        if position < 2 {
            value = value.checked_mul(16)?.checked_add(digit)?;
        }
    }
    if hex.len() == 1 { value.checked_mul(17) } else { Some(value) }
}
