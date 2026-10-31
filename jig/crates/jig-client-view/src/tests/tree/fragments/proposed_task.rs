#![expect(
    clippy::disallowed_methods,
    clippy::disallowed_types,
    clippy::disallowed_macros,
    reason = "ordinary fixtures inspect public semantic trees"
)]
use crate::tests::kit::Goldens;
use crate::tests::tree::{
    cards::{CLIENT, limits, words},
    work_pages::list,
};
use crate::{JigBindings, tree::fragments::proposed_task};
use skein_ui::{Builder, Role, Tree};

use jig_client::contract::*;

fn builder() -> Builder {
    let mut limits = limits();
    limits.nodes = 512;
    limits.depth = 16;
    let mut builder = Builder::new(&limits);
    builder.open(Role::Page);
    builder
}
fn finish(mut builder: Builder) -> Tree {
    builder.close();
    builder.finish()
}

#[test]
fn exact_part_slots_keep_separate_literal_segments_all_owner_kinds_and_numeric_keys() {
    let mut goldens = Goldens::new("tree/fragments/proposed_task/exact_part_slots");
    let words = words();
    for kind in [
        OwnerPartKind::Resource,
        OwnerPartKind::Pool,
        OwnerPartKind::Result,
        OwnerPartKind::Procedure,
        OwnerPartKind::Effect,
        OwnerPartKind::Outbox,
    ] {
        for key in [
            OwnerPartKey::Path(list([b"a/b".as_slice().into(), b"c\xff".as_slice().into()])),
            OwnerPartKey::Path(list([b"a".as_slice().into(), b"b/c".as_slice().into()])),
            OwnerPartKey::Number(u64::MAX),
        ] {
            let reference = OwnerPart { connector: u16::MAX, kind, key };
            let mut b = builder();
            let mut bindings = JigBindings::new(16, CLIENT.client.text);
            proposed_task::part(&mut b, &reference, words, &mut bindings);
            let tree = finish(b);
            let slot = tree.nodes().iter().find_map(|node| node.slot).unwrap();
            assert_eq!(slot.owner, (crate::Owner::Application { number: u16::MAX }).opaque());
            assert_eq!(bindings.owner_part(slot.token), Some(&reference));
            assert_eq!(bindings.part(slot.token), None);
            assert_eq!(crate::decode(&mut bindings, slot.token, skein_ui::Act::Press), None);
            match &reference.key {
                OwnerPartKey::Path(segments) => {
                    assert_eq!(segments.len(), 2);
                    goldens.check(&tree);
                }
                OwnerPartKey::Number(number) => {
                    assert_eq!(*number, u64::MAX);
                    goldens.check(&tree);
                }
            }
            bindings.begin();
            assert_eq!(bindings.owner_part(slot.token), None, "a departed tree's exact part token is retired");
            assert_eq!(crate::decode(&mut bindings, slot.token, skein_ui::Act::Press), None);
        }
    }
}

#[test]
fn proposed_executor_and_funder_keep_full_width_codes_and_original_period_identity() {
    let mut goldens = Goldens::new("tree/fragments/proposed_task/proposed_executor_and");
    let words = words();
    let mut b = builder();
    for executor in [
        OwnerExecutor::Agent { charter: u32::MAX },
        OwnerExecutor::Procedure { connector: u16::MAX, code: u32::MAX },
        OwnerExecutor::Person(PersonAddress::Party(u64::MAX)),
        OwnerExecutor::Person(PersonAddress::Role(u32::MAX)),
    ] {
        proposed_task::executor(&mut b, executor, words);
    }
    for funder in [
        Funder::Task(u64::MAX),
        Funder::Pool { project: u32::MAX, person: u64::MAX, period: 83 },
        Funder::Recurring { project: u32::MAX, task: u64::MAX, period: 71 },
        Funder::Period { project: u32::MAX, period: 59 },
    ] {
        proposed_task::funder(&mut b, funder, words);
    }
    let tree = finish(b);
    assert_eq!(tree.len(), 5 + proposed_task::funder_nodes() * 4);
    goldens.check(&tree);
}

#[test]
fn exact_contracts_keep_approval_state_verdict_fields_and_each_result_bound() {
    let mut goldens = Goldens::new("tree/fragments/proposed_task/exact_contracts_keep");
    let words = words();
    let mut b = builder();
    let mut bindings = JigBindings::new(16, CLIENT.client.text);
    for contract in [
        TaskContractDocument::Approval {
            state: b"opaque\xff".as_slice().into(),
            part: OwnerPart { connector: 9, kind: OwnerPartKind::Resource, key: OwnerPartKey::Number(83) },
        },
        TaskContractDocument::Report { words: u32::MAX },
        TaskContractDocument::Verdict {
            choices: list([
                VerdictChoice {
                    code: u32::MAX,
                    words: 83,
                    followups: 71,
                    fields: list([FieldRule { name: b"field/name".as_slice().into(), max: 59 }]),
                },
                VerdictChoice {
                    code: 31,
                    words: 29,
                    followups: 23,
                    fields: list([FieldRule { name: b"last\xff".as_slice().into(), max: 19 }]),
                },
            ]),
            cut: true,
        },
        TaskContractDocument::Change { connector: u16::MAX, kind: u16::MAX, words: u32::MAX },
    ] {
        proposed_task::contract(&mut b, &contract, words, &mut bindings);
    }
    let tree = finish(b);
    goldens.check(&tree);
}

#[test]
fn normal_words_adjust_extreme_public_offsets_by_saturating_the_wall_instant() {
    let mut words = words().clone();
    for offset in [i32::MAX, i32::MIN] {
        words.set_offset_minutes(offset);
        for instant in [skein_lib::Wall::EPOCH, skein_lib::Wall::from_nanos(u64::MAX)] {
            let expected = if offset > 0_i32 { u64::MAX } else { 0 };
            assert_eq!(words.adjusted_wall(instant).as_nanos(), expected);
        }
    }
    words.set_offset_minutes(60_i32);
    assert_eq!(words.adjusted_wall(skein_lib::Wall::EPOCH).as_nanos(), 3_600_000_000_000);
    words.set_offset_minutes(-60_i32);
    assert_eq!(words.adjusted_wall(skein_lib::Wall::from_nanos(7_200_000_000_000)).as_nanos(), 3_600_000_000_000);
}

fn selected_owner_language() -> skein_ui::phrases::Language {
    use crate::words::phrases::english;
    use skein_ui::phrases::Segment;
    let mut selected = english::language();
    for phrase in &mut selected.phrases {
        if phrase.id.number >= 3000
            || [
                jig_client::words::DecisionWord::FunderTask,
                jig_client::words::DecisionWord::FunderPool,
                jig_client::words::DecisionWord::FunderRecurring,
                jig_client::words::DecisionWord::FunderPeriod,
            ]
            .iter()
            .any(|word| jig_client::words::Word::Decision(*word).id() == phrase.id)
        {
            for form in &mut phrase.forms {
                for segment in &mut form.segments {
                    match segment {
                        Segment::Text(_) => {
                            *segment = Segment::Text(b"Selected owner fact: ".as_slice().into());
                            break;
                        }
                        Segment::Hole { .. } => {}
                    }
                }
            }
        }
    }
    let mut long = vec![b'L'; 1024];
    long.extend_from_slice(b": ");
    selected
        .phrases
        .iter_mut()
        .find(|phrase| phrase.id == crate::words::proposed_task_words::ProposedPhrase::ExecutorCharter.id())
        .unwrap()
        .forms[0]
        .segments[0] = Segment::Text(long.into_boxed_slice());
    selected
}
#[test]
fn normal_phrasebooks_require_and_render_exact_owner_words_in_the_selected_language() {
    use crate::words::phrases::english;
    use crate::words::phrases::jig::shapes;
    use skein_ui::phrases::{PhraseLimits, Phrasebook};
    let mut goldens = Goldens::new("tree/fragments/proposed_task/normal_phrasebooks_require");
    let selected = selected_owner_language();
    let limits = PhraseLimits {
        languages: 2,
        phrases: crate::words::owner_words::phrase_count(),
        holes: 4,
        segments: crate::tests::english_phrase_segments(2),
        text_bytes: 200_000,
        hole_bytes: CLIENT.client.text,
    };
    let mut incomplete = selected_owner_language();
    incomplete.phrases = incomplete
        .phrases
        .into_vec()
        .into_iter()
        .filter(|phrase| phrase.id != crate::words::wake_words::WakePhrase::Policy.id())
        .collect::<Vec<_>>()
        .into_boxed_slice();
    Phrasebook::load(shapes(CLIENT.client.text), Box::new([english::language(), incomplete]), &limits).unwrap_err();
    let words = crate::words::Words::new(
        Phrasebook::load(shapes(CLIENT.client.text), Box::new([english::language(), selected]), &limits).unwrap(),
        1,
        60,
    );
    let limits = crate::sizes::limits::of_words(&CLIENT, &(CLIENT).client, &words).unwrap();
    assert!(limits.text_per_node >= 2048, "owner saying capacities contribute to normal tree limits");
    let mut b = Builder::new(&limits);
    b.open(Role::Page);
    let mut bindings = JigBindings::new(limits.bindings, CLIENT.client.text);
    let task = crate::tests::tree::work_pages::recurring_task(u64::MAX, b"whole template words");
    crate::tree::cards::proposal::proposed_task(&mut b, &task, &words, &mut bindings, CLIENT.client.rows);
    b.close();
    let tree = b.finish();
    goldens.check(&tree);
}

#[test]
fn a_proposed_span_is_distinct_from_no_deadline() {
    let mut goldens = Goldens::new("tree/fragments/proposed_task/a_proposed_span");
    let words = words();
    for selector in [NewDeadline::Named, NewDeadline::Span(skein_lib::Duration::from_secs(120))] {
        let mut task = crate::tests::tree::work_pages::recurring_task(1, b"work");
        task.deadline = selector;
        let mut b = builder();
        proposed_task::authority(&mut b, &task, words, CLIENT.client.text, CLIENT.client.rows);
        let tree = finish(b);
        goldens.check(&tree);
    }
}
