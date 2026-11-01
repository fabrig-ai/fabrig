//! Project release regressions (contracts, 4).
use crate::*;
use alloc::boxed::Box;
use skein_lib::{Duration, Env, Queue, Time, Token, Wall};
use temper_engine_domain_forge_issues as issues;
use temper_engine_domain_forge_landing as policy;

const OTHER: skein_forge::api::Repository = skein_forge::api::Repository { forge: 1, repository: 3 };
const REQUEST: Token = Token::new(71);

fn release(d: &mut Domain) -> Box<[Request]> {
    outputs(d, Event::ReleaseProject { request: REQUEST, project: 5 })
}

#[test]
fn a_projects_repository_rows_are_erased_then_released() {
    let mut d = domain();
    let mut other = adopted();
    other.provider = OTHER;
    for row in [adopted(), other] {
        assert!(outputs(&mut d, Event::Restore { record: Stored::Repository(row) }).is_empty());
    }
    // Ended forges retain their repositories until the project's release too.
    d.ended.insert(1, (Box::new([]), None)).expect("forge room");
    assert_eq!(
        release(&mut d).as_ref(),
        &[
            Request::Erase { key: Key::Repository(OTHER) },
            Request::Erase { key: Key::Repository(REPO) },
            Request::ProjectReleased { request: REQUEST, project: 5 },
        ]
    );
    assert!(d.repositories.is_empty());
}

#[expect(clippy::too_many_lines, reason = "one fixture pairs each repository-linked family with its store key")]
fn linked_rows(repository: skein_forge::api::Repository, project: u32, task: u64) -> Box<[(Key, Stored)]> {
    let mut repo = adopted();
    repo.provider = repository;
    repo.project = project;
    let name = Name { forge: repository.forge, repository: repository.repository, what: What::Issue(task) };
    let topic = Topic::Ci { repository, head: [3; 32] };
    let receipt = ReceiptKey { project, repository, check: Check::Gate(b"check".as_slice().into()), key: [4; 32] };
    let mut entry = status(task, task);
    entry.repository = repository;
    let call = NamedCall { task, attempt: 1, completion: 1, position: 1 };
    let mut change = change_row();
    change.task = task;
    change.change.task = task;
    change.repository = repository;
    Box::new([
        (Key::Change(task), Stored::Change(change)),
        (Key::Repository(repository), Stored::Repository(repo.clone())),
        (
            Key::Ci { repository, head: [3; 32] },
            Stored::Ci(CiState { repository, head: [3; 32], status: skein_forge::api::Ci::Passed }),
        ),
        (
            Key::BranchHead(name.clone()),
            Stored::BranchHead(BranchHead { name: name.clone(), commit: [3; 32], owned: true }),
        ),
        (
            Key::PullState(name.clone()),
            Stored::PullState(PullState {
                name: name.clone(),
                head: [3; 32],
                ci: skein_forge::api::Ci::Passed,
                state: skein_forge::api::State::Open,
            }),
        ),
        (Key::Names(task), Stored::Names { task, resources: Box::new([name.clone()]) }),
        (Key::Hold(name.clone()), Stored::Hold(Hold { name, task, writer: None, drift: None })),
        (
            Key::Subscription { task, topic: topic.clone() },
            Stored::Subscription(Subscriber {
                task,
                number: 1,
                topic,
                own_change: None,
                goal_tasks: None,
                paths: Box::new([]),
            }),
        ),
        (
            Key::Issue(task),
            Stored::Issue(IssueRow {
                goal: task,
                repository,
                state: issues::Projected { closed: true, ..Default::default() },
                number: Some(1),
                pending: None,
                before: None,
                failed: false,
                desired: None,
            }),
        ),
        (
            Key::ProposedEffect(task),
            Stored::ProposedEffect(ProposedEffect { number: task, resource: What::Repository, entry: entry.clone() }),
        ),
        (Key::Entry(task), Stored::Entry { entry, held: true }),
        (Key::Call(call), Stored::Call { key: call, answer: CallPayload::Refused(skein_forge::api::Error::Refused) }),
        (
            Key::Landed([u8::try_from(project).expect("small fixture project"); 32]),
            Stored::Landed { task, commit: [u8::try_from(project).expect("small fixture project"); 32] },
        ),
        (
            Key::Receipt(receipt.clone()),
            Stored::Receipt {
                key: receipt,
                receipt: Receipt {
                    passed: true,
                    run: ReceiptRun { id: 1, number: 1, url: Box::new([]) },
                    candidate: [3; 32],
                    ended: Wall::EPOCH,
                },
            },
        ),
        (
            Key::Landing(task),
            Stored::Landing(LandingRow {
                landing: policy::Landing {
                    task,
                    project,
                    phase: policy::Phase::Landed,
                    members: Box::new([]),
                    preparation: 0,
                    stale_proofs: 0,
                },
                asks: Box::new([]),
                entries: Box::new([]),
            }),
        ),
        (
            Key::Application(task),
            Stored::Application {
                token: task,
                until: Wall::from_nanos(200_000_000_000),
                outcome: ApplicationOutcome::Adopted(AdoptedRepository {
                    repository,
                    owner: repo.owner,
                    name: repo.name,
                    role: repo.role,
                    home: repo.home,
                    kinds: repo.kinds,
                    protection: repo.protection,
                }),
            },
        ),
    ])
}

fn restore_rows(d: &mut Domain, rows: &[(Key, Stored)]) {
    for (_, row) in rows {
        assert!(outputs(d, Event::Restore { record: row.clone() }).is_empty());
    }
    assert_eq!(d.restore_fit_failure(), None);
}

#[test]
fn rows_linking_to_a_released_repository_go_with_it() {
    let mut d = domain();
    let rows = linked_rows(REPO, 5, 7);
    let other = linked_rows(OTHER, 6, 8);
    // Keep repository, CI, head and receipt rows from the other project.
    for index in [1, 2, 3, 13] {
        let (_, row) = &other[index];
        outputs(&mut d, Event::Restore { record: row.clone() });
    }
    restore_rows(&mut d, &rows);
    let out = release(&mut d);
    assert_eq!(out.last(), Some(&Request::ProjectReleased { request: REQUEST, project: 5 }));
    assert_eq!(out.len(), rows.len() + 1);
    let mut restored = domain();
    for (key, row) in &rows {
        assert!(out.contains(&Request::Erase { key: key.clone() }), "{key:?}");
        if !out.contains(&Request::Erase { key: key.clone() }) {
            outputs(&mut restored, Event::Restore { record: row.clone() });
        }
    }
    for index in [1, 2, 3, 13] {
        let (key, row) = &other[index];
        assert!(!out.contains(&Request::Erase { key: key.clone() }));
        outputs(&mut restored, Event::Restore { record: row.clone() });
    }
    restored.reconcile(&empty_tasks());
    assert_eq!(restored.reconciliation(), Ok(()));
    // In-memory rows go in the same decision as their store keys.
    assert!(d.names.is_empty());
    assert!(d.holds.is_empty());
    assert!(d.subscriptions.is_empty());
    assert!(d.pulls.is_empty());
    assert!(d.changes.is_empty());
    assert!(d.issues.is_empty());
    assert!(d.proposed_effects.is_empty());
    assert!(d.entries.is_empty());
    assert!(d.named_calls.is_empty());
    assert!(d.landed.is_empty());
    assert!(d.landing.rows.is_empty());
    assert!(d.application.kept.is_empty());
    assert_eq!(d.repositories.len(), 1);
    assert_eq!(d.ci.len(), 1);
    assert_eq!(d.heads.len(), 1);
    assert_eq!(d.landing.receipts.len(), 1);
    assert_eq!(release(&mut d).as_ref(), &[Request::ProjectReleased { request: REQUEST, project: 5 }]);
}

#[test]
fn another_projects_rows_stay() {
    let mut d = domain();
    let rows = linked_rows(REPO, 5, 7);
    let other = linked_rows(OTHER, 6, 8);
    restore_rows(&mut d, &rows);
    restore_rows(&mut d, &other);
    let out = release(&mut d);
    let mut restored = domain();
    for (key, row) in &other {
        assert!(!out.contains(&Request::Erase { key: key.clone() }), "{key:?}");
        outputs(&mut restored, Event::Restore { record: row.clone() });
    }
    assert_eq!(d.repositories.get(&OTHER), restored.repositories.get(&OTHER));
    assert_eq!(d.issues.get(&8), restored.issues.get(&8));
    assert_eq!(d.names.get(&8), restored.names.get(&8));
    assert_eq!(d.landing.rows.get(&8), restored.landing.rows.get(&8));
    assert_eq!(d.application_outcome(Token::new(8)), restored.application_outcome(Token::new(8)));
}

#[test]
fn a_second_release_answers_at_once() {
    let mut d = domain();
    let rows = linked_rows(REPO, 5, 7);
    restore_rows(&mut d, &rows);
    let erased = release(&mut d);
    // Replay the committed store after a cut between erasure and the answer.
    let mut restarted = domain();
    for (key, row) in &rows {
        if !erased.contains(&Request::Erase { key: key.clone() }) {
            outputs(&mut restarted, Event::Restore { record: row.clone() });
        }
    }
    restarted.reconcile(&empty_tasks());
    assert_eq!(release(&mut restarted).as_ref(), &[Request::ProjectReleased { request: REQUEST, project: 5 }]);
    assert_eq!(release(&mut domain()).as_ref(), &[Request::ProjectReleased { request: REQUEST, project: 5 }]);
}

#[test]
fn project_rows_are_erased_without_a_repository_row() {
    let mut d = domain();
    let rows = linked_rows(REPO, 5, 7);
    for index in [13, 14] {
        let (_, row) = &rows[index];
        outputs(&mut d, Event::Restore { record: row.clone() });
    }
    let out = release(&mut d);
    assert_eq!(out.len(), 3);
    assert!(out.contains(&Request::Erase { key: rows[13].0.clone() }));
    assert!(out.contains(&Request::Erase { key: rows[14].0.clone() }));
    assert_eq!(out.last(), Some(&Request::ProjectReleased { request: REQUEST, project: 5 }));
    assert!(d.landing.rows.is_empty());
    assert!(d.landing.receipts.is_empty());
}

#[cfg(debug_assertions)]
#[test]
#[should_panic(expected = "flight of task 7 while releasing project 5")]
fn a_flight_on_an_erased_task_breaks_the_contract() {
    let mut d = domain();
    restore_rows(&mut d, &linked_rows(REPO, 5, 7));
    outputs(&mut d, Event::Restore { record: Stored::Flight { task: 7, entry: 7 } });
    release(&mut d);
}

#[cfg(debug_assertions)]
#[test]
#[should_panic(expected = "unfinished release of task 7 in project 5")]
fn an_unfinished_task_release_breaks_the_contract() {
    let mut d = domain();
    restore_rows(&mut d, &linked_rows(REPO, 5, 7));
    outputs(
        &mut d,
        Event::Restore {
            record: Stored::Release(ReleaseRow {
                task: 7,
                root: 7,
                ending: ReleaseEnding::Done,
                close_pull: None,
                comment: None,
                pending: None,
                failed: false,
                effects_settled: true,
                releasing: true,
            }),
        },
    );
    release(&mut d);
}

const REPO: skein_forge::api::Repository = skein_forge::api::Repository { forge: 1, repository: 2 };
const CLIENT: skein_forge_client::Limits = skein_forge_client::Limits {
    pending: 4,
    calls: 2,
    rate: 100,
    reserve: 0,
    window: Duration::from_secs(10),
    op_bytes: 512,
    answer_bytes: 4096,
    rows: 8,
    inbox: 16,
    read_attempts: 3,
    backoff: Duration::from_secs(1),
    backoff_max: Duration::from_secs(4),
    entries: 8,
    write_attempts: 3,
    lifetime: Duration::from_secs(10),
    clock_margin: Duration::from_secs(1),
    resources: 8,
    repositories: 2,
    poll: Duration::from_secs(5),
    poll_max: Duration::from_secs(30),
    hinted: Duration::from_secs(1),
    slow: Duration::from_secs(40),
    settle: Duration::from_secs(30),
    facts: 16,
};
const LIMITS: Limits = Limits {
    forges: 4,
    repositories: 2,
    tasks: 8,
    holds: 8,
    subscriptions: 8,
    entries: 8,
    named_calls: 16,
    changes: 8,
    issues: 8,
    resources_per_task: 4,
    paths_per_subscription: 4,
    // forge/host/owner/repository/branch/temper/root/marker-task.
    authority_segments: 8,
    name_bytes: 128,
    output: 80,
    facts: 16,
    adoptions: 2,
    applications: 4,
    bindings: 4,
    collaborators: 8,
    landings: 8,
    landing_members: 2,
    gates: 2,
    receipts_per_gate: 4,
    ignored_paths: 4,
    key_entries: 16,
    key_path_bytes: 1024,
    trees_in_flight: 2,
    judge_projects: 2,
    judge_criteria: 8,
    brief_sections: 32,
    brief_bytes: 4096,
    issue_policy: temper_engine_domain_forge_issues::Limits {
        plan_items: 8,
        milestones: 8,
        title_bytes: 128,
        body_bytes: 512,
        comment_bytes: 256,
        interval: Duration::from_secs(5),
    },
    change_policy: temper_engine_domain_forge_change::Limits {
        repairs: 3,
        resolutions: 3,
        updates: 3,
        stall: Duration::from_secs(30),
        gates: 8,
        clean_heads: 8,
    },
    queue_window: Duration::from_secs(30),
    client: CLIENT,
};

fn env() -> Env<Limits> {
    Env { now: Time::from_nanos(100_000_000_000), wall: Wall::from_nanos(100_000_000_000), limits: LIMITS }
}

fn domain() -> Domain {
    Domain::new(
        &LIMITS,
        4,
        skein_forge_client::Config {
            namespace: Box::from(&b"test"[..]),
            writers: Box::new([skein_forge_client::Writer { forge: 1, author: 7 }]),
        },
        &[Forge::connected(1, Kind::Forgejo, &[])],
        false,
    )
    .expect("valid connector")
}
fn outputs(d: &mut Domain, event: Event) -> Box<[Request]> {
    let mut out = Queue::uncapped();
    step(d, &env(), event, &mut out);
    let mut values = skein_lib::List::with_capacity(out.len());
    for _ in 0..out.len() {
        values.push(out.pop().expect("counted output")).expect("sized from queue");
    }
    values.into_boxed()
}
fn adopted() -> Repository {
    Repository {
        project: 5,
        home: true,
        provider: REPO,
        host: Box::from(&b"forge.example"[..]),
        owner: Box::from(&b"org"[..]),
        name: Box::from(&b"repo"[..]),
        prefix: Box::from(&b"temper/"[..]),
        role: Role::Owned,
        ci: true,
        checks: Box::new([]),
        kinds: Kinds {
            read: true,
            push: true,
            open: true,
            land: true,
            review: true,
            status: true,
            comment: true,
            issue: true,
            branch: true,
        },
        protection: Protection::Absent,
        settings: skein_forge::api::Settings {
            default_branch: Box::from(&b"main"[..]),
            merge: true,
            squash: true,
            rebase: false,
        },
    }
}
/// A commit status write: a set, made without a find.
fn status(number: u64, task: u64) -> skein_forge_client::Entry {
    skein_forge_client::Entry {
        number,
        task,
        repository: REPO,
        effect: skein_forge_client::Effect {
            write: skein_forge::api::Write::Status {
                commit: [3; 32],
                context: Box::from(&b"test"[..]),
                check: skein_forge::api::Check::Passed,
                description: Box::new([]),
                link: Box::new([]),
            },
            condition: skein_forge_client::Condition::None,
        },
        start: None,
        attempt: None,
        failures: 0,
        queued: false,
    }
}
fn empty_tasks() -> jig_domain_tasks::Domain {
    use jig_domain_tasks as tasks;
    let retry = tasks::Retry { retries: 1, base: Duration::from_millis(10), max: Duration::from_secs(1) };
    let limits = tasks::Limits {
        placed_holds: 1,
        project_placed_holds: 1,
        party_placed_holds: 1,
        exemptions: 1,
        circle: 8,
        tasks: 2,
        funders: 4,
        project_tasks: 2,
        project_goals: 8,
        tree_tasks: 2,
        depth: 1,
        delegates: 1,
        references: 2,
        subscriptions: 2,
        batch: 1,
        dependencies: 1,
        holdings: 2,
        hold_kinds: 2,
        resource_reports: 64,
        pools: 2,
        hold_segments: 4,
        hold_bytes: 128,
        hold_waiters: 2,
        grace: Duration::from_secs(600),
        hold_wait: Duration::from_secs(10),
        inputs: 1,
        spec_bytes: 64,
        parameters: 1,
        result_bytes: 128,
        run_inbox: 4,
        inbox_messages: 4,
        inbox_bytes: 192,
        message_bytes: 64,
        proposal_stall: Duration::from_millis(10),
        escalation_stall: Duration::from_secs(3600),
        person_stall: Duration::from_secs(3600),
        saved_resources: 2,
        contract_choices: 1,
        charters: 1,
        revisions: 4,
        versions: 8,
        authority_grants: 1,
        authority_segments: 1,
        authority_bytes: 32,
        executor_kinds: 2,
        retries: tasks::Retries {
            transient: retry,
            permanent: retry,
            run: retry,
            agent: retry,
            lost: retry,
            invalid: retry,
        },
        facts: 2,
        room_refusals: 2,
    };
    tasks::Domain::new(&limits, 1, 1, Box::new([]), Box::new([]))
}

fn change_row() -> ChangeRow {
    use temper_engine_domain_forge_change as change;
    ChangeRow {
        task: 7,
        repository: REPO,
        branch: Box::from(b"temper/51".as_slice()),
        base: Box::from(b"main".as_slice()),
        title: Box::from(b"Small fix".as_slice()),
        body: Box::new([]),
        priority: 1,
        ci: None,
        change: change::Change {
            way: temper_engine_domain_forge_change::Way::Queue,
            landing: None,
            task: 7,
            state: change::State::Opening { requested: true },
            gates: Box::new([]),
            clean: Box::new([]),
            gate_failures: Box::new([]),
            released: None,
            repairs: 0,
            resolutions: 0,
            updates: 0,
            since: env().wall,
            ready_since: None,
            owns_turn: false,
            last_head: Some([3; 32]),
        },
        pull: None,
        pending: Some(7),
        effect: change::EffectResult::Pending,
        delegate: None,
        delegate_status: change::Status::Passed,
        verdicts: Box::new([]),
        result: ChangeResult::empty(),
        gate_remarks: Box::new([]),
        drift: None,
        base_repair: false,
        queue_repair: None,
        queue_repairs: 0,
        producing_charter: 2,
        review_charter: 4,
        effect_account: None,
        producer_failure: None,
    }
}
