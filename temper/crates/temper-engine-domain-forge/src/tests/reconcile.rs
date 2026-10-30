use super::*;
use crate::reconcile::{MissingLink, Need, Reconciliation};
use alloc::boxed::Box;
use skein_lib::{Time, Wall};
use skein_world::domain::heap::Meter;

const TASK: u64 = 7;
const ENTRY: u64 = 11;
const OTHER_TASK: u64 = 99;
const OTHER_FORGE: u16 = 99;
const OTHER_REPOSITORY: skein_forge::api::Repository = skein_forge::api::Repository { forge: 1, repository: 99 };

fn resolved(d: &Domain, row: &Stored, live: bool) -> bool {
    let tasks: &[u64] = if live { &[TASK] } else { &[] };
    crate::reconcile::references_resolve(d, &LIMITS, row, tasks)
}
fn held_domain() -> Domain {
    let mut d = domain();
    outputs(&mut d, Event::Restore { record: Stored::Repository(adopted()) });
    outputs(&mut d, Event::Restore { record: Stored::Entry { entry: status(ENTRY, TASK), held: true } });
    d
}
fn issue(closed: bool) -> Stored {
    Stored::Issue(IssueRow {
        goal: TASK,
        repository: REPO,
        state: temper_engine_domain_forge_issues::Projected { closed, ..Default::default() },
        number: None,
        pending: None,
        before: None,
        failed: false,
        desired: None,
    })
}

#[test]
fn an_ended_goals_pending_projection_entry_restores() {
    for (pending, repository, expected) in [
        (Some(ENTRY), REPO, true),
        (Some(ENTRY + 1), REPO, false),
        (None, REPO, false),
        (Some(ENTRY), OTHER_REPOSITORY, false),
    ] {
        let (d, entry) = closing_history(pending, repository);
        assert_eq!(resolved(&d, &Stored::Entry { entry, held: false }, false), expected);
        assert_eq!(resolved(&d, &Stored::Flight { task: TASK, entry: ENTRY }, false), expected);
    }
}
fn closing_history(
    pending: Option<u64>,
    repository: skein_forge::api::Repository,
) -> (Domain, skein_forge_client::Entry) {
    let mut d = domain();
    let mut entry = held_issue();
    entry.effect.write = skein_forge::api::Write::Post {
        number: 1,
        key: Box::from(b"milestone".as_slice()),
        body: Box::from(b"Goal finished".as_slice()),
    };
    outputs(&mut d, Event::Restore { record: Stored::Repository(adopted()) });
    outputs(&mut d, Event::Restore { record: Stored::Entry { entry: entry.clone(), held: false } });
    let Stored::Issue(mut row) = issue(false) else { unreachable!("issue fixture") };
    row.pending = pending;
    row.repository = repository;
    row.number = Some(1);
    row.state.opened = true;
    row.desired = Some(issues::GoalView {
        phase: issues::Phase::Done,
        goal: TASK,
        repository: u64::from(REPO.repository),
        title: "Work".into(),
        goal_text: "Complete the work".into(),
        plan: Box::new([]),
        milestones: Box::new([]),
        finished: Some("Goal finished".into()),
    });
    outputs(&mut d, Event::Restore { record: Stored::Issue(row) });
    (d, entry)
}

fn release() -> Stored {
    Stored::Release(ReleaseRow {
        task: TASK,
        root: TASK,
        ending: ReleaseEnding::Done,
        close_pull: None,
        comment: None,
        pending: None,
        failed: false,
        effects_settled: true,
        releasing: true,
    })
}
fn subscription() -> Stored {
    Stored::Subscription(Subscriber {
        task: TASK,
        number: 1,
        topic: Topic::Ci { repository: REPO, head: [3; 32] },
        own_change: Some(OTHER_TASK),
        goal_tasks: Some(Box::new([OTHER_TASK])),
        paths: Box::new([]),
    })
}
fn hold(task: u64, writer: Option<Writer>) -> Stored {
    Stored::Hold(Hold { name: branch(), task, writer, drift: None })
}
fn owner_rows() -> [Stored; 8] {
    [
        Stored::Call {
            key: NamedCall { task: TASK, attempt: 1, completion: 1, position: 1 },
            answer: CallPayload::Refused(skein_forge::api::Error::Refused),
        },
        Stored::ProposedEffect(ProposedEffect { number: 1, resource: What::Repository, entry: status(ENTRY, TASK) }),
        hold(TASK, Some(Writer::Lost { task: TASK, attempt: 1 })),
        Stored::Names { task: TASK, resources: Box::new([branch()]) },
        subscription(),
        Stored::Entry { entry: status(ENTRY, TASK), held: true },
        issue(false),
        release(),
    ]
}
fn repository_rows(repository: skein_forge::api::Repository) -> [Stored; 7] {
    let mut name = branch();
    name.forge = repository.forge;
    name.repository = repository.repository;
    let mut change = fixtures::opening_row();
    change.repository = repository;
    [
        Stored::BranchHead(BranchHead { name: name.clone(), commit: [3; 32], owned: true }),
        Stored::PullState(PullState {
            name,
            head: [3; 32],
            ci: skein_forge::api::Ci::Passed,
            state: skein_forge::api::State::Open,
        }),
        Stored::Ci(CiState { repository, head: [3; 32], status: skein_forge::api::Ci::Passed }),
        Stored::Client(skein_forge_client::Stored::Repository(skein_forge_client::RepositoryRecord {
            repository,
            mark: None,
            clock: Time::ZERO,
        })),
        Stored::Client(skein_forge_client::Stored::Live(skein_forge_client::LiveRecord {
            watch: skein_forge_client::Watch {
                resource: skein_forge_client::Resource { repository, what: skein_forge_client::What::Repository },
                participating: false,
            },
            position: skein_forge_client::Position { at: Time::ZERO, comment: 0, review: 0, review_page: 0 },
            listed: Time::ZERO,
            comment_after: 0,
            comment_page: 0,
            comment_scanning: false,
            comment_repair: false,
            review_page: 0,
            comments: Box::new([]),
            reviews: Box::new([]),
            cached: Box::new(skein_forge_client::Cached { item: None, pull: None, tip: None }),
            pushed: None,
            echoes: Box::new([]),
        })),
        Stored::Change(change),
        Stored::Application {
            token: 1,
            until: Wall::EPOCH,
            outcome: ApplicationOutcome::Adopted(AdoptedRepository {
                repository,
                owner: b"org".as_slice().into(),
                name: b"repo".as_slice().into(),
                role: Role::Owned,
                home: true,
                kinds: adopted().kinds,
                protection: Protection::Absent,
            }),
        },
    ]
}
#[test]
fn live_owners_resolve_for_every_live_record_family() {
    let d = held_domain();
    for row in owner_rows() {
        assert!(resolved(&d, &row, true), "{row:?}");
    }
}
#[test]
fn absent_owners_refuse_for_every_live_record_family() {
    let d = held_domain();
    for row in owner_rows() {
        assert!(!resolved(&d, &row, false), "{row:?}");
    }
}
#[test]
fn held_repositories_resolve_for_every_repository_record_family() {
    let d = held_domain();
    for row in repository_rows(REPO) {
        assert!(resolved(&d, &row, true), "{row:?}");
    }
}
#[test]
fn absent_repositories_refuse_for_every_repository_record_family() {
    let d = held_domain();
    for row in repository_rows(OTHER_REPOSITORY) {
        assert!(!resolved(&d, &row, true), "{row:?}");
    }
}
#[test]
fn historical_records_need_no_live_task_or_member() {
    let d = held_domain();
    for row in [
        Stored::Landed { task: OTHER_TASK, commit: [9; 32] },
        Stored::Change(fixtures::opening_row()),
        issue(true),
        Stored::Binding { address: b"former@example.com".as_slice().into(), subject: b"former".as_slice().into() },
        Stored::DemoAct(DemoAct { key: [1; 16], act: DemoActKind::Close { project: 1, task: OTHER_TASK } }),
        Stored::DemoAct(DemoAct { key: [2; 16], act: DemoActKind::RemoveAccount { account: 99 } }),
        Stored::Application {
            token: 1,
            until: Wall::EPOCH,
            outcome: ApplicationOutcome::ForgeRefused(AdoptionRefusal::NotFound),
        },
    ] {
        assert!(resolved(&d, &row, false), "{row:?}");
    }
}
#[test]
fn configured_forges_resolve_without_being_connected() {
    let d = held_domain();
    for row in [Stored::Repository(adopted()), Stored::Writer(skein_forge_client::Writer { forge: 1, author: 7 })] {
        assert!(resolved(&d, &row, false), "{row:?}");
    }
}
#[test]
fn a_restored_writer_does_not_configure_a_missing_forge() {
    let mut d = held_domain();
    let mut repository = adopted();
    repository.provider.forge = OTHER_FORGE;
    let writer = skein_forge_client::Writer { forge: OTHER_FORGE, author: 7 };
    outputs(&mut d, Event::Restore { record: Stored::Writer(writer) });
    for row in [Stored::Repository(repository), Stored::Writer(writer)] {
        assert!(!resolved(&d, &row, true), "{row:?}");
    }
}
#[test]
fn flights_resolve_only_to_the_entry_of_the_same_task() {
    let d = held_domain();
    assert!(resolved(&d, &Stored::Flight { task: TASK, entry: ENTRY }, true));
}
#[test]
fn flights_refuse_an_absent_entry_or_another_tasks_entry() {
    let d = held_domain();
    for row in [Stored::Flight { task: TASK, entry: 99 }, Stored::Flight { task: OTHER_TASK, entry: ENTRY }] {
        assert!(!resolved(&d, &row, true), "{row:?}");
    }
}
#[test]
fn a_demo_record_requires_the_configured_demo() {
    let d = held_domain();
    assert!(!resolved(&d, &Stored::Demo(Demo { forge: 1, state: DemoState::Ended }), false));
}

// Measured peak: 3,072 bytes; 25% headroom, rounded up to two significant digits.
const REFERENCE_HEAP_CEILING: u64 = 3_900;

#[test]
fn retained_reconciliation_references_fit_their_heap_ceiling() {
    let mut rows = skein_lib::List::with_capacity(LIMITS.tasks);
    for task in 1..=u64::from(LIMITS.tasks) {
        let mut resources = skein_lib::List::with_capacity(LIMITS.resources_per_task);
        for _ in 0..LIMITS.resources_per_task {
            resources
                .push(Name { forge: REPO.forge, repository: REPO.repository, what: What::Issue(task) })
                .expect("configured resources");
        }
        rows.push(Stored::Names { task, resources: resources.into_boxed() }).expect("configured tasks");
    }
    let meter = Meter::new();
    meter.start();
    let mut reconciliation = Reconciliation::new(&LIMITS);
    for row in &rows {
        reconciliation.restore(row).expect("fixture fits references");
    }
    let measured = meter.end();
    meter.check(measured, REFERENCE_HEAP_CEILING, &LIMITS);
    core::hint::black_box(&reconciliation);
    assert!(meter.held() > 0, "restoration retains references");
}

// Measured peak: 9,139 bytes; 25% headroom, rounded up to two significant digits.
const RECONCILIATION_HEAP_CEILING: u64 = 12_000;

#[test]
fn populated_reconciliation_owner_fits_its_heap_ceiling() {
    let meter = Meter::new();
    meter.start();
    // The boxed owner includes inline reconciliation storage in the measured heap.
    let d = Box::new(held_domain());
    let measured = meter.end();
    meter.check(measured, RECONCILIATION_HEAP_CEILING, &LIMITS);
    assert!(resolved(&d, &Stored::Flight { task: TASK, entry: ENTRY }, true));
}

// Measured peak: 9,170 bytes; 25% headroom, rounded up to two significant digits.
const DEMO_HEAP_CEILING: u64 = 12_000;

#[test]
fn retained_demo_and_its_owner_fit_their_heap_ceiling() {
    let meter = Meter::new();
    meter.start();
    // The boxed owner includes its optional demo record in the measured heap.
    let d = Box::new(demo_domain());
    let measured = meter.end();
    meter.check(measured, DEMO_HEAP_CEILING, &LIMITS);
    assert!(resolved(&d, &Stored::Demo(Demo { forge: 1, state: DemoState::Ended }), false));
}

fn demo_domain() -> Domain {
    let mut d = held_domain();
    d.configure_demo(Some(DemoConfig {
        provider: 1,
        adoption: fixtures::adoption(),
        deadline: Duration::from_secs(10),
    }));
    outputs(&mut d, Event::Restore { record: Stored::Demo(Demo { forge: 1, state: DemoState::Ended }) });
    d
}
#[test]
fn the_configured_ended_demo_and_its_retired_owners_resolve() {
    let d = demo_domain();
    for row in [
        Stored::Demo(Demo { forge: 1, state: DemoState::Ended }),
        Stored::Names { task: OTHER_TASK, resources: Box::new([branch()]) },
        hold(OTHER_TASK, Some(Writer::Lost { task: OTHER_TASK, attempt: 1 })),
        subscription(),
        Stored::Entry { entry: status(ENTRY, OTHER_TASK), held: true },
        issue(false),
        Stored::Flight { task: TASK, entry: ENTRY },
    ] {
        assert!(resolved(&d, &row, false), "{row:?}");
    }
}
#[test]
fn an_ended_demo_does_not_resolve_a_different_demo_record() {
    assert!(!resolved(&demo_domain(), &Stored::Demo(Demo { forge: OTHER_FORGE, state: DemoState::Ended }), false));
}
#[test]
fn a_mixed_resource_set_still_requires_its_live_owner() {
    let d = demo_domain();
    let mut other = branch();
    other.forge = OTHER_FORGE;
    assert!(!resolved(&d, &Stored::Names { task: OTHER_TASK, resources: Box::new([branch(), other]) }, false));
}

#[test]
fn a_held_writer_must_resolve_independently_of_the_owner() {
    let d = held_domain();
    let row = hold(TASK, Some(Writer::Run { task: OTHER_TASK, attempt: 1 }));
    assert!(!resolved(&d, &row, true));
}
#[test]
fn a_release_root_must_resolve_independently_of_the_closing_task() {
    let d = held_domain();
    let Stored::Release(mut row) = release() else { unreachable!("release fixture") };
    row.root = OTHER_TASK;
    assert!(!resolved(&d, &Stored::Release(row), true));
}
#[test]
fn every_resource_bearing_owner_refuses_a_missing_repository() {
    let d = domain();
    // Call has no repository. Release's task/root are checked separately.
    for row in [
        Stored::ProposedEffect(ProposedEffect { number: 1, resource: What::Repository, entry: status(ENTRY, TASK) }),
        hold(TASK, None),
        Stored::Names { task: TASK, resources: Box::new([branch()]) },
        subscription(),
        Stored::Entry { entry: status(ENTRY, TASK), held: true },
        issue(false),
    ] {
        assert!(!resolved(&d, &row, true), "{row:?}");
    }
}

#[test]
fn a_flight_on_a_live_forge_requires_its_entrys_task() {
    assert!(!resolved(&held_domain(), &Stored::Flight { task: TASK, entry: ENTRY }, false));
}

// Empty restored tasks use the core's small task fixture limits; no charter is needed.
pub(super) fn empty_tasks() -> jig_domain_tasks::Domain {
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

#[test]
fn production_link_asserts_a_missing_task() {
    let mut d = domain();
    outputs(&mut d, Event::Restore { record: Stored::Names { task: TASK, resources: Box::new([]) } });
    assert_eq!(
        d.missing_link(&empty_tasks()),
        Some(MissingLink {
            kind: RecordKind::Names,
            key: Some(Key::Names(TASK)),
            need: Need::Task { task: TASK, retired: None }
        })
    );
}

#[test]
fn production_link_asserts_a_missing_repository() {
    let mut d = domain();
    outputs(
        &mut d,
        Event::Restore {
            record: Stored::Ci(CiState {
                repository: OTHER_REPOSITORY,
                head: [3; 32],
                status: skein_forge::api::Ci::Passed,
            }),
        },
    );
    assert_eq!(
        d.missing_link(&empty_tasks()),
        Some(MissingLink {
            kind: RecordKind::Ci,
            key: Some(Key::Ci { repository: OTHER_REPOSITORY, head: [3; 32] }),
            need: Need::Repository(OTHER_REPOSITORY)
        })
    );
}

#[test]
fn production_link_asserts_a_missing_configured_forge() {
    let mut d = domain();
    let mut repository = adopted();
    repository.provider.forge = OTHER_FORGE;
    let key = Some(Key::Repository(repository.provider));
    outputs(&mut d, Event::Restore { record: Stored::Repository(repository) });
    assert_eq!(
        d.missing_link(&empty_tasks()),
        Some(MissingLink { kind: RecordKind::Repository, key, need: Need::Forge(OTHER_FORGE) })
    );
}

#[test]
fn production_link_asserts_a_missing_flight_entry() {
    let mut d = domain();
    outputs(&mut d, Event::Restore { record: Stored::Flight { task: TASK, entry: ENTRY } });
    assert_eq!(
        d.missing_link(&empty_tasks()),
        Some(MissingLink {
            kind: RecordKind::Flight,
            key: Some(Key::Flight { task: TASK, entry: ENTRY }),
            need: Need::Entry { task: TASK, entry: ENTRY }
        })
    );
}

fn failed_history(desired: bool) -> Stored {
    let Stored::Issue(mut row) = issue(false) else { unreachable!("issue fixture") };
    row.failed = true;
    if desired {
        row.desired = Some(issues::GoalView {
            phase: issues::Phase::Active,
            goal: TASK,
            repository: u64::from(REPO.repository),
            title: "Work".into(),
            goal_text: "Complete the work".into(),
            plan: Box::new([]),
            milestones: Box::new([]),
            finished: None,
        });
    }
    Stored::Issue(row)
}

#[test]
fn production_link_keeps_failed_issue_history_without_a_goal() {
    for desired in [false, true] {
        let mut d = domain();
        outputs(&mut d, Event::Restore { record: Stored::Repository(adopted()) });
        outputs(&mut d, Event::Restore { record: failed_history(desired) });
        d.reconcile(&empty_tasks());
        assert_eq!(d.reconciliation(), Ok(()));
        let out = outputs(&mut d, Event::Restored { clock: skein_forge_client::RecoveryClock::Wall });
        assert!(!has_projection_effect(&out));
        assert!(d.issue(TASK).expect("retained diagnostic").failed);
    }
}

#[test]
fn production_link_requires_a_failed_issues_repository() {
    let mut d = domain();
    outputs(&mut d, Event::Restore { record: failed_history(true) });
    assert_eq!(
        d.missing_link(&empty_tasks()),
        Some(MissingLink { kind: RecordKind::Issue, key: Some(Key::Issue(TASK)), need: Need::Repository(REPO) })
    );
}

#[test]
fn production_link_preserves_pending_closing_entry_and_flight() {
    let (mut d, _) = closing_history(Some(ENTRY), REPO);
    outputs(&mut d, Event::Restore { record: Stored::Flight { task: TASK, entry: ENTRY } });
    d.reconcile(&empty_tasks());
    assert_eq!(d.reconciliation(), Ok(()));
}

#[test]
fn production_link_preserves_retired_demo_owners() {
    let mut d = demo_domain();
    for record in [
        Stored::Names { task: OTHER_TASK, resources: Box::new([branch()]) },
        hold(OTHER_TASK, Some(Writer::Lost { task: OTHER_TASK, attempt: 1 })),
        subscription(),
        issue(false),
        Stored::Flight { task: TASK, entry: ENTRY },
    ] {
        outputs(&mut d, Event::Restore { record });
    }
    d.reconcile(&empty_tasks());
    assert_eq!(d.reconciliation(), Ok(()));
}

#[test]
fn production_link_requires_a_live_unfinished_issues_goal() {
    let mut d = domain();
    outputs(&mut d, Event::Restore { record: Stored::Repository(adopted()) });
    outputs(&mut d, Event::Restore { record: issue(false) });
    assert_eq!(
        d.missing_link(&empty_tasks()),
        Some(MissingLink {
            kind: RecordKind::Issue,
            key: Some(Key::Issue(TASK)),
            need: Need::Task { task: TASK, retired: Some(REPO.forge) }
        })
    );
}

#[test]
#[should_panic(expected = "forge link")]
fn reconcile_panics_on_a_missing_link() {
    let mut d = domain();
    outputs(&mut d, Event::Restore { record: Stored::Names { task: TASK, resources: Box::new([]) } });
    d.reconcile(&empty_tasks());
}

#[test]
fn all_made_links_have_no_missing_link() {
    let (mut d, _) = closing_history(Some(ENTRY), REPO);
    outputs(&mut d, Event::Restore { record: Stored::Flight { task: TASK, entry: ENTRY } });
    assert_eq!(d.missing_link(&empty_tasks()), None);
}

#[test]
fn a_missing_link_names_its_row_after_rows_with_multiple_or_no_links() {
    let (mut d, _) = closing_history(Some(ENTRY), REPO);
    outputs(&mut d, Event::Restore { record: Stored::Landed { task: TASK, commit: [9; 32] } });
    outputs(&mut d, Event::Restore { record: Stored::Names { task: OTHER_TASK, resources: Box::new([]) } });
    assert_eq!(
        d.missing_link(&empty_tasks()),
        Some(MissingLink {
            kind: RecordKind::Names,
            key: Some(Key::Names(OTHER_TASK)),
            need: Need::Task { task: OTHER_TASK, retired: None },
        })
    );
}

#[test]
fn calls_with_the_same_target_keep_the_first_restored_keys_identity() {
    const FIRST_ATTEMPT: u64 = 2;
    const LOWER_ATTEMPT: u64 = 1;
    let mut d = domain();
    for attempt in [FIRST_ATTEMPT, LOWER_ATTEMPT, LOWER_ATTEMPT] {
        outputs(
            &mut d,
            Event::Restore {
                record: Stored::Call {
                    key: NamedCall { task: TASK, attempt, completion: 1, position: 1 },
                    answer: CallPayload::Refused(skein_forge::api::Error::Refused),
                },
            },
        );
    }
    assert_eq!(
        d.missing_link(&empty_tasks()),
        Some(MissingLink {
            kind: RecordKind::Call,
            key: Some(Key::Call(NamedCall { task: TASK, attempt: FIRST_ATTEMPT, completion: 1, position: 1 })),
            need: Need::Task { task: TASK, retired: None },
        })
    );
}

#[test]
fn opaque_client_links_report_repository_and_writer_keys_but_cannot_recover_live_keys() {
    for row in repository_rows(OTHER_REPOSITORY) {
        if let Stored::Client(client) = row {
            let expected = match &client {
                skein_forge_client::Stored::Live(_) => None,
                skein_forge_client::Stored::Repository(row) => {
                    Some(Key::Client(skein_forge_client::Key::Repository(row.repository)))
                }
                skein_forge_client::Stored::Writer(_) => unreachable!("repository fixtures"),
            };
            let mut d = domain();
            outputs(&mut d, Event::Restore { record: Stored::Client(client) });
            assert_eq!(
                d.missing_link(&empty_tasks()),
                Some(MissingLink { kind: RecordKind::Client, key: expected, need: Need::Repository(OTHER_REPOSITORY) })
            );
        }
    }
    let mut d = domain();
    outputs(
        &mut d,
        Event::Restore {
            record: Stored::Client(skein_forge_client::Stored::Writer(skein_forge_client::Writer {
                forge: OTHER_FORGE,
                author: 1,
            })),
        },
    );
    assert_eq!(
        d.missing_link(&empty_tasks()),
        Some(MissingLink {
            kind: RecordKind::Client,
            key: Some(Key::Writer(OTHER_FORGE)),
            need: Need::Forge(OTHER_FORGE),
        })
    );
}

#[test]
fn references_keep_the_base_footprint() {
    assert_eq!(size_of::<crate::reconcile::Reference>(), 32);
}
