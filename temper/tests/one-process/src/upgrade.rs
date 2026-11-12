//! Real owner sets and a mark-only upgrade (temper/docs/design/protocol/baseline.md, 4 and 8).
pub mod check;
mod maps;
pub mod story;
use crate::{fixtures, referee, world};
use jig_protocol_store::{KeptMap, Owner, PatchSets, codec, keys};
use std::{cell::RefCell, collections::BTreeSet, path::Path, rc::Rc};

/// The real owners with one extra application patch that changes no family.
pub const FIXTURE_SETS: PatchSets = PatchSets {
    sets: &[
        jig_protocol_store::SKEIN_PATCH_SET,
        jig_protocol_store::SMITH_PATCH_SET,
        jig_protocol_accounts::JIG_PATCH_SET,
        jig_protocol_store::PatchSet {
            patches: &[
                include_str!("../../../crates/temper-protocol-store/patches/0001.patch"),
                "patch 1\nowner temper 2 after jig 1\nmark fresh-windows\n",
            ],
            fresh: Some("fresh-windows"),
            ..temper_protocol_store::PATCH_SET
        },
    ],
};

/// Every non-reserved record belongs to exactly one family. Report all bad keys,
/// including both family names when ranges overlap.
pub fn golden_ranges(directory: &Path) {
    let mut disk = crate::golden::files::load(directory);
    let maps = maps::Maps::read(&mut disk);
    let reserved = [
        (keys::encode_own(&keys::Own::Format, 64).expect("format key"), "format"),
        (keys::encode_own(&keys::Own::Layouts, 64).expect("layouts key"), "layouts"),
        (keys::encode_own(&keys::Own::Wires, 64).expect("wires key"), "wires"),
        (keys::installation(64).expect("installation key"), "installation"),
    ];
    let mut seen = BTreeSet::new();
    let mut bad = Vec::new();
    for (map, rows) in [(KeptMap::Store, &maps.store), (KeptMap::Secrets, &maps.secrets)] {
        for (key, value) in rows {
            let mut families = Vec::new();
            for set in temper_service::store::PATCH_SETS.sets {
                let relative = if set.owner == Owner::Application {
                    key.strip_prefix(match map {
                        KeptMap::Store => &[keys::APPLICATION][..],
                        KeptMap::Secrets => &[2, 0][..],
                    })
                } else {
                    Some(key.as_slice())
                };
                let Some(relative) = relative else { continue };
                for family in set.families.iter().filter(|family| family.map == map) {
                    for range in family.keys {
                        if range.from <= relative && relative < range.to {
                            families.push(family.name);
                        }
                    }
                }
            }
            let own = (map == KeptMap::Store)
                .then(|| reserved.iter().find(|(reserved, _)| reserved.as_ref() == key.as_slice()))
                .flatten();
            if let Some((_, name)) = own {
                seen.insert(*name);
                assert!(families.is_empty(), "reserved {name} key {key:02x?} in {families:?}");
                match *name {
                    "format" => {
                        codec::component::Format::decode(value).expect("format at its own key");
                    }
                    "layouts" => {
                        codec::component::Layouts::decode(value).expect("layouts at their own key");
                    }
                    "wires" => {
                        jig_protocol_store::component_v3::Wires::decode(
                            &jig_protocol_store::component_v3::CEILINGS,
                            &mut skein_lib::Reader::new(value),
                        )
                        .expect("wires at their own key");
                    }
                    "installation" => {
                        codec::accounts::Installation::decode(value).expect("installation at its own key");
                    }
                    _ => unreachable!("listed reserved row"),
                }
            } else if families.len() != 1 {
                bad.push(format!("{map:?} {key:02x?}: {families:?}"));
            }
        }
    }
    assert_eq!(seen, reserved.iter().map(|(_, name)| *name).collect(), "all component-owned rows present");
    assert!(bad.is_empty(), "keys in none or multiple ranges:\n{}", bad.join("\n"));
    eprintln!("golden ranges: {} store keys (4 reserved), {} secrets keys", maps.store.len(), maps.secrets.len());
}

fn assert_levels(maps: &maps::Maps, sets: &PatchSets) {
    assert_eq!(maps.format().levels, jig_protocol_store::levels(sets));
}

fn chat_window(maps: &maps::Maps) -> jig_domain_core::RunProof {
    let key =
        keys::encode(&jig_domain_core::Key::Core(jig_domain_core::CoreKey::RunProof(1)), 64).expect("chat proof key");
    let bytes = maps.store.get(key.as_ref()).expect("chat proof");
    let jig_domain_core::Record::Core(jig_domain_core::CoreRecord::RunProof(proof)) =
        codec::decode(&keys::decode(&key).expect("chat key"), bytes, &fixtures::codec(), fixtures::store_names())
            .expect("chat proof decodes")
    else {
        panic!("chat proof record")
    };
    proof
}

/// A signal-stopped chat keeps its live window at the real owners' levels. An equal open
/// patches nothing; the marked upgrade closes the window and switches once.
pub fn fresh_windows() {
    let scenario = world::Story::ChatReadThenWait { words: b"Hello" };
    let run = Rc::new(RefCell::new(referee::Run::default()));
    let settings = fixtures::calm(fixtures::SEED);
    let (first, mut disk, store) = world::with_patch_sets(temper_service::store::PATCH_SETS, || {
        world::run_phase(
            settings,
            scenario,
            world::Phase::Story,
            temper_playground::machine::fresh(),
            referee::Store::new(),
            run.clone(),
        )
    });
    assert!(first.drained && first.end == Some(temper_service::End::Drained));
    assert!(first.patched.is_empty(), "first start patches nothing");
    let before = maps::Maps::read(&mut disk);
    assert_levels(&before, &temper_service::store::PATCH_SETS);
    let proof = chat_window(&before);
    let window = proof.window.expect("chat has a live window");
    assert!(!window.closed, "signal preserves the live chat window");

    let (equal, mut disk, store) = world::with_patch_sets(temper_service::store::PATCH_SETS, || {
        world::run_phase(settings, scenario, world::Phase::SignIn, disk, store, run.clone())
    });
    assert!(equal.drained);
    assert!(equal.patched.is_empty(), "equal baseline levels patch nothing");
    let equal_maps = maps::Maps::read(&mut disk);
    assert_eq!(equal_maps.active, before.active, "equal open does not switch");
    assert_levels(&equal_maps, &temper_service::store::PATCH_SETS);
    assert!(!chat_window(&equal_maps).window.expect("window retained").closed);

    let (patched, mut disk, store) = world::with_patch_sets(FIXTURE_SETS, || {
        world::run_phase(settings, scenario, world::Phase::SignIn, disk, store, run.clone())
    });
    assert!(patched.drained);
    let patch = jig_protocol_store::levels(&FIXTURE_SETS).application.expect("fixture application level");
    assert_eq!(
        patched.patched,
        vec![(Owner::Application, patch, jig_protocol_store::Counts { records: 0, commits: 0 })]
    );
    let after = maps::Maps::read(&mut disk);
    assert_eq!(after.active, before.active + 1, "one copy switched");
    assert_levels(&after, &FIXTURE_SETS);
    let proof = chat_window(&after);
    assert!(proof.crossed);
    let closed = proof.window.expect("window retained closed");
    assert!(closed.closed);
    assert_eq!((closed.attempt, closed.turn), (window.attempt, window.turn), "same window is closed, not replaced");
    assert_eq!(after.secrets, equal_maps.secrets, "mark-only patch preserves secret records");

    let (equal, mut disk, _) = world::with_patch_sets(FIXTURE_SETS, || {
        world::run_phase(settings, scenario, world::Phase::SignIn, disk, store, run)
    });
    assert!(equal.drained);
    assert!(equal.patched.is_empty(), "equal fixture levels patch nothing");
    let again = maps::Maps::read(&mut disk);
    assert_levels(&again, &FIXTURE_SETS);
    assert_eq!(again.active, after.active, "equal fixture open does not switch");
}
