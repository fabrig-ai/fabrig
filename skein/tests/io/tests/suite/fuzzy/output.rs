//! Native output settlement under the simulated kernel's reordered completions
//! (`skein/docs/design/testing/sweeps.md`, 4.2).

use std::convert::Infallible;

use skein_io_world::output::conversation;
use skein_lib::Duration;
use skein_sim::{Config, Faults};
use skein_world::sweep::cases;
use skein_world::this_test;

#[test]
fn independent_output_and_unanswered_reads_progress_across_reordered_seeds() {
    let config = Config {
        buffer: 1,
        faults: Faults {
            latency: 700,
            latency_max: Duration::from_millis(3),
            short_send: 700,
            short_recv: 700,
            ..Faults::NONE
        },
        ..Config::calm()
    };
    let test = this_test!("independent_output_and_unanswered_reads_progress_across_reordered_seeds");
    cases::<_, Infallible, _>(test, 0..64, |seed| {
        let first = conversation(seed, config);
        assert_eq!(first, conversation(seed, config), "native seed {seed} replays");
        Ok(())
    })
    .unwrap_or_else(|case| panic!("{case}"));
}
