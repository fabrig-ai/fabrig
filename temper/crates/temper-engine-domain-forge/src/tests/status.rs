//! Project landings and party sittings (temper/docs/design/domain/forge.md, 5.6).
use super::*;
use jig_domain_people::ApplicationScope;

const PARTY: u64 = 7;
const OTHER_PARTY: u64 = 8;
const REQUEST: Token = Token::new(90);
const ANSWER: Token = Token::new(91);
// Four bindings times two projects, shared LIMITS.
const LOOKS: u32 = 8;
const BEFORE: Wall = Wall::from_nanos(10_000_000_000_000);

fn facts(project: u32, landed: u64) -> ProjectFacts {
    ProjectFacts { project, landed, last_landed: BEFORE, brief: None, running: None }
}
fn look_row(party: u64, project: u32) -> Looked {
    Looked { party, project, previous: Some(Wall::EPOCH), latest: BEFORE, seen: 1, next_seen: 2 }
}
fn restore(d: &mut Domain, env: &Env<Limits>, row: Stored) {
    d.restore_begin(Wall::EPOCH);
    super::restore(d, env, row);
}
fn restored() -> Domain {
    let mut d = configured_domain(&[]);
    restore(&mut d, &env(), Stored::ProjectFacts(facts(PROJECT, 3)));
    restore(&mut d, &env(), Stored::Looked(look_row(PARTY, PROJECT)));
    d
}
fn look(d: &mut Domain, party: u64, now: Wall) -> Queue<Request> {
    let mut env = env();
    env.wall = now;
    let mut out = Queue::uncapped();
    step(d, &env, Event::Looked { request: REQUEST, token: ANSWER, party, project: PROJECT }, &mut out);
    out
}
fn view(d: &Domain, party: u64) -> StatusView {
    let Document::Status(view) =
        d.document(status_subject(PROJECT), ApplicationScope::Project(PROJECT), party, env().wall, &LIMITS).unwrap()
    else {
        panic!("status document")
    };
    view
}
fn rows(mut out: Queue<Request>) -> Box<[Request]> {
    let mut rows = List::with_capacity(out.len());
    for _ in 0..out.len() {
        rows.push(out.pop().unwrap()).unwrap();
    }
    rows.into_boxed()
}
fn outputs(d: &mut Domain, event: Event) -> Box<[Request]> {
    let mut out = Queue::uncapped();
    step(d, &env(), event, &mut out);
    rows(out)
}
fn marked() -> Request {
    Request::Marked { tasks: Box::new([status_subject(PROJECT)]) }
}
fn after(seconds: u64) -> Wall {
    Wall::from_nanos(BEFORE.as_nanos().checked_add(Duration::from_secs(seconds).as_nanos()).unwrap())
}
fn observe_pull(d: &mut Domain) {
    let pull = skein_forge::api::Pull {
        number: PULL,
        state: skein_forge::api::State::Closed,
        merged: Some(FIRST),
        head: Box::from(b"temper/51".as_slice()),
        commit: HEAD,
        base_commit: Some(FIRST),
        base: Box::from(b"main".as_slice()),
        mergeable: true,
        reviewers: Box::new([]),
        ci: skein_forge::api::Ci::Passed,
        queued: None,
        dequeued: None,
    };
    let pending = PendingStep {
        task: CHANGE,
        entry: OPEN,
        stage: StepStage::Branch,
        heard: heard(false),
        gates: Box::new([]),
        queue_repair_active: false,
        kept: false,
        merge_job: false,
        pending: Box::new([]),
        branch: Some(HEAD),
        base_tip: Some(FIRST),
        pull: Some(pull),
        ci: change::Status::Passed,
        base_ci: change::Status::Passed,
        contains_base: change::Status::Passed,
        contains_catch_up: change::Status::Passed,
        reviews: Box::new([]),
        reviews_complete: true,
        woken: false,
    };
    decide_step(d, &env(), &pending, &mut Queue::uncapped());
}

#[test]
fn a_landing_counts_once_and_at_either_site() {
    for observe in [false, true] {
        let mut d = configured_domain(&[]);
        add_change(&mut d, CHANGE, b"main");
        if observe {
            observe_pull(&mut d);
        } else {
            merged(&mut d, CHANGE, FIRST);
        }
        let mut repeated = Queue::uncapped();
        landing_log::landed(&mut d, env().wall, FIRST, CHANGE, &mut repeated);
        assert!(repeated.is_empty());
        assert_eq!(
            d.project_facts.get(&PROJECT),
            Some(&ProjectFacts { project: PROJECT, landed: 1, last_landed: env().wall, brief: None, running: None })
        );
        add_change(&mut d, SECOND, b"main");
        merged(&mut d, SECOND, FIRST);
        assert_eq!(view(&d, PARTY).landed, 2);
    }
}
#[test]
fn a_goal_branch_landing_counts() {
    let mut d = configured_domain(&[]);
    add_change(&mut d, CHANGE, b"temper/goal/1");
    let out = merged(&mut d, CHANGE, FIRST);
    assert!(contains(&out, &marked()));
    assert_eq!(view(&d, PARTY).landed, 1);
}
#[test]
fn a_first_look_has_no_previous_and_nothing_new() {
    let mut d = configured_domain(&[]);
    restore(&mut d, &env(), Stored::ProjectFacts(facts(PROJECT, 3)));
    let now = BEFORE;
    let out = look(&mut d, PARTY, now);
    let row = Looked { party: PARTY, project: PROJECT, previous: None, latest: now, seen: 3, next_seen: 3 };
    assert_eq!(
        rows(out).as_ref(),
        [
            Request::Save { record: Stored::Looked(row.clone()), release: None, read_afresh: None },
            marked(),
            Request::ApplicationDone { request: REQUEST, token: ANSWER },
        ]
    );
    assert_eq!(d.looked.get(&(PARTY, PROJECT)), Some(&row));
    assert_eq!(d.application_answer(ANSWER), Some(Answered::Looked(LookedAnswer::Looked)));
    let shown = view(&d, PARTY);
    assert_eq!((shown.previous, shown.landed.checked_sub(shown.seen)), (None, Some(0)));
}
#[test]
fn a_look_within_a_sitting_moves_latest_alone() {
    let mut d = restored();
    let now = after(1799);
    look(&mut d, PARTY, now);
    assert_eq!(
        d.looked.get(&(PARTY, PROJECT)),
        Some(&Looked { latest: now, next_seen: 3, ..look_row(PARTY, PROJECT) })
    );
}
#[test]
fn a_look_at_thirty_minutes_starts_a_sitting() {
    let mut d = restored();
    let now = after(1800);
    look(&mut d, PARTY, now);
    assert_eq!(
        d.looked.get(&(PARTY, PROJECT)),
        Some(&Looked { previous: Some(BEFORE), latest: now, seen: 2, next_seen: 3, ..look_row(PARTY, PROJECT) })
    );
}
#[test]
fn a_change_landed_between_sittings_reads_as_new() {
    let mut d = restored();
    look(&mut d, PARTY, after(1800));
    let shown = view(&d, PARTY);
    assert_eq!(shown.landed.checked_sub(shown.seen), Some(1));
}
#[test]
fn a_clock_behind_stays_in_the_sitting() {
    let mut d = restored();
    let now = Wall::EPOCH;
    look(&mut d, PARTY, now);
    assert_eq!(
        d.looked.get(&(PARTY, PROJECT)),
        Some(&Looked { latest: now, next_seen: 3, ..look_row(PARTY, PROJECT) })
    );
}
#[test]
fn past_the_bound_a_look_is_too_large_and_writes_nothing() {
    let mut d = domain();
    for project in 0..LOOKS {
        restore(&mut d, &env(), Stored::Looked(look_row(OTHER_PARTY, project)));
    }
    let out = look(&mut d, PARTY, BEFORE);
    assert_eq!(rows(out).as_ref(), [Request::ApplicationDone { request: REQUEST, token: ANSWER }]);
    assert_eq!(d.application_answer(ANSWER), Some(Answered::Looked(LookedAnswer::TooLarge)));
    // Updating a held row at the bound remains admitted.
    look(&mut d, OTHER_PARTY, after(1799));
    assert_eq!(d.looked.get(&(OTHER_PARTY, PROJECT)).unwrap().latest, after(1799));
}
#[test]
fn the_document_is_the_asking_partys_own() {
    let mut d = restored();
    let brief = Brief {
        sentence: Box::from(b"Landed".as_slice()),
        lede: Box::from(b"A change".as_slice()),
        at: BEFORE,
        landed: 3,
    };
    let running = StatusRunning { task: Some(TIER_TASK), landed: 3, since: BEFORE };
    let row = ProjectFacts { brief: Some(brief.clone()), running: Some(running), ..facts(PROJECT, 3) };
    restore(&mut d, &env(), Stored::ProjectFacts(row.clone()));
    assert_eq!(d.project_facts.get(&PROJECT), Some(&row));
    assert_eq!((view(&d, PARTY).previous, view(&d, PARTY).seen), (Some(Wall::EPOCH), 1));
    assert_eq!((view(&d, OTHER_PARTY).previous, view(&d, OTHER_PARTY).seen), (None, 3));
    assert_eq!(view(&d, PARTY).brief, Some(brief));
    for scope in [ApplicationScope::Own, ApplicationScope::Deployment, ApplicationScope::Project(PROJECT + 1)] {
        assert_eq!(d.document(status_subject(PROJECT), scope, PARTY, env().wall, &LIMITS), None);
    }
}
#[test]
fn queued_is_what_the_changes_document_counts() {
    let mut d = configured_domain(&[]);
    for task in [CHANGE, SECOND] {
        add_change(&mut d, task, b"main");
        let row = d.changes.get_mut(&task).unwrap();
        row.change.state = change::State::Queued { head: HEAD, ready_since: BEFORE };
        row.change.ready_since = Some(BEFORE);
    }
    let mut limits = LIMITS;
    limits.changes_rows = 1;
    let Document::Changes(changes) =
        d.document(changes_subject(PROJECT), ApplicationScope::Project(PROJECT), PARTY, env().wall, &limits).unwrap()
    else {
        panic!("changes")
    };
    let Document::Status(status) =
        d.document(status_subject(PROJECT), ApplicationScope::Project(PROJECT), PARTY, env().wall, &limits).unwrap()
    else {
        panic!("status")
    };
    assert_eq!(changes.more, 1);
    let mut listed = 0_usize;
    for queue in &changes.queues {
        listed = listed.checked_add(queue.changes.len()).unwrap();
    }
    assert_eq!(listed, 1);
    assert_eq!(status.queued, 2);
}
#[test]
fn red_is_the_home_landing_branchs_alone() {
    let mut d = configured_domain(&[]);
    add_change(&mut d, CHANGE, b"goal");
    d.changes.get_mut(&CHANGE).unwrap().base_repair = true;
    assert!(!view(&d, PARTY).red);
    add_change(&mut d, SECOND, b"main");
    d.changes.get_mut(&SECOND).unwrap().base_repair = true;
    assert!(view(&d, PARTY).red);
    assert_eq!(view(&d, PARTY).branch.as_ref(), b"main");
}
#[test]
fn no_home_has_no_branch_and_no_red() {
    let mut d = configured_domain(&[]);
    d.repositories.get_mut(&REPO).unwrap().home = false;
    add_change(&mut d, CHANGE, b"main");
    d.changes.get_mut(&CHANGE).unwrap().base_repair = true;
    let shown = view(&d, PARTY);
    assert_eq!((shown.landed, shown.last_landed, shown.brief, shown.previous, shown.seen), (0, None, None, None, 0));
    assert!(shown.branch.is_empty());
    assert!(!shown.red);
}
#[test]
fn erased_with_its_project() {
    let mut d = domain();
    for record in [
        Stored::ProjectFacts(facts(PROJECT, 3)),
        Stored::ProjectFacts(facts(PROJECT + 1, 4)),
        Stored::Looked(look_row(PARTY, PROJECT)),
        Stored::Looked(look_row(OTHER_PARTY, PROJECT)),
        Stored::Looked(look_row(PARTY, PROJECT + 1)),
    ] {
        restore(&mut d, &env(), record);
    }
    let out = outputs(&mut d, Event::ReleaseProject { project: PROJECT, request: REQUEST });
    assert_eq!(
        out.as_ref(),
        [
            Request::Erase { key: Key::ProjectFacts { project: PROJECT } },
            Request::Erase { key: Key::Looked { party: PARTY, project: PROJECT } },
            Request::Erase { key: Key::Looked { party: OTHER_PARTY, project: PROJECT } },
            marked(),
            Request::ProjectReleased { request: REQUEST },
        ]
    );
    assert_eq!(d.project_facts.len(), 1);
    assert_eq!(d.looked.len(), 1);
    assert_eq!(
        outputs(&mut d, Event::ReleaseProject { project: PROJECT, request: REQUEST }).as_ref(),
        [Request::ProjectReleased { request: REQUEST }]
    );
}
fn facts_record(project: u32) -> Stored {
    Stored::ProjectFacts(facts(project, 3))
}
fn looked_record(project: u32) -> Stored {
    Stored::Looked(look_row(PARTY, project))
}
type RestoreCase = (RecordKind, u32, fn(u32) -> Stored);
#[test]
fn restored_rows_past_their_bound_are_refused() {
    let cases: [RestoreCase; 2] =
        [(RecordKind::ProjectFacts, LIMITS.judge_projects, facts_record), (RecordKind::Looked, LOOKS, looked_record)];
    for (kind, bound, make) in cases {
        let mut d = domain();
        for project in 0..bound {
            let row = make(project);
            restore(&mut d, &env(), row);
        }
        assert_eq!(d.restore_fit_failure(), None);
        let row = make(bound);
        restore(&mut d, &env(), row);
        assert_eq!(
            d.restore_fit_failure(),
            Some(&RestoreFit {
                family: Some(kind),
                set: RestoreSet::Rows(kind),
                held: u64::from(bound),
                additional: 1,
                limit: u64::from(bound)
            })
        );
    }
}

#[test]
fn a_project_past_the_facts_bound_is_not_counted() {
    let mut d = configured_domain(&[]);
    for project in 0..LIMITS.judge_projects {
        restore(&mut d, &env(), Stored::ProjectFacts(facts(project, 3)));
    }
    let mut out = Queue::uncapped();
    status::landed(&mut d, BEFORE, REPO, &mut out);
    assert!(out.is_empty());
    assert_eq!(view(&d, PARTY).landed, 0);
    d.repositories.get_mut(&REPO).unwrap().project = 0;
    status::landed(&mut d, BEFORE, REPO, &mut out);
    assert_eq!(d.project_facts.get(&0).unwrap().landed, 4);
}

#[test]
fn a_repository_removed_counts_no_landing() {
    let mut d = configured_domain(&[]);
    d.repositories.remove(&REPO);
    let mut out = Queue::uncapped();
    status::landed(&mut d, BEFORE, REPO, &mut out);
    assert!(out.is_empty());
    assert!(d.project_facts.is_empty());
}
