//! Level-triggered verdict decisions (temper/docs/design/domain/after-landing.md, sections 3–5).
use crate::{
    Blame, Commit, Failures, Finding, Flake, Group, Judged, Key, Landing, Limits, Next, Outcome, Part, Receipt, Record,
    Run, Selection, Settings, Suspects, TestId, Verdict,
};
use alloc::boxed::Box;
use skein_lib::List;

/// On a tip's move, whether the tier needs a run.
#[must_use]
pub fn due(record: &Record, tip: Commit) -> bool {
    if record.run.is_some() {
        return false;
    }
    match record.judged {
        Some(judged) => judged.tip != tip,
        None => true,
    }
}

/// Starts the next run, or judges the tip by its receipt.
#[must_use]
pub fn start(
    record: &Record,
    tip: Commit,
    key: Key,
    landings: Box<[Landing]>,
    receipt: Option<Receipt>,
    settings: &Settings,
) -> Next {
    let run = Run {
        number: record.sequence.checked_add(1).expect("the tier sequence fits u64"),
        tip,
        key,
        landings,
        part: Part::Whole,
        faults: 0,
    };
    match receipt {
        None => Next::Dispatch(run),
        Some(Receipt::Passed) => green(&run, Box::new([])),
        Some(Receipt::Failed(failing)) => blame(record, &run, failing, Box::new([]), *settings),
    }
}

/// Reads the completion of the committed part of a run.
#[must_use]
pub fn step(record: &Record, run: &Run, outcome: Outcome, settings: &Settings) -> Next {
    let reported = match outcome {
        Outcome::Fault => {
            let mut retry = run.clone();
            // Even u32::MAX retries has a terminal fault; it must not wrap.
            let Some(faults) = run.faults.checked_add(1) else {
                return Next::Held(retry);
            };
            retry.faults = faults;
            return if faults <= settings.retries { Next::Dispatch(retry) } else { Next::Held(retry) };
        }
        Outcome::Passed => Failures::Tests(Box::new([])),
        Outcome::Failed(failing) => failing,
    };
    match &run.part {
        Part::Whole => match reported {
            Failures::Tests(tests) => {
                if tests.is_empty() {
                    green(run, Box::new([]))
                } else {
                    dispatch(run, Part::Rerun { failed: tests })
                }
            }
            Failures::Many => blame(record, run, Failures::Many, Box::new([]), *settings),
        },
        Part::Rerun { failed } => {
            let confirmed = within(failed, &reported);
            let flakes = select(failed, &confirmed, false);
            blame(record, run, Failures::Tests(confirmed), flakes, *settings)
        }
        Part::Probe { landing, low, high, tests, waiting, finding } => {
            let finding = finding.clone();
            let (owned, remaining) = match tests {
                Selection::All => match reported {
                    Failures::Many => (Failures::Many, empty()),
                    Failures::Tests(tests) => {
                        if tests.is_empty() {
                            (empty(), Failures::Many)
                        } else {
                            (Failures::Many, empty())
                        }
                    }
                },
                Selection::Tests(tests) => {
                    let owned = within(tests, &reported);
                    let remaining = select(tests, &owned, false);
                    (Failures::Tests(owned), Failures::Tests(remaining))
                }
            };
            let mut groups = List::with_capacity(length(waiting.len()).checked_add(2).expect("two halves fit"));
            add_group(&mut groups, *low, *landing, owned);
            add_group(&mut groups, landing.checked_add(1).expect("the middle is before high"), *high, remaining);
            for group in waiting {
                let tests = expand(&finding, &group.tests);
                groups
                    .push(SearchGroup { low: group.low, high: group.high, tests })
                    .expect("room for waiting groups and two halves");
            }
            probe(record, run, finding, groups.as_slice())
        }
    }
}

fn empty() -> Failures {
    Failures::Tests(Box::new([]))
}

fn none(failing: &Failures) -> bool {
    match failing {
        Failures::Tests(tests) => tests.is_empty(),
        Failures::Many => false,
    }
}

fn length(length: usize) -> u32 {
    u32::try_from(length).expect("top-owned bounded slices fit u32")
}

fn cleared(run: &Run) -> Box<[Blame]> {
    let mut blamed = List::with_capacity(length(run.landings.len()));
    for _landing in &run.landings {
        blamed.push(Blame::Cleared).expect("one blame per landing");
    }
    blamed.into_boxed()
}

fn finish(run: &Run, finding: Finding) -> Next {
    Next::Judged(Verdict {
        number: run.number,
        tip: run.tip,
        key: run.key,
        failing: finding.failing,
        landings: finding.blamed,
        flakes: finding.flakes,
    })
}

fn green(run: &Run, flakes: Box<[TestId]>) -> Next {
    finish(run, Finding { failing: empty(), flakes, blamed: cleared(run) })
}

fn dispatch(run: &Run, part: Part) -> Next {
    let mut next = run.clone();
    next.part = part;
    next.faults = 0;
    Next::Dispatch(next)
}

/// Intersection or difference of two sorted sets, by a monotone merge.
fn select(left: &[TestId], right: &[TestId], shared: bool) -> Box<[TestId]> {
    let mut selected = List::with_capacity(length(left.len()));
    let mut from = 0_usize;
    for test in left {
        for other in right.get(from..).expect("merge cursor stays within right") {
            if other >= test {
                break;
            }
            from = from.checked_add(1).expect("cursor within the slice");
        }
        if (right.get(from) == Some(test)) == shared {
            selected.push(test.clone()).expect("a subset fits its source length");
        }
    }
    selected.into_boxed()
}

// An oversized partial report cannot identify which selected tests passed:
// conservatively retain the selection. Unrequested test ids never count.
fn within(selected: &[TestId], reported: &Failures) -> Box<[TestId]> {
    match reported {
        Failures::Tests(tests) => select(selected, tests, true),
        Failures::Many => Box::from(selected),
    }
}

fn blame(record: &Record, run: &Run, failing: Failures, flakes: Box<[TestId]>, settings: Settings) -> Next {
    let new = match &failing {
        Failures::Tests(tests) => match &record.failing {
            Failures::Tests(previous) => Failures::Tests(select(tests, previous, false)),
            Failures::Many => failing.clone(),
        },
        Failures::Many => match &record.failing {
            Failures::Many => empty(),
            Failures::Tests(_) => Failures::Many,
        },
    };
    let mut finding = Finding { failing, flakes, blamed: cleared(run) };
    if none(&new) || run.landings.is_empty() {
        return finish(run, finding);
    }
    if run.landings.len() == 1 {
        own(&mut finding, 0, new);
        return finish(run, finding);
    }
    match settings.suspects {
        Suspects::TellAll => {
            for blame in &mut finding.blamed {
                *blame = Blame::Suspect(new.clone());
            }
            finish(run, finding)
        }
        Suspects::Bisect => {
            let mut groups = List::with_capacity(1);
            let high = length(run.landings.len()).checked_sub(1).expect("a non-baseline has landings");
            add_group(&mut groups, 0, high, new);
            probe(record, run, finding, groups.as_slice())
        }
    }
}

fn own(finding: &mut Finding, landing: u32, failing: Failures) {
    if !none(&failing) {
        let blame = finding.blamed.get_mut(skein_lib::usize_of(landing));
        *blame.expect("a probe names a landing in its range") = Blame::Owns(failing);
    }
}

#[derive(Debug)]
struct SearchGroup {
    low: u32,
    high: u32,
    tests: Selection,
}

fn expand(finding: &Finding, places: &[u16]) -> Selection {
    let Failures::Tests(failing) = &finding.failing else {
        unreachable!("after many no group waits");
    };
    let mut tests = List::with_capacity(length(places.len()));
    for place in places {
        tests
            .push(failing.get(usize::from(*place)).expect("a waiting group's place is in the finding").clone())
            .expect("one test per place");
    }
    Selection::Tests(tests.into_boxed())
}

fn compact(finding: &Finding, groups: &[SearchGroup]) -> Box<[Group]> {
    let mut waiting = List::with_capacity(length(groups.len()));
    for group in groups {
        let Failures::Tests(failing) = &finding.failing else {
            unreachable!("after many only one half survives");
        };
        let Selection::Tests(tests) = &group.tests else {
            unreachable!("only enumerated failures have waiting groups");
        };
        let mut places = List::with_capacity(length(tests.len()));
        for (place, test) in failing.iter().enumerate() {
            if tests.binary_search(test).is_ok() {
                places
                    .push(u16::try_from(place).expect("at most 1024 enumerated failures"))
                    .expect("a subset of the selected tests");
            }
        }
        waiting
            .push(Group { low: group.low, high: group.high, tests: places.into_boxed() })
            .expect("one waiting group per remaining group");
    }
    waiting.into_boxed()
}

fn add_group(groups: &mut List<SearchGroup>, low: u32, high: u32, failing: Failures) {
    if low <= high && !none(&failing) {
        let tests = match failing {
            Failures::Tests(tests) => Selection::Tests(tests),
            Failures::Many => Selection::All,
        };
        groups.push(SearchGroup { low, high, tests }).expect("room for the two nonempty halves");
    }
}

fn candidates(record: &Record, run: &Run, group: &SearchGroup) -> Box<[u32]> {
    let mut previous = None;
    if let Some(judged) = record.judged {
        previous = Some(judged.key);
    }
    let mut candidates = List::with_capacity(length(run.landings.len()));
    for (index, landing) in run.landings.iter().enumerate() {
        let index = length(index);
        if index >= group.low && index <= group.high && previous != Some(landing.key) {
            candidates.push(index).expect("at most one candidate per landing");
        }
        previous = Some(landing.key);
    }
    candidates.into_boxed()
}

fn probe(record: &Record, run: &Run, mut finding: Finding, groups: &[SearchGroup]) -> Next {
    // The lower half precedes the upper, both before the disjoint waiting groups.
    for (index, group) in groups.iter().enumerate() {
        let candidates = candidates(record, run, group);
        if candidates.len() <= 1 {
            let landing = candidates.first().copied().unwrap_or(group.high);
            let failing = match &group.tests {
                Selection::Tests(tests) => Failures::Tests(tests.clone()),
                Selection::All => Failures::Many,
            };
            own(&mut finding, landing, failing);
        } else {
            let middle = candidates.len().checked_sub(1).expect("several candidates").checked_div(2).expect("two");
            let landing = *candidates.get(middle).expect("the middle candidate");
            let after = index.checked_add(1).expect("group cursor fits");
            let waiting = compact(&finding, groups.get(after..).expect("groups after this one"));
            return dispatch(
                run,
                Part::Probe { landing, low: group.low, high: group.high, tests: group.tests.clone(), waiting, finding },
            );
        }
    }
    finish(run, finding)
}

/// Saves the judged tip, failures and bounded cumulative flake counts.
#[must_use]
pub fn judged(record: &Record, verdict: &Verdict, limits: &Limits) -> Record {
    let mut flakes = List::with_capacity(limits.failing_tests);
    let mut previous = record.flakes.iter().peekable();
    let mut room = limits.failing_tests.saturating_sub(length(record.flakes.len()));
    for test in &verdict.flakes {
        // Each old entry is visited once, and new entries enter in order.
        for _ in 0..record.flakes.len() {
            let Some(old) = previous.peek() else { break };
            if old.test >= *test {
                break;
            }
            let old = previous.next().expect("the peeked old entry");
            flakes.push(old.clone()).expect("old flakes fit the bound");
        }
        let same = match previous.peek() {
            Some(old) => old.test == *test,
            None => false,
        };
        if same {
            let old = previous.next().expect("the matching old entry");
            flakes.push(Flake { test: test.clone(), count: old.count.saturating_add(1) }).expect("old flakes fit");
        } else if room != 0 {
            flakes.push(Flake { test: test.clone(), count: 1 }).expect("new flakes have room");
            room = room.checked_sub(1).expect("room is nonzero");
        }
    }
    for old in previous {
        flakes.push(old.clone()).expect("old flakes fit the bound");
    }
    Record {
        sequence: verdict.number,
        judged: Some(Judged { tip: verdict.tip, key: verdict.key }),
        failing: verdict.failing.clone(),
        flakes: flakes.into_boxed(),
        run: None,
    }
}

/// Whether the judged run passed the failures blamed on a landing.
#[must_use]
pub fn repaired(blame: &Blame, failing: &Failures) -> bool {
    let blamed = match blame {
        Blame::Cleared => return true,
        Blame::Owns(failures) | Blame::Suspect(failures) => failures,
    };
    match failing {
        Failures::Many => false,
        Failures::Tests(tests) => match blamed {
            Failures::Many => true,
            Failures::Tests(blamed) => select(blamed, tests, true).is_empty(),
        },
    }
}
