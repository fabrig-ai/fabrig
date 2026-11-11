//! A tier's reports against the independent forge (temper/docs/design/domain/after-landing.md, 12).
use skein_fake_forge_domain::api as raw;
use skein_forge::api;
use skein_lib::Duration;
use temper_engine_domain_forge as top;
use temper_engine_domain_forge_change as change;
use temper_engine_domain_forge_tiers as p;
use temper_engine_forge_world::{REPO, World, translate};

const BASELINE: u64 = 101;
const RANGE: u64 = 102;
const INHERITED: u64 = 103;
const STEPS: u64 = 400; // Four workflows of sixty simulated seconds plus reads and releases.
const FAILING: &[u8] = b"suite.first\nsuite.second\n";
const SETTINGS: p::Settings = p::Settings { retries: 2, suspects: p::Suspects::Bisect };
fn tests() -> Box<[Box<[u8]>]> {
    Box::new([b"suite.first".as_slice().into(), b"suite.second".as_slice().into()])
}
fn failures() -> p::Failures {
    p::Failures::Tests(tests())
}
fn workflow(flaky: Box<[Box<[u8]>]>, reports: bool) -> raw::Workflow {
    raw::Workflow {
        fails: Box::new([]),
        flaky,
        report: reports.then(|| raw::Report {
            artifact: b"report".as_slice().into(),
            format: raw::ReportFormat::Lines,
            deflate: false,
        }),
        select: Some(b"temper_tests".as_slice().into()),
        file: b"browser.yml".as_slice().into(),
        jobs: Box::new([b"browser".as_slice().into()]),
        latency_min: Duration::from_secs(60),
        latency_max: Duration::from_secs(60),
        silent: 0,
        passes: 1000,
        cue: Some(raw::Cue::Tests { path: b"failures".as_slice().into() }),
    }
}
fn world(seed: u64, flaky: Box<[Box<[u8]>]>, reports: bool) -> World {
    configured_world(seed, workflow(flaky, reports))
}
fn configured_world(seed: u64, workflow: raw::Workflow) -> World {
    let mut world = following_world(seed, workflow, top::Follows::Landing);
    attach(&mut world, 1, BASELINE);
    world
}
fn following_world(seed: u64, workflow: raw::Workflow, follows: top::Follows) -> World {
    let mut world = World::tiered(seed, workflow);
    world.adopt();
    world.run_for(10);
    if follows == top::Follows::Goals {
        hold_goal(&mut world);
    }
    assert!(world.project_landing(
        5,
        change::Way::Queue,
        Box::new([top::RepositoryGates {
            repository: REPO,
            gates: Box::new([]),
            tiers: Box::new([top::Tier {
                name: b"browser".as_slice().into(),
                workflow: b"browser.yml".as_slice().into(),
                ignored: Box::new([b"docs".as_slice().into()]),
                artifact: b"report".as_slice().into(),
                format: p::Format::Lines,
                follows,
                span: None,
                description: Box::default(),
            }]),
        }])
    ));
    world
}
fn row(world: &World) -> &top::TierRow {
    let Some(top::Stored::Tier(row)) =
        world.stored().get(&top::Key::Tier { repository: REPO, tier: b"browser".as_slice().into() })
    else {
        panic!("saved tier")
    };
    row
}
fn attach(world: &mut World, number: u64, task: u64) {
    for _ in 0..40 {
        if let Some(top::Stored::Tier(row)) =
            world.stored().get(&top::Key::Tier { repository: REPO, tier: b"browser".as_slice().into() })
            && row.record.run.as_ref().is_some_and(|run| run.number == number)
        {
            break;
        }
        world.run_for(1);
    }
    assert_eq!(row(world).record.run.as_ref().expect("due tier").number, number);
    world.event(top::Event::TierTask { repository: REPO, tier: b"browser".as_slice().into(), number, task });
}
fn step(world: &mut World, task: u64, entry: u64) -> top::TierDecision {
    step_news(world, task, entry, &mut Vec::new())
}
fn keep_news(world: &mut World, news: &mut Vec<top::Request>) {
    for request in world.take_seen() {
        if matches!(request, top::Request::News { news: top::News::Red(_), .. }) {
            news.push(request);
        }
    }
}
fn step_news(world: &mut World, task: u64, entry: u64, news: &mut Vec<top::Request>) -> top::TierDecision {
    keep_news(world, news);
    world.event(top::Event::StepTier { task, entry, heard: top::TierHeard { released: false } });
    world.run_for(1);
    let mut decision = None;
    let mut committed = None;
    for request in world.take_seen() {
        match request {
            top::Request::TierDecision { task: decided, decision: choice, entry, .. } if decided == task => {
                decision = Some(choice);
                committed = entry;
            }
            request @ top::Request::News { news: top::News::Red(_), .. } => news.push(request),
            _ => {}
        }
    }
    if let Some(entry) = committed {
        assert!(world.stored().contains_key(&top::Key::Entry(entry)), "outbox is durable before dispatch");
        world.event(top::Event::Committed { entry });
        world.run_for(1);
    }
    decision.expect("tier step decides")
}
// The final observation is durable before the verification join and release.
// Read the policy's decision at that committed observation.
fn verdict(world: &World) -> Option<p::Verdict> {
    let row = row(world);
    let observation = row.result.as_ref()?;
    let run = row.record.run.as_ref()?;
    let outcome = if observation.passed {
        p::Outcome::Passed
    } else {
        p::Outcome::Failed(observation.failures.clone().expect("failed report"))
    };
    let next = if row.received {
        p::start(
            &row.record,
            run.tip,
            run.key,
            run.landings.clone(),
            Some(if observation.passed {
                p::Receipt::Passed
            } else {
                p::Receipt::Failed(observation.failures.clone().expect("receipt failures"))
            }),
            &SETTINGS,
        )
    } else {
        p::step(&row.record, run, outcome, &SETTINGS)
    };
    if let p::Next::Judged(verdict) = next { Some(verdict) } else { None }
}
fn finish(world: &mut World, task: u64) -> p::Verdict {
    finish_news(world, task, &mut Vec::new())
}
fn finish_news(world: &mut World, task: u64, news: &mut Vec<top::Request>) -> p::Verdict {
    let mut judged = None;
    for at in 0..STEPS {
        if let Some(verdict) = verdict(world) {
            judged = Some(verdict);
        }
        let decision = step_news(world, task, task * 1000 + at, news);
        if let Some(verdict) = verdict(world) {
            judged = Some(verdict);
        }
        match decision {
            top::TierDecision::Done { .. } => return judged.expect("durable final observation"),
            top::TierDecision::Held { words } => panic!("tier held: {}", String::from_utf8_lossy(&words)),
            top::TierDecision::Wait { .. } => {}
        }
    }
    panic!("tier did not finish: {:?}", row(world));
}
fn start_baseline(world: &mut World) {
    step(world, BASELINE, 1000);
    step(world, BASELINE, 1001);
}
fn push(world: &mut World, path: &[u8], content: &[u8]) -> api::Commit {
    push_on(world, b"main", path, content)
}
fn push_on(world: &mut World, branch: &[u8], path: &[u8], content: &[u8]) -> api::Commit {
    world.push_file(branch, path, content);
    let commit = translate::commit(world.branch(branch));
    for _ in 0..80 {
        if world
            .stored()
            .iter()
            .any(|(_, stored)| matches!(stored, top::Stored::BranchHead(head) if head.commit == commit))
        {
            return commit;
        }
        world.run_for(1);
    }
    panic!("followed branch did not observe its push")
}
fn selections(calls: &[api::Op]) -> Vec<Option<Box<[u8]>>> {
    calls
        .iter()
        .filter_map(|op| match op {
            api::Op::Write(api::Write::Dispatch { inputs, .. }) => Some(
                inputs.iter().find(|input| input.name.as_ref() == b"temper_tests").map(|input| input.value.clone()),
            ),
            api::Op::Read(_) | api::Op::Write(_) => None,
        })
        .collect()
}
fn receipts(world: &World) -> Vec<&top::Receipt> {
    world
        .stored()
        .iter()
        .filter_map(|(_, stored)| {
            if let top::Stored::Receipt { key, receipt } = stored
                && key.check == top::Check::Tier(b"browser".as_slice().into())
            {
                Some(receipt)
            } else {
                None
            }
        })
        .collect()
}
#[test]
fn red_over_three_landings_skips_the_unchanged_one() {
    let mut world = world(11, Box::new([]), true);
    start_baseline(&mut world);
    let first = push(&mut world, b"docs/guide", b"guide");
    let second = push(&mut world, b"failures", FAILING);
    let third = push(&mut world, b"file", b"source");
    finish(&mut world, BASELINE);
    attach(&mut world, 2, RANGE);
    world.take_calls();
    let verdict = finish(&mut world, RANGE);
    assert_eq!(verdict.landings.as_ref(), &[p::Blame::Cleared, p::Blame::Owns(failures()), p::Blame::Cleared]);
    assert_eq!(verdict.tip, third);
    let calls = world.take_calls();
    let probes: Vec<_> = calls
        .iter()
        .filter_map(|op| match op {
            api::Op::Write(api::Write::CreateBranch { branch, commit })
                if branch.starts_with(b"temper/tier/browser.2.") =>
            {
                Some((branch.clone(), *commit))
            }
            api::Op::Read(_) | api::Op::Write(_) => None,
        })
        .collect();
    assert_eq!(probes, [(Box::from(b"temper/tier/browser.2.1".as_slice()), second)]);
    assert_ne!(first, second);
    assert_eq!(selections(&calls), [None, Some(FAILING.into()), Some(FAILING.into())]);
    assert!(!world.has_branch(b"temper/tier/browser.2.1"));
    assert!(!world.has_branch(b"temper/tier/browser.2"));
    assert_eq!(receipts(&world).len(), 2);
    assert_eq!(row(&world).record.failing, failures());
}
#[test]
fn inherited_failures_clear_the_landing_after() {
    let mut world = world(12, Box::new([]), true);
    start_baseline(&mut world);
    push(&mut world, b"failures", FAILING);
    finish(&mut world, BASELINE);
    attach(&mut world, 2, RANGE);
    let red = finish(&mut world, RANGE);
    assert_eq!(red.landings.as_ref(), &[p::Blame::Owns(failures())]);
    push(&mut world, b"file", b"next source");
    attach(&mut world, 3, INHERITED);
    let inherited = finish(&mut world, INHERITED);
    assert_eq!(inherited.landings.as_ref(), &[p::Blame::Cleared]);
    assert_eq!(inherited.failing, failures());
}
#[test]
fn a_selected_flake_clears_the_range_and_is_counted() {
    let flaky = b"suite.flaky".as_slice();
    let mut world = world(13, Box::new([flaky.into()]), true);
    start_baseline(&mut world);
    push(&mut world, b"file", b"source");
    finish(&mut world, BASELINE);
    attach(&mut world, 2, RANGE);
    world.take_calls();
    let verdict = finish(&mut world, RANGE);
    assert_eq!(verdict.landings.as_ref(), &[p::Blame::Cleared]);
    assert_eq!(row(&world).record.flakes.as_ref(), &[p::Flake { test: flaky.into(), count: 2 }]);
    assert_eq!(selections(&world.take_calls()), [None, Some(b"suite.flaky\n".as_slice().into())]);
    assert_eq!(receipts(&world).len(), 2);
    assert!(receipts(&world).iter().all(|receipt| !receipt.passed));
}
#[test]
fn many_failures_probe_without_a_selection() {
    let mut world = world(14, Box::new([]), true);
    world.tier_failures(1);
    start_baseline(&mut world);
    push(&mut world, b"file", b"first source");
    push(&mut world, b"failures", FAILING);
    push(&mut world, b"file", b"last source");
    finish(&mut world, BASELINE);
    attach(&mut world, 2, RANGE);
    world.take_calls();
    let verdict = finish(&mut world, RANGE);
    assert_eq!(verdict.landings.as_ref(), &[p::Blame::Cleared, p::Blame::Owns(p::Failures::Many), p::Blame::Cleared]);
    assert_eq!(selections(&world.take_calls()), [None, None, None]);
    assert_eq!(row(&world).record.failing, p::Failures::Many);
}
#[test]
fn a_lost_runner_is_retried_then_green() {
    let mut world = world(15, Box::new([]), true);
    world.runner_lost(1000);
    start_baseline(&mut world);
    for at in 0..STEPS {
        step(&mut world, BASELINE, 2000 + at);
        if row(&world).record.run.as_ref().expect("faulting run").faults == 1 {
            break;
        }
    }
    assert_eq!(row(&world).record.run.as_ref().expect("faulting run").faults, 1);
    world.runner_lost(0);
    let verdict = finish(&mut world, BASELINE);
    assert_eq!(verdict.failing, p::Failures::Tests(Box::new([])));
    assert!(world.take_calls().iter().any(|op| matches!(op, api::Op::Write(api::Write::Rerun { .. }))));
    assert!(receipts(&world).iter().all(|receipt| receipt.passed));
}
#[test]
fn an_absent_failed_report_holds_as_a_runner_fault() {
    let mut world = world(16, Box::new([]), false);
    start_baseline(&mut world);
    push(&mut world, b"failures", FAILING);
    finish(&mut world, BASELINE);
    attach(&mut world, 2, RANGE);
    let mut held = false;
    for at in 0..STEPS {
        if matches!(step(&mut world, RANGE, 2000 + at), top::TierDecision::Held { .. }) {
            held = true;
            break;
        }
    }
    assert!(held);
    assert_eq!(row(&world).record.run.as_ref().expect("held run").faults, 3);
    assert!(row(&world).result.is_none());
    assert_eq!(receipts(&world).len(), 1);
    assert!(
        world
            .take_calls()
            .iter()
            .any(|op| matches!(op, api::Op::Read(api::Read::Artifact { name, .. }) if name.as_ref() == b"report"))
    );
}
#[test]
fn a_push_to_another_branch_starts_no_run() {
    let mut world = world(17, Box::new([]), true);
    finish(&mut world, BASELINE);
    world.take_calls();
    world.produce_on(b"temper/other", b"main");
    world.run_for(20);
    assert!(row(&world).record.run.is_none());
    assert_eq!(row(&world).record.sequence, 1);
    assert!(selections(&world.take_calls()).is_empty());
}
fn restarted_part(seed: u64, wanted: u8) {
    let mut world = world(seed, Box::new([]), true);
    start_baseline(&mut world);
    push(&mut world, b"docs/guide", b"guide");
    push(&mut world, b"failures", FAILING);
    push(&mut world, b"file", b"source");
    finish(&mut world, BASELINE);
    attach(&mut world, 2, RANGE);
    world.take_calls();
    let mut reached = false;
    for at in 0..STEPS {
        step(&mut world, RANGE, 2000 + at);
        let row = row(&world);
        let part = match row.record.run.as_ref().expect("live range").part {
            p::Part::Whole => 0,
            p::Part::Rerun { .. } => 1,
            p::Part::Probe { .. } => 2,
        };
        if part == wanted
            && row
                .flight
                .as_ref()
                .is_some_and(|flight| flight.start.is_some() && flight.run.is_some() && flight.report.is_none())
        {
            reached = true;
            break;
        }
    }
    assert!(reached);
    world.restart();
    world.run_for(1);
    let verdict = finish(&mut world, RANGE);
    assert_eq!(verdict.landings.as_ref(), &[p::Blame::Cleared, p::Blame::Owns(failures()), p::Blame::Cleared]);
    assert_eq!(selections(&world.take_calls()), [None, Some(FAILING.into()), Some(FAILING.into())]);
    assert!(!world.has_branch(b"temper/tier/browser.2.1"));
}
#[test]
fn a_selected_rerun_survives_a_cold_restart() {
    restarted_part(18, 1);
}
#[test]
fn a_selected_probe_survives_a_cold_restart() {
    restarted_part(19, 2);
}

#[test]
fn baseline_failures_are_inherited_by_the_first_range() {
    let mut configured = workflow(Box::new([]), true);
    configured.fails = tests();
    configured.cue = None;
    let mut world = configured_world(20, configured);
    let baseline = finish(&mut world, BASELINE);
    assert!(baseline.landings.is_empty());
    assert_eq!(baseline.failing, failures());
    push(&mut world, b"file", b"source");
    attach(&mut world, 2, RANGE);
    let inherited = finish(&mut world, RANGE);
    assert_eq!(inherited.landings.as_ref(), &[p::Blame::Cleared]);
    assert_eq!(inherited.failing, failures());
}

const GOAL: u64 = 201;
const CHANGE: u64 = 202;
const SUBSCRIPTION: u64 = 203;
// The fake's failed workflow step writes four numbered lines.
const LOG: &[u8] =
    b"failing check browser 0\nfailing check browser 1\nfailing check browser 2\nfailing check browser 3\n";

fn goal_name() -> top::Name {
    top::Name {
        forge: REPO.forge,
        repository: REPO.repository,
        what: top::What::Branch(b"temper/1/g201".split(|byte| *byte == b'/').map(Box::from).collect()),
    }
}
fn hold_goal(world: &mut World) {
    world.produce_on(b"temper/1/g201", b"main");
    world.event(top::Event::Hold { entry: 0, task: GOAL, resource: goal_name(), from: None });
}
fn goal_push(world: &mut World, path: &[u8], content: &[u8]) -> api::Commit {
    let name = goal_name();
    world.event(top::Event::Claim {
        designs: Box::new([]),
        task: GOAL,
        attempt: 1,
        writes: Box::new([name.clone()]),
        holders: Box::new([]),
    });
    world.push_file(b"temper/1/g201", path, content);
    let commit = translate::commit(world.branch(b"temper/1/g201"));
    world.event(top::Event::Answered { task: GOAL, attempt: 1, pushed: Box::new([(name, commit)]) });
    // The restored tier supplies the tracked tip; the forge supplies runs and logs.
    let mut saved = row(world).clone();
    saved.goals[0].tip = Some(commit);
    world.restore_tier(saved);
    world.run_for(10);
    commit
}
fn subscribe_red(world: &mut World, branch: Box<[u8]>) {
    world.event(top::Event::Subscribe {
        subscription: top::Subscriber {
            task: GOAL,
            number: SUBSCRIPTION,
            topic: top::Topic::Landings { repository: REPO, branch },
            own_change: None,
            goal_tasks: Some(Box::new([CHANGE])),
            paths: Box::new([]),
        },
    });
}
fn red_range(follows: &top::Follows) -> (World, api::Commit, Box<[u8]>) {
    let mut world = following_world(21, workflow(Box::new([]), true), follows.clone());
    let branch: Box<[u8]> = if *follows == top::Follows::Goals {
        world.run_for(10);
        world.restart();
        world.run_for(10);
        goal_push(&mut world, b"file", b"baseline");
        b"temper/1/g201".as_slice().into()
    } else {
        b"main".as_slice().into()
    };
    attach(&mut world, 1, BASELINE);
    subscribe_red(&mut world, branch.clone());
    start_baseline(&mut world);
    let commit = if *follows == top::Follows::Goals {
        assert_eq!(finish(&mut world, BASELINE).failing, p::Failures::Tests(Box::new([])));
        goal_push(&mut world, b"failures", b"suite.first\n")
    } else {
        push_on(&mut world, &branch, b"failures", b"suite.first\n")
    };
    world.event(top::Event::Restore {
        record: top::Stored::Verification(top::Verification {
            repository: REPO,
            commit,
            changes: Box::new([CHANGE]),
            tiers: Box::new([top::TierVerification {
                tier: b"browser".as_slice().into(),
                verdict: None,
                repaired: None,
            }]),
        }),
    });
    if *follows != top::Follows::Goals {
        assert_eq!(finish(&mut world, BASELINE).failing, p::Failures::Tests(Box::new([])));
    }
    attach(&mut world, 2, RANGE);
    world.take_calls();
    (world, commit, branch)
}
fn red_news(world: &mut World) -> (Vec<top::Request>, Vec<api::Op>) {
    let mut news = Vec::new();
    let verdict = finish_news(world, RANGE, &mut news);
    keep_news(world, &mut news);
    assert_eq!(verdict.failing, p::Failures::Tests(Box::new([b"suite.first".as_slice().into()])));
    let calls = world.take_calls();
    assert_eq!(selections(&calls), [None, Some(b"suite.first\n".as_slice().into())]);
    (news, calls)
}
fn assert_red_news(
    news: &[top::Request],
    commit: api::Commit,
    branch: Box<[u8]>,
    follows: &top::Follows,
    end: Option<Box<[u8]>>,
) {
    let failing = p::Failures::Tests(Box::new([b"suite.first".as_slice().into()]));
    let on = if *follows == top::Follows::Goals {
        top::RedOn::Goal { branch: branch.clone(), tip: commit, new: failing, still: p::Failures::Tests(Box::new([])) }
    } else {
        top::RedOn::Landing { commit, branch: branch.clone(), failing }
    };
    let [top::Request::News { news: top::News::Red(red), .. }] = news else { panic!("one confirmed red: {news:?}") };
    assert!(red.url.as_ref().expect("run URL").starts_with(b"http://forge/org/repo/actions/runs/"));
    assert_eq!(
        news,
        [top::Request::News {
            task: GOAL,
            subscription: SUBSCRIPTION,
            topic: top::Topic::Landings { repository: REPO, branch },
            news: top::News::Red(top::RedNews {
                tier: b"browser".as_slice().into(),
                run: 2,
                url: red.url.clone(),
                on,
                logs: Box::new([top::JobLog { name: b"browser".as_slice().into(), end }]),
            }),
            class: top::Class::Wakes,
        }]
    );
}
fn confirmed_red(follows: &top::Follows, brief_bytes: u32, answer_bytes: u32) {
    let (mut world, commit, branch) = red_range(follows);
    world.tier_log_room(brief_bytes, answer_bytes);
    let (news, calls) = red_news(&mut world);
    let cap = usize::try_from(brief_bytes.min(answer_bytes)).expect("log room fits usize");
    let end = &LOG[LOG.len().saturating_sub(cap)..];
    assert_red_news(&news, commit, branch, follows, Some(end.into()));
    let reads: Vec<_> = calls
        .iter()
        .filter_map(|op| if let api::Op::Read(api::Read::Job { max_bytes, .. }) = op { Some(*max_bytes) } else { None })
        .collect();
    assert_eq!(reads, [brief_bytes.min(answer_bytes)]);
}
#[test]
fn a_confirmed_red_tells_its_goal_the_failed_jobs_end() {
    confirmed_red(&top::Follows::Landing, 4096, 4096);
}
#[test]
fn red_news_keeps_job_logs_when_brief_exceeds_client_answer_room() {
    const STANDARD_BRIEF: u32 = 131_584;
    const STANDARD_ANSWER: u32 = 65_536;
    const TAIL: u32 = 32;
    for follows in [top::Follows::Landing, top::Follows::Goals] {
        confirmed_red(&follows, STANDARD_BRIEF, STANDARD_ANSWER);
        confirmed_red(&follows, TAIL, 4096);
    }
}
