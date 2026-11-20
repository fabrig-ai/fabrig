#![expect(clippy::disallowed_macros, reason = "ordinary tests supply environment entries as JSON")]
use crate::startup;
use crate::tests::fixtures;

#[test]
fn the_box_keeps_entries_then_its_own_home_and_path() {
    let mut document: serde_json::Value = serde_json::from_slice(&fixtures::stock_document()).unwrap();
    document["application"]["environment"] = serde_json::json!([{"name":"CARGO_TARGET_DIR","value":"/srv/target"}]);
    let own = [
        (Box::from(b"PATH".as_slice()), Box::from(b"/usr/bin:/bin".as_slice())),
        (Box::from(b"OTHER".as_slice()), Box::from(b"x".as_slice())),
        (Box::from(b"HOME".as_slice()), Box::from(b"/home/w".as_slice())),
    ];
    let part = startup::box_worker(
        &serde_json::to_vec(&document).unwrap(),
        &jig_protocol_config::Directories::working(),
        &crate::stock::Given::Stock,
        &crate::stock::agents(),
        4,
        &own,
    )
    .unwrap()
    .unwrap();
    assert_eq!(
        part.environment.as_ref(),
        [
            (Box::from(b"CARGO_TARGET_DIR".as_slice()), Box::from(b"/srv/target".as_slice())),
            (Box::from(b"HOME".as_slice()), Box::from(b"/home/w".as_slice())),
            (Box::from(b"PATH".as_slice()), Box::from(b"/usr/bin:/bin".as_slice())),
        ]
    );
    let checked = startup::checked_start_with_worker(
        &serde_json::to_vec(&document).unwrap(),
        &jig_protocol_config::Directories::working(),
        &fixtures::stock_files(),
        Some(&fixtures::parsed_agent(fixtures::stock_configuration(63_740, 48_000).unwrap())),
        &fixtures::seeds(),
        skein_lib::List::with_capacity(0),
        None,
        &crate::stock::Given::Stock,
        None,
        Some(part),
    )
    .unwrap();
    assert!(checked.worker.is_some());
}

#[test]
fn the_box_builds_at_local_places_not_document_slots() {
    let mut document: serde_json::Value = serde_json::from_slice(&fixtures::stock_document()).unwrap();
    let directories = jig_protocol_config::Directories::working();
    let agent = fixtures::parsed_agent(fixtures::stock_configuration(63_740, 48_000).unwrap());
    for (slots, places) in [(1_u32, 4), (4, 4), (1, 2), (4, 2), (41, 2)] {
        document["workers"][0]["slots"] = slots.into();
        let bytes = serde_json::to_vec(&document).unwrap();
        let part = startup::box_worker(
            &bytes,
            &directories,
            &crate::stock::Given::Stock,
            &crate::stock::agents(),
            places,
            &[],
        )
        .unwrap()
        .unwrap();
        assert_eq!(part.figures.as_ref().unwrap().places, places);
        assert_eq!(part.worker.jig.slots, places);
        assert_eq!(part.worker.jig.limits.as_ref().unwrap().hub.slots, places);
        assert_eq!(part.limits.checkout.workspaces, places);
        assert_eq!(part.limits.git_component.operations, places);
        let checked = startup::checked_start_with_worker(
            &bytes,
            &directories,
            &fixtures::stock_files(),
            Some(&agent),
            &fixtures::seeds(),
            skein_lib::List::with_capacity(0),
            None,
            &crate::stock::Given::Stock,
            None,
            Some(part),
        )
        .unwrap();
        assert_eq!(checked.engine.jig.limits.core.fleet.slots, 32);
        let (worker, prepared) = checked.worker.unwrap();
        assert_eq!(worker.jig.slots, places);
        assert_eq!(prepared.root.host.slots, places);
        assert_eq!(prepared.link.sizes.slots, places);
    }
}

#[test]
fn a_box_derives_its_composed_carriers_from_its_agent() {
    use temper_client_protocol_wire::v1 as w;

    let document = fixtures::stock_document();
    let agent = fixtures::parsed_agent(fixtures::stock_configuration(8192, 48_000).unwrap());
    let mut agents = crate::stock::agents();
    for figure in agents.profile.iter_mut() {
        if figure.name.as_ref() == b"declared.tool_payload" {
            *figure = w::AgentFigure { name: b"declared.tool_payload".as_slice().into(), value: 8192 };
        }
    }
    let agents = agents;
    let declaration = agent.declaration();
    let (mut jig, own) =
        crate::limits::base(40, 4096, temper_service_core::derived::Sources::agent(&declaration)).unwrap();
    jig.core.people.words = jig.core.tasks.message_bytes;
    let part = startup::box_worker_with_limits(
        &document,
        &jig_protocol_config::Directories::working(),
        &crate::stock::Given::Stock,
        &agents,
        4,
        &[],
        Some(&declaration),
        (&jig, &own),
    )
    .unwrap()
    .unwrap();
    let limits = crate::worker_settings::root_limits(&part.worker, &agent, &part.limits);
    let mut problems = temper_worker_service::startup::Problems::empty();
    temper_worker_service::startup::worker::worker(&limits, &mut problems);
    assert!(problems.is_empty(), "{problems:?}");
    let figures = part.figures.unwrap();
    let mut turn = None;
    for limit in &figures.limits {
        if limit.name.as_ref() == b"hub.turn_bytes" {
            turn = Some(limit.value);
        }
    }
    assert_eq!(turn, Some(u64::from(agent.declaration().turn_body)));
}

#[test]
fn an_engine_alone_refuses_application_environment() {
    let mut document: serde_json::Value = serde_json::from_slice(&fixtures::engine_alone_document()).unwrap();
    document["application"]["environment"] = serde_json::json!([{"name":"A","value":"one"}]);
    let found = startup::box_worker(
        &serde_json::to_vec(&document).unwrap(),
        &jig_protocol_config::Directories::working(),
        &crate::stock::Given::Stock,
        &crate::stock::agents(),
        4,
        &[],
    )
    .unwrap_err();
    assert_eq!(startup::lines(&found).as_ref(), b"application.environment: unknown member\n");
}
#[test]
fn in_process_opening_is_quiet_until_the_engines_figures_change() {
    let document = fixtures::stock_document();
    let directories = jig_protocol_config::Directories::working();
    let files = fixtures::stock_files();
    let agent = fixtures::parsed_agent(fixtures::stock_configuration(63_740, 48_000).unwrap());
    let seeds = fixtures::seeds();
    let given = crate::stock::Given::Stock;
    let agents = crate::stock::agents();
    let part = startup::box_worker(&document, &directories, &given, &agents, 4, &[]).unwrap().unwrap();
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
    let mut opening = temper_engine_service::figures::compose(&checked.engine, &agents);
    opening.places = 4;

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
    let mut changed = opening;
    changed.places = changed.places.checked_add(1).unwrap();

    worker.link(fixtures::NOW, jig_protocol_worker_link::Request::Figures { figures: changed });
    assert_eq!(worker.notes().pop(), Some(crate::worker_alone::Note::FiguresChanged));
    assert_eq!(worker.notes().pop(), None);
}
