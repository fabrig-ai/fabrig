#![expect(
    clippy::disallowed_types,
    clippy::disallowed_methods,
    reason = "ordinary fixture code builds and edits configuration bytes"
)]
use crate::settings;
use jig_domain_link::EndpointBound;
use skein_lib::{List, Secret};

pub const APPLICATION: &[u8] = br#"{
"webhooks":null,
"clients":{"listener":{"socket":"127.0.0.1:7100"},"public":null,"bundle":null},
"machine":{},
"landing":{"projects":[{"project":1}]},
"demo":null,"sign_in":null,"limits":{"profile":"small","overrides":{}},"workers":[{"root":"worker-root","git":"/usr/bin/git"}]
}"#;
pub const DEMO: &str = r#""welcome":true,"demo":{"llm":{"listen":{"socket":"$llm"},"provider":3,"key":"demo/llm.key",
"asked":[{"cue":"Add CSV export to widget","turns":["CSV"]},
{"cue":"Add a dark theme to widget","turns":["Theme"]},
{"cue":"Fix widget's empty search state","turns":["Search"]}],
"cued":[{"cue":"Export widget's table as CSV","turns":["Code"]}],"otherwise":["Demo"]}}"#;
pub const DEMO_SETUP: &str = r#""setup":[{"forge":1,"directory":"demo/forge",
"bot":{"login":"temper","email":"temper@example.test"},"mode":{"fake":{
"listen":{"socket":"$forge"},"git":"demo/bin/git","owner":"demo",
"repositories":[{"name":"widget","landing":"main"},{"name":"other","landing":"main"}]}}}]"#;
pub fn demo_application() -> Box<[u8]> {
    core::str::from_utf8(APPLICATION)
        .unwrap()
        .replace("\"setup\":[]", DEMO_SETUP)
        .replace("\"demo\":null", DEMO)
        .into_bytes()
        .into_boxed_slice()
}

pub fn document() -> Box<[u8]> {
    let mut document = Vec::from(b"{\"engine\":".as_slice());
    let engine = core::str::from_utf8(super::ENGINE)
        .unwrap()
        .replace(",\n{\"name\": \"worker-b\", \"digest\": \"secrets/worker-b.digest\"}", "");
    document.extend_from_slice(engine.as_bytes());
    document.extend_from_slice(br#", "workers":[{
"engine":{"in_process":"workers"},"name":"worker-a","secret":"worker-secret","slots":1,
"graces":{"grace_ms":10000,"agent_stop_ms":5000,"delivery_ms":1000,"save_ms":1000},
"programs":{"check_interval_ms":60000},
"agent":{"program":"/opt/temper/bin/temper","configuration":"/opt/temper/agent.json","arguments":["internal", "agent","/opt/temper/agent.json"]},
"limits":{"profile":"small","overrides":{"hub.slots":1,"hub.run_calls":1,"tasks.run_inbox":2,"hub.turns":1,"tasks.message_bytes":512,"people.words":512,"core.call_answer_bytes":2048}},
"memory_bytes":1152921504606846976}],"application":"#);
    document.extend_from_slice(core::str::from_utf8(APPLICATION).unwrap().replace("\"overrides\":{}", "\"overrides\":{\"checkout.name_bytes\":600,\"checkout.repositories\":4,\"git_component.stdout_bytes\":131072,\"forge.name_bytes\":256,\"forge.resources_per_task\":4}").as_bytes());
    document.extend_from_slice(b"}");
    let document = document.into_boxed_slice();
    let document = with(&document, b"engine.limits.overrides", engine_turn_limits().as_bytes());
    let document = with(&document, b"workers[0].limits.overrides", worker_turn_limits().as_bytes());
    // Retain the checkout bounds alongside the turn-derived application bounds.
    with(&document, b"application.limits.overrides", br#"{"root.journal.run_bytes":32768,"checkout.name_bytes":600,"checkout.repositories":4,"git_component.stdout_bytes":131072,"forge.name_bytes":256,"forge.resources_per_task":4}"#)
}
pub fn parsed() -> skein_json::Document {
    jig_protocol_config::document(
        &document(),
        &jig_protocol_config::DocumentLimits { bytes: 1_048_576, tokens: 16_384, depth: 32 },
    )
    .unwrap()
}
pub fn files() -> settings::Files {
    let mut entries = List::with_capacity(8);
    for (path, value) in [
        (b"worker-secret".as_slice(), super::worker::WORKER_SECRET),
        (
            b"secrets/worker-a.digest".as_slice(),
            b"8f7be71db3941de11249a5e3fba7750a41c7d9958083ef522cb79e9fd7822878".as_slice(),
        ),
        (super::AGENT_JSON, b"{}".as_slice()),
    ] {
        entries.push((path.into(), Secret::new(value.into()))).unwrap();
    }
    settings::Files { entries }
}
pub fn without(entries: List<(Box<[u8]>, Secret)>, name: &[u8]) -> List<(Box<[u8]>, Secret)> {
    let mut kept = List::with_capacity(entries.len());
    for entry in entries.into_boxed() {
        if entry.0.as_ref() != name {
            kept.push(entry).unwrap();
        }
    }
    kept
}
pub fn worker_runtime() -> crate::WorkerRuntime {
    crate::WorkerRuntime { root: skein_io::kernel::Fd::new(9), path: b"/worker".as_slice().into() }
}
pub fn startup_agent() -> settings::Agent {
    let mut agent = super::agent();
    let hub = jig_protocol_config::profile(jig_protocol_config::Profile::Small).hub;
    agent.host_limits.directories = 4;
    agent.host_limits.conflicts = 1024;
    agent.host_limits.name_bytes = 600;
    agent.host_limits.accounts = hub.accounts;
    agent.host_limits.messages = hub.held.max(hub.inbox);
    agent.host_limits.call_bytes = 256;
    agent.host_limits.message_bytes = hub.event_bytes.checked_add(temper_worker_domain::SEPARATOR).unwrap();
    agent.bounds.message_bytes = 512;
    agent.bounds.spend = 43_584;
    agent.bounds.endpoints = Box::new([EndpointBound { endpoint: 1, window: 1_048_576, output: 8192 }]);
    agent
}

// Whole-map edits in unrelated startup stories inherit this fixture's turn
// allowances. An explicitly supplied key still wins, including a short bound.
fn turn_limits(path: &[u8], json: &[u8]) -> Box<[u8]> {
    let turn = super::agent().limits.bodies.turn_body;
    let resume = turn.checked_add(4).unwrap();
    let window = u64::from(resume) * 4;
    let loaded = turn.checked_add(u32::try_from(size_of::<temper_engine_domain::Record>()).unwrap()).unwrap();
    let keys: &[(&str, u64)] = if path == b"engine.limits.overrides" {
        &[
            ("hub.turn_bytes", u64::from(turn)),
            ("hub.transcript_bytes", window),
            ("hub.turn_queue_bytes", u64::from(turn) * 8),
            ("core.resume_bytes", u64::from(resume)),
            ("authority.grants", 16),
            ("notes.scopes", 32),
        ]
    } else if path == b"workers[0].limits.overrides" {
        &[
            ("hub.turn_bytes", u64::from(turn)),
            ("hub.transcript_bytes", window),
            ("hub.turn_queue_bytes", u64::from(turn) * 8),
        ]
    } else if path == b"application.limits.overrides" {
        &[
            ("root.journal.transcript_bytes", u64::from(resume)),
            ("root.loads.transcript_bytes", u64::from(turn)),
            ("root.loads.bytes", u64::from(loaded)),
            ("root.loads.reply_bytes", u64::from(loaded)),
            ("root.loads.read_bytes", u64::from(turn) * 2),
        ]
    } else {
        return json.into();
    };
    let mut text = core::str::from_utf8(json).unwrap().trim().trim_end_matches('}').to_string();
    for (key, value) in keys {
        let mut quoted = String::from("\"");
        quoted.push_str(key);
        quoted.push('"');
        if !text.contains(&quoted) {
            if text.trim() != "{" {
                text.push(',');
            }
            text.push_str(&quoted);
            text.push(':');
            text.push_str(&value.to_string());
        }
    }
    text.push('}');
    text.into_bytes().into_boxed_slice()
}
fn engine_turn_limits() -> String {
    let document = core::str::from_utf8(super::ENGINE).unwrap();
    let overrides = document.split_once("\"overrides\":").unwrap().1;
    let mut result = overrides.split_once('}').unwrap().0.to_string();
    result.push('}');
    result
}
fn worker_turn_limits() -> String {
    r#"{"hub.slots":1,"hub.run_calls":1,"tasks.run_inbox":2,"hub.turns":1,"tasks.message_bytes":512,"people.words":512,"core.call_answer_bytes":2048}"#.into()
}
/// Copy a JSON value for a fixture derived from another document.
pub fn value(input: &[u8], path: &[u8]) -> Box<[u8]> {
    input[member_span(input, path)].into()
}
pub fn with(input: &[u8], path: &[u8], json: &[u8]) -> Box<[u8]> {
    let json = turn_limits(path, json);
    let span = member_span(input, path);
    let mut result = input[..span.start].to_vec();
    result.extend_from_slice(&json);
    result.extend_from_slice(&input[span.end..]);
    result.into_boxed_slice()
}

/// Several reads and disjoint edits of one fixture share its parsed document.
pub struct Json<'a> {
    input: &'a [u8],
    document: skein_json::Document,
}
impl<'a> Json<'a> {
    pub fn new(input: &'a [u8]) -> Self {
        let document = jig_protocol_config::document(
            input,
            &jig_protocol_config::DocumentLimits { bytes: 1_048_576, tokens: 16_384, depth: 32 },
        )
        .unwrap();
        Self { input, document }
    }
    pub fn value(&self, path: &[u8]) -> &[u8] {
        &self.input[member_span_in(self.input, &self.document, path)]
    }
    pub fn with(&self, edits: &[(&[u8], &[u8])]) -> Box<[u8]> {
        let mut edits: Vec<_> = edits
            .iter()
            .map(|(path, json)| (member_span_in(self.input, &self.document, path), turn_limits(path, json)))
            .collect();
        edits.sort_unstable_by_key(|(span, _)| span.start);
        let mut result = Vec::new();
        let mut after = 0;
        for (span, json) in edits {
            assert!(after <= span.start, "fixture edits are disjoint");
            result.extend_from_slice(&self.input[after..span.start]);
            result.extend_from_slice(&json);
            after = span.end;
        }
        result.extend_from_slice(&self.input[after..]);
        result.into_boxed_slice()
    }
}
fn member_span(input: &[u8], path: &[u8]) -> core::ops::Range<usize> {
    let json = Json::new(input);
    member_span_in(input, &json.document, path)
}
fn member_span_in(input: &[u8], document: &skein_json::Document, path: &[u8]) -> core::ops::Range<usize> {
    use jig_protocol_config::read;
    let mut at = 0;
    for member in core::str::from_utf8(path).unwrap().split('.') {
        let (member, index) = match member.split_once('[') {
            Some((member, index)) => (member, Some(index.trim_end_matches(']').parse::<usize>().unwrap())),
            None => (member, None),
        };
        at = read::Object::new(document, at).unwrap().member(document, member.as_bytes()).unwrap();
        if let Some(index) = index {
            at = *read::list(document, at, 16_384).unwrap().get(u32::try_from(index).unwrap()).unwrap();
        }
    }
    let after = read::after(document, at).unwrap();
    let mut token = 0;
    let mut start = None;
    let mut end = None;
    let mut offset = 0_usize;
    // Compact tokens hold decoded text offsets, so recover their raw spans.
    while offset < input.len() {
        let byte = input[offset];
        if byte.is_ascii_whitespace() || byte == b',' || byte == b':' {
            offset = offset.checked_add(1).unwrap();
            continue;
        }
        if token == at {
            start = Some(offset);
        }
        if token == after {
            end = Some(offset);
            break;
        }
        if byte == b'"' {
            offset = offset.checked_add(1).unwrap();
            while input[offset] != b'"' {
                if input[offset] == b'\\' {
                    offset = offset.checked_add(1).unwrap();
                }
                offset = offset.checked_add(1).unwrap();
            }
            offset = offset.checked_add(1).unwrap();
        } else if b"{}[]".contains(&byte) {
            offset = offset.checked_add(1).unwrap();
        } else {
            while offset < input.len() && !b",:}] \n\r\t".contains(&input[offset]) {
                offset = offset.checked_add(1).unwrap();
            }
        }
        token = token.checked_add(1).unwrap();
        if token == after {
            end = Some(offset);
            break;
        }
    }
    start.unwrap()..end.unwrap_or(input.len())
}

/// One webhook listener on the shipped profile.
pub fn webhook_document() -> Box<[u8]> {
    core::str::from_utf8(&document())
        .unwrap()
        .replace("\"webhooks\":null", "\"webhooks\":{\"listener\":{\"socket\":\"127.0.0.1:7200\"}}")
        .into_bytes()
        .into_boxed_slice()
}

/// The same deployment with its worker hosted by another process.
pub fn engine_alone_document() -> Box<[u8]> {
    let input = with(&document(), b"engine.hosts.listener", br#"[{"socket":"127.0.0.1:0"}]"#);
    let input = with(&input, b"workers", b"[]");
    with(&input, b"application.workers", b"[]")
}

/// A worker's document has no engine or engine application members.
pub fn worker_alone_document() -> Box<[u8]> {
    let input = with(&document(), b"workers[0].engine", br#"{"socket":"127.0.0.1:7100"}"#);
    let input = with(
        &input,
        b"application",
        br#"{"workers":[{"root":"worker-root","git":"/usr/bin/git"}],"limits":{"profile":"small","overrides":{}}}"#,
    );
    let text = core::str::from_utf8(&input).unwrap();
    let (_, workers) = text.split_once(", \"workers\":").unwrap();
    [b"{\"workers\":".as_slice(), workers.as_bytes()].concat().into_boxed_slice()
}

#[expect(clippy::too_many_arguments, reason = "the fixture supplies its own defaults at the startup boundary")]
pub fn start(
    document: &[u8],
    directories: &jig_protocol_config::Directories,
    files: &settings::Files,
    agent: &settings::Agent,
    seeds: &settings::Seeds,
    accounts: Option<crate::AccountsRuntime>,
    runtime: crate::WorkerRuntime,
    forges: List<crate::ForgeRuntime>,
) -> Result<crate::Service, crate::startup::Problems> {
    let defaults = crate::stock::with_models(Box::new([super::small_model()]), 32);
    crate::startup::startup_with(
        document,
        directories,
        files,
        agent,
        seeds,
        accounts,
        runtime,
        forges,
        &crate::stock::Given::Own { jig: defaults, forge: Box::default(), ways: crate::stock::ways() },
    )
}
