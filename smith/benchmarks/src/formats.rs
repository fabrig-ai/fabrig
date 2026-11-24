//! Strict TOML reads retain no state beyond their returned document.
//! Each refusal names its file and key path (../../docs/design/benchmarks/README.md, section 5).

use std::path::Path;

use serde::de::DeserializeOwned;

use crate::Refusal;
use crate::files::{Files, Native};

pub(crate) fn read_document<T: DeserializeOwned>(file: &Path) -> Result<T, Refusal> {
    read_document_with(&Native, file)
}

/// Read a strict TOML document from the supplied filesystem in a step test.
pub fn read_document_with<T: DeserializeOwned>(files: &dyn Files, file: &Path) -> Result<T, Refusal> {
    let text = files.read_to_string(file).map_err(|error| Refusal::new(file, "document", error.to_string()))?;
    parse_document(file, &text)
}

/// Parse a strict TOML document, naming the key in a refusal.
pub fn parse_document<T: DeserializeOwned>(file: &Path, text: &str) -> Result<T, Refusal> {
    let deserializer =
        toml::de::Deserializer::parse(text).map_err(|error| Refusal::new(file, "document", error.to_string()))?;
    serde_path_to_error::deserialize(deserializer).map_err(|error| {
        let mut key = error.path().to_string();
        let reason = error.inner().to_string();
        if let Some(rest) = reason.split("unknown field `").nth(1)
            && let Some(field) = rest.split('`').next()
        {
            if key == "." {
                key.clear();
            }
            if key != field && !key.ends_with(&format!(".{field}")) {
                if !key.is_empty() {
                    key.push('.');
                }
                key.push_str(field);
            }
        }
        Refusal::new(file, &key, reason)
    })
}

/// Fixed-width lowercase identities in committed manifests.
pub(crate) fn lowercase_hex(value: &str, length: usize) -> bool {
    value.len() == length
        && value.bytes().all(|byte| {
            skein_lib::bytes::hex_value(byte).is_some_and(|value| skein_lib::bytes::hex_digit(value) == byte)
        })
}
