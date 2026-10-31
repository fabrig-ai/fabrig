#![expect(
    clippy::disallowed_methods,
    clippy::wildcard_enum_match_arm,
    reason = "ordinary public proofs inspect checked trees and scripted requests"
)]
use crate::JigBindings;
use crate::tests::fixtures::{Fixture, ReadPage, Watched};
use crate::tests::kit::Goldens;
use crate::tests::tree::cards::{CLIENT, document, drive, start};
use crate::words::Words;
use skein_ui::phrases::{Hole, Invalid, PhraseLimits, Phrasebook, Segment};
use skein_ui::{Builder, Role, Tree};

use crate::Request;
use jig_client::contract::*;
use skein_lib::{List, bytes::copy_of};
fn words(selected: bool) -> Words {
    let english = crate::words::phrases::english::language();
    let mut second = english.clone();
    for phrase in &mut second.phrases {
        for form in &mut phrase.forms {
            for segment in &mut form.segments {
                if let Segment::Text(bytes) = segment {
                    *bytes = copy_of(b"second ");
                }
            }
        }
    }
    Words::new(
        Phrasebook::load(
            crate::words::phrases::jig::shapes(CLIENT.client.text),
            Box::new([english, second]),
            &PhraseLimits {
                languages: 2,
                phrases: crate::words::document_words::phrase_count(),
                holes: 4,
                segments: crate::tests::english_phrase_segments(2),
                text_bytes: 100_000,
                hole_bytes: CLIENT.client.text,
            },
        )
        .unwrap(),
        u16::from(selected),
        0,
    )
}
fn row(object: InboxObject) -> InboxEntry {
    InboxEntry {
        reply: None,
        task: TaskLink::Stored { number: u64::MAX, status: None },
        project: None,
        object,
        activity: None,
        at: None,
        unread: None,
        cursor: InboxCursor::Number { number: 0 },
    }
}
fn whole(rows: impl IntoIterator<Item = InboxEntry>, limited: bool) -> Inbox {
    let mut needs = List::with_capacity(CLIENT.client.rows);
    let mut updates = List::with_capacity(CLIENT.client.rows);
    for row in rows {
        if crate::state::page::inbox::needs(row.group()) {
            needs.push(row).unwrap();
        } else {
            updates.push(row).unwrap();
        }
    }
    Inbox {
        party: 1,
        needs,
        updates,
        next: None,
        cached_needs: u32::MAX,
        cached_updates: Count { value: 17, limited },
        results_seen: u64::MAX,
        updates_time_complete: false,
        selection: None,
    }
}
fn domain(value: Inbox) -> Fixture {
    let (mut client, frame) = start();
    let out =
        drive(&mut client, Watched { watch: frame, event: WatchEvent::Delivered(Delivery::Me(Box::new(document()))) });
    let watch =
        out.iter()
            .find_map(|r| match r {
                Request::Client {
                    asked:
                        jig_client::Event::Watch {
                            handle: watch,
                            subject: jig_client::Watchable::Jig(Subject::Inbox { .. }),
                        },
                } => Some(*watch),
                _ => None,
            })
            .unwrap();
    drive(&mut client, Watched { watch, event: WatchEvent::Delivered(Delivery::Inbox(Box::new(value))) });
    client
}
fn build(client: &Fixture, w: &Words) -> Tree {
    let mut limits = crate::sizes::limits::of_words(&CLIENT, &(CLIENT).client, w).unwrap();
    limits.nodes = crate::inbox_nodes(&CLIENT, &(CLIENT).client, 0).unwrap().checked_add(1).unwrap();
    limits.bindings = limits.nodes;
    limits.slots = limits.nodes;
    limits.text_bytes = limits.nodes.checked_mul(limits.text_per_node).unwrap();
    let mut b = Builder::new(&limits);
    let mut bindings = JigBindings::new(limits.bindings, CLIENT.client.text);
    b.open(Role::Page);
    crate::tree::pages::main(&mut b, &client.view(), w, &mut bindings);
    b.close();
    let tree = b.finish();
    assert!(tree.len() <= limits.nodes);
    let mut total = 0_u64;
    for node in tree.nodes() {
        assert!(crate::tests::text_bytes(node) <= limits.text_per_node);
        total = total.checked_add(u64::from(crate::tests::text_bytes(node))).unwrap();
    }
    assert!(total <= u64::from(limits.text_bytes));
    tree
}
#[test]
fn owner_inbox_public_six_objects_equal_makers_full_width_keys_and_missing_facts_are_checked() {
    let mut goldens = Goldens::new("tree/sections/inbox/owner_inbox_public");
    let w = words(false);
    let cases = [
        (InboxObject::Question { message: u64::MAX }, 50),
        (InboxObject::Proposal { proposer: Proposer::Task(u64::MAX), number: u64::MAX }, 51),
        (InboxObject::Proposal { proposer: Proposer::Party(u64::MAX), number: u64::MAX }, 52),
        (InboxObject::Escalation { revision: u64::MAX }, 53),
        (InboxObject::PersonTask, 54),
        (InboxObject::Reply { message: u64::MAX }, 55),
        (InboxObject::Result { position: u64::MAX }, 56),
    ];
    for (object, kind) in cases {
        let value = whole([row(object)], false);
        let unchanged = value.clone();
        let client = domain(value);
        let tree = build(&client, &w);
        goldens.check(&tree);
        assert!(tree.nodes().iter().any(|node| if let Some(key) = node.key {
            key.kind == kind && key.a == u64::MAX
        } else {
            false
        }));
        let Some(InboxDocument::Owner(source)) = &client.inbox().document else {
            panic!("whole owner envelope");
        };
        assert_eq!(**source, unchanged);
    }
}
#[test]
fn owner_inbox_second_checked_language_rejects_wrong_holes_without_fallback() {
    let mut goldens = Goldens::new("tree/sections/inbox/owner_inbox_second");
    let w = words(true);
    let mut value = row(InboxObject::Result { position: u64::MAX });
    value.project = Some(0);
    value.activity = Some(0);
    value.unread = Some(true);
    let client = domain(whole([value], true));
    let tree = build(&client, &w);
    goldens.check(&tree);
    let id = crate::words::inbox_words::InboxPhrase::Result.id();
    assert_eq!(w.book().say(1, id, &[Hole::Number(u64::MAX)]).unwrap().text.as_ref(), b"second 18446744073709551615");
    assert_eq!(w.book().say(1, id, &[]).err(), Some(Invalid::Values { id }));
    assert!(w.book().capacity(id).unwrap() >= u32::try_from(b"second 18446744073709551615".len()).unwrap());
}
#[test]
fn owner_inbox_both_full_groups_two_read_windows_counts_source_cuts_fit_the_whole_page_bound() {
    let mut goldens = Goldens::new("tree/sections/inbox/owner_inbox_both");
    let w = words(false);
    let values = [
        row(InboxObject::Question { message: 1 }),
        row(InboxObject::Escalation { revision: 2 }),
        row(InboxObject::Reply { message: 3 }),
        row(InboxObject::Result { position: 4 }),
    ];
    let mut client = domain(whole(values, true));
    let needs = client.inbox().needs_window.reading.unwrap();
    let updates = client.inbox().updates_window.reading.unwrap();
    for (token, entries, after) in [
        (needs, [row(InboxObject::Question { message: 5 }), row(InboxObject::PersonTask)], &[255, 0][..]),
        (updates, [row(InboxObject::Reply { message: 6 }), row(InboxObject::Result { position: 7 })], &[254, 0][..]),
    ] {
        let mut rows = List::with_capacity(CLIENT.client.rows);
        for entry in entries {
            rows.push(entry).unwrap();
        }
        drive(
            &mut client,
            ReadPage {
                read: crate::state::handles::handle(token),
                result: ReadResult::Value(ReadValue::Inbox(Box::new(InboxPage {
                    selection: None,
                    cut: true,
                    rows,
                    before: Some(Cursor::new(copy_of(after), CLIENT.client.text).unwrap()),
                    after: Some(Cursor::new(copy_of(after), CLIENT.client.text).unwrap()),
                }))),
            },
        );
    }
    let tree = build(&client, &w);
    assert_eq!(client.inbox().rows.len(), 8);
    assert!(client.inbox().needs_window.cut);
    assert!(client.inbox().updates_window.cut);
    assert_ne!(client.inbox().needs_window.after, client.inbox().updates_window.after);
    goldens.check(&tree);
    assert!(
        crate::inbox_nodes(
            &crate::tests::fixtures::Profile {
                view: crate::Config { ..(CLIENT).view },
                client: jig_client::DomainLimits { rows: u32::MAX, ..(CLIENT).client }
            },
            &(crate::tests::fixtures::Profile {
                view: crate::Config { ..(CLIENT).view },
                client: jig_client::DomainLimits { rows: u32::MAX, ..(CLIENT).client }
            })
            .client,
            0
        )
        .is_none()
    );
}
