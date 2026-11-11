//! Configuration reference pages (temper/docs/design/protocol/documentation.md, 6 and 8).
#[path = "../../../../bin/temper-box/tests/support/configuration.rs"]
mod support;
use core::num::NonZeroU16;
use serde_json::Value;
use skein_lib::Writer;
pub(crate) use support::{
    check_document, demo_document, directories, documents, fake_entry, flags, old_form_check_document,
};
use temper_service::startup::{self, members};

type Renderer = fn(&[u8], &mut Writer) -> Result<(), skein_lib::Overflow>;

pub(super) fn init_document() -> Box<[u8]> {
    // protocol/setup/init.md, 2: fixed starting ports and deterministic facts for the reference rendering.
    let ports = [8367, 8368, 8369, 8370, 8371].map(|port| NonZeroU16::new(port).expect("fixed nonzero port"));
    let facts = temper_init::Facts {
        sign_in: true,
        binary: b"/opt/temper/bin/temper-box",
        git: Some(b"/usr/bin/git"),
        claude: Some(b"/usr/bin/claude"),
        ports: temper_init::Ports {
            clients: ports[0],
            owned: ports[1],
            forge: ports[2],
            llm: ports[3],
            webhooks: ports[4],
        },
        login: Some(b"person"),
    };
    let directories = temper_init::Directories { config: b"/etc/temper", data: b"/var/lib/temper" };
    let answer = temper_init::files(&facts, &directories, &[0; 32], &[1; 32]).expect("init fixture");
    for file in answer.files {
        if file.name == b"temper.json" {
            let temper_init::Content::Bytes { bytes, .. } = file.content else { panic!("document bytes") };
            return bytes;
        }
    }
    panic!("init writes temper.json")
}

#[test]
fn the_pages_configuration_are_what_the_tables_render() {
    let init = init_document();
    let directory = skein_scratch::paths::tree().join("docs/help/reference");
    let pages: [(&str, Renderer); 2] = [
        ("configuration", startup::configuration_reference),
        ("configuration-engine", startup::engine_configuration_reference),
    ];
    let write = temper_test_env::rewrite();
    for (name, render) in pages {
        let mut measured = Writer::new(temper_docs::PAGE_BYTES);
        render(&init, &mut measured).expect("configuration page fits PAGE_BYTES");
        let mut out = Writer::new(measured.written());
        render(&init, &mut out).expect("configuration page fits its measured length");
        let rendered = String::from_utf8(out.finish().into_vec()).expect("reference UTF-8");
        assert!(rendered.len() <= temper_docs::PAGE_BYTES, "{name}: {} bytes", rendered.len());
        assert_eq!(rendered.matches("restart").count(), 1, "one restart rule per page");
        render(&init, &mut Writer::new(rendered.len() - 1)).expect_err("one byte short");
        let path = directory.join(name).with_extension("md");
        if write {
            std::fs::write(path, &rendered).expect("write configuration reference");
        } else {
            assert_eq!(
                std::fs::read_to_string(path).expect("configuration page"),
                rendered,
                "{name} should be:\n{rendered}; regenerate with SKEIN_CODEGEN_WRITE=1 and this test"
            );
            let page = temper_docs::find(name.as_bytes()).expect("compiled configuration page");
            assert_eq!(page.kind, temper_docs::Kind::Reference);
            assert_eq!(page.text, rendered, "compiled {name} should be:\n{rendered}");
        }
    }
}

fn row(parent: &str, name: &str) -> Option<&'static members::Member> {
    for (shapes, rows) in [
        (&members::SHAPES[..], &members::MEMBERS[..]),
        (&jig_protocol_config::members::SHAPES[..], &jig_protocol_config::members::MEMBERS[..]),
    ] {
        for shape in shapes {
            let takes_shape =
                shape.paths.contains(&parent) || shape.name == "address" && members::ADDRESS_PATHS.contains(&parent);
            if takes_shape && let Some(row) = rows.iter().find(|row| row.shape == shape.name && row.name == name) {
                return Some(row);
            }
        }
    }
    None
}

fn check_members(value: &Value, parent: &str) {
    match value {
        Value::Object(object) => {
            for (name, value) in object {
                let path = if parent.is_empty() { name.clone() } else { format!("{parent}.{name}") };
                let row = row(parent, name).unwrap_or_else(|| panic!("init writes {path}, which has no member row"));
                assert!(row.gone.is_none(), "init writes refused member {path}");
                // Tool inputs are opaque JSON; their members belong to the called tool's schema.
                // Overrides are keyed by the limits table, rather than configuration shape members.
                if name == "overrides" {
                    for name in value.as_object().expect("named overrides").keys() {
                        let known = if parent.starts_with("application") {
                            temper_service::limits::NAMES.iter().any(|row| row.name == name)
                        } else {
                            jig_protocol_config::names::NAMES.iter().any(|row| row.name == name)
                        };
                        assert!(known, "init writes {path}.{name}, which has no named limit row");
                    }
                } else if row.kind != "JSON value" {
                    check_members(value, &path);
                }
            }
        }
        Value::Array(items) => {
            for value in items {
                check_members(value, &format!("{parent}[]"));
            }
        }
        Value::Null | Value::Bool(_) | Value::Number(_) | Value::String(_) => {}
    }
}

#[test]
fn every_member_init_writes_is_a_row() {
    let document: Value = serde_json::from_slice(&init_document()).expect("init JSON");
    check_members(&document, "");
}
