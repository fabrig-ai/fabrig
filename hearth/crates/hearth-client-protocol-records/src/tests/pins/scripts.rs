use super::*;
use crate::{
    diff::ScriptsChange,
    parts::{config::Script, software},
};
use skein_lib::bytes::copy_of;

fn script(task: &[u8], digest: [u8; 32]) -> Script {
    Script { task: copy_of(task), order: 300, content_digest: digest }
}
fn config(scripts: ScriptsChange) -> Record {
    let mut config = config_change();
    config.scripts = Some(scripts);
    change(PartChange::Config(config))
}
fn software(scripts: ScriptsChange) -> Record {
    change(PartChange::Software(software::Change { downloads: Box::new([]), scripts: Some(scripts) }))
}
fn added() -> ScriptsChange {
    ScriptsChange { before: Box::new([]), after: Box::new([script(b"new", [2; 32])]) }
}
fn removed() -> ScriptsChange {
    ScriptsChange { before: Box::new([script(b"gone", [1; 32])]), after: Box::new([]) }
}
fn mixed() -> ScriptsChange {
    ScriptsChange {
        before: Box::new([script(b"gone", [1; 32]), script(b"keep", [3; 32])]),
        after: Box::new([script(b"keep", [3; 32]), script(b"new", [2; 32])]),
    }
}
macro_rules! config_json {
    ($body:literal) => {
        change_json!(
            r#"{"kind":"config","groups":[],"users":[],"members":[],"presets":[],"scripts":{"#,
            $body,
            r#"},"tmpfiles":[],"tree":[]}"#
        )
    };
}
macro_rules! software_json {
    ($body:literal) => {
        change_json!(r#"{"kind":"software","software":{"downloads":[],"scripts":{"#, $body, "}}}")
    };
}
#[test]
fn scripts_added_removed_and_unchanged_in_both_parts() {
    check(&[
        (
            config(added()),
            change_text!("  config   + scripts                                new 300 020202020202\n"),
            config_json!(
                r#""after":[{"content_digest":"0202020202020202020202020202020202020202020202020202020202020202","order":300,"task":"new"}],"before":[]"#
            ),
        ),
        (
            config(removed()),
            change_text!("  config   - scripts                                gone 300 010101010101\n"),
            config_json!(
                r#""after":[],"before":[{"content_digest":"0101010101010101010101010101010101010101010101010101010101010101","order":300,"task":"gone"}]"#
            ),
        ),
        (
            config(mixed()),
            change_text!(
                "  config   - scripts                                gone 300 010101010101\n  config   + scripts                                new 300 020202020202\n"
            ),
            config_json!(
                r#""after":[{"content_digest":"0303030303030303030303030303030303030303030303030303030303030303","order":300,"task":"keep"},{"content_digest":"0202020202020202020202020202020202020202020202020202020202020202","order":300,"task":"new"}],"before":[{"content_digest":"0101010101010101010101010101010101010101010101010101010101010101","order":300,"task":"gone"},{"content_digest":"0303030303030303030303030303030303030303030303030303030303030303","order":300,"task":"keep"}]"#
            ),
        ),
        (
            software(added()),
            change_text!("  software + scripts                                new 300 020202020202\n"),
            software_json!(
                r#""after":[{"content_digest":"0202020202020202020202020202020202020202020202020202020202020202","order":300,"task":"new"}],"before":[]"#
            ),
        ),
        (
            software(removed()),
            change_text!("  software - scripts                                gone 300 010101010101\n"),
            software_json!(
                r#""after":[],"before":[{"content_digest":"0101010101010101010101010101010101010101010101010101010101010101","order":300,"task":"gone"}]"#
            ),
        ),
        (
            software(mixed()),
            change_text!(
                "  software - scripts                                gone 300 010101010101\n  software + scripts                                new 300 020202020202\n"
            ),
            software_json!(
                r#""after":[{"content_digest":"0303030303030303030303030303030303030303030303030303030303030303","order":300,"task":"keep"},{"content_digest":"0202020202020202020202020202020202020202020202020202020202020202","order":300,"task":"new"}],"before":[{"content_digest":"0101010101010101010101010101010101010101010101010101010101010101","order":300,"task":"gone"},{"content_digest":"0303030303030303030303030303030303030303030303030303030303030303","order":300,"task":"keep"}]"#
            ),
        ),
    ]);
}
