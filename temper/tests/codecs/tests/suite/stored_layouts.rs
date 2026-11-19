//! Deployed layouts, not generated-code drift (temper/docs/design/protocol/floor.md, 6).
use jig_protocol_store::{self as store, JobLayout, KeptFamily, OwnerSets};
use skein_codec::{
    patch::{Capacity, Patcher},
    schema::{self, DeclIx, Schema},
};
use std::{collections::BTreeMap, fs, path::Path, process::Command};
use temper_engine_service_vocabulary::store::OWNER_SETS;

const BRIDGE: &str = "3d1c717d379e7987fb088a3756d937079bd8e074";
const RUNNING: &str = "13efa774615022299aea9ed57b2577df517494ad";

fn name(schema: &Schema) -> &str {
    str::from_utf8(schema.spelled(schema.family())).expect("ASCII family")
}
fn parsed(text: &str) -> Schema {
    schema::parse(text.as_bytes()).expect("schema parses")
}
fn git(args: &[&str]) -> String {
    let output = Command::new("git").current_dir(skein_scratch::paths::workspace()).args(args).output().expect("git");
    assert!(output.status.success(), "git {args:?}: {}", String::from_utf8_lossy(&output.stderr));
    String::from_utf8(output.stdout).expect("schema texts are UTF-8")
}

// Discover the build's families from its tree, not a second owner-set inventory.
// This also distinguishes a new family from an accidentally removed fixture.
fn sources(build: &str) -> BTreeMap<(String, u16), String> {
    let lines = git(&["grep", "^family ", build, "--", "*/crates/*/schema/*.schema", ":!*/schema/kept/*"]);
    let mut sources = BTreeMap::new();
    for line in lines.lines() {
        let mut parts = line.splitn(3, ':');
        assert_eq!(parts.next(), Some(build));
        let path = parts.next().expect("schema path");
        let mut family = parts.next().expect("family line").split_whitespace();
        assert_eq!(family.next(), Some("family"));
        let family_name = family.next().expect("family name");
        let version = family.next().expect("version").parse().expect("u16 version");
        sources.insert((family_name.to_owned(), version), path.to_owned());
    }
    sources
}

fn load(build: &str, directory: &str, sources: &BTreeMap<(String, u16), String>) -> BTreeMap<String, Schema> {
    let directory = skein_scratch::paths::package().join("deployed").join(directory);
    let mut texts = BTreeMap::new();
    for entry in fs::read_dir(directory).expect("deployed texts") {
        let path = entry.expect("directory entry").path();
        if path.extension().is_none_or(|ext| ext != "schema") {
            continue;
        }
        let text = fs::read_to_string(&path).expect("kept schema text");
        let schema = parsed(&text);
        let family = name(&schema).to_owned();
        assert_eq!(
            path.file_name().expect("filename").to_str(),
            Some(format!("{family}-v{}.schema", schema.version()).as_str())
        );
        let source = sources.get(&(family.clone(), schema.version())).expect("family at deployed build");
        assert_eq!(
            text,
            git(&["show", &format!("{build}:{source}")]),
            "{build} {family}: nothing rewrites a deployed text"
        );
        assert!(texts.insert(family, schema).is_none(), "one text per family at a build");
    }
    texts
}

fn inventory(
    build: &str,
    family: &str,
    texts: &BTreeMap<String, Schema>,
    sources: &BTreeMap<(String, u16), String>,
) -> Result<(), String> {
    if !texts.contains_key(family) && sources.keys().any(|(found, _)| found == family) {
        return Err(format!("{build} {family}: inventory: keep the family's deployed text in this build's directory"));
    }
    Ok(())
}

fn held<'a>(texts: &'a BTreeMap<String, Schema>, family: &str) -> Vec<&'a Schema> {
    texts.iter().filter(|(name, _)| *name != family).map(|(_, schema)| schema).collect()
}

fn registered(build: &str, sets: &OwnerSets, family: &KeptFamily, texts: &BTreeMap<String, Schema>) -> Vec<String> {
    let Some(kept) = texts.get(family.name) else { return vec![] };
    let held = held(texts, family.name);
    let mut failures = Vec::new();
    for ty in family.types {
        if kept.root(ty.root.as_bytes(), ty.variant.map(str::as_bytes)).is_none() {
            continue; // This value type was not written at this build.
        }
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

fn unchanged(
    build: &str,
    old: &Schema,
    old_held: &[&Schema],
    new: &Schema,
    new_held: &[&Schema],
    only: Option<&str>,
) -> Vec<String> {
    let mut failures = Vec::new();
    for index in 0..old.declarations() {
        let declaration = old.declaration(DeclIx::new(index));
        let name = old.spelled(declaration.name);
        if only.is_some_and(|root| root.as_bytes() != name) {
            continue;
        }
        let Some(new_root) = new.root(name, None) else { continue };
        let old_root = old.root(name, None).expect("declaration root");
        let before = old.layout(old_root, old_held);
        let after = new.layout(new_root, new_held);
        if before.is_err() || after.is_err() || before != after {
            failures.push(format!(
                "{build} {}: declaration {}: layout changed outside the registry at version {}; move the version and keep its text and patch",
                self::name(old), String::from_utf8_lossy(name), old.version(),
            ));
        }
    }
    failures
}

fn schema_files(directory: &Path, extension: &str) -> Vec<String> {
    if !directory.is_dir() {
        return vec![];
    }
    let mut paths: Vec<_> =
        fs::read_dir(directory).expect("schema directory").map(|entry| entry.expect("entry").path()).collect();
    paths.sort();
    paths
        .iter()
        .filter(|path| path.extension().is_some_and(|ext| ext == extension))
        .map(|path| fs::read_to_string(path).expect("schema or patch text"))
        .collect()
}

// Non-frame families keep their histories beside the crate's current schema.
fn history(family: &str) -> (Vec<String>, Vec<String>) {
    let root = skein_scratch::paths::workspace();
    for tree in ["skein", "smith", "jig", "temper"] {
        for entry in fs::read_dir(root.join(tree).join("crates")).expect("crates") {
            let owner = entry.expect("crate").path();
            let directory = owner.join("schema");
            for path in fs::read_dir(&directory).into_iter().flatten() {
                let path = path.expect("schema entry").path();
                if path.extension().is_some_and(|ext| ext == "schema") {
                    let text = fs::read_to_string(path).expect("schema text");
                    if name(&parsed(&text)) == family {
                        return (
                            schema_files(&directory.join("kept"), "schema"),
                            schema_files(&owner.join("patches"), "patch"),
                        );
                    }
                }
            }
        }
    }
    (vec![], vec![])
}

fn patched(old: &Schema, new: &Schema, texts: &[String], patches: &[String]) -> bool {
    // Runtime patch planning validates the history, not merely a file name.
    let count = u32::try_from(texts.len()).expect("history length").checked_add(1).expect("current text");
    let mut planner = Patcher::new(Capacity {
        versions: count,
        patches: u32::try_from(patches.len()).expect("patch count"),
        text: schema::MAX_TEXT,
        bytes: 0,
        nodes: 0,
    });
    let mut kept = false;
    for text in texts {
        let schema = parsed(text);
        if name(&schema) != name(old) {
            continue;
        }
        if schema.version() == old.version() {
            kept = schema.text() == old.text();
        }
        if schema.version() != new.version() && planner.keep(text.as_bytes()).is_err() {
            return false;
        }
    }
    if !kept || planner.keep(new.text()).is_err() {
        return false;
    }
    for patch in patches {
        if planner.plan(patch.as_bytes()).is_err() {
            return false;
        }
    }
    let mut version = old.version();
    for _ in 0..patches.len() {
        let Some(plan) = planner.plan_from(version) else { return false };
        version = planner.versions(plan).to;
        if version == new.version() {
            return true;
        }
    }
    false
}

fn outside(
    build: &str,
    family: &KeptFamily,
    old: &Schema,
    old_held: &[&Schema],
    new: &Schema,
    new_held: &[&Schema],
    only: Option<&str>,
) -> Vec<String> {
    if old.version() == new.version() {
        return unchanged(build, old, old_held, new, new_held, only);
    }
    let (texts, patches) = match family.frames {
        Some(frames) => (
            frames.texts.iter().map(|text| (*text).to_owned()).collect(),
            frames.patches.iter().map(|text| (*text).to_owned()).collect(),
        ),
        None => history(family.name),
    };
    if patched(old, new, &texts, &patches) {
        vec![]
    } else {
        vec![format!(
            "{build} {}: declaration {}: version {} -> {} needs the kept version's text and patch",
            family.name,
            only.unwrap_or("every"),
            old.version(),
            new.version()
        )]
    }
}

fn guard(
    build: &str,
    sets: &OwnerSets,
    texts: &BTreeMap<String, Schema>,
    sources: &BTreeMap<(String, u16), String>,
) -> Vec<String> {
    let mut current = BTreeMap::new();
    let client = KeptFamily {
        name: "jig-client-view-records",
        current: jig_client_view_codec::SCHEMA,
        map: store::KeptMap::Store,
        keys: &[],
        frames: None,
        types: &[],
        retired: &[],
    };
    let families: Vec<_> = sets.sets.iter().flat_map(|set| set.families).chain(std::iter::once(&client)).collect();
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
        if family.types.is_empty() {
            failures.extend(outside(
                build,
                family,
                old,
                &held(texts, family.name),
                new,
                &held(&current, family.name),
                None,
            ));
        } else {
            failures.extend(registered(build, sets, family, texts));
            if let Some(frames) = family.frames {
                failures.extend(outside(
                    build,
                    family,
                    old,
                    &held(texts, family.name),
                    new,
                    &held(&current, family.name),
                    Some(frames.root),
                ));
            }
        }
    }
    failures
}

fn deployed(build: &str, directory: &str) {
    let sources = sources(build);
    let texts = load(build, directory, &sources);
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
    let failures = unchanged("mutation-build", &old, &[], &changed, &[], None);
    assert_eq!(failures.len(), 1);
    assert!(failures.first().expect("failure").contains("declaration Value"));
    assert!(failures.first().expect("failure").contains("move the version"));
    let renamed = parsed(&KEPT.replace("count", "renamed"));
    assert_eq!(unchanged("mutation-build", &old, &[], &renamed, &[], None), Vec::<String>::new());
    let nested = parsed("family fixture-map 1\nrecord Piece { count: u32 }\nversioned record Value { piece: Piece }");
    let renamed =
        parsed("family fixture-map 1\nrecord Renamed { count: u32 }\nversioned record Value { piece: Renamed }");
    assert_eq!(unchanged("mutation-build", &nested, &[], &renamed, &[], None), Vec::<String>::new());
    let gone = parsed("family fixture-map 1\nrecord Other { other: bool }");
    assert_eq!(unchanged("mutation-build", &old, &[], &gone, &[], None), Vec::<String>::new());
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
fn moved_versions_need_the_kept_text_and_patch() {
    let old = parsed(KEPT);
    let new = parsed(&ADD.replace("fixture-map 1", "fixture-map 2"));
    let texts = [KEPT.to_owned()];
    let patches = ["patch 1\nowner fixture 1\nfamily fixture-map 1 to 2\nadd Value.extra u32 0\n".to_owned()];
    let unrelated = [patches.first().expect("patch").replace("fixture-map", "another-family")];
    assert!(!patched(&old, &new, &texts, &unrelated));
    assert!(!patched(&old, &new, &texts, &[]));
    assert!(!patched(&old, &new, &[], &patches));
    assert!(patched(&old, &new, &texts, &patches));
    let edited = [KEPT.replace("count", "renamed")];
    assert!(!patched(&old, &new, &edited, &patches));
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
