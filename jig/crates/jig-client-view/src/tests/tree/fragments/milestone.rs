use crate::tests::fixtures::Fixture;
use crate::tests::kit::Goldens;
use crate::words::Words;
use crate::{JigBindings, tree::fragments::milestone as view};
use skein_ui::Limits;
use skein_ui::phrases::{Hole, Invalid, PhraseLimits, Phrasebook, SpanUnit, WallFormat};
use skein_ui::{Builder, Tree};

use jig_client::contract::*;
use skein_lib::{List, Wall, bytes::copy_of};
fn doc(bytes: &[u8], cut: bool) -> DocumentText {
    DocumentText { bytes: Text::new(copy_of(bytes), 8).unwrap(), cut }
}
fn part(capacity: u32) -> OwnerPart {
    let mut segments = List::with_capacity(capacity);
    if capacity >= 3 {
        for bytes in [b"a/b".as_slice(), &[255, 0], b""] {
            segments.push(copy_of(bytes)).unwrap();
        }
    }
    OwnerPart { connector: u16::MAX, kind: OwnerPartKind::Resource, key: OwnerPartKey::Path(segments) }
}
fn line(phase: OwnerMilestonePhase) -> OwnerMilestoneLine {
    OwnerMilestoneLine { task: u64::MAX, position: u64::MAX, phase, at: None }
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
                phrases: crate::words::owner_words::phrase_count(),
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
fn build(value: &OwnerMilestoneLine, words: &Words) -> (Tree, JigBindings, Limits) {
    let f = crate::tests::render_limits(&limits(), words).unwrap();
    let capacity = Limits {
        nodes: f.nodes,
        depth: f.depth,
        text_per_node: f.text_per_node,
        text_bytes: f.text_bytes,
        drawing_bytes: 0,
        styles_per_node: 2,
        patches: 0,
        bindings: f.bindings,
        slots: f.slots,
        markdown_depth: 2,
        scene_marks: 0,
        scene_bytes: 0,
    };
    let mut b = Builder::new(&capacity);
    let mut bindings = JigBindings::new(f.bindings, 256);
    let client = Fixture::new(&limits());
    view::render(&mut b, value, &client.view(), words, &mut bindings);
    let tree = b.finish();
    let region = tree.nodes().first().expect("standalone milestone region");
    assert_eq!(region.role, skein_ui::Role::Region);

    assert!(tree.len() <= f.nodes);
    let mut total = 0_u64;
    for node in tree.nodes() {
        assert!(crate::tests::text_bytes(node) <= f.text_per_node);
        total = total.checked_add(u64::from(crate::tests::text_bytes(node))).unwrap();
    }
    assert!(total <= u64::from(f.text_bytes));
    (tree, bindings, f)
}

#[test]
fn owner_milestone_all_five_phases_six_activation_and_five_closing_stages_keep_owner_coordinates() {
    let mut goldens = Goldens::new("tree/fragments/milestone/owner_milestone_all_five");
    let w = words(false);
    let waiting = line(OwnerMilestonePhase::Waiting);
    goldens.check(&build(&waiting, &w).0);
    for stage in [
        RunStage::Idle,
        RunStage::Due,
        RunStage::Preparing,
        RunStage::Claimed { attempt: u64::MAX },
        RunStage::Running { attempt: u64::MAX },
        RunStage::BackingOff { until: Wall::from_nanos(u64::MAX) },
    ] {
        let value = line(OwnerMilestonePhase::Active(stage));
        let tree = build(&value, &w).0;
        goldens.check(&tree);
        assert_eq!(value.phase, OwnerMilestonePhase::Active(stage));
    }
    for stage in [
        ClosingStage::Run { attempt: u64::MAX },
        ClosingStage::Delegates,
        ClosingStage::Effects,
        ClosingStage::Releases,
        ClosingStage::Settled,
    ] {
        let value =
            line(OwnerMilestonePhase::Closing { stage, ending: OwnerChipEnding::Failed(doc(b"closing", false)) });
        let tree = build(&value, &w).0;
        goldens.check(&tree);
    }
    goldens.check(&build(&line(OwnerMilestonePhase::Ended(OwnerChipEnding::Failed(doc(b"end", false)))), &w).0);
    goldens.check(
        &build(
            &line(OwnerMilestonePhase::Held {
                before: OwnerPriorPhase::Waiting,
                why: HoldCategory::Budget,
                source: None,
                ending: None,
            }),
            &w,
        )
        .0,
    );
}
#[test]
fn owner_milestone_all_sixteen_hold_categories_and_six_failures_preserve_prior_without_live_revision_or_clock() {
    let mut goldens = Goldens::new("tree/fragments/milestone/owner_milestone_all_sixteen");
    let w = words(false);
    let reasons = [
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
        let value =
            line(OwnerMilestonePhase::Held { before: OwnerPriorPhase::Waiting, why, source: None, ending: None });
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
    ] {
        let value = line(OwnerMilestonePhase::Held {
            before: OwnerPriorPhase::Active(RunStage::Running { attempt: u64::MAX }),
            why: HoldCategory::Failures { class },
            source: None,
            ending: None,
        });
        let tree = build(&value, &w).0;
        goldens.check(&tree);
    }
    let value = line(OwnerMilestonePhase::Held {
        before: OwnerPriorPhase::Closing(ClosingStage::Run { attempt: u64::MAX }),
        why: HoldCategory::Budget,
        source: Some(part(3)),
        ending: Some(OwnerChipEnding::Failed(doc(b"pending", true))),
    });
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
fn owner_milestone_task_and_position_are_full_width_and_optional_time_is_separate_from_cancellation_time() {
    let mut goldens = Goldens::new("tree/fragments/milestone/owner_milestone_task");
    let w = words(false);
    for task in [0, u64::MAX] {
        for position in [0, u64::MAX] {
            for at in [None, Some(Wall::from_nanos(0)), Some(Wall::from_nanos(u64::MAX))] {
                let value = OwnerMilestoneLine { task, position, at, phase: OwnerMilestonePhase::Waiting };
                let tree = build(&value, &w).0;
                assert_eq!(value.task, task);
                assert_eq!(value.position, position);
                assert_eq!(value.at, at);
                goldens.check(&tree);
            }
        }
    }
    for by in [
        OwnerRequester::Task(u64::MAX),
        OwnerRequester::Person(u64::MAX),
        OwnerRequester::Deployment { project: u32::MAX },
    ] {
        let value = line(OwnerMilestonePhase::Ended(OwnerChipEnding::Cancelled {
            reason: doc(b"stop", true),
            result: None,
            by,
            at: Wall::from_nanos(12),
        }));
        let tree = build(&value, &w).0;
        goldens.check(&tree);
    }
}
#[test]
fn owner_milestone_whole_results_keep_all_nine_variants_and_independent_approval_presence_loss_and_raw_bytes() {
    let mut goldens = Goldens::new("tree/fragments/milestone/owner_milestone_whole");
    let w = words(false);
    let fields = crate::tests::kit::list_in(3, [ResultField { name: copy_of(b"name"), value: copy_of(b"value") }]);
    let values = [
        ResultValue::Report(doc(b"report", true)),
        ResultValue::OwnerVerdict { verdict: u32::MAX, words: doc(b"verdict", false), fields: fields.clone() },
        ResultValue::Verdict {
            verdict: u32::MAX,
            words: doc(b"followup", true),
            fields,
            tasks: List::with_capacity(3),
            proposal: None,
        },
        ResultValue::Connector { summary: doc(b"summary", false), part: Part::Connector { connector: u16::MAX } },
        ResultValue::Change { kind: u16::MAX, part: part(3), words: doc(b"change", true) },
        ResultValue::Choice {
            option: u32::MAX,
            by: Party { number: u64::MAX, kind: PartyKind::Person, name: Text::new(copy_of(b"person"), 8).unwrap() },
            reason: doc(b"choice", false),
        },
        ResultValue::Approved { state: copy_of(&[255, 0]), words: None },
        ResultValue::Failure(doc(b"failure", true)),
        ResultValue::Words(doc(b"words", false)),
    ];
    for value in values {
        let source = line(OwnerMilestonePhase::Ended(OwnerChipEnding::Done(value)));
        let original = source.clone();
        let tree = build(&source, &w).0;
        goldens.check(&tree);
        assert_eq!(source, original);
    }
    let mut absent: Option<u32> = None;
    for explanation in [None, Some(doc(b"", false)), Some(doc(b"explain", true))] {
        let supplied = explanation.is_some();
        let lost = match &explanation {
            Some(value) => value.cut,
            None => false,
        };
        let value = line(OwnerMilestonePhase::Ended(OwnerChipEnding::Done(ResultValue::Approved {
            state: copy_of(&[255, 0, 254]),
            words: explanation,
        })));
        let tree = build(&value, &w).0;
        goldens.check(&tree);
        if supplied {
            assert_eq!(tree.len(), absent.unwrap().checked_add(if lost { 2 } else { 1 }).unwrap());
        } else {
            absent = Some(tree.len());
        }
    }
    let value = line(OwnerMilestonePhase::Ended(OwnerChipEnding::Failed(doc(b"failed", true))));
    goldens.check(&build(&value, &w).0);
}
#[test]
fn owner_milestone_second_checked_language_and_full_fragment_preserve_raw_slots_and_reject_wrong_holes() {
    let mut goldens = Goldens::new("tree/fragments/milestone/owner_milestone_second");
    let w = words(true);
    let value = line(OwnerMilestonePhase::Held {
        before: OwnerPriorPhase::Closing(ClosingStage::Settled),
        why: HoldCategory::PoolLost { pool: u64::MAX },
        source: Some(part(3)),
        ending: Some(OwnerChipEnding::Cancelled {
            reason: doc(UTF8_TEXTS[0], true),
            result: Some(ResultValue::Change { kind: u16::MAX, part: part(3), words: doc(UTF8_TEXTS[1], true) }),
            by: OwnerRequester::Person(u64::MAX),
            at: Wall::from_nanos(u64::MAX),
        }),
    });
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
    let id = crate::words::milestone_words::MilestonePhrase::Position.id();
    let saying = w.book().say(1, id, &[Hole::Number(u64::MAX)]).expect("checked numeric hole");
    assert_eq!(saying.text.as_ref(), b"second 18446744073709551615");
    assert_eq!(w.book().say(1, id, &[Hole::Wall(Wall::from_nanos(0))]).err(), Some(Invalid::Values { id }));
}

const UTF8_TEXTS: [&[u8]; 2] = [&[b'~'; 8], &[b'!'; 8]];
