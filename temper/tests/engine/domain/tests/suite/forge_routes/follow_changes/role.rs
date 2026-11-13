//! A standing role requests, funds and answers its follow (follows.md, 4 and 11).
use super::*;

fn standing_ask(world: &World, budget: u64) -> people::Ask {
    let people::Ask::Create { project, mut batch } = world.chat(2, b"Watch the source and answer its ports") else {
        panic!("role defaults")
    };
    // Template 7's fields, as status::standing_role fills them for its scripted charter.
    let role = &mut batch[0];
    role.label = 3;
    role.tracked = None;
    role.flags.conversation = false;
    role.flags.ended_by = people::EndedBy::Standing;
    role.budget = people::Budget::Named;
    role.authority.spend = budget;
    people::Ask::Create { project, batch }
}
pub(crate) fn standing(world: &mut World, key: u8, budget: u64) -> u64 {
    let ask = standing_ask(world, budget);
    let run = assigned_chat(world, key, ask);
    park_chat(world, &run);
    run.task
}
pub(crate) fn owner(world: &mut World, owner: s::FollowOwner, key: u8) {
    let mut value = setting(s::FollowStart::Owner);
    value.owner = owner;
    let before = revision(world);
    change_setting(world, Some(value), key);
    assert_eq!(revision(world), before + 1, "owner edit accepted: {:?}", world.web_replies);
    world.drain_cut();
}
pub(crate) fn idle(world: &mut World, role: u64) {
    settle(world);
    if matches!(live_task(world, role).phase, tasks::Phase::Active(tasks::Active::Running { .. })) {
        let run = world.assigned.iter().rev().find(|run| run.task == role).expect("fixture value").clone();
        world.scripted_read_inbox(&run);
        let turns = live_task(world, role).turn;
        park_after_turn(world, &run, None, turns);
        settle(world);
    }
}
fn ready_role(world: &mut World, role: u64) -> engine::Assignment {
    for _ in 0..TICKS {
        if let Some(run) = world.assigned.iter().rev().find(|run| run.task == role)
            && matches!(live_task(world, role).phase, tasks::Phase::Active(tasks::Active::Running { .. }))
        {
            return run.clone();
        }
        world.tick();
    }
    panic!("role did not wake: {:?}", live_task(world, role));
}
fn relayed(world: &mut World, change: u64) -> forge_top::follows::FollowQuestion {
    for _ in 0..TICKS {
        if let Some(question) = row(world).questions.iter().find(|q| q.change == change && q.relayed.is_some())
            && world.store.pending.is_empty()
        {
            return question.clone();
        }
        world.tick();
    }
    panic!("question not relayed: {:?}", row(world));
}
fn answered(world: &mut World, producer: u64, change: u64, question: u64, words: &[u8]) {
    for _ in 0..TICKS {
        if questions::replies(world, producer, change)
            .iter()
            .any(|word| word.kind == tasks::MessageKind::Answer { question } && word.words.as_ref() == words)
        {
            return;
        }
        world.tick();
    }
    panic!("answer missing: {:?}", questions::replies(world, producer, change));
}
fn owned(start: s::FollowStart, budget: u64, passes: u32) -> (World, u64) {
    let mut world = unaccepted_follow(passes, Gate::default(), false, true);
    let role = standing(&mut world, 150, budget);
    let mut value = setting(start);
    value.owner = s::FollowOwner::Task(role);
    change_setting(&mut world, Some(value), 151);
    assert!(row(&world).stopped.is_none(), "{:?}", row(&world));
    (world, role)
}

#[test]
fn role_follow_is_its_delegate_with_inherited_authority_and_capped_funds() {
    // The role can run in forty units; the person's follow would receive sixty.
    let (mut world, role) = owned(s::FollowStart::Owner, 40, 1000);
    let follow = row(&world).task.expect("follow made");
    let child = live_task(&world, follow);
    let parent = live_task(&world, role);
    assert_eq!(child.requester, tasks::Party::Task(role));
    assert_eq!(child.funder, tasks::Funder::Task(role));
    assert_eq!(child.executor, tasks::Executor::Procedure { connector: 0, code: 7 });
    let mut authority = parent.authority.clone();
    authority.delegation.depth -= 1;
    authority.budget.spend = 40;
    assert_eq!(child.authority, authority);
    assert_eq!(child.numbers.budget, 40);
    assert_eq!(parent.numbers.reserved, 40);
    world.close();

    let (mut world, _) = owned(s::FollowStart::Owner, 200, 1000);
    let follow = row(&world).task.expect("fixture value");
    assert_eq!(live_task(&world, follow).numbers.budget, 60, "the person's follow ceiling still applies");
    world.close();
}

#[test]
fn role_question_survives_restart_words_answer_nothing_and_answer_resumes_producer() {
    let (mut world, role) = owned(s::FollowStart::Code, 200, 1000);
    let after = push(&mut world, b"Port vsr_tick");
    let change = change(&world, after);
    let producer = next_delegate(&mut world, change, 0);
    let question = world.producer_question(&producer, 152, b"Port `vsr_tick` as a loop or recursion?");
    let open = relayed(&mut world, change);
    let follow = row(&world).task.expect("fixture value");
    let asked = open.relayed.expect("fixture value");
    assert!(questions::replies(&world, role, follow).iter().any(|word| {
        word.kind == tasks::MessageKind::Question
            && word.number == asked
            && word.words.as_ref()
                == format!(
                    "Follow 1's change {change} asks (question {}):\n\nPort `vsr_tick` as a loop or recursion?",
                    open.number
                )
                .as_bytes()
    }));
    world.drain_cut();
    world.restart_same();
    settle(&mut world);
    assert_eq!(row(&world).questions.first(), Some(&open), "own question number is durable");
    let run = ready_role(&mut world, role);
    let answer = world.scripted_call(
        &run,
        153,
        engine::Tool::Message {
            target: follow,
            form: engine::MessageForm::Words,
            words: b"Words are not an answer".as_slice().into(),
        },
    );
    assert!(matches!(answer, engine::CallAnswer::Sent { .. }));
    settle(&mut world);
    assert_eq!(row(&world).questions.first(), Some(&open));
    world.goal_answer(&run, 154, follow, asked, b"A loop, as C does.");
    answered(&mut world, producer.task, change, question, b"A loop, as C does.");
    assert!(row(&world).questions.is_empty());
    // The same worker's next turn is admitted after reading the relayed answer.
    world.scripted_read_inbox(&producer);
    assert!(live_task(&world, producer.task).turn > 0);
    let words = "é".repeat(usize::try_from(world.env.limits.tasks.message_bytes / 2).expect("message bound"));
    world.producer_question(&producer, 178, words.as_bytes());
    let cut = relayed(&mut world, change);
    let reply = questions::replies(&world, role, follow)
        .into_iter()
        .find(|word| word.number == cut.relayed.expect("own question"))
        .expect("role question");
    assert!(reply.words.len() <= usize::try_from(world.env.limits.tasks.message_bytes).expect("message bound"));
    assert!(std::str::from_utf8(&reply.words).expect("cut on UTF-8").ends_with('…'));
    world.close();
}

fn independent(world: &mut World, change: u64) {
    // Make both live changes able to ask at once, without completing either port.
    let revision = live_task(world, change).revisions.spec;
    world.send(engine::Event::Party(engine::PartyInput::Ask {
        models: Box::new([]),
        reply_to: ReplyTo::new(Token::new(155)),
        sign_in: world.signed_in.expect("fixture value"),
        key: [155; 16],
        ask: people::Ask::Amend {
            project: 2,
            task: change,
            revision,
            release: None,
            amendment: people::Amendment {
                tracked: None,
                addressed: None,
                instructions: None,
                spec: None,
                wake: None,
                dependencies: Some(Box::new([])),
                authority: None,
                llm: None,
                reason: b"Exercise independent question credits".as_slice().into(),
            },
        },
    }));
    settle(world);
    assert!(live_task(world, change).waiting_on.is_empty());
}
#[test]
fn concurrent_role_questions_answer_their_own_askers_and_cancelled_questions_drop_late_answers() {
    let (mut world, role) = owned(s::FollowStart::Code, 240, 1000);
    let first = push(&mut world, b"First");
    let a = change(&world, first);
    let producer_a = next_delegate(&mut world, a, 0);
    let second = push(&mut world, b"Second");
    let b = change(&world, second);
    independent(&mut world, b);
    let producer_b = next_delegate(&mut world, b, 0);
    let qa = world.producer_question(&producer_a, 156, b"First question");
    let qb = world.producer_question(&producer_b, 157, b"Second question");
    let a_open = relayed(&mut world, a);
    let b_open = relayed(&mut world, b);
    assert_eq!(row(&world).questions.len(), 2);
    assert_ne!(a_open.relayed, b_open.relayed);
    let follow = row(&world).task.expect("fixture value");
    let run = ready_role(&mut world, role);
    world.goal_answer(&run, 158, follow, b_open.relayed.expect("fixture value"), b"Second answer");
    answered(&mut world, producer_b.task, b, qb, b"Second answer");
    assert_eq!(row(&world).questions.first(), Some(&a_open));
    world.goal_answer(&run, 159, follow, a_open.relayed.expect("fixture value"), b"First answer");
    answered(&mut world, producer_a.task, a, qa, b"First answer");
    assert!(row(&world).questions.is_empty());
    let _next = world.producer_question(&producer_a, 160, b"Question left open");
    let left = relayed(&mut world, a);
    cancel(&mut world, a, 161);
    assert!(row(&world).questions.is_empty());
    world.goal_answer(&run, 162, follow, left.relayed.expect("fixture value"), b"Too late");
    settle(&mut world);
    assert!(row(&world).questions.is_empty());
    assert!(live_task(&world, follow).questions.is_empty(), "late answer is taken, not left standing");
    world.close();
}

#[test]
fn owner_edits_remake_the_queue_keep_words_and_holds_and_role_end_stops_the_follow() {
    let mut world = world(s::FollowStart::Owner);
    let role = standing(&mut world, 150, 300);
    let first = push(&mut world, b"First");
    let second = push(&mut world, b"Second");
    let third = push(&mut world, b"Third");
    start(&mut world, first);
    let old = row(&world).clone();
    let a = change(&world, first);
    let producer = next_delegate(&mut world, a, 0);
    assert_eq!(act(&mut world, 163, FollowAct::Hold { after: third, reason: b"wait".as_slice().into() }), None);
    owner(&mut world, s::FollowOwner::Task(role), 164);
    // The scripted cancelled worker drains its run; a producing port starts over.
    let event = world.protocol.answer(
        Token::new(7),
        producer.task,
        producer.attempt,
        0,
        tasks::End::Finished {
            result: tasks::TaskResult::Report { words: b"cancelled".as_slice().into() },
            cancel_delegates: false,
        },
        engine::Parts::none(),
    );
    world.send(event);
    settle(&mut world);
    let follow = row(&world).task.expect("fixture value");
    assert_ne!(Some(follow), old.task);
    assert_eq!(live_task(&world, follow).requester, tasks::Party::Task(role));
    let tasks::Phase::Ended(tasks::Ending::Cancelled { reason, .. }) =
        &ended_task(&world, old.task.expect("fixture value")).phase
    else {
        panic!("old follow cancelled")
    };
    assert_eq!(reason.as_ref(), b"its owner changed");
    let now = row(&world);
    assert!(now.queue[0].word);
    assert!(!now.queue[1].word);
    assert_eq!(now.queue[2].state, FollowState::HeldByOwner(b"wait".as_slice().into()));
    let changes: Vec<_> = [first, second, third].map(|after| change(&world, after)).into_iter().collect();
    for (at, task) in changes.iter().enumerate() {
        assert_ne!(Some(*task), old.queue[at].change);
        assert_eq!(
            live_task(&world, *task).dependencies.as_ref(),
            if at == 0 { &[] } else { std::slice::from_ref(&changes[at - 1]) }
        );
    }
    no_holds(&world, &changes);
    let next_role = standing(&mut world, 176, 200);
    idle(&mut world, role);
    owner(&mut world, s::FollowOwner::Task(next_role), 177);
    assert_ne!(row(&world).task, Some(follow), "a different task requester makes another follow");
    assert_eq!(live_task(&world, row(&world).task.expect("new follow")).requester, tasks::Party::Task(next_role));
    cancel(&mut world, next_role, 165);
    assert!(row(&world).stopped.is_some());
    world.close();
}

#[test]
fn an_independently_ended_entry_stays_ended_across_owner_edits_until_start() {
    let mut world = world(s::FollowStart::Owner);
    let role = standing(&mut world, 150, 200);
    let first = push(&mut world, b"First");
    let second = push(&mut world, b"Second");
    let a = change(&world, first);
    cancel(&mut world, a, 167);
    owner(&mut world, s::FollowOwner::Task(role), 168);
    assert_eq!(change(&world, first), a);
    assert!(matches!(row(&world).queue[0].state, FollowState::Ended(_)));
    assert!(matches!(ended_task(&world, change(&world, second)).phase, tasks::Phase::Ended(_)));
    start(&mut world, first);
    assert_ne!(change(&world, first), a);
    let b = change(&world, second);
    assert_eq!(live_task(&world, b).waiting_on.as_ref(), &[change(&world, first)]);
    no_holds(&world, &[change(&world, first), b]);
    world.close();
}

fn amend_funds(world: &mut World, role: u64, budget: u64, key: u8) {
    let people::Ask::Create { batch, .. } = standing_ask(world, budget) else { panic!("role template") };
    let authority = batch[0].authority.clone();
    let revision = live_task(world, role).revisions.spec;
    world.send(engine::Event::Party(engine::PartyInput::Ask {
        models: Box::new([]),
        reply_to: ReplyTo::new(Token::new(u64::from(key))),
        sign_in: world.signed_in.expect("fixture value"),
        key: [key; 16],
        ask: people::Ask::Amend {
            project: 2,
            task: role,
            revision,
            release: None,
            amendment: people::Amendment {
                tracked: None,
                addressed: None,
                instructions: None,
                spec: None,
                wake: None,
                dependencies: None,
                authority: Some(authority),
                llm: None,
                reason: b"Set the role's available funds".as_slice().into(),
            },
        },
    }));
    settle(world);
    idle(world, role);
    assert_eq!(live_task(world, role).numbers.budget, budget);
}
#[test]
fn refused_owner_funding_stops_the_follow_and_the_next_edit_asks_again() {
    let mut world = unaccepted_follow(1000, Gate::default(), false, true);
    let role = standing(&mut world, 150, 80);
    amend_funds(&mut world, role, 0, 170);
    owner(&mut world, s::FollowOwner::Task(role), 171);
    assert_eq!(
        row(&world).stopped.as_deref(),
        Some(format!("its owner, task {role}, could not make its task: insufficient spend").as_bytes())
    );
    let refused = row(&world).task;
    amend_funds(&mut world, role, 200, 172);
    let mut edited = setting(s::FollowStart::Code);
    edited.owner = s::FollowOwner::Task(role);
    change_setting(&mut world, Some(edited), 173);
    assert!(row(&world).stopped.is_none());
    assert_ne!(row(&world).task, refused);
    assert_eq!(live_task(&world, row(&world).task.expect("fixture value")).funder, tasks::Funder::Task(role));
    world.close();
}

fn escalation(world: &mut World, task: u64, role: u64) -> u64 {
    for _ in 0..TICKS {
        if let tasks::Escalation::Waiting { holder: tasks::EscalationHolder::Task(holder), revision, .. } =
            live_task(world, task).escalation
            && holder == role
        {
            return revision;
        }
        world.tick();
    }
    panic!(
        "hold did not reach role {role}: {:?}; phase {:?}; role phase {:?}; authority {:?}",
        live_task(world, task).escalation,
        live_task(world, task).phase,
        live_task(world, role).phase,
        live_task(world, role).authority
    );
}
#[test]
fn exhausted_repairs_and_the_follows_own_spend_hold_escalate_to_the_role() {
    let (mut world, role) = owned(s::FollowStart::Code, 240, 0);
    let first = push(&mut world, b"First");
    let a = change(&world, first);
    let mut run = next_delegate(&mut world, a, 0);
    produce(&mut world, a, &run);
    for _ in 0..world.env.limits.forge.change_policy.repairs {
        run = next_delegate(&mut world, a, run.task);
        produce(&mut world, a, &run);
    }
    let revision = escalation(&mut world, a, role);
    assert!(matches!(
        questions::row(&world, a).change.state,
        temper_engine_domain_forge_change::State::Held { why: temper_engine_domain_forge_change::Hold::Repairs, .. }
    ));
    let run = ready_role(&mut world, role);
    let answer = world.scripted_call(
        &run,
        174,
        engine::Tool::DecideEscalation { task: a, revision, decision: engine::EscalationChoice::Release },
    );
    assert!(matches!(answer, engine::CallAnswer::EscalationDecided { .. }), "role release: {answer:?}");
    settle(&mut world);
    assert!(!matches!(live_task(&world, a).phase, tasks::Phase::Held { .. }));
    world.close();

    // Queued, unstarted changes reserve the follow's finite funds; the role still has its own.
    let (mut world, role) = owned(s::FollowStart::Owner, 240, 1000);
    let follow = row(&world).task.expect("fixture value");
    for number in 0..8 {
        let message = format!("reserving follow spend {number}");
        let after = proof::landing(&mut world, b"src/a.c", message.as_bytes());
        proof::heard(&mut world, after);
        if matches!(live_task(&world, follow).phase, tasks::Phase::Held { .. }) {
            break;
        }
    }
    assert!(
        matches!(live_task(&world, follow).phase, tasks::Phase::Held { .. }),
        "follow numbers {:?}, queue {:?}",
        live_task(&world, follow).numbers,
        row(&world).queue
    );
    escalation(&mut world, follow, role);
    assert!(row(&world).queue.last().expect("queued shortage").change.is_none());
    world.close();
}
