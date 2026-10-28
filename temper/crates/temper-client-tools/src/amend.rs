//! Draft task controls through the client domain's amendment editor (client/tools.md, 5.5).
use crate::jig::{
    Limits,
    call::{Call, Value, number, value},
    runner::acts::{Failed, Prepared, Request, choice, words},
};
use alloc::boxed::Box;
use jig_client::contract as c;
use skein_lib::{List, Writer};

#[expect(clippy::manual_map, reason = "step code keeps the optional shown revision explicit")]
pub(crate) fn prepare(call: &Call, task: &c::Task, me: &c::Me, limits: &Limits) -> Prepared {
    let mut draft = temper_client_domain::amend::Draft::of(task);
    let shown = match number(call, 1) {
        Some(value) => Some(c::Revision { value }),
        None => None,
    };
    if let Some(shown) = shown
        && call.key.is_none()
        && draft.revision() != shown
    {
        return Err(amend_error(
            task,
            b" moved: its spec revision is ",
            Some((draft.revision().value, shown.value)),
            b"",
        ));
    }
    controls(&mut draft, call, task, me, limits)?;
    let drafted = if call.key.is_some() { Some(named_draft(call, task, me, limits)?) } else { draft.amendment() };
    let Some(drafted) = drafted else {
        return Err(amend_error(task, b"", None, b"amend changes nothing on task "));
    };
    if drafted.revision.is_some() && shown.is_none() {
        return Err(Failed::Fixed(b"task amend needs --rev REV"));
    }
    let revision = shown.or(drafted.revision);
    let change = Box::new(drafted.change);
    let mut admission = None;
    for offer in &task.acts {
        if offer.act == c::ActKind::Amend {
            admission = Some(c::admission::amendment(&offer.admission, &change, &task.authority));
        }
    }
    let reason = words(call, 12, limits)?;
    let (ask, to) = match admission {
        Some(c::Admission::Allowed) => {
            (c::Ask::Amend { project: task.project, task: task.chip.task, revision, change, reason }, None)
        }
        Some(c::Admission::Proposed { to }) => (
            c::Ask::Propose {
                project: task.project,
                action: c::ProposeAction::Amendment {
                    task: task.chip.task,
                    revision: revision.unwrap_or(draft.revision()),
                    change,
                },
                reason,
            },
            Some(to),
        ),
        Some(c::Admission::Bounded { .. }) => return Err(Failed::Fixed(b"amendment has no bounded draft")),
        Some(c::Admission::Refused(_)) | None if call.key.is_some() => {
            (c::Ask::Amend { project: task.project, task: task.chip.task, revision, change, reason }, None)
        }
        Some(c::Admission::Refused(_)) | None => return Err(Failed::Fixed(b"amendment not offered")),
    };
    let request = if call.key.is_some() {
        let alternate = alternate(&ask, revision, draft.revision());
        Request::Amend { ask, alternate: Some(alternate) }
    } else {
        Request::Ask(ask)
    };
    Ok((request, to))
}

// The first retained ask may have been direct or proposed. Only a key conflict
// (or a locally held name) permits trying the other representation, once, under
// the same engine key. Neither answer admits a new action under that key.
fn alternate(ask: &c::Ask, direct_revision: Option<c::Revision>, shown: c::Revision) -> c::Ask {
    if let c::Ask::Amend { project, task, revision, change, reason } = ask {
        return c::Ask::Propose {
            project: *project,
            action: c::ProposeAction::Amendment {
                task: *task,
                revision: revision.unwrap_or(shown),
                change: change.clone(),
            },
            reason: reason.clone(),
        };
    }
    if let c::Ask::Propose { project, action: c::ProposeAction::Amendment { task, change, .. }, reason } = ask {
        return c::Ask::Amend {
            project: *project,
            task: *task,
            revision: direct_revision,
            change: change.clone(),
            reason: reason.clone(),
        };
    }
    unreachable!("amendment request")
}

// A named replacement is written even when it already equals today's value.
// Comparing it away (or rejecting yesterday's revision locally) prevents the
// engine from recognizing its retained key after the amendment has taken effect.
fn named_draft(
    call: &Call,
    task: &c::Task,
    me: &c::Me,
    limits: &Limits,
) -> Result<temper_client_domain::amend::Drafted, Failed> {
    let mut change = c::Amendment {
        addressed: None,
        spec: None,
        wake: None,
        dependencies: None,
        authority: None,
        instructions: None,
        tracked: None,
        llm: None,
    };
    if let Some(priority) = number(call, 2) {
        change.tracked = Some(c::Tracking::Tracked { priority: u32::try_from(priority).expect("priority fits") });
    } else if value(call, 3).is_some() {
        change.tracked = Some(c::Tracking::Untracked);
    }
    if value(call, 4).is_some() || value(call, 5).is_some() {
        let Some(mut authority) = task.authority.whole() else {
            return Err(Failed::Fixed(b"task authority is cut"));
        };
        authority.deadline = match value(call, 4) {
            Some(Value::Text(text)) => Some(crate::jig::argv::deadline(text).expect("checked deadline")),
            _ => None,
        };
        change.authority = Some(authority);
    }
    if value(call, 6).is_some() {
        change.instructions = Some(words(call, 6, limits)?);
    }
    if value(call, 7).is_some() {
        let mut spec = task.spec.clone();
        spec.specification.words = c::DocumentText::complete(words(call, 7, limits)?);
        change.spec = Some(spec.whole().ok_or(Failed::Fixed(b"task spec is cut"))?);
    }
    if let Some(Some(crate::jig::call::Values::Many(values))) = call.values.get(8) {
        let mut dependencies = List::with_capacity(u32::try_from(values.len()).expect("dependencies fit"));
        for value in values {
            let Value::Number(number) = value else { return Err(Failed::Fixed(b"dependency not loaded")) };
            dependencies.push(*number).expect("same count");
        }
        change.dependencies = Some(dependencies);
    } else if value(call, 9).is_some() {
        change.dependencies = Some(List::with_capacity(0));
    }
    if value(call, 10).is_some() {
        change.llm = Some(choice::llm(call, 10, 11, me, task.project, task.llm.as_ref())?.expect("model supplied"));
    }
    let other = change.spec.is_some()
        || change.authority.is_some()
        || change.instructions.is_some()
        || change.dependencies.is_some()
        || change.llm.is_some();
    let priority_alone = !other
        && task.priority.is_some()
        && match change.tracked {
            Some(c::Tracking::Tracked { .. }) => true,
            Some(c::Tracking::Untracked) | None => false,
        };
    let revision = if priority_alone { None } else { Some(task.spec_revision) };
    Ok(temper_client_domain::amend::Drafted { revision, change })
}

fn amend_error(task: &c::Task, suffix: &[u8], moved: Option<(u64, u64)>, prefix: &[u8]) -> Failed {
    let prefix = if prefix.is_empty() { b"task ".as_slice() } else { prefix };
    let task_number = skein_lib::Decimal::of(task.chip.task);
    let mut length = prefix
        .len()
        .checked_add(task_number.as_bytes().len())
        .expect("error length")
        .checked_add(suffix.len())
        .expect("error length");
    if let Some((now, shown)) = moved {
        length = length
            .checked_add(skein_lib::Decimal::of(now).as_bytes().len())
            .expect("revision length")
            .checked_add(b", not ".len())
            .expect("separator length")
            .checked_add(skein_lib::Decimal::of(shown).as_bytes().len())
            .expect("shown revision length")
            .checked_add(b"; read again: task show ".len())
            .expect("retry length")
            .checked_add(task_number.as_bytes().len())
            .expect("task length");
    }
    let mut out = Writer::new(length);
    out.put(prefix).expect("error prefix");
    out.put(skein_lib::Decimal::of(task.chip.task).as_bytes()).expect("task number");
    out.put(suffix).expect("error suffix");
    if let Some((now, shown)) = moved {
        out.put(skein_lib::Decimal::of(now).as_bytes()).expect("revision");
        out.put(b", not ").expect("separator");
        out.put(skein_lib::Decimal::of(shown).as_bytes()).expect("shown revision");
        out.put(b"; read again: task show ").expect("retry");
        out.put(skein_lib::Decimal::of(task.chip.task).as_bytes()).expect("task number");
    }
    Failed::Detail(out.finish())
}

fn controls(
    draft: &mut temper_client_domain::amend::Draft,
    call: &Call,
    task: &c::Task,
    me: &c::Me,
    limits: &Limits,
) -> Result<(), Failed> {
    if let Some(priority) = number(call, 2) {
        draft.track(c::Tracking::Tracked { priority: u32::try_from(priority).expect("checked priority") });
    }
    if value(call, 3).is_some() {
        draft.track(c::Tracking::Untracked);
    }
    if let Some(Value::Text(text)) = value(call, 4) {
        whole(draft.deadline(Some(crate::jig::argv::deadline(text).expect("checked deadline"))), task)?;
    } else if value(call, 5).is_some() {
        whole(draft.deadline(None), task)?;
    }
    if value(call, 6).is_some() {
        draft.instructions(words(call, 6, limits)?);
    }
    if value(call, 7).is_some() {
        whole(draft.words(words(call, 7, limits)?), task)?;
    }
    if let Some(Some(crate::jig::call::Values::Many(values))) = call.values.get(8) {
        let mut dependencies = List::with_capacity(u32::try_from(values.len()).expect("parsed dependencies"));
        for value in values {
            let Value::Number(number) = value else { return Err(Failed::Fixed(b"dependency not loaded")) };
            dependencies.push(*number).expect("same dependency count");
        }
        draft.dependencies(dependencies);
    } else if value(call, 9).is_some() {
        draft.dependencies(List::with_capacity(0));
    }
    if value(call, 10).is_some() {
        draft.llm(choice::llm(call, 10, 11, me, task.project, task.llm.as_ref())?.expect("model supplied"));
    }
    Ok(())
}
fn whole(result: Result<(), temper_client_domain::amend::Cut>, task: &c::Task) -> Result<(), Failed> {
    use temper_client_domain::amend::Cut;
    let suffix = match result {
        Ok(()) => return Ok(()),
        Err(Cut::Authority) => {
            b"'s authority is cut in this read; amend its deadline from a client with wider bounds".as_slice()
        }
        Err(Cut::Spec) => b"'s spec is cut in this read; amend its words from a client with wider bounds".as_slice(),
    };
    Err(amend_error(task, suffix, None, b""))
}
