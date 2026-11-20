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
    let (root, inbox, findings, _) = setup_watches(projects);
    (root, inbox, findings)
}
#[expect(clippy::type_complexity, reason = "the fixture returns the inbox token and the two kinds of project watches")]
fn setup_watches(projects: u32) -> (Domain, Token, Box<[(u32, Token)]>, Box<[(u32, Token)]>) {
    setup_watches_with(projects, &f::limits())
}
#[expect(clippy::type_complexity, reason = "the fixture returns the inbox and both project watches")]
fn setup_watches_with(
    projects: u32,
    limits: &crate::Limits,
) -> (Domain, Token, Box<[(u32, Token)]>, Box<[(u32, Token)]>) {
    let mut root = crate::unlinked(limits, f::SEED);
    let watch = f::start(&mut root, List::with_capacity(0));
    let mut out = super::welcome::deliver(&mut root, watch, c::Delivery::Me(Box::new(me(projects))));
    let mut opening = Queue::uncapped();
    crate::step(
        &mut root,
        &f::env(limits),
        Event::Went { place: Place::Address(Address::Inbox { filter: Filter::ALL }) },
        &mut opening,
    );
    while let Some(request) = opening.pop() {
        out.push(request);
    }
    let mut watches = List::with_capacity(4);
    let mut decisions = List::with_capacity(4);
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
        {
            let watching = wire::decode_watching(&record.bytes, &w::CEILINGS).unwrap();
            if let w::Watch::Findings(subject) = &watching.watch {
                let w::FindingsFilter::Triage(filter) = &subject.filter else { panic!("triage watch") };
                assert_eq!(filter.state, Some(w::FindingState::New));
                assert!(filter.mine);
                watches.push((subject.project, *watch)).unwrap();
            }
            if let w::Watch::Decided(subject) = &watching.watch {
                decisions.push((subject.project, *watch)).unwrap();
            }
        }
    }
    let inbox = inbox.unwrap();
    f::inbox_entries(&mut root, inbox, List::with_capacity(0));
    (root, inbox, watches.into_boxed(), decisions.into_boxed())
}
fn row(number: u32, at: u64) -> w::FindingRow {
    w::FindingRow {
        number,
        at,
        kind: w::FindingKind::Feedback,
        severity: w::Severity::High,
        area: None,
        state: w::FindingState::New,
        title: b"The phone card is clipped".as_slice().into(),
        holder: w::Holder::Party(w::HolderParty { number: f::PARTY }),
        why: Some(b"Your area owns the view".as_slice().into()),
        hander: Some(w::Filer::Task(w::HolderTask { number: 55 })),
        goal: None,
        reason: None,
        cleared: None,
    }
}
fn document(project: u32, rows: Box<[w::FindingRow]>, cut: bool) -> w::Findings {
    let holder = row(1, 0).holder.clone();
    w::Findings {
        project,
        rows: List::from_boxed(rows),
        areas: List::from_boxed(Box::new([w::AreaRow {
            name: b"view".as_slice().into(),
            words: b"What the person sees".as_slice().into(),
            owner: w::AreaOwner::Party(w::AreaParty { number: f::PARTY }),
            holder: Some(holder.clone()),
        }])),
        triager: None,
        holder,
        cut,
    }
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
fn finding_why_uses_the_area_or_the_standing_person_triager_after_the_document() {
    use pages::{
        findings::why,
        needs::{Holds, Why},
    };
    let (mut root, _, watches) = setup(1);
    let (unread, _, _) = setup(1);
    let mine = w::AreaOwner::Party(w::AreaParty { number: f::PARTY });
    let other = w::AreaOwner::Party(w::AreaParty { number: 99 });
    for (area, triager, expected) in [
        (Some(b"store".as_slice()), None, Holds::Area { name: b"store".as_slice().into() }),
        (None, Some(mine), Holds::Triager),
        (None, Some(other), Holds::Owner),
        (None, None, Holds::Owner),
    ] {
        let mut row = row(7, 20);
        row.area = match area {
            Some(name) => Some(name.into()),
            None => None,
        };
        let mut parts = document(1, Box::new([row]), false);
        parts.triager = triager;
        deliver(&mut root, watches[0].1, parts);
        let cards = root.pages().needs().findings().cards();
        let need = Need::Finding(cards[0]);
        assert_eq!(why(&unread, need), None);
        assert_eq!(why(&root, need), Some(Why::Finding { project: 1, holds: expected }));
    }
    let other = w::AreaOwner::Party(w::AreaParty { number: 99 });
    let mut parts = document(1, Box::new([row(7, 20)]), false);
    parts.triager = Some(other);
    parts.holder = w::Holder::Party(w::HolderParty { number: 99 });
    deliver(&mut root, watches[0].1, parts);
    let cards = root.pages().needs().findings().cards();
    assert_eq!(why(&root, Need::Finding(cards[0])), Some(Why::Finding { project: 1, holds: Holds::Owner }));
}

#[test]
fn team_areas_read_the_findings_document_and_its_triager() {
    use pages::team::{self, HeldArea, HeldAreas, Holder};
    let (mut root, _, watches) = setup(1);
    assert_eq!(team::areas(&root, 1), None, "unread is not empty");
    let task = w::Holder::Task(w::HolderTask { number: 40 });
    let person = w::Holder::Party(w::HolderParty { number: 2 });
    let mut parts = document(1, Box::new([]), false);
    parts.areas = List::with_capacity(3);
    for (name, holder) in [(b"tests".as_slice(), Some(task)), (b"core", Some(person)), (b"docs", None)] {
        parts
            .areas
            .push(w::AreaRow {
                name: name.into(),
                words: name.into(),
                owner: w::AreaOwner::Task(w::AreaTask { number: 40 }),
                holder,
            })
            .unwrap();
    }
    parts.triager = Some(w::AreaOwner::Task(w::AreaTask { number: 41 }));
    parts.holder = w::Holder::Task(w::HolderTask { number: 41 });
    deliver(&mut root, watches[0].1, parts.clone());
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
    deliver(&mut root, watches[0].1, parts.clone());
    assert_eq!(team::areas(&root, 1).unwrap().triager, None, "no configured triager, despite fallback holder");
    parts.triager = Some(w::AreaOwner::Role(w::AreaRole { number: 3 }));
    parts.holder = w::Holder::Role(w::HolderRole { number: 3 });
    deliver(&mut root, watches[0].1, parts);
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
    assert_eq!(&cards[0].row.hander, &row(7, 20).hander);
    assert_eq!(&cards[0].row.why, &row(7, 20).why);
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
            && let w::Watch::Findings(subject) = &watching.watch
        {
            focused |= subject.project == 5;
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
            sent.push(wire::decode_requesting(&record.bytes, &w::CEILINGS).unwrap().request.clone()).unwrap();
        }
    }
    assert_eq!(sent.len(), 1);
    sent.swap_remove(0).unwrap()
}
fn answer(root: &mut Domain, outcome: w::Outcome) {
    let key = root.pages().needs().findings().cards()[0].pending.unwrap();
    let answer = w::Answer { outcome };
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
        (request.project, request.number, request.area.as_ref(), request.why.as_ref()),
        (ID.project, ID.number, b"view".as_slice(), b"This is the view".as_slice())
    );
    assert!(act(&mut root, Act::Backlog { id: ID }).is_empty(), "one send per pending card");
    answer(&mut root, w::Outcome::FindingRefused(w::FindingRefused { reason: w::FindingRefusal::NoGoal }));
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
        c::Delivery::Tasks(Box::new(c::Tasks { ended_mark: 0, rows: List::from_boxed(Box::new([goal])), more: false })),
    );
    act(&mut root, Act::Goal { id: ID, task: 41 });
    let w::Request::TriageFinding(route) = sent(act(&mut root, Act::Route { id: ID })) else { panic!("triage") };
    let w::Triage::Route(goal) = &route.act else { panic!("route") };
    assert_eq!(goal.task, 41);
    answer(&mut root, w::Outcome::Triaged);
    assert_eq!(ids(&root).as_ref(), [ID], "the answer never removes the card before the subject");
    let w::Request::TriageFinding(backlog) = sent(act(&mut root, Act::Backlog { id: ID })) else { panic!("triage") };
    assert_eq!(&backlog.act, &w::Triage::Backlog);
    answer(&mut root, w::Outcome::Triaged);
    act(&mut root, Act::Reason { id: ID, words: Box::new([b'x'; 401]) });
    assert!(act(&mut root, Act::Drop { id: ID }).is_empty(), "a drop reason fits the forge's bound");
    act(&mut root, Act::Reason { id: ID, words: b"Duplicate of the same report".as_slice().into() });
    let w::Request::TriageFinding(drop) = sent(act(&mut root, Act::Drop { id: ID })) else { panic!("triage") };
    let w::Triage::Drop(reason) = &drop.act else { panic!("drop") };
    assert_eq!(reason.reason.as_ref(), b"Duplicate of the same report");
    answer(&mut root, w::Outcome::Triaged);
    deliver(&mut root, watch, document(ID.project, Box::new([]), false));
    assert!(ids(&root).is_empty());
}

#[test]
fn open_cards_rank_severity_before_age_and_keep_later_last() {
    use pages::findings::{Counted, counted};
    const HOUR: u64 = 3_600_000_000_000;
    const NOW: u64 = 3 * 24 * HOUR;
    let (mut root, inbox, watches) = setup(1);
    let later = c::ObjectKey::Question { task: f::acts::TASK, message: 99 };
    f::inbox_entries(
        &mut root,
        inbox,
        List::from_boxed(Box::new([
            f::inbox_entry(f::acts::QUESTION, c::InboxGroup::Questions, ID.project, 1),
            f::inbox_entry(later, c::InboxGroup::Questions, ID.project, 2),
        ])),
    );
    for card in root.pages.inbox.as_mut().unwrap().cards.iter_mut() {
        let c::ReadValue::Question(mut question) = f::acts::question() else { panic!("question fixture") };
        question.since = Wall::from_nanos(if card.object == later { 0 } else { NOW - HOUR });
        card.page = Some(c::ReadValue::Question(question));
    }
    let rows = [
        (1, NOW - 48 * HOUR, w::Severity::Low),
        (2, NOW - 24 * HOUR, w::Severity::High),
        (3, NOW - 3 * HOUR, w::Severity::Medium),
    ]
    .map(|(number, at, severity)| {
        let mut parts = row(number, at);
        parts.severity = severity;
        parts
    });
    deliver(&mut root, watches[0].1, document(ID.project, Box::new(rows), false));
    event(
        &mut root,
        Event::Act { action: Action::Pages(pages::Act::Needs(pages::needs::Act::Later { object: later })) },
    );
    let all = root.pages().needs().all(root.pages().inbox().unwrap(), root.forge().approvals());
    let [
        Need::Finding(high),
        Need::Existing(pages::needs::Need::Card(question)),
        Need::Finding(medium),
        Need::Finding(low),
        Need::Existing(pages::needs::Need::Card(last)),
    ] = all.as_ref()
    else {
        panic!("ranked cards: {all:?}")
    };
    assert_eq!(
        (high.id.number, question.object, medium.id.number, low.id.number, last.object),
        (2, f::acts::QUESTION, 3, 1, later)
    );
    for (need, severity) in
        all.iter().zip([Some(w::Severity::High), None, Some(w::Severity::Medium), Some(w::Severity::Low), None])
    {
        assert_eq!(need.severity(root.pages().needs().findings()), severity);
    }
    assert_eq!(counted(root.pages().needs().findings(), &all), Counted { high: 1, medium: 1, low: 1, only: false });
    assert_eq!(
        counted(root.pages().needs().findings(), &[Need::Finding(high), Need::Finding(medium), Need::Finding(low)]),
        Counted { high: 1, medium: 1, low: 1, only: true }
    );
    assert_eq!(counted(root.pages().needs().findings(), &[]), Counted { high: 0, medium: 0, low: 0, only: false });
    let mut proposal = (*question).clone();
    proposal.object = c::ObjectKey::Proposal(f::acts::PROPOSAL);
    assert_eq!(Need::Existing(pages::needs::Need::Card(&proposal)).severity(root.pages().needs().findings()), None);
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

const GROUP_WORDS: &[u8] = "Three bugs?\nFindings: 2/2 2/8\nGroup: bugs · in temper\nChoice: A".as_bytes();

fn group_page(person: bool, words: &[u8]) -> c::ReadValue {
    let words = c::DocumentText::complete(c::Text::new(words.into(), 512).unwrap());
    if person {
        let c::ReadValue::PersonTask(mut task) = f::acts::person(c::ActKind::Answer) else { panic!("person fixture") };
        task.spec.specification.words = words;
        task.since = Some(Wall::from_nanos(40));
        c::ReadValue::PersonTask(task)
    } else {
        let c::ReadValue::Question(mut question) = f::acts::question() else { panic!("question fixture") };
        question.words = words;
        question.since = Wall::from_nanos(40);
        c::ReadValue::Question(question)
    }
}
fn open_findings(out: &Queue<Request>) -> Option<Token> {
    let mut found = None;
    for request in out {
        if let Request::Unlinked {
            request:
                jig_client::UnlinkedRequest::Down {
                    down: Down::Open { watch, subject: jig_client::Watchable::Family { record, .. } },
                },
        } = request
            && let w::Watch::Findings(subject) = wire::decode_watching(&record.bytes, &w::CEILINGS).unwrap().watch
            && let w::FindingsFilter::Triage(filter) = subject.filter
            && !filter.mine
        {
            assert_eq!(subject.project, 2);
            assert_eq!(filter.state, Some(w::FindingState::New));
            assert!(found.is_none(), "one open findings watch per project");
            found = Some(*watch);
        }
    }
    found
}
fn group_inbox(limits: &crate::Limits) -> (Domain, Token, Token) {
    let (mut root, inbox, mine, decisions) = setup_watches_with(2, limits);
    for (project, watch) in decisions {
        let document =
            w::Decisions { project, rows: List::with_capacity(0), asked: List::with_capacity(0), cut: false };
        event(
            &mut root,
            Event::Client {
                event: jig_client::Event::Up {
                    up: Up::FamilyDelivered {
                        watch,
                        record: jig_client::Family {
                            family: wire::FORGE,
                            version: wire::CLIENT_VERSION,
                            bytes: wire::decisions(&document),
                        },
                    },
                },
            },
        );
    }
    let mut own = None;
    for (project, watch) in mine {
        if project == 2 {
            own = Some(watch);
        }
    }
    (root, inbox, own.unwrap())
}
fn group_setup(person: bool, words: &[u8]) -> (Domain, Token, Token, Option<Token>) {
    let (mut root, inbox, own) = group_inbox(&f::limits());
    let object = if person { c::ObjectKey::PersonTask(f::acts::TASK) } else { f::acts::QUESTION };
    let entries = List::from_boxed(Box::new([f::inbox_entry(object, c::InboxGroup::Questions, 2, 1)]));
    let out = f::inbox_entries(&mut root, inbox, entries);
    let mut open = None;
    for request in &out {
        if let Request::Unlinked { request: jig_client::UnlinkedRequest::Down { down: Down::Read { read, .. } } } =
            request
        {
            let out = event(
                &mut root,
                Event::Client {
                    event: jig_client::Event::Up {
                        up: Up::ReadDone {
                            read: *read,
                            result: jig_client::Readout::Jig(c::ReadResult::Value(group_page(person, words))),
                        },
                    },
                },
            );
            if let Some(watch) = open_findings(&out) {
                open = Some(watch);
            }
        }
    }
    (root, inbox, own, open)
}
fn closed(out: &Queue<Request>, token: Token) -> bool {
    for request in out {
        if let Request::Unlinked { request: jig_client::UnlinkedRequest::Down { down: Down::Close { watch } } } =
            request
            && *watch == token
        {
            return true;
        }
    }
    false
}
fn group_need(root: &Domain) -> Need<'_> {
    Need::Existing(pages::needs::Need::Card(root.pages().inbox().unwrap().cards.get(0).unwrap()))
}
fn severity_row(number: u32, at: u64, severity: w::Severity) -> w::FindingRow {
    let mut parts = row(number, at);
    parts.severity = severity;
    parts
}

#[test]
fn groups_watch_whole_questions_hold_then_match_rows_in_line_order() {
    for person in [false, true] {
        let (mut root, inbox, mine, open) = group_setup(person, GROUP_WORDS);
        let open = open.expect("whole question opens project 2's new findings, not mine");
        assert!(!group_need(&root).ready(&root));
        assert!(root.pages().needs().findings().group(group_need(&root)).is_none());
        deliver(&mut root, mine, document(2, Box::new([severity_row(7, 1, w::Severity::Medium)]), false));
        assert!(!group_need(&root).ready(&root), "mine is not the group's read");
        deliver(
            &mut root,
            open,
            document(
                2,
                Box::new([severity_row(8, 10, w::Severity::Low), severity_row(2, 30, w::Severity::High)]),
                true,
            ),
        );
        let store = root.pages().needs().findings();
        let need = group_need(&root);
        assert!(need.ready(&root));
        let group = store.group(need).unwrap();
        assert_eq!(group.project, 2);
        assert_eq!(&*group.noun, b"bugs");
        assert_eq!(&*group.phrase, b"in temper");
        let [first, second] = group.rows.as_ref() else { panic!("two named rows") };
        assert_eq!((first.number, second.number), (2, 8));
        assert_eq!(group.severity(), w::Severity::High);
        assert_eq!(group.oldest(), Wall::from_nanos(10));
        assert_eq!(need.severity(store), Some(w::Severity::High));
        let all = root.pages().needs().all(root.pages().inbox().unwrap(), root.forge().approvals());
        assert!(all[0].same(need), "a newer high group precedes the older medium finding");
        let Need::Finding(medium) = all[1] else { panic!("medium finding after high group") };
        assert_eq!(medium.id.number, 7);
        deliver(
            &mut root,
            open,
            document(
                2,
                Box::new([severity_row(8, 10, w::Severity::Low), severity_row(2, 30, w::Severity::Medium)]),
                false,
            ),
        );
        assert_eq!(group_need(&root).severity(root.pages().needs().findings()), Some(w::Severity::Medium));
        deliver(&mut root, open, document(2, Box::new([severity_row(8, 10, w::Severity::Low)]), false));
        assert_eq!(group_need(&root).severity(root.pages().needs().findings()), Some(w::Severity::Low));
        deliver(&mut root, open, document(2, Box::new([]), false));
        assert!(group_need(&root).ready(&root));
        assert!(root.pages().needs().findings().group(group_need(&root)).is_none());
        assert_eq!(group_need(&root).severity(root.pages().needs().findings()), None);
        let out = f::inbox_entries(&mut root, inbox, List::with_capacity(0));
        assert!(closed(&out, open));
        assert!(!closed(&out, mine));
    }
}

fn read_value(root: &mut Domain, read: Token, value: c::ReadValue) -> Queue<Request> {
    event(
        root,
        Event::Client {
            event: jig_client::Event::Up {
                up: Up::ReadDone { read, result: jig_client::Readout::Jig(c::ReadResult::Value(value)) },
            },
        },
    )
}

fn standing_task(number: u64) -> c::Task {
    let mut standing = f::chat::task();
    standing.chip.task = number;
    standing.chip.title = f::chat::document_text(b"Triager");
    standing.chip.executor = c::ChipExecutor::Agent { charter: 1 };
    standing.chip.details.details.executor = c::Executor::Agent { charter: 1 };
    standing.flags.ended_by = c::EndedBy::Standing;
    standing.project = 2;
    standing.requester = c::Requester::Person(f::PARTY);
    standing.priority = None;
    standing.goal = None;
    standing.lineage = List::with_capacity(0);
    standing.lineage_chips = List::with_capacity(0);
    standing
}

#[test]
fn standing_person_groups_read_whole_words_then_open_in_line_order() {
    const STANDING: u64 = 55;
    for cut in [false, true] {
        let mut limits = f::limits();
        limits.client.domain.long_text = 512;
        let (mut root, inbox, _) = group_inbox(&limits);
        let object = c::ObjectKey::PersonTask(f::acts::TASK);
        let out = f::inbox_entries(
            &mut root,
            inbox,
            List::from_boxed(Box::new([f::inbox_entry(object, c::InboxGroup::PersonTasks, 2, 1)])),
        );
        let mut person_read = None;
        for request in &out {
            if let Request::Unlinked {
                request:
                    jig_client::UnlinkedRequest::Down {
                        down: Down::Read { read, what: jig_client::Readable::Jig(c::Read::PersonTask { task }) },
                    },
            } = request
            {
                assert_eq!(*task, f::acts::TASK);
                assert!(person_read.replace(*read).is_none());
            }
        }
        let read = person_read.expect("the inbox reads its person task");
        let c::ReadValue::PersonTask(mut person) = group_page(true, GROUP_WORDS) else { panic!("person fixture") };
        person.requester = c::Requester::Task(STANDING);
        person.task.details.details.executor = c::Executor::Person(person.address);
        person.spec.specification.words.cut = cut;
        let out = read_value(&mut root, read, c::ReadValue::PersonTask(person));
        let mut open = open_findings(&out);
        let mut whole = None;
        let mut source = None;
        for request in &out {
            if let Request::Unlinked {
                request:
                    jig_client::UnlinkedRequest::Down { down: Down::Read { read, what: jig_client::Readable::Jig(what) } },
            } = request
            {
                if let c::Read::Task { task } = what {
                    assert_eq!(*task, STANDING);
                    source = Some(*read);
                } else if let c::Read::TaskText { task, field, from } = what {
                    assert_eq!((*task, *field, *from), (f::acts::TASK, c::TaskTextField::Specification, 0));
                    whole = Some(*read);
                } else {
                    panic!("unexpected read {what:?}");
                }
            }
        }
        let standing = standing_task(STANDING);
        read_value(
            &mut root,
            source.expect("read the standing requester"),
            c::ReadValue::Task(Box::new(standing.clone())),
        );
        assert_eq!(root.pages().inbox().unwrap().cards.get(0).unwrap().document(), Some(&standing));
        assert_eq!(whole.is_some(), cut);
        if cut {
            assert!(open.is_none(), "cut words must be read whole before watching findings");
            assert!(root.pages().needs().findings().group(group_need(&root)).is_none());
            let out = read_value(
                &mut root,
                whole.unwrap(),
                c::ReadValue::TaskText(Box::new(c::TaskText {
                    task: f::acts::TASK,
                    field: c::TaskTextField::Specification,
                    from: 0,
                    total: u64::try_from(GROUP_WORDS.len()).unwrap(),
                    next: None,
                    text: c::Text::new(GROUP_WORDS.into(), 512).unwrap(),
                })),
            );
            open = open_findings(&out);
        }
        assert!(!group_need(&root).ready(&root), "wait for the project's open findings");
        deliver(
            &mut root,
            open.expect("whole standing question watches open findings"),
            document(
                2,
                Box::new([severity_row(8, 10, w::Severity::Low), severity_row(2, 30, w::Severity::High)]),
                false,
            ),
        );
        let store = root.pages().needs().findings();
        let need = group_need(&root);
        assert!(need.ready(&root));
        let open = root.pages().needs().open(root.pages().inbox().unwrap());
        let [card] = open.as_ref() else { panic!("one open person card") };
        assert_eq!(card.object, object);
        let group = store.group(need).expect("a standing role's person task is a group");
        assert_eq!(group.project, 2);
        assert_eq!(&*group.noun, b"bugs");
        assert_eq!(&*group.phrase, b"in temper");
        let [first, second] = group.rows.as_ref() else { panic!("two findings") };
        assert_eq!((first.number, second.number), (2, 8));
        assert_eq!(need.severity(store), Some(w::Severity::High));
    }
}

#[test]
fn groups_end_or_refuse_to_plain_questions() {
    for (delivered, end) in [
        (false, c::WatchEnd::Closed),
        (false, c::WatchEnd::NotAllowed),
        (true, c::WatchEnd::Closed),
        (true, c::WatchEnd::NotAllowed),
    ] {
        let (mut root, _, _, open) = group_setup(false, GROUP_WORDS);
        let open = open.unwrap();
        assert!(!group_need(&root).ready(&root));
        if delivered {
            deliver(&mut root, open, document(2, Box::new([row(2, 30)]), false));
            assert!(root.pages().needs().findings().group(group_need(&root)).is_some());
        }
        let out = event(
            &mut root,
            Event::Client { event: jig_client::Event::Up { up: Up::WatchEnded { watch: open, end } } },
        );
        assert!(group_need(&root).ready(&root));
        assert!(root.pages().needs().findings().group(group_need(&root)).is_none());
        assert_eq!(group_need(&root).severity(root.pages().needs().findings()), None);
        assert_eq!(open_findings(&out), None, "no repeated refused watch");
    }
}

#[test]
fn groups_ignore_other_projects_closed_and_missing_rows_and_allow_no_label() {
    let (mut root, _, _, open) = group_setup(false, b"Q?\nFindings: 2/8 1/2 2/10 2/2\nChoice: A");
    let mut closed = row(8, 5);
    closed.state = w::FindingState::Backlog;
    deliver(&mut root, open.unwrap(), document(2, Box::new([closed, row(2, 30)]), true));
    let group = root.pages().needs().findings().group(group_need(&root)).unwrap();
    assert_eq!(group.rows.len(), 1);
    assert_eq!(group.rows[0].number, 2);
    assert!(group.noun.is_empty());
    assert!(group.phrase.is_empty());
}

#[test]
fn groups_count_findings_once_across_cards_and_plain_questions_change_only() {
    use pages::findings::{Counted, counted};
    let (mut root, _, mine, open) = group_setup(false, GROUP_WORDS);
    deliver(&mut root, mine, document(2, Box::new([severity_row(7, 1, w::Severity::Medium)]), false));
    deliver(
        &mut root,
        open.unwrap(),
        document(2, Box::new([severity_row(8, 10, w::Severity::Low), row(2, 30)]), false),
    );
    let first = root.pages().inbox().unwrap().cards.get(0).unwrap();
    let mut second = first.clone();
    second.object = c::ObjectKey::Question { task: 55, message: 1 };
    second.page = Some(group_page(false, b"Q?\nFindings: 2/8\nChoice: A"));
    let mut plain = first.clone();
    plain.object = c::ObjectKey::Question { task: 55, message: 2 };
    plain.page = Some(group_page(false, b"Q?\nChoice: A"));
    let store = root.pages().needs().findings();
    let mine = store.cards()[0];
    let open = [group_need(&root), Need::Existing(pages::needs::Need::Card(&second)), Need::Finding(mine)];
    let counts = counted(store, &open);
    assert_eq!(counts, Counted { high: 1, medium: 1, low: 1, only: true });
    assert_eq!(counts.total(), 3);
    let mixed = [open[0], open[1], open[2], Need::Existing(pages::needs::Need::Card(&plain))];
    assert_eq!(counted(store, &mixed), Counted { only: false, ..counts });
    assert_eq!(counted(store, &mixed).total(), 3);
}

#[test]
fn groups_with_no_project_or_watch_or_whole_words_are_plain() {
    let (root, _, _, _) = group_setup(false, GROUP_WORDS);
    let template = root.pages().inbox().unwrap().cards.get(0).unwrap();
    for project in [None, Some(99)] {
        let mut card = template.clone();
        card.project = project;
        let need = Need::Existing(pages::needs::Need::Card(&card));
        assert!(need.ready(&root));
        assert!(root.pages().needs().findings().group(need).is_none());
    }
    for person in [false, true] {
        let (mut root, _, _, open) = group_setup(person, GROUP_WORDS);
        let open = open.unwrap();
        deliver(&mut root, open, document(2, Box::new([row(2, 30)]), false));
        let card = root.pages.inbox.as_mut().unwrap().cards.get_mut(0).unwrap();
        if let Some(c::ReadValue::Question(question)) = card.page.as_mut() {
            question.words.cut = true;
        } else if let Some(c::ReadValue::PersonTask(task)) = card.page.as_mut() {
            task.spec.specification.words.cut = true;
        } else {
            panic!("question card");
        }
        let out = event(&mut root, Event::ScreenLeft);
        assert!(group_need(&root).ready(&root));
        assert!(root.pages().needs().findings().group(group_need(&root)).is_none());
        assert!(closed(&out, open));
    }
}
