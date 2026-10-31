#![expect(
    clippy::disallowed_methods,
    reason = "ordinary proofs inspect public trees and selected checked language data"
)]
use crate::tests::kit::Goldens;
use crate::tree::fragments::amendment as view;
use crate::words::Words;
use crate::words::{amendment_words as data, amendment_words::AmendmentPhrase as P};
use skein_ui::Limits;
use skein_ui::phrases::{Invalid, PhraseLimits, Phrasebook, Segment, SpanUnit, WallFormat};
use skein_ui::{Builder, Tree};

use jig_client::contract::*;
use skein_lib::{Duration, List, bytes::copy_of};
fn doc(bytes: &[u8], cut: bool) -> DocumentText {
    DocumentText { bytes: Text::new(copy_of(bytes), 8).unwrap(), cut }
}
fn pattern(last: PatternEnd, capacity: u32) -> Pattern {
    let mut segments = List::with_capacity(capacity);
    if capacity != 0 {
        for bytes in [b"a/b".as_slice(), &[255, 0], b""] {
            segments.push(copy_of(bytes)).unwrap();
        }
    }
    Pattern { segments, last }
}
fn fixture() -> AmendmentState {
    let parameters = crate::tests::kit::list([
        ParameterDocument::Number { name: u32::MAX, value: u64::MAX },
        ParameterDocument::Bytes { name: 1, value: doc(b"bytes", false) },
        ParameterDocument::Resource { name: 2, connector: u16::MAX, resource: u64::MAX },
    ]);
    let inputs = crate::tests::kit::list_in(3, [9, u64::MAX]);
    let grants = crate::tests::kit::list_in(
        3,
        [Grant { connector: u16::MAX, kind: u16::MAX, pattern: pattern(PatternEnd::Exact(copy_of(b"tail")), 3) }],
    );
    let mut kinds = List::with_capacity(3);
    for kind in [
        AuthorityExecutor::Charter(u32::MAX),
        AuthorityExecutor::Procedure(u32::MAX),
        AuthorityExecutor::Role(u32::MAX),
    ] {
        kinds.push(kind).unwrap();
    }
    let resources = crate::tests::kit::list_in(
        3,
        [ResourceScope { connector: 7, pattern: pattern(PatternEnd::Open(copy_of(b"*")), 0) }],
    );
    let dependencies = crate::tests::kit::list_in(3, [u64::MAX, 7]);
    AmendmentState {
        addressed: Some(PersonAddress::Party(u64::MAX)),
        taken_by: Some(u64::MAX),
        instructions: Some(doc(b"live", false)),
        spec: SpecificationDocument {
            specification: DocumentSpecification { asks: false, words: doc(b"spec", false), parameters, inputs },
            cut: false,
        },
        authority: AuthorityDocument {
            authority: Authority {
                models: None,
                tools: u64::MAX,
                grants,
                delegation: Delegation { kinds, tasks: u32::MAX, depth: u32::MAX },
                spend: u64::MAX,
                deadline: Some(skein_lib::Wall::from_nanos(u64::MAX)),
                notes: u8::MAX,
                note_resources: resources,
            },
            cut: false,
        },
        wake: WakePolicy {
            words: WakeRule::Never,
            notices: WakeRule::Immediate,
            news: WakeRule::Batch { count: u32::MAX, age: Duration::from_nanos(u64::MAX) },
            results: ResultsWake::LastOrFailure,
            questions: true,
            answers: false,
            timers: true,
        },
        dependencies,
        cut: false,
    }
}
fn client_limits() -> crate::tests::fixtures::Profile {
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
fn phrase_limits() -> PhraseLimits {
    PhraseLimits {
        languages: 2,
        phrases: crate::words::document_words::phrase_count(),
        holes: 4,
        segments: crate::tests::english_phrase_segments(2),
        text_bytes: 100_000,
        hole_bytes: 8,
    }
}
fn normal_words() -> &'static Words {
    static WORDS: std::sync::OnceLock<Words> = std::sync::OnceLock::new();
    WORDS.get_or_init(|| words(0, false))
}
fn words(language: u16, selected: bool) -> Words {
    let mut english = crate::words::phrases::english::language();
    english.wall = WallFormat::Nanos;
    english.span_unit = SpanUnit::Nanos;
    let languages: Box<[skein_ui::phrases::Language]> = if selected {
        let translated = crate::tests::kit::translated_language(b"chosen language: ");
        Box::new([english, translated])
    } else {
        Box::new([english])
    };
    Words::new(
        Phrasebook::load(crate::words::phrases::jig::shapes(8), languages, &phrase_limits()).unwrap(),
        language,
        0,
    )
}
fn build(value: &AmendmentState, words: &Words) -> Tree {
    let f = crate::tests::render_limits(&client_limits(), words).unwrap();
    let limits = Limits {
        nodes: f.nodes,
        depth: f.depth,
        text_bytes: f.text_bytes,
        text_per_node: f.text_per_node,
        drawing_bytes: 0,
        styles_per_node: 2,
        patches: 0,
        bindings: f.bindings,
        slots: f.slots,
        markdown_depth: 2,
        scene_marks: 0,
        scene_bytes: 0,
    };

    let mut builder = Builder::new(&limits);
    view::render(&mut builder, value, words, client_limits().client.text, client_limits().client.rows);
    let tree = builder.finish();
    assert!(tree.len() <= f.nodes);
    assert!(tree.nodes().iter().all(|node| crate::tests::text_bytes(node) <= f.text_per_node));
    assert!(
        tree.nodes().iter().map(|node| u64::from(crate::tests::text_bytes(node))).sum::<u64>()
            <= u64::from(f.text_bytes)
    );
    assert!(tree.nodes().iter().all(|node| node.binding.is_none() && node.slot.is_none()));

    tree
}

#[test]
fn owner_amendment_instruction_absence_empty_and_each_loss_keep_context_without_default_substitution() {
    let mut goldens = Goldens::new("tree/fragments/amendment/owner_amendment_instruction");
    let variants =
        [None, Some(doc(b"", false)), Some(doc(b"live", false)), Some(doc(b"", true)), Some(doc(b"live", true))];
    let mut states = List::with_capacity(5);
    let mut trees: List<Tree> = List::with_capacity(5);
    for instructions in variants {
        let state = AmendmentState { instructions, ..fixture() };
        let tree = build(&state, normal_words());
        goldens.check(&tree);
        for previous in &states {
            assert_ne!(&state, previous);
        }
        for previous in &trees {
            assert_ne!(tree.nodes(), previous.nodes(), "rendered None/empty/cut facts stay distinguishable");
        }
        states.push(state).unwrap();
        trees.push(tree).unwrap();
    }
    assert_eq!(
        states.get(0).unwrap().spec,
        states.get(2).unwrap().spec,
        "retained None and live effective share other facts"
    );
    assert_eq!(crate::tree::fragments::instructions::whole(states.get(0).unwrap().instructions.as_ref()), None);
    assert_eq!(
        crate::tree::fragments::instructions::whole(states.get(1).unwrap().instructions.as_ref()).unwrap().bytes(),
        b""
    );
    assert_eq!(crate::tree::fragments::instructions::whole(states.get(3).unwrap().instructions.as_ref()), None);
    assert_eq!(states.len(), 5, "the consumer can retain every independent state");
}
#[test]
fn owner_amendment_each_text_spec_authority_and_dependency_loss_is_independent() {
    let mut goldens = Goldens::new("tree/fragments/amendment/owner_amendment_each");
    for flag in 0_u32..5 {
        let mut value = fixture();
        match flag {
            0 => value.spec.specification.words.cut = true,
            1 => {
                if let ParameterDocument::Bytes { value, .. } = value.spec.specification.parameters.get_mut(1).unwrap()
                {
                    value.cut = true;
                }
            }
            2 => value.spec.cut = true,
            3 => value.authority.cut = true,
            4 => value.cut = true,
            _ => unreachable!(),
        }
        let tree = build(&value, normal_words());
        goldens.check(&tree);
        assert_eq!(value.spec.whole().is_none(), flag <= 2);
        assert!(crate::tree::fragments::instructions::whole(value.instructions.as_ref()).is_some());

        assert_eq!(value.dependencies.get(0), Some(&u64::MAX));
        assert_eq!(value.spec.specification.parameters.capacity(), 3);
        assert_eq!(value.spec.specification.inputs.capacity(), 3);
    }
}
#[test]
fn owner_amendment_address_taker_spec_wake_and_opaque_authority_keep_full_independent_values() {
    let mut goldens = Goldens::new("tree/fragments/amendment/owner_amendment_address");
    let words = normal_words();
    let mut value = fixture();
    for address in [PersonAddress::Party(u64::MAX), PersonAddress::Role(u32::MAX)] {
        value.addressed = Some(address);
        let tree = build(&value, words);

        goldens.check(&tree);
        let input_order: Box<[&[u8]]> = tree
            .nodes()
            .iter()
            .flat_map(|n| &n.text)
            .filter(|r| r.text.starts_with(b"Input task "))
            .map(|r| r.text.as_ref())
            .collect();
        assert_eq!(input_order.as_ref(), [b"Input task 9".as_slice(), b"Input task 18446744073709551615"].as_slice());

        assert_eq!(value.clone().addressed, Some(address));
        assert_eq!(
            value.authority.authority.grants.get(0).unwrap().pattern.segments.get(1).unwrap().as_ref(),
            &[255, 0]
        );
        assert_eq!(value.spec.specification.inputs.get(0), Some(&9));
        assert_eq!(value.spec.specification.inputs.get(1), Some(&u64::MAX));
        assert_eq!(
            value.spec.specification.parameters.get(0),
            Some(&ParameterDocument::Number { name: u32::MAX, value: u64::MAX })
        );
        assert_eq!(
            value.spec.specification.parameters.get(2),
            Some(&ParameterDocument::Resource { name: 2, connector: u16::MAX, resource: u64::MAX })
        );
    }
    value.addressed = None;
    value.taken_by = None;
    let tree = build(&value, words);
    goldens.check(&tree);
    assert_wake_fields(&mut value, words);
}

fn assert_wake_fields(value: &mut AmendmentState, words: &Words) {
    let mut goldens = Goldens::new("tree/fragments/amendment/assert_wake_fields");
    for results in [ResultsWake::Never, ResultsWake::Each, ResultsWake::LastOrFailure] {
        for switches in 0_u8..8 {
            value.wake.results = results;
            value.wake.questions = switches & 1 != 0;
            value.wake.answers = switches & 2 != 0;
            value.wake.timers = switches & 4 != 0;
            let tree = build(value, words);
            assert_eq!(tree.nodes().iter().filter(|n| n.role == skein_ui::Role::Region).count(), 2);

            goldens.check(&tree);
        }
    }
}

#[test]
fn owner_amendment_selected_normal_words_preserve_checked_captions_numbers_and_nested_fragments() {
    let mut goldens = Goldens::new("tree/fragments/amendment/owner_amendment_selected");
    let selected = words(1, true);
    let value = fixture();
    let tree = build(&value, &selected);
    goldens.check(&tree);
    let mut lost = value;
    lost.cut = true;
    lost.authority.cut = true;
    lost.spec.cut = true;
    lost.instructions.as_mut().unwrap().cut = true;
    let tree = build(&lost, &selected);
    goldens.check(&tree);
    for phrase in data::ALL {
        assert!(selected.book().capacity(phrase.id()).is_some());
    }
    let cloned = selected.clone();
    assert_eq!(
        build(&lost, &cloned).nodes(),
        tree.nodes(),
        "normal Words Clone retains the complete checked owner composition"
    );
}
#[test]
fn owner_amendment_checked_book_refuses_missing_duplicate_and_wrong_number_holes() {
    let limits = phrase_limits();
    let mut language = crate::words::phrases::english::language();
    language.phrases = language.phrases.into_vec().into_iter().filter(|p| p.id != P::State.id()).collect();
    assert_eq!(
        Phrasebook::load(crate::words::phrases::jig::shapes(8), Box::new([language]), &limits).err(),
        Some(Invalid::Missing { id: P::State.id() })
    );
    let mut language = crate::words::phrases::english::language();
    language.phrases.iter_mut().find(|p| p.id == P::State.id()).unwrap().id = P::Party.id();
    assert_eq!(
        Phrasebook::load(crate::words::phrases::jig::shapes(8), Box::new([language]), &limits).err(),
        Some(Invalid::Duplicate { id: P::Party.id() })
    );
    let mut language = crate::words::phrases::english::language();
    language.phrases.iter_mut().find(|p| p.id == P::Party.id()).unwrap().forms[0].segments =
        Box::new([Segment::Text(copy_of(b"wrong hole"))]);
    assert_eq!(
        Phrasebook::load(crate::words::phrases::jig::shapes(8), Box::new([language]), &limits).err(),
        Some(Invalid::Holes { id: P::Party.id() })
    );
}
