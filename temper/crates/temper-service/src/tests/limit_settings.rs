//! A box's engine resolves settings without lending them to its worker.
#![expect(
    clippy::disallowed_types,
    clippy::disallowed_methods,
    reason = "ordinary focused tests inspect diagnostic strings"
)]
use super::{fixtures, stock};
use crate::startup;
use jig_domain_core::settings::Override;
use skein_lib::List;
fn item(name: &[u8], value: u64) -> Override {
    Override { name: name.into(), value, reason: b"engine room".as_slice().into() }
}
#[test]
fn a_box_resolves_the_engine_and_keeps_the_workers_document_and_stock_shared_figures() {
    let mut input = stock::generated();
    // A worker's jig document remains its operator's, independently of the engine setting.
    let mut document: serde_json::Value = serde_json::from_slice(&input.document).unwrap();
    document["workers"][0]["limits"]["overrides"]["tasks.tasks"] = 128_u32.into();
    document["workers"][0]["limits"]["overrides"]["tasks.exemptions"] = 128_u32.into();
    input.document = serde_json::to_vec(&document).unwrap().into_boxed_slice();
    let agent = fixtures::parsed_agent(fixtures::stock_configuration(63_740, 48_000).unwrap());
    let declaration = agent.declaration();
    let directories = jig_protocol_config::Directories::working();
    let seeds = fixtures::seeds();
    let files = stock::files();
    let inputs = startup::LimitInputs::new(
        &input.document,
        &directories,
        &files,
        Some(&declaration),
        &seeds,
        &crate::stock::Given::Stock,
        true,
        None,
    );
    let resolved = startup::resolve_limits(
        &inputs,
        &[item(b"tasks.tasks", 1024)],
        &[item(b"forge.client.entries", 16), item(b"forge.entries", 16), item(b"live_goals", 20)],
    )
    .unwrap();
    let checked = startup::checked_start_with_resolved(
        &input.document,
        &directories,
        &files,
        Some(&agent),
        &seeds,
        List::with_capacity(0),
        None,
        &crate::stock::Given::Stock,
        None,
        &resolved,
    )
    .unwrap();
    assert_eq!(checked.engine.jig.limits.core.tasks.tasks, 1024);
    let worker = &checked.worker.as_ref().unwrap().0;
    assert_eq!(worker.jig.limits.core.tasks.tasks, 128);
    let shared = checked.worker_limits;
    let stock = crate::limits::worker(&inputs.stock().unwrap().1);
    assert_eq!(shared.checkout.repositories, stock.checkout.repositories);
    assert_eq!(shared.checkout.message_bytes, stock.checkout.message_bytes);
    assert_eq!(shared.git_component.stdout_bytes, stock.git_component.stdout_bytes);
    assert_eq!(shared.peer, stock.peer);
    for name in [b"checkout.repositories".as_slice(), b"git_component.stdout_bytes", b"forges.installations"] {
        let Err(found) = startup::resolve_limits(&inputs, &[], &[item(name, 1)]) else {
            panic!("a worker's setting started: {}", String::from_utf8_lossy(name));
        };
        let lines = String::from_utf8(temper_engine_service::startup::lines(&found).into_vec()).unwrap();
        assert!(lines.contains("this is a worker's limit"), "{lines}");
    }
}
