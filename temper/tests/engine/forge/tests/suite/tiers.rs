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
    let mut world = World::tiered(seed, workflow);
    world.adopt();
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
                follows: top::Follows::Landing,
                span: None,
            }]),
        }])
    ));
    attach(&mut world, 1, BASELINE);
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
#[expect(
    clippy::wildcard_enum_match_arm,
    reason = "the story retains tier decisions and red news from the world's requests"
)]
fn step_news(world: &mut World, task: u64, entry: u64, news: &mut Vec<top::Request>) -> top::TierDecision {
    keep_news(world, news);
    world.event(top::Event::StepTier { task, entry, heard: top::TierHeard { released: false, cancelled: false } });
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
    world.push_file(b"main", path, content);
    let commit = translate::commit(world.branch(b"main"));
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

#[test]
fn a_confirmed_red_tells_its_goal_the_failed_jobs_end() {
    const GOAL: u64 = 201;
    const CHANGE: u64 = 202;
    const SUBSCRIPTION: u64 = 203;
    const TEST: &[u8] = b"suite.first";
    // The fake's workflow step writes four numbered failure lines.
    const LOG: &[u8] =
        b"failing check browser 0\nfailing check browser 1\nfailing check browser 2\nfailing check browser 3\n";
    let mut world = world(21, Box::new([]), true);
    start_baseline(&mut world);
    let commit = push(&mut world, b"failures", b"suite.first\n");
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
    world.event(top::Event::Subscribe {
        subscription: top::Subscriber {
            task: GOAL,
            number: SUBSCRIPTION,
            topic: top::Topic::Landings { repository: REPO, branch: b"main".as_slice().into() },
            own_change: None,
            goal_tasks: Some(Box::new([CHANGE])),
            paths: Box::new([]),
        },
    });
    finish(&mut world, BASELINE);
    attach(&mut world, 2, RANGE);
    world.take_calls();
    let mut news = Vec::new();
    let verdict = finish_news(&mut world, RANGE, &mut news);
    keep_news(&mut world, &mut news);
    let failing = p::Failures::Tests(Box::new([TEST.into()]));
    assert_eq!(verdict.landings.as_ref(), &[p::Blame::Owns(failing.clone())]);
    let calls = world.take_calls();
    assert_eq!(selections(&calls), [None, Some(b"suite.first\n".as_slice().into())]);
    let run = calls
        .iter()
        .filter_map(|op| match op {
            api::Op::Read(api::Read::Job { attempt, .. }) => Some(attempt.run),
            api::Op::Read(_) | api::Op::Write(_) => None,
        })
        .next_back()
        .expect("failed job log read");
    let url = format!("http://forge/org/repo/actions/runs/{run}").into_bytes().into_boxed_slice();
    assert_eq!(
        news,
        [top::Request::News {
            task: GOAL,
            subscription: SUBSCRIPTION,
            topic: top::Topic::Landings { repository: REPO, branch: b"main".as_slice().into() },
            news: top::News::Red(top::RedNews {
                tier: b"browser".as_slice().into(),
                run: 2,
                url: Some(url),
                on: top::RedOn::Landing { commit, branch: b"main".as_slice().into(), failing },
                logs: Box::new([top::JobLog { name: b"browser".as_slice().into(), end: Some(LOG.into()) }]),
            }),
            class: top::Class::Wakes,
        }]
    );
}
