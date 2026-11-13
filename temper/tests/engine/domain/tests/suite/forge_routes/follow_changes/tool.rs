//! A scripted standing owner reaches the same retained follow acts as ctl.
use super::*;
use engine::{FollowCall, FollowToolAnswer};

fn role(world: &mut World, key: u8) -> engine::Assignment {
    let people::Ask::Create { project, mut batch } = world.chat(2, b"Own the port") else { panic!("role defaults") };
    batch[0].label = 3;
    batch[0].tracked = None;
    batch[0].flags.conversation = false;
    batch[0].flags.ended_by = people::EndedBy::Standing;
    batch[0].budget = people::Budget::AsAllowed { cap: Some(100) };
    assigned_chat(world, key, people::Ask::Create { project, batch })
}
fn owned_world() -> (World, engine::Assignment) {
    let mut world = unaccepted_follow(1000, Gate::default(), false, true);
    let owner = role(&mut world, 115);
    let mut follow = setting(s::FollowStart::Owner);
    follow.owner = s::FollowOwner::Task(owner.task);
    change_setting(&mut world, Some(follow), 116);
    assert!(row(&world).heard.is_some());
    assert_eq!(world.root.tasks().task(owner.task).expect("standing owner task").label, 3);
    (world, owner)
}
fn tool(world: &mut World, run: &engine::Assignment, number: u64, input: &str) -> engine::CallAnswer {
    let input = smith_domain_run::HostInput::attested(input.as_bytes().into()).expect("bounded follow input");
    let temper_engine_smith::Decoded::Call(call) = temper_engine_smith::call(
        smith_domain_run::CallName { activation: run.attempt, completion: 1, position: 1 },
        b"follow",
        &input,
    )
    .expect("follow tool decodes") else {
        panic!("decoded follow")
    };
    world.scripted_call(run, number, call.tool)
}
fn kept(world: &mut World, run: &engine::Assignment, number: u64, input: &str) {
    let answer = tool(world, run, number, input);
    let answer = temper_engine_smith::answer(&answer);
    assert!(!answer.error(), "{}", String::from_utf8_lossy(answer.text()));
    assert_eq!(answer.text(), b"Kept: follow 1's next step applies it.");
    settle(world);
}
fn hex(after: &skein_forge::api::Commit) -> String {
    String::from_utf8(forge_top::commit_text(after).into_vec()).expect("hexadecimal commit text")
}
#[test]
fn a_standing_owners_follow_tool_holds_starts_skips_and_merges_the_chain() {
    let (mut world, owner) = owned_world();
    let a = push(&mut world, b"First fix");
    let b = push(&mut world, b"Second fix");
    let c = push(&mut world, b"Third fix");
    let changes = [change(&world, a), change(&world, b), change(&world, c)];
    kept(
        &mut world,
        &owner,
        180,
        &format!(r#"{{"act":"hold","follow":1,"commit":"{}","reason":"waits for the handoff fix"}}"#, hex(&a)),
    );
    assert_eq!(row(&world).queue[0].state, FollowState::HeldByOwner(b"waits for the handoff fix".as_slice().into()));
    let shown = tool(&mut world, &owner, 181, r#"{"act":"show","follow":1}"#);
    let shown = temper_engine_smith::answer(&shown);
    assert!(!shown.error());
    assert!(String::from_utf8_lossy(shown.text()).contains("held by its owner: waits for the handoff fix"));
    kept(&mut world, &owner, 182, &format!(r#"{{"act":"merge","follow":1,"commits":["{}","{}"]}}"#, hex(&b), hex(&c)));
    assert_eq!(change(&world, b), changes[1]);
    assert_eq!(change(&world, c), changes[1]);
    assert_eq!(questions::row(&world, changes[1]).follows.as_ref().unwrap().commits.as_ref(), &[b, c]);
    let spec = &live_task(&world, changes[1]).spec;
    assert!(spec.parameters.iter().any(|p| matches!(p, tasks::Parameter::Bytes { name: 18, value } if value.as_ref() == format!("1:{},{}", hex(&b), hex(&c)).as_bytes())));
    kept(&mut world, &owner, 183, &format!(r#"{{"act":"start","follow":1,"commit":"{}"}}"#, hex(&a)));
    next_delegate(&mut world, changes[0], 0);
    assert_ne!(row(&world).queue[0].state, FollowState::HeldByOwner(b"waits for the handoff fix".as_slice().into()));
    let refused = tool(
        &mut world,
        &owner,
        184,
        &format!(r#"{{"act":"hold","follow":1,"commit":"{}","reason":"too late"}}"#, hex(&a)),
    );
    let refused = temper_engine_smith::answer(&refused);
    assert!(refused.error());
    assert!(refused.text().ends_with(b"the change has started producing"));
    kept(
        &mut world,
        &owner,
        185,
        &format!(r#"{{"act":"skip","follow":1,"commit":"{}","reason":"already ported"}}"#, hex(&a)),
    );
    assert_eq!(row(&world).skips.last().unwrap().after, a);
    assert_eq!(row(&world).skips.last().unwrap().reason.as_ref(), b"already ported");
    assert!(!row(&world).queue.iter().any(|entry| entry.after == a));
    assert!(live_task(&world, changes[1]).waiting_on.is_empty());
    no_holds(&world, &changes);
    world.close();
}
fn read(world: &mut World, run: &engine::Assignment, number: u32, call: FollowCall) -> engine::CallAnswer {
    settle(world);
    let before = world.answers.len();
    let applied = world.store.applied;
    let event = world.protocol.call(
        Token::new(7),
        run.task,
        run.attempt,
        u64::from(number),
        engine::Call { completion: number, position: 1, tool: engine::Tool::Follow(call) },
    );
    world.send(event);
    settle(world);
    assert_eq!(world.store.applied, applied, "follow read keeps no call or mutation");
    world.answers.get(before).expect("read answered").clone()
}
#[test]
fn follows_are_readable_in_the_project_but_only_the_owning_task_may_act() {
    let (mut world, owner) = owned_world();
    let a = push(&mut world, b"Fix handoff");
    let other = role(&mut world, 117);
    let people::Ask::Create { project, mut batch } = world.chat(2, b"A goal") else { panic!("goal defaults") };
    batch[0].label = 2;
    batch[0].tracked = Some(1);
    batch[0].budget = people::Budget::AsAllowed { cap: Some(100) };
    let goal = assigned_chat(&mut world, 118, people::Ask::Create { project, batch });
    let before = row(&world).clone();
    for (number, caller) in [(180, &other), (181, &goal)] {
        let denied = tool(
            &mut world,
            caller,
            number,
            &format!(r#"{{"act":"hold","follow":1,"commit":"{}","reason":"not mine"}}"#, hex(&a)),
        );
        let denied = temper_engine_smith::answer(&denied);
        assert!(denied.error());
        assert_eq!(denied.text(), format!("follow 1 is not yours: its owner is task {}", owner.task).as_bytes());
    }
    assert_eq!(row(&world), &before, "refused calls decide no owner act");
    let listed = read(&mut world, &goal, 182, FollowCall::List);
    let listed = temper_engine_smith::answer(&listed);
    assert!(!listed.error());
    assert_eq!(listed.text(), "follow 1: org/vsr main → org/vsr-rs main · 1 behind · no check yet".as_bytes());
    let shown = read(&mut world, &other, 183, FollowCall::Show { follow: 1 });
    assert!(
        String::from_utf8_lossy(temper_engine_smith::answer(&shown).text())
            .contains(&format!("owner: task {}", owner.task))
    );
    let ask = world.chat(1, b"Other project");
    let stranger = assigned_chat(&mut world, 119, ask);
    let unknown = read(&mut world, &stranger, 184, FollowCall::Show { follow: 1 });
    let unknown = temper_engine_smith::answer(&unknown);
    assert!(unknown.error());
    assert_eq!(unknown.text(), b"no such follow");
    let unknown_act =
        tool(&mut world, &stranger, 185, &format!(r#"{{"act":"start","follow":1,"commit":"{}"}}"#, hex(&a)));
    assert_eq!(temper_engine_smith::answer(&unknown_act).text(), b"no such follow");
    let unknown_commit =
        tool(&mut world, &owner, 186, &format!(r#"{{"act":"start","follow":1,"commit":"{}"}}"#, hex(&[0xff; 32])));
    let unknown_commit = temper_engine_smith::answer(&unknown_commit);
    assert!(unknown_commit.error());
    assert!(unknown_commit.text().ends_with(b"no queued entry has this commit; it may already be followed or skipped"));
    // A later turn retires this caller's refusal from the four-entry named-call table.
    let proof_key = Key::Core(jig_domain_core::Key::Core(jig_domain_core::CoreKey::RunProof(other.task)));
    let Some(Record::Core(jig_domain_core::Record::Core(jig_domain_core::CoreRecord::RunProof(proof)))) =
        world.store.rows.get(&proof_key)
    else {
        panic!("role proof")
    };
    let read = proof.turn.as_ref().and_then(|turn| turn.read);
    world.send(protocol::turn(
        Token::new(7),
        other.task,
        other.attempt,
        live_task(&world, other.task).turn + 1,
        0,
        read,
        b"Read the follow refusal",
    ));
    settle(&mut world);
    let mut follow = setting(s::FollowStart::Owner);
    follow.owner = s::FollowOwner::Person(1);
    change_setting(&mut world, Some(follow), 120);
    let person = tool(&mut world, &owner, 187, &format!(r#"{{"act":"start","follow":1,"commit":"{}"}}"#, hex(&a)));
    let person = temper_engine_smith::answer(&person);
    assert!(person.error());
    assert_eq!(person.text(), b"follow 1 is not yours: its owner is person 1");
    world.close();
}
#[test]
fn follow_write_replays_the_kept_answer_after_restart_before_rendering() {
    let (mut world, owner) = owned_world();
    let a = push(&mut world, b"Already ported");
    world.protocol.defer_render = true;
    let call = engine::Call {
        completion: 1,
        position: 1,
        tool: engine::Tool::Follow(FollowCall::Act {
            follow: 1,
            act: FollowAct::Skip { after: a, reason: b"already ported".as_slice().into() },
        }),
    };
    let event = world.protocol.call(Token::new(7), owner.task, owner.attempt, 180, call.clone());
    world.send(event);
    for _ in 0..TICKS {
        world.tick();
        if !world.protocol.pending_renders.is_empty() && row(&world).skips.len() == 1 && world.store.pending.is_empty()
        {
            break;
        }
    }
    assert!(!world.protocol.pending_renders.is_empty(), "answer held before render");
    assert_eq!(row(&world).skips.len(), 1);
    let answer = world.protocol.answers.last().expect("deferred answer").1.clone();
    assert_eq!(answer, engine::CallAnswer::Follow(Box::new(FollowToolAnswer::Write { follow: 1, refusal: None })));
    assert!(world.store.rows.values().any(|record| matches!(record, Record::Forge { row } if matches!(row.as_ref(), forge_top::Stored::Call { answer: forge_top::CallPayload::Follow { refusal: None, .. }, .. }))));
    world.drain_cut();
    world.protocol.pending_renders.clear();
    world.protocol.answers.clear();
    world.protocol.defer_render = false;
    world.restart_same();
    settle(&mut world);
    let event = world.protocol.call(Token::new(7), owner.task, owner.attempt, 180, call);
    world.send(event);
    settle(&mut world);
    assert_eq!(world.answers.last(), Some(&answer), "the skipped commit is not acted on again");
    assert_eq!(row(&world).skips.len(), 1);
    world.close();
}
