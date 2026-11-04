//! A whole body within each room grant (skein/docs/design/link.md, 3.4).
use alloc::boxed::Box;
use skein_lib::bytes;
use skein_lib::stream::{Down, Read};

#[derive(Debug)]
pub struct Upload {
    body: Box<[u8]>,
    at: usize,
}
impl Upload {
    #[must_use]
    pub fn new(body: Box<[u8]>) -> Upload {
        Upload { body, at: 0 }
    }
    /// The next piece within `room`, then Finish or a demand for more room.
    pub fn room(&mut self, room: u32) -> (Down, Down) {
        let end = self.at.saturating_add(usize::try_from(room).expect("room fits usize")).min(self.body.len());
        let piece = bytes::copy_of(self.body.get(self.at..end).expect("within the body"));
        self.at = end;
        let next = if end == self.body.len() { Down::Finish } else { Down::Demand { read: Read::Nothing, room } };
        (Down::Send(piece), next)
    }
}
