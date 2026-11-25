use temper_one_process_world::{cluster_start, fixtures};
use temper_service::{End, EngineEnd, worker_alone};

#[test]
fn standalone_workers_take_figures_then_link_once_and_drain() {
    let outcome = cluster_start::run(fixtures::SEED);
    assert_eq!(outcome.figures_sent, [1, 1], "one figures opening per position");
    assert_eq!(outcome.places_taken, [1, 1], "both workers completed the take for their one place");
    assert_eq!(outcome.links, [1, 1], "one run link per worker");
    assert_eq!(outcome.engine, Some(End::Engine(EngineEnd::Drained)));
    assert_eq!(outcome.workers, [Some(worker_alone::End::Drained), Some(worker_alone::End::Drained)]);
    assert!(outcome.settled, "no host left pending");
}
