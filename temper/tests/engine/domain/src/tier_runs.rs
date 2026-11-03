//! Engine story fixtures for temper/docs/design/domain/after-landing.md, 3–6 and 12.
use crate::forge_routes::{Gate, StoryLimits, World};
use jig_domain_people as people;
use jig_domain_tasks as tasks;
use skein_fake_forge_domain::api as raw;
use skein_lib::Duration;
use temper_engine_domain as engine;
use temper_engine_domain_forge_tiers::Format;

/// The root, store and independent forge all replay seed 71.
pub const SEED: u64 = 71;
/// Three 600-tick windows cover a 90-second workflow and its polling.
pub const RUN_TICKS: u32 = 1800;
/// The producer and landing fixture's 600 ticks, at 100 milliseconds each.
pub const LANDING_TICKS: u32 = 600;
pub const FAILURES: &[u8] = b"suite.alpha\nsuite.beta\n";

#[derive(Clone, Copy)]
pub struct Tier {
    pub silent: bool,
    pub bound_ms: u64,
}

impl Default for Tier {
    fn default() -> Self {
        Self { silent: false, bound_ms: 7_200_000 }
    }
}

#[must_use]
pub(crate) fn checks() -> Box<[engine::TierCheck]> {
    Box::new([engine::TierCheck {
        name: b"slow".as_slice().into(),
        workflow: b"slow.yml".as_slice().into(),
        ignored: Box::new([b"docs".as_slice().into()]),
        artifact: b"report".as_slice().into(),
        format: Format::Junit,
        follows: temper_engine_domain_forge::Follows::Landing,
        span_ms: 0,
    }])
}

#[must_use]
pub(crate) fn workflows(tier: Option<Tier>, gates: Box<[raw::Workflow]>) -> Box<[raw::Workflow]> {
    let mut workflows = gates.into_vec();
    if let Some(tier) = tier {
        workflows.push(raw::Workflow {
            file: b"slow.yml".as_slice().into(),
            jobs: Box::new([b"test".as_slice().into()]),
            latency_min: Duration::from_secs(90),
            latency_max: Duration::from_secs(90),
            silent: if tier.silent { 1000 } else { 0 },
            passes: 1000,
            cue: Some(raw::Cue::Tests { path: b"failures".as_slice().into() }),
            fails: Box::new([]),
            flaky: Box::new([]),
            report: Some(raw::Report {
                artifact: b"report".as_slice().into(),
                format: raw::ReportFormat::Junit,
                deflate: false,
            }),
            select: Some(b"temper_tests".as_slice().into()),
        });
    }
    workflows.into_boxed_slice()
}

#[must_use]
pub fn world(turn: bool, tier: Tier) -> World {
    World::configured_limits(
        true,
        true,
        false,
        1000,
        Gate::default(),
        None,
        StoryLimits { turn, tier: Some(tier), tasks: Some(12), spec_bytes: Some(4096), ..StoryLimits::default() },
    )
}

/// Room for the red's header, test ids, run URL and log, and both change results.
#[must_use]
pub fn news_world() -> World {
    let mut world = world(false, Tier::default());
    world.env.limits.tasks.message_bytes = 1024;
    world.env.limits.tasks.result_bytes = 512;
    world.env.limits.tasks.inbox_bytes = 4096;
    let mut limits = crate::store::widest();
    limits.tasks = world.env.limits.tasks;
    limits.authority = world.env.limits.authority;
    let mut store = crate::store::Store::with_limits(&limits);
    store.seed(world.store.rows.values().cloned().collect(), Box::new([]));
    world.store = store;
    let client = world.env.limits.forge.client;
    world.with_forge_client(client)
}

/// A tracked goal without a goal branch; its changes land into main.
#[must_use]
pub fn goal(world: &World) -> people::Ask {
    let people::Ask::Create { project, mut batch } = world.chat(1, b"Repair a red after landing") else {
        panic!("goal template")
    };
    batch[0].tracked = Some(0);
    batch[0].label = 2;
    batch[0].flags = people::Flags { listed: true, ..people::Flags::PLAIN };
    batch[0].budget = people::Budget::AsAllowed { cap: Some(400) };
    batch[0].authority.tools |= jig_domain_core::ToolFamilies::JIG.control.0;
    people::Ask::Create { project, batch }
}

/// Red news alone, apart from ordinary landing news in the same inbox.
#[must_use]
pub fn red_news(world: &World, goal: u64) -> Vec<&tasks::Word> {
    let Some(engine::Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Live(row)))) =
        world.store.rows.get(&engine::Key::Core(jig_domain_core::Key::Tasks(tasks::Key::Live(goal))))
    else {
        panic!("live goal")
    };
    row.inbox
        .iter()
        .filter(|word| {
            matches!(word.kind, tasks::MessageKind::News { .. }) && word.words.starts_with(b"Red after landing:")
        })
        .collect()
}

/// The independent forge's dispatched slow runs, oldest first.
#[must_use]
pub fn runs(world: &World) -> Vec<raw::Run> {
    let mut runs = Vec::new();
    for page in 1..=world.fake_env.limits.limits.workflow_runs + 1 {
        let raw::Answer::Runs(part) = world
            .fake
            .inspect(
                &world.fake_env.limits,
                b"org/repo",
                &raw::Read::Runs {
                    workflow: Some(b"slow.yml".as_slice().into()),
                    commit: None,
                    number: None,
                    page,
                    limit: 100,
                },
            )
            .expect("forge runs")
        else {
            panic!("forge runs")
        };
        if part.is_empty() {
            runs.sort_by_key(|run: &raw::Run| run.number);
            return runs;
        }
        runs.extend(part.into_vec().into_iter().filter(|run| run.event == raw::RunEvent::Dispatch));
    }
    panic!("forge run pages exceed their bound")
}

#[must_use]
pub fn run_url(run: &raw::Run) -> String {
    format!("http://forge/org/repo/actions/runs/{}", run.number)
}

/// Failed jobs' failing-step logs, the same ends the forge serves for a job read.
#[must_use]
pub fn job_logs(world: &World, run: &raw::Run) -> Vec<temper_engine_domain_forge::JobLog> {
    let raw::Answer::Jobs(jobs) =
        world.fake.inspect(&world.fake_env.limits, b"org/repo", &raw::Read::Jobs { run: run.id }).expect("run jobs")
    else {
        panic!("run jobs")
    };
    jobs.into_vec()
        .into_iter()
        .filter(|job| job.conclusion == Some(raw::Conclusion::Failure))
        .map(|job| {
            let step = job.steps.iter().find(|step| step.status == raw::StepStatus::Failure).expect("failed step");
            temper_engine_domain_forge::JobLog { name: job.name, end: Some(step.log.clone()) }
        })
        .collect()
}

/// Complete the typed run read with the fake's jobs, as the forge protocol does
/// (skein/docs/design/forge.md, 5.2). The root world's generic projection carries
/// the run alone; the forge world's projection uses the same `run_jobs` builder.
pub fn tick(world: &mut World) {
    for event in &mut world.events {
        if let engine::Event::Forge(event) = event
            && let temper_engine_domain_forge::Event::Client(skein_forge_client::Event::Answered {
                result: Ok(skein_forge::api::Answer::Run(run)),
                ..
            }) = event.as_mut()
        {
            let raw::Answer::Jobs(jobs) = world
                .fake
                .inspect(&world.fake_env.limits, b"org/repo", &raw::Read::Jobs { run: run.id })
                .expect("run jobs")
            else {
                panic!("jobs answer")
            };
            run.jobs = temper_engine_forge_world::translate::run_jobs(jobs);
        }
    }
    world.tick();
}
