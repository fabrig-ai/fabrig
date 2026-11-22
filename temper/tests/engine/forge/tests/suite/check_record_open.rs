//! Open check-record scenarios (draft/failure-worlds.md, 6–7; testing/fakes.md, 2).
use skein_fake_forge_domain::api as raw;
use skein_forge::api;
use skein_lib::Duration;
use temper_engine_domain_forge as top;
use temper_engine_domain_forge_change as change;
use temper_engine_domain_forge_landing as landing;
use temper_engine_forge_world::{
    REPO, World, changes,
    landings::{self, CHANGE, LANDING, PROJECT},
    translate,
};

const BRANCH: &[u8] = b"temper/51";

#[test]
#[ignore = "check record: one structured record of what a check found"]
fn a_gate_repair_names_its_failed_tests() {
    // Dispatched jobs cannot use scenario::set_opening: it sets commit-CI contexts only.
    // A one-byte workflow name leaves 60 bytes for its longest admitted job name.
    // Its failed step's log is 308 bytes, just past the client's 300-byte answer.
    let job = vec![b'x'; 60];
    let workflow = raw::Workflow {
        requirements: Box::new([]),
        fails: Box::new([]),
        flaky: Box::new([]),
        report: Some(raw::Report {
            artifact: b"failed".as_slice().into(),
            format: raw::ReportFormat::Lines,
            deflate: false,
        }),
        select: None,
        file: b"g".as_slice().into(),
        jobs: Box::new([job.into_boxed_slice()]),
        latency_min: Duration::from_secs(2),
        latency_max: Duration::from_secs(2),
        silent: 0,
        passes: 1000,
        cue: Some(raw::Cue::Tests { path: b"failures".as_slice().into() }),
    };
    let mut world = landings::turn_in(World::batch_turn(31, Some(workflow)));
    assert!(world.project_landing(
        PROJECT,
        change::Way::Turn,
        Box::new([top::RepositoryGates {
            repository: REPO,
            tiers: Box::new([]),
            gates: Box::new([landing::Gate {
                name: b"gate".as_slice().into(),
                workflow: b"g".as_slice().into(),
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
    let tests = ["test suite report_first", "test suite report_second"];
    world.produce_file(BRANCH, b"failures", tests.join("\n").as_bytes());
    world.push_file(b"main", b"base", b"base moved");
    landings::ready_change(&mut world, CHANGE, BRANCH);
    let row = landings::landing_row(world.change(CHANGE));
    landings::register(&mut world, row);
    // The client's answer bound, not the larger brief allowance, limits this log read.
    world.tier_log_room(4096, 300);
    world.take_calls();
    let mut failed = None;
    for entry in 100..180 {
        if let landing::Decision::Leave { why: landing::Leave::Failed { gate, .. }, .. } =
            landings::step_landing(&mut world, LANDING, entry)
        {
            failed = Some(gate);
            break;
        }
    }
    let gate = failed.expect("a failed workflow step with a report leaves for repair");
    let calls = world.take_calls();
    let (attempt, max_bytes) = calls
        .iter()
        .find_map(|call| match call {
            api::Op::Read(api::Read::Job { attempt, max_bytes, .. }) => Some((*attempt, *max_bytes)),
            _ => None,
        })
        .expect("landing reads the failed job");
    assert_eq!(max_bytes, 300, "the read respects the client answer bound");
    let raw::Answer::Job { steps, .. } = world.inspect(&raw::Read::Job { job: attempt.job }) else {
        panic!("failed job's independent forge view")
    };
    let step = steps.iter().find(|step| step.status == raw::StepStatus::Failure).expect("workflow step failed");
    assert!(step.log.len() > usize::try_from(max_bytes).expect("bounded read"));
    assert!(tests.iter().all(|test| !String::from_utf8_lossy(&step.log).contains(test)), "only the report names tests");
    assert!(calls.iter().any(|call| matches!(call, api::Op::Read(api::Read::Artifact { .. }))), "report read");

    let why = change::Repair::LandingAt { gate, candidate: attempt.head, below: None };
    assert_eq!(
        changes::change_step_task(&mut world, CHANGE, 300, change::Status::Passed, Box::new([]), false, false),
        change::Decision::Delegate(change::Delegate::Repair(why)),
    );
    let head = translate::commit(world.branch(BRANCH));
    let mut brief = world.repair_words(CHANGE, why).into_vec();
    brief.push(b'\n');
    brief.extend_from_slice(&world.ci_brief(CHANGE, head));
    let brief = String::from_utf8(brief).expect("repair brief text");
    for test in tests {
        assert!(brief.contains(&format!("Failed test: {test} (focused)")), "repair brief must name {test}: {brief}");
    }
}

#[test]
#[ignore = "check record: one structured record of what a check found"]
fn a_repair_reads_an_oversized_pull() {
    let mut world = World::new(37);
    world.adopt();
    let before = ["old-a", "old-b"];
    let after = ["new-a", "new-b", "new-c"];
    let tree = |paths: &[&str]| {
        paths
            .iter()
            .map(|path| raw::File { path: path.as_bytes().into(), content: b"contents\n".as_slice().into() })
            .collect()
    };
    world.push_tree(b"main", tree(&before));
    world.produce_on(BRANCH, b"main");
    world.push_tree(BRANCH, tree(&after));
    world.status(BRANCH, raw::Check::Failed);
    let pull = world.open_pull(BRANCH, b"main");
    let head = translate::commit(world.branch(BRANCH));
    // Both trees fit four files; their five changed paths do not. Even page one is refused.
    assert_eq!(world.try_inspect(&raw::Read::PullFiles { number: pull, page: 1, limit: 1 }), Err(raw::Error::TooLarge),);
    let mut row = changes::change_row();
    row.pull = Some(pull);
    row.change.last_head = Some(head);
    row.change.state = change::State::Checking { head };
    world.event(top::Event::Change { row });
    assert_eq!(
        changes::change_step(&mut world, 1, change::Status::Passed, Box::new([])),
        change::Decision::Delegate(change::Delegate::Repair(change::Repair::Ci)),
    );
    world.take_calls();
    let _reason = world.ci_brief(CHANGE, head);
    let brief = String::from_utf8(world.pull_brief(CHANGE, head).into_vec()).expect("repair pull brief");
    let calls = world.take_calls();
    assert!(
        calls.iter().any(|call| matches!(call,
            api::Op::Read(api::Read::PullFiles { number, head: read_head, .. }) if *number == pull && *read_head == head
        )),
        "the repair reads its pinned pull's files"
    );
    for path in before.into_iter().chain(after) {
        let section = brief
            .split_once(&format!("File: {path}\n"))
            .map(|(_, rest)| rest.split("\nFile: ").next().expect("first file section"));
        let (old, new) = if before.contains(&path) { ("contents\n", "") } else { ("", "contents\n") };
        let supplied = section.is_some_and(|text| text.contains(&format!("Before:\n{old}\nAfter:\n{new}\n")));
        let unavailable = ["omitted", "unread", "could not", "not read", "not carried", "unavailable"];
        let named_unread = section.is_some_and(|text| unavailable.iter().any(|words| text.contains(words)))
            || brief.lines().any(|line| line.contains(path) && unavailable.iter().any(|words| line.contains(words)));
        assert!(supplied || named_unread, "repair must get {path} or name it as unread: {brief}");
    }
}
