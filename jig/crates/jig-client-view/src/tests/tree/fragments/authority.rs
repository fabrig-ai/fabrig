#![expect(clippy::disallowed_methods, reason = "ordinary tests inspect standalone trees and language data")]
use crate::tree::fragments::authority as view;
use crate::words::authority_words::AuthorityPhrase;
use skein_ui::Limits;
use skein_ui::phrases::{Hole, Segment, WallFormat};
use skein_ui::{Builder, Tree};

use jig_client::contract::{
    OwnerAuthority, OwnerAuthorityDocument, OwnerAuthorityExecutor, OwnerDelegation, OwnerGrant, OwnerPattern,
    OwnerPatternEnd, OwnerResourceScope,
};
use skein_lib::{List, Wall, bytes::copy_of};
fn fixture() -> OwnerAuthorityDocument {
    let mut grants = List::with_capacity(1);
    let mut segments = List::with_capacity(3);
    for value in [&b"a/b"[..], &[255, 0, 128], &b""[..]] {
        segments.push(copy_of(value)).expect("segments fit");
    }
    grants
        .push(OwnerGrant {
            connector: u16::MAX,
            kind: u16::MAX,
            pattern: OwnerPattern { segments, last: OwnerPatternEnd::Exact(copy_of(b"tail/")) },
        })
        .expect("grant fits");
    let mut kinds = List::with_capacity(3);
    for kind in [
        OwnerAuthorityExecutor::Charter(u32::MAX),
        OwnerAuthorityExecutor::Procedure(u32::MAX),
        OwnerAuthorityExecutor::Role(u32::MAX),
    ] {
        kinds.push(kind).expect("executors fit");
    }
    let mut resources = List::with_capacity(1);
    resources
        .push(OwnerResourceScope {
            connector: 2,
            pattern: OwnerPattern { segments: List::with_capacity(0), last: OwnerPatternEnd::Open(copy_of(b"*raw/")) },
        })
        .expect("resource fits");
    OwnerAuthorityDocument {
        cut: true,
        authority: OwnerAuthority {
            models: None,
            tools: u64::MAX,
            grants,
            delegation: OwnerDelegation { kinds, tasks: u32::MAX, depth: u32::MAX },
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
fn render(value: &OwnerAuthorityDocument, words: &crate::words::Words, rows: u32) -> Tree {
    let required =
        crate::tests::render_limits(&crate::Limits { rows: 3, text: 4096, ..crate::tests::tree::cards::CLIENT }, words)
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
            <= usize::try_from(required.text_bytes).expect("bytes")
    );
    tree
}
fn has(tree: &Tree, bytes: &[u8]) -> bool {
    tree.nodes().iter().any(|node| node.text.iter().any(|run| run.text.as_ref() == bytes))
}
#[test]
fn exact_authority_fragment_keeps_opaque_boundaries_full_codes_and_explicit_cut() {
    let value = fixture();
    let words = english();
    let tree = render(&value, &words, 3);
    assert_eq!(tree.len(), 27, "one region and every independent fact/segment/end leaf");
    for text in [
        &b"a/b"[..],
        &b"\\xFF\\x00\\x80"[..],
        &b"tail/"[..],
        &b"*raw/"[..],
        &b"Exact ending"[..],
        &b"Open ending"[..],
        &b"Role: 4294967295"[..],
        &b"Charter: 4294967295"[..],
        &b"Procedure: 4294967295"[..],
        &b"Tool bits: 18446744073709551615"[..],
        &b"Note scope bits: 255"[..],
        &b"More are held than shown"[..],
    ] {
        assert!(has(&tree, text), "exact independent fact retained");
    }
    assert_eq!(value.authority.grants.get(0).unwrap().pattern.segments.get(1).unwrap().as_ref(), &[255, 0, 128]);
    let mut complete = value;
    complete.cut = false;
    let tree = render(&complete, &words, 3);
    assert!(!has(&tree, b"More are held than shown"));
    assert_eq!(tree.len(), 26);
    let envelope = crate::tests::render_limits(
        &crate::Limits { rows: 3, text: 4096, ..crate::tests::tree::cards::CLIENT },
        &words,
    )
    .expect("configured envelope");
    assert!(tree.len() <= envelope.nodes);
}
#[test]
fn owner_authority_words_use_selected_language_and_person_offset() {
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
    assert!(has(&tree, b"selected 00:01"));
    assert!(has(&tree, b"selected 4294967295"));
    assert!(!has(&tree, b"Exact ending"));
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
    let mut value = fixture();
    value.cut = false;
    let grant = value.authority.grants.get(0).unwrap().clone();
    value.authority.grants = List::with_capacity(3);
    for connector in GRANT_CONNECTORS {
        value.authority.grants.push(OwnerGrant { connector, ..grant.clone() }).unwrap();
    }
    value.authority.delegation.kinds.clear();
    let words = english();
    let tree = render(&value, &words, SHOWN);
    assert!(has(&tree, b"Connector: 11"));
    assert!(has(&tree, b"Connector: 12"));
    assert!(!has(&tree, b"Connector: 13"));
    assert_eq!(
        tree.nodes()
            .iter()
            .flat_map(|node| &node.text)
            .filter(|run| run.text.as_ref() == b"More are held than shown")
            .count(),
        1
    );
    assert_eq!(value.authority.grants.len(), 3);
    assert_eq!(value.authority.grants.get(SHOWN).unwrap().connector, GRANT_CONNECTORS[2]);
}

#[test]
fn model_selectors_render_literal_names_all_empty_and_default_with_exact_budgets() {
    let words = english();
    let mut value = fixture();
    value.cut = false;
    let mut names = List::with_capacity(2);
    names.push(copy_of(b"arbitrary model")).unwrap();
    names.push(copy_of(&[b'x'; 4096])).unwrap();
    for (models, label) in [
        (None, b"Keep default models".as_slice()),
        (Some(jig_client::contract::AuthorityModels::All), b"All models"),
        (Some(jig_client::contract::AuthorityModels::Named(List::with_capacity(0))), b"No models"),
        (Some(jig_client::contract::AuthorityModels::Named(names)), b"Allowed models"),
    ] {
        value.authority.models = models;
        let tree = render(&value, &words, 3);
        assert!(has(&tree, label));
        if let Some(jig_client::contract::AuthorityModels::Named(names)) = &value.authority.models {
            for name in names {
                assert!(has(&tree, name));
            }
        }
    }
    let required = crate::tests::render_limits(
        &crate::Limits { rows: 3, text: 4096, ..crate::tests::tree::cards::CLIENT },
        &words,
    )
    .unwrap();
    let tree = render(&value, &words, 1);
    assert!(tree.len() <= required.nodes);
    assert!(has(&tree, b"More are held than shown"));
}
