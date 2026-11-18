//! Each jig codec's records and pinned constructs (skein's data/codec/README.md, 6).
use skein_codegen::{generate, parse, unpinned};
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
}

pub const FAMILIES: &[Family] = &[
    Family {
        name: "engine cursor",
        root: "crates/jig-protocol-engine-clients",
        codec_root: "crates/jig-protocol-engine-clients-codec",
        schemas: &["schema/cursor-v2.schema"],
        generated: &["src/generated/cursor.rs"],
    },
    Family {
        name: "client",
        root: "crates/jig-client-protocol-wire",
        codec_root: "crates/jig-client-protocol-wire",
        schemas: &["schema/client-v35.schema", "schema/client-v35-part1.schema"],
        generated: &["src/generated/client.rs"],
    },
    Family {
        name: "view records",
        root: "crates/jig-client-view",
        codec_root: "crates/jig-client-view-codec",
        schemas: &["schema/records-v1.schema"],
        generated: &["src/generated/records.rs"],
    },
    Family {
        name: "ops client",
        root: "examples/ops/client-protocol-wire",
        codec_root: "examples/ops/client-protocol-wire",
        schemas: &["schema/ops-client-v1.schema"],
        generated: &["src/generated/ops_client.rs"],
    },
    Family {
        name: "test client",
        root: "testing/jig-client-test-protocol-wire",
        codec_root: "testing/jig-client-test-protocol-wire-codec",
        schemas: &["schema/test-client-v2.schema"],
        generated: &["src/generated/test_client.rs"],
    },
    Family {
        name: "component",
        root: "crates/jig-protocol-store",
        codec_root: "crates/jig-protocol-store-families",
        schemas: &["schema/component-v3.schema"],
        generated: &["src/generated/component.rs"],
    },
    Family {
        name: "core",
        root: "crates/jig-protocol-store",
        codec_root: "crates/jig-protocol-store-core",
        schemas: &["schema/core-v6.schema"],
        generated: &["src/generated/core.rs"],
    },
    Family {
        name: "tasks",
        root: "crates/jig-protocol-store",
        codec_root: "crates/jig-protocol-store-tasks",
        schemas: &["schema/tasks-v13.schema"],
        generated: &["src/generated/tasks.rs"],
    },
    Family {
        name: "people",
        root: "crates/jig-protocol-store",
        codec_root: "crates/jig-protocol-store-people",
        schemas: &["schema/people-v17.schema"],
        generated: &["src/generated/people.rs"],
    },
    Family {
        name: "registry",
        root: "crates/jig-protocol-store",
        codec_root: "crates/jig-protocol-store-families",
        schemas: &["schema/registry-v1.schema"],
        generated: &["src/generated/registry.rs"],
    },
    Family {
        name: "openings",
        root: "crates/jig-protocol-store",
        codec_root: "crates/jig-protocol-store-families",
        schemas: &["schema/openings-v1.schema"],
        generated: &["src/generated/openings.rs"],
    },
    Family {
        name: "notes",
        root: "crates/jig-protocol-store",
        codec_root: "crates/jig-protocol-store-families",
        schemas: &["schema/notes-v1.schema"],
        generated: &["src/generated/notes.rs"],
    },
    Family {
        name: "accounts",
        root: "crates/jig-protocol-store",
        codec_root: "crates/jig-protocol-store-families",
        schemas: &["schema/accounts-v2.schema"],
        generated: &["src/generated/accounts.rs"],
    },
    Family {
        name: "settings",
        root: "crates/jig-protocol-store",
        codec_root: "crates/jig-protocol-store-settings",
        schemas: &["schema/settings-v3.schema"],
        generated: &["src/generated/settings.rs"],
    },
    Family {
        name: "secrets",
        root: "crates/jig-protocol-store",
        codec_root: "crates/jig-protocol-store-families",
        schemas: &["schema/secrets-v1.schema"],
        generated: &["src/generated/secrets.rs"],
    },
    Family {
        name: "test connector",
        root: "testing/jig-test-store",
        codec_root: "testing/jig-test-store-codec",
        schemas: &["schema/test-connector-v2.schema"],
        generated: &["src/generated/test_connector.rs"],
    },
    Family {
        name: "worker wire",
        root: "crates/jig-protocol-worker-wire",
        codec_root: "crates/jig-protocol-worker-wire",
        schemas: &["schema/worker-wire-v17.schema"],
        generated: &["src/generated/worker_wire.rs"],
    },
    Family {
        name: "accounts moved",
        root: "crates/jig-protocol-accounts",
        codec_root: "crates/jig-protocol-accounts-codec",
        schemas: &["schema/moved-v1.schema"],
        generated: &["src/generated/moved.rs"],
    },
    Family {
        name: "public",
        root: "crates/jig-client-protocol-wire",
        codec_root: "crates/jig-client-protocol-wire",
        schemas: &["schema/public-v1.schema"],
        generated: &["src/generated/public.rs"],
    },
];

#[expect(clippy::disallowed_methods, reason = "jig's round: reads SKEIN_CODEGEN_WRITE to refresh generated records")]
pub fn check() {
    let tree = skein_scratch::paths::tree();
    let write = std::env::var_os("SKEIN_CODEGEN_WRITE").is_some_and(|value| value == "1");
    let again = "run SKEIN_CODEGEN_WRITE=1 cargo nextest run -p jig-client-wire-world -p jig-store-codecs-world -E 'binary(drift) | test(drift::)'";
    let mut inventories: BTreeMap<_, BTreeSet<_>> = BTreeMap::new();
    for family in FAMILIES {
        let root = tree.join(family.root);
        let mut source = String::new();
        for schema in family.schemas {
            source.push_str(&fs::read_to_string(root.join(schema)).expect("schema readable"));
        }
        let unpinned = unpinned(&source).expect("schema valid");
        assert!(unpinned.is_empty(), "{}: unpinned constructs: {unpinned:?}", family.name);
        let output = generate(&parse(&source).expect("schema valid"));
        let codec_root = tree.join(family.codec_root);
        let code = codec_root.join(family.generated.first().expect("codec path"));
        inventories
            .entry(root.join("src/generated"))
            .or_default()
            .extend(family.generated.iter().skip(1).map(|path| root.join(path)));
        inventories.entry(codec_root.join("src/generated")).or_default().insert(code.clone());
        check_text(&code, &output.records, write, again);
    }
    for (directory, expected) in inventories {
        assert_eq!(entries(&directory), expected, "{}: generated inventory", directory.display());
    }
}

fn check_text(path: &Path, text: &str, write: bool, again: &str) {
    if write {
        fs::create_dir_all(path.parent().expect("generated parent")).expect("create generated directory");
        fs::write(path, text).expect("write generated text");
    } else {
        assert_eq!(fs::read_to_string(path).expect("generated text committed"), text, "{}: {again}", path.display());
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
