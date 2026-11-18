use crate::startup;
use crate::tests::fixtures;
#[test]
fn in_process_opening_is_quiet_until_the_engines_figures_change() {
    let document = fixtures::stock_document();
    let directories = jig_protocol_config::Directories::working();
    let files = fixtures::stock_files();
    let agent = fixtures::parsed_agent(fixtures::stock_configuration(63_740, 48_000).unwrap());
    let seeds = fixtures::seeds();
    let given = crate::stock::Given::Stock;
    let agents = crate::stock::agents();
    let part = startup::box_worker(&document, &directories, &given, &agents).unwrap().unwrap();
    let built = part.figures.clone().unwrap();
    let mut checked = startup::checked_start_with_worker(
        &document,
        &directories,
        &files,
        Some(&agent),
        &seeds,
        skein_lib::List::with_capacity(0),
        None,
        &given,
        None,
        Some(part),
    )
    .unwrap();
    assert_eq!(
        checked.worker.as_ref().unwrap().0.jig.limits.as_ref().unwrap().hub.source_interval,
        skein_lib::Duration::from_millis(60_000)
    );
    let mut opening = temper_engine_service::figures::compose(&checked.engine, &agents).into_parts();
    opening.places = checked.engine.jig.limits.core.fleet.slots;
    let opening = jig_protocol_worker_wire::Figures::new(&jig_protocol_worker_wire::CEILINGS, opening).unwrap();
    assert_eq!(built, opening, "the in-process host sends what the box composed");
    // This worker-notice fixture has no accounts shell.
    for provider in &mut checked.engine.jig.llm.providers {
        provider.sign_in = None;
    }
    let mut service = startup::from_checked(
        checked,
        &files,
        Some(&agent),
        &seeds,
        None,
        Some(fixtures::worker_runtime()),
        None,
        None,
        Box::default(),
    )
    .unwrap();
    let worker = service.worker.as_mut().unwrap();
    worker.link(fixtures::NOW, jig_protocol_worker_link::Request::Figures { figures: opening.clone() });
    assert_eq!(worker.notes().pop(), None);
    let mut changed = opening.into_parts();
    changed.places = changed.places.checked_add(1).unwrap();
    let changed = jig_protocol_worker_wire::Figures::new(&jig_protocol_worker_wire::CEILINGS, changed).unwrap();
    worker.link(fixtures::NOW, jig_protocol_worker_link::Request::Figures { figures: changed });
    assert_eq!(worker.notes().pop(), Some(crate::worker_alone::Note::FiguresChanged));
    assert_eq!(worker.notes().pop(), None);
}
