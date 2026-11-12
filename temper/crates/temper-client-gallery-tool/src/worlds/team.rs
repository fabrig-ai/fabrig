//! Team's hand-made deliveries (temper/docs/design/client/web/gallery.md, 2).
mod sessions;
use super::{Address, Domain, List, c, doc, started};
use crate::driver::{answer, deliver, down, step};
use jig_client::{Down, Readable, Up, Watchable};
use temper_client_domain::Event;
use temper_client_protocol_wire::{self as wire, v1 as w};
use temper_fixtures::client_documents as documents;

const PROJECT: u32 = 1;
const OTHER_PROJECT: u32 = 2;
const SAM: u64 = 74;
const SERVICE: u64 = 75;
const FIRST_QUESTION: u64 = 1;
const SECOND_QUESTION: u64 = 2;
const ELSEWHERE_QUESTION: u64 = 3;
const SAM_CHAT: u64 = 4;
const REVIEW: u64 = 5;
const FORGE: u16 = 1;
// The reader, Sam and the service fit the project's document.
const MEMBERS: u32 = 3;
// Two questions here, one elsewhere, in the reader's standing inbox.
const QUESTIONS: u32 = 3;

#[expect(clippy::too_many_lines, reason = "one delivery loop serves Team's people and role-card reads")]
pub(super) fn world(name: &str) -> Domain {
    let role = name == "team-role";
    let alone = name == "team-alone" || role;
    let me = super::base_frame("Reader", "shop");
    let reader = me.party.number;
    let (mut root, mut out) = started(Address::Team { project: PROJECT, role: if role { Some(52) } else { None } });
    while let Some(request) = out.pop() {
        let Some(request) = down(request) else { continue };
        match request {
            Down::Open { watch, subject: Watchable::Jig(c::Subject::Me) } => {
                let mut me = me.clone();
                me.needs_total = if alone { 0 } else { QUESTIONS };
                me.needs.questions = me.needs_total;
                deliver(&mut root, watch, c::Delivery::Me(Box::new(me)), &mut out);
            }
            Down::Read { read, what: Readable::Jig(c::Read::Project { project: PROJECT }) } => {
                answer(&mut root, read, c::ReadValue::Project(Box::new(project(reader, alone))), &mut out);
            }
            Down::Open { watch, subject: Watchable::Jig(c::Subject::Inbox { .. }) } => {
                let mut inbox = documents::inbox(
                    entries(alone),
                    if alone { 0 } else { QUESTIONS },
                    c::Count { value: 0, limited: false },
                    0,
                    false,
                );
                inbox.party = reader;
                deliver(&mut root, watch, c::Delivery::Inbox(Box::new(inbox)), &mut out);
            }
            Down::Open { watch, subject: Watchable::Jig(c::Subject::Tasks { query, .. }) }
                if query.project == Some(PROJECT) && query.requester == Some(c::Requester::Person(SAM)) =>
            {
                deliver(&mut root, watch, c::Delivery::Tasks(Box::new(sams_tasks())), &mut out);
            }
            Down::Open { watch, subject: Watchable::Jig(c::Subject::Tasks { query, .. }) }
                if query.label == Some(3) =>
            {
                let rows = if role {
                    List::from_boxed(Box::new([super::task_line(prover(reader))]))
                } else {
                    List::with_capacity(0)
                };
                deliver(&mut root, watch, c::Delivery::Tasks(Box::new(c::Tasks { rows, more: false })), &mut out);
            }
            Down::Open { watch, subject: Watchable::Jig(c::Subject::Task { task: 52 }) } if role => {
                deliver(&mut root, watch, c::Delivery::Task(Box::new(prover(reader))), &mut out);
            }
            Down::Read {
                read,
                what: Readable::Jig(c::Read::Notes { scope: c::NoteScope::Task { task: 52, .. }, .. }),
            } if role => {
                answer(&mut root, read, c::ReadValue::Notes(Box::new(provers_now())), &mut out);
            }
            Down::Read { read, what: Readable::Jig(c::Read::Activity { task: 52, .. }) } if role => {
                answer(&mut root, read, c::ReadValue::Activity(Box::new(sessions::page())), &mut out);
            }
            Down::Read { read, what: Readable::Family { record, .. } } if role => {
                if let Some(reading) = wire::decode_reading(&record.bytes, &w::CEILINGS)
                    && let w::Read::View(view) = reading.read()
                    && view.address() == b"charter 13"
                {
                    let answer = w::ViewAnswer::new(&w::CEILINGS, w::ViewAnswerParts { view: charter() })
                        .expect("charter answer");
                    step(
                        &mut root,
                        Event::Client {
                            event: jig_client::Event::Up {
                                up: Up::ReadDone {
                                    read,
                                    result: jig_client::Readout::Family {
                                        family: wire::FORGE,
                                        record: Some(wire::view_answer(&answer)),
                                    },
                                },
                            },
                        },
                        &mut out,
                    );
                }
            }
            Down::Open { watch, subject: Watchable::Family { record, .. } }
                if wire::decode_watching(&record.bytes, &w::CEILINGS)
                    .is_some_and(|watching| *watching.watch() == w::Watch::Approvals) =>
            {
                step(
                    &mut root,
                    Event::Client {
                        event: jig_client::Event::Up {
                            up: Up::FamilyDelivered {
                                watch,
                                record: jig_client::Family {
                                    family: wire::FORGE,
                                    version: wire::CLIENT_VERSION,
                                    bytes: wire::approvals(&approvals(alone)),
                                },
                            },
                        },
                    },
                    &mut out,
                );
            }
            Down::Read { .. }
            | Down::Open { .. }
            | Down::Continue { .. }
            | Down::Close { .. }
            | Down::Save { .. }
            | Down::Erase { .. }
            | Down::Send { .. } => {}
        }
    }
    root
}

fn member(party: u64, kind: c::PartyKind, name: &str, role: c::HeldRole) -> c::ProjectMember {
    c::ProjectMember { party, kind, name: doc(name), login: doc(name), role }
}

fn project(reader: u64, alone: bool) -> c::Project {
    let mut members = List::with_capacity(MEMBERS);
    members.push(member(reader, c::PartyKind::Person, "Reader", c::HeldRole::Owner)).expect("reader");
    if !alone {
        members.push(member(SAM, c::PartyKind::Person, "Sam", c::HeldRole::Owner)).expect("Sam");
        members.push(member(SERVICE, c::PartyKind::Service, "Service", c::HeldRole::Member)).expect("service");
    }
    c::Project {
        number: PROJECT,
        name: doc("shop"),
        members,
        acts: List::with_capacity(0),
        resources: List::with_capacity(0),
        policy: c::ProjectPolicy {
            templates: List::with_capacity(0),
            ceiling: c::AuthorityDocument::complete(c::Authority {
                models: None,
                tools: 0,
                grants: List::with_capacity(0),
                delegation: c::Delegation { kinds: List::with_capacity(0), tasks: 0, depth: 0 },
                spend: 0,
                deadline: None,
                notes: 0,
                note_resources: List::with_capacity(0),
            }),
            period_spend: 0,
            escalation_role: None,
            projections: List::with_capacity(0),
            roles: List::with_capacity(0),
            requirements: List::with_capacity(0),
            permissions: None,
            permissions_cut: false,
            cut: false,
        },
        current_period: None,
        pools: List::with_capacity(0),
        configured_period: 0,
        configured_pools: List::with_capacity(0),
        configured_pools_cut: false,
        spend: List::with_capacity(0),
        recurring: List::with_capacity(0),
        cut: false,
    }
}

fn entries(alone: bool) -> List<c::InboxEntry> {
    let mut rows = List::with_capacity(QUESTIONS);
    if !alone {
        for (task, project) in
            [(FIRST_QUESTION, PROJECT), (SECOND_QUESTION, PROJECT), (ELSEWHERE_QUESTION, OTHER_PROJECT)]
        {
            rows.push(documents::entry(
                c::ObjectKey::Question { task, message: 1 },
                project,
                task,
                super::now(crate::samples::World::Solo),
                None,
            ))
            .expect("three questions");
        }
    }
    rows
}

fn sams_tasks() -> c::Tasks {
    let mut task = super::base_task(SAM_CHAT, "Sam's question");
    task.project = PROJECT;
    task.requester = c::Requester::Person(SAM);
    task.waits_on =
        c::WaitsOn::Party { whom: c::Whom::Person(SAM), entries: 1, task: SAM_CHAT, kind: c::WaitedFor::Question };
    c::Tasks { rows: List::from_boxed(Box::new([super::task_line(task)])), more: false }
}

fn approvals(alone: bool) -> w::Approvals {
    let mut rows = List::with_capacity(1);
    if !alone {
        rows.push(
            w::ApprovalRow::new(
                &w::CEILINGS,
                w::ApprovalRowParts {
                    task: REVIEW,
                    project: PROJECT,
                    goal: None,
                    forge: FORGE,
                    kind: w::Kind::Forgejo,
                    owner: b"work".as_slice().into(),
                    name: b"shop".as_slice().into(),
                    pull: 1,
                    address: b"https://forge.example/work/shop/pulls/1/files".as_slice().into(),
                    title: b"A change".as_slice().into(),
                    since: super::now(crate::samples::World::Solo).as_nanos(),
                    scope: 1,
                    rule: 1,
                    role: 0,
                    required: 1,
                    counted: 0,
                    sets: List::with_capacity(0),
                    whole: true,
                    yours: w::ApprovalYours::Counts,
                    size: None,
                },
            )
            .expect("review"),
        )
        .expect("one review");
    }
    w::Approvals::new(&w::CEILINGS, w::ApprovalsParts { rows, more: 0 }).expect("approvals")
}

const NOW: &[u8] = b"- Finding 91, from the fuzz tier at 01:50: a commit is lost when a witness rejoins during a view change. Reproduced with seed 4417 in a deterministic test, tests/sim/rejoin.c.\n- The fix is the protocol core's: count a rejoining witness only once it holds the log suffix. Asked you; I start the change at your answer.\n- Triaged three findings this week: two to you, one kept, a flaky replica timeout, repaired and landed.\n- Next: the fuzz tier's next run, then my weekly report on Monday.";
fn prover(reader: u64) -> c::Task {
    let mut task = super::base_task(52, "Prover");
    task.project = PROJECT;
    task.label = 3;
    task.flags.ended_by = c::EndedBy::Standing;
    task.flags.conversation = false;
    task.chip.executor = c::ChipExecutor::Agent { charter: 13 };
    task.requester = c::Requester::Person(reader);
    task.made = skein_lib::Wall::from_nanos(1_793_577_600_000_000_000);
    task.budget.numbers.spent = 21;
    task.budget.numbers.budget = 40;
    task.wake.timers = true;
    task.wake.news = c::WakeRule::Paced { span: skein_lib::Duration::from_secs(1800) };
    task.wake.notices = c::WakeRule::Immediate;
    task.subscriptions.entries = List::from_boxed(Box::new([
        c::Subscription { number: 1, kind: c::SubscriptionKind::Topic { connector: wire::FORGE, topic: 1 } },
        c::Subscription {
            number: 2,
            kind: c::SubscriptionKind::Timer {
                at: skein_lib::Wall::from_nanos(1_794_204_000_000_000_000),
                period: Some(skein_lib::Duration::from_secs(604_800)),
            },
        },
        c::Subscription { number: 3, kind: c::SubscriptionKind::Topic { connector: wire::FORGE, topic: 2 } },
        c::Subscription { number: 4, kind: c::SubscriptionKind::Task { target: 61, held: false, result: true } },
    ]));
    task.acts = List::from_boxed(Box::new([c::Offered {
        act: c::ActKind::Stop,
        admission: c::Admission::Allowed,
        effect: None,
    }]));
    task
}
fn provers_now() -> c::NotePage {
    c::NotePage {
        rows: List::from_boxed(Box::new([c::Note {
            number: 1,
            revision: c::Revision { value: 1 },
            scope: c::NoteScope::Task { project: PROJECT, task: 52 },
            description: doc("now"),
            body: c::DocumentText::complete(c::Text::new(NOW.into(), 1024).expect("Now fits")),
            writer: c::NoteAuthor::Task { task: 52, attempt: 1 },
            tasks: List::with_capacity(0),
            changed: Some(skein_lib::Wall::from_nanos(1_794_105_660_000_000_000)),
            acts: List::with_capacity(0),
            cut: false,
        }])),
        cut: false,
        before: None,
        after: None,
    }
}

fn charter() -> w::View {
    let fact = w::Fact::new(
        &w::CEILINGS,
        w::FactParts {
            label: b"Asked to".as_slice().into(),
            value: b"Reproduce before routing: a finding gets a seed or a test first.".as_slice().into(),
        },
    )
    .expect("Asked to");
    let card = w::Card::new(
        &w::CEILINGS,
        w::CardParts {
            who: b"an agent".as_slice().into(),
            name: b"Prover".as_slice().into(),
            facts: List::from_boxed(Box::new([fact])),
        },
    )
    .expect("charter card");
    w::View::Charter(w::CharterView::new(&w::CEILINGS, w::CharterViewParts { charter: 13, card }).expect("charter"))
}
