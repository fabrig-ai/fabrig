#![expect(
    clippy::disallowed_types,
    clippy::disallowed_methods,
    reason = "ordinary fixture code builds and edits configuration bytes"
)]
use crate::settings;
use skein_lib::{List, Secret};

pub const APPLICATION: &[u8] = br#"{
"webhooks":null,
"clients":{"listener":{"socket":"127.0.0.1:7100"},"public":null,"bundle":null},

"sign_in":null
}"#;
pub fn demo_service() -> crate::Service {
    let document = document();
    let document = with(&document, b"workers", b"[]");
    let document = with(&document, b"engine.hosts.listener", br#"[{"socket":"127.0.0.1:7200"}]"#);
    let mut start = crate::startup::checked_start_with_limits(
        &document,
        &jig_protocol_config::Directories::working(),
        &files(),
        None,
        &super::seeds(),
        super::forge_runtime(),
        None,
        &crate::stock::Given::Own {
            jig: {
                let mut defaults = crate::stock::with_models(Box::new([super::small_model()]), 32);
                defaults.providers = temper_worker_service_fixtures::providers();
                defaults
            },
            forge: Box::default(),
            ways: crate::stock::ways(),
        },
        &startup_limits().0,
        &startup_limits().1,
    )
    .unwrap();
    start.engine.demo = Some(demo());
    let seeded = super::engine().jig;
    start.engine.jig.projects = seeded.projects;
    start.engine.jig.owners = seeded.owners;
    start.engine.jig.acts = Box::new([jig_domain_people::ActGrant {
        connector: 1,
        may: Box::new([
            jig_domain_people::ActKind::Close,
            jig_domain_people::ActKind::AddKey,
            jig_domain_people::ActKind::RemoveAccount,
            jig_domain_people::ActKind::Adopt,
        ]),
    }]);
    crate::Service::prepared(start.engine, &super::seeds(), None, start.forges, start.io, None, None).unwrap()
}
pub fn demo() -> settings::Demo {
    settings::Demo {
        provider: 3,
        key_file: b"demo/llm.key".as_slice().into(),
        owner: b"playground".as_slice().into(),
        name: b"widget".as_slice().into(),
        landing: b"main".as_slice().into(),
        cues: Box::new([
            b"Add CSV export to widget".as_slice().into(),
            b"Add a dark theme to widget".as_slice().into(),
            b"Fix widget's empty search state".as_slice().into(),
        ]),
    }
}
pub fn document() -> Box<[u8]> {
    let mut document = Vec::from(b"{\"engine\":".as_slice());
    let engine = core::str::from_utf8(super::ENGINE)
        .unwrap()
        .replace(",\n{\"name\": \"worker-b\", \"digest\": \"secrets/worker-b.digest\"}", "");
    document.extend_from_slice(engine.as_bytes());
    document.extend_from_slice(br#", "workers":[{
"engine":{"in_process":"workers"},"name":"local","secret":"worker-secret","slots":1,
"agent":{"program":"/opt/temper/bin/temper","configuration":"/opt/temper/agent.json","arguments":["internal", "agent","/opt/temper/agent.json"]},
"limits":{"overrides":{"hub.slots":1,"hub.run_calls":1,"tasks.run_inbox":2,"hub.turns":1,"tasks.result_bytes":512,"tasks.message_bytes":512,"people.words":512,"core.call_answer_bytes":2048}}}],"application":"#);
    document.extend_from_slice(APPLICATION);
    document.extend_from_slice(b"}");
    with(&document, b"workers[0].limits.overrides", worker_turn_limits().as_bytes())
}
pub fn files() -> settings::Files {
    let mut entries = List::with_capacity(8);
    for (path, value) in [
        (b"worker-secret".as_slice(), temper_worker_service_fixtures::WORKER_SECRET),
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

// Whole-map edits in unrelated startup stories inherit this fixture's turn
// allowances. An explicitly supplied key still wins, including a short bound.
fn turn_limits(path: &[u8], json: &[u8]) -> Box<[u8]> {
    let turn = super::agent().limits.bodies.turn_body;
    let resume = turn.checked_add(4).unwrap();
    let window = u64::from(resume) * 4;
    let keys: &[(&str, u64)] = if path == b"workers[0].limits.overrides" {
        &[
            ("hub.accounts", 8),
            ("hub.inbox", 8),
            ("hub.held", 8),
            ("tasks.result_bytes", 512),
            ("hub.turn_bytes", u64::from(turn)),
            ("hub.transcript_bytes", window),
            ("hub.turn_queue_bytes", u64::from(turn) * 8),
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
fn worker_turn_limits() -> String {
    r#"{"hub.slots":1,"hub.run_calls":1,"tasks.run_inbox":2,"hub.turns":1,"tasks.result_bytes":512,"tasks.message_bytes":512,"people.words":512,"core.call_answer_bytes":2048}"#.into()
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

/// The engine member of a generated standalone fixture, with no local worker.
pub fn engine_part(input: &[u8]) -> Box<[u8]> {
    let input = with(input, b"workers", b"[]");
    with(&input, b"engine.hosts.listener", br#"[{"socket":"127.0.0.1:7200"}]"#)
}

pub fn startup_limits() -> (jig_protocol_config::Limits, crate::limits::Limits) {
    let sources = temper_service_core::derived::Sources::agent(&super::agent().declaration());
    let stock = crate::limits::stock();
    let (base, mut own) = crate::limits::base(stock.live_goals, stock.tool_payload, sources).unwrap();
    let mut jig = super::jig_limits_over(&base);
    let turn = sources.turn;
    own.root.transcript_memory = 1_152_921_504_606_846_976;
    own.root.journal.run_bytes = 32_768;
    own.checkout.workspaces = 1;
    own.git_component.operations = 1;
    let resume = turn.checked_add(4).unwrap();
    let loaded = turn.checked_add(u32::try_from(size_of::<temper_engine_domain::Record>()).unwrap()).unwrap();
    jig.hub.turn_bytes = u64::from(turn);
    jig.hub.transcript_bytes = u64::from(resume) * 4;
    jig.hub.turn_queue_bytes = u64::from(turn) * 8;
    jig.core.resume_bytes = resume;
    own.checkout.name_bytes = 600;
    own.checkout.repositories = 4;
    own.git_component.stdout_bytes = 131_072;
    own.forge.name_bytes = 256;
    own.forge.resources_per_task = 4;
    crate::limits::apply(&mut own, b"forge.client.answer_bytes", 65_536).unwrap();
    own.forge.client.op_bytes = 512;
    own.forge_component.body_bytes = 16_384;
    own.root.journal.transcript_bytes = resume;
    own.root.loads.transcript_bytes = turn;
    own.root.loads.bytes = loaded;
    own.root.loads.reply_bytes = loaded;
    own.root.loads.read_bytes = u64::from(turn) * 2;
    (jig, own)
}
