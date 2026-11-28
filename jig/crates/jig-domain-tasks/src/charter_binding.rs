//! The core resolves project charter names; the child admits and records the result
//! (jig/docs/design/domain/engine/project-charters.md, 4).
use crate::Executor;
use alloc::boxed::Box;

/// A resolved binding alongside a batch member, not part of the creation it requested.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct CharterBinding {
    pub charter: u32,
    pub scope: Box<[u8]>,
}

/// Read the executor that will be recorded without changing the creation's identity.
#[must_use]
pub fn bound_executor(executor: Executor, bindings: &[Option<CharterBinding>], at: usize) -> Executor {
    match bindings.get(at) {
        Some(Some(binding)) => Executor::Agent { charter: binding.charter },
        Some(None) | None => executor,
    }
}
