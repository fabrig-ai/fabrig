//! The pure workspace-call decoder and fixture bounds recover reference presence
//! from the provider's original input in recorded turns that lack the live
//! session's tickets (smith/docs/design/shapes.md, 2–3).

use skein_lib::Duration;
use smith_domain_session::llm as agent;
use smith_domain_tools::{self as tools, Grants};

/// The tools' limits in the calm world: a kit for each session, as many calls
/// a kit as the session runs at once, and room for the fixture's files.
pub const TOOLS: tools::Limits = tools::Limits {
    shell_output: 512,
    kits: 4,
    repos: 1,
    path_bytes: 256,
    known_files: 16,
    file_bytes: 1 << 16,
    tool_payload: 1 << 16,
    read_bytes: 4096,
    list_entries: 64,
    match_lines: 8,
    file_timeout: Duration::from_secs(30),
    shell_timeout: Duration::from_secs(30),
    tool_deadline: Duration::from_secs(120),
    shell_head: 256,
    shell_tail: 256,
    search_hits: 16,
    search_bytes: 1024,
    ..smith_domain_tools_fixtures::SMALL
};

/// The call the LLM made to the tool `name` with `arguments`, as the agent's
/// protocol layer decodes it: or what keeps it from being one.
#[must_use]
pub fn decode(name: &[u8], arguments: &[u8]) -> agent::Decoded {
    decode_bounded(name, arguments, &decode_limits(&TOOLS))
}

/// Protocol bounds for a session fixture with these tools limits.
#[must_use]
pub fn decode_limits(tools: &smith_domain_tools::Limits) -> smith_protocol_llm::Limits {
    let mut client = skein_llm_kit::limits();
    // Like the profile, leave room for the longest path and Write's JSON
    // framing, then apply the native escape factor (limits-derivation.md, 1).
    let framing = u32::try_from(br#"{"path":"","content":""}"#.len()).expect("small framing");
    let escape = skein_llm::ESCAPE;
    let input = tools.tool_payload.saturating_add(tools.path_bytes).saturating_add(framing.div_ceil(escape));
    client.input = client.input.min(input.saturating_mul(escape));
    let stated = smith_protocol_llm::Stated::from_tools(tools, client.input).expect("finite workspace fixture");
    smith_protocol_llm_fixtures::small(client, stated)
}

/// Decode a workspace call using the given protocol bounds.
#[must_use]
pub fn decode_bounded(name: &[u8], arguments: &[u8], limits: &smith_protocol_llm::Limits) -> agent::Decoded {
    use smith_domain::llm;

    let grants = Grants { inspect: true, modify: true, shell: true };
    match smith_protocol_llm::decode(name, arguments, grants, &[], llm::Asker::Main, limits) {
        llm::Decoded::Owned { call } => agent::Decoded::Owned { call },
        llm::Decoded::Invalid { problem } => agent::Decoded::Invalid { descriptor: None, problem },
        llm::Decoded::Builtin { .. } | llm::Decoded::Served { .. } => unreachable!("only workspace tools are granted"),
    }
}

/// Whether the original JSON object supplied a path and an offset, respectively.
#[must_use]
pub fn path_fields(input: &[u8]) -> (bool, bool) {
    let client = skein_llm_kit::limits();
    let bytes = u32::try_from(input.len()).unwrap_or(u32::MAX).min(client.input);
    let bounds = skein_llm::DocumentLimits { bytes, strings: bytes, tokens: bytes, depth: client.depth };
    skein_llm::Json::from_bytes(input, &bounds).map_or((false, false), |json| {
        let doc = json.document();
        (doc.member(0, b"path").is_some(), doc.member(0, b"offset").is_some())
    })
}
