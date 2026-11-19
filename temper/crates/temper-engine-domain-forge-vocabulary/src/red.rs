//! A red tier run told to a goal (temper/docs/design/domain/after-landing.md, 7).

use alloc::boxed::Box;
use skein_forge::api::Commit;
use temper_engine_domain_forge_tiers::{Failures, ManyTests};

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct RedNews {
    /// The run's whole report's names, shown only for many failures.
    pub many: Option<ManyTests>,
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
    job_log_words(&mut out, &red.logs);
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
    red_test_lines(out, b"", failing, red)
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
    if !red_test_lines(out, b"new: ", new, red) {
        return false;
    }
    red_test_lines(out, b"still failing: ", still, red)
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

fn red_test_lines(out: &mut List<u8>, prefix: &[u8], failing: &Failures, red: &RedNews) -> bool {
    match failing {
        Failures::Tests(tests) => {
            for test in tests {
                if !red_line(out, &[prefix, test]) {
                    return false;
                }
            }
            true
        }
        Failures::Many => match &red.many {
            Some(many) => red_many_lines(out, prefix, many, red.url.as_deref()),
            None => red_line(out, &[prefix, b"many"]),
        },
    }
}

/// The report's unshown cases and ids, without a full stop (after-landing.md, 4).
#[must_use]
pub fn many_remainder(many: &ManyTests, left_out: u32) -> Option<Box<[u8]>> {
    let count = many.more.saturating_add(left_out);
    if count == 0 && !many.cut {
        return None;
    }
    let mut out = List::uncapped();
    append(&mut out, b"and ");
    if count != 0 {
        append(&mut out, skein_lib::Decimal::of(u64::from(count)).as_bytes());
        append(&mut out, b" ");
    }
    append(&mut out, b"more");
    if many.cut {
        append(&mut out, b", its report cut at the read's bound");
    }
    Some(out.into_boxed())
}

fn red_many_lines(out: &mut List<u8>, prefix: &[u8], many: &ManyTests, url: Option<&[u8]>) -> bool {
    if !red_line(out, &[prefix, b"many; its report names:"]) {
        return false;
    }
    let run_bytes = match url {
        Some(url) => red_line_bytes(&[b"\nRun: ", url]),
        None => 0,
    };
    let count = u32::try_from(many.tests.len()).expect("bounded many names");
    let mut kept = 0_u32;
    let mut ids_bytes = 0_usize;
    // The final id can remove the remainder line, so examine every prefix.
    for (at, test) in many.tests.iter().enumerate() {
        ids_bytes = ids_bytes.saturating_add(red_line_bytes(&[b"\n", prefix, test]));
        let take = u32::try_from(at).expect("bounded names").checked_add(1).expect("one more id");
        let left = count.checked_sub(take).expect("prefix within names");
        let remainder_bytes = match many_remainder(many, left) {
            Some(line) => red_line_bytes(&[b"\n", prefix, &line]),
            None => 0,
        };
        if ids_bytes.saturating_add(remainder_bytes).saturating_add(run_bytes) <= skein_lib::usize_of(out.room()) {
            kept = take;
        }
    }
    for test in many.tests.iter().take(skein_lib::usize_of(kept)) {
        assert!(red_line(out, &[prefix, test]), "id leaves room for remainder and run");
    }
    match many_remainder(many, count.checked_sub(kept).expect("kept names within count")) {
        Some(line) => red_line(out, &[prefix, &line]),
        None => true,
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

/// Append failed jobs' log ends, sharing the remaining room (domain/after-landing.md, 7).
pub fn job_log_words(out: &mut List<u8>, logs: &[JobLog]) {
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
