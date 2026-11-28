//! Joined closes and aborts across moves under small intakes
//! (`skein/docs/design/runtime/io/README.md`, 5; `skein/docs/design/testing/sweeps.md`, 4.2).

use std::convert::Infallible;

use skein_io_world::joined::random;
use skein_sim::Config;
use skein_world::sweep::cases;
use skein_world::this_test;

#[test]
fn joined_closes_and_aborts_hold_over_many_seeds() {
    let test = this_test!("joined_closes_and_aborts_hold_over_many_seeds");
    cases::<_, Infallible, _>(test, 0..150, |seed| {
        let outcome = random(seed, Config::chaos()).run();
        assert!(outcome.trace.is_empty(), "seed {seed}: joined pairs never reach the simulator");
        Ok(())
    })
    .unwrap_or_else(|case| panic!("{case}"));
}
