use super::*;
use temper_engine_domain_forge_landing as policy;

#[test]
fn project_landing_keeps_repository_tiers_and_settings_separate_by_project() {
    use temper_engine_domain_forge_change::Way;
    use temper_engine_domain_forge_tiers::{Format, Settings, Suspects};
    let mut d = domain();
    let tier = Tier {
        name: Box::from(b"slow".as_slice()),
        workflow: Box::from(b"slow.yml".as_slice()),
        ignored: Box::new([Box::from(b"docs".as_slice())]),
        artifact: Box::from(b"failures".as_slice()),
        format: Format::Junit,
        follows: Follows::Branch(Box::from(b"release/next".as_slice())),
        span: None,
        description: Box::default(),
    };
    let repositories = Box::new([
        RepositoryGates {
            repository: REPO,
            gates: Box::new([policy::Gate {
                name: Box::from(b"gate".as_slice()),
                workflow: Box::from(b"gate.yml".as_slice()),
                ignored: Box::new([]),
                blocking: true,
                base: false,
            }]),
            tiers: Box::new([tier.clone()]),
        },
        RepositoryGates {
            repository: skein_forge::api::Repository { repository: 3, ..REPO },
            gates: Box::new([]),
            tiers: Box::new([Tier { follows: Follows::Landing, format: Format::Lines, ..tier }]),
        },
    ]);
    let after = AfterLanding {
        verdict: Settings { retries: 7, suspects: Suspects::Bisect },
        bound: Duration::from_millis(123_456),
    };
    assert_eq!(d.after_landing(5), None);
    let settings = policy::Project { gate_retries: 7, gate_timeout: Duration::from_secs(90) };
    assert!(d.project_landing(5, Way::Turn, repositories.clone(), settings, after));
    assert_eq!(d.landing.projects.get(&5).expect("configured project").5, settings);
    let other = AfterLanding {
        verdict: Settings { retries: 0, suspects: Suspects::Bisect },
        bound: Duration::from_millis(456_789),
    };
    assert!(d.project_landing(
        6,
        Way::Queue,
        Box::new([]),
        temper_engine_domain_forge_landing::Project::DEFAULT,
        other
    ));
    let (way, kept, _, _, _, _) = d.landing.projects.get(&5).expect("configured project");
    assert_eq!(*way, Way::Turn);
    assert_eq!(kept.as_ref(), repositories.as_ref());
    assert_eq!(d.landing.gates(5, REPO), repositories[0].gates.as_ref());
    assert_eq!(d.after_landing(5), Some(after));
    assert_eq!(d.after_landing(6), Some(other));
    assert_eq!(d.after_landing(7), None);
    assert!(d.project_landing(
        5,
        Way::Queue,
        Box::new([]),
        temper_engine_domain_forge_landing::Project::DEFAULT,
        other
    ));
    assert_eq!(d.after_landing(5), Some(other));
    assert_eq!(d.after_landing(6), Some(other));
    assert!(d.landing.gates(5, REPO).is_empty());
}

/// Merge commands belong to configured projects (temper/docs/design/protocol/configuration/engine.md).
#[test]
fn a_projects_merge_job_is_kept_per_project() {
    use temper_engine_domain_forge_change::Way;
    use temper_engine_domain_forge_tiers::{Settings, Suspects};
    const PROJECT: u32 = 5;
    const OTHER: u32 = 6;
    const UNCONFIGURED: u32 = 7;
    const COMMAND: &[u8] = b".temper/merge";
    const OTHER_COMMAND: &[u8] = b"scripts/merge";
    let after = AfterLanding {
        verdict: Settings { retries: 2, suspects: Suspects::Bisect },
        bound: Duration::from_millis(7_200_000),
    };
    for way in [Way::Queue, Way::Turn] {
        let mut d = domain();
        assert_eq!(d.merge_job(PROJECT), None);
        assert!(!d.project_merge_job(PROJECT, COMMAND.into()));
        assert!(d.landing.projects.is_empty());
        assert!(d.project_landing(
            PROJECT,
            way,
            Box::new([]),
            temper_engine_domain_forge_landing::Project::DEFAULT,
            after
        ));
        assert!(d.project_landing(
            OTHER,
            way,
            Box::new([]),
            temper_engine_domain_forge_landing::Project::DEFAULT,
            after
        ));
        assert_eq!(d.landing.projects.len(), LIMITS.judge_projects);
        assert!(d.project_merge_job(PROJECT, COMMAND.into()));
        assert_eq!(d.merge_job(PROJECT), Some(COMMAND));
        assert_eq!(d.merge_job(OTHER), None);
        assert!(!d.project_landing(
            UNCONFIGURED,
            way,
            Box::new([]),
            temper_engine_domain_forge_landing::Project::DEFAULT,
            after
        ));
        assert!(!d.project_merge_job(UNCONFIGURED, COMMAND.into()));
        assert_eq!(d.merge_job(UNCONFIGURED), None);
        assert_eq!(d.landing.projects.len(), LIMITS.judge_projects);
        assert!(d.project_merge_job(OTHER, OTHER_COMMAND.into()));
        assert_eq!(d.merge_job(PROJECT), Some(COMMAND));
        assert_eq!(d.merge_job(OTHER), Some(OTHER_COMMAND));
        assert!(d.project_merge_job(PROJECT, OTHER_COMMAND.into()));
        assert_eq!(d.merge_job(PROJECT), Some(OTHER_COMMAND));
        assert!(d.project_landing(
            PROJECT,
            way,
            Box::new([]),
            temper_engine_domain_forge_landing::Project::DEFAULT,
            after
        ));
        assert_eq!(d.merge_job(PROJECT), None);
        assert_eq!(d.merge_job(OTHER), Some(OTHER_COMMAND));
    }
}

/// Close commands belong to configured projects (temper/docs/design/protocol/configuration/engine.md).
#[test]
fn a_projects_close_job_is_kept_per_project() {
    use temper_engine_domain_forge_change::Way;
    use temper_engine_domain_forge_tiers::{Settings, Suspects};
    const PROJECT: u32 = 5;
    const OTHER: u32 = 6;
    const UNCONFIGURED: u32 = 7;
    const COMMAND: &[u8] = b".temper/close";
    const OTHER_COMMAND: &[u8] = b"scripts/close";
    let after = AfterLanding {
        verdict: Settings { retries: 2, suspects: Suspects::Bisect },
        bound: Duration::from_millis(7_200_000),
    };
    for way in [Way::Queue, Way::Turn] {
        let mut d = domain();
        assert_eq!(d.close_job(PROJECT), None);
        assert!(!d.project_close_job(PROJECT, COMMAND.into()));
        assert!(d.landing.projects.is_empty());
        assert!(d.project_landing(
            PROJECT,
            way,
            Box::new([]),
            temper_engine_domain_forge_landing::Project::DEFAULT,
            after
        ));
        assert!(d.project_landing(
            OTHER,
            way,
            Box::new([]),
            temper_engine_domain_forge_landing::Project::DEFAULT,
            after
        ));
        assert_eq!(d.landing.projects.len(), LIMITS.judge_projects);
        assert!(d.project_close_job(PROJECT, COMMAND.into()));
        assert_eq!(d.close_job(PROJECT), Some(COMMAND));
        assert_eq!(d.close_job(OTHER), None);
        assert!(!d.project_landing(
            UNCONFIGURED,
            way,
            Box::new([]),
            temper_engine_domain_forge_landing::Project::DEFAULT,
            after
        ));
        assert!(!d.project_close_job(UNCONFIGURED, COMMAND.into()));
        assert_eq!(d.close_job(UNCONFIGURED), None);
        assert_eq!(d.landing.projects.len(), LIMITS.judge_projects);
        assert!(d.project_close_job(OTHER, OTHER_COMMAND.into()));
        assert_eq!(d.close_job(PROJECT), Some(COMMAND));
        assert_eq!(d.close_job(OTHER), Some(OTHER_COMMAND));
        assert!(d.project_close_job(PROJECT, OTHER_COMMAND.into()));
        assert_eq!(d.close_job(PROJECT), Some(OTHER_COMMAND));
        assert!(d.project_landing(
            PROJECT,
            way,
            Box::new([]),
            temper_engine_domain_forge_landing::Project::DEFAULT,
            after
        ));
        assert_eq!(d.close_job(PROJECT), None);
        assert_eq!(d.close_job(OTHER), Some(OTHER_COMMAND));
    }
}

#[test]
fn project_landing_refuses_tiers_past_the_bound_and_duplicate_check_names() {
    use temper_engine_domain_forge_change::Way;
    use temper_engine_domain_forge_tiers::{Format, Settings, Suspects};
    let tier = Tier {
        name: Box::from(b"slow".as_slice()),
        workflow: Box::from(b"slow.yml".as_slice()),
        ignored: Box::new([]),
        artifact: Box::from(b"failures".as_slice()),
        format: Format::Lines,
        follows: Follows::Landing,
        span: None,
        description: Box::default(),
    };
    let after = AfterLanding {
        verdict: Settings { retries: 2, suspects: Suspects::Bisect },
        bound: Duration::from_millis(7_200_000),
    };
    for (names, expected) in [(b"ab".as_slice(), true), (b"abc", false), (b"aa", false), (b"g", false)] {
        let mut d = domain();
        let mut tiers = skein_lib::List::with_capacity(u32::try_from(names.len()).expect("test count"));
        for name in names {
            tiers.push(Tier { name: Box::new([*name]), ..tier.clone() }).expect("test count");
        }
        assert_eq!(
            d.project_landing(
                5,
                Way::Queue,
                Box::new([RepositoryGates {
                    repository: REPO,
                    gates: Box::new([policy::Gate {
                        name: Box::from(b"g".as_slice()),
                        workflow: Box::from(b"gate.yml".as_slice()),
                        ignored: Box::new([]),
                        blocking: true,
                        base: false,
                    }]),
                    tiers: tiers.into_boxed(),
                }]),
                temper_engine_domain_forge_landing::Project::DEFAULT,
                after
            ),
            expected
        );
        assert_eq!(d.after_landing(5), if expected { Some(after) } else { None });
    }
}

#[test]
fn tier_limits_are_nonzero_and_failure_lists_fit_the_store_row() {
    for (tiers, expected) in [
        (TierLimits { tiers: 0, ..LIMITS.tiers }, false),
        (TierLimits { failing_tests: 0, ..LIMITS.tiers }, false),
        (TierLimits { test_id_bytes: 0, ..LIMITS.tiers }, false),
        (TierLimits { tiers: 1, failing_tests: 1024, test_id_bytes: 1 }, true),
        (TierLimits { failing_tests: 1025, ..LIMITS.tiers }, false),
    ] {
        assert_eq!(valid(&Limits { tiers, ..LIMITS }), expected, "{tiers:?}");
    }
}

fn landing_fixture() -> Domain {
    let mut d = domain();
    outputs(&mut d, Event::Restore { record: Stored::Repository(adopted()) });
    let mut row = fixtures::opening_row();
    row.repository = REPO;
    row.pull = Some(1);
    row.change.landing = None;
    outputs(&mut d, Event::Restore { record: Stored::Change(Box::new(row)) });
    assert!(d.project_landing(
        5,
        temper_engine_domain_forge_change::Way::Turn,
        Box::new([]),
        temper_engine_domain_forge_landing::Project::DEFAULT,
        AfterLanding {
            verdict: temper_engine_domain_forge_tiers::Settings {
                retries: 2,
                suspects: temper_engine_domain_forge_tiers::Suspects::Bisect
            },
            bound: Duration::from_millis(7_200_000)
        }
    ));
    d
}
fn landing_row(task: u64, phase: policy::Phase) -> LandingRow {
    LandingRow {
        landing: policy::Landing {
            round: temper_engine_domain_forge_landing::Round::new(Box::from(b"temper/1/l61".as_slice())),

            task,
            project: 5,
            phase,
            members: Box::new([]),
            preparation: 0,
            stale_proofs: 0,
            branch: Box::from(b"main".as_slice()),
        },
        asks: Box::new([]),
        entries: Box::new([]),
    }
}
#[test]
fn landing_preflight_checks_rules_and_live_room() {
    let mut d = landing_fixture();
    assert_eq!(d.landing_admits(6, &[fixtures::CHANGE]), Err(LandingRefusal::Way));
    assert_eq!(d.landing_admits(5, &[]), Err(LandingRefusal::Members));
    assert_eq!(d.landing_admits(5, &[fixtures::CHANGE; 3]), Err(LandingRefusal::Members));
    assert_eq!(d.landing_admits(5, &[999]), Err(LandingRefusal::Repository));
    assert_eq!(d.landing_admits(5, &[fixtures::CHANGE; 2]), Err(LandingRefusal::Members));
    assert_eq!(d.landing_admits(5, &[fixtures::CHANGE]), Ok(()));
    for task in 100..108 {
        d.landing.rows.insert(task, landing_row(task, policy::Phase::Preparing)).expect("room");
    }
    assert_eq!(d.landing_admits(5, &[fixtures::CHANGE]), Err(LandingRefusal::Landings));
    d.landing.rows.get_mut(&100).expect("landing").landing.phase = policy::Phase::Landed;
    assert_eq!(d.landing_admits(5, &[fixtures::CHANGE]), Ok(()));
    d.landing.rows.get_mut(&100).expect("landing").entries =
        Box::new([LandingEntry { entry: 1, member: 0, gate: 0, operation: Operation::Merge }]);
    assert_eq!(d.landing_admits(5, &[fixtures::CHANGE]), Err(LandingRefusal::Landings));
}

#[test]
fn landing_candidate_uses_reverse_ancestry_and_waits_on_unknown() {
    use skein_forge::api::{Answer, Op, Read};
    const CANDIDATE: [u8; 32] = [9; 32];
    for complete in [true, false] {
        let mut d = landing_fixture();
        fixtures::connect(&mut d);
        outputs(&mut d, Event::Restored { clock: skein_forge_client::RecoveryClock::Monotonic });
        outputs(&mut d, Event::ReadAfresh);
        outputs(&mut d, Event::SettleOutbox);
        d.changes.get_mut(&fixtures::CHANGE).expect("change").change.state =
            temper_engine_domain_forge_change::State::Ready { head: fixtures::HEAD, ready_since: env().wall };
        member_landing(&mut d);
        let row = d.landing.rows.get_mut(&61).expect("round");
        row.landing.phase = policy::Phase::Preparing;
        row.landing.round.pull = None;
        row.landing.round.combined = None;
        let member = &mut row.landing.members[0];
        member.candidate = None;
        member.head = None;
        member.base = None;
        outputs(
            &mut d,
            Event::StepLanding {
                task: 61,
                entry: 91,
                heard: LandingHeard {
                    failure: Box::new([]),
                    preparation: policy::Preparation::Idle,
                    released: false,
                    cancelled: false,
                    kept: Box::new([]),
                },
            },
        );
        fixtures::answer_next(&mut d, Ok(Answer::Commit(fixtures::HEAD)));
        fixtures::answer_next(&mut d, Ok(Answer::Commit(fixtures::BASE)));
        fixtures::answer_next(&mut d, Ok(Answer::Commits { commits: Box::new([]), more: false }));
        fixtures::answer_next(&mut d, Ok(Answer::Pull(fixtures::open_pull())));
        fixtures::answer_next(&mut d, Ok(Answer::Reviews { reviews: Box::new([]), more: false, cut: false }));
        let (call, op) = fixtures::next_call(&mut d);
        assert_eq!(op, Op::Read(Read::Branch { branch: b"temper/1/l61".as_slice().into() }));
        fixtures::answer(&mut d, call, Ok(Answer::Commit(CANDIDATE)));
        let (call, op) = fixtures::next_call(&mut d);
        assert_eq!(op, Op::Read(Read::Commits { before: CANDIDATE, after: fixtures::BASE, page: 1 }));
        fixtures::answer(&mut d, call, Ok(Answer::Commits { commits: Box::new([]), more: false }));
        let (call, op) = fixtures::next_call(&mut d);
        assert_eq!(op, Op::Read(Read::Commits { before: CANDIDATE, after: fixtures::HEAD, page: 1 }));
        let mut requests =
            fixtures::answer(&mut d, call, Ok(Answer::Commits { commits: Box::new([]), more: !complete }));
        if complete {
            fixtures::answer_next(&mut d, Ok(Answer::Commits { commits: Box::new([]), more: false }));
            fixtures::answer_next(
                &mut d,
                Ok(Answer::Commits {
                    commits: Box::new([skein_forge::api::CommitMessage {
                        id: fixtures::HEAD,
                        parents: Box::new([]),
                        message: Box::new([]),
                    }]),
                    more: false,
                }),
            );
            requests = fixtures::answer_next(&mut d, Err(skein_forge::api::Error::Missing));
        }
        let mut decision = None;
        for request in requests {
            if let Request::LandingDecision { decision: found, .. } = request {
                decision = Some(found);
            }
        }
        let decision = decision.expect("decision");
        assert!(if complete {
            decision == policy::Decision::Open
        } else {
            matches!(decision, policy::Decision::Wait { .. })
        });
    }
}

#[test]
fn landing_gather_reads_each_branch_and_pull_once() {
    skein_rows::check(
        [
            ("shared base", (false, false)),
            ("shared head and pull", (true, false)),
            ("same names in another repository", (true, true)),
        ],
        landing_shared_gather_row,
    );
}
fn landing_shared_gather_row((same, other_repository): (bool, bool)) {
    use skein_forge::api::{Answer, Op, Read};
    let mut d = landing_fixture();
    outputs(&mut d, Event::Restored { clock: skein_forge_client::RecoveryClock::Monotonic });
    outputs(&mut d, Event::ReadAfresh);
    outputs(&mut d, Event::SettleOutbox);
    d.changes.get_mut(&fixtures::CHANGE).expect("change").change.state =
        temper_engine_domain_forge_change::State::Ready { head: fixtures::HEAD, ready_since: env().wall };
    member_landing(&mut d);
    let mut change = d.changes.get(&fixtures::CHANGE).expect("first change").clone();
    change.task = 52;
    change.change.task = 52;
    if !same {
        change.branch = b"temper/52".as_slice().into();
        change.pull = Some(2);
    }
    if other_repository {
        change.repository.repository = 3;
    }
    let repo = change.repository;
    d.changes.insert(52, change).expect("second member");
    // A stack's gather can name one repository more than once (landing.md, 5.3).
    let row = d.landing.rows.get_mut(&61).expect("landing");
    let first = row.landing.members.first().expect("first member").clone();
    let mut second = first.clone();
    second.change = 52;
    second.repository.repository = repo.repository;
    if !same {
        second.pull = 2;
    }
    row.landing.members = Box::new([first, second]);
    row.asks = Box::new([None, None]);
    outputs(
        &mut d,
        Event::StepLanding {
            task: 61,
            entry: 91,
            heard: LandingHeard {
                failure: Box::new([]),
                preparation: temper_engine_domain_forge_landing::Preparation::Idle,
                released: false,
                cancelled: false,
                kept: Box::new([]),
            },
        },
    );
    let mut branches = 0_u32;
    let mut bases = 0_u32;
    let mut pulls = 0_u32;
    let mut ended = false;
    for _ in 0..10_u32 {
        let (call, op) = fixtures::next_call(&mut d);
        let answer = match op {
            Op::Read(Read::Branch { branch }) => {
                branches = branches.checked_add(1).expect("bounded reads");
                if branch.as_ref() == b"main" {
                    bases = bases.checked_add(1).expect("bounded reads");
                    Answer::Commit(fixtures::BASE)
                } else {
                    Answer::Commit(fixtures::HEAD)
                }
            }
            Op::Read(Read::Pull { number, ci: false }) => {
                pulls = pulls.checked_add(1).expect("bounded reads");
                let mut pull = fixtures::open_pull();
                pull.number = number;
                if number == 2 {
                    pull.head = b"temper/52".as_slice().into();
                }
                Answer::Pull(pull)
            }
            Op::Read(Read::Commits { .. }) => Answer::Commits { commits: Box::new([]), more: false },
            Op::Read(Read::Reviews { .. }) => Answer::Reviews { reviews: Box::new([]), more: false, cut: false },
            Op::Read(_) | Op::Write(_) => panic!("unexpected gather read {op:?}"),
        };
        for request in fixtures::answer(&mut d, call, Ok(answer)) {
            if let Request::LandingDecision { .. } = request {
                ended = true;
            }
        }
        if ended {
            break;
        }
    }
    assert!(ended, "gather completed");
    assert_eq!(bases, if other_repository { 2 } else { 1 }, "one base read per repository");
    assert_eq!(
        branches,
        if other_repository {
            4
        } else if same {
            2
        } else {
            3
        }
    );
    assert_eq!(pulls, if same && !other_repository { 1 } else { 2 });
}

#[test]
fn landing_merge_carries_coherent_evidence_and_veto_withdraws_it() {
    use skein_forge::api::{Answer, Op, Read};
    use temper_engine_domain_forge_change as change;
    let mut d = landing_fixture();
    outputs(&mut d, Event::Restored { clock: skein_forge_client::RecoveryClock::Monotonic });
    outputs(&mut d, Event::ReadAfresh);
    outputs(&mut d, Event::SettleOutbox);
    d.changes.get_mut(&fixtures::CHANGE).expect("change").change.state =
        change::State::Ready { head: fixtures::HEAD, ready_since: env().wall };
    let mut row = landing_row(61, policy::Phase::Turn { at: env().wall });
    row.landing.members = Box::new([policy::Member {
        change: fixtures::CHANGE,
        repository: policy::Repository { forge: REPO.forge, repository: REPO.repository },
        pull: 1,
        head: Some(fixtures::HEAD),
        base: Some(fixtures::BASE),
        candidate: Some(fixtures::HEAD),
        gates: Box::new([]),
        asked: None,
        landed: None,
    }]);
    row.asks = Box::new([None]);
    row.landing.round.pull = Some(2);
    row.landing.round.combined = Some(policy::Combine::Merge);
    outputs(&mut d, Event::Landing { row });
    outputs(
        &mut d,
        Event::StepLanding {
            task: 61,
            entry: 91,
            heard: LandingHeard {
                failure: Box::new([]),
                preparation: temper_engine_domain_forge_landing::Preparation::Idle,
                released: false,
                cancelled: false,
                kept: Box::new([]),
            },
        },
    );
    let (branch_call, op) = fixtures::next_call(&mut d);
    assert_eq!(op, Op::Read(Read::Branch { branch: b"temper/51".as_slice().into() }));
    outputs(&mut d, Event::Enqueue { entry: status(92, 8) });
    outputs(&mut d, Event::Committed { entry: 92 });
    assert!(ready(&mut d).is_empty(), "the higher entry waits while the landing's lower entry is reserved");
    fixtures::answer(&mut d, branch_call, Ok(Answer::Commit(fixtures::HEAD)));
    fixtures::answer_next(&mut d, Ok(Answer::Commit(fixtures::BASE)));
    fixtures::answer_next(&mut d, Ok(Answer::Pull(fixtures::open_pull())));
    let (call, op) = fixtures::next_call(&mut d);
    assert_eq!(op, Op::Read(Read::Reviews { number: 1, page: 1, skip: 0 }));
    let requests =
        fixtures::answer(&mut d, call, Ok(Answer::Reviews { reviews: Box::new([]), more: false, cut: false }));
    assert!(ready(&mut d).is_empty(), "the higher entry waits for the landing's lower reserved entry");
    outputs(&mut d, Event::Withdraw { entry: 92 });
    let mut evidence = None;
    for request in &requests {
        if let Request::LandingDecision { evidence: Some(facts), entry: Some(91), .. } = request {
            evidence = Some(facts);
        }
    }
    let evidence = evidence.expect("merge evidence");
    assert_eq!(evidence.change, fixtures::CHANGE);
    assert_eq!(evidence.facts.head, Some(fixtures::HEAD));
    assert_eq!(evidence.facts.base_tip, fixtures::BASE);
    assert_eq!(evidence.facts.contains_base, change::Status::Passed);
    assert_eq!(evidence.facts.ci, change::Status::Passed);
    assert!(evidence.facts.reviews_complete);
    let (repository, _, described) = d.describe_landing_effect(61, 91).expect("saved merge description");
    assert_eq!(repository, REPO);
    assert_eq!(described.kind, 4);
    assert_eq!(described.state, fixtures::HEAD);
    let vetoed = outputs(&mut d, Event::VetoLanding { task: 61, entry: 91 });
    assert!(vetoed.contains(&Request::Erase { key: Key::Entry(91) }));
    assert!(d.describe_landing_effect(61, 91).is_none());
    assert_eq!(d.landing.results.get(&(61, 0, 0, 5)), Some(&policy::Status::Failed));
    outputs(&mut d, Event::Committed { entry: 91 });
    for request in ready(&mut d) {
        if let Request::Call { op: Op::Write(_), .. } = request {
            panic!("vetoed write reached client");
        }
    }
}

#[test]
fn landing_deadline_wakes_once_and_an_offered_step_clears_it() {
    let mut d = landing_fixture();
    outputs(&mut d, Event::Restored { clock: skein_forge_client::RecoveryClock::Monotonic });
    outputs(&mut d, Event::ReadAfresh);
    outputs(&mut d, Event::SettleOutbox);
    d.wait_landing(&env(), 61, Some(env().wall));
    assert_eq!(d.next_wall_deadline(), Some(env().wall));
    let mut out = Queue::uncapped();
    fire(&mut d, &env(), &mut out);
    let mut wakes = 0_u32;
    while let Some(request) = out.pop() {
        if request == (Request::WakeProcedure { task: 61 }) {
            wakes = wakes.checked_add(1).expect("bounded wake count");
        }
    }
    assert_eq!(wakes, 1);
    assert_eq!(d.next_wall_deadline(), None);
    d.wait_landing(&env(), 61, None);
    d.offered_landing(61);
    assert_eq!(d.next_wall_deadline(), None);
}

#[test]
fn landing_waits_use_the_repositories_hearing_deadline() {
    skein_rows::check(
        [
            ("quiet", (false, 60, 0, false, 5)),
            ("hearing", (true, 60, 0, false, 60)),
            ("poll is later", (true, 60, 58, false, 5)),
            ("expired", (true, 60, 60, false, 5)),
            ("disabled", (true, 0, 0, false, 5)),
            ("another repository", (true, 60, 0, true, 5)),
        ],
        landing_hearing_row,
    );
}
fn landing_hearing_row((hint, fallback, elapsed, other, delay): (bool, u64, u64, bool, u64)) {
    let mut d = landing_fixture();
    outputs(&mut d, Event::Restored { clock: skein_forge_client::RecoveryClock::Monotonic });
    outputs(&mut d, Event::ReadAfresh);
    outputs(&mut d, Event::SettleOutbox);
    member_landing(&mut d);
    d.offered_change(fixtures::CHANGE);
    let mut clock = env();
    clock.limits.client.fallback = Duration::from_secs(fallback);
    if hint {
        fixtures::repository_heard(
            &mut d,
            &clock,
            if other { skein_forge::api::Repository { repository: 3, ..REPO } } else { REPO },
        );
    }
    clock.now = clock.now.saturating_add(Duration::from_secs(elapsed));
    clock.wall = Wall::from_nanos(clock.wall.as_nanos().saturating_add(Duration::from_secs(elapsed).as_nanos()));
    let expected = Wall::from_nanos(clock.wall.as_nanos().saturating_add(Duration::from_secs(delay).as_nanos()));
    d.wait_landing(&clock, 61, None);
    assert_eq!(d.next_wall_deadline(), Some(expected), "default wait");
    d.offered_landing(61);
    d.wait_landing(&clock, 61, Some(clock.wall));
    assert_eq!(d.next_wall_deadline(), Some(clock.wall), "explicit deadlines are not stretched");
    d.offered_landing(61);
    d.changes.get_mut(&fixtures::CHANGE).expect("change").pull = None;
    let mut out = Queue::uncapped();
    step(
        &mut d,
        &clock,
        Event::StepLanding {
            task: 61,
            entry: 91,
            heard: LandingHeard {
                failure: Box::new([]),
                preparation: temper_engine_domain_forge_landing::Preparation::Idle,
                released: false,
                cancelled: false,
                kept: Box::new([]),
            },
        },
        &mut out,
    );
    let mut until = None;
    while let Some(request) = out.pop() {
        if let Request::LandingDecision { decision: policy::Decision::Wait { until: found }, .. } = request {
            until = found;
        }
    }
    assert_eq!(until, Some(expected), "missing fact wait");
}

fn member_landing(d: &mut Domain) {
    member_landing_with_gates(d, Box::new([]));
}

fn member_landing_with_gates(d: &mut Domain, gates: Box<[policy::GateProof]>) {
    let mut row = landing_row(61, policy::Phase::Proving);
    row.landing.members = Box::new([policy::Member {
        change: fixtures::CHANGE,
        repository: policy::Repository { forge: REPO.forge, repository: REPO.repository },
        pull: 1,
        head: Some(fixtures::HEAD),
        base: Some(fixtures::BASE),
        candidate: Some(fixtures::HEAD),
        gates,
        asked: None,
        landed: None,
    }]);
    row.asks = Box::new([None]);
    row.landing.round.pull = Some(2);
    row.landing.round.combined = Some(policy::Combine::Merge);
    outputs(d, Event::Landing { row });
}

fn merged_owner(turn: bool) -> (Domain, Box<[Request]>) {
    use skein_forge::api::{Answer, Op, Read, Write};
    let mut d = domain();
    outputs(&mut d, Event::Restore { record: Stored::Repository(adopted()) });
    outputs(&mut d, Event::Restored { clock: skein_forge_client::RecoveryClock::Monotonic });
    outputs(&mut d, Event::ReadAfresh);
    outputs(&mut d, Event::SettleOutbox);
    let mut change = fixtures::opening_row();
    change.pull = Some(1);
    change.pending = None;
    change.change.way =
        if turn { temper_engine_domain_forge_change::Way::Turn } else { temper_engine_domain_forge_change::Way::Queue };
    d.changes.insert(fixtures::CHANGE, change).expect("member change");
    let task = if turn { 61 } else { fixtures::CHANGE };
    if turn {
        assert!(d.project_landing(
            5,
            temper_engine_domain_forge_change::Way::Turn,
            Box::new([]),
            temper_engine_domain_forge_landing::Project::DEFAULT,
            AfterLanding {
                verdict: temper_engine_domain_forge_tiers::Settings {
                    retries: 2,
                    suspects: temper_engine_domain_forge_tiers::Suspects::Bisect
                },
                bound: Duration::from_millis(7_200_000)
            }
        ));
        member_landing(&mut d);
        let row = d.landing.rows.get_mut(&task).expect("landing");
        row.landing.phase = policy::Phase::Turn { at: env().wall };
        let member = row.landing.members.get_mut(0).expect("member");
        member.head = Some(fixtures::HEAD);
        member.base = Some(fixtures::BASE);
        member.candidate = Some(fixtures::HEAD);
        row.entries = Box::new([LandingEntry { entry: 91, member: 0, gate: 0, operation: Operation::Merge }]);
    }
    let mut entry = held_issue();
    entry.number = 91;
    entry.task = task;
    entry.start = None;
    entry.attempt = None;
    entry.failures = 0;
    entry.effect = skein_forge_client::Effect {
        write: Write::Merge {
            number: 1,
            head: fixtures::HEAD,
            how: if turn { skein_forge::api::MergeHow::FastForwardOnly } else { skein_forge::api::MergeHow::Style },
        },
        condition: skein_forge_client::Condition::Merge { base: Box::from(b"main".as_slice()) },
    };
    outputs(&mut d, Event::Enqueue { entry });
    outputs(&mut d, Event::Committed { entry: 91 });
    let mut merged = false;
    for _ in 0..8_u32 {
        let (call, op) = fixtures::next_call(&mut d);
        let answer = match op {
            Op::Read(Read::Branch { branch }) if branch.as_ref() == b"main" => Answer::Commit(fixtures::BASE),
            Op::Read(Read::Branch { branch }) if branch.as_ref() == b"temper/51" => Answer::Commit(fixtures::HEAD),
            Op::Read(Read::Items { .. }) => {
                Answer::Items { cut: false, items: Box::new([]), more: false, now: env().now }
            }
            Op::Read(Read::Pull { number: 1, ci: false }) => {
                let mut pull = fixtures::open_pull();
                if merged {
                    pull.state = skein_forge::api::State::Closed;
                    pull.merged = Some(fixtures::HEAD);
                }
                Answer::Pull(pull)
            }
            Op::Write(Write::Merge { head, .. }) => {
                assert_eq!(head, fixtures::HEAD);
                merged = true;
                Answer::Merged(fixtures::HEAD)
            }
            other @ (Op::Read(_) | Op::Write(_)) => panic!("unexpected merge call: {other:?}"),
        };
        let requests = fixtures::answer(&mut d, call, Ok(answer));
        let mut landed = false;
        for request in &requests {
            if let Request::Save { record: Stored::Landed { .. }, .. } = request {
                landed = true;
            }
        }
        if landed {
            assert!(merged, "outcome follows merge write");
            return (d, requests);
        }
    }
    panic!("merge reaches its write");
}

#[test]
fn a_turn_merge_records_its_members_change_as_the_landed_owner() {
    for turn in [false, true] {
        let (d, requests) = merged_owner(turn);
        assert!(
            requests.contains(&Request::Save {
                read_afresh: None,
                release: None,
                record: Stored::Landed { commit: fixtures::HEAD, task: fixtures::CHANGE },
            }),
            "merge saves the change as owner, turn={turn}: {requests:?}"
        );
        assert_eq!(d.landed.get(&fixtures::HEAD), Some(&fixtures::CHANGE));
        for request in &requests {
            if let Request::Save { record: Stored::Landed { task, .. }, .. } = request {
                assert_ne!(*task, 61, "landing task never owns the commit");
            }
        }
    }
}

#[test]
fn a_turn_merges_landed_owner_survives_rebuild() {
    let (_, requests) = merged_owner(true);
    let mut landed = None;
    for request in &requests {
        if let Request::Save { record: row @ Stored::Landed { .. }, .. } = request {
            landed = Some(row.clone());
        }
    }
    let landed = landed.expect("durable landed owner");
    let mut restarted = landing_fixture();
    outputs(&mut restarted, Event::Restore { record: landed });
    outputs(&mut restarted, Event::Restored { clock: skein_forge_client::RecoveryClock::Monotonic });
    outputs(&mut restarted, Event::ReadAfresh);
    outputs(&mut restarted, Event::SettleOutbox);
    assert_eq!(restarted.landed.get(&fixtures::HEAD), Some(&fixtures::CHANGE));
}
#[test]
fn cancelling_a_landing_leaves_its_change_live_and_detached() {
    let mut d = landing_fixture();
    member_landing(&mut d);
    let requests = outputs(&mut d, Event::SettleEffects { task: 61, root: 61, ending: ReleaseEnding::Cancelled });
    assert_eq!(d.landing(61).expect("cancelled landing").landing.phase, policy::Phase::Cancelled);
    assert_eq!(d.change(fixtures::CHANGE).expect("member stays").change.landing, None);
    for request in &requests {
        if let Request::Save { record: Stored::Release(row), .. } = request {
            assert_ne!(row.task, fixtures::CHANGE, "member is not cancelled");
        }
    }
}
#[test]
fn cancelling_the_last_member_cancels_its_landing_without_reading_the_missing_branch() {
    let mut d = landing_fixture();
    member_landing(&mut d);
    outputs(
        &mut d,
        Event::SettleEffects { task: fixtures::CHANGE, root: fixtures::CHANGE, ending: ReleaseEnding::Cancelled },
    );
    assert!(d.landing(61).expect("landing").landing.members.is_empty());
    let requests = outputs(
        &mut d,
        Event::StepLanding {
            task: 61,
            entry: 92,
            heard: LandingHeard {
                failure: Box::new([]),
                preparation: temper_engine_domain_forge_landing::Preparation::Idle,
                released: false,
                cancelled: false,
                kept: Box::new([]),
            },
        },
    );
    assert!(requests.contains(&Request::LandingDecision {
        task: 61,
        decision: policy::Decision::Cancel,
        entry: None,
        evidence: None
    }));
}

const OTHER_CHANGE: u64 = fixtures::CHANGE + 1;
const OTHER_REPOSITORY: skein_forge::api::Repository = skein_forge::api::Repository { forge: 1, repository: 3 };

fn across_fixture() -> Domain {
    let mut d = landing_fixture();
    let mut repository = adopted();
    repository.provider = OTHER_REPOSITORY;
    repository.name = b"lib".as_slice().into();
    repository.home = false;
    outputs(&mut d, Event::Restore { record: Stored::Repository(repository) });
    let mut change = fixtures::opening_row();
    change.task = OTHER_CHANGE;
    change.change.task = OTHER_CHANGE;
    change.change.landing = None;
    change.repository = OTHER_REPOSITORY;
    outputs(&mut d, Event::Restore { record: Stored::Change(Box::new(change)) });
    d
}

#[test]
fn a_dependency_closure_is_upstream_first_and_stops_at_a_formed_landing() {
    let mut d = across_fixture();
    let graph = [ChangeDependencies { change: fixtures::CHANGE, pending: Box::new([OTHER_CHANGE]) }];
    assert_eq!(d.joined_changes(5, fixtures::CHANGE, &graph).as_ref(), [OTHER_CHANGE, fixtures::CHANGE]);
    assert_eq!(d.joined_changes(5, OTHER_CHANGE, &graph).as_ref(), [OTHER_CHANGE, fixtures::CHANGE]);
    let mut third = d.changes.get(&fixtures::CHANGE).expect("change").clone();
    third.task = 53;
    third.change.task = 53;
    d.changes.insert(53, third).expect("third change");
    let graph = [
        ChangeDependencies { change: fixtures::CHANGE, pending: Box::new([OTHER_CHANGE]) },
        ChangeDependencies { change: OTHER_CHANGE, pending: Box::new([53]) },
    ];
    assert_eq!(d.joined_changes(5, fixtures::CHANGE, &graph).as_ref(), [53, OTHER_CHANGE, fixtures::CHANGE]);
    assert_eq!(d.landing_admits(5, &[53, fixtures::CHANGE]), Err(LandingRefusal::Members));
    let words = d.unit_words(&[53, fixtures::CHANGE]);
    assert_eq!(words.as_ref(), b"unit of 2 changes: C53, C51: several changes in one repository land with the merge queue's batches (temper/docs/design/draft/merge-queue.md, 4)");
    member_landing(&mut d);
    assert_eq!(d.joined_changes(5, OTHER_CHANGE, &graph).as_ref(), [53, OTHER_CHANGE]);
}

#[test]
fn a_formed_landing_is_sealed_against_joining_and_ordering() {
    let mut d = across_fixture();
    member_landing(&mut d);
    let mut member = d.landing(61).expect("formed landing").landing.members[0].clone();
    member.change = OTHER_CHANGE;
    member.repository.repository = OTHER_REPOSITORY.repository;
    let key = [7; 32];
    d.landing.rows.get_mut(&61).expect("landing").entries =
        Box::new([LandingEntry { entry: 91, member: 0, gate: 0, operation: Operation::Dispatch }]);
    d.landing.results.insert((61, 0, 0, 1), policy::Status::Pending).expect("result room");
    d.landing.keys.insert((61, 0, 0), (fixtures::HEAD, None, key)).expect("key room");
    d.landing.keys.insert((62, 0, 0), (fixtures::HEAD, None, key)).expect("other landing key room");
    d.landing.results.insert((61, 0, 0, 5), policy::Status::Passed).expect("merge result room");
    let before = d.landing(61).expect("formed landing").clone();
    outputs(&mut d, Event::JoinLanding { task: 61, member, after: Box::new([]) });
    outputs(&mut d, Event::OrderLanding { task: 61, changes: Box::new([OTHER_CHANGE, fixtures::CHANGE]) });
    assert_eq!(d.landing(61), Some(&before));
    assert_eq!(d.change(OTHER_CHANGE).expect("waiting change").change.landing, None);
    assert_eq!(d.landing.results.get(&(61, 0, 0, 1)), Some(&policy::Status::Pending));
    assert_eq!(d.landing.keys.get(&(61, 0, 0)), Some(&(fixtures::HEAD, None, key)));
    assert_eq!(d.landing.keys.get(&(62, 0, 0)), Some(&(fixtures::HEAD, None, key)));
    assert_eq!(d.landing.results.get(&(61, 0, 0, 5)), Some(&policy::Status::Passed));
    outputs(&mut d, Event::SettleEffects { task: OTHER_CHANGE, root: OTHER_CHANGE, ending: ReleaseEnding::Cancelled });
    let row = d.landing(61).expect("remaining landing");
    assert_eq!(row.landing.members[0].change, fixtures::CHANGE);
    assert_eq!(row.entries[0].member, 0);
    assert_eq!(d.landing.results.get(&(61, 0, 0, 1)), Some(&policy::Status::Pending));
    assert_eq!(d.landing.keys.get(&(61, 0, 0)), Some(&(fixtures::HEAD, None, key)));
    assert_eq!(d.landing.keys.get(&(62, 0, 0)), Some(&(fixtures::HEAD, None, key)));
    assert_eq!(d.landing.results.get(&(61, 0, 0, 5)), Some(&policy::Status::Passed));
}

#[test]
fn project_tiers_require_a_nonempty_explicit_branch_only() {
    use temper_engine_domain_forge_change::Way;
    use temper_engine_domain_forge_tiers::{Format, Settings, Suspects};
    for (follows, expected) in [
        (Follows::Landing, true),
        (Follows::Goals, true),
        (Follows::Branch(Box::new([])), false),
        (Follows::Branch(b"release/next".as_slice().into()), true),
    ] {
        let mut d = domain();
        assert_eq!(
            d.project_landing(
                5,
                Way::Queue,
                Box::new([RepositoryGates {
                    repository: REPO,
                    gates: Box::new([]),
                    tiers: Box::new([Tier {
                        name: b"g".as_slice().into(),
                        workflow: b"g.yml".as_slice().into(),
                        ignored: Box::new([]),
                        artifact: b"report".as_slice().into(),
                        format: Format::Lines,
                        follows,
                        span: None,
                        description: Box::default(),
                    }]),
                }]),
                temper_engine_domain_forge_landing::Project::DEFAULT,
                AfterLanding {
                    verdict: Settings { retries: 2, suspects: Suspects::Bisect },
                    bound: Duration::from_secs(7200),
                }
            ),
            expected
        );
    }
}

#[test]
fn committing_a_merge_drops_its_changes_review_facts_and_marks_the_wait_and_panel() {
    use crate::settings::{Asked, Edit, RuleApproval, SizedValue, Value};
    use jig_domain_people::ApplicationScope;
    use skein_forge::api::Write;
    for (turn, merge) in [(false, false), (true, false), (false, true), (true, true)] {
        let mut d = landing_fixture();
        if turn {
            member_landing(&mut d);
        }
        let repository = adopted();
        let rule = crate::settings::LandingRule {
            pattern: GatePattern {
                segments: Box::new([
                    b"forge".as_slice().into(),
                    repository.host,
                    repository.owner,
                    repository.name,
                    b"branch".as_slice().into(),
                ]),
                last: GateLast::Exact(b"main".as_slice().into()),
            },
            enforce: true,
            ci: false,
            up_to_date: false,
            gates: Box::new([]),
            approvals: Box::new([RuleApproval { role: 1, people: 1, freshness: JudgeFreshness::Exact }]),
        };
        d.keep_settings(
            &env(),
            2,
            None,
            &[Asked {
                kind: 1,
                scope: 0,
                number: 1,
                edit: Edit::Set(SizedValue { value: Value::Rule(rule), size: 128 }),
            }],
            &mut Queue::uncapped(),
        );
        let facts = ReviewFacts {
            head: Some(fixtures::HEAD),
            reviews_complete: true,
            reviewers: Box::new([]),
            unmapped: Box::new([]),
        };
        assert!(d.keep_review_facts(env().wall, fixtures::CHANGE, facts.clone()));
        let mut other = fixtures::opening_row();
        other.task = 52;
        other.change.task = 52;
        d.changes.insert(52, other).expect("unrelated change");
        assert!(d.keep_review_facts(env().wall, 52, facts.clone()));
        let panel = d.panel(fixtures::CHANGE, ApplicationScope::Project(5), env().wall, &LIMITS).expect("panel");
        assert_eq!(panel.approvals[0].read, ReviewRead::Complete);
        let mut entry = held_issue();
        entry.number = 91;
        entry.task = if turn { 61 } else { fixtures::CHANGE };
        entry.start = None;
        entry.attempt = None;
        if merge {
            entry.effect.write =
                Write::Merge { number: 1, head: fixtures::HEAD, how: skein_forge::api::MergeHow::Style };
        }
        outputs(&mut d, Event::Enqueue { entry });
        let requests = outputs(&mut d, Event::Committed { entry: 91 });
        let mark = Request::Marked { tasks: Box::new([APPROVALS_SUBJECT, fixtures::CHANGE]) };
        assert_eq!(requests.contains(&mark), merge, "turn={turn}, merge={merge}: {requests:?}");
        assert_eq!(d.review_facts.get(&fixtures::CHANGE), if merge { None } else { Some(&facts) });
        assert_eq!(d.review_since.get(&fixtures::CHANGE).copied(), if merge { None } else { Some(env().wall) });
        let panel = d.panel(fixtures::CHANGE, ApplicationScope::Project(5), env().wall, &LIMITS).expect("panel");
        assert_eq!(panel.approvals[0].read, if merge { ReviewRead::Unread } else { ReviewRead::Complete });
        assert_eq!(d.review_facts.get(&52), Some(&facts), "another change keeps its facts");
        assert_eq!(d.review_since.get(&52), Some(&env().wall));
        assert!(!outputs(&mut d, Event::Committed { entry: 91 }).contains(&mark), "no repeated drop mark");
    }
}

#[test]
#[expect(
    clippy::too_many_lines,
    reason = "one landing world drives both filtered-key bounds through the same read chain"
)]
fn filtered_tree_key_refusals_keep_the_bound_and_measured_usage() {
    use skein_forge::api::{Answer, Op, Read, TreeEntry, TreeEntryKind};
    for (member, entries, path_bytes, configured, needed) in
        [("forge.key_entries", 2, 1024, 2, 3), ("forge.key_path_bytes", 16, 4, 4, 6)]
    {
        let mut d = landing_fixture();
        fixtures::connect(&mut d);
        outputs(&mut d, Event::Restored { clock: skein_forge_client::RecoveryClock::Monotonic });
        outputs(&mut d, Event::ReadAfresh);
        outputs(&mut d, Event::SettleOutbox);
        d.changes.get_mut(&fixtures::CHANGE).unwrap().change.state =
            temper_engine_domain_forge_change::State::Ready { head: fixtures::HEAD, ready_since: env().wall };
        let after = d.after_landing(5).unwrap();
        assert!(d.project_landing(
            5,
            temper_engine_domain_forge_change::Way::Turn,
            Box::new([RepositoryGates {
                repository: REPO,
                tiers: Box::new([]),
                gates: Box::new([policy::Gate {
                    name: b"gate".as_slice().into(),
                    workflow: b"gate.yml".as_slice().into(),
                    ignored: Box::new([b"dir/skip".as_slice().into()]),
                    blocking: true,
                    base: false,
                }])
            }]),
            temper_engine_domain_forge_landing::Project::DEFAULT,
            after
        ));
        member_landing_with_gates(
            &mut d,
            Box::new([policy::GateProof { proof: policy::Proof::Unkeyed, posted: policy::Posted::Nothing, faults: 0 }]),
        );
        let mut env = env();
        env.limits.key_entries = entries;
        env.limits.key_path_bytes = path_bytes;
        let mut out = Queue::uncapped();
        step(
            &mut d,
            &env,
            Event::StepLanding {
                task: 61,
                entry: 91,
                heard: LandingHeard {
                    failure: Box::new([]),
                    preparation: temper_engine_domain_forge_landing::Preparation::Idle,
                    released: false,
                    cancelled: false,
                    kept: Box::new([]),
                },
            },
            &mut out,
        );
        assert!(!d.landing.pending.is_empty(), "initial {out:?}");
        let mut refused = false;
        for _ in 0..10_u32 {
            let (call, op) = fixtures::next_at(&mut d, &env);
            let seen = op.clone();
            let answer = match op {
                Op::Read(Read::Branch { branch }) => {
                    Answer::Commit(if branch.as_ref() == b"main" { fixtures::BASE } else { fixtures::HEAD })
                }
                Op::Read(Read::Commits { .. }) => Answer::Commits { commits: Box::new([]), more: false },
                Op::Read(Read::Pull { .. }) => Answer::Pull(fixtures::open_pull()),
                Op::Read(Read::Reviews { .. }) => Answer::Reviews { reviews: Box::new([]), more: false, cut: false },
                Op::Read(Read::Tree { object, .. }) => {
                    let entries = if object == [9; 32] {
                        Box::new([
                            TreeEntry {
                                name: b"a".as_slice().into(),
                                mode: 0o100_644,
                                kind: TreeEntryKind::Blob,
                                object: [7; 32],
                            },
                            TreeEntry {
                                name: b"b".as_slice().into(),
                                mode: 0o100_644,
                                kind: TreeEntryKind::Blob,
                                object: [8; 32],
                            },
                        ])
                    } else {
                        Box::new([
                            TreeEntry {
                                name: b"dir".as_slice().into(),
                                mode: 0o040_000,
                                kind: TreeEntryKind::Tree,
                                object: [9; 32],
                            },
                            TreeEntry {
                                name: b"x".as_slice().into(),
                                mode: 0o100_644,
                                kind: TreeEntryKind::Blob,
                                object: [7; 32],
                            },
                        ])
                    };
                    Answer::Tree { entries, more: false }
                }
                Op::Read(_) | Op::Write(_) => panic!("unexpected key read {op:?}"),
            };
            let out = fixtures::answer_at(&mut d, &env, call, Ok(answer));
            for request in &out {
                if let Request::ResourceRefused { task: 61, cause, .. } = request {
                    assert_eq!(*cause, RefusalCause::Limit { member, configured, needed });
                    refused = true;
                }
                assert!(!matches!(request, Request::Refused { .. }), "a key bound is not structural: {out:?}");
            }
            assert!(refused || !d.landing.pending.is_empty(), "chain ended after {seen:?}: {out:?}");
            if refused {
                break;
            }
        }
        assert!(refused, "filtered key must name {member}");
    }
}

#[test]
fn waiting_train_order_uses_priority_age_ready_time_and_task() {
    use temper_engine_domain_forge_change as c;
    let mut d = landing_fixture();
    let template = d.change(fixtures::CHANGE).expect("change").clone();
    for (older_priority, later_priority, older, later, now, expected) in [
        (9_i32, 0_i32, 100, 101, 102, OTHER_CHANGE),
        (9_i32, 0_i32, 100, 131, 131, fixtures::CHANGE),
        (0_i32, 0_i32, 100, 101, 102, fixtures::CHANGE),
        (0_i32, 0_i32, 100, 100, 102, fixtures::CHANGE),
    ] {
        for (task, priority, since) in
            [(fixtures::CHANGE, older_priority, older), (OTHER_CHANGE, later_priority, later)]
        {
            let mut row = template.clone();
            row.task = task;
            row.change.task = task;
            row.priority = priority;
            row.change.way = c::Way::Turn;
            row.change.state =
                c::State::Ready { head: fixtures::HEAD, ready_since: Wall::from_nanos(since * 1_000_000_000) };
            row.change.ready_since = Some(Wall::from_nanos(since * 1_000_000_000));
            d.changes.insert(task, row).expect("room");
        }
        let now = Wall::from_nanos(now * 1_000_000_000);
        assert_eq!(d.first_waiting(REPO, b"main", &[], now), Some(expected));
        let other = if expected == fixtures::CHANGE { OTHER_CHANGE } else { fixtures::CHANGE };
        assert_eq!(d.first_waiting(REPO, b"main", &[expected], now), Some(other), "kept tasks are not candidates");
    }
}

#[test]
#[expect(clippy::disallowed_methods, reason = "the test builds independent hexadecimal input expectations")]
fn dispatch_inputs_add_only_an_opted_in_base_at_the_forges_commit_width() {
    use skein_forge::api::Input;
    let key = [0xab; 32];
    for width in [20, 32] {
        let mut candidate = [0; 32];
        candidate[..width].fill(0xcd);
        let mut base = [0; 32];
        base[..width].fill(0xef);
        let expected = [
            Input { name: b"temper_commit".as_slice().into(), value: b"cd".repeat(width).into_boxed_slice() },
            Input { name: b"temper_key".as_slice().into(), value: b"ab".repeat(32).into_boxed_slice() },
        ];
        assert_eq!(landing::inputs(candidate, key, None).as_ref(), &expected);
        let mut expected = expected.to_vec();
        expected.push(Input { name: b"temper_base".as_slice().into(), value: b"ef".repeat(width).into_boxed_slice() });
        let inputs = landing::inputs(candidate, key, Some(base));
        assert_eq!(inputs.as_ref(), expected.as_slice());
        assert!(inputs.len() <= skein_forge::api::INPUTS);
    }
}

#[test]
fn a_train_is_held_from_forming_through_a_hold_until_its_landing_ends() {
    let mut d = landing_fixture();
    member_landing(&mut d);
    for (phase, free) in [
        (policy::Phase::Waiting, false),
        (policy::Phase::Preparing, false),
        (policy::Phase::Proving, false),
        (policy::Phase::Proved { at: env().wall }, false),
        (policy::Phase::Turn { at: env().wall }, false),
        (
            policy::Phase::Held {
                was: Box::new(policy::Phase::Preparing),
                why: policy::Hold::Faults { member: 0, gate: 0 },
            },
            false,
        ),
        (policy::Phase::Landed, true),
        (policy::Phase::Cancelled, true),
    ] {
        d.landing.rows.get_mut(&61).expect("landing").landing.phase = phase;
        assert_eq!(d.train_free(REPO, b"main"), free);
        assert!(d.train_free(REPO, b"temper/g1"));
        assert!(d.train_free(OTHER_REPOSITORY, b"main"));
    }
}

#[test]
fn a_landing_uses_its_projects_gate_retry_setting() {
    use skein_forge::api::{Answer, Op, Read};
    use temper_engine_domain_forge_change as change;
    const KEY: policy::Key = [7; 32];
    for retries in [0, 3] {
        let mut d = landing_fixture();
        fixtures::connect(&mut d);
        outputs(&mut d, Event::Restored { clock: skein_forge_client::RecoveryClock::Monotonic });
        outputs(&mut d, Event::ReadAfresh);
        outputs(&mut d, Event::SettleOutbox);
        d.changes.get_mut(&fixtures::CHANGE).expect("change").change.state =
            change::State::Ready { head: fixtures::HEAD, ready_since: env().wall };
        let after = d.after_landing(5).expect("configured project");
        assert!(d.project_landing(
            5,
            change::Way::Turn,
            Box::new([RepositoryGates {
                repository: REPO,
                gates: Box::new([policy::Gate {
                    name: b"gate".as_slice().into(),
                    workflow: b"gate.yml".as_slice().into(),
                    ignored: Box::new([]),
                    blocking: true,
                    base: false
                }]),
                tiers: Box::new([]),
            }]),
            policy::Project { gate_retries: retries, ..policy::Project::DEFAULT },
            after
        ));
        member_landing_with_gates(
            &mut d,
            Box::new([policy::GateProof {
                proof: policy::Proof::Dispatched { key: KEY, try_: 1 },
                posted: policy::Posted::Pending,
                faults: 0,
            }]),
        );
        let row = d.landing.rows.get_mut(&61).expect("landing");
        row.landing.phase = policy::Phase::Proving;
        let member = &mut row.landing.members[0];
        member.head = Some(fixtures::HEAD);
        member.base = Some(fixtures::BASE);
        member.candidate = Some(fixtures::HEAD);
        d.landing.main_keys.insert((REPO, b"gate".as_slice().into()), (fixtures::BASE, KEY)).expect("main key");
        let mut flight = runs::follow();
        flight.outcome = Some(policy::Verdict::Fault(policy::Fault::Lost));
        d.landing
            .flights
            .insert(
                ReceiptKey { project: 5, repository: REPO, check: Check::Gate(b"gate".as_slice().into()), key: KEY },
                flight,
            )
            .expect("flight");
        outputs(
            &mut d,
            Event::StepLanding {
                task: 61,
                entry: 91,
                heard: LandingHeard {
                    failure: Box::new([]),
                    preparation: temper_engine_domain_forge_landing::Preparation::Idle,
                    released: false,
                    cancelled: false,
                    kept: Box::new([]),
                },
            },
        );
        let mut decision = None;
        for _ in 0..5_u32 {
            let (call, op) = fixtures::next_call(&mut d);
            let answer = match op {
                Op::Read(Read::Branch { branch }) => {
                    Answer::Commit(if branch.as_ref() == b"main" { fixtures::BASE } else { fixtures::HEAD })
                }
                Op::Read(Read::Commits { .. }) => Answer::Commits { commits: Box::new([]), more: false },
                Op::Read(Read::Pull { .. }) => Answer::Pull(fixtures::open_pull()),
                Op::Read(Read::Reviews { .. }) => Answer::Reviews { reviews: Box::new([]), more: false, cut: false },
                Op::Read(_) | Op::Write(_) => panic!("unexpected gather read {op:?}"),
            };
            for request in fixtures::answer(&mut d, call, Ok(answer)) {
                if let Request::LandingDecision { decision: choice, .. } = request {
                    decision = Some(choice);
                }
            }
            if decision.is_some() {
                break;
            }
        }
        let expected = if retries == 0 {
            policy::Decision::Hold(policy::Hold::Faults { member: 0, gate: 0 })
        } else {
            policy::Decision::Retry { member: 0, gate: 0, key: KEY, try_: 2 }
        };
        assert_eq!(decision, Some(expected));
    }
}
