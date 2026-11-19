//! Real registered sets at open (protocol/store/layouts.md, 7).
use super::upgrades::commits;
use jig_protocol_store::{KeptFamily, KeptMap, KeyRange, Keys, Owner, OwnerSet, OwnerSets, ValueType, Write};
use jig_store_world::{World, root};
use jig_test_layouts::layouts::{CURRENT, KEPT};
use skein_fake_machine::Item;
use skein_heap::Meter;

// Ceilings start a quarter above measured bytes, rounded up to two significant digits,
// and move when a real change passes them (skein's testing-strategy.md, 6).
// Empty: peak 1,250,951, registry and read workspace retained 8,749 bytes.
const EMPTY_PEAK: u64 = 1_600_000;
const EMPTY_REGISTRY: u64 = 9_900;
// Seeded, converting open at the fixture's 39,054,932-byte value bound: peak 78,552,960 bytes,
// registry and read workspace retained 10,666 bytes. The ceiling keeps about 24.8% headroom.
// The conversion buffer grows per read; at four times the bound, the registry retains the same bytes.
const SEEDED_PEAK: u64 = 98_000_000;
const SEEDED_REGISTRY: u64 = 2_400_000;

const SETS: OwnerSets = OwnerSets {
    sets: &[
        jig_protocol_store::SKEIN_SET,
        jig_protocol_store::SMITH_SET,
        jig_protocol_accounts::JIG_SET,
        OwnerSet {
            owner: Owner::Application,
            name: "fixture-layouts",
            level: Some(1),
            jobs: &[],
            families: &[KeptFamily {
                name: "fixture-layouts",
                current: CURRENT,
                retired: &[],
                map: KeptMap::Store,
                frames: None,
                keys: &[Keys { from: b"a", to: b"b", root: "Growing" }],
                types: &[ValueType {
                    number: 0,
                    root: "Growing",
                    variant: None,
                    keys: &[KeyRange { from: b"a", to: b"b" }],
                }],
            }],
        },
    ],
};

// A source binary registers these fixture layouts in its first commit.
fn kept_text(family: &KeptFamily) -> &'static str {
    match family.name {
        "fixture-layouts" => KEPT,
        "jig-store-people" => include_str!("fixtures/jig-store-people-v17.schema"),
        "jig-store-notes" => include_str!("fixtures/jig-store-notes-v1.schema"),
        "jig-store-accounts" => include_str!("fixtures/jig-store-accounts-v2.schema"),
        "jig-store-settings" => include_str!("fixtures/jig-store-settings-v3.schema"),
        _ => family.current,
    }
}

fn kept_types(family: &KeptFamily) -> &'static [ValueType] {
    let text = kept_text(family);
    let schema = skein_codec::schema::parse(text.as_bytes()).expect("kept family schema");
    let mut kept: Vec<ValueType> = Vec::new();
    for ty in family.types {
        if schema.root(ty.root.as_bytes(), ty.variant.map(str::as_bytes)).is_some() {
            kept.push(*ty);
        } else {
            let previous = kept.last_mut().expect("new types append after kept types");
            let mut ranges = previous.keys.to_vec();
            let last = ranges.last_mut().expect("kept type's range");
            assert_eq!(last.to, ty.keys.first().expect("new type's first range").from);
            last.to = ty.keys.last().expect("new type's last range").to;
            previous.keys = Box::leak(ranges.into_boxed_slice());
        }
    }
    Box::leak(kept.into_boxed_slice())
}

fn sets(kept: bool, registered: bool) -> OwnerSets {
    let sets = SETS
        .sets
        .iter()
        .map(|set| {
            let families = set
                .families
                .iter()
                .map(|family| KeptFamily {
                    retired: if registered { family.retired } else { &[] },
                    current: if kept { kept_text(family) } else { family.current },
                    types: if !registered {
                        &[]
                    } else if kept {
                        kept_types(family)
                    } else {
                        family.types
                    },
                    ..*family
                })
                .collect::<Vec<_>>();
            OwnerSet {
                families: Box::leak(families.into_boxed_slice()),
                jobs: if registered { set.jobs } else { &[] },
                ..*set
            }
        })
        .collect::<Vec<_>>();
    OwnerSets { sets: Box::leak(sets.into_boxed_slice()) }
}

// The source binary's registered header predates the restart wait field.
fn kept_header(header: &jig_domain_core::Record) -> Box<[u8]> {
    let family =
        jig_protocol_store::STORE_FAMILIES.iter().find(|family| family.name == "jig-store-core").expect("core family");
    let current = skein_codec::schema::parse(family.current.as_bytes()).expect("current core text");
    let kept = skein_codec::schema::parse(kept_text(family).as_bytes()).expect("kept core schema");
    let plan = skein_codec::layout::Plan::derive(
        &current,
        current.root(b"Row", Some(b"deployment")).expect("current header root"),
        &[],
        &kept,
        kept.root(b"Row", Some(b"deployment")).expect("kept header root"),
        &[],
    )
    .expect("header at the source layout");
    let limits = jig_protocol_store::codec::limits(&root::CORE_LIMITS).expect("codec limits");
    let bytes = jig_protocol_store::codec::encode(header, &limits, &[]).expect("header encoded");
    let mut work = skein_codec::layout::Workspace::new();
    assert!(work.fit(&plan));
    let mut out = skein_lib::Buffer::with_capacity(u32::try_from(bytes.len()).expect("fixed header bytes"));
    plan.convert(&bytes, &mut out, &mut work).expect("header at kept layout");
    Box::from(out.as_slice())
}

fn seeded(sets: OwnerSets) -> Vec<Item> {
    let mut world = World::unopened(7);
    world.owner_sets(sets);
    assert_eq!(world.open(), Some(0), "{:?}", world.refusal());
    let header = root::header(1);
    // This value stays at the kept layout through the open, retaining a converting plan while serving.
    assert_eq!(
        world.commit(Box::new([
            Write::Save(root::header(1)),
            Write::Put { key: Box::from(b"a/0".as_slice()), value: Box::from([0, 2, 0, 42]) },
        ])),
        Some(1)
    );
    let key = jig_protocol_store::keys::encode(&header.key(), 64).expect("header key");
    let value = kept_header(&header);
    world
        .contents()
        .into_iter()
        .map(|(path, bytes)| match bytes {
            Some(bytes) if path.starts_with(b"generations/1/store/log-") => {
                let mut puts = commits::commit_keys(&bytes).expect("one source commit of puts");
                for (found, bytes) in &mut puts {
                    if *found == key {
                        bytes.clone_from(&value);
                    }
                }
                let frame = commits::source_commit(&puts, 1);
                Item::file(&path, &frame).mode(0o600)
            }
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

fn measure(items: &[Item], sets: OwnerSets, limits: &jig_protocol_store::Limits) -> Measured {
    let meter = Meter::new();
    meter.start();
    let mut world = World::unopened_observed(7, *limits, None);
    world.lay(items);
    world.owner_sets(sets);
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
    let current = jig_store_world::patches::current(SETS);
    let plain = sets(false, false);
    let kept = sets(true, true);
    let items = seeded(kept);
    // Identical families and serving machines without value types: subtract
    // their private heap to isolate the registry's retained share, not disk
    // copies of its durable rows, the simulator or the store's other tables.
    let limits = root::limits();
    let baseline = measure(&[], plain, &limits).private;
    let empty = measure(&[], current, &limits);
    let registry = empty.private - baseline;
    eprintln!("registry empty: peak={}, held={}, registry={registry}", empty.peak, empty.held);
    assert!(empty.peak <= EMPTY_PEAK, "empty: open peak {} > {EMPTY_PEAK}", empty.peak);
    assert!(registry <= EMPTY_REGISTRY, "empty: registry retained {registry} > {EMPTY_REGISTRY}");
    let seeded = measure(&items, current, &limits);
    let seeded_registry = seeded.private - baseline;
    eprintln!("registry seeded: peak={}, held={}, registry={seeded_registry}", seeded.peak, seeded.held);
    assert!(seeded.peak <= SEEDED_PEAK, "seeded: open peak {} > {SEEDED_PEAK}", seeded.peak);
    let mut larger = limits;
    larger.store.value = limits.store.value.checked_mul(4).expect("four fixture values fit u32");
    larger.store.chunk = skein_kv::chunk_bytes(larger.store.key, larger.store.value).expect("larger row chunk");
    let baseline = measure(&[], plain, &larger).private;
    let measured = measure(&items, current, &larger);
    let registry = measured.private - baseline;
    eprintln!(
        "registry seeded at 4x bound {} (fixture {}): peak={}, held={}, registry={registry}",
        larger.store.value, limits.store.value, measured.peak, measured.held
    );
    // The retained registry share may vary by at most 1% of the fixture-bound share.
    let margin = seeded_registry / 100;
    assert!(
        registry.abs_diff(seeded_registry) <= margin,
        "registry retained {registry} at 4x bound, versus {seeded_registry} at fixture bound (margin {margin})"
    );
    assert!(seeded_registry <= SEEDED_REGISTRY, "seeded registry retained {seeded_registry} > {SEEDED_REGISTRY}");
}
