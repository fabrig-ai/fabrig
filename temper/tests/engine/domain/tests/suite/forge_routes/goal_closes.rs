//! Stock template goals close without an agent's judgement after their work lands
//! (temper/docs/design/protocol/configuration/stock.md;
//! temper/docs/design/domain/forge/branches.md, 2; jig/docs/design/domain/tasks/lifecycle.md, 11).
use super::goal_branch::{branch_tip, stock_defaults};
use super::goal_closer::{
    close_job, closer_row, finish_close as push_close, producer_for, result_words, wait_ended, wait_held, wait_landing,
};
use super::goal_landing::{ended, finish_run, mine, row, status};
use super::*;
use jig_client::contract as c;
use temper_engine_domain_forge_change as change;

const CHANNEL: Token = Token::new(7);
const STEPS: u32 = 4000;
const WORDS: &[u8] = b"Build the draft\n\nDraft: PATH";

#[expect(
    clippy::too_many_lines,
    reason = "one fixture joins the stock template, job capability and person creating offer"
)]
fn goal_world_in(template: u32, branch: bool) -> (World, engine::Assignment, Box<[u8]>) {
    let defaults = stock_defaults();
    let model = stock::box_model();
    let mut limits = temper_engine_domain_world::forge_routes::box_limits();
    // One repair reaches the held landing without a long repair tree.
    limits.forge.change_policy.repairs = 1;
    let mut world = World::configured_stock_limits(&defaults, &model, &limits)
        .with_close_job(Some(b".temper/close"), true)
        .with_goal_closer()
        .with_landing_deployment(Box::new([engine::LandingRule {
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
                last: authority::Last::Exact(b"main".as_slice().into()),
            },
            ci: true,
            up_to_date: false,
            gates: Box::new([]),
            approvals: Box::new([]),
        }]));
    world.adopt();
    let mut event = protocol::hello_slots(CHANNEL, 4, Box::new([]));
    let engine::Event::Worker(engine::WorkerInput::Link { link: engine::Link::Hello { hello, .. }, .. }) = &mut event
    else {
        panic!("hello")
    };
    hello.bounds.turns = 800;
    hello.bounds.time = Duration::from_secs(28_800);
    hello.bounds.spend = defaults.rules.maximum_run_spend;
    hello.bounds.message_bytes = world.env.limits.tasks.message_bytes;
    hello.bounds.inbox = world.env.limits.tasks.run_inbox;
    hello.bounds.endpoints = Box::new([jig_domain_link::EndpointBound {
        endpoint: model.endpoint,
        window: model.window,
        output: model.output,
    }]);
    hello.jobs = Box::new([jig_domain_link::JobBound {
        kind: jig_domain_link::JobKind(forge_top::CLOSE_KIND),
        spec: 16_384,
        result: 512,
        deadline: Duration::from_secs(600),
    }]);
    world.send(event);
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
    };
    let wire = jig_protocol_engine_clients::translate::owner::frame(&frame, &[catalogue], link.wire())
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
    if !branch {
        member.symbols = Box::new([]);
    }
    let goal = assigned_chat(&mut world, 92, people::Ask::Create { project, batch: Box::new([member]) });
    world.until(Until::Ready);
    let branch = if template == 4 && branch {
        format!("temper/{}/g{}", goal.task, goal.task).into_bytes().into_boxed_slice()
    } else {
        b"main".as_slice().into()
    };
    (world, goal, branch)
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
    wait_ended(world, task);
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
    assert_eq!(spec.command(), b"repo/.temper/close");
    assert_eq!(Some(spec.request()), row(world, close).close.as_deref());
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
fn finish_closing(world: &mut World, goal: u64, branch: &[u8]) -> (u64, u64) {
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
    let head = world.fake.branch(b"org/repo", branch).expect("goal branch awaiting its landing");
    status(world, head, raw::Check::Passed);
    wait_ended(world, goal);
    (closer, landing)
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

fn park_goal(world: &mut World, run: &engine::Assignment) {
    world.scripted_read_inbox(run);
    let turns = live_task(world, run.task).turn;
    super::park_after_turn(world, run, None, turns);
    world.until(Until::Ready);
}

fn awakened(world: &mut World, goal: u64) -> engine::Assignment {
    for _ in 0..STEPS {
        if let Some(run) = world.assigned.iter().filter(|run| run.task == goal).nth(1) {
            assert_eq!(goal_runs(world, goal), 2);
            assert!(live_task(world, goal).reasons.contains(tasks::Reasons::ESCALATION));
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
    for _ in 0..100 {
        world.tick();
    }
    assert_eq!(goal_runs(world, goal), 2, "the escalation wakes the goal exactly once");
    assert!(ended(world, goal).is_some());
}

#[test]
fn a_failing_close_command_wakes_the_goal_once() {
    let (mut world, goal, branch) = goal_world(4);
    park_goal(&mut world, &goal);
    let closer = closer_of(&mut world, goal.task);
    let mut close = None;
    for _ in 0..=world.env.limits.tasks.retries.permanent.retries {
        let job = close_job(&mut world);
        let tasks::Party::Task(task) = live_task(&world, job.run.raw()).requester else { panic!("close change") };
        close = Some(task);
        let event = world.protocol.answer_job(
            CHANNEL,
            job.run.raw(),
            job.attempt.raw(),
            host::JobEnding::Failed {
                failure: host::JobFailure::Failed,
                detail: b"status command exited 1".as_slice().into(),
                work: host::Work { left: None, saved: None },
            },
        );
        world.send(event);
    }
    let first = close.expect("close attempted");
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

fn held_landing() -> (World, engine::Assignment, Box<[u8]>, u64, u64, u64) {
    let (mut world, goal, branch) = goal_world(4);
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
    let next = fake::advance(&mut world.fake, &world.fake_env, b"org/repo", &branch, b"repair", b"first repair", 1)
        .expect("repair pushed");
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
    assert!(matches!(row(&world, landing).change.state, change::State::Held { why: change::Hold::Repairs, .. }));
    let run = awakened(&mut world, goal.task);
    (world, run, branch, closer, close, landing)
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
