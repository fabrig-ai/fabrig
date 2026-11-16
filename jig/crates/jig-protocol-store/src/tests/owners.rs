//! Fixture owners at fixed levels (protocol/store/opening.md, 3).
use crate::upgrade::admission::Levels;
use crate::{KeptFamily, KeptMap, Keys, Owner, OwnerSet, OwnerSets};
use jig_test_layouts::{self as fixture, records as app};
use jig_test_patches_jig::{self as upstream, patches as jig};
use skein_lib::Writer;
const LABEL: KeptFamily = KeptFamily {
    retired: &[],
    current: app::LABEL_CURRENT,
    types: &[],
    frames: None,
    name: "fixture-label",
    map: KeptMap::Store,
    keys: &[Keys { from: fixture::keys::LABELS_FROM, to: fixture::keys::LABELS_TO, root: fixture::keys::LABELS_ROOT }],
};
const SHELF: KeptFamily = KeptFamily {
    retired: &[],
    current: app::SHELF_CURRENT,
    types: &[],
    frames: None,
    name: "fixture-shelf",
    map: KeptMap::Store,
    keys: &[Keys {
        from: fixture::keys::SHELVES_FROM,
        to: fixture::keys::SHELVES_TO,
        root: fixture::keys::SHELVES_ROOT,
    }],
};
const TOKEN: KeptFamily = KeptFamily {
    retired: &[],
    current: app::TOKEN_CURRENT,
    types: &[],
    frames: None,
    name: "fixture-token",
    map: KeptMap::Secrets,
    keys: &[Keys { from: fixture::keys::TOKENS_FROM, to: fixture::keys::TOKENS_TO, root: fixture::keys::TOKENS_ROOT }],
};
const NOTE: KeptFamily = KeptFamily {
    retired: &[],
    current: jig::kept::fixture_note::TEXTS.last().expect("kept text"),
    types: &[],
    frames: None,
    name: "fixture-note",
    map: KeptMap::Store,
    keys: &[Keys { from: upstream::keys::NOTES_FROM, to: upstream::keys::NOTES_TO, root: upstream::keys::NOTES_ROOT }],
};
const APPLICATION: OwnerSet = OwnerSet {
    jobs: &[],
    owner: Owner::Application,
    name: fixture::NAME,
    families: &[LABEL, SHELF, TOKEN],
    level: Some(2),
};
const JIG: OwnerSet =
    OwnerSet { jobs: &[], owner: Owner::Jig, name: upstream::NAME, families: &[NOTE], level: Some(2) };
pub(super) const SETS: OwnerSets = OwnerSets { sets: &[APPLICATION, JIG] };
pub(super) const BASE: OwnerSets = OwnerSets {
    sets: &[
        OwnerSet {
            families: &[
                KeptFamily { current: app::LABEL, ..LABEL },
                KeptFamily { current: app::SHELF, ..SHELF },
                KeptFamily { current: app::TOKEN, ..TOKEN },
            ],
            level: Some(0),
            ..APPLICATION
        },
        OwnerSet {
            families: &[KeptFamily { current: jig::kept::fixture_note::TEXTS[0], ..NOTE }],
            level: Some(0),
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
pub(super) const REGISTERED: OwnerSets = OwnerSets {
    sets: &[
        OwnerSet {
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
