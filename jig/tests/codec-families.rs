//! Each jig codec's records and bounds (skein's data/codec/golden.md, 1).
use skein_codegen::{bounds_text, generate, translate, values};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
};

#[path = "../testing/codec-drift.rs"]
mod codec_drift;
use codec_drift::check_text;

pub struct Family {
    pub name: &'static str,
    pub root: &'static str,
    pub codec_root: &'static str,
    pub text: &'static str,
    /// Record code under `codec_root`, then glue under `root`.
    pub generated: &'static [&'static str],
    /// Compiled bounds, unless the family has its own drift world.
    pub bounds: Option<fn() -> String>,
    pub translations: &'static [Translation],
}

pub struct Translation {
    /// The end's maps text, `include_str!` of its `schema/<module>.maps`.
    pub maps: &'static str,
    /// The end's generated module, relative to the tree.
    pub module: &'static str,
}

macro_rules! compiled_bounds {
    ($module:path) => {
        Some(|| {
            use $module as codec;
            bounds_text(codec::DECLARATIONS, codec::CEILINGS.values().as_slice())
        })
    };
}

pub const FAMILIES: &[Family] = &[
    Family {
        name: "engine cursor",
        text: jig_protocol_engine_clients::SCHEMA,
        root: "crates/jig-protocol-engine-clients",
        codec_root: "crates/jig-protocol-engine-clients-codec",
        generated: &["src/generated/cursor.rs"],
        bounds: compiled_bounds!(jig_protocol_engine_clients::cursor_v2),
        translations: &[Translation {
            maps: include_str!("../crates/jig-protocol-engine-clients/schema/cursor.maps"),
            module: "crates/jig-protocol-engine-clients/src/generated/cursor.rs",
        }],
    },
    Family {
        name: "client",
        text: jig_client_protocol_wire::CLIENT_SCHEMA,
        root: "crates/jig-client-protocol-wire",
        codec_root: "crates/jig-client-protocol-wire",
        generated: &["src/generated/client.rs"],
        bounds: compiled_bounds!(jig_client_protocol_wire::v44),
        translations: &[
            Translation {
                maps: include_str!("../crates/jig-protocol-engine-clients/schema/client.maps"),
                module: "crates/jig-protocol-engine-clients/src/generated/client.rs",
            },
            Translation {
                maps: include_str!("../crates/jig-client-protocol-link/schema/client.maps"),
                module: "crates/jig-client-protocol-link/src/generated/client.rs",
            },
        ],
    },
    Family {
        name: "view records",
        text: jig_client_view_codec::SCHEMA,
        root: "crates/jig-client-view",
        codec_root: "crates/jig-client-view-codec",
        generated: &["src/generated/records.rs"],
        bounds: compiled_bounds!(jig_client_view_codec::records_v1),
        translations: &[Translation {
            maps: include_str!("../crates/jig-client-view/schema/records.maps"),
            module: "crates/jig-client-view/src/generated/records.rs",
        }],
    },
    Family {
        name: "ops client",
        text: jig_ops_client_protocol_wire::SCHEMA,
        root: "examples/ops/client-protocol-wire",
        codec_root: "examples/ops/client-protocol-wire",
        generated: &["src/generated/ops_client.rs"],
        bounds: compiled_bounds!(jig_ops_client_protocol_wire::v1),
        translations: &[],
    },
    Family {
        name: "test client",
        text: jig_client_test_protocol_wire::SCHEMA,
        root: "testing/jig-client-test-protocol-wire",
        codec_root: "testing/jig-client-test-protocol-wire-codec",
        generated: &["src/generated/test_client.rs"],
        bounds: compiled_bounds!(jig_client_test_protocol_wire::v2),
        translations: &[],
    },
    Family {
        name: "component",
        text: jig_protocol_store::STORE_FAMILIES[0].current,
        root: "crates/jig-protocol-store",
        codec_root: "crates/jig-protocol-store-families",
        generated: &["src/generated/component.rs"],
        bounds: compiled_bounds!(jig_protocol_store::component_v3),
        translations: &[Translation {
            maps: include_str!("../crates/jig-protocol-store/schema/component.maps"),
            module: "crates/jig-protocol-store-records/src/generated/component.rs",
        }],
    },
    Family {
        name: "core",
        text: jig_protocol_store::STORE_FAMILIES[3].current,
        root: "crates/jig-protocol-store",
        codec_root: "crates/jig-protocol-store-core",
        generated: &["src/generated/core.rs"],
        bounds: compiled_bounds!(jig_protocol_store::core_v6),
        translations: &[Translation {
            maps: include_str!("../crates/jig-protocol-store/schema/core.maps"),
            module: "crates/jig-protocol-store-records/src/generated/core.rs",
        }],
    },
    Family {
        name: "tasks",
        text: jig_protocol_store::STORE_FAMILIES[4].current,
        root: "crates/jig-protocol-store",
        codec_root: "crates/jig-protocol-store-tasks",
        generated: &["src/generated/tasks.rs"],
        bounds: compiled_bounds!(jig_protocol_store::tasks_v13),
        translations: &[Translation {
            maps: include_str!("../crates/jig-protocol-store/schema/tasks.maps"),
            module: "crates/jig-protocol-store-records/src/generated/tasks.rs",
        }],
    },
    Family {
        name: "people",
        text: jig_protocol_store::STORE_FAMILIES[5].current,
        root: "crates/jig-protocol-store",
        codec_root: "crates/jig-protocol-store-people",
        generated: &["src/generated/people.rs"],
        bounds: compiled_bounds!(jig_protocol_store::people_v17),
        translations: &[Translation {
            maps: include_str!("../crates/jig-protocol-store/schema/people.maps"),
            module: "crates/jig-protocol-store-records/src/generated/people.rs",
        }],
    },
    Family {
        name: "registry",
        text: jig_protocol_store::STORE_FAMILIES[2].current,
        root: "crates/jig-protocol-store",
        codec_root: "crates/jig-protocol-store-families",
        generated: &["src/generated/registry.rs"],
        bounds: compiled_bounds!(jig_protocol_store::registry_v1),
        translations: &[],
    },
    Family {
        name: "openings",
        text: jig_protocol_store::STORE_FAMILIES[1].current,
        root: "crates/jig-protocol-store",
        codec_root: "crates/jig-protocol-store-families",
        generated: &["src/generated/openings.rs"],
        bounds: compiled_bounds!(jig_protocol_store::openings_v1),
        translations: &[Translation {
            maps: include_str!("../crates/jig-protocol-store/schema/openings.maps"),
            module: "crates/jig-protocol-store/src/generated/openings.rs",
        }],
    },
    Family {
        name: "notes",
        text: jig_protocol_store::STORE_FAMILIES[6].current,
        root: "crates/jig-protocol-store",
        codec_root: "crates/jig-protocol-store-families",
        generated: &["src/generated/notes.rs"],
        bounds: compiled_bounds!(jig_protocol_store::notes_v1),
        translations: &[Translation {
            maps: include_str!("../crates/jig-protocol-store/schema/notes.maps"),
            module: "crates/jig-protocol-store-records/src/generated/notes.rs",
        }],
    },
    Family {
        name: "accounts",
        text: jig_protocol_store::STORE_FAMILIES[7].current,
        root: "crates/jig-protocol-store",
        codec_root: "crates/jig-protocol-store-families",
        generated: &["src/generated/accounts.rs"],
        bounds: compiled_bounds!(jig_protocol_store::accounts_v2),
        translations: &[Translation {
            maps: include_str!("../crates/jig-protocol-store/schema/accounts.maps"),
            module: "crates/jig-protocol-store-records/src/generated/accounts.rs",
        }],
    },
    Family {
        name: "settings",
        text: jig_protocol_store::STORE_FAMILIES[8].current,
        root: "crates/jig-protocol-store",
        codec_root: "crates/jig-protocol-store-settings",
        generated: &["src/generated/settings.rs"],
        bounds: compiled_bounds!(jig_protocol_store::settings_v3),
        translations: &[Translation {
            maps: include_str!("../crates/jig-protocol-store/schema/settings.maps"),
            module: "crates/jig-protocol-store-records/src/generated/settings.rs",
        }],
    },
    Family {
        name: "secrets",
        text: jig_protocol_store::STORE_FAMILIES[9].current,
        root: "crates/jig-protocol-store",
        codec_root: "crates/jig-protocol-store-families",
        generated: &["src/generated/secrets.rs"],
        bounds: compiled_bounds!(jig_protocol_store::secrets_v1),
        translations: &[],
    },
    Family {
        name: "test connector",
        text: jig_test_store::SCHEMA,
        root: "testing/jig-test-store",
        codec_root: "testing/jig-test-store-codec",
        generated: &["src/generated/test_connector.rs"],
        bounds: compiled_bounds!(jig_test_store::v2),
        translations: &[],
    },
    Family {
        name: "worker wire",
        text: jig_protocol_worker_wire::SCHEMA,
        root: "crates/jig-protocol-worker-wire",
        codec_root: "crates/jig-protocol-worker-wire",
        generated: &["src/generated/worker_wire.rs"],
        bounds: None,
        translations: &[
            Translation {
                maps: include_str!("../crates/jig-protocol-engine-workers/schema/worker_wire.maps"),
                module: "crates/jig-protocol-engine-workers/src/generated/worker_wire.rs",
            },
            Translation {
                maps: include_str!("../crates/jig-protocol-worker-link/schema/worker_wire.maps"),
                module: "crates/jig-protocol-worker-link/src/generated/worker_wire.rs",
            },
        ],
    },
    Family {
        name: "accounts moved",
        text: jig_protocol_accounts::SCHEMA,
        root: "crates/jig-protocol-accounts",
        codec_root: "crates/jig-protocol-accounts-codec",
        generated: &["src/generated/moved.rs"],
        bounds: None,
        translations: &[],
    },
    Family {
        name: "public",
        text: jig_client_protocol_wire::PUBLIC_SCHEMA,
        root: "crates/jig-client-protocol-wire",
        codec_root: "crates/jig-client-protocol-wire",
        generated: &["src/generated/public.rs"],
        bounds: compiled_bounds!(jig_client_protocol_wire::public_v1),
        translations: &[Translation {
            maps: include_str!("../crates/jig-protocol-engine-clients/schema/public.maps"),
            module: "crates/jig-protocol-engine-clients/src/generated/public.rs",
        }],
    },
];

#[expect(clippy::disallowed_methods, reason = "jig's round: reads SKEIN_CODEGEN_WRITE to refresh codec drift files")]
pub fn check() {
    let tree = skein_scratch::paths::tree();
    let write = std::env::var_os("SKEIN_CODEGEN_WRITE").is_some_and(|value| value == "1");
    let again = "run SKEIN_CODEGEN_WRITE=1 cargo nextest run -p jig-store-codecs-world --test drift";
    let mut inventories: BTreeMap<_, BTreeSet<_>> = BTreeMap::new();
    for family in FAMILIES {
        let root = tree.join(family.root);
        let source = family.text;

        let schema = &values(source).schema;
        let output = generate(schema);
        let codec_root = tree.join(family.codec_root);
        let code = codec_root.join(family.generated.first().expect("codec path"));
        inventories
            .entry(root.join("src/generated"))
            .or_default()
            .extend(family.generated.iter().skip(1).map(|path| root.join(path)));
        inventories.entry(codec_root.join("src/generated")).or_default().insert(code.clone());
        check_text(&code, &output.records, write, again);
        if let Some(bounds) = family.bounds {
            let path = tree.join("tests/bounds").join(code.file_name().expect("codec file")).with_extension("bounds");
            check_text(&path, &bounds(), write, again);
        }
        for translation in family.translations {
            let code = tree.join(translation.module);
            let output = translate(schema, translation.maps)
                .unwrap_or_else(|error| panic!("{}: {}: {error}", family.name, translation.module));
            inventories
                .entry(code.parent().expect("translation parent").to_path_buf())
                .or_default()
                .insert(code.clone());
            check_text(&code, &output, write, again);
        }
    }
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
