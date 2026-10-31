//! P and Q's supplied owner facts and public-tree observations.
#![allow(clippy::wildcard_imports, reason = "finite story fixtures enumerate the complete client vocabulary")]
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
    Address::Infrastructure(infra::Address::Environment { project: 1, environment: 7 })
}
pub(super) fn seen(tree: &ViewTree) -> Vec<Seen> {
    let mut seen = observe::jig_seen(tree);
    for (at, node) in tree.nodes().iter().enumerate() {
        let Some(key) = node.key else { continue };
        if key.owner != ((Owner::Application { number: number(Connector::Infrastructure) }).opaque())
            || !matches!(key.kind, 2 | 12)
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
        // The native form supplies the owner's end revision as the public
        // field's write version, alongside the whole client object identity.
        let revision = if key.kind == 12 {
            tree.nodes()[at..at + usize::try_from(node.size).expect("card subtree")]
                .iter()
                .find(|field| field.role == Role::TextField)
                .and_then(|field| field.value.as_ref())
                .map(|value| value.written)
        } else {
            None
        };
        seen.push(Seen::Card { key: CardKey::of(key), state, revision });
    }
    seen
}
#[must_use]
pub fn incident_document() -> obs::documents::Incident {
    let previous = obs::documents::Health {
        observed: Wall::from_nanos(36_000_000_000_000),
        healthy_replicas: 3,
        errors: 2,
        error_rate_percent: 2,
    };
    obs::documents::Incident {
        project: 1,
        number: 12,
        revision: Revision { value: 1 },
        watch: 888,
        raised: previous.observed,
        service: obs::documents::Service {
            environment: b"production".as_slice().into(),
            name: b"checkout".as_slice().into(),
        },
        severity: 2,
        state: obs::documents::State::Alerting,
        timeline: list([obs::documents::Point { at: previous.observed, health: Some(previous) }]),
        cut: false,
        health: Some(obs::documents::Health {
            observed: Wall::from_nanos(36_120_000_000_000),
            healthy_replicas: 2,
            errors: 30,
            error_rate_percent: 30,
        }),
        lead: Some(9),
        proposals: list([]),
    }
}
#[must_use]
pub fn environment_document() -> infra::documents::Environment {
    infra::documents::Environment {
        project: 1,
        number: 7,
        pool: 3,
        name: b"staging-7".as_slice().into(),
        revision: Revision { value: 99 },
        created_by: infra::documents::Creation {
            deployment: [1; 16],
            task: 10,
            origin: infra::documents::Purpose::Procedure { purpose: 2 },
            purpose: 2,
        },
        ready_at: Some(Wall::from_nanos(30_000_000_000)),
        ready: Some(true),
        cost: Some(40),
        observed_until: Some(Wall::from_nanos(123_000_000_000)),
        end: Some(infra::documents::End { revision: Revision { value: 5 }, until: Wall::from_nanos(600_000_000_000) }),
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
    p.action = OwnerProposalAction::Effect {
        part: OwnerPart {
            connector: number(Connector::Infrastructure),
            kind: OwnerPartKind::Effect,
            key: OwnerPartKey::Number(p.key.number),
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
    if let ChipPhase::Owner { phase, .. } = lead.chip.details.as_mut() {
        *phase = OwnerChipPhase::Active(RunStage::Idle);
    }
    let mut restart = task();
    restart.chip.task = 10;
    restart.chip.title = DocumentText::complete(text(b"R restart checkout"));
    restart.requester = OwnerRequester::Task(9);
    restart.proposals = list([p.key]);
    restart.chip.executor = Executor::Procedure { connector: number(Connector::Infrastructure), kind: 1 };
    if let ChipPhase::Owner { details, .. } = restart.chip.details.as_mut() {
        details.procedure = Some(OwnerPart {
            connector: number(Connector::Infrastructure),
            kind: OwnerPartKind::Procedure,
            key: OwnerPartKey::Number(10),
        });
    }
    restart.context.as_mut().expect("supplied task context").parts =
        list([Part::Panel { connector: number(Connector::Infrastructure), task: 10 }]);
    lead.delegates = list([LiveDelegate { chip: restart.chip.clone(), takeover_budget: 1 }]);
    lead.context.as_mut().expect("supplied task context").impact.tasks.running = 1;
    for (key, value) in [
        (ObjectKey::Task(9), ReadValue::Task(Box::new(lead.clone()))),
        (ObjectKey::Task(10), ReadValue::Task(Box::new(restart.clone()))),
        (ObjectKey::Proposal(p.key), ReadValue::OwnerProposal(Box::new(p.clone()))),
    ] {
        truth.object_projects.insert(key, 1);
        truth.objects.insert(key, value);
    }
    truth.trees.insert(
        9,
        OwnerTree {
            root: lead.chip.clone(),
            rows: list([OwnerTreeRow {
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
        rows: jig_client_scripted_engine::Rows::Activity(vec![ActivityLine {
            position: 1,
            at: Wall::EPOCH,
            event: turn_event(
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
            ),
        }]),
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
    let incident_doc = obs::Document::Incident(incident.clone());
    ops.documents.insert(Subject::Observability(incident_doc.subject()), Document::Observability(incident_doc));
    let subject = obs::Subject { scope: obs::Scope::Project(1), kind: obs::Kind::Incidents { open: true } };
    ops.documents.insert(
        Subject::Observability(subject),
        Document::Observability(obs::Document::Incidents {
            subject,
            revision: Revision { value: 1 },
            incidents: list([incident]),
        }),
    );
    let panel_subject = infra::Subject { scope: infra::Scope::Project(1), kind: infra::Kind::Panel(10) };
    let mut panel = part(panel_subject.clone());
    panel.panel = Some(infra::documents::Panel {
        task: 10,
        procedure: infra::documents::Procedure::Remediate {
            service: infra::documents::Service {
                environment: b"production".as_slice().into(),
                name: b"checkout".as_slice().into(),
            },
            operation: 17,
            deadline: Wall::from_nanos(900_000_000_000),
        },
        phase: infra::documents::Phase::WaitingCondition,
    });
    ops.documents.insert(
        Subject::Infrastructure(panel_subject),
        Document::Infrastructure(infra::Document::Part(Box::new(panel))),
    );
    let effect_subject =
        infra::Subject { scope: infra::Scope::Project(1), kind: infra::Kind::OwnerEffect(p.key.number) };
    let mut effect = part(effect_subject.clone());
    effect.description = Some(infra::documents::Description {
        words: b"Restart checkout".as_slice().into(),
        resource: b"checkout in production".as_slice().into(),
        kind: 1,
        recovery: infra::documents::Recovery::Keyed,
        price: None,
        condition: true,
        form: infra::documents::Form::Transition,
        state: 1,
    });
    effect.effect = Some(infra::documents::EffectAction::Restart {
        service: infra::documents::Service {
            environment: b"production".as_slice().into(),
            name: b"checkout".as_slice().into(),
        },
        operation: 17,
    });
    ops.documents.insert(
        Subject::Infrastructure(effect_subject),
        Document::Infrastructure(infra::Document::Part(Box::new(effect))),
    );
    let decided = decided_proposal(&p, OwnerProposalHistoryState::Accepted { by: OwnerRequester::Person(7) });
    let rule = jig_client_scripted_engine::Rule {
        party: Some(7),
        ask: Ask::Decide {
            project: 1,
            on: DecideOn::Proposal { proposal: p.key, decision: Decision::Accepted, running: None },
        },
        answer: Answer::Done(Outcome::Owner(OwnerOutcome::ProposalDecided {
            proposal: p.key,
            by: 7,
            choice: OwnerProposalChoice::Accepted,
            at: Wall::EPOCH,
            repeated: false,
        })),
        changes: vec![
            jig_client_scripted_engine::Change::Object {
                project: 1,
                key: ObjectKey::Proposal(p.key),
                value: ReadValue::Historical(Box::new(decided)),
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
fn part(subject: infra::Subject) -> infra::documents::PartDocument {
    infra::documents::PartDocument {
        subject,
        revision: Revision { value: 1 },
        panel: None,
        description: None,
        effect: None,
        resources: list([]),
        effects: list([]),
        drift: list([]),
        environments: list([]),
        pools: list([]),
        backend: None,
        made: None,
        proposal_limit: None,
        staged_limit: None,
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
    if let ChipPhase::Owner { phase, .. } = chat.chip.details.as_mut() {
        *phase = OwnerChipPhase::Active(RunStage::Idle);
    }
    let mut p = proposal();
    p.reason = DocumentText::complete(text(b"Provision staging for one week"));
    p.funding = list([OwnerFundingOffer {
        source: Funder::Pool { project: 1, person: 7, period: 3 },
        needed: 40,
        left: Some(40),
        after: Some(0),
    }]);
    p.action = OwnerProposalAction::Effect {
        part: OwnerPart {
            connector: number(Connector::Infrastructure),
            kind: OwnerPartKind::Effect,
            key: OwnerPartKey::Number(p.key.number),
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
    let accepted = decided_proposal(&p, OwnerProposalHistoryState::Accepted { by: OwnerRequester::Person(7) });
    let rule = jig_client_scripted_engine::Rule {
        party: Some(7),
        ask,
        answer: Answer::Done(Outcome::Made { what: Made::Tasks { tasks: list([9]) } }),
        changes: vec![
            jig_client_scripted_engine::Change::Source(jig_client_scripted_engine::Source {
                party: 7,
                read: Read::Activity { task: 9, before: None },
                rows: jig_client_scripted_engine::Rows::Activity(vec![ActivityLine {
                    position: 1,
                    at: Wall::EPOCH,
                    event: turn_event(
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
                    ),
                }]),
            }),
            jig_client_scripted_engine::Change::Object {
                project: 1,
                key: ObjectKey::Task(9),
                value: ReadValue::Task(Box::new(chat)),
            },
            jig_client_scripted_engine::Change::Object {
                project: 1,
                key: ObjectKey::Proposal(p.key),
                value: ReadValue::OwnerProposal(Box::new(p.clone())),
            },
        ],
    };
    let accept = jig_client_scripted_engine::Rule {
        party: Some(7),
        ask: Ask::Decide {
            project: 1,
            on: DecideOn::Proposal { proposal: p.key, decision: Decision::Accepted, running: None },
        },
        answer: Answer::Done(Outcome::Owner(OwnerOutcome::ProposalDecided {
            proposal: p.key,
            by: 7,
            choice: OwnerProposalChoice::Accepted,
            at: Wall::EPOCH,
            repeated: false,
        })),
        changes: vec![jig_client_scripted_engine::Change::Object {
            project: 1,
            key: ObjectKey::Proposal(p.key),
            value: ReadValue::Historical(Box::new(accepted)),
        }],
    };
    let mut ops = OpsScript::default();
    let subject = infra::Subject { scope: infra::Scope::Project(1), kind: infra::Kind::OwnerEffect(p.key.number) };
    let mut effect = part(subject.clone());
    effect.description = Some(infra::documents::Description {
        words: b"Provision staging".as_slice().into(),
        resource: b"Q pool".as_slice().into(),
        kind: 3,
        recovery: infra::documents::Recovery::Keyed,
        price: Some(40),
        condition: false,
        form: infra::documents::Form::Creation,
        state: 1,
    });
    effect.effect = Some(infra::documents::EffectAction::CreateEnvironment {
        environment: 7,
        until: Wall::from_nanos(600_000_000_000),
        price: 40,
    });
    ops.documents.insert(
        Subject::Infrastructure(subject.clone()),
        Document::Infrastructure(infra::Document::Part(Box::new(effect))),
    );
    configured(seed, truth, ops, vec![rule, accept])
}
/// The connector supplies completion and readiness in one application commit.
pub fn provision(world: &mut ClientWorld) {
    let now = world.now();
    let mut ops = world.front().engine.application().clone();
    let environment = environment_document();
    let document = infra::Document::Environment(environment.clone());
    ops.documents.insert(Subject::Infrastructure(document.subject()), Document::Infrastructure(document));
    let subject = infra::Subject { scope: infra::Scope::Project(1), kind: infra::Kind::Environments { pool: None } };
    ops.documents.insert(
        Subject::Infrastructure(subject.clone()),
        Document::Infrastructure(infra::Document::Environments {
            subject,
            revision: Revision { value: 1 },
            pools: list([infra::documents::Pool {
                number: 3,
                name: b"Q pool".as_slice().into(),
                quota: Some(2),
                used: Some(1),
            }]),
            environments: list([environment]),
        }),
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
    incident.revision.value += 1;
    incident.state = obs::documents::State::Resolved;
    incident.health = Some(obs::documents::Health {
        observed: Wall::from_nanos(36_240_000_000_000),
        healthy_replicas: 3,
        errors: 0,
        error_rate_percent: 0,
    });
    let document = obs::Document::Incident(incident);
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
    let rows: Vec<_> = rows.into_iter().collect();
    let mut list = skein_lib::List::with_capacity(u32::try_from(rows.len()).expect("finite fixture row count"));
    for row in rows {
        list.push(row).ok().expect("fixture list capacity");
    }
    list
}

use jig_client::contract::DecideOn;

fn inbox_entry(
    object: ObjectKey,
    project: u32,
    group: InboxGroup,
    position: u64,
    at: Wall,
    reply: Option<Box<TaskLine>>,
) -> OwnerInboxEntry {
    let (task, object) = match object {
        ObjectKey::Question { task, message } => (task, OwnerInboxObject::Question { message }),
        ObjectKey::Proposal(key) => (
            match key.proposer {
                Proposer::Task(n) | Proposer::Party(n) => n,
            },
            OwnerInboxObject::Proposal { proposer: key.proposer, number: key.number },
        ),
        ObjectKey::Escalation(task) => (task, OwnerInboxObject::Escalation { revision: 1 }),
        ObjectKey::PersonTask(task) => (task, OwnerInboxObject::PersonTask),
        ObjectKey::Task(task) => (task, OwnerInboxObject::Reply { message: position }),
        ObjectKey::Result(task) => (task, OwnerInboxObject::Result { position }),
        ObjectKey::Note { .. } => unreachable!("notes have no inbox row"),
    };
    let row = OwnerInboxEntry {
        reply,
        task: OwnerTaskLink::Stored { number: task, status: None },
        project: Some(project),
        object,
        activity: None,
        at: Some(at),
        unread: None,
        cursor: OwnerInboxCursor::Number { number: position },
    };
    assert_eq!(row.group(), group);
    row
}
