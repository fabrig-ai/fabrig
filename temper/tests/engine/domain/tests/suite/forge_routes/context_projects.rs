//! Context adoption across projects, through the root and durable store.
use super::*;

fn adopt_source_and_target(world: &mut World) {
    adopt_pair(world, b"source", b"target");
}
pub(super) fn adopt_pair(world: &mut World, source: &[u8], target: &[u8]) {
    adopt_pair_configured(world, source, target, 1000, false, false, true);
}
pub(super) fn adopt_pair_passing(world: &mut World, source: &[u8], target: &[u8], passes: u32) {
    adopt_pair_configured(world, source, target, passes, true, false, true);
}
pub(super) fn adopt_pair_cued(world: &mut World) {
    adopt_pair_configured(world, b"vsr", b"vsr-rs", 1000, true, true, true);
}
pub(super) fn adopt_pair_without_context(world: &mut World) {
    adopt_pair_configured(world, b"vsr", b"vsr-rs", 1000, true, true, false);
}
fn adopt_pair_configured(
    world: &mut World,
    source: &[u8],
    target: &[u8],
    passes: u32,
    following: bool,
    cued: bool,
    context: bool,
) {
    let workflows: Box<[_]> = if following {
        Box::new([follow_workflow(cued)])
    } else {
        Box::new([])
    };
    adopt_pair_workflows(world, source, target, passes, workflows, context);
}

fn follow_workflow(cued: bool) -> raw::Workflow {
    raw::Workflow {
        file: b"test.yml".as_slice().into(),
        jobs: Box::new([b"test".as_slice().into()]),
        latency_min: Duration::ZERO,
        latency_max: Duration::ZERO,
        silent: 0,
        passes: 1000,
        fails: Box::new([]),
        flaky: Box::new([]),
        select: None,
        cue: cued.then(|| raw::Cue::Tests { path: b"failures".as_slice().into() }),
        report: Some(raw::Report { artifact: b"results".as_slice().into(), format: raw::ReportFormat::Lines, deflate: false }),
    }
}

/// The exact-head follow check passes; only the target's after-landing tier reads the failure cue.
pub(super) fn adopt_pair_with_red_tier(world: &mut World) {
    let mut fuzz = follow_workflow(true);
    fuzz.file = b"fuzz.yml".as_slice().into();
    fuzz.latency_min = Duration::from_secs(90);
    fuzz.latency_max = fuzz.latency_min;
    adopt_pair_workflows(world, b"vsr", b"vsr-rs", 1000, Box::new([follow_workflow(false), fuzz]), true);
}

fn adopt_pair_workflows(
    world: &mut World,
    source: &[u8],
    target: &[u8],
    passes: u32,
    workflows: Box<[raw::Workflow]>,
    context: bool,
) {
    let following = !workflows.is_empty();
    for (id, name) in [(3, source), (4, target)] {
        let full = [b"org/".as_slice(), name].concat();
        let name = full.as_slice();
        fake::repository(
            &mut world.fake,
            &world.fake_env.limits,
            raw::Setup {
                id,
                name: name.into(),
                default: b"main".as_slice().into(),
                tree: Box::new([]),
                labels: Box::new([]),
                workflows: workflows.clone(),
                protection: None,
                hooked: following,
                checks: raw::Checks {
                    cues: Box::new([]),
                    contexts: if following { Box::new([b"ci".as_slice().into()]) } else { Box::new([]) },
                    latency_min: Duration::ZERO,
                    latency_max: Duration::ZERO,
                    silent: 0,
                    passes,
                    reruns: 0,
                    cue: None,
                },
            },
        );
        fake::grant(&mut world.fake, name, 1, raw::Permission::Admin);
    }
    for (index, (project, name, role)) in [
        (1, source, forge_top::Role::Owned),
        (2, target, forge_top::Role::Owned),
        (2, source, forge_top::Role::Context),
    ]
    .into_iter()
    .enumerate()
    {
        if role == forge_top::Role::Context && !context {
            continue;
        }
        let key = u8::try_from(94 + index).expect("three adoption keys fit u8");
        let mut adoption = temper_engine_domain_world::fixtures::adoption(1);
        adoption.project = project;
        adoption.name = name.into();
        adoption.role = role;
        adoption.home = role != forge_top::Role::Context;
        world.send(engine::Event::Party(engine::PartyInput::Application {
            scope: people::ApplicationScope::Project(project),
            reply_to: ReplyTo::new(Token::new(u64::from(key))),
            sign_in: world.signed_in.expect("owner session"),
            key: [key; 16],
            digest: [key; 32],
            carried: None,
            request: engine::Application::Forge(forge_top::Ask::Adopt(adoption)),
        }));
        for _ in 0..200 {
            world.tick();
            if world.adopted.len() > index {
                break;
            }
        }
        assert!(
            matches!(world.adopted.get(index), Some(forge_top::ApplicationOutcome::Adopted(_))),
            "{:?}",
            world.adopted
        );
        world.until(Until::Ready);
    }
}

#[test]
fn another_projects_writer_is_a_read_only_checkout_and_its_landings_are_heard() {
    let mut world = World::configured(true, true).with_second_project();
    world.fake = fake::Domain::new(&world.fake_env.limits, 71);
    world.sign_in();
    adopt_source_and_target(&mut world);
    world.restart_same();
    world.sign_in();
    world.send(engine::Event::Party(engine::PartyInput::Period { project: 2, period: 1, budget: 500, next: None }));
    world.send(protocol::hello_slots(Token::new(7), 2, Box::new([])));
    let ask = world.chat(1, b"Work on source");
    let writer = assigned_chat(&mut world, 98, ask);
    assert_eq!(writer.workspace.repositories.len(), 1);
    let source = &writer.workspace.repositories[0];
    assert_eq!(source.name.as_ref(), b"source");
    assert!(source.push.is_some() && source.save.is_some(), "the writer's runs stay writable");
    let ask = world.chat(2, b"Read source beside target");
    let reader = assigned_chat(&mut world, 99, ask);
    assert_eq!(reader.workspace.repositories.len(), 2);
    let context = reader.workspace.repositories.iter().find(|r| r.name.as_ref() == b"source").unwrap();
    assert!(context.push.is_none() && context.save.is_none() && context.landing.is_none() && !context.delivers);
    assert_eq!(context.start, engine::ForgeStart::Base(b"main".as_slice().into()));
    let home = reader.workspace.repositories.iter().find(|r| r.name.as_ref() == b"target").unwrap();
    assert!(home.push.is_some() && home.save.is_some());
    let provider = context.provider;
    let event = world.protocol.call(
        Token::new(7),
        reader.task,
        reader.attempt,
        100,
        engine::Call {
            completion: 1,
            position: 1,
            tool: engine::Tool::SubscribeForge {
                topic: forge_top::Topic::Landings { repository: provider, branch: b"main".as_slice().into() },
                own_change: None,
                paths: Box::new([]),
            },
        },
    );
    world.send(event);
    world.until(Until::Subscribed);
    for _ in 0..10 {
        world.tick();
    }
    let before = news_count(&world, reader.task);
    assert!(fake::advance(&mut world.fake, &world.fake_env, b"org/source", b"main", b"file", b"changed", 1).is_ok());
    world.send(engine::Event::Forge(Box::new(forge_top::Event::Hint {
        hint: skein_forge::api::Hint {
            repository: provider,
            change: skein_forge::api::Change::Branch(b"main".as_slice().into()),
            key: None,
        },
    })));
    await_new_forge_news(&mut world, reader.task, before);
    world.close();
}
