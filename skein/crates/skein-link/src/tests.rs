//! Upload's pieces (skein/docs/design/net/link.md, 3.4).
use super::Upload;
use alloc::boxed::Box;
use skein_lib::bytes;
use skein_lib::stream::{Down, Read};

#[test]
#[expect(clippy::disallowed_macros, reason = "format the complete Debug text to pin omission of secret body bytes")]
fn an_upload_shows_its_length_and_no_byte() {
    let upload = Upload::new(bytes::copy_of(b"secret-token"));
    assert_eq!(format!("{upload:?}"), "Upload { bytes: 12, at: 0 }");
}

#[test]
fn upload_of_keeps_a_body_and_no_body() {
    // The five-byte body fits a single room grant.
    const ROOM: u32 = 5;
    let mut upload = Upload::of(Some(bytes::copy_of(b"abcde"))).expect("a body makes an upload");
    assert_eq!(upload.room(ROOM), (Down::Send(bytes::copy_of(b"abcde")), Down::Finish));
    assert!(Upload::of(None).is_none());
}

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
