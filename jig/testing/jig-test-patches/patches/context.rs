//! The shell sets the record key before each walk (skein/docs/design/data/codec/patches.md 5).
use alloc::boxed::Box;

#[derive(Default, Debug)]
pub struct Context {
    pub key: Box<[u8]>,
}
