mod support;
use skein_lib::Duration;
#[test]
fn random_cuts_and_stdout_delays_lose_no_command_or_change() {
    for seed in 1..=16 {
        let mut world = support::World::random(seed);
        world.run_until(Duration::from_secs(10));
        assert_eq!(world.client.acts(), world.typed(), "seed {seed}: each command acted once, in order");
        assert!(world.shown_last(), "seed {seed}: last change printed");
    }
}
