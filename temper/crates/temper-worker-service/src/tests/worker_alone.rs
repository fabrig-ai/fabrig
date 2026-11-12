use super::fixtures;
#[test]
fn a_refused_git_start_stops_and_aborts_the_workers_service() {
    let mut service = super::worker::alone_service();
    let mut stand = fixtures::WorkerStand::new();
    stand.worker = service.worker;
    stand.below.script.invocations.front_mut().unwrap().transcript.stdout = b"git version 2.40.0\n".as_slice().into();
    for _ in 0_u16..1000 {
        if !stand.turn() {
            break;
        }
    }
    let problems = stand.worker.refused().expect("git's version is refused").clone();
    service.worker = stand.worker;
    crate::worker_alone::iterate_stage(&mut service, crate::Stage::Worker, fixtures::NOW, skein_lib::Wall::EPOCH);
    assert_eq!(service.end(), Some(&crate::worker_alone::End::Stopped(crate::worker_alone::Stop::Start(problems))));
    assert!(service.worker.aborted.is_some(), "refused startup aborts every worker child");
}

