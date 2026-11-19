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
                assert_eq!(root.pages().inbox().unwrap().rows.len(), 0);
                assert_eq!(section_reads(&out, contract::InboxReadGroup::Updates).as_ref(), []);
            } else {
                assert_eq!(section_reads(&out, contract::InboxReadGroup::Needs).as_ref(), []);
                assert_eq!(section_reads(&out, contract::InboxReadGroup::Updates).as_ref(), []);
                card_reads = queries(&out);
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
    let mut card_reads = queries(&out);
    let mut read = needs_read(&out, None);
    for start in (0..TOTAL).step_by(usize::try_from(WINDOW).unwrap()) {
        let end = start.checked_add(WINDOW).unwrap().min(TOTAL);
        let after = if end < TOTAL { Some(cursor(end)) } else { None };
        let out = answer(&mut root, read, value(entries(start, end), after.clone()));
        if end < TOTAL {
            read = needs_read(&out, after);
        } else {
            card_reads = queries(&out);
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
fn stopped_needs_chains_are_no_answer() {
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
        let out = answer(&mut root, read, value(entries(0, 2), Some(cursor(2))));
        let read = needs_read(&out, Some(cursor(2)));
        let page =
            if refused { Readout::Limit } else { Readout::Jig(contract::ReadResult::None(contract::ReadNone::Busy)) };
        let out = event(&mut root, Up::ReadDone { read, result: page });
        assert_eq!(section_reads(&out, contract::InboxReadGroup::Needs).as_ref(), []);
        finish_cards(&mut root, &queries(&out));
        let inbox = root.pages().inbox().unwrap();
        assert!(inbox.rows.is_empty());
        assert!(inbox.cards.is_empty());
        assert!(!inbox.answered(contract::InboxReadGroup::Needs));
        assert!(!inbox.known());
        assert!(!inbox.cut[0]);
        assert_eq!(root.pages().needs().counted(inbox, root.forge().approvals()), crate::pages::home::Needs::Unknown);
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
                assert!(section_reads(&out, contract::InboxReadGroup::Needs).is_empty());
                break;
            }
            read = needs_read(&out, after);
        }
        assert_eq!(root.pages().inbox().unwrap().rows.len(), 64);
        assert_eq!(root.pages().inbox().unwrap().cut[0], total > 64);
    }
}

fn refresh(root: &mut crate::Domain, watch: Token, rows: List<contract::InboxEntry>, more: bool) -> Queue<Request> {
    let document = crate::tests::documents::inbox(rows, 4, contract::Count { value: 0, limited: false }, 0, more);
    event(
        root,
        Up::Watched { watch, event: contract::WatchEvent::Delivered(contract::Delivery::Inbox(Box::new(document))) },
    )
}
#[test]
fn an_updates_tail_does_not_cut_or_reread_the_needs_window() {
    let (mut root, watch, _) = begin(Filter::ALL, false);
    let mut rows = entries(0, 5);
    *rows.get_mut(4).unwrap() = fixtures::inbox_entry(ObjectKey::Result(9), InboxGroup::Results, PROJECT, 5);
    let out = refresh(&mut root, watch, rows, true);
    assert!(section_reads(&out, contract::InboxReadGroup::Needs).is_empty());
    assert_eq!(root.pages().inbox().unwrap().rows, entries(0, 4));
    assert_eq!(count(&root), four(false));
    assert_eq!(root.pages().inbox().unwrap().cut, [false, true]);
}
fn count(root: &crate::Domain) -> crate::pages::home::Needs {
    root.pages().needs().counted(root.pages().inbox().unwrap(), root.forge().approvals())
}
fn four(cut: bool) -> crate::pages::home::Needs {
    crate::pages::home::Needs::Count { count: 4, cut, findings: crate::pages::findings::Counted::default() }
}
fn told_at(root: &mut crate::Domain, seconds: u64, told: jig_client::Told) -> Queue<Request> {
    let mut env = fixtures::env(&limits());
    env.now = skein_lib::Time::ZERO.saturating_add(skein_lib::Duration::from_secs(seconds));
    let mut out = Queue::uncapped();
    crate::route::told(root, &env, told, &mut out);
    out
}
fn fire_at(root: &mut crate::Domain, seconds: u64) -> Queue<crate::pages::Request> {
    let mut env = fixtures::env(&limits());
    env.now = skein_lib::Time::ZERO.saturating_add(skein_lib::Duration::from_secs(seconds));
    let mut out = Queue::uncapped();
    crate::pages::fire(&mut root.pages, &env, &root.client, &mut out);
    out
}
fn needs_asks(out: &Queue<crate::pages::Request>) -> Box<[contract::Read]> {
    let mut found = List::with_capacity(out.len());
    for request in out {
        if let crate::pages::Request::Client(jig_client::Event::Read {
            read: Readable::Jig(read @ contract::Read::Inbox { group: contract::InboxReadGroup::Needs, .. }),
            ..
        }) = request
        {
            found.push(read.clone()).unwrap();
        }
    }
    found.into_boxed()
}

#[test]
fn refresh_without_a_value_keeps_whole_rows_cards_cut_and_rail() {
    for filter in [Filter::ALL, Filter { project: Some(PROJECT), ..Filter::ALL }] {
        for failed in [
            Some(Readout::Jig(contract::ReadResult::None(contract::ReadNone::Busy))),
            Some(Readout::Jig(contract::ReadResult::None(contract::ReadNone::Unreachable))),
            Some(Readout::Limit),
            Some(Readout::Jig(contract::ReadResult::None(contract::ReadNone::Refused))),
            None,
        ] {
            for cut in [false, true] {
                let (mut root, watch, _) = begin(filter, false);
                let out = refresh(&mut root, watch, entries(0, 4), cut);
                let mut cards = queries(&out);
                if filter.project.is_some() || cut {
                    let read = needs_read(&out, None);
                    cards = queries(&answer(&mut root, read, cut_value(entries(0, 4), None, cut)));
                    if filter.project.is_some() {
                        let updates = section_reads(&out, contract::InboxReadGroup::Updates);
                        cards = queries(&answer(&mut root, updates[0].0, value(List::with_capacity(0), None)));
                    }
                }
                for (read, what) in &cards {
                    if let contract::Read::Escalation { .. } = what {
                        answer(
                            &mut root,
                            *read,
                            contract::ReadResult::Value(fixtures::acts::held(contract::ActKind::Release)),
                        );
                    }
                }
                assert!(root.pages().inbox().unwrap().known());
                assert_eq!(count(&root), four(cut));
                let before = super::objects(&root);
                // Include a partial chain on the refresh: none of it replaces the whole answer.
                let out = refresh(&mut root, watch, entries(4, 12), true);
                let read = needs_read(&out, None);
                let more = answer(&mut root, read, value(entries(4, 6), Some(cursor(6))));
                let _: Token = needs_read(&more, Some(cursor(6)));
                let handle = root.pages().inbox().unwrap().queries[0];
                let told = match failed.clone() {
                    Some(page) => jig_client::Told::Page { handle, page },
                    None => jig_client::Told::Ended { handle, why: jig_client::Why::SubjectEnded },
                };
                told_at(&mut root, 10, told);
                for (read, _) in section_reads(&out, contract::InboxReadGroup::Updates) {
                    answer(&mut root, read, value(List::with_capacity(0), None));
                }
                assert_eq!(root.pages().inbox().unwrap().rows, entries(0, 4));
                assert_eq!(super::objects(&root), before);
                for card in &root.pages().inbox().unwrap().cards {
                    assert_eq!(card.page, Some(fixtures::acts::held(contract::ActKind::Release)));
                }
                assert_eq!(count(&root), four(cut));
                if filter.project.is_some() {
                    let facts = crate::pages::rail::facts(&root, PROJECT, Wall::EPOCH);
                    assert_eq!((facts.needs, facts.needs_cut), (4, cut));
                }
                // A later whole answer with the same rows keeps the same count.
                let out = refresh(&mut root, watch, entries(0, 4), true);
                let read = needs_read(&out, None);
                answer(&mut root, read, cut_value(entries(0, 4), None, cut));
                for (read, _) in section_reads(&out, contract::InboxReadGroup::Updates) {
                    answer(&mut root, read, value(List::with_capacity(0), None));
                }
                assert_eq!(count(&root), four(cut));
            }
        }
    }
}
fn cut_value(rows: List<contract::InboxEntry>, after: Option<contract::Cursor>, cut: bool) -> contract::ReadResult {
    let mut result = value(rows, after);
    if let contract::ReadResult::Value(contract::ReadValue::Inbox(page)) = &mut result {
        page.cut = cut;
    }
    result
}
#[test]
fn any_needs_page_can_mark_the_whole_list_cut() {
    for cut_first in [false, true] {
        let (mut root, _, out) = begin(Filter::ALL, true);
        let read = needs_read(&out, None);
        let out = answer(&mut root, read, cut_value(entries(0, 2), Some(cursor(2)), cut_first));
        let read = needs_read(&out, Some(cursor(2)));
        answer(&mut root, read, value(entries(2, 4), None));
        assert_eq!(root.pages().inbox().unwrap().rows, entries(0, 4));
        assert_eq!(count(&root), four(cut_first));
    }
}
#[test]
fn needs_retry_from_first_page_backs_off_resets_and_stops_on_leave() {
    for failure in [
        Readout::Limit,
        Readout::Jig(contract::ReadResult::None(contract::ReadNone::Busy)),
        Readout::Jig(contract::ReadResult::None(contract::ReadNone::Unreachable)),
    ] {
        let filter = Filter { project: Some(PROJECT), ..Filter::ALL };
        let (mut root, watch, out) = begin(filter, true);
        let read = needs_read(&out, None);
        let out = answer(&mut root, read, value(entries(0, 2), Some(cursor(2))));
        let _: Token = needs_read(&out, Some(cursor(2)));
        let handle = root.pages().inbox().unwrap().queries[0];
        let expected = contract::Read::Inbox {
            group: contract::InboxReadGroup::Needs,
            project: Some(PROJECT),
            kind: None,
            goal: None,
            after: None,
        };
        let mut at = 10;
        for delay in [1, 2, 4, 8, 16, 30, 30] {
            told_at(&mut root, at, jig_client::Told::Page { handle, page: failure.clone() });
            let due = at + delay;
            assert_eq!(
                crate::pages::next_deadline(root.pages()),
                Some(skein_lib::Time::ZERO.saturating_add(skein_lib::Duration::from_secs(due)))
            );
            assert!(needs_asks(&fire_at(&mut root, due - 1)).is_empty());
            assert_eq!(needs_asks(&fire_at(&mut root, due)).as_ref(), core::slice::from_ref(&expected));
            assert!(needs_asks(&fire_at(&mut root, due)).is_empty());
            at = due;
        }
        told_at(&mut root, at, jig_client::Told::Page { handle, page: Readout::Jig(value(entries(0, 4), None)) });
        assert_eq!(crate::pages::next_deadline(root.pages()), None);
        refresh(&mut root, watch, entries(0, 4), true);
        told_at(&mut root, at, jig_client::Told::Page { handle, page: failure });
        assert_eq!(needs_asks(&fire_at(&mut root, at + 1)).as_ref(), core::slice::from_ref(&expected));
        told_at(&mut root, at + 1, jig_client::Told::Page { handle, page: Readout::Limit });
        crate::pages::step(
            &mut root.pages,
            &fixtures::env(&limits()),
            &root.client,
            crate::pages::Event::Went { address: crate::Address::Settings { project: PROJECT } },
            &mut Queue::uncapped(),
        );
        assert!(needs_asks(&fire_at(&mut root, at + 100)).is_empty());
    }
}
#[test]
fn lasting_needs_ends_wait_for_refresh_and_refresh_cancels_retry() {
    let filter = Filter { project: Some(PROJECT), ..Filter::ALL };
    for ended in [false, true] {
        let (mut root, watch, _) = begin(filter, true);
        let handle = root.pages().inbox().unwrap().queries[0];
        told_at(&mut root, 10, jig_client::Told::Page { handle, page: Readout::Limit });
        refresh(&mut root, watch, entries(0, 4), true);
        assert_eq!(crate::pages::next_deadline(root.pages()), None);
        let told = if ended {
            jig_client::Told::Ended { handle, why: jig_client::Why::SubjectEnded }
        } else {
            jig_client::Told::Page {
                handle,
                page: Readout::Jig(contract::ReadResult::None(contract::ReadNone::Refused)),
            }
        };
        told_at(&mut root, 10, told);
        assert_eq!(crate::pages::next_deadline(root.pages()), None);
        assert!(needs_asks(&fire_at(&mut root, 100)).is_empty());
        let out = refresh(&mut root, watch, entries(0, 4), true);
        let _: Token = needs_read(&out, None);
    }
}
