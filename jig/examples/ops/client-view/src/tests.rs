#![expect(
    clippy::disallowed_types,
    clippy::disallowed_methods,
    clippy::wildcard_enum_match_arm,
    reason = "fixtures inspect public tree nodes and observed boundary outputs"
)]
use crate::{Limits, View, build, words};
use jig_client::contract::*;
use jig_ops_client_domain as support_domain;
use jig_ops_client_domain::{self as root, Address, Event, infrastructure, observability};
#[path = "../../client-domain/src/tests_support.rs"]
mod support;
use skein_lib::{Duration, Env, List, Queue, Time, Wall};
use support::{Domain, JigOut, Request};
const JIG: jig_client_view::Config = jig_client_view::Config {
    notices: 2,
    notice_life: Duration::from_secs(5),
    linger: Duration::from_secs(5),
    save_delay: Duration::from_secs(1),
    intents: 2,

    objects: 3,
    sources: 8,

    drafts: 1,

    roles: 2,

    conversation_words: 2,
    navigation_entries: 8,
    updates_count: 99,
};
fn limits() -> root::Limits {
    root::Limits {
        jig: JIG,
        infrastructure: infrastructure::Limits { objects: 12 },
        observability: observability::Limits { objects: 12 },
        client: support::client_limits(),
    }
}

fn drive(root: &mut Domain, event: Event) -> Vec<Request> {
    let mut out = Queue::uncapped();
    support::step(root, &Env { now: Time::ZERO, wall: Wall::EPOCH, limits: limits() }, event, &mut out);
    let mut rows = Vec::new();
    while let Some(r) = out.pop() {
        rows.push(r);
    }
    rows
}
fn ready() -> Domain {
    let mut root = support::new(&limits(), 5);
    root::configure_arrangement(&mut root, crate::arrangement()).expect("ops arrangement before start");
    let mut rows = drive(&mut root, Event::Client { event: jig_client::Event::Up { up: jig_client::Up::SignedIn } });
    rows.extend(drive(
        &mut root,
        Event::Start {
            address: Address::Jig(jig_client_view::address::Address::Inbox { project: None }),
            kept: List::with_capacity(0),
        },
    ));
    let watch = rows
        .iter()
        .find_map(|r| match r {
            Request::Jig(JigOut::Open { watch, subject: Subject::Me }) => Some(*watch),
            _ => None,
        })
        .unwrap();
    drive(
        &mut root,
        Event::Client { event: jig_client::Event::Up { up: jig_client::Up::Served { served: true, versions: None } } },
    );
    // Explicit synthetic identity and empty offers, supplied independently of the party's display name.
    let frame = Me {
        providers: List::with_capacity(0),
        providers_cut: false,
        key_bytes: 256,
        redirect_bytes: 4096,
        words_bytes: 8192,
        party: MeParty {
            number: 7,
            kind: PartyKind::Person,
            login: DocumentText::complete(Text::new(Box::from(*b"fixture-login"), 32).unwrap()),
            name: DocumentText::complete(Text::new(Box::from(*b"Pat"), 32).unwrap()),
        },
        projects: List::with_capacity(0),
        needs: Needs { questions: 0, proposals: 0, escalations: 0, person_tasks: 0 },
        needs_total: 0,
        updates: Count { value: 0, limited: false },
        results_seen: 0,
        cut: false,
        retention: Duration::from_secs(1000),
        deployment_role: Some(HeldRole::Policy { role: 1 }),
    };
    drive(
        &mut root,
        Event::Client {
            event: jig_client::Event::Up {
                up: jig_client::Up::Watched { watch, event: WatchEvent::Delivered(Delivery::Me(Box::new(frame))) },
            },
        },
    );
    root
}
fn phrasebooks() -> (jig_client_view::words::Words, words::Words, words::Words) {
    static WORDS: std::sync::OnceLock<(jig_client_view::words::Words, words::Words, words::Words)> =
        std::sync::OnceLock::new();
    WORDS
        .get_or_init(|| {
            (
                jig_client_view::words::Words::new(jig_client_view::phrases::english::english(32).unwrap(), 0, 0),
                words::Words::english(1, 32).unwrap(),
                words::Words::english(2, 32).unwrap(),
            )
        })
        .clone()
}
fn view() -> View {
    let (jig, obs, infra) = phrasebooks();
    let v = Limits::of(&limits(), &jig, &obs, &infra).unwrap();
    View::new(&v, &limits(), jig, obs, infra, 1)
}

fn small_view() -> View {
    let (jig, obs, infra) = phrasebooks();
    let mut bounds = Limits::of(&limits(), &jig, &obs, &infra).unwrap();
    bounds.tree.nodes = 512;
    bounds.tree.bindings = 512;
    bounds.tree.slots = 64;
    bounds.tree.patches = 2560;
    bounds.tree.text_bytes = bounds.tree.nodes.checked_mul(bounds.tree.text_per_node).unwrap();
    View::new(&bounds, &limits(), jig, obs, infra, 1)
}

fn incident() -> observability::values::Document {
    use observability::{
        documents::{Service, State},
        values as n,
    };
    n::Document {
        subject: observability::Subject {
            scope: observability::Scope::Project(1),
            kind: observability::Kind::Incident(12),
        },
        value: n::Value::Incident {
            incident: n::Incident {
                project: 1,
                number: 12,
                watch: 888,
                raised: Wall::from_nanos(36_000_000_000_000),
                service: Service { environment: Box::from(*b"production"), name: Box::from(*b"checkout") },
                severity: 2,
                state: State::Alerting,
                timeline: list([n::Point {
                    at: Wall::from_nanos(36_000_000_000_000),
                    health: Some(n::Health { observed: 36_000, healthy_replicas: 3, errors: 2, error_rate_percent: 2 }),
                }]),
                cut: false,
                lead: None,
                alerts: Box::new([]),
            },
            fact: Some(n::Fact {
                observed: 36_120,
                healthy_replicas: 2,
                errors: 30,
                error_rate_percent: 30,
                load_percent: 0,
                load: Box::new([]),
            }),
        },
    }
}
fn environment_name() -> infrastructure::values::Environment {
    infrastructure::values::Environment { pool: Box::from(*b"Q.pool"), name: Box::from(*b"staging-7") }
}
fn environment() -> infrastructure::Document {
    use infrastructure::values as n;
    infrastructure::Document::Value(Box::new(n::Document {
        subject: infrastructure::Subject {
            scope: infrastructure::Scope::Project(1),
            kind: infrastructure::Kind::Environment(environment_name()),
        },
        value: n::Value::Environment {
            environment: environment_name(),
            end_revision: 5,
            fact: n::EnvironmentFact {
                created_by: n::Key {
                    deployment: [1; 16],
                    task: 9,
                    origin: n::Purpose::Procedure { purpose: 2 },
                    purpose: 2,
                },
                ready_at: 30,
                until: 123,
                price: 40,
            },
            end: Some(n::EnvironmentEnd { revision: 5, until: 600 }),
            pool: None,
        },
    }))
}

#[test]
fn deployment_keys_are_distinct_from_both_zero_and_maximum_project_identifiers() {
    let infrastructure = [
        infrastructure::Scope::Project(0),
        infrastructure::Scope::Project(u32::MAX),
        infrastructure::Scope::Deployment,
    ]
    .map(|scope| {
        crate::infrastructure::key(&infrastructure::Subject { scope, kind: infrastructure::Kind::Outbox(17) })
            .expect("numeric owner key")
    });
    let observability =
        [observability::Scope::Project(0), observability::Scope::Project(u32::MAX), observability::Scope::Deployment]
            .map(|scope| {
                crate::observability::key(&observability::Subject { scope, kind: observability::Kind::Outbox(17) })
            });
    for keys in [infrastructure, observability] {
        assert_ne!(keys[0], keys[1]);
        assert_ne!(keys[0], keys[2]);
        assert_ne!(keys[1], keys[2]);
        assert_eq!(keys[0].a, 0);
        assert_eq!(keys[1].a, u64::from(u32::MAX));
        assert_eq!(keys[2].a, u64::MAX);
        assert!(keys.iter().all(|key| key.b == 17 && key.kind == 6));
    }
}
#[test]
fn an_incident_drawing_has_a_readable_fallback_and_missing_lead_is_explicit() {
    let mut root = ready();
    let requests = drive(
        &mut root,
        Event::Went {
            place: root::Place::Address(Address::Observability(observability::Address::Incident {
                project: 1,
                incident: 12,
            })),
        },
    );
    let read = requests
        .iter()
        .find_map(|r| match r {
            Request::Observability(observability::Out::Read { read, .. }) => Some(*read),
            _ => None,
        })
        .unwrap();
    drive(
        &mut root,
        Event::Observability(observability::Event::Read {
            read,
            result: observability::ReadResult::Value(Box::new(observability::Document::Value(Box::new(incident())))),
        }),
    );
    let mut view = view();
    let mut patches = Queue::uncapped();
    build(&mut view, &root, &mut patches);
    assert!(
        view.tree()
            .nodes()
            .iter()
            .any(|n| n.role == skein_ui::Role::Drawing && n.name.as_deref() == Some(b"Error rate".as_slice()))
    );
    let text: Vec<u8> =
        view.tree().nodes().iter().flat_map(|n| n.text.iter().flat_map(|r| r.text.iter().copied())).collect();
    assert!(String::from_utf8(text).unwrap().contains("Error rate 30 % Since 10:02, Before: 2 %"));
    assert!(
        view.tree().nodes().iter().any(|n| n.role == skein_ui::Role::Status
            && n.name.as_deref() == Some(b"No lead task has been supplied".as_slice()))
    );
    while patches.pop().is_some() {}
    build(&mut view, &root, &mut patches);
    assert!(patches.is_empty());
}
#[test]
fn family_keys_keep_scope_and_original_owner_identity() {
    use observability::documents::{Kind, Scope, Subject};
    let a = crate::observability::key(&Subject { scope: Scope::Project(1), kind: Kind::Incident(12) });
    let b = crate::observability::key(&Subject { scope: Scope::Project(2), kind: Kind::Incident(12) });
    assert_ne!(a, b);
    assert_eq!(a.b, 12);
    assert_ne!(
        crate::observability::key(&Subject {
            scope: Scope::Project(1),
            kind: Kind::Effect(ProposalKey { proposer: Proposer::Task(12), number: 3 })
        }),
        crate::observability::key(&Subject { scope: Scope::Project(1), kind: Kind::Outbox(3) })
    );
}

#[test]
fn equal_task_and_party_maker_numbers_keep_full_width_effect_keys_in_both_families() {
    let makers = [Proposer::Task(0), Proposer::Party(0), Proposer::Task(u64::MAX), Proposer::Party(u64::MAX)];
    let obs_keys = makers.map(|proposer| {
        crate::observability::key(&observability::Subject {
            scope: observability::Scope::Project(1),
            kind: observability::Kind::Effect(ProposalKey { proposer, number: u64::MAX }),
        })
    });
    let infra_keys = makers.map(|proposer| {
        crate::infrastructure::key(&infrastructure::Subject {
            scope: infrastructure::Scope::Project(1),
            kind: infrastructure::Kind::Effect(ProposalKey { proposer, number: u64::MAX }),
        })
        .expect("maker key")
    });
    for keys in [obs_keys, infra_keys] {
        for (index, key) in keys.iter().enumerate() {
            assert!(keys[index + 1..].iter().all(|other| key != other));
            assert_eq!(key.b, u64::MAX);
        }
        assert_eq!(keys[0].a, 0);
        assert_eq!(keys[1].a, 0);
        assert_eq!(keys[2].a, u64::MAX);
        assert_eq!(keys[3].a, u64::MAX);
    }
}

fn text(bytes: &[u8]) -> Text {
    Text::new(bytes.into(), 32).unwrap()
}
fn list<T>(rows: impl IntoIterator<Item = T>) -> List<T> {
    List::from_boxed(rows.into_iter().collect::<Vec<_>>().into_boxed_slice())
}
fn procedure_phase() -> ChipPhaseFacts {
    ChipPhaseFacts {
        phase: ChipPhase::Active(RunStage::Running { attempt: 1 }),
        details: Box::new(ChipDetails {
            dependencies: list([]),
            project: 1,
            executor: Executor::Procedure { connector: 2, code: 1 },
            phase_since: Wall::EPOCH,
            run_since: Some(Wall::EPOCH),
            attempt: Some(1),
            retry: None,
            wake: WakePolicy {
                words: WakeRule::Never,
                notices: WakeRule::Never,
                news: WakeRule::Never,
                results: ResultsWake::Never,
                questions: false,
                answers: false,
                timers: false,
            },
            person_wait: None,
            hold: None,
            budget: Budget {
                numbers: Numbers { budget: 1, spent: 0, spent_below: 0, reserved: 0 },
                left: 1,
                funder: Funder::Pool { project: 1, person: 7, period: 3 },
                deadline: None,
            },
            activity: 0,
            resources: list([]),
            resources_cut: false,
            resources_observed: false,
            procedure: Some(Part { connector: 2, kind: PartKind::Procedure, key: PartKey::Number(9) }),
            live_delegates: 0,
            closing_connectors: None,
            ending: None,
            ended_at: None,
            observation: None,
        }),
    }
}
fn task() -> Task {
    Task {
        talk: None,
        session: None,
        dependency_facts: List::with_capacity(0),
        kept_account: None,
        run_wait: None,
        settings: None,
        llm: None,
        label: 0,
        flags: TaskFlags { ended_by: EndedBy::Result, conversation: false, listed: true, retention: Retention::Kept },
        last_activity: Wall::EPOCH,
        made: Wall::EPOCH,
        waits_on: WaitsOn::Nothing,
        chip: Chip {
            task: 9,
            title: DocumentText::complete(text(&[b't'; 32])),
            executor: ChipExecutor::Procedure { connector: 2, kind: 1 },
            phase: Phase::Running,
            details: Box::new(procedure_phase()),
        },
        goal: None,
        priority: Some(1),
        lineage_chips: list([]),
        inputs: list([]),
        project: 1,
        requester: Requester::Person(7),
        lineage: list([]),
        spec: SpecificationDocument::complete(Specification {
            asks: false,
            words: text(b"spec"),
            parameters: list([]),
            inputs: list([]),
        }),
        spec_revision: Revision { value: 1 },
        person_revision: None,
        taken_by: None,
        instructions: Some(DocumentText::complete(text(b"instructions"))),
        wake: WakePolicy {
            words: WakeRule::Never,
            notices: WakeRule::Never,
            news: WakeRule::Never,
            results: ResultsWake::Never,
            questions: false,
            answers: false,
            timers: false,
        },
        subscriptions: Subscriptions { entries: list([]) },
        result_contract: TaskContractDocument::Report { words: 32 },
        dependencies: list([]),
        references: list([]),
        cut: false,
        resources: list([]),
        budget: Budget {
            numbers: Numbers { budget: 1, spent: 0, spent_below: 0, reserved: 0 },
            left: 1,
            funder: Funder::Pool { project: 1, person: 7, period: 3 },
            deadline: None,
        },
        authority: AuthorityDocument {
            authority: Authority {
                models: None,
                tools: 0,
                grants: list([]),
                delegation: Delegation { kinds: list([]), tasks: 1, depth: 1 },
                spend: 1,
                deadline: None,
                notes: 0,
                note_resources: list([]),
            },
            cut: false,
        },
        hold: None,
        context: Some(Box::new(TaskContext {
            parts: list([]),
            closing: None,
            spend: list([]),
            runtime: Runtime::Procedure { panel: ApplicationPart::Panel { connector: 2, task: 9 } },
            impact: Impact {
                tasks: ImpactCounts {
                    waiting: 0,
                    running: 0,
                    held: 0,
                    closing: 0,
                    done: 0,
                    failed: 0,
                    cancelled: 0,
                    ended: 0,
                },
                live_runs: 0,
                proposals: 0,
            },
            writer_slots: list([]),
        })),
        proposals: list([]),
        activity: 0,
        changed: Wall::EPOCH,
        acts: list([]),
        result: None,
        delegates: list([]),
    }
}

fn arranged_task_view(section: jig_client_view::address::TaskSection) -> View {
    let mut root = ready();
    let requests = drive(
        &mut root,
        Event::Went {
            place: root::Place::Address(Address::Jig(jig_client_view::address::Address::Task { task: 9, section })),
        },
    );
    let watch = requests
        .iter()
        .find_map(|request| match request {
            Request::Jig(JigOut::Open { watch, subject: Subject::Task { task: 9 } }) => Some(*watch),
            _ => None,
        })
        .expect("task watch");
    let mut document = task();
    document.context.as_mut().expect("task context").impact.tasks.waiting = 1;
    drive(
        &mut root,
        Event::Client {
            event: jig_client::Event::Up {
                up: jig_client::Up::Watched { watch, event: WatchEvent::Delivered(Delivery::Task(Box::new(document))) },
            },
        },
    );
    let mut view = small_view();
    let mut patches = Queue::uncapped();
    build(&mut view, &root, &mut patches);
    view
}

#[test]
fn an_ops_task_page_puts_budget_and_panels_in_an_aside() {
    use jig_client_view::regions::JigRegion;
    use skein_ui::Role;
    let view = arranged_task_view(jig_client_view::address::TaskSection::Plan);
    let (index, aside) = view.tree().nodes().iter().enumerate().find(|(_, n)| n.role == Role::Complementary).unwrap();
    let regions: Vec<_> = view
        .tree()
        .nodes()
        .iter()
        .skip(index + 1)
        .take(usize::try_from(aside.size - 1).unwrap())
        .filter_map(|node| node.region)
        .collect();
    assert_eq!(regions, [JigRegion::TaskBudget.region(), JigRegion::TaskPanels.region()]);
}

#[test]
fn an_ops_task_page_marks_the_addresss_own_section_current() {
    use jig_client_view::regions::JigRegion;
    let view = arranged_task_view(jig_client_view::address::TaskSection::Plan);
    let rows = [
        (JigRegion::TaskOverview, &[][..]),
        (JigRegion::TaskActivity, &[][..]),
        (JigRegion::TaskPlan, &[skein_ui::State::Current][..]),
        (JigRegion::TaskHistory, &[][..]),
        (JigRegion::TaskBudget, &[][..]),
        (JigRegion::TaskPanels, &[][..]),
    ];
    for (region, expected) in rows {
        let node = view.tree().nodes().iter().find(|node| node.region == Some(region.region())).unwrap();
        assert_eq!(node.states.as_slice(), expected, "{region:?}");
    }
}

#[test]
fn an_ops_arranged_task_page_keeps_the_outline() {
    let view = arranged_task_view(jig_client_view::address::TaskSection::Plan);
    assert_eq!(skein_ui::outline::check(view.tree()), None);
}

#[test]
fn an_ops_arranged_task_page_has_no_tabs() {
    let view = arranged_task_view(jig_client_view::address::TaskSection::Plan);
    assert_eq!(view.tree().nodes().iter().filter(|node| node.role == skein_ui::Role::Tabs).count(), 0);
}

fn incident_plan_page() -> Domain {
    let mut root = ready();
    let requests = drive(
        &mut root,
        Event::Went {
            place: root::Place::Address(Address::Observability(observability::Address::Incident {
                project: 1,
                incident: 12,
            })),
        },
    );
    let read = requests
        .iter()
        .find_map(|request| match request {
            Request::Observability(observability::Out::Read { read, .. }) => Some(*read),
            _ => None,
        })
        .unwrap();
    let mut facts = incident();
    let observability::values::Value::Incident { incident, .. } = &mut facts.value else { unreachable!() };
    incident.lead = Some(9);
    let document = observability::Document::Value(Box::new(facts));
    let requests = drive(
        &mut root,
        Event::Observability(observability::Event::Read {
            read,
            result: observability::ReadResult::Value(Box::new(document)),
        }),
    );
    let watch = requests
        .iter()
        .find_map(|request| match request {
            Request::Jig(JigOut::Open { watch, subject: Subject::Task { task: 9 } }) => Some(*watch),
            _ => None,
        })
        .expect("lead's section opens its task watch");
    let mut document = task();
    document.context.as_mut().unwrap().impact.tasks.waiting = 1;
    drive(
        &mut root,
        Event::Client {
            event: jig_client::Event::Up {
                up: jig_client::Up::Watched { watch, event: WatchEvent::Delivered(Delivery::Task(Box::new(document))) },
            },
        },
    );
    root
}

#[test]
fn the_incident_page_fills_its_leads_plan_in_a_section_slot() {
    {
        let root = incident_plan_page();
        let mut view = small_view();
        let mut patches = Queue::uncapped();
        build(&mut view, &root, &mut patches);
        let plans: Vec<_> = view
            .tree()
            .nodes()
            .iter()
            .filter(|node| node.region == Some(jig_client_view::regions::JigRegion::TaskPlan.region()))
            .collect();
        assert_eq!(plans.len(), 1);
        assert_eq!(plans[0].name.as_deref(), Some(b"Plan".as_slice()));
    }
}
fn contains(view: &View, words: &[u8]) -> bool {
    view.tree()
        .nodes()
        .iter()
        .flat_map(|n| n.text.iter())
        .any(|r| r.text.windows(words.len()).any(|slice| slice == words))
}
#[test]
fn moving_an_environment_end_names_the_end_revision_and_waits_for_the_final_envelope() {
    let mut root = ready();
    let requests = drive(
        &mut root,
        Event::Went {
            place: root::Place::Address(Address::Infrastructure(infrastructure::Address::Environment {
                project: 1,
                environment: environment_name(),
            })),
        },
    );
    let read = requests
        .iter()
        .find_map(|r| match r {
            Request::Infrastructure(infrastructure::Out::Read { read, .. }) => Some(*read),
            _ => None,
        })
        .unwrap();
    let document = environment();
    drive(
        &mut root,
        Event::Infrastructure(infrastructure::Event::Read {
            read,
            result: infrastructure::ReadResult::Value(Box::new(document.clone())),
        }),
    );
    let mut view = view();
    let mut patches = Queue::uncapped();
    while patches.pop().is_some() {}
    build(&mut view, &root, &mut patches);
    assert!(contains(&view, b"Observed backend lease (seconds since epoch):"));
    let field = view
        .tree()
        .nodes()
        .iter()
        .find(|n| {
            n.role == skein_ui::Role::TextField
                && n.name.as_deref() == Some(b"New end (seconds since epoch)".as_slice())
        })
        .unwrap()
        .id;
    let action = crate::decode(&mut view, field, skein_ui::Act::Input { value: Box::from(*b"900") }).unwrap();
    drive(&mut root, Event::Act { action });
    while patches.pop().is_some() {}
    build(&mut view, &root, &mut patches);
    let button = view
        .tree()
        .nodes()
        .iter()
        .find(|n| n.role == skein_ui::Role::Button && n.name.as_deref() == Some(b"Move end".as_slice()))
        .unwrap()
        .id;
    let action = crate::decode(&mut view, button, skein_ui::Act::Press).unwrap();
    assert!(drive(&mut root, Event::Act { action }).is_empty());
    let mut out = Queue::uncapped();
    support::take(&mut root, &Env { now: Time::ZERO, wall: Wall::EPOCH, limits: limits() }, &mut out);
    let Some(Request::Save { record: root::store::Record::Unlinked(_) }) = out.pop() else {
        panic!("save precedes send");
    };
    let request = out.pop().unwrap();
    let Request::Send {
        attempt,
        ask: root::Ask::Infrastructure(infrastructure::Ask::MoveEnd { revision, until, .. }),
        ..
    } = request
    else {
        panic!("keyed move leaves through table")
    };
    assert_eq!(revision.value, 5);
    assert_eq!(until, 900);
    drive(
        &mut root,
        support::answer(
            attempt,
            skein_lib::RequestEnvelope::Again,
            Some(root::Answered::Infrastructure(infrastructure::Answer::MovedEnd {
                environment: environment_name(),
                revision: Revision { value: 6 },
                until,
            })),
        ),
    );
    while patches.pop().is_some() {}
    build(&mut view, &root, &mut patches);
    assert!(!contains(&view, b"End moved"));
}
#[test]
fn extreme_capacity_is_refused_before_building() {
    let jig = jig_client_view::words::Words::new(jig_client_view::phrases::english::english(32).unwrap(), 0, 0);
    let obs = words::Words::english(1, 32).unwrap();
    let infra = words::Words::english(2, 32).unwrap();
    let mut limits = limits();
    limits.client.domain.rows = u32::MAX;
    assert!(Limits::of(&limits, &jig, &obs, &infra).is_none());
}

#[test]
fn a_common_entrance_refusal_prints_supplied_budget_facts_without_claiming_gone() {
    let mut root = ready();
    let requests = drive(
        &mut root,
        Event::Went {
            place: root::Place::Address(Address::Infrastructure(infrastructure::Address::Environment {
                project: 1,
                environment: environment_name(),
            })),
        },
    );
    let read = requests
        .iter()
        .find_map(|r| match r {
            Request::Infrastructure(infrastructure::Out::Read { read, .. }) => Some(*read),
            _ => None,
        })
        .unwrap();
    let document = environment();
    drive(
        &mut root,
        Event::Infrastructure(infrastructure::Event::Read {
            read,
            result: infrastructure::ReadResult::Value(Box::new(document.clone())),
        }),
    );
    let object = root.infrastructure().page().unwrap().token();
    let revision = Revision { value: 5 };
    drive(
        &mut root,
        Event::Act {
            action: root::Action::Infrastructure(infrastructure::Act::Until { object, revision, value: 900 }),
        },
    );
    drive(
        &mut root,
        Event::Act { action: root::Action::Infrastructure(infrastructure::Act::MoveEnd { object, revision }) },
    );
    let mut out = Queue::uncapped();
    support::take(&mut root, &Env { now: Time::ZERO, wall: Wall::EPOCH, limits: limits() }, &mut out);
    let mut attempt = None;
    while let Some(request) = out.pop() {
        if let Request::Send { attempt: token, .. } = request {
            attempt = Some(token);
        }
    }
    let refusal =
        EntranceRefusal::new(Refusal::Authority { lacking: Lacking::Budget { needed: 10, left: 3 }, proposable: None })
            .unwrap();
    drive(
        &mut root,
        support::final_answer(
            attempt.unwrap(),
            root::Answered::Infrastructure(infrastructure::Answer::EntranceRefused(refusal)),
        ),
    );
    let mut view = view();
    let mut patches = Queue::uncapped();
    build(&mut view, &root, &mut patches);
    assert!(contains(&view, b"Your pool has 3 left; this needs 10"));
    assert!(!contains(&view, b"Gone"));
    assert!(!contains(&view, b"End moved"));
    assert!(view.tree().len() <= view.limits().tree.nodes);
}

fn outbox_entry() -> infrastructure::values::Entry {
    use infrastructure::values as n;
    n::Entry {
        project: None,
        number: 17,
        key: n::Key { deployment: [4; 16], task: 29, origin: n::Purpose::Procedure { purpose: 2 }, purpose: 2 },
        effect: n::Effect::Restart {
            service: n::Service { environment: Box::from(*b"production"), name: Box::from(*b"checkout") },
            operation: 31,
        },
        attempt: 2,
        deadline: 0,
        phase: n::Phase::Sent,
    }
}
#[test]
fn resource_slots_preserve_whole_opaque_segments_and_distinct_equal_concatenations_in_both_connectors() {
    let domain_limits = limits();
    let (jig_words, obs_words, infra_words) = phrasebooks();
    let envelope = Limits::of(&domain_limits, &jig_words, &obs_words, &infra_words).unwrap().tree;
    // A finite public fragment tree, borrowing the checked byte/depth envelope.
    let mut tree_limits = envelope;
    tree_limits.nodes = 64;
    tree_limits.slots = 8;
    tree_limits.text_bytes = tree_limits.nodes.checked_mul(tree_limits.text_per_node).unwrap();
    assert!(tree_limits.nodes <= envelope.nodes);
    assert!(tree_limits.slots <= envelope.slots);
    assert!(tree_limits.text_per_node >= 128);
    let mut builder = skein_ui::Builder::new(&tree_limits);
    let mut jig_bindings = jig_client_view::JigBindings::new(16, domain_limits.client.domain.text);
    let mut obs_bindings = crate::bindings::Bindings::new(16, 32, 1);
    let mut infra_bindings = crate::bindings::Bindings::new(16, 32, 2);
    let mut obs = observability::Side::new(&domain_limits.observability, &domain_limits.client.domain);
    let mut infra = infrastructure::Side::new(&domain_limits.infrastructure, &domain_limits.client.domain);
    let mut expected = Vec::new();
    builder.open(skein_ui::Role::Page);
    for connector in [1, 2] {
        for path in [
            list([Box::<[u8]>::from(*b"a/b"), Box::<[u8]>::from(*b"c")]),
            list([Box::<[u8]>::from(*b"a"), Box::<[u8]>::from(*b"b/c")]),
            list([Box::<[u8]>::from(*b"/\0"), Box::<[u8]>::from([0xFF, 0])]),
            list([Box::<[u8]>::from([0xFF; 32]), Box::<[u8]>::from([0xFE; 32])]),
        ] {
            let part = ApplicationPart::Resource { connector, path };
            if connector == 1 {
                let mut out = Queue::uncapped();
                observability::step(
                    &mut obs,
                    &Env { now: Time::ZERO, wall: Wall::EPOCH, limits: domain_limits.client.domain },
                    observability::Event::Show { part: part.clone(), scope: observability::Scope::Project(1) },
                    &mut out,
                );
                assert!(obs.contains(&part));
                assert!(out.is_empty());
            } else {
                let mut out = Queue::uncapped();
                infrastructure::step(
                    &mut infra,
                    &Env { now: Time::ZERO, wall: Wall::EPOCH, limits: domain_limits.client.domain },
                    infrastructure::Event::Show { part: part.clone(), scope: infrastructure::Scope::Project(1) },
                    &mut out,
                );
                assert!(infra.contains(&part));
                assert!(out.is_empty());
            }
            jig_client_view::cards::common::part(&mut builder, &part, &jig_words, &mut jig_bindings);
            expected.push(part);
        }
    }
    builder.close();
    assert_ne!(expected[0], expected[1]);
    let slots = builder.slots(0);
    assert_eq!(slots.len(), 8);
    for (slot, part) in slots.iter().zip(&expected) {
        assert_eq!(jig_bindings.part(slot.reference.token), Some(part));
        let mut fragment = skein_ui::Builder::new(&tree_limits);
        if slot.reference.owner == ((jig_client_view::Owner::Application { number: 1 }).opaque()) {
            crate::observability::part(&mut fragment, &obs, part, &obs_words, &mut obs_bindings);
        } else {
            assert_eq!(slot.reference.owner, (jig_client_view::Owner::Application { number: 2 }).opaque());
            crate::infrastructure::part(&mut fragment, &infra, part, &infra_words, &jig_words, &mut infra_bindings);
        }
        builder.splice(slot.id, fragment);
    }
    let tree = builder.finish();
    let code: Vec<_> = tree
        .nodes()
        .iter()
        .flat_map(|node| node.text.iter())
        .filter(|run| run.run == skein_ui::Run::Code)
        .map(|run| run.text.as_ref())
        .collect();
    for bytes in [b"a/b".as_slice(), b"c", b"a", b"b/c", b"/\0", b"\\xFF\\x00"] {
        assert_eq!(code.iter().filter(|run| **run == bytes).count(), 2);
    }
    assert!(!code.contains(&b"a/b/c".as_slice()));
    let escaped_maximum_byte = b"\\xFF".repeat(32);
    let neighboring_segment_text = b"\\xFE".repeat(32);
    assert_eq!(code.iter().filter(|run| **run == escaped_maximum_byte.as_slice()).count(), 2);
    assert_eq!(code.iter().filter(|run| **run == neighboring_segment_text.as_slice()).count(), 2);
    assert!(tree.len() <= envelope.nodes);
    assert!(
        tree.nodes()
            .iter()
            .all(|node| node.text.iter().all(|run| run.text.len() <= usize::try_from(envelope.text_per_node).unwrap()))
    );
    for (slot, part) in slots.iter().zip(&expected) {
        assert_eq!(jig_bindings.part(slot.reference.token), Some(part));
    }
    jig_bindings.begin();
    assert!(slots.iter().all(|slot| jig_bindings.part(slot.reference.token).is_none()));
}

fn deployment_read(root: &mut Domain, connector: u16) -> Request {
    let rows = drive(
        root,
        Event::Went { place: root::Place::Address(Address::Jig(jig_client_view::address::Address::Deployment)) },
    );
    let watch = rows
        .iter()
        .find_map(|request| match request {
            Request::Jig(JigOut::Open { watch, subject: Subject::Deployment }) => Some(*watch),
            _ => None,
        })
        .unwrap();
    let rows = drive(
        root,
        Event::Client {
            event: jig_client::Event::Up {
                up: jig_client::Up::Watched {
                    watch,
                    event: WatchEvent::Delivered(Delivery::Deployment(Box::new(configured_deployment(connector)))),
                },
            },
        },
    );
    rows.into_iter()
        .find_map(|request| match request {
            Request::Infrastructure(infrastructure::Out::Read { .. })
            | Request::Observability(observability::Out::Read { .. }) => Some(request),
            _ => None,
        })
        .unwrap()
}

fn infrastructure_deployment(subject: infrastructure::Subject) -> infrastructure::Document {
    use infrastructure::values as n;
    let mut uncertain = outbox_entry();
    uncertain.number = 18;
    uncertain.phase = n::Phase::Uncertain;
    infrastructure::Document::Value(Box::new(n::Document {
        subject,
        value: n::Value::Deployment {
            backend: n::Backend::OperationIds,
            effects: list([outbox_entry(), uncertain]),
            made: 0,
            proposal_limit: u32::MAX,
            staged_limit: 7,
        },
    }))
}
fn observability_deployment(subject: observability::Subject) -> observability::Document {
    use observability::values as n;
    let entry = n::Entry {
        project: Some(91),
        number: 17,
        key: n::Key { deployment: [4; 16], task: 29, origin: n::Purpose::Procedure { purpose: 2 }, purpose: 2 },
        effect: n::Effect { rule: Box::from(*b"checkout"), until: 0 },
        attempt: 2,
        deadline: 0,
        phase: n::Phase::Sent,
    };
    let mut uncertain = entry.clone();
    uncertain.number = 18;
    uncertain.phase = n::Phase::Uncertain;
    observability::Document::Value(Box::new(n::Document {
        subject,
        value: n::Value::Deployment {
            watches: 3,
            effects: list([entry, uncertain]),
            read_limit: u32::MAX,
            incident_limit: 7,
        },
    }))
}

#[test]
fn known_deployment_parts_name_actual_capacities_and_deferred_diagnostics_without_changing_outbox_facts() {
    for connector in [1, 2] {
        let mut root = ready();
        let request = deployment_read(&mut root, connector);
        let mut view = view();
        let mut patches = Queue::uncapped();
        build(&mut view, &root, &mut patches);
        assert!(contains(&view, b"Loading"));
        assert!(!contains(&view, b"not kept by this connector"));
        while patches.pop().is_some() {}
        match request {
            Request::Infrastructure(infrastructure::Out::Read { read, subject }) => {
                assert_eq!(
                    subject,
                    infrastructure::Subject {
                        scope: infrastructure::Scope::Deployment,
                        kind: infrastructure::Kind::Deployment
                    }
                );
                let document = infrastructure_deployment(subject.clone());
                drive(
                    &mut root,
                    Event::Infrastructure(infrastructure::Event::Read {
                        read,
                        result: infrastructure::ReadResult::Value(Box::new(document.clone())),
                    }),
                );
                let object = root.infrastructure().find(subject).unwrap();
                assert_eq!(
                    root.infrastructure().object(object).unwrap().value,
                    infrastructure::Value::Known(Box::new(document))
                );
            }
            Request::Observability(observability::Out::Read { read, subject }) => {
                assert_eq!(
                    subject,
                    observability::Subject {
                        scope: observability::Scope::Deployment,
                        kind: observability::Kind::Deployment
                    }
                );
                let document = observability_deployment(subject);
                drive(
                    &mut root,
                    Event::Observability(observability::Event::Read {
                        read,
                        result: observability::ReadResult::Value(Box::new(document.clone())),
                    }),
                );
                let object = root.observability().find(subject).unwrap();
                assert_eq!(
                    root.observability().object(object).unwrap().value,
                    observability::Value::Known(Box::new(document))
                );
            }
            _ => panic!("connector Deployment read"),
        }
        build(&mut view, &root, &mut patches);
        if connector == 2 {
            assert!(contains(&view, b"System request budget: not kept by this connector"));
            assert!(contains(&view, b"What the system last said: not kept by this connector"));
        }
        assert!(contains(
            &view,
            if connector == 1 { b"Read capacity: 4294967295" } else { b"Proposal capacity: 4294967295" }
        ));
        assert!(!contains(&view, b"Request limit:"));
        assert!(contains(&view, b"Outbox entry 17"));
        assert!(contains(&view, b"Outbox entry 18"));
        if connector == 2 {
            assert!(contains(&view, b"Sent"));
            assert!(contains(&view, b"Uncertain"));
        }
        assert!(view.tree().len() <= view.limits().tree.nodes);
        assert!(view.tree().nodes().iter().all(|node| {
            node.text.iter().all(|run| run.text.len() <= usize::try_from(view.limits().tree.text_per_node).unwrap())
        }));
    }
}

#[test]
fn unknown_deployment_parts_keep_missing_owner_facts_without_not_kept_captions() {
    for connector in [1, 2] {
        let mut root = ready();
        match deployment_read(&mut root, connector) {
            Request::Infrastructure(infrastructure::Out::Read { read, subject }) => {
                drive(
                    &mut root,
                    Event::Infrastructure(infrastructure::Event::Read {
                        read,
                        result: infrastructure::ReadResult::Unknown,
                    }),
                );
                let object = root.infrastructure().find(subject).unwrap();
                assert_eq!(root.infrastructure().object(object).unwrap().value, infrastructure::Value::Unknown);
            }
            Request::Observability(observability::Out::Read { read, subject }) => {
                drive(
                    &mut root,
                    Event::Observability(observability::Event::Read {
                        read,
                        result: observability::ReadResult::Unknown,
                    }),
                );
                let object = root.observability().find(subject).unwrap();
                assert_eq!(root.observability().object(object).unwrap().value, observability::Value::Unknown);
            }
            _ => panic!("connector Deployment read"),
        }
        let mut view = view();
        let mut patches = Queue::uncapped();
        build(&mut view, &root, &mut patches);
        assert!(contains(&view, b"Unknown"));
        assert!(!contains(&view, b"not kept by this connector"));
        assert!(!contains(&view, b"capacity:"));
        assert!(view.tree().len() <= view.limits().tree.nodes);
    }
}

#[test]
fn deferred_connector_diagnostic_words_select_bounded_checked_language_without_holes() {
    use skein_ui::phrases::{
        Form, Language, Phrase, PhraseLimits, PhraseShape, Phrasebook, PluralClass, PluralRule, Segment, SpanUnit,
        WallFormat,
    };
    for connector in [1, 2] {
        let english = words::Words::english(connector, 32).unwrap();
        let ids = [words::Word::SystemRequestBudgetNotKept, words::Word::LastSystemResponseNotKept];
        let texts = [b"Budget non conserve".as_slice(), b"Derniere reponse non conservee".as_slice()];
        let mut shapes = Vec::new();
        let mut original = Vec::new();
        let mut selected = Vec::new();
        let mut text_bytes = 0;
        for (word, text) in ids.into_iter().zip(texts) {
            shapes.push(PhraseShape { id: word.id(connector), holes: Box::new([]), plural: None });
            let literal = english.say(word, &[]);
            text_bytes += u32::try_from(literal.len() + text.len()).unwrap();
            original.push(Phrase {
                id: word.id(connector),
                forms: Box::new([Form { class: PluralClass::Other, segments: Box::new([Segment::Text(literal)]) }]),
            });
            selected.push(Phrase {
                id: word.id(connector),
                forms: Box::new([Form {
                    class: PluralClass::Other,
                    segments: Box::new([Segment::Text(Box::from(text))]),
                }]),
            });
        }
        let language = |phrases: Vec<Phrase>| Language {
            plural: PluralRule::English,
            span_unit: SpanUnit::Seconds,
            wall: WallFormat::Seconds,
            phrases: phrases.into_boxed_slice(),
        };
        let bounds = PhraseLimits { languages: 2, phrases: 2, holes: 0, segments: 4, text_bytes, hole_bytes: 0 };
        let languages = Box::new([language(original), language(selected)]);
        let book = Phrasebook::load(shapes.clone().into_boxed_slice(), languages.clone(), &bounds).unwrap();
        let selected_words = words::Words { book, language: 1, connector };
        let tree_limits = view().limits().tree;
        let mut builder = skein_ui::Builder::new(&tree_limits);
        for (word, text) in ids.into_iter().zip(texts) {
            assert_eq!(selected_words.say(word, &[]).as_ref(), text);
            selected_words.text(&mut builder, word, &[]);
        }
        let tree = builder.finish();
        assert_eq!(tree.len(), 2);
        assert!(tree.len() <= tree_limits.nodes);
        for (node, text) in tree.nodes().iter().zip(texts) {
            assert!(node.binding.is_none() && node.link.is_none());
            assert_eq!(node.text.as_slice()[0].text.as_ref(), text);
            assert!(node.text.as_slice()[0].text.len() <= usize::try_from(tree_limits.text_per_node).unwrap());
        }
        let mut short = bounds;
        short.text_bytes -= 1;
        let Err(_) = Phrasebook::load(shapes.clone().into_boxed_slice(), languages.clone(), &short) else {
            panic!("one-under phrase byte capacity must be refused");
        };
        for omitted in 0..2 {
            let mut missing = languages.clone();
            missing[1].phrases = missing[1]
                .phrases
                .iter()
                .enumerate()
                .filter(|(index, _)| *index != omitted)
                .map(|(_, phrase)| phrase.clone())
                .collect();
            let Err(_) = Phrasebook::load(shapes.clone().into_boxed_slice(), missing, &bounds) else {
                panic!("each omitted checked caption must be refused");
            };
        }
    }
}

#[test]
fn effect_slots_load_infrastructure_without_loading_a_maker_effect_and_keep_observability_unsupported() {
    let domain_limits = limits();
    let (jig_words, obs_words, infra_words) = phrasebooks();
    let mut tree_limits = Limits::of(&domain_limits, &jig_words, &obs_words, &infra_words).unwrap().tree;
    tree_limits.nodes = 8;
    tree_limits.slots = 0;
    tree_limits.text_bytes = tree_limits.nodes.checked_mul(tree_limits.text_per_node).unwrap();
    for connector in [1, 2] {
        for proposal in [0, u64::MAX] {
            let part = ApplicationPart::OwnerEffect { connector, proposal };
            let mut builder = skein_ui::Builder::new(&tree_limits);
            builder.open(skein_ui::Role::Page);
            let mut bindings = crate::bindings::Bindings::new(0, 32, connector);
            if connector == 1 {
                let mut side = observability::Side::new(&domain_limits.observability, &domain_limits.client.domain);
                let mut out = Queue::uncapped();
                observability::step(
                    &mut side,
                    &Env { now: Time::ZERO, wall: Wall::EPOCH, limits: domain_limits.client.domain },
                    observability::Event::Show { part: part.clone(), scope: observability::Scope::Project(0) },
                    &mut out,
                );
                assert!(side.contains(&part));
                assert!(out.is_empty());
                crate::observability::part(&mut builder, &side, &part, &obs_words, &mut bindings);
            } else {
                let mut side = infrastructure::Side::new(&domain_limits.infrastructure, &domain_limits.client.domain);
                let mut out = Queue::uncapped();
                infrastructure::step(
                    &mut side,
                    &Env { now: Time::ZERO, wall: Wall::EPOCH, limits: domain_limits.client.domain },
                    infrastructure::Event::Show { part: part.clone(), scope: infrastructure::Scope::Project(0) },
                    &mut out,
                );
                assert!(side.contains(&part));
                assert!(out.is_empty());
                crate::infrastructure::part(&mut builder, &side, &part, &infra_words, &jig_words, &mut bindings);
            }
            builder.close();
            let tree = builder.finish();
            let status: Vec<_> = tree.nodes().iter().filter(|node| node.role == skein_ui::Role::Status).collect();
            assert_eq!(status.len(), 1);
            let expected =
                if connector == 1 { b"Cannot show this part".as_slice() } else { b"Loading document".as_slice() };
            assert_eq!(status[0].name.as_deref(), Some(expected));
            // Builder::text creates a Text child under the named Status region.
            let text: Vec<_> =
                tree.nodes().iter().flat_map(|node| node.text.iter()).map(|run| run.text.as_ref()).collect();
            assert_eq!(text, [expected]);
            assert!(tree.nodes().iter().all(|node| node.binding.is_none() && node.link.is_none()));
            assert!(tree.len() <= tree_limits.nodes);
        }
    }
    effect_facts_have_distinct_proposal_and_state_labels();
}

fn effect_facts_have_distinct_proposal_and_state_labels() {
    use infrastructure::values as n;
    let limits = limits();
    let (jig_words, _, words) = phrasebooks();
    let tree_limits = Limits::of(&limits, &jig_words, &words, &words).unwrap().tree;
    let part = ApplicationPart::OwnerEffect { connector: 2, proposal: u64::MAX };
    let mut root = ready();
    let offline = support::offline(&root);
    drive(&mut root, offline);
    let mut out = Queue::uncapped();
    support::reconnect(&mut root, &Env { now: Time::ZERO, wall: Wall::EPOCH, limits }, &mut out);
    let rows = drive(
        &mut root,
        Event::Infrastructure(infrastructure::Event::Show {
            part: part.clone(),
            scope: infrastructure::Scope::Project(0),
        }),
    );
    let read = rows.iter().find_map(|request| match request {
        Request::Infrastructure(infrastructure::Out::Read { read, .. }) => Some(*read),
        _ => None,
    });
    let document = infrastructure::Document::Value(Box::new(n::Document {
        subject: infrastructure::Subject {
            scope: infrastructure::Scope::Project(0),
            kind: infrastructure::Kind::OwnerEffect(u64::MAX),
        },
        value: n::Value::Effect {
            proposal: u64::MAX,
            task: 6,
            effect: n::Effect::Restart {
                service: n::Service { environment: Box::from(*b"production"), name: Box::from(*b"checkout") },
                operation: 77,
            },
            description: n::Description {
                kind: 1,
                resources: Box::new([]),
                condition: true,
                form: n::Form::Set,
                recovery: n::Recovery::Keyed,
                price: None,
                state: 17,
            },
        },
    }));
    drive(
        &mut root,
        Event::Infrastructure(infrastructure::Event::Read {
            read: read.expect("raw effect subject read"),
            result: infrastructure::ReadResult::Value(Box::new(document.clone())),
        }),
    );
    let side = root.infrastructure();
    let mut builder = skein_ui::Builder::new(&tree_limits);
    builder.open(skein_ui::Role::Page);
    let mut bindings = crate::bindings::Bindings::new(tree_limits.bindings, limits.client.domain.text, 2);
    crate::infrastructure::part(&mut builder, side, &part, &words, &jig_words, &mut bindings);
    builder.close();
    let tree = builder.finish();
    let text: Vec<_> = tree.nodes().iter().flat_map(|node| node.text.iter()).map(|run| run.text.as_ref()).collect();
    assert!(text.contains(&b"Proposal number: 18446744073709551615".as_slice()));
    assert!(text.contains(&b"Observed state: 17".as_slice()));
    assert!(!text.contains(&b"Observed state: 18446744073709551615".as_slice()));
    assert_eq!(&side.part(&part).expect("raw effect object").value, &infrastructure::Value::Known(Box::new(document)));
}

#[test]
fn environment_form_uses_the_supplied_request_revision_and_keeps_whole_width_seconds() {
    use infrastructure::values as n;
    let environment = n::Environment { pool: Box::from(*b"Q.pool"), name: Box::from(*b"preview") };
    let subject = infrastructure::Subject {
        scope: infrastructure::Scope::Project(3),
        kind: infrastructure::Kind::Environment(environment.clone()),
    };
    let document = infrastructure::Document::Value(Box::new(n::Document {
        subject: subject.clone(),
        value: n::Value::Environment {
            environment: environment.clone(),
            end_revision: 17,
            fact: n::EnvironmentFact {
                created_by: n::Key {
                    deployment: [7; 16],
                    task: 9,
                    origin: n::Purpose::Procedure { purpose: 2 },
                    purpose: 4,
                },
                ready_at: u64::MAX,
                until: u64::MAX,
                price: 12,
            },
            end: None,
            pool: None,
        },
    }));
    let mut root = ready();
    let requests = drive(
        &mut root,
        Event::Went {
            place: jig_ops_client_domain::Place::Address(Address::Infrastructure(
                infrastructure::Address::Environment { project: 3, environment: environment.clone() },
            )),
        },
    );
    let read = requests
        .iter()
        .find_map(|request| match request {
            Request::Infrastructure(infrastructure::Out::Read { read, .. }) => Some(*read),
            _ => None,
        })
        .expect("values read");
    drive(
        &mut root,
        Event::Infrastructure(infrastructure::Event::Read {
            read,
            result: infrastructure::ReadResult::Value(Box::new(document)),
        }),
    );
    let mut view = view();
    let mut patches = Queue::uncapped();
    build(&mut view, &root, &mut patches);
    assert!(contains(&view, b"Observed backend lease (seconds since epoch): 18446744073709551615"));
    assert!(contains(&view, b"Ready at (seconds since epoch): 18446744073709551615"));
    assert!(!contains(&view, b"Desired end"));
    let id = root.infrastructure().find(subject).expect("opaque environment");
    let field = view
        .tree()
        .nodes()
        .iter()
        .find(|node| {
            node.role == skein_ui::Role::TextField
                && node.name.as_deref() == Some(b"New end (seconds since epoch)".as_slice())
        })
        .expect("values end field");
    assert_eq!(field.value.as_ref().expect("values value").text.as_ref(), b"18446744073709551615");
    assert_eq!(field.value.as_ref().expect("values value").written, 17);
    let field = field.id;
    assert_eq!(
        crate::decode(&mut view, field, skein_ui::Act::Input { value: Box::from(*b"18446744073709551614") }),
        Some(jig_ops_client_domain::Action::Infrastructure(infrastructure::Act::Until {
            object: id.token(),
            revision: Revision { value: 17 },
            value: u64::MAX - 1
        }))
    );
}

fn configured_deployment(connector: u16) -> Deployment {
    Deployment {
        store: StoreFill { bytes: 0, mark: 0 },
        hosts: list([]),
        attempts: list([]),
        accounts: list([]),
        tasks: Capacity { used: 0, limit: 1 },
        workers: Capacity { used: 0, limit: 1 },
        attempts_capacity: Capacity { used: 0, limit: 1 },
        waiting_attempts: 0,
        proposals: Capacity { used: 0, limit: 1 },
        projects: list([]),
        grace_bound: Duration::from_secs(1),
        hello_refusals: list([]),
        hosts_forgotten: 0,
        hello_refusals_forgotten: 0,
        person_room_refusals: list([]),
        person_room_refusals_forgotten: 0,
        person_answer_retention: Duration::from_secs(1),
        task_room_refusals: list([]),
        task_room_refusals_forgotten: 0,
        cut: false,
        connectors: list([connector]),
        connectors_cut: false,
        providers: list([]),
        providers_cut: false,
        key_bytes: 256,
        readiness: list([]),
        readiness_cut: false,
    }
}
