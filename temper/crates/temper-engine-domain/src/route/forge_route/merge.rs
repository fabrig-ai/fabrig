//! The merge job and the resolution it leaves (temper/docs/design/domain/forge/change/repairs.md;
//! temper/docs/design/domain/worker/jobs.md).
use alloc::boxed::Box;
use skein_lib::{List, Writer};
use temper_protocol_worker_wire::v1::{self, Record};

pub(super) fn spec(
    repository: &[u8],
    command: &[u8],
    head: &[u8; 32],
    base: &[u8; 32],
    bound: u32,
) -> Option<Box<[u8]>> {
    let length = repository.len().checked_add(1)?.checked_add(command.len())?;
    let mut path = Writer::new(length);
    path.put(repository).ok()?;
    path.put(b"/").ok()?;
    path.put(command).ok()?;
    let mut request = List::with_capacity(164);
    super::append(&mut request, b"{\"version\":1,\"head\":\"");
    super::append(&mut request, &super::commit_text(head));
    super::append(&mut request, b"\",\"base\":\"");
    super::append(&mut request, &super::commit_text(base));
    super::append(&mut request, b"\"}");
    let spec = v1::JobSpec {
        command: path.finish(),
        request: request.into_boxed(),
        refusal: false,
        result: temper_protocol_worker_wire::DEFAULT_JOB_RESULT_BYTES,
        outdated: false,
    };
    if spec.measure(&v1::CEILINGS).ok()? > bound {
        return None;
    }
    spec.encode(&v1::CEILINGS).ok()
}
