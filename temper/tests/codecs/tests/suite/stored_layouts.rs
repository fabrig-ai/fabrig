//! Deployed layouts, not generated-code drift (temper/docs/design/protocol/floor.md, 6).
use jig_protocol_store::{self as store, JobLayout, KeptFamily, OwnerSets, ValueType};
use sha2::{Digest, Sha256};
use skein_codec::{
    layout::{Mode, Plan},
    schema::{self, DeclIx, FieldIx, Held, Root, Schema, Shape, Type, TypeIx},
};
use std::collections::BTreeMap;
use temper_engine_service_vocabulary::store::OWNER_SETS;

const BRIDGE: &str = "3d1c717d379e7987fb088a3756d937079bd8e074";
const RUNNING: &str = "13efa774615022299aea9ed57b2577df517494ad";

type Sources = BTreeMap<(String, u16), String>;

struct Deployed {
    directory: &'static str,
    inventory: &'static str,
    texts: &'static [(&'static str, &'static str)],
}

include!(concat!(env!("OUT_DIR"), "/stored_layout_fixtures.rs"));

fn name(schema: &Schema) -> &str {
    str::from_utf8(schema.spelled(schema.family())).expect("ASCII family")
}
fn parsed(text: &str) -> Schema {
    schema::parse(text.as_bytes()).expect("schema parses")
}
fn sources(build: &str, inventory: &'static str) -> Sources {
    let mut lines = inventory.lines();
    assert_eq!(lines.next(), Some(build));
    let mut sources = BTreeMap::new();
    for line in lines {
        let mut fields = line.split_whitespace();
        let family = fields.next().expect("family name");
        let version = fields.next().expect("family version").parse().expect("u16 version");
        let _path = fields.next().expect("historical schema path");
        let digest = fields.next().expect("historical schema digest");
        assert!(fields.next().is_none(), "one source per line");
        assert!(
            sources.insert((family.to_owned(), version), digest.to_owned()).is_none(),
            "one source per family and version"
        );
    }
    sources
}

fn load(build: &str, deployed: &Deployed, sources: &Sources) -> BTreeMap<String, Schema> {
    let mut texts = BTreeMap::new();
    for &(filename, text) in deployed.texts {
        let schema = parsed(text);
        let family = name(&schema).to_owned();
        assert_eq!(filename, format!("{family}-v{}.schema", schema.version()));
        let digest = sources.get(&(family.clone(), schema.version())).expect("family at deployed build");
        assert_eq!(
            format!("{:x}", Sha256::digest(text)),
            *digest,
            "{build} {family}: nothing rewrites a deployed text"
        );
        assert!(texts.insert(family, schema).is_none(), "one text per family at a build");
    }
    texts
}

fn inventory(build: &str, family: &str, texts: &BTreeMap<String, Schema>, sources: &Sources) -> Result<(), String> {
    if !texts.contains_key(family) && sources.keys().any(|(found, _)| found == family) {
        return Err(format!("{build} {family}: inventory: keep the family's deployed text in this build's directory"));
    }
    Ok(())
}

fn held<'a>(texts: &'a BTreeMap<String, Schema>, family: &str) -> Vec<&'a Schema> {
    texts.iter().filter(|(name, _)| *name != family).map(|(_, schema)| schema).collect()
}

fn roots_at<'a>(
    kept: &'a Schema,
    types: &'static [ValueType],
) -> impl Iterator<Item = (&'static ValueType, Root)> + 'a {
    types.iter().filter_map(move |ty| Some((ty, kept.root(ty.root.as_bytes(), ty.variant.map(str::as_bytes))?)))
}

fn registered(build: &str, sets: &OwnerSets, family: &KeptFamily, texts: &BTreeMap<String, Schema>) -> Vec<String> {
    let Some(kept) = texts.get(family.name) else { return vec![] };
    let held = held(texts, family.name);
    let mut failures = Vec::new();
    for (ty, _) in roots_at(kept, family.types) {
        let layout = JobLayout {
            root: ty.root.into(),
            variant: ty.variant.map(Into::into),
            text: kept.text().into(),
            held: held.iter().map(|schema| schema.text().into()).collect(),
        };
        if let Err(why) = store::judge_held_layout(sets, family.name, ty.number, &layout) {
            failures.push(format!(
                "{build} {}: value type {} root {} variant {:?}: {why:?}; add a default (= value), a former name (was), or a written job for a map-held layout; log-held layouts must derive",
                family.name, ty.number, ty.root, ty.variant,
            ));
        }
    }
    failures
}

fn outside(build: &str, old: &Schema, old_held: &[&Schema], new: &Schema, new_held: &[&Schema]) -> Vec<String> {
    if old.version() != new.version() {
        // temper/docs/design/protocol/floor.md, 6: version 1 openings are read by head
        // alone and cleared by the next start, without translating their bodies.
        if (name(old), name(new), old.version(), new.version()) == ("jig-store-openings", "jig-store-openings", 1, 2) {
            return vec![];
        }
        return vec![format!(
            "{build} {}: version changed outside the registry from {} to {}; provide an upgrade path",
            name(old),
            old.version(),
            new.version()
        )];
    }
    let mut failures = Vec::new();
    for index in 0..old.declarations() {
        let declaration = old.declaration(DeclIx::new(index));
        let name = old.spelled(declaration.name);
        let Some(new_root) = new.root(name, None) else { continue };
        let old_root = old.root(name, None).expect("declaration root");
        let before = old.layout(old_root, old_held);
        let after = new.layout(new_root, new_held);
        if before.is_err() || after.is_err() || before != after {
            failures.push(format!(
                "{build} {}: declaration {}: layout changed outside the registry at version {}; provide an upgrade path",
                self::name(old), String::from_utf8_lossy(name), old.version(),
            ));
        }
    }
    failures
}

fn frame_layout(build: &str, sets: &OwnerSets, family: &KeptFamily, old: &Schema, new: &Schema) -> Vec<String> {
    let frames = family.frames.expect("frame value type");
    let root = old.find(frames.root.as_bytes()).or_else(|| {
        let current = new.find(frames.root.as_bytes())?;
        new.was(skein_codec::schema::Named::Declaration(current))
            .iter()
            .find_map(|former| old.find(new.spelled(*former)))
    });
    let root = match root {
        Some(root) => std::str::from_utf8(old.spelled(old.declaration(root).name)).expect("schema name"),
        None => frames.root,
    };
    let layout = JobLayout { root: root.into(), variant: None, text: old.text().into(), held: Box::new([]) };
    match store::judge_held_layout(sets, family.name, frames.number, &layout) {
        Ok(store::HeldLayout::Planned) => vec![],
        result => vec![format!(
            "{build} {}: frame value type {} root {root}: {result:?}; frames must derive",
            family.name, frames.number
        )],
    }
}

fn guard(build: &str, sets: &OwnerSets, texts: &BTreeMap<String, Schema>, sources: &Sources) -> Vec<String> {
    let mut current = BTreeMap::new();
    let families: Vec<_> = sets.sets.iter().flat_map(|set| set.families).collect();
    for family in &families {
        current.insert(family.name.to_owned(), parsed(family.current));
    }
    let mut failures = Vec::new();
    for family in families {
        if let Err(why) = inventory(build, family.name, texts, sources) {
            failures.push(why);
        }
        let Some(old) = texts.get(family.name) else { continue };
        let new = current.get(family.name).expect("binary family");
        if family.frames.is_some() {
            failures.extend(frame_layout(build, sets, family, old, new));
        }
        if family.types.is_empty() && family.frames.is_none() {
            failures.extend(outside(build, old, &held(texts, family.name), new, &held(&current, family.name)));
        } else {
            failures.extend(registered(build, sets, family, texts));
        }
    }
    failures
}

fn deployed_texts(build: &str, directory: &str) -> (Sources, BTreeMap<String, Schema>) {
    let deployed = DEPLOYED.iter().find(|fixture| fixture.directory == directory).expect("deployed build");
    let sources = sources(build, deployed.inventory);
    let texts = load(build, deployed, &sources);
    (sources, texts)
}

fn deployed(build: &str, directory: &str) {
    let (sources, texts) = deployed_texts(build, directory);
    let failures = guard(build, &OWNER_SETS, &texts, &sources);
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn floor_bridge_layouts_open() {
    deployed(BRIDGE, "3d1c717d379e");
}
#[test]
fn running_build_layouts_open() {
    deployed(RUNNING, "13efa77461");
}

#[test]
fn deployed_layouts_allow_only_the_named_openings_version_move() {
    let current = parsed(jig_protocol_store::STORE_FAMILIES[1].current);
    deployed(BRIDGE, "3d1c717d379e");
    let (sources, deployed) = deployed_texts(RUNNING, "13efa77461");
    assert_eq!(guard(RUNNING, &OWNER_SETS, &deployed, &sources), Vec::<String>::new());
    let old = &deployed["jig-store-openings"];
    assert_eq!((old.version(), current.version()), (1, 2));
    assert_eq!(outside("mutation", old, &[], &current, &[]), Vec::<String>::new());
    let unrelated = parsed("family fixture-map 1\nversioned record Value { count: u32 }");
    let moved = parsed("family fixture-map 2\nversioned record Value { count: u32 }");
    assert_eq!(outside("mutation", &unrelated, &[], &moved, &[]).len(), 1);
    let too_far =
        parsed(&jig_protocol_store::STORE_FAMILIES[1].current.replace("jig-store-openings 2", "jig-store-openings 3"));
    assert_eq!(outside("mutation", old, &[], &too_far, &[]).len(), 1);
}

fn held_family(schema: &Schema, mut ty: TypeIx) -> Option<&str> {
    loop {
        match schema.ty(ty) {
            Type::Option(item) | Type::List { item, .. } => ty = item,
            Type::Bytes { held: Held::Record { family, .. }, .. }
            | Type::Secret { held: Held::Record { family, .. }, .. } => {
                return Some(str::from_utf8(schema.spelled(family)).expect("ASCII family"));
            }
            _ => return None,
        }
    }
}

// The judge resolves the binary's held texts over the transitive reach of
// each whole owner text, in discovery order (registry/texts.rs, Binary::held).
fn binary_held<'a>(schema: &'a Schema, current: &'a BTreeMap<&str, Schema>) -> Vec<&'a Schema> {
    let mut pending = vec![schema];
    for index in 0..=current.len() {
        let Some(&schema) = pending.get(index) else { break };
        for decl in 0..schema.declarations() {
            let Shape::Record { first, count, .. } = schema.declaration(DeclIx::new(decl)).shape else { continue };
            for offset in 0..count {
                let field = schema.field(FieldIx::new(first.index() + offset));
                if let Some(held) = held_family(schema, field.ty).and_then(|name| current.get(name))
                    && !pending.iter().any(|seen| std::ptr::eq(*seen, held))
                {
                    pending.push(held);
                }
            }
        }
    }
    pending.remove(0);
    pending
}

#[test]
fn registered_deployed_plans_equal_non_refusing_read_plans() {
    let families = OWNER_SETS.sets.iter().flat_map(|set| set.families);
    let current: BTreeMap<_, _> = families.clone().map(|family| (family.name, parsed(family.current))).collect();
    for (build, directory) in [(BRIDGE, "3d1c717d379e"), (RUNNING, "13efa77461")] {
        let (_, texts) = deployed_texts(build, directory);
        let mut compared = 0;
        for family in families.clone().filter(|family| !family.types.is_empty()) {
            let Some(old) = texts.get(family.name) else { continue };
            let old_held = held(&texts, family.name);
            let new = current.get(family.name).expect("binary family");
            let new_held = binary_held(new, &current);
            for (ty, old_root) in roots_at(old, family.types) {
                let context = format!(
                    "{build} {} value type {} root {} variant {:?}",
                    family.name, ty.number, ty.root, ty.variant
                );
                let new_root = new
                    .root(ty.root.as_bytes(), ty.variant.map(str::as_bytes))
                    .unwrap_or_else(|| panic!("{context}: missing binary root"));
                let Ok(strict) = Plan::derive(old, old_root, &old_held, new, new_root, &new_held) else { continue };
                let read = Plan::between(Mode::Read, old, old_root, &old_held, new, new_root, &new_held)
                    .unwrap_or_else(|why| panic!("{context}: Read underived where the store planned: {why:?}"));
                assert!(!read.refuses(), "{context}: Read adds a refusal: {:?}", read.refusal(0));
                assert_eq!(read, strict, "{context}: Read and store plans differ");
                compared += 1;
            }
        }
        assert!(compared > 0, "{build}: checked at least one derived plan");
    }
}

// Test the defaults themselves, without pending conversion jobs masking a missing
// default on tier. The written-job escape hatch is pinned separately below.
fn dev_mutation(current: &str) -> Vec<String> {
    let family = KeptFamily {
        current: Box::leak(current.to_owned().into_boxed_str()),
        ..temper_protocol_store::OWNER_SET.families[0]
    };
    let sets = OwnerSets {
        sets: Box::leak(Box::new([store::OwnerSet {
            families: Box::leak(Box::new([family])),
            jobs: &[],
            ..temper_protocol_store::OWNER_SET
        }])),
    };
    let texts = BTreeMap::from([(
        family.name.to_owned(),
        parsed(include_str!("../../deployed/13efa77461/temper-store-v25.schema")),
    )]);
    guard(RUNNING, &sets, &texts, &BTreeMap::new())
}
fn dev_default_refusal(field: &str, without: &str, number: u16, variant: &str) {
    let current = temper_protocol_store::OWNER_SET.families[0].current;
    assert_eq!(current.matches(field).count(), 1, "mutation selects one field");
    let failures = dev_mutation(&current.replace(field, without));
    assert_eq!(failures.len(), 1, "{failures:?}");
    let failure = failures.first().expect("missing default refusal");
    for required in [
        RUNNING,
        "temper-store",
        &format!("value type {number}"),
        &format!("variant Some({variant:?})"),
        "NoDefault",
        "= value",
    ] {
        assert!(failure.contains(required), "missing {required}: {failure}");
    }
    assert_eq!(dev_mutation(current), Vec::<String>::new());
}
#[test]
fn dev_tier_waiting_needs_its_default() {
    dev_default_refusal("waiting: list 8 VerdictFiling = []", "waiting: list 8 VerdictFiling", 25, "tier");
}
#[test]
fn dev_finding_signature_many_needs_its_default() {
    dev_default_refusal("many: bool = false", "many: bool", 28, "finding");
}
#[test]
fn dev_finding_blame_keeps_implicit_none() {
    let current = temper_protocol_store::OWNER_SET.families[0].current;
    let field = "blame: option FindingBlame = none";
    assert_eq!(current.matches(field).count(), 1);
    assert_eq!(dev_mutation(&current.replace(field, "blame: option FindingBlame")), Vec::<String>::new());
}

// In-memory mutations of a kept layout; no kept file is ever rewritten.
const KEPT: &str = "family fixture-map 1\nversioned record Value { count: u32 }";
const FAMILY: KeptFamily = KeptFamily {
    name: "fixture-map",
    current: KEPT,
    map: store::KeptMap::Store,
    keys: &[],
    frames: None,
    types: &[store::ValueType { number: 7, root: "Value", variant: None, keys: &[] }],
    retired: &[],
};

fn mutation(sets: &OwnerSets) -> Vec<String> {
    let texts = BTreeMap::from([(FAMILY.name.to_owned(), parsed(KEPT))]);
    guard("mutation-build", sets, &texts, &BTreeMap::new())
}
const SET: store::OwnerSet =
    store::OwnerSet { owner: store::Owner::Application, name: "fixture", families: &[], level: None, jobs: &[] };
const ADD: &str = "family fixture-map 1\nversioned record Value { count: u32 extra: u32 }";
const DEFAULT: &str = "family fixture-map 1\nversioned record Value { count: u32 extra: u32 = 0 }";
const OPTION: &str = "family fixture-map 1\nversioned record Value { count: u32 extra: option u32 }";
const RENAME: &str = "family fixture-map 1\nversioned record Value { renamed: u32 }";
const WAS: &str = "family fixture-map 1\nversioned record Value { renamed was count: u32 }";
const ADD_SETS: OwnerSets =
    OwnerSets { sets: &[store::OwnerSet { families: &[KeptFamily { current: ADD, ..FAMILY }], ..SET }] };
const DEFAULT_SETS: OwnerSets =
    OwnerSets { sets: &[store::OwnerSet { families: &[KeptFamily { current: DEFAULT, ..FAMILY }], ..SET }] };
const OPTION_SETS: OwnerSets =
    OwnerSets { sets: &[store::OwnerSet { families: &[KeptFamily { current: OPTION, ..FAMILY }], ..SET }] };
const RENAME_SETS: OwnerSets =
    OwnerSets { sets: &[store::OwnerSet { families: &[KeptFamily { current: RENAME, ..FAMILY }], ..SET }] };
const WAS_SETS: OwnerSets =
    OwnerSets { sets: &[store::OwnerSet { families: &[KeptFamily { current: WAS, ..FAMILY }], ..SET }] };

fn refuses(failures: &[String]) {
    assert_eq!(failures.len(), 1, "{failures:?}");
    let failure = failures.first().expect("failure");
    for required in [
        "mutation-build",
        "fixture-map",
        "value type 7",
        "root Value",
        "variant None",
        "= value",
        "was",
        "written job",
        "NoDefault",
    ] {
        assert!(failure.contains(required), "missing {required}: {failure}");
    }
}
#[test]
fn registered_additions_need_defaults() {
    refuses(&mutation(&ADD_SETS));
    assert_eq!(mutation(&DEFAULT_SETS), Vec::<String>::new());
    assert_eq!(mutation(&OPTION_SETS), Vec::<String>::new());
}
#[test]
fn registered_renames_need_was() {
    refuses(&mutation(&RENAME_SETS));
    assert_eq!(mutation(&WAS_SETS), Vec::<String>::new());
}
#[test]
fn outside_layouts_are_fixed_but_names_are_not() {
    let old = parsed(KEPT);
    let changed = parsed(&KEPT.replace("u32", "u64"));
    let failures = outside("mutation-build", &old, &[], &changed, &[]);
    assert_eq!(failures.len(), 1);
    assert!(failures.first().expect("failure").contains("declaration Value"));
    assert!(failures.first().expect("failure").contains("provide an upgrade path"));
    let renamed = parsed(&KEPT.replace("count", "renamed"));
    assert_eq!(outside("mutation-build", &old, &[], &renamed, &[]), Vec::<String>::new());
    let nested = parsed("family fixture-map 1\nrecord Piece { count: u32 }\nversioned record Value { piece: Piece }");
    let renamed =
        parsed("family fixture-map 1\nrecord Renamed { count: u32 }\nversioned record Value { piece: Renamed }");
    assert_eq!(outside("mutation-build", &nested, &[], &renamed, &[]), Vec::<String>::new());
    let gone = parsed("family fixture-map 1\nrecord Other { other: bool }");
    assert_eq!(outside("mutation-build", &old, &[], &gone, &[]), Vec::<String>::new());
}
// OwnerSets is a static shell contract. These tiny, test-owned sets live for
// the test process, like the upgrade fixtures in one-process/src/upgrade.rs.
fn job_sets(family: KeptFamily, from: store::Layouts) -> OwnerSets {
    let job = store::Job {
        name: "convert-layout",
        keys: &[],
        counts: &[],
        converts: Some(store::Converts {
            family: family.name,
            value_type: family.types.first().expect("type").number,
            from,
        }),
    };
    OwnerSets {
        sets: Box::leak(Box::new([store::OwnerSet {
            families: Box::leak(Box::new([family])),
            jobs: Box::leak(Box::new([job])),
            ..SET
        }])),
    }
}

#[test]
fn written_jobs_cover_map_layouts_but_not_logs() {
    const OLD: &str = "family jig-store-core 6\nrecord Payload { count: u32 }\nenum Entry { deployment: Payload, terminal: Payload }\nversioned record Row { value: Entry }";
    const NEW: &str = "family jig-store-core 6\nrecord Payload { count: u32 extra: u32 }\nenum Entry { deployment: Payload, terminal: Payload }\nversioned record Row { value: Entry }";
    const MAP: KeptFamily = KeptFamily {
        name: "jig-store-core",
        current: NEW,
        types: &[store::ValueType { number: 0, root: "Row", variant: Some("deployment"), keys: &[] }],
        ..FAMILY
    };
    const LOG: KeptFamily = KeptFamily {
        types: &[store::ValueType { number: 5, root: "Row", variant: Some("terminal"), keys: &[] }],
        ..MAP
    };
    let old = parsed(OLD);
    let fingerprint =
        |variant| old.layout(old.root(b"Row", Some(variant)).expect("root"), &[]).expect("layout").digest();
    let layout = |variant: &str| JobLayout {
        root: "Row".into(),
        variant: Some(variant.into()),
        text: OLD.as_bytes().into(),
        held: Box::new([]),
    };
    let sets = job_sets(MAP, store::Layouts::One(fingerprint(b"deployment")));
    assert_eq!(
        store::judge_held_layout(&sets, MAP.name, 0, &layout("deployment")),
        Ok(store::HeldLayout::Converted { job: "convert-layout" })
    );
    let wrong = job_sets(MAP, store::Layouts::One([0; 32]));
    assert!(store::judge_held_layout(&wrong, MAP.name, 0, &layout("deployment")).is_err());
    let unplanned = job_sets(MAP, store::Layouts::Unplanned);
    assert_eq!(
        store::judge_held_layout(&unplanned, MAP.name, 0, &layout("deployment")),
        Ok(store::HeldLayout::Converted { job: "convert-layout" })
    );
    let logs = job_sets(LOG, store::Layouts::One(fingerprint(b"terminal")));
    assert!(store::judge_held_layout(&logs, LOG.name, 5, &layout("terminal")).is_err());
    let logs = job_sets(LOG, store::Layouts::Unplanned);
    assert!(store::judge_held_layout(&logs, LOG.name, 5, &layout("terminal")).is_err());
    let malformed = JobLayout { text: b"not a schema".as_slice().into(), ..layout("deployment") };
    assert_eq!(store::judge_held_layout(&unplanned, MAP.name, 0, &malformed), Err(store::Unplanned::Text));
}

#[test]
fn held_layouts_use_the_builds_texts() {
    const HOLDER: &str = "family fixture-map 1\nversioned record Value { held: bytes 64 holds fixture-held Content }";
    const HELD: &str = "family fixture-held 1\nversioned record Content { count: u32 }";
    const CHANGED: &str = "family fixture-held 1\nversioned record Content { count: u32 extra: u32 }";
    const DEFAULTED: &str = "family fixture-held 1\nversioned record Content { count: u32 extra: u32 = 0 }";
    const HOLDER_FAMILY: KeptFamily = KeptFamily { current: HOLDER, ..FAMILY };
    const HELD_FAMILY: KeptFamily = KeptFamily { name: "fixture-held", current: CHANGED, types: &[], ..FAMILY };
    const BAD: OwnerSets = OwnerSets { sets: &[store::OwnerSet { families: &[HOLDER_FAMILY, HELD_FAMILY], ..SET }] };
    const GOOD: OwnerSets = OwnerSets {
        sets: &[store::OwnerSet {
            families: &[HOLDER_FAMILY, KeptFamily { current: DEFAULTED, ..HELD_FAMILY }],
            ..SET
        }],
    };
    let texts = BTreeMap::from([(FAMILY.name.to_owned(), parsed(HOLDER)), ("fixture-held".to_owned(), parsed(HELD))]);
    refuses(&registered("mutation-build", &BAD, &HOLDER_FAMILY, &texts));
    assert_eq!(registered("mutation-build", &GOOD, &HOLDER_FAMILY, &texts), Vec::<String>::new());
}

#[test]
fn frame_body_and_turn_must_each_derive_independently() {
    const BODY: &str = "family fixture-body 1\nversioned record Frame { turn: bytes 64 holds fixture-turn Turn }";
    const CHANGED_BODY: &str =
        "family fixture-body 1\nversioned record Frame { turn: bytes 64 holds fixture-turn Turn added: u64 }";
    const TURN: &str = "family fixture-turn 1\nversioned record Turn { count: u32 }";
    const NEXT: &str = "family fixture-turn 1\nversioned record Turn { count: u32 extra: u32 = 0 }";
    const UNPLANNED: &str = "family fixture-turn 1\nversioned record Turn { count: u32 extra: u32 }";
    const BODY_FAMILY: KeptFamily = KeptFamily {
        name: "fixture-body",
        current: BODY,
        types: &[],
        frames: Some(store::Frames { root: "Frame", number: 0, texts: &[BODY] }),
        ..FAMILY
    };
    const TURN_FAMILY: KeptFamily = KeptFamily {
        name: "fixture-turn",
        current: NEXT,
        types: &[],
        frames: Some(store::Frames { root: "Turn", number: 0, texts: &[TURN] }),
        ..FAMILY
    };
    let old = BTreeMap::from([("fixture-body".to_owned(), parsed(BODY)), ("fixture-turn".to_owned(), parsed(TURN))]);
    for (body, turn, refused) in
        [(BODY, NEXT, None), (BODY, UNPLANNED, Some("fixture-turn")), (CHANGED_BODY, NEXT, Some("fixture-body"))]
    {
        let sets = OwnerSets {
            sets: Box::leak(Box::new([
                store::OwnerSet {
                    owner: store::Owner::Jig,
                    families: Box::leak(Box::new([KeptFamily { current: body, ..BODY_FAMILY }])),
                    ..SET
                },
                store::OwnerSet {
                    owner: store::Owner::Smith,
                    families: Box::leak(Box::new([KeptFamily { current: turn, ..TURN_FAMILY }])),
                    ..SET
                },
            ])),
        };
        let failures = guard("mutation-build", &sets, &old, &BTreeMap::new());
        if let Some(family) = refused {
            assert_eq!(failures.len(), 1, "{failures:?}");
            assert!(failures[0].contains(family), "{failures:?}");
        } else {
            assert!(failures.is_empty(), "{failures:?}");
        }
    }
}

#[test]
fn moving_the_version_does_not_bypass_the_floor_guard() {
    let old = parsed(KEPT);
    let new = parsed(&KEPT.replace("fixture-map 1", "fixture-map 2"));
    let failures = outside("mutation-build", &old, &[], &new, &[]);
    assert_eq!(failures.len(), 1);
    assert!(failures[0].contains("fixture-map: version changed outside the registry from 1 to 2"));
}

#[test]
fn inventory_cannot_lose_a_deployed_family() {
    let sources = BTreeMap::from([((FAMILY.name.to_owned(), 1), "crate/schema/value-v1.schema".to_owned())]);
    let texts = BTreeMap::from([(FAMILY.name.to_owned(), parsed(KEPT))]);
    assert!(inventory("mutation-build", FAMILY.name, &texts, &sources).is_ok());
    let failure = inventory("mutation-build", FAMILY.name, &BTreeMap::new(), &sources).expect_err("missing fixture");
    assert!(failure.contains("mutation-build fixture-map: inventory"));
    assert!(inventory("mutation-build", "new-family", &texts, &sources).is_ok());
}
