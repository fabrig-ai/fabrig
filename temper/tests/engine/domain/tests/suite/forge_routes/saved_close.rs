//! Saved work is deleted through the task's durable close.
use super::saved_retry::{finish_change, saved_repair_retry};
use super::*;

const LANDINGS: u32 = 1; // The repair lets its one change land.
const TICKS: u32 = 200; // The root world's durable round-trip bound.

fn ended(world: &World, task: u64) -> bool {
    world.store.rows.contains_key(&Key::Core(jig_domain_core::Key::Tasks(tasks::Key::Ended(task))))
}

#[test]
fn a_retried_repairs_saved_branch_is_deleted_as_it_closes() {
    let (mut world, mut workspace, retry, _) = saved_repair_retry();
    let engine::ForgeStart::Saved(saved) = &retry.workspace.repositories[0].start else {
        panic!("retry resumes its saved work")
    };
    let saved = saved.clone();
    let parts = workspace.attempt(&mut world, &retry, false);
    let head = temper_worker_checkout_world::translate::fake(temper_worker_domain_checkout::git::Commit::new(
        parts.pushed[0].commit,
    ));
    for context in [b"CI".as_slice(), b"build"] {
        world.external(
            1,
            raw::Op::Write(raw::Write::Status {
                commit: head,
                context: context.into(),
                state: raw::Check::Passed,
                description: Box::new([]),
                link: Box::new([]),
            }),
        );
    }
    finish_change(&mut world, &retry, b"repaired", parts);
    for _ in 0..TICKS {
        world.tick();
        if ended(&world, retry.task) && world.merges != 0 {
            break;
        }
    }
    assert!(ended(&world, retry.task), "repair closed");
    assert_eq!(world.merges, LANDINGS, "repaired change landed");
    assert_eq!(world.fake.branch(b"org/repo", &saved), None, "closed task left its saved branch on the forge");
    assert!(world.writes.contains(&skein_forge::api::Write::DeleteBranch { branch: saved }));
    world.close();
}

#[test]
fn a_change_cancelled_before_its_first_push_deletes_no_branch() {
    let (mut world, _, producer, branch) = change_world(false, 1000, None);
    let tasks::Party::Task(change) = live_task(&world, producer.task).requester else { panic!("change") };
    world.send(engine::Event::Party(engine::PartyInput::Ask {
        models: Box::new([]),
        reply_to: ReplyTo::new(Token::new(96)),
        sign_in: world.signed_in.expect("owner"),
        key: [96; 16],
        ask: people::Ask::Cancel { project: 1, task: change, reason: b"stop".as_slice().into(), grace: None },
    }));
    let event = world.protocol.answer(
        Token::new(7),
        producer.task,
        producer.attempt,
        5,
        tasks::End::Refused,
        engine::Parts::none(),
    );
    world.send(event);
    for _ in 0..TICKS {
        world.tick();
        if ended(&world, change) {
            break;
        }
    }
    assert!(ended(&world, change), "cancelled change closes");
    assert!(!world.writes.contains(&skein_forge::api::Write::DeleteBranch { branch: branch.clone() }));
    assert!(super::goal_branch::forge_hold(&world, &branch).is_none());
    world.close();
}

#[test]
fn a_task_with_no_saved_work_deletes_no_saved_branch() {
    let (mut world, _, producer, branch) = change_world(false, 1000, None);
    let saved = producer.workspace.repositories[0].save.clone().expect("producer can save work");
    world.external(2, raw::Op::Git(raw::Git::Create { branch: branch.clone(), commit: 1 }));
    fake::advance(&mut world.fake, &world.fake_env, b"org/repo", &branch, b"file", b"fixed", 1).expect("producer head");
    finish_change(&mut world, &producer, b"pushed", engine::Parts::none());
    for _ in 0..TICKS {
        world.tick();
        if ended(&world, producer.task) && world.merges != 0 {
            break;
        }
    }
    assert!(ended(&world, producer.task));
    assert!(!world.writes.contains(&skein_forge::api::Write::DeleteBranch { branch: saved.clone() }));
    assert!(!world.store.rows.values().any(|record| matches!(record,
        Record::Forge { row } if matches!(row.as_ref(), forge_top::Stored::Hold(hold)
            if matches!(&hold.name.what, forge_top::What::Branch(parts)
                if parts.iter().map(AsRef::as_ref).eq(saved.split(|byte| *byte == b'/'))))
    )));
    world.close();
}
