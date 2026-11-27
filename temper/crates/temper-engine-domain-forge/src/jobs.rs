//! Merge and close job specs ask the worker to find the hook in its checkout
//! (temper/docs/design/protocol/hooks.md, 4.2; temper/docs/design/protocol/wires.md, 8).
use alloc::boxed::Box;
use skein_forge::api::Commit;
use skein_lib::{List, bytes};
use temper_protocol_worker_wire::v1::{self, Record};

pub const CLOSE_HOOK: &[u8] = b".temper/hooks/close";
const MERGE_HOOK: &[u8] = b".temper/hooks/merge";

fn find_hook(
    repository: &[u8],
    hook: &[u8],
    fallback: Option<&[u8]>,
    request: Box<[u8]>,
    refusal: bool,
) -> v1::JobSpec {
    v1::JobSpec {
        command: bytes::concat(&[repository, b"/", hook]),
        request,
        refusal,
        outdated: refusal,
        find: true,
        fallback: match fallback {
            Some(path) => bytes::concat(&[repository, b"/", path]),
            None => Box::new([]),
        },
    }
}

fn merge_spec(repository: &[u8], fallback: Option<&[u8]>, head: &Commit, base: &Commit) -> v1::JobSpec {
    let mut request = List::with_capacity(164);
    crate::routing::append(&mut request, b"{\"version\":1,\"head\":\"");
    crate::routing::append(&mut request, &crate::commit_text(head));
    crate::routing::append(&mut request, b"\",\"base\":\"");
    crate::routing::append(&mut request, &crate::commit_text(base));
    crate::routing::append(&mut request, b"\"}");
    find_hook(repository, MERGE_HOOK, fallback, request.into_boxed(), false)
}

fn fits(spec: &v1::JobSpec, bound: u32) -> bool {
    match spec.measure(&v1::CEILINGS) {
        Ok(size) => size <= bound,
        Err(_) => false,
    }
}

pub(crate) fn merge_fits(repository: &[u8], fallback: Option<&[u8]>, head: &Commit, base: &Commit, bound: u32) -> bool {
    fits(&merge_spec(repository, fallback, head, base), bound)
}

#[must_use]
pub fn merge(
    repository: &[u8],
    fallback: Option<&[u8]>,
    head: &Commit,
    base: &Commit,
    bound: u32,
) -> Option<Box<[u8]>> {
    encode(&merge_spec(repository, fallback, head, base), bound)
}

#[must_use]
pub fn close(repository: &[u8], fallback: Option<&[u8]>, request: &[u8], bound: u32) -> Option<Box<[u8]>> {
    encode(&find_hook(repository, CLOSE_HOOK, fallback, bytes::copy_of(request), true), bound)
}

fn encode(spec: &v1::JobSpec, bound: u32) -> Option<Box<[u8]>> {
    if !fits(spec, bound) {
        return None;
    }
    spec.encode(&v1::CEILINGS).ok()
}
