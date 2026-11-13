use crate::{
    Action, Address, Domain, Event, Place, Request,
    address::{Filter, Kind},
    pages::{
        self,
        findings::{Act, Id, Mode, Need},
    },
    tests::fixtures as f,
};
use jig_client::{Down, Up, contract as c};
use skein_lib::{List, Queue, Token, Wall};
use temper_client_protocol_wire::{self as wire, v1 as w};
const ID: Id = Id { project: f::PROJECT, number: 7 };
fn event(root: &mut Domain, event: Event) -> Queue<Request> {
    super::welcome::event(root, event)
}
fn act(root: &mut Domain, act: Act) -> Queue<Request> {
    event(root, Event::Act { action: Action::Pages(pages::Act::Finding(act)) })
}
fn me(projects: u32) -> c::Me {
    let mut me = f::chats::me();
    let mut row = me.projects.get(0).unwrap().clone();
    let mut goal = row.templates.get(0).unwrap().clone();
    goal.number = jig_client::shapes::GOAL;
    goal.tracked = Some(1);
    row.templates = List::from_boxed(Box::new([goal]));
    me.projects = List::with_capacity(projects);
    for project in 1..=projects {
        let mut row = row.clone();
        row.project = project;
        me.projects.push(row).unwrap();
    }
    me
}
fn setup(projects: u32) -> (Domain, Token, Box<[(u32, Token)]>) {
    let mut root = crate::unlinked(&f::limits(), f::SEED);
    let watch = f::start(&mut root, List::with_capacity(0));
    let mut out = super::welcome::deliver(&mut root, watch, c::Delivery::Me(Box::new(me(projects))));
    let mut opening = event(&mut root, Event::Went { place: Place::Address(Address::Inbox { filter: Filter::ALL }) });
    while let Some(request) = opening.pop() {
        out.push(request);
    }
    let mut watches = List::with_capacity(4);
    let mut inbox = None;
    for request in &out {
        if let Request::Unlinked {
            request:
                jig_client::UnlinkedRequest::Down {
                    down: Down::Open { watch, subject: jig_client::Watchable::Jig(c::Subject::Inbox { .. }) },
                },
        } = request
        {
            inbox = Some(*watch);
        }
        if let Request::Unlinked {
            request:
                jig_client::UnlinkedRequest::Down {
                    down: Down::Open { watch, subject: jig_client::Watchable::Family { record, .. } },
                },
        } = request
            && let w::Watch::Findings(subject) = wire::decode_watching(&record.bytes, &w::CEILINGS).unwrap().watch()
        {
            assert_eq!(subject.state(), &Some(w::FindingState::New));
            assert!(subject.mine());
            watches.push((subject.project(), *watch)).unwrap();
        }
    }
    let inbox = inbox.unwrap();
    f::inbox_entries(&mut root, inbox, List::with_capacity(0));
    (root, inbox, watches.into_boxed())
}
fn row(number: u32, at: u64) -> w::FindingRow {
    w::FindingRow::new(
        &w::CEILINGS,
        w::FindingRowParts {
            number,
            at,
            kind: w::FindingKind::Feedback,
            severity: w::Severity::High,
            area: None,
            state: w::FindingState::New,
            title: b"The phone card is clipped".as_slice().into(),
            holder: w::Holder::Party(
                w::HolderParty::new(&w::CEILINGS, w::HolderPartyParts { number: f::PARTY }).unwrap(),
            ),
            why: Some(b"Your area owns the view".as_slice().into()),
            hander: Some(w::Filer::Task(w::HolderTask::new(&w::CEILINGS, w::HolderTaskParts { number: 55 }).unwrap())),
        },
    )
    .unwrap()
}
fn document(project: u32, rows: Box<[w::FindingRow]>, cut: bool) -> w::Findings {
    let holder = row(1, 0).holder().clone();
    w::Findings::new(
        &w::CEILINGS,
        w::FindingsParts {
            project,
            rows: List::from_boxed(rows),
            areas: List::from_boxed(Box::new([w::AreaRow::new(
                &w::CEILINGS,
                w::AreaRowParts {
                    name: b"view".as_slice().into(),
                    words: b"What the person sees".as_slice().into(),
                    owner: w::AreaOwner::Party(
                        w::AreaParty::new(&w::CEILINGS, w::AreaPartyParts { number: f::PARTY }).unwrap(),
                    ),
                    holder: Some(holder.clone()),
                },
            )
            .unwrap()])),
            triager: None,
            holder,
            cut,
        },
    )
    .unwrap()
}
fn deliver(root: &mut Domain, watch: Token, document: w::Findings) -> Queue<Request> {
    event(
        root,
        Event::Client {
            event: jig_client::Event::Up {
                up: Up::FamilyDelivered {
                    watch,
                    record: jig_client::Family {
                        family: wire::FORGE,
                        version: wire::CLIENT_VERSION,
                        bytes: wire::findings(&document),
                    },
                },
            },
        },
    )
}
fn ids(root: &Domain) -> Box<[Id]> {
    let mut ids = List::with_capacity(512);
    for need in &root.pages().needs().all(root.pages().inbox().unwrap(), root.forge().approvals()) {
        if let Need::Finding(card) = need {
            ids.push(card.id).unwrap();
        }
    }
    ids.into_boxed()
}
#[test]
fn team_areas_read_the_findings_document_and_its_triager() {
    use pages::team::{self, HeldArea, HeldAreas, Holder};
    let (mut root, _, watches) = setup(1);
    assert_eq!(team::areas(&root, 1), None, "unread is not empty");
    let task = w::Holder::Task(w::HolderTask::new(&w::CEILINGS, w::HolderTaskParts { number: 40 }).unwrap());
    let person = w::Holder::Party(w::HolderParty::new(&w::CEILINGS, w::HolderPartyParts { number: 2 }).unwrap());
    let mut parts = document(1, Box::new([]), false).into_parts();
    parts.areas = List::with_capacity(3);
    for (name, holder) in [(b"tests".as_slice(), Some(task)), (b"core", Some(person)), (b"docs", None)] {
        parts
            .areas
            .push(
                w::AreaRow::new(
                    &w::CEILINGS,
                    w::AreaRowParts {
                        name: name.into(),
                        words: name.into(),
                        owner: w::AreaOwner::Task(
                            w::AreaTask::new(&w::CEILINGS, w::AreaTaskParts { number: 40 }).unwrap(),
                        ),
                        holder,
                    },
                )
                .unwrap(),
            )
            .unwrap();
    }
    parts.triager = Some(w::AreaOwner::Task(w::AreaTask::new(&w::CEILINGS, w::AreaTaskParts { number: 41 }).unwrap()));
    parts.holder = w::Holder::Task(w::HolderTask::new(&w::CEILINGS, w::HolderTaskParts { number: 41 }).unwrap());
    deliver(&mut root, watches[0].1, w::Findings::new(&w::CEILINGS, parts.clone()).unwrap());
    assert_eq!(
        team::areas(&root, 1),
        Some(HeldAreas {
            rows: Box::new([
                HeldArea { name: b"tests".as_slice().into(), holder: Some(Holder::Task(40)) },
                HeldArea { name: b"core".as_slice().into(), holder: Some(Holder::Party(2)) },
                HeldArea { name: b"docs".as_slice().into(), holder: None },
            ]),
            triager: Some(Holder::Task(41)),
        })
    );
    assert_eq!(team::areas(&root, 2), None, "documents belong to their project");
    parts.triager = None;
    deliver(&mut root, watches[0].1, w::Findings::new(&w::CEILINGS, parts.clone()).unwrap());
    assert_eq!(team::areas(&root, 1).unwrap().triager, None, "no configured triager, despite fallback holder");
    parts.triager = Some(w::AreaOwner::Role(w::AreaRole::new(&w::CEILINGS, w::AreaRoleParts { number: 3 }).unwrap()));
    parts.holder = w::Holder::Role(w::HolderRole::new(&w::CEILINGS, w::HolderRoleParts { number: 3 }).unwrap());
    deliver(&mut root, watches[0].1, w::Findings::new(&w::CEILINGS, parts).unwrap());
    assert_eq!(team::areas(&root, 1).unwrap().triager, Some(Holder::Role(3)));
}

#[test]
fn findings_watch_first_projects_count_and_leave_on_the_next_document() {
    let (mut root, _, watches) = setup(5);
    assert_eq!(watches.len(), 4);
    for ((project, _), expected) in watches.iter().zip(1..=4) {
        assert_eq!(*project, expected);
    }
    assert!(root.pages().needs().findings().more);
    let watch = watches[0].1;
    deliver(&mut root, watch, document(ID.project, Box::new([row(7, 20), row(8, 10)]), true));
    assert_eq!(
        ids(&root).as_ref(),
        [Id { number: 8, ..ID }, ID],
        "oldest filed first, not the subject's newest-first order"
    );
    assert_eq!(pages::parts::needs_facts(&root, Wall::EPOCH).open, 2);
    let count = root.pages().needs().count(root.client(), root.forge().approvals(), ID.project);
    assert_eq!(count.count, 2);
    assert!(count.cut);
    let cards = root.pages().needs().findings().cards();
    assert_eq!(cards[0].row.hander(), row(7, 20).hander());
    assert_eq!(cards[0].row.why(), row(7, 20).why());
    act(&mut root, Act::Open { id: ID });
    let Some(Need::Finding(card)) =
        root.pages().needs().shown_all(root.pages().inbox().unwrap(), root.forge().approvals())
    else {
        panic!("finding shown")
    };
    assert_eq!(card.id, ID);
    deliver(&mut root, watch, document(ID.project, Box::new([]), false));
    assert!(ids(&root).is_empty());
    assert_eq!(pages::parts::needs_facts(&root, Wall::EPOCH).open, 0);
    let opening = event(
        &mut root,
        Event::Went { place: Place::Address(Address::Inbox { filter: Filter { project: Some(5), ..Filter::ALL } }) },
    );
    let mut focused = false;
    for request in &opening {
        if let Request::Unlinked {
            request:
                jig_client::UnlinkedRequest::Down {
                    down: Down::Open { subject: jig_client::Watchable::Family { record, .. }, .. },
                },
        } = request
            && let Some(watching) = wire::decode_watching(&record.bytes, &w::CEILINGS)
            && let w::Watch::Findings(subject) = watching.watch()
        {
            focused |= subject.project() == 5;
        }
    }
    assert!(focused, "a filtered project past the first four is watched");
    assert_eq!(
        crate::address::parse(&crate::address::print(&Address::Inbox {
            filter: Filter { kind: Some(Kind::Finding), ..Filter::ALL }
        })),
        Place::Address(Address::Inbox { filter: Filter { kind: Some(Kind::Finding), ..Filter::ALL } })
    );
}
fn sent(out: Queue<Request>) -> w::Request {
    let mut sent = List::with_capacity(1);
    for request in &out {
        if let Request::Unlinked {
            request:
                jig_client::UnlinkedRequest::Down {
                    down: Down::Send { request: jig_client::Ask::Family { scope, record }, .. },
                },
        } = request
        {
            assert_eq!(*scope, c::Scope::Project(ID.project));
            assert_eq!(record.family, wire::FORGE);
            sent.push(wire::decode_requesting(&record.bytes, &w::CEILINGS).unwrap().request().clone()).unwrap();
        }
    }
    assert_eq!(sent.len(), 1);
    sent.swap_remove(0).unwrap()
}
fn answer(root: &mut Domain, outcome: w::Outcome) {
    let key = root.pages().needs().findings().cards()[0].pending.unwrap();
    let answer = w::Answer::new(&w::CEILINGS, w::AnswerParts { outcome }).unwrap();
    crate::route::told(
        root,
        &f::env(&f::limits()),
        jig_client::Told::Answered {
            key,
            outcome: jig_client::Outcome::Answered(jig_client::Answered::Family(jig_client::Family {
                family: wire::FORGE,
                version: wire::CLIENT_VERSION,
                bytes: wire::answer(&answer),
            })),
        },
        &mut Queue::uncapped(),
    );
}
#[test]
fn finding_acts_send_the_forge_requests_and_keep_the_card_until_refresh() {
    let (mut root, _, watches) = setup(1);
    let watch = watches[0].1;
    deliver(&mut root, watch, document(ID.project, Box::new([row(7, 20)]), false));
    act(&mut root, Act::Area { id: ID, words: b"view".as_slice().into() });
    assert!(act(&mut root, Act::Hand { id: ID }).is_empty(), "a hand-off needs why");
    act(&mut root, Act::Why { id: ID, words: "é".repeat(101).into_bytes().into_boxed_slice() });
    assert!(act(&mut root, Act::Hand { id: ID }).is_empty(), "an over-bound UTF-8 prefix is not a shorter valid why");
    act(&mut root, Act::Why { id: ID, words: b"This is the view".as_slice().into() });
    let request = sent(act(&mut root, Act::Hand { id: ID }));
    let w::Request::HandFinding(request) = request else { panic!("hand request") };
    assert_eq!(
        (request.project(), request.number(), request.area(), request.why()),
        (ID.project, ID.number, b"view".as_slice(), b"This is the view".as_slice())
    );
    assert!(act(&mut root, Act::Backlog { id: ID }).is_empty(), "one send per pending card");
    answer(
        &mut root,
        w::Outcome::FindingRefused(
            w::FindingRefused::new(&w::CEILINGS, w::FindingRefusedParts { reason: w::FindingRefusal::NoGoal }).unwrap(),
        ),
    );
    assert_eq!(root.pages().needs().findings().cards()[0].refusal, Some(w::FindingRefusal::NoGoal));
    let out = act(&mut root, Act::Show { id: ID, mode: Mode::Route });
    let mut goals = None;
    for request in &out {
        if let Request::Unlinked {
            request:
                jig_client::UnlinkedRequest::Down {
                    down: Down::Open { watch, subject: jig_client::Watchable::Jig(c::Subject::Tasks { query, .. }) },
                },
        } = request
        {
            assert_eq!(query.project, Some(ID.project));
            assert_eq!(query.tracked, Some(true));
            goals = Some(*watch);
        }
    }
    let mut goal = super::chats::chat_line();
    goal.chip.task = 41;
    super::welcome::deliver(
        &mut root,
        goals.unwrap(),
        c::Delivery::Tasks(Box::new(c::Tasks { rows: List::from_boxed(Box::new([goal])), more: false })),
    );
    act(&mut root, Act::Goal { id: ID, task: 41 });
    let w::Request::TriageFinding(route) = sent(act(&mut root, Act::Route { id: ID })) else { panic!("triage") };
    let w::Triage::Route(goal) = route.act() else { panic!("route") };
    assert_eq!(goal.task(), 41);
    answer(&mut root, w::Outcome::Triaged);
    assert_eq!(ids(&root).as_ref(), [ID], "the answer never removes the card before the subject");
    let w::Request::TriageFinding(backlog) = sent(act(&mut root, Act::Backlog { id: ID })) else { panic!("triage") };
    assert_eq!(backlog.act(), &w::Triage::Backlog);
    answer(&mut root, w::Outcome::Triaged);
    act(&mut root, Act::Reason { id: ID, words: Box::new([b'x'; 401]) });
    assert!(act(&mut root, Act::Drop { id: ID }).is_empty(), "a drop reason fits the forge's bound");
    act(&mut root, Act::Reason { id: ID, words: b"Duplicate of the same report".as_slice().into() });
    let w::Request::TriageFinding(drop) = sent(act(&mut root, Act::Drop { id: ID })) else { panic!("triage") };
    let w::Triage::Drop(reason) = drop.act() else { panic!("drop") };
    assert_eq!(reason.reason(), b"Duplicate of the same report");
    answer(&mut root, w::Outcome::Triaged);
    deliver(&mut root, watch, document(ID.project, Box::new([]), false));
    assert!(ids(&root).is_empty());
}

#[test]
fn findings_join_questions_by_wait_and_obey_the_inbox_filters() {
    let (mut root, inbox, watches) = setup(1);
    deliver(&mut root, watches[0].1, document(ID.project, Box::new([row(7, 20), row(8, 10)]), false));
    let entries =
        List::from_boxed(Box::new([f::inbox_entry(f::acts::QUESTION, c::InboxGroup::Questions, ID.project, 1)]));
    let out = f::inbox_entries(&mut root, inbox, entries);
    for request in &out {
        if let Request::Unlinked {
            request:
                jig_client::UnlinkedRequest::Down {
                    down: Down::Read { read, what: jig_client::Readable::Jig(c::Read::Question { .. }) },
                },
        } = request
        {
            let c::ReadValue::Question(mut question) = f::acts::question() else { panic!("question fixture") };
            question.since = Wall::from_nanos(15);
            super::welcome::event(
                &mut root,
                Event::Client {
                    event: jig_client::Event::Up {
                        up: Up::ReadDone {
                            read: *read,
                            result: jig_client::Readout::Jig(c::ReadResult::Value(c::ReadValue::Question(question))),
                        },
                    },
                },
            );
        }
    }
    let all = root.pages().needs().all(root.pages().inbox().unwrap(), root.forge().approvals());
    let [Need::Finding(first), Need::Existing(pages::needs::Need::Card(question)), Need::Finding(last)] = all.as_ref()
    else {
        panic!("finding, question, finding: {all:?}")
    };
    assert_eq!(first.id.number, 8);
    assert_eq!(question.object, f::acts::QUESTION);
    assert_eq!(last.id, ID);
    for filter in [
        Filter { project: Some(2), ..Filter::ALL },
        Filter { goal: Some(41), ..Filter::ALL },
        Filter { kind: Some(Kind::Question), ..Filter::ALL },
    ] {
        root.pages.inbox.as_mut().unwrap().filter = filter;
        assert!(ids(&root).is_empty(), "filter {filter:?}");
    }
    root.pages.inbox.as_mut().unwrap().filter = Filter { kind: Some(Kind::Finding), ..Filter::ALL };
    // Jig's rows are filtered by its own refresh; the findings' inclusion is independent of it.
    assert_eq!(ids(&root).len(), 2);
}
