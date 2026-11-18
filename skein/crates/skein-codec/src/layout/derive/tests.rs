//! Interning preserves the first-discovery worklists (data/codec/layouts.md, 2).
use super::*;

const FIRST: Place = Place { decl: Span { start: 0, end: 1 }, part: None, old: false };
const LATER: Place = Place { decl: Span { start: 2, end: 3 }, part: None, old: true };
const HERE: Location = Location { text: 0, at: At::Decl(DeclIx::new(0)), variant: None };
const KEY: Key = Key { old: HERE, new: HERE, root: false };

#[test]
fn repeated_requests_keep_their_first_indices_and_places() {
    let mut builder = Builder::new(None, false, None, Policy::Strict);
    assert_eq!(builder.request(KEY, FIRST), 0);
    assert_eq!(builder.request_read(HERE, FIRST), 0);
    let next = Key { new: Location { text: 1, ..HERE }, ..KEY };
    assert_eq!(builder.request(next, FIRST), 1);
    assert_eq!(builder.request(KEY, LATER), 0);
    assert_eq!(builder.request_read(HERE, LATER), 0);
    assert_eq!(builder.request(next, LATER), 1);
    assert_eq!(builder.keys.as_slice(), &[KEY, next]);
    assert_eq!(builder.read_keys.as_slice(), &[HERE]);
    assert_eq!(builder.plan.nodes.len(), 2);
    assert_eq!(builder.plan.reads.len(), 1);
    assert_eq!(builder.places.get(0).expect("first place").decl, FIRST.decl);
    assert_eq!(builder.read_places.get(0).expect("first read place").decl, FIRST.decl);
}

#[test]
fn interning_distinguishes_texts_roots_types_and_variants() {
    let mut builder = Builder::new(None, false, None, Policy::Strict);
    let variant = Location { variant: Some(VariantIx::new(0)), ..HERE };
    let cases = [
        (KEY, 0),
        (Key { new: Location { text: 1, ..HERE }, ..KEY }, 0),
        (Key { old: Location { text: 1, ..HERE }, ..KEY }, 1),
        (Key { old: Location { at: At::Type(TypeIx::new(0)), ..HERE }, ..KEY }, 2),
        (Key { new: Location { at: At::Type(TypeIx::new(0)), ..HERE }, ..KEY }, 0),
        (Key { old: variant, ..KEY }, 3),
        (Key { new: variant, ..KEY }, 0),
        (Key { root: true, ..KEY }, 0),
        (Key { old: Location { variant: Some(VariantIx::new(1)), ..HERE }, ..KEY }, 4),
        (Key { new: Location { variant: Some(VariantIx::new(1)), ..HERE }, ..KEY }, 0),
    ];
    for (index, (key, read)) in cases.iter().enumerate() {
        let index = u32::try_from(index).expect("fixture index");
        assert_eq!(builder.request(*key, FIRST), index, "{key:?}");
        assert_eq!(builder.request_read(key.old, FIRST), *read, "{key:?}");
    }
    assert_eq!(builder.keys.len(), u32::try_from(cases.len()).expect("fixture keys"));
}
