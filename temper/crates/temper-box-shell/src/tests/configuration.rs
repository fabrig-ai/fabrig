//! Configuration reference pages (temper/docs/design/protocol/documentation.md, 6 and 8).
#[path = "../../../../testing/temper-box-kit/src/configuration.rs"]
#[expect(dead_code, reason = "these shell tests use the shared startup documents, not the cluster's figures")]
pub(crate) mod support;
use serde_json::Value;
pub(crate) use support::{check_document, demo_document, directories, documents, flags, setup, typed_documents};
use temper_service::startup::members;

pub(super) fn init_document() -> Box<[u8]> {
    temper_config_shell::init_document(true)
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
