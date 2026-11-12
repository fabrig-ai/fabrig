//! Independent whole-carrier proofs through the scripted peer's public entries.
use super::*;

pub(super) fn empty() -> Deployment {
    Deployment {
        providers: List::with_capacity(0),
        providers_cut: false,
        key_bytes: 256,
        readiness: List::with_capacity(0),
        readiness_cut: false,
        store: StoreFill { bytes: 0, mark: 0 },
        hosts: List::with_capacity(2),
        attempts: List::with_capacity(2),
        accounts: List::with_capacity(2),
        tasks: Capacity { used: 0, limit: u32::MAX },
        workers: Capacity { used: u32::MAX, limit: 1 },
        attempts_capacity: Capacity { used: 7, limit: 11 },
        waiting_attempts: 19,
        proposals: Capacity { used: 3, limit: 17 },
        projects: List::with_capacity(2),
        grace_bound: Duration::from_nanos(u64::MAX),
        hello_refusals: List::with_capacity(2),
        hosts_forgotten: u64::MAX,
        hello_refusals_forgotten: 1,
        person_room_refusals: List::with_capacity(2),
        person_room_refusals_forgotten: 2,
        person_answer_retention: Duration::from_nanos(5),
        task_room_refusals: List::with_capacity(2),
        task_room_refusals_forgotten: 3,
        cut: true,
        connectors: List::with_capacity(2),
        connectors_cut: false,
    }
}
fn chip(number: u64) -> Chip {
    Chip {
        task: number,
        title: DocumentText::complete(text(b"task")),
        executor: ChipExecutor::Agent { charter: 1 },
        phase: Phase::Waiting,
        details: Box::new(chip_details(ChipPhase::Waiting { dependencies: list(vec![]), more: false, holds: false })),
    }
}
fn run(task: TaskLink, state: AttemptState, location: RunLocation) -> HostRun {
    HostRun { task, attempt: u64::MAX, kept: u32::MAX, listed: false, state, location, grace_until: None }
}
fn round_trip(value: Deployment) {
    let mut setup = script();
    setup.truth.deployment = Some(value.clone());
    let mut engine = Engine::new(setup, 1).expect("valid script");
    engine.read(Time::ZERO, REQUESTER_SIGN_IN, Token::new(1), Read::Deployment).expect("typed read fits");
    assert_eq!(
        engine.poll(Time::ZERO),
        vec![Up::Read {
            token: Token::new(1),
            terminal: ReadResult::Value(ReadValue::Deployment(Box::new(value.clone())))
        }]
    );
    engine.open(Time::ZERO, REQUESTER_SIGN_IN, Token::new(2), Subject::Deployment).expect("typed watch fits");
    let events = engine.poll(Time::ZERO);
    let expected =
        Up::Watch { token: Token::new(2), event: WatchEvent::Delivered(Delivery::Deployment(Box::new(value))) };
    assert!(events.contains(&expected));
    assert_eq!(engine.commits(), []);
}
#[test]
fn deployment_whole_twenty_one_fields_independent_capacities_loss_and_source_order_reach_read_and_watch() {
    let mut value = empty();
    value.projects = list(vec![
        ProjectCapacity { project: u32::MAX, tasks: Capacity { used: 23, limit: 29 }, proposals: 31 },
        ProjectCapacity { project: 0, tasks: Capacity { used: 37, limit: 41 }, proposals: 43 },
    ]);
    round_trip(value);
    let mut value = empty();
    value.cut = false;
    value.hosts_forgotten = 0;
    value.hello_refusals_forgotten = u64::MAX - 1;
    value.person_room_refusals_forgotten = u64::MAX - 2;
    value.task_room_refusals_forgotten = u64::MAX - 3;
    round_trip(value);
}
#[test]
fn deployment_hosts_ten_attempt_states_three_placements_task_links_and_absent_clocks_are_independent() {
    let states = [
        AttemptState::Waiting,
        AttemptState::Adopted,
        AttemptState::Claimed,
        AttemptState::Cancelled,
        AttemptState::Handed,
        AttemptState::Acknowledged,
        AttemptState::Stray,
        AttemptState::Kept,
        AttemptState::Fenced,
        AttemptState::Closed,
    ];
    let places = [
        RunLocation::None,
        RunLocation::Connected { channel: u64::MAX, kind: HostKind::Engine },
        RunLocation::Adrift { channel: None, until: None, remaining: None },
        RunLocation::Adrift { channel: Some(0), until: Some(Wall::EPOCH), remaining: Some(Duration::ZERO) },
    ];
    for (i, state) in states.into_iter().enumerate() {
        let mut value = empty();
        let attempt = run(TaskLink::Stored { number: u64::MAX, status: None }, state, places[i % places.len()]);
        value.attempts.push(attempt.clone()).expect("fixture attempt fits");
        value
            .hosts
            .push(Host {
                name: None,
                awaiting_logins: false,
                rule: HostRule::Open,
                programs: List::with_capacity(0),
                channel: u64::MAX,
                kind: HostKind::Worker,
                slots: u32::MAX,
                used: 7,
                free: 13,
                draining: i % 2 == 0,
                connected: i % 2 != 0,
                grace_until: None,
                grace_remaining: Some(Duration::ZERO),
                runs: list(vec![attempt]),
                cut: false,
            })
            .expect("fixture host fits");
        round_trip(value);
    }
    for task in [
        TaskLink::Live(chip(0)),
        TaskLink::Retained { chip: chip(u64::MAX), at: None },
        TaskLink::Retained { chip: chip(u64::MAX), at: Some(Wall::EPOCH) },
        TaskLink::Stored { number: 0, status: Some(TaskStatus::Cancelled) },
    ] {
        let mut value = empty();
        let mut attempt = run(task, AttemptState::Closed, places[0]);
        attempt.grace_until = Some(Wall::EPOCH);
        value.attempts.push(attempt).expect("fixture attempt fits");
        round_trip(value);
    }
}
fn account_states() -> [AccountState; 8] {
    [
        AccountState::Granted,
        AccountState::Asking,
        AccountState::SignInAgain,
        AccountState::Misconfigured,
        AccountState::Later { until: Some(Wall::EPOCH) },
        AccountState::Unloaded { at: AccountPlace::File, why: AccountUnread::Refused },
        AccountState::Cancelled,
        AccountState::Closing,
    ]
}
#[test]
fn deployment_accounts_five_states_independent_flags_spend_overlay_and_generation_reach_both_carriers() {
    for state in account_states() {
        for flags in 0_u8..8 {
            let mut value = empty();
            value
                .accounts
                .push(Account {
                    list: account_list_rows()[usize::from(flags)],
                    facts: list(
                        vec![
                            AccountFactValue::Text(Box::from(b"you@example.com".as_slice())),
                            AccountFactValue::Count(123),
                            AccountFactValue::Instant(Wall::from_nanos(127)),
                            AccountFactValue::Span(Duration::from_nanos(131)),
                            AccountFactValue::Share(10_000),
                            AccountFactValue::Window(AccountWindow {
                                length: Some(Duration::from_nanos(137)),
                                used: Some(10_000),
                                resets: Some(Wall::from_nanos(139)),
                                state: 2,
                            }),
                        ]
                        .into_iter()
                        .map(|value| AccountFact {
                            kind: 1,
                            label: Box::from(b"label".as_slice()),
                            value,
                            seen: Wall::from_nanos(149),
                            source: Some(11),
                            origin: 6,
                        })
                        .collect(),
                    ),
                    provider_name: None,
                    manage_usage: None,
                    place: 0,
                    uses: None,
                    plan: false,
                    provider: None,
                    display: DocumentText { bytes: Text::new(Box::from([]), 32).expect("empty display"), cut: false },
                    holder: AccountHolder::Deployment,
                    sources: List::with_capacity(0),
                    number: u32::MAX,
                    generation: Some(u64::MAX),
                    state,
                    usable: flags & 1 != 0,
                    expiring: flags & 2 != 0,
                    attention: flags & 4 != 0,
                    spent_until: if flags & 1 == 0 { None } else { Some(Wall::EPOCH) },
                    runs: list(vec![chip(0)]),
                    waiting: list(vec![chip(u64::MAX)]),
                    cut: flags & 2 != 0,
                })
                .expect("fixture account fits");
            round_trip(value);
        }
    }
}
#[test]
fn deployment_three_hello_refusals_keep_advertised_stop_bound_allowed_grace_and_original_clock() {
    for why in [
        HelloRefusal::Position { worker: u32::MAX },
        HelloRefusal::ReservedChannel,
        HelloRefusal::Workers { limit: u32::MAX },
        HelloRefusal::StopBound { grace: Duration::from_nanos(17) },
    ] {
        let mut value = empty();
        value
            .hello_refusals
            .push(HelloRefusalLine {
                channel: u64::MAX,
                at: Wall::EPOCH,
                slots: u32::MAX,
                stop_bound: Duration::from_nanos(19),
                why,
            })
            .expect("fixture hello refusal fits");
        round_trip(value);
    }
}

fn person(action: PersonCapacityAction, cause: PersonCapacityCause) -> PersonCapacityRefusal {
    PersonCapacityRefusal {
        key: RequestKey { person: u64::MAX, key: [0, 255, 1, 254, 2, 253, 3, 252, 4, 251, 5, 250, 6, 249, 7, 248] },
        project: None,
        action,
        at: Wall::EPOCH,
        cause,
    }
}
fn task(action: TaskCapacityAction, cause: TaskCapacityCause) -> TaskCapacityRefusal {
    TaskCapacityRefusal {
        request: TaskCapacityRequest {
            project: Some(u32::MAX),
            by: Requester::Deployment { project: u32::MAX },
            action,
            cause,
        },
        at: Wall::from_nanos(u64::MAX),
    }
}
#[test]
fn deployment_person_actions_four_causes_all_structural_limits_and_opaque_request_keys_survive() {
    use AdmissionLimit as L;
    use PersonCapacityAction as A;
    let actions = [
        A::Create,
        A::Compound,
        A::Application { connector: u16::MAX },
        A::Note { name: u64::MAX },
        A::Service,
        A::Adoption,
        A::Roles,
        A::Policy,
        A::Pool { person: u64::MAX },
        A::Escalation { task: u64::MAX },
        A::Proposal { proposer: Proposer::Task(u64::MAX), number: u64::MAX },
        A::Proposal { proposer: Proposer::Party(u64::MAX), number: 0 },
        A::Words { task: u64::MAX },
        A::Question { task: u64::MAX, question: u64::MAX },
        A::Amend { task: u64::MAX },
        A::Move { task: u64::MAX },
        A::TakePerson { task: u64::MAX },
        A::HandBackPerson { task: u64::MAX },
        A::AnswerPerson { task: u64::MAX },
        A::Stop { task: u64::MAX },
        A::Cancel { task: u64::MAX },
        A::Release { task: u64::MAX },
        A::MarkSeen { seen: SeenThrough::Results { through: u64::MAX } },
        A::MarkSeen { seen: SeenThrough::Conversation { task: u64::MAX, through: u64::MAX } },
    ];
    for action in actions {
        let mut value = empty();
        value
            .person_room_refusals
            .push(person(action, PersonCapacityCause::Requests { limit: u32::MAX }))
            .expect("fixture person refusal fits");
        round_trip(value);
    }
    let limits = [
        L::Words,
        L::Identity,
        L::Roles,
        L::Arithmetic,
        L::Duplicate,
        L::Empty,
        L::Batch,
        L::ProjectLiveTasks,
        L::LiveTasks,
        L::Tree,
        L::Depth,
        L::Delegates,
        L::Subscription,
        L::Dependencies,
        L::Cycle,
        L::Spec,
        L::Contract,
        L::Authority,
        L::Inputs,
        L::Attempts,
        L::LiveDelegates,
        L::Restore,
        L::Read,
        L::Turn,
        L::HoldKind,
        L::HoldTaken,
        L::Holds,
        L::InboxFull,
        L::Notes,
        L::Revision,
        L::Question,
        L::Priority,
        L::Policy,
        L::Resource,
        L::Service,
    ];
    assert_eq!(limits.len(), 35);
    for which in limits {
        let mut value = empty();
        value
            .person_room_refusals
            .push(person(A::Application { connector: u16::MAX }, PersonCapacityCause::Limit { which }))
            .expect("fixture person refusal fits");
        round_trip(value);
    }
    for cause in [PersonCapacityCause::Pending { limit: u32::MAX }, PersonCapacityCause::Waiters { limit: u32::MAX }] {
        let mut value = empty();
        let mut refusal = person(A::Application { connector: u16::MAX }, cause);
        refusal.project = Some(0);
        value.person_room_refusals.push(refusal).expect("fixture person refusal fits");
        round_trip(value);
    }
}
#[test]
fn deployment_task_actions_fourteen_causes_three_requesters_and_four_funders_keep_full_width_values() {
    use TaskCapacityAction as A;
    use TaskCapacityCause as C;
    let actions = [
        A::Batch { first: None, members: u32::MAX },
        A::Batch { first: Some(0), members: 0 },
        A::Proposal { number: u64::MAX },
        A::PersonProposal { number: u64::MAX },
        A::Message { task: u64::MAX, number: u64::MAX },
        A::Source { funder: Funder::Task(u64::MAX) },
        A::Source { funder: Funder::Pool { project: u32::MAX, person: u64::MAX, period: u64::MAX } },
        A::Source { funder: Funder::Recurring { project: u32::MAX, task: u64::MAX, period: u64::MAX } },
        A::Source { funder: Funder::Period { project: u32::MAX, period: u64::MAX } },
        A::Introduce { left: u64::MAX, right: 0 },
        A::Subscription { number: u64::MAX },
        A::Effect { entry: u64::MAX },
        A::Claim { attempt: u64::MAX },
        A::Release,
        A::Move,
    ];
    for action in actions {
        let mut value = empty();
        value.task_room_refusals.push(task(action, C::Tasks { limit: u32::MAX })).expect("fixture task refusal fits");
        round_trip(value);
    }
    let causes = [
        C::Tasks { limit: u32::MAX },
        C::CreationFlights { limit: u32::MAX },
        C::ResourceReports { limit: u32::MAX },
        C::ProjectTasks { limit: u32::MAX },
        C::Tree { limit: u32::MAX },
        C::Delegates { limit: u32::MAX },
        C::Inbox { task: u64::MAX, messages: u32::MAX, bytes: u32::MAX },
        C::Holdings { limit: u32::MAX },
        C::HoldWaiters { limit: u32::MAX },
        C::Proposals { limit: u32::MAX },
        C::References { limit: u32::MAX },
        C::Subscriptions { limit: u32::MAX },
        C::Funders { limit: u32::MAX },
        C::Writers { limit: u32::MAX },
    ];
    assert_eq!(causes.len(), 14);
    for cause in causes {
        let mut value = empty();
        value.task_room_refusals.push(task(A::Move, cause)).expect("fixture task refusal fits");
        round_trip(value);
    }
    for by in [Requester::Task(u64::MAX), Requester::Person(u64::MAX), Requester::Deployment { project: u32::MAX }] {
        let mut value = empty();
        let mut refusal = task(A::Release, C::Tasks { limit: 0 });
        refusal.request.by = by;
        refusal.request.project = None;
        value.task_room_refusals.push(refusal).expect("fixture task refusal fits");
        round_trip(value);
    }
}
