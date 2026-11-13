//! Each generated jig codec's schema, code and golden inventory (protocol/README.md, 7).
use skein_codegen::{generate, parse};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
};

pub struct Family {
    pub name: &'static str,
    pub root: &'static str,
    pub codec_root: &'static str,
    pub schemas: &'static [&'static str],
    /// Record code under `codec_root`, then glue under `root`.
    pub generated: &'static [&'static str],
    pub golden: &'static str,
    pub fixtures: &'static [&'static str],
}

pub const FAMILIES: &[Family] = &[
    Family {
        name: "engine cursor",
        root: "crates/jig-protocol-engine-clients",
        codec_root: "crates/jig-protocol-engine-clients-codec",
        schemas: &["schema/cursor-v1.schema"],
        generated: &["src/generated/cursor_v1.rs"],
        golden: "golden",
        fixtures: &[],
    },
    Family {
        name: "client",
        root: "crates/jig-client-protocol-wire",
        codec_root: "crates/jig-client-protocol-wire",
        schemas: &["schema/client-v21.schema", "schema/client-v21-part1.schema"],
        generated: &["src/generated/v21.rs"],
        golden: "golden",
        fixtures: &["task_hold_placed_full.bin", "task_hold_placed_smallest.bin", "public"],
    },
    Family {
        name: "view records",
        root: "crates/jig-client-view",
        codec_root: "crates/jig-client-view-codec",
        schemas: &["schema/records-v1.schema"],
        generated: &["src/generated/records_v1.rs"],
        golden: "golden",
        fixtures: &["rendered"],
    },
    Family {
        name: "ops client",
        root: "examples/ops/client-protocol-wire",
        codec_root: "examples/ops/client-protocol-wire",
        schemas: &["schema/ops-client-v1.schema"],
        generated: &["src/generated/v1.rs"],
        golden: "golden",
        fixtures: &[],
    },
    Family {
        name: "test client",
        root: "testing/jig-client-test-protocol-wire",
        codec_root: "testing/jig-client-test-protocol-wire-codec",
        schemas: &["schema/test-client-v2.schema"],
        generated: &["src/generated/v2.rs"],
        golden: "golden",
        fixtures: &[],
    },
    Family {
        name: "component",
        root: "crates/jig-protocol-store",
        codec_root: "crates/jig-protocol-store-families",
        schemas: &["schema/component-v3.schema"],
        generated: &["src/generated/component_v3.rs", "src/generated/layouts.rs"],
        golden: "golden/component",
        fixtures: &[],
    },
    Family {
        name: "core",
        root: "crates/jig-protocol-store",
        codec_root: "crates/jig-protocol-store-core",
        schemas: &["schema/core-v6.schema"],
        generated: &["src/generated/core_v6.rs"],
        golden: "golden/core",
        fixtures: &[],
    },
    Family {
        name: "tasks",
        root: "crates/jig-protocol-store",
        codec_root: "crates/jig-protocol-store-tasks",
        schemas: &["schema/tasks-v12.schema"],
        generated: &["src/generated/tasks_v12.rs"],
        golden: "golden/tasks",
        fixtures: &[],
    },
    Family {
        name: "people",
        root: "crates/jig-protocol-store",
        codec_root: "crates/jig-protocol-store-people",
        schemas: &["schema/people-v17.schema"],
        generated: &["src/generated/people_v17.rs"],
        golden: "golden/people",
        fixtures: &[],
    },
    Family {
        name: "openings",
        root: "crates/jig-protocol-store",
        codec_root: "crates/jig-protocol-store-families",
        schemas: &["schema/openings-v1.schema"],
        generated: &["src/generated/openings_v1.rs"],
        golden: "golden/openings",
        fixtures: &[],
    },
    Family {
        name: "notes",
        root: "crates/jig-protocol-store",
        codec_root: "crates/jig-protocol-store-families",
        schemas: &["schema/notes-v1.schema"],
        generated: &["src/generated/notes_v1.rs"],
        golden: "golden/notes",
        fixtures: &[],
    },
    Family {
        name: "accounts",
        root: "crates/jig-protocol-store",
        codec_root: "crates/jig-protocol-store-families",
        schemas: &["schema/accounts-v2.schema"],
        generated: &["src/generated/accounts_v2.rs"],
        golden: "golden/accounts",
        fixtures: &[],
    },
    Family {
        name: "settings",
        root: "crates/jig-protocol-store",
        codec_root: "crates/jig-protocol-store-settings",
        schemas: &["schema/settings-v3.schema"],
        generated: &["src/generated/settings_v3.rs"],
        golden: "golden/settings",
        fixtures: &[],
    },
    Family {
        name: "secrets",
        root: "crates/jig-protocol-store",
        codec_root: "crates/jig-protocol-store-families",
        schemas: &["schema/secrets-v1.schema"],
        generated: &["src/generated/secrets_v1.rs"],
        golden: "golden/secrets",
        fixtures: &[],
    },
    Family {
        name: "test connector",
        root: "testing/jig-test-store",
        codec_root: "testing/jig-test-store-codec",
        schemas: &["schema/test-connector-v2.schema"],
        generated: &["src/generated/v2.rs"],
        golden: "golden",
        fixtures: &[],
    },
    Family {
        name: "worker wire",
        root: "crates/jig-protocol-worker-wire",
        codec_root: "crates/jig-protocol-worker-wire",
        schemas: &["schema/worker-wire-v11.schema"],
        generated: &["src/generated/v11.rs"],
        golden: "golden",
        fixtures: &[],
    },
    Family {
        name: "accounts moved",
        root: "crates/jig-protocol-accounts",
        codec_root: "crates/jig-protocol-accounts-codec",
        schemas: &["schema/moved-v1.schema"],
        generated: &["src/generated/v1.rs"],
        golden: "golden",
        fixtures: &[],
    },
    Family {
        name: "public",
        root: "crates/jig-client-protocol-wire",
        codec_root: "crates/jig-client-protocol-wire",
        schemas: &["schema/public-v1.schema"],
        generated: &["src/generated/public_v1.rs"],
        golden: "golden/public",
        fixtures: &[],
    },
];

#[expect(
    clippy::disallowed_methods,
    reason = "jig's round: reads SKEIN_CODEGEN_WRITE to refresh generated codecs and goldens"
)]
pub fn check() {
    let tree = skein_scratch::paths::tree();
    let write = std::env::var_os("SKEIN_CODEGEN_WRITE").is_some_and(|value| value == "1");
    let again = "run SKEIN_CODEGEN_WRITE=1 cargo nextest run -p jig-client-wire-world -p jig-store-codecs-world -E 'test(drift::)'";
    let mut inventories: BTreeMap<_, BTreeSet<_>> = BTreeMap::new();
    for family in FAMILIES {
        let root = tree.join(family.root);
        let mut source = String::new();
        for schema in family.schemas {
            source.push_str(&fs::read_to_string(root.join(schema)).expect("schema readable"));
        }
        let output = generate(&parse(&source).expect("schema valid"));
        let codec_root = tree.join(family.codec_root);
        let code = codec_root.join(family.generated.first().expect("codec path"));
        inventories
            .entry(root.join("src/generated"))
            .or_default()
            .extend(family.generated.iter().skip(1).map(|path| root.join(path)));
        inventories.entry(codec_root.join("src/generated")).or_default().insert(code.clone());
        let goldens = root.join(family.golden);
        if write {
            fs::write(&code, &output.rust).expect("write generated code");
            fs::create_dir_all(&goldens).expect("create golden directory");
        } else {
            assert_eq!(
                fs::read_to_string(&code).expect("generated code committed"),
                output.rust,
                "{}: {again}",
                family.name
            );
        }
        let mut expected: BTreeSet<_> = family.fixtures.iter().map(|name| goldens.join(name)).collect();
        for golden in output.goldens {
            let path = goldens.join(golden.name);
            expected.insert(path.clone());
            if write {
                fs::write(&path, golden.bytes).expect("write golden bytes");
            } else {
                assert_eq!(fs::read(&path).expect("golden committed"), golden.bytes, "{}: {again}", family.name);
            }
        }
        let extras: Vec<_> = entries(&goldens).difference(&expected).cloned().collect();
        if write {
            for path in extras {
                fs::remove_file(path).expect("remove golden outside the schema");
            }
        } else {
            assert!(extras.is_empty(), "{}: goldens outside the schema: {extras:?}", family.name);
        }
    }
    for root in FAMILIES.iter().map(|family| family.root).collect::<BTreeSet<_>>() {
        if root == "crates/jig-protocol-store" {
            let goldens: BTreeSet<_> = FAMILIES
                .iter()
                .filter(|family| family.root == root)
                .map(|family| tree.join(root).join(family.golden))
                .collect();
            assert_eq!(entries(&tree.join(root).join("golden")), goldens, "store golden directories");
        }
    }
    // Patch 1 keeps its v16 executor decoder after the live people family moves.
    let kept_people = tree.join("crates/jig-protocol-store-people/src/generated");
    inventories
        .get_mut(&kept_people)
        .expect("the live people codec has an inventory")
        .insert(kept_people.join("people_v16.rs"));
    for (directory, expected) in inventories {
        assert_eq!(entries(&directory), expected, "{}: generated inventory", directory.display());
    }
}

fn entries(path: &Path) -> BTreeSet<PathBuf> {
    let entries = match fs::read_dir(path) {
        Ok(entries) => entries,
        Err(why) if why.kind() == std::io::ErrorKind::NotFound => return BTreeSet::new(),
        Err(why) => panic!("{}: directory readable: {why}", path.display()),
    };
    entries.map(|entry| entry.expect("entry readable").path()).collect()
}
