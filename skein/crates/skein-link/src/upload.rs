//! A whole body within each room grant (skein/docs/design/net/link.md, 3.4).
use alloc::boxed::Box;
use core::fmt;
use skein_lib::stream::{Down, Read};
use skein_lib::{bytes, usize_of};

pub struct Upload {
    body: Box<[u8]>,
    at: usize,
}
impl fmt::Debug for Upload {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Upload").field("bytes", &self.body.len()).field("at", &self.at).finish()
    }
}
impl Upload {
    #[must_use]
    pub fn new(body: Box<[u8]>) -> Upload {
        Upload { body, at: 0 }
    }
    /// A request's optional body.
    #[must_use]
    #[expect(clippy::manual_map, reason = "Option::map is outside the step subset")]
    pub fn of(body: Option<Box<[u8]>>) -> Option<Upload> {
        match body {
            Some(body) => Some(Upload::new(body)),
            None => None,
        }
    }
    /// The next piece within `room`, then Finish or a demand for more room.
    pub fn room(&mut self, room: u32) -> (Down, Down) {
        let end = self.at.saturating_add(usize_of(room)).min(self.body.len());
        let piece = bytes::copy_of(self.body.get(self.at..end).expect("within the body"));
        self.at = end;
        let next = if end == self.body.len() { Down::Finish } else { Down::Demand { read: Read::Nothing, room } };
        (Down::Send(piece), next)
    }
}
