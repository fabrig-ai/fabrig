use crate::tests::fixtures::Fixture;
use crate::tests::kit::Goldens;
use crate::words::Words;
use crate::{JigBindings, tree::fragments::lifecycle as view};
use skein_ui::Limits;
use skein_ui::phrases::{Hole, Invalid, PhraseLimits, Phrasebook, SpanUnit, WallFormat};
use skein_ui::{Builder, Tree};

use jig_client::contract::*;
use skein_lib::{List, Wall, bytes::copy_of};
fn doc(bytes: &[u8], cut: bool) -> DocumentText {
    DocumentText { bytes: Text::new(copy_of(bytes), 8).unwrap(), cut }
}
fn part(capacity: u32) -> Part {
    let mut segments = List::with_capacity(capacity);
    if capacity >= 3 {
        for bytes in [b"a/b".as_slice(), &[255, 0], b""] {
            segments.push(copy_of(bytes)).unwrap();
        }
    }
    Part { connector: u16::MAX, kind: PartKind::Resource, key: PartKey::Path(segments) }
}
fn with_ending(ending: ChipEnding) -> HoldLine {
    HoldLine {
        task: u64::MAX,
        position: u64::MAX,
        at: Wall::EPOCH,
        before: PriorPhase::Closing(ClosingStage::Settled),
        why: HoldCategory::Budget,
        source: None,
        ending: Some(ending),
    }
}
fn limits() -> crate::tests::fixtures::Profile {
    crate::tests::fixtures::Profile {
        view: crate::Config { ..(crate::tests::tree::cards::CLIENT).view },
        client: jig_client::DomainLimits {
            rows: 3,
            segments: 3,
            text: 8,
            ..(crate::tests::tree::cards::CLIENT).client
        },
    }
}
fn words(selected: bool) -> Words {
    let mut english = crate::words::phrases::english::language();
    english.wall = WallFormat::Nanos;
    english.span_unit = SpanUnit::Nanos;
    let languages: Box<[skein_ui::phrases::Language]> = if selected {
        let second = crate::tests::kit::translated_language(b"second ");
        Box::new([english, second])
    } else {
        Box::new([english])
    };
    Words::new(
        Phrasebook::load(
            crate::words::phrases::jig::shapes(256),
            languages,
            &PhraseLimits {
                languages: 2,
                phrases: crate::words::document_words::phrase_count(),
                holes: 4,
                segments: crate::tests::english_phrase_segments(2),
                text_bytes: 100_000,
                hole_bytes: 256,
            },
        )
        .unwrap(),
        u16::from(selected),
        0,
    )
}
fn build(value: &HoldLine, words: &Words) -> (Tree, JigBindings, Limits) {
    let f = crate::tests::render_limits(&limits(), words).unwrap();
    let capacity = Limits {
        nodes: f.nodes,
        depth: f.depth,
        text_per_node: f.text_per_node,
        text_bytes: f.text_bytes,
        styles_per_node: 2,
        bindings: f.bindings,
        slots: f.slots,
        markdown_depth: 2,
        ..Limits::EMPTY
    };
    let mut b = Builder::new(&capacity);
    let mut bindings = JigBindings::new(f.bindings, 256);
    let client = Fixture::new(&limits());
    view::hold(&mut b, value, &client.view(), words, &mut bindings);
    let tree = b.finish();
    let region = tree.nodes().first().expect("standalone hold region");
    assert_eq!(region.role, skein_ui::Role::Region);

    assert!(tree.len() <= f.nodes);
    let mut total = 0_u64;
    for node in tree.nodes() {
        assert!(node.text_bytes() <= f.text_per_node);
        total = total.checked_add(u64::from(node.text_bytes())).unwrap();
    }
    assert!(total <= u64::from(f.text_bytes));
    (tree, bindings, f)
}

#[test]
fn owner_hold_categories_and_failures_preserve_prior_source_and_ending() {
    let mut goldens = Goldens::new("tree/fragments/hold/owner_hold_all_categories");
    let w = words(false);
    let reasons = [
        HoldCategory::Circle,
        HoldCategory::Exceeded(Exceeded { bound: Bound::Output, value: u64::MAX }),
        HoldCategory::Stalled,
        HoldCategory::Dependency { task: u64::MAX },
        HoldCategory::Stopped,
        HoldCategory::Placed { hold: u64::MAX },
        HoldCategory::Drift,
        HoldCategory::Effects,
        HoldCategory::EffectFailed,
        HoldCategory::Uncertain { entry: u64::MAX },
        HoldCategory::Budget,
        HoldCategory::Deadline,
        HoldCategory::HoldsWaited,
        HoldCategory::ResourceUnavailable,
        HoldCategory::ResourceAccess,
        HoldCategory::PoolLost { pool: u64::MAX },
        HoldCategory::Procedure,
    ];
    for why in reasons {
        let value = HoldLine {
            task: u64::MAX,
            position: u64::MAX,
            at: Wall::EPOCH,
            before: PriorPhase::Waiting,
            why,
            source: None,
            ending: None,
        };
        let tree = build(&value, &w).0;
        goldens.check(&tree);
    }
    for class in [
        HoldFailure::Transient,
        HoldFailure::Permanent,
        HoldFailure::Run,
        HoldFailure::Agent,
        HoldFailure::Lost,
        HoldFailure::Invalid,
        HoldFailure::Infrastructure,
    ] {
        let value = HoldLine {
            task: u64::MAX,
            position: u64::MAX,
            at: Wall::EPOCH,
            before: PriorPhase::Active(RunStage::Running { attempt: u64::MAX }),
            why: HoldCategory::Failures { class },
            source: None,
            ending: None,
        };
        let tree = build(&value, &w).0;
        goldens.check(&tree);
    }
    let value = HoldLine {
        task: u64::MAX,
        position: u64::MAX,
        at: Wall::EPOCH,
        before: PriorPhase::Closing(ClosingStage::Run { attempt: u64::MAX }),
        why: HoldCategory::Budget,
        source: Some(part(3)),
        ending: Some(ChipEnding::Failed(doc(b"pending", true))),
    };
    let (tree, bindings, _) = build(&value, &w);
    goldens.check(&tree);
    let mut slots = 0_u32;
    for node in tree.nodes() {
        if let Some(slot) = node.slot {
            assert_eq!(bindings.owner_part(slot.token), Some(&part(3)));
            slots = slots.checked_add(1).unwrap();
        }
    }
    assert_eq!(slots, 1);
}
#[test]
fn owner_hold_task_and_position_are_full_width_and_hold_time_is_separate_from_cancellation_time() {
    let mut goldens = Goldens::new("tree/fragments/hold/owner_hold_task");
    let w = words(false);
    let value = HoldLine {
        task: u64::MAX,
        position: u64::MAX,
        at: Wall::from_nanos(19),
        before: PriorPhase::Waiting,
        why: HoldCategory::Stopped,
        source: None,
        ending: None,
    };
    goldens.check(&build(&value, &w).0);
    for by in [Requester::Task(u64::MAX), Requester::Person(u64::MAX), Requester::Deployment { project: u32::MAX }] {
        let value = with_ending(ChipEnding::Cancelled {
            reason: doc(b"stop", true),
            result: None,
            by,
            at: Wall::from_nanos(12),
        });
        let tree = build(&value, &w).0;
        goldens.check(&tree);
    }
}
#[test]
fn owner_hold_approval_explanation_presence_loss_and_raw_bytes() {
    let mut goldens = Goldens::new("tree/fragments/hold/owner_hold_approval");
    let w = words(false);
    let mut absent: Option<u32> = None;
    for explanation in [None, Some(doc(b"", false)), Some(doc(b"explain", true))] {
        let supplied = explanation.is_some();
        let lost = match &explanation {
            Some(value) => value.cut,
            None => false,
        };
        let value =
            with_ending(ChipEnding::Done(ResultValue::Approved { state: copy_of(&[255, 0, 254]), words: explanation }));
        let tree = build(&value, &w).0;
        goldens.check(&tree);
        if supplied {
            assert_eq!(tree.len(), absent.unwrap().checked_add(if lost { 2 } else { 1 }).unwrap());
        } else {
            absent = Some(tree.len());
        }
    }
    let value = with_ending(ChipEnding::Failed(doc(b"failed", true)));
    goldens.check(&build(&value, &w).0);
}
#[test]
fn owner_hold_second_checked_language_and_full_fragment_preserve_raw_slots_and_reject_wrong_holes() {
    let mut goldens = Goldens::new("tree/fragments/hold/owner_hold_second");
    let w = words(true);
    let value = HoldLine {
        task: u64::MAX,
        position: u64::MAX,
        at: Wall::EPOCH,
        before: PriorPhase::Closing(ClosingStage::Settled),
        why: HoldCategory::PoolLost { pool: u64::MAX },
        source: Some(part(3)),
        ending: Some(ChipEnding::Cancelled {
            reason: doc(UTF8_TEXTS[0], true),
            result: Some(ResultValue::Change { kind: u16::MAX, part: part(3), words: doc(UTF8_TEXTS[1], true) }),
            by: Requester::Person(u64::MAX),
            at: Wall::from_nanos(u64::MAX),
        }),
    };
    let (tree, bindings, _) = build(&value, &w);

    goldens.check(&tree);
    let mut slots = 0_u32;
    for n in tree.nodes() {
        if let Some(slot) = n.slot {
            assert_eq!(bindings.owner_part(slot.token), Some(&part(3)));
            slots = slots.checked_add(1).unwrap();
        }
    }
    assert_eq!(slots, 2);
    let id = crate::words::lifecycle_words::LifecyclePhrase::Position.id();
    let saying = w.book().say(1, id, &[Hole::Number(u64::MAX)]).expect("checked numeric hole");
    assert_eq!(saying.text.as_ref(), b"second 18446744073709551615");
    assert_eq!(w.book().say(1, id, &[Hole::Wall(Wall::from_nanos(0))]).err(), Some(Invalid::Values { id }));
}

const UTF8_TEXTS: [&[u8]; 2] = [&[b'~'; 8], &[b'!'; 8]];

#[test]
fn attempts_render_every_run_end_recorded_times_phase_count_and_absences_in_checked_languages() {
    let mut goldens = Goldens::new("tree/fragments/attempt/recorded_and_absent");
    let ends = [
        None,
        Some(RunEnd::Parked),
        Some(RunEnd::Finished),
        Some(RunEnd::Cancelled),
        Some(RunEnd::Refused),
        Some(RunEnd::Failed { class: HoldFailure::Invalid, invalid_reason: Some(InvalidReason::Form) }),
    ];
    for selected in [false, true] {
        let words = words(selected);
        for end in ends {
            let recorded = end.is_some();
            let value = AttemptLine {
                task: u64::MAX,
                attempt: 2,
                claimed: recorded.then_some(Wall::from_nanos(14)),
                started: recorded.then_some(Wall::from_nanos(15)),
                ended: recorded.then_some(Wall::from_nanos(19)),
                end,
                phases: recorded.then_some(3),
            };
            let mut builder = Builder::new(&crate::tests::kit::limits());
            view::attempt(&mut builder, &value, &words);
            let tree = builder.finish();
            goldens.check(&tree);
            assert_eq!(tree.nodes().first().unwrap().role, skein_ui::Role::Region);
            assert!(tree.len() <= view::max_nodes(&limits().view, &limits().client).unwrap());
        }
        let id = crate::words::lifecycle_words::LifecyclePhrase::Claimed.id();
        words.book().say(u16::from(selected), id, &[Hole::Wall(Wall::EPOCH)]).unwrap();
        assert_eq!(words.book().say(u16::from(selected), id, &[Hole::Number(14)]).err(), Some(Invalid::Values { id }));
    }
}
