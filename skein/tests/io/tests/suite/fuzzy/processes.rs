//! All process-group stories under delayed, reordered kernel completions
//! (`skein/docs/design/testing/sweeps.md`, 4.2).

use std::convert::Infallible;

use skein_io_world::processes::{Story, chaos, run};
use skein_world::sweep::cases;
use skein_world::{Memory, this_test};

#[test]
fn process_groups_settle_over_every_seed() {
    let test = this_test!("process_groups_settle_over_every_seed");
    cases::<_, Infallible, _>(test, 0..200, |seed| {
        for story in [Story::SignalRunning, Story::CloseExited, Story::SignalExited] {
            drop(run(seed, chaos(), story, Memory::Unchecked));
        }
        Ok(())
    })
    .unwrap_or_else(|case| panic!("{case}"));
}
