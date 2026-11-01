//! Standalone command fixtures use group files (temper/docs/design/draft/playground-reset.md, 3).
use crate::support;
use serde_json::{Value, json};
use std::{fs, os::unix::fs::PermissionsExt, path::Path};

pub fn write(directory: &Path, forge: u16, entry: &Value) {
    let bytes =
        temper_forge_setup::setup::group_settings(&serde_json::to_vec(&body(entry)).expect("entry JSON"), forge)
            .expect("group settings");
    let groups = directory.join("box/groups");
    fs::create_dir_all(&groups).expect("group directory");
    fs::set_permissions(directory.join("box"), fs::Permissions::from_mode(0o700)).expect("private box");
    fs::set_permissions(&groups, fs::Permissions::from_mode(0o700)).expect("private groups");
    support::write_private(&groups.join(forge.to_string()), &bytes);
}
fn body(entry: &Value) -> Value {
    let mut body = entry.clone();
    for name in ["forge", "directory", "start"] {
        body.as_object_mut().expect("entry object").remove(name);
    }
    body
}
pub fn save(directory: &Path, mut document: Value) {
    if let Some(entries) = document["application"]["setup"].as_array() {
        for entry in entries {
            write(
                directory,
                entry["forge"].as_u64().expect("forge number").try_into().expect("u16 forge"),
                &body(entry),
            );
        }
    }
    template(&mut document);
    support::write_private(&directory.join("temper.json"), &serde_json::to_vec(&document).expect("document JSON"));
}
pub fn template(document: &mut Value) {
    let application = &mut document["application"];
    let owned = application["setup"]
        .as_array()
        .and_then(|entries| entries.iter().find(|entry| entry["mode"].get("owned").is_some()))
        .map(body);
    if application.get("machine").is_none() {
        application["machine"] =
            json!({"owned":null,"github":{"api":null,"transport":null,"git":null,"web":null},"git_program":null});
    }
    if let Some(owned) = owned {
        application["machine"]["owned"] = owned;
    }
    for name in ["forges", "identities", "setup"] {
        application.as_object_mut().expect("application").remove(name);
    }
}
pub fn migrate(directory: &Path) {
    let document =
        serde_json::from_slice(&fs::read(directory.join("temper.json")).expect("configuration")).expect("JSON");
    save(directory, document);
}
pub fn scripted(directory: &Path) -> support::owned::Programs {
    let programs = support::owned::scripted(directory);
    migrate(directory);
    programs
}
pub fn configuration(directory: &Path, names: &support::owned::Names<'_>) {
    support::owned::configuration(directory, names);
    migrate(directory);
}
