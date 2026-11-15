use crate::tests::fixtures::{Fixture, ReadPage};
pub(in crate::tests) use crate::tests::kit::text;
use crate::tests::state::domain::spine::{LIMITS, delivered, drive, frame, start};
use crate::{
    Event, Request,
    state::{
        action::Act,
        address::{Address, Place, TaskSection},
    },
};
use jig_client::contract::*;
use jig_client::thread::window::End;
use skein_lib::{List, Time, Token, Wall};

const HISTORY_LIMITS: crate::tests::fixtures::Profile =
    crate::tests::fixtures::Profile { client: jig_client::DomainLimits { rows: 3, ..LIMITS.client }, ..LIMITS };

fn change(task: u64, revision: u64) -> HistoryRow {
    HistoryRow::Change(Box::new(HistoryLine {
        task,
        revision,
        by: Requester::Deployment { project: u32::MAX },
        change: HistoryChange::Amended,
        reason: DocumentText { bytes: text(b""), cut: true },
        proposal: None,
        amendment: None,
        at: None,
    }))
}
fn attempt(task: u64, attempt: u64) -> HistoryRow {
    HistoryRow::Attempt(Box::new(AttemptLine {
        task,
        attempt,
        claimed: Some(Wall::EPOCH),
        started: None,
        ended: None,
        end: None,
        phases: None,
    }))
}
fn hold(task: u64, position: u64) -> HistoryRow {
    HistoryRow::Hold(Box::new(HoldLine {
        task,
        position,
        at: Wall::EPOCH,
        before: PriorPhase::Waiting,
        why: HoldCategory::Budget,
        source: None,
        ending: None,
    }))
}
fn rows(values: [HistoryRow; 3]) -> List<HistoryRow> {
    List::from_boxed(Box::new(values))
}

fn history_read(requests: &[Request]) -> jig_client::Handle {
    for request in requests {
        if let Request::Client {
            asked: jig_client::Event::Read { handle: read, read: jig_client::Readable::Jig(Read::History { .. }) },
        } = request
        {
            return *read;
        }
    }
    panic!("original history request");
}
fn page(cut: bool, after: Option<Cursor>) -> ReadValue {
    ReadValue::History(Box::new(HistoryPage {
        cut,
        rows: rows([change(u64::MAX, 0), attempt(u64::MAX, 0), hold(u64::MAX, 0)]),
        before: None,
        after,
    }))
}
#[test]
fn owner_history_original_scope_cursors_and_page_loss_survive_continuation_and_reset() {
    let mut domain = Fixture::new(&HISTORY_LIMITS);
    let watch = start(&mut domain, List::with_capacity(0));
    let initial = delivered(&mut domain, Time::ZERO, watch, frame(7));
    crate::tests::state::domain::pages::finish_initial_inbox_reads(&mut domain, &initial);
    let original = history_read(&drive(
        &mut domain,
        Time::ZERO,
        Event::Went { place: Place::Jig(Address::Task { task: 8, section: TaskSection::History }) },
    ));
    let subtree =
        history_read(&drive(&mut domain, Time::ZERO, Event::Act { action: Act::HistoryScope { subtree: true } }));
    drive(&mut domain, Time::ZERO, ReadPage { read: original, result: ReadResult::Value(page(true, None)) });
    assert!(domain.history_rows().rows.is_empty());
    assert!(!domain.history_rows().cut);
    let cursor = Cursor::new(Box::from(b"whole supplied cursor".as_slice()), 1024).expect("bounded opaque cursor");
    drive(
        &mut domain,
        Time::ZERO,
        ReadPage { read: subtree, result: ReadResult::Value(page(true, Some(cursor.clone()))) },
    );
    assert_eq!(domain.history_rows().rows.as_slice(), &[change(u64::MAX, 0), attempt(u64::MAX, 0), hold(u64::MAX, 0)]);
    assert!(domain.history_rows().cut);
    let more = drive(
        &mut domain,
        Time::ZERO,
        Event::Act {
            action: Act::More { window: Token::new(crate::state::page::task::HISTORY_WINDOW), end: End::After },
        },
    );
    let continued = history_read(&more);
    let mut matching_reads = 0_u32;
    for request in &more {
        if let Request::Client { asked: jig_client::Event::Read { read: jig_client::Readable::Jig(what), .. } } =
            request
            && *what == (Read::History { task: 8, subtree: true, after: Some(cursor.clone()) })
        {
            matching_reads += 1;
        }
    }
    assert_eq!(matching_reads, 1);
    drive(&mut domain, Time::ZERO, ReadPage { read: continued, result: ReadResult::Value(page(false, None)) });
    assert!(domain.history_rows().cut);
    assert_eq!(domain.history_rows().rows.len(), 3);
    drive(&mut domain, Time::ZERO, Event::Act { action: Act::HistoryScope { subtree: false } });
    assert!(domain.history_rows().rows.is_empty());
    assert!(!domain.history_rows().cut);
}

#[test]
fn history_section_renders_changes_attempts_and_holds_with_distinct_keys_in_both_scopes() {
    let mut domain = Fixture::new(&HISTORY_LIMITS);
    let watch = start(&mut domain, List::with_capacity(0));
    let initial = delivered(&mut domain, Time::ZERO, watch, frame(7));
    crate::tests::state::domain::pages::finish_initial_inbox_reads(&mut domain, &initial);
    let own = history_read(&drive(
        &mut domain,
        Time::ZERO,
        Event::Went { place: Place::Jig(Address::Task { task: 8, section: TaskSection::History }) },
    ));
    let mut goldens = crate::tests::kit::Goldens::new("state/domain/history_facts/rows_in_page_order");
    for subtree in [false, true] {
        let read = if subtree {
            history_read(&drive(&mut domain, Time::ZERO, Event::Act { action: Act::HistoryScope { subtree } }))
        } else {
            own
        };
        let cursor = Cursor::new(Box::from(b"next page".as_slice()), 1024).unwrap();
        drive(&mut domain, Time::ZERO, ReadPage { read, result: ReadResult::Value(page(true, Some(cursor))) });
        let mut builder = skein_ui::Builder::new(&crate::tests::kit::limits());
        let mut bindings = crate::JigBindings::new(512, 256);
        assert!(crate::tree::sections::task::section(
            &mut builder,
            &domain.view(),
            crate::tests::kit::words(),
            &mut bindings,
            8,
            TaskSection::History
        ));
        let tree = builder.finish();
        let mut keys = List::with_capacity(3);
        for node in tree.nodes() {
            if let Some(key) = node.key {
                keys.push(key).unwrap();
            }
        }
        assert_eq!(
            keys.as_slice(),
            &[
                skein_ui::Key { owner: crate::Owner::Jig.opaque(), kind: 41, a: u64::MAX, b: 0 },
                skein_ui::Key { owner: crate::Owner::Jig.opaque(), kind: 43, a: u64::MAX, b: 0 },
                skein_ui::Key { owner: crate::Owner::Jig.opaque(), kind: 42, a: u64::MAX, b: 0 },
            ]
        );
        goldens.check(&tree);
    }
}
