#![expect(clippy::disallowed_methods, reason = "ordinary tests inspect standalone trees and language data")]
use crate::tests::kit::Goldens;
use crate::tree::fragments::authority as view;
use crate::words::authority_words::AuthorityPhrase;
use skein_ui::Limits;
use skein_ui::phrases::{Hole, Segment, WallFormat};
use skein_ui::{Builder, Tree};

use jig_client::contract::{
    Authority, AuthorityDocument, AuthorityExecutor, Delegation, Grant, Pattern, PatternEnd, ResourceScope,
};
use skein_lib::{List, Wall, bytes::copy_of};
fn fixture() -> AuthorityDocument {
    let mut grants = List::with_capacity(1);
    let mut segments = List::with_capacity(3);
    for value in [&b"a/b"[..], &[255, 0, 128], &b""[..]] {
        segments.push(copy_of(value)).expect("segments fit");
    }
    grants
        .push(Grant {
            connector: u16::MAX,
            kind: u16::MAX,
            pattern: Pattern { segments, last: PatternEnd::Exact(copy_of(b"tail/")) },
        })
        .expect("grant fits");
    let mut kinds = List::with_capacity(3);
    for kind in [
        AuthorityExecutor::Charter(u32::MAX),
        AuthorityExecutor::Procedure(u32::MAX),
        AuthorityExecutor::Role(u32::MAX),
    ] {
        kinds.push(kind).expect("executors fit");
    }
    let mut resources = List::with_capacity(1);
    resources
        .push(ResourceScope {
            connector: 2,
            pattern: Pattern { segments: List::with_capacity(0), last: PatternEnd::Open(copy_of(b"*raw/")) },
        })
        .expect("resource fits");
    AuthorityDocument {
        cut: true,
        authority: Authority {
            models: None,
            tools: u64::MAX,
            grants,
            delegation: Delegation { kinds, tasks: u32::MAX, depth: u32::MAX },
            spend: u64::MAX,
            deadline: Some(Wall::from_nanos(120_000_000_000)),
            notes: u8::MAX,
            note_resources: resources,
        },
    }
}
fn english() -> crate::words::Words {
    crate::tests::english_words()
}
fn render(value: &AuthorityDocument, words: &crate::words::Words, rows: u32) -> Tree {
    let required = crate::tests::render_limits(
        &crate::tests::fixtures::Profile {
            view: crate::Config { ..(crate::tests::tree::cards::CLIENT).view },
            client: jig_client::DomainLimits { rows: 3, text: 4096, ..(crate::tests::tree::cards::CLIENT).client },
        },
        words,
    )
    .expect("capacity fits");
    let limits = Limits {
        nodes: required.nodes,
        depth: required.depth,
        text_per_node: required.text_per_node,
        text_bytes: required.text_bytes,
        drawing_bytes: 0,
        styles_per_node: 0,
        patches: 0,
        bindings: 0,
        slots: 0,
        markdown_depth: 0,
        scene_marks: 0,
        scene_bytes: 0,
    };
    let mut builder = Builder::new(&limits);
    view::document_words(&mut builder, value, words, 5, rows);
    let tree = builder.finish();

    assert!(
        tree.nodes().iter().flat_map(|n| &n.text).map(|t| t.text.len()).sum::<usize>()
            <= skein_lib::length::usize_of(required.text_bytes)
    );
    tree
}

#[test]
fn exact_authority_fragment_keeps_opaque_boundaries_full_codes_and_explicit_cut() {
    let mut goldens = Goldens::new("tree/fragments/authority/exact_authority_fragment");
    let value = fixture();
    let words = english();
    let tree = render(&value, &words, 3);
    assert_eq!(tree.len(), 27, "one region and every independent fact/segment/end leaf");
    goldens.check(&tree);
    assert_eq!(value.authority.grants.get(0).unwrap().pattern.segments.get(1).unwrap().as_ref(), &[255, 0, 128]);
    let mut complete = value;
    complete.cut = false;
    let tree = render(&complete, &words, 3);
    goldens.check(&tree);
    assert_eq!(tree.len(), 26);
    let envelope = crate::tests::render_limits(
        &crate::tests::fixtures::Profile {
            view: crate::Config { ..(crate::tests::tree::cards::CLIENT).view },
            client: jig_client::DomainLimits { rows: 3, text: 4096, ..(crate::tests::tree::cards::CLIENT).client },
        },
        &words,
    )
    .expect("configured envelope");
    assert!(tree.len() <= envelope.nodes);
}
#[test]
fn owner_authority_words_use_selected_language_and_person_offset() {
    let mut goldens = Goldens::new("tree/fragments/authority/owner_authority_words");
    let mut translated = crate::words::phrases::english::language();
    translated.wall = WallFormat::Clock;
    for phrase in &mut translated.phrases {
        for form in &mut phrase.forms {
            for segment in &mut form.segments {
                match segment {
                    Segment::Text(bytes) => *bytes = copy_of(b"selected "),
                    Segment::Hole { .. } => {}
                }
            }
        }
    }
    let words = crate::tests::checked_words(Box::new([crate::words::phrases::english::language(), translated]), 1, -1);
    let tree = render(&fixture(), &words, 3);
    goldens.check(&tree);
    assert_eq!(
        words
            .book()
            .say(words.language(), AuthorityPhrase::Deadline.id(), &[Hole::Wall(words.adjusted_wall(Wall::EPOCH))])
            .unwrap()
            .text
            .as_ref(),
        b"selected 00:00"
    );
}

const SHOWN: u32 = 2;
const GRANT_CONNECTORS: [u16; 3] = [11, 12, 13];
#[test]
fn a_long_authority_shows_rows_of_grants_and_one_partial_mark() {
    let mut goldens = Goldens::new("tree/fragments/authority/a_long_authority");
    let mut value = fixture();
    value.cut = false;
    let grant = value.authority.grants.get(0).unwrap().clone();
    value.authority.grants = List::with_capacity(3);
    for connector in GRANT_CONNECTORS {
        value.authority.grants.push(Grant { connector, ..grant.clone() }).unwrap();
    }
    value.authority.delegation.kinds.clear();
    let words = english();
    let tree = render(&value, &words, SHOWN);
    goldens.check(&tree);
    assert_eq!(value.authority.grants.len(), 3);
    assert_eq!(value.authority.grants.get(SHOWN).unwrap().connector, GRANT_CONNECTORS[2]);
}

#[test]
fn model_selectors_render_literal_names_all_empty_and_default_with_exact_budgets() {
    let mut goldens = Goldens::new("tree/fragments/authority/model_selectors_render");
    let words = english();
    let mut value = fixture();
    value.cut = false;
    let names = crate::tests::kit::list([copy_of(b"arbitrary model"), copy_of(&[b'x'; 4096])]);
    for models in [
        None,
        Some(jig_client::contract::AuthorityModels::All),
        Some(jig_client::contract::AuthorityModels::Named(List::with_capacity(0))),
        Some(jig_client::contract::AuthorityModels::Named(names)),
    ] {
        value.authority.models = models;
        let tree = render(&value, &words, 3);
        goldens.check(&tree);
    }
    let required = crate::tests::render_limits(
        &crate::tests::fixtures::Profile {
            view: crate::Config { ..(crate::tests::tree::cards::CLIENT).view },
            client: jig_client::DomainLimits { rows: 3, text: 4096, ..(crate::tests::tree::cards::CLIENT).client },
        },
        &words,
    )
    .unwrap();
    let tree = render(&value, &words, 1);
    assert!(tree.len() <= required.nodes);
    goldens.check(&tree);
}
