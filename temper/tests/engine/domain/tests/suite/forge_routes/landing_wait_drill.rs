//! A due readiness step holds forming while its forge read waits (temper/docs/design/domain/landing-turn.md, 2, temper/docs/design/domain/landing.md, 14).
use super::*;

pub(crate) fn waiting_on_due_member() -> (World, u64, u64) {
    // A passed gating head takes its next step on the branch hint. Hold that
    // step's forge read while the first landing ends and gives this train room.
    let (mut world, ready, blocking) = full_landing_room();
    let chat = world.assigned[0].clone();
    let (due, producer) = make_change(&mut world, &chat, 102, b"org/repo");
    push_change(&mut world, &producer, b"org/repo");
    finish_producer(&mut world, &producer, forge_world::REPO);
    world.advance_ticks(u32::try_from(PAIR_TICKS).expect("bounded paired horizon"));
    assert!(
        landing_rows(&world)
            .iter()
            .all(|row| { row.landing.members.iter().all(|member| member.change != ready && member.change != due) }),
        "no room before the hint: {:?}",
        landing_rows(&world)
    );
    // Bring the live blocker to its last write before starting the due step,
    // so the step's bounded read need not wait through preparation and proof.
    world.held_merge_calls = Some(Vec::new());
    world.automatic_prepare_jobs();
    for _ in 0..PAIR_TICKS {
        super::restarts::tick(&mut world);
        if world.held_merge_calls.as_ref().is_some_and(|calls| !calls.is_empty()) {
            break;
        }
    }
    assert_eq!(world.held_merge_calls.as_ref().expect("merge hold").len(), 1);
    world.manual_prepare_jobs();
    world.drain_cut();
    let mut row = world.root.forge().change(due).expect("delivered change").clone();
    row.change.since = world.env.wall;
    row.change.state = temper_engine_domain_forge_change::State::Gating {
        head: row.change.last_head.expect("delivered head"),
        asked: Box::new([]),
    };
    assert!(
        forge_top::gates_settled(&row, None, world.env.limits.forge.change_policy.gates),
        "seeded gates: {:?}, {:?}",
        row.change.gates,
        row.verdicts
    );
    let branch = row.branch.clone();
    world.held_reads = Some(Vec::new());
    world.hold_read_for = Some(due);
    world.send(engine::Event::Forge(Box::new(forge_top::Event::Restore {
        record: forge_top::Stored::Change(Box::new(row)),
    })));
    world.send(engine::Event::Forge(Box::new(forge_top::Event::Hint {
        hint: api::Hint { repository: forge_world::REPO, change: api::Change::Branch(branch), key: None },
    })));
    for _ in 0..PAIR_TICKS {
        super::restarts::tick(&mut world);
        if world.held_reads.as_ref().is_some_and(|reads| !reads.is_empty()) {
            break;
        }
    }
    assert!(world.root.tasks().procedure_due(due).is_some());
    for call in world.held_merge_calls.take().expect("blocked merge") {
        world.out.push(call);
    }
    world.collect();
    for _ in 0..PAIR_TICKS {
        super::restarts::tick(&mut world);
        if world.root.forge().train_free(forge_world::REPO, b"main")
            && world
                .root
                .forge()
                .landing(blocking)
                .is_none_or(|row| matches!(row.landing.phase, forge_top::LandingPhase::Landed))
            && world.held_reads.as_ref().is_some_and(|reads| !reads.is_empty())
            && world.pending.is_empty()
            && world.events.is_empty()
            && world.store.idle()
            && world.root.quiescent()
        {
            return (world, ready, due);
        }
    }
    panic!("the blocking landing frees the room before the held read answers");
}

// Observe this local watch without draining unrelated forge calls held by the story.
fn document(world: &mut World, sign_in: u64, subject: u64, watcher: u64) -> forge_top::Document {
    use jig_domain_core::documents;
    let watcher = Token::new(watcher);
    world.oracle_documents.clear();
    world.send(engine::Event::Party(engine::PartyInput::Watch {
        watcher,
        sign_in,
        subject: documents::Watched::Application {
            scope: people::ApplicationScope::Project(1),
            connector: 0,
            subject: Token::new(subject),
        },
        window: documents::Window { rows: world.env.limits.documents.rows, after: None },
    }));
    for _ in 0..PAIR_TICKS {
        if let Some((_, document)) = world.oracle_documents.iter().find(|(id, _)| *id == watcher) {
            let document = document.clone();
            world.send(engine::Event::Party(engine::PartyInput::Unwatch { watcher }));
            return document;
        }
        world.tick();
    }
    panic!("local document delivered");
}

#[test]
fn drill_20_a_due_change_blocks_formation_without_an_account_in_the_train_view() {
    let (mut world, ready, due) = waiting_on_due_member();
    assert_eq!(
        world.root.tasks().procedure_due(due).map(|(_, code, _)| code),
        Some(2),
        "at {:?}: {:?}, {:?}",
        world.env.now,
        world.root.tasks().task(due).map(|row| &row.phase),
        world.root.forge().change(due).map(|row| &row.change.state)
    );
    assert_eq!(world.held_reads.as_ref().expect("held read").len(), 1);
    assert!(forge_top::gates_settled(
        world.root.forge().change(due).expect("gating member"),
        None,
        world.env.limits.forge.change_policy.gates
    ));
    assert!(matches!(
        super::super::goal_landing::row(&world, ready).change.state,
        temper_engine_domain_forge_change::State::Ready { .. }
    ));
    assert!(
        landing_rows(&world)
            .iter()
            .all(|row| { row.landing.members.iter().all(|member| member.change != ready && member.change != due) }),
        "the train waits for due readiness"
    );
    assert!(!matches!(task_row(&world, due).expect("due task").phase, tasks::Phase::Held { .. }));
    // Local view reads take no simulated time from the held forge call.
    let interval = world.tick_interval;
    world.tick_interval = Duration::ZERO;
    world.signed_in = None;
    world.request_sign_in();
    world.until(Until::SignedIn);
    let sign_in = world.signed_in.expect("person");
    let forge_top::Document::Changes(view) = document(&mut world, sign_in, forge_top::changes_subject(1), 110) else {
        panic!("train view")
    };
    let train = view.queues.iter().find(|queue| queue.branch.as_ref() == b"main").expect("main train");
    assert_eq!(train.changes.iter().map(|change| change.task).collect::<Vec<_>>(), [ready]);
    assert_eq!(train.in_flight, None);
    // The current wire has no formation-wait account; neither view names `due`.
    let forge_top::Document::Landings(view) = document(&mut world, sign_in, forge_top::landings_subject(1, true), 111)
    else {
        panic!("landing view")
    };
    assert!(
        view.landings.iter().all(|landing| { !landing.changes.contains(&ready) && !landing.changes.contains(&due) })
    );
    let chat = world.assigned[0].clone();
    for (key, task) in [(112, ready), (113, due)] {
        let cancel = world.protocol.call(
            Token::new(7),
            chat.task,
            chat.attempt,
            key,
            engine::Call {
                completion: u32::try_from(key).expect("fixture call"),
                position: 1,
                tool: engine::Tool::Cancel { target: task, reason: b"End the readiness drill".as_slice().into() },
            },
        );
        world.send(cancel);
    }
    world.tick_interval = interval;
    world.hold_read_for = None;
    world.oracle_settle();
    world.automatic_prepare_jobs();
    for _ in 0..PAIR_TICKS {
        world.tick();
        if [ready, due]
            .iter()
            .all(|task| task_row(&world, *task).is_some_and(|row| matches!(row.phase, tasks::Phase::Ended(_))))
        {
            break;
        }
    }
    assert!(
        [ready, due]
            .iter()
            .all(|task| { task_row(&world, *task).is_some_and(|row| matches!(row.phase, tasks::Phase::Ended(_))) })
    );
    world.close();
}
