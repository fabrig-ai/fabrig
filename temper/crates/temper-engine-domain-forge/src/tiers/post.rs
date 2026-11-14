//! Verdict projections and their ordered writes (temper/docs/design/domain/after-landing.md, 3, 9 and 10).
use super::{Box, Commit, Domain, Env, Limits, List, Request, TierRow, Write, api, joined, p};
use skein_lib::Queue;

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub(super) struct Posting {
    pub number: u64,
    pub tip: Commit,
    pub link: Box<[u8]>,
    pub check: api::Check,
    pub description: Box<[u8]>,
    pub landings: Box<[Landing]>,
    pub comments: Box<[Comment]>,
    pub next: u32,
    pub entry: Option<u64>,
    pub words: Option<Box<[u8]>>,
    pub part: u16,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub(super) struct Landing {
    pub commit: Commit,
    pub blame: p::Blame,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub(super) struct Comment {
    pub pull: u64,
    pub landing: u32,
    pub kind: CommentKind,
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub(super) enum CommentKind {
    Verdict,
    Repaired { by: Option<u64> },
}

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
        CommentKind::Verdict => match blame {
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
pub(super) fn compose(d: &Domain, row: &TierRow, verdict: &p::Verdict, words: Box<[u8]>) -> Posting {
    let range = &row.record.run.as_ref().expect("judged run").landings;
    let (check, description) = status(verdict, row.received);
    let link = match d.landing.receipts.get(&super::receipt_key(d, row, verdict.key)) {
        Some(receipt) => receipt.run.url.clone(),
        None => Box::new([]),
    };
    let mut landings = List::uncapped();
    let mut comments = List::uncapped();
    for (landing, blame) in range.iter().zip(&verdict.landings) {
        let at = landings.len();
        landings.push(Landing { commit: landing.commit, blame: blame.clone() }).expect("bounded judged range");
        for pull in members(d, row, landing.commit) {
            comments.push(Comment { pull, landing: at, kind: CommentKind::Verdict }).expect("bounded landing members");
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
                let at = landings.len();
                let blame = tier.verdict.as_ref().expect("repaired red verdict").blame.clone();
                landings.push(Landing { commit: *commit, blame }).expect("bounded repaired landings");
                let kind = CommentKind::Repaired { by: repairing_pull(d, row, *commit, range) };
                for pull in members(d, row, *commit) {
                    comments.push(Comment { pull, landing: at, kind }).expect("bounded repaired members");
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
        words: Some(words),
        part: 0,
    }
}

pub(super) fn pending(d: &Domain, row: &TierRow, follow: &crate::runs::Follow) -> Posting {
    let run = row.record.run.as_ref().expect("held run");
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
        next: 0,
        entry: None,
        words: None,
        // A later hold of the same run must not reuse an already settled client purpose.
        part: u16::try_from(run.faults).expect("bounded retries").checked_add(1).expect("pending part"),
    }
}
fn write(d: &Domain, limits: &Limits, row: &TierRow, posting: &Posting) -> Write {
    if posting.next == 0 {
        return Write::Status {
            commit: posting.tip, context: joined(&[b"temper/verdict/", &row.tier]), check: posting.check,
            description: posting.description.clone(), link: posting.link.clone(),
        };
    }
    let part = posting.next.checked_sub(1).expect("comment index");
    let comment_row = posting.comments.get(skein_lib::usize_of(part)).expect("next comment");
    let landing = posting.landings.get(skein_lib::usize_of(comment_row.landing)).expect("comment landing");
    let key = super::purpose(d, row, 7, u16::try_from(part).expect("bounded comments"))
        .key(&d.namespace, limits.client.op_bytes).expect("tier key fits operation");
    let room = skein_lib::usize_of(limits.client.op_bytes).saturating_sub(key.len());
    Write::Post {
        number: comment_row.pull, key,
        body: comment(&row.tier, posting.number, &landing.blame, comment_row.kind, &posting.link, room),
    }
}
pub(super) fn step(d: &mut Domain, env: &Env<Limits>, row: &mut TierRow, entry: u64, out: &mut Queue<Request>) {
    let key = (row.repository, row.tier.clone());
    let mut posting = d.tier_ops.postings.remove(&key).expect("posting run");
    if posting.entry.is_some() {
        d.tier_ops.postings.insert(key, posting).expect("one posting per tier");
        super::wait(row.task.expect("posting task"), out);
        return;
    }
    if posting.next > u32::try_from(posting.comments.len()).expect("bounded comments") {
        if let Some(words) = posting.words.take() {
            super::done(d, row, words, out);
        } else {
            super::held(d, row, out);
        }
        return;
    }
    if !super::room(d, env, entry) {
        d.tier_ops.postings.insert(key, posting).expect("one posting per tier");
        super::wait(row.task.expect("posting task"), out);
        return;
    }
    let write = write(d, &env.limits, row, &posting);
    posting.entry = Some(entry);
    d.tier_ops.postings.insert(key, posting).expect("one posting per tier");
    super::effect(d, row, entry, write, false, out);
}
pub(super) fn settled(d: &mut Domain, entry: u64, out: &mut Queue<Request>) -> bool {
    let mut found = None;
    for (key, posting) in &d.tier_ops.postings {
        if posting.entry == Some(entry) {
            found = Some(key.clone());
            break;
        }
    }
    let Some(key) = found else { return false };
    let posting = d.tier_ops.postings.get_mut(&key).expect("posting entry");
    posting.entry = None;
    posting.next = posting.next.checked_add(1).expect("bounded writes");
    if let Some(row) = d.tiers.get(&key)
        && let Some(task) = row.task
    {
        out.push(Request::WakeProcedure { task });
    }
    true
}

pub(super) fn identity(d: &Domain, row: &TierRow) -> (u8, u16) {
    let posting = d.tier_ops.postings.get(&(row.repository, row.tier.clone())).expect("posting effect");
    if posting.next == 0 { (6, posting.part) } else {
        (7, u16::try_from(posting.next.checked_sub(1).expect("comment index")).expect("bounded comments"))
    }
}
pub(super) fn describe(d: &Domain, row: &TierRow, entry: u64) -> Option<(u16, crate::What, Commit)> {
    let posting = d.tier_ops.postings.get(&(row.repository, row.tier.clone()))?;
    if posting.entry != Some(entry) || d.entries.get(&entry)?.entry.effect.write != write(d, &d.landing.limits, row, posting) {
        return None;
    }
    let (operation, part) = identity(d, row);
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
