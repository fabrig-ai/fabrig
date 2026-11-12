//! Deployment values (temper/docs/design/playground.md, 3).
mod channel;
pub mod hosted_git;
pub mod settings;
mod smith;
pub mod turn_bounds;
use jig_domain_link::{Bounds, EndpointBound};
use skein_lib::{Duration, List, Secret};
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};
/// Forge for the composition.
pub const FORGE: SocketAddr = SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::LOCALHOST, 3000));
/// Hook secret for the composition.
pub const HOOK_SECRET: &[u8] = b"world-webhook-secret";
/// Forge connector for the composition.
pub const FORGE_CONNECTOR: u16 = 1;
/// Grant kinds for the composition.
pub const GRANT_KINDS: [u16; 2] = [3, 11];
/// Procedure code for the composition.
pub const PROCEDURE_CODE: u16 = 2;
/// Charter number for the composition.
pub const CHARTER_NUMBER: u16 = 1;
/// Repository for the composition.
pub const REPOSITORY: u32 = 2;
/// Producer tools for the composition.
pub const PRODUCER_TOOLS: u64 = 3;
/// Document lines for the composition.
pub const DOCUMENT_LINES: u32 = 9;
/// Chat wait ms for the composition.
pub const CHAT_WAIT_MS: u32 = 1000;
/// Worker user for the composition.
pub const WORKER_USER: u64 = 1;
/// Authority depth for the composition.
pub const AUTHORITY_DEPTH: u32 = 3;
/// The repository agent's spending allowance.
pub const PRODUCER_SPEND: u64 = 40;
/// The repository grants' implied kinds.
pub const IMPLICATIONS: [u16; 2] = [1, 2];
/// The repository agent's turn allowance.
pub const CHARTER_TURNS: u32 = 8;
/// Engine overrides for the composition.
pub const ENGINE_OVERRIDES: [(&str, u32); 22] = [
    ("tasks.tasks", 4),
    ("tasks.project_tasks", 4),
    ("tasks.tree_tasks", 6),
    ("people.people", 4),
    ("people.inbox_tasks", 8),
    ("fleet.attempts", 4),
    ("fleet.turns", 16),
    ("brief.sections", 10),
    ("brief.brief_bytes", 4096),
    ("tasks.result_bytes", 128),
    ("tasks.resource_reports", 4),
    ("tasks.pools", 2),
    ("tasks.parameters", 5),
    ("tasks.spec_bytes", 187),
    ("tasks.contract_choices", 2),
    ("tasks.executor_kinds", 2),
    ("tasks.authority_grants", 4),
    ("tasks.authority_segments", 4),
    ("tasks.authority_bytes", 128),
    ("core.policy_bytes", 8192),
    ("authority.segments", 8),
    ("core.call_answer_bytes", 2048),
];
/// Agents for the composition.
pub const AGENTS: u32 = 2;
/// Clients for the composition.
pub const CLIENTS: std::net::SocketAddr =
    std::net::SocketAddr::new(std::net::IpAddr::V4(std::net::Ipv4Addr::LOCALHOST), 7102);
/// The page's fetch bound: temper-client-shell-web's `limits::EXCHANGES + 1`.
pub const VISITOR_EXCHANGES: u32 = 9;

/// The deployment's clients listener and Host, with the playground's bounds.
#[must_use]
pub fn visitor(document: &serde_json::Value) -> skein_playground::visitor::Config {
    let clients = &document["application"]["clients"];
    let socket = clients["listener"]["socket"].as_str().expect("clients socket");
    let host = clients["public"].as_str().map_or(socket.as_bytes().into(), |public| {
        public
            .strip_prefix("https://")
            .or_else(|| public.strip_prefix("http://"))
            .expect("public origin")
            .as_bytes()
            .into()
    });
    skein_playground::visitor::Config {
        listener: socket.parse().expect("clients address"),
        host,
        exchanges: VISITOR_EXCHANGES,
        // jig's only sign-in cookie: jig-sign-in (11 bytes) and a 40-byte credential
        // in unpadded URL base64 (54 bytes); jig/docs/design/protocol/browser/README.md, 11.
        cookies: 1,
        cookie_bytes: 65,
        // Nine fetches and one stream, each with its own socket. HTTP's 4096-byte
        // head and temper-client-shell-web's limits::PIECE fit the intake.
        io: skein_io::Limits {
            sockets: VISITOR_EXCHANGES + 1,
            intake: 4096,
            receive: 4096,
            output: 16_384,
            sends: 8,
            backlog: 2,
            retry: Duration::from_millis(1),
            ..skein_io::Limits::TEST
        },
        // Heads match the clients listener; bodies stream within its 16384-byte
        // send/read bounds, with responses read in the shell's 4096-byte pieces.
        http: skein_http::client::Limits { request: 4096, head: 4096, headers: 32, read: 4096, send: 16_384 },
    }
}

/// The scripted provider's synthetic key.
pub const KEY: &[u8] = b"test-key";
/// The base engine deployment document.
#[must_use]
pub fn engine_document() -> Vec<u8> {
    channel::engine_document()
}
/// The base worker deployment document.
pub const WORKER_DOCUMENT: &[u8] = channel::WORKER_DOCUMENT;
/// The deployment bytes read by temper serve.
#[must_use]
pub fn document() -> Vec<u8> {
    let mut value = world_document();
    value.as_object_mut().expect("deployment").remove("world_settings");
    settings::strip_models(&mut value);
    serde_json::to_vec(&value).expect("deployment JSON")
}
/// The deployment value before harness edits: the chat stories' policy.
#[must_use]
pub fn world_document() -> serde_json::Value {
    world_document_with(chat_policy)
}
/// The deployment value with `edit` applied after the staged policies and before the
/// composition's shared limits, where a harness's own policy goes.
#[must_use]
pub fn world_document_with(edit: impl FnOnce(&mut serde_json::Value)) -> serde_json::Value {
    let mut value: serde_json::Value = serde_json::from_slice(&channel::engine_document()).expect("engine JSON");
    value.as_object_mut().expect("object").remove("service");
    let worker: serde_json::Value = serde_json::from_slice(channel::WORKER_DOCUMENT).expect("worker JSON");
    value["workers"] = worker["workers"].clone();
    value["workers"][0]["engine"] = serde_json::json!({"in_process":"workers"});
    value["workers"][0]["agent"] = serde_json::json!({"program":"/opt/temper/bin/temper","configuration":"/opt/temper/agent.json","arguments":["internal", "agent","/opt/temper/agent.json"]});
    value["workers"][0]["limits"]["overrides"]["hub.accounts"] = 4.into();
    value["workers"][0]["limits"]["overrides"]["hub.event_bytes"] = 1024.into();
    value["engine"]["hosts"]["listener"] = serde_json::json!([{"in_process":"workers"}]);
    value["engine"]["connectors"] = serde_json::json!([1]);
    for (name, count) in [
        ("core.load_slots", 8),
        ("core.run_bytes", 16_384),
        ("core.templates", 6),
        ("documents.rows", 2),
        ("documents.lines", DOCUMENT_LINES),
        ("documents.text_bytes", 128),
        ("people.sign_ins", 2),
    ] {
        value["engine"]["limits"]["overrides"][name] = count.into();
    }
    value["application"] = serde_json::json!({
        "webhooks":{"listener":{"socket":"127.0.0.1:0"}},
"clients":{"listener":{"socket":CLIENTS.to_string()},"public":null,"bundle":null},
        "machine":{},"demo":null,"sign_in":null,
        "limits":{"profile":"small","overrides":{"checkout.name_bytes":256,"checkout.repositories":2,"checkout.message_bytes":4096,"forge.name_bytes":64,"forge.resources_per_task":2,"git_component.stdout_bytes":131_072}},
        "workers":[{"root":"worker","git":"/usr/bin/git"}]});
    settings::stage_policies(&mut value);
    for member in ["charters", "templates", "policies", "rules"] {
        value["engine"].as_object_mut().expect("engine").remove(member);
    }
    edit(&mut value);
    for (name, count) in [
        // Two added 8-byte executor places make the largest stock chat payload 1,117 bytes.
        ("people.amendment_bytes", 1125),
        ("tasks.hold_bytes", 256),
        ("tasks.funders", 4),
        ("tasks.dependencies", 2),
        // Ten slots hold the stock charters.
        ("tasks.charters", 10),
        ("notes.entries_per_scope", 2),
        ("notes.party_entries", 2),
        ("notes.recalled", 2),
        ("people.holdings", 2),
        ("people.pending", 5),
        ("people.waiters", 2),
    ] {
        value["engine"]["limits"]["overrides"][name] = count.into();
    }
    value["engine"]["limits"]["overrides"]["people.projects"] = 2.into();
    value["engine"]["limits"]["overrides"]["authority.projects"] = 2.into();
    value["engine"]["limits"]["overrides"]["tasks.funders"] = 4.into();
    turn_bounds::apply(&mut value, agent().limits.bodies.turn_body);
    value
}
/// The chat stories' policy: seed projects on the world's policy 2, delegation depth 4, and the
/// chat limits.
pub fn chat_policy(value: &mut serde_json::Value) {
    for seed in value["engine"]["projects"].as_array_mut().expect("seed projects") {
        seed["policy"] = 2.into();
    }
    for project in value["world_settings"]["policies"].as_array_mut().expect("world policies") {
        project["policy"]["ceiling"]["delegation"]["depth"] = 4.into();
        for role in project["policy"]["roles"].as_array_mut().expect("roles") {
            role["authority"]["delegation"]["depth"] = 4.into();
        }
    }
    for (name, count) in [
        ("core.policy_bytes", 8192),
        ("authority.grants", 16),
        // The standing charter, closer procedure and close job are delegation targets.
        ("authority.executors", 18),
        ("authority.segments", 8),
        ("tasks.authority_grants", 15),
        ("tasks.authority_bytes", 70),
        // The standing charter, closer procedure and close job are delegation targets.
        ("tasks.executor_kinds", 18),
        ("tasks.result_bytes", 128),
        ("notes.scopes", 32),
    ] {
        value["engine"]["limits"]["overrides"][name] = count.into();
    }
}
/// The composition's startup file contents.
#[must_use]
pub fn files() -> temper_service::settings::Files {
    let mut entries = List::with_capacity(10);
    for source in [channel::files(false), channel::files(true)] {
        for (name, value) in source.entries.into_boxed() {
            if name.starts_with(b"charters/") {
                continue;
            }
            let name = if name.as_ref() == b"agent.json" { b"/opt/temper/agent.json".as_slice().into() } else { name };
            entries.push((name, value)).expect("fixture files");
        }
    }
    for name in [b"secrets/forge.api-token".as_slice(), b"secrets/forge.git-token"] {
        entries.push((name.into(), Secret::new(hosted_git::TOKEN.into()))).expect("forge files");
    }
    entries
        .push((b"secrets/forge-1.user".as_slice().into(), Secret::new(b"1\n".as_slice().into())))
        .expect("user file");
    entries
        .push((b"secrets/forge-1.webhook-secret".as_slice().into(), Secret::new(HOOK_SECRET.into())))
        .expect("webhook secret file");
    temper_service::settings::Files { entries }
}
/// The hosted agent's declared profile.
#[must_use]
pub fn agent_profile() -> smith_agent_service::profile::Profile {
    let mut profile = smith::profile();
    profile.policy.host_tools =
        u32::try_from(temper_engine_smith::tools(Duration::from_secs(10)).len()).expect("tool count");
    profile.declared.tool_payload = 2048;
    profile.declared.read_window = 256;
    profile.declared.conversations = 1;
    profile.declared.calls_per_response = 1;
    profile.policy.max_turns = 800;
    profile.policy.max_time = Duration::from_secs(28_800);
    profile.policy.check_timeout = Duration::from_secs(5);
    let hub = jig_protocol_config::profile(jig_protocol_config::Profile::Small).hub;
    profile.policy.inbox = hub.held.max(hub.inbox);
    profile.policy.sections = 8;
    profile
}
/// The hosted agent's derived service limits.
#[must_use]
pub fn agent_limits() -> smith_agent_service::Limits {
    let mut limits =
        smith_agent_service::profile::derive(&agent_profile(), &agent_declarations()).expect("chat agent limits");
    limits.llm.accounts = 2;
    limits
}
/// The hosted agent's endpoint and model declarations.
#[must_use]
pub fn agent_declarations() -> smith_agent_service::profile::Configuration {
    let mut declarations = smith::declarations(0);
    declarations.endpoints[0].limits.output = 2048;
    declarations
}
/// The hosted agent's complete configuration.
#[must_use]
pub fn agent_configuration() -> smith_agent_service::Config {
    let mut config = smith::configuration_with_limits(skein_llm::Provider::OpenAi, &smith::derive_limits());
    config.limits = agent_limits();
    config.channel_endpoints = channel::endpoints();
    let (destinations, _) = config.llm_endpoints.into_parts();
    let mut destination = destinations.into_boxed().into_vec().pop().expect("one LLM endpoint");
    destination.limits = config.limits.llm.translation.client;
    config.llm_endpoints = smith_protocol_llm::Endpoints::new(
        Box::new([smith_protocol_llm::ConfiguredEndpoint {
            name: smith_domain::llm::Endpoint(0),
            destination,
            connect: Duration::from_secs(1),
            handshake: Duration::from_secs(1),
            oversized_reasoning: smith_protocol_llm::OversizedReasoning::Fail,
        }]),
        1,
    )
    .expect("derived provider endpoint");
    config
}
/// The forge runtime destinations.
#[must_use]
pub fn forges() -> List<temper_service::ForgeRuntime> {
    let mut forges = List::with_capacity(1);
    forges
        .push(temper_service::ForgeRuntime {
            log_origins: skein_lib::List::with_capacity(0),
            number: 1,
            address: FORGE,
            transport: skein_forge_connection::settings::Transport::Plaintext { name: None },
        })
        .expect("one loopback forge");
    forges
}
/// The service's hosted agent settings.
#[must_use]
pub fn agent() -> temper_service::settings::Agent {
    let limits = agent_limits();
    let host_limits = smith_local_service::profile::host_limits(&agent_profile(), &limits).expect("agent host limits");
    let channel = limits.channel;
    let mut protocol = smith_host_protocol::Limits {
        bodies: channel.bodies,
        charter: channel.charter,
        transcript: channel.transcript,
        endpoints: channel.endpoints,
        channel: channel.channel,
        start_bytes: channel.start_bytes,
        calls: host_limits.calls,
    };
    protocol.channel.output_bytes = smith_host_protocol::output_bytes(&protocol).expect("host frames");
    let bounds = Bounds {
        program_endpoints: Box::new([]),
        endpoints: Box::new([EndpointBound { endpoint: 1, window: 1_048_576, output: 512 }]),
        turns: agent_profile().policy.max_turns,
        // Story A's stock run allowance at its model's template prices.
        spend: 4200,
        time: Duration::from_secs(28_800),
        waiting: Duration::from_secs(60),
        checks: Duration::from_secs(5),
        message_bytes: 1024,
        inbox: 8,
        unacknowledged: 8,
        reply_bytes: None,
    };
    temper_service::settings::Agent {
        run_limits: limits.domain.run,
        tool_payload: agent_profile().declared.tool_payload,
        programs: Box::new([]),
        endpoints: channel::endpoints(),
        limits: protocol,
        bounds,
        host_limits,
    }
}
/// Apply the repository composition's policies and worker settings.
pub fn configure(value: &mut serde_json::Value) {
    value["workers"][0]["slots"] = AGENTS.into();
    value["workers"][0]["limits"]["overrides"]["hub.slots"] = AGENTS.into();
    let grants: Vec<_> = GRANT_KINDS
        .into_iter()
        .map(|kind| {
            serde_json::json!({
                "connector":FORGE_CONNECTOR,"kind":kind,"pattern":{"segments":["forge","127.0.0.1:3000"],"last":"open","value":"acme"}
            })
        })
        .collect();
    let kinds =
        serde_json::json!([{"kind":"charter","number":CHARTER_NUMBER},{"kind":"procedure","number":PROCEDURE_CODE}]);
    let mut ceiling = value["world_settings"]["policies"][0]["policy"]["ceiling"].clone();
    ceiling["tools"] = PRODUCER_TOOLS.into();
    ceiling["grants"] = grants.into();
    ceiling["delegation"]["kinds"] = kinds;
    ceiling["delegation"]["depth"] = AUTHORITY_DEPTH.into();
    value["world_settings"]["policies"][0]["policy"]["ceiling"] = ceiling.clone();
    value["world_settings"]["policies"][0]["policy"]["roles"][0]["authority"] = ceiling;
    for (name, count) in ENGINE_OVERRIDES {
        value["engine"]["limits"]["overrides"][name] = count.into();
    }
    let overrides = value["engine"]["limits"]["overrides"].as_object_mut().expect("engine overrides");
    for name in [
        "tasks.funders",
        "tasks.dependencies",
        "tasks.charters",
        "people.amendment_bytes",
        "people.projects",
        "tasks.hold_bytes",
        "notes.entries_per_scope",
        "notes.party_entries",
        "notes.recalled",
        "views.runs",
        "people.holdings",
        "people.waiters",
        "people.goals",
        "people.pending",
    ] {
        overrides.remove(name);
    }
}
/// The stock charter settings file with the composition's wait.
#[must_use]
pub fn settings_file() -> Vec<u8> {
    let document: serde_json::Value = serde_json::from_slice(&document()).expect("deployment JSON");
    let stock = settings::stock(&document);
    let mut charter = stock["charter"].clone();
    charter["waiting"] = (u64::from(CHAT_WAIT_MS) * 1_000_000 / 2).into();
    serde_json::to_vec(&serde_json::json!({"items":[settings::item("charter", 1, charter)]})).expect("settings FILE")
}
/// Typed defaults from the stock settings and composition policy.
#[must_use]
pub fn given(document: &serde_json::Value) -> temper_service::stock::Given {
    given_with(document, true, false)
}
/// Typed defaults with the composition's stock or repository policy and waiting mode.
#[must_use]
pub fn given_with(document: &serde_json::Value, stock: bool, read_wait: bool) -> temper_service::stock::Given {
    let mut document = document.clone();
    if document["world_settings"].is_null() {
        document["world_settings"] = world_document()["world_settings"].clone();
    }
    let engine = settings::configuration(&document);
    let mut defaults =
        temper_service::stock::with_models(engine.llm.models.clone(), engine.limits.core.tasks.result_bytes);
    if read_wait {
        defaults.charters[0].policy.waiting = skein_lib::Duration::from_millis(u64::from(CHAT_WAIT_MS));
    } else if document["application"]["demo"].is_null() {
        defaults.templates[0].flags.ended_by = jig_domain_tasks::EndedBy::Result;
    }
    let seed = engine.projects.first().expect("world policy");
    let number = if stock {
        let mut policies = skein_lib::Map::with_capacity(2);
        policies.insert(1, defaults.policies.get(&1).expect("stock policy 1").clone()).expect("two world policies");
        defaults.policies = policies;
        2
    } else {
        1
    };
    defaults
        .policies
        .insert(
            number,
            jig_domain_core::ConfiguredPolicy {
                policy: seed.policy.clone(),
                permissions: seed.roles.clone(),
                templates: seed.templates.clone(),
            },
        )
        .expect("existing policy slot");
    if !stock {
        let seed = engine.projects.first().expect("repository seed");
        defaults.rules.ceiling = seed.policy.ceiling.clone();
        defaults.rules.maximum_run_spend = PRODUCER_SPEND;
        let pairs = IMPLICATIONS.map(|implies| jig_domain_authority::Implication {
            connector: FORGE_CONNECTOR,
            kind: GRANT_KINDS[0],
            implies,
        });
        defaults.rules.implies = jig_domain_authority::Implies::new(Box::new(pairs), 2).expect("fixture implications");
        let charter = &mut defaults.charters[0];
        charter.policy.inspect = true;
        charter.policy.modify = true;
        charter.policy.shell = true;
        charter.policy.agents = false;
        charter.policy.turns = CHARTER_TURNS;
        charter.spend = PRODUCER_SPEND;
    }
    let forge = if document["application"]["demo"].is_null() {
        settings::forge(&document)
    } else {
        let limits = temper_service::limits::engine(
            &engine.limits,
            &temper_service::limits::profile(temper_service::limits::Profile::Small),
        )
        .expect("playground engine limits");
        temper_service::stock::forge(&limits)
    };
    temper_service::stock::Given::Own { jig: defaults, forge, ways: temper_service::stock::ways() }
}
/// Independent streams derived from a world's single seed.
#[derive(Debug, PartialEq, Eq)]
pub struct Seeds {
    /// The service's random stream.
    pub service: u64,
    /// The agents' random stream.
    pub agent: u64,
    /// The LLM fake's random stream.
    pub llm: u64,
    /// The forge fake's random stream.
    pub forge: u64,
    /// The initial deployment identifier.
    pub deployment: [u8; 16],
    /// The initial installation identifier.
    pub installation: [u8; 16],
    /// The service's deterministic cryptographic input.
    pub crypto: Secret,
}
impl Seeds {
    /// The service startup's seed inputs.
    #[must_use]
    pub fn service(&self) -> temper_service::settings::Seeds {
        temper_service::settings::Seeds {
            seed: self.service,
            deployment: self.deployment,
            installation: self.installation,
            crypto: self.crypto.clone(),
            link_secret: None,
            local_secret: None,
        }
    }
}
/// Derive distinct streams and identifiers in a stable order from one seed.
#[must_use]
pub fn seeds(one: u64) -> Seeds {
    let mut rng = skein_lib::Rng::new(one);
    let service = rng.next_u64();
    let agent = rng.next_u64();
    let llm = rng.next_u64();
    let forge = rng.next_u64();
    let deployment = [rng.next_u64().to_le_bytes(), rng.next_u64().to_le_bytes()]
        .concat()
        .try_into()
        .expect("deployment identifier");
    let installation = [rng.next_u64().to_le_bytes(), rng.next_u64().to_le_bytes()]
        .concat()
        .try_into()
        .expect("installation identifier");
    let crypto = Secret::new((0..4).flat_map(|_| rng.next_u64().to_le_bytes()).collect());
    Seeds { service, agent, llm, forge, deployment, installation, crypto }
}

pub(crate) fn base_agent_limits() -> smith_agent_service::Limits {
    smith::derive_limits()
}
