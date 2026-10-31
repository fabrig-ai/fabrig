//! Each generated jig codec's schema, code and golden inventory (protocol/README.md, 7).
use skein_codegen::{generate, parse};
use std::{
    collections::BTreeSet,
    fs,
    path::{Path, PathBuf},
};

pub struct Family {
    pub name: &'static str,
    pub root: &'static str,
    pub schemas: &'static [&'static str],
    pub generated: &'static [&'static str],
    pub golden: &'static str,
    pub fixtures: &'static [&'static str],
}

pub const FAMILIES: &[Family] = &[
    Family {
        name: "client",
        root: "crates/jig-client-protocol-wire",
        schemas: &["schema/client-v19.schema", "schema/client-v19-part1.schema"],
        generated: &["src/generated/v19.rs"],
        golden: "golden",
        fixtures: &["task_hold_placed_full.bin", "task_hold_placed_smallest.bin"],
    },
    Family {
        name: "view records",
        root: "crates/jig-client-view",
        schemas: &["schema/records-v1.schema"],
        generated: &["src/generated/records_v1.rs"],
        golden: "golden",
        fixtures: &[],
    },
    Family {
        name: "ops client",
        root: "examples/ops/client-protocol-wire",
        schemas: &["schema/ops-client-v1.schema"],
        generated: &["src/generated/v1.rs"],
        golden: "golden",
        fixtures: &[],
    },
    Family {
        name: "test client",
        root: "testing/jig-client-test-protocol-wire",
        schemas: &["schema/test-client-v2.schema"],
        generated: &["src/generated/v2.rs"],
        golden: "golden",
        fixtures: &[],
    },
    Family {
        name: "component",
        root: "crates/jig-protocol-store",
        schemas: &["schema/component-v3.schema"],
        generated: &["src/generated/component_v3.rs", "src/generated/layouts.rs"],
        golden: "golden/component",
        fixtures: &[],
    },
    Family {
        name: "core",
        root: "crates/jig-protocol-store",
        schemas: &["schema/core-v6.schema"],
        generated: &["src/generated/core_v6.rs"],
        golden: "golden/core",
        fixtures: &[],
    },
    Family {
        name: "tasks",
        root: "crates/jig-protocol-store",
        schemas: &["schema/tasks-v9.schema"],
        generated: &["src/generated/tasks_v9.rs"],
        golden: "golden/tasks",
        fixtures: &[],
    },
    Family {
        name: "people",
        root: "crates/jig-protocol-store",
        schemas: &["schema/people-v13.schema"],
        generated: &["src/generated/people_v13.rs"],
        golden: "golden/people",
        fixtures: &[],
    },
    Family {
        name: "notes",
        root: "crates/jig-protocol-store",
        schemas: &["schema/notes-v1.schema"],
        generated: &["src/generated/notes_v1.rs"],
        golden: "golden/notes",
        fixtures: &[],
    },
    Family {
        name: "accounts",
        root: "crates/jig-protocol-store",
        schemas: &["schema/accounts-v2.schema"],
        generated: &["src/generated/accounts_v2.rs"],
        golden: "golden/accounts",
        fixtures: &[],
    },
    Family {
        name: "settings",
        root: "crates/jig-protocol-store",
        schemas: &["schema/settings-v1.schema"],
        generated: &["src/generated/settings_v1.rs"],
        golden: "golden/settings",
        fixtures: &[],
    },
    Family {
        name: "secrets",
        root: "crates/jig-protocol-store",
        schemas: &["schema/secrets-v1.schema"],
        generated: &["src/generated/secrets_v1.rs"],
        golden: "golden/secrets",
        fixtures: &[],
    },
    Family {
        name: "test connector",
        root: "testing/jig-test-store",
        schemas: &["schema/test-connector-v2.schema"],
        generated: &["src/generated/v2.rs"],
        golden: "golden",
        fixtures: &[],
    },
    Family {
        name: "worker wire",
        root: "crates/jig-protocol-worker-wire",
        schemas: &["schema/worker-wire-v9.schema"],
        generated: &["src/generated/v9.rs"],
        golden: "golden",
        fixtures: &[],
    },
    Family {
        name: "accounts moved",
        root: "crates/jig-protocol-accounts",
        schemas: &["schema/moved-v1.schema"],
        generated: &["src/generated/v1.rs"],
        golden: "golden",
        fixtures: &[],
    },
];

pub fn check() {
    let tree = skein_scratch::paths::tree();
    let write = std::env::var_os("SKEIN_CODEGEN_WRITE").is_some_and(|value| value == "1");
    let again = "run SKEIN_CODEGEN_WRITE=1 heavy cargo nextest run -p jig-client-wire-world -p jig-store-codecs-world -E 'test(drift::)'";
    for family in FAMILIES {
        let root = tree.join(family.root);
        let mut source = String::new();
        for schema in family.schemas {
            source.push_str(&fs::read_to_string(root.join(schema)).expect("schema readable"));
        }
        let output = generate(&parse(&source).expect("schema valid"));
        let code = root.join(family.generated.first().expect("codec path"));
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
        let expected: BTreeSet<_> = FAMILIES
            .iter()
            .filter(|family| family.root == root)
            .flat_map(|family| family.generated.iter().map(|path| tree.join(root).join(path)))
            .collect();
        if root == "crates/jig-protocol-store" {
            let goldens: BTreeSet<_> = FAMILIES
                .iter()
                .filter(|family| family.root == root)
                .map(|family| tree.join(root).join(family.golden))
                .collect();
            assert_eq!(entries(&tree.join(root).join("golden")), goldens, "store golden directories");
        }
        assert_eq!(entries(&tree.join(root).join("src/generated")), expected, "{root}: generated inventory");
    }
}

fn entries(path: &Path) -> BTreeSet<PathBuf> {
    fs::read_dir(path).expect("directory readable").map(|entry| entry.expect("entry readable").path()).collect()
}
