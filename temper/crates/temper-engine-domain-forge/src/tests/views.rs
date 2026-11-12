use super::*;
use jig_domain_people::ApplicationScope;
use skein_lib::List;
use temper_engine_domain_forge_landing as policy;

fn row(task: u64, phase: policy::Phase) -> LandingRow {
    LandingRow {
        landing: policy::Landing {
            round: temper_engine_domain_forge_landing::Round::new(Box::from(b"temper/1/l61".as_slice())),

            task,
            project: 5,
            phase,
            preparation: 1,
            stale_proofs: 2,
            branch: Box::from(b"main".as_slice()),
            members: Box::new([policy::Member {
                left: None,
                change: fixtures::CHANGE,
                repository: policy::Repository { forge: REPO.forge, repository: REPO.repository },
                pull: 1,
                head: Some(fixtures::HEAD),
                base: Some(fixtures::BASE),
                candidate: Some(fixtures::HEAD),
                gates: Box::new([policy::GateProof {
                    proof: policy::Proof::Dispatched { key: [7; 32], try_: 2 },
                    posted: policy::Posted::Pending,
                    faults: 1,
                }]),
                asked: None,
                landed: None,
            }]),
        },
        asks: Box::new([None]),
        entries: Box::new([]),
    }
}
fn setup() -> Domain {
    let mut d = domain();
    outputs(&mut d, Event::Restore { record: Stored::Repository(adopted()) });
    let mut change = fixtures::opening_row();
    change.pull = Some(1);
    outputs(&mut d, Event::Restore { record: Stored::Change(Box::new(change)) });
    assert!(d.project_landing(
        5,
        temper_engine_domain_forge_change::Way::Turn,
        Box::new([RepositoryGates {
            tiers: Box::new([]),
            repository: REPO,
            gates: Box::new([policy::Gate {
                name: Box::from(b"gate".as_slice()),
                workflow: Box::from(b"gate.yml".as_slice()),
                ignored: Box::new([]),
                blocking: true,
                base: false,
            }])
        }]),
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
fn document(d: &Domain, task: u64) -> Document {
    d.document(task, ApplicationScope::Project(5), 1, env().wall, &LIMITS).expect("document")
}
fn view(d: &Domain, task: u64) -> LandingView {
    let Document::Landing(view) = document(d, task) else { panic!("landing view") };
    view
}
fn list(d: &Domain, ended: bool) -> LandingsView {
    let Document::Landings(view) = document(d, landings_subject(5, ended)) else { panic!("list view") };
    view
}
#[test]
fn a_landing_view_shows_its_member_gate_and_turn() {
    let mut d = setup();
    outputs(&mut d, Event::Restore { record: Stored::Landing(row(61, policy::Phase::Proving)) });
    outputs(&mut d, Event::Restore { record: Stored::Landing(row(62, policy::Phase::Turn { at: env().wall })) });
    let got = view(&d, 61);
    assert_eq!((got.task, got.project, got.turn, got.preparation, got.stale_proofs), (61, 5, Some(62), 1, 2));
    assert_eq!(got.phase, policy::Phase::Proving);
    let member = &got.members[0];
    assert_eq!(
        (member.change, member.pull, member.base, member.candidate),
        (51, Some(1), Some(fixtures::BASE), Some(fixtures::HEAD))
    );
    assert_eq!(member.branch.as_ref(), b"temper/51");
    assert_eq!(
        member.gates[0],
        LandingGate {
            name: Box::from(b"gate".as_slice()),
            blocking: true,
            key: Some([7; 32]),
            proof: GateProof::Dispatched { attempt: 2, run: None },
            faults: 1
        }
    );
    assert!(d.document(61, ApplicationScope::Project(6), 1, env().wall, &LIMITS).is_none());
}
#[test]
fn a_gate_whose_key_another_landing_dispatched_is_waiting() {
    let mut d = setup();
    outputs(&mut d, Event::Restore { record: Stored::Landing(row(61, policy::Phase::Proving)) });
    let key =
        ReceiptKey { project: 5, repository: REPO, check: Check::Gate(Box::from(b"gate".as_slice())), key: [7; 32] };
    let mut flight = runs::follow();
    flight.task = 62;
    d.landing.flights.insert(key.clone(), flight).unwrap();
    assert_eq!(view(&d, 61).members[0].gates[0].proof, GateProof::Waiting);
    let flight = d.landing.flights.get_mut(&key).unwrap();
    flight.task = 61;
    flight.run = Some(skein_forge::api::Run {
        id: 7,
        number: 3,
        workflow: Box::from(b"gate.yml".as_slice()),
        commit: fixtures::HEAD,
        event: skein_forge::api::Event::Dispatch,
        actor: 1,
        state: skein_forge::api::RunState::Running,
        conclusion: None,
        url: Box::from(b"https://forge.example/run/7".as_slice()),
        created: env().now,
        jobs: Box::new([]),
    });
    assert_eq!(view(&d, 61).members[0].gates[0].proof, GateProof::Dispatched { attempt: 2, run: Some(7) });
    let flight = d.landing.flights.remove(&key).unwrap();
    d.landing.faults.insert((61, 0, 0), flight).unwrap();
    assert_eq!(view(&d, 61).members[0].gates[0].proof, GateProof::Dispatched { attempt: 2, run: Some(7) });
    let receipt = Receipt {
        logs: Box::new([]),
        failures: None,
        passed: true,
        candidate: fixtures::HEAD,
        ended: env().wall,
        run: ReceiptRun { id: 7, number: 3, url: Box::from(b"https://forge.example/run/7".as_slice()) },
    };
    d.landing.receipts.insert(key, receipt.clone()).unwrap();
    assert_eq!(view(&d, 61).members[0].gates[0].proof, GateProof::Passed(receipt.run));
}
#[test]
fn landings_list_live_first_then_newest() {
    let mut d = setup();
    for (task, phase) in
        [(61, policy::Phase::Turn { at: env().wall }), (62, policy::Phase::Proving), (63, policy::Phase::Waiting)]
    {
        outputs(&mut d, Event::Restore { record: Stored::Landing(row(task, phase)) });
    }
    let got = list(&d, false);
    assert_eq!(got.turn, Some(61));
    assert_eq!([got.landings[0].task, got.landings[1].task, got.landings[2].task], [63, 62, 61]);
    assert_eq!(got.landings[0].repositories[0].as_ref(), b"org/repo");
}
#[test]
fn ended_landings_are_listed_only_when_asked() {
    let mut d = setup();
    for (task, phase) in [(61, policy::Phase::Proving), (62, policy::Phase::Cancelled), (63, policy::Phase::Landed)] {
        outputs(&mut d, Event::Restore { record: Stored::Landing(row(task, phase)) });
    }
    assert_eq!(list(&d, false).landings.len(), 1);
    let all = list(&d, true);
    assert_eq!([all.landings[0].task, all.landings[1].task, all.landings[2].task], [61, 63, 62]);
}
#[test]
fn saving_a_landing_marks_its_subject_and_its_projects_lists() {
    let mut d = setup();
    let mut row = row(61, policy::Phase::Waiting);
    row.landing.members[0].gates[0].proof = policy::Proof::Unkeyed;
    let got = outputs(&mut d, Event::Landing { row });
    let mut landing_marked = false;
    let mut member_marked = false;
    for request in &got {
        if let Request::Marked { tasks } = request {
            landing_marked |=
                tasks.as_ref() == [61, landings_subject(5, false), landings_subject(5, true), changes_subject(5)];
            member_marked |= tasks.as_ref() == [51, 61];
        }
    }
    assert!(landing_marked && member_marked);
}

#[test]
fn a_landed_queue_change_is_a_change_document_and_has_no_turn_landing() {
    let mut d = domain();
    outputs(&mut d, Event::Restore { record: Stored::Repository(adopted()) });
    let mut row = fixtures::opening_row();
    row.change.state =
        temper_engine_domain_forge_change::State::Landed { merge: fixtures::HEAD, since: Wall::from_nanos(17) };
    row.pending = None;
    outputs(&mut d, Event::Restore { record: Stored::Change(Box::new(row)) });
    let Document::Change(panel) = document(&d, fixtures::CHANGE) else { panic!("queue change document") };
    assert_eq!(panel.task, fixtures::CHANGE);
    assert_eq!(panel.way, temper_engine_domain_forge_change::Way::Queue);
    assert_eq!(panel.in_landing, None);
    assert_eq!(panel.step, PanelStep::Landed { merge: fixtures::HEAD });
    for ended in [false, true] {
        let panel = list(&d, ended);
        assert_eq!(panel.project, 5);
        assert_eq!(panel.turn, None);
        assert!(panel.landings.is_empty());
    }
    assert!(d.document(fixtures::CHANGE, ApplicationScope::Project(6), 1, env().wall, &LIMITS).is_none());
}

#[test]
fn dismissed_is_each_watchers_own_rows() {
    let mut d = domain();
    for (party, project) in [(1, 7), (2, 9), (1, 3)] {
        outputs(&mut d, Event::Restore { record: Stored::Dismissed { party, what: PersonFact::ToBegin { project } } });
    }
    for (party, projects) in [(1, [3, 7].as_slice()), (2, [9].as_slice()), (3, [].as_slice())] {
        let mut rows = List::with_capacity(2);
        for project in projects {
            rows.push(PersonFact::ToBegin { project: *project }).unwrap();
        }
        let rows = rows.into_boxed();
        let expected = Document::Dismissals(rows);
        assert_eq!(
            d.document(DISMISSED_SUBJECT, ApplicationScope::Own, party, env().wall, &LIMITS),
            Some(expected.clone())
        );
        assert_eq!(expected.owned_bytes(), size_of::<Document>() + size_of::<PersonFact>() * d.dismissals(party).len());
    }
}

#[test]
fn the_own_scope_answers_no_other_subject() {
    let d = setup();
    for subject in [0, fixtures::CHANGE, landings_subject(5, false)] {
        assert_eq!(d.document(subject, ApplicationScope::Own, 1, env().wall, &LIMITS), None);
    }
    for scope in [ApplicationScope::Project(5), ApplicationScope::Deployment] {
        assert_eq!(d.document(DISMISSED_SUBJECT, scope, 1, env().wall, &LIMITS), None);
    }
}

// Whole queues and retained titles (temper/docs/design/domain/forge/queues.md).
use temper_engine_domain_forge_change as change;
const PROJECT: u32 = 5;
const SECOND: u64 = fixtures::CHANGE + 1;
const THIRD: u64 = SECOND + 1;
const FOURTH: u64 = THIRD + 1;
const OTHER_REPO: skein_forge::api::Repository = skein_forge::api::Repository { forge: 1, repository: 3 };
const TITLE: &[u8] = b"Small fix";

fn queued(task: u64, priority: i32, seconds: u64) -> ChangeRow {
    let mut row = fixtures::opening_row();
    row.task = task;
    row.change.task = task;
    row.priority = priority;
    row.change.state = change::State::Queued {
        head: fixtures::HEAD,
        ready_since: Wall::from_nanos(seconds.checked_mul(1_000_000_000).expect("fixture seconds")),
    };
    row.change.ready_since = Some(Wall::from_nanos(seconds.checked_mul(1_000_000_000).expect("fixture seconds")));
    row.pending = None;
    row
}
fn changes_setup(rows: &[(u64, i32, u64)]) -> Domain {
    let mut d = domain();
    outputs(&mut d, Event::Restore { record: Stored::Repository(adopted()) });
    for (task, priority, seconds) in rows {
        outputs(&mut d, Event::Restore { record: Stored::Change(Box::new(queued(*task, *priority, *seconds))) });
    }
    d
}
fn changes(d: &Domain) -> ChangesView {
    let Document::Changes(view) = document(d, changes_subject(PROJECT)) else { panic!("changes document") };
    view
}
fn tasks(view: &ChangesView) -> Box<[u64]> {
    let mut tasks = List::with_capacity(LIMITS.changes_rows);
    for queue in &view.queues {
        for row in &queue.changes {
            tasks.push(row.task).unwrap();
        }
    }
    tasks.into_boxed()
}
fn logged(task: u64, commit: u8, at: u64, branch: &[u8]) -> LoggedLanding {
    LoggedLanding {
        commit: [commit; 32],
        branch: Box::from(branch),
        at: Wall::from_nanos(at),
        changes: Box::new([LoggedChange { task, pull: None, title: Box::from(TITLE) }]),
    }
}
fn log(d: &mut Domain, repository: skein_forge::api::Repository, landings: Box<[LoggedLanding]>) {
    outputs(d, Event::Restore { record: Stored::LandingLog(LandingLog { repository, landings }) });
}
fn marked(out: &[Request]) -> bool {
    for request in out {
        if let Request::Marked { tasks } = request
            && tasks.contains(&changes_subject(PROJECT))
            && tasks.contains(&status_subject(PROJECT))
            && tasks.contains(&APPROVALS_SUBJECT)
        {
            return true;
        }
    }
    false
}
fn queue_marked(out: &Queue<Request>) -> bool {
    for request in out {
        if let Request::Marked { tasks } = request
            && tasks.contains(&changes_subject(PROJECT))
            && tasks.contains(&status_subject(PROJECT))
            && tasks.contains(&APPROVALS_SUBJECT)
        {
            return true;
        }
    }
    false
}
fn red(d: &mut Domain) {
    outputs(
        d,
        Event::Restore {
            record: Stored::Verification(Verification {
                repository: REPO,
                commit: fixtures::HEAD,
                changes: Box::new([fixtures::CHANGE]),
                tiers: Box::new([TierVerification {
                    tier: Box::from(b"browser".as_slice()),
                    repaired: None,
                    verdict: Some(TierVerdict {
                        run: 1,
                        blame: temper_engine_domain_forge_tiers::Blame::Suspect(
                            temper_engine_domain_forge_tiers::Failures::Many,
                        ),
                    }),
                }]),
            }),
        },
    );
}
#[test]
fn changes_document_lists_a_queue_in_order() {
    let d =
        changes_setup(&[(fixtures::CHANGE, 9_i32, 95), (SECOND, 1_i32, 96), (THIRD, 9_i32, 90), (FOURTH, 20_i32, 60)]);
    let got = changes(&d);
    assert_eq!(tasks(&got).as_ref(), &[FOURTH, SECOND, THIRD, fixtures::CHANGE]);
    assert_eq!(got.queues[0].branch.as_ref(), b"main");
    assert_eq!(got.queues[0].changes[0].title.as_ref(), TITLE);
    assert_eq!(got.queues[0].changes[0].ready_since, Wall::from_nanos(60_000_000_000));
    assert_eq!(got.queues[0].changes[0].checks_since, Some(env().wall));
    for row in &got.queues[0].changes[1..] {
        assert_eq!(row.checks_since, None);
    }
}
#[test]
fn changes_document_says_a_red_with_nothing_queued() {
    let mut d = changes_setup(&[]);
    let mut row = fixtures::opening_row();
    row.change.state = change::State::Red { merge: fixtures::HEAD, since: env().wall };
    outputs(&mut d, Event::Restore { record: Stored::Change(Box::new(row)) });
    red(&mut d);
    let got = changes(&d);
    assert_eq!(got.landing.as_ref(), b"main");
    assert_eq!(got.queues.len(), 1);
    assert!(got.queues[0].red);
    assert!(got.queues[0].changes.is_empty());
    d.verifications.get_mut(&(REPO, fixtures::HEAD)).unwrap().tiers[0].repaired =
        Some(TierRepair { run: 2, tip: fixtures::BASE, by: None });
    assert!(changes(&d).queues.is_empty());
    d.changes.get_mut(&fixtures::CHANGE).unwrap().change.state = change::State::Producing { requested: false };
    d.changes.get_mut(&fixtures::CHANGE).unwrap().base_repair = true;
    assert!(changes(&d).queues[0].red);
}
#[test]
fn changes_document_cuts_and_counts() {
    let mut d = changes_setup(&[(fixtures::CHANGE, 0_i32, 90), (SECOND, 0_i32, 91), (THIRD, 0_i32, 92)]);
    d.changes.get_mut(&THIRD).unwrap().base = Box::from(b"next".as_slice());
    log(&mut d, REPO, Box::new([logged(fixtures::CHANGE, 1, 1, b"main"), logged(SECOND, 2, 2, b"next")]));
    let limits = Limits { changes_rows: 1, changes_landed: 1, ..LIMITS };
    let Some(Document::Changes(got)) =
        d.document(changes_subject(PROJECT), ApplicationScope::Project(PROJECT), 1, env().wall, &limits)
    else {
        panic!("bounded changes")
    };
    assert_eq!(tasks(&got).as_ref(), &[fixtures::CHANGE]);
    assert_eq!((got.more, got.landed_more), (2, 1));
    assert_eq!(got.queues.len(), 2);
    assert!(got.queues[1].changes.is_empty());
    assert_eq!(got.landed.len(), 1);
    assert_eq!(got.landed[0].task, SECOND);
    assert_eq!(got.landing.as_ref(), b"next");
}
#[test]
fn changes_document_lists_landed_newest_first_across_repositories() {
    let mut d = changes_setup(&[]);
    let mut repository = adopted();
    repository.provider = OTHER_REPO;
    repository.home = false;
    outputs(&mut d, Event::Restore { record: Stored::Repository(repository) });
    log(&mut d, REPO, Box::new([logged(fixtures::CHANGE, 2, 1, b"old"), logged(THIRD, 3, 3, b"new")]));
    log(&mut d, OTHER_REPO, Box::new([logged(SECOND, 1, 2, b"middle"), logged(FOURTH, 1, 3, b"tie")]));
    let got = changes(&d);
    assert_eq!(
        [got.landed[0].task, got.landed[1].task, got.landed[2].task, got.landed[3].task],
        [THIRD, FOURTH, SECOND, fixtures::CHANGE]
    );
    assert_eq!(got.landing.as_ref(), b"new");
    for row in &got.landed {
        assert_eq!(row.title.as_ref(), TITLE);
    }
}
#[test]
fn changes_document_keeps_to_its_project() {
    let mut d = changes_setup(&[(fixtures::CHANGE, 0_i32, 90)]);
    let mut repository = adopted();
    repository.provider = OTHER_REPO;
    repository.project = PROJECT + 1;
    outputs(&mut d, Event::Restore { record: Stored::Repository(repository) });
    let mut row = queued(SECOND, -10_i32, 50);
    row.repository = OTHER_REPO;
    outputs(&mut d, Event::Restore { record: Stored::Change(Box::new(row)) });
    log(&mut d, OTHER_REPO, Box::new([logged(SECOND, 1, 5, b"private")]));
    assert_eq!(tasks(&changes(&d)).as_ref(), &[fixtures::CHANGE]);
    assert!(changes(&d).landed.is_empty());
    assert_eq!(
        d.document(changes_subject(PROJECT + 1), ApplicationScope::Project(PROJECT), 1, env().wall, &LIMITS),
        None
    );
}
#[test]
fn changes_marks_queue_entry_and_priority_moves() {
    let mut d = changes_setup(&[]);
    assert!(marked(&outputs(&mut d, Event::Change { row: queued(fixtures::CHANGE, 0_i32, 90) })));
    let mut out = Queue::uncapped();
    d.priority(&env(), fixtures::CHANGE, -10, &mut out);
    assert!(queue_marked(&out));
    outputs(&mut d, Event::Delegated { task: fixtures::CHANGE, child: 90, kind: change::Delegate::Produce });
    assert!(marked(&outputs(&mut d, Event::DelegateRefused { task: fixtures::CHANGE, child: 90 })));
}
#[test]
fn changes_marks_the_owners_and_base_repairs_steps() {
    for (owner, repair) in [(true, false), (false, true)] {
        let mut d = changes_setup(&[(fixtures::CHANGE, 0_i32, 90)]);
        let mut row = d.changes.get(&fixtures::CHANGE).unwrap().clone();
        row.change.owns_turn = owner;
        row.base_repair = repair;
        let mut out = Queue::uncapped();
        domain::save_change(&mut d, &mut out, row);
        assert!(queue_marked(&out));
    }
}
#[test]
fn changes_marks_each_logged_merge() {
    let mut d = changes_setup(&[(fixtures::CHANGE, 0_i32, 90)]);
    let mut out = Queue::uncapped();
    landing_log::landed(&mut d, env().wall, fixtures::HEAD, fixtures::CHANGE, &mut out);
    assert!(queue_marked(&out));
}
#[test]
fn changes_marks_a_verifications_save_and_erasure() {
    let mut d = setup();
    // setup's configured gate is independent of a followed tier.
    d.project_landing(
        PROJECT,
        change::Way::Queue,
        Box::new([RepositoryGates {
            repository: REPO,
            gates: Box::new([]),
            tiers: Box::new([Tier {
                name: Box::from(b"browser".as_slice()),
                workflow: Box::from(b"browser.yml".as_slice()),
                follows: Follows::Landing,
                ignored: Box::new([]),
                artifact: Box::from(b"tests".as_slice()),
                format: temper_engine_domain_forge_tiers::Format::Lines,
                span: None,
                description: Box::default(),
            }]),
        }]),
        temper_engine_domain_forge_landing::Project::DEFAULT,
        AfterLanding {
            verdict: temper_engine_domain_forge_tiers::Settings {
                retries: 2,
                suspects: temper_engine_domain_forge_tiers::Suspects::Bisect,
            },
            bound: Duration::from_secs(60),
        },
    );
    let mut out = Queue::uncapped();
    verification::landed(&mut d, fixtures::HEAD, fixtures::CHANGE, &mut out);
    assert!(queue_marked(&out));
    let mut out = Queue::uncapped();
    let verdict = temper_engine_domain_forge_tiers::Verdict {
        number: 1,
        tip: fixtures::HEAD,
        key: [7; 32],
        failing: temper_engine_domain_forge_tiers::Failures::Many,
        landings: Box::new([temper_engine_domain_forge_tiers::Blame::Owns(
            temper_engine_domain_forge_tiers::Failures::Many,
        )]),
        flakes: Box::new([]),
    };
    verification::judged(
        &mut d,
        REPO,
        b"browser",
        &verdict,
        &[temper_engine_domain_forge_tiers::Landing { commit: fixtures::HEAD, key: [7; 32] }],
        &mut out,
    );
    assert!(queue_marked(&out));
    assert!(changes(&d).queues[0].red);
    let mut out = Queue::uncapped();
    let verdict = temper_engine_domain_forge_tiers::Verdict {
        number: 2,
        tip: fixtures::BASE,
        key: [8; 32],
        failing: temper_engine_domain_forge_tiers::Failures::Tests(Box::new([])),
        landings: Box::new([]),
        flakes: Box::new([]),
    };
    verification::judged(&mut d, REPO, b"browser", &verdict, &[], &mut out);
    assert!(queue_marked(&out));
    assert!(changes(&d).queues.is_empty());
    let mut out = Queue::uncapped();
    verification::released(&mut d, fixtures::CHANGE, &mut out);
    assert!(queue_marked(&out));
}
#[test]
fn changes_owned_bytes_charges_every_field() {
    const STRINGS: usize = 4 + 4 + 4 + TITLE.len() * 2;
    let view = ChangesView {
        landed_default: Box::new([ChangesLanded {
            task: THIRD,
            title: Box::from(TITLE),
            at: env().wall,
            commit: [0xab; 32],
            default_branch: true,
        }]),
        landed_default_more: 0,
        project: PROJECT,
        landing: Box::from(b"main".as_slice()),
        more: 3,
        landed_more: 4,
        queues: Box::new([ChangesQueue {
            way: change::Way::Queue,
            in_flight: None,
            repository: Box::from(b"repo".as_slice()),
            branch: Box::from(b"next".as_slice()),
            red: true,
            changes: Box::new([ChangesQueued {
                task: fixtures::CHANGE,
                title: Box::from(TITLE),
                ready_since: env().wall,
                checks_since: Some(env().wall),
            }]),
        }]),
        landed: Box::new([ChangesLanded {
            commit: [0xab; 32],
            default_branch: true,
            task: SECOND,
            title: Box::from(TITLE),
            at: env().wall,
        }]),
    };
    assert_eq!(
        Document::Changes(view).owned_bytes(),
        size_of::<Document>()
            + size_of::<ChangesQueue>()
            + size_of::<ChangesQueued>()
            + 2 * size_of::<ChangesLanded>()
            + STRINGS
            + TITLE.len()
    );
}
#[test]
fn changes_limits_accept_the_bounds_and_refuse_outside() {
    for count in [1, 32] {
        assert!(valid(&Limits { changes_rows: count, changes_landed: count, ..LIMITS }));
    }
    for count in [0, 33] {
        assert!(!valid(&Limits { changes_rows: count, ..LIMITS }));
        assert!(!valid(&Limits { changes_landed: count, ..LIMITS }));
    }
}

#[test]
fn changes_history_keeps_default_landings_apart_with_their_commit_and_branch_kind() {
    let mut d = changes_setup(&[]);
    let mut repository = adopted();
    repository.provider = OTHER_REPO;
    repository.home = false;
    repository.settings.default_branch = b"trunk".as_slice().into();
    outputs(&mut d, Event::Restore { record: Stored::Repository(repository) });
    let mut outside = logged(90, 90, 90, b"main");
    outside.changes = Box::new([]);
    log(
        &mut d,
        REPO,
        Box::new([
            logged(10, 10, 10, b"main"),
            logged(20, 20, 20, b"main"),
            logged(30, 30, 30, b"goal/7"),
            logged(40, 40, 40, b"goal/7"),
            logged(50, 50, 50, b"goal/7"),
            outside,
        ]),
    );
    log(&mut d, OTHER_REPO, Box::new([logged(25, 25, 25, b"trunk")]));
    let limits = Limits { changes_landed: 2, ..LIMITS };
    let Document::Changes(got) =
        d.document(changes_subject(PROJECT), ApplicationScope::Project(PROJECT), 1, env().wall, &limits).unwrap()
    else {
        panic!("changes")
    };
    assert_eq!(
        got.landed.as_ref(),
        &[
            ChangesLanded {
                task: 50,
                title: TITLE.into(),
                at: Wall::from_nanos(50),
                commit: [50; 32],
                default_branch: false
            },
            ChangesLanded {
                task: 40,
                title: TITLE.into(),
                at: Wall::from_nanos(40),
                commit: [40; 32],
                default_branch: false
            },
        ]
    );
    assert_eq!(got.landed_more, 4);
    assert_eq!(
        got.landed_default.as_ref(),
        &[
            ChangesLanded {
                task: 25,
                title: TITLE.into(),
                at: Wall::from_nanos(25),
                commit: [25; 32],
                default_branch: true
            },
            ChangesLanded {
                task: 20,
                title: TITLE.into(),
                at: Wall::from_nanos(20),
                commit: [20; 32],
                default_branch: true
            },
        ]
    );
    assert_eq!(got.landed_default_more, 1);
    assert_eq!(got.landing.as_ref(), b"main", "a push without changes still names the latest branch");
}

#[test]
fn a_squashed_landing_keeps_its_train_round_and_prefix_judgement_after_landing() {
    let mut d = setup();
    let mut row = row(61, policy::Phase::Landed);
    row.landing.round.pull = Some(7);
    row.landing.round.combined = Some(policy::Combine::Squash);
    row.landing.round.judging = true;
    row.landing.members[0].candidate = Some([9; 32]);
    row.landing.members[0].landed = Some([9; 32]);
    let change = d.changes.get_mut(&fixtures::CHANGE).unwrap();
    change.change.way = change::Way::Turn;
    change.change.gates = Box::new([change::Gate {
        number: 3,
        kind: change::GateKind::Agent,
        blocking: true,
        freshness: change::Freshness::Exact,
        eager: false,
    }]);
    change.verdicts = Box::new([
        change::GateReport { number: 3, head: [9; 32], status: change::Status::Passed },
        change::GateReport { number: 3, head: fixtures::HEAD, status: change::Status::Failed },
    ]);
    outputs(&mut d, Event::Restore { record: Stored::Landing(row) });
    let got = view(&d, 61);
    let train = got.train.unwrap();
    assert_eq!(
        (train.repository, train.owner.as_ref(), train.name.as_ref(), train.branch.as_ref()),
        (REPO, b"org".as_slice(), b"repo".as_slice(), b"main".as_slice())
    );
    assert_eq!(got.round.branch.as_ref(), b"temper/1/l61");
    assert_eq!(
        (got.preparation, got.round.pull, got.round.combined, got.step),
        (1, Some(7), Some(policy::Combine::Squash), LandingStep::Landed)
    );
    let member = &got.members[0];
    assert_eq!(
        (member.head, member.base, member.candidate, member.landed),
        (Some(fixtures::HEAD), Some(fixtures::BASE), Some([9; 32]), Some([9; 32]))
    );
    let judgement = member.judgement.as_ref().unwrap();
    assert_eq!(
        (judgement.prefix, judgement.verdict, judgement.gates[0].verdict),
        ([9; 32], change::Status::Passed, change::Status::Passed)
    );
    let Document::Change(panel) = document(&d, fixtures::CHANGE) else { panic!("change panel") };
    assert_eq!((panel.way, panel.in_landing), (change::Way::Turn, Some(61)));
}

#[test]
fn turn_changes_are_grouped_by_train_with_the_landing_before_its_waiters() {
    let mut d = setup();
    for (task, base, priority) in [
        (fixtures::CHANGE, b"main".as_slice(), 0_i32),
        (SECOND, b"main", 0_i32),
        (THIRD, b"release", 2_i32),
        (FOURTH, b"release", 1_i32),
    ] {
        let mut row = queued(task, priority, 0);
        row.change.way = change::Way::Turn;
        row.change.state = change::State::Ready { head: fixtures::HEAD, ready_since: env().wall };
        row.change.ready_since = Some(env().wall);
        row.base = base.into();
        row.title = TITLE.into();
        row.change.landing = if task == fixtures::CHANGE { Some(61) } else { None };
        outputs(&mut d, Event::Restore { record: Stored::Change(Box::new(row)) });
    }
    let mut landing =
        row(61, policy::Phase::Held { was: Box::new(policy::Phase::Proving), why: policy::Hold::Prepare });
    landing.landing.preparation = 2;
    outputs(&mut d, Event::Restore { record: Stored::Landing(landing) });
    let got = changes(&d);
    assert_eq!(got.queues.len(), 2);
    let busy = &got.queues[0];
    assert_eq!(
        (busy.repository.as_ref(), busy.branch.as_ref(), busy.way),
        (b"org/repo".as_slice(), b"main".as_slice(), change::Way::Turn)
    );
    let flight = busy.in_flight.as_ref().unwrap();
    assert_eq!((flight.task, flight.member, flight.round, flight.step), (61, fixtures::CHANGE, 2, LandingStep::Held));
    assert_eq!(busy.changes.len(), 1);
    assert_eq!(busy.changes[0].task, SECOND);
    assert_eq!(busy.changes[0].checks_since, None);
    let free = &got.queues[1];
    assert_eq!(free.branch.as_ref(), b"release");
    assert_eq!(free.in_flight, None);
    assert_eq!(free.changes.len(), 2);
    assert_eq!([free.changes[0].task, free.changes[1].task], [FOURTH, THIRD]);
}

#[test]
fn landing_steps_distinguish_prefix_judgement_leaving_and_recording() {
    let mut d = setup();
    let mut landing = row(61, policy::Phase::Proving);
    landing.landing.round.judging = true;
    outputs(&mut d, Event::Restore { record: Stored::Landing(landing) });
    d.changes.get_mut(&fixtures::CHANGE).unwrap().change.gates = Box::new([change::Gate {
        number: 3,
        kind: change::GateKind::Agent,
        blocking: true,
        freshness: change::Freshness::Exact,
        eager: false,
    }]);
    assert_eq!(view(&d, 61).step, LandingStep::Judging);
    assert_eq!(view(&d, 61).members[0].judgement.as_ref().unwrap().verdict, change::Status::Pending);
    d.changes.get_mut(&fixtures::CHANGE).unwrap().verdicts =
        Box::new([change::GateReport { number: 3, head: fixtures::HEAD, status: change::Status::Passed }]);
    assert_eq!(view(&d, 61).step, LandingStep::Proving);
    d.landing.rows.get_mut(&61).unwrap().landing.round.ending = Some(policy::RoundEnd::Stale);
    assert_eq!(view(&d, 61).step, LandingStep::Leaving);
    for why in [
        policy::Leave::Conflict,
        policy::Leave::HeadMoved { head: [8; 32] },
        policy::Leave::Cancelled,
        policy::Leave::Held,
        policy::Leave::Refused { gate: 3 },
        policy::Leave::Failed { gate: 0, run: 7 },
    ] {
        d.landing.rows.get_mut(&61).unwrap().landing.members[0].left = Some(why);
        assert_eq!(view(&d, 61).members[0].left, Some(why));
    }
    let landing = &mut d.landing.rows.get_mut(&61).unwrap().landing;
    landing.round.ending = None;
    landing.phase = policy::Phase::Turn { at: env().wall };
    landing.round.combined = Some(policy::Combine::Squash);
    landing.members[0].landed = Some(fixtures::HEAD);
    assert_eq!(view(&d, 61).step, LandingStep::Recording);
    d.landing.rows.get_mut(&61).unwrap().landing.round.recorded = true;
    assert_eq!(view(&d, 61).step, LandingStep::Telling);
}
