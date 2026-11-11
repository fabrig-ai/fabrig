//! A stalled row holds its task and reaches its deciding holder
//! (temper/docs/design/domain/forge/change/repairs.md; jig/docs/design/domain/tasks/proposals.md).
use super::early_start::{observe_push, stop};
use super::questions::{next_producer, row};
use super::*;
use jig_domain_core::{documents, reads};
use temper_engine_domain_forge_change::{Hold, State};

const CHANNEL: Token = Token::new(7);
const STEPS: u32 = 1000; // A stall's read, durable hold, escalation and worker delivery.

fn preparing(goal: bool) -> (World, engine::Assignment, u64, engine::Assignment) {
    let (mut world, owner) = if goal {
        let mut world = World::configured_ci(true, true, true);
        world.adopt();
        world.send(protocol::hello_slots(CHANNEL, 2, Box::new([])));
        let people::Ask::Create { project, mut batch } = goal_branch::goal_request(&world, true) else {
            panic!("goal template")
        };
        // This stall waits for main's CI rule on a task-by-task goal.
        batch[0].symbols = Box::new([]);
        let goal = assigned_chat(&mut world, 92, people::Ask::Create { project, batch });
        (world, goal)
    } else {
        let mut world = World::configured_ci(true, true, true);
        world.adopt();
        world.send(protocol::hello_slots(CHANNEL, 2, Box::new([])));
        let people::Ask::Create { project, mut batch } = world.chat(1, b"Make the fix") else {
            panic!("chat template")
        };
        batch[0].flags.ended_by = people::EndedBy::Requester;
        let chat = assigned_chat(&mut world, 92, people::Ask::Create { project, batch });
        (world, chat)
    };
    let engine::CallAnswer::Delegated(made) = super::change_tool::change(&mut world, &owner, 93, None, b"Small fix")
    else {
        panic!("change admitted")
    };
    let task = made[0].task;
    let producer = next_producer(&mut world, task);
    (world, owner, task, producer)
}

fn waiting(world: &mut World, task: u64, producer: &engine::Assignment, ci: bool) {
    if ci {
        temper_engine_domain_world::kinds::produce(world, task, producer);
    } else {
        // The producer ends successfully but has pushed no head for the change to use.
        let event = world.protocol.answer(
            CHANNEL,
            producer.task,
            producer.attempt,
            5,
            tasks::End::Finished {
                result: tasks::TaskResult::Change {
                    connector: 0,
                    kind: 2,
                    resource: forge_top::tag(forge_world::REPO.forge, forge_world::REPO.repository),
                    title: tasks::ChangeTitle::new(b"Small fix".as_slice().into()),
                    words: b"done".as_slice().into(),
                },
                cancel_delegates: false,
            },
            engine::Parts::none(),
        );
        world.send(event);
        observe_push(world, producer);
    }
    for _ in 0..STEPS {
        world.tick();
        if world.store.rows.contains_key(&Key::Core(jig_domain_core::Key::Tasks(tasks::Key::Ended(producer.task))))
            && (!ci || matches!(row(world, task).change.state, State::Checking { .. }))
            && world.store.pending.is_empty()
        {
            break;
        }
    }
    if ci {
        assert!(
            matches!(row(world, task).change.state, State::Checking { .. }),
            "row={:?}; task={:?}",
            row(world, task).change,
            live_task(world, task).phase
        );
    } else {
        assert_eq!(row(world, task).change.state, State::Producing { requested: true });
        assert_eq!(row(world, task).change.last_head, None);
    }
}

fn stall(world: &mut World, task: u64) {
    world.env.now = world.env.now.saturating_add(world.env.limits.forge.change_policy.stall);
    for _ in 0..STEPS {
        world.tick();
        if matches!(live_task(world, task).escalation, tasks::Escalation::Waiting { .. }) {
            break;
        }
    }
    assert_stalled(world, task);
}

fn assert_stalled(world: &World, task: u64) {
    assert!(matches!(row(world, task).change.state, State::Held { why: Hold::Stalled, .. }));
    assert!(
        matches!(live_task(world, task).phase, tasks::Phase::Held { why: tasks::Hold::Stalled, .. }),
        "held row must hold its task: {:?}",
        live_task(world, task).phase
    );
}

fn goal_entry(world: &World, task: u64, goal: u64) -> u64 {
    let tasks::Escalation::Waiting { holder, entry, revision, .. } = live_task(world, task).escalation else {
        panic!("stall escalates")
    };
    assert_eq!(holder, tasks::EscalationHolder::Task(goal));
    assert_eq!(revision, live_task(world, task).revisions.hold);
    entry
}

fn heard_live(world: &mut World, goal: &engine::Assignment, entry: u64) {
    for _ in 0..STEPS {
        if world.delivered_messages.get(&(goal.task, goal.attempt)).is_some_and(|words| words.contains(&entry)) {
            break;
        }
        world.tick();
    }
    assert!(
        world.delivered_messages.get(&(goal.task, goal.attempt)).is_some_and(|words| words.contains(&entry)),
        "running goal hears the escalation"
    );
}

fn person_inbox(world: &mut World, task: u64) {
    let tasks::Escalation::Waiting { holder: tasks::EscalationHolder::Person(_), revision, .. } =
        live_task(world, task).escalation
    else {
        panic!("stall reaches a person: {:?}", live_task(world, task).escalation)
    };
    let answer = world.party_read(reads::Read::Inbox {
        group: reads::Group::Escalations,
        filter: reads::Filter { project: Some(1), goal: None },
        from: None,
    });
    let reads::Answer::Page(page) = answer else { panic!("person's escalation inbox: {answer:?}") };
    assert!(
        page.rows.iter().any(|row| matches!(row, reads::PageRow::Inbox(entry)
        if matches!(&entry.task, documents::TaskLink::Live(chip) if chip.number == task)
        && entry.object == documents::InboxObject::Escalation { revision })),
        "inbox: {page:?}"
    );
}

fn close(mut world: World, owner: &engine::Assignment) {
    world.signed_in = None;
    world.sign_in();
    stop(&mut world, owner);
}

#[test]
fn a_stalled_update_names_its_wait_and_reaches_its_goal() {
    // A tiny stall reaches the wait before the pending write's ten-second lifetime.
    let (mut world, goal, _, _, task) =
        super::goal_closer::updating_landing_with_stall(false, Some(Duration::from_secs(5)));
    let State::Updating { from, base, .. } = row(&world, task).change.state else { panic!("updating") };
    world.hold_write_answers = Some(HoldWrite::All);
    for _ in 0..STEPS {
        world.tick();
        if !world.held_write_answers.is_empty() {
            break;
        }
    }
    assert!(!world.held_write_answers.is_empty(), "the forge's update answer remains outstanding");
    stall(&mut world, task);
    goal_entry(&world, task, goal.task);
    let tasks::Phase::Held { source: Some(source), .. } = &live_task(&world, task).phase else {
        panic!("stall account")
    };
    let tasks::HoldSource::Account { words, .. } = source.as_ref() else { panic!("stall's wait") };
    let from = forge_top::commit_text(&from);
    let base = forge_top::commit_text(&base);
    assert_eq!(
        words.as_ref(),
        format!(
            "no progress at updating for 0 minutes: waiting for its update of {} with main at {}",
            core::str::from_utf8(&from[..12]).unwrap(),
            core::str::from_utf8(&base[..12]).unwrap()
        )
        .as_bytes()
    );
    world.hold_write_answers = None;
    world.events.extend(world.held_write_answers.drain(..));
    close(world, &goal);
}

#[test]
fn a_stalled_change_holds_its_task_and_wakes_its_idle_goal() {
    let (mut world, goal, task, producer) = preparing(true);
    waiting(&mut world, task, &producer, true);
    world.until(Until::Ready);
    // A goal has no CI news to read while its change waits for checks.
    let read = live_task(&world, goal.task).inbox.last().map(|word| word.number);
    world.send(protocol::turn(CHANNEL, goal.task, goal.attempt, 1, 0, read, b"Wait for CI"));
    world.until(Until::Ready);
    park_after_turn(&mut world, &goal, read, 1);
    world.until(Until::Ready);
    assert_eq!(live_task(&world, goal.task).phase, tasks::Phase::Active(tasks::Active::Idle));
    let assigned = world.assigned.len();
    stall(&mut world, task);
    let entry = goal_entry(&world, task, goal.task);
    let resumed = super::reviews::next_assignment(&mut world, assigned);
    assert_eq!(resumed.task, goal.task);
    assert!(resumed.attempt > goal.attempt);
    assert!(resumed.messages.iter().any(|word| word.number == entry), "new run hears the escalation");
    close(world, &goal);
}

#[test]
fn a_stalled_change_reaches_its_goals_live_run() {
    for ci in [true, false] {
        let (mut world, goal, task, producer) = preparing(true);
        waiting(&mut world, task, &producer, ci);
        let assigned = world.assigned.len();
        stall(&mut world, task);
        let entry = goal_entry(&world, task, goal.task);
        heard_live(&mut world, &goal, entry);
        assert_eq!(world.assigned.len(), assigned, "running goal keeps its run");
        close(world, &goal);
    }
}

#[test]
fn a_stalled_change_with_no_goal_reaches_its_persons_inbox() {
    let (mut world, chat, task, producer) = preparing(false);
    waiting(&mut world, task, &producer, true);
    stall(&mut world, task);
    person_inbox(&mut world, task);
    close(world, &chat);
}

#[test]
fn an_undecided_stall_passes_up_to_a_person() {
    let (mut world, goal, task, producer) = preparing(true);
    waiting(&mut world, task, &producer, true);
    stall(&mut world, task);
    goal_entry(&world, task, goal.task);
    world.env.now = world.env.now.saturating_add(world.env.limits.tasks.escalation_stall);
    for _ in 0..STEPS {
        world.tick();
        if matches!(
            live_task(&world, task).escalation,
            tasks::Escalation::Waiting { holder: tasks::EscalationHolder::Person(_), .. }
        ) {
            break;
        }
    }
    assert_stalled(&world, task);
    world.signed_in = None;
    world.sign_in();
    person_inbox(&mut world, task);
    close(world, &goal);
}

#[test]
fn a_hold_decided_while_its_procedure_is_not_due_still_holds_its_task() {
    let (mut world, goal, task, producer) = preparing(true);
    waiting(&mut world, task, &producer, true);
    world.env.now = world.env.now.saturating_add(world.env.limits.forge.change_policy.stall);
    let mut read = None;
    for _ in 0..STEPS {
        if let Some(index) = world.events.iter().position(|event| matches!(event,
            engine::Event::Forge(event) if matches!(event.as_ref(),
                forge_top::Event::Client(skein_forge_client::Event::Answered { result: Ok(skein_forge::api::Answer::Checks(_)), .. })))) {
            read = world.events.remove(index);
            break;
        }
        world.tick();
    }
    let read = read.expect("stall's checks read waits before its decision");
    let tasks::Executor::Procedure { connector, code } = live_task(&world, task).executor else {
        panic!("change procedure")
    };
    world.send(engine::Event::Party(engine::PartyInput::ProcedureStep {
        task,
        step: live_task(&world, task).attempt + 1,
        connector,
        code,
        decision: engine::ProcedureStep::new(engine::StepEnd::Wait),
    }));
    for _ in 0..STEPS {
        if live_task(&world, task).phase == tasks::Phase::Active(tasks::Active::Idle) && world.store.pending.is_empty()
        {
            break;
        }
        world.tick();
    }
    assert_eq!(live_task(&world, task).phase, tasks::Phase::Active(tasks::Active::Idle));
    world.send(read);
    for _ in 0..STEPS {
        world.tick();
        if matches!(live_task(&world, task).escalation, tasks::Escalation::Waiting { .. }) {
            break;
        }
    }
    assert_stalled(&world, task);
    let entry = goal_entry(&world, task, goal.task);
    heard_live(&mut world, &goal, entry);
    close(world, &goal);
}
