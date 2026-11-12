//! The owner's per-record inputs to written patch functions (data/codec/patches.md, 5): the command wrote it once; it is the owner's.

use alloc::boxed::Box;

#[derive(Debug)]
pub struct Context {
    /// The deployment's admitted delegation grants, supplied before opening the store.
    pub executors: Box<[jig_domain_authority::Executor]>,
}

// Default by contract (data/codec/patches.md, 5), and empty: no live state. Written
// out, since production clippy.toml refuses the derive.
impl Default for Context {
    fn default() -> Self {
        Self { executors: Box::new([]) }
    }
}
