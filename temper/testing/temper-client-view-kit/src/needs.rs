use super as f;
use crate::look;
use jig_client::contract as c;
use skein_lib::{Env, List, Queue, Time, Wall};
use temper_client_domain::{self as d, Address, Event};
use temper_fixtures::client_documents as documents;

pub const QUESTION: &str = "Fix the dropped watch in jig’s store, or keep it alive in temper serve?\n\nMakeProject answers after the client’s watch drops.\n\nSaw: The box test reproduces it every time.\nChoice: In jig’s store (recommended)\nChoice: In temper serve";

#[must_use]
pub fn limits() -> d::Limits {
    let mut limits = f::domain_limits();
    limits.client.domain.reads = 8;
    limits.client.domain.text = 512;
    limits.client.domain.turn_text = 512;
    limits.client.domain.long_text = 4096;
    limits
}

#[must_use]
pub fn object(message: u64) -> c::ObjectKey {
    c::ObjectKey::Question { task: f::INBOX_TASK, message }
}

pub fn event(root: &mut d::Domain, event: Event, out: &mut Queue<d::Request>) {
    d::step(root, &Env { now: Time::ZERO, wall: f::NOW, limits: limits() }, event, out);
}

#[must_use]
pub fn fixture(words: &str, cut: bool, count: u32, more: bool, filter: d::address::Filter) -> d::Domain {
    fixture_title(words, cut, count, more, filter, None)
}

#[must_use]
pub fn fixture_title(
    words: &str,
    cut: bool,
    count: u32,
    more: bool,
    filter: d::address::Filter,
    title: Option<&[u8]>,
) -> d::Domain {
    let (mut root, _, decided) = fixture_watched(words, cut, count, more, filter, title);
    deliver_asked(&mut root, decided, None);
    root
}

#[must_use]
pub fn fixture_watched(
    words: &str,
    cut: bool,
    count: u32,
    more: bool,
    filter: d::address::Filter,
    title: Option<&[u8]>,
) -> (d::Domain, skein_lib::Token, skein_lib::Token) {
    fixture_classes(words, cut, count, more, filter, title, None)
}

#[must_use]
pub fn fixture_classes(
    words: &str,
    cut: bool,
    count: u32,
    more: bool,
    filter: d::address::Filter,
    title: Option<&[u8]>,
    classes: Option<&temper_protocol_reaches::Classes>,
) -> (d::Domain, skein_lib::Token, skein_lib::Token) {
    fixture_classes_role(words, cut, count, more, filter, title, classes, false, 2)
}

#[expect(clippy::too_many_arguments, reason = "the card fixture carries independent document and role rows")]
#[expect(clippy::too_many_lines, reason = "the fixture delivers independent question and class documents")]
#[must_use]
pub fn fixture_classes_role(
    words: &str,
    cut: bool,
    count: u32,
    more: bool,
    filter: d::address::Filter,
    title: Option<&[u8]>,
    classes: Option<&temper_protocol_reaches::Classes>,
    owner: bool,
    projects: u32,
) -> (d::Domain, skein_lib::Token, skein_lib::Token) {
    let (mut root, me_watch, _, _, _, _, _, standing, _) = f::root_at_with_classes(
        Address::Settings { project: f::PROJECT },
        &limits(),
        b"Shop",
        None,
        None,
        None,
        Some(&[]),
        Some(d::forge::ways::Ways::STOCK),
        classes.is_none(),
        true,
        classes,
    );
    if owner || projects == 1 {
        let mut me = root.pages().me(root.client()).unwrap().clone();
        me.projects.truncate(projects);
        if owner {
            me.projects.get_mut(0).unwrap().role = c::HeldRole::Owner;
        }
        event(
            &mut root,
            Event::Client {
                event: jig_client::Event::Up {
                    up: jig_client::Up::Watched {
                        watch: me_watch,
                        event: c::WatchEvent::Delivered(c::Delivery::Me(Box::new(me))),
                    },
                },
            },
            &mut Queue::uncapped(),
        );
    }
    let standing = standing.unwrap();
    let watch = standing;
    let mut opening = Queue::uncapped();
    event(&mut root, Event::Went { place: d::Place::Address(Address::Inbox { filter }) }, &mut opening);
    let mut decided = None;
    let subject = d::forge::decided::watch(f::PROJECT, &limits().forge).unwrap();
    for request in &opening {
        if let d::Request::Unlinked {
            request: jig_client::UnlinkedRequest::Down { down: jig_client::Down::Open { watch, subject: found } },
        } = request
            && found == &subject
        {
            decided = Some(*watch);
        }
    }
    for request in &opening {
        if let d::Request::Unlinked {
            request: jig_client::UnlinkedRequest::Down { down: jig_client::Down::Open { watch, subject } },
        } = request
            && Some(subject) == d::forge::decided::watch(f::SECOND_PROJECT, &limits().forge).as_ref()
        {
            deliver_asked_for(&mut root, *watch, f::SECOND_PROJECT, None);
        }
    }
    let mut entries = List::with_capacity(count);
    for message in 1..=u64::from(count) {
        let project =
            if projects > 1 && filter.project.is_none() && message == 2 { f::SECOND_PROJECT } else { f::PROJECT };
        entries.push(documents::entry(object(message), project, message, Wall::EPOCH, None)).unwrap();
    }
    let mut out = Queue::uncapped();
    event(
        &mut root,
        Event::Client {
            event: jig_client::Event::Up {
                up: jig_client::Up::Watched {
                    watch,
                    event: c::WatchEvent::Delivered(c::Delivery::Inbox(Box::new(documents::inbox(
                        entries.clone(),
                        count,
                        c::Count { value: 0, limited: false },
                        0,
                        more,
                    )))),
                },
            },
        },
        &mut out,
    );
    while let Some(request) = out.pop() {
        if let d::Request::Unlinked {
            request:
                jig_client::UnlinkedRequest::Down {
                    down: jig_client::Down::Read { read, what: jig_client::Readable::Jig(c::Read::Inbox { group, .. }) },
                },
        } = &request
        {
            let rows = if *group == c::InboxReadGroup::Needs { entries.clone() } else { List::with_capacity(0) };
            event(
                &mut root,
                Event::Client {
                    event: jig_client::Event::Up {
                        up: jig_client::Up::ReadDone {
                            read: *read,
                            result: jig_client::Readout::Jig(c::ReadResult::Value(c::ReadValue::Inbox(Box::new(
                                c::InboxPage { selection: None, before: None, rows, after: None, cut: more },
                            )))),
                        },
                    },
                },
                &mut out,
            );
        }
        if let d::Request::Unlinked {
            request:
                jig_client::UnlinkedRequest::Down {
                    down:
                        jig_client::Down::Read { read, what: jig_client::Readable::Jig(c::Read::Question { message, .. }) },
                },
        } = request
        {
            let mut task = f::task::document().chip;
            task.task = f::INBOX_TASK;
            task.title = f::task::document_words(match message {
                1 => b"welcome-stall",
                2 => b"forge-cards",
                _ => b"third",
            });
            if let Some(title) = title {
                task.title = f::task::document_words(title);
            }
            let q = c::Question {
                task,
                from: None,
                message,
                words: c::DocumentText { bytes: c::Text::new(words.as_bytes().into(), 4096).unwrap(), cut },
                since: Wall::EPOCH,
                acts: {
                    let mut offers = List::with_capacity(1);
                    offers
                        .push(c::Offered { act: c::ActKind::Answer, admission: c::Admission::Allowed, effect: None })
                        .unwrap();
                    offers
                },
                ending: None,
            };
            event(
                &mut root,
                Event::Client {
                    event: jig_client::Event::Up {
                        up: jig_client::Up::ReadDone {
                            read,
                            result: jig_client::Readout::Jig(c::ReadResult::Value(c::ReadValue::Question(Box::new(q)))),
                        },
                    },
                },
                &mut out,
            );
        }
    }
    assert!(root.pages().inbox().unwrap().delivered(root.client()));
    (root, watch, decided.unwrap())
}

pub fn deliver_asked(root: &mut d::Domain, watch: skein_lib::Token, reach: Option<temper_protocol_reaches::Reach>) {
    deliver_asked_for(root, watch, f::PROJECT, reach);
}

pub fn deliver_asked_for(
    root: &mut d::Domain,
    watch: skein_lib::Token,
    project: u32,
    reach: Option<temper_protocol_reaches::Reach>,
) {
    use temper_client_protocol_wire::{self as wire, v1 as w};
    let mut asked = List::with_capacity(1);
    if let Some(reach) = reach {
        asked
            .push(
                w::Asked::new(
                    &w::CEILINGS,
                    w::AskedParts {
                        number: 1,
                        task: f::INBOX_TASK,
                        at: 0,
                        reach: temper_protocol_reaches::reach_record(&reach),
                    },
                )
                .unwrap(),
            )
            .unwrap();
    }
    let document =
        w::Decisions::new(&w::CEILINGS, w::DecisionsParts { project, rows: List::with_capacity(0), asked, cut: false })
            .unwrap();
    event(
        root,
        Event::Client {
            event: jig_client::Event::Up {
                up: jig_client::Up::FamilyDelivered {
                    watch,
                    record: jig_client::Family {
                        family: wire::FORGE,
                        version: wire::CLIENT_VERSION,
                        bytes: wire::decisions(&document),
                    },
                },
            },
        },
        &mut Queue::uncapped(),
    );
}

pub fn decisions_delivered(
    root: &mut d::Domain,
    project: u32,
    watch: skein_lib::Token,
    count: u32,
    out: &mut Queue<d::Request>,
) {
    use temper_client_protocol_wire::{self as wire, v1 as w};
    let mut rows = List::with_capacity(count);
    for number in 1..=count {
        if (number % 2 == 1) != (project == f::PROJECT) {
            continue;
        }
        rows.push(
            w::Decision::new(
                &w::CEILINGS,
                w::DecisionParts {
                    number,
                    task: 90 + u64::from(number),
                    at: 3_900_000_000_000,
                    question: b"Which approach?".as_slice().into(),
                    took: b"Fixed the harness.".as_slice().into(),
                    why: b"Tests are its area.".as_slice().into(),
                    overturned: false,
                    reach: w::Reach::new(
                        &w::CEILINGS,
                        w::ReachParts {
                            kind: w::ReachKind::Tests,
                            class: w::ReachClass::Decide,
                            reason: w::ReachReason::Kind,
                            under: 0,
                            sets: 0,
                            matched: 0,
                            away: false,
                        },
                    )
                    .expect("decision reach"),
                },
            )
            .unwrap(),
        )
        .unwrap();
    }
    let bytes = wire::decisions(
        &w::Decisions::new(
            &w::CEILINGS,
            w::DecisionsParts { project, rows, asked: List::with_capacity(0), cut: false },
        )
        .unwrap(),
    );
    event(
        root,
        Event::Client {
            event: jig_client::Event::Up {
                up: jig_client::Up::FamilyDelivered {
                    watch,
                    record: jig_client::Family { family: wire::FORGE, version: wire::CLIENT_VERSION, bytes },
                },
            },
        },
        out,
    );
}

pub fn empty_decided_inbox(root: &mut d::Domain, watch: skein_lib::Token, out: &mut Queue<d::Request>) {
    event(
        root,
        Event::Client {
            event: jig_client::Event::Up {
                up: jig_client::Up::Watched {
                    watch,
                    event: c::WatchEvent::Delivered(c::Delivery::Inbox(Box::new(documents::inbox(
                        List::with_capacity(0),
                        0,
                        c::Count { value: 0, limited: false },
                        0,
                        false,
                    )))),
                },
            },
        },
        out,
    );
}

#[must_use]
pub fn decided_fixture(filter: d::address::Filter, count: u32, open: bool) -> d::Domain {
    let (mut root, _, initial) = f::root_at(Address::Inbox { filter }, &limits());
    let mut out = Queue::uncapped();
    event(&mut root, Event::Went { place: d::Place::Address(Address::Accounts) }, &mut out);
    event(&mut root, Event::Went { place: d::Place::Address(Address::Inbox { filter }) }, &mut out);
    if let Some(watch) = initial {
        empty_decided_inbox(&mut root, watch, &mut out);
    }
    while let Some(request) = out.pop() {
        let d::Request::Unlinked { request: jig_client::UnlinkedRequest::Down { down } } = request else { continue };
        match down {
            jig_client::Down::Open { watch, subject: jig_client::Watchable::Jig(c::Subject::Inbox { .. }) } => {
                empty_decided_inbox(&mut root, watch, &mut out);
            }
            jig_client::Down::Open { watch, subject } => {
                for project in [f::PROJECT, f::SECOND_PROJECT] {
                    if Some(subject.clone()) != d::forge::decided::watch(project, &limits().forge) {
                        continue;
                    }
                    decisions_delivered(&mut root, project, watch, count, &mut out);
                }
            }
            jig_client::Down::Read { read, what: jig_client::Readable::Jig(c::Read::Inbox { .. }) } => {
                event(
                    &mut root,
                    Event::Client {
                        event: jig_client::Event::Up {
                            up: jig_client::Up::ReadDone {
                                read,
                                result: jig_client::Readout::Jig(c::ReadResult::Value(c::ReadValue::Inbox(Box::new(
                                    c::InboxPage {
                                        selection: None,
                                        before: None,
                                        rows: List::with_capacity(0),
                                        after: None,
                                        cut: false,
                                    },
                                )))),
                            },
                        },
                    },
                    &mut out,
                );
            }
            jig_client::Down::Read { read, what: jig_client::Readable::Jig(c::Read::Task { task }) } => {
                let mut document = f::task::document();
                document.chip.task = task;
                document.chip.title = f::task::document_words(b"Decided task");
                event(
                    &mut root,
                    Event::Client {
                        event: jig_client::Event::Up {
                            up: jig_client::Up::ReadDone {
                                read,
                                result: jig_client::Readout::Jig(c::ReadResult::Value(c::ReadValue::Task(Box::new(
                                    document,
                                )))),
                            },
                        },
                    },
                    &mut out,
                );
            }
            jig_client::Down::Save { .. }
            | jig_client::Down::Erase { .. }
            | jig_client::Down::Send { .. }
            | jig_client::Down::Continue { .. }
            | jig_client::Down::Close { .. }
            | jig_client::Down::Read { .. } => {}
        }
    }
    assert!(root.pages().inbox().unwrap().known(), "decided fixture has an answered inbox");
    if open {
        approvals(&mut root, &[approval_row()], 0);
    }
    root
}

#[must_use]
pub fn words(value: &str) -> c::DocumentText {
    c::DocumentText::complete(c::Text::new(value.as_bytes().into(), 4096).unwrap())
}

#[must_use]
pub fn offers(kinds: &[c::ActKind]) -> List<c::Offered> {
    let mut acts = List::with_capacity(u32::try_from(kinds.len()).unwrap());
    for act in kinds {
        acts.push(c::Offered { act: *act, admission: c::Admission::Allowed, effect: None }).unwrap();
    }
    acts
}

#[must_use]
pub fn proposal(proposer: c::Proposer) -> c::Proposal {
    let source = f::chat::proposal_root();
    let mut proposal = source
        .pages()
        .cards()
        .unwrap()
        .iter()
        .find_map(|card| if let Some(c::ReadValue::Proposal(p)) = &card.page { Some(p.as_ref().clone()) } else { None })
        .unwrap();
    proposal.key.proposer = proposer;
    proposal.reason = words("Accept how temper will run your work? The agent follows this plan.");
    proposal.acts = offers(&[c::ActKind::Accept, c::ActKind::Reject]);
    let task = f::task::document();
    let mut tasks = List::with_capacity(1);
    tasks
        .push(c::ProposedTask {
            number: 1,
            executor: task.chip.details.details.executor,
            spec: task.spec,
            authority: task.authority,
            contract: c::TaskContractDocument::Verdict { choices: List::uncapped(), cut: false },
            deadline: c::NewDeadline::Named,
            budget: 6,
            funder: task.budget.funder,
            wake: task.wake,
            dependencies: List::with_capacity(0),
            resources: List::with_capacity(0),
            priority: None,
            cut: false,
        })
        .unwrap();
    proposal.action = c::ProposalAction::Batch(tasks);
    proposal.funding = List::with_capacity(1);
    proposal
        .funding
        .push(c::FundingOffer {
            source: c::Funder::Pool { project: f::PROJECT, person: 73, period: 1 },
            needed: 5,
            left: Some(20),
            after: Some(15),
        })
        .unwrap();
    proposal
}

#[must_use]
pub fn held(reason: Option<c::HoldCategory>) -> c::Escalation {
    let source = f::task::held_root();
    let mut task = source.pages().task().unwrap().document(source.client()).unwrap().clone();
    task.chip.title = words("first-run-projects");
    task.spec.specification.words = words("Both halves of the store task run.");
    task.budget.numbers.spent = 31;
    task.budget.numbers.budget = 40;
    match reason {
        Some(reason) => {
            let hold = task.hold.as_mut().unwrap();
            hold.reason = reason;
            hold.words = Some(words("The Ready page waits for this decision."));
        }
        None => task.hold = None,
    }
    c::Escalation { task: Box::new(task), acts: offers(&[c::ActKind::Release, c::ActKind::LeaveHeld]) }
}

#[must_use]
pub fn person() -> c::PersonTask {
    let mut task = f::task::document();
    task.chip.title = words("Review the plan");
    task.spec.specification.words = words("Read the proposed plan and record your review.");
    c::PersonTask {
        task: task.chip,
        revision: c::Revision { value: 3 },
        requester: task.requester,
        spec: task.spec,
        address: c::PersonAddress::Party(73),
        taken_by: None,
        since: None,
        stall: None,
        deadline: None,
        contract: task.result_contract,
        results: List::with_capacity(0),
        cut: false,
        acts: offers(&[c::ActKind::Take, c::ActKind::HandBack]),
    }
}

#[must_use]
pub fn kinds(values: &[(c::ObjectKey, c::Read, c::ReadValue)]) -> d::Domain {
    kinds_read(values, None)
}

#[must_use]
pub fn kinds_read(values: &[(c::ObjectKey, c::Read, c::ReadValue)], whole: Option<c::ReadResult>) -> d::Domain {
    kinds_owned(values, whole, None)
}

#[must_use]
pub fn settings_owned(page: c::ReadValue, owned: Option<bool>) -> d::Domain {
    let c::ReadValue::Proposal(proposal) = &page else { panic!("settings proposal") };
    let key = proposal.key;
    kinds_owned(&[(c::ObjectKey::Proposal(key), c::Read::Proposal { proposal: key }, page)], None, owned)
}

#[expect(
    clippy::too_many_lines,
    reason = "the fixture delivers the inbox, decision document and each card read together"
)]
pub fn kinds_owned(
    values: &[(c::ObjectKey, c::Read, c::ReadValue)],
    whole: Option<c::ReadResult>,
    owned: Option<bool>,
) -> d::Domain {
    let (mut root, _, _, watch) = f::root_at_with_needs(Address::Settings { project: f::PROJECT }, &limits());
    let mut entries = List::with_capacity(u32::try_from(values.len()).unwrap());
    for (i, (object, _, _)) in values.iter().enumerate() {
        entries.push(documents::entry(*object, f::PROJECT, u64::try_from(i).unwrap(), Wall::EPOCH, None)).unwrap();
    }
    let mut out = Queue::uncapped();
    event(
        &mut root,
        Event::Went { place: d::Place::Address(Address::Inbox { filter: d::address::Filter::ALL }) },
        &mut out,
    );
    event(
        &mut root,
        Event::Client {
            event: jig_client::Event::Up {
                up: jig_client::Up::Watched {
                    watch,
                    event: c::WatchEvent::Delivered(c::Delivery::Inbox(Box::new(documents::inbox(
                        entries.clone(),
                        u32::try_from(values.len()).unwrap(),
                        c::Count { value: 0, limited: false },
                        0,
                        false,
                    )))),
                },
            },
        },
        &mut out,
    );
    while let Some(request) = out.pop() {
        if let d::Request::Unlinked {
            request: jig_client::UnlinkedRequest::Down { down: jig_client::Down::Open { watch, subject } },
        } = &request
            && Some(subject) == d::forge::decided::watch(f::PROJECT, &limits().forge).as_ref()
        {
            deliver_asked(&mut root, *watch, None);
        }
        if let d::Request::Unlinked {
            request:
                jig_client::UnlinkedRequest::Down {
                    down: jig_client::Down::Read { read, what: jig_client::Readable::Jig(what) },
                },
        } = request
        {
            let page = if let c::Read::Inbox { group, .. } = &what {
                Some(c::ReadValue::Inbox(Box::new(c::InboxPage {
                    selection: None,
                    before: None,
                    rows: if *group == c::InboxReadGroup::Needs { entries.clone() } else { List::with_capacity(0) },
                    after: None,
                    cut: false,
                })))
            } else {
                values.iter().find(|(_, wanted, _)| *wanted == what).map(|(_, _, page)| page.clone())
            };
            let result =
                if let c::Read::TaskText { .. } = what { whole.clone() } else { page.map(c::ReadResult::Value) };
            if let Some(result) = result {
                event(
                    &mut root,
                    Event::Client {
                        event: jig_client::Event::Up {
                            up: jig_client::Up::ReadDone { read, result: jig_client::Readout::Jig(result) },
                        },
                    },
                    &mut out,
                );
            }
        } else if let d::Request::Unlinked {
            request:
                jig_client::UnlinkedRequest::Down {
                    down: jig_client::Down::Read { read, what: jig_client::Readable::Family { record, .. } },
                },
        } = request
            && let Some(owned_approval) = owned
            && let Some(reading) =
                temper_client_protocol_wire::decode_reading(&record.bytes, &temper_client_protocol_wire::v1::CEILINGS)
            && let temper_client_protocol_wire::v1::Read::ProposalItems(_) = reading.read()
        {
            use temper_client_protocol_wire::{self as wire, v1 as w};
            let answer = w::ProposalItemsAnswer::new(
                &w::CEILINGS,
                w::ProposalItemsAnswerParts { items: List::with_capacity(0), owned_approval },
            )
            .unwrap();
            event(
                &mut root,
                Event::Client {
                    event: jig_client::Event::Up {
                        up: jig_client::Up::ReadDone {
                            read,
                            result: jig_client::Readout::Family {
                                family: wire::FORGE,
                                record: Some(wire::proposal_items_answer(&answer)),
                            },
                        },
                    },
                },
                &mut out,
            );
        }
    }
    root
}

#[must_use]
pub fn approval_row() -> d::forge::approvals::Row {
    d::forge::approvals::Row {
        task: 81,
        project: f::PROJECT,
        goal: None,
        forge: 3,
        kind: d::forge::forges::Kind::Forgejo,
        owner: b"work".as_slice().into(),
        name: b"hello".as_slice().into(),
        pull: 24,
        address: b"https://forge.example/work/hello/pulls/24/files".as_slice().into(),
        title: b"A change".as_slice().into(),
        since: Wall::EPOCH,
        scope: f::PROJECT,
        rule: 1,
        role: 1,
        required: 1,
        counted: 0,
        sets: Box::new([]),
        whole: true,
        yours: d::forge::approvals::Yours::Counts,
        signed: Box::new([]),
        can_sign: Box::new([]),
        can_sign_more: 0,
        size: None,
    }
}

pub fn approvals(root: &mut d::Domain, rows: &[d::forge::approvals::Row], more: u32) {
    f::root_event(
        root,
        Event::Forge(d::forge::Event::Approvals {
            event: d::forge::approvals::Event::Rows { rows: rows.into(), more },
        }),
    );
}

#[must_use]
pub fn size_cases() -> [(Option<d::forge::approvals::Size>, &'static str, &'static str); 6] {
    use d::forge::approvals::Size;
    [
        (Some(Size { files: 4, additions: 142, deletions: 18, cut: false }), "+142 -18 in 4 files", " in 4 files"),
        (Some(Size { files: 1, additions: 3, deletions: 0, cut: false }), "+3 -0 in 1 file", " in 1 file"),
        (
            Some(Size { files: u32::MAX, additions: 10, deletions: 2, cut: true }),
            "+10 -2 in 4294967295+ files",
            " in 4294967295+ files",
        ),
        (
            Some(Size { files: 0, additions: u32::MAX, deletions: u32::MAX, cut: true }),
            "+4294967295+ -4294967295+ in 0 files",
            " in 0 files",
        ),
        (
            Some(Size { files: u32::MAX, additions: u32::MAX, deletions: u32::MAX, cut: false }),
            "+4294967295 -4294967295 in 4294967295 files",
            " in 4294967295 files",
        ),
        (None, "", ""),
    ]
}

pub fn check_size(tree: &skein_ui::Tree, words: &str, files: &str) {
    let nodes = tree.nodes();
    let rows: Vec<_> = nodes.iter().enumerate().filter(|(_, n)| n.styles.as_slice().contains(&look::CHIPS)).collect();
    assert_eq!(rows.len(), usize::from(!words.is_empty()));
    assert_eq!(look::NAMES.style(look::CHIPS), Some(b"temper-chips".as_slice()));
    assert_eq!(look::NAMES.style(look::SIZE_FILES), Some(b"temper-size-files".as_slice()));
    if let Some((start, row)) = rows.first() {
        let row = &nodes[*start..start.checked_add(usize::try_from(row.size).unwrap()).unwrap()];
        let chip = &row[1];
        assert_eq!(row.len(), usize::try_from(chip.size).unwrap().checked_add(1).unwrap(), "one chip only");
        assert_eq!(chip.styles.as_slice(), [look::CHIP]);
        assert!(row.iter().all(|n| n.binding.is_none() && n.link.is_none() && n.external.is_none()));
        let actual: Vec<_> = row.iter().flat_map(|n| n.text.iter()).flat_map(|run| run.text.iter().copied()).collect();
        assert_eq!(actual, words.as_bytes());
        let files_node =
            row.iter().find(|n| n.styles.as_slice().contains(&look::SIZE_FILES)).expect("files' own element");
        assert_eq!(files_node.name.as_deref(), Some(files.as_bytes()));
    } else {
        assert!(!nodes.iter().any(|n| n.styles.as_slice().contains(&look::SIZE_FILES)));
    }
}

#[must_use]
pub fn stop_root(owner: bool, set: bool) -> d::Domain {
    use temper_protocol_reaches::{Class, Classes, Item, Kind, PathSet, Reach, Reason, Subject, Way};
    let classes = if set {
        Classes {
            items: Box::new([Item { subject: Subject::Set(1), words: Box::default(), class: Class::Ask }]),
            sets: Box::new([PathSet {
                number: 1,
                name: b"protocol core".as_slice().into(),
                patterns: Box::new([b"src/**".as_slice().into(), b"wire/**".as_slice().into()]),
            }]),
        }
    } else {
        Classes::stock(Way::AgentsReview)
    };
    let (mut root, _, watch) = fixture_classes_role(
        QUESTION,
        false,
        1,
        false,
        d::address::Filter { project: Some(f::PROJECT), ..d::address::Filter::ALL },
        None,
        Some(&classes),
        owner,
        2,
    );
    deliver_asked(
        &mut root,
        watch,
        Some(Reach {
            kind: Kind::Implementation,
            class: Class::Ask,
            reason: if set { Reason::Set(1) } else { Reason::Kind },
            under: 0,
            sets: u8::from(set),
            matched: u8::from(set),
            away: false,
        }),
    );
    root
}
