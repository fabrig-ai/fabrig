//! The merge job and the resolution it leaves (temper/docs/design/domain/forge.md, 8.4;
//! temper/docs/design/domain/worker.md, 4.4).
use alloc::boxed::Box;
use jig_domain_tasks as tasks;
use skein_lib::{List, Writer};
use temper_protocol_worker_wire::v1;

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
    let spec =
        v1::JobSpec::new(&v1::CEILINGS, v1::JobSpecParts { command: path.finish(), request: request.into_boxed() })
            .ok()?;
    if spec.measure() > bound {
        return None;
    }
    let mut writer = Writer::new(usize::try_from(spec.measure()).ok()?);
    spec.encode(&mut writer).ok()?;
    Some(writer.finish())
}

pub(super) fn resolution_words(result: Option<&[u8]>, parameters: &[tasks::Parameter], bound: u32) -> Box<[u8]> {
    let mut room = bound;
    for parameter in parameters {
        if let tasks::Parameter::Bytes { value, .. } = parameter {
            room = room.saturating_sub(u32::try_from(value.len()).unwrap_or(u32::MAX));
        }
    }
    let mut words = List::with_capacity(room);
    super::append(&mut words, b"Resolve the merge conflict and push the merge commit");
    if let Some(result) = result {
        super::append(
            &mut words,
            b"\n\nThe merge job settled what the project's merge command knows and left the rest:\n",
        );
        let result = super::forge::text::fit(result, usize::try_from(words.room()).expect("u32 fits usize"), b"...");
        super::append(&mut words, &result);
    }
    words.into_boxed()
}
