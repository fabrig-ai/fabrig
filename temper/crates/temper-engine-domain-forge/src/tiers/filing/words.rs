//! Titles, signatures and bounded words of verdict filings (domain/red-findings.md, 4).
use super::{BlameStrength, Domain, FindingBlame, FindingKind, List, TierRow, VerdictFiling, p};
use crate::{JobLog, LoggedChange, Severity, findings::FindingSignature};
use alloc::boxed::Box;
use skein_forge::api::Commit;
use skein_lib::Decimal;

fn decimal(n: u64) -> Box<[u8]> {
    Box::from(Decimal::of(n).as_bytes())
}
fn short(commit: &Commit) -> Box<[u8]> {
    Box::from(crate::commit_text(commit).get(..12).expect("commit digits"))
}
fn joined(parts: &[&[u8]]) -> Box<[u8]> {
    super::super::joined(parts)
}
pub(super) fn members(d: &Domain, row: &TierRow, commit: Commit) -> Box<[LoggedChange]> {
    if let Some(log) = d.landing_log(row.repository) {
        for landing in &log.landings {
            if landing.commit == commit {
                return landing.changes.clone();
            }
        }
    }
    let mut members = List::uncapped();
    if let Some(verification) = d.landing_verification(row.repository, &commit) {
        for task in &verification.changes {
            let Some(change) = d.change(*task) else { continue };
            members
                .push(LoggedChange {
                    fixes: Box::new([]),
                    task: *task,
                    pull: change.pull,
                    title: change.title.clone(),
                    signed: Box::new([]),
                    goal: change.goal,
                    milestone: change.milestone,
                    person: change.person,
                    goal_title: change.goal_title.clone(),
                    draft: change.draft.clone(),
                })
                .expect("retained landing members");
        }
    }
    members.into_boxed()
}
fn goal_name(d: &Domain, row: &TierRow, landing: &crate::findings::BlamedLanding) -> Box<[u8]> {
    let Some(first) = landing.goals.first() else { return short(&landing.commit) };
    for member in members(d, row, landing.commit) {
        if member.goal == Some(first.goal) {
            if let Some(title) = &member.goal_title
                && let Some(id) = crate::findings::goal_id(title)
            {
                return id.into();
            }
            break;
        }
    }
    joined(&[b"goal ", &decimal(first.goal)])
}
fn failure_words(failing: &p::Failures) -> Box<[u8]> {
    match failing {
        p::Failures::Many => b"many tests fail".as_slice().into(),
        p::Failures::Tests(ids) if ids.len() == 1 => b"1 test fails".as_slice().into(),
        p::Failures::Tests(ids) => joined(&[&decimal(skein_lib::len_u64(ids.len())), b" tests fail"]),
    }
}
fn title(d: &Domain, row: &TierRow, run: u64, failing: &p::Failures, blame: &FindingBlame) -> Box<[u8]> {
    let since = if blame.landings.is_empty() {
        joined(&[b" from before run ", &decimal(run)])
    } else if blame.landings.len() == 1 {
        joined(&[b" since ", &goal_name(d, row, blame.landings.first().expect("one landing")), b" landed"])
    } else {
        joined(&[b" since one of ", &decimal(skein_lib::len_u64(blame.landings.len())), b" landings"])
    };
    crate::text::fit(&joined(&[&failure_words(failing), b" in ", &row.tier, &since]), 200, b"...")
}
pub(super) fn counted(title: &[u8], failing: &p::Failures) -> Box<[u8]> {
    let suffix = title.strip_prefix(b"many tests fail").expect("a many finding's filed title");
    crate::text::fit(&joined(&[&failure_words(failing), suffix]), 200, b"...")
}
fn line(out: &mut List<u8>, words: &[u8], reserve: usize) -> bool {
    if words.len().saturating_add(usize::from(!out.is_empty())).saturating_add(reserve)
        > skein_lib::usize_of(out.room())
    {
        return false;
    }
    if !out.is_empty() {
        out.push(b'\n').expect("line separator room");
    }
    for byte in words {
        out.push(*byte).expect("whole line room");
    }
    true
}
fn url(d: &Domain, row: &TierRow, verdict: &p::Verdict) -> Box<[u8]> {
    match d.landing.receipts.get(&super::super::receipt_key(d, row, verdict.key)) {
        Some(receipt) => receipt.run.url.clone(),
        None => Box::new([]),
    }
}
fn run_words(d: &Domain, row: &TierRow, verdict: &p::Verdict, range: bool) -> Box<[u8]> {
    let tier = super::super::configured(d, row).expect("configured tier");
    let branch = super::super::followed(d, row.repository, &tier).expect("landing tier");
    let landings = &row.record.run.as_ref().expect("judged range").landings;
    let at = if range && !landings.is_empty() {
        let first = short(&landings.first().expect("range first").commit);
        if landings.len() == 1 {
            joined(&[b": landing ", &first])
        } else {
            joined(&[b": landings ", &first, b" to ", &short(&landings.last().expect("range last").commit)])
        }
    } else {
        joined(&[b" at ", &short(&verdict.tip)])
    };
    joined(&[&row.tier, b" run ", &decimal(verdict.number), b" on ", &branch, &at, b"."])
}
fn goal_lines(members: &[LoggedChange], goal: u64) -> List<Box<[u8]>> {
    let mut lines = List::uncapped();
    let mut first = None;
    let mut pulls = List::uncapped();
    for member in members {
        if member.goal == Some(goal) {
            if first.is_none() {
                first = Some(member);
            }
            if let Some(pull) = member.pull
                && !pulls.as_slice().contains(&pull)
            {
                pulls.push(pull).expect("members' pulls");
            }
        }
    }
    let Some(first) = first else { return lines };
    let title = first.goal_title.as_deref().unwrap_or(b"");
    lines.push(joined(&[b"Goal ", &decimal(goal), b": ", title, b"."])).expect("goal line");
    if !pulls.is_empty() {
        let mut text = if pulls.len() == 1 {
            Box::from(b"Pull request #".as_slice())
        } else {
            Box::from(b"Pull requests #".as_slice())
        };
        for (at, pull) in pulls.as_slice().iter().enumerate() {
            text = joined(&[&text, if at == 0 { b"" } else { b", #" }, &decimal(*pull)]);
        }
        lines.push(joined(&[&text, b"."])).expect("pull line");
    }
    if let Some(draft) = &first.draft {
        lines.push(joined(&[b"Draft: ", draft, b"."])).expect("draft line");
    }
    if let Some(person) = first.person {
        lines.push(joined(&[b"Person ", &decimal(person), b"."])).expect("person line");
    }
    lines
}
fn landing_lines(d: &Domain, row: &TierRow, blame: &FindingBlame, out: &mut List<u8>) {
    for (at, landing) in blame.landings.iter().enumerate() {
        let left = blame.landings.len().checked_sub(at).expect("remaining landings");
        let more = joined(&[b"and ", &decimal(skein_lib::len_u64(left)), b" more landings."]);
        let reserve = more.len().saturating_add(b"\nFailing:".len()).saturating_add(1);
        let mut lines = List::uncapped();
        lines.push(joined(&[b"Landing ", &short(&landing.commit), b":"])).expect("landing line");
        for goal in &landing.goals {
            for text in goal_lines(&members(d, row, landing.commit), goal.goal).into_boxed() {
                lines.push(text).expect("bounded goal lines");
            }
        }
        for text in lines.into_boxed() {
            if !line(out, &text, reserve) {
                line(out, &more, b"\nFailing:".len());
                return;
            }
        }
    }
}
fn ids(out: &mut List<u8>, ids: &[Box<[u8]>]) {
    for (at, id) in ids.iter().enumerate() {
        let left = ids.len().checked_sub(at).expect("remaining tests");
        let more = joined(&[b"and ", &decimal(skein_lib::len_u64(left)), b" more tests"]);
        let reserve = if left == 1 { 0 } else { more.len().saturating_add(1) };
        if !line(out, id, reserve) {
            line(out, &more, 0);
            break;
        }
    }
}
fn first(logs: &[JobLog]) -> Box<[u8]> {
    if let Some(log) = logs.first()
        && let Some(end) = &log.end
    {
        let mut start = 0;
        let mut nonblank = false;
        let mut last = b"".as_slice();
        for at in 0..=end.len() {
            if at == end.len() || end.get(at) == Some(&b'\n') {
                if nonblank {
                    last = end.get(start..at).expect("log line");
                }
                start = at.saturating_add(1);
                nonblank = false;
            } else if let Some(byte) = end.get(at) {
                nonblank |= !byte.is_ascii_whitespace();
            }
        }
        return crate::text::fit(last, 512, b"");
    }
    Box::new([])
}
pub(super) fn red(
    d: &Domain,
    row: &TierRow,
    verdict: &p::Verdict,
    failing: p::Failures,
    blame: FindingBlame,
    logs: &[JobLog],
) -> VerdictFiling {
    let mut body = List::with_capacity(4096);
    line(&mut body, &run_words(d, row, verdict, true), 0);
    let url = url(d, row, verdict);
    if !url.is_empty() {
        line(&mut body, &joined(&[b"Run: ", &url]), 0);
    }
    red_body(d, row, verdict.number, failing, blame, logs, body)
}

pub(super) fn switched(
    d: &Domain,
    row: &TierRow,
    verdict: &crate::TierVerdict,
    failing: p::Failures,
    blame: FindingBlame,
    branch: &[u8],
) -> VerdictFiling {
    let mut body = List::with_capacity(4096);
    let commit = blame.landings.first().expect("switched landing").commit;
    line(
        &mut body,
        &joined(&[
            &row.tier,
            b" run ",
            &decimal(verdict.run),
            b" blamed landing ",
            &short(&commit),
            b" on ",
            branch,
            b" before its project ended changes at landing.",
        ]),
        0,
    );
    red_body(d, row, verdict.run, failing, blame, &[], body)
}

fn blame_words(blame: &FindingBlame, run: u64) -> Box<[u8]> {
    match blame.strength {
        BlameStrength::Owned => {
            joined(&[b"Owned by landing ", &short(&blame.landings.first().expect("owned landing").commit), b"."])
        }
        BlameStrength::Suspected => {
            joined(&[b"Suspected: ", &decimal(skein_lib::len_u64(blame.landings.len())), b" landings."])
        }
        BlameStrength::Unblamed => joined(&[b"Blamed on no landing: failing before run ", &decimal(run), b"."]),
        BlameStrength::Moved => match blame.landings.first() {
            Some(landing) => joined(&[b"Moved to landing ", &short(&landing.commit), b"."]),
            None => Box::from(b"Moved to no landing.".as_slice()),
        },
    }
}

fn red_body(
    d: &Domain,
    row: &TierRow,
    run: u64,
    failing: p::Failures,
    blame: FindingBlame,
    logs: &[JobLog],
    mut body: List<u8>,
) -> VerdictFiling {
    let title = title(d, row, run, &failing, &blame);
    if let Some(tier) = super::super::configured(d, row) {
        line(&mut body, &joined(&[b"Artifact: ", &tier.artifact]), 0);
    }
    let reason = blame_words(&blame, run);
    line(&mut body, &reason, b"\nFailing:".len());
    landing_lines(d, row, &blame, &mut body);
    line(&mut body, b"Failing:", 0);
    let (tests, many): (Box<[Box<[u8]>]>, bool) = match failing {
        p::Failures::Tests(tests) => {
            ids(&mut body, &tests);
            (tests, false)
        }
        p::Failures::Many => {
            line(&mut body, b"many tests", 0);
            (Box::new([]), true)
        }
    };
    temper_engine_domain_forge_vocabulary::red::job_log_words(&mut body, logs);
    let mut follow = None;
    for landing in &blame.landings {
        if follow.is_none() {
            follow = super::red::follow(d, row.repository, &landing.commit);
        }
    }
    VerdictFiling {
        project: d.repositories.get(&row.repository).expect("adopted tier").project,
        kind: FindingKind::Red,
        severity: Severity::High,
        title,
        body: body.into_boxed(),
        signature: FindingSignature {
            repository: row.repository,
            source: row.tier.clone(),
            ids: tests,
            first: first(logs),
            many,
        },
        blame: Some(blame),
        follow,
    }
}
pub(super) fn flake(d: &Domain, row: &TierRow, verdict: &p::Verdict, tests: Box<[Box<[u8]>]>) -> VerdictFiling {
    let count = decimal(skein_lib::len_u64(tests.len()));
    let title = joined(&[
        &count,
        if tests.len() == 1 { b" test flaked in " } else { b" tests flaked in " },
        &row.tier,
        b" run ",
        &decimal(verdict.number),
    ]);
    let mut body = List::with_capacity(4096);
    line(&mut body, &run_words(d, row, verdict, false), 0);
    let url = url(d, row, verdict);
    if !url.is_empty() {
        line(&mut body, &joined(&[b"Run: ", &url]), 0);
    }
    ids(&mut body, &tests);
    VerdictFiling {
        project: d.repositories.get(&row.repository).expect("adopted tier").project,
        kind: FindingKind::Flake,
        severity: Severity::Low,
        title: crate::text::fit(&title, 200, b"..."),
        body: body.into_boxed(),
        signature: FindingSignature {
            repository: row.repository,
            source: row.tier.clone(),
            ids: tests,
            first: Box::new([]),
            many: false,
        },
        blame: None,
        follow: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::findings::{BlameMove, BlamedLanding, Filer};

    #[test]
    fn moved_blame_words_name_twelve_commit_digits_or_no_landing() {
        let mut blame = FindingBlame {
            strength: BlameStrength::Moved,
            landings: Box::new([BlamedLanding { commit: [0xab; 32], goals: Box::new([]) }]),
            moved: Some(BlameMove { by: Filer::Task(7), why: b"triaged".as_slice().into() }),
        };
        assert_eq!(blame_words(&blame, 42).as_ref(), b"Moved to landing abababababab.");
        blame.landings = Box::new([]);
        assert_eq!(blame_words(&blame, 42).as_ref(), b"Moved to no landing.");
    }
}
