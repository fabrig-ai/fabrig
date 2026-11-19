//! Each jig codec's records and pinned constructs (skein's data/codec/README.md, 6).
use skein_codegen::{form_of, generate_as, values, unpinned};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
};

pub struct Family {
    pub name: &'static str,
    pub root: &'static str,
    pub codec_root: &'static str,
    pub text: &'static str,
    /// Record code under `codec_root`, then glue under `root`.
    pub generated: &'static [&'static str],
}

pub const FAMILIES: &[Family] = &[
    Family {
        name: "engine cursor",
        text: jig_protocol_engine_clients::SCHEMA,
        root: "crates/jig-protocol-engine-clients",
        codec_root: "crates/jig-protocol-engine-clients-codec",
        generated: &["src/generated/cursor.rs"],
    },
    Family {
        name: "client",
        text: jig_client_protocol_wire::CLIENT_SCHEMA,
        root: "crates/jig-client-protocol-wire",
        codec_root: "crates/jig-client-protocol-wire",
        generated: &["src/generated/client.rs"],
    },
    Family {
        name: "view records",
        text: jig_client_view_codec::SCHEMA,
        root: "crates/jig-client-view",
        codec_root: "crates/jig-client-view-codec",
        generated: &["src/generated/records.rs"],
    },
    Family {
        name: "ops client",
        text: jig_ops_client_protocol_wire::SCHEMA,
        root: "examples/ops/client-protocol-wire",
        codec_root: "examples/ops/client-protocol-wire",
        generated: &["src/generated/ops_client.rs"],
    },
    Family {
        name: "test client",
        text: jig_client_test_protocol_wire::SCHEMA,
        root: "testing/jig-client-test-protocol-wire",
        codec_root: "testing/jig-client-test-protocol-wire-codec",
        generated: &["src/generated/test_client.rs"],
    },
    Family {
        name: "component",
        text: jig_protocol_store::STORE_FAMILIES[0].current,
        root: "crates/jig-protocol-store",
        codec_root: "crates/jig-protocol-store-families",
        generated: &["src/generated/component.rs"],
    },
    Family {
        name: "core",
        text: jig_protocol_store::STORE_FAMILIES[3].current,
        root: "crates/jig-protocol-store",
        codec_root: "crates/jig-protocol-store-core",
        generated: &["src/generated/core.rs"],
    },
    Family {
        name: "tasks",
        text: jig_protocol_store::STORE_FAMILIES[4].current,
        root: "crates/jig-protocol-store",
        codec_root: "crates/jig-protocol-store-tasks",
        generated: &["src/generated/tasks.rs"],
    },
    Family {
        name: "people",
        text: jig_protocol_store::STORE_FAMILIES[5].current,
        root: "crates/jig-protocol-store",
        codec_root: "crates/jig-protocol-store-people",
        generated: &["src/generated/people.rs"],
    },
    Family {
        name: "registry",
        text: jig_protocol_store::STORE_FAMILIES[2].current,
        root: "crates/jig-protocol-store",
        codec_root: "crates/jig-protocol-store-families",
        generated: &["src/generated/registry.rs"],
    },
    Family {
        name: "openings",
        text: jig_protocol_store::STORE_FAMILIES[1].current,
        root: "crates/jig-protocol-store",
        codec_root: "crates/jig-protocol-store-families",
        generated: &["src/generated/openings.rs"],
    },
    Family {
        name: "notes",
        text: jig_protocol_store::STORE_FAMILIES[6].current,
        root: "crates/jig-protocol-store",
        codec_root: "crates/jig-protocol-store-families",
        generated: &["src/generated/notes.rs"],
    },
    Family {
        name: "accounts",
        text: jig_protocol_store::STORE_FAMILIES[7].current,
        root: "crates/jig-protocol-store",
        codec_root: "crates/jig-protocol-store-families",
        generated: &["src/generated/accounts.rs"],
    },
    Family {
        name: "settings",
        text: jig_protocol_store::STORE_FAMILIES[8].current,
        root: "crates/jig-protocol-store",
        codec_root: "crates/jig-protocol-store-settings",
        generated: &["src/generated/settings.rs"],
    },
    Family {
        name: "secrets",
        text: jig_protocol_store::STORE_FAMILIES[9].current,
        root: "crates/jig-protocol-store",
        codec_root: "crates/jig-protocol-store-families",
        generated: &["src/generated/secrets.rs"],
    },
    Family {
        name: "test connector",
        text: jig_test_store::SCHEMA,
        root: "testing/jig-test-store",
        codec_root: "testing/jig-test-store-codec",
        generated: &["src/generated/test_connector.rs"],
    },
    Family {
        name: "worker wire",
        text: jig_protocol_worker_wire::SCHEMA,
        root: "crates/jig-protocol-worker-wire",
        codec_root: "crates/jig-protocol-worker-wire",
        generated: &["src/generated/worker_wire.rs"],
    },
    Family {
        name: "accounts moved",
        text: jig_protocol_accounts::SCHEMA,
        root: "crates/jig-protocol-accounts",
        codec_root: "crates/jig-protocol-accounts-codec",
        generated: &["src/generated/moved.rs"],
    },
    Family {
        name: "public",
        text: jig_client_protocol_wire::PUBLIC_SCHEMA,
        root: "crates/jig-client-protocol-wire",
        codec_root: "crates/jig-client-protocol-wire",
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
        let source = family.text;
        let unpinned = unpinned(source).expect("schema valid");
        assert!(unpinned.is_empty(), "{}: unpinned constructs: {unpinned:?}", family.name);
        let schema = &values(source).schema;
        let form = form_of(&root.join("schema"), &schema.family).expect("family form readable");
        let output = generate_as(schema, form);
        if family.name == "client" {
            client_layout(&root, source, write, again);
        }
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

// Client layouts have no bridge: retain every old line and append the current one.
fn client_layout(root: &Path, source: &str, write: bool, again: &str) {
    use std::fmt::Write as _;
    let schema = skein_codec::schema::parse(source.as_bytes()).expect("client schema valid");
    let mut line = format!("{} ", schema.version());
    for byte in schema.fingerprints().family().digest() {
        write!(&mut line, "{byte:02x}").expect("fingerprint text");
    }
    line.push('\n');
    let path = root.join("schema/jig-client-wire.layouts");
    let mut layouts = fs::read_to_string(&path).expect("client layouts");
    if write && !layouts.ends_with(&line) {
        assert!(
            !layouts.lines().any(|held| held.starts_with(&format!("{} ", schema.version()))),
            "a committed layout is immutable"
        );
        layouts.push_str(&line);
        fs::write(&path, &layouts).expect("append client layout");
    }
    assert!(layouts.ends_with(&line), "current client layout: {again}");
}
