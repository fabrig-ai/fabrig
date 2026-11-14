//! Real registered sets at open (protocol/store/layouts.md, 7).
use jig_protocol_store::{KeptFamily, PatchSet, PatchSets, Write};
use jig_store_world::{World, root};
use skein_fake_machine::Item;
use skein_heap::Meter;

// Each ceiling is its measured bytes plus a quarter, rounded up to two significant digits.
// Empty: peak 806,140, registry and read workspace retained 7,905 bytes.
const EMPTY_PEAK: u64 = 1_100_000;
const EMPTY_REGISTRY: u64 = 9_900;
// Seeded at the last kept texts: peak 3,731,403, retained share 7,905 bytes.
const SEEDED_PEAK: u64 = 4_700_000;
const SEEDED_REGISTRY: u64 = 9_900;

const SETS: PatchSets = PatchSets {
    sets: &[
        jig_protocol_store::SKEIN_PATCH_SET,
        jig_protocol_store::SMITH_PATCH_SET,
        jig_protocol_accounts::JIG_PATCH_SET,
    ],
};

fn sets(kept: bool, registered: bool) -> PatchSets {
    let sets = SETS
        .sets
        .iter()
        .map(|set| {
            let families = set
                .families
                .iter()
                .map(|family| KeptFamily {
                    current: if kept { family.texts.last().copied().unwrap_or(family.current) } else { family.current },
                    types: if registered { family.types } else { &[] },
                    ..*family
                })
                .collect::<Vec<_>>();
            PatchSet {
                families: Box::leak(families.into_boxed_slice()),
                jobs: if registered { set.jobs } else { &[] },
                ..*set
            }
        })
        .collect::<Vec<_>>();
    PatchSets { sets: Box::leak(sets.into_boxed_slice()) }
}

fn seeded(sets: PatchSets) -> Vec<Item> {
    let mut world = World::unopened(7);
    world.patch_sets(sets);
    assert_eq!(world.open(), Some(0));
    assert_eq!(world.commit(Box::new([Write::Save(root::header(1))])), Some(1));
    world
        .contents()
        .into_iter()
        .map(|(path, bytes)| match bytes {
            Some(bytes) => Item::file(&path, &bytes).mode(0o600),
            None => Item::directory(&path).mode(0o700),
        })
        .collect()
}

struct Measured {
    peak: u64,
    held: u64,
    private: u64,
}

fn measure(items: &[Item], sets: PatchSets) -> Measured {
    let meter = Meter::new();
    meter.start();
    let mut world = World::unopened(7);
    world.lay(items);
    world.patch_sets(sets);
    assert!(world.open_switched().is_some(), "{:?}", world.refusal());
    if items.is_empty() {
        assert_eq!(world.commit(Box::new([Write::Save(root::header(1))])), Some(1));
    }
    let measured = meter.end();
    // Remove the world's heap, then the KV maps. The difference is the
    // component's private heap, including its registry and read workspace.
    let component = world.into_component();
    let component_held = meter.held();
    let stores = component.into_stores();
    let private = component_held - meter.held();
    drop(stores);
    Measured { peak: measured.peak(), held: measured.held(), private }
}

#[test]
fn real_registry_open_memory() {
    let plain = sets(false, false);
    let kept = sets(true, true);
    let items = seeded(kept);
    // Identical families and serving machines without value types: subtract
    // their private heap to isolate the registry's retained share, not disk
    // copies of its durable rows, the simulator or the store's other tables.
    let baseline = measure(&[], plain).private;
    for (label, items, peak_bound, held_bound) in
        [("empty", &[][..], EMPTY_PEAK, EMPTY_REGISTRY), ("seeded", items.as_slice(), SEEDED_PEAK, SEEDED_REGISTRY)]
    {
        let measured = measure(items, SETS);
        let registry = measured.private - baseline;
        eprintln!("registry {label}: peak={}, held={}, registry={registry}", measured.peak, measured.held);
        assert!(measured.peak <= peak_bound, "{label}: open peak {} > {peak_bound}", measured.peak);
        assert!(registry <= held_bound, "{label}: registry retained {registry} > {held_bound}");
    }
}
