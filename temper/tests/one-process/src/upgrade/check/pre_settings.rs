//! A refused pre-settings start preserves the deployment directory.
use super::{configured, first_commit, fixtures, image};
use crate::testing::{Disk, USER};
use skein_fake_machine::Machine;

/// A pre-settings deployment is refused on each start and keeps its entire directory.
pub fn pre_settings_refusals_leave_the_whole_directory_unchanged() {
    use jig_domain_core::{CoreRecord, Deployment, Record};
    use jig_protocol_store::{Older, OpenRefusal, codec, keys};
    use skein_fake_machine::Item;
    use temper_service::{EngineEnd, EngineStop};

    let document = fixtures::document();
    let deployment = [7; 16];
    let mut format = Vec::from([0, 2, 0, 2, 0, 6, 0, 9, 0, 14, 0, 1, 0, 2]);
    format.extend_from_slice(&temper_protocol_store::STORE_VERSION.to_be_bytes());
    format.extend_from_slice(&deployment);
    let header = Record::Core(CoreRecord::Deployment(Deployment {
        id: deployment,
        restart_wait_ms: None,
        projects: 0,
        tasks: 0,
        people: 0,
        sign_ins: 0,
        messages: 0,
        runs: 0,
        calls: 0,
        connector_rows: 0,
        applications: 0,
        commits: 1,
    }));
    let limit = document.jig.store.store.key;
    let format_key = keys::encode_own(&keys::Own::Format, limit).expect("format key");
    let header_key = keys::encode(&header.key(), limit).expect("deployment key");
    let view = codec::limits(&document.jig.core).expect("record bounds");
    let header = codec::encode(&header, &view, &[]).expect("deployment row");
    let frame = first_commit(&[(&format_key, &format), (&header_key, &header)]);
    let mut machine = Machine::for_user(USER);
    let root = machine.lay(&[
        Item::directory(b"store").mode(0o700),
        Item::directory(b"secrets").mode(0o700),
        Item::directory(b"transcripts").mode(0o700),
        Item::file(b"store/log-00000000000000000001", &frame).mode(0o600),
        Item::file(b"secrets/log-00000000000000000001", b"").mode(0o600),
    ]);
    machine.close(root);
    let before = image(&mut machine);
    let mut disk = Disk::new(machine);
    let refusal = EngineEnd::Stopped(EngineStop::StoreOpen { why: OpenRefusal::OlderBuild { why: Older::Directory } });
    for attempt in ["first", "second"] {
        let mut service = configured(&document);
        service.adopt_store(disk.root(0), USER).expect("startup adopts the directory");
        for _ in 0..10_000 {
            let now = disk.sim.now();
            temper_service::iterate(&mut service, now, disk.sim.wall());
            if let Some(end) = service.engine_component().end() {
                assert_eq!(end, &refusal, "unexpected startup end");
            }
            if service.wait_state().engine.expect("engine state").store_closed {
                break;
            }
            let pending = service.work_pending(now);
            disk.pass(service.submissions(), pending);
            while let Some(complete) = disk.completions().pop() {
                service.completions().push(complete);
            }
        }
        assert_eq!(service.engine_component().end(), Some(&refusal));
        assert!(service.wait_state().engine.expect("engine state").store_closed, "file drivers drained");
        assert_eq!(image(&mut disk.machine), before, "{attempt} refusal leaves the whole inventory unchanged");
    }
}
