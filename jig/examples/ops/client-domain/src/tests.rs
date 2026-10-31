#![expect(
    clippy::disallowed_types,
    clippy::disallowed_methods,
    clippy::wildcard_enum_match_arm,
    clippy::match_like_matches_macro,
    reason = "test fixtures collect observed root outputs and locate their variants"
)]
#[path = "tests_client.rs"]
mod client_surface;
use crate as support_domain;
#[path = "tests_support.rs"]
mod support;
use crate::{Address, Answered, Ask, Event, Limits, infrastructure};
use jig_client::contract::{self, Delivery, DocumentText, HeldRole, Me, MeParty, Needs, PartyKind, WatchEvent};
use skein_lib::{Duration, Env, List, Queue, RequestEnvelope, Time, Token, Wall};
use support::{Domain, JigOut, Request, new, step, take};
const JIG: jig_client_view::Config = jig_client_view::Config {
    notices: 2,
    notice_life: Duration::from_secs(5),
    linger: Duration::from_secs(5),
    save_delay: Duration::from_secs(1),
    intents: 2,

    objects: 2,
    sources: 8,

    drafts: 1,

    roles: 2,

    conversation_words: 2,
    navigation_entries: 8,
    updates_count: 99,
};
fn limits() -> Limits {
    Limits {
        jig: JIG,
        infrastructure: infrastructure::Limits { objects: 12 },
        observability: crate::observability::Limits { objects: 12 },
        client: client_limits(),
    }
}
fn env() -> Env<Limits> {
    Env { now: Time::ZERO, wall: Wall::EPOCH, limits: limits() }
}
fn drive(root: &mut Domain, event: Event) -> Vec<Request> {
    let mut out = Queue::uncapped();
    step(root, &env(), event, &mut out);
    out_iter(out)
}
fn out_iter(mut out: Queue<Request>) -> Vec<Request> {
    let mut rows = Vec::new();
    while let Some(row) = out.pop() {
        rows.push(row);
    }
    rows
}
fn taken(root: &mut Domain) -> Vec<Request> {
    let mut out = Queue::uncapped();
    take(root, &env(), &mut out);
    out_iter(out)
}
fn start(root: &mut Domain, kept: List<crate::store::Record>) -> Token {
    let mut rows = drive(root, Event::Client { event: jig_client::Event::Up { up: jig_client::Up::SignedIn } });
    rows.extend(drive(
        root,
        Event::Start { address: Address::Jig(jig_client_view::address::Address::Inbox { project: None }), kept },
    ));
    rows.iter()
        .find_map(|row| match row {
            Request::Jig(JigOut::Open { watch, subject: contract::Subject::Me }) => Some(*watch),
            _ => None,
        })
        .unwrap()
}
fn confirm(root: &mut Domain, watch: Token, party: u64) {
    drive(
        root,
        Event::Client { event: jig_client::Event::Up { up: jig_client::Up::Served { served: true, versions: None } } },
    );
    // Explicit synthetic identity and empty offers, supplied independently of the party's display name.
    let frame = Me {
        providers: List::with_capacity(0),
        providers_cut: false,
        key_bytes: 256,
        redirect_bytes: 4096,
        party: MeParty {
            number: party,
            kind: PartyKind::Person,
            login: DocumentText::complete(contract::Text::new(Box::from(*b"fixture-login"), 32).unwrap()),
            name: DocumentText::complete(contract::Text::new(Box::from(*b"Pat"), 32).unwrap()),
        },
        projects: List::with_capacity(0),
        needs: Needs { questions: 0, proposals: 0, escalations: 0, person_tasks: 0 },
        needs_total: 0,
        updates: contract::Count { value: 0, limited: false },
        results_seen: 0,
        cut: false,
        retention: Duration::from_secs(1000),
        deployment_role: Some(HeldRole::Policy { role: 1 }),
    };
    drive(
        root,
        Event::Client {
            event: jig_client::Event::Up {
                up: jig_client::Up::Watched { watch, event: WatchEvent::Delivered(Delivery::Me(Box::new(frame))) },
            },
        },
    );
}
fn ready(seed: u64) -> Domain {
    let mut root = new(&limits(), seed);
    let watch = start(&mut root, List::with_capacity(0));
    confirm(&mut root, watch, 7);
    root
}
fn environment_name() -> infrastructure::values::Environment {
    infrastructure::values::Environment { pool: Box::from(*b"Q.pool"), name: Box::from(*b"preview") }
}
fn environment(revision: u64) -> infrastructure::Document {
    use infrastructure::values as n;
    infrastructure::Document::Value(Box::new(n::Document {
        subject: infrastructure::Subject {
            scope: infrastructure::Scope::Project(3),
            kind: infrastructure::Kind::Environment(environment_name()),
        },
        value: n::Value::Environment {
            environment: environment_name(),
            end_revision: revision,
            fact: n::EnvironmentFact {
                created_by: n::Key {
                    deployment: [7; 16],
                    task: 9,
                    origin: n::Purpose::Procedure { purpose: 2 },
                    purpose: 4,
                },
                ready_at: 0,
                until: 50,
                price: 0,
            },
            end: Some(n::EnvironmentEnd { revision, until: 50 }),
            pool: None,
        },
    }))
}
fn open_families(root: &mut Domain, rows: &[Request]) {
    for row in rows {
        let watch = match row {
            Request::Infrastructure(infrastructure::Out::Open { watch, .. })
            | Request::Observability(crate::observability::Out::Open { watch, .. }) => *watch,
            _ => continue,
        };
        drive(
            root,
            Event::Client {
                event: jig_client::Event::Up { up: jig_client::Up::Watched { watch, event: WatchEvent::Opened } },
            },
        );
    }
}
fn family_page(root: &mut Domain) -> Vec<Request> {
    drive(
        root,
        Event::Went {
            place: crate::Place::Address(Address::Infrastructure(infrastructure::Address::Environment {
                project: 3,
                environment: environment_name(),
            })),
        },
    )
}
fn family_read(rows: &[Request]) -> Token {
    rows.iter()
        .find_map(|row| match row {
            Request::Infrastructure(infrastructure::Out::Read { read, .. }) => Some(*read),
            _ => None,
        })
        .unwrap()
}
fn family_watch(rows: &[Request]) -> Token {
    rows.iter()
        .find_map(|row| match row {
            Request::Infrastructure(infrastructure::Out::Open { watch, .. }) => Some(*watch),
            _ => None,
        })
        .unwrap()
}
fn load_environment(root: &mut Domain) -> skein_lib::Id<infrastructure::Object> {
    let rows = family_page(root);
    drive(
        root,
        Event::Infrastructure(infrastructure::Event::Read {
            read: family_read(&rows),
            result: infrastructure::ReadResult::Value(Box::new(environment(2))),
        }),
    );
    root.infrastructure().page().unwrap()
}
fn until(root: &mut Domain, object: Token) {
    drive(
        root,
        Event::Act {
            action: crate::Action::Infrastructure(infrastructure::Act::Until {
                object,
                revision: contract::Revision { value: 2 },
                value: 100,
            }),
        },
    );
}
#[test]
fn an_end_move_names_current_revision_and_only_durable_answer_finishes_the_card() {
    let mut root = ready(41);
    let object = load_environment(&mut root);
    until(&mut root, object.token());
    drive(
        &mut root,
        Event::Act {
            action: crate::Action::Infrastructure(infrastructure::Act::MoveEnd {
                object: object.token(),
                revision: contract::Revision { value: 1 },
            }),
        },
    );
    assert_eq!(
        root.infrastructure().object(object).unwrap().card,
        infrastructure::Card::Refused {
            answer: infrastructure::Answer::Moved {
                environment: environment_name(),
                revision: contract::Revision { value: 2 },
                until: 50
            }
        }
    );
    assert!(!taken(&mut root).iter().any(|row| match row {
        Request::Send { .. } => true,
        _ => false,
    }));
    drive(
        &mut root,
        Event::Act {
            action: crate::Action::Infrastructure(infrastructure::Act::MoveEnd {
                object: object.token(),
                revision: contract::Revision { value: 2 },
            }),
        },
    );
    let rows = taken(&mut root);
    let (attempt, ask) = rows
        .iter()
        .find_map(|row| match row {
            Request::Send { attempt, ask: Ask::Infrastructure(ask), .. } => Some((*attempt, ask)),
            _ => None,
        })
        .unwrap();
    assert_eq!(
        ask,
        &infrastructure::Ask::MoveEnd {
            object: object.token(),
            project: 3,
            environment: environment_name(),
            revision: contract::Revision { value: 2 },
            until: 100
        }
    );
    let answer = infrastructure::Answer::MovedEnd {
        environment: environment_name(),
        revision: contract::Revision { value: 3 },
        until: 100,
    };
    drive(&mut root, support::answer(attempt, RequestEnvelope::Again, Some(Answered::Infrastructure(answer.clone()))));
    assert!(match root.infrastructure().object(object).unwrap().card {
        infrastructure::Card::Deciding { .. } => true,
        _ => false,
    });
    // A retry envelope did not deliver this body to the child.
    let mut future = env();
    future.now = Time::from_nanos(80);
    let mut out = Queue::uncapped();
    take(&mut root, &future, &mut out);
    let attempt = out_iter(out)
        .iter()
        .find_map(|row| match row {
            Request::Send { attempt, .. } => Some(*attempt),
            _ => None,
        })
        .unwrap();
    drive(&mut root, support::answer(attempt, RequestEnvelope::Final, Some(Answered::Infrastructure(answer.clone()))));
    assert_eq!(root.infrastructure().object(object).unwrap().card, infrastructure::Card::Done { answer });
}
#[test]
fn environment_end_revision_changes_clear_drafts() {
    let mut root = ready(42);
    let rows = family_page(&mut root);
    let watch = family_watch(&rows);
    drive(
        &mut root,
        Event::Infrastructure(infrastructure::Event::Read {
            read: family_read(&rows),
            result: infrastructure::ReadResult::Value(Box::new(environment(2))),
        }),
    );
    let object = root.infrastructure().page().unwrap();
    until(&mut root, object.token());
    drive(
        &mut root,
        Event::Infrastructure(infrastructure::Event::Watch {
            watch,
            event: infrastructure::WatchEvent::Delivered(Box::new(environment(3))),
        }),
    );
    assert_eq!(root.infrastructure().object(object).unwrap().draft, None);
}
#[test]
fn abandoned_read_and_closing_watch_tokens_cannot_recreate_a_page() {
    let mut root = ready(43);
    let rows = family_page(&mut root);
    let watch = family_watch(&rows);
    let read = family_read(&rows);
    open_families(&mut root, &rows);
    let rows = drive(
        &mut root,
        Event::Went {
            place: crate::Place::Address(Address::Jig(jig_client_view::address::Address::Inbox { project: None })),
        },
    );
    assert!(rows.contains(&Request::Infrastructure(infrastructure::Out::Close { watch })));
    drive(
        &mut root,
        Event::Infrastructure(infrastructure::Event::Watch {
            watch,
            event: infrastructure::WatchEvent::Delivered(Box::new(environment(2))),
        }),
    );
    drive(
        &mut root,
        Event::Infrastructure(infrastructure::Event::Ended { watch, end: infrastructure::WatchEnd::Closed }),
    );
    drive(
        &mut root,
        Event::Infrastructure(infrastructure::Event::Read {
            read,
            result: infrastructure::ReadResult::Value(Box::new(environment(2))),
        }),
    );
    assert_eq!(root.infrastructure().page(), None);
    assert!(
        drive(
            &mut root,
            Event::Infrastructure(infrastructure::Event::Ended { watch, end: infrastructure::WatchEnd::Closed })
        )
        .is_empty()
    );
}
#[test]
fn whole_document_limits_keep_unknown_owner_facts_unknown() {
    let document = environment(2);
    assert!(document.fits(&limits().client.domain));
    let mut bound = limits().client.domain;
    bound.text = 2;
    assert!(!document.fits(&bound));
    bound.text = 32;
    bound.document_bytes = 1;
    assert!(!document.fits(&bound));
    let mut root = ready(44);
    let object = load_environment(&mut root);
    assert_eq!(root.infrastructure().object(object).unwrap().value, infrastructure::Value::Known(Box::new(document)));
    let mut bound = limits();
    bound.client.domain.kept = bound.client.domain.requests.requests - 1;
    assert!(crate::valid(&bound));
}
fn incidents_naming_sections(lead: Option<u64>) -> [crate::observability::Document; 1] {
    use crate::observability::{documents::*, values};
    let values = values::Document {
        subject: Subject { scope: Scope::Project(3), kind: Kind::Incident(8) },
        value: values::Value::Incident {
            incident: values::Incident {
                project: 3,
                number: 8,
                watch: 99,
                lead,
                alerts: Box::from([]),
                raised: Wall::EPOCH,
                service: Service { environment: Box::from(*b"prod"), name: Box::from(*b"api") },
                severity: 1,
                state: State::Alerting,
                timeline: List::with_capacity(0),
                cut: false,
            },
            fact: None,
        },
    };
    [Document::Value(Box::new(values))]
}

#[test]
fn an_incident_with_a_lead_names_its_plan_section() {
    let expected =
        [jig_client_view::page::Section::Task { task: 9, section: jig_client_view::address::TaskSection::Plan }];
    for (name, document) in ["values"].into_iter().zip(incidents_naming_sections(Some(9))) {
        assert_eq!(document.sections().as_slice(), expected, "{name}");
    }
}

#[test]
fn an_incident_without_a_lead_names_no_sections() {
    for (name, document) in ["values"].into_iter().zip(incidents_naming_sections(None)) {
        assert_eq!(document.sections().as_slice(), [], "{name}");
    }
}
#[test]
fn application_navigation_is_forwarded_before_start_and_retains_order() {
    use jig_client_view::page::navigation::{Destination, Navigation};
    let mut root = new(&limits(), 47);
    let mut navigation = Navigation::new(2);
    navigation.entries.push(Destination::Jig(jig_client_view::address::Address::Inbox { project: None })).unwrap();
    navigation
        .entries
        .push(Destination::Application {
            owner: jig_ops_domain_connectors::number(jig_ops_domain_connectors::Connector::Observability),
            number: 1,
        })
        .unwrap();
    crate::configure_navigation(&mut root, navigation).unwrap();
    assert_eq!(
        root.jig().navigation().entries.as_slice()[1],
        Destination::Application {
            owner: jig_ops_domain_connectors::number(jig_ops_domain_connectors::Connector::Observability),
            number: 1
        }
    );
    start(&mut root, List::with_capacity(0));
    assert!(crate::configure_navigation(&mut root, Navigation::new(0)).is_err());
}

#[test]
fn observability_project_lists_are_admitted_and_priced_whole() {
    use crate::observability::{
        documents::{Document, Kind, Scope, Service, Subject},
        values as n,
    };
    let mut services = List::with_capacity(2);
    services
        .push(n::ServiceState {
            service: Service { environment: Box::from(*b"production"), name: Box::from(*b"checkout") },
            fact: None,
        })
        .unwrap();
    let mut part = n::Document {
        subject: Subject { scope: Scope::Project(1), kind: Kind::Project(1) },
        value: n::Value::Project { project: 1, services, incidents: List::with_capacity(2) },
    };
    assert!(Document::Value(Box::new(part.clone())).fits(&limits().client.domain));
    let n::Value::Project { services, .. } = &mut part.value else { unreachable!() };
    *services = List::with_capacity(3);
    assert!(!Document::Value(Box::new(part)).fits(&limits().client.domain));
}

#[test]
fn an_entrance_refusal_finishes_only_its_original_child_and_survives_a_duplicate_final() {
    let mut root = ready(5091);
    let object = load_environment(&mut root);
    until(&mut root, object.token());
    drive(
        &mut root,
        Event::Act {
            action: crate::Action::Infrastructure(infrastructure::Act::MoveEnd {
                object: object.token(),
                revision: contract::Revision { value: 2 },
            }),
        },
    );
    let attempt = taken(&mut root)
        .iter()
        .find_map(|row| match row {
            Request::Send { attempt, .. } => Some(*attempt),
            _ => None,
        })
        .unwrap();
    let refusal = contract::EntranceRefusal::new(contract::Refusal::Authority {
        lacking: contract::Lacking::Budget { needed: 10, left: 3 },
        proposable: None,
    })
    .unwrap();
    let answer = infrastructure::Answer::EntranceRefused(refusal);
    drive(&mut root, support::answer(attempt, RequestEnvelope::Again, Some(Answered::Infrastructure(answer.clone()))));
    assert!(match root.infrastructure().object(object).unwrap().card {
        infrastructure::Card::Deciding { .. } => true,
        _ => false,
    });
    let mut future = env();
    future.now = Time::from_nanos(80);
    let mut out = Queue::uncapped();
    take(&mut root, &future, &mut out);
    let attempt = out_iter(out)
        .iter()
        .find_map(|row| match row {
            Request::Send { attempt, .. } => Some(*attempt),
            _ => None,
        })
        .unwrap();
    drive(&mut root, support::answer(attempt, RequestEnvelope::Final, Some(Answered::Infrastructure(answer.clone()))));
    let card = infrastructure::Card::Refused { answer };
    assert_eq!(root.infrastructure().object(object).unwrap().card, card);
    drive(
        &mut root,
        support::answer(attempt, RequestEnvelope::Final, Some(Answered::Infrastructure(infrastructure::Answer::Gone))),
    );
    assert_eq!(root.infrastructure().object(object).unwrap().card, card);
}

#[test]
fn outbox_subjects_copy_the_supplied_scope_and_numeric_entry_independently_of_effects() {
    use crate::{infrastructure as infra, observability as obs};
    let proposal = contract::ProposalKey { proposer: contract::Proposer::Task(17), number: 17 };
    let effect = contract::Part::Effect { connector: 2, proposal };
    for scope in [contract::PartScope::Project(91), contract::PartScope::Deployment] {
        let part = contract::Part::Outbox { connector: 2, scope, entry: 17 };
        let infrastructure = infra::documents::subject(&part, infra::Scope::Project(3)).unwrap();
        let observability = obs::documents::subject(&part, obs::Scope::Project(3)).unwrap();
        assert_eq!(infrastructure.kind, infra::Kind::Outbox(17));
        assert_eq!(observability.kind, obs::Kind::Outbox(17));
        match scope {
            contract::PartScope::Project(project) => {
                assert_eq!(infrastructure.scope, infra::Scope::Project(project));
                assert_eq!(observability.scope, obs::Scope::Project(project));
            }
            contract::PartScope::Deployment => {
                assert_eq!(infrastructure.scope, infra::Scope::Deployment);
                assert_eq!(observability.scope, obs::Scope::Deployment);
            }
        }
        assert_ne!(infra::documents::subject(&effect, infrastructure.scope), Some(infrastructure));
        assert_ne!(obs::documents::subject(&effect, observability.scope), Some(observability));
    }
}

#[test]
fn resource_parts_bound_allocated_segments_and_whole_segment_bytes_in_both_connectors() {
    let limits = limits();
    for connector in [1, 2] {
        let mut valid = List::with_capacity(2);
        valid.push(Box::<[u8]>::from([0xFF; 32])).unwrap();
        valid.push(Box::<[u8]>::from(*b"/\0")).unwrap();
        let empty_overallocated = List::with_capacity(5);
        let mut too_long = List::with_capacity(1);
        too_long.push(Box::<[u8]>::from([b'x'; 33])).unwrap();
        for (path, accepted) in [(valid, true), (empty_overallocated, false), (too_long, false)] {
            let part = contract::Part::Resource { connector, path };
            if connector == 1 {
                let mut side = crate::observability::Side::new(&limits.observability, &limits.client.domain);
                let mut out = Queue::uncapped();
                crate::observability::step(
                    &mut side,
                    &Env { now: Time::ZERO, wall: Wall::EPOCH, limits: limits.client.domain },
                    crate::observability::Event::Show {
                        part: part.clone(),
                        scope: crate::observability::Scope::Project(1),
                    },
                    &mut out,
                );
                assert_eq!(side.contains(&part), accepted);
                if accepted {
                    assert!(out.is_empty());
                } else {
                    let Some(crate::observability::Request::Out(crate::observability::Out::Fact { fact })) = out.pop()
                    else {
                        panic!("rejected resource emits its Full fact");
                    };
                    assert_eq!(fact, crate::observability::Fact::Full);
                    assert!(out.is_empty());
                }
            } else {
                let mut side = infrastructure::Side::new(&limits.infrastructure, &limits.client.domain);
                let mut out = Queue::uncapped();
                infrastructure::step(
                    &mut side,
                    &Env { now: Time::ZERO, wall: Wall::EPOCH, limits: limits.client.domain },
                    infrastructure::Event::Show { part: part.clone(), scope: infrastructure::Scope::Project(1) },
                    &mut out,
                );
                assert_eq!(side.contains(&part), accepted);
                if accepted {
                    assert!(out.is_empty());
                } else {
                    let Some(infrastructure::Request::Out(infrastructure::Out::Fact { fact })) = out.pop() else {
                        panic!("rejected resource emits its Full fact");
                    };
                    assert_eq!(fact, infrastructure::Fact::Full);
                    assert!(out.is_empty());
                }
            }
        }
    }
}

#[test]
fn effect_parts_keep_connector_numbers_separate_from_maker_subjects() {
    use crate::observability as obs;
    let limits = limits();
    for connector in [1, 2, u16::MAX] {
        for number in [0, u64::MAX] {
            let values = contract::Part::OwnerEffect { connector, proposal: number };
            assert_eq!(crate::parts::connector(&values), connector);
            for proposer in [contract::Proposer::Task(number), contract::Proposer::Party(number)] {
                let key = contract::ProposalKey { proposer, number };
                let maker_effect = contract::Part::Effect { connector, proposal: key };
                assert_ne!(values, maker_effect);
                for project in [0, u32::MAX] {
                    assert_eq!(
                        infrastructure::documents::subject(&values, infrastructure::Scope::Project(project)),
                        Some(infrastructure::Subject {
                            scope: infrastructure::Scope::Project(project),
                            kind: infrastructure::Kind::OwnerEffect(number),
                        })
                    );
                    assert_eq!(obs::documents::subject(&values, obs::Scope::Project(project)), None);
                    assert_eq!(
                        infrastructure::documents::subject(&maker_effect, infrastructure::Scope::Project(project)),
                        Some(infrastructure::Subject {
                            scope: infrastructure::Scope::Project(project),
                            kind: infrastructure::Kind::Effect(key)
                        })
                    );
                    assert_eq!(
                        obs::documents::subject(&maker_effect, obs::Scope::Project(project)),
                        Some(obs::Subject { scope: obs::Scope::Project(project), kind: obs::Kind::Effect(key) })
                    );
                }
            }
            assert_eq!(infrastructure::documents::subject(&values, infrastructure::Scope::Deployment), None);
            assert_eq!(obs::documents::subject(&values, obs::Scope::Deployment), None);
            let mut infra = infrastructure::Side::new(&limits.infrastructure, &limits.client.domain);
            let mut infra_out = Queue::uncapped();
            let infra_env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits: limits.client.domain };
            infrastructure::step(
                &mut infra,
                &infra_env,
                infrastructure::Event::Show { part: values.clone(), scope: infrastructure::Scope::Deployment },
                &mut infra_out,
            );
            assert!(infra.contains(&values));
            assert!(infra.part(&values).is_none());
            assert!(infra_out.is_empty());
            infrastructure::step(&mut infra, &infra_env, infrastructure::Event::PartsLeft, &mut infra_out);
            assert!(!infra.contains(&values));
            assert!(infra_out.is_empty());
            let mut side = obs::Side::new(&limits.observability, &limits.client.domain);
            let mut out = Queue::uncapped();
            let side_env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits: limits.client.domain };
            obs::step(
                &mut side,
                &side_env,
                obs::Event::Show { part: values.clone(), scope: obs::Scope::Deployment },
                &mut out,
            );
            assert!(side.contains(&values));
            assert!(side.part(&values).is_none());
            assert!(out.is_empty());
            obs::step(&mut side, &side_env, obs::Event::PartsLeft, &mut out);
            assert!(!side.contains(&values));
            assert!(out.is_empty());
        }
    }
}

#[test]
#[expect(
    clippy::too_many_lines,
    reason = "one whole environment request exercises document, draft, journal and opaque identity"
)]
fn environment_names_request_revision_and_whole_width_end_survive_the_client_request() {
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
            end_revision: 0,
            fact: n::EnvironmentFact {
                created_by: n::Key {
                    deployment: [7; 16],
                    task: 9,
                    origin: n::Purpose::Procedure { purpose: 2 },
                    purpose: 4,
                },
                ready_at: 5,
                until: 50,
                price: 12,
            },
            end: None,
            pool: None,
        },
    }));
    assert!(document.fits(&limits().client.domain));
    let mut root = ready(512);
    let requests = drive(
        &mut root,
        Event::Went {
            place: crate::Place::Address(Address::Infrastructure(infrastructure::Address::Environment {
                project: 3,
                environment: environment.clone(),
            })),
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
            result: infrastructure::ReadResult::Value(Box::new(document.clone())),
        }),
    );
    let id = root.infrastructure().find(subject.clone()).expect("opaque subject retained");
    assert_eq!(
        root.infrastructure().object(id).expect("named object").value,
        infrastructure::Value::Known(Box::new(document))
    );
    drive(
        &mut root,
        Event::Act {
            action: crate::Action::Infrastructure(infrastructure::Act::Until {
                object: id.token(),
                revision: contract::Revision { value: 0 },
                value: u64::MAX,
            }),
        },
    );
    let requests = drive(
        &mut root,
        Event::Act {
            action: crate::Action::Infrastructure(infrastructure::Act::MoveEnd {
                object: id.token(),
                revision: contract::Revision { value: 0 },
            }),
        },
    );
    assert!(requests.is_empty(), "the table journals before sending");
    let requests = taken(&mut root);
    assert!(requests.iter().any(|request| {
        match request {
            Request::Save { record: crate::store::Record::Unlinked(record) } => {
                crate::wire::ask_value(match &record.request.request {
                    jig_client::Ask::Family { record: value, .. } => &value.bytes,
                    jig_client::Ask::Jig(_) => panic!("family ask"),
                })
                .unwrap()
                    == (infrastructure::Ask::MoveEnd {
                        object: id.token(),
                        project: 3,
                        environment: environment.clone(),
                        revision: contract::Revision { value: 0 },
                        until: u64::MAX,
                    })
            }
            _ => false,
        }
    }));
    let mut different = environment.clone();
    different.pool = Box::from(*b"P.pool");
    assert!(
        root.infrastructure()
            .find(infrastructure::Subject {
                scope: infrastructure::Scope::Project(3),
                kind: infrastructure::Kind::Environment(different)
            })
            .is_none()
    );
}

#[test]
fn infrastructure_reopens_a_dropped_watch_after_the_link_goes_offline_and_live() {
    let mut root = ready(61);
    let initial = family_page(&mut root);
    let old = family_watch(&initial);
    drive(
        &mut root,
        Event::Infrastructure(infrastructure::Event::Read {
            read: family_read(&initial),
            result: infrastructure::ReadResult::Value(Box::new(environment(2))),
        }),
    );
    drive(
        &mut root,
        Event::Client {
            event: jig_client::Event::Up {
                up: jig_client::Up::WatchEnded { watch: old, end: contract::WatchEnd::Dropped },
            },
        },
    );
    let offline = support::offline(&root);
    drive(&mut root, offline);
    let id = root.infrastructure().page().unwrap();
    let mut out = Queue::uncapped();
    support::reconnect(&mut root, &env(), &mut out);
    let rows = out_iter(out);
    let watch = family_watch(&rows);
    assert_ne!(old, watch, "the retired watch epoch is not reused");
    let subject = root.infrastructure().object(id).unwrap().subject.clone();
    assert!(rows.iter().any(|row| match row {
        Request::Infrastructure(infrastructure::Out::Open { subject: got, .. }) => got == &subject,
        _ => false,
    }));
    drive(
        &mut root,
        Event::Infrastructure(infrastructure::Event::Ended { watch: old, end: infrastructure::WatchEnd::NotAllowed }),
    );
    let document = environment(3);
    drive(
        &mut root,
        Event::Infrastructure(infrastructure::Event::Watch {
            watch,
            event: infrastructure::WatchEvent::Delivered(Box::new(document.clone())),
        }),
    );
    assert_eq!(root.infrastructure().object(id).unwrap().value, infrastructure::Value::Known(Box::new(document)));
}

#[test]
fn observability_reopens_a_dropped_watch_after_the_link_goes_offline_and_live() {
    use crate::observability;
    let mut root = ready(62);
    let initial = drive(
        &mut root,
        Event::Went {
            place: crate::Place::Address(Address::Observability(observability::Address::Incidents {
                project: 3,
                open: true,
            })),
        },
    );
    let old = initial
        .iter()
        .find_map(|row| match row {
            Request::Observability(observability::Out::Open { watch, .. }) => Some(*watch),
            _ => None,
        })
        .unwrap();
    drive(
        &mut root,
        Event::Client {
            event: jig_client::Event::Up {
                up: jig_client::Up::WatchEnded { watch: old, end: contract::WatchEnd::Dropped },
            },
        },
    );
    let offline = support::offline(&root);
    drive(&mut root, offline);
    let id = root.observability().page().unwrap();
    let mut out = Queue::uncapped();
    support::reconnect(&mut root, &env(), &mut out);
    let rows = out_iter(out);
    let watch = rows
        .iter()
        .find_map(|row| match row {
            Request::Observability(observability::Out::Open { watch, .. }) => Some(*watch),
            _ => None,
        })
        .unwrap();
    assert_ne!(old, watch, "the retired watch epoch is not reused");
    let subject = root.observability().object(id).unwrap().subject;
    assert!(rows.iter().any(|row| match row {
        Request::Observability(observability::Out::Open { subject: got, .. }) => got == &subject,
        _ => false,
    }));
    drive(
        &mut root,
        Event::Observability(observability::Event::Ended { watch: old, end: observability::WatchEnd::NotAllowed }),
    );
    let document = observability::Document::Value(Box::new(observability::values::Document {
        subject,
        value: observability::values::Value::Project {
            project: 3,
            services: List::with_capacity(0),
            incidents: List::with_capacity(2),
        },
    }));
    drive(
        &mut root,
        Event::Observability(observability::Event::Watch {
            watch,
            event: observability::WatchEvent::Delivered(Box::new(document.clone())),
        }),
    );
    assert_eq!(root.observability().object(id).unwrap().value, observability::Value::Known(Box::new(document)));
}

fn client_limits() -> jig_client::Limits {
    support::client_limits()
}
