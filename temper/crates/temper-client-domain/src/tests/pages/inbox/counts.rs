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
