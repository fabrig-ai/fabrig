use super::{Filter, InboxGroup, ObjectKey, PROJECT, Request, contract, fixtures};
use jig_client::{Down, Readable, Readout, Up};
use skein_lib::{List, Queue, Token, Wall};

const TOTAL: u32 = 29;
const WINDOW: u32 = 8;
const ESCALATIONS: u32 = 11;

fn limits() -> crate::Limits {
    let mut limits = fixtures::limits();
    limits.client.domain.rows = WINDOW;
    limits.client.domain.reads = 16;
    limits
}
fn event(root: &mut crate::Domain, up: Up) -> Queue<Request> {
    let mut out = Queue::uncapped();
    crate::step(
        root,
        &fixtures::env(&limits()),
        crate::Event::Client { event: jig_client::Event::Up { up } },
        &mut out,
    );
    out
}
fn entries(start: u32, end: u32) -> List<contract::InboxEntry> {
    let mut rows = List::with_capacity(end.checked_sub(start).unwrap());
    for number in start..end {
        let object = if number < ESCALATIONS {
            ObjectKey::Escalation(u64::from(number))
        } else {
            ObjectKey::PersonTask(u64::from(number))
        };
        let mut row = fixtures::inbox_entry(object, InboxGroup::Escalations, PROJECT, u64::from(number));
        row.at = Some(Wall::from_nanos(u64::from(number)));
        rows.push(row).unwrap();
    }
    rows
}
fn cursor(number: u32) -> contract::Cursor {
    contract::Cursor::new(number.to_le_bytes().into(), 4).unwrap()
}
fn value(rows: List<contract::InboxEntry>, after: Option<contract::Cursor>) -> contract::ReadResult {
    contract::ReadResult::Value(contract::ReadValue::Inbox(Box::new(contract::InboxPage {
        rows,
        after,
        before: None,
        cut: false,
        selection: None,
    })))
}
fn answer(root: &mut crate::Domain, read: Token, result: contract::ReadResult) -> Queue<Request> {
    event(root, Up::ReadDone { read, result: Readout::Jig(result) })
}
fn queries(out: &Queue<Request>) -> Box<[(Token, contract::Read)]> {
    let mut found = List::with_capacity(out.len());
    for request in out {
        if let Request::Unlinked {
            request: jig_client::UnlinkedRequest::Down { down: Down::Read { read, what: Readable::Jig(what) } },
        } = request
        {
            found.push((*read, what.clone())).unwrap();
        }
    }
    found.into_boxed()
}
fn section_reads(out: &Queue<Request>, section: contract::InboxReadGroup) -> Box<[(Token, contract::Read)]> {
    let mut found = List::with_capacity(out.len());
    for (read, what) in queries(out) {
        if let contract::Read::Inbox { group, .. } = &what
            && *group == section
        {
            found.push((read, what)).unwrap();
        }
    }
    found.into_boxed()
}
fn begin(filter: Filter, more: bool) -> (crate::Domain, Token, Queue<Request>) {
    let (mut root, watch) = fixtures::inbox_home_with_limits(filter, &limits());
    let document = crate::tests::documents::inbox(
        entries(0, WINDOW),
        TOTAL,
        contract::Count { value: 0, limited: false },
        0,
        more,
    );
    let out = event(
        &mut root,
        Up::Watched { watch, event: contract::WatchEvent::Delivered(contract::Delivery::Inbox(Box::new(document))) },
    );
    (root, watch, out)
}
fn needs_read(out: &Queue<Request>, after: Option<contract::Cursor>) -> Token {
    let found = section_reads(out, contract::InboxReadGroup::Needs);
    assert_eq!(found.len(), 1, "one needs page in flight");
    if let contract::Read::Inbox { after: actual, .. } = &found[0].1 {
        assert_eq!(actual, &after);
    } else {
        panic!("needs read");
    }
    found[0].0
}
fn finish_cards(root: &mut crate::Domain, reads: &[(Token, contract::Read)]) {
    for (read, what) in reads {
        if let contract::Read::Inbox { .. } = what {
            continue;
        }
        answer(root, *read, contract::ReadResult::None(contract::ReadNone::Busy));
    }
}

#[test]
fn needs_read_every_page_but_only_the_window_of_cards() {
    for filter in [Filter { project: Some(PROJECT), ..Filter::ALL }, Filter::ALL] {
        let (mut root, watch, mut out) = begin(filter, true);
        let mut card_reads = queries(&out);
        if filter.project.is_some() {
            let updates = section_reads(&out, contract::InboxReadGroup::Updates);
            assert_eq!(updates.len(), 1);
            answer(
                &mut root,
                updates[0].0,
                contract::ReadResult::Value(contract::ReadValue::Inbox(Box::new(contract::InboxPage {
                    rows: List::with_capacity(0),
                    after: None,
                    before: None,
                    cut: true,
                    selection: None,
                }))),
            );
        }
        let mut read = needs_read(&out, None);
        for start in (0..TOTAL).step_by(usize::try_from(WINDOW).unwrap()) {
            let end = (start + WINDOW).min(TOTAL);
            let after = if end < TOTAL { Some(cursor(end)) } else { None };
            out = answer(&mut root, read, value(entries(start, end), after.clone()));
            if end < TOTAL {
                read = needs_read(&out, after);
                assert_eq!(root.pages().inbox().unwrap().rows.len(), if filter.project.is_some() { 0 } else { WINDOW });
                assert_eq!(section_reads(&out, contract::InboxReadGroup::Updates).as_ref(), []);
            } else {
                assert_eq!(section_reads(&out, contract::InboxReadGroup::Needs).as_ref(), []);
                assert_eq!(section_reads(&out, contract::InboxReadGroup::Updates).as_ref(), []);
                if filter.project.is_some() {
                    card_reads = queries(&out);
                }
            }
        }
        let inbox = root.pages().inbox().unwrap();
        assert_eq!(inbox.rows, entries(0, TOTAL));
        assert_eq!(inbox.cards.len(), WINDOW);
        assert_eq!(inbox.cut, [false, true]);
        finish_cards(&mut root, &card_reads);
        let needs = if filter.project.is_some() {
            crate::pages::home::facts(&root, PROJECT, Wall::EPOCH).needs
        } else {
            crate::pages::all_home::facts(&root, Wall::EPOCH).needs
        };
        assert_eq!(
            needs,
            crate::pages::home::Needs::Count {
                count: TOTAL,
                cut: false,
                findings: crate::pages::findings::Counted::default()
            }
        );

        let document =
            crate::tests::documents::inbox(entries(30, 38), 12, contract::Count { value: 0, limited: false }, 0, true);
        let out = event(
            &mut root,
            Up::Watched {
                watch,
                event: contract::WatchEvent::Delivered(contract::Delivery::Inbox(Box::new(document))),
            },
        );
        for (read, _) in section_reads(&out, contract::InboxReadGroup::Updates) {
            answer(&mut root, read, value(List::with_capacity(0), None));
        }
        let read = needs_read(&out, None);
        let out = answer(&mut root, read, value(entries(30, 38), Some(cursor(38))));
        assert_eq!(root.pages().inbox().unwrap().rows, entries(0, TOTAL), "complete rows stand during refresh");
        let counted = root.pages().needs().counted(root.pages().inbox().unwrap(), root.forge().approvals());
        assert_eq!(
            counted,
            crate::pages::home::Needs::Count {
                count: TOTAL,
                cut: false,
                findings: crate::pages::findings::Counted::default()
            }
        );
        let read = needs_read(&out, Some(cursor(38)));
        answer(&mut root, read, value(entries(38, 42), None));
        assert_eq!(root.pages().inbox().unwrap().rows, entries(30, 42));
        assert!(!root.pages().inbox().unwrap().cut[0]);
    }
}

fn listed() -> crate::Domain {
    let (mut root, _, out) = begin(Filter::ALL, true);
    let card_reads = queries(&out);
    let mut read = needs_read(&out, None);
    for start in (0..TOTAL).step_by(usize::try_from(WINDOW).unwrap()) {
        let end = start.checked_add(WINDOW).unwrap().min(TOTAL);
        let after = if end < TOTAL { Some(cursor(end)) } else { None };
        let out = answer(&mut root, read, value(entries(start, end), after.clone()));
        if end < TOTAL {
            read = needs_read(&out, after);
        }
    }
    for (index, (read, what)) in card_reads.iter().enumerate() {
        if let contract::Read::Escalation { task } = what {
            let contract::ReadValue::Escalation(mut held) = fixtures::acts::held(contract::ActKind::Release) else {
                panic!("held fixture")
            };
            held.task.chip.task = *task;
            held.task.hold.as_mut().unwrap().hold_wait_since = Some(Wall::from_nanos(u64::try_from(index).unwrap()));
            answer(&mut root, *read, contract::ReadResult::Value(contract::ReadValue::Escalation(held)));
        }
    }
    root
}

#[test]
fn every_open_row_joins_the_read_cards_in_asked_order() {
    use crate::pages::{findings::Need, needs};
    let root = listed();
    let inbox = root.pages().inbox().unwrap();
    assert_eq!(inbox.cards.len(), WINDOW);
    assert_eq!(inbox.rows.len(), TOTAL);
    let all = root.pages().needs().all(inbox, root.forge().approvals());
    assert_eq!(all.len(), usize::try_from(TOTAL).unwrap());
    for (index, need) in all.iter().enumerate() {
        let expected = entries(u32::try_from(index).unwrap(), u32::try_from(index + 1).unwrap());
        let object = match need {
            Need::Existing(needs::Need::Card(card)) => {
                assert!(index < usize::try_from(WINDOW).unwrap());
                card.object
            }
            Need::Row(row) => {
                assert!(index >= usize::try_from(WINDOW).unwrap());
                assert!(!need.ready(&root));
                row.card_key()
            }
            Need::Existing(needs::Need::Approval(_)) | Need::Finding(_) => panic!("inbox needs only"),
        };
        assert_eq!(object, expected.get(0).unwrap().card_key());
    }
    assert_eq!(
        root.pages().needs().counted(inbox, root.forge().approvals()),
        crate::pages::home::Needs::Count {
            count: TOTAL,
            cut: false,
            findings: crate::pages::findings::Counted::default()
        }
    );
}

#[test]
fn opening_an_unread_row_replaces_the_last_needs_read_and_waits_for_its_answer() {
    use crate::pages::{self, findings::Need, needs};
    let mut root = listed();
    let object = ObjectKey::PersonTask(19);
    let last = root.pages().inbox().unwrap().cards.get(WINDOW - 1).unwrap().handle;
    let mut out = Queue::uncapped();
    crate::step(
        &mut root,
        &fixtures::env(&limits()),
        crate::Event::Act { action: crate::Action::Pages(pages::Act::Needs(needs::Act::Open { object })) },
        &mut out,
    );
    let reads = queries(&out);
    assert_eq!(reads.len(), 1);
    assert_eq!(reads[0].1, contract::Read::PersonTask { task: 19 });
    let inbox = root.pages().inbox().unwrap();
    assert_eq!(inbox.cards.len(), WINDOW);
    for card in &inbox.cards {
        assert_ne!(card.handle, last);
    }
    let shown = root.pages().needs().shown_all(inbox, root.forge().approvals()).unwrap();
    let Need::Row(row) = shown else { panic!("closed until its read answers") };
    assert_eq!(row.card_key(), object);
    assert!(!shown.ready(&root));
    let mut out = Queue::uncapped();
    crate::step(
        &mut root,
        &fixtures::env(&limits()),
        crate::Event::Act { action: crate::Action::Pages(pages::Act::Needs(needs::Act::Open { object })) },
        &mut out,
    );
    assert_eq!(queries(&out).len(), 0, "opening reads only once");
    let contract::ReadValue::PersonTask(mut person) = fixtures::acts::person(contract::ActKind::Answer) else {
        panic!("person fixture")
    };
    person.task.task = 19;
    person.since = Some(Wall::from_nanos(19));
    answer(&mut root, reads[0].0, contract::ReadResult::Value(contract::ReadValue::PersonTask(person)));
    let inbox = root.pages().inbox().unwrap();
    let all = root.pages().needs().all(inbox, root.forge().approvals());
    let Need::Existing(needs::Need::Card(card)) = all[19] else { panic!("the 20th is read") };
    assert_eq!(card.object, object);
    assert!(card.page.is_some());
    for card in &inbox.cards {
        assert_ne!(card.object, ObjectKey::Escalation(7));
    }
    assert_eq!(all.len(), usize::try_from(TOTAL).unwrap());
}

#[test]
fn refused_and_failed_pages_keep_the_needs_count_cut() {
    for (filter, refused) in [
        (Filter::ALL, false),
        (Filter::ALL, true),
        (Filter { project: Some(PROJECT), ..Filter::ALL }, false),
        (Filter { project: Some(PROJECT), ..Filter::ALL }, true),
    ] {
        let (mut root, _, out) = begin(filter, true);
        for (read, _) in section_reads(&out, contract::InboxReadGroup::Updates) {
            answer(&mut root, read, value(List::with_capacity(0), None));
        }
        finish_cards(&mut root, &queries(&out));
        let read = needs_read(&out, None);
        let out = answer(&mut root, read, value(entries(0, WINDOW), Some(cursor(WINDOW))));
        let read = needs_read(&out, Some(cursor(WINDOW)));
        let page =
            if refused { Readout::Limit } else { Readout::Jig(contract::ReadResult::None(contract::ReadNone::Busy)) };
        let out = event(&mut root, Up::ReadDone { read, result: page });
        assert_eq!(section_reads(&out, contract::InboxReadGroup::Needs).as_ref(), []);
        finish_cards(&mut root, &queries(&out));
        assert_eq!(root.pages().inbox().unwrap().rows, entries(0, WINDOW));
        assert_eq!(
            root.pages().needs().counted(root.pages().inbox().unwrap(), root.forge().approvals()),
            crate::pages::home::Needs::Count {
                count: WINDOW,
                cut: true,
                findings: crate::pages::findings::Counted::default(),
            }
        );
    }
}

#[test]
fn needs_rows_stop_at_64_and_only_a_complete_read_clears_cut() {
    for total in [64, 65] {
        let (mut root, _, out) = begin(Filter::ALL, true);
        finish_cards(&mut root, &queries(&out));
        let mut read = needs_read(&out, None);
        for start in (0..total).step_by(usize::try_from(WINDOW).unwrap()) {
            let end = (start + WINDOW).min(total);
            let after = if end < total { Some(cursor(end)) } else { None };
            let out = answer(&mut root, read, value(entries(start, end), after.clone()));
            if end == 64 {
                assert_eq!(queries(&out).as_ref(), []);
                break;
            }
            read = needs_read(&out, after);
        }
        assert_eq!(root.pages().inbox().unwrap().rows.len(), 64);
        assert_eq!(root.pages().inbox().unwrap().cut[0], total > 64);
    }
}
