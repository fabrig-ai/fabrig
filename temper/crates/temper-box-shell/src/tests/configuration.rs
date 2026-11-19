//! Configuration reference pages (temper/docs/design/protocol/documentation.md, 6 and 8).
#[path = "../../../../testing/temper-box-kit/src/configuration.rs"]
#[expect(dead_code, reason = "these shell tests use the shared startup documents, not the cluster's figures")]
pub(crate) mod support;
use core::num::NonZeroU16;
use serde_json::Value;
pub(crate) use support::{check_document, demo_document, directories, documents, flags, setup, typed_documents};
use temper_service::startup::members;

pub(super) fn init_document() -> Box<[u8]> {
    // protocol/setup/init.md, 2: fixed starting ports and deterministic facts for the reference rendering.
    let ports = [8367, 8368, 8369, 8370, 8371].map(|port| NonZeroU16::new(port).expect("fixed nonzero port"));
    let facts = temper_init::Facts {
        sign_in: true,
        binary: b"/opt/temper/bin/temper-box",
        git: Some(b"/usr/bin/git"),
        claude: Some(b"/usr/bin/claude"),
        ports: temper_init::Ports { clients: ports[0], webhooks: ports[4] },
        login: Some(b"person"),
    };
    let directories = temper_init::Directories { config: b"/etc/temper", data: b"/var/lib/temper" };
    let answer = temper_init::files(&facts, &directories, &[0; 32]).expect("init fixture");
    for file in answer.files {
        if file.name == b"temper.json" {
            let temper_init::Content::Bytes { bytes, .. } = file.content else { panic!("document bytes") };
            return bytes;
        }
    }
    panic!("init writes temper.json")
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
