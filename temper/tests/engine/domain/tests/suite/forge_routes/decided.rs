//! A goal's named decision is live and durable (temper/docs/design/domain/forge.md, 5.9).
use super::*;
use skein_lib::Wall;

const CHANNEL: Token = Token::new(7);
const CALL: u64 = 96;
const STEPS: u32 = 200; // A root call's durable round trip, without forge IO.

fn start(tracked: bool, words: &[u8]) -> (World, engine::Assignment) {
    let mut world = World::configured_limits(
        true,
        true,
        true,
        1000,
        Gate::default(),
        None,
        StoryLimits { slots: Some(3), ..StoryLimits::default() },
    );
    world.adopt();
    world.send(protocol::hello_slots(CHANNEL, 3, Box::new([])));
    let people::Ask::Create { project, mut batch } = world.chat(1, words) else { panic!("chat template") };
    batch[0].tracked = tracked.then_some(0);
    batch[0].flags.conversation = !tracked;
    batch[0].budget = people::Budget::AsAllowed { cap: Some(400) };
    batch[0].authority.tools = 1 | 2 | 4 | 512;
    batch[0].authority.delegation.tasks = 11;
    batch[0].authority.delegation.depth = 3;
    let run = assigned_chat(&mut world, 92, people::Ask::Create { project, batch });
    world.until(Until::Ready);
    (world, run)
}

fn away(world: &mut World, on: bool, until: Option<Wall>, kinds: u8) {
    world.send(engine::Event::Forge(Box::new(forge_top::Event::SetAway {
        request: Token::new(301),
        token: Token::new(302),
        party: 1,
        project: 1,
        on,
        until,
        kinds,
    })));
    world.until(Until::Ready);
}

fn tool(kind: forge_top::DecisionKind) -> engine::Tool {
    engine::Tool::Decided(forge_top::DecidedCall {
        question: Box::from(b"Which implementation?".as_slice()),
        options: Box::new([Box::from(b"Small".as_slice()), Box::from(b"Broad".as_slice())]),
        took: 0,
        why: Box::from(b"Fits the design".as_slice()),
        kind,
    })
}

fn rows(world: &World) -> Vec<forge_top::Decided> {
    world
        .store
        .rows
        .values()
        .filter_map(|record| match record {
            Record::Forge { row } => match row.as_ref() {
                forge_top::Stored::Decided(row) => Some(row.clone()),
                _ => None,
            },
            Record::Core(_) => None,
        })
        .collect()
}

fn ask(world: &mut World, run: &engine::Assignment, kind: forge_top::DecisionKind) {
    assert_eq!(world.scripted_call(run, CALL, tool(kind)), engine::CallAnswer::Decided { took: None });
    assert!(rows(world).is_empty(), "asking saves no decision row");
}

#[test]
fn a_goal_decides_while_its_person_is_away() {
    let (mut world, run) = start(true, b"Make a small fix");
    away(&mut world, true, None, 15);
    assert_eq!(
        world.scripted_call(&run, CALL, tool(forge_top::DecisionKind::Implementation)),
        engine::CallAnswer::Decided { took: Some(Box::from(b"Small".as_slice())) }
    );
    let decisions = rows(&world);
    assert_eq!(decisions.len(), 1);
    let row = &decisions[0];
    assert_eq!((row.project, row.party, row.task, row.number), (1, 1, run.task, 1));
    assert_eq!(row.question.as_ref(), b"Which implementation?");
    assert_eq!(row.options.as_ref(), &[Box::from(b"Small".as_slice()), Box::from(b"Broad".as_slice())]);
    assert_eq!(row.took, 0);
    assert_eq!(row.why.as_ref(), b"Fits the design");
    assert!(!row.overturned);
    world.close();
}

#[test]
fn a_chats_goal_finds_the_chats_person() {
    let (mut world, chat) = start(false, b"Make a small fix");
    let engine::CallAnswer::Delegated(made) = world.scripted_call(
        &chat,
        93,
        engine::Tool::Goal { words: Box::from(b"Make a small fix".as_slice()), advisor: false, agree: false },
    ) else {
        panic!("goal delegated")
    };
    world.until(Until::SecondAssignment);
    let run = world.assigned.iter().find(|run| run.task == made[0].task).unwrap().clone();
    away(&mut world, true, None, 15);
    assert_eq!(
        world.scripted_call(&run, CALL, tool(forge_top::DecisionKind::Retry)),
        engine::CallAnswer::Decided { took: Some(Box::from(b"Small".as_slice())) }
    );
    assert_eq!(rows(&world)[0].party, 1);
    world.close();
}

#[test]
fn its_person_here_it_is_told_to_ask() {
    let (mut world, run) = start(true, b"Make a small fix");
    ask(&mut world, &run, forge_top::DecisionKind::Implementation);
    world.close();
}

#[test]
fn a_kind_not_ticked_is_told_to_ask() {
    let (mut world, run) = start(true, b"Make a small fix");
    away(&mut world, true, None, 7);
    ask(&mut world, &run, forge_top::DecisionKind::Dependency);
    world.close();
}

#[test]
fn other_is_told_to_ask() {
    let (mut world, run) = start(true, b"Make a small fix");
    away(&mut world, true, None, 15);
    ask(&mut world, &run, forge_top::DecisionKind::Other);
    world.close();
}

#[test]
fn past_until_it_is_told_to_ask() {
    let (mut world, run) = start(true, b"Make a small fix");
    let until = world.env.wall;
    away(&mut world, true, Some(until), 15);
    ask(&mut world, &run, forge_top::DecisionKind::Retry);
    world.close();
}

#[test]
fn a_replay_answers_the_same_after_the_switch_moved() {
    for first in [false, true] {
        let (mut world, run) = start(true, b"Make a small fix");
        away(&mut world, first, None, 15);
        let call = engine::Call { completion: 1, position: 1, tool: tool(forge_top::DecisionKind::Implementation) };
        let event = world.protocol.call(CHANNEL, run.task, run.attempt, CALL, call.clone());
        world.send(event);
        world.until(Until::Ready);
        let original = world.answers.last().unwrap().clone();
        let before = rows(&world);
        away(&mut world, !first, None, 15);
        let answers = world.answers.len();
        let event = world.protocol.call(CHANNEL, run.task, run.attempt, CALL + 1, call);
        world.send(event);
        world.until(Until::Ready);
        assert_eq!(world.answers.len(), answers + 1);
        assert_eq!(world.answers.last(), Some(&original));
        assert_eq!(rows(&world), before, "replay writes no second decision");
        world.close();
    }
}

#[test]
fn a_chat_is_told_to_ask() {
    let (mut world, run) = start(false, b"Make a small fix");
    away(&mut world, true, None, 15);
    ask(&mut world, &run, forge_top::DecisionKind::Implementation);
    world.close();
}

#[test]
fn asks_always_is_told_to_ask() {
    let (mut world, run) = start(true, b"Make a small fix\nAsks: always\nKeep it small");
    away(&mut world, true, None, 15);
    ask(&mut world, &run, forge_top::DecisionKind::Implementation);
    world.close();
}

#[test]
fn an_advisor_is_beyond_authority() {
    let (mut world, goal) = start(true, b"Make a small fix");
    let engine::CallAnswer::Delegated(made) = world.scripted_call(
        &goal,
        93,
        engine::Tool::Goal { words: Box::from(b"Survey".as_slice()), advisor: true, agree: false },
    ) else {
        panic!("advisor delegated")
    };
    world.until(Until::SecondAssignment);
    let run = world.assigned.iter().find(|run| run.task == made[0].task).unwrap().clone();
    away(&mut world, true, None, 15);
    let answer = world.scripted_call(&run, CALL, tool(forge_top::DecisionKind::Implementation));
    assert!(matches!(answer, engine::CallAnswer::ToolDenied { .. }), "{answer:?}");
    let rendered = temper_engine_smith::answer_for_tool(&answer, b"decided", 1024);
    assert!(String::from_utf8_lossy(rendered.text()).contains("control"));
    assert!(rows(&world).is_empty());
    world.close();
}

#[test]
fn decided_answers_in_its_decision() {
    let (mut world, run) = start(true, b"Make a small fix");
    away(&mut world, true, None, 15);
    world.scripted_read_inbox(&run);
    world.until(Until::Ready);
    let event = world.protocol.call(
        CHANNEL,
        run.task,
        run.attempt,
        CALL,
        engine::Call { completion: 1, position: 1, tool: tool(forge_top::DecisionKind::Implementation) },
    );
    world.send(event);
    let mut found = false;
    for _ in 0..STEPS {
        for (_, writes) in &world.store.pending {
            if writes.iter().any(|write| matches!(write, engine::Write::Save(Record::Forge { row }) if matches!(row.as_ref(), forge_top::Stored::Decided(_)))) {
                assert!(writes.iter().any(|write| matches!(write, engine::Write::Save(Record::Forge { row }) if matches!(row.as_ref(), forge_top::Stored::Call { answer: forge_top::CallPayload::Decided { took: Some(_) }, .. }))), "decision row and named answer share a commit");
                assert!(writes.iter().any(|write| matches!(write, engine::Write::Save(Record::Core(jig_domain_core::Record::Core(jig_domain_core::CoreRecord::Call(_)))))), "core call shares the decision");
                found = true;
            }
        }
        if found {
            break;
        }
        if let Some(event) = world.events.pop_front() {
            world.send(event);
        } else {
            world.tick();
        }
    }
    assert!(found, "the decoded call answers in its decision");
    world.until(Until::Ready);
    world.close();
}
