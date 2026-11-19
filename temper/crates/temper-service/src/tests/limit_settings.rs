//! A box shares its resolved engine figures with its local worker.
use super::{fixtures, stock};
use crate::startup;
use jig_domain_core::settings::Override;
use skein_lib::List;
fn item(name: &[u8], value: u64) -> Override {
    Override { name: name.into(), value, reason: b"engine room".as_slice().into() }
}
#[test]
fn a_box_gives_its_worker_the_resolved_engine_and_shared_figures() {
    let input = stock::generated();
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
        None,
        &resolved,
    )
    .unwrap();
    assert_eq!(checked.engine.jig.limits.core.tasks.tasks, 1024);
    let worker = &checked.worker.as_ref().unwrap().0;
    assert_eq!(worker.jig.limits.as_ref().unwrap().core.tasks.tasks, 1024);
    let shared = checked.worker_limits;
    let stock = crate::limits::worker(&inputs.stock().unwrap().1);
    assert_eq!(shared.checkout.repositories, stock.checkout.repositories);
    assert_eq!(shared.checkout.message_bytes, stock.checkout.message_bytes);
    assert_eq!(shared.git_component.stdout_bytes, stock.git_component.stdout_bytes);
    assert_eq!(shared.peer, stock.peer);
    let value = u64::from(stock.checkout.repositories);
    let shared = startup::resolve_limits(&inputs, &[], &[item(b"checkout.repositories", value)]).unwrap();
    assert_eq!(shared.temper.checkout.repositories, stock.checkout.repositories);
}
