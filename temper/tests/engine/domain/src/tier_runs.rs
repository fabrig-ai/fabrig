//! Engine story fixtures for temper/docs/design/domain/after-landing.md, 3–6 and 12.
use crate::forge_routes::{Gate, StoryLimits, World};
use crate::tier_runs;
use jig_domain_people as people;
use jig_domain_tasks as tasks;
use skein_fake_forge_domain as fake;
use skein_fake_forge_domain::api as raw;
use skein_lib::Duration;
use skein_lib::Token;
use temper_engine_domain as engine;
use temper_engine_domain::{Key, Record};
use temper_engine_domain_forge as forge_top;
use temper_engine_domain_forge_change::State;
use temper_engine_domain_forge_tiers::Format;
use temper_engine_forge_world as forge_world;

/// The root, store and independent forge all replay seed 71.
pub const SEED: u64 = 71;
/// Three 600-tick windows cover a 90-second workflow and its polling.
pub const RUN_TICKS: u32 = 1800;
/// The producer and landing fixture's 600 ticks, at 100 milliseconds each.
pub const LANDING_TICKS: u32 = 600;
pub const FAILURES: &[u8] = b"suite.alpha\nsuite.beta\n";
// Four changes' CI topics, the goal's landing topic and its idle timer.
const GOAL_SUBSCRIPTIONS: u32 = 6;
// Result reservations and the subscription credits, with core news kept separately.
const GOAL_INBOX_MESSAGES: u32 = 16;
const GOAL_INBOX_BYTES: u32 = 8192;
// Four change calls and the timer retain their replay answers.
const GOAL_CALL_RECORDS: u32 = 5;
const GOAL_SPAN_MS: u64 = 1_200_000; // after-landing.md, 2: g's 20-minute span.

#[derive(Clone, Copy)]
pub struct Tier {
    pub silent: bool,
    pub bound_ms: u64,
    pub goals: bool,
}

impl Default for Tier {
    fn default() -> Self {
        Self { silent: false, bound_ms: 7_200_000, goals: false }
    }
}

#[must_use]
pub(crate) fn checks(options: Tier) -> Box<[engine::TierCheck]> {
    let slow = engine::TierCheck {
        name: b"slow".as_slice().into(),
        workflow: b"slow.yml".as_slice().into(),
        ignored: Box::new([b"docs".as_slice().into()]),
        artifact: b"report".as_slice().into(),
        format: Format::Junit,
        follows: temper_engine_domain_forge::Follows::Landing,
        span_ms: 0,
    };
    if options.goals {
        let goal = engine::TierCheck {
            name: b"goal".as_slice().into(),
            workflow: b"goal.yml".as_slice().into(),
            follows: forge_top::Follows::Goals,
            span_ms: GOAL_SPAN_MS,
            ..slow.clone()
        };
        Box::new([slow, goal])
    } else {
        Box::new([slow])
    }
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
        if tier.goals {
            let mut goal = workflows.last().expect("slow workflow").clone();
            goal.file = b"goal.yml".as_slice().into();
            workflows.push(goal);
        }
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
    news_world_with(Tier::default())
}

/// The news world with a second tier following goal branches every 20 minutes.
#[must_use]
pub fn goal_world() -> World {
    news_world_with(Tier { goals: true, ..Tier::default() })
}

fn news_world_with(options: Tier) -> World {
    let mut world = world(false, options);
    world.env.limits.tasks.message_bytes = 1024;
    world.env.limits.tasks.result_bytes = 512;
    world.env.limits.tasks.inbox_bytes = 4096;
    if options.goals {
        world.env.limits.tasks.subscriptions = GOAL_SUBSCRIPTIONS;
        world.env.limits.tasks.inbox_messages = GOAL_INBOX_MESSAGES;
        world.env.limits.tasks.inbox_bytes = GOAL_INBOX_BYTES;
        world.env.limits.people.inbox_recipients = GOAL_INBOX_MESSAGES * 2 + world.env.limits.authority.roles + 4;
        world.env.limits.call_records = GOAL_CALL_RECORDS;
    }
    let mut limits = crate::store::widest();
    limits.tasks = world.env.limits.tasks;
    limits.authority = world.env.limits.authority;
    limits.call_records = world.env.limits.call_records;
    limits.people.inbox_recipients = world.env.limits.people.inbox_recipients;
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
    workflow_runs(world, b"slow.yml")
}

/// Dispatched runs of one fixture workflow, oldest first.
#[must_use]
pub fn workflow_runs(world: &World, workflow: &[u8]) -> Vec<raw::Run> {
    let mut runs = Vec::new();
    for page in 1..=world.fake_env.limits.limits.workflow_runs + 1 {
        let raw::Answer::Runs(part) = world
            .fake
            .inspect(
                &world.fake_env.limits,
                b"org/repo",
                &raw::Read::Runs { workflow: Some(workflow.into()), commit: None, number: None, page, limit: 100 },
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

#[must_use]
#[expect(clippy::wildcard_enum_match_arm, reason = "the fixture selects only the rows relevant to the tier story")]
pub fn task(world: &World, number: u64) -> &tasks::TaskRecord {
    world
        .store
        .rows
        .values()
        .find_map(|row| match row {
            Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Live(row) | tasks::Stored::Ended(row)))
                if row.number == number =>
            {
                Some(row.as_ref())
            }
            _ => None,
        })
        .expect("story task")
}

#[must_use]
pub fn change(world: &World, number: u64) -> &forge_top::ChangeRow {
    let Some(Record::Forge { row }) = world.store.rows.get(&Key::Forge(forge_top::Key::Change(number))) else {
        panic!("change {number}")
    };
    let forge_top::Stored::Change(row) = row.as_ref() else { panic!("change row") };
    row
}

#[must_use]
pub fn tier(world: &World) -> &forge_top::TierRow {
    named_tier(world, b"slow")
}

#[must_use]
pub fn named_tier<'a>(world: &'a World, name: &[u8]) -> &'a forge_top::TierRow {
    let Some(Record::Forge { row }) =
        world.store.rows.get(&Key::Forge(forge_top::Key::Tier { repository: forge_world::REPO, tier: name.into() }))
    else {
        panic!("tier {:?}", std::str::from_utf8(name))
    };
    let forge_top::Stored::Tier(row) = row.as_ref() else { panic!("tier row") };
    row
}

#[expect(clippy::wildcard_enum_match_arm, reason = "the fixture selects only the rows relevant to the tier story")]
pub fn until(world: &mut World, ticks: u32, description: &str, met: impl Fn(&World) -> bool) {
    for _ in 0..ticks {
        if met(world) {
            return;
        }
        tier_runs::tick(world);
    }
    assert!(
        met(world),
        "{description}: seed={SEED}, time={:?}, tier={:?}, changes={:?}, tasks={:?}",
        world.env.now,
        tier(world),
        world
            .store
            .rows
            .values()
            .filter_map(|row| match row {
                Record::Forge { row } => match row.as_ref() {
                    forge_top::Stored::Change(row) => Some((row.task, &row.change.state)),
                    _ => None,
                },
                Record::Core(_) => None,
            })
            .collect::<Vec<_>>(),
        world
            .store
            .rows
            .values()
            .filter_map(|row| match row {
                Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Live(row))) =>
                    Some((row.number, &row.phase)),
                _ => None,
            })
            .collect::<Vec<_>>()
    );
}

pub fn produce_delegated(
    world: &mut World,
    answer: engine::CallAnswer,
    base: &[u8],
    path: &[u8],
    content: &[u8],
) -> u64 {
    let engine::CallAnswer::Delegated(made) = answer else { panic!("change delegated") };
    let number = made[0].task;
    until(world, LANDING_TICKS, "producer assigned", |world| {
        world.assigned.iter().any(|run| task(world, run.task).requester == tasks::Party::Task(number))
    });
    let producer = world
        .assigned
        .iter()
        .find(|run| task(world, run.task).requester == tasks::Party::Task(number))
        .expect("producer")
        .clone();
    let branch = producer.workspace.repositories[0].push.clone().expect("producer branch");
    let raw::Answer::Commit(base) = world.external(1, raw::Op::Read(raw::Read::Branch { branch: base.into() })) else {
        panic!("landing branch tip")
    };
    world.external(2, raw::Op::Git(raw::Git::Create { branch: branch.clone(), commit: base }));
    fake::advance(&mut world.fake, &world.fake_env, b"org/repo", &branch, path, content, 1).expect("producer pushed");
    let event = world.protocol.answer(
        Token::new(7),
        producer.task,
        producer.attempt,
        5,
        tasks::End::Finished {
            result: tasks::TaskResult::Change {
                title: tasks::ChangeTitle::new(b"Small fix".as_slice().into()),
                connector: 0,
                kind: 2,
                resource: forge_top::tag(forge_world::REPO.forge, forge_world::REPO.repository),
                words: b"pushed".as_slice().into(),
            },
            cancel_delegates: false,
        },
        engine::Parts::none(),
    );
    world.send(event);
    until(world, LANDING_TICKS, "change landed and verifying", |world| {
        matches!(change(world, number).change.state, State::Landed { .. })
    });
    number
}

#[must_use]
#[expect(clippy::wildcard_enum_match_arm, reason = "the fixture selects only the rows relevant to the tier story")]
pub fn merge(world: &World, number: u64) -> skein_forge::api::Commit {
    match change(world, number).change.state {
        State::Landed { merge, .. }
        | State::Red { merge, .. }
        | State::Cleared { merge }
        | State::Uncleared { merge } => merge,
        ref state => panic!("change landed: {state:?}"),
    }
}

#[must_use]
pub fn restart_cut(world: &World) -> bool {
    world.store.pending.is_empty() && world.pending.is_empty()
}
