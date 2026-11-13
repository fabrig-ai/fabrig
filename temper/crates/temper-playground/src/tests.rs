//! Composition startup checks (temper/docs/design/playground.md, 3).
mod visitor;

use crate::{documents, fakes};

#[test]
fn the_deployment_document_is_one_temper_serve_starts() {
    let bytes = documents::document();
    let value = documents::world_document();
    assert!(value["engine"]["limits"]["overrides"]["core.resume_bytes"].as_u64().unwrap() >= 4_194_304);
    assert_eq!(
        value["engine"]["limits"]["overrides"]["io.output"],
        value["workers"][0]["limits"]["overrides"]["io.output"]
    );
    let result = temper_service::startup::with_setup_given(
        &bytes,
        &jig_protocol_config::Directories::working(),
        &documents::files(),
        Some(&documents::agent()),
        &documents::seeds(7).service(),
        None,
        Some(temper_service::WorkerRuntime {
            root: skein_io::kernel::Fd::new(2),
            path: b"/worker".as_slice().into(),
            agent: documents::pinned_agent(),
        }),
        documents::forges(),
        None,
        None,
        None,
        &documents::given(&value),
        Box::new([]),
    );
    if let Err(problems) = result {
        panic!("{}", String::from_utf8_lossy(&temper_service::startup::lines(&problems)));
    }
}

#[test]
fn resumes_have_a_floor_without_raising_turn_carriers() {
    for turn in [65_536_u32, 4_194_304] {
        let mut document = documents::world_document();
        documents::turn_bounds::apply(&mut document, turn);
        let turn = u64::from(turn);
        let resume = (turn + 4).max(4_194_304);
        let window = resume + (u64::try_from(size_of::<Box<[u8]>>()).unwrap() - 4) * (resume / 4);
        let engine = &document["engine"]["limits"]["overrides"];
        assert_eq!(engine["core.resume_bytes"], resume);
        for hub in [engine, &document["workers"][0]["limits"]["overrides"]] {
            assert_eq!(hub["hub.transcript_bytes"], window);
            assert_eq!(hub["hub.turn_bytes"], turn);
            assert_eq!(hub["hub.turn_queue_bytes"], 8 * turn);
        }
        let root = &document["application"]["limits"]["overrides"];
        assert_eq!(root["root.journal.transcript_bytes"], resume);
        assert_eq!(root["root.loads.transcript_bytes"], turn);
        let loaded = turn + u64::try_from(size_of::<temper_engine_domain::Record>()).unwrap();
        assert_eq!(root["root.loads.bytes"], loaded);
        assert_eq!(root["root.loads.reply_bytes"], loaded);
        assert_eq!(root["root.loads.read_bytes"], 2 * turn);
    }
}

#[test]
fn one_seed_derives_every_seed() {
    let seven = documents::seeds(7);
    assert_eq!(seven, documents::seeds(7));
    assert_ne!(seven, documents::seeds(8));
    let streams = [7, seven.service, seven.agent, seven.llm, seven.forge];
    for (index, seed) in streams.iter().enumerate() {
        for other in &streams[index + 1..] {
            assert_ne!(seed, other);
        }
    }
    assert_ne!(seven.deployment, seven.installation);
    assert_eq!(seven.crypto.len(), 32);
}

#[test]
fn the_fakes_limits_are_their_constructors() {
    let seeds = documents::seeds(7);
    let _llm = fakes::scripted_provider(Box::new([]), seeds.llm);
    let domain = skein_fake_forge_domain::Domain::new(&documents::hosted_git::config(), seeds.forge);
    let forge = std::rc::Rc::new(std::cell::RefCell::new(skein_fake_peers::forge::Forge::new(
        domain,
        documents::hosted_git::config(),
    )));
    let mut peer = skein_fake_peers::forge::Peer::new(
        documents::FORGE,
        fakes::peer_limits(),
        fakes::forge_table(),
        fakes::forge_limits(),
        forge,
        seeds.forge,
    )
    .expect("configured forge peer");
    fakes::setup_peer(&mut peer);
}

#[test]
fn the_owned_values_equal_the_test_kits_values() {
    use jig_channel_kit::fixtures as channel;
    assert_eq!(documents::KEY, channel::KEY);
    let owned: serde_json::Value = serde_json::from_slice(&documents::engine_document()).expect("owned engine JSON");
    let mut source: serde_json::Value =
        serde_json::from_slice(&channel::engine_document()).expect("channel engine JSON");
    // Temper sizes both watch rooms from clients.streams (temper/docs/design/protocol/configuration/limits.md).
    for name in ["views.watchers", "views.watchers_per_party"] {
        source["engine"]["limits"]["overrides"].as_object_mut().expect("engine overrides").remove(name);
    }
    assert_eq!(owned, source);
    let mut worker: serde_json::Value = serde_json::from_slice(channel::WORKER_DOCUMENT).expect("kit worker JSON");
    worker["workers"][0]["graces"]["grace_ms"] = 300_000.into();
    assert_eq!(serde_json::from_slice::<serde_json::Value>(documents::WORKER_DOCUMENT).expect("worker JSON"), worker);

    let mut profile = smith_agent_process_world::profile();
    profile.policy.host_tools =
        u32::try_from(temper_engine_smith::tools(skein_lib::Duration::from_secs(10)).len()).expect("tools");
    profile.declared.tool_payload = 2048;
    profile.declared.read_window = 256;
    profile.declared.conversations = 1;
    profile.declared.calls_per_response = 1;
    profile.policy.max_turns = 800;
    profile.policy.max_time = skein_lib::Duration::from_secs(28_800);
    profile.policy.check_timeout = skein_lib::Duration::from_secs(5);
    let hub = jig_protocol_config::profile(jig_protocol_config::Profile::Small).hub;
    profile.policy.inbox = hub.held.max(hub.inbox);
    profile.policy.sections = 8;
    assert_eq!(documents::agent_profile(), profile);
    let mut declarations = smith_agent_process_world::declarations(0);
    declarations.endpoints[0].limits.output = 2048;
    assert_eq!(documents::agent_declarations(), declarations);
    let owned = fakes::llm_peer_limits();
    let source = smith_agent_process_world::fake::limits();
    assert_eq!(owned.io, source.io);
    assert_eq!(owned.connections, source.connections);
    assert_eq!(owned.queue, source.queue);
    assert_eq!(owned.plaintext, source.plaintext);
    assert_eq!(owned.ciphertext, source.ciphertext);
    assert_eq!(owned.observations, source.observations);
    assert_eq!(owned.observation_bytes, source.observation_bytes);
    let mut config = skein_llm_kit::fake::config();
    config.query_bytes = 65_536;
    assert_eq!(fakes::llm_config(), config);
    let mut protocol = skein_llm_kit::fake::limits(&skein_llm_kit::limits());
    protocol.documents.openai.request = 65_536;
    protocol.documents.openai.strings = 16_384;
    protocol.http.body = 65_536;
    protocol.documents.openai.tokens = 4096;
    protocol.documents.openai.output_items = 256;
    protocol.documents.anthropic.output_items = 256;
    assert_eq!(fakes::llm_limits(), protocol);
    assert_eq!(documents::hosted_git::config(), temper_worker_checkout_world::hosted::config());
    let identity = documents::hosted_git::identity();
    let original = temper_worker_checkout_world::hosted::identity();
    assert_eq!(identity.token, original.token);
    assert_eq!(identity.login, original.login);
    assert_eq!(identity.name, original.name);
    assert_eq!(identity.email, original.email);
}

#[test]
fn the_default_restart_provider_is_admitted() {
    use crate::programs::Harness;
    let _provider = <() as Harness>::provider(7);
    assert_eq!(<() as Harness>::forge_seed(7), documents::seeds(7).forge);
}
