//! Story A's native high water (temper/docs/design/playground.md, 8 and 11).
use temper_one_process_world::{fixtures, world};

#[global_allocator]
static HEAP: skein_heap::Counting = skein_heap::Counting;

#[test]
#[ignore = "a service world's observer has no way to keep its storage outside the process's span (skein/docs/design/testing/README.md, 5): process 0's exact release check fails (playground.status.md, 9)"]
fn story_a_at_its_high_water() {
    let mut settings = fixtures::calm(fixtures::SEED);
    settings.config.trace = skein_sim::Trace::Off;
    settings.memory = skein_world::Memory::Checked;
    let outcome = world::run(settings, world::Story::PlaygroundA);
    let peaks = outcome.peaks.expect("checked memory reports process peaks");
    println!("story A high water; process index: name | peak bytes");
    for (at, (name, peak)) in outcome.process_names.iter().zip(&peaks).enumerate() {
        println!("{at}: {name} | {peak}");
    }
    assert!(peaks.iter().any(|peak| *peak > 0), "the allocator measured heap");
}
