//! Upload's pieces (skein/docs/design/link.md, 3.4).
use super::Upload;
use alloc::boxed::Box;
use skein_lib::bytes;
use skein_lib::stream::{Down, Read};

#[test]
fn upload_gives_pieces_then_finish() {
    // Two-byte grants split the five-byte body into two full pieces and a tail.
    const ROOM: u32 = 2;
    let mut upload = Upload::new(bytes::copy_of(b"abcde"));
    for (piece, next) in [
        (b"ab".as_slice(), Down::Demand { read: Read::Nothing, room: ROOM }),
        (b"cd".as_slice(), Down::Demand { read: Read::Nothing, room: ROOM }),
        (b"e".as_slice(), Down::Finish),
    ] {
        assert_eq!(upload.room(ROOM), (Down::Send(bytes::copy_of(piece)), next));
    }
    assert_eq!(Upload::new(Box::new([])).room(ROOM), (Down::Send(Box::new([])), Down::Finish));
}
