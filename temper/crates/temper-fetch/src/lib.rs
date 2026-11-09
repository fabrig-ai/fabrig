//! Pinned binary fetches (temper/docs/design/protocol/setup.md, 6.6 and 11).
#![cfg_attr(not(test), no_std)]
#![forbid(unsafe_code)]
extern crate alloc;
use alloc::boxed::Box;
use skein_lib::Writer;
pub mod claude_code;
pub mod fetch;

/// An HTTPS authority, without a path.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Origin {
    pub host: Box<[u8]>,
    pub port: u16,
}

fn joined(parts: &[&[u8]]) -> Box<[u8]> {
    let mut len = 0_usize;
    for p in parts {
        len = len.checked_add(p.len()).expect("bounded text");
    }
    let mut w = Writer::new(len);
    for p in parts {
        w.put(p).expect("measured text");
    }
    w.finish()
}
