//! Open acceptance scenarios (design/testing/engine-checks.md, 5.1).
mod p124;
mod p37;
pub(super) use p37::check_starved_gate;
mod p67;
mod p88;
mod p90;

use skein_fake_forge_domain::api as raw;
use skein_lib::Duration;
use temper_engine_domain_forge as top;
use temper_engine_domain_forge_change as change;
use temper_engine_domain_forge_landing as landing;
use temper_engine_forge_world::{
    REPO, World,
    landings::{self, CHANGE, PROJECT},
};

/// A failed workflow step whose uploaded report is independent of its exit status.
pub(super) fn reported_gate(seed: u64, failing_tests: &[&[u8]], blame: bool) -> World {
    let workflow = raw::Workflow {
        requirements: Box::new([]),
        fails: failing_tests.iter().map(|test| Box::from(*test)).collect(),
        flaky: Box::new([]),
        report: Some(raw::Report {
            artifact: b"failed".as_slice().into(),
            format: raw::ReportFormat::Lines,
            deflate: false,
        }),
        select: None,
        file: b"gate.yml".as_slice().into(),
        jobs: Box::new([b"test".as_slice().into()]),
        latency_min: Duration::from_secs(2),
        latency_max: Duration::from_secs(2),
        silent: 0,
        passes: 0,
        cue: None,
    };
    let mut world = World::batch_turn(seed, Some(workflow));
    if blame {
        world.watch_blame();
    }
    let mut world = landings::turn_in(world);
    assert!(world.project_landing(
        PROJECT,
        change::Way::Turn,
        Box::new([top::RepositoryGates {
            repository: REPO,
            tiers: Box::new([]),
            gates: Box::new([landing::Gate {
                name: b"gate".as_slice().into(),
                workflow: b"gate.yml".as_slice().into(),
                ignored: Box::new([]),
                blocking: true,
                base: false,
                report: Some(landing::GateReport {
                    artifact: b"failed".as_slice().into(),
                    format: landing::Format::Lines,
                    fuzzy: None,
                }),
            }]),
        }]),
    ));
    world.produce_file(b"temper/51", b"file", b"broken");
    world.push_file(b"main", b"base", b"base moved");
    landings::ready_change(&mut world, CHANGE, b"temper/51");
    let row = landings::landing_row(world.change(CHANGE));
    landings::register(&mut world, row);
    world
}

fn words_name(words: &[u8], names: &[&str]) {
    let words = String::from_utf8_lossy(words).to_lowercase();
    for name in names {
        assert!(words.contains(name), "missing {name:?}: {words}");
    }
}
