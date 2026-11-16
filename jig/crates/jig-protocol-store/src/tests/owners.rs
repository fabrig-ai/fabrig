//! Fixture owners at fixed levels (protocol/store/opening.md, 3).
use crate::patching::plan::Levels;
use crate::{KeptFamily, KeptMap, Keys, Owner, PatchSet, PatchSets};
use jig_test_patches::{self as fixture, patches as app};
use jig_test_patches_jig::{self as upstream, patches as jig};
use skein_lib::Writer;
const LABEL: KeptFamily = KeptFamily {
    retired: &[],
    current: app::kept::fixture_label::TEXTS.last().expect("kept text"),
    types: &[],
    flights: &[],
    frames: None,
    name: "fixture-label",
    map: KeptMap::Store,
    texts: app::kept::fixture_label::TEXTS,
    keys: &[Keys { from: fixture::keys::LABELS_FROM, to: fixture::keys::LABELS_TO, root: fixture::keys::LABELS_ROOT }],
};
const SHELF: KeptFamily = KeptFamily {
    retired: &[],
    current: app::kept::fixture_shelf::TEXTS.last().expect("kept text"),
    types: &[],
    flights: &[],
    frames: None,
    name: "fixture-shelf",
    map: KeptMap::Store,
    texts: app::kept::fixture_shelf::TEXTS,
    keys: &[Keys {
        from: fixture::keys::SHELVES_FROM,
        to: fixture::keys::SHELVES_TO,
        root: fixture::keys::SHELVES_ROOT,
    }],
};
const TOKEN: KeptFamily = KeptFamily {
    retired: &[],
    current: app::kept::fixture_token::TEXTS.last().expect("kept text"),
    types: &[],
    flights: &[],
    frames: None,
    name: "fixture-token",
    texts: app::kept::fixture_token::TEXTS,
    map: KeptMap::Secrets,
    keys: &[Keys { from: fixture::keys::TOKENS_FROM, to: fixture::keys::TOKENS_TO, root: fixture::keys::TOKENS_ROOT }],
};
const NOTE: KeptFamily = KeptFamily {
    retired: &[],
    current: jig::kept::fixture_note::TEXTS.last().expect("kept text"),
    types: &[],
    flights: &[],
    frames: None,
    name: "fixture-note",
    map: KeptMap::Store,
    texts: jig::kept::fixture_note::TEXTS,
    keys: &[Keys { from: upstream::keys::NOTES_FROM, to: upstream::keys::NOTES_TO, root: upstream::keys::NOTES_ROOT }],
};
const APPLICATION: PatchSet = PatchSet {
    jobs: &[],
    fresh: None,
    owner: Owner::Application,
    name: fixture::NAME,
    families: &[LABEL, SHELF, TOKEN],
    level: Some(2),
    patches: &[],
};
const JIG: PatchSet = PatchSet {
    jobs: &[],
    fresh: None,
    owner: Owner::Jig,
    name: upstream::NAME,
    families: &[NOTE],
    level: Some(2),
    patches: &[],
};
pub(super) const SETS: PatchSets = PatchSets { sets: &[APPLICATION, JIG] };
pub(super) const BASE: PatchSets = PatchSets {
    sets: &[
        PatchSet {
            families: &[
                KeptFamily {
                    current: app::kept::fixture_label::TEXTS[0],
                    texts: &[app::kept::fixture_label::TEXTS[0]],
                    ..LABEL
                },
                KeptFamily {
                    current: app::kept::fixture_shelf::TEXTS[0],
                    texts: &[app::kept::fixture_shelf::TEXTS[0]],
                    ..SHELF
                },
                KeptFamily {
                    current: app::kept::fixture_token::TEXTS[0],
                    texts: &[app::kept::fixture_token::TEXTS[0]],
                    ..TOKEN
                },
            ],
            level: Some(0),
            patches: &[],
            ..APPLICATION
        },
        PatchSet {
            families: &[KeptFamily {
                current: jig::kept::fixture_note::TEXTS[0],
                texts: &[jig::kept::fixture_note::TEXTS[0]],
                ..NOTE
            }],
            level: Some(0),
            patches: &[],
            ..JIG
        },
    ],
};
pub(super) fn levels(application: u32, jig: u32) -> Levels {
    Levels { skein: None, smith: None, jig: Some(jig), application: Some(application) }
}
pub(super) fn label() -> Box<[u8]> {
    let mut out = Writer::new(7);
    out.put(&1_u16.to_be_bytes()).expect("version");
    out.put(&1_u32.to_be_bytes()).expect("caption length");
    out.put(b"a").expect("caption");
    out.finish()
}
pub(super) fn shelf(books: u32, label: &[u8]) -> Box<[u8]> {
    let size = 15_usize.checked_add(label.len()).expect("fixture size");
    let mut out = Writer::new(size);
    out.put(&1_u16.to_be_bytes()).expect("version");
    out.put(&1_u32.to_be_bytes()).expect("name length");
    out.put(b"a").expect("name");
    out.put(&books.to_be_bytes()).expect("books");
    out.put(&u32::try_from(label.len()).expect("held length").to_be_bytes()).expect("label length");
    out.put(label).expect("label");
    out.finish()
}

/// The job fixtures register their label layout in their first commit.
pub(super) const REGISTERED: PatchSets = PatchSets {
    sets: &[
        PatchSet {
            families: &[
                KeptFamily {
                    types: &[crate::ValueType {
                        number: 0,
                        root: "Label",
                        variant: None,
                        keys: &[crate::KeyRange { from: b"l", to: b"m" }],
                    }],
                    ..BASE.sets[0].families[0]
                },
                BASE.sets[0].families[1],
                BASE.sets[0].families[2],
            ],
            ..BASE.sets[0]
        },
        BASE.sets[1],
    ],
};
