use super::fixtures;
use crate::worker_settings;
use skein_io as io;
use skein_lib::{Token, Wall};
use temper_protocol_worker_wire::v1;
#[test]
fn a_dump_directory_reaches_the_agent_as_its_one_variable() {
    let mut service = fixtures::system::one_process().service;
    service.worker = Some(fixtures::started_worker().worker);
    service.dump_llm(b"/request-dumps");
    let worker = service.worker.as_mut().expect("one worker");
    worker.link(fixtures::NOW, fixtures::chat_assignment());
    let (_, launched) = fixtures::spawn(worker);
    assert_eq!(launched.program.as_ref(), fixtures::TEMPER);
    assert_eq!(
        launched.args.as_ref(),
        [Box::from(&b"internal"[..]), Box::from(&b"agent"[..]), Box::from(fixtures::AGENT_JSON)]
    );
    assert_eq!(launched.root, io::kernel::Fd::new(9));
    assert_eq!(launched.env.as_ref(), [Box::from(b"TEMPER_INTERNAL_DUMP_LLM=/request-dumps".as_slice())]);
}

#[test]
fn a_dump_directory_on_an_engine_alone_does_nothing() {
    let mut service = crate::startup::with_setup(
        &fixtures::engine_alone_document(),
        &jig_protocol_config::Directories::working(),
        &fixtures::files(),
        None,
        &fixtures::seeds(),
        None,
        None,
        fixtures::forge_runtime(),
        None,
        None,
    )
    .expect("an engine alone starts");
    assert!(service.worker_domain().is_none());
    service.dump_llm(b"/request-dumps");
    assert!(service.worker_domain().is_none());
}

#[test]
fn grant_sizes_take_credentials_source_names_and_generated_identities() {
    let name = u32::try_from(smith_host_domain::SourceName::BYTES).unwrap();
    let mut engine = fixtures::engine();
    let mut worker = fixtures::worker_settings();
    let payload = worker_settings::payload_limits(&fixtures::worker_limits()).unwrap();
    for credential in [1, name, 4096] {
        engine.jig.limits.token = credential;
        // Table capacity, not OAuth's separate parsing bound, sizes every credential scheme.
        engine.jig.limits.driver.client.document.token_bytes = 1;
        worker.jig.limits.token = credential;
        let left = crate::workers::limits(&engine).unwrap();
        let right = worker_settings::link_limits(&worker.jig, &payload, fixtures::worker_limits().peer.forges).unwrap();
        let wire = crate::translate::limits(&engine.limits).unwrap();
        let identity = u32::try_from(v1::Identity::encoded_bound(&wire).unwrap()).unwrap();
        assert_eq!(left.sizes.grant, credential.max(name).max(identity));
        let root = crate::limits::engine(&engine.jig.limits, &engine.limits).unwrap();
        let effective = temper_engine_domain::core_limits(&root);
        let store = crate::store::limits(&root, &root.forge, &engine.jig.limits.store).unwrap();
        assert_eq!(left.sizes.answer, effective.call_answer_bytes);
        assert_eq!(left.sizes.call_input, effective.call_answer_bytes);
        assert_eq!(store.core.call_answer_bytes, left.sizes.answer);
        assert_eq!(right.sizes.answer, effective.call_answer_bytes);
        assert_eq!(
            right.sizes.grant,
            worker_settings::credential_bytes(credential)
                .max(name)
                .max(temper_protocol_worker_wire::largest(&payload).unwrap().grant)
        );
    }
    engine.jig.limits.hub.charter_bytes = u64::MAX;
    assert!(crate::workers::limits(&engine).is_none());
    worker.jig.limits.hub.charter_bytes = u64::MAX;
    assert!(worker_settings::link_limits(&worker.jig, &payload, fixtures::worker_limits().peer.forges).is_none());
    worker_settings::prepare(&worker, &fixtures::worker_agent(), &fixtures::worker_limits())
        .expect_err("an unrepresentable charter envelope is refused");
}

#[test]
fn a_workers_listener_failure_is_noted_before_the_service_finishes() {
    let mut service = fixtures::started();
    let owner = Token::new(crate::tokens::WORKER_LISTENERS);
    service.process.told().push(io::Event::Failed { owner, error: io::Error::Unreachable });
    crate::iterate(&mut service, fixtures::NOW, Wall::EPOCH);
    let address = Some(io::Addr::InProcess(io::Name::new(b"workers").unwrap()));
    assert_eq!(
        service.notes().pop(),
        Some(crate::Note::WorkerListenerFailed { address, error: Some(io::Error::Unreachable) })
    );
    assert!(!service.is_empty(), "the listener's line does not wait for shutdown");
    service.process.told().push(io::Event::Closed { owner });
    crate::iterate(&mut service, fixtures::NOW, Wall::EPOCH);
    assert!(service.notes().is_empty(), "the closing event does not repeat the failure");
}
