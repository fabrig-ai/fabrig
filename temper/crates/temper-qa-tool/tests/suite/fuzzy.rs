//! Seeded deletion invariants over event sequences (qa/ledger.md, 4.6).
use super::{case, landing};
use temper_qa_tool::{BuildState, CaseState, Exit, FindingState, Ledger, PassState};

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

#[expect(clippy::too_many_lines, reason = "one seeded loop shows the nine events and checks its records after each")]
fn pass_sequence(seed: u64) -> (Ledger, [u64; 9]) {
    let mut random = Random(seed);
    let mut ledger = Ledger::default();
    let mut covered = [0; 9];
    for step in 0..512_u64 {
        let event = random.next() % 9;
        let place = if random.next().is_multiple_of(2) { "places/1" } else { "places/2" };
        let running: Vec<_> = ledger
            .passes
            .iter()
            .filter(|pass| pass.state == PassState::Running)
            .filter_map(|pass| pass.sessions.last().copied())
            .collect();
        let session = if running.is_empty() {
            None
        } else {
            running.get(usize::try_from(random.next()).expect("host") % running.len()).copied()
        };
        let owner = session
            .and_then(|session| ledger.sessions.iter().find(|record| record.number == session))
            .and_then(|record| record.pass);
        let assigned: Vec<_> = ledger
            .cases
            .iter()
            .filter(|case| owner.is_some() && case.state == CaseState::Assigned && case.pass == owner)
            .map(|case| case.number)
            .collect();
        let case = if assigned.is_empty() {
            None
        } else {
            assigned.get(usize::try_from(random.next()).expect("host") % assigned.len()).copied()
        };
        match event {
            0 => {
                let (page, surface) = match random.next() % 3 {
                    0 => ("work", "web"),
                    1 => ("home", "web"),
                    _ => ("chat", "tui"),
                };
                super::passes::queue(&mut ledger, page, surface);
                covered[0] += 1;
            }
            1 => {
                let count = u64::try_from(ledger.landings.len()).expect("length");
                let held = random.next() % (count + 1);
                super::passes::build(&mut ledger, &format!("B{step}"), &(1..=held).collect::<Vec<_>>(), step);
                covered[1] += 1;
            }
            2 | 7 => {
                let started = if event == 2 {
                    ledger.start_pass(place, &["web".into(), "tui".into()], "high", step)
                } else {
                    ledger.resume_pass(place, &["web".into(), "tui".into()], "high", step)
                };
                if let Some(started) = started {
                    covered[usize::try_from(event).expect("event")] += 1;
                    // Spawn failure is detected before the loop schedules anything else.
                    if random.next().is_multiple_of(4) {
                        ledger = Ledger::decode(&ledger.encode().expect("persist start")).expect("restart");
                        ledger.end_pass_session(started.session, Exit::Unstarted, None, step, "qa", &[]).expect("undo");
                        covered[8] += 1;
                    }
                }
            }
            3 => {
                if let (Some(session), Some(case)) = (session, case) {
                    let mut input = super::passes::finding(case);
                    if random.next().is_multiple_of(2) {
                        input.lines = vec![format!("drive case {case}")];
                        input.expected = format!("case {case} works");
                    }
                    ledger.record_finding(session, &input, &[], "product", step).expect("finding");
                    covered[3] += 1;
                }
            }
            4 => {
                if let (Some(session), Some(case)) = (session, case) {
                    let pass = ledger.pass_command_session(session).expect("session").pass;
                    let findings: Vec<_> = ledger
                        .findings
                        .iter()
                        .filter(|finding| finding.case == case && Some(finding.pass) == pass)
                        .map(|finding| finding.number)
                        .collect();
                    let result = match random.next() % 3 {
                        0 => "passed",
                        1 => "blocked",
                        _ => {
                            if findings.is_empty() {
                                "blocked"
                            } else {
                                "failed"
                            }
                        }
                    };
                    ledger.record_outcome(session, case, result, "observation", &findings, step).expect("outcome");
                    covered[4] += 1;
                }
            }
            5 => {
                if let Some(session) = session {
                    let pass = ledger.pass_command_session(session).expect("session").pass;
                    let finding = ledger
                        .findings
                        .iter()
                        .find(|finding| Some(finding.pass) == pass && finding.state == FindingState::Open)
                        .map(|finding| finding.number);
                    if let Some(finding) = finding {
                        let _ = ledger.record_check(
                            session,
                            finding,
                            if random.next().is_multiple_of(2) { "gone" } else { "holds" },
                            "check",
                            step,
                        );
                    }
                    ledger.end_pass_session(session, Exit::Lost, None, step, "qa", &[]).expect("end");
                    covered[5] += 1;
                }
            }
            6 => {
                let request = ledger.passes.iter().find_map(|pass| {
                    pass.requests
                        .iter()
                        .find(|request| request.filed.is_none())
                        .map(|request| (pass.number, request.number))
                });
                if let Some((pass, request)) = request {
                    let goal = 1000 + step;
                    ledger.file_request(pass, request, Some(goal), step).expect("file");
                    if random.next().is_multiple_of(2) {
                        ledger.end_goal(goal, step).expect("ended goal");
                    } else {
                        let number = ledger
                            .record_landing(&format!("fix-{step}"), goal, None, "fix", step)
                            .expect("goal landing");
                        let session = super::session(&mut ledger, number);
                        ledger.answer_none(session, "fix").expect("none");
                        ledger.end_session(session, Exit::Code(0), None, step, "qa").expect("end triage");
                    }
                    covered[6] += 1;
                }
            }
            8 => {
                let build = ledger.deletable_builds().first().map(|build| build.commit.clone());
                if let Some(build) = build {
                    ledger.mark_build_deleted(&build, step).expect("delete");
                }
            }
            _ => unreachable!("nine events"),
        }
        for case in ledger.cases.iter().filter(|case| case.state == CaseState::Assigned) {
            let pass = ledger.passes.iter().find(|pass| Some(pass.number) == case.pass).expect("assigned owner");
            assert!(pass.cases.contains(&case.number), "seed {seed}, step {step}");
            assert!(
                pass.state == PassState::Running
                    || (pass.state == PassState::Waiting
                        && ledger.findings.iter().any(|finding| finding.pass == pass.number
                            && finding.state == FindingState::Open
                            && finding.surface == case.surface
                            && finding.page == case.page
                            && case.asks.iter().any(|ask| ask.setup == finding.setup))),
                "seed {seed}, step {step}: assigned outside running or joined waiting pass"
            );
            let takers: Vec<_> = ledger
                .sessions
                .iter()
                .filter(|session| session.ended.is_none())
                .filter_map(|session| session.before.as_ref().map(|before| (session.pass, before)))
                .filter(|(_, before)| before.cases.iter().any(|saved| saved.number == case.number))
                .collect();
            assert_eq!(
                takers.len(),
                usize::from(pass.state == PassState::Running),
                "seed {seed}, step {step}: two active assignments"
            );
            assert!(
                takers.iter().all(|(owner, _)| *owner == case.pass),
                "seed {seed}, step {step}: assignment stolen from another live session"
            );
        }
        for build in &ledger.builds {
            if build.held.is_some() {
                assert_eq!(build.state, BuildState::Kept, "seed {seed}, step {step}");
                assert!(
                    !ledger.deletable_builds().iter().any(|candidate| candidate.commit == build.commit),
                    "seed {seed}, step {step}: held deletable"
                );
            }
        }
    }
    (ledger, covered)
}
#[test]
fn pass_invariants_replay_queue_build_record_end_file_resume_and_unstarted_sequences() {
    let mut coverage = [0; 9];
    for seed in 1..=32 {
        let first = pass_sequence(seed);
        let second = pass_sequence(seed);
        assert_eq!(first, second, "seed {seed} replay");
        for (total, count) in coverage.iter_mut().zip(first.1) {
            *total += count;
        }
    }
    assert!(coverage.iter().all(|count| *count > 0), "every operation and unstarted end exercised: {coverage:?}");
}
