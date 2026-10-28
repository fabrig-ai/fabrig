//! Application-owned flight rows in the existing label/token fixture (store.md 10.3).
use super::limits;
use crate::{World, root};
use jig_protocol_store::{KeptFamily, Keys, PatchSet, PatchSets, Write};
use jig_test_patches::patches as app;

pub const SEED: u64 = 930;
/// store.md 10.3 names at most 64 keys, while counting every record.
pub const NAMED: u32 = 64;
pub const LABEL_NAME: &str = "fixture-label";
pub const ROW: &[u8] = b"\x00\x01\x00\x00\x00\x04sent";
const RANGES: &[Keys] = &[Keys { from: b"l/f/", to: b"l/g/", root: "Label" }];
const LABEL: KeptFamily = KeptFamily { flights: RANGES, ..super::BASE.sets[0].families[0] };
const TOKEN: KeptFamily = super::BASE.sets[0].families[2];
const APPLICATION: PatchSet = PatchSet { families: &[LABEL, TOKEN], ..super::BASE.sets[0] };
pub const BEFORE: PatchSets = PatchSets { sets: &[APPLICATION] };
pub const CHANGES_FLIGHT: PatchSets = PatchSets {
    sets: &[PatchSet {
        families: &[KeptFamily { texts: app::kept::fixture_label::TEXTS, ..LABEL }, TOKEN],
        patches: &[app::PATCHES[0]],
        ..APPLICATION
    }],
};
// The existing token block, at level 1 to isolate the other family from labels.
const TOKEN_PATCH: &str =
    "patch 1\nowner fixture 1\n\nfamily fixture-token 1 to 2\nadd Token.expires option u64 none\n";
pub const CHANGES_OTHER: PatchSets = PatchSets {
    sets: &[PatchSet {
        families: &[LABEL, KeptFamily { texts: app::kept::fixture_token::TEXTS, ..TOKEN }],
        patches: &[TOKEN_PATCH],
        ..APPLICATION
    }],
};
pub const NO_RANGES: PatchSets = PatchSets {
    sets: &[PatchSet {
        families: &[KeptFamily { flights: &[], ..CHANGES_FLIGHT.sets[0].families[0] }, TOKEN],
        ..CHANGES_FLIGHT.sets[0]
    }],
};

#[must_use]
pub fn key(number: u32) -> Box<[u8]> {
    format!("l/f/{number:03}").into_bytes().into_boxed_slice()
}

#[must_use]
pub fn committed(count: u32) -> World {
    let mut world = World::unopened_observed(SEED, limits(), None);
    world.patch_sets(BEFORE);
    assert_eq!(world.open(), Some(0));
    // Each commit retains one slot for the deployment header.
    let per_commit = limits().journal.writes - 1;
    let mut number = 0_u64;
    for start in (0..count).step_by(usize::try_from(per_commit).expect("fixture writes")) {
        number += 1;
        let mut writes = vec![Write::Save(root::header(number))];
        writes
            .extend((start..count.min(start + per_commit)).map(|row| Write::Put { key: key(row), value: ROW.into() }));
        assert_eq!(world.commit(writes.into_boxed_slice()), Some(number));
    }
    world.crash(SEED);
    world
}
