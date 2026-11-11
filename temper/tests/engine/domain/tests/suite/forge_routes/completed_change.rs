//! Completed changes close delegates whose work no longer counts (domain/forge/change/README.md, 2).
use super::goal_landing::{ended, row};
use super::*;
use temper_engine_domain_forge_change as change;

const STEPS: u32 = 300;
const CHANNEL: Token = Token::new(7);

pub(super) fn acknowledge_cancel(world: &mut World, run: &engine::Assignment, reason: &[u8], parts: engine::Parts) {
    for _ in 0..STEPS {
        world.tick();
        if matches!(live_task(world, run.task).phase, tasks::Phase::Closing(_)) {
            break;
        }
    }
    let tasks::Phase::Closing(closing) = &live_task(world, run.task).phase else {
        panic!("completed change must cancel its delegate: {:?}", live_task(world, run.task).phase)
    };
    let tasks::Ending::Cancelled { reason: heard, .. } = &closing.ending else {
        panic!("delegate must be cancelled: {closing:?}")
    };
    assert_eq!(heard.as_ref(), reason);
    assert_eq!(closing.stage, tasks::Stage::Run { attempt: run.attempt });
    // The scripted worker acknowledges the engine's cancel. Even a park terminal
    // closes this run and task; the test sends no cancel of its own.
    let event = world.protocol.answer(CHANNEL, run.task, run.attempt, 1, tasks::End::Parked, parts);
    world.send(event);
    for _ in 0..STEPS {
        world.tick();
        if ended(world, run.task).is_some() {
            break;
        }
    }
    assert!(matches!(ended(world, run.task).expect("cancelled delegate ended").phase,
        tasks::Phase::Ended(tasks::Ending::Cancelled { .. })));
}

#[test]
fn a_landed_change_cancels_its_live_resolution() {
    let (mut world, _, producer, branch) = change_world(false, 1000, None);
    let tasks::Party::Task(task) = live_task(&world, producer.task).requester else { panic!("change") };
    world.external(2, raw::Op::Git(raw::Git::Create { branch: branch.clone(), commit: 1 }));
    fake::advance(&mut world.fake, &world.fake_env, b"org/repo", &branch, b"file", b"branch change", 1)
        .expect("producer head");
    fake::advance(&mut world.fake, &world.fake_env, b"org/repo", b"main", b"file", b"base change", 1)
        .expect("conflicting base");
    super::delegates::fixtures::answer_producer(&mut world, &producer);
    let resolution = super::delegates::fixtures::next_delegate(&mut world);
    assert!(matches!(row(&world, task).delegate, Some((child, change::Delegate::Resolve { .. })) if child == resolution.task));
    fake::advance(&mut world.fake, &world.fake_env, b"org/repo", &branch, b"file", b"base change", 1)
        .expect("resolve conflict");
    let head = fake::advance(&mut world.fake, &world.fake_env, b"org/repo", &branch, b"resolved", b"kept change", 1)
        .expect("resolution head");
    let pull = row(&world, task).pull.expect("open pull");
    assert!(matches!(world.external(1, raw::Op::Write(raw::Write::Merge { number: pull, head, how: raw::MergeHow::Style })), raw::Answer::Merged(_)));
    world.send(engine::Event::Forge(Box::new(forge_top::Event::Hint {
        hint: skein_forge::api::Hint {
            repository: forge_world::REPO,
            change: skein_forge::api::Change::Branch(branch),
            key: None,
        },
    })));
    acknowledge_cancel(&mut world, &resolution, b"its change landed", engine::Parts::none());
    for _ in 0..STEPS {
        world.tick();
        if ended(&world, task).is_some() {
            break;
        }
    }
    assert!(matches!(row(&world, task).change.state, change::State::Landed { .. }));
    assert!(matches!(ended(&world, task).expect("resolution terminal wakes its change").phase,
        tasks::Phase::Ended(tasks::Ending::Done(tasks::TaskResult::Change { .. }))));
    world.close();
}
