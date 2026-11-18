//! A batch's answer cut as each frame completes (jig/docs/design/protocol/client/default-http.md, 6).
use crate::Limits;
use alloc::boxed::Box;
use jig_client_protocol_link::Heard;
use skein_lib::{List, Queue, Token};

#[derive(Debug)]
enum Cut {
    Length { word: u32, bytes: u8 },
    Record { length: u32, bytes: List<u8> },
    Ended,
}

/// Keeps only a length and the current record, never the whole answer.
#[derive(Debug)]
pub struct Frames {
    exchange: Token,
    bound: u32,
    cut: Cut,
}
impl Frames {
    #[must_use]
    pub fn new(exchange: Token, limits: &Limits) -> Self {
        Self { exchange, bound: limits.record, cut: Cut::Length { word: 0, bytes: 0 } }
    }
    #[must_use]
    pub fn active(&self) -> bool {
        match self.cut {
            Cut::Length { .. } | Cut::Record { .. } => true,
            Cut::Ended => false,
        }
    }
    pub fn opened(&mut self, status: u16, out: &mut Queue<Heard>) {
        if status != 200 {
            self.failed(out);
        }
    }
    pub fn body(&mut self, piece: &[u8], out: &mut Queue<Heard>) {
        for byte in piece {
            match self.byte(*byte) {
                Ok(Some(message)) => out.push(Heard::Part { exchange: self.exchange, message }),
                Ok(None) => {}
                Err(()) => {
                    self.failed(out);
                    return;
                }
            }
        }
    }
    fn byte(&mut self, byte: u8) -> Result<Option<Box<[u8]>>, ()> {
        match &mut self.cut {
            Cut::Length { word, bytes } => {
                *word = word.checked_mul(256).expect("at most four length bytes") | u32::from(byte);
                *bytes = bytes.checked_add(1).expect("at most four length bytes");
                if *bytes == 4 {
                    if *word > self.bound {
                        return Err(());
                    }
                    if *word == 0 {
                        self.cut = Cut::Length { word: 0, bytes: 0 };
                        return Ok(Some(Box::new([])));
                    }
                    self.cut = Cut::Record { length: *word, bytes: List::with_capacity(*word) };
                }
                Ok(None)
            }
            Cut::Record { length, bytes } => {
                bytes.push(byte).expect("declared frame length");
                if bytes.len() == *length {
                    let cut = core::mem::replace(&mut self.cut, Cut::Length { word: 0, bytes: 0 });
                    match cut {
                        Cut::Record { bytes, .. } => Ok(Some(bytes.into_boxed())),
                        Cut::Length { .. } | Cut::Ended => unreachable!("completed record"),
                    }
                } else {
                    Ok(None)
                }
            }
            Cut::Ended => Ok(None),
        }
    }
    pub fn ended(&mut self, out: &mut Queue<Heard>) {
        match self.cut {
            Cut::Length { bytes: 0, .. } => {
                self.cut = Cut::Ended;
                out.push(Heard::Done { exchange: self.exchange });
            }
            Cut::Length { .. } | Cut::Record { .. } => self.failed(out),
            Cut::Ended => {}
        }
    }
    pub fn failed(&mut self, out: &mut Queue<Heard>) {
        if self.active() {
            self.cut = Cut::Ended;
            out.push(Heard::Lost { exchange: self.exchange });
        }
    }
}
