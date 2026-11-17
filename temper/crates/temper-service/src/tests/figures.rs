use crate::startup;
use crate::tests::fixtures;
#[test]
fn in_process_opening_is_quiet_until_the_engines_figures_change() {
    let document = fixtures::with(&fixtures::document(), b"workers[0].programs.check_interval_ms", b"1250");
    let directories = jig_protocol_config::Directories::working();
    let files = fixtures::files();
    let agent = fixtures::startup_agent();
    let seeds = fixtures::seeds();
    let given = crate::stock::Given::Own {
        jig: crate::stock::with_models(Box::new([fixtures::small_model()]), 32),
        forge: Box::default(),
        ways: crate::stock::ways(),
    };
    let agents = crate::stock::agents();
    let part = startup::box_worker(&document, &directories, &given, &agents).unwrap().unwrap();
    let built = part.figures.clone().unwrap();
    let checked = startup::checked_start_with(
        &document,
        &directories,
        &files,
        Some(&agent),
        &seeds,
        fixtures::forge_runtime(),
        None,
        &given,
        None,
    )
    .unwrap();
    assert_eq!(
        checked.worker.as_ref().unwrap().0.jig.limits.as_ref().unwrap().hub.source_interval,
        skein_lib::Duration::from_millis(1250)
    );
    let mut opening = temper_engine_service::figures::compose(&checked.engine, &agents).into_parts();
    opening.places = checked.engine.jig.limits.core.fleet.slots;
    let opening = jig_protocol_worker_wire::Figures::new(&jig_protocol_worker_wire::CEILINGS, opening).unwrap();
    assert_eq!(built, opening, "the in-process host sends what the box composed");
    let mut service = startup::with_setup_worker_given(
        &document,
        &directories,
        &files,
        Some(&agent),
        &seeds,
        None,
        Some(fixtures::worker_runtime()),
        fixtures::forge_runtime(),
        None,
        None,
        None,
        &given,
        Box::default(),
        Some(part),
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
