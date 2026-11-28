//! A closer's catch-up belongs to the close request that found the goal outdated
//! (temper/docs/design/domain/forge/branches-closer.md, 1).
use super::super::forge_change::{CloseAt, CloseOutdated};
use super::super::{ChangeBuildRefused, ChangeOrigin, chat_change};
use super::{Box, Domain, List, c, forge, tasks};
use super::{Env, Limits};
use forge::closer_words;

pub(super) fn last_close<'a>(domain: &'a Domain, closer: &forge::CloserRow) -> Option<&'a forge::ChangeRow> {
    domain.forge.change(*closer.close_changes.last()?)
}

pub(super) fn finding(row: &forge::ChangeRow) -> Option<(CloseAt, CloseOutdated)> {
    let at = row.close_at?;
    Some((at, at.outdated?))
}

pub(super) fn observed(domain: &Domain, closer: &forge::CloserRow, task: &tasks::TaskRecord) -> Option<Box<[u64]>> {
    if lack(domain.config.merge_kind.is_some(), &task.authority.delegation.kinds).is_some() {
        return None;
    }
    Some(request_tasks(last_close(domain, closer)?))
}

pub(super) fn lack(declared: bool, kinds: &[tasks::AuthorityExecutor]) -> Option<&'static [u8]> {
    if !declared {
        return Some(b"the engine declares no merge job kind");
    }
    if kinds.contains(&tasks::AuthorityExecutor::Job(forge::MERGE_KIND)) {
        None
    } else {
        Some(b"its authority does not delegate to the merge job")
    }
}

pub(super) fn request_tasks(close: &forge::ChangeRow) -> Box<[u64]> {
    let request = close.close.as_deref().expect("a closer's close keeps its request");
    tasks_in(request).expect("a closer makes a version 1 close request")
}

// The request includes changes that did not land, unlike the close step's `closed` snapshot.
fn tasks_in(request: &[u8]) -> Option<Box<[u64]>> {
    let length = u32::try_from(request.len()).ok()?;
    let tokens = skein_json::tokenizer::Limits { depth: 3, string: length, number: 20, chunk: 256, length };
    let doc =
        skein_json::parse(request, &tokens, &skein_json::document::Limits { tokens: length, text: length }).ok()?;
    let changes = doc.member(0, b"changes")?;
    let end = doc.value_end(changes)?.checked_sub(1)?;
    let start = changes.checked_add(1)?;
    let mut item = start;
    let mut tasks = List::uncapped();
    for _ in start..end {
        if item == end {
            break;
        }
        let task = doc.member(item, b"task")?;
        let number = skein_lib::bytes::decimal(doc.text(doc.token(task)?)?)?;
        tasks.push(number).expect("close request bounds its changes");
        item = doc.value_end(item)?;
    }
    Some(tasks.into_boxed())
}

pub(super) fn reclose_request<'a>(domain: &'a Domain, closer: &forge::CloserRow, landed: &[u64]) -> Option<&'a [u8]> {
    if closer.closer.running?.step != c::Step::CatchUp {
        return None;
    }
    let caught_up = closer.closer.caught_up.as_deref()?;
    for task in landed {
        if !caught_up.contains(task) {
            return None;
        }
    }
    last_close(domain, closer)?.close.as_deref()
}

pub(super) fn delegate(
    domain: &Domain,
    env: &Env<Limits>,
    origin: &ChangeOrigin,
    repository: &forge::Repository,
    close: &forge::ChangeRow,
    goal: u64,
    spend: u64,
) -> Result<super::super::super::Delegate, ChangeBuildRefused> {
    let at = close.close_at.expect("an outdated close read its landing tip");
    let mut change = super::questions_up(closer_words::catch_up(goal, &at.landing));
    change.merge = Some(at.landing);
    chat_change(domain, env, origin, &change, repository, close.base.clone(), None, spend)
}

pub(super) fn hold_words(domain: &Domain, closer: &forge::CloserRow, step: c::Step, change: u64) -> Option<Box<[u8]>> {
    match step {
        c::Step::Land => None,
        c::Step::Close => {
            let close = domain.forge.change(change)?;
            if super::ending(close) != Some(forge::Ending::NoChange) {
                return None;
            }
            let (at, finding) = finding(close)?;
            let update = match closer.catch_up_changes.last() {
                Some(task) => domain.forge.change(*task),
                None => None,
            };
            if let Some(update) = update
                && succeeded(update)
                && closer.closer.caught_up.as_deref() == Some(request_tasks(close).as_ref())
            {
                let commit = update.catch_up.expect("the catch-up keeps its merge commit");
                return Some(closer_words::outdated(change, at, finding, Ok((update.task, commit))));
            }
            let task = domain.engine.core().tasks.task(closer.task).expect("held closer");
            let why = lack(domain.config.merge_kind.is_some(), &task.authority.delegation.kinds)
                .unwrap_or(b"no catch-up was admitted");
            Some(closer_words::outdated(change, at, finding, Err(why)))
        }
        c::Step::CatchUp => {
            let close = last_close(domain, closer).expect("a catch-up follows an outdated close");
            let (at, finding) = finding(close).expect("a catch-up follows an outdated finding");
            let update = domain.forge.change(change).expect("the closer retains its catch-up");
            Some(closer_words::catch_up_ended(change, super::ending(update), close.task, at, finding))
        }
    }
}

fn succeeded(change: &forge::ChangeRow) -> bool {
    match super::ending(change) {
        Some(forge::Ending::Landed | forge::Ending::NoChange) => true,
        Some(forge::Ending::Failed { .. } | forge::Ending::Cancelled) | None => false,
    }
}
