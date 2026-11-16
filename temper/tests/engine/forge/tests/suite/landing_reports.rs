//! Gate reports at the receipt boundary (domain/landing-search-by-test.md, 2).
use skein_fake_forge_domain::api as raw;
use skein_forge::api;
use skein_lib::Duration;
use temper_engine_domain_forge as top;
use temper_engine_domain_forge_change as change;
use temper_engine_domain_forge_landing as landing;
use temper_engine_domain_forge_tiers::Failures;
use temper_engine_forge_world::{
    REPO, World, changes,
    landings::{self, CHANGE, LANDING, PROJECT},
};

fn world(reports: bool, fault: bool, contents: &[u8], limit: u32) -> World {
    let workflow = raw::Workflow {
        requirements: Box::new([]),
        fails: Box::new([]),
        flaky: Box::new([]),
        report: reports.then(|| raw::Report {
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
        passes: 1000,
        cue: Some(raw::Cue::Tests { path: b"failures".as_slice().into() }),
    };
    let mut world = landings::turn_in(World::batch_turn(31, Some(workflow)));
    world.tier_failures(limit);
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
                    fuzzy: None
                }),
            }]),
        }]),
    ));
    if fault {
        world.gate_policy(landing::Project { gate_retries: 0, ..landing::Project::DEFAULT });
        world.runner_setup_faults(1000);
    }
    world.produce_file(b"temper/51", b"failures", contents);
    world.push_file(b"main", b"base", b"base moved");
    landings::ready_change(&mut world, CHANGE, b"temper/51");
    let row = landings::landing_row(world.change(CHANGE));
    landings::register(&mut world, row);
    world
}

#[test]
fn gate_receipts_read_reports_only_for_failed_workflow_steps() {
    let ids = Failures::Tests(Box::new([b"suite.first".as_slice().into(), b"suite.second".as_slice().into()]));
    for (reports, fault, contents, limit, expected) in [
        (true, false, b"suite.second\nsuite.first\n".as_slice(), 2, Some(ids)),
        (true, false, b"suite.second\nsuite.first\n".as_slice(), 1, Some(Failures::Many)),
        (false, false, b"suite.first\n".as_slice(), 2, None),
        (true, false, b"".as_slice(), 2, None),
        (true, true, b"suite.first\n".as_slice(), 2, None),
    ] {
        let mut world = world(reports, fault, contents, limit);
        let mut terminal = None;
        for entry in 100..180 {
            let decision = landings::step_landing(&mut world, LANDING, entry);
            if matches!(
                decision,
                landing::Decision::Leave { .. } | landing::Decision::Finish | landing::Decision::Hold(_)
            ) {
                terminal = Some(decision);
                break;
            }
        }
        let receipt = world.stored().iter().find_map(|(_, row)| {
            if let top::Stored::Receipt { receipt, .. } = row { Some(receipt.clone()) } else { None }
        });
        let reads = world.take_calls();
        let artifacts: Vec<_> =
            reads.iter().filter(|op| matches!(op, api::Op::Read(api::Read::Artifact { .. }))).collect();
        if fault {
            assert!(matches!(terminal, Some(landing::Decision::Hold(landing::Hold::Faults { .. }))));
            assert!(receipt.is_none(), "a runner fault makes no receipt");
            assert!(artifacts.is_empty(), "a runner fault reads no report");
            assert_eq!(world.change(CHANGE).change.repairs, 0);
        } else if contents.is_empty() {
            assert_eq!(terminal, Some(landing::Decision::Finish));
            assert!(receipt.expect("passed receipt").passed);
            assert!(artifacts.is_empty(), "a passed run reads no report");
        } else {
            let receipt = receipt.expect("failed receipt");
            assert!(!receipt.passed);
            assert_eq!(receipt.failures, expected);
            assert!(matches!(
                terminal,
                Some(landing::Decision::Leave { why: landing::Leave::Failed { gate: 0, .. }, .. })
            ));
            assert_eq!(artifacts.len(), 1, "even an absent artifact is read once");
            let job = reads
                .iter()
                .position(|op| matches!(op, api::Op::Read(api::Read::Job { .. })))
                .expect("failed job read");
            let artifact = reads
                .iter()
                .position(|op| matches!(op, api::Op::Read(api::Read::Artifact { .. })))
                .expect("report read");
            assert!(job < artifact, "failed jobs are read before the report");
            assert!(String::from_utf8_lossy(&receipt.logs).contains("Failed job: test"));
            let candidate = receipt.candidate;
            assert_eq!(
                changes::change_step_task(&mut world, CHANGE, 300, change::Status::Passed, Box::new([]), false, false),
                change::Decision::Delegate(change::Delegate::Repair(change::Repair::LandingAt {
                    gate: 0,
                    candidate,
                    below: None
                }))
            );
        }
    }
}
