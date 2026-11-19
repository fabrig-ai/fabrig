#![expect(
    clippy::disallowed_types,
    clippy::disallowed_methods,
    reason = "ordinary fixture code builds and edits configuration bytes"
)]
use super::{Input, Start};
use crate::settings;
use jig_domain_link::EndpointBound;
use skein_lib::{List, Secret};

pub const APPLICATION: &[u8] = br#"{
"webhooks":null,
"clients":{"listener":{"socket":"127.0.0.1:7100"},"public":null,"bundle":null},

"sign_in":null
}"#;
pub fn document() -> Start {
    let mut document = Vec::from(b"{\"engine\":".as_slice());
    let engine = core::str::from_utf8(super::ENGINE)
        .unwrap()
        .replace(",\n{\"name\": \"worker-b\", \"digest\": \"secrets/worker-b.digest\"}", "");
    document.extend_from_slice(engine.as_bytes());
    document.extend_from_slice(
        br#", "workers":[{
"engine":{"in_process":"workers"},"name":"local","secret":"worker-secret","slots":1}],"application":"#,
    );
    document.extend_from_slice(APPLICATION);
    document.extend_from_slice(b"}");
    let (mut jig, mut own) = startup_limits();
    jig.hub.accounts = 8;
    jig.hub.inbox = 8;
    jig.hub.held = 8;
    jig.hub.event_bytes = 512;
    own.checkout.name_bytes = 600;
    own.checkout.repositories = 4;
    own.git_component.stdout_bytes = 131_072;
    own.forge.name_bytes = 256;
    own.forge.landing_members = 2;
    own.forge.resources_per_task = 4;
    Start::given(&document, &jig, &own)
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
    for (path, value) in
        [(b"worker-secret".as_slice(), super::worker::WORKER_SECRET), (super::AGENT_JSON, b"{}".as_slice())]
    {
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
/// The small worlds' provider and catalogue facts are typed inputs, not deployment members.
pub fn given(input: &(impl Input + ?Sized), _directories: &jig_protocol_config::Directories) -> crate::stock::Given {
    use jig_protocol_config as config;
    let input = input.configured();
    let words = input.jig.core.tasks.result_bytes;
    let llm = config::Llm { models: Box::new([]), providers: Box::new([]), endpoints: Box::new([]) };
    let models = crate::stock::models(&llm).into_vec().into_iter().filter(|row| row.model.provider == 1).collect();
    let mut jig = crate::stock::with_models(models, words);
    jig.providers = temper_worker_service_fixtures::providers();
    let root = crate::limits::engine(&config::stock(), &crate::limits::stock()).expect("stock fixture limits");
    crate::stock::Given::Own { jig, forge: crate::stock::forge(&root), ways: crate::stock::ways() }
}
pub fn checked_start(
    input: &(impl Input + ?Sized),
    directories: &jig_protocol_config::Directories,
    files: &settings::Files,
    agent: Option<&settings::Agent>,
    seeds: &settings::Seeds,
    forges: List<crate::ForgeRuntime>,
    setup: Option<&temper_forge_setup::check::Checked>,
) -> Result<crate::startup::CheckedStart, crate::startup::Problems> {
    let input = input.configured();
    crate::startup::checked_start_with_limits(
        &input,
        directories,
        files,
        agent,
        seeds,
        forges,
        setup,
        &given(&input, directories),
        None,
        &input.jig,
        &input.own,
    )
}
#[expect(clippy::too_many_arguments, reason = "the fixture retains the public check's runtime facts")]
pub fn check(
    input: &(impl Input + ?Sized),
    directories: &jig_protocol_config::Directories,
    files: &settings::Files,
    agent: &settings::Agent,
    seeds: &settings::Seeds,
    accounts: Option<&crate::AccountsRuntime>,
    forges: List<crate::ForgeRuntime>,
    setup: Option<&temper_forge_setup::check::Checked>,
) -> Result<(), crate::startup::Problems> {
    let input = input.configured();
    if crate::startup::nearest(&input) == Some(crate::startup::Shape::WorkerAlone) {
        return crate::startup::check(&input, directories, files, agent, seeds, accounts, forges, setup);
    }
    checked_start(&input, directories, files, Some(agent), seeds, forges, setup)?;
    Ok(())
}
#[expect(clippy::too_many_arguments, reason = "the fixture supplies typed providers to the existing startup entry")]
pub fn startup(
    input: &(impl Input + ?Sized),
    directories: &jig_protocol_config::Directories,
    files: &settings::Files,
    agent: &settings::Agent,
    seeds: &settings::Seeds,
    accounts: Option<crate::AccountsRuntime>,
    runtime: crate::WorkerRuntime,
    forges: List<crate::ForgeRuntime>,
) -> Result<crate::Service, crate::startup::Problems> {
    let checked = checked_start(input, directories, files, Some(agent), seeds, forges, None)?;
    crate::startup::from_checked(checked, files, Some(agent), seeds, accounts, Some(runtime), None, None, Box::new([]))
}
#[expect(clippy::too_many_arguments, reason = "the fixture retains setup's existing runtime facts")]
pub fn with_setup(
    input: &(impl Input + ?Sized),
    directories: &jig_protocol_config::Directories,
    files: &settings::Files,
    agent: Option<&settings::Agent>,
    seeds: &settings::Seeds,
    accounts: Option<crate::AccountsRuntime>,
    runtime: Option<crate::WorkerRuntime>,
    forges: List<crate::ForgeRuntime>,
    setup: Option<&temper_forge_setup::check::Checked>,
    box_line_bytes: Option<u32>,
) -> Result<crate::Service, crate::startup::Problems> {
    let checked = checked_start(input, directories, files, agent, seeds, forges, setup)?;
    crate::startup::from_checked(checked, files, agent, seeds, accounts, runtime, box_line_bytes, None, Box::new([]))
}
pub fn named(
    input: &(impl Input + ?Sized),
    directories: &jig_protocol_config::Directories,
) -> Result<crate::startup::Named, crate::startup::Problems> {
    let input = input.configured();
    input.named_with(directories, &given(&input, directories))
}
pub fn startup_agent() -> settings::Agent {
    // Match init's maximum run spend, including Fable's cache-write reserve.
    let init = super::super::stock::initial_engine();
    startup_agent_with_spend(init.rules.maximum_run_spend)
}

pub fn startup_agent_with_spend(spend: u64) -> settings::Agent {
    let mut agent = super::agent();
    let hub = jig_protocol_config_fixtures::small().hub;
    agent.host_limits.directories = 4;
    agent.host_limits.conflicts = 1024;
    agent.host_limits.name_bytes = 600;
    agent.host_limits.accounts = hub.accounts;
    agent.host_limits.messages = hub.held.max(hub.inbox);
    agent.host_limits.call_bytes = 256;
    agent.host_limits.message_bytes = hub.event_bytes.checked_add(temper_worker_domain::SEPARATOR).unwrap();
    agent.bounds.message_bytes = 512;
    agent.bounds.spend = spend;
    agent.bounds.endpoints = Box::new([EndpointBound { endpoint: 1, window: 1_048_576, output: 8192 }]);
    agent
}

// Whole-map edits in unrelated startup stories inherit this fixture's turn
// allowances. An explicitly supplied key still wins, including a short bound.
fn turn_limits(path: &[u8], json: &[u8]) -> Box<[u8]> {
    let turn = super::agent().limits.bodies.turn_body;
    let resume = turn.checked_add(4).unwrap();
    let window = u64::from(resume) * 4;
    let keys: &[(&str, u64)] = if path == b"engine.limits.overrides" {
        &[
            ("hub.accounts", 8),
            ("hub.inbox", 8),
            ("hub.held", 8),
            ("hub.event_bytes", 512),
            ("hub.turn_bytes", u64::from(turn)),
            ("hub.transcript_bytes", window),
            ("hub.turn_queue_bytes", u64::from(turn) * 8),
            ("core.resume_bytes", u64::from(resume)),
            ("tasks.inbox_bytes", 4096),
            ("tasks.run_inbox", 8),
            ("authority.grants", 16),
            ("notes.scopes", 32),
        ]
    } else if path == b"workers[0].limits.overrides" {
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
/// Copy a JSON value for a fixture derived from another document.
pub fn value(input: &(impl Input + ?Sized), path: &[u8]) -> Box<[u8]> {
    let input = input.configured();
    if let Some(value) = input.limits(path) {
        return value;
    }
    input[member_span(&input, path)].into()
}
pub fn with(input: &(impl Input + ?Sized), path: &[u8], json: &[u8]) -> Start {
    let mut input = input.configured();
    if input.over(path, json) {
        return input;
    }
    let json = turn_limits(path, json);
    let span = member_span(&input, path);
    let mut result = input[..span.start].to_vec();
    result.extend_from_slice(&json);
    result.extend_from_slice(&input[span.end..]);
    input.document = result.into_boxed_slice();
    input
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

/// One webhook listener on the shipped profile.
pub fn webhook_document() -> Start {
    with(&document(), b"application.webhooks", br#"{"listener":{"socket":"127.0.0.1:7200"}}"#)
}

/// The same deployment with its worker hosted by another process.
pub fn engine_alone_document() -> Start {
    let input = with(&document(), b"engine.hosts.listener", br#"[{"socket":"127.0.0.1:0"}]"#);
    with(&input, b"workers", b"[]")
}

/// A worker's document has no engine or engine application members.
pub fn worker_alone_document() -> Start {
    let mut input: serde_json::Value = serde_json::from_slice(&document()).unwrap();
    input.as_object_mut().unwrap().remove("engine");
    input["workers"][0]["engine"] = serde_json::from_str(r#"{"socket":"127.0.0.1:7100"}"#).unwrap();
    input["workers"][0].as_object_mut().unwrap().remove("agent");
    input["workers"][0].as_object_mut().unwrap().remove("limits");
    input["application"] = serde_json::json!({});
    Start::stock(&serde_json::to_vec(&input).unwrap())
}

pub fn start_defaults() -> crate::stock::Given {
    let mut defaults = crate::stock::with_models(Box::new([super::small_model()]), 32);
    defaults.providers = temper_worker_service_fixtures::providers();
    crate::stock::Given::Own { jig: defaults, forge: Box::default(), ways: crate::stock::ways() }
}

#[expect(clippy::too_many_arguments, reason = "the fixture supplies its own defaults at the startup boundary")]
pub fn start(
    document: &(impl Input + ?Sized),
    directories: &jig_protocol_config::Directories,
    files: &settings::Files,
    agent: &settings::Agent,
    seeds: &settings::Seeds,
    accounts: Option<crate::AccountsRuntime>,
    runtime: crate::WorkerRuntime,
    forges: List<crate::ForgeRuntime>,
) -> Result<crate::Service, crate::startup::Problems> {
    start_with(document, directories, files, agent, seeds, accounts, runtime, forges, &start_defaults())
}

#[expect(clippy::too_many_arguments, reason = "the fixture reuses defaults beside its typed startup limits")]
pub fn start_with(
    document: &(impl Input + ?Sized),
    directories: &jig_protocol_config::Directories,
    files: &settings::Files,
    agent: &settings::Agent,
    seeds: &settings::Seeds,
    accounts: Option<crate::AccountsRuntime>,
    runtime: crate::WorkerRuntime,
    forges: List<crate::ForgeRuntime>,
    given: &crate::stock::Given,
) -> Result<crate::Service, crate::startup::Problems> {
    let document = document.configured();
    let checked = crate::startup::checked_start_with_limits(
        &document,
        directories,
        files,
        Some(agent),
        seeds,
        forges,
        None,
        given,
        None,
        &document.jig,
        &document.own,
    )?;
    crate::startup::from_checked(checked, files, Some(agent), seeds, accounts, Some(runtime), None, None, Box::new([]))
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
