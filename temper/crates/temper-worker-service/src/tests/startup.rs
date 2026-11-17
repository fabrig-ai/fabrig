#![expect(clippy::disallowed_methods, reason = "ordinary tests inspect diagnostic text and edit document fixtures")]
use super::fixtures;
use crate::{settings, startup::What};
use jig_protocol_config::{self as config, Path, Segment};
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
    for (member, value, path, what) in [
        (
            b"application.workers[0].root".as_slice(),
            b"12".as_slice(),
            Path::root().member(b"application").member(b"workers").child(Segment::Index(0)).member(b"root"),
            What::Jig(config::What::Kind),
        ),
        (
            b"application.workers[0].root",
            br#""../escape""#,
            Path::root().member(b"application").member(b"workers").child(Segment::Index(0)).member(b"root"),
            What::Jig(config::What::Relative),
        ),
        (
            b"application.workers[0].git",
            br#""git""#,
            Path::root().member(b"application").member(b"workers").child(Segment::Index(0)).member(b"git"),
            What::Absolute,
        ),
    ] {
        let input = fixtures::with(&input, member, value);
        let found = crate::startup::shape(&input, &config::Directories::working()).unwrap_err();
        assert_eq!(found.problems.len(), 1, "{}: {:?}", core::str::from_utf8(member).unwrap(), found);
        assert_eq!((found.problems.get(0).unwrap().path.clone(), found.problems.get(0).unwrap().what), (path, what));
    }
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
    let mut part = crate::startup::worker_part(&input, &directories).unwrap();
    let owned = include_bytes!("../../../../docs/examples/owned.json");
    let document =
        config::document(owned, &config::DocumentLimits { bytes: 1_048_576, tokens: 1_048_576, depth: 32 }).unwrap();
    let ats = config::read::list(
        &document,
        temper_service_core::startup::member(&document, 0, b"workers").unwrap(),
        document.len(),
    )
    .unwrap();
    let worker = config::worker(&document, *ats.get(0).unwrap(), &directories).unwrap();
    part.worker.jig.limits = worker.limits;
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
