//! Engine story fixtures for temper/docs/design/domain/after-landing.md, 3–6 and 12.
use crate::forge_routes::{Gate, StoryLimits, World};
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
        branch: None,
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
