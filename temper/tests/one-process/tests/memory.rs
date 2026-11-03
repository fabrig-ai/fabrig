//! Story A's native high water (temper/docs/design/playground.md, 8 and 11).
use temper_one_process_world::{fixtures, world};

#[global_allocator]
static HEAP: skein_heap::Counting = skein_heap::Counting;

#[test]
#[ignore = "a measurement, run by hand: temper/docs/development/playground-memory.md"]
fn story_a_at_its_high_water() {
    let mut settings = fixtures::calm(fixtures::SEED);
    settings.config.trace = skein_sim::Trace::Off;
    settings.memory = skein_world::Memory::Checked;
    let outcome = world::run(settings, world::Story::PlaygroundA);
    let peaks = outcome.peaks.expect("checked memory reports process peaks");
    println!("story A high water; process index | peak bytes");
    for (at, peak) in peaks.iter().enumerate() {
        println!("{at} | {peak}");
    }
    assert!(peaks.iter().any(|peak| *peak > 0), "the allocator measured heap");
}
