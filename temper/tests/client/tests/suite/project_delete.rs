//! Owner deletion through ctl and the real engine (client/ctl/tools.md, 3).
use jig_client_kit::harness::{ClientId, World};
use temper_client_tools::jig::Printed;
use temper_client_world::{app::Temper, fixtures, front::Joint};

fn call(world: &mut World<Temper, Joint>, agent: ClientId, words: &[&[u8]]) -> Printed {
    let number = fixtures::give(world, agent, words);
    fixtures::answered(world, agent, number)
}

fn text(world: &mut World<Temper, Joint>, agent: ClientId, words: &[&[u8]]) -> String {
    let printed = call(world, agent, words);
    assert_eq!(printed.status, 0, "{printed:?}");
    assert!(printed.stderr.is_empty(), "{printed:?}");
    String::from_utf8(printed.stdout.into_vec()).expect("ctl UTF-8")
}

#[test]
fn an_owner_ends_live_work_deletes_the_project_and_replays_the_answer() {
    let mut world = fixtures::asking_world(fixtures::SEED, true);
    let member = fixtures::asking_agent(&mut world);
    world.front().asking_owner();
    let owner = fixtures::asking_agent(&mut world);
    let started = call(
        &mut world,
        owner,
        &[b"chat", b"start", b"--project", b"1", b"--template", b"5", b"Help with this project", b"--key", b"chat"],
    );
    assert_eq!(started.status, 0, "{started:?}");
    let task = fixtures::done_task(&started);
    assert!(world.front().engine.task(task).is_some(), "the chat is live");
    let refused = call(&mut world, owner, &[b"project", b"delete", b"1", b"--key", b"drop-1"]);
    assert_eq!(refused.status, 2, "{refused:?}");
    assert!(refused.stdout.is_empty());
    assert_eq!(
        refused.stderr.as_ref(),
        format!("project delete: This project has live tasks; end them first: {task}\n").as_bytes()
    );
    let refused = call(&mut world, member, &[b"project", b"delete", b"1", b"--key", b"m-1"]);
    assert_eq!(refused.status, 2, "{refused:?}");
    assert!(refused.stdout.is_empty());
    assert_eq!(refused.stderr.as_ref(), b"project delete: The held role does not permit this action\n");

    // Let the run reply, then close the idle chat through ctl.
    world.front().asking_reply(task, b"Ready to close");
    let task_number = task.to_string();
    assert_eq!(
        text(&mut world, owner, &[b"chat", b"close", task_number.as_bytes(), b"--key", b"close"]),
        format!("done: task {task}, close\n")
    );
    let ended = text(&mut world, owner, &[b"task", b"show", task_number.as_bytes(), b"--until", b"ended"]);
    assert!(ended.ends_with("met\n"), "{ended}");
    assert!(world.front().engine.task(task).is_none(), "the closed chat ended");
    assert_eq!(
        text(&mut world, owner, &[b"project", b"delete", b"1", b"--key", b"drop-2"]),
        "deleted: project 1, Project 1\n"
    );
    // A fresh ctl has neither the old name nor a local answer for this key.
    let replay = fixtures::asking_agent(&mut world);
    let me = text(&mut world, replay, &[b"me"]);
    assert!(!me.contains("1 · Project 1 ·"), "{me}");
    let shown = call(&mut world, replay, &[b"project", b"show", b"1"]);
    assert_eq!(shown.status, 2, "{shown:?}");
    assert!(shown.stdout.is_empty());
    assert_eq!(shown.stderr.as_ref(), b"project 1 was deleted\n");

    assert_eq!(
        text(&mut world, replay, &[b"project", b"delete", b"1", b"--key", b"drop-2"]),
        "deleted: project 1\n"
    );
}
