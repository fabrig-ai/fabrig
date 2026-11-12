//! Team's hand-made deliveries (temper/docs/design/client/web/gallery.md, 2).
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

pub(super) fn world(name: &str) -> Domain {
    let alone = name == "team-alone";
    let me = super::base_frame("Reader", "shop");
    let reader = me.party.number;
    let (mut root, mut out) = started(Address::Team { project: PROJECT });
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
            Down::Open { watch, subject: Watchable::Family { record, .. } }
                if wire::decode_watching(&record.bytes, &w::CEILINGS)
                    .is_some_and(|watching| matches!(watching.watch(), w::Watch::Changes(_))) =>
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
                                    bytes: wire::changes_panel(&changes(reader, alone)),
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

fn changes(reader: u64, alone: bool) -> w::ChangesPanel {
    let mut rows = List::with_capacity(5);
    if !alone {
        let at = super::now(crate::samples::World::Solo).as_nanos().saturating_sub(86_400_000_000_000);
        let both = [reader, SAM];
        let sam = [SAM];
        for task in 1..=5 {
            rows.push(
                w::ChangesLanded::new(
                    &w::CEILINGS,
                    w::ChangesLandedParts {
                        task,
                        title: b"A signed change".as_slice().into(),
                        at,
                        commit: [0xab; 32],
                        default_branch: true,
                        signed: crate::samples::signoffs(if task <= 4 { both.as_slice() } else { sam.as_slice() }, at),
                    },
                )
                .expect("signed sample landing"),
            )
            .expect("five sample landings");
        }
    }
    w::ChangesPanel::new(
        &w::CEILINGS,
        w::ChangesPanelParts {
            project: PROJECT,
            landing: b"main".as_slice().into(),
            queues: List::with_capacity(0),
            more: 0,
            landed: rows.clone(),
            landed_more: 0,
            landed_default: rows,
            landed_default_more: 0,
        },
    )
    .expect("sample changes")
}

fn member(party: u64, kind: c::PartyKind, name: &str, role: c::HeldRole) -> c::ProjectMember {
    c::ProjectMember { party, kind, name: doc(name), login: doc(name), role }
}

pub(super) fn project(reader: u64, alone: bool) -> c::Project {
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
                    signed: List::with_capacity(0),
                    can_sign: List::with_capacity(0),
                    can_sign_more: 0,
                    size: None,
                },
            )
            .expect("review"),
        )
        .expect("one review");
    }
    w::Approvals::new(&w::CEILINGS, w::ApprovalsParts { rows, more: 0 }).expect("approvals")
}
