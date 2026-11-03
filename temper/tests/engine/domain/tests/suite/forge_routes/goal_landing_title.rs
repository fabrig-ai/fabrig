//! A goal's landing uses its words' first line as the pull title
//! (temper/docs/design/domain/forge.md, 8.1).
use super::goal_branch::{change, goal_request};
use super::*;

const LANDING_CALL: u64 = 95;
const STEPS: u32 = 1000; // The forge world's bound for opening a landing.
const TITLE_BYTES: usize = 256; // forge.md, 8.1.
const SPEC_BYTES: u32 = 1024; // Holds the longest words, their title parameter, and branch names.
const GOAL_REQUEST: u8 = 92;

fn landing_world() -> (World, engine::Assignment, Box<[u8]>) {
    let mut world = World::configured_limits(
        true,
        true,
        true,
        STEPS,
        Gate::default(),
        None,
        StoryLimits { spec_bytes: Some(SPEC_BYTES), ..StoryLimits::default() },
    );
    world.adopt();
    world.send(protocol::hello_slots(Token::new(7), 2, Box::new([])));
    let ask = goal_request(&world, true);
    let goal = assigned_chat(&mut world, GOAL_REQUEST, ask);
    let branch = format!("temper/{}/g{}", goal.task, goal.task).into_bytes().into_boxed_slice();
    world.until(Until::Ready);
    (world, goal, branch)
}

fn change_row(world: &World, task: u64) -> &forge_top::ChangeRow {
    let Some(Record::Forge { row }) = world.store.rows.get(&Key::Forge(forge_top::Key::Change(task))) else {
        panic!("registered change {task}")
    };
    let forge_top::Stored::Change(row) = row.as_ref() else { panic!("change row") };
    row
}

fn assert_landing_title(words: &str, first_line: Option<&[u8]>, title: &[u8]) {
    let (mut world, goal, branch) = landing_world();
    let engine::CallAnswer::Delegated(made) = change(&mut world, &goal, engine::ChangeLanding::Goal) else {
        panic!("goal's first change admitted")
    };
    world.until(Until::SecondAssignment);
    let producer = world.assigned[1].clone();
    let first_branch = change_row(&world, made[0].task).branch.clone();
    change_tool::land_change(&mut world, &producer, &first_branch);
    world.until(Until::Ready);
    world.fake_env.limits.limits.title_bytes = u32::try_from(TITLE_BYTES).expect("title bound");
    let input = serde_json::to_vec(&serde_json::json!({ "words": words, "branch": "mine" })).expect("change input");
    let input = smith_domain_run::HostInput::attested(input.into_boxed_slice()).expect("bounded change input");
    let temper_engine_smith::Decoded::Call(decoded) = temper_engine_smith::call(
        smith_domain_run::CallName {
            activation: goal.attempt,
            completion: u32::try_from(LANDING_CALL).expect("call"),
            position: 1,
        },
        b"change",
        &input,
    )
    .expect("change decodes") else {
        panic!("change call")
    };
    let before = world.answers.len();
    let event = world.protocol.call(Token::new(7), goal.task, goal.attempt, LANDING_CALL, decoded);
    world.send(event);
    world.until(Until::Ready);
    let Some(engine::CallAnswer::Delegated(made)) = world.answers.get(before) else {
        panic!("goal landing admitted: {:?}", world.call_errors)
    };
    assert_eq!(made.len(), 1);
    let landing = made[0].task;
    let mut parameters = vec![
        tasks::Parameter::Resource {
            name: 1,
            connector: 0,
            resource: forge_top::tag(forge_world::REPO.forge, forge_world::REPO.repository),
        },
        tasks::Parameter::Bytes { name: 2, value: b"main".as_slice().into() },
        tasks::Parameter::Bytes { name: 3, value: branch },
        tasks::Parameter::Number { name: 11, value: 1 },
    ];
    if let Some(first_line) = first_line {
        parameters.push(tasks::Parameter::Bytes { name: 10, value: first_line.into() });
    }
    assert_eq!(live_task(&world, landing).spec.parameters.as_ref(), parameters);
    assert_eq!(live_task(&world, landing).spec.words.as_ref(), words.as_bytes());
    for _ in 0..STEPS {
        world.tick();
        if change_row(&world, landing).pull.is_some() {
            break;
        }
    }
    let row = change_row(&world, landing);
    let number = row.pull.unwrap_or_else(|| panic!("the held branch opens immediately: {row:?}"));
    let raw::Answer::Item { item, .. } = world.external(1, raw::Op::Read(raw::Read::Item { number, after: 0 })) else {
        panic!("landing pull")
    };
    assert_eq!(item.title.as_ref(), title);
    world.close();
}

#[test]
fn a_goals_landing_is_titled_by_its_words_first_line() {
    for (words, title) in [
        ("Land the reference goal\nWhat it built: ...", "Land the reference goal"),
        ("Land the reference goal\r\nWhat it built: ...", "Land the reference goal"),
        ("  Land v1.2. Keep punctuation!  \nWhat it built: ...", "  Land v1.2. Keep punctuation!  "),
        ("Land the reference goal", "Land the reference goal"),
    ] {
        assert_landing_title(words, Some(title.as_bytes()), title.as_bytes());
    }
}

#[test]
fn an_empty_first_line_uses_the_handed_branch_default_title() {
    for words in ["\nWhat it built: ...", "\r\nWhat it built: ..."] {
        assert_landing_title(words, None, b"Update repository");
    }
}

#[test]
fn a_goals_landing_fits_its_title_only_when_opening_the_pull() {
    for (words, title) in [
        ("x".repeat(TITLE_BYTES - 1), "x".repeat(TITLE_BYTES - 1)),
        ("x".repeat(TITLE_BYTES), "x".repeat(TITLE_BYTES)),
        ("x".repeat(TITLE_BYTES + 1), format!("{}…", "x".repeat(253))),
        ("é".repeat(129), format!("{}…", "é".repeat(126))),
    ] {
        assert_landing_title(&words, Some(words.as_bytes()), title.as_bytes());
    }
}
