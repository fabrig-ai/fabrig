//! A remade landing owns its own pull while its predecessor stays frozen
//! (temper/docs/design/domain/forge/change/README.md, 2; forge/keeping-up.md, 3;
//! temper/docs/design/domain/landing-history.md, 2.2).
use super::goal_branch::{branch_tip, forge_hold, running_goal};
use super::goal_landing::{ended, land_into_goal, mine, row, status};
use super::*;
use forge_top::history as h;
use skein_forge::api::{Read, Write};
use skein_lib::Queue;

const STEPS: u32 = 1000; // The forge world's bound for outstanding watch and gather answers.

fn quiet(world: &mut World) {
    world.until(Until::Ready);
    for _ in 0..STEPS {
        if world.root.quiescent()
            && world.pending.is_empty()
            && world.store.pending.is_empty()
            && world.events.is_empty()
        {
            return;
        }
        world.tick();
    }
    panic!("forge watches and gathers settle before observation");
}

#[test]
fn an_admitted_goal_push_survives_restart_before_it_reaches_the_forge() {
    use forge_top::writes::{Operation, Sender, WriteState};
    use temper_protocol_worker_wire::v1::Answer;

    let (mut world, goal, branch) = running_goal();
    let name = forge_hold(&world, &branch).expect("goal branch hold").name.clone();
    let before = branch_tip(&mut world, &branch);
    let commit = world.worker_commit(before, b"design", b"admitted before restart");
    let attempt = (Token::new(7), goal.task, goal.attempt);
    let tag = goal.workspace.repositories[0].tag;
    assert_eq!(world.ask_push(attempt, tag, &branch, Some(before), commit), Some(Answer::Admitted));
    let written = world.root.forge().nearest_write(&name).expect("admitted push").clone();
    assert_eq!(written.sender, Sender::Attempt { task: goal.task, attempt: goal.attempt });
    assert_eq!(
        written.operation,
        Operation::Push { before: Some(translate::commit(before)), after: translate::commit(commit) }
    );
    assert_eq!(written.state, WriteState::Admitted);

    world.drain_cut();
    world.restart_same();
    quiet(&mut world);
    assert_eq!(world.root.forge().nearest_write(&name), Some(&written), "the live lease survives unchanged");

    // The worker already received admission. It spends that lease, without a new ask.
    assert_eq!(
        world.external(1, raw::Op::Git(raw::Git::Push { branch: branch.clone(), commit, expected: Some(before) })),
        raw::Answer::Pushed(raw::Pushed::Pushed)
    );
    hint(&mut world, skein_forge::api::Change::BranchMoved { branch, head: translate::commit(commit), actor: 1 });
    for _ in 0..STEPS {
        world.tick();
        if world
            .root
            .forge()
            .write_entry(&name, written.order)
            .is_some_and(|entry| entry.state == WriteState::Confirmed)
        {
            break;
        }
    }
    assert_eq!(
        world.root.forge().nearest_write(&name),
        Some(&forge_top::writes::Written { state: WriteState::Confirmed, ..written })
    );
    world.close();
}

fn landing(world: &mut World, goal: &engine::Assignment, call: u64) -> u64 {
    let answer = mine(world, goal, call, br#"{"words":"Land the goal","branch":"mine"}"#);
    let Some(engine::CallAnswer::Delegated(made)) = answer else { panic!("landing admitted: {answer:?}") };
    made[0].task
}

fn opened(world: &mut World, task: u64) -> u64 {
    quiet(world);
    row(world, task).pull.unwrap_or_else(|| panic!("landing opens its own pull: {:?}", row(world, task)))
}

fn read_pull(world: &mut World, number: u64, from: usize) {
    for _ in 0..STEPS {
        if world
            .forge_reads
            .as_ref()
            .expect("record numbered reads")
            .iter()
            .skip(from)
            .any(|(_, read)| matches!(read, Read::Pull { number: read, .. } if *read == number))
        {
            quiet(world);
            return;
        }
        world.tick();
    }
    panic!("pull #{number} is read by its number: {:?}", world.forge_reads);
}

fn assert_pull(world: &mut World, number: u64, state: raw::State, branch: &[u8], head: u64) {
    let raw::Answer::Pull(pull) = world.external(1, raw::Op::Read(raw::Read::Pull { number })) else {
        panic!("landing's pull #{number}")
    };
    assert_eq!((pull.state, pull.head.as_ref(), pull.commit, pull.merged), (state, branch, head, None));
}

fn opens(world: &World) -> usize {
    world.writes.iter().filter(|write| matches!(write, Write::OpenPull { .. })).count()
}

fn writes_without_projection(world: &World, goal: u64) -> Vec<Write> {
    let Some(Record::Forge { row }) = world.store.rows.get(&Key::Forge(forge_top::Key::Issue(goal))) else {
        panic!("goal issue")
    };
    let forge_top::Stored::Issue(issue) = row.as_ref() else { panic!("goal issue row") };
    let issue = issue.number.expect("goal issue opened");
    world
        .writes
        .iter()
        .filter(|write| {
            !matches!(write,
        Write::Edit { number, .. } | Write::Post { number, .. } if *number == issue)
        })
        .cloned()
        .collect()
}

fn history(world: &World, task: u64) -> Vec<h::Event> {
    world
        .store
        .rows
        .values()
        .filter_map(|record| match record {
            Record::Forge { row } => match row.as_ref() {
                forge_top::Stored::HistoryEvent(event) if event.change == task => Some(event.clone()),
                _ => None,
            },
            Record::Core(_) => None,
        })
        .collect()
}

fn refused_reopen(world: &mut World, pull: u64) {
    // World::external expects success; this call must expose Forgejo's refusal.
    let mut config = world.fake_env.limits.clone();
    config.latency_min = Duration::ZERO;
    config.latency_max = Duration::ZERO;
    let env = world.fake_env.with(config);
    let mut out = Queue::uncapped();
    let call = Token::new(10_001);
    fake::step(
        &mut world.fake,
        &env,
        fake::Event::Call {
            reply_to: ReplyTo::new(call),
            user: 2,
            repository: b"org/repo".as_slice().into(),
            op: raw::Op::Write(raw::Write::Reopen { number: pull }),
        },
        &mut out,
    );
    if world.fake.is_due(env.now) {
        fake::fire(&mut world.fake, &env, &mut out);
    }
    assert_eq!(out.pop(), Some(fake::Request::Reply { to: ReplyTo::new(call), result: Err(raw::Error::Exists) }));
    assert!(out.is_empty());
}

fn hint(world: &mut World, change: skein_forge::api::Change) {
    world.send(engine::Event::Forge(Box::new(forge_top::Event::Hint {
        hint: skein_forge::api::Hint { repository: forge_world::REPO, change, key: None },
    })));
    quiet(world);
}

struct Remade {
    first: u64,
    second: u64,
    goal: u64,
    pull: u64,
    branch: Box<[u8]>,
    history: Vec<h::Event>,
    holds: (u64, u64),
    writes: Vec<Write>,
}

impl Remade {
    #[track_caller]
    fn hold_revisions(&self, world: &World) -> (u64, u64) {
        assert!(
            ended(world, self.second).is_none(),
            "B stays live: state={:?}; ending={:?}",
            row(world, self.second).change.state,
            ended(world, self.second).map(|task| &task.phase)
        );
        (live_task(world, self.second).revisions.hold, live_task(world, self.goal).revisions.hold)
    }

    #[track_caller]
    fn facts(&self, world: &World) {
        // The root consumes Request::Drift internally. Its effects replace a direct
        // observation: no new hold, no drift, no work from a spurious change wake.
        assert_eq!(self.hold_revisions(world), self.holds, "P adds no hold to B or its goal");
        assert_eq!(row(world, self.second).pull, Some(self.pull));
        assert_eq!(row(world, self.second).drift, None);
        assert!(live_task(world, self.second).holdings.is_empty(), "B takes no resources from the goal");
        assert!(!matches!(live_task(world, self.goal).phase, tasks::Phase::Held { .. }));
        assert_eq!(row(world, self.first).change.state, temper_engine_domain_forge_change::State::Ended);
        assert_eq!(forge_hold(world, &self.branch).expect("goal's forge hold").drift, None);
        assert_eq!(history(world, self.first), self.history, "A gains no history after ended");
        // The goal's issue projection can report the intentional stop and release.
        let writes = writes_without_projection(world, self.goal);
        assert_eq!(writes.len(), self.writes.len(), "B opens no third pull and writes nothing else");
        assert_eq!(writes, self.writes);
        let reads = world.forge_reads.as_ref().expect("record numbered reads");
        assert!(
            !reads.iter().any(|(_, read)| matches!(read, Read::PullFor { .. })),
            "numbered changes never look up branches: {reads:?}"
        );
    }

    #[track_caller]
    fn unheld(&self, world: &World) {
        self.facts(world);
        assert!(!matches!(live_task(world, self.second).phase, tasks::Phase::Held { .. }));
        assert!(!matches!(row(world, self.second).change.state, temper_engine_domain_forge_change::State::Held { .. }));
    }
}

#[test]
#[expect(
    clippy::too_many_lines,
    reason = "one story follows a remade landing through predecessor news, stop, release, restart and push"
)]
fn a_remade_goal_landing_ignores_its_ended_predecessors_news() {
    let (mut world, goal, branch) = running_goal();
    let head = land_into_goal(&mut world, &goal, &branch, 94, b"plan");
    status(&mut world, head, raw::Check::Pending);
    quiet(&mut world);
    let before = opens(&world);
    let first = landing(&mut world, &goal, 95);
    let p = opened(&mut world, first);
    assert_eq!(opens(&world), before + 1);
    assert_pull(&mut world, p, raw::State::Open, &branch, head);
    // Keep P's watch beside the goal's branch, including across restart.
    let resources = Box::new([
        forge_hold(&world, &branch).expect("goal branch watch").name.clone(),
        forge_top::Name {
            forge: forge_world::REPO.forge,
            repository: forge_world::REPO.repository,
            what: forge_top::What::Pull(p),
        },
    ]);
    world.send(engine::Event::Forge(Box::new(forge_top::Event::Names { task: goal.task, resources })));
    quiet(&mut world);
    let goal_hold = live_task(&world, goal.task).revisions.hold;
    cancel_task(&mut world, first);
    assert_eq!(live_task(&world, goal.task).revisions.hold, goal_hold, "A's cancelled pull never holds its goal");
    assert!(!matches!(live_task(&world, goal.task).phase, tasks::Phase::Held { .. }));
    assert_pull(&mut world, p, raw::State::Closed, &branch, head);
    assert!(matches!(
        ended(&world, first).expect("A ended").phase,
        tasks::Phase::Ended(tasks::Ending::Cancelled { .. })
    ));
    let frozen = history(&world, first);
    assert_eq!(frozen.last().expect("A's ended event").what, h::What::Ended { ending: h::Ending::Cancelled });
    assert_eq!(branch_tip(&mut world, &branch), head);
    let second = landing(&mut world, &goal, 96);
    let q = opened(&mut world, second);
    assert_ne!(q, p, "a closed predecessor is not the new landing's pull");
    assert_eq!(opens(&world), before + 2, "one opening per landing");
    assert_eq!(row(&world, first).branch, row(&world, second).branch);
    assert_eq!(row(&world, second).change.last_head, Some(translate::commit(head)));
    assert_pull(&mut world, q, raw::State::Open, &branch, head);
    world.forge_reads = Some(Vec::new());
    let mut remade = Remade {
        first,
        second,
        goal: goal.task,
        pull: q,
        branch: branch.clone(),
        history: frozen,
        holds: (live_task(&world, second).revisions.hold, live_task(&world, goal.task).revisions.hold),
        writes: writes_without_projection(&world, goal.task),
    };
    remade.unheld(&world);

    refused_reopen(&mut world, p);
    assert_pull(&mut world, p, raw::State::Closed, &branch, head);
    quiet(&mut world);
    remade.unheld(&world);
    hint(&mut world, skein_forge::api::Change::Item(p));
    read_pull(&mut world, p, 0);
    remade.unheld(&world);

    world.send(engine::Event::Party(engine::PartyInput::Ask {
        models: Box::new([]),
        reply_to: ReplyTo::new(Token::new(970)),
        sign_in: world.signed_in.expect("owner signed in"),
        key: [97; 16],
        ask: people::Ask::Stop { project: 1, task: second, reason: Box::from(b"Parking the task".as_slice()) },
    }));
    quiet(&mut world);
    assert!(matches!(live_task(&world, second).phase, tasks::Phase::Held { .. }));
    assert!(live_task(&world, second).placed.is_some(), "the party's stop holds B");
    assert!(live_task(&world, second).revisions.hold > remade.holds.0);
    remade.holds.0 = live_task(&world, second).revisions.hold;
    remade.facts(&world);
    world.forge_reads = Some(Vec::new());
    let revision = remade.holds.0;
    // A party's stop is the party's to release, not its goal's.
    let answer = world.scripted_call(&goal, 98, engine::Tool::Release { target: second, revision, again: None });
    assert!(matches!(answer, engine::CallAnswer::HoldOwner { task, .. } if task == second), "{answer:?}");
    remade.facts(&world);
    world.send(engine::Event::Party(engine::PartyInput::Ask {
        models: Box::new([]),
        reply_to: ReplyTo::new(Token::new(990)),
        sign_in: world.signed_in.expect("stopping party signed in"),
        key: [99; 16],
        ask: people::Ask::Release { project: 1, task: second, revision, again: None },
    }));
    quiet(&mut world);
    assert!(ended(&world, second).is_none(), "B remains live after release");
    remade.holds.0 = live_task(&world, second).revisions.hold;
    read_pull(&mut world, q, 0);
    remade.unheld(&world);

    world.forge_reads = Some(Vec::new());
    world.restart_same();
    quiet(&mut world);
    read_pull(&mut world, q, 0);
    assert_pull(&mut world, p, raw::State::Closed, &branch, head);
    assert_pull(&mut world, q, raw::State::Open, &branch, head);
    remade.unheld(&world);
    let reads = world.forge_reads.as_ref().expect("numbered reads after restart").len();
    hint(&mut world, skein_forge::api::Change::Item(p));
    read_pull(&mut world, p, reads);
    remade.unheld(&world);

    world.forge_reads = Some(Vec::new());
    let next = world.run_push_edit(&goal, &branch, b"design", b"goal's push", 1);
    assert_ne!(next, head);
    // Finish the fake's zero-latency run before placing this head's CI in waiting.
    world.external(1, raw::Op::Read(raw::Read::Pull { number: q }));
    let mut out = Queue::uncapped();
    if world.fake.is_due(world.fake_env.now) {
        fake::fire(&mut world.fake, &world.fake_env, &mut out);
    }
    assert!(out.is_empty(), "no engine calls remain outstanding at the push");
    status(&mut world, next, raw::Check::Pending);
    hint(
        &mut world,
        skein_forge::api::Change::BranchMoved { branch: branch.clone(), head: translate::commit(next), actor: 1 },
    );
    read_pull(&mut world, q, 0);
    remade.unheld(&world);
    assert_eq!(row(&world, second).change.last_head, Some(translate::commit(next)), "B follows the goal's new head");
    assert_eq!(opens(&world), before + 2);
    assert_pull(&mut world, q, raw::State::Open, &branch, next);
    world.close();
}
