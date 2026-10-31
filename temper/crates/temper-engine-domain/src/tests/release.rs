//! Project release answers cross the journal barrier
//! (temper/docs/design/draft/playground-reset.md, 5).
use super::*;
use temper_engine_domain_forge as forge;

#[test]
fn release_answers_project_released_after_its_commit() {
    let (mut domain, env) = resume_fixture::root();
    let connector = domain.config.forge_connector;
    let mut decision = Decision::decide(&mut domain.journal, &env.limits.journal);
    let mut requests = Queue::uncapped();
    requests
        .push(jig_domain_core::Request::Ask { connector, ask: jig_domain_core::Ask::ReleaseProject { project: 1 } });
    crate::route::route_core_requests(&mut domain, &env, &mut decision, jig_domain_core::Requests::Out(requests));
    let Some(crate::Work::Forge(forge::Event::ReleaseProject { project: 1, request })) = domain.work.pop() else {
        panic!("release reaches the forge");
    };
    let mut child = Queue::uncapped();
    let repository = skein_forge::api::Repository { forge: 1, repository: 2 };
    child.push(forge::Request::Erase { key: forge::Key::Repository(repository) });
    child.push(forge::Request::ProjectReleased { request });
    crate::route::forge_route::outputs(&mut domain, &env, &mut decision, &mut child);
    assert!(domain.work.is_empty(), "core hears nothing before commit");
    accept_routed(&mut domain.journal, &mut domain.core.counters, &env.limits.journal, decision, true).unwrap();
    let Some(Output::Commit { number, writes }) = crate::decision::commit(&mut domain.journal, &env.limits.journal)
    else {
        panic!("release erasures commit");
    };
    assert!(writes.contains(&Write::Erase(Key::Forge(forge::Key::Repository(repository)))));
    let mut out = Queue::uncapped();
    release_ready(&mut domain.journal, &mut out);
    assert!(out.is_empty(), "release waits for durability");
    mark_committed(&mut domain.journal, number);
    release_ready(&mut domain.journal, &mut out);
    let Some(Output::Deliver(delivery)) = out.pop() else {
        panic!("committed release answers");
    };
    assert_eq!(delivery, Delivery::ProjectReleased { connector, project: 1 });
    let crate::Request::ToChild(released) = crate::boundary::delivery_output(delivery) else {
        panic!("release returns to the core");
    };
    crate::try_step(&mut domain, &env, crate::Event::Released(released)).unwrap();
}
