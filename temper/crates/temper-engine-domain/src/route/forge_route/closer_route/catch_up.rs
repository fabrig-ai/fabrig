//! A closer's catch-up belongs to the close request that found the goal outdated
//! (temper/docs/design/domain/forge/branches.md, 2).
use super::super::forge_change::{CloseAt, CloseOutdated};
use super::super::{ChangeOrigin, chat_change};
use super::{Box, Domain, List, append, c, forge, tasks};
use super::{Env, Limits};
use skein_forge::api::Commit;

const UPDATE: &[u8] =
    b". Catch the goal up from main with a change naming main's tip and release the closer, or release it to ";

pub(super) fn last_close<'a>(domain: &'a Domain, closer: &forge::CloserRow) -> Option<&'a forge::ChangeRow> {
    domain.forge.change(*closer.close_changes.last()?)
}

pub(super) fn finding(row: &forge::ChangeRow) -> Option<(CloseAt, CloseOutdated)> {
    let at = row.close_at?;
    Some((at, at.outdated?))
}

pub(super) fn observed(domain: &Domain, closer: &forge::CloserRow, task: &tasks::TaskRecord) -> Option<Box<[u64]>> {
    if lack(domain.forge.merge_job(task.project).is_some(), &task.authority.delegation.kinds).is_some() {
        return None;
    }
    Some(request_tasks(last_close(domain, closer)?))
}

pub(super) fn lack(job: bool, kinds: &[tasks::AuthorityExecutor]) -> Option<&'static [u8]> {
    if !job {
        Some(b"the project names no merge_job")
    } else if !kinds.contains(&tasks::AuthorityExecutor::Job(forge::MERGE_KIND)) {
        Some(b"its authority does not delegate to the merge job")
    } else {
        None
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
) -> Result<super::super::super::Delegate, tasks::Refusal> {
    let at = close.close_at.expect("an outdated close read its landing tip");
    let mut change = super::questions_up(change_words(goal, &at.landing));
    change.merge = Some(at.landing);
    chat_change(domain, env, origin, &change, repository, close.base.clone(), None, spend)
}

fn change_words(goal: u64, commit: &Commit) -> Box<[u8]> {
    let mut words = List::uncapped();
    append(&mut words, b"Catch goal ");
    append(&mut words, &super::super::decimal(goal));
    append(&mut words, b" up from main at ");
    short_commit(&mut words, commit);
    append(&mut words, b".");
    words.into_boxed()
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
                return Some(outdated_words(change, at, finding, Ok((update.task, commit))));
            }
            let task = domain.engine.core().tasks.task(closer.task).expect("held closer");
            let why = lack(domain.forge.merge_job(task.project).is_some(), &task.authority.delegation.kinds)
                .expect("a first outdated close holds only where a catch-up can't be made");
            Some(outdated_words(change, at, finding, Err(why)))
        }
        c::Step::CatchUp => {
            let close = last_close(domain, closer).expect("a catch-up follows an outdated close");
            let (at, finding) = finding(close).expect("a catch-up follows an outdated finding");
            let update = domain.forge.change(change).expect("the closer retains its catch-up");
            let mut words = List::uncapped();
            append(&mut words, b"the catch-up from main at ");
            short_commit(&mut words, &at.landing);
            append(&mut words, b", change T");
            append(&mut words, &super::super::decimal(change));
            append(&mut words, b", ended ");
            super::append_ending(&mut words, super::ending(update));
            append(&mut words, b", after ");
            close_words(&mut words, close.task, &at.branch);
            append(&mut words, b" outdated: ");
            append(&mut words, finding_words(finding));
            append(&mut words, UPDATE);
            append(&mut words, b"catch up again.");
            Some(fit(&words))
        }
    }
}

fn succeeded(change: &forge::ChangeRow) -> bool {
    match super::ending(change) {
        Some(forge::Ending::Landed | forge::Ending::NoChange) => true,
        Some(forge::Ending::Failed { .. } | forge::Ending::Cancelled) | None => false,
    }
}

pub(super) fn outdated_words(
    close: u64,
    at: CloseAt,
    finding: CloseOutdated,
    update: Result<(u64, Commit), &[u8]>,
) -> Box<[u8]> {
    let mut words = List::uncapped();
    close_words(&mut words, close, &at.branch);
    match update {
        Ok((change, commit)) => {
            append(&mut words, b" outdated again, after its catch-up from main at ");
            short_commit(&mut words, &commit);
            append(&mut words, b", change T");
            append(&mut words, &super::super::decimal(change));
            append(&mut words, b", landed: ");
            append(&mut words, finding_words(finding));
            append(&mut words, UPDATE);
            append(&mut words, b"close again as it stands.");
        }
        Err(why) => {
            append(&mut words, b" outdated: ");
            append(&mut words, finding_words(finding));
            append(&mut words, b"; the closer can't catch it up from main: ");
            append(&mut words, why);
            append(&mut words, b". Merge main into the goal's branch and release the closer to close again.");
        }
    }
    fit(&words)
}

fn close_words(words: &mut List<u8>, close: u64, branch: &Commit) {
    append(words, b"the close step's change T");
    append(words, &super::super::decimal(close));
    append(words, b" found the goal's branch at ");
    short_commit(words, branch);
}

fn short_commit(words: &mut List<u8>, commit: &Commit) {
    for byte in commit.get(..6).expect("six bytes of a commit") {
        append(words, &forge::hex(*byte));
    }
}

fn finding_words(finding: CloseOutdated) -> &'static [u8] {
    match finding {
        CloseOutdated::Window => b"it holds none of the commits main's published counts cover",
        CloseOutdated::Ancestry => b"its ancestry was incomplete",
    }
}

fn fit(words: &List<u8>) -> Box<[u8]> {
    forge::text::fit(words.as_slice(), skein_lib::usize_of(tasks::HOLD_ACCOUNT_BYTES), b"")
}
