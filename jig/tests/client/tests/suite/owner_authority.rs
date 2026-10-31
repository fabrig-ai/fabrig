use jig_client_domain::contract::{
    Authority, AuthorityDocument, AuthorityExecutor, AuthorityModels, Delegation, Grant, Pattern, PatternEnd,
    ResourceScope,
};
use skein_lib::{List, Wall};
fn list<T: core::fmt::Debug>(items: impl IntoIterator<Item = T>) -> List<T> {
    let items: Vec<T> = items.into_iter().collect();
    let mut result = List::with_capacity(u32::try_from(items.len()).expect("fixture size"));
    for item in items {
        result.push(item).expect("fixture capacity");
    }
    result
}
fn pattern(open: bool) -> Pattern {
    Pattern {
        segments: list([Box::from(*b"a/b\0"), Box::from([255, 0, 128, 47]), Box::from(*b"last")]),
        last: if open { PatternEnd::Open(Box::from(*b"/end")) } else { PatternEnd::Exact(Box::from(*b"/end")) },
    }
}
fn authority() -> Authority {
    Authority {
        models: Some(AuthorityModels::Named(list((0..64).map(|n| Box::from([b'm', b'o', b'd', n]))))),
        tools: u64::MAX,
        grants: list((0..3).map(|n| Grant { connector: u16::MAX, kind: n, pattern: pattern(n % 2 == 0) })),
        delegation: Delegation {
            kinds: list([
                AuthorityExecutor::Charter(u32::MAX),
                AuthorityExecutor::Procedure(u32::MAX),
                AuthorityExecutor::Role(u32::MAX),
            ]),
            tasks: u32::MAX,
            depth: u32::MAX,
        },
        spend: u64::MAX,
        deadline: Some(Wall::from_nanos(u64::MAX)),
        notes: u8::MAX,
        note_resources: list((0..3).map(|n| ResourceScope { connector: n, pattern: pattern(n % 2 != 0) })),
    }
}

#[test]
fn authority_document_cut_is_separate_from_whole_payload_and_keeps_exact_codes() {
    let payload = authority();
    let cut = AuthorityDocument { authority: payload.clone(), cut: true };
    let complete = AuthorityDocument { authority: payload.clone(), cut: false };
    assert_ne!(cut, complete);
    assert_eq!(cut.authority, complete.authority);
    assert_eq!(payload.tools, u64::MAX);
    assert_eq!(payload.notes, u8::MAX);
    assert_eq!(*payload.delegation.kinds.get(2).expect("executor"), AuthorityExecutor::Role(u32::MAX));
    assert_eq!(payload.grants.get(1).expect("grant").pattern.last, PatternEnd::Exact(Box::from(*b"/end")));
    assert_eq!(payload.grants.get(0).expect("grant").pattern.segments.get(0).expect("segment").as_ref(), b"a/b\0");
    assert_eq!(
        payload.grants.get(0).expect("grant").pattern.segments.get(1).expect("segment").as_ref(),
        &[255, 0, 128, 47]
    );
}

#[test]
fn saturated_owner_authority_renders_all_literal_boundaries() {
    use jig_client_view::authority as view;
    use jig_client_view::words::Words;
    use skein_ui::Builder;
    use skein_ui::Limits;
    use skein_ui::phrases::PhraseLimits;

    let language_limits = PhraseLimits {
        languages: 1,
        phrases: jig_client_view::document_words::phrase_count(),
        holes: 4,
        segments: 20_000,
        text_bytes: 200_000,
        hole_bytes: 4096,
    };
    let mut english = jig_client_view::phrases::english::language();
    english.wall = skein_ui::phrases::WallFormat::Nanos;
    let words = Words::new(
        skein_ui::phrases::Phrasebook::load(
            jig_client_view::phrases::jig::shapes(4096),
            Box::new([english]),
            &language_limits,
        )
        .expect("checked rendering fixture"),
        0,
        0,
    );
    let receiving = jig_client_world::test::limits().jig;
    let limits = Limits {
        nodes: 16_384,
        text_bytes: 16_777_216,
        ..jig_client_view::limits::envelope(
            &receiving,
            &jig_client_world::test::limits().client.domain,
            words.jig_capacity().expect("checked rendering fixture"),
            &jig_client_view::page::Arrangement::none(),
            0,
            0,
            0,
        )
        .expect("checked rendering fixture")
    };

    let value = AuthorityDocument { authority: authority(), cut: true };
    let mut builder = Builder::new(&limits);
    view::document_words(&mut builder, &value, &words, 4, 3);
    let tree = builder.finish();
    assert_eq!(tree.len(), 63);

    assert_eq!(
        tree.nodes().iter().flat_map(|n| &n.text).filter(|r| r.text.as_ref() == b"a/b\0").count(),
        6,
        "all six literal pattern segment boundaries retained"
    );
}
