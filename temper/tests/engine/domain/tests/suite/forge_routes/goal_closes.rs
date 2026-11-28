//! Stock template goals close without an agent's judgement after their work lands
//! (temper/docs/design/protocol/configuration/stock.md;
//! temper/docs/design/domain/forge/branches.md, 2; jig/docs/design/domain/tasks/lifecycle-endings.md, 3).
use super::close_change::{CLOSE_COUNTS, CLOSE_FAILURE, finish_with, refusal_with};
use super::goal_branch::{branch_tip, stock_defaults};
use super::goal_closer::{
    assert_hold_words, close_job, closer_row, finish_close as push_close, producer_for, result_words, wait_ended,
    wait_held, wait_landing,
};
use super::goal_landing::{ended, finish_run, mine, row, status};
use super::*;
use jig_client::contract as c;
use jig_domain_link::JobBound;
use jig_domain_notes as notes;
use temper_engine_domain_forge_change as change;
use temper_engine_service_vocabulary::limits::close_bound;
use temper_protocol_worker_wire::v1::Record as WorkerRecord;

#[path = "close_wait.rs"]
mod close_wait;
#[path = "ends_goals.rs"]
mod ends;

const CHANNEL: Token = Token::new(7);
const WORKER_POSITION: u32 = 0;
const WORKER_SLOTS: u32 = 4;
const STEPS: u32 = 4000;
const WORDS: &[u8] = b"Build the draft\n\nDraft: PATH";
const SINGLE_ATTEMPT: u8 = 1; // The project's bound holds on the first unsuccessful attempt.
const CLOSE_RETRIES: u32 = 1; // One retry distinguishes an untyped failure from a finished finding.

fn goal_world_in(template: u32, branch: bool) -> (World, engine::Assignment, Box<[u8]>) {
    goal_world_with_attempts(template, branch, engine::LANDING_ATTEMPTS)
}

fn goal_world_with_attempts(template: u32, branch: bool, attempts: u8) -> (World, engine::Assignment, Box<[u8]>) {
    let (world, goal, branch, _) = goal_world_with_authority(template, branch, attempts, None, None);
    (world, goal, branch)
}

fn goal_world_with_authority(
    template: u32,
    branch: bool,
    attempts: u8,
    spend: Option<u64>,
    room: Option<u32>,
) -> (World, engine::Assignment, Box<[u8]>, people::Authority) {
    goal_world_on_landing(template, branch, attempts, spend, room, b"main")
}

fn goal_world_on_landing(
    template: u32,
    branch: bool,
    attempts: u8,
    spend: Option<u64>,
    room: Option<u32>,
    landing: &[u8],
) -> (World, engine::Assignment, Box<[u8]>, people::Authority) {
    let defaults = stock_defaults();
    let mut world = closer_world(landing, attempts, &defaults, None);
    hello_goal_worker(
        &mut world,
        CHANNEL,
        WORKER_POSITION,
        defaults.rules.maximum_run_spend,
        Box::new([JobBound { result: 8192, ..close_bound() }]),
    );
    start_goal(world, template, branch, spend, room, landing)
}

fn closer_world(
    landing: &[u8],
    attempts: u8,
    defaults: &jig_domain_core::settings::Defaults,
    worker_settings: Option<Box<[jig_domain_core::settings::WorkerSetting]>>,
) -> World {
    closer_world_with_merge(landing, attempts, defaults, worker_settings, false)
}

fn closer_world_with_merge(
    landing: &[u8],
    attempts: u8,
    defaults: &jig_domain_core::settings::Defaults,
    worker_settings: Option<Box<[jig_domain_core::settings::WorkerSetting]>>,
    merge: bool,
) -> World {
    let model = stock::box_model();
    let mut limits = temper_engine_domain_world::forge_routes::box_limits();
    // One repair reaches the held landing without a long repair tree.
    limits.forge.change_policy.repairs = 1;
    if merge {
        limits.tasks.retries.run.retries = CLOSE_RETRIES;
    }
    let landing_deployment = Box::new([engine::LandingRule {
        sets: Box::new([]),
        connector: 1,
        kind: 4,
        enforce: true,
        pattern: authority::Pattern {
            segments: Box::new([
                b"forge".as_slice().into(),
                b"forge.example".as_slice().into(),
                b"org".as_slice().into(),
                b"repo".as_slice().into(),
                b"branch".as_slice().into(),
            ]),
            last: authority::Last::Exact(landing.into()),
        },
        ci: true,
        up_to_date: false,
        gates: Box::new([]),
        approvals: Box::new([]),
    }]);
    // Bootstrap the store and root once with the complete closer fixture.
    let mut world = World::configured_stock_limits_with(
        defaults,
        &model,
        &limits,
        StoryLimits {
            worker_settings,
            close_job: Some((Some(b".temper/close".as_slice().into()), true)),
            merge_kind: merge,
            goal_closer: true,
            landing_attempts: Some(attempts),
            landing_deployment: Some(landing_deployment),
            ..StoryLimits::default()
        },
    );
    if merge {
        world = world.with_merge_job(b".temper/merge");
    }
    world.adopt_landing(landing);
    if landing != b"main" {
        fake::advance(&mut world.fake, &world.fake_env, b"org/repo", landing, b"Prepare landing", b"landing", 1)
            .expect("landing differs from the forge default");
    }
    world
}

fn hello_goal_worker(world: &mut World, channel: Token, position: u32, run_spend: u64, jobs: Box<[JobBound]>) {
    let model = stock::box_model();
    let mut event = protocol::hello_at(channel, position, WORKER_SLOTS, Box::new([]));
    let engine::Event::Worker(engine::WorkerInput::Link { link: engine::Link::Hello { hello, .. }, .. }) = &mut event
    else {
        panic!("hello")
    };
    hello.bounds.turns = 800;
    hello.bounds.time = Duration::from_secs(28_800);
    hello.bounds.spend = run_spend;
    hello.bounds.message_bytes = world.env.limits.tasks.message_bytes;
    hello.bounds.inbox = world.env.limits.tasks.run_inbox;
    hello.bounds.endpoints = Box::new([jig_domain_link::EndpointBound {
        endpoint: model.endpoint,
        window: model.window,
        output: model.output,
    }]);
    hello.jobs = jobs;
    world.send(event);
}

fn start_goal(
    mut world: World,
    template: u32,
    branch: bool,
    spend: Option<u64>,
    room: Option<u32>,
    landing: &[u8],
) -> (World, engine::Assignment, Box<[u8]>, people::Authority) {
    let model = stock::box_model();
    let jig_domain_core::reads::Answer::Document(document) = world.party_read(jig_domain_core::reads::Read::Frame)
    else {
        panic!("person's frame")
    };
    let jig_domain_core::documents::Document::Frame(frame) = *document else { panic!("frame document") };
    let preset = temper_client_tools::limits::client().client;
    let link = jig_client_protocol_link::Limits::new(preset.domain, preset.exchanges).expect("client limits");
    let catalogue = jig_protocol_engine_clients::Model {
        new_until: None,
        number: 1,
        source: jig_domain_core::settings::ModelSource::Stock,
        estimated: false,
        name: model.name.clone(),
        provider: Some(model.provider),
        window: model.window,
        output: model.output,
        efforts: model.efforts.clone(),
        effort: model.effort.clone(),
        prices: jig_protocol_engine_clients::Prices {
            input: model.input_price,
            cached: model.cached_price,
            cache_write: model.cache_write_price,
            output: model.output_price,
            unit: model.price_unit,
        },
        description: None,
        shown: None,
        picker: true,
        speeds: Box::default(),
        speed: None,
    };
    let wire = jig_protocol_engine_clients::translate::owner::frame(
        &frame,
        skein_lib::Wall::from_nanos(86_400_000_000_000),
        &[catalogue],
        link.wire(),
    )
    .expect("person's creating offer");
    let me = jig_client_protocol_link::translate::documents::frame(wire, &link).expect("client's creating offer");
    let charter = if template == 4 { 5 } else { 2 };
    let judged = jig_client::shapes::set_goal_from(
        &me,
        1,
        template,
        jig_client::shapes::GoalDraft {
            words: c::Text::new(WORDS.into(), me.words_bytes).expect("goal words fit"),
            executor: c::Executor::Agent { charter },
            budget: 95_920,
            deadline: None,
            priority: Some(1),
            llm: None,
        },
        c::Text::new(Box::new([]), me.words_bytes).expect("empty reason fits"),
    );
    let jig_client::shapes::Judged::Create(c::Ask::Create { project, tasks }) = judged else {
        panic!("goal offered: {judged:?}")
    };
    let mut member = member(tasks.as_slice().first().expect("one goal"));
    let full = member.authority.clone();
    if let Some(spend) = spend {
        member.budget = people::Budget::AsAllowed { cap: Some(spend) };
    }
    if let Some(tasks) = room {
        member.authority.delegation.tasks = tasks;
    }
    if !branch {
        member.symbols = Box::new([]);
    }
    let goal = assigned_chat(&mut world, 92, people::Ask::Create { project, batch: Box::new([member]) });
    world.until(Until::Ready);
    let branch = if template == 4 && branch {
        format!("temper/{}/g{}", goal.task, goal.task).into_bytes().into_boxed_slice()
    } else {
        landing.into()
    };
    (world, goal, branch, full)
}

fn goal_world(template: u32) -> (World, engine::Assignment, Box<[u8]>) {
    goal_world_in(template, true)
}

// Only the shape this fixture asks set_goal_from for crosses the domain world's worker boundary.
fn member(task: &c::NewTask) -> people::Member {
    assert!(
        task.spec.parameters.is_empty()
            && task.spec.inputs.is_empty()
            && task.dependencies.is_empty()
            && task.named.is_empty()
    );
    assert!(task.llm.is_none() && task.wake.is_none());
    assert_eq!(task.authority.models, Some(c::AuthorityModels::All));
    assert_eq!(task.deadline, c::NewDeadline::Named);
    assert_eq!(task.budget, c::NewBudget::Named);
    let c::Executor::Agent { charter } = task.executor else { panic!("agent goal") };
    let c::NewContract::Report { words } = task.contract else { panic!("goal report") };
    let mut member = jig_test_profile::fixtures::people_member(task.spec.words.bytes(), task.authority.spend);
    member.executor = people::TaskExecutor::Agent { charter };
    member.contract = people::Contract::Report { words };
    member.authority = people::Authority {
        tools: task.authority.tools,
        models: Some(people::Models::All),
        grants: task
            .authority
            .grants
            .iter()
            .map(|grant| people::Grant {
                connector: grant.connector,
                kind: grant.kind,
                pattern: pattern(&grant.pattern),
            })
            .collect(),
        delegation: people::Delegation {
            kinds: task
                .authority
                .delegation
                .kinds
                .iter()
                .map(|kind| match kind {
                    c::AuthorityExecutor::Charter(n) => people::Executor::Charter(*n),
                    c::AuthorityExecutor::Procedure(n) => people::Executor::Procedure(*n),
                    c::AuthorityExecutor::Role(n) => people::Executor::Role(*n),
                    c::AuthorityExecutor::Job(n) => people::Executor::Job(*n),
                })
                .collect(),
            tasks: task.authority.delegation.tasks,
            depth: task.authority.delegation.depth,
        },
        spend: task.authority.spend,
        deadline: task.authority.deadline,
        notes: task.authority.notes,
        note_resources: task
            .authority
            .note_resources
            .iter()
            .map(|scope| people::ResourceScope { connector: scope.connector, pattern: pattern(&scope.pattern) })
            .collect(),
    };
    member.wake = None;
    member.symbols = task
        .symbols
        .iter()
        .map(|symbol| people::Symbol { connector: symbol.connector, symbol: symbol.symbol })
        .collect();
    member.tracked = task.tracked;
    member.label = task.label;
    member.flags = people::Flags {
        ended_by: match task.flags.ended_by {
            c::EndedBy::Result => people::EndedBy::Result,
            c::EndedBy::Requester => people::EndedBy::Requester,
            c::EndedBy::Standing => people::EndedBy::Standing,
            c::EndedBy::Closer(closer) => {
                people::EndedBy::Closer(people::Closer { connector: closer.connector, code: closer.code })
            }
        },
        conversation: task.flags.conversation,
        listed: task.flags.listed,
        retention: match task.flags.retention {
            c::Retention::Kept => people::Retention::Kept,
            c::Retention::ForgottenOnceRead => people::Retention::ForgottenOnceRead,
        },
    };
    member
}
fn pattern(value: &c::Pattern) -> people::Pattern {
    people::Pattern {
        segments: value.segments.iter().cloned().collect(),
        last: match &value.last {
            c::PatternEnd::Exact(bytes) => people::Last::Exact(bytes.clone()),
            c::PatternEnd::Open(bytes) => people::Last::Open(bytes.clone()),
        },
    }
}
fn make_change(world: &mut World, goal: &engine::Assignment, call: u64) -> (u64, engine::Assignment) {
    let answer = mine(world, goal, call, br#"{"words":"Build part of the draft","landing":"goal"}"#);
    let Some(engine::CallAnswer::Delegated(made)) = answer else {
        panic!("change admitted: {answer:?}; {:?}", world.call_errors)
    };
    let task = made[0].task;
    (task, producer_for(world, task))
}
fn produce(world: &mut World, task: u64, branch: &[u8], producer: &engine::Assignment, title: &[u8]) {
    push_production(world, task, branch, producer, title);
    wait_ended(world, task);
}

fn push_production(world: &mut World, task: u64, branch: &[u8], producer: &engine::Assignment, title: &[u8]) {
    let own = row(world, task).branch.clone();
    let tip = branch_tip(world, branch);
    assert_eq!(
        world.external(1, raw::Op::Git(raw::Git::Create { branch: own.clone(), commit: tip })),
        raw::Answer::Branch(raw::Created::Created)
    );
    let head = fake::advance(&mut world.fake, &world.fake_env, b"org/repo", &own, title, b"implemented", 1)
        .expect("producer pushed");
    status(world, head, raw::Check::Passed);
    finish_run(
        world,
        producer,
        tasks::TaskResult::Change {
            title: tasks::ChangeTitle::new(title.into()),
            connector: 1,
            kind: 2,
            resource: forge_top::tag(forge_world::REPO.forge, forge_world::REPO.repository),
            words: b"Implemented.\n\n## Left:\nnone".as_slice().into(),
        },
    );
}
fn goal_runs(world: &World, goal: u64) -> usize {
    world.assigned.iter().filter(|run| run.task == goal).count()
}
fn closer_of(world: &mut World, goal: u64) -> u64 {
    for _ in 0..STEPS {
        if let Some(closer) = live_task(world, goal).closer {
            return closer;
        }
        world.tick();
    }
    panic!("goal did not make its closer: {:?}", live_task(world, goal))
}
fn finish_close(world: &mut World, close: u64, branch: &[u8], job: &host::JobAssignment) {
    use temper_protocol_worker_wire::v1;
    let spec = v1::JobSpec::decode(&v1::CEILINGS, &mut skein_lib::Reader::new(&job.spec)).expect("close job spec");
    assert_eq!(spec.command.as_ref(), b"repo/.temper/hooks/close");
    assert!(spec.find && spec.refusal);
    assert_eq!(spec.fallback.as_ref(), b"repo/.temper/close");
    assert_eq!(Some(spec.request.as_ref()), row(world, close).close.as_deref());
    for _ in 0..STEPS {
        if world.pending.is_empty() {
            break;
        }
        world.tick();
    }
    let before = world.fake.branch(b"org/repo", branch).expect("goal branch");
    push_close(world, close, branch, job);
    let own = row(world, close).branch.clone();
    let head = world.fake.branch(b"org/repo", &own).expect("scripted close branch");
    assert_ne!(head, before, "the scripted command commits its status edit");
    status(world, head, raw::Check::Passed);
}
fn begin_closing(world: &mut World, goal: u64, branch: &[u8]) -> (u64, u64) {
    let closer = closer_of(world, goal);
    let job = close_job(world);
    let tasks::Party::Task(close) = live_task(world, job.run.raw()).requester else { panic!("close change") };
    finish_close(world, close, branch, &job);
    for _ in 0..STEPS {
        if ended(world, close).is_some() {
            break;
        }
        world.tick();
    }
    assert!(ended(world, close).is_some(), "close change: {:?}", row(world, close).change.state);
    let landing = wait_landing(world, closer);
    (closer, landing)
}
fn finish_closing(world: &mut World, goal: u64, branch: &[u8]) -> (u64, u64) {
    let steps = begin_closing(world, goal, branch);
    pass_landing(world, goal, branch);
    steps
}

fn pass_landing(world: &mut World, goal: u64, branch: &[u8]) {
    let head = world.fake.branch(b"org/repo", branch).expect("goal branch awaiting its landing");
    status(world, head, raw::Check::Passed);
    wait_ended(world, goal);
}
fn assert_report(world: &World, goal: u64, closer: u64, landing: u64, changes: &[(u64, &[u8])]) {
    assert_eq!(result_words(world, goal), result_words(world, closer));
    let output = core::str::from_utf8(result_words(world, goal)).expect("goal report UTF-8");
    assert!(output.starts_with(&format!("Goal T{goal}: Build the draft\nLanded:\n")), "{output}");
    for (task, title) in changes {
        let title = core::str::from_utf8(title).expect("change title UTF-8");
        let pull = row(world, *task).pull.expect("landed change pull");
        assert!(
            output.contains(&format!("- T{task} {title}: pull request #{pull} into temper/{goal}/g{goal}\n")),
            "{output}"
        );
    }
    let merge = match row(world, landing).change.state {
        change::State::Landed { merge, .. } | change::State::Cleared { merge } => merge,
        ref state => panic!("landing landed: {state:?}"),
    };
    let commit = forge_top::commit_text(&merge);
    let commit = core::str::from_utf8(&commit).expect("merge commit UTF-8");
    assert!(
        output.contains(&format!(
            "The landing: pull request #{}, merged into main as {commit}",
            row(world, landing).pull.expect("landing pull")
        )),
        "{output}"
    );
}

struct OutdatedStory {
    world: World,
    goal: u64,
    branch: Box<[u8]>,
    work: u64,
    closer: u64,
    close: u64,
    main: u64,
}

fn outdated_close_world(attempts: u8) -> OutdatedStory {
    let (mut world, goal, branch, work, closer, main) = awaiting_close_with_merge(attempts);
    let (close, job) = next_close(&mut world, closer);
    assert_eq!(finish_outdated(&mut world, close, &job).landing, translate::commit(main));
    OutdatedStory { world, goal, branch, work, closer, close, main }
}

fn awaiting_close_with_merge(attempts: u8) -> (World, u64, Box<[u8]>, u64, u64, u64) {
    let defaults = stock_defaults();
    let mut world = closer_world_with_merge(b"main", attempts, &defaults, None, true);
    hello_goal_worker(
        &mut world,
        CHANNEL,
        WORKER_POSITION,
        defaults.rules.maximum_run_spend,
        Box::new([protocol::merge_bound(), close_bound()]),
    );
    let (mut world, run, branch, _) = start_goal(world, 4, true, None, None, b"main");
    let (work, producer) = make_change(&mut world, &run, 93);
    produce(&mut world, work, &branch, &producer, b"Parser");
    let main = fake::advance(&mut world.fake, &world.fake_env, b"org/repo", b"main", b"main", b"main work", 1)
        .expect("main advanced after the goal branched");
    park_goal(&mut world, &run);
    let goal = run.task;
    let closer = closer_of(&mut world, goal);
    (world, goal, branch, work, closer, main)
}

fn next_close(world: &mut World, closer: u64) -> (u64, host::JobAssignment) {
    let job = close_job(world);
    assert_eq!(job.kind, close_bound().kind);
    let tasks::Party::Task(task) = live_task(world, job.run.raw()).requester else { panic!("close task") };
    assert_eq!(live_task(world, task).requester, tasks::Party::Task(closer));
    (task, job)
}

fn finish_outdated(world: &mut World, task: u64, job: &host::JobAssignment) -> change::CloseAt {
    let at = row(world, task).close_at.expect("close read both tips");
    let head = forge_top::commit_text(&at.branch);
    let head = core::str::from_utf8(&head).expect("head in hex");
    let result = format!(r#"{{"outdated":{{"head":"{head}","why":"window"}}}}"#);
    finish_with(world, job, result.as_bytes());
    wait_ended(world, task);
    let record = row(world, task);
    assert!(matches!(record.change.state, change::State::NoChange { .. }));
    let at = record.close_at.expect("retained close finding");
    assert_eq!(at.outdated, Some(change::CloseOutdated::Window));
    at
}

fn next_catch_up(world: &mut World, goal: u64, closer: u64, branch: &[u8], main: u64) -> (u64, host::JobAssignment) {
    for _ in 0..STEPS {
        if !world.jobs.is_empty() || ended(world, goal).is_some() {
            break;
        }
        assert!(!matches!(live_task(world, closer).phase, tasks::Phase::Held { .. }));
        world.tick();
    }
    assert!(!world.jobs.is_empty(), "the closer made no catch-up: {:?}", closer_row(world, closer));
    let job = world.jobs.remove(0);
    assert_eq!(job.kind, protocol::merge_bound().kind, "an outdated close must ask for a catch-up");
    let tasks::Party::Task(task) = live_task(world, job.run.raw()).requester else { panic!("catch-up task") };
    let record = live_task(world, task);
    assert_eq!(record.requester, tasks::Party::Task(closer));
    let short = short_hex(&translate::commit(main));
    assert_eq!(record.spec.words.as_ref(), format!("Catch goal {goal} up from main at {short}.").as_bytes());
    let head = branch_tip(world, branch);
    super::catch_up::check_job(world, task, &job, branch, head, main);
    (task, job)
}

fn finish_catch_up(world: &mut World, task: u64, job: &host::JobAssignment, branch: &[u8], main: u64) {
    let head = branch_tip(world, branch);
    super::catch_up::push_merge(world, super::merge_job::Writer::Job(CHANNEL, job), head, main);
    let own = row(world, task).branch.clone();
    let pushed = branch_tip(world, &own);
    status(world, pushed, raw::Check::Passed);
    super::catch_up::finish_job(world, job, br#"{"left":[]}"#);
    wait_ended(world, task);
    assert!(matches!(row(world, task).change.state, change::State::Landed { .. }));
}

/// temper/docs/design/domain/forge/branches.md, 2: the closer catches up once for the same request.
#[test]
fn an_outdated_close_catches_up_from_main_and_closes_again() {
    let OutdatedStory { mut world, goal, branch, work, closer, close, main } = outdated_close_world(SINGLE_ATTEMPT);
    let request = row(&world, close).close.clone();
    let (catch_up, merge) = next_catch_up(&mut world, goal, closer, &branch, main);
    finish_catch_up(&mut world, catch_up, &merge, &branch, main);
    let (again, job) = next_close(&mut world, closer);
    assert_ne!(again, close);
    assert_eq!(row(&world, again).close, request);
    finish_close(&mut world, again, &branch, &job);
    let landing = wait_landing(&mut world, closer);
    pass_landing(&mut world, goal, &branch);
    assert_report(&world, goal, closer, landing, &[(work, b"Parser")]);
    let report = core::str::from_utf8(result_words(&world, closer)).expect("closer report");
    assert!(report.contains(&format!("- T{catch_up} ")), "{report}");
    assert_eq!(goal_runs(&world, goal), 1, "no escalation");
    assert!(world.jobs.is_empty(), "no extra job attempts");
    let row = closer_row(&world, closer).expect("closer row");
    assert_eq!(row.close_changes.as_ref(), &[close, again]);
    assert_eq!(row.closer.caught_up.as_deref(), Some([work].as_slice()));
    world.close();
}

const WINDOW_FINDING: &str = "it holds none of the commits main's published counts cover";
const UPDATE_AND_RELEASE: &str =
    "Catch the goal up from main with a change naming main's tip and release the closer, or release it to";

fn short_hex(commit: &[u8; 32]) -> Box<str> {
    let text = forge_top::commit_text(commit);
    core::str::from_utf8(&text[..12]).expect("commit in hex").into()
}

fn assert_closer_hold(
    world: &World,
    closer: u64,
    step: temper_engine_domain_forge_closer::Step,
    child: u64,
    words: &str,
) {
    use temper_engine_domain_forge_closer::Hold;
    assert_eq!(
        closer_row(world, closer).expect("held closer").closer.hold,
        Some(Hold::ClosingStep { step, change: child })
    );
    assert_hold_words(world, closer, words);
}

/// temper/docs/design/domain/forge/branches.md, 2: close again, but never catch up the same request again.
#[test]
fn a_close_outdated_again_after_its_catch_up_holds_naming_both() {
    use temper_engine_domain_forge_closer::Step;
    let OutdatedStory { mut world, goal, branch, closer, main, .. } = outdated_close_world(SINGLE_ATTEMPT);
    let (catch_up, merge) = next_catch_up(&mut world, goal, closer, &branch, main);
    finish_catch_up(&mut world, catch_up, &merge, &branch, main);
    let main = short_hex(&translate::commit(main));
    let (again, job) = next_close(&mut world, closer);
    let found = short_hex(&finish_outdated(&mut world, again, &job).branch);
    wait_held(&mut world, closer, goal);
    let words = |task| {
        format!(
            "the close step's change T{task} found the goal's branch at {found} outdated again, after its catch-up from main at {main}, change T{catch_up}, landed: {WINDOW_FINDING}. {UPDATE_AND_RELEASE} close again as it stands."
        )
    };
    assert_closer_hold(&world, closer, Step::Close, again, &words(again));
    let run = awakened(&mut world, goal);
    world.advance_ticks(10);
    assert_eq!(goal_runs(&world, goal), 2, "the hold escalates once");
    release(&mut world, &run, closer, 94);
    park_goal(&mut world, &run);
    let (released, job) = next_close(&mut world, closer);
    assert_ne!(released, again);
    finish_outdated(&mut world, released, &job);
    wait_held(&mut world, closer, goal);
    assert_closer_hold(&world, closer, Step::Close, released, &words(released));
    assert_eq!(closer_row(&world, closer).expect("closer row").catch_up_changes.as_ref(), &[catch_up]);
    world.close();
}

/// temper/docs/design/domain/forge/branches.md, 2: a failed catch-up needs a release even below the attempt bound.
#[test]
fn a_catch_up_that_ends_without_landing_holds_the_closer_naming_both() {
    use temper_engine_domain_forge_closer::Step;
    let OutdatedStory { mut world, goal, branch, closer, close, main, .. } =
        outdated_close_world(engine::LANDING_ATTEMPTS);
    let (catch_up, merge) = next_catch_up(&mut world, goal, closer, &branch, main);
    world.sign_in();
    world.send(engine::Event::Party(engine::PartyInput::Ask {
        models: Box::new([]),
        reply_to: ReplyTo::new(Token::new(95)),
        sign_in: world.signed_in.expect("person"),
        key: [95; 16],
        ask: people::Ask::Cancel {
            project: 1,
            task: catch_up,
            reason: b"Stop the catch-up".as_slice().into(),
            grace: None,
        },
    }));
    let event = world.protocol.answer_job(
        CHANNEL,
        merge.run.raw(),
        merge.attempt.raw(),
        host::JobEnding::Failed {
            failure: host::JobFailure::Cancelled(host::Reason::Engine),
            detail: Box::new([]),
            work: host::Work { left: None, saved: None },
        },
    );
    world.send(event);
    wait_held(&mut world, closer, goal);
    let found = short_hex(&row(&world, close).close_at.expect("outdated finding").branch);
    let short_main = short_hex(&translate::commit(main));
    assert_closer_hold(
        &world,
        closer,
        Step::CatchUp,
        catch_up,
        &format!(
            "the catch-up from main at {short_main}, change T{catch_up}, ended cancelled, after the close step's change T{close} found the goal's branch at {found} outdated: {WINDOW_FINDING}. {UPDATE_AND_RELEASE} catch up again."
        ),
    );
    fake::advance(&mut world.fake, &world.fake_env, b"org/repo", b"main", b"later", b"more work", 1)
        .expect("main moved while the closer was held");
    let run = awakened(&mut world, goal);
    release(&mut world, &run, closer, 94);
    park_goal(&mut world, &run);
    let (released, _) = next_catch_up(&mut world, goal, closer, &branch, main);
    assert_ne!(released, catch_up);
    world.close();
}

/// temper/docs/design/domain/forge/change/repairs-releases.md, 1: untyped job failure is not an outdated finding.
#[test]
fn an_older_workers_broken_close_holds_as_today() {
    const BROKE: &[u8] =
        b"exit 2: the goal is older than main's last twenty landings; update from main and close again";
    let (mut world, goal, _, _, closer, _) = awaiting_close_with_merge(SINGLE_ATTEMPT);
    assert_eq!(world.env.limits.tasks.retries.run.retries, CLOSE_RETRIES);
    let close = fail_close_with(&mut world, BROKE);
    wait_held(&mut world, close, goal);
    world.advance_ticks(10);
    assert_hold_words(
        &world,
        close,
        "the close check broke and didn't judge the goal: exit 2: the goal is older than main's last twenty landings; update from main and close again; its release runs it again.",
    );
    let row = row(&world, close);
    assert!(matches!(&row.change.state, change::State::Held { hold, .. } if hold.cause == change::Cause::Failed));
    assert_eq!(row.close_at.and_then(|at| at.outdated), None);
    assert_eq!(closer_row(&world, closer).expect("closer row").catch_up_changes.as_ref(), &[] as &[u64]);
    assert_eq!(world.jobs, [] as [host::JobAssignment; 0]);
    world.close();
}

const REPORT_DESCRIPTION: &[u8] = b"report: Two of three parts are in.";
const REPORT_WITH_LEFT: &[u8] = b"I landed the parser.\n\nLeft: the switch, not yet cut.";
const REPORT_WITH_ANSWERS: &[u8] = b"I landed it.\n\nAsked: the parser's figures.\nparse: 41 ms before, 12 ms after\nallocations: 310 before, 96 after\n\nRisk: none.";

fn write_goal_report(
    world: &mut World,
    run: &engine::Assignment,
    call: u64,
    body: &[u8],
    previous: Option<(u64, u32)>,
) -> (u64, u32) {
    let (name, recalled) = previous.map_or((0, None), |(name, revision)| (name, Some(revision)));
    let answer = world.scripted_call(
        run,
        call,
        engine::Tool::Note {
            entry: notes::New {
                name,
                scope: notes::Scope::Goal { project: 1, goal: run.task },
                description: REPORT_DESCRIPTION.into(),
                body: body.into(),
                references: skein_lib::List::with_capacity(1),
                author: notes::Author::Task { task: run.task, attempt: run.attempt },
            },
            recalled,
            records: Box::new([]),
        },
    );
    let engine::CallAnswer::NoteWritten { name, revision, .. } = answer else { panic!("report written: {answer:?}") };
    (name, revision)
}

struct ReportStory {
    world: World,
    goal: u64,
    branch: Box<[u8]>,
    change: u64,
    note: (u64, u32),
}

fn goal_with_report(body: &[u8]) -> ReportStory {
    let (mut world, goal, branch) = goal_world(4);
    let note = write_goal_report(&mut world, &goal, 93, body, None);
    let (change, producer) = make_change(&mut world, &goal, 94);
    park_goal(&mut world, &goal);
    produce(&mut world, change, &branch, &producer, b"Parser");
    ReportStory { world, goal: goal.task, branch, change, note }
}

fn wait_report_held(world: &mut World, goal: u64) -> u64 {
    let closer = closer_of(world, goal);
    wait_held(world, closer, goal);
    closer
}

fn restart_closing(world: &mut World) {
    for _ in 0..STEPS {
        if temper_engine_domain_world::tier_runs::restart_cut(world) {
            world.restart_same();
            world.until(Until::Ready);
            return;
        }
        world.tick();
    }
    panic!("closer did not reach a durable restart cut");
}

fn assert_report_hold_words(world: &World, closer: u64) {
    let tasks::Phase::Held { source: Some(source), .. } = &live_task(world, closer).phase else {
        panic!("closer's hold account")
    };
    let tasks::HoldSource::Account { words, .. } = source.as_ref() else { panic!("closer's hold words") };
    assert_eq!(words.as_ref(), b"the goal's report leaves work undone: the switch, not yet cut.");
}

#[test]
fn a_closer_holds_before_stepping_on_work_left_uncut() {
    let ReportStory { mut world, goal, .. } = goal_with_report(REPORT_WITH_LEFT);
    let closer = wait_report_held(&mut world, goal);
    let run = awakened(&mut world, goal);
    park_goal(&mut world, &run);
    restart_closing(&mut world);
    world.advance_ticks(100);
    let state = closer_row(&world, closer).expect("closer row");
    assert!(state.close_changes.is_empty(), "no close change while work is left: {:?}", state.close_changes);
    assert!(state.landing_changes.is_empty());
    assert_report_hold_words(&world, closer);
    assert_eq!(goal_runs(&world, goal), 2);
    assert!(ended(&world, goal).is_none());
    world.close();
}

#[test]
fn a_released_closer_lands_and_reports_the_work_left() {
    let ReportStory { mut world, goal, branch, change, .. } =
        goal_with_report(b"I landed the parser.\n\nAsked: causes\nthe cache was cold\nLeft: the switch, not yet cut.");
    let closer = wait_report_held(&mut world, goal);
    assert_report_hold_words(&world, closer);
    let run = awakened(&mut world, goal);
    release(&mut world, &run, closer, 95);
    park_goal(&mut world, &run);
    let (same, landing) = begin_closing(&mut world, goal, &branch);
    assert_eq!(same, closer);
    assert!(ended(&world, goal).is_none(), "restart before the released closer reports");
    restart_closing(&mut world);
    let row = closer_row(&world, closer).expect("closer row after restart");
    assert_eq!(row.closer.goal_asked.as_deref(), Some(b"Asked: causes\nthe cache was cold".as_slice()));
    pass_landing(&mut world, goal, &branch);
    assert_report(&world, goal, closer, landing, &[(change, b"Parser")]);
    let words = core::str::from_utf8(result_words(&world, goal)).expect("goal report UTF-8");
    assert!(
        words.contains(
            "\nAsked: causes\n  the cache was cold\nLeft by the goal: the switch, not yet cut.\nThe landing:"
        ),
        "{words}"
    );
    settle(&mut world, goal);
    world.close();
}

#[test]
fn a_released_closer_waits_for_the_work_cut_instead() {
    let ReportStory { mut world, goal, branch, change, note } = goal_with_report(REPORT_WITH_LEFT);
    let closer = wait_report_held(&mut world, goal);
    let run = awakened(&mut world, goal);
    let _ = write_goal_report(&mut world, &run, 95, b"The parser landed and I cut the switch.", Some(note));
    let (switch, producer) = make_change(&mut world, &run, 96);
    release(&mut world, &run, closer, 97);
    park_goal(&mut world, &run);
    world.advance_ticks(100);
    let state = closer_row(&world, closer).expect("closer row");
    assert!(state.close_changes.is_empty());
    assert!(state.landing_changes.is_empty());
    produce(&mut world, switch, &branch, &producer, b"Switch");
    let (same, landing) = finish_closing(&mut world, goal, &branch);
    assert_eq!(same, closer);
    assert_report(&world, goal, closer, landing, &[(change, b"Parser"), (switch, b"Switch")]);
    let words = core::str::from_utf8(result_words(&world, goal)).expect("goal report UTF-8");
    assert!(!words.contains("Left by the goal:"), "{words}");
    settle(&mut world, goal);
    world.close();
}

fn unheld_report_words(body: &[u8]) -> String {
    let ReportStory { mut world, goal, branch, change, .. } = goal_with_report(body);
    let (closer, landing) = finish_closing(&mut world, goal, &branch);
    assert_report(&world, goal, closer, landing, &[(change, b"Parser")]);
    assert_eq!(goal_runs(&world, goal), 1);
    let words = core::str::from_utf8(result_words(&world, goal)).expect("goal report UTF-8").to_owned();
    world.close();
    words
}

#[test]
fn a_goal_whose_report_answers_its_request_ends_with_the_answers() {
    let words = unheld_report_words(REPORT_WITH_ANSWERS);
    assert!(
        words.contains("\n  Left: none given\nAsked: the parser's figures.\n  parse: 41 ms before, 12 ms after\n  allocations: 310 before, 96 after\nThe landing:"),
        "{words}"
    );
}

#[test]
fn a_goal_whose_report_answers_nothing_ends_as_before() {
    let words = unheld_report_words(b"I landed the parser.");
    assert!(!words.contains("Asked:"), "{words}");
    assert!(words.contains("\n  Left: none given\nThe landing:"), "{words}");
}

#[test]
fn a_goal_whose_last_change_lands_closes_with_no_agent_run() {
    let (mut world, goal, branch) = goal_world(4);
    let (first, first_run) = make_change(&mut world, &goal, 93);
    let (second, second_run) = make_change(&mut world, &goal, 94);
    park_goal(&mut world, &goal);
    assert_eq!(live_task(&world, goal.task).phase, tasks::Phase::Active(tasks::Active::Idle));
    produce(&mut world, first, &branch, &first_run, b"First");
    assert_eq!(live_task(&world, goal.task).closer, None);
    produce(&mut world, second, &branch, &second_run, b"Second");
    let (closer, landing) = finish_closing(&mut world, goal.task, &branch);
    assert_report(&world, goal.task, closer, landing, &[(first, b"First"), (second, b"Second")]);
    assert_eq!(goal_runs(&world, goal.task), 1);
    world.close();
}

#[test]
fn a_goal_lands_its_changes_into_its_branch_then_into_the_adopted_landing() {
    let (mut world, goal, branch, _) = goal_world_on_landing(4, true, engine::LANDING_ATTEMPTS, None, None, b"staging");
    let staging = branch_tip(&mut world, b"staging");
    assert_ne!(staging, branch_tip(&mut world, b"main"));
    assert_eq!(branch_tip(&mut world, &branch), staging, "the goal starts at the landing tip");
    let (first, first_run) = make_change(&mut world, &goal, 93);
    let (second, second_run) = make_change(&mut world, &goal, 94);
    assert_eq!(row(&world, first).base, branch);
    assert_eq!(row(&world, second).base, branch);
    park_goal(&mut world, &goal);
    produce(&mut world, first, &branch, &first_run, b"First");
    assert_eq!(live_task(&world, goal.task).closer, None);
    produce(&mut world, second, &branch, &second_run, b"Second");
    let (closer, landing) = begin_closing(&mut world, goal.task, &branch);
    for _ in 0..STEPS {
        if row(&world, landing).pull.is_some() {
            break;
        }
        world.tick();
    }
    let number = row(&world, landing).pull.expect("goal landing pull");
    let raw::Answer::Pull(pull) = world.external(1, raw::Op::Read(raw::Read::Pull { number })) else {
        panic!("goal landing pull")
    };
    assert_eq!(pull.head, branch);
    assert_eq!(pull.base.as_ref(), b"staging");
    status(&mut world, pull.commit, raw::Check::Passed);
    wait_ended(&mut world, goal.task);
    let merge = match row(&world, landing).change.state {
        change::State::Landed { merge, .. } | change::State::Cleared { merge } => merge,
        ref state => panic!("goal landing landed: {state:?}"),
    };
    assert_eq!(translate::commit(branch_tip(&mut world, b"staging")), merge, "the goal landed whole");
    assert_eq!(result_words(&world, goal.task), result_words(&world, closer));
    let output = core::str::from_utf8(result_words(&world, goal.task)).expect("goal report UTF-8");
    assert!(output.contains(&format!("The landing: pull request #{number}, merged into staging as ")), "{output}");
    assert_eq!(goal_runs(&world, goal.task), 1);
    world.close();
}

#[test]
fn a_closer_ended_goal_without_a_goal_branch_creates_no_main_subscription() {
    let (mut world, goal, _) = goal_world_in(4, false);
    let answer = mine(&mut world, &goal, 93, br#"{"words":"Build on main","landing":"main"}"#);
    let Some(engine::CallAnswer::Delegated(made)) = answer else {
        panic!("change admitted: {answer:?}; {:?}", world.call_errors)
    };
    let task = made[0].task;
    producer_for(&mut world, task);
    assert_eq!(row(&world, task).base.as_ref(), b"main");
    assert!(
        !world.store.rows.values().any(|row| matches!(row,
            Record::Forge { row } if matches!(row.as_ref(),
                forge_top::Stored::Subscription(sub) if sub.task == goal.task
                    && sub.topic == forge_top::Topic::Landings {
                        repository: forge_world::REPO, branch: b"main".as_slice().into()
                    }
            )
        )),
        "a closer-ended goal must not subscribe to main"
    );
    world.close();
}

#[test]
fn a_goal_from_template_2_keeps_the_nothing_awaited_notice() {
    let (mut world, goal, _) = goal_world(2);
    park_goal(&mut world, &goal);
    for _ in 0..STEPS {
        if live_task(&world, goal.task).inbox.iter().any(|word| word.kind == tasks::MessageKind::NothingAwaited) {
            break;
        }
        world.tick();
    }
    assert!(live_task(&world, goal.task).inbox.iter().any(|word| word.kind == tasks::MessageKind::NothingAwaited));
    assert_eq!(live_task(&world, goal.task).closer, None);
    for _ in 0..STEPS {
        if goal_runs(&world, goal.task) == 2 {
            break;
        }
        world.tick();
    }
    assert_eq!(goal_runs(&world, goal.task), 2, "the notice wakes the result-ended goal");
    world.close();
}

pub(super) fn park_goal(world: &mut World, run: &engine::Assignment) {
    park_goal_spent(world, run, 0);
}

fn park_goal_spent(world: &mut World, run: &engine::Assignment, spent: u64) {
    answer_parked(world, run, spent);
    world.until(Until::Ready);
}

fn answer_parked(world: &mut World, run: &engine::Assignment, spent: u64) {
    world.scripted_read_inbox(run);
    let turns = live_task(world, run.task).turn;
    let mut event =
        world.protocol.answer(CHANNEL, run.task, run.attempt, spent, tasks::End::Parked, engine::Parts::none());
    let engine::Event::Worker(engine::WorkerInput::Link {
        link: engine::Link::Up { up: host::LinkUp::Answer { answer, .. }, .. },
        ..
    }) = &mut event
    else {
        panic!("worker answer")
    };
    answer.turns = turns;
    world.send(event);
}

fn awakened(world: &mut World, goal: u64) -> engine::Assignment {
    let run = next_goal_run(world, goal);
    assert!(live_task(world, goal).reasons.contains(tasks::Reasons::ESCALATION));
    run
}

fn next_goal_run(world: &mut World, goal: u64) -> engine::Assignment {
    for _ in 0..STEPS {
        if let Some(run) = world.assigned.iter().filter(|run| run.task == goal).nth(1) {
            assert_eq!(goal_runs(world, goal), 2);
            return run.clone();
        }
        world.tick();
    }
    panic!("goal was not woken by its escalation")
}
fn release(world: &mut World, goal: &engine::Assignment, task: u64, call: u64) {
    let revision = live_task(world, task).revisions.hold;
    let answer = world.scripted_call(goal, call, engine::Tool::Release { target: task, revision, again: None });
    assert!(matches!(answer, engine::CallAnswer::Revised { .. }), "{answer:?}");
}
fn settle(world: &mut World, goal: u64) {
    world.advance_ticks(100);
    assert_eq!(goal_runs(world, goal), 2, "the escalation wakes the goal exactly once");
    assert!(ended(world, goal).is_some());
}

fn fail_close(world: &mut World) -> u64 {
    fail_close_with(world, b"status command exited 1")
}

fn fail_close_with(world: &mut World, detail: &[u8]) -> u64 {
    let mut close = None;
    for _ in 0..=world.env.limits.tasks.retries.run.retries {
        let job = close_job(world);
        let tasks::Party::Task(task) = live_task(world, job.run.raw()).requester else { panic!("close change") };
        assert_eq!(job.kind, close_bound().kind);
        assert_eq!(*close.get_or_insert(task), task, "retry within the same close change");
        let event = world.protocol.answer_job(
            CHANNEL,
            job.run.raw(),
            job.attempt.raw(),
            host::JobEnding::Failed {
                failure: host::JobFailure::Failed,
                detail: detail.into(),
                work: host::Work { left: None, saved: None },
            },
        );
        world.send(event);
    }
    close.expect("close attempted")
}

#[test]
fn a_failing_close_command_wakes_the_goal_once() {
    // This story judges every ended close change, rather than automatic attempts.
    let (mut world, goal, branch) = goal_world_with_attempts(4, true, SINGLE_ATTEMPT);
    park_goal(&mut world, &goal);
    let closer = closer_of(&mut world, goal.task);
    let first = fail_close(&mut world);
    assert_eq!(live_task(&world, closer).requester, tasks::Party::Task(goal.task));
    assert_eq!(live_task(&world, first).requester, tasks::Party::Task(closer));
    assert!(
        live_task(&world, goal.task).waiting_on.is_empty(),
        "goal waits: {:?}",
        live_task(&world, goal.task).waiting_on
    );
    for _ in 0..STEPS {
        if matches!(live_task(&world, first).escalation, tasks::Escalation::Waiting { .. }) {
            break;
        }
        world.tick();
    }
    let context = live_task(&world, goal.task);
    assert!(
        matches!(live_task(&world, first).escalation, tasks::Escalation::Waiting { holder: tasks::EscalationHolder::Task(holder), .. } if holder == goal.task),
        "goal phase {:?}; reasons {:?}; funding {:?}; escalation {:?}",
        context.phase,
        context.reasons,
        context.numbers,
        live_task(&world, first).escalation
    );
    wait_held(&mut world, first, goal.task);
    let run = awakened(&mut world, goal.task);
    let answer = world.scripted_call(
        &run,
        93,
        engine::Tool::Cancel { target: first, reason: b"Retry the close step".as_slice().into() },
    );
    assert!(matches!(answer, engine::CallAnswer::Controlled), "{answer:?}");
    wait_ended(&mut world, first);
    wait_held(&mut world, closer, goal.task);
    release(&mut world, &run, closer, 94);
    park_goal(&mut world, &run);
    let (same_closer, landing) = finish_closing(&mut world, goal.task, &branch);
    assert_eq!(same_closer, closer);
    let close_changes = &closer_row(&world, closer).expect("closer row").close_changes;
    assert_eq!(close_changes.len(), 2);
    assert_eq!(close_changes[0], first);
    assert_ne!(close_changes[0], close_changes[1]);
    assert!(result_words(&world, goal.task).windows(b"The landing:".len()).any(|word| word == b"The landing:"));
    assert!(row(&world, landing).pull.is_some());
    settle(&mut world, goal.task);
    world.close();
}

#[test]
fn a_closer_reports_the_goals_later_finish_after_an_escalation() {
    let (mut world, goal, branch) = goal_world(4);
    let _note = write_goal_report(&mut world, &goal, 93, REPORT_WITH_LEFT, None);
    finish_run(&mut world, &goal, tasks::TaskResult::Report { words: b"First finish.".as_slice().into() });
    let closer = wait_report_held(&mut world, goal.task);
    let run = awakened(&mut world, goal.task);
    release(&mut world, &run, closer, 94);
    finish_run(&mut world, &run, tasks::TaskResult::Failure { reason: b"Second finish.".as_slice().into() });

    let (same, landing) = finish_closing(&mut world, goal.task, &branch);
    assert_eq!(same, closer);
    assert_report(&world, goal.task, closer, landing, &[]);
    let words = core::str::from_utf8(result_words(&world, goal.task)).expect("goal report UTF-8");
    assert!(words.ends_with("The goal's finish:\n  failed: Second finish.\n"), "{words}");
    assert!(!words.contains("First finish."), "{words}");
    settle(&mut world, goal.task);
    world.close();
}

fn goal_with_failed_close() -> (World, engine::Assignment, Box<[u8]>, u64, u64) {
    let (mut world, goal, branch) = goal_world(4);
    park_goal(&mut world, &goal);
    let closer = closer_of(&mut world, goal.task);
    let close = fail_close(&mut world);
    wait_held(&mut world, close, goal.task);
    let run = awakened(&mut world, goal.task);
    (world, run, branch, closer, close)
}

#[test]
fn a_released_close_waits_until_its_repair_lands() {
    let (mut world, run, branch, closer, first) = goal_with_failed_close();
    let (work, producer) = make_change(&mut world, &run, 93);
    release(&mut world, &run, first, 94);
    world.advance_ticks(100);
    assert!(world.jobs.is_empty(), "a fresh close job was assigned while the repair runs");
    assert_eq!(live_task(&world, first).phase, tasks::Phase::Active(tasks::Active::Idle));
    assert_eq!(row(&world, first).delegate, None);

    land_close_repair(&mut world, &run, &branch, work, &producer, first);
    let (second, job) = replacement_close(&mut world, run.task, closer, first, work);
    world.advance_ticks(100);
    assert!(world.jobs.is_empty(), "only the new settle change assigns a close job");
    finish_replacement_close(&mut world, run.task, &branch, closer, second, &job);
    world.close();
}

struct RefusedClose {
    world: World,
    run: engine::Assignment,
    closer: u64,
    close: u64,
    job: u64,
    at: change::CloseAt,
}

fn goal_at_close() -> (World, u64, u64, u64, host::JobAssignment, change::CloseAt) {
    let (mut world, goal, branch) = goal_world(4);
    let head = translate::commit(branch_tip(&mut world, &branch));
    let main = translate::commit(branch_tip(&mut world, b"main"));
    park_goal(&mut world, &goal);
    let closer = closer_of(&mut world, goal.task);
    let job = close_job(&mut world);
    let tasks::Party::Task(close) = live_task(&world, job.run.raw()).requester else { panic!("close change") };
    let at = change::CloseAt { branch: head, landing: main, asked: true, refused: None, outdated: None };
    (world, goal.task, closer, close, job, at)
}

fn goal_with_refused_close(failures: &[&str], more: usize, counts: Option<[u8; 32]>) -> RefusedClose {
    let (mut world, goal, closer, close, job, at) = goal_at_close();
    let result = refusal_with(at.branch, counts.map(|counts| (at.landing, counts)), failures, more);
    finish_with(&mut world, &job, &result);
    wait_held(&mut world, close, goal);
    let run = awakened(&mut world, goal);
    let at = change::CloseAt { refused: Some(change::CloseRefused { counts }), ..at };
    RefusedClose { world, run, closer, close, job: job.run.raw(), at }
}

fn short(commit: [u8; 32]) -> String {
    let digits = forge_top::commit_text(&commit);
    core::str::from_utf8(&digits[..12]).expect("short commit").to_owned()
}

fn refusal_words(at: change::CloseAt, failures: &[&str], more: usize) -> String {
    let branch = short(at.branch);
    let (base, counts) = match at.refused.expect("refusal fixture").counts {
        Some(counts) => (short(at.landing), short(counts)),
        None => ("no base".to_owned(), "none".to_owned()),
    };
    let names = failures.join(", ");
    let more = if more == 0 { String::new() } else { format!(" and {more} more") };
    format!(
        "the close check refused the goal's branch at {branch}, against main at {base} with counts {counts}: {names}{more}; it checks again once the goal's branch or main moves."
    )
}

fn replay_words(at: change::CloseAt) -> String {
    let branch = short(at.branch);
    let main = short(at.landing);
    let words = refusal_words(at, &[CLOSE_FAILURE], 0);
    format!(
        "not measured again: the goal's branch is at {branch} and main at {main}, as when its close check refused; it checks again once either moves.\n\n{words}"
    )
}

fn assert_refused_close(refusal: RefusedClose, words: &str) {
    let RefusedClose { mut world, run, close, job, at, .. } = refusal;
    world.advance_ticks(100);
    let ended = ended(&world, job).expect("one successful terminal at the first attempt");
    assert!(matches!(ended.phase, tasks::Phase::Ended(tasks::Ending::Done(tasks::TaskResult::Job { .. }))));
    assert!(world.jobs.is_empty(), "a judgment spent no retry");
    assert_eq!(goal_runs(&world, run.task), 2);
    assert_eq!(row(&world, close).close_at, Some(at));
    let change::State::Held { was, hold } = &row(&world, close).change.state else { panic!("close held at producing") };
    assert_eq!(**was, change::State::Producing { requested: true });
    assert_eq!(hold.cause, change::Cause::CloseRefused);
    assert_eq!(hold.words.as_ref(), words.as_bytes());
    super::goal_closer::assert_hold_words(&world, close, words);
    world.close();
}

#[test]
fn a_refused_close_holds_and_wakes_its_goal_once() {
    let refusal = goal_with_refused_close(&[CLOSE_FAILURE], 0, Some(CLOSE_COUNTS));
    let words = refusal_words(refusal.at, &[CLOSE_FAILURE], 0);
    assert_refused_close(refusal, &words);
}

#[test]
fn a_close_refusal_holds_all_failures_in_check_order() {
    let failures = [CLOSE_FAILURE, "test B::world rule_01", "test A::fuzzy rule_02"];
    let refusal = goal_with_refused_close(&failures, 0, Some(CLOSE_COUNTS));
    let words = refusal_words(refusal.at, &failures, 0);
    assert_refused_close(refusal, &words);
}

#[test]
fn a_close_refusal_counts_failures_that_do_not_fit() {
    const ALREADY_OMITTED: usize = 3; // Omitted by the check, before the engine fits the hold.
    const KEPT: usize = 1; // The first name fits; including the second would cross the hold bound.
    const LONG_NAME_BYTES: usize = 323; // 513 minus 169 bytes of fixed text/separators and 21 for the second name.
    let bound = usize::try_from(tasks::HOLD_ACCOUNT_BYTES).expect("hold bound");
    let prefix = "test A::rules ";
    let long_name = format!("{prefix}{}", "x".repeat(LONG_NAME_BYTES - prefix.len()));
    let failures = [long_name.as_str(), "test B::world rule_01"];
    // With no base or counts, the two names fit in the 512-byte JSON result, but not in the hold.
    let refusal = goal_with_refused_close(&failures, ALREADY_OMITTED, None);
    assert_eq!(refusal_words(refusal.at, &failures, ALREADY_OMITTED).len(), bound + 1);
    let (kept, omitted) = failures.split_at(KEPT);
    let words = refusal_words(refusal.at, kept, omitted.len() + ALREADY_OMITTED);
    assert!(words.len() <= bound);
    assert_refused_close(refusal, &words);
}

fn nine_failures() -> [String; 9] {
    core::array::from_fn(|index| {
        let package = if index % 2 == 0 { "temper-engine-domain-world" } else { "temper-worker-domain-world" };
        format!("test {package}::suite forge_routes::goal_closes::rule_{index:02}")
    })
}

fn assert_goal_failure_list(refusal: &RefusedClose, failures: &[&str], more: usize) {
    let head = short(refusal.at.branch);
    let names = failures.join(", ");
    let tail = if more == 0 { String::new() } else { format!(" and {more} more") };
    let prefix = "the close check refused the goal's branch at ";
    let words = format!("{prefix}{head}; every failure its result names, in the check's order: {names}{tail}.");
    let messages = &refusal.run.messages;
    assert!(
        messages
            .iter()
            .any(|word| matches!(word.kind, tasks::MessageKind::Escalation { task, .. } if task == refusal.close))
    );
    let lists: Vec<_> = messages
        .iter()
        .filter(|word| {
            word.kind == tasks::MessageKind::Words
                && word.words.starts_with(prefix.as_bytes())
                && word.words.windows(b"; every failure".len()).any(|part| part == b"; every failure")
        })
        .map(|word| word.words.as_ref())
        .collect();
    assert_eq!(lists, [words.as_bytes()], "the escalation wake offers one complete failure list");
}

#[test]
fn a_close_refusal_lists_nine_failures_at_the_goals_escalation_wake() {
    let failures = nine_failures();
    let failures = failures.each_ref().map(String::as_str);
    let sample = refusal_with([1; 32], Some(([2; 32], CLOSE_COUNTS)), &failures, 0);
    assert!((513..=8192).contains(&sample.len()));
    let refusal = goal_with_refused_close(&failures, 0, Some(CLOSE_COUNTS));
    assert_eq!(
        ended(&refusal.world, refusal.job).expect("finished close job").contract,
        tasks::Contract::Job { bytes: 8192 }
    );
    assert_goal_failure_list(&refusal, &failures, 0);
    let words = refusal_words(refusal.at, &failures[..4], 5);
    assert!(words.len() <= usize::try_from(tasks::HOLD_ACCOUNT_BYTES).expect("hold bound"));
    assert_refused_close(refusal, &words);
}

#[test]
fn a_close_refusal_lists_the_results_own_more_count_at_the_goal() {
    let failures = [CLOSE_FAILURE];
    let refusal = goal_with_refused_close(&failures, 3, Some(CLOSE_COUNTS));
    assert_goal_failure_list(&refusal, &failures, 3);
    let words = refusal_words(refusal.at, &failures, 3);
    assert_refused_close(refusal, &words);
}

#[test]
fn a_close_result_past_eight_kib_is_broke() {
    let (mut world, goal, _, close, mut job, at) = goal_at_close();
    let counts = Some((at.landing, CLOSE_COUNTS));
    let framing = refusal_with(at.branch, counts, &["test "], 0).len();
    let oversized = format!("test {}", "x".repeat(8193 - framing));
    let result = refusal_with(at.branch, counts, &[&oversized], 0);
    assert_eq!(result.len(), 8193);
    finish_with(&mut world, &job, &result);
    for _ in 0..world.env.limits.tasks.retries.invalid.retries {
        job = close_job(&mut world);
        finish_with(&mut world, &job, &result);
    }
    wait_held(&mut world, close, goal);
    let change::State::Held { hold, .. } = &row(&world, close).change.state else { panic!("close holds") };
    assert_eq!(hold.cause, change::Cause::Failed);
    assert!(hold.words.starts_with(b"the close check broke and didn't judge the goal: "));
    assert_eq!(row(&world, close).close_at, Some(at));
    world.close();
}

#[test]
fn a_close_refusal_naming_no_failure_is_broke() {
    const RESULT_PREFIX: usize = 64; // tooling.md, 4.2 and repairs-releases.md, 1
    let (mut world, goal, _, close, job, at) = goal_at_close();
    let result = refusal_with(at.branch, Some((at.landing, CLOSE_COUNTS)), &[], 0);
    finish_with(&mut world, &job, &result);
    wait_held(&mut world, close, goal);
    let prefix = core::str::from_utf8(&result[..RESULT_PREFIX]).expect("ASCII refusal");
    super::goal_closer::assert_hold_words(
        &world,
        close,
        &format!(
            "the close check broke and didn't judge the goal: it finished with {prefix}; its release runs it again."
        ),
    );
    assert_eq!(row(&world, close).close_at, Some(at));
    assert!(world.jobs.is_empty(), "the finished job spent no retry");
    world.close();
}

#[test]
fn a_restart_keeps_a_refused_closes_hold_and_inputs() {
    let RefusedClose { mut world, run, close, at, .. } =
        goal_with_refused_close(&[CLOSE_FAILURE], 0, Some(CLOSE_COUNTS));
    park_goal(&mut world, &run);
    restart_closing(&mut world);
    wait_held(&mut world, close, run.task);
    assert_eq!(row(&world, close).close_at, Some(at));
    super::goal_closer::assert_hold_words(&world, close, &refusal_words(at, &[CLOSE_FAILURE], 0));
    world.close();
}

#[test]
fn a_released_close_does_not_measure_unchanged_tips_again() {
    let RefusedClose { mut world, run, close, at, .. } =
        goal_with_refused_close(&[CLOSE_FAILURE], 0, Some(CLOSE_COUNTS));
    let words = replay_words(at);
    for call in [93, 94] {
        let revision = live_task(&world, close).revisions.hold;
        release(&mut world, &run, close, call);
        if call == 94 {
            park_goal(&mut world, &run);
            restart_closing(&mut world);
        }
        wait_held(&mut world, close, run.task);
        world.advance_ticks(100);
        assert!(world.jobs.is_empty(), "a fresh close job was assigned for unchanged tips");
        assert!(live_task(&world, close).revisions.hold > revision);
        let change::State::Held { hold, .. } = &row(&world, close).change.state else { panic!("close re-held") };
        assert_eq!(hold.cause, change::Cause::CloseRefused);
        assert_eq!(hold.words.as_ref(), words.as_bytes());
        super::goal_closer::assert_hold_words(&world, close, &words);
    }
    world.close();
}

#[test]
fn a_released_close_wakes_when_its_repair_changes_nothing() {
    let RefusedClose { mut world, run, closer, close, at, .. } =
        goal_with_refused_close(&[CLOSE_FAILURE], 0, Some(CLOSE_COUNTS));
    let words = replay_words(at);
    let (work, producer) = make_change(&mut world, &run, 93);
    release(&mut world, &run, close, 94);
    world.advance_ticks(100);
    assert_eq!(live_task(&world, close).phase, tasks::Phase::Active(tasks::Active::Idle));

    finish_run(&mut world, &producer, tasks::TaskResult::NoChange { words: b"No fix needed.".as_slice().into() });
    wait_ended(&mut world, work);
    wait_held(&mut world, close, run.task);
    super::goal_closer::assert_hold_words(&world, close, &words);
    assert_eq!(closer_row(&world, closer).expect("closer row").close_changes.as_ref(), &[close]);
    assert!(world.jobs.is_empty(), "no close job was needed before or after the wake");
    world.close();
}

fn land_work_during_held_close(world: &mut World, goal: &engine::Assignment, branch: &[u8]) -> (u64, u64, u64) {
    park_goal(world, goal);
    let closer = closer_of(world, goal.task);
    let first = fail_close(world);
    wait_held(world, first, goal.task);
    let run = awakened(world, goal.task);
    let (work, producer) = make_change(world, &run, 93);
    land_close_repair(world, &run, branch, work, &producer, first);
    (closer, first, work)
}

fn land_close_repair(
    world: &mut World,
    goal: &engine::Assignment,
    branch: &[u8],
    work: u64,
    producer: &engine::Assignment,
    first: u64,
) {
    park_goal(world, goal);
    let assigned = world.assigned.len();
    produce(world, work, branch, producer, b"Repair after close");
    // The successful change can wake the goal while the closer reads for its replacement.
    // Return the goal run's lent funds without releasing the obsolete close.
    for _ in 0..STEPS {
        if ended(world, first).is_some() {
            break;
        }
        if let Some(next) = world.assigned[assigned..].iter().find(|next| next.task == goal.task).cloned() {
            park_goal(world, &next);
            break;
        }
        world.tick();
    }
    wait_ended(world, first);
}

fn replacement_close(world: &mut World, goal: u64, closer: u64, first: u64, work: u64) -> (u64, host::JobAssignment) {
    let tasks::Phase::Ended(tasks::Ending::Cancelled { reason, .. }) =
        &ended(world, first).expect("superseded close").phase
    else {
        panic!("held close was not cancelled");
    };
    let expected = format!("superseded by the goal's new work: changes {work} landed into its branch");
    assert_eq!(reason.as_ref(), expected.as_bytes());
    let job = close_job(world);
    let tasks::Party::Task(second) = live_task(world, job.run.raw()).requester else { panic!("second close change") };
    assert_eq!(closer_row(world, closer).expect("closer row").close_changes.as_ref(), &[first, second]);
    let expected = format!("{{\"version\":1,\"goal\":{goal},\"changes\":[{{\"task\":{work},\"landed\":true}}]}}");
    assert_eq!(row(world, second).close.as_deref(), Some(expected.as_bytes()));
    (second, job)
}

fn finish_replacement_close(
    world: &mut World,
    goal: u64,
    branch: &[u8],
    closer: u64,
    second: u64,
    job: &host::JobAssignment,
) {
    finish_close(world, second, branch, job);
    wait_ended(world, second);
    let _landing = wait_landing(world, closer);
    let head = world.fake.branch(b"org/repo", branch).expect("goal branch");
    status(world, head, raw::Check::Passed);
    wait_ended(world, closer);
    // A coordinator run still live at the report acknowledges its cancellation.
    for _ in 0..STEPS {
        if ended(world, goal).is_some() {
            break;
        }
        if let tasks::Phase::Closing(tasks::Closing { stage: tasks::Stage::Run { attempt }, .. }) =
            live_task(world, goal).phase
        {
            let event = world.protocol.answer(CHANNEL, goal, attempt, 0, tasks::End::Parked, engine::Parts::none());
            world.send(event);
            break;
        }
        world.tick();
    }
    wait_ended(world, goal);
}

#[test]
fn landed_work_supersedes_an_unreleased_held_close() {
    let (mut world, goal, branch) = goal_world(4);
    let (closer, first, work) = land_work_during_held_close(&mut world, &goal, &branch);
    let (second, job) = replacement_close(&mut world, goal.task, closer, first, work);
    finish_replacement_close(&mut world, goal.task, &branch, closer, second, &job);
    let report = core::str::from_utf8(result_words(&world, closer)).expect("closer report UTF-8");
    assert!(!report.contains(&format!("- T{first} ")), "{report}");
    world.close();
}

#[test]
fn a_superseded_close_does_not_count_against_the_bound() {
    let (mut world, goal, branch) = goal_world_with_attempts(4, true, SINGLE_ATTEMPT);
    let (closer, _, _) = land_work_during_held_close(&mut world, &goal, &branch);
    let before = world.assigned.len();
    let second = fail_close(&mut world);
    wait_held(&mut world, second, goal.task);

    assert_eq!(closer_row(&world, closer).expect("closer row").closer.hold, None);
    let run = reviews::next_assignment(&mut world, before);
    cancel_step(&mut world, &run, second, 94);
    wait_held(&mut world, closer, goal.task);
    super::goal_closer::assert_hold_words(
        &world,
        closer,
        &format!(
            "the close step's change T{second} ended cancelled; attempts ended without landing: 1, the project's bound 1"
        ),
    );
    world.close();
}

fn held_landing() -> (World, engine::Assignment, Box<[u8]>, u64, u64, u64) {
    held_landing_with_attempts(engine::LANDING_ATTEMPTS)
}

fn held_landing_with_attempts(attempts: u8) -> (World, engine::Assignment, Box<[u8]>, u64, u64, u64) {
    let (mut world, goal, branch) = goal_world_with_attempts(4, true, attempts);
    park_goal(&mut world, &goal);
    let closer = closer_of(&mut world, goal.task);
    let job = close_job(&mut world);
    let tasks::Party::Task(close) = live_task(&world, job.run.raw()).requester else { panic!("close change") };
    finish_close(&mut world, close, &branch, &job);
    wait_ended(&mut world, close);
    let landing = wait_landing(&mut world, closer);
    let head = world.fake.branch(b"org/repo", &branch).expect("goal branch");
    let before = world.assigned.len();
    status(&mut world, head, raw::Check::Failed);
    let repair = reviews::next_assignment(&mut world, before);
    assert_eq!(live_task(&world, repair.task).requester, tasks::Party::Task(landing));
    let next = world.run_push_edit(&repair, &branch, b"repair", b"first repair", 1);
    status(&mut world, next, raw::Check::Pending);
    finish_run(
        &mut world,
        &repair,
        tasks::TaskResult::Change {
            title: tasks::ChangeTitle::new(b"Repair".as_slice().into()),
            connector: 1,
            kind: 2,
            resource: forge_top::tag(forge_world::REPO.forge, forge_world::REPO.repository),
            words: b"Repair pushed".as_slice().into(),
        },
    );
    for _ in 0..STEPS {
        if row(&world, landing).change.state == (change::State::Checking { head: translate::commit(next) }) {
            break;
        }
        world.tick();
    }
    status(&mut world, next, raw::Check::Failed);
    wait_held(&mut world, landing, goal.task);
    assert!(matches!(
        row(&world, landing).change.state,
        change::State::Held { hold: change::Hold { cause: change::Cause::Repairs, .. }, .. }
    ));
    let run = awakened(&mut world, goal.task);
    (world, run, branch, closer, close, landing)
}
fn cancel_step(world: &mut World, run: &engine::Assignment, task: u64, call: u64) {
    let answer = world.scripted_call(
        run,
        call,
        engine::Tool::Cancel { target: task, reason: b"Try the step again".as_slice().into() },
    );
    assert!(matches!(answer, engine::CallAnswer::Controlled), "{answer:?}");
    wait_ended(world, task);
}

fn assert_funds_wait(
    world: &mut World,
    run: &engine::Assignment,
    closer: u64,
    kind: temper_engine_domain_forge_closer::Step,
) {
    for _ in 0..STEPS {
        world.tick();
        if live_task(world, closer).phase == tasks::Phase::Active(tasks::Active::Idle)
            && closer_row(world, closer).expect("stepped closer").closer.hold.is_none()
            && closer_row(world, closer).expect("stepped closer").closer.retry == Some(kind)
        {
            break;
        }
    }
    assert!(matches!(
        live_task(world, run.task).phase,
        tasks::Phase::Active(tasks::Active::Claimed { .. } | tasks::Active::Running { .. })
    ));
    assert_eq!(
        live_task(world, closer).phase,
        tasks::Phase::Active(tasks::Active::Idle),
        "funds lent to the live goal run must wait, not hold"
    );
    assert_eq!(
        closer_row(world, closer).expect("waiting closer").closer.retry,
        Some(kind),
        "closer {:?}; numbers {:?}; errors {:?}",
        closer_row(world, closer),
        live_task(world, closer).numbers,
        world.call_errors
    );
    assert_eq!(closer_row(world, closer).expect("waiting closer").closer.hold, None);
}

#[test]
fn a_released_closer_waits_for_its_funds_lent_to_the_goals_run() {
    use temper_engine_domain_forge_closer::Step;
    let (mut world, goal, branch) = goal_world_with_attempts(4, true, 1);
    park_goal(&mut world, &goal);
    let closer = closer_of(&mut world, goal.task);
    let job = close_job(&mut world);
    let tasks::Party::Task(close) = live_task(&world, job.run.raw()).requester else { panic!("close change") };
    finish_close(&mut world, close, &branch, &job);
    wait_ended(&mut world, close);
    let landing = wait_landing(&mut world, closer);
    super::cancel_task(&mut world, landing);
    wait_held(&mut world, closer, goal.task);
    let run = awakened(&mut world, goal.task);
    release(&mut world, &run, closer, 94);
    assert_funds_wait(&mut world, &run, closer, Step::Land);
    assert_eq!(closer_row(&world, closer).expect("waiting closer").landing_changes.len(), 1);
    park_goal(&mut world, &run);
    let left = live_task(&world, closer).numbers;
    // Only the next landing may reserve funds after this park.
    let available = left.budget - left.spent - left.spent_below;
    for _ in 0..STEPS {
        if closer_row(&world, closer).expect("closer").landing_changes.len() == 2 {
            break;
        }
        world.tick();
    }
    let next = closer_row(&world, closer).expect("closer").closer.landing.expect("next landing");
    assert_ne!(next, landing);
    assert_eq!(live_task(&world, next).authority.budget.spend, available / 2);
    world.close();
}

#[test]
fn a_cancelled_stalled_landing_retries_without_holding_or_waking_its_goal() {
    let (mut world, goal, branch) = goal_world(4);
    park_goal(&mut world, &goal);
    let closer = closer_of(&mut world, goal.task);
    let job = close_job(&mut world);
    let tasks::Party::Task(close) = live_task(&world, job.run.raw()).requester else { panic!("close change") };
    finish_close(&mut world, close, &branch, &job);
    wait_ended(&mut world, close);
    let landing = wait_landing(&mut world, closer);
    let head = world.fake.branch(b"org/repo", &branch).expect("goal branch");
    status(&mut world, head, raw::Check::Pending);
    for _ in 0..STEPS {
        if matches!(row(&world, landing).change.state, change::State::Checking { .. }) {
            break;
        }
        world.tick();
    }
    assert!(matches!(row(&world, landing).change.state, change::State::Checking { .. }));
    super::stall_hold::execution_reads(&mut world, landing, skein_forge::api::RunState::Running);
    world.advance_to(world.env.now.saturating_add(world.env.limits.forge.change_policy.stall));
    wait_held(&mut world, landing, goal.task);
    assert!(matches!(
        row(&world, landing).change.state,
        change::State::Held { hold: change::Hold { cause: change::Cause::Stalled, .. }, .. }
    ));
    let run = awakened(&mut world, goal.task);
    cancel_step(&mut world, &run, landing, 93);
    assert_eq!(row(&world, landing).change.ended_held, Some(change::Cause::Stalled));
    world.advance_ticks(100);
    assert!(!matches!(live_task(&world, closer).phase, tasks::Phase::Held { .. }));
    park_goal(&mut world, &run);
    for _ in 0..STEPS {
        if closer_row(&world, closer).expect("closer").landing_changes.len() == 2 {
            break;
        }
        world.tick();
    }
    assert_eq!(closer_row(&world, closer).expect("closer").landing_changes.len(), 2);
    assert_eq!(goal_runs(&world, goal.task), 2, "the retry causes no further goal wake");
    assert_eq!(live_task(&world, closer).revisions.hold, 0);
    world.close();
}

fn closer_step_wait(world: &mut World, task: u64) -> Option<jig_domain_core::Wait> {
    use jig_domain_core::{WaitKind, documents, reads};
    let reads::Answer::Document(document) = world.party_read(reads::Read::Task { task }) else {
        panic!("live closer's document")
    };
    let documents::Document::Task(document) = *document else { panic!("task document") };
    document.waits.as_slice().iter().find(|wait| wait.kind == WaitKind::Step).cloned()
}

#[test]
fn a_closer_bounds_containment_only_after_its_landing_ends() {
    use jig_domain_core::{AtBound, Bound, WaitObject};
    use temper_client_protocol_wire::waits::CLOSER_LANDING;

    let (mut world, goal, branch) = goal_world(4);
    park_goal(&mut world, &goal);
    let (closer, landing) = begin_closing(&mut world, goal.task, &branch);
    let head = world.fake.branch(b"org/repo", &branch).expect("goal branch");
    status(&mut world, head, raw::Check::Pending);
    for _ in 0..STEPS {
        if matches!(row(&world, landing).change.state, change::State::Checking { .. }) {
            break;
        }
        world.tick();
    }
    super::stall_hold::execution_reads(&mut world, landing, skein_forge::api::RunState::Waiting);
    let figure = world.env.limits.forge.waits.step;
    world.jump(figure.saturating_add(Duration::from_secs(60)));
    assert!(matches!(live_task(&world, landing).phase, tasks::Phase::Active(_)));
    assert_eq!(live_task(&world, closer).revisions.hold, 0);
    assert_eq!(closer_step_wait(&mut world, closer), None);

    world.hold_ancestry_answers = true;
    super::cancel_task(&mut world, landing);
    let retry = ended(&world, landing).expect("cancelled landing").ended_at.expect("landing's end");
    // The retry reads the goal report before asking for fresh containment.
    for _ in 0..STEPS {
        if !world.held_ancestry_answers.is_empty() {
            break;
        }
        world.tick();
    }
    let engine::Event::Forge(mut answer) =
        world.held_ancestry_answers.pop().expect("the retry reads fresh containment")
    else {
        panic!("forge answer")
    };
    let forge_top::Event::Client(skein_forge_client::Event::Answered { result, .. }) = answer.as_mut() else {
        panic!("containment result")
    };
    // Park on an inconclusive read so the task projects its procedure wait.
    *result = Ok(skein_forge::api::Answer::Commits { commits: Box::new([]), more: true });
    world.events.push_front(engine::Event::Forge(answer));
    for _ in 0..STEPS {
        if live_task(&world, closer).phase == tasks::Phase::Active(tasks::Active::Idle) {
            break;
        }
        world.tick();
    }
    let wait = closer_step_wait(&mut world, closer).expect("the retry reports its containment wait");
    assert!(matches!(wait.object, WaitObject::Procedure { step: CLOSER_LANDING, .. }));
    assert_eq!(wait.since, retry);
    let at = skein_lib::Wall::from_nanos(retry.as_nanos().saturating_add(figure.as_nanos()));
    assert_eq!(wait.bound, Bound::At { at, then: AtBound::Hold });

    world.hold_ancestry_answers = false;
    world.events.extend(world.held_ancestry_answers.drain(..));
    for _ in 0..STEPS {
        if closer_row(&world, closer).expect("closer").landing_changes.len() == 2 {
            break;
        }
        world.tick();
    }
    assert_ne!(wait_landing(&mut world, closer), landing);
    assert_eq!(live_task(&world, closer).revisions.hold, 0);
    world.close();
}

#[test]
fn a_cancelled_landing_held_for_repairs_holds_below_the_attempt_bound() {
    let (mut world, run, _, closer, _, landing) = held_landing();
    cancel_step(&mut world, &run, landing, 93);
    wait_held(&mut world, closer, run.task);
    super::goal_closer::assert_hold_words(
        &world,
        closer,
        &format!("the landing T{landing} ended cancelled, held for its branch: repairs"),
    );
    release(&mut world, &run, closer, 94);
    park_goal(&mut world, &run);
    for _ in 0..STEPS {
        if closer_row(&world, closer).expect("closer").landing_changes.len() == 2 {
            break;
        }
        world.tick();
    }
    assert_eq!(closer_row(&world, closer).expect("closer").landing_changes.len(), 2);
    world.close();
}

#[test]
fn a_short_closer_names_its_lack_and_release_uses_the_goals_new_authority() {
    let minimum = stock_defaults().rules.minimum_run_spend;
    for spend_short in [true, false] {
        let left = minimum + 1;
        let (mut world, goal, branch, full) =
            goal_world_with_authority(4, true, 3, spend_short.then_some(left * 2), (!spend_short).then_some(2));
        // The run is admitted with `left`, then spends it so the closer still has too little.
        park_goal_spent(&mut world, &goal, if spend_short { left } else { 0 });
        let closer = closer_of(&mut world, goal.task);
        wait_held(&mut world, closer, goal.task);
        let run = awakened(&mut world, goal.task);
        if spend_short {
            assert_eq!(run.run.budget, left, "the whole funds the goal's run when half is too little");
        }
        let expected = if spend_short {
            format!(
                "the goal closer cannot delegate its next change within its authority: spend: {left} left, its change's half must pass {minimum}"
            )
        } else {
            "the goal closer cannot delegate its next change within its authority: tasks: 1 left, its change needs 2"
                .into()
        };
        let tasks::Phase::Held { source: Some(source), .. } = &live_task(&world, closer).phase else {
            panic!("short hold")
        };
        let tasks::HoldSource::Account { words, .. } = source.as_ref() else { panic!("short words") };
        assert!(words.starts_with(expected.as_bytes()), "{}", core::str::from_utf8(words).expect("hold UTF-8"));
        assert_eq!(
            closer_row(&world, closer).expect("short closer").closer.retry,
            Some(temper_engine_domain_forge_closer::Step::Close)
        );
        let revision = live_task(&world, goal.task).revisions.spec;
        world.send(engine::Event::Party(engine::PartyInput::Ask {
            models: Box::new([]),
            reply_to: ReplyTo::new(Token::new(97)),
            sign_in: world.signed_in.expect("owner"),
            key: [97; 16],
            ask: people::Ask::Amend {
                project: 1,
                task: goal.task,
                revision,
                release: None,
                amendment: people::Amendment {
                    tracked: None,
                    addressed: None,
                    instructions: None,
                    spec: None,
                    wake: None,
                    dependencies: None,
                    authority: Some(full),
                    llm: None,
                    reason: b"Fund the next closing step".as_slice().into(),
                },
            },
        }));
        world.until(Until::Ready);
        // The goal widens its delegate's terms as well as replenishing its funds.
        let mut authority = live_task(&world, closer).authority.clone();
        let funds = live_task(&world, goal.task).numbers;
        authority.budget.spend = funds.budget - funds.spent - funds.spent_below - funds.reserved;
        authority.delegation.tasks = live_task(&world, goal.task).authority.delegation.tasks;
        let revision = live_task(&world, closer).revisions.spec;
        let amendment =
            temper_engine_domain_world::fixtures::task_amendment(None, Some(authority), b"Fund the closer's change");
        let answer = world.scripted_call(
            &run,
            93,
            engine::Tool::Amend { target: closer, revision, amendment, keep_parameters: false },
        );
        assert!(matches!(answer, engine::CallAnswer::Amended { .. }), "{answer:?}");
        release(&mut world, &run, closer, 94);
        park_goal(&mut world, &run);
        for _ in 0..STEPS {
            if !world.jobs.is_empty() {
                break;
            }
            world.tick();
        }
        assert!(
            !world.jobs.is_empty(),
            "spend short {spend_short}; closer {:?}; row {:?}",
            live_task(&world, closer).phase,
            closer_row(&world, closer)
        );
        let job = close_job(&mut world);
        let tasks::Party::Task(close) = live_task(&world, job.run.raw()).requester else { panic!("close change") };
        assert_eq!(live_task(&world, close).requester, tasks::Party::Task(closer));
        finish_close(&mut world, close, &branch, &job);
        wait_ended(&mut world, close);
        wait_landing(&mut world, closer);
        let head = world.fake.branch(b"org/repo", &branch).expect("goal branch");
        status(&mut world, head, raw::Check::Passed);
        wait_ended(&mut world, goal.task);
        world.close();
    }
}

fn work_after_escalation(world: &mut World, run: &engine::Assignment, branch: &[u8], closer: u64, landing: u64) -> u64 {
    let (work, producer) = make_change(world, run, 93);
    assert_eq!(live_task(world, closer).waiting_on.as_ref(), &[work]);
    assert_eq!(live_task(world, closer).phase, tasks::Phase::Active(tasks::Active::Idle));
    let before = closer_row(world, closer).expect("closer row").close_changes.clone();
    let head = world.fake.branch(b"org/repo", branch).expect("goal branch");
    status(world, head, raw::Check::Pending);
    release(world, run, landing, 94);
    park_goal(world, run);
    assert_eq!(closer_row(world, closer).expect("closer row").close_changes, before);
    produce(world, work, branch, &producer, b"New work");
    let head = world.fake.branch(b"org/repo", branch).expect("goal branch");
    for _ in 0..STEPS {
        if row(world, landing).change.last_head == Some(translate::commit(head)) {
            break;
        }
        world.tick();
    }
    assert_eq!(row(world, landing).change.last_head, Some(translate::commit(head)), "the landing takes the new head");
    assert_eq!(goal_runs(world, run.task), 2);
    work
}

#[test]
fn a_held_landing_wakes_the_goal_once() {
    let (mut world, run, branch, closer, _, landing) = held_landing();
    let work = work_after_escalation(&mut world, &run, &branch, closer, landing);
    let (same_closer, _) = finish_closing(&mut world, run.task, &branch);
    assert_eq!(same_closer, closer);
    let last_landing = *closer_row(&world, closer).expect("closer row").landing_changes.last().expect("goal landing");
    assert_report(&world, run.task, closer, last_landing, &[(work, b"New work")]);
    settle(&mut world, run.task);
    world.close();
}

#[test]
fn a_goal_woken_by_an_escalation_that_makes_a_change_closes_after_it() {
    let (mut world, run, branch, closer, first, landing) = held_landing();
    let work = work_after_escalation(&mut world, &run, &branch, closer, landing);
    assert!(ended(&world, run.task).is_none(), "new work requires another close before the report");
    let job = close_job(&mut world);
    let tasks::Party::Task(second) = live_task(&world, job.run.raw()).requester else { panic!("second close change") };
    assert_ne!(second, first);
    let expected = format!("{{\"version\":1,\"goal\":{},\"changes\":[{{\"task\":{work},\"landed\":true}}]}}", run.task);
    assert_eq!(row(&world, second).close.as_deref(), Some(expected.as_bytes()));
    assert_eq!(closer_row(&world, closer).expect("closer row").close_changes.as_ref(), &[first, second]);
    finish_close(&mut world, second, &branch, &job);
    wait_ended(&mut world, second);
    let head = world.fake.branch(b"org/repo", &branch).expect("goal branch");
    status(&mut world, head, raw::Check::Passed);
    wait_ended(&mut world, run.task);
    let last_landing = *closer_row(&world, closer).expect("closer row").landing_changes.last().expect("goal landing");
    assert_report(&world, run.task, closer, last_landing, &[(work, b"New work")]);
    settle(&mut world, run.task);
    world.close();
}
