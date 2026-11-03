//! Composition startup checks (temper/docs/design/playground.md, 3).
use crate::{documents, fakes};

#[test]
fn the_deployment_document_is_one_temper_serve_starts() {
    let bytes = documents::document();
    let value = serde_json::from_slice(&bytes).expect("deployment JSON");
    let result = temper_service::startup::with_setup_given(
        &bytes,
        &jig_protocol_config::Directories::working(),
        &documents::files(),
        Some(&documents::agent()),
        &documents::seeds(7).service(),
        None,
        Some(temper_service::WorkerRuntime { root: skein_io::kernel::Fd::new(2), path: b"/worker".as_slice().into() }),
        documents::forges(),
        None,
        None,
        &documents::given(&value),
    );
    if let Err(problems) = result {
        panic!("{}", String::from_utf8_lossy(&temper_service::startup::lines(&problems)));
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
    let domain = skein_fake_forge_domain::Domain::new(&documents::hosted_git::CONFIG, seeds.forge);
    let forge = std::rc::Rc::new(std::cell::RefCell::new(skein_fake_peers::forge::Forge::new(
        domain,
        documents::hosted_git::CONFIG,
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
    use jig_channel_world::fixtures as channel;
    assert_eq!(documents::KEY, channel::KEY);
    assert_eq!(documents::engine_document(), channel::engine_document());
    assert_eq!(documents::WORKER_DOCUMENT, channel::WORKER_DOCUMENT);

    let mut profile = smith_agent_process_world::profile();
    profile.policy.host_tools =
        u32::try_from(temper_engine_smith::tools(skein_lib::Duration::from_secs(10)).len()).expect("tools");
    profile.declared.tool_payload = 2048;
    profile.declared.read_window = 256;
    profile.declared.conversations = 1;
    profile.declared.calls_per_response = 1;
    profile.policy.max_turns = 16;
    profile.policy.check_timeout = skein_lib::Duration::from_secs(5);
    let hub = jig_protocol_config::profile(jig_protocol_config::Profile::Small).hub;
    profile.policy.inbox = hub.held.max(hub.inbox);
    profile.policy.sections = 8;
    assert_eq!(documents::agent_profile(), profile);
    let mut declarations = smith_agent_process_world::declarations(0);
    declarations.endpoints[0].models[0].output = 2048;
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
    let mut config = skein_llm_world::fake::config();
    config.query_bytes = 65_536;
    assert_eq!(fakes::llm_config(), config);
    let mut protocol = skein_llm_world::fake::limits(&skein_llm_world::limits());
    protocol.documents.openai.request = 65_536;
    protocol.documents.openai.strings = 16_384;
    protocol.http.body = 65_536;
    protocol.documents.openai.tokens = 4096;
    protocol.documents.openai.output_items = 256;
    protocol.documents.anthropic.output_items = 256;
    assert_eq!(fakes::llm_limits(), protocol);
    assert_eq!(documents::hosted_git::CONFIG, temper_worker_checkout_world::hosted::CONFIG);
    let identity = documents::hosted_git::identity();
    let original = temper_worker_checkout_world::hosted::identity();
    assert_eq!(identity.token, original.token);
    assert_eq!(identity.login, original.login);
    assert_eq!(identity.name, original.name);
    assert_eq!(identity.email, original.email);
}
