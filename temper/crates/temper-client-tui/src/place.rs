//! The engine's place (temper/docs/design/client/tui/pages/chat.md, 1).
use crate::Origin;
use alloc::boxed::Box;
use skein_lib::{
    Decimal,
    bytes::{concat, copy_of},
};

/// A box's name and non-default port, or the engine's place unboxed.
#[must_use]
pub fn place(origin: &Origin, boxed: bool) -> Box<[u8]> {
    match origin {
        Origin::Http(socket) => {
            if !boxed {
                origin.authority()
            } else if socket.port() == 8367 {
                copy_of(b"box")
            } else {
                concat(&[b"box:", Decimal::of(u64::from(socket.port())).as_bytes()])
            }
        }
        Origin::Https { name, port: 443 } => name.clone(),
        Origin::Https { .. } => origin.authority(),
    }
}
