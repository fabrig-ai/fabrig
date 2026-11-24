//! Shared scripted landing exchange (temper/docs/design/testing/tiers.md, 1.5).
use serde_json::{Value, json};
use std::net::SocketAddr;

pub const EDIT: &[u8] = b"Changed by temper.\n";
pub const README: &[u8] = b"# app\n";
pub const REQUESTED: &str = "Change requested";
pub const LANDED: &str = "The change landed.";
pub const CODING_WORDS: &str = "@coding Fix";
pub const REVIEW_WORDS: &str = "Review this change at the named head";

/// The coding charter and grant scope used by a landing exchange.
#[derive(Clone, Copy)]
pub struct ChangeAuthority<'a> {
    pub coding_kind: u64,
    /// None uses the first repository's deployment-wide forge grants.
    pub scope: Option<(SocketAddr, &'a str)>,
}

pub fn delegate_input(authority: ChangeAuthority<'_>, repository: u64, coding_words: &str) -> Value {
    let grant = |kind| match authority.scope {
        Some((forge, org)) => {
            json!({"connector":1,"kind":kind,"segments":["forge",forge.to_string()],"terminal":"open","last":org})
        }
        None => json!({"connector":1,"kind":kind,"segments":["forge"],"terminal":"open","last":""}),
    };
    json!({"batch":[{"executor":{"kind":"procedure","connector":1,"code":2},"spec":{"words":coding_words,"parameters":[{"name":1,"kind":"resource","connector":1,"resource":repository},{"name":2,"kind":"bytes","value":"main"}]},"contract":{"kind":"change","connector":1,"change_kind":1,"words":512},"authority":{"tools":1,"grants":[grant(2),grant(3),grant(4),grant(10),grant(11),grant(12)],"delegation":{"kinds":[{"kind":"agent","number":authority.coding_kind},{"kind":"agent","number":4}],"tasks":4,"depth":2},"budget":{"spend":10},"notes":0}}]})
}

pub fn readme_text<'a>(readme: &'a [u8], edit: &[u8]) -> (&'a str, String) {
    let original = std::str::from_utf8(readme).expect("README UTF-8");
    let changed = format!("{original}{}", std::str::from_utf8(edit).expect("edit UTF-8"));
    (original, changed)
}

pub fn change_result() -> Value {
    json!({"form":"change","fields":{"title":"Update README","body":"Changed by temper."}})
}

pub fn verdict() -> Value {
    json!({"form":"verdict","label":"1","text":"Checked","fields":{},"items":[]})
}
