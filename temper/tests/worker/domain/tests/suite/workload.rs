//! The workload draw is independent of its fault stream.
use skein_world::streams::Streams;
use temper_worker_domain_world::whole::{
    Case,
    workload::{self, Run, Workload},
};

#[test]
fn worker_workload_is_unchanged_when_faults_are_drawn_first() {
    let streams = Streams::new(0);
    let plain = workload::draw(&streams);
    let faults_first = Streams::new(0);
    let _park = workload::draw_park(&faults_first, &plain);
    let after = workload::draw(&faults_first);
    assert_eq!(plain, after);
    assert_eq!(plain, Workload { runs: vec![Run { story: Case::Delivered, start: 2, checks: 2 }] });
    workload::run(0, false).expect("fault-free workload ends as drawn");
}
