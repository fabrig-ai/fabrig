pub use temper_worker_service_fixtures::*;
fn assigned(repository: bool) -> jig_protocol_worker_link::Request {
    let settings = super::engine();
    let encoding = super::encoding(&settings);
    let mut assignment = super::assignment_with_settings(super::repository(3, b"acme", b"app"), &settings);
    if !repository {
        assignment.workspace.repositories = Box::new([]);
    }
    let values = crate::values::Values::new(2, 8192);
    let jig_protocol_engine_workers::Event::Down { mut down, mut parts, .. } = crate::assignment::down(
        assignment,
        &crate::tests::fixtures::forge_table(&crate::tests::fixtures::forges()),
        &encoding,
        &values,
        skein_lib::Token::new(1),
        NOW,
    )
    .expect("assignment encodes") else {
        panic!("assignment down")
    };
    if repository {
        let jig_domain_host::LinkDown::Assign { assignment } = &mut down else { panic!("assignment") };
        assignment.assignment.save = true;
        let record = temper_protocol_worker_wire::v1::Items::decode(
            &encoding.limits,
            &mut skein_lib::Reader::new(&parts[0].bytes),
        )
        .unwrap();
        let mut fields = record.into_parts();
        let mut repositories = skein_lib::List::with_capacity(1);
        for repository in fields.repositories.into_boxed() {
            let mut fields = repository.into_parts();
            fields.save = Some(b"temper/saved-7".as_slice().into());
            repositories
                .push(temper_protocol_worker_wire::v1::Repository::new(&encoding.limits, fields).unwrap())
                .unwrap();
        }
        fields.repositories = repositories;
        parts[0].bytes = temper_protocol_worker_wire::items(
            &temper_protocol_worker_wire::v1::Items::new(&encoding.limits, fields).unwrap(),
        );
    }
    jig_protocol_worker_link::Request::Down { down, parts }
}

pub fn chat_assignment() -> jig_protocol_worker_link::Request {
    assigned(false)
}

pub fn spawn(worker: &mut crate::Worker) -> (skein_lib::Token, skein_io::kernel::Spawn) {
    worker.iterate(NOW, skein_lib::Wall::EPOCH);
    for _ in 0..worker.io().len() {
        if let Some(skein_io::Request::Spawn { owner, spawn }) = worker.io().pop() {
            return (owner, spawn);
        }
    }
    panic!("decoded start asks for a process");
}
