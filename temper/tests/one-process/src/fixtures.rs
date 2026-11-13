//! Configuration bytes, private files and scripts shared by the stories.
#[cfg(test)]
pub(crate) mod durable;
use crate::world::Settings;
use jig_channel_kit::fixtures as channel;
use skein_lib::{Duration, Secret};
pub(crate) use temper_playground::documents::{
    CHAT_WAIT_MS, CLIENTS, FORGE, FORGE_CONNECTOR, GRANT_KINDS, PROCEDURE_CODE, PRODUCER_SPEND, PRODUCER_TOOLS,
    REPOSITORY, WORKER_USER, agent_limits,
};
pub use temper_playground::documents::{HOOK_SECRET, KEY as LLM_TOKEN};
pub use temper_playground::fakes::{API_TOKEN, GIT_TOKEN};
pub(crate) use temper_playground::fakes::{forge_limits, peer_limits};
use temper_playground::{documents, fakes};
pub const SEED: u64 = 7;
pub const ANSWER: &[u8] = b"World answer";
#[must_use]
pub fn two_forge_records() -> Vec<temper_engine_domain_forge::Stored> {
    [1, 2]
        .into_iter()
        .map(|task| temper_engine_domain_forge::Stored::Names { task, resources: Box::default() })
        .collect()
}
pub const BRANCH_HEAD: u64 = 2;
pub const MOVED_BASE: u64 = 3;
pub(crate) const MERGE_BRANCH: &[u8] = b"temper/1/c2";
pub(crate) const PROJECT: u32 = 1;
pub(crate) const AGENT_KIND: u16 = 1;
// A task names the repository by forge 1 and its id, packed (domain/forge/adoption.md).
pub(crate) const REPOSITORY_TAG: u64 = temper_engine_domain_forge::tag(1, REPOSITORY);
pub(crate) const PARAMETER_NAMES: [u16; 3] = [1, 2, 3];
pub(crate) const CHANGE_KIND: u16 = 1;
pub(crate) const CHANGE_WORDS: u32 = 64;
pub(crate) const PRODUCER_TASKS: u32 = 2;
pub(crate) const PRODUCER_DEPTH: u32 = 1;
pub(crate) const PRODUCER_NOTES: u32 = 0;
pub(crate) const CALL_TOKENS: u64 = 1;
pub(crate) const PERSON_WAIT: Duration = Duration::from_secs(5);
pub(crate) const FORGE_STATUSES: u32 = 8;
pub(crate) const CI_SILENT: u32 = 0;
pub(crate) const CI_PASSES: u32 = 1000;
pub(crate) const CI_RERUNS: u32 = 0;
pub(crate) const MOVING_USER: u64 = 2;
pub(crate) const ADOPTION_FORM_START: usize = 7;
pub(crate) const ADOPTION_FORM_STEPS: usize = 9;
pub(crate) const ADOPTION_REPLY_STEP: usize = 9;
#[must_use]
pub fn calm(seed: u64) -> Settings {
    Settings { seed, config: skein_sim::Config::calm(), memory: skein_world::Memory::Unchecked }
}
pub(crate) fn client_limits() -> temper_client_shell_terminal::limits::Limits {
    let mut limits = temper_client_shell_terminal::limits::native();
    limits.connections.body_bytes = 262_144;
    limits.client.client.domain.fields = 8;
    limits.client.client.domain.parts = 4;
    limits.client.client.domain.reads = 8;
    limits.client.client.domain.text = 128;
    limits.client.client.domain.long_text = 128;
    limits.client.client.domain.turn_text = 128;
    limits.client.client.domain.document_bytes = 65_536;
    limits.client.client.domain.requests.bytes = 262_144;
    limits.client.client.domain.ask_bytes =
        limits.client.client.domain.requests.bytes / u64::from(limits.client.client.domain.requests.requests);
    limits
}
pub(crate) fn document() -> Vec<u8> {
    let mut value = world_document();
    documents::settings::strip_models(&mut value);
    value.as_object_mut().expect("deployment").remove("world_settings");
    serde_json::to_vec(&value).expect("deployment JSON")
}
pub(crate) fn world_document() -> serde_json::Value {
    let mut value = documents::world_document_with(|value| {
        crate::stories::configure(value);
        if crate::stories::uses_stock() {
            documents::chat_policy(value);
        }
    });
    // The native client admits eight exchanges beside its event stream.
    value["engine"]["limits"]["overrides"]["clients_http.calls"] = 9.into();
    value["engine"]["limits"]["overrides"]["clients.exchanges"] = 9.into();
    crate::push::configure(&mut value);
    crate::lower::configure(&mut value);
    crate::lower::github::configure(&mut value);
    crate::m15::configure(&mut value);
    crate::public::configure(&mut value);
    crate::review_delivery::configure(&mut value);
    if crate::stories::model_list() {
        value["engine"]["llm"]["providers"][0]["keep"] = serde_json::json!(["*"]);
    }
    value
}
pub(crate) fn files() -> temper_service::settings::Files {
    let mut files = documents::files();
    crate::push::files(&mut files);
    if crate::stories::held_change() && crate::stories::first_start() {
        files
            .entries
            .push((b"forge-1/admin_token".as_slice().into(), Secret::new(crate::restart_control::ADMIN_TOKEN.into())))
            .expect("fixture handover");
    }
    files
}
pub(crate) fn seeds() -> temper_service::settings::Seeds {
    let seed = channel::seeds();
    temper_service::settings::Seeds {
        deployment: seed.deployment,
        seed: seed.seed,
        crypto: seed.crypto,
        installation: seed.installation,
        link_secret: None,
        local_secret: None,
    }
}
pub(crate) fn provider() -> skein_fake_peers::llm::Peer {
    let mut script = channel::one_chat(b"Hello");
    script.result = br#"{"form":"report","text":"World answer","fields":{}}"#.as_slice().into();
    scripted_provider(Box::new([skein_fake_llm_domain::api::Script {
        cue: Box::default(),
        turns: channel::provider_turns(&script),
    }]))
}
pub(crate) fn read_then_wait_provider() -> skein_fake_peers::llm::Peer {
    use skein_fake_llm_domain::api::{Finish, Line, Script, Turn};
    scripted_provider(Box::new([Script {
        cue: Box::default(),
        turns: Box::new([
            Turn {
                lines: Box::new([Line::Call {
                    name: b"read".as_slice().into(),
                    arguments: br#"{"path":"README.md"}"#.as_slice().into(),
                }]),
                finish: Finish::ToolCalls,
                tokens: 20,
            },
            Turn {
                lines: Box::new([
                    Line::Text { text: ANSWER.into() },
                    Line::Call { name: b"wait".as_slice().into(), arguments: b"{}".as_slice().into() },
                ]),
                finish: Finish::ToolCalls,
                tokens: 20,
            },
        ]),
    }]))
}
/// Temper's flat host schemas and a mounted workspace fit this fake's request.
pub(crate) fn scripted_provider(scripts: Box<[skein_fake_llm_domain::api::Script]>) -> skein_fake_peers::llm::Peer {
    let mut peer = fakes::scripted_provider(scripts, 17);
    if crate::stories::model_list() {
        peer.set_list(Some(br#"{"data":[{"id":"fake"},{"id":"gpt-future"}]}"#.as_slice().into()), None);
    }
    peer
}
/// Adoption defaults do not vary with a seed or a deployment phase.
pub(crate) fn adoption_given() -> temper_service::stock::Given {
    use temper_service::stock::Given;
    static GIVEN: std::sync::OnceLock<Given> = std::sync::OnceLock::new();
    match GIVEN.get_or_init(|| crate::stories::given(&serde_json::from_slice(&document()).expect("deployment JSON"))) {
        Given::Stock => Given::Stock,
        Given::Own { jig, forge, ways } => {
            let mut policies = skein_lib::Map::with_capacity(jig.policies.capacity());
            for (number, policy) in &jig.policies {
                policies.insert(*number, policy.clone()).expect("adoption policies");
            }
            Given::Own {
                jig: jig_domain_core::settings::Defaults {
                    charters: jig.charters.clone(),
                    templates: jig.templates.clone(),
                    policies,
                    rules: jig.rules.clone(),
                    models: jig.models.clone(),
                },
                forge: forge.clone(),
                ways: ways.clone(),
            }
        }
    }
}
/// The adoption story has the same codec bounds across seeds, cuts and cold starts.
pub(crate) fn adoption_codec() -> jig_protocol_store::codec::View {
    static CODEC: std::sync::OnceLock<jig_protocol_store::codec::View> = std::sync::OnceLock::new();
    *CODEC.get_or_init(codec)
}
pub(crate) fn codec() -> jig_protocol_store::codec::View {
    let limits = jig_test_service_limits(&engine().limits);
    jig_protocol_store::codec::limits(&limits.root.core).expect("codec")
}
/// The configured models' names the people records name models by (jig's protocol/store/records.md, 1).
pub(crate) fn store_names() -> &'static [jig_domain_core::settings::ModelName] {
    static NAMES: std::sync::OnceLock<Box<[jig_domain_core::settings::ModelName]>> = std::sync::OnceLock::new();
    NAMES.get_or_init(|| jig_protocol_store::codec::names(&engine().llm.models()))
}
fn engine() -> jig_protocol_config::Engine {
    let bytes = document();
    let size = u32::try_from(bytes.len()).expect("fixture document");
    let doc = jig_protocol_config::document(
        &bytes,
        &jig_protocol_config::DocumentLimits { bytes: size, tokens: size, depth: 32 },
    )
    .expect("document");
    let mut object = jig_protocol_config::read::Object::new(&doc, 0).expect("object");
    let at = object.member(&doc, b"engine").expect("engine member");
    temper_service::startup::engine_with(
        &doc,
        at,
        &jig_protocol_config::Directories::working(),
        &crate::stories::given(&serde_json::from_slice(&bytes).expect("deployment JSON")),
    )
    .expect("engine")
}
fn jig_test_service_limits(overrides: &jig_protocol_config::Limits) -> jig_test_service::limits::Limits {
    jig_test_service::limits::for_profile(jig_protocol_config::Profile::Small, overrides)
}

/// Provision a deployment source through the native client before each fresh story.
pub(crate) fn account_steps() -> Box<[skein_fake_person::Step<temper_client_domain::Address>]> {
    use skein_fake_person::{Find, Step};
    use skein_ui::Role;
    Box::new([
        Step::Go { address: temper_client_domain::Address::ConnectModel { way: None } },
        Step::See { find: Find::named(Role::Link, b"Use deployment accounts"), within: PERSON_WAIT },
        Step::Press { find: Find::named(Role::Link, b"Use deployment accounts") },
        Step::Press { find: Find::named(Role::Link, b"Add key") },
        Step::Type { find: Find::named(Role::TextField, b"Key (pasted once)"), words: documents::KEY.into() },
        Step::Press { find: Find::named(Role::Button, b"Add key") },
        Step::See { find: Find::text(b"Added your"), within: PERSON_WAIT },
    ])
}

pub mod core_after_project;
