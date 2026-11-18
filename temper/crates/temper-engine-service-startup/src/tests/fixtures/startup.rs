#![expect(
    clippy::disallowed_types,
    clippy::disallowed_methods,
    reason = "ordinary fixture code builds and edits configuration bytes"
)]

pub const APPLICATION: &[u8] = br#"{
"webhooks":null,
"clients":{"listener":{"socket":"127.0.0.1:7100"},"public":null,"bundle":null},

"demo":null,"sign_in":null
}"#;

pub fn document() -> Box<[u8]> {
    let mut document = Vec::from(b"{\"engine\":".as_slice());
    let engine = core::str::from_utf8(super::ENGINE)
        .unwrap()
        .replace(",\n{\"name\": \"worker-b\", \"digest\": \"secrets/worker-b.digest\"}", "");
    document.extend_from_slice(engine.as_bytes());
    document.extend_from_slice(br#", "workers":[{
"engine":{"in_process":"workers"},"name":"worker-a","secret":"worker-secret","slots":1,
"agent":{"program":"/opt/temper/bin/temper","configuration":"/opt/temper/agent.json","arguments":["internal", "agent","/opt/temper/agent.json"]},
"limits":{"overrides":{"hub.slots":1,"hub.run_calls":1,"tasks.run_inbox":2,"hub.turns":1,"tasks.result_bytes":512,"tasks.message_bytes":512,"people.words":512,"core.call_answer_bytes":2048}}}],"application":"#);
    document.extend_from_slice(APPLICATION);
    document.extend_from_slice(b"}");
    with(&document, b"workers[0].limits.overrides", worker_turn_limits().as_bytes())
}

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

fn member_span(input: &[u8], path: &[u8]) -> core::ops::Range<usize> {
    let document = jig_protocol_config::document(
        input,
        &jig_protocol_config::DocumentLimits { bytes: 1_048_576, tokens: 16_384, depth: 32 },
    )
    .unwrap();
    member_span_in(input, &document, path)
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
    own.forge.landing_members = 2;
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
