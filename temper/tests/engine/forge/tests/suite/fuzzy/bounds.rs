//! Bounds faults and repair evidence (testing/fakes.md, 2; failure-worlds.md, 4.1).
use skein_fake_forge_domain::{self as fake, api as raw};
use skein_forge::api;
use skein_lib::{Duration, Token};
use skein_world::{
    faults::{Kind, Struck},
    streams::{Stream, Streams},
    sweep::{Exclusion, Sweep},
};
use temper_engine_domain_forge as top;
use temper_engine_domain_forge_change as change;
use temper_engine_forge_world::{World, changes, landings::LANDING, translate};
use temper_faults::forge::{COMPARISON, JOB_LOG, PAST_BOUNDS, PULL_FILES};

fn holding(world: &World, task: u64) -> Vec<&'static str> {
    match world.change(task).change.state {
        change::State::Repairing { why: change::Repair::LandingAt { .. }, .. } => {
            vec!["a landing gate's repair reads the failed job's log"]
        }
        change::State::Repairing { .. } => vec!["a repair reads the pull request's files"],
        _ => panic!("draw outside a repair"),
    }
}

fn left_out(world: &mut World, task: u64, sweep: &Sweep, kind: Kind) -> bool {
    if let Some(exclusion) = sweep.leaves_out(kind, &holding(world, task)) {
        world.leave_out(exclusion.scenario, kind);
        true
    } else {
        false
    }
}

fn struck(world: &World, task: u64) -> Struck {
    let row = world.change(task);
    Struck {
        candidate: Some(
            String::from_utf8(top::commit_text(&row.change.last_head.expect("repair head")).into()).expect("hex"),
        ),
        request: Some(row.pull.expect("repair pull").to_string()),
        ..Struck::default()
    }
}

fn evidence(world: &World, task: u64, kind: Kind, brief: &str, needed: &[&str]) {
    assert!(
        world
            .ledger()
            .striking(world.now(), &struck(world, task))
            .iter()
            .any(|entry| world.ledger().get(*entry).kind == kind),
        "seed 1: evidence checked outside its fault"
    );
    for words in needed {
        assert!(brief.contains(words), "seed 1: repair missing {words:?}: {brief}");
    }
}

fn gate_repair(world: &mut World, sweep: &Sweep) -> u64 {
    let task = world
        .landing(LANDING)
        .landing
        .members
        .iter()
        .find(|member| {
            matches!(
                member.left,
                Some(
                    temper_engine_domain_forge_landing::Leave::Blamed { .. }
                        | temper_engine_domain_forge_landing::Leave::Failed { .. }
                )
            )
        })
        .unwrap_or_else(|| panic!("seed 1 has a red member: {:?}", world.landing(LANDING)))
        .change;
    let change::Decision::Delegate(change::Delegate::Repair(why @ change::Repair::LandingAt { .. })) =
        changes::change_step_task(world, task, 80_000, change::Status::Passed, Box::new([]), false, false)
    else {
        panic!("red member asks a landing repair: {:?}", world.change(task));
    };
    assert!(left_out(world, task, sweep, JOB_LOG));
    assert!(!left_out(world, task, sweep, PULL_FILES));
    let pull = world.change(task).pull.expect("pull");
    let head = world.change(task).change.last_head.expect("head");
    // The workload's tree already changes multiple paths. Lower the fake's bound,
    // not the client's, so even a single requested row cannot list them.
    let old = world.file_bound(1);
    assert_eq!(world.try_inspect(&raw::Read::PullFiles { number: pull, page: 1, limit: 1 }), Err(raw::Error::TooLarge));
    let fault = world.inject(PULL_FILES, struck(world, task));
    world.take_calls();
    let mut brief = String::from_utf8(world.repair_words(task, why).into()).expect("instruction");
    brief.push_str(&String::from_utf8(world.ci_brief(task, head).into()).expect("CI brief"));
    evidence(
        world,
        task,
        PULL_FILES,
        &brief,
        &["Gate: gate", "Run: ", "Candidate: ", "Base: ", "Failed job: test", "Failed step:", "End of its log:"],
    );
    assert!(
        !world.take_calls().iter().any(|call| matches!(call, api::Op::Read(api::Read::PullFiles { .. }))),
        "gate repair does not read files"
    );
    world.stop(fault);
    world.file_bound(old);
    task
}

fn ci_repair(world: &mut World, task: u64) -> api::Commit {
    // After the batch, the scripted root sends this member's replacement through
    // the queue's head-check path. Restore its checking snapshot before asking.
    assert!(world.project_landing(5, change::Way::Queue, Box::new([])));
    let mut row = world.change(task).clone();
    row.change.way = change::Way::Queue;
    world.push_file(&row.branch, b"file", b"still broken");
    let head = translate::commit(world.branch(&row.branch));
    row.change.last_head = Some(head);
    row.change.state = change::State::Checking { head };
    row.pending = None;
    row.delegate = None;
    row.effect = change::EffectResult::None;
    row.ci = None;
    row.asks = Box::new([]);
    row.handed = 0;
    row.previous = None;
    world.event(top::Event::Restore { record: top::Stored::Change(Box::new(row)) });
    world.landing_ci(b"main", true);
    let decision = changes::change_step_task(world, task, 80_001, change::Status::Passed, Box::new([]), false, false);
    assert_eq!(
        decision,
        change::Decision::Delegate(change::Delegate::Repair(change::Repair::Ci)),
        "row {:?}; pull {:?}",
        world.change(task),
        world.inspect(&raw::Read::Pull { number: world.change(task).pull.expect("pull") })
    );
    head
}

fn comparison(world: &mut World, task: u64, head: api::Commit, sweep: &Sweep) {
    assert!(left_out(world, task, sweep, PULL_FILES));
    assert!(!left_out(world, task, sweep, COMPARISON));
    let pull = world.change(task).pull.expect("pull");
    let raw::Answer::Pull(identity) = world.inspect(&raw::Read::Pull { number: pull }) else { panic!("pull identity") };
    let base = translate::commit(world.branch(&identity.base));
    let section = Token::new(980);
    world.take_calls();
    world.take_seen();
    world.event(top::Event::PlanBrief {
        section,
        partner: None,
        source: top::BriefSource::Pull {
            item: top::BriefItem { repository: world.change(task).repository.repository, number: pull },
            head: top::BriefCommit(head),
        },
    });
    world.event(top::Event::GatherPlanned {
        task,
        section,
        parts: 2,
        bytes: 2048,
        ci_budget: 2048,
        deadline: world.now().saturating_add(Duration::from_secs(30)),
    });
    world.run_until_read(&api::Read::Compare { before: base, after: head });
    // File pages have been read. Bound the pending comparison at the fake,
    // without excluding or cutting any preceding read.
    let old = world.file_bound(1);
    let fault = world.inject(COMPARISON, struck(world, task));
    world.run_for(1);
    world.event(top::Event::TakeBrief { section });
    let brief = world
        .seen()
        .iter()
        .find_map(|request| match request {
            top::Request::BriefTaken { section: found, bytes: Some(bytes) } if *found == section => {
                Some(String::from_utf8_lossy(bytes).into_owned())
            }
            _ => None,
        })
        .expect("comparison repair brief");
    evidence(
        world,
        task,
        COMPARISON,
        &brief,
        &[
            "File: file\nBefore:\nfixed\nAfter:\nstill broken",
            "[comparison omitted: past forge.client.answer_bytes (4096)",
        ],
    );
    world.stop(fault);
    world.file_bound(old);
}

fn job_log(world: &mut World, task: u64, head: api::Commit, sweep: &Sweep) {
    assert!(!left_out(world, task, sweep, JOB_LOG));
    fake::set_opening(world.forge_mut(), b"org/repo", b"build", &[b'x'; 512]);
    fake::set_failing(world.forge_mut(), b"org/repo", b"build", raw::Failing::Setup);
    world.tier_log_room(2048, 512);
    let fault = world.inject(JOB_LOG, struck(world, task));
    world.take_calls();
    let brief = String::from_utf8(world.ci_brief(task, head).into()).expect("CI brief");
    let calls = world.take_calls();
    let (attempt, max_bytes, part) = calls
        .iter()
        .find_map(|call| match call {
            api::Op::Read(api::Read::Job { attempt, max_bytes, part }) => Some((*attempt, *max_bytes, *part)),
            _ => None,
        })
        .expect("repair reads failed job");
    let raw::Answer::Job { steps, .. } = world.inspect(&raw::Read::Job { job: attempt.job }) else { panic!("job") };
    let failed = steps.iter().find(|step| step.status == raw::StepStatus::Failure).expect("failed step");
    assert_eq!(part, api::LogPart::End, "repair needs the failed step's tail");
    let cap = usize::try_from(max_bytes).expect("byte bound");
    assert!(failed.log.len() > cap, "the selected log, not an unrelated step, exceeds the read");
    let tail = std::str::from_utf8(&failed.log[failed.log.len() - cap..]).expect("log text");
    evidence(world, task, JOB_LOG, &brief, &["build", "set up build", tail]);
    world.stop(fault);
}

#[test]
fn answers_past_bounds_leave_repairs_their_evidence() {
    let mut sweep = Sweep::new("answers_past_bounds_leave_repairs_their_evidence");
    for kind in PAST_BOUNDS {
        sweep.configure(kind);
    }
    sweep
        .exclude(Exclusion::new(
            "a_gate_repair_names_its_failed_tests",
            "check record",
            JOB_LOG,
            &["a landing gate's repair reads the failed job's log"],
        ))
        .expect("S5");
    sweep
        .exclude(Exclusion::new(
            "a_repair_reads_an_oversized_pull",
            "check record",
            PULL_FILES,
            &["a repair reads the pull request's files"],
        ))
        .expect("S2");
    let streams = Streams::new(1);
    let seed = streams.seed_of(Stream::Workload, "landing_batches");
    eprintln!("bounds seed 1, workload {seed}");
    let world = World::bounds_in(&streams, super::super::landing_batches::workflow(true));
    let (mut world, _) = super::landing_batches::run_repairing_in(world, seed, 160);
    let task = gate_repair(&mut world, &sweep);
    let head = ci_repair(&mut world, task);
    comparison(&mut world, task, head, &sweep);
    job_log(&mut world, task, head, &sweep);
    sweep.ran(world.ledger());
    for name in ["a_gate_repair_names_its_failed_tests", "a_repair_reads_an_oversized_pull"] {
        assert_eq!(sweep.left_out(name), 1);
        eprintln!("{name}: {} left out", sweep.left_out(name));
    }
    sweep.check_fell().expect("each bounds kind fell");
}
