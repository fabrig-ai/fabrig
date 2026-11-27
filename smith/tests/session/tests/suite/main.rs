//! Session integration tests.

#[global_allocator]
static HEAP: skein_world::domain::heap::Counting = skein_world::domain::heap::Counting;

mod budget;
mod child_final;
mod compaction;
mod empty_turn;
mod jobs;
mod memory;
mod orders;
mod paths;
mod recorded;
mod referee;
mod shapes;
mod simulation;
mod wind_down;
mod world_jobs;

mod fuzzy {
    mod recorded;
    mod simulation;

    #[test]
    fn edge_seeds_are_valid_cases() {
        let mut sample = smith_session_world::shapes::Sample::new("edge_seeds_are_valid_cases");
        let mut decodings = smith_session_world::translate::Decodings::default();
        for seed in [u64::MAX, 1 << 63] {
            let (world, _) = simulation::random_case(seed, &mut decodings);
            sample.ran(&world.reached()).expect("the case holds only shapes in form");
        }
    }
}
mod world_paths;
