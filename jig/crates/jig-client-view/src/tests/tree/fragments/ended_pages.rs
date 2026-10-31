#![expect(
    clippy::disallowed_methods,
    reason = "ordinary proofs inspect complete public trees and checked language data"
)]
use crate::tests::fixtures::{Fixture, ReadPage, Watched};
use crate::tests::kit::Goldens;
use crate::tests::tree::cards::CLIENT;
use crate::words::Words;
use crate::words::{ended_pages_words as sayings, ended_pages_words::EndedPhrase as P};
use crate::{JigBindings, tree::fragments::ended_pages as view};
use skein_ui::Limits;
use skein_ui::phrases::{Invalid, PhraseLimits, Phrasebook, Segment};
use skein_ui::{Builder, Tree};

use jig_client::contract::*;
use skein_lib::{List, Wall, bytes::copy_of};
fn doc(bytes: &[u8], cut: bool) -> DocumentText {
    DocumentText { bytes: Text::new(copy_of(bytes), CLIENT.client.text).unwrap(), cut }
}
fn words() -> Words {
    let mut language = crate::words::phrases::english::language();
    language.wall = skein_ui::phrases::WallFormat::Nanos;
    Words::new(
        Phrasebook::load(
            crate::words::phrases::jig::shapes(CLIENT.client.text),
            Box::new([language]),
            &phrase_limits(),
        )
        .unwrap(),
        0,
        0,
    )
}
fn phrase_limits() -> PhraseLimits {
    PhraseLimits {
        languages: 2,
        phrases: crate::words::document_words::phrase_count(),
        holes: 4,
        segments: crate::tests::english_phrase_segments(2),
        text_bytes: 100_000,
        hole_bytes: 256,
    }
}
fn chip() -> Chip {
    let mut chip = crate::tests::tree::cards_objects::task_doc(&[]).chip;
    chip.task = u64::MAX;
    chip.title = doc(b"", true);
    chip.phase = Phase::Ended;
    let mut metadata = crate::tests::tree::fragments::chip::details();
    metadata.resources = List::with_capacity(CLIENT.client.rows);
    metadata.closing_connectors = Some(List::with_capacity(CLIENT.client.rows));
    *chip.details = ChipPhaseFacts { phase: ChipPhase::Ended, details: Box::new(metadata) };
    chip
}
fn ended(value: ChipEnding, at: Option<Wall>) -> EndedPage {
    let rows = crate::tests::kit::list_in(CLIENT.client.rows, [EndedDelegate { chip: chip(), ended: value, at }]);
    EndedPage { rows, before: None, after: None, cut: true }
}
fn funding(funder: Funder, parent: Option<Funder>, period: Option<PeriodTimes>, closed: bool) -> SpendPage {
    let rows = crate::tests::kit::list_in(
        CLIENT.client.rows,
        [FundingRecord {
            period,
            funder,
            parent,
            numbers: Numbers { budget: 1, spent: 7, spent_below: 11, reserved: u64::MAX },
            made: u32::MAX,
            closed,
        }],
    );
    SpendPage { rows, before: None, after: None, cut: true }
}

fn ended_tree(page: &EndedPage, w: &Words, cap: Option<Limits>) -> Tree {
    let f = crate::tests::render_limits(&CLIENT, w).unwrap();
    let cap = cap.unwrap_or(f);
    let mut b = Builder::new(&cap);
    let mut bindings = JigBindings::new(cap.bindings, CLIENT.client.text);
    view::ended_render(&mut b, &Fixture::new(&CLIENT).view(), page, w, &mut bindings);
    b.finish()
}
fn spend_tree(page: &SpendPage, w: &Words, cap: Option<Limits>) -> Tree {
    let f = crate::tests::render_limits(&CLIENT, w).unwrap();
    let mut b = Builder::new(&cap.unwrap_or(f));
    view::spend_render(&mut b, page, w);
    b.finish()
}

#[test]
fn ended_ended_view_keeps_all_owner_causes_requesters_optional_times_loss_and_empty_title_identity() {
    let mut goldens = Goldens::new("tree/fragments/ended_pages/ended_ended_view");
    let w = words();
    for by in [Requester::Task(u64::MAX), Requester::Person(u64::MAX), Requester::Deployment { project: u32::MAX }] {
        for at in [None, Some(Wall::EPOCH), Some(Wall::from_nanos(u64::MAX))] {
            for ending in [
                ChipEnding::Done(ResultValue::Words(doc(b"actual result", true))),
                ChipEnding::Failed(doc(b"actual failure", true)),
                ChipEnding::Cancelled {
                    reason: doc(b"actual cancellation", true),
                    result: Some(ResultValue::Words(doc(b"actual result", true))),
                    by,
                    at: Wall::from_nanos(u64::MAX),
                },
            ] {
                let page = ended(ending.clone(), at);
                let untouched = page.clone();
                let tree = ended_tree(&page, &w, None);
                crate::tests::tree::cards_access::check(&tree);
                goldens.check(&tree);
                assert!(tree.nodes().iter().all(|n| n.key.is_none()));
                assert_eq!(page, untouched);
                assert_eq!(page.rows.capacity(), CLIENT.client.rows);
            }
        }
    }
    let without = ended(
        ChipEnding::Cancelled {
            reason: doc(b"reason without result", false),
            result: None,
            by: Requester::Person(9),
            at: Wall::EPOCH,
        },
        None,
    );
    let tree = ended_tree(&without, &w, None);
    goldens.check(&tree);
}
#[test]
fn ended_funding_view_preserves_all_source_parent_coordinates_four_counters_period_absence_and_closed_state() {
    let mut goldens = Goldens::new("tree/fragments/ended_pages/ended_funding_view_preserves");
    let w = words();
    let kinds = [
        Funder::Task(u64::MAX),
        Funder::Pool { project: u32::MAX, person: u64::MAX, period: u64::MAX },
        Funder::Recurring { project: u32::MAX, task: u64::MAX, period: u64::MAX },
        Funder::Period { project: u32::MAX, period: u64::MAX },
    ];
    for funder in kinds {
        for parent in [None, Some(kinds[0]), Some(kinds[1]), Some(kinds[2]), Some(kinds[3])] {
            for period in [
                None,
                Some(PeriodTimes { began: Wall::EPOCH, next: None }),
                Some(PeriodTimes { began: Wall::EPOCH, next: Some(Wall::EPOCH) }),
                Some(PeriodTimes { began: Wall::from_nanos(u64::MAX), next: Some(Wall::from_nanos(u64::MAX)) }),
            ] {
                for closed in [false, true] {
                    let page = funding(funder, parent, period, closed);
                    let original = page.clone();
                    let tree = spend_tree(&page, &w, None);
                    crate::tests::tree::cards_access::check(&tree);
                    goldens.check(&tree);
                    assert!(tree.nodes().iter().all(|n| n.key.is_none()));
                    assert_eq!(page, original);
                }
            }
        }
    }
}
#[test]
fn ended_checked_selected_language_has_no_fallback_and_rejects_a_missing_number_counter_hole() {
    let mut goldens = Goldens::new("tree/fragments/ended_pages/ended_checked_selected");
    let english = crate::words::phrases::english::language();
    let mut selected = english.clone();
    for phrase in &mut selected.phrases {
        if (9500..9600).contains(&phrase.id.number) {
            for form in &mut phrase.forms {
                for segment in &mut form.segments {
                    if let Segment::Text(bytes) = segment {
                        *bytes = copy_of(b"selected history ");
                    }
                }
            }
        }
    }
    let w = Words::new(
        Phrasebook::load(
            crate::words::phrases::jig::shapes(CLIENT.client.text),
            Box::new([english, selected]),
            &phrase_limits(),
        )
        .unwrap(),
        1,
        0,
    );
    let page = ended(ChipEnding::Failed(doc(b"source failure", true)), None);
    let tree = ended_tree(&page, &w, None);
    goldens.check(&tree);
    let page = funding(Funder::Period { project: 1, period: 9 }, None, None, false);
    let tree = spend_tree(&page, &w, None);
    goldens.check(&tree);
    let mut broken = sayings::language();
    for phrase in &mut broken.phrases {
        if phrase.id == P::Made.id() {
            phrase.forms[0].segments = Box::new([Segment::Text(copy_of(b"missing number"))]);
        }
    }
    assert_eq!(
        Phrasebook::load(sayings::shapes(), Box::new([broken]), &phrase_limits()).err(),
        Some(Invalid::Holes { id: P::Made.id() })
    );
}
fn exact_limits(tree: &Tree, maximum: Limits) -> Limits {
    let depth = tree
        .nodes()
        .iter()
        .enumerate()
        .map(|(at, _)| {
            1usize
                .checked_add(
                    tree.nodes()
                        .iter()
                        .enumerate()
                        .take(at)
                        .filter(|(index, node)| index.checked_add(usize::try_from(node.size).unwrap()).unwrap() > at)
                        .count(),
                )
                .unwrap()
        })
        .max()
        .unwrap();
    let bindings = tree.nodes().iter().filter(|n| n.binding.is_some() || n.link.is_some() || n.slot.is_some()).count();
    Limits {
        nodes: tree.len(),
        depth: u32::try_from(depth).unwrap(),
        text_per_node: tree.nodes().iter().map(crate::tests::text_bytes).max().unwrap(),
        text_bytes: tree.nodes().iter().map(crate::tests::text_bytes).sum(),
        bindings: u32::try_from(bindings).unwrap(),
        slots: u32::try_from(tree.nodes().iter().filter(|n| n.slot.is_some()).count()).unwrap(),
        ..maximum
    }
}
#[test]
fn ended_ended_view_exact_node_depth_text_binding_and_slot_storage_refuses_each_one_under_whole() {
    let w = words();
    let page = ended(
        ChipEnding::Done(ResultValue::Connector {
            summary: doc(b"whole connector result", true),
            part: ApplicationPart::Result { connector: u16::MAX, task: u64::MAX },
        }),
        Some(Wall::EPOCH),
    );
    let original = page.clone();
    let f = crate::tests::render_limits(&CLIENT, &w).unwrap();
    let tree = ended_tree(&page, &w, None);
    let exact = exact_limits(&tree, f);
    assert!(exact.bindings > 0 && exact.slots > 0);
    assert_eq!(ended_tree(&page, &w, Some(exact)).len(), tree.len());
    for below in [
        Limits { nodes: exact.nodes - 1, ..exact },
        Limits { depth: exact.depth - 1, ..exact },
        Limits { text_per_node: exact.text_per_node - 1, ..exact },
        Limits { text_bytes: exact.text_bytes - 1, ..exact },
        Limits { bindings: exact.bindings - 1, ..exact },
        Limits { slots: exact.slots - 1, ..exact },
    ] {
        std::panic::catch_unwind(core::panic::AssertUnwindSafe(|| ended_tree(&page, &w, Some(below))))
            .expect_err("each one-under whole view storage refuses");
    }
    assert_eq!(page, original);
    assert!(
        view::ended_nodes(
            &crate::tests::fixtures::Profile {
                view: crate::Config { ..(CLIENT).view },
                client: jig_client::DomainLimits { rows: u32::MAX, ..(CLIENT).client }
            },
            &(crate::tests::fixtures::Profile {
                view: crate::Config { ..(CLIENT).view },
                client: jig_client::DomainLimits { rows: u32::MAX, ..(CLIENT).client }
            })
            .client
        )
        .is_none()
    );
}
#[test]
fn ended_funding_view_exact_geometry_and_reserved_empty_rows_remain_independent_of_scalar_values() {
    let mut goldens = Goldens::new("tree/fragments/ended_pages/ended_funding_view_exact");
    let w = words();
    let page = funding(
        Funder::Period { project: u32::MAX, period: u64::MAX },
        Some(Funder::Task(u64::MAX)),
        Some(PeriodTimes { began: Wall::EPOCH, next: Some(Wall::from_nanos(u64::MAX)) }),
        true,
    );
    let f = crate::tests::render_limits(&CLIENT, &w).unwrap();
    let tree = spend_tree(&page, &w, None);
    let exact = exact_limits(&tree, f);
    assert_eq!(exact.bindings, 0);
    assert_eq!(exact.slots, 0);
    assert_eq!(spend_tree(&page, &w, Some(exact)).len(), tree.len());
    for below in [
        Limits { nodes: exact.nodes - 1, ..exact },
        Limits { depth: exact.depth - 1, ..exact },
        Limits { text_per_node: exact.text_per_node - 1, ..exact },
        Limits { text_bytes: exact.text_bytes - 1, ..exact },
    ] {
        std::panic::catch_unwind(|| spend_tree(&page, &w, Some(below)))
            .expect_err("one-under ended scalar tree refuses");
    }
    let mut reserved = page.clone();
    reserved.rows = List::with_capacity(CLIENT.client.rows);
    let tree = spend_tree(&reserved, &w, None);
    assert_eq!(tree.len(), 1);
    goldens.check(&tree);
}

#[test]
#[expect(
    clippy::wildcard_enum_match_arm,
    reason = "ordinary public journey selects the original Task, Tree and Ended requests"
)]
fn ended_public_plan_keeps_owner_tree_early_return_and_companion_ended_read_reset() {
    use crate::tests::tree::cards::{document, drive, start};
    use crate::{
        Event, Request,
        state::{
            action::Act,
            address::{Address, Place, TaskSection},
        },
    };
    let mut goldens = Goldens::new("tree/fragments/ended_pages/ended_public_plan");
    let (mut client, frame) = start();
    drive(&mut client, Watched { watch: frame, event: WatchEvent::Delivered(Delivery::Me(Box::new(document()))) });
    let out =
        drive(&mut client, Event::Went { place: Place::Jig(Address::Task { task: 8, section: TaskSection::Plan }) });
    let task =
        out.iter()
            .find_map(|r| match r {
                Request::Client {
                    asked:
                        jig_client::Event::Watch {
                            handle: watch,
                            subject: jig_client::Watchable::Jig(Subject::Task { .. }),
                        },
                } => Some(*watch),
                _ => None,
            })
            .unwrap();
    let out = drive(
        &mut client,
        Watched {
            watch: task,
            event: WatchEvent::Delivered(Delivery::Task(Box::new(crate::tests::tree::cards_objects::task_doc(&[])))),
        },
    );
    let watch =
        out.iter()
            .find_map(|r| match r {
                Request::Client {
                    asked:
                        jig_client::Event::Watch {
                            handle: watch,
                            subject: jig_client::Watchable::Jig(Subject::Tree { .. }),
                        },
                } => Some(*watch),
                _ => None,
            })
            .unwrap();
    let tree = jig_client::contract::Tree {
        root: crate::tests::tree::cards_objects::task_doc(&[]).chip,
        rows: List::with_capacity(CLIENT.client.rows),
        folded: List::with_capacity(CLIENT.client.rows),
        held_below: 0,
        next: None,
    };
    drive(&mut client, Watched { watch, event: WatchEvent::Delivered(Delivery::Tree(Box::new(tree.clone()))) });
    let out = drive(&mut client, Event::Act { action: Act::OpenEnded { task: 8 } });
    let read = out
        .iter()
        .find_map(|r| match r {
            Request::Client {
                asked:
                    jig_client::Event::Read {
                        handle: read,
                        read: jig_client::Readable::Jig(Read::Ended { task: 8, after: None }),
                    },
            } => Some(*read),
            _ => None,
        })
        .unwrap();
    let page = ended(ChipEnding::Failed(doc(b"companion actual cause", true)), None);
    drive(&mut client, ReadPage { read, result: ReadResult::Value(ReadValue::EndedPage(Box::new(page.clone()))) });
    let w = words();
    let cap = crate::tests::render_limits(&CLIENT, &w).unwrap();
    let mut b = Builder::new(&cap);
    let mut bindings = JigBindings::new(cap.bindings, CLIENT.client.text);
    crate::tree::pages::main(&mut b, &client.view(), &w, &mut bindings);
    let rendered = b.finish();
    goldens.check(&rendered);
    assert_eq!(client.task_page().plan.owner, Some(&tree));
    assert_eq!(client.task_page().owner_ended.as_deref(), Some(&page));
    drive(&mut client, Event::Act { action: Act::OpenEnded { task: 8 } });
    assert!(client.task_page().owner_ended.is_none());
    assert_eq!(client.task_page().plan.owner, Some(&tree));
    assert!(!client.task_page().ended_cut);
    assert!(client.task_page().ended_before.is_none() && client.task_page().ended_after.is_none());
}
