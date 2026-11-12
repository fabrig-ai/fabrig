//! A red tier run told to a goal (temper/docs/design/domain/after-landing.md, 7).

use alloc::boxed::Box;
use skein_forge::api::Commit;
use temper_engine_domain_forge_tiers::Failures;

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct RedNews {
    pub tier: Box<[u8]>,
    /// The run's number in the tier's sequence.
    pub run: u64,
    /// The forge run's link, once observed.
    pub url: Option<Box<[u8]>>,
    pub on: RedOn,
    /// Each failed job of the run's last failed part, in the run's order.
    pub logs: Box<[JobLog]>,
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum RedOn {
    /// A red after landing: the landing, its branch, the failures blamed on it.
    Landing { commit: Commit, branch: Box<[u8]>, failing: Failures },
    /// A red of a tier following goal branches: the branch, the run's tip, its tests new and still failing.
    Goal { branch: Box<[u8]>, tip: Commit, new: Failures, still: Failures },
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct JobLog {
    pub name: Box<[u8]>,
    /// The log's end, cut to its share; `None` when it could not be read.
    pub end: Option<Box<[u8]>>,
}

use crate::commit_text;
use skein_lib::List;

/// A red's whole lines precede log tails (temper/docs/design/domain/after-landing.md, 7).
#[must_use]
pub fn red_news_words(red: &RedNews, limit: u32) -> Box<[u8]> {
    let mut out = List::with_capacity(limit);
    let complete = match &red.on {
        RedOn::Landing { commit, branch, failing } => red_landing_words(&mut out, red, commit, branch, failing),
        RedOn::Goal { branch, tip, new, still } => red_goal_words(&mut out, red, branch, tip, new, still),
    };
    if !complete {
        return out.into_boxed();
    }
    if let Some(url) = &red.url
        && !red_line(&mut out, &[b"Run: ", url])
    {
        return out.into_boxed();
    }
    red_logs(&mut out, &red.logs);
    out.into_boxed()
}

fn red_landing_words(out: &mut List<u8>, red: &RedNews, commit: &Commit, branch: &[u8], failing: &Failures) -> bool {
    let run = skein_lib::Decimal::of(red.run);
    let commit = commit_text(commit);
    let count = red_failure_count(failing);
    if !red_line(
        out,
        &[
            b"Red after landing: tier ",
            &red.tier,
            b" run ",
            run.as_bytes(),
            b" blames landing ",
            &commit,
            b" on ",
            branch,
            b": ",
            &count,
            b" failing.",
        ],
    ) {
        return false;
    }
    red_test_lines(out, b"", failing)
}

fn red_goal_words(
    out: &mut List<u8>,
    red: &RedNews,
    branch: &[u8],
    tip: &Commit,
    new: &Failures,
    still: &Failures,
) -> bool {
    let run = skein_lib::Decimal::of(red.run);
    let tip = commit_text(tip);
    let tip = tip.get(..12).expect("a commit has at least twelve digits");
    let new_count = red_failure_count(new);
    let still_count = red_failure_count(still);
    if !red_line(
        out,
        &[
            b"Red on ",
            branch,
            b": tier ",
            &red.tier,
            b" run ",
            run.as_bytes(),
            b" at ",
            tip,
            b": ",
            &new_count,
            b" new, ",
            &still_count,
            b" still failing.",
        ],
    ) {
        return false;
    }
    if !red_test_lines(out, b"new: ", new) {
        return false;
    }
    red_test_lines(out, b"still failing: ", still)
}

fn red_failure_count(failing: &Failures) -> Box<[u8]> {
    match failing {
        Failures::Tests(tests) => {
            let count = skein_lib::len_u64(tests.len());
            Box::from(skein_lib::Decimal::of(count).as_bytes())
        }
        Failures::Many => Box::from(b"many".as_slice()),
    }
}

fn red_test_lines(out: &mut List<u8>, prefix: &[u8], failing: &Failures) -> bool {
    match failing {
        Failures::Tests(tests) => {
            for test in tests {
                if !red_line(out, &[prefix, test]) {
                    return false;
                }
            }
            true
        }
        Failures::Many => red_line(out, &[prefix, b"many"]),
    }
}

fn red_line_bytes(parts: &[&[u8]]) -> usize {
    let mut bytes = 0_usize;
    for part in parts {
        bytes = bytes.saturating_add(part.len());
    }
    bytes
}

fn red_line(out: &mut List<u8>, parts: &[&[u8]]) -> bool {
    let separator = usize::from(!out.is_empty());
    let bytes = red_line_bytes(parts).saturating_add(separator);
    if bytes > usize::try_from(out.room()).expect("message room fits usize") {
        return false;
    }
    if !out.is_empty() {
        append(out, b"\n");
    }
    for part in parts {
        append(out, part);
    }
    true
}

fn red_logs(out: &mut List<u8>, logs: &[JobLog]) {
    let room = usize::try_from(out.room()).expect("message room fits usize");
    let share = room.checked_div(logs.len()).unwrap_or(0);
    for log in logs {
        match &log.end {
            Some(end) => {
                let heading: &[&[u8]] = &[b"End of job ", &log.name, b"'s log:"];
                let overhead = red_line_bytes(heading).saturating_add(2);
                if overhead > share {
                    continue;
                }
                let length = end.len().min(share.saturating_sub(overhead));
                let start = end.len().checked_sub(length).expect("tail within log");
                let tail = end.get(start..).expect("tail within log");
                assert!(red_line(out, heading), "heading fits the job's share");
                append(out, b"\n");
                append(out, tail);
            }
            None => {
                let line: &[&[u8]] = &[b"End of job ", &log.name, b"'s log: [no log]"];
                if red_line_bytes(line).saturating_add(1) <= share {
                    assert!(red_line(out, line), "unread log line fits the job's share");
                }
            }
        }
    }
}

fn append(out: &mut List<u8>, bytes: &[u8]) {
    for byte in bytes {
        if out.room() > 0 {
            out.push(*byte).expect("checked message room");
        }
    }
}
