//! Seeded deletion invariants over event sequences (qa/ledger.md, 4.6).
use super::{case, landing};
use temper_qa_tool::{BuildState, Exit, Ledger};

struct Random(u64);
impl Random {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }
}
fn run(seed: u64) -> (Ledger, usize) {
    let mut random = Random(seed);
    let mut ledger = Ledger::default();
    let mut now = 0;
    let mut candidates = 0;
    for step in 0..256 {
        let event = random.next() % 11;
        match event {
            0 => {
                let number = u64::try_from(ledger.landings.len()).expect("length") + 1;
                landing(&mut ledger, number);
            }
            1 => {
                if let Some(next) = ledger.next_landing().map(|landing| landing.number) {
                    let _ = ledger.start_session(next, "low", super::pages(), vec!["fresh".into()], now);
                }
            }
            2..=4 => {
                if let Some(number) = ledger.running_sessions().first().map(|session| session.number) {
                    if event == 2 {
                        let _ = ledger.answer_none(number, "internals");
                    }
                    if event == 3 {
                        let _ = ledger.add_case(number, &case(), now);
                    }
                    if event == 4 {
                        let exit = match random.next() % 4 {
                            0 => Exit::Unstarted,
                            1 => Exit::Lost,
                            2 => Exit::Bound,
                            _ => Exit::Code(0),
                        };
                        ledger.end_session(number, exit, None, now, "qa").expect("end");
                    }
                }
            }
            5 => {
                ledger.keep_build(&format!("B{step}"), now).expect("unique build");
            }
            6 => {
                let count = u64::try_from(ledger.landings.len()).expect("count");
                let length = random.next() % (count + 1);
                let commit = ledger.builds_without_holds().first().map(|build| build.commit.clone());
                if let Some(commit) = commit {
                    ledger.set_build_holds(&commit, (1..=length).collect(), length).expect("known landings");
                }
            }
            7 => {
                if !ledger.builds.is_empty() {
                    let index = usize::try_from(random.next()).expect("64-bit host") % ledger.builds.len();
                    ledger.builds[index].held = if random.next().is_multiple_of(2) { Some(1) } else { None };
                }
            }
            8 => {
                now += random.next() % 100;
            }
            9 => {
                ledger.waiting_lines(60, now, "qa");
            }
            10 => {
                let commit = ledger.deletable_builds().first().map(|build| build.commit.clone());
                if let Some(commit) = commit {
                    ledger.mark_build_deleted(&commit, now).expect("deletable");
                }
            }
            _ => unreachable!("eleven events"),
        }
        // The referee computes reachability and newest-held-case protection from records,
        // not by calling the predicates it checks.
        for build in ledger.deletable_builds() {
            candidates += 1;
            assert_eq!(build.state, BuildState::Kept, "seed {seed}, step {step}");
            assert_eq!(build.held, None, "seed {seed}, step {step}: held");
            let holds = build.holds.as_ref().expect("deletable has known holds");
            assert!(
                holds.iter().all(|number| ledger
                    .landings
                    .iter()
                    .any(|landing| landing.number == *number && landing.triaged.is_some())),
                "seed {seed}, step {step}: unreached"
            );
            let newest = ledger
                .builds
                .iter()
                .filter(|other| other.state == BuildState::Kept)
                .filter(|other| {
                    other.holds.as_ref().is_some_and(|holds| {
                        holds.iter().all(|number| {
                            ledger.landings.iter().any(|landing| landing.number == *number && landing.triaged.is_some())
                        })
                    })
                })
                .filter_map(|other| other.order)
                .max();
            if build.order == newest {
                assert!(
                    !ledger.cases.iter().flat_map(|case| &case.asks).any(|ask| holds.contains(&ask.landing)),
                    "seed {seed}, step {step}: newest reached case build"
                );
            }
        }
    }
    (ledger, candidates)
}
#[test]
fn deletion_invariants_replay_seeded_event_sequences() {
    let mut candidates = 0;
    for seed in 1..=32 {
        let first = run(seed);
        let second = run(seed);
        assert_eq!(first, second, "seed {seed} replays");
        candidates += first.1;
    }
    assert!(candidates > 0, "deletion was exercised");
}
