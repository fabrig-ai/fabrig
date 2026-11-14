//! Verdict projections and their ordered writes (temper/docs/design/domain/after-landing.md, 3, 9 and 10).
use super::{Box, Commit, Domain, Env, Limits, List, Request, TierRow, Write, api, joined, p};
use skein_lib::Queue;

use crate::{TierPosting as Posting, TierPostedLanding as Landing, TierComment as Comment, TierCommentKind as CommentKind};

pub(super) fn status(verdict: &p::Verdict, received: bool) -> (api::Check, Box<[u8]>) {
    let prefix = joined(&[b"run ", skein_lib::Decimal::of(verdict.number).as_bytes()]);
    if let p::Failures::Tests(tests) = &verdict.failing
        && tests.is_empty()
    {
        return (api::Check::Passed, joined(&[&prefix, if received { b" passed by receipt" } else { b" passed" }]));
    }
    let mut distinct = List::uncapped();
    let mut many = false;
    for blame in &verdict.landings {
        match blame {
            p::Blame::Cleared => {}
            p::Blame::Owns(p::Failures::Many) | p::Blame::Suspect(p::Failures::Many) => many = true,
            p::Blame::Owns(p::Failures::Tests(tests)) | p::Blame::Suspect(p::Failures::Tests(tests)) => {
                for test in tests {
                    if !distinct.as_slice().contains(test) {
                        distinct.push(test.clone()).expect("bounded failing ids");
                    }
                }
            }
        }
    }
    let blamed = if many { Box::from(b"many".as_slice()) } else { decimal(distinct.len()) };
    let inherited = match &verdict.failing {
        p::Failures::Many if !many => Box::from(b"many".as_slice()),
        p::Failures::Many => Box::from(b"0".as_slice()),
        p::Failures::Tests(_) => Box::from(skein_lib::Decimal::of(super::inherited_tests(verdict)).as_bytes()),
    };
    (api::Check::Failed, description(&joined(&[&prefix, b" failed: ", &blamed, b" blamed, ", &inherited, b" inherited"])))
}
fn decimal(value: u32) -> Box<[u8]> {
    Box::from(skein_lib::Decimal::of(u64::from(value)).as_bytes())
}
pub(super) fn description(words: &[u8]) -> Box<[u8]> {
    crate::text::fit(words, api::DESCRIPTION_BYTES, b"")
}

/// The key's bytes are charged along with the body, as the client charges a Post.
pub(super) fn comment(
    tier: &[u8],
    number: u64,
    blame: &p::Blame,
    kind: CommentKind,
    link: &[u8],
    room: usize,
) -> Box<[u8]> {
    let run = joined(&[tier, b" run ", skein_lib::Decimal::of(number).as_bytes()]);
    let suffix = if link.is_empty() { Box::from(b".".as_slice()) } else { joined(&[b".\n\nRun: ", link]) };
    let (prefix, tests) = match kind {
        CommentKind::Repaired { by: pull } => {
            let prefix = match pull {
                Some(pull) => joined(&[b"Repaired by #", skein_lib::Decimal::of(pull).as_bytes(), b": "]),
                None => Box::from(b"Repaired: ".as_slice()),
            };
            return joined(&[&prefix, &run, b" passed the failures blamed on this landing", &suffix]);
        }
        CommentKind::Verdict { .. } => match blame {
            p::Blame::Cleared => return joined(&[b"Cleared by ", &run, &suffix]),
            p::Blame::Owns(tests) => (joined(&[b"Red: ", &run, b" blames this landing for "]), tests),
            p::Blame::Suspect(tests) => (joined(&[b"Suspected: ", &run, b" suspects this landing of "]), tests),
        },
    };
    let p::Failures::Tests(tests) = tests else { return joined(&[&prefix, b"many tests", &suffix]) };
    let mut complete = Box::from(b"".as_slice());
    for (at, test) in tests.iter().enumerate() {
        complete = joined(&[&complete, if at == 0 { b"" } else { b", " }, test]);
    }
    let complete = joined(&[&prefix, &complete, &suffix]);
    if complete.len() <= room { return complete; }
    let mut ids = Box::from(b"".as_slice());
    let count = u32::try_from(tests.len()).expect("bounded test ids");
    for (at, test) in tests.iter().enumerate() {
        let separator = if at == 0 { b"".as_slice() } else { b", ".as_slice() };
        let next = joined(&[&ids, separator, test]);
        let left = count.checked_sub(u32::try_from(at).expect("test index")).expect("remaining ids");
        let remaining = left.checked_sub(1).expect("includes current id");
        let tail = remainder(remaining, true);
        if joined(&[&prefix, &next, &tail, &suffix]).len() > room {
            return joined(&[&prefix, &ids, &remainder(left, at != 0), &suffix]);
        }
        ids = next;
    }
    joined(&[&prefix, &ids, &suffix])
}
fn remainder(count: u32, preceding: bool) -> Box<[u8]> {
    if count == 0 {
        return Box::new([]);
    }
    joined(&[if preceding { b", and " } else { b"and " }, &decimal(count), b" more"])
}

fn members(d: &Domain, row: &TierRow, commit: Commit) -> Box<[u64]> {
    let mut pulls = List::uncapped();
    if let Some(log) = d.landing_logs.get(&row.repository) {
        for landing in &log.landings {
            if landing.commit == commit {
                for member in &landing.changes {
                    if let Some(pull) = member.pull {
                        pulls.push(pull).expect("bounded logged members");
                    }
                }
                return pulls.into_boxed();
            }
        }
    }
    if let Some(verification) = d.verifications.get(&(row.repository, commit)) {
        for task in &verification.changes {
            if let Some(change) = d.changes.get(task)
                && let Some(pull) = change.pull
            {
                pulls.push(pull).expect("bounded live members");
            }
        }
    }
    pulls.into_boxed()
}
fn repairing_pull(d: &Domain, row: &TierRow, commit: Commit, range: &[p::Landing]) -> Option<u64> {
    for landing in range.iter().rev() {
        if let Some(verification) = d.verifications.get(&(row.repository, landing.commit)) {
            for task in &verification.changes {
                if let Some(change) = d.changes.get(task)
                    && change.repairs == Some(commit)
                {
                    return change.pull;
                }
            }
        }
    }
    None
}

/// Called after verification is judged, before its changes can release their rows.
pub(super) fn compose(d: &Domain, row: &TierRow, verdict: &p::Verdict) -> Posting {
    let range = &row.record.run.as_ref().expect("judged run").landings;
    let (check, description) = status(verdict, row.received);
    let link = match d.landing.receipts.get(&super::receipt_key(d, row, verdict.key)) {
        Some(receipt) => receipt.run.url.clone(),
        None => Box::new([]),
    };
    let mut landings = List::with_capacity(u32::try_from(range.len()).expect("bounded range"));
    let mut comments = List::with_capacity(temper_engine_domain_forge_vocabulary::tiers::posting_comments(&d.landing.limits).expect("checked posting bound"));
    for (landing, blame) in range.iter().zip(&verdict.landings) {
        let at = landings.len();
        landings.push(Landing { commit: landing.commit, blame: blame.clone(), change: super::landing_change(d, row.repository, landing.commit) }).expect("bounded judged range");
        for pull in members(d, row, landing.commit) {
            comments.push(Comment { pull, landing: landing.commit, kind: CommentKind::Verdict { at: u16::try_from(at).expect("bounded range") } }).expect("bounded landing members");
        }
    }
    for ((repository, commit), verification) in &d.verifications {
        if *repository != row.repository {
            continue;
        }
        for tier in &verification.tiers {
            if tier.tier == row.tier
                && let Some(repaired) = tier.repaired
                && repaired.run == verdict.number
            {
                let kind = CommentKind::Repaired { by: repairing_pull(d, row, *commit, range) };
                for pull in members(d, row, *commit) {
                    comments.push(Comment { pull, landing: *commit, kind }).expect("bounded repaired members");
                }
            }
        }
    }
    Posting {
        number: verdict.number,
        tip: verdict.tip,
        link,
        check,
        description,
        landings: landings.into_boxed(),
        comments: comments.into_boxed(),
        next: 0,
        entry: None,
        part: 0,
        op_bytes: d.landing.limits.client.op_bytes,
    }
}

pub(super) fn pending(d: &Domain, row: &TierRow, follow: &crate::runs::Follow) -> Posting {
    let run = row.record.run.as_ref().expect("held run");
    let previous = match &row.posting { Some(posting) => posting.part, None => 0 };
    let exhausted = previous == u16::MAX;
    let description = description(&joined(&[
        b"held after ", &decimal(run.faults), b" runner faults, last: ",
        &super::fault_words(run, &super::configured(d, row).expect("configured tier").workflow, row.held.expect("fault")),
    ]));
    Posting {
        number: run.number,
        tip: run.tip,
        link: match &follow.run { Some(run) => run.url.clone(), None => Box::new([]) },
        check: api::Check::Pending,
        description,
        landings: Box::new([]),
        comments: Box::new([]),
        next: u32::from(exhausted),
        entry: None,
        // Completed pending progress stays until the verdict, so another hold has a fresh purpose.
        part: previous.saturating_add(1),
        op_bytes: d.landing.limits.client.op_bytes,
    }
}
fn write(d: &Domain, row: &TierRow, posting: &Posting) -> Write {
    if posting.next == 0 {
        return Write::Status {
            commit: posting.tip, context: joined(&[b"temper/verdict/", &row.tier]), check: posting.check,
            description: posting.description.clone(), link: posting.link.clone(),
        };
    }
    let part = posting.next.checked_sub(1).expect("comment index");
    let comment_row = posting.comments.get(skein_lib::usize_of(part)).expect("next comment");
    let blame = match comment_row.kind {
        CommentKind::Verdict { at } => &posting.landings.get(usize::from(at)).expect("comment landing").blame,
        CommentKind::Repaired { .. } => &p::Blame::Cleared,
    };
    let key = super::purpose(d, row, 7, u16::try_from(part).expect("bounded comments"))
        .key(&d.namespace, posting.op_bytes).expect("tier key fits operation");
    let room = skein_lib::usize_of(posting.op_bytes).saturating_sub(key.len());
    Write::Post {
        number: comment_row.pull, key,
        body: comment(&row.tier, posting.number, blame, comment_row.kind, &posting.link, room),
    }
}
pub(super) fn active(row: &TierRow) -> bool {
    match &row.posting {
        Some(posting) => posting.part == 0 || posting.next <= u32::try_from(posting.comments.len()).expect("bounded comments"),
        None => false,
    }
}
pub(super) fn step(d: &mut Domain, env: &Env<Limits>, row: &mut TierRow, entry: u64, out: &mut Queue<Request>) {
    let mut posting = row.posting.take().expect("posting run");
    if posting.entry.is_some() {
        row.posting = Some(posting);
        super::wait(row.task.expect("posting task"), out);
        return;
    }
    if posting.next > u32::try_from(posting.comments.len()).expect("bounded comments") {
        if posting.part == 0 {
            let mut blames = List::with_capacity(u32::try_from(posting.landings.len()).expect("bounded range"));
            for landing in &posting.landings { blames.push(landing.blame.clone()).expect("bounded range"); }
            let verdict = p::Verdict { number: posting.number, tip: posting.tip, key: row.record.run.as_ref().expect("posting run").key,
                failing: row.record.failing.clone(), landings: blames.into_boxed(), flakes: Box::new([]) };
            let words = super::posted_words(&row.tier, &verdict, &posting.landings);
            super::done(d, row, words, out);
        } else {
            row.posting = Some(posting);
            super::save(row, out);
            super::held(d, row, out);
        }
        return;
    }
    if !super::room(d, env, entry) {
        row.posting = Some(posting);
        super::wait(row.task.expect("posting task"), out);
        return;
    }
    let write = write(d, row, &posting);
    posting.entry = Some(entry);
    row.posting = Some(posting);
    super::effect(d, row, entry, write, false, out);
}
pub(super) fn settled(d: &mut Domain, entry: u64, out: &mut Queue<Request>) -> bool {
    let mut found = None;
    for (key, row) in &d.tiers {
        if let Some(posting) = &row.posting
            && posting.entry == Some(entry)
        {
            found = Some(key.clone());
            break;
        }
    }
    let Some(key) = found else { return false };
    let row = d.tiers.get_mut(&key).expect("posting tier");
    let posting = row.posting.as_mut().expect("posting entry");
    posting.entry = None;
    posting.next = posting.next.checked_add(1).expect("bounded writes");
    if let Some(task) = row.task {
        out.push(Request::WakeProcedure { task });
    }
    super::save(row, out);
    true
}

pub(super) fn identity(row: &TierRow) -> (u8, u16) {
    let posting = row.posting.as_ref().expect("posting effect");
    if posting.next == 0 { (6, posting.part) } else {
        (7, u16::try_from(posting.next.checked_sub(1).expect("comment index")).expect("bounded comments"))
    }
}
pub(super) fn describe(d: &Domain, row: &TierRow, entry: u64) -> Option<(u16, crate::What, Commit)> {
    let posting = row.posting.as_ref()?;
    if posting.entry != Some(entry) || d.entries.get(&entry)?.entry.effect.write != write(d, row, posting) {
        return None;
    }
    let (operation, part) = identity(row);
    if row.purpose != Some(super::purpose(d, row, operation, part)) { return None; }
    let (kind, what) = if posting.next == 0 { (6, crate::What::Repository) } else {
        let comment = posting.comments.get(skein_lib::usize_of(posting.next.checked_sub(1)?))?;
        (7, crate::What::Pull(comment.pull))
    };
    Some((kind, what, posting.tip))
}

#[cfg(test)]
#[path = "../tests/tiers/post_words.rs"]
mod tests;
