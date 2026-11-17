#![expect(clippy::disallowed_methods, reason = "ordinary tests inspect diagnostic text and edit document fixtures")]
use super::fixtures;
use crate::{settings, startup::What};
use jig_protocol_config::{self as config, Path};
use skein_lib::List;
#[test]
fn a_checked_worker_alone_has_its_shape_and_refuses_engine_application_members() {
    let input = fixtures::worker_alone_document();
    assert_eq!(crate::startup::shape(&input, &config::Directories::working()), Ok(crate::startup::Shape::WorkerAlone));
    let text = core::str::from_utf8(&input).unwrap().replace("\"application\":{", "\"application\":{\"forges\":[],");
    let found = crate::startup::shape(text.as_bytes(), &config::Directories::working()).unwrap_err();
    assert_eq!(found.problems.len(), 1);
    assert_eq!(found.problems.get(0).unwrap().path, Path::root().member(b"application").member(b"forges"));
    assert_eq!(found.problems.get(0).unwrap().what, What::WorkerAlone);
}

#[test]
fn a_worker_alone_checks_its_application_paths_and_its_git_and_channel_limits() {
    let input = fixtures::worker_alone_document();
    let mut value: serde_json::Value = serde_json::from_slice(&input).unwrap();
    value["application"]["workers"] = serde_json::from_slice(br#"[{"root":"../escape","git":"git"}]"#).unwrap();
    let found =
        crate::startup::shape(&serde_json::to_vec(&value).unwrap(), &config::Directories::working()).unwrap_err();
    assert_eq!(found.problems.get(0).unwrap().path, Path::root().member(b"application").member(b"workers"));
    assert_eq!(found.problems.get(0).unwrap().what, What::Jig(config::What::Fixed));
}

#[test]
#[expect(clippy::disallowed_macros, reason = "the regression reads the shipped example")]
fn a_worker_alone_with_the_owned_examples_worker_fits_one_ring() {
    // Linux's IORING_MAX_ENTRIES, the largest submission ring.
    const IORING_MAX_ENTRIES: u32 = 32_768;
    let input = fixtures::owned_worker_alone_document();
    let directories = config::Directories::working();
    let mut agent = fixtures::owned_worker_agent();
    agent.host_limits.message_bytes = 10_662;
    agent.host_limits.messages = 82;
    let mut part = crate::startup::worker_part(&input, &directories, Some(b"/usr/bin/git")).unwrap();
    let owned = include_bytes!("../../../../docs/examples/owned.json");
    let document =
        config::document(owned, &config::DocumentLimits { bytes: 1_048_576, tokens: 1_048_576, depth: 32 }).unwrap();
    let ats = config::read::list(
        &document,
        temper_service_core::startup::member(&document, 0, b"workers").unwrap(),
        document.len(),
    )
    .unwrap();
    let worker =
        config::worker(&document, *ats.get(0).unwrap(), &directories, &temper_service_core::startup::fixed_worker())
            .unwrap();
    let engine_at = temper_service_core::startup::member(&document, 0, b"engine").unwrap();
    let engine = config::engine(
        &document,
        engine_at,
        &directories,
        config::Defaults::Passed,
        &temper_engine_service_startup::startup::fixed(&directories).unwrap(),
    )
    .unwrap();
    part.worker.jig.limits = Some(engine.limits);
    part.worker.jig.slots = worker.slots;
    let application = temper_service_core::startup::member(&document, 0, b"application").unwrap();
    let at = temper_service_core::startup::member(&document, application, b"limits").unwrap();
    part.limits = crate::worker_settings::Limits::shared(
        temper_service_core::startup::configured_limits(
            &document,
            at,
            &Path::root(),
            &mut crate::startup::Problems::empty(),
        )
        .unwrap(),
    );
    let payload = crate::worker_settings::payload_limits(&part.limits).unwrap();
    let link = crate::worker_settings::link_limits(&part.worker.jig, &payload, part.limits.peer.forges).unwrap();
    assert_eq!(
        link.room,
        jig_protocol_worker_wire::sizes::largest_body(&link.sizes, jig_protocol_worker_wire::Kind::Turn).unwrap() + 8
    );
    let operations = temper_service_core::process::Process::operations_for(
        &part.worker.jig.limits.as_ref().expect("checked worker limits").io,
        crate::worker_settings::operations(&crate::worker_settings::git_component(part.limits.git_component)).unwrap(),
    )
    .unwrap();
    let mut files = settings::Files { entries: List::with_capacity(2) };
    for file in &part.files {
        files
            .entries
            .push((file.path.clone(), skein_lib::Secret::new(b"fixture-worker-secret".as_slice().into())))
            .unwrap();
    }
    let service = crate::startup::worker_alone(part, &files, &agent, fixtures::worker_runtime(), 7).unwrap();
    assert!(service.operations() <= IORING_MAX_ENTRIES);
    assert_eq!(service.operations(), operations, "the start's check and the ring agree");
}

#[test]
fn a_worker_alone_applies_its_names_without_engine_range_checks() {
    let directories = config::Directories::working();
    let input = fixtures::worker_alone_document();
    assert_eq!(crate::startup::shape(&input, &directories).unwrap(), crate::startup::Shape::WorkerAlone);
    let mut part = fixtures::part(&input);
    part.limits.checkout.conflicts = 7;
    assert_eq!(part.limits.checkout.conflicts, 7);
    let service = crate::startup::worker_alone(
        part,
        &fixtures::files(),
        &fixtures::startup_agent(),
        fixtures::worker_runtime(),
        7,
    )
    .unwrap();
    assert_eq!(service.worker.env.limits.checkout.conflicts, 7);
}

#[test]
fn the_worker_uses_constant_deadlines_workspaces_and_the_shells_git() {
    use skein_lib::Duration;
    let directories = config::Directories::given(b"/config", b"/data", b"/cache").unwrap();
    for git in [Some(b"/tools/git".as_slice()), None] {
        let part = crate::startup::worker_part(&fixtures::worker_alone_document(), &directories, git).unwrap();
        let worker = &part.worker;
        assert_eq!(worker.root.as_ref(), b"/data/workspaces");
        assert_eq!(part.named.root, worker.root);
        assert_eq!(worker.git.as_deref(), git);
        assert_eq!(
            worker.jig.graces,
            config::Graces {
                grace: Duration::from_millis(300_000),
                drain: Duration::from_millis(180_000),
                agent_stop: Duration::from_millis(5_000),
                delivery: Duration::from_millis(1_000),
                save: Duration::from_millis(1_000)
            }
        );
        assert_eq!(worker.jig.stop_bound, Duration::from_secs(326));
        assert_eq!(worker.jig.check_interval, Duration::from_secs(60));
        assert_eq!(worker.jig.limits.as_ref().expect("worker limits").hub.source_interval, worker.jig.check_interval);
    }
    let found =
        crate::startup::worker_part(&fixtures::worker_alone_document(), &directories, Some(b"git")).unwrap_err();
    assert_eq!(found.problems.get(0).unwrap().what, What::Absolute);
}
