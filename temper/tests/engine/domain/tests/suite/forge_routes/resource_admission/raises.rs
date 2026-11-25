//! The service face offers raised figures only after the root's durable word.
use super::*;

fn raise(world: &mut World, member: &[u8], figure: u32) {
    use jig_domain_core::settings as s;
    let revision = world.root.settings_revision();
    let values = Box::new([s::Override {
        name: member.into(),
        value: u64::from(figure),
        reason: b"room for held work".as_slice().into(),
    }]);
    let item = temper_engine_domain_world::kinds::item(22, 0, 1, &forge_top::settings::Value::Limits(values.clone()));
    world.send(temper_engine_domain_world::kinds::change(
        world.signed_in.expect("signed-in owner"),
        revision,
        51,
        Box::new([item]),
    ));
    assert!(world.limit_settings.is_empty(), "acceptance alone sends no raised figures");
    for _ in 0..STEPS {
        if !world.limit_settings.is_empty() {
            break;
        }
        world.tick();
    }
    let Some(engine::Request::LimitSettingsCommitted { revision: durable, jig, temper }) = world.limit_settings.pop()
    else {
        panic!("the raise must be accepted and durable: {:?}", world.web_replies);
    };
    assert_eq!(durable, revision + 1);
    assert!(jig.is_empty());
    assert_eq!(temper.as_ref(), values.as_ref());
    match member {
        b"forge.entries" => world.env.limits.forge.entries = figure,
        b"forge.holds" => world.env.limits.forge.holds = figure,
        b"forge.tasks" => world.env.limits.forge.tasks = figure,
        _ => unreachable!("this world's three held-work cases"),
    }
    // This world plays the service, whose decision is tested in the component.
    world.send(engine::Event::LimitsRaised);
}

fn producer(world: &mut World, after: usize, task: u64) -> engine::Assignment {
    for _ in 0..STEPS {
        world.tick();
        if let Some(run) = world
            .assigned
            .iter()
            .skip(after)
            .find(|run| live_task(world, run.task).requester == tasks::Party::Task(task))
        {
            return run.clone();
        }
    }
    panic!("the released procedure assigns its producer: {:?}", live_task(world, task));
}

fn fill_outbox(world: &mut World, chat: &engine::Assignment) {
    let before = world.assigned.len();
    let engine::CallAnswer::Delegated(made) = super::super::change_tool::change(world, chat, 94, None, b"First fix")
    else {
        panic!("first change")
    };
    let first = made[0].task;
    let run = producer(world, before, first);
    world.hold_write_answers = Some(HoldWrite::All);
    let branch = row(world, first).branch.clone();
    super::super::change_tool::deliver_change(world, &run, &branch, b"First fix");
    for _ in 0..STEPS {
        if !world.held_write_answers.is_empty() {
            break;
        }
        world.tick();
    }
    assert!(!world.held_write_answers.is_empty(), "the first outbox entry stays in flight");
}

fn check_outbox_wait(world: &World, task: u64) {
    use jig_domain_core::{Bound, Unbounded, WaitKind, WaitObject};
    use skein_world::inspect::Moment;
    use temper_client_protocol_wire::waits::THING_ENTRY;

    let core = world.root.core();
    let row = live_task(world, task);
    let [number] = *world.root.forge().inspection_entries() else { panic!("one holder fills this outbox") };
    let entry = world.root.forge().inspection_entry(number).expect("holder");
    let waits = core.inspection_wait_records(row);
    let wait = waits.iter().find(|wait| wait.kind == WaitKind::Room).expect("forge room report reached the core");
    let WaitObject::Procedure { things, .. } = &wait.object else { panic!("procedure object") };
    assert_eq!(things.len(), 1);
    assert_eq!(things.as_slice()[0].what, THING_ENTRY);
    assert_eq!(things.as_slice()[0].number, number);
    assert_eq!(things.as_slice()[0].owner, Some(entry.task));
    assert_eq!(wait.bound, Bound::None(Unbounded::Behind(entry.task)));
    jig_inspect::records(core, Moment { seed: 91, at: world.env.now, iteration: 0 }, world.env.wall)
        .expect("the core's records pass with a change waiting on the outbox");
}

#[expect(clippy::too_many_lines, reason = "one story keeps its filled bound, durable raise and resumed work together")]
fn held_work(member: &[u8], figure: u32) {
    let owner = forge_top::user_key(1, 7_u64);
    let mut configured = World::configured(true, true);
    match member {
        b"forge.entries" => configured.env.limits.forge.entries = 1,
        b"forge.holds" => configured.env.limits.forge.holds = 1,
        b"forge.tasks" => configured.env.limits.forge.tasks = 1,
        _ => unreachable!("this world's three held-work cases"),
    }
    let mut world = configured.with_settings_owner(&owner);
    world.adopt();
    world.send(protocol::hello_slots(CHANNEL, SLOTS, Box::new([])));
    let ask = adoption_chat(&world);
    let chat = assigned_chat(&mut world, 92, ask);
    if member == b"forge.entries" {
        fill_outbox(&mut world, &chat);
    }
    let before = world.assigned.len();
    let engine::CallAnswer::Delegated(made) =
        super::super::change_tool::change(&mut world, &chat, 93, None, b"Small fix")
    else {
        panic!("change delegated")
    };
    let task = made[0].task;
    let mut delivered = false;
    for _ in 0..STEPS {
        world.tick();
        if (member == b"forge.entries" && row(&world, task).room.is_some())
            || world.root.forge_limit_holds().iter().any(|hold| hold.member.as_ref() == member)
        {
            break;
        }
        if !delivered
            && let Some(producer) = world
                .assigned
                .iter()
                .skip(before)
                .find(|run| live_task(&world, run.task).requester == tasks::Party::Task(task))
                .cloned()
        {
            if member == b"forge.entries" {
                world.hold_write_answers = Some(HoldWrite::All);
            }
            let branch = row(&world, task).branch.clone();
            super::super::change_tool::deliver_change(&mut world, &producer, &branch, b"Small fix");
            delivered = true;
        }
    }
    let waiting = if member == b"forge.entries" {
        assert!(row(&world, task).room.is_some(), "a full outbox waits for room without a limit hold");
        assert!(world.root.forge_limit_holds().is_empty());
        check_outbox_wait(&world, task);
        task
    } else {
        let waits = world.root.forge_limit_holds();
        let hold = waits
            .iter()
            .find(|hold| hold.member.as_ref() == member)
            .unwrap_or_else(|| panic!("{member:?} must hold work: {waits:?}; task {:?}", live_task(&world, task)));
        assert_eq!(hold.configured, 1);
        assert_eq!(hold.needed, 2);
        hold.task
    };
    raise(&mut world, member, figure);
    for _ in 0..STEPS {
        if row(&world, task).room.is_none() && !world.root.forge_limit_holds().iter().any(|hold| hold.task == waiting) {
            break;
        }
        world.tick();
    }
    assert!(
        row(&world, task).room.is_none() && !world.root.forge_limit_holds().iter().any(|hold| hold.task == waiting),
        "a raise resumes waiting work without room freeing"
    );
    world.hold_write_answers = None;
    for answer in std::mem::take(&mut world.held_write_answers) {
        world.send(answer);
    }
    if !delivered {
        let producer = producer(&mut world, before, task);
        let branch = row(&world, task).branch.clone();
        super::super::change_tool::land_change(&mut world, &producer, &branch);
    }
    for _ in 0..STEPS {
        if matches!(row(&world, task).change.state, change_step::State::Landed { .. }) {
            break;
        }
        world.tick();
    }
    assert!(
        matches!(row(&world, task).change.state, change_step::State::Landed { .. }),
        "held work completes: {:?}",
        row(&world, task)
    );
    assert!(world.merges > 0);
    if member == b"forge.entries" {
        for _ in 0..STEPS {
            if world.merges == 2 {
                break;
            }
            world.tick();
        }
        assert_eq!(world.merges, 2, "both outbox writers finish before the restart oracle");
    }
    world.close();
}

#[test]
fn an_entries_raise_resumes_held_work_without_restart() {
    held_work(b"forge.entries", 2);
}
#[test]
fn a_holds_raise_resumes_held_work_without_restart() {
    held_work(b"forge.holds", 4);
}
#[test]
fn a_tasks_raise_resumes_held_work_without_restart() {
    held_work(b"forge.tasks", 16);
}
