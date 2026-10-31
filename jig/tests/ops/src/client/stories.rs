//! P and Q's supplied owner facts and public-tree observations.
#![cfg_attr(
    not(test),
    expect(clippy::wildcard_imports, reason = "story fixtures enumerate the complete client vocabulary")
)]
use super::{Document, Ops, OpsScript, Scripted, Subject};
use jig_client_domain::contract::*;
use jig_client_scripted_engine::{Bounds, Script, Truth};
use jig_client_view::Owner;
use jig_client_view::address::{Address as JigAddress, TaskSection};
use jig_client_world::{
    Settings, World,
    harness::{CardKey, CardSeen, Seen},
    observe,
    stories::{decided_proposal, frame, proposal, task, text, turn_event},
};
use jig_ops_client_domain::{Address, infrastructure as infra, observability as obs};
use jig_ops_domain_connectors::{Connector, number};
use skein_fake_person::{Face, Find, Found, TreeFace};
use skein_lib::{Duration, Wall};
use skein_ui::{Role, Tree as ViewTree};
/// The same independent shell world used by the reference client.
pub type ClientWorld = World<Ops, Scripted>;
/// Explicit incident family address.
#[must_use]
pub fn incident() -> Address {
    Address::Observability(obs::Address::Incident { project: 1, incident: 12 })
}
/// Explicit lead conversation supplied by the incident owner.
#[must_use]
pub fn lead() -> Address {
    Address::Jig(JigAddress::Task { task: 9, section: TaskSection::Activity })
}
/// Explicit environment address, independent of its creation purpose.
#[must_use]
pub fn environment() -> Address {
    Address::Infrastructure(infra::Address::Environment { project: 1, environment: environment_name() })
}
pub(super) fn seen(tree: &ViewTree) -> Vec<Seen> {
    let mut seen = observe::jig_seen(tree);
    for (at, node) in tree.nodes().iter().enumerate() {
        let Some(key) = node.key else { continue };
        if key.owner != ((Owner::Application { number: number(Connector::Infrastructure) }).opaque())
            || key.kind != 12
            || node.role != Role::Item
        {
            continue;
        }
        let mut face = TreeFace::subtree(tree, node.id).expect("public card subtree");
        let has = |face: &mut TreeFace<'_>, words: &[u8]| {
            matches!(face.find(&Find::named(Role::Status, words)), Found::Node(_))
        };
        let state = if has(&mut face, b"In flight") || has(&mut face, b"Retrying") || has(&mut face, b"Parked") {
            CardSeen::Deciding
        } else if has(&mut face, b"End moved") || has(&mut face, b"Moved to revision") || has(&mut face, b"Gone") {
            CardSeen::Done
        } else {
            CardSeen::Waiting
        };
        // The form supplies the owner's end revision as the public
        // field's write version, alongside the whole client object identity.
        let revision = tree.nodes()[at..at + skein_lib::length::usize_of(node.size)]
            .iter()
            .find(|field| field.role == Role::TextField)
            .and_then(|field| field.value.as_ref())
            .map(|value| value.written);
        seen.push(Seen::Card { key: CardKey::of(key), state, revision });
    }
    seen
}
#[must_use]
pub fn incident_document() -> obs::values::Document {
    let previous = obs::values::Health { observed: 36_000, healthy_replicas: 3, errors: 2, error_rate_percent: 2 };
    obs::values::Document {
        subject: obs::Subject { scope: obs::Scope::Project(1), kind: obs::Kind::Incident(12) },
        value: obs::values::Value::Incident {
            incident: obs::values::Incident {
                project: 1,
                number: 12,
                watch: 888,
                raised: Wall::from_nanos(36_000_000_000_000),
                service: obs::documents::Service {
                    environment: b"production".as_slice().into(),
                    name: b"checkout".as_slice().into(),
                },
                severity: 2,
                state: obs::documents::State::Alerting,
                timeline: list([obs::values::Point {
                    at: Wall::from_nanos(36_000_000_000_000),
                    health: Some(previous),
                }]),
                cut: false,
                lead: Some(9),
                alerts: Box::new([]),
            },
            fact: Some(obs::values::Fact {
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
#[must_use]
pub fn environment_name() -> infra::values::Environment {
    infra::values::Environment { pool: b"Q pool".as_slice().into(), name: b"staging-7".as_slice().into() }
}
#[must_use]
pub fn environment_document() -> infra::values::Document {
    infra::values::Document {
        subject: infra::Subject { scope: infra::Scope::Project(1), kind: infra::Kind::Environment(environment_name()) },
        value: infra::values::Value::Environment {
            environment: environment_name(),
            end_revision: 5,
            fact: infra::values::EnvironmentFact {
                created_by: infra::values::Key {
                    deployment: [1; 16],
                    task: 10,
                    origin: infra::values::Purpose::Procedure { purpose: 2 },
                    purpose: 2,
                },
                ready_at: 30,
                until: 123,
                price: 40,
            },
            end: Some(infra::values::EnvironmentEnd { revision: 5, until: 600 }),
            pool: Some(infra::values::ClientPool { quota: 2, used: 1 }),
        },
    }
}
/// Starts the incident story; every identity is explicitly supplied by its fixture.
#[must_use]
#[expect(
    clippy::too_many_lines,
    reason = "one complete finite incident fixture keeps its supplied identities together"
)]
pub fn incident_world(seed: u64) -> ClientWorld {
    let mut truth = Truth::default();
    for party in [7, 8] {
        truth.frames.insert(party, frame(party));
        truth.sign_ins.insert(party, party);
    }
    let mut p = proposal();
    p.action = ProposalAction::Effect {
        part: Part {
            connector: number(Connector::Infrastructure),
            kind: PartKind::Effect,
            key: PartKey::Number(p.key.number),
        },
        authority: task().authority,
        attempt: 1,
        completion: 1,
        position: 1,
        purpose: 1,
    };
    p.reason = DocumentText::complete(text(b"Restart checkout in production"));
    let mut lead = task();
    lead.chip.title = DocumentText::complete(text(b"I incident lead"));
    {
        let ChipPhaseFacts { phase, .. } = lead.chip.details.as_mut();
        *phase = ChipPhase::Active(RunStage::Idle);
    }
    let mut restart = task();
    restart.chip.task = 10;
    restart.chip.title = DocumentText::complete(text(b"R restart checkout"));
    restart.requester = Requester::Task(9);
    restart.proposals = list([p.key]);
    restart.chip.executor = ChipExecutor::Procedure { connector: number(Connector::Infrastructure), kind: 1 };
    {
        let ChipPhaseFacts { details, .. } = restart.chip.details.as_mut();
        details.procedure = Some(Part {
            connector: number(Connector::Infrastructure),
            kind: PartKind::Procedure,
            key: PartKey::Number(10),
        });
    }
    lead.delegates = list([LiveDelegate { chip: restart.chip.clone(), takeover_budget: 1 }]);
    lead.context.as_mut().expect("supplied task context").impact.tasks.running = 1;
    for (key, value) in [
        (ObjectKey::Task(9), ReadValue::Task(Box::new(lead.clone()))),
        (ObjectKey::Task(10), ReadValue::Task(Box::new(restart.clone()))),
        (ObjectKey::Proposal(p.key), ReadValue::Proposal(Box::new(p.clone()))),
    ] {
        truth.object_projects.insert(key, 1);
        truth.objects.insert(key, value);
    }
    truth.trees.insert(
        9,
        Tree {
            root: lead.chip.clone(),
            rows: list([TreeRow {
                chip: restart.chip.clone(),
                parent: 9,
                depth: 1,
                dependencies: list([]),
                proposals: list([p.key]),
                spent: 0,
                cut: false,
            }]),
            folded: list([]),
            held_below: 0,
            next: None,
        },
    );
    truth.sources.push(jig_client_scripted_engine::Source {
        party: 7,
        read: Read::Activity { task: 9, before: None },
        rows: jig_client_scripted_engine::Rows::Activity(vec![turn_event(
            9,
            1,
            TurnName { attempt: 1, number: 1 },
            TurnRead::Decoded(Turn {
                messages: list([Message {
                    speaker: Speaker::Assistant,
                    blocks: list([Block::Call(Call {
                        number: 1,
                        tool: text(b"delegate"),
                        input: text(b"Restart checkout"),
                        outcome: Some(text(b"R restart checkout")),
                        failed: false,
                        objects: list([ObjectKey::Task(10)]),
                        parts: list([]),
                    })]),
                }]),
                used: Usage { input: 1, output: 1, cached: 0, cache_written: 0 },
                spent: 1,
                read_through: None,
                cut: false,
            }),
        )]),
    });
    truth.sources.push(jig_client_scripted_engine::Source {
        party: 7,
        read: Read::Inbox { group: InboxReadGroup::Needs, project: None, kind: None, goal: None, after: None },
        rows: jig_client_scripted_engine::Rows::Inbox(vec![inbox_entry(
            ObjectKey::Proposal(p.key),
            1,
            InboxGroup::Proposals,
            1,
            Wall::EPOCH,
            None,
        )]),
    });
    let mut ops = OpsScript::default();
    let incident = incident_document();
    let obs::values::Value::Incident { incident: row, .. } = &incident.value else { unreachable!() };
    let rows = list([row.clone()]);
    let incident_doc = obs::Document::Value(Box::new(incident));
    ops.documents.insert(Subject::Observability(incident_doc.subject()), Document::Observability(incident_doc));
    let subject = obs::Subject { scope: obs::Scope::Project(1), kind: obs::Kind::Incidents { open: true } };
    ops.documents.insert(
        Subject::Observability(subject),
        Document::Observability(obs::Document::Value(Box::new(obs::values::Document {
            subject,
            value: obs::values::Value::Project { project: 1, services: list([]), incidents: rows },
        }))),
    );
    let subject = infra::Subject { scope: infra::Scope::Project(1), kind: infra::Kind::Panel(10) };
    ops.documents.insert(
        Subject::Infrastructure(subject.clone()),
        Document::Infrastructure(infra::Document::Value(Box::new(infra::values::Document {
            subject,
            value: infra::values::Value::Panel {
                task: 10,
                state: infra::values::ProcedureState {
                    task: 10,
                    procedure: infra::values::Procedure::Remediate { service: service(), operation: 17, deadline: 900 },
                    phase: infra::values::ProcedurePhase::WaitingCondition,
                },
                resources: list([]),
                effects: list([]),
            },
        }))),
    );
    let subject = infra::Subject { scope: infra::Scope::Project(1), kind: infra::Kind::OwnerEffect(p.key.number) };
    ops.documents.insert(
        Subject::Infrastructure(subject.clone()),
        Document::Infrastructure(infra::Document::Value(Box::new(infra::values::Document {
            subject,
            value: infra::values::Value::Effect {
                proposal: p.key.number,
                task: 10,
                effect: infra::values::Effect::Restart { service: service(), operation: 17 },
                description: description(1, None, true, infra::values::Form::Transition),
            },
        }))),
    );
    let decided = decided_proposal(&p, ProposalHistoryState::Accepted { by: Requester::Person(7) });
    let rule = jig_client_scripted_engine::Rule {
        party: Some(7),
        ask: Ask::Decide {
            project: 1,
            on: DecideOn::Proposal { proposal: p.key, decision: Decision::Accepted, running: None },
        },
        answer: Answer::Done(Outcome::Document(DocumentOutcome::ProposalDecided {
            proposal: p.key,
            by: 7,
            choice: ProposalChoice::Accepted,
            at: Wall::EPOCH,
            repeated: false,
        })),
        changes: vec![
            jig_client_scripted_engine::Change::Object {
                project: 1,
                key: ObjectKey::Proposal(p.key),
                value: ReadValue::Ended(Box::new(decided)),
            },
            jig_client_scripted_engine::Change::Source(jig_client_scripted_engine::Source {
                party: 7,
                read: Read::Inbox { group: InboxReadGroup::Needs, project: None, kind: None, goal: None, after: None },
                rows: jig_client_scripted_engine::Rows::Inbox(vec![]),
            }),
        ],
    };
    configured(seed, truth, ops, vec![rule])
}
fn configured(seed: u64, truth: Truth, ops: OpsScript, rules: Vec<jig_client_scripted_engine::Rule>) -> ClientWorld {
    World::new(
        Settings { seed, ..Settings::default() },
        Scripted::new(
            Script {
                versions: VersionRange { first: 19, last: 19 },
                truth,
                bounds: Bounds {
                    text: 32,
                    rows: 8,
                    authority: 8,
                    segments: 4,
                    document_bytes: 65536,
                    commit_delay: Duration::from_millis(5),
                    heartbeat: Duration::from_millis(20),
                    watches: 64,
                    watches_per_party: 32,
                },
                rules,
            },
            seed,
            ops,
        ),
    )
}
fn service() -> infra::values::Service {
    infra::values::Service { environment: b"production".as_slice().into(), name: b"checkout".as_slice().into() }
}
fn description(
    kind: u16,
    price: Option<u64>,
    condition: bool,
    form: infra::values::Form,
) -> infra::values::Description {
    infra::values::Description {
        kind,
        resources: Box::new([]),
        condition,
        form,
        recovery: infra::values::Recovery::Keyed,
        price,
        state: 1,
    }
}
/// Starts Q's new chat with the complete price and owner request fixture.
#[must_use]
#[expect(
    clippy::too_many_lines,
    reason = "one complete finite chat fixture keeps its asks and resulting documents together"
)]
pub fn environment_world(seed: u64) -> ClientWorld {
    let mut truth = Truth::default();
    for party in [7, 8] {
        let mut row = frame(party);
        row.party.name = DocumentText::complete(text(if party == 7 { b"Q" } else { b"Other" }));
        truth.frames.insert(party, row);
        truth.sign_ins.insert(party, party);
    }
    let mut chat = task();
    chat.chip.title = DocumentText::complete(text(b"Q staging chat"));
    {
        let ChipPhaseFacts { phase, .. } = chat.chip.details.as_mut();
        *phase = ChipPhase::Active(RunStage::Idle);
    }
    let mut p = proposal();
    p.reason = DocumentText::complete(text(b"Provision staging for one week"));
    p.funding = list([FundingOffer {
        source: Funder::Pool { project: 1, person: 7, period: 3 },
        needed: 40,
        left: Some(40),
        after: Some(0),
    }]);
    p.action = ProposalAction::Effect {
        part: Part {
            connector: number(Connector::Infrastructure),
            kind: PartKind::Effect,
            key: PartKey::Number(p.key.number),
        },
        authority: task().authority,
        attempt: 1,
        completion: 1,
        position: 1,
        purpose: 1,
    };
    chat.proposals = list([p.key]);
    let ask = jig_client::shapes::chat(&truth.frames[&7], 1, text(b"staging for one week"))
        .expect("fixture offers the chat template");
    let accepted = decided_proposal(&p, ProposalHistoryState::Accepted { by: Requester::Person(7) });
    let rule = jig_client_scripted_engine::Rule {
        party: Some(7),
        ask,
        answer: Answer::Done(Outcome::Made { what: Made::Tasks { tasks: list([9]) } }),
        changes: vec![
            jig_client_scripted_engine::Change::Source(jig_client_scripted_engine::Source {
                party: 7,
                read: Read::Activity { task: 9, before: None },
                rows: jig_client_scripted_engine::Rows::Activity(vec![turn_event(
                    9,
                    1,
                    TurnName { attempt: 1, number: 1 },
                    TurnRead::Decoded(Turn {
                        messages: list([Message {
                            speaker: Speaker::Assistant,
                            blocks: list([Block::Call(Call {
                                number: 1,
                                tool: text(b"provision"),
                                input: text(b"staging for one week"),
                                outcome: Some(text(b"Price 40 exceeds chat budget 10")),
                                failed: false,
                                objects: list([ObjectKey::Proposal(p.key)]),
                                parts: list([]),
                            })]),
                        }]),
                        used: Usage { input: 1, output: 1, cached: 0, cache_written: 0 },
                        spent: 1,
                        read_through: None,
                        cut: false,
                    }),
                )]),
            }),
            jig_client_scripted_engine::Change::Object {
                project: 1,
                key: ObjectKey::Task(9),
                value: ReadValue::Task(Box::new(chat)),
            },
            jig_client_scripted_engine::Change::Object {
                project: 1,
                key: ObjectKey::Proposal(p.key),
                value: ReadValue::Proposal(Box::new(p.clone())),
            },
        ],
    };
    let accept = jig_client_scripted_engine::Rule {
        party: Some(7),
        ask: Ask::Decide {
            project: 1,
            on: DecideOn::Proposal { proposal: p.key, decision: Decision::Accepted, running: None },
        },
        answer: Answer::Done(Outcome::Document(DocumentOutcome::ProposalDecided {
            proposal: p.key,
            by: 7,
            choice: ProposalChoice::Accepted,
            at: Wall::EPOCH,
            repeated: false,
        })),
        changes: vec![jig_client_scripted_engine::Change::Object {
            project: 1,
            key: ObjectKey::Proposal(p.key),
            value: ReadValue::Ended(Box::new(accepted)),
        }],
    };
    let mut ops = OpsScript::default();
    let subject = infra::Subject { scope: infra::Scope::Project(1), kind: infra::Kind::OwnerEffect(p.key.number) };
    ops.documents.insert(
        Subject::Infrastructure(subject.clone()),
        Document::Infrastructure(infra::Document::Value(Box::new(infra::values::Document {
            subject,
            value: infra::values::Value::Effect {
                proposal: p.key.number,
                task: 10,
                effect: infra::values::Effect::CreateEnvironment {
                    environment: environment_name(),
                    until: 600,
                    price: 40,
                },
                description: description(3, Some(40), false, infra::values::Form::Creation),
            },
        }))),
    );
    configured(seed, truth, ops, vec![rule, accept])
}
/// The connector supplies completion and readiness in one application commit.
pub fn provision(world: &mut ClientWorld) {
    let now = world.now();
    let mut ops = world.front().engine.application().clone();
    let environment = environment_document();
    let infra::values::Value::Environment { environment: name, fact, end, pool, .. } = &environment.value else {
        unreachable!()
    };
    let rows =
        list([infra::values::ClientEnvironment { environment: name.clone(), fact: *fact, end: *end, pool: *pool }]);
    let document = infra::Document::Value(Box::new(environment));
    ops.documents.insert(Subject::Infrastructure(document.subject()), Document::Infrastructure(document));
    let subject = infra::Subject { scope: infra::Scope::Project(1), kind: infra::Kind::Environments { pool: None } };
    ops.documents.insert(
        Subject::Infrastructure(subject.clone()),
        Document::Infrastructure(infra::Document::Value(Box::new(infra::values::Document {
            subject,
            value: infra::values::Value::Project { project: 1, resources: list([]), environments: rows },
        }))),
    );
    world
        .front()
        .engine
        .act_application(now, Wall::EPOCH, number(Connector::Infrastructure), 1, ops, vec![])
        .expect("owner provision facts commit");
}
/// Owner-supplied recovery and the actual lead report remain one durable change.
pub fn resolve(world: &mut ClientWorld) {
    let now = world.now();
    let mut ops = world.front().engine.application().clone();
    let mut incident = incident_document();
    let obs::values::Value::Incident { incident: row, fact } = &mut incident.value else { unreachable!() };
    row.state = obs::documents::State::Resolved;
    *fact = Some(obs::values::Fact {
        observed: 36_240,
        healthy_replicas: 3,
        errors: 0,
        error_rate_percent: 0,
        load_percent: 0,
        load: Box::new([]),
    });
    let document = obs::Document::Value(Box::new(incident));
    ops.documents.insert(Subject::Observability(document.subject()), Document::Observability(document));
    let lead = world.front().engine.truth().objects[&ObjectKey::Task(9)].clone();
    let ReadValue::Task(lead) = lead else { unreachable!("supplied lead is task") };
    let lead = jig_client_world::stories::reported(*lead, b"Incident resolved");
    world
        .front()
        .engine
        .act_application(
            now,
            Wall::EPOCH,
            number(Connector::Observability),
            1,
            ops,
            vec![jig_client_scripted_engine::Change::Object {
                project: 1,
                key: ObjectKey::Task(9),
                value: ReadValue::Task(Box::new(lead)),
            }],
        )
        .expect("owner resolved report commits");
}

pub(super) fn list<T>(rows: impl IntoIterator<Item = T>) -> skein_lib::List<T> {
    skein_lib::List::from_boxed(rows.into_iter().collect::<Vec<_>>().into_boxed_slice())
}

use jig_client::contract::DecideOn;

fn inbox_entry(
    object: ObjectKey,
    project: u32,
    group: InboxGroup,
    position: u64,
    at: Wall,
    reply: Option<Box<TaskLine>>,
) -> InboxEntry {
    let (task, object) = match object {
        ObjectKey::Question { task, message } => (task, InboxObject::Question { message }),
        ObjectKey::Proposal(key) => (
            match key.proposer {
                Proposer::Task(n) | Proposer::Party(n) => n,
            },
            InboxObject::Proposal { proposer: key.proposer, number: key.number },
        ),
        ObjectKey::Escalation(task) => (task, InboxObject::Escalation { revision: 1 }),
        ObjectKey::PersonTask(task) => (task, InboxObject::PersonTask),
        ObjectKey::Task(task) => (task, InboxObject::Reply { message: position }),
        ObjectKey::Result(task) => (task, InboxObject::Result { position }),
        ObjectKey::Note { .. } => unreachable!("notes have no inbox row"),
    };
    let row = InboxEntry {
        reply,
        task: TaskLink::Stored { number: task, status: None },
        project: Some(project),
        object,
        activity: None,
        at: Some(at),
        unread: None,
        cursor: InboxCursor::Number { number: position },
    };
    assert_eq!(row.group(), group);
    row
}
