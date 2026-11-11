//! The engine's place (temper/docs/design/client/tui/README.md, 2 and temper/docs/design/client/tui/pages/README.md).
use alloc::boxed::Box;
use skein_lib::{Writer, bytes::copy_of};

/// A box's name and non-default port, or the address as given.
#[must_use]
pub fn place(address: &[u8], boxed: bool) -> Box<[u8]> {
    if !boxed {
        return copy_of(address);
    }
    let mut port = b"8367".as_slice();
    for (at, byte) in address.iter().enumerate() {
        if *byte == b':' {
            port = address.get(at.saturating_add(1)..).expect("port follows colon");
        }
    }
    if port == b"8367" {
        return copy_of(b"box");
    }
    let mut out = Writer::new(port.len().checked_add(4).expect("place fits memory"));
    out.put(b"box:").expect("place measured");
    out.put(port).expect("place measured");
    out.finish()
}
