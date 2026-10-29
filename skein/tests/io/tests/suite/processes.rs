//! io's process-group worlds, with exact replay and per-process heap checking.
use skein_io_world::processes::{Story, chaos, run};
use skein_sim::Config;
use skein_world::Memory;

fn check(story: Story) {
    // Process-group stories peak at 68,216 B. Ceiling adds 25% headroom, rounded up to two significant digits.
    const CEILING: u64 = 86_000;
    for seed in 0..4 {
        for config in [Config::calm(), chaos()] {
            let outcome = run(seed, config, story, Memory::Checked);
            assert!(outcome.peaks().expect("memory checked").iter().all(|peak| *peak > 0 && *peak <= CEILING));
        }
    }
}

#[test]
fn a_descendant_holding_the_pipe_ends_with_a_signal_to_the_group() {
    check(Story::SignalRunning);
}
#[test]
fn closing_a_child_whose_group_still_runs_ends_the_group() {
    check(Story::CloseExited);
}
#[test]
fn a_signal_to_the_group_after_its_leader_exited_reaches_the_group() {
    check(Story::SignalExited);
}
#[test]
fn the_group_story_replays_its_actual_kernel_trace() {
    let first = run(7, chaos(), Story::SignalExited, Memory::Checked);
    let second = run(7, chaos(), Story::SignalExited, Memory::Checked);
    assert_eq!(first.trace, second.trace);
    assert_eq!(first.peaks(), second.peaks());
    assert_ne!(first.trace, run(8, chaos(), Story::SignalExited, Memory::Checked).trace);
}

#[test]
fn a_childs_usage_joins_the_childrens_only_at_its_close() {
    // Child usage and owner storage peak at 2,163 B. Ceiling adds 25% headroom, rounded up to two significant digits.
    const CEILING: u64 = 2_800;
    for seed in 0..4 {
        for config in [Config::calm(), chaos()] {
            let outcome = skein_io_world::usage::run(seed, config, Memory::Checked);
            outcome.procs[0].check(true);
            assert!(outcome.peaks().expect("memory checked").iter().all(|peak| *peak > 0 && *peak <= CEILING));
        }
    }
}
