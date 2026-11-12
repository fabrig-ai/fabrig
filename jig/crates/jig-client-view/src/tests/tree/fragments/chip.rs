#![expect(clippy::disallowed_methods, reason = "ordinary tests inspect public trees and checked language data")]
use crate::tests::fixtures::Fixture;
use crate::tests::kit::Goldens;
use crate::words::Words;
use crate::words::{chip_words as data, chip_words::ChipPhrase};
use crate::{JigBindings, tree::fragments::chip as view};
use skein_ui::Limits;
use skein_ui::phrases::{Invalid, PhraseLimits, Segment};
use skein_ui::{Builder, Tree};

use jig_client::contract::*;
use skein_lib::{Duration, List, Token, Wall, bytes::copy_of};
fn doc(bytes: &[u8], cut: bool) -> DocumentText {
    DocumentText { bytes: Text::new(copy_of(bytes), 256).unwrap(), cut }
}
fn part() -> Part {
    let path = crate::tests::kit::list([copy_of(b"a/b"), copy_of(&[255, 0])]);
    Part { connector: u16::MAX, kind: PartKind::Resource, key: PartKey::Path(path) }
}
pub(in crate::tests) fn details() -> ChipDetails {
    ChipDetails {
        dependencies: List::with_capacity(0),
        project: u32::MAX,
        executor: Executor::Agent { charter: u32::MAX },
        phase_since: Wall::from_nanos(10),
        run_since: Some(Wall::from_nanos(20)),
        attempt: Some(u64::MAX),
        retry: Some(RetryNotice::Refused { count: u32::MAX, reason: None }),
        wake: WakePolicy {
            words: WakeRule::Never,
            notices: WakeRule::Immediate,
            news: WakeRule::Batch { count: u32::MAX, age: Duration::from_nanos(30) },
            results: ResultsWake::LastOrFailure,
            questions: true,
            answers: false,
            timers: true,
        },
        person_wait: None,
        hold: None,
        budget: Budget {
            numbers: Numbers { budget: 100, spent: 2, spent_below: 3, reserved: 5 },
            left: 7,
            funder: Funder::Task(u64::MAX),
            deadline: None,
        },
        activity: u64::MAX,
        resources: List::with_capacity(0),
        resources_cut: false,
        resources_observed: false,
        procedure: None,
        live_delegates: u32::MAX,
        closing_connectors: None,
        ending: None,
        ended_at: None,
        observation: None,
    }
}
fn hold() -> ChipHold {
    ChipHold {
        words: None,
        placed: None,
        circle: None,
        account: None,
        source: None,
        reason: HoldCategory::Budget,
        revision: u64::MAX,
        before: PriorPhase::Waiting,
        escalation: None,
        tries: FailureTries {
            transient: 1,
            permanent: 2,
            run: 3,
            agent: 4,
            lost: 5,
            invalid: u32::MAX,
            infrastructure: 0,
        },
        hold_wait_since: Some(Wall::from_nanos(40)),
    }
}
fn client_limits() -> crate::tests::fixtures::Profile {
    crate::tests::fixtures::Profile {
        view: crate::Config { objects: 4, ..(crate::tests::tree::cards::CLIENT).view },
        client: jig_client::DomainLimits {
            rows: 4,
            text: 256,
            long_text: 256,
            ..(crate::tests::tree::cards::CLIENT).client
        },
    }
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
fn words() -> &'static Words {
    static WORDS: std::sync::OnceLock<Words> = std::sync::OnceLock::new();
    WORDS.get_or_init(|| {
        let mut language = crate::words::phrases::english::language();
        language.wall = skein_ui::phrases::WallFormat::Nanos;
        language.span_unit = skein_ui::phrases::SpanUnit::Nanos;
        Words::new(
            skein_ui::phrases::Phrasebook::load(
                crate::words::phrases::jig::shapes(256),
                Box::new([language]),
                &phrase_limits(),
            )
            .unwrap(),
            0,
            0,
        )
    })
}
fn build(value: &ChipDetails, words: &Words) -> (Tree, JigBindings, Limits) {
    let client = Fixture::new(&client_limits());
    let f = crate::tests::render_limits(&client_limits(), words).unwrap();
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
    view::render(&mut b, value, &client.view(), words, &mut bindings);
    let tree = b.finish();
    assert!(tree.len() <= f.nodes);
    assert!(tree.nodes().iter().all(|node| node.text_bytes() <= f.text_per_node));
    (tree, bindings, f)
}

#[test]
fn retained_chip_preserves_concrete_executor_project_and_full_width_owner_scalars() {
    let mut goldens = Goldens::new("tree/fragments/chip/retained_chip_preserves");
    let mut value = details();
    for executor in [
        Executor::Agent { charter: u32::MAX },
        Executor::Procedure { connector: u16::MAX, code: u32::MAX },
        Executor::Person(PersonAddress::Party(u64::MAX)),
        Executor::Person(PersonAddress::Role(u32::MAX)),
    ] {
        value.executor = executor;
        let (tree, _, _) = build(&value, words());
        assert_eq!(tree.nodes()[0].role, skein_ui::Role::Region);
        goldens.check(&tree);
        assert_eq!(value.clone().executor, executor);
    }
}
#[test]
fn placement_and_attempt_do_not_attest_unobserved_work_or_actual_start() {
    let mut goldens = Goldens::new("tree/fragments/chip/placement_and_attempt");
    let mut value = details();
    let (tree, _, _) = build(&value, words());
    goldens.check(&tree);
    let observation = ChipObservation {
        channel: Token::new(u64::MAX),
        work: ChipWork::Tool { call: copy_of(&[255, 0]), tool: copy_of(b"opaque/tool") },
        since: Wall::from_nanos(50),
        reported_at: Wall::from_nanos(60),
        started: None,
    };
    value.observation = Some(observation.clone());
    let (tree, _, _) = build(&value, words());
    assert_eq!(value.observation, Some(observation));
    goldens.check(&tree);
    value.observation.as_mut().unwrap().started = Some(Wall::from_nanos(70));
    goldens.check(&build(&value, words()).0);
    for work in [
        ChipWork::Preparing,
        ChipWork::Starting,
        ChipWork::Active,
        ChipWork::Writing,
        ChipWork::Waiting,
        ChipWork::Ending,
    ] {
        value.observation.as_mut().unwrap().work = work;
        goldens.check(&build(&value, words()).0);
    }
}
#[test]
fn closing_connector_absence_empty_and_terminal_clock_are_independent_owner_facts() {
    let mut goldens = Goldens::new("tree/fragments/chip/closing_connector_absence");
    let mut value = details();
    goldens.check(&build(&value, words()).0);
    value.closing_connectors = Some(List::with_capacity(2));
    goldens.check(&build(&value, words()).0);
    value.closing_connectors.as_mut().unwrap().push(u16::MAX).unwrap();
    let (tree, _, _) = build(&value, words());
    goldens.check(&tree);
    value.ending = Some(ChipEnding::Failed(doc(b"failed", false)));
    goldens.check(&build(&value, words()).0);
    value.ended_at = Some(Wall::from_nanos(80));
    value.ending = None;
    let tree = build(&value, words()).0;
    goldens.check(&tree);
}
#[test]
fn resource_rows_keep_raw_segmented_parts_and_independent_holder_writer_loss_observation() {
    let mut goldens = Goldens::new("tree/fragments/chip/resource_rows_keep");
    let mut value = details();
    let reference = part();
    value.resources = List::with_capacity(2);
    value
        .resources
        .push(ResourceLine {
            part: reference.clone(),
            kind: u16::MAX,
            taken: false,
            holder: None,
            writer: Some(ResourceWriter::Effect { entry: u64::MAX }),
            writer_lost: true,
        })
        .unwrap();
    value.resources_cut = true;
    let (tree, bindings, _) = build(&value, words());
    goldens.check(&tree);
    for node in tree.nodes() {
        if let Some(slot) = node.slot {
            assert_eq!(bindings.owner_part(slot.token), Some(&reference));
        }
    }
    let row = value.resources.get_mut(0).unwrap();
    row.holder = Some(u64::MAX);
    row.writer = Some(ResourceWriter::Run { task: u64::MAX, attempt: u64::MAX });
    row.taken = true;
    row.writer_lost = false;
    value.resources_observed = true;
    value.resources_cut = false;
    let tree = build(&value, words()).0;
    goldens.check(&tree);
}
#[test]
fn all_hold_categories_and_seven_failure_classes_retain_only_supplied_payloads_and_counters() {
    let mut goldens = Goldens::new("tree/fragments/chip/all_hold_categories");
    let mut value = details();
    value.hold = Some(hold());
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
    for reason in reasons {
        value.hold.as_mut().unwrap().reason = reason;
        let tree = build(&value, words()).0;
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
        value.hold.as_mut().unwrap().reason = HoldCategory::Failures { class };
        let tree = build(&value, words()).0;
        goldens.check(&tree);
    }
}
#[test]
fn held_prior_stages_have_no_invented_clocks_and_escalations_keep_holder_scope_and_local_loss() {
    let mut goldens = Goldens::new("tree/fragments/chip/held_prior_stages");
    let mut value = details();
    value.hold = Some(hold());
    value.run_since = None;
    value.attempt = None;
    let priors = [
        PriorPhase::Waiting,
        PriorPhase::Active(RunStage::Idle),
        PriorPhase::Active(RunStage::Due),
        PriorPhase::Active(RunStage::Preparing),
        PriorPhase::Active(RunStage::Claimed { attempt: u64::MAX }),
        PriorPhase::Active(RunStage::Running { attempt: u64::MAX }),
        PriorPhase::Active(RunStage::BackingOff { until: Wall::from_nanos(90) }),
        PriorPhase::Closing(ClosingStage::Run { attempt: u64::MAX }),
        PriorPhase::Closing(ClosingStage::Delegates),
        PriorPhase::Closing(ClosingStage::Effects),
        PriorPhase::Closing(ClosingStage::Releases),
        PriorPhase::Closing(ClosingStage::Settled),
    ];
    for prior in priors {
        value.hold.as_mut().unwrap().before = prior;
        let tree = build(&value, words()).0;
        goldens.check(&tree);
    }
    value.hold.as_mut().unwrap().escalation = Some(HoldEscalation::Routing);
    goldens.check(&build(&value, words()).0);
    for holder in [
        EscalationHolder::Task(u64::MAX),
        EscalationHolder::Person(u64::MAX),
        EscalationHolder::Role { project: u32::MAX, role: u32::MAX },
    ] {
        value.hold.as_mut().unwrap().escalation =
            Some(HoldEscalation::Waiting { holder, entry: u64::MAX, since: Wall::from_nanos(100) });
        let tree = build(&value, words()).0;
        goldens.check(&tree);
    }
    value.hold.as_mut().unwrap().escalation =
        Some(HoldEscalation::Rejected { by: u64::MAX, reason: doc(UTF8_TEXTS[0], true) });
    let tree = build(&value, words()).0;
    goldens.check(&tree);
}
#[test]
fn person_wait_notices_keep_full_message_activity_and_independent_owner_times() {
    let mut goldens = Goldens::new("tree/fragments/chip/person_wait_notices");
    let mut value = details();
    value.person_wait = Some(PersonWait {
        since: Wall::from_nanos(110),
        stall: Duration::from_nanos(120),
        deadline: Wall::from_nanos(130),
        notice: Some(PersonStallNotice { message: u64::MAX, at: Wall::from_nanos(140), activity: u64::MAX }),
    });
    let tree = build(&value, words()).0;
    goldens.check(&tree);
}
#[test]
fn complete_chip_endings_keep_result_requester_and_each_independent_reason_loss() {
    let mut goldens = Goldens::new("tree/fragments/chip/complete_chip_endings");
    let mut value = details();
    value.ending = Some(ChipEnding::Done(ResultValue::Report(doc(b"whole result", true))));
    let tree = build(&value, words()).0;
    goldens.check(&tree);
    for by in [Requester::Task(u64::MAX), Requester::Person(u64::MAX), Requester::Deployment { project: u32::MAX }] {
        value.ending = Some(ChipEnding::Cancelled {
            reason: doc(b"reason", true),
            result: Some(ResultValue::Words(doc(b"prior result", false))),
            by,
            at: Wall::from_nanos(150),
        });
        let tree = build(&value, words()).0;
        goldens.check(&tree);
    }
    value.ending = Some(ChipEnding::Failed(doc(b"terminal failure", false)));
    let tree = build(&value, words()).0;
    goldens.check(&tree);
    value.ending = Some(ChipEnding::Failed(doc(b"terminal failure", true)));
    goldens.check(&build(&value, words()).0);
    value.ending = Some(ChipEnding::Cancelled {
        reason: doc(b"reason", false),
        result: None,
        by: Requester::Task(7),
        at: Wall::from_nanos(150),
    });
    let tree = build(&value, words()).0;
    goldens.check(&tree);
    let mut absent_nodes = None;
    for explanation in [None, Some(doc(b"", false)), Some(doc(b"approval explanation", true))] {
        let supplied = explanation.is_some();
        let lost = explanation.as_ref().is_some_and(|value| value.cut);
        value.ending =
            Some(ChipEnding::Done(ResultValue::Approved { state: copy_of(&[255, 0, 254]), words: explanation }));
        let tree = build(&value, words()).0;
        goldens.check(&tree);
        if supplied {
            assert_eq!(tree.nodes().len(), absent_nodes.unwrap() + if lost { 2 } else { 1 });
        } else {
            absent_nodes = Some(tree.nodes().len());
        }
    }
}

#[test]
fn exact_owner_verdict_fits_chip_result_fragment_at_full_field_capacity_and_rejects_whole_overflow() {
    let mut goldens = Goldens::new("tree/fragments/chip/exact_owner_verdict");
    let mut fields = List::with_capacity(client_limits().client.rows);
    for _ in 0..client_limits().client.rows {
        fields.push(ResultField { name: copy_of(&[255; 256]), value: copy_of(&[b'x'; 256]) }).unwrap();
    }
    let mut value = details();
    value.ending =
        Some(ChipEnding::Done(ResultValue::OwnerVerdict { verdict: u32::MAX, words: doc(&[b'w'; 256], true), fields }));
    let before = value.clone();
    let (tree, _, envelope) = build(&value, words());
    goldens.check(&tree);
    assert!(u32::try_from(tree.nodes().len()).unwrap() <= envelope.nodes);
    assert!(
        tree.nodes().iter().map(|node| u64::from(node.text_bytes())).sum::<u64>() <= u64::from(envelope.text_bytes)
    );

    assert_eq!(value, before);
}
#[test]
fn maximum_chip_fragment_checks_selected_language_storage_raw_slots_loss_and_arithmetic_bounds() {
    let mut goldens = Goldens::new("tree/fragments/chip/maximum_chip_fragment");
    let mut value = details();
    value.hold = Some(hold());
    value.hold.as_mut().unwrap().source = Some(part());
    value.procedure = Some(part());
    value.resources = List::with_capacity(4);
    for _ in 0_u32..4 {
        value
            .resources
            .push(ResourceLine {
                part: part(),
                kind: u16::MAX,
                taken: true,
                holder: Some(u64::MAX),
                writer: Some(ResourceWriter::Run { task: u64::MAX, attempt: u64::MAX }),
                writer_lost: true,
            })
            .unwrap();
    }
    value.resources_cut = true;
    value.closing_connectors = Some(List::with_capacity(4));
    for _ in 0_u32..4 {
        value.closing_connectors.as_mut().unwrap().push(u16::MAX).unwrap();
    }
    value.hold.as_mut().unwrap().escalation =
        Some(HoldEscalation::Rejected { by: u64::MAX, reason: doc(UTF8_TEXTS[1], true) });
    value.observation = Some(ChipObservation {
        channel: Token::new(u64::MAX),
        work: ChipWork::Tool { call: copy_of(&[255; 256]), tool: copy_of(&[254; 256]) },
        since: Wall::from_nanos(160),
        reported_at: Wall::from_nanos(170),
        started: Some(Wall::from_nanos(180)),
    });
    value.ending = Some(ChipEnding::Cancelled {
        reason: doc(UTF8_TEXTS[2], true),
        result: Some(ResultValue::Change { kind: u16::MAX, part: part(), words: doc(b"change", true) }),
        by: Requester::Deployment { project: u32::MAX },
        at: Wall::from_nanos(190),
    });
    value.ended_at = Some(Wall::from_nanos(200));
    let (tree, bindings, envelope) = build(&value, words());
    assert!(
        tree.nodes().iter().map(|node| u64::from(node.text_bytes())).sum::<u64>() <= u64::from(envelope.text_bytes)
    );
    assert_eq!(tree.nodes().iter().filter(|node| node.slot.is_some()).count(), 7);
    for node in tree.nodes() {
        if let Some(slot) = node.slot {
            assert!(bindings.owner_part(slot.token).is_some());
        }
    }

    let mut translated = crate::words::phrases::english::language();
    for phrase in &mut translated.phrases {
        for form in &mut phrase.forms {
            for segment in &mut form.segments {
                if let Segment::Text(bytes) = segment {
                    *bytes = copy_of(b"selected ");
                }
            }
        }
    }
    let selected =
        crate::tests::checked_words(Box::new([crate::words::phrases::english::language(), translated]), 1, 0);
    let selected_tree = build(&details(), &selected).0;
    assert_eq!(selected_tree.nodes()[0].role, skein_ui::Role::Region);
    goldens.check(&selected_tree);
    let mut language = data::language();
    language.phrases[0].id = ChipPhrase::PhaseSince.id();
    assert_eq!(
        skein_ui::phrases::Phrasebook::load(data::shapes(), Box::new([language]), &phrase_limits()).err(),
        Some(Invalid::Duplicate { id: ChipPhrase::PhaseSince.id() })
    );
}

#[test]
fn owner_retry_keeps_all_six_classes_and_independent_zero_max_counts_in_chip_and_activation() {
    let mut goldens = Goldens::new("tree/fragments/chip/owner_retry_keeps");
    let classes = [
        HoldFailure::Transient,
        HoldFailure::Permanent,
        HoldFailure::Run,
        HoldFailure::Agent,
        HoldFailure::Lost,
        HoldFailure::Invalid,
        HoldFailure::Infrastructure,
    ];
    for class in classes {
        for (count, allowance) in [(0, 0), (0, u32::MAX), (u32::MAX, 0), (u32::MAX, u32::MAX)] {
            let retry = Some(RetryNotice::OwnerFailed { class, count, allowance, reason: None });
            let mut value = details();
            value.retry = retry.clone();
            assert_eq!(value.clone().retry, retry);
            let tree = build(&value, words()).0;
            goldens.check(&tree);
            assert_eq!(value.retry, retry, "rendering cannot mutate owner class or independent counts");
        }
    }
}

#[test]
fn owner_retry_absence_refusal_and_all_supplied_failure_classes_remain_independent() {
    let mut goldens = Goldens::new("tree/fragments/chip/owner_retry_absence");
    let mut value = details();
    value.retry = None;
    {
        let tree = build(&value, words()).0;
        goldens.check(&tree);
    }
    for count in [0, u32::MAX] {
        value.retry = Some(RetryNotice::Refused { count, reason: None });
        {
            let tree = build(&value, words()).0;
            goldens.check(&tree);
        }
    }
    for class in [FailureClass::HostLost, FailureClass::Temporary, FailureClass::Permanent, FailureClass::Stall] {
        value.retry = Some(RetryNotice::Failed { class, count: 7, allowance: 9, reason: None });
        {
            let tree = build(&value, words()).0;
            goldens.check(&tree);
        }
    }
}

#[test]
fn owner_retry_borrows_selected_checked_six_class_words_and_both_count_holes() {
    let mut goldens = Goldens::new("tree/fragments/chip/owner_retry_borrows");
    let cases = [
        (HoldFailure::Transient, ChipPhrase::Transient, b"chosen transient".as_slice()),
        (HoldFailure::Permanent, ChipPhrase::Permanent, b"chosen permanent"),
        (HoldFailure::Run, ChipPhrase::RunFailure, b"chosen run"),
        (HoldFailure::Agent, ChipPhrase::AgentFailure, b"chosen agent"),
        (HoldFailure::Lost, ChipPhrase::Lost, b"chosen lost"),
        (HoldFailure::Invalid, ChipPhrase::InvalidFailure, b"chosen invalid"),
    ];
    let mut selected = crate::words::phrases::english::language();
    for phrase in &mut selected.phrases {
        for (_, class_phrase, label) in cases {
            if phrase.id == class_phrase.id() {
                phrase.forms[0].segments = Box::new([Segment::Text(copy_of(label))]);
            }
        }
        if phrase.id == crate::words::phrases::jig::JigPhrase::RunRetryFailed.id() {
            phrase.forms[0].segments = Box::new([
                Segment::Text(copy_of(b"chosen count ")),
                Segment::Hole { index: 0, kind: skein_ui::phrases::HoleKind::Number },
                Segment::Text(copy_of(b" versus ")),
                Segment::Hole { index: 1, kind: skein_ui::phrases::HoleKind::Number },
            ]);
        }
    }
    let checked = crate::tests::checked_words(Box::new([crate::words::phrases::english::language(), selected]), 1, 0);
    for (class, _, _) in cases {
        let mut value = details();
        value.retry = Some(RetryNotice::OwnerFailed { class, count: 0, allowance: u32::MAX, reason: None });
        {
            let tree = build(&value, &checked).0;
            goldens.check(&tree);
        }
    }
}

#[test]
fn supplied_hold_account_and_origin_keep_distinct_public_slots_and_absence() {
    let mut goldens = Goldens::new("tree/fragments/chip/supplied_hold_account");
    let account = Part { connector: 7, kind: PartKind::Hold, key: PartKey::Number(u64::MAX) };
    let origin = part();
    for supplied in [None, Some(account.clone())] {
        for source in [None, Some(origin.clone())] {
            let mut held = hold();
            held.account = supplied.clone();
            held.source = source.clone();
            held.reason = HoldCategory::Drift;
            let mut value = details();
            value.hold = Some(held);
            let (tree, bindings, envelope) = build(&value, words());
            let parts: Box<[_]> = tree
                .nodes()
                .iter()
                .filter_map(|node| {
                    node.slot.map(|slot| bindings.owner_part(slot.token).expect("exact owner part").clone())
                })
                .collect();
            let expected: Box<[_]> = supplied.iter().chain(source.iter()).cloned().collect();
            assert_eq!(parts, expected);
            goldens.check(&tree);
            assert!(tree.len() <= envelope.nodes);
            assert_eq!(value.hold.as_ref().unwrap().account, supplied);
            assert_eq!(value.hold.as_ref().unwrap().source, source);
        }
    }
}

const UTF8_TEXTS: [&[u8]; 3] = [b"~", &[b'~'; 256], &[b'?'; 256]];

#[test]
fn a_long_resource_path_shows_its_tail_and_keeps_the_whole_slot() {
    let mut goldens = Goldens::new("tree/fragments/chip/a_long_resource");
    let path = crate::tests::fixtures::long_resource_path::<256>();
    let reference = Part { connector: u16::MAX, kind: PartKind::Resource, key: PartKey::Path(path) };
    let mut value = details();
    value.procedure = Some(reference.clone());
    let (tree, bindings, _) = build(&value, words());
    goldens.check(&tree);
    let slot = tree.nodes().iter().find_map(|node| node.slot).expect("whole resource slot");
    assert_eq!(bindings.owner_part(slot.token), Some(&reference));
}

#[test]
fn a_circle_hold_names_the_waits_in_order_and_lifts_on_its_own() {
    let mut goldens = Goldens::new("tree/fragments/chip/a_circle_hold");
    let mut value = details();
    let mut held = hold();
    held.reason = HoldCategory::Circle;
    let tasks = crate::tests::kit::list([7, 5, 9]);
    held.circle = Some(Circle { tasks, cut: true });
    value.hold = Some(held);
    let (tree, _, _) = build(&value, words());
    goldens.check(&tree);
}

#[test]
fn an_effect_or_procedure_hold_card_shows_its_retained_account() {
    let mut goldens = Goldens::new("tree/fragments/chip/an_effect_or");
    let mut value = details();
    let mut held = hold();
    held.words = Some(doc(b"restart on production/checkout: permission refused; amend authority and release", false));
    value.hold = Some(held);
    let (tree, _, _) = build(&value, words());
    goldens.check(&tree);
}

#[test]
fn a_retry_notice_shows_its_preparation_reason_after_the_count() {
    let mut goldens = Goldens::new("tree/fragments/chip/a_retry_notice_shows");
    let reason = Some(doc(b"brief: its required sections do not fit its budget", false));
    for retry in [
        RetryNotice::OwnerFailed { class: HoldFailure::Permanent, count: 2, allowance: 3, reason: reason.clone() },
        RetryNotice::Failed { class: FailureClass::Temporary, count: 2, allowance: 3, reason: reason.clone() },
        RetryNotice::Refused { count: 2, reason },
    ] {
        let mut value = details();
        value.retry = Some(retry.clone());
        let tree = build(&value, words()).0;
        assert_retry_reason(&tree);
        goldens.check(&tree);
        let mut builder = Builder::new(&crate::tests::kit::limits());
        words().chat_status(
            &mut builder,
            &jig_client::facts::ChatStatus::Retrying { until: Wall::from_nanos(90), notice: Some(retry) },
            Wall::EPOCH,
        );
        assert_retry_reason(&builder.finish());
    }
}
fn assert_retry_reason(tree: &Tree) {
    let text: Box<[_]> = tree.nodes().iter().flat_map(|node| node.text.iter()).map(|run| run.text.as_ref()).collect();
    let why = text
        .iter()
        .position(|text| *text == b"Why: brief: its required sections do not fit its budget")
        .expect("supplied preparation reason");
    assert!(
        text[..why].iter().any(|text| skein_lib::bytes::find(text, b"count 2").is_some()),
        "reason follows the count"
    );
}

#[test]
fn a_held_chip_shows_its_preparation_reason() {
    let mut value = details();
    let mut held = hold();
    held.reason = HoldCategory::Failures { class: HoldFailure::Permanent };
    held.words = Some(doc(b"brief: its required sections do not fit its budget", false));
    value.hold = Some(held);
    let tree = build(&value, words()).0;
    assert!(
        tree.nodes()
            .iter()
            .flat_map(|node| node.text.iter())
            .any(|run| run.text.as_ref() == b"brief: its required sections do not fit its budget")
    );
    Goldens::new("tree/fragments/chip/a_held_chip_shows").check(&tree);
}
