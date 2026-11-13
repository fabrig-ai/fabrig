//! Admission pressure and actionable holds through the root and fake Forgejo.
use super::*;
use skein_forge::api::{Op, Read};
use std::collections::BTreeSet;
use temper_engine_domain_forge_change as change_step;

const CHANNEL: Token = Token::new(7);
const STEPS: u32 = 1000; // The world's durable-round-trip allowance.
const CHAT_GAP: Duration = Duration::from_secs(11);
const DEPARTING: &[u8] = b"temper/departing";
const SLOTS: u32 = 8; // The world prices eight worker slots.
const TASKS: u32 = 12; // Three chats, changes and producers fit the twelve-task world.
const RESOURCES: u32 = 10; // Eight current watches, a departing read and replacement headroom.

fn row(world: &World, task: u64) -> &forge_top::ChangeRow {
    let Some(Record::Forge { row }) = world.store.rows.get(&Key::Forge(forge_top::Key::Change(task))) else {
        panic!("change row")
    };
    let forge_top::Stored::Change(row) = row.as_ref() else { panic!("change row") };
    row
}

pub(super) fn names(world: &World, task: u64) -> Box<[forge_top::Name]> {
    let Some(Record::Forge { row }) = world.store.rows.get(&Key::Forge(forge_top::Key::Names(task))) else {
        panic!("named resources")
    };
    let forge_top::Stored::Names { resources, .. } = row.as_ref() else { panic!("names row") };
    resources.clone()
}

fn watch_count(world: &World) -> u32 {
    let mut watches = BTreeSet::new();
    for record in world.store.rows.values() {
        if let Record::Forge { row } = record
            && let forge_top::Stored::Names { task, resources } = row.as_ref()
            && matches!(
                world.store.rows.get(&Key::Core(jig_domain_core::Key::Tasks(tasks::Key::Live(*task)))),
                Some(Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Live(_))))
            )
        {
            watches.extend(resources.iter().cloned());
        }
    }
    u32::try_from(watches.len()).expect("small watch set")
}

fn finish(world: &mut World, producer: &engine::Assignment) {
    let branch = producer.workspace.repositories[0].push.clone().expect("producer branch");
    world.external(2, raw::Op::Git(raw::Git::Create { branch: branch.clone(), commit: 1 }));
    let head = fake::advance(&mut world.fake, &world.fake_env, b"org/repo", &branch, b"file", b"fixed", 1)
        .expect("producer pushed");
    world.external(
        1,
        raw::Op::Write(raw::Write::Status {
            commit: head,
            context: b"build".as_slice().into(),
            state: raw::Check::Pending,
            description: Box::new([]),
            link: Box::new([]),
        }),
    );
    let event = world.protocol.answer(
        CHANNEL,
        producer.task,
        producer.attempt,
        5,
        tasks::End::Finished {
            result: tasks::TaskResult::Change {
                title: tasks::ChangeTitle::new(b"Change".as_slice().into()),
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
}

fn verifying(world: &mut World, producer: &engine::Assignment) -> u64 {
    let tasks::Party::Task(task) = live_task(world, producer.task).requester else { panic!("change requester") };
    finish(world, producer);
    for _ in 0..STEPS {
        world.tick();
        if row(world, task).pull.is_some()
            && matches!(row(world, task).change.state, change_step::State::Checking { .. })
        {
            return task;
        }
    }
    panic!("change did not reach verification: {:?}; {:?}", row(world, task), live_task(world, task).phase);
}

fn start_change(world: &mut World, key: u8) -> (u64, engine::Assignment) {
    let ask = adoption_chat(world);
    let chat = assigned_chat(world, key, ask);
    let before = world.assigned.len();
    let engine::CallAnswer::Delegated(made) =
        super::change_tool::change(world, &chat, u64::from(key), None, b"Small fix")
    else {
        panic!("change delegated")
    };
    let task = made[0].task;
    for _ in 0..STEPS {
        world.tick();
        if let Some(producer) = world
            .assigned
            .iter()
            .skip(before)
            .find(|run| live_task(world, run.task).requester == tasks::Party::Task(task))
        {
            return (task, producer.clone());
        }
    }
    panic!("producer not assigned");
}

pub(super) fn kept_branch(world: &World, branch: &[u8]) -> bool {
    world.store.rows.values().any(|record| {
        matches!(record,
        Record::Forge { row } if matches!(row.as_ref(),
            forge_top::Stored::Client(skein_forge_client::Stored::Live(live))
                if matches!(&live.watch.resource.what, skein_forge_client::What::Branch(name)
                    if name.as_ref() == branch)))
    })
}

pub(super) fn branch_reads(world: &World, branch: &[u8]) -> usize {
    world
        .forge_reads
        .as_ref()
        .expect("request log enabled")
        .iter()
        .filter(|(_, read)| matches!(read, Read::Branch { branch: name } if name.as_ref() == branch))
        .count()
}

#[test]
fn a_producing_change_watches_only_its_base_until_a_head_is_known() {
    let mut world = World::configured_limits(
        true,
        true,
        true,
        STEPS,
        Gate::default(),
        None,
        StoryLimits { stall: Some(Duration::from_secs(1800)), slots: Some(SLOTS), ..StoryLimits::default() },
    );
    world.forge_reads = Some(Vec::new());
    world.adopt();
    world.send(protocol::hello_slots(CHANNEL, SLOTS, Box::new([])));
    let (task, producer) = start_change(&mut world, 92);
    let branch = row(&world, task).branch.clone();
    let base = forge_top::Name {
        forge: 1,
        repository: 2,
        what: forge_top::What::Branch(Box::new([b"main".as_slice().into()])),
    };
    assert_eq!(names(&world, task).as_ref(), &[base]);
    assert!(!kept_branch(&world, &branch), "neither the change nor its producer keeps an unpushed branch");
    let before = branch_reads(&world, &branch);
    for _ in 0..STEPS {
        world.tick();
    }
    assert_eq!(branch_reads(&world, &branch), before, "no polling while the producer runs");
    assert!(kept_branch(&world, b"main"));
    assert_eq!(verifying(&mut world, &producer), task);
    assert!(kept_branch(&world, &branch));
    assert!(branch_reads(&world, &branch) > before, "the result reads the pushed branch afresh");
    let resources = names(&world, task);
    assert_eq!(resources.len(), 2);
    assert!(resources.iter().any(|name| name.what == forge_top::What::Pull(row(&world, task).pull.unwrap())));
    assert!(resources.iter().any(|name| matches!(&name.what, forge_top::What::Branch(parts)
        if parts.iter().map(AsRef::as_ref).eq(branch.split(|byte| *byte == b'/')))));
    world.close();
}

fn pressure(world: &mut World, first: u64, second: u64, later: u64) {
    // A procedure replaces its own watches when fresh facts return. Keep the
    // pressure under the chats, whose watches the scripted parent controls.
    let tasks::Party::Task(first) = live_task(world, first).requester else { panic!("first chat") };
    let tasks::Party::Task(second) = live_task(world, second).requester else { panic!("second chat") };
    let original = names(world, first);
    let mut expanded = original.to_vec();
    expanded.push(forge_top::Name {
        forge: 1,
        repository: 2,
        what: forge_top::What::Branch(Box::new([b"temper".as_slice().into(), b"departing".as_slice().into()])),
    });
    world.external(2, raw::Op::Git(raw::Git::Create { branch: DEPARTING.into(), commit: 1 }));
    world.slow_branch = Some(DEPARTING.into());
    world.send(engine::Event::Forge(Box::new(forge_top::Event::Names { task: first, resources: expanded.into() })));
    for _ in 0..STEPS {
        world.tick();
        if world
            .pending
            .values()
            .any(|op| matches!(op, Op::Read(Read::Branch { branch }) if branch.as_ref() == DEPARTING))
        {
            break;
        }
    }
    assert!(
        world
            .pending
            .values()
            .any(|op| matches!(op, Op::Read(Read::Branch { branch }) if branch.as_ref() == DEPARTING)),
        "watches={}; cap={}; first={:?}; names={:?}; pending={:?}",
        watch_count(world),
        world.env.limits.forge.client.resources,
        live_task(world, first).phase,
        names(world, first),
        world.pending
    );
    world.send(engine::Event::Forge(Box::new(forge_top::Event::Names { task: first, resources: original })));
    for _ in 0..SLOTS {
        world.tick();
    }
    // Leave one slot for the later change's first head. Its admission then fits
    // exactly, while the departing read still consumes one extra slot.
    let mut steady = names(world, second).to_vec();
    for index in 0..RESOURCES.saturating_sub(watch_count(world)).saturating_sub(1) {
        steady.push(forge_top::Name {
            forge: 1,
            repository: 2,
            // The first pull is verifying; the next opening will be pull 2.
            what: if index == 0 {
                forge_top::What::Pull(2)
            } else {
                forge_top::What::Issue(u64::from(index).saturating_add(100))
            },
        });
    }
    world.send(engine::Event::Forge(Box::new(forge_top::Event::Names { task: second, resources: steady.into() })));
    let resources = names(world, later);
    world.send(engine::Event::Forge(Box::new(forge_top::Event::Names { task: later, resources })));
}

#[test]
fn overlapping_changes_open_after_pressure_without_manual_release() {
    let configured = World::configured_limits(
        true,
        true,
        true,
        STEPS,
        Gate::default(),
        None,
        StoryLimits {
            tasks: Some(TASKS),
            slots: Some(SLOTS),
            stall: Some(Duration::from_secs(1800)),
            ..StoryLimits::default()
        },
    );
    let client = skein_forge_client::Limits { resources: RESOURCES, ..configured.env.limits.forge.client };
    let mut world = configured.with_forge_client(client);
    world.adopt();
    world.send(protocol::hello_slots(CHANNEL, SLOTS, Box::new([])));
    let (_, first_producer) = start_change(&mut world, 92);
    let first = verifying(&mut world, &first_producer);
    let second_started = world.env.now;
    let (second, _) = start_change(&mut world, 93);
    world.env.now = world.env.now.max(second_started.saturating_add(CHAT_GAP));
    world.env.wall = skein_lib::Wall::from_nanos(world.env.now.as_nanos());
    let (third, producer) = start_change(&mut world, 94);
    pressure(&mut world, first, second, third);
    finish(&mut world, &producer);
    for _ in 0..STEPS {
        world.tick();
        for task in [first, second, third] {
            assert!(
                !matches!(live_task(&world, task).phase, tasks::Phase::Held { .. }),
                "{task}: {:?}",
                live_task(&world, task).phase
            );
        }
        if row(&world, third).pull.is_some() {
            return;
        }
    }
    panic!("later change did not open its pull: {:?}", row(&world, third));
}

fn held_words(world: &World, task: u64) -> &[u8] {
    let tasks::Phase::Held { why: tasks::Hold::ResourceUnavailable, source: Some(source), .. } =
        &live_task(world, task).phase
    else {
        panic!("resource hold: {:?}", live_task(world, task).phase)
    };
    let tasks::HoldSource::Account { words, .. } = source.as_ref() else { panic!("hold account") };
    words
}

fn verified_world() -> (World, u64) {
    let (mut world, _, producer, _) = change_world(true, STEPS, None);
    let task = verifying(&mut world, &producer);
    (world, task)
}

fn await_hold(world: &mut World, task: u64) {
    for _ in 0..STEPS {
        world.tick();
        if matches!(live_task(world, task).phase, tasks::Phase::Held { .. }) {
            return;
        }
    }
    panic!("change was not held");
}

fn release(world: &mut World, task: u64) {
    let revision = live_task(world, task).revisions.hold;
    let before = world.forge_calls;
    send_release(world, task, revision);
    for _ in 0..STEPS {
        world.tick();
        if !matches!(live_task(world, task).phase, tasks::Phase::Held { .. }) && world.forge_calls > before {
            return;
        }
    }
    panic!("release did not resume the change and read afresh");
}

#[test]
fn persistent_watch_holds_name_the_limit_and_release_reads_afresh() {
    let (mut world, task) = verified_world();
    let resources = names(&world, task);
    let configured = world.env.limits.forge.resources_per_task;
    let mut oversized = resources.to_vec();
    for index in 0..configured.saturating_add(1).saturating_sub(u32::try_from(oversized.len()).expect("small set")) {
        oversized.push(forge_top::Name {
            forge: 1,
            repository: 2,
            what: forge_top::What::Issue(u64::from(index).saturating_add(100)),
        });
    }
    world.send(engine::Event::Forge(Box::new(forge_top::Event::Names { task, resources: oversized.into() })));
    await_hold(&mut world, task);
    let words = std::str::from_utf8(held_words(&world, task)).expect("hold words");
    assert!(words.contains("forge.resources_per_task = 4; required usage 5"), "{words}");
    assert!(
        words.contains("watch resources on branch") && words.contains("org/repo") && words.contains("then release"),
        "{words}"
    );
    world.send(engine::Event::Forge(Box::new(forge_top::Event::Names { task, resources })));
    release(&mut world, task);
}

#[test]
fn backend_read_holds_name_the_permission_failure_and_recover() {
    let (mut world, task) = verified_world();
    fake::grant(&mut world.fake, b"org/repo", 1, raw::Permission::None);
    let branch = row(&world, task).branch.clone();
    world.send(engine::Event::Forge(Box::new(forge_top::Event::Hint {
        hint: skein_forge::api::Hint {
            repository: forge_world::REPO,
            change: skein_forge::api::Change::Branch(branch),
            key: None,
        },
    })));
    await_hold(&mut world, task);
    let words = std::str::from_utf8(held_words(&world, task)).expect("hold words");
    assert!(words.contains("read branch on branch") && words.contains("org/repo"), "{words}");
    assert!(
        words.contains("forge refused permission")
            && words.contains("restore forge access or permission, then release"),
        "{words}"
    );
    fake::grant(&mut world.fake, b"org/repo", 1, raw::Permission::Admin);
    release(&mut world, task);
}

fn fill_chat_watches(world: &mut World, task: u64) {
    let mut resources = names(world, task).to_vec();
    resources.push(forge_top::Name { forge: 1, repository: 2, what: forge_top::What::Issue(100) });
    world.send(engine::Event::Forge(Box::new(forge_top::Event::Names { task, resources: resources.into() })));
    world.until(Until::Ready);
}

fn send_release(world: &mut World, task: u64, revision: u64) {
    world.send(engine::Event::Party(engine::PartyInput::Ask {
        models: Box::new([]),
        reply_to: ReplyTo::new(Token::new(950)),
        sign_in: world.signed_in.expect("owner"),
        key: [95; 16],
        ask: people::Ask::Release { project: 1, task, revision, again: None },
    }));
}

#[test]
fn released_changes_name_the_branch_only_with_a_known_head() {
    for pushed in [false, true] {
        let (mut world, _, producer, branch) = change_world(true, STEPS, None);
        let tasks::Party::Task(task) = live_task(&world, producer.task).requester else { panic!("change") };
        if pushed {
            verifying(&mut world, &producer);
        }
        let resources = (0..world.env.limits.forge.resources_per_task.saturating_add(1))
            .map(|index| forge_top::Name {
                forge: 1,
                repository: 2,
                what: forge_top::What::Issue(u64::from(index).saturating_add(100)),
            })
            .collect();
        world.send(engine::Event::Forge(Box::new(forge_top::Event::Names { task, resources })));
        await_hold(&mut world, task);
        world.send(engine::Event::Forge(Box::new(forge_top::Event::Unnamed { task })));
        world.until(Until::Ready);
        assert!(!world.store.rows.contains_key(&Key::Forge(forge_top::Key::Names(task))));
        release(&mut world, task);
        world.until(Until::Ready);
        let resources = names(&world, task);
        let named = resources.iter().any(|name| {
            matches!(&name.what, forge_top::What::Branch(parts)
            if parts.iter().map(AsRef::as_ref).eq(branch.split(|byte| *byte == b'/')))
        });
        assert_eq!(named, pushed, "retry only watches a known head");
        world.close();
    }
}

#[test]
fn releasing_an_initial_admission_hold_rechecks_the_same_bound() {
    let mut configured = World::configured(true, true);
    // The chat's branch and issue fill the client; the change still needs its base alone.
    configured.env.limits.forge.holds = 2;
    let client = skein_forge_client::Limits { resources: 2, ..configured.env.limits.forge.client };
    let mut world = configured.with_forge_client(client);
    world.adopt();
    world.send(protocol::hello_slots(CHANNEL, 2, Box::new([])));
    let ask = adoption_chat(&world);
    let chat = assigned_chat(&mut world, 92, ask);
    fill_chat_watches(&mut world, chat.task);
    let engine::CallAnswer::Delegated(made) = super::change_tool::change(&mut world, &chat, 93, None, b"Small fix")
    else {
        panic!("change delegated")
    };
    let task = made[0].task;
    await_hold(&mut world, task);
    let revision = live_task(&world, task).revisions.hold;
    send_release(&mut world, task, revision);
    for _ in 0..STEPS {
        world.tick();
        if live_task(&world, task).revisions.hold > revision
            && matches!(live_task(&world, task).phase, tasks::Phase::Held { .. })
        {
            let words = std::str::from_utf8(held_words(&world, task)).expect("hold words");
            assert!(words.contains("forge.client.resources = 2; required usage 3"), "{words}");
            return;
        }
    }
    panic!("release bypassed the persistent admission bound: {:?}", live_task(&world, task).phase);
}

fn stock_overlap_world() -> World {
    let mut configured = World::configured_limits(
        true,
        true,
        false,
        STEPS,
        Gate::default(),
        None,
        StoryLimits {
            turn: true,
            tier: Some(temper_engine_domain_world::tier_runs::Tier { silent: true, ..Default::default() }),
            tasks: Some(TASKS),
            slots: Some(SLOTS),
            spec_bytes: Some(4096),
            ..StoryLimits::default()
        },
    );
    let stock = temper_engine_service::limits::profile(temper_engine_service::limits::Profile::Small);
    configured.env.limits.forge.resources_per_task = stock.forge.resources_per_task;
    configured.env.limits.forge.tasks = stock.forge.tasks;
    let client = skein_forge_client::Limits {
        resources: stock.forge.client.resources,
        repositories: stock.forge.client.repositories,
        ..configured.env.limits.forge.client
    };
    configured.with_forge_client(client)
}

#[test]
fn stock_resources_fit_three_chats_changes_and_a_pending_tier() {
    use temper_engine_domain_world::tier_runs::{self, change, task, tier};
    let mut world = stock_overlap_world();
    world.adopt();
    world.send(protocol::hello_slots(CHANNEL, SLOTS, Box::new([])));
    let mut changes = Vec::new();
    for key in 92..95 {
        world.env.now = world.env.now.saturating_add(CHAT_GAP);
        world.env.wall = skein_lib::Wall::from_nanos(world.env.now.as_nanos());
        super::tiers::sign_in(&mut world);
        let ask = adoption_chat(&world);
        let chat = assigned_chat(&mut world, key, ask);
        let answer = super::change_tool::change(&mut world, &chat, u64::from(key), None, b"Small fix");
        let number = tier_runs::produce_delegated(&mut world, answer, b"main", b"file", &[key]);
        changes.push(number);
        assert!(matches!(change(&world, number).change.state, change_step::State::Landed { .. }));
        for number in &changes {
            assert!(!matches!(task(&world, *number).phase, tasks::Phase::Held { .. } | tasks::Phase::Ended(_)));
        }
        assert_eq!(tier(&world).record.sequence, 0, "the baseline must remain pending");
    }
    eprintln!("distinct task watches={}, stock={}", watch_count(&world), world.env.limits.forge.client.resources);
    assert_eq!(watch_count(&world), 9, "three chat branches, three change branches and pulls");
    assert!(tier(&world).record.run.is_some(), "the tier still watches shared main");
}

#[test]
fn resource_hold_account_survives_short_documents_and_unchanged_release() {
    let mut documents = jig_protocol_config::profile(jig_protocol_config::Profile::Small).core.documents;
    documents.text_bytes = 128;
    let mut configured = World::configured_limits(
        true,
        true,
        false,
        STEPS,
        Gate::default(),
        Some(documents),
        StoryLimits { slots: Some(SLOTS), ..StoryLimits::default() },
    );
    configured.env.limits.forge.holds = 2;
    let client = skein_forge_client::Limits { resources: 2, ..configured.env.limits.forge.client };
    let mut world = configured.with_forge_client(client);
    world.adopt();
    world.send(protocol::hello_slots(CHANNEL, SLOTS, Box::new([])));
    let ask = adoption_chat(&world);
    let chat = assigned_chat(&mut world, 92, ask);
    fill_chat_watches(&mut world, chat.task);
    let engine::CallAnswer::Delegated(made) = super::change_tool::change(&mut world, &chat, 93, None, b"Small fix")
    else {
        panic!("change delegated")
    };
    let task = made[0].task;
    await_hold(&mut world, task);
    let words = String::from_utf8(held_words(&world, task).to_vec()).expect("hold words");
    assert!(words.contains("forge.client.resources = 2; required usage 3"), "{words}");
    assert!(words.len() > 128 && words.contains("restart, then release"), "{words}");
    super::tiers::task_show::assert_hold_words(&mut world, task, &words);
    let revision = live_task(&world, task).revisions.hold;
    send_release(&mut world, task, revision);
    for _ in 0..STEPS {
        world.tick();
        if live_task(&world, task).revisions.hold > revision {
            let again = std::str::from_utf8(held_words(&world, task)).expect("hold words").to_owned();
            assert_eq!(again.split(" for branch").next(), words.split(" for branch").next());
            super::tiers::task_show::assert_hold_words(&mut world, task, &again);
            return;
        }
    }
    panic!("unchanged capacity must hold again");
}
