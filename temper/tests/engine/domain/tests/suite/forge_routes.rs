//! Root forge stories with its durable store and the independent fake Forgejo.
use crate::stock;

use jig_domain_authority as authority;
use jig_domain_host as host;
use jig_domain_people as people;
use jig_domain_tasks as tasks;
use skein_fake_forge_domain::{self as fake, api as raw};
use skein_lib::{Duration, ReplyTo, Time, Token};
use std::fmt::Write;
use temper_engine_domain as engine;
use temper_engine_domain::{Key, Record};
use temper_engine_domain_forge as forge_top;
use temper_engine_domain_world::forge_routes::{
    Change, Gate, HoldWrite, StoryLimits, Until, World, change_world, change_world_in, change_world_with_gate,
    change_world_with_policy, failed_ci_with_delayed_brief,
};
use temper_engine_domain_world::{protocol, walking};

use temper_engine_forge_world::{self as forge_world, translate};

const CANCEL_TICKS: u32 = 1000; // The forge world's bound for cancellation and hold release.

fn cancel_task(world: &mut World, task: u64) {
    world.send(engine::Event::Party(engine::PartyInput::Ask {
        models: Box::new([]),
        reply_to: ReplyTo::new(Token::new(951)),
        sign_in: world.signed_in.expect("owner signed in"),
        key: [96; 16],
        ask: people::Ask::Cancel { project: 1, task, reason: b"stop the landing".as_slice().into(), grace: None },
    }));
    if let Some(run) = world.assigned.iter().find(|run| run.task == task).cloned() {
        // The scripted worker drains its cancelled run and returns its racing result.
        goal_landing::finish_run(world, &run, tasks::TaskResult::Report { words: b"Done".as_slice().into() });
    }
    for _ in 0..CANCEL_TICKS {
        world.tick();
        if goal_landing::ended(world, task).is_some() {
            break;
        }
    }
    assert!(matches!(
        goal_landing::ended(world, task).expect("cancelled task ended").phase,
        tasks::Phase::Ended(tasks::Ending::Cancelled { .. })
    ));
    world.until(Until::Ready);
}

/// Approval-rule stories shared with the change-panel regression.
#[path = "forge_routes/approval_identity.rs"]
pub(super) mod approval_identity;
mod rules_by_path;

#[path = "forge_routes/context_projects.rs"]
mod context_projects;
#[path = "forge_routes/kind_charters.rs"]
mod kind_charters;
#[path = "forge_routes/resource_admission.rs"]
mod resource_admission;

#[path = "forge_routes/asks_no_one.rs"]
mod asks_no_one;

#[path = "forge_routes/questions.rs"]
mod questions;

#[path = "forge_routes/goal_closer.rs"]
mod goal_closer;
#[path = "forge_routes/goal_closes.rs"]
mod goal_closes;
#[path = "forge_routes/status.rs"]
mod status;

#[path = "forge_routes/flights.rs"]
mod flights;

#[path = "forge_routes/paths.rs"]
mod paths;

#[path = "forge_routes/delegates.rs"]
mod delegates;

#[path = "forge_routes/change_tool.rs"]
mod change_tool;

#[path = "forge_routes/light_rules.rs"]
mod light_rules;

#[path = "forge_routes/catch_up.rs"]
mod catch_up;

#[path = "forge_routes/merge_job.rs"]
mod merge_job;

#[path = "forge_routes/plain_change.rs"]
mod plain_change;

#[path = "forge_routes/held_change.rs"]
mod held_change;
#[path = "forge_routes/landing.rs"]
mod landing;
#[path = "forge_routes/producer_failure.rs"]
mod producer_failure;
mod question_target;
#[path = "forge_routes/repair_cancel.rs"]
mod repair_cancel;
mod repair_cause;
mod repair_green;
mod resolution_head;
mod stall_hold;

#[path = "forge_routes/result.rs"]
mod result;

mod red_findings;
#[path = "forge_routes/tiers.rs"]
mod tiers;
mod triager_prover;

#[path = "forge_routes/brief_failure.rs"]
mod brief_failure;

#[path = "forge_routes/repairs.rs"]
mod repairs;

#[path = "forge_routes/goal_tier.rs"]
mod goal_tier;

#[path = "forge_routes/reviews.rs"]
mod reviews;

#[path = "forge_routes/b2_tree.rs"]
mod b2_tree;

mod check_timeout;
mod completed_change;
mod completed_reads;
mod saved_close;
#[path = "forge_routes/saved_retry.rs"]
mod saved_retry;

#[path = "forge_routes/goal_tool.rs"]
mod goal_tool;

#[path = "forge_routes/goal_branch.rs"]
mod goal_branch;

#[path = "forge_routes/per_wake.rs"]
mod per_wake;

#[path = "forge_routes/goal_changes.rs"]
mod goal_changes;

#[path = "forge_routes/change_producer.rs"]
mod change_producer;

#[path = "forge_routes/early_start.rs"]
mod early_start;
mod follows;

#[path = "forge_routes/goal_questions.rs"]
mod goal_questions;

mod goal_delivery;
#[path = "forge_routes/goal_landing.rs"]
mod goal_landing;

#[path = "forge_routes/goal_landing_held.rs"]
mod goal_landing_held;

#[path = "forge_routes/hints.rs"]
mod hints;

#[path = "forge_routes/ci_poll.rs"]
mod ci_poll;
mod update_first;
mod update_lost;

#[path = "forge_routes/documents.rs"]
mod documents;

#[path = "forge_routes/goal_landing_title.rs"]
mod goal_landing_title;

#[test]
#[expect(clippy::too_many_lines, reason = "one race follows the kept merge through refusal and Met to landing")]
fn a_merge_asked_while_kept_is_refused_naming_the_pending() {
    let (mut world, chat, producer, branch) = change_world(false, 1000, None);
    let c = match live_task(&world, producer.task).requester {
        tasks::Party::Task(task) => task,
        tasks::Party::Person(_) | tasks::Party::Deployment { .. } => panic!("producer belongs to its change"),
    };
    let event = world.protocol.call(
        Token::new(7),
        chat.task,
        chat.attempt,
        791,
        engine::Call {
            completion: 1,
            position: 2,
            tool: engine::Tool::Delegate {
                batch: Box::new([temper_engine_domain_world::fixtures::report_delegate(b"dependency e", Box::new([]))]),
            },
        },
    );
    world.send(event);
    world.external(2, raw::Op::Git(raw::Git::Create { branch: branch.clone(), commit: 1 }));
    fake::advance(&mut world.fake, &world.fake_env, b"org/repo", &branch, b"file", b"fixed", 1)
        .expect("producer pushed");
    let event = world.protocol.answer(
        Token::new(7),
        producer.task,
        producer.attempt,
        5,
        tasks::End::Finished {
            result: tasks::TaskResult::Change {
                title: jig_domain_tasks::ChangeTitle::new(b"Change".as_slice().into()),
                connector: 0,
                kind: 2,
                resource: temper_engine_domain_forge::tag(forge_world::REPO.forge, forge_world::REPO.repository),
                words: b"pushed".as_slice().into(),
            },
            cancel_delegates: false,
        },
        engine::Parts::none(),
    );
    world.held_reads = Some(Vec::new());
    world.send(event);
    for _ in 0..200 {
        world.tick();
        if world.held_reads.as_ref().is_some_and(|reads| !reads.is_empty()) && world.assigned.len() > 2 {
            break;
        }
    }
    assert!(
        world.held_reads.as_ref().is_some_and(|reads| !reads.is_empty()),
        "the first, ready change is reading its merge facts: rows={:?}; assigned={:?}; merges={}",
        world.store.rows.values().filter(|row| matches!(row, Record::Forge { .. })).collect::<Vec<_>>(),
        world.assigned.iter().map(|run| run.task).collect::<Vec<_>>(),
        world.merges
    );
    let e = world.assigned.get(2).expect("dependency assigned after producer finished").clone();
    assert_eq!(live_task(&world, e.task).spec.words.as_ref(), b"dependency e");
    let mut amendment = temper_engine_domain_world::fixtures::task_amendment(None, None, b"wait for e");
    amendment.dependencies = Some(Box::new([e.task]));
    let revision = spec_revision(&world, c);
    let event = world.protocol.call(
        Token::new(7),
        chat.task,
        chat.attempt,
        792,
        engine::Call {
            completion: 1,
            position: 3,
            tool: engine::Tool::Amend { keep_parameters: false, target: c, revision, amendment },
        },
    );
    world.send(event);
    for _ in 0..40 {
        world.tick();
        if live_task(&world, c).waiting_on.as_ref() == [e.task] {
            break;
        }
    }
    assert_eq!(
        live_task(&world, c).waiting_on.as_ref(),
        [e.task],
        "amendment committed before merge reads resume: {:?}",
        world.answers
    );
    for read in world.held_reads.take().expect("paused merge reads") {
        world.out.push(read);
    }
    world.collect();
    for _ in 0..80 {
        world.tick();
    }
    assert_eq!(world.merges, 0, "a kept merge never reaches the fake forge");
    let problem = std::iter::from_fn(|| world.root.pop_fact())
        .find_map(|fact| match fact {
            engine::Fact::StepRefused { task, problem, .. } if task == c => Some(problem),
            _ => None,
        })
        .expect("the core refused c's merge step");
    let rendered = temper_engine_smith::answer(&engine::CallAnswer::EffectRefused(problem));
    assert_eq!(rendered.text(), format!("Refused: kept until T{} is done", e.task).as_bytes());
    assert!(matches!(live_task(&world, c).phase, tasks::Phase::Active(_)), "kept change is not held");
    let event = world.protocol.answer(
        Token::new(7),
        e.task,
        e.attempt,
        5,
        tasks::End::Finished {
            result: tasks::TaskResult::Report { words: b"e done".as_slice().into() },
            cancel_delegates: false,
        },
        engine::Parts::none(),
    );
    world.send(event);
    let mut met = false;
    for _ in 0..200 {
        world.tick();
        if let Some(Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Live(row)))) =
            world.store.rows.get(&Key::Core(jig_domain_core::Key::Tasks(tasks::Key::Live(c))))
        {
            met |= row
                .inbox
                .iter()
                .any(|word| word.kind == tasks::MessageKind::Met { dependency: e.task, removed: false });
        }
        if world.store.rows.values().any(|stored| matches!(stored,
            Record::Forge { row, .. } if matches!(row.as_ref(), forge_top::Stored::Change(row)
                if row.task == c && matches!(row.change.state, temper_engine_domain_forge_change::State::Landed { .. }))))
        {
            break;
        }
    }
    assert!(met, "the dependency's completion delivers Met to c");
    assert!(matches!(world.store.rows.get(&Key::Core(jig_domain_core::Key::Tasks(tasks::Key::Ended(e.task)))),
        Some(Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Ended(row)))) if matches!(row.phase, tasks::Phase::Ended(tasks::Ending::Done(_)))));
    assert_eq!(world.merges, 1, "Met wakes the vetoed change and its merge reaches the forge");
    assert!(world.store.rows.values().any(|stored| matches!(stored,
        Record::Forge { row, .. } if matches!(row.as_ref(), forge_top::Stored::Change(row)
            if row.task == c && matches!(row.change.state, temper_engine_domain_forge_change::State::Landed { .. })))));
    world.close();
}

fn live_task(world: &World, task: u64) -> &tasks::TaskRecord {
    match world.store.rows.get(&Key::Core(jig_domain_core::Key::Tasks(tasks::Key::Live(task)))) {
        Some(Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Live(row)))) => row,
        _ => panic!("live task {task}"),
    }
}

#[test]
fn a_pushed_change_is_recovered_before_its_worker_reports_the_head() {
    let (mut world, chat, producer, branch) = change_world(false, 1000, None);
    assert!(matches!(
        world.external(2, raw::Op::Git(raw::Git::Create { branch: branch.clone(), commit: 1 })),
        raw::Answer::Branch(raw::Created::Created)
    ));
    let pushed = fake::advance(&mut world.fake, &world.fake_env, b"org/repo", &branch, b"file", b"fixed", 1)
        .expect("worker push reached the forge");
    assert!(world.store.pending.is_empty(), "push precedes the worker's answer decision");
    world.forge_reads = Some(Vec::new());
    // The connector's lost slot must reread even though c has never been watched.
    world.send(engine::Event::Forge(Box::new(forge_top::Event::Lost {
        task: producer.task,
        attempt: producer.attempt,
    })));
    world.drain_cut();
    world.restart(true, true);
    for _ in 0..200 {
        world.tick();
        if world.root.ready() {
            break;
        }
    }
    assert!(world.root.ready(), "restored root did not become ready");
    let tasks::Party::Task(change) = live_task(&world, producer.task).requester else { panic!("change") };
    assert!(resource_admission::branch_reads(&world, &branch) > 0, "lost writers still read afresh");
    assert!(resource_admission::kept_branch(&world, &branch), "the recovered head joins the keep before the result");
    assert_eq!(resource_admission::names(&world, change).len(), 2, "branch and base before the pull");
    let event = world.protocol.answer(
        Token::new(7),
        producer.task,
        producer.attempt,
        5,
        tasks::End::Finished {
            result: tasks::TaskResult::Change {
                title: jig_domain_tasks::ChangeTitle::new(b"Small fix".as_slice().into()),
                connector: 0,
                kind: 2,
                resource: temper_engine_domain_forge::tag(forge_world::REPO.forge, forge_world::REPO.repository),
                words: Box::from(&b"pushed"[..]),
            },
            cancel_delegates: false,
        },
        engine::Parts::none(),
    );
    world.send(event);
    for _ in 0..200 {
        world.tick();
        if world.store.rows.values().any(|stored| {
            matches!(stored,
                Record::Forge { row, .. } if matches!(row.as_ref(), forge_top::Stored::Change(change)
                    if matches!(change.change.state, temper_engine_domain_forge_change::State::Landed { .. }))
            )
        }) {
            break;
        }
    }
    assert!(
        world.store.rows.values().any(|stored| matches!(stored,
            Record::Forge { row, .. } if matches!(row.as_ref(), forge_top::Stored::Change(change)
                if matches!(change.change.state, temper_engine_domain_forge_change::State::Landed { .. }))
        )),
        "recovered push did not land"
    );
    let raw::Answer::Commit(found) = world.external(1, raw::Op::Read(raw::Read::Branch { branch: branch.clone() }))
    else {
        panic!("pushed branch remains readable")
    };
    assert_eq!(found, pushed, "the recovered procedure used the pushed head");
    assert!(
        world.store.rows.values().any(|stored| matches!(stored,
            Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Live(task))) if task.number == chat.task
        )),
        "the chat still owns the change result"
    );
    world.close();
}

#[test]
fn a_change_whose_ci_never_reports_is_stalled_and_held() {
    let (mut world, chat, producer, branch) = change_world(true, 1000, None);
    assert!(matches!(
        world.external(2, raw::Op::Git(raw::Git::Create { branch: branch.clone(), commit: 1 })),
        raw::Answer::Branch(raw::Created::Created)
    ));
    fake::advance(&mut world.fake, &world.fake_env, b"org/repo", &branch, b"file", b"fixed", 1)
        .expect("producer pushed its branch");
    let event = world.protocol.answer(
        Token::new(7),
        producer.task,
        producer.attempt,
        5,
        tasks::End::Finished {
            result: tasks::TaskResult::Change {
                title: jig_domain_tasks::ChangeTitle::new(b"Small fix".as_slice().into()),
                connector: 0,
                kind: 2,
                resource: temper_engine_domain_forge::tag(forge_world::REPO.forge, forge_world::REPO.repository),
                words: Box::from(&b"pushed"[..]),
            },
            cancel_delegates: false,
        },
        engine::Parts::none(),
    );
    world.send(event);
    for _ in 0..450 {
        world.tick();
    }
    assert!(
        world.store.rows.values().any(|stored| matches!(stored,
            Record::Forge { row, .. } if matches!(row.as_ref(), forge_top::Stored::Change(change)
                if matches!(change.change.state, temper_engine_domain_forge_change::State::Held {
                    why: temper_engine_domain_forge_change::Hold::Stalled, .. }))
        )),
        "CI silence did not stall the change: {:?}",
        world.store.rows.values().filter(|row| matches!(row, Record::Forge { .. })).collect::<Vec<_>>()
    );
    assert!(
        !world.store.rows.values().any(|stored| matches!(stored,
            Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Ended(row))) if row.requester == tasks::Party::Task(chat.task)
        )),
        "stalled change must not report a landing"
    );
    world.close();
}

#[test]
fn a_change_whose_ci_takes_minutes_lands_without_being_held() {
    let stall = temper_service::limits::profile(temper_service::limits::Profile::Small).forge.change_policy.stall;
    let world = World::configured_limits(
        true,
        true,
        true,
        1000,
        paths::Gate::default(),
        None,
        StoryLimits { stall: Some(stall), ..StoryLimits::default() },
    );
    let (mut world, _chat, producer, branch) =
        change_world_in(world, paths::Change { no_ci: false, owner_gate: false, tracked: false });
    assert!(matches!(
        world.external(2, raw::Op::Git(raw::Git::Create { branch: branch.clone(), commit: 1 })),
        raw::Answer::Branch(raw::Created::Created)
    ));
    let head = fake::advance(&mut world.fake, &world.fake_env, b"org/repo", &branch, b"file", b"fixed", 1)
        .expect("producer pushed its branch");
    let event = world.protocol.answer(
        Token::new(7),
        producer.task,
        producer.attempt,
        5,
        tasks::End::Finished {
            result: tasks::TaskResult::Change {
                title: jig_domain_tasks::ChangeTitle::new(b"Small fix".as_slice().into()),
                connector: 0,
                kind: 2,
                resource: temper_engine_domain_forge::tag(forge_world::REPO.forge, forge_world::REPO.repository),
                words: Box::from(&b"pushed"[..]),
            },
            cancel_delegates: false,
        },
        engine::Parts::none(),
    );
    world.send(event);
    // Three minutes of pending CI pass on the virtual clock, without a real wait.
    for _ in 0..1800 {
        world.tick();
    }
    assert_eq!(world.merges, 0, "pending CI cannot land");
    let waiting = world
        .store
        .rows
        .values()
        .find_map(|stored| match stored {
            Record::Forge { row, .. } => match row.as_ref() {
                forge_top::Stored::Change(change) => Some(change),
                _ => None,
            },
            Record::Core(_) => None,
        })
        .expect("the change is durable");
    assert!(
        matches!(waiting.change.state, temper_engine_domain_forge_change::State::Checking { .. }),
        "ordinary CI must still be waiting, never held: {:?}",
        waiting.change.state
    );
    assert!(waiting.change.since.as_nanos() + Duration::from_secs(120).as_nanos() < world.env.wall.as_nanos());
    world.external(
        1,
        raw::Op::Write(raw::Write::Status {
            commit: head,
            context: Box::from(&b"build"[..]),
            state: raw::Check::Passed,
            description: Box::new([]),
            link: Box::new([]),
        }),
    );
    // Main resumes the wait at its stall alarm; the fresh read then sees green.
    // Jump the virtual clock instead of spending ticks on that quiet interval.
    world.env.now = world.env.now.saturating_add(stall);
    for _ in 0..300 {
        world.tick();
        if world.store.rows.values().any(|stored| {
            matches!(stored,
            Record::Forge { row, .. } if matches!(row.as_ref(), forge_top::Stored::Change(change)
                if matches!(change.change.state, temper_engine_domain_forge_change::State::Landed { .. })))
        }) {
            break;
        }
    }
    assert_eq!(world.merges, 1, "ordinary CI lands when it passes");
    assert!(world.store.rows.values().any(|stored| matches!(stored,
        Record::Forge { row, .. } if matches!(row.as_ref(), forge_top::Stored::Change(change)
            if matches!(change.change.state, temper_engine_domain_forge_change::State::Landed { .. })))));
    world.close();
}

#[test]
fn an_agent_review_gate_runs_at_the_head_and_its_approval_lands_the_change() {
    check_gate_landing(false, false);
}

#[test]
fn a_change_to_a_repository_without_ci_lands_on_its_checks() {
    check_gate_landing(true, false);
}

#[test]
fn a_projects_own_enforced_rule_holds_landing_for_its_gate() {
    check_gate_landing(false, true);
}

fn check_gate_landing(no_ci: bool, owner_gate: bool) {
    let (mut world, _chat, producer, branch) =
        change_world_with_policy(no_ci, 1000, None, !owner_gate, no_ci, owner_gate, false);
    assert!(matches!(
        world.external(2, raw::Op::Git(raw::Git::Create { branch: branch.clone(), commit: 1 })),
        raw::Answer::Branch(raw::Created::Created)
    ));
    fake::advance(&mut world.fake, &world.fake_env, b"org/repo", &branch, b"file", b"review this diff", 1)
        .expect("producer pushed review head");
    let event = world.protocol.answer(
        Token::new(7),
        producer.task,
        producer.attempt,
        5,
        tasks::End::Finished {
            result: tasks::TaskResult::Change {
                title: jig_domain_tasks::ChangeTitle::new(b"Small fix".as_slice().into()),
                connector: 0,
                kind: 2,
                resource: temper_engine_domain_forge::tag(forge_world::REPO.forge, forge_world::REPO.repository),
                words: Box::from(&b"pushed"[..]),
            },
            cancel_delegates: false,
        },
        engine::Parts::none(),
    );
    world.send(event);
    for _ in 0..200 {
        world.tick();
        if world.assigned.len() >= 3 {
            break;
        }
    }
    let gate = world.assigned.get(2).expect("agent gate assigned").clone();
    if owner_gate {
        assert!(
            world.store.rows.values().all(|stored| !matches!(stored,
                Record::Forge { row, .. } if matches!(row.as_ref(), forge_top::Stored::Change(change)
                    if matches!(change.change.state, temper_engine_domain_forge_change::State::Landed { .. }))
            )),
            "the project's rule holds landing until its verdict"
        );
    }
    assert!(
        gate.sections.iter().any(|section| section.kind == engine::BriefKind::Forge(engine::ForgeBriefKind::Pull)
            && matches!(&section.body, engine::BriefBody::Text(words)
            if words.windows(b"review this diff".len()).any(|part| part == b"review this diff"))),
        "sections={:?}",
        gate.sections
    );
    assert_eq!(gate.workspace.repositories[0].start, engine::ForgeStart::Branch(branch));
    assert_eq!(gate.workspace.repositories[0].landing, None);
    let event = world.protocol.answer(
        Token::new(7),
        gate.task,
        gate.attempt,
        5,
        tasks::End::Finished {
            result: tasks::TaskResult::Verdict { code: 1, fields: Box::new([]), words: Box::from(&b"approved"[..]) },
            cancel_delegates: false,
        },
        engine::Parts::none(),
    );
    world.send(event);
    for _ in 0..200 {
        world.tick();
        if world.store.rows.values().any(|stored| {
            matches!(stored,
                Record::Forge { row, .. } if matches!(row.as_ref(), forge_top::Stored::Change(change)
                    if matches!(change.change.state, temper_engine_domain_forge_change::State::Landed { .. }))
            )
        }) {
            break;
        }
    }
    assert!(world.store.rows.values().any(|stored| matches!(stored,
        Record::Forge { row, .. } if matches!(row.as_ref(), forge_top::Stored::Change(change)
            if matches!(change.change.state, temper_engine_domain_forge_change::State::Landed { .. })
                && change.verdicts.iter().any(|report| Some(report.head) == change.change.last_head
                    && report.status == temper_engine_domain_forge_change::Status::Passed))
    )));
    world.close();
}

#[test]
fn a_review_asking_for_changes_is_repaired_with_its_remarks_and_reviewed_again() {
    check_gate_repair(false);
}

#[test]
fn a_failing_check_is_repaired_and_checked_again() {
    check_gate_repair(true);
}

fn assert_gate_repair_words(words: &[u8], kind: &str) {
    let expected =
        format!("Repair gate 1 ({kind}) at 975835de1c97: changes requested\nRepair the requested gate changes.");
    let words = core::str::from_utf8(words).expect("repair instruction");
    assert!(words.starts_with(&expected), "{words}");
    assert!(words.contains("Preserve the requested change"), "{words}");
    assert!(words.contains("do not remove content to make a check pass"), "{words}");
}

#[expect(clippy::too_many_lines, reason = "the check repair story spans both gate heads and their worker reports")]
fn check_gate_repair(no_ci: bool) {
    let (mut world, _chat, producer, branch) = change_world_with_policy(no_ci, 1000, None, true, no_ci, false, false);
    assert!(matches!(
        world.external(2, raw::Op::Git(raw::Git::Create { branch: branch.clone(), commit: 1 })),
        raw::Answer::Branch(raw::Created::Created)
    ));
    fake::advance(&mut world.fake, &world.fake_env, b"org/repo", &branch, b"file", b"first version", 1)
        .expect("first head pushed");
    let event = world.protocol.answer(
        Token::new(7),
        producer.task,
        producer.attempt,
        5,
        tasks::End::Finished {
            result: tasks::TaskResult::Change {
                title: jig_domain_tasks::ChangeTitle::new(b"Small fix".as_slice().into()),
                connector: 0,
                kind: 2,
                resource: temper_engine_domain_forge::tag(forge_world::REPO.forge, forge_world::REPO.repository),
                words: Box::from(&b"pushed"[..]),
            },
            cancel_delegates: false,
        },
        engine::Parts::none(),
    );
    world.send(event);
    for _ in 0..200 {
        world.tick();
        if world.assigned.len() >= 3 {
            break;
        }
    }
    let first = world.assigned.get(2).expect("first review assigned").clone();
    let event = world.protocol.answer(
        Token::new(7),
        first.task,
        first.attempt,
        5,
        tasks::End::Finished {
            result: tasks::TaskResult::Verdict {
                code: 2,
                fields: Box::new([]),
                words: Box::from(&b"Please fix the unsafe edge"[..]),
            },
            cancel_delegates: false,
        },
        engine::Parts::none(),
    );
    world.send(event);
    for _ in 0..200 {
        world.tick();
        if world.assigned.len() >= 4 {
            break;
        }
    }
    let repair = world.assigned.get(3).expect("review repair assigned").clone();
    let kind = if no_ci { "check" } else { "review" };
    assert_gate_repair_words(&live_task(&world, repair.task).spec.words, kind);
    assert!(
        repair.sections.iter().any(|section| section.kind == engine::BriefKind::Forge(engine::ForgeBriefKind::Reviews)
            && matches!(&section.body, engine::BriefBody::Text(words)
            if words.windows(b"Please fix the unsafe edge".len()).any(|part| part == b"Please fix the unsafe edge"))),
        "review remarks in repair brief: {:?}",
        repair.sections
    );
    fake::advance(&mut world.fake, &world.fake_env, b"org/repo", &branch, b"file", b"safe revision", 1)
        .expect("repair pushed second head");
    let event = world.protocol.answer(
        Token::new(7),
        repair.task,
        repair.attempt,
        5,
        tasks::End::Finished {
            result: tasks::TaskResult::Change {
                title: jig_domain_tasks::ChangeTitle::new(b"Small fix".as_slice().into()),
                connector: 0,
                kind: 2,
                resource: temper_engine_domain_forge::tag(forge_world::REPO.forge, forge_world::REPO.repository),
                words: Box::from(&b"repaired"[..]),
            },
            cancel_delegates: false,
        },
        engine::Parts::none(),
    );
    world.send(event);
    for _ in 0..200 {
        world.tick();
        if world.assigned.len() >= 5 {
            break;
        }
    }
    let second = world.assigned.get(4).expect("second review assigned").clone();
    assert_ne!(first.task, second.task);
    let event = world.protocol.answer(
        Token::new(7),
        second.task,
        second.attempt,
        5,
        tasks::End::Finished {
            result: tasks::TaskResult::Verdict {
                code: 1,
                fields: Box::new([]),
                words: Box::from(&b"Approved revision"[..]),
            },
            cancel_delegates: false,
        },
        engine::Parts::none(),
    );
    world.send(event);
    for _ in 0..200 {
        world.tick();
        if world.store.rows.values().any(|stored| {
            matches!(stored,
            Record::Forge { row, .. } if matches!(row.as_ref(), forge_top::Stored::Change(change)
                if matches!(change.change.state, temper_engine_domain_forge_change::State::Landed { .. })))
        }) {
            break;
        }
    }
    assert!(world.store.rows.values().any(|stored| matches!(stored,
        Record::Forge { row, .. } if matches!(row.as_ref(), forge_top::Stored::Change(change)
            if matches!(change.change.state, temper_engine_domain_forge_change::State::Landed { .. })
                && change.change.repairs == 1))));
    world.close();
}

/// Temper 03.1's fake workflow step and log literals, read by hand.
const BUILD_STEP: &[u8] = b"Failed step: 1 Run build\nEnd of its log:\n[log bytes cut]\n";
const BUILD_LOG_END: &[u8] = b"failing check build 3\n";
/// Temper 03.1's cancelled job's completion log, read by hand.
const JOB_LOG_END: &[u8] = b"job build cancelled\n";

/// Push the producer's head, fail the named checks, end the producer, and await its repair.
fn failed_ci_repair(
    world: &mut World,
    producer: &engine::Assignment,
    branch: &[u8],
    checks: &[&[u8]],
) -> engine::Assignment {
    assert!(matches!(
        world.external(2, raw::Op::Git(raw::Git::Create { branch: Box::from(branch), commit: 1 })),
        raw::Answer::Branch(raw::Created::Created)
    ));
    let head = fake::advance(&mut world.fake, &world.fake_env, b"org/repo", branch, b"file", b"first", 1)
        .expect("producer pushed its branch");
    for context in checks {
        world.external(
            1,
            raw::Op::Write(raw::Write::Status {
                commit: head,
                context: Box::from(*context),
                state: raw::Check::Failed,
                description: Box::new([]),
                link: Box::new([]),
            }),
        );
    }
    let event = world.protocol.answer(
        Token::new(7),
        producer.task,
        producer.attempt,
        5,
        tasks::End::Finished {
            result: tasks::TaskResult::Change {
                title: jig_domain_tasks::ChangeTitle::new(b"Small fix".as_slice().into()),
                connector: 0,
                kind: 2,
                resource: temper_engine_domain_forge::tag(forge_world::REPO.forge, forge_world::REPO.repository),
                words: Box::from(&b"pushed"[..]),
            },
            cancel_delegates: false,
        },
        engine::Parts::none(),
    );
    world.send(event);
    for _ in 0..200 {
        world.tick();
        if world.assigned.len() >= 3 {
            break;
        }
    }
    world.assigned.get(2).expect("Failed CI assigned a repair").clone()
}

/// The words of a repair's CI section.
fn ci_words(repair: &engine::Assignment) -> &[u8] {
    repair
        .sections
        .iter()
        .find_map(|section| match &section.body {
            engine::BriefBody::Text(words) if section.kind == engine::BriefKind::Forge(engine::ForgeBriefKind::Ci) => {
                Some(words.as_ref())
            }
            engine::BriefBody::Text(_) | engine::BriefBody::Missing(_) => None,
        })
        .unwrap_or_else(|| panic!("No CI section: {:?}", repair.sections))
}

fn contains(words: &[u8], part: &[u8]) -> bool {
    words.windows(part.len()).any(|window| window == part)
}

#[test]
fn a_failed_ci_repair_brief_names_the_failed_step_and_its_logs_end() {
    let (mut world, _chat, producer, branch) = change_world(false, 0, None);
    world.external(
        1,
        raw::Op::Write(raw::Write::Status {
            commit: 1,
            context: Box::from(&b"build"[..]),
            state: raw::Check::Passed,
            description: Box::new([]),
            link: Box::new([]),
        }),
    );
    let repair = failed_ci_repair(&mut world, &producer, &branch, &[b"build"]);
    assert!(
        matches!(repair.workspace.repositories[0].start, engine::ForgeStart::Branch(ref name) if name.as_ref() == branch.as_ref())
    );
    let words = ci_words(&repair);
    assert!(contains(words, BUILD_STEP), "{}", String::from_utf8_lossy(words));
    assert!(contains(words, BUILD_LOG_END), "{}", String::from_utf8_lossy(words));
    world.close();
}

#[test]
fn a_failed_ci_with_no_failed_step_briefs_the_end_of_the_jobs_log() {
    let (mut world, _chat, producer, branch) = change_world(false, 0, None);
    fake::set_failing(&mut world.fake, b"org/repo", b"build", raw::Failing::NoStep);
    let repair = failed_ci_repair(&mut world, &producer, &branch, &[b"build"]);
    let words = ci_words(&repair);
    assert!(
        contains(words, b"the runner's: no step failed; run again 1 time at this head, failed again"),
        "{}",
        String::from_utf8_lossy(words)
    );
    assert!(
        contains(words, b"No step failed. End of the job's log:\n[log bytes cut]\n"),
        "{}",
        String::from_utf8_lossy(words)
    );
    assert!(contains(words, JOB_LOG_END), "{}", String::from_utf8_lossy(words));
    assert!(!contains(words, b"Failed step: "), "{}", String::from_utf8_lossy(words));
    world.close();
}

#[test]
fn an_unreadable_failed_job_log_is_named_and_repair_still_runs() {
    let (mut world, _chat, producer, branch) = change_world(false, 0, None);
    world.fail_job_reads = true;
    let repair = failed_ci_repair(&mut world, &producer, &branch, &[b"build", b"lint"]);
    assert!(live_task(&world, repair.task).spec.words.starts_with(b"Repair CI at 975835de1c97: build, lint\n"));
    let words = ci_words(&repair);
    assert!(contains(words, b"[job log could not be read]"), "{}", String::from_utf8_lossy(words));
    assert!(contains(words, b"Description:"), "{}", String::from_utf8_lossy(words));
    assert!(contains(words, b"Link:"), "{}", String::from_utf8_lossy(words));
    assert!(contains(words, b"Failed check: build\n"), "{}", String::from_utf8_lossy(words));
    assert!(contains(words, b"Failed check: lint\n"), "{}", String::from_utf8_lossy(words));
    assert_eq!(words.windows(b"Description:".len()).filter(|part| *part == b"Description:").count(), 2);
    assert_eq!(words.windows(b"Link:".len()).filter(|part| *part == b"Link:").count(), 2);
    assert!(words.ends_with(b"Repair the failed check.\n"), "{}", String::from_utf8_lossy(words));
    world.close();
}

#[test]
fn a_required_section_past_its_deadline_counts_as_transient_without_an_attempt() {
    let (mut world, chat, producer) = failed_ci_with_delayed_brief();
    let mut repair = None;
    for _ in 0..150 {
        world.tick();
        for stored in world.store.rows.values() {
            let Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Live(row))) = stored else { continue };
            if row.number != chat
                && row.number != producer
                && matches!(row.executor, tasks::Executor::Agent { .. })
                && row.tries.transient > 0
            {
                repair = Some(row.clone());
                break;
            }
        }
        if repair.is_some() {
            break;
        }
    }
    let repair = repair.unwrap_or_else(|| {
        panic!(
            "required forge section timed out while preparing: tasks={:?}; pending={:?}; assigned={:?}",
            world
                .store
                .rows
                .values()
                .filter(|row| matches!(row, Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Live(_)))))
                .collect::<Vec<_>>(),
            world.pending,
            world.assigned,
        )
    });
    assert_eq!(repair.attempt, 0, "no claim was made");
    assert_eq!(repair.tries.transient, 1, "a late required section counts in its class");
    assert_eq!(repair.tries.permanent, 0);
    assert!(matches!(&repair.retry_notice,
        Some(tasks::RetryNotice::Failed { class: tasks::Class::Transient, count: 1, reason: Some(reason), .. })
        if reason.as_ref() == b"ci: not gathered by its deadline"));
    assert_eq!(world.assigned.len(), 2, "repair was never assigned");
    world.close();
}

#[test]
fn an_amendment_while_gathering_discards_the_old_brief_without_claiming() {
    let (mut world, chat, producer) = failed_ci_with_delayed_brief();
    let mut repair = None;
    for _ in 0..150 {
        world.tick();
        for stored in world.store.rows.values() {
            let Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Live(row))) = stored else { continue };
            if row.number != chat
                && row.number != producer
                && matches!(row.executor, tasks::Executor::Agent { .. })
                && row.phase == tasks::Phase::Active(tasks::Active::Preparing)
                && world
                    .pending
                    .values()
                    .any(|op| matches!(op, skein_forge::api::Op::Read(skein_forge::api::Read::Job { .. })))
            {
                repair = Some(row.number);
                break;
            }
        }
        if repair.is_some() {
            break;
        }
    }
    let repair = repair.unwrap_or_else(|| {
        panic!(
            "repair gathering its required section: phases={:?}; pending={:?}",
            world
                .store
                .rows
                .values()
                .filter_map(|stored| match stored {
                    Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Live(row))) =>
                        Some((row.number, row.phase.clone())),
                    _ => None,
                })
                .collect::<Vec<_>>(),
            world.pending
        )
    });
    let boundary = world.assigned.len();
    let prior_attempts: Vec<_> = world.assigned.iter().map(|assignment| assignment.attempt).collect();
    world.send(engine::Event::Party(engine::PartyInput::Ask {
        models: Box::new([]),
        reply_to: ReplyTo::new(Token::new(790)),
        sign_in: world.signed_in.expect("owner signed in"),
        key: [79; 16],
        ask: people::Ask::Amend {
            project: 1,
            task: repair,
            // The person names the specification revision they amend.
            revision: spec_revision(&world, repair),
            release: None,
            amendment: people::Amendment {
                tracked: None,
                addressed: None,
                instructions: None,
                spec: Some(people::Spec {
                    asks: false,
                    words: b"revised repair".as_slice().into(),
                    parameters: Box::new([]),
                    inputs: Box::new([]),
                }),
                wake: None,
                dependencies: None,
                authority: None,
                llm: None,
                reason: b"new failure detail".as_slice().into(),
            },
        },
    }));
    for _ in 0..40 {
        world.tick();
        let Some(Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Live(row)))) =
            world.store.rows.get(&Key::Core(jig_domain_core::Key::Tasks(tasks::Key::Live(repair))))
        else {
            continue;
        };
        if row.refusals > 0 && row.last_message == 2 {
            assert_eq!(row.spec.words.as_ref(), b"revised repair");
            assert_eq!(row.attempt, 0, "old preparation was not claimed");
            assert_eq!(row.tries, tasks::Tries::NONE);
            assert_eq!(world.assigned.len(), 2);
            for _ in 0..250 {
                world.tick();
            }
            // The amended repair may retry; the abandoned preparation may never claim.
            for assignment in &world.assigned[boundary..] {
                assert_eq!(assignment.task, repair, "only the revised repair may be assigned");
                assert!(!prior_attempts.contains(&assignment.attempt), "no prior attempt can return");
                let words = task_brief(assignment);
                assert!(words.starts_with(b"revised repair\n"), "fresh amended spec: {words:?}");
                assert!(!words.windows(b"make the fix".len()).any(|part| part == b"make the fix"));
                assert!(!words.windows(b"Small fix".len()).any(|part| part == b"Small fix"));
            }
            world.close();
            return;
        }
    }
    panic!(
        "amended repair did not abandon its pending brief: {:?}",
        world.store.rows.get(&Key::Core(jig_domain_core::Key::Tasks(tasks::Key::Live(repair))))
    );
}

#[test]
#[expect(clippy::too_many_lines, reason = "one slow blocking review, its preparation and landing")]
fn a_review_whose_forge_reads_take_seconds_is_prepared_once() {
    let brief = temper_service::limits::profile(temper_service::limits::Profile::Small).root.brief;
    let world = World::configured_brief(
        true,
        true,
        false,
        1000,
        paths::Gate { agent: true, ..paths::Gate::default() },
        None,
        Some(brief),
    );
    let (mut world, _chat, producer, branch) =
        change_world_in(world, paths::Change { no_ci: false, owner_gate: false, tracked: false });
    world.external(2, raw::Op::Git(raw::Git::Create { branch: branch.clone(), commit: 1 }));
    let head = fake::advance(&mut world.fake, &world.fake_env, b"org/repo", &branch, b"file", b"review this diff", 1)
        .expect("producer pushed");
    // Even four domain reads take >1s; GitHub expands them into more HTTP requests.
    world.fake_env.limits.latency_min = Duration::from_millis(300);
    world.fake_env.limits.latency_max = Duration::from_millis(300);
    let event = world.protocol.answer(
        Token::new(7),
        producer.task,
        producer.attempt,
        5,
        tasks::End::Finished {
            result: tasks::TaskResult::Change {
                title: jig_domain_tasks::ChangeTitle::new(b"Small fix".as_slice().into()),
                connector: 0,
                kind: 2,
                resource: forge_top::tag(forge_world::REPO.forge, forge_world::REPO.repository),
                words: Box::from(&b"pushed"[..]),
            },
            cancel_delegates: false,
        },
        engine::Parts::none(),
    );
    world.send(event);
    let mut review = None;
    for _ in 0..300 {
        world.tick();
        review = world.store.rows.values().find_map(|stored| match stored {
            Record::Forge { row, .. } => match row.as_ref() {
                forge_top::Stored::Change(row) => match row.delegate {
                    Some((task, temper_engine_domain_forge_change::Delegate::Gate { .. }))
                        if live_task(&world, task).phase == tasks::Phase::Active(tasks::Active::Preparing) =>
                    {
                        Some(task)
                    }
                    _ => None,
                },
                _ => None,
            },
            Record::Core(_) => None,
        });
        if review.is_some() {
            break;
        }
    }
    let review = review.expect("blocking review preparing");
    let preparing = world.env.now;
    for _ in 0..300 {
        world.tick();
        if world.assigned.iter().any(|assignment| assignment.task == review) {
            break;
        }
    }
    let row = live_task(&world, review);
    assert_eq!(row.refusals, 0, "first preparation: task={review}, phase={:?}, attempt={}", row.phase, row.attempt);
    let assignments: Vec<_> = world.assigned.iter().filter(|assignment| assignment.task == review).collect();
    let [assignment] = assignments.as_slice() else { panic!("review assigned once: {assignments:?}") };
    assert!(world.env.now.as_nanos() - preparing.as_nanos() > Duration::from_secs(1).as_nanos());
    assert!(task_brief(assignment).starts_with(b"Review this change at the named head\n"));
    let event = world.protocol.answer(
        Token::new(7),
        review,
        assignment.attempt,
        5,
        tasks::End::Finished {
            result: tasks::TaskResult::Verdict { code: 1, fields: Box::new([]), words: Box::from(&b"approved"[..]) },
            cancel_delegates: false,
        },
        engine::Parts::none(),
    );
    world.send(event);
    world.external(
        1,
        raw::Op::Write(raw::Write::Status {
            commit: head,
            context: Box::from(&b"pulse"[..]),
            state: raw::Check::Passed,
            description: Box::new([]),
            link: Box::new([]),
        }),
    );
    for _ in 0..300 {
        world.tick();
        if world.merges > 0 {
            break;
        }
    }
    assert_eq!(world.merges, 1, "the first review's verdict lands its change");
    assert_eq!(world.assigned.iter().filter(|assignment| assignment.task == review).count(), 1);
    let reviews: Vec<_> = world
        .store
        .rows
        .values()
        .filter_map(|stored| match stored {
            Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Live(row) | tasks::Stored::Ended(row)))
                if row.spec.words.starts_with(b"Review this change at the named head\n") =>
            {
                Some(row.number)
            }
            _ => None,
        })
        .collect();
    assert_eq!(reviews, [review], "only one review task is needed");
    world.close();
}

#[test]
#[expect(clippy::too_many_lines, reason = "one delayed blocking review, wakes during preparation and its landing")]
fn a_review_whose_forge_reads_take_seconds_is_prepared_once_and_merges() {
    let brief = temper_service::limits::profile(temper_service::limits::Profile::Small).root.brief;
    let world = World::configured_brief(
        true,
        true,
        false,
        1000,
        paths::Gate { agent: true, ..paths::Gate::default() },
        None,
        Some(brief),
    );
    let (mut world, _chat, producer, branch) =
        change_world_in(world, paths::Change { no_ci: false, owner_gate: false, tracked: false });
    world.external(2, raw::Op::Git(raw::Git::Create { branch: branch.clone(), commit: 1 }));
    let head = fake::advance(&mut world.fake, &world.fake_env, b"org/repo", &branch, b"file", b"review this diff", 1)
        .expect("producer pushed");
    // Keep the review's file gather outstanding while the hint's fresh chain runs.
    world.review_files_delay = Some(Duration::from_secs(20));
    // Even four domain reads take >1s; GitHub expands them into more HTTP requests.
    world.fake_env.limits.latency_min = Duration::from_millis(300);
    world.fake_env.limits.latency_max = Duration::from_millis(300);
    let event = world.protocol.answer(
        Token::new(7),
        producer.task,
        producer.attempt,
        5,
        tasks::End::Finished {
            result: tasks::TaskResult::Change {
                title: jig_domain_tasks::ChangeTitle::new(b"Small fix".as_slice().into()),
                connector: 0,
                kind: 2,
                resource: forge_top::tag(forge_world::REPO.forge, forge_world::REPO.repository),
                words: Box::from(&b"pushed"[..]),
            },
            cancel_delegates: false,
        },
        engine::Parts::none(),
    );
    world.send(event);
    let mut review = None;
    for _ in 0..300 {
        world.tick();
        review = world.store.rows.values().find_map(|stored| match stored {
            Record::Forge { row, .. } => match row.as_ref() {
                forge_top::Stored::Change(row) => match row.delegate {
                    Some((task, temper_engine_domain_forge_change::Delegate::Gate { .. }))
                        if live_task(&world, task).phase == tasks::Phase::Active(tasks::Active::Preparing) =>
                    {
                        Some(task)
                    }
                    _ => None,
                },
                _ => None,
            },
            Record::Core(_) => None,
        });
        if review.is_some() {
            break;
        }
    }
    let review = review.expect("blocking review preparing before wakes");
    let preparing = world.env.now;
    // A changed CI answer and a delivery hint both reach the owned pull while
    // its required brief section is still gathering; neither may replace its gate.
    world.external(
        1,
        raw::Op::Write(raw::Write::Status {
            commit: head,
            context: Box::from(&b"pulse"[..]),
            state: raw::Check::Pending,
            description: Box::new([]),
            link: Box::new([]),
        }),
    );
    world.send(engine::Event::Forge(Box::new(forge_top::Event::Hint {
        hint: skein_forge::api::Hint {
            repository: forge_world::REPO,
            change: skein_forge::api::Change::Item(1),
            key: None,
        },
    })));
    let mut changed_while_preparing = false;
    for _ in 0..300 {
        world.tick();
        if live_task(&world, review).phase == tasks::Phase::Active(tasks::Active::Preparing) {
            changed_while_preparing |= world.store.rows.values().any(|stored| {
                matches!(stored,
                Record::Forge { row, .. } if matches!(row.as_ref(), forge_top::Stored::PullState(pull)
                    if pull.ci == skein_forge::api::Ci::Pending))
            });
        }
        if world.assigned.iter().any(|assignment| assignment.task == review) {
            break;
        }
    }
    assert!(changed_while_preparing, "the pull's Changed reaches its owner during the review gather");
    let row = live_task(&world, review);
    assert_eq!(row.refusals, 0, "first preparation: task={review}, phase={:?}, attempt={}", row.phase, row.attempt);
    let assignments: Vec<_> = world.assigned.iter().filter(|assignment| assignment.task == review).collect();
    let [assignment] = assignments.as_slice() else { panic!("review assigned once: {assignments:?}") };
    assert!(world.env.now.as_nanos() - preparing.as_nanos() > Duration::from_secs(1).as_nanos());
    assert!(task_brief(assignment).starts_with(b"Review this change at the named head\n"));
    let event = world.protocol.answer(
        Token::new(7),
        review,
        assignment.attempt,
        5,
        tasks::End::Finished {
            result: tasks::TaskResult::Verdict { code: 1, fields: Box::new([]), words: Box::from(&b"approved"[..]) },
            cancel_delegates: false,
        },
        engine::Parts::none(),
    );
    world.send(event);
    world.external(
        1,
        raw::Op::Write(raw::Write::Status {
            commit: head,
            context: Box::from(&b"pulse"[..]),
            state: raw::Check::Passed,
            description: Box::new([]),
            link: Box::new([]),
        }),
    );
    for _ in 0..300 {
        world.tick();
        if world.merges > 0 {
            break;
        }
    }
    assert_eq!(world.merges, 1, "the first review's verdict lands its change");
    assert_eq!(world.assigned.iter().filter(|assignment| assignment.task == review).count(), 1);
    let reviews: Vec<_> = world
        .store
        .rows
        .values()
        .filter_map(|stored| match stored {
            Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Live(row) | tasks::Stored::Ended(row)))
                if row.spec.words.starts_with(b"Review this change at the named head\n") =>
            {
                Some(row.number)
            }
            _ => None,
        })
        .collect();
    assert_eq!(reviews, [review], "wakes cannot make a second review task");
    world.close();
}

#[test]
fn a_large_conflicting_pull_is_sized_before_its_brief_deadline() {
    let world = World::configured_brief(
        true,
        true,
        false,
        1000,
        Gate::default(),
        None,
        Some(temper_engine_domain_world::forge_routes::box_limits().brief),
    );
    let (mut world, _chat, producer, branch) = change_world_in(world, Change::default());
    assert!(matches!(
        world.external(2, raw::Op::Git(raw::Git::Create { branch: branch.clone(), commit: 1 })),
        raw::Answer::Branch(raw::Created::Created)
    ));
    fake::advance(&mut world.fake, &world.fake_env, b"org/repo", &branch, b"file", b"branch change", 1)
        .expect("producer pushed branch");
    let base = fake::advance(&mut world.fake, &world.fake_env, b"org/repo", b"main", b"file", b"base change", 1)
        .expect("another change moved the base");
    world.files_too_large = true;
    world.comparison_delay = Some(Duration::from_secs(40));
    let event = world.protocol.answer(
        Token::new(7),
        producer.task,
        producer.attempt,
        5,
        tasks::End::Finished {
            result: tasks::TaskResult::Change {
                title: jig_domain_tasks::ChangeTitle::new(b"Small fix".as_slice().into()),
                connector: 0,
                kind: 2,
                resource: temper_engine_domain_forge::tag(forge_world::REPO.forge, forge_world::REPO.repository),
                words: Box::from(&b"pushed"[..]),
            },
            cancel_delegates: false,
        },
        engine::Parts::none(),
    );
    world.send(event);
    for _ in 0..500 {
        world.tick();
        if world.assigned.len() >= 3 {
            break;
        }
    }
    assert!(
        world.assigned.len() >= 3,
        "conflict did not assign a resolver: tasks={:?}; forge={:?}",
        world
            .store
            .rows
            .values()
            .filter(|row| matches!(row, Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Live(_)))))
            .collect::<Vec<_>>(),
        world.store.rows.values().filter(|row| matches!(row, Record::Forge { .. })).collect::<Vec<_>>()
    );
    let resolver = world.assigned[2].clone();
    assert!(matches!(resolver.workspace.repositories[0].start,
        engine::ForgeStart::Merge { branch: ref source, base: expected }
            if source.as_ref() == branch.as_ref() && expected == translate::commit(base)));
    let limit = world.env.limits.forge.client.answer_bytes;
    let omitted = format!(
        "[files omitted from page 1: past forge.client.answer_bytes ({limit}): at least {} bytes; the pull changes 1 files; read_pull_files from page 2 lists the rest, and read_file reads a path at the head or the base]",
        u64::from(limit) + 1
    );
    let pull = resolver
        .sections
        .iter()
        .find(|section| section.kind == engine::BriefKind::Forge(engine::ForgeBriefKind::Pull))
        .expect("resolver's pull section");
    let engine::BriefBody::Text(words) = &pull.body else { panic!("pull text: {pull:?}") };
    let words = std::str::from_utf8(words).expect("pull text is UTF-8");
    assert!(words.contains(&omitted), "{words}");
    assert!(words.contains("[comparison omitted: not read by the section's cutoff]"), "{words}");
    let Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Live(row))) = world
        .store
        .rows
        .get(&Key::Core(jig_domain_core::Key::Tasks(tasks::Key::Live(resolver.task))))
        .expect("resolver task")
    else {
        panic!("live resolver")
    };
    assert_eq!(row.tries.transient, 0);
    assert_eq!(row.tries.permanent, 0);
    world.close();
}

#[test]
#[expect(clippy::too_many_lines, reason = "one conflict, its resolution and cold restart/readback")]
fn a_conflicting_update_is_resolved_from_a_merge_in_progress() {
    let (mut world, _chat, producer, branch) = change_world(false, 1000, None);
    assert!(matches!(
        world.external(2, raw::Op::Git(raw::Git::Create { branch: branch.clone(), commit: 1 })),
        raw::Answer::Branch(raw::Created::Created)
    ));
    fake::advance(&mut world.fake, &world.fake_env, b"org/repo", &branch, b"file", b"branch change", 1)
        .expect("producer pushed branch");
    let base = fake::advance(&mut world.fake, &world.fake_env, b"org/repo", b"main", b"file", b"base change", 1)
        .expect("another change moved the base");
    let event = world.protocol.answer(
        Token::new(7),
        producer.task,
        producer.attempt,
        5,
        tasks::End::Finished {
            result: tasks::TaskResult::Change {
                title: jig_domain_tasks::ChangeTitle::new(b"Small fix".as_slice().into()),
                connector: 0,
                kind: 2,
                resource: temper_engine_domain_forge::tag(forge_world::REPO.forge, forge_world::REPO.repository),
                words: Box::from(&b"pushed"[..]),
            },
            cancel_delegates: false,
        },
        engine::Parts::none(),
    );
    world.send(event);
    for _ in 0..200 {
        world.tick();
        if world.assigned.len() >= 3 {
            break;
        }
    }
    assert!(
        world.assigned.len() >= 3,
        "conflict did not assign a resolver: task2={:?}; forge={:?}",
        world.store.rows.get(&Key::Core(jig_domain_core::Key::Tasks(tasks::Key::Live(2)))),
        world.store.rows.values().filter(|row| matches!(row, Record::Forge { .. })).collect::<Vec<_>>()
    );
    let resolver = world.assigned[2].clone();
    assert!(matches!(resolver.workspace.repositories[0].start,
        engine::ForgeStart::Merge { branch: ref source, base: expected }
            if source.as_ref() == branch.as_ref() && expected == translate::commit(base)));
    assert!(
        resolver.sections.iter().any(|section| section.kind == engine::BriefKind::Forge(engine::ForgeBriefKind::Pull)
            && matches!(&section.body, engine::BriefBody::Text(words)
            if words.windows(b"What landed in the base".len()).any(|part| part == b"What landed in the base")
                && words.windows(b"Conflicting file".len()).any(|part| part == b"Conflicting file"))),
        "resolution sections: {:?}",
        resolver.sections
    );
    let old = world.fake.branch(b"org/repo", &branch).expect("existing branch head");
    let merge = fake::merge_commit(
        &mut world.fake,
        &world.fake_env.limits,
        old,
        base,
        Box::new([raw::File { path: Box::from(&b"file"[..]), content: Box::from(&b"resolved"[..]) }]),
        b"Resolve conflict",
    )
    .expect("resolved merge commit");
    assert!(matches!(
        world.external(1, raw::Op::Git(raw::Git::Push { branch: branch.clone(), commit: merge, expected: Some(old) })),
        raw::Answer::Pushed(raw::Pushed::Pushed)
    ));
    let event = world.protocol.answer(
        Token::new(7),
        resolver.task,
        resolver.attempt,
        5,
        tasks::End::Finished {
            result: tasks::TaskResult::Change {
                title: jig_domain_tasks::ChangeTitle::new(b"Small fix".as_slice().into()),
                connector: 0,
                kind: 2,
                resource: temper_engine_domain_forge::tag(forge_world::REPO.forge, forge_world::REPO.repository),
                words: Box::from(&b"resolved"[..]),
            },
            cancel_delegates: false,
        },
        engine::Parts::none(),
    );
    world.send(event);
    for _ in 0..200 {
        world.tick();
        if world.store.rows.values().any(|stored| {
            matches!(stored,
                Record::Forge { row, .. } if matches!(row.as_ref(), forge_top::Stored::Change(change)
                    if matches!(change.change.state, temper_engine_domain_forge_change::State::Landed { .. }))
            )
        }) {
            break;
        }
    }
    assert!(
        world.store.rows.values().any(|stored| matches!(stored,
            Record::Forge { row, .. } if matches!(row.as_ref(), forge_top::Stored::Change(change)
                if matches!(change.change.state, temper_engine_domain_forge_change::State::Landed { .. })
                    && change.change.resolutions == 1)
        )),
        "resolved change did not land"
    );
    world.close();
}

#[test]
#[expect(clippy::too_many_lines, reason = "the story checks both clean-update carryover and a new review after repair")]
fn an_approval_carries_over_a_clean_update_and_is_asked_again_after_a_repair() {
    let (mut world, _chat, producer, branch) = change_world(false, 1000, Some(engine::Freshness::Clean));
    assert!(matches!(
        world.external(2, raw::Op::Git(raw::Git::Create { branch: branch.clone(), commit: 1 })),
        raw::Answer::Branch(raw::Created::Created)
    ));
    let first = fake::advance(&mut world.fake, &world.fake_env, b"org/repo", &branch, b"file", b"fix", 1)
        .expect("producer pushed branch");
    let event = world.protocol.answer(
        Token::new(7),
        producer.task,
        producer.attempt,
        5,
        tasks::End::Finished {
            result: tasks::TaskResult::Change {
                title: jig_domain_tasks::ChangeTitle::new(b"Small fix".as_slice().into()),
                connector: 0,
                kind: 2,
                resource: temper_engine_domain_forge::tag(forge_world::REPO.forge, forge_world::REPO.repository),
                words: Box::from(&b"pushed"[..]),
            },
            cancel_delegates: false,
        },
        engine::Parts::none(),
    );
    world.send(event);
    for _ in 0..100 {
        world.tick();
        if world.store.rows.values().any(|stored| matches!(stored,
            Record::Forge { row, .. } if matches!(row.as_ref(), forge_top::Stored::Change(change) if change.pull.is_some())
        )) { break; }
    }
    let pull = world
        .store
        .rows
        .values()
        .find_map(|stored| match stored {
            Record::Forge { row, .. } => match row.as_ref() {
                forge_top::Stored::Change(change) => change.pull,
                _ => None,
            },
            Record::Core(_) => None,
        })
        .expect("opened pull before review");
    assert!(
        !world.store.rows.values().any(|stored| matches!(stored,
            Record::Forge { row, .. } if matches!(row.as_ref(), forge_top::Stored::Change(change)
                if matches!(change.change.state, temper_engine_domain_forge_change::State::Landed { .. }))
        )),
        "approval requirement holds the first head"
    );
    assert!(matches!(
        world.external(
            2,
            raw::Op::Write(raw::Write::Review {
                number: pull,
                verdict: Some(raw::Verdict::Approve),
                body: Box::from(&b"Approved"[..]),
            })
        ),
        raw::Answer::Reviewed(_)
    ));
    let moved = fake::advance(&mut world.fake, &world.fake_env, b"org/repo", b"main", b"other", b"unrelated", 1)
        .expect("base advanced cleanly");
    world.send(engine::Event::Forge(Box::new(temper_engine_domain_forge::Event::Hint {
        hint: skein_forge::api::Hint {
            repository: forge_world::REPO,
            change: skein_forge::api::Change::Branch(Box::from(&b"main"[..])),
            key: None,
        },
    })));
    for _ in 0..240 {
        world.tick();
        if world.store.rows.values().any(|stored| {
            matches!(stored,
                Record::Forge { row, .. } if matches!(row.as_ref(), forge_top::Stored::Change(change)
                    if matches!(change.change.state, temper_engine_domain_forge_change::State::Landed { .. }))
            )
        }) {
            break;
        }
    }
    assert!(
        world.store.rows.values().any(|stored| matches!(stored,
            Record::Forge { row, .. } if matches!(row.as_ref(), forge_top::Stored::Change(change)
                if matches!(change.change.state, temper_engine_domain_forge_change::State::Landed { .. })
                    && change.change.updates == 1 && change.change.last_head != Some(translate::commit(first)))
        )),
        "old approval did not carry across the clean update; base={moved:?}; forge={:?}",
        world.store.rows.values().filter(|row| matches!(row, Record::Forge { .. })).collect::<Vec<_>>()
    );

    let (mut world, _chat, producer, branch) = change_world(false, 0, Some(engine::Freshness::Clean));
    world.external(
        1,
        raw::Op::Write(raw::Write::Status {
            commit: 1,
            context: Box::from(&b"build"[..]),
            state: raw::Check::Passed,
            description: Box::new([]),
            link: Box::new([]),
        }),
    );
    assert!(matches!(
        world.external(2, raw::Op::Git(raw::Git::Create { branch: branch.clone(), commit: 1 })),
        raw::Answer::Branch(raw::Created::Created)
    ));
    let first = fake::advance(&mut world.fake, &world.fake_env, b"org/repo", &branch, b"file", b"broken", 1)
        .expect("producer pushed branch");
    world.external(
        1,
        raw::Op::Write(raw::Write::Status {
            commit: first,
            context: Box::from(&b"build"[..]),
            state: raw::Check::Failed,
            description: Box::new([]),
            link: Box::new([]),
        }),
    );
    let event = world.protocol.answer(
        Token::new(7),
        producer.task,
        producer.attempt,
        5,
        tasks::End::Finished {
            result: tasks::TaskResult::Change {
                title: jig_domain_tasks::ChangeTitle::new(b"Small fix".as_slice().into()),
                connector: 0,
                kind: 2,
                resource: temper_engine_domain_forge::tag(forge_world::REPO.forge, forge_world::REPO.repository),
                words: Box::from(&b"pushed"[..]),
            },
            cancel_delegates: false,
        },
        engine::Parts::none(),
    );
    world.send(event);
    for _ in 0..200 {
        world.tick();
        if world.assigned.len() >= 3 {
            break;
        }
    }
    let repair = world.assigned.get(2).expect("CI failure assigned repair").clone();
    let pull = world
        .store
        .rows
        .values()
        .find_map(|stored| match stored {
            Record::Forge { row, .. } => match row.as_ref() {
                forge_top::Stored::Change(change) => change.pull,
                _ => None,
            },
            Record::Core(_) => None,
        })
        .expect("opened pull for repair");
    assert!(matches!(
        world.external(
            2,
            raw::Op::Write(raw::Write::Review {
                number: pull,
                verdict: Some(raw::Verdict::Approve),
                body: Box::from(&b"Old head approved"[..]),
            })
        ),
        raw::Answer::Reviewed(_)
    ));
    let repaired = fake::advance(&mut world.fake, &world.fake_env, b"org/repo", &branch, b"file", b"repaired", 1)
        .expect("repair pushed new head");
    for _ in 0..5 {
        world.tick();
    }
    world.external(
        1,
        raw::Op::Write(raw::Write::Status {
            commit: repaired,
            context: Box::from(&b"build"[..]),
            state: raw::Check::Passed,
            description: Box::new([]),
            link: Box::new([]),
        }),
    );
    let event = world.protocol.answer(
        Token::new(7),
        repair.task,
        repair.attempt,
        5,
        tasks::End::Finished {
            result: tasks::TaskResult::Change {
                title: jig_domain_tasks::ChangeTitle::new(b"Small fix".as_slice().into()),
                connector: 0,
                kind: 2,
                resource: temper_engine_domain_forge::tag(forge_world::REPO.forge, forge_world::REPO.repository),
                words: Box::from(&b"repaired"[..]),
            },
            cancel_delegates: false,
        },
        engine::Parts::none(),
    );
    world.send(event);
    for _ in 0..100 {
        world.tick();
    }
    assert!(
        !world.store.rows.values().any(|stored| matches!(stored,
            Record::Forge { row, .. } if matches!(row.as_ref(), forge_top::Stored::Change(change)
                if matches!(change.change.state, temper_engine_domain_forge_change::State::Landed { .. }))
        )),
        "old review must not approve repaired head"
    );
    assert!(matches!(
        world.external(
            2,
            raw::Op::Write(raw::Write::Review {
                number: pull,
                verdict: Some(raw::Verdict::Approve),
                body: Box::from(&b"Repaired head approved"[..]),
            })
        ),
        raw::Answer::Reviewed(_)
    ));
    world.send(engine::Event::Forge(Box::new(temper_engine_domain_forge::Event::Hint {
        hint: skein_forge::api::Hint {
            repository: forge_world::REPO,
            change: skein_forge::api::Change::Item(pull),
            key: None,
        },
    })));
    for _ in 0..200 {
        world.tick();
        if world.store.rows.values().any(|stored| {
            matches!(stored,
                Record::Forge { row, .. } if matches!(row.as_ref(), forge_top::Stored::Change(change)
                    if matches!(change.change.state, temper_engine_domain_forge_change::State::Landed { .. }))
            )
        }) {
            break;
        }
    }
    assert!(
        world.store.rows.values().any(|stored| matches!(stored,
            Record::Forge { row, .. } if matches!(row.as_ref(), forge_top::Stored::Change(change)
                if matches!(change.change.state, temper_engine_domain_forge_change::State::Landed { .. })
                    && change.change.repairs == 1)
        )),
        "new review must approve repaired head; forge={:?}",
        world.store.rows.values().filter(|row| matches!(row, Record::Forge { .. })).collect::<Vec<_>>()
    );
    world.close();
}

#[test]
fn a_worker_frozen_past_its_grace_resumes_with_a_push_in_hand_and_lands_nothing_twice() {
    let (mut world, chat, producer, branch) = change_world(false, 1000, None);
    assert!(matches!(
        world.external(2, raw::Op::Git(raw::Git::Create { branch: branch.clone(), commit: 1 })),
        raw::Answer::Branch(raw::Created::Created)
    ));
    let pushed = fake::advance(&mut world.fake, &world.fake_env, b"org/repo", &branch, b"file", b"already pushed", 1)
        .expect("worker pushed before freezing");
    world.send(protocol::lost(Token::new(7)));
    world.send(protocol::hello_slots(Token::new(8), 2, Box::new([])));
    for _ in 0..150 {
        world.tick();
        if world.assigned.iter().any(|run| run.task == producer.task && run.attempt != producer.attempt) {
            break;
        }
    }
    let resumed = world
        .assigned
        .iter()
        .find(|run| run.task == producer.task && run.attempt != producer.attempt)
        .expect("lost producer reassigned")
        .clone();
    assert_eq!(resumed.task, producer.task);
    assert_ne!(resumed.attempt, producer.attempt);
    assert!(
        matches!(resumed.workspace.repositories[0].start, engine::ForgeStart::Branch(ref name)
        if name.as_ref() == branch.as_ref()),
        "resumed worker sees its pushed branch: {:?}",
        resumed.workspace.repositories[0].start
    );
    assert_eq!(resumed.workspace.repositories[0].landing.as_deref(), Some(b"main".as_slice()));
    assert_eq!(world.fake.branch(b"org/repo", &branch), Some(pushed));
    let event = world.protocol.answer(
        Token::new(8),
        resumed.task,
        resumed.attempt,
        5,
        tasks::End::Finished {
            result: tasks::TaskResult::Change {
                title: jig_domain_tasks::ChangeTitle::new(b"Small fix".as_slice().into()),
                connector: 0,
                kind: 2,
                resource: temper_engine_domain_forge::tag(forge_world::REPO.forge, forge_world::REPO.repository),
                words: Box::from(&b"pushed"[..]),
            },
            cancel_delegates: false,
        },
        engine::Parts::none(),
    );
    world.send(event);
    for _ in 0..200 {
        world.tick();
    }
    let landed = world
        .store
        .rows
        .values()
        .filter_map(|stored| match stored {
            Record::Forge { row, .. } => match row.as_ref() {
                forge_top::Stored::Change(change) => match change.change.state {
                    temper_engine_domain_forge_change::State::Landed { merge, .. } => Some(merge),
                    _ => None,
                },
                _ => None,
            },
            Record::Core(_) => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(landed.len(), 1, "one landing after a frozen worker resumes");
    let merge_id = landed[0].iter().fold(String::new(), |mut id, byte| {
        write!(id, "{byte:02x}").expect("writing a merge id into a string");
        id
    });
    assert_eq!(
        world
            .store
            .rows
            .values()
            .filter_map(|stored| match stored {
                Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Ended(row)))
                    if row.requester == tasks::Party::Task(chat.task) =>
                    match &row.phase {
                        tasks::Phase::Ended(tasks::Ending::Done(tasks::TaskResult::Change { words, .. })) =>
                            Some(words.as_ref()),
                        _ => None,
                    },
                _ => None,
            })
            .collect::<Vec<_>>(),
        [&merge_id.as_bytes()[..32]],
        "one result after recovery"
    );
    world.close();
}

#[test]
#[expect(clippy::too_many_lines, reason = "the base repair story drives two consecutive change landings")]
fn a_broken_landing_branch_gets_one_deployment_repair_before_waiting_changes_land() {
    let (mut world, _chat, producer, branch) = change_world(false, 1000, None);
    assert!(matches!(
        world.external(
            1,
            raw::Op::Write(raw::Write::Status {
                commit: 1,
                context: Box::from(&b"build"[..]),
                state: raw::Check::Failed,
                description: Box::new([]),
                link: Box::new([])
            })
        ),
        raw::Answer::Done
    ));
    assert!(matches!(
        world.external(2, raw::Op::Git(raw::Git::Create { branch: branch.clone(), commit: 1 })),
        raw::Answer::Branch(raw::Created::Created)
    ));
    fake::advance(&mut world.fake, &world.fake_env, b"org/repo", &branch, b"file", b"fix", 1)
        .expect("original change pushed");
    let event = world.protocol.answer(
        Token::new(7),
        producer.task,
        producer.attempt,
        5,
        tasks::End::Finished {
            result: tasks::TaskResult::Change {
                title: jig_domain_tasks::ChangeTitle::new(b"Small fix".as_slice().into()),
                connector: 0,
                kind: 2,
                resource: temper_engine_domain_forge::tag(forge_world::REPO.forge, forge_world::REPO.repository),
                words: Box::from(&b"pushed"[..]),
            },
            cancel_delegates: false,
        },
        engine::Parts::none(),
    );
    world.send(event);
    for _ in 0..200 {
        world.tick();
        if world.assigned.len() >= 3 {
            break;
        }
    }
    let repair = world
        .assigned
        .get(2)
        .unwrap_or_else(|| {
            panic!(
                "base failure assigned deployment repair producer; task4={:?}; task5={:?}; assignments={:?}",
                world.store.rows.get(&Key::Core(jig_domain_core::Key::Tasks(tasks::Key::Live(4)))),
                world.store.rows.get(&Key::Core(jig_domain_core::Key::Tasks(tasks::Key::Live(5)))),
                world.assigned
            )
        })
        .clone();
    let repair_branch = repair.workspace.repositories[0].push.as_ref().expect("repair branch").clone();
    assert_ne!(repair_branch, branch);
    assert!(world.store.rows.values().any(|stored| matches!(stored,
        Record::Forge { row, .. } if matches!(row.as_ref(), forge_top::Stored::Change(change)
            if change.base_repair && change.queue_repair.is_none())
    )));
    assert!(matches!(
        world.external(2, raw::Op::Git(raw::Git::Create { branch: repair_branch.clone(), commit: 1 })),
        raw::Answer::Branch(raw::Created::Created)
    ));
    fake::advance(&mut world.fake, &world.fake_env, b"org/repo", &repair_branch, b"base", b"fixed", 1)
        .expect("base repair pushed");
    let event = world.protocol.answer(
        Token::new(7),
        repair.task,
        repair.attempt,
        5,
        tasks::End::Finished {
            result: tasks::TaskResult::Change {
                title: jig_domain_tasks::ChangeTitle::new(b"Small fix".as_slice().into()),
                connector: 0,
                kind: 2,
                resource: temper_engine_domain_forge::tag(forge_world::REPO.forge, forge_world::REPO.repository),
                words: Box::from(&b"repaired"[..]),
            },
            cancel_delegates: false,
        },
        engine::Parts::none(),
    );
    world.send(event);
    for _ in 0..1000 {
        world.tick();
        if world
            .store
            .rows
            .values()
            .filter(|stored| {
                matches!(stored,
                    Record::Forge { row, .. } if matches!(row.as_ref(), forge_top::Stored::Change(change)
                        if matches!(change.change.state, temper_engine_domain_forge_change::State::Landed { .. }))
                )
            })
            .count()
            >= 2
        {
            break;
        }
    }
    assert_eq!(
        world
            .store
            .rows
            .values()
            .filter(|stored| matches!(stored,
                Record::Forge { row, .. } if matches!(row.as_ref(), forge_top::Stored::Change(change)
                    if matches!(change.change.state, temper_engine_domain_forge_change::State::Landed { .. }))
            ))
            .count(),
        2,
        "deployment repair and waiting change land; forge={:?}",
        world.store.rows.values().filter(|row| matches!(row, Record::Forge { .. })).collect::<Vec<_>>()
    );
    world.close();
}

fn adoption_chat(world: &World) -> people::Ask {
    let people::Ask::Create { project, mut batch } = world.chat(1, &[b'x'; 128]) else { unreachable!() };
    // Three standing chats share the owner's 500-unit allotment; each run reserves at most 60.
    batch[0].budget = people::Budget::AsAllowed { cap: Some(100) };
    people::Ask::Create { project, batch }
}

fn assigned_chat(world: &mut World, key: u8, ask: people::Ask) -> engine::Assignment {
    let before = world.assigned.len();
    world.send(engine::Event::Party(engine::PartyInput::Ask {
        models: Box::new([]),
        reply_to: ReplyTo::new(Token::new(u64::from(key))),
        sign_in: world.signed_in.expect("owner session"),
        key: [key; 16],
        ask,
    }));
    for _ in 0..200 {
        world.tick();
        if world.assigned.len() > before {
            return world.assigned[before].clone();
        }
    }
    panic!(
        "chat request {key} was not assigned; tasks={:?}; answers={:?}",
        world
            .store
            .rows
            .values()
            .filter_map(|row| match row {
                Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Live(task))) =>
                    Some((task.number, &task.phase)),
                _ => None,
            })
            .collect::<Vec<_>>(),
        world
            .store
            .rows
            .values()
            .filter_map(|row| match row {
                Record::Core(jig_domain_core::Record::People(people::Stored::Answer { outcome, .. })) => Some(outcome),
                _ => None,
            })
            .collect::<Vec<_>>()
    );
}

fn park_chat(world: &mut World, assignment: &engine::Assignment) {
    let read = assignment.messages.last().map(|word| word.number);
    if read.is_some() {
        world.send(protocol::turn(Token::new(7), assignment.task, assignment.attempt, 1, 0, read, b"Read the message"));
        world.until(Until::Ready);
    }
    park_after_turn(world, assignment, read, u32::from(read.is_some()));
    world.until(Until::Ready);
}

fn park_after_turn(world: &mut World, assignment: &engine::Assignment, read: Option<u64>, turns: u32) {
    let mut event = world.protocol.answer(
        Token::new(7),
        assignment.task,
        assignment.attempt,
        0,
        tasks::End::Parked,
        engine::Parts::none(),
    );
    let engine::Event::Worker(engine::WorkerInput::Link {
        link: engine::Link::Up { up: host::LinkUp::Answer { answer, .. }, .. },
        ..
    }) = &mut event
    else {
        unreachable!("the protocol's answer is a worker answer")
    };
    // A resumed chat consumes its Say before parking, rather than waking on it again.
    answer.read = read.map(Token::new);
    answer.turns = turns;
    world.send(event);
}

fn task_brief(assignment: &engine::Assignment) -> &[u8] {
    assignment
        .sections
        .iter()
        .find_map(|section| match &section.body {
            engine::BriefBody::Text(words) if section.kind == engine::BriefKind::Core(jig_domain_brief::Core::Task) => {
                Some(words.as_ref())
            }
            _ => None,
        })
        .expect("required Task section")
}

#[test]
fn a_chat_made_before_its_home_was_adopted_reads_the_home_and_says_why() {
    let small = temper_service::limits::profile(temper_service::limits::Profile::Small);
    let mut world =
        World::configured_brief(true, true, false, 1000, paths::Gate::default(), None, Some(small.root.brief));
    assert_eq!(world.env.limits.brief.budgets.task, 4608);
    assert_eq!(world.env.limits.brief.budgets.pull, 4608);
    assert_eq!(world.env.limits.brief.brief_bytes, 14_592);
    world.sign_in();
    world.send(protocol::hello_slots(Token::new(7), 2, Box::new([])));
    let mut chats = Vec::new();
    for key in [94, 95] {
        let mut ask = adoption_chat(&world);
        let people::Ask::Create { batch, .. } = &mut ask else { panic!("chat creation") };
        batch[0].flags.ended_by = people::EndedBy::Requester;
        let assignment = assigned_chat(&mut world, key, ask);
        assert!(live_task(&world, assignment.task).holdings.is_empty(), "no home exists to hold");
        assert!(
            task_brief(&assignment).windows(128).any(|words| words == [b'x'; 128]),
            "the whole task text before adoption"
        );
        assert!(
            !assignment
                .sections
                .iter()
                .any(|section| section.kind == engine::BriefKind::Forge(engine::ForgeBriefKind::Home))
        );
        park_chat(&mut world, &assignment);
        chats.push(assignment);
    }
    world.adopt();
    // Parked attempts are no longer hosted; retain their identities in chats alone.
    world.assigned.clear();
    world.restart(true, true);
    world.until(Until::Ready);
    world.send(protocol::hello_slots(Token::new(7), 2, Box::new([])));
    let reason = b"This chat began before org/repo was adopted, so you can read org/repo but not push to it. Tell the person, in your next reply: \"This chat began before org/repo was adopted, so I can read it but not push. Start a new chat to push.\"";
    for (chat, key) in chats.iter().zip([96, 97]) {
        let assignment = assigned_chat(
            &mut world,
            key,
            people::Ask::Say { project: 1, task: chat.task, words: Box::from(&b"Continue"[..]) },
        );
        assert_eq!(assignment.task, chat.task);
        assert!(assignment.attempt > chat.attempt);
        let [home] = assignment.workspace.repositories.as_ref() else { panic!("one home checkout") };
        assert_eq!(home.provider, forge_world::REPO);
        assert_eq!(home.push, None);
        assert_eq!(home.landing, None);
        assert_eq!(task_brief(&assignment), task_brief(chat), "the person's task text is unchanged");
        assert!(task_brief(&assignment).windows(128).any(|words| words == [b'x'; 128]), "the whole task text, uncut");
        let home_sections: Vec<_> = assignment
            .sections
            .iter()
            .filter(|section| section.kind == engine::BriefKind::Forge(engine::ForgeBriefKind::Home))
            .collect();
        assert_eq!(home_sections.len(), 1, "one required Home section");
        assert_eq!(home_sections[0].body, engine::BriefBody::Text(reason.as_slice().into()));
        assert_eq!(live_task(&world, chat.task).hold, 0, "the chat was never held for Effects");
        assert!(live_task(&world, chat.task).holdings.is_empty(), "read access acquires no holding");
        park_chat(&mut world, &assignment);
    }
    let ask = adoption_chat(&world);
    let assignment = assigned_chat(&mut world, 98, ask);
    let task = live_task(&world, assignment.task);
    let branch = format!("temper/{}/r{}", task.root, task.number).into_bytes();
    assert!(task.holdings.iter().any(|holding| matches!(holding,
    tasks::Holding::Write { resource, kind: 1 }
        if resource.connector == 0 && resource.path.as_ref() == [
            Box::from(forge_world::REPO.forge.to_be_bytes()),
            Box::from(forge_world::REPO.repository.to_be_bytes()),
            branch.clone().into_boxed_slice(),
        ])));
    let [home] = assignment.workspace.repositories.as_ref() else { panic!("one writable home checkout") };
    assert_eq!(home.push.as_deref(), Some(branch.as_slice()));
    assert!(
        !assignment
            .sections
            .iter()
            .any(|section| section.kind == engine::BriefKind::Forge(engine::ForgeBriefKind::Home))
    );
    world.close();
}

#[test]
fn an_asking_or_readless_charter_gets_no_workspace_and_every_family_keeps_the_home_and_context() {
    use jig_domain_core::settings as s;

    let policy = walking::charter().policy;
    for (tools, inspect, modify, shell, empty) in [
        (engine::RunPolicy::EVERY_FAMILY, policy.inspect, policy.modify, policy.shell, false),
        (tasks::Tools(3072), policy.inspect, policy.modify, policy.shell, true),
        (engine::RunPolicy::EVERY_FAMILY, false, false, false, true),
    ] {
        let mut world = World::configured(true, false);
        world.adopt();
        let mut charter = walking::charter();
        charter.policy.tools = tools;
        charter.policy.inspect = inspect;
        charter.policy.modify = modify;
        charter.policy.shell = shell;
        world.send(engine::Event::Party(engine::PartyInput::ChangeSettings {
            reply_to: ReplyTo::new(Token::new(300)),
            sign_in: world.signed_in.expect("deployment owner"),
            key: [30; 16],
            digest: [30; 32],
            kinds: Box::new([]),
            change: Box::new(s::Change {
                against: temper_engine_domain_world::kinds::revision(&world.store.rows),
                items: Box::new([s::Item {
                    setting: s::Setting::Charter(1),
                    edit: s::Edit::Set(s::Value::Charter(Box::new(charter))),
                }]),
                running: s::Running::Stays,
            }),
        }));
        world.until(Until::Ready);
        fake::repository(
            &mut world.fake,
            &world.fake_env.limits,
            raw::Setup {
                id: 3,
                name: b"org/extra".as_slice().into(),
                default: b"main".as_slice().into(),
                tree: Box::new([]),
                labels: Box::new([]),
                workflows: Box::new([]),
                protection: None,
                hooked: false,
                checks: raw::Checks {
                    cues: Box::new([]),
                    contexts: Box::new([]),
                    latency_min: Duration::ZERO,
                    latency_max: Duration::ZERO,
                    silent: 0,
                    passes: 1000,
                    reruns: 0,
                    cue: None,
                },
            },
        );
        fake::grant(&mut world.fake, b"org/extra", 1, raw::Permission::Admin);
        let mut context = temper_engine_domain_world::fixtures::adoption(1);
        context.home = false;
        context.name = b"extra".as_slice().into();
        context.role = forge_top::Role::Context;
        world.send(engine::Event::Party(engine::PartyInput::Application {
            scope: people::ApplicationScope::Project(1),
            reply_to: ReplyTo::new(Token::new(95)),
            sign_in: world.signed_in.expect("owner session"),
            key: [95; 16],
            digest: [6; 32],
            carried: None,
            request: engine::Application::Forge(forge_top::Ask::Adopt(context)),
        }));
        for _ in 0..200 {
            world.tick();
            if world.adopted.len() == 2 {
                break;
            }
        }
        assert_eq!(world.adopted.len(), 2, "home and context adopted: {:?}", world.adopted);
        assert!(matches!(world.adopted[1], forge_top::ApplicationOutcome::Adopted(_)), "{:?}", world.adopted);
        world.send(protocol::hello_slots(Token::new(7), 2, Box::new([])));
        let ask = world.chat(1, b"Explain this project");
        let assignment = assigned_chat(&mut world, 92, ask);
        assert_eq!(assignment.run.policy.tools, tools, "the run uses the world's changed charter");
        if empty {
            assert!(
                assignment.workspace.repositories.is_empty(),
                "an asking or readless charter selects no repository"
            );
        } else {
            assert_eq!(assignment.workspace.repositories.len(), 2);
            assert!(
                assignment.workspace.repositories.iter().any(|repository| repository.provider == forge_world::REPO)
            );
            let context = assignment
                .workspace
                .repositories
                .iter()
                .find(|repository| repository.provider.repository == 3)
                .expect("context repository remains selected");
            assert!(context.push.is_none(), "context remains read only");
        }
        world.close();
    }
}

#[test]
fn a_saved_repository_tag_becomes_a_concrete_checkout_in_the_next_attempt() {
    let mut world = World::configured(true, false);
    world.adopt();
    fake::repository(
        &mut world.fake,
        &world.fake_env.limits,
        raw::Setup {
            workflows: Box::new([]),
            id: 3,
            name: Box::from(&b"org/extra"[..]),
            default: Box::from(&b"main"[..]),
            tree: Box::new([]),
            labels: Box::new([]),
            checks: raw::Checks {
                cues: Box::new([]),
                contexts: Box::new([]),
                latency_min: Duration::ZERO,
                latency_max: Duration::ZERO,
                silent: 0,
                passes: 1000,
                reruns: 0,
                cue: None,
            },
            protection: None,
            hooked: false,
        },
    );
    fake::grant(&mut world.fake, b"org/extra", 1, raw::Permission::Admin);
    world.send(engine::Event::Party(engine::PartyInput::Application {
        scope: jig_domain_people::ApplicationScope::Project(1),
        reply_to: ReplyTo::new(Token::new(98)),
        sign_in: world.signed_in.expect("owner session"),
        key: [98; 16],
        digest: [5; 32],
        carried: None,
        request: engine::Application::Forge(forge_top::Ask::Adopt(forge_top::Adoption {
            project: 1,
            home: false,
            forge: 1,
            host: Box::from(&b"forge.example"[..]),
            owner: Box::from(&b"org"[..]),
            name: Box::from(&b"extra"[..]),
            prefix: Box::from(&b"temper/"[..]),
            role: forge_top::Role::Owned,
            landing: Box::from(&b"main"[..]),
            ci: true,
            checks: Box::new([]),
        })),
    }));
    for _ in 0..100 {
        world.tick();
        if world.adopted.len() == 2 {
            break;
        }
    }
    assert!(
        matches!(world.adopted.get(1), Some(forge_top::ApplicationOutcome::Adopted(_))),
        "second repository adopted: {:?}",
        world.adopted
    );
    world.send(protocol::hello_slots(Token::new(7), 2, Box::new([])));
    world.send(engine::Event::Party(engine::PartyInput::Ask {
        models: Box::new([]),
        reply_to: ReplyTo::new(Token::new(99)),
        sign_in: world.signed_in.expect("owner session"),
        key: [99; 16],
        ask: world.chat(1, &b"Work in two repositories"[..]),
    }));
    world.until(Until::Assigned);
    let first = world.assigned[0].clone();
    let event = world.protocol.answer(
        Token::new(7),
        first.task,
        first.attempt,
        0,
        tasks::End::Parked,
        engine::Parts { saved: Some(Box::new([forge_top::tag(1, 3)])), pushed: Box::new([]), preparation: None },
    );
    world.send(event);
    for _ in 0..30 {
        world.tick();
    }
    world.send(engine::Event::Party(engine::PartyInput::Ask {
        models: Box::new([]),
        reply_to: ReplyTo::new(Token::new(100)),
        sign_in: world.signed_in.expect("owner session"),
        key: [100; 16],
        ask: people::Ask::Say { project: 1, task: first.task, words: Box::from(&b"Continue"[..]) },
    }));
    for _ in 0..150 {
        world.tick();
        if world.assigned.len() >= 2 {
            break;
        }
    }
    let second = world.assigned.get(1).expect("saved task reassigned");
    assert_eq!(second.saved.as_ref(), [forge_top::tag(1, 3)]);
    assert!(second.workspace.repositories.iter().any(|repository| repository.tag == forge_top::tag(1, 3)
        && matches!(repository.start, engine::ForgeStart::Saved(ref branch)
            if branch.as_ref() == b"temper/1/s1")));
    world.close();
}

#[test]
fn a_goal_ending_while_its_issue_is_opening_eventually_closes_that_issue() {
    let mut world = World::configured(true, false);
    world.adopt();
    world.send(protocol::hello_slots(Token::new(7), 2, Box::new([])));
    world.send(engine::Event::Party(engine::PartyInput::Ask {
        models: Box::new([]),
        reply_to: ReplyTo::new(Token::new(94)),
        sign_in: world.signed_in.expect("owner session"),
        key: [94; 16],
        ask: walking::goal_from(
            world.chat(1, b"Finish the goal"),
            people::TaskExecutor::Agent { charter: 1 },
            jig_test_profile::fixtures::Goal { budget: 50, priority: 3, deadline: None },
        ),
    }));
    world.until(Until::Assigned);
    let goal = world.assigned[0].clone();
    let event = world.protocol.answer(
        Token::new(7),
        goal.task,
        goal.attempt,
        1,
        tasks::End::Finished {
            result: tasks::TaskResult::Report { words: Box::from(&b"done"[..]) },
            cancel_delegates: false,
        },
        engine::Parts::none(),
    );
    world.send(event);
    for _ in 0..300 {
        world.tick();
        if world.store.rows.values().any(|stored| matches!(stored,
            Record::Forge { row, .. } if matches!(row.as_ref(), forge_top::Stored::Issue(issue) if issue.state.closed)
        )) { break; }
    }
    assert!(
        world.store.rows.values().any(|stored| matches!(stored,
            Record::Forge { row, .. } if matches!(row.as_ref(), forge_top::Stored::Issue(issue) if issue.state.closed)
        )),
        "ended goal's issue did not close"
    );
    world.close();
}

#[test]
fn a_goal_can_subscribe_to_main_without_guessing_repository_ids() {
    let (world, goal, branch) = subscription_goal();
    subscription_after_first_batch(
        world,
        &goal,
        &branch,
        br#"{"topic":{"kind":"landings","branch":"main"},"paths":["file"]}"#,
    );
}

#[test]
fn a_stock_goal_can_subscribe_to_main_with_its_held_branches() {
    for repository in [r#", "repository":{"forge":1,"repository":2}"#, r#", "repository":"org/repo""#, ""] {
        let (world, goal) = stock_subscription_goal();
        let branch = format!("temper/{}/g{}", goal.task, goal.task);
        let json = format!(r#"{{"topic":{{"kind":"landings","branch":"main"{repository}}},"paths":["file"]}}"#);
        subscription_after_first_batch(world, &goal, branch.as_bytes(), json.as_bytes());
    }
}

fn stock_subscription_goal() -> (World, engine::Assignment) {
    let limits = temper_engine_domain_world::forge_routes::box_limits();
    let llm = jig_protocol_config::Llm {
        endpoints: Box::new([]),
        providers: Box::new([]),
        models: (walking::models())
            .into_iter()
            .enumerate()
            .map(|(at, model)| jig_domain_core::settings::CatalogueModel {
                first_read: None,
                number: u32::try_from(at).expect("bounded fixture models") + 1,
                model,
                shown: None,
                description: None,
                picker: true,
                withdrawn: false,
                source: jig_domain_core::settings::ModelSource::Stock,
                estimated: false,
            })
            .collect(),
    };
    let mut defaults = temper_service::stock::with_models(llm.models.clone(), limits.tasks.result_bytes);
    defaults.rules.ceiling.tools = authority::Tools(1967);
    let policy = defaults.policies.get_mut(&1).expect("project policy");
    policy.policy.ceiling.tools = authority::Tools(1967);
    for role in &mut policy.policy.roles {
        role.authority.tools = authority::Tools(1967);
    }
    let mut world = World::configured_stock_limits(&defaults, &walking::models()[0], &limits);
    world.adopt();
    let mut event = protocol::hello_slots(Token::new(7), 2, Box::new([]));
    let engine::Event::Worker(engine::WorkerInput::Link { link: engine::Link::Hello { hello, .. }, .. }) = &mut event
    else {
        panic!("worker hello")
    };
    hello.bounds.turns = 800;
    hello.bounds.time = Duration::from_secs(28_800);
    hello.bounds.spend = defaults.rules.maximum_run_spend;
    hello.bounds.message_bytes = limits.tasks.message_bytes.max(limits.tasks.result_bytes);
    hello.bounds.inbox = limits.tasks.run_inbox;
    world.send(event);
    let people::Ask::Create { project, mut batch } = goal_branch::goal_request(&world, true) else { panic!("goal") };
    batch[0].authority.tools = 1967;
    batch[0].budget = people::Budget::AsAllowed { cap: None };
    batch[0].tracked = Some(1);
    batch[0].symbols[0].connector = 1;
    let goal = assigned_chat(&mut world, 92, people::Ask::Create { project, batch });
    world.until(Until::Ready);
    (world, goal)
}

fn subscription_after_first_batch(mut world: World, goal: &engine::Assignment, branch: &[u8], json: &[u8]) {
    let held = live_task(&world, goal.task).holdings.clone();
    assert_eq!(held.len(), 2, "goal holds g and r");
    let run_branch = format!("temper/{}/r{}", goal.task, goal.task);
    for branch in [branch, run_branch.as_bytes()] {
        assert!(
            held.iter().any(|holding| matches!(holding, tasks::Holding::Write { resource, .. }
            if resource.path.last().is_some_and(|name| name.as_ref() == branch))),
            "held branch"
        );
    }
    let made = change_tool::change_landing(
        &mut world,
        goal,
        93,
        None,
        b"Make the first batch",
        Some(engine::ChangeLanding::Goal),
    );
    assert!(matches!(made, engine::CallAnswer::Delegated(_)), "first batch: {made:?}");
    assert!(
        world.store.rows.values().any(|row| matches!(row,
        Record::Forge { row } if matches!(row.as_ref(), forge_top::Stored::Change(change)
            if change.base.as_ref() == branch))),
        "first batch targets g"
    );
    let answer = subscription_json(&mut world, goal, b"subscribe_forge", 94, json);
    assert!(matches!(answer, engine::CallAnswer::Subscribed { .. }), "subscription: {answer:?}");
    assert_eq!(live_task(&world, goal.task).holdings, held, "subscription retains g and r");
    let event =
        world.protocol.answer(Token::new(7), goal.task, goal.attempt, 0, tasks::End::Parked, engine::Parts::none());
    world.send(event);
    world.until(Until::Ready);
    let before = news_count(&world, goal.task);
    fake::advance(&mut world.fake, &world.fake_env, b"org/repo", b"main", b"file", b"changed", 1)
        .expect("external landing");
    world.send(engine::Event::Forge(Box::new(forge_top::Event::Hint {
        hint: skein_forge::api::Hint {
            repository: forge_world::REPO,
            change: skein_forge::api::Change::Branch(b"main".as_slice().into()),
            key: None,
        },
    })));
    await_new_forge_news(&mut world, goal.task, before);
    assert_eq!(live_task(&world, goal.task).holdings, held, "news retains g and r");
    finish_subscription_batch(&mut world, goal);
    world.close();
}

fn finish_subscription_batch(world: &mut World, goal: &engine::Assignment) {
    world.until(Until::SecondAssignment);
    let producer = world.assigned.iter().find(|run| run.task != goal.task).expect("batch producer").clone();
    let tasks::Party::Task(change) = live_task(world, producer.task).requester else { panic!("change producer") };
    let Some(Record::Forge { row }) = world.store.rows.get(&Key::Forge(forge_top::Key::Change(change))) else {
        panic!("change")
    };
    let forge_top::Stored::Change(row) = row.as_ref() else { panic!("change row") };
    let branch = row.branch.clone();
    change_tool::land_change(world, &producer, &branch);
    assert_eq!(world.merges, 1, "first batch lands before teardown");
    let current = world.assigned.iter().rev().find(|run| run.task == goal.task).expect("current goal").clone();
    let event = world.protocol.answer(
        Token::new(7),
        current.task,
        current.attempt,
        0,
        tasks::End::Finished {
            result: tasks::TaskResult::Report { words: b"Done".as_slice().into() },
            cancel_delegates: false,
        },
        engine::Parts::none(),
    );
    world.send(event);
    world.until(Until::Ready);
}

#[test]
fn forge_subscription_admission_names_the_failed_check() {
    for (repository, expected) in [
        ("{\"forge\":0,\"repository\":0}", "not adopted"),
        ("{\"forge\":0,\"repository\":1}", "not adopted"),
        ("{\"forge\":1,\"repository\":0}", "not adopted"),
        ("{\"forge\":1,\"repository\":1}", "not adopted"),
        ("{\"forge\":2,\"repository\":1}", "not adopted"),
    ] {
        let (mut world, goal, _) = subscription_goal();
        let input = format!(r#"{{"topic":{{"kind":"landings","repository":{repository},"branch":"main"}}}}"#);
        let answer = subscription_json(&mut world, &goal, b"subscribe_forge", 94, input.as_bytes());
        subscription_feedback(&answer, expected);
        subscription_feedback(&answer, "landings branch main repository {forge:");
        world.close();
    }
}

#[test]
fn forge_subscription_rooms_name_their_limits() {
    for (bound, tool, expected) in [
        ("forge", b"subscribe_forge".as_slice(), "subscriptions (limit 1)"),
        ("resources", b"subscribe_forge", "resources_per_task (limit 1)"),
        ("task", b"subscribe_forge", "tasks.subscriptions (limit 1)"),
        ("timer", b"subscribe", "it holds its most subscriptions (at most 1"),
        ("delegate", b"subscribe", "it holds its most subscriptions (at most 1"),
        ("paths", b"subscribe_forge", "paths_per_subscription (limit 0)"),
        ("path_bytes", b"subscribe_forge", "path exceeds name_bytes"),
        ("branch_bytes", b"subscribe_forge", "branch exceeds name_bytes"),
        ("branch", b"subscribe_forge", "malformed branch"),
    ] {
        let task_room = bound == "timer" || bound == "delegate";
        let (mut world, goal, _) = subscription_goal_bound(if task_room { "task" } else { bound });
        let delegate = if bound == "delegate" {
            let mut delegate =
                temper_engine_domain_world::fixtures::report_delegate(b"Wait for the goal", Box::new([]));
            delegate.authority = live_task(&world, goal.task).authority.clone();
            delegate.authority.budget.spend = 10;
            delegate.authority.delegation = tasks::Delegation { kinds: Box::new([]), tasks: 0, depth: 0 };
            let answer = world.scripted_call(&goal, 90, engine::Tool::Delegate { batch: Box::new([delegate]) });
            let engine::CallAnswer::Delegated(made) = answer else { panic!("delegate admitted: {answer:?}") };
            Some(made[0].task)
        } else {
            None
        };
        if bound == "forge" || bound == "task" || task_room {
            let first = subscription_json(
                &mut world,
                &goal,
                b"subscribe_forge",
                93,
                br#"{"topic":{"kind":"landings","branch":"main"}}"#,
            );
            assert!(matches!(first, engine::CallAnswer::Subscribed { .. }), "room at the bound: {first:?}");
        }
        let long = "x".repeat(usize::try_from(world.env.limits.forge.name_bytes).expect("name bound") + 1);
        let branch = match bound {
            "branch_bytes" => long.as_str(),
            "branch" => "bad//branch",
            _ => "next",
        };
        let paths = match bound {
            "path_bytes" => format!("[\"{long}\"]"),
            "paths" => "[\"file\"]".into(),
            _ => "[]".into(),
        };
        let input = if bound == "timer" {
            format!(r#"{{"kind":"timer","at":{}}}"#, world.env.wall.as_nanos() + 1_000_000_000)
        } else if let Some(target) = delegate {
            assert_eq!(live_task(&world, target).requester, tasks::Party::Task(goal.task));
            format!(r#"{{"kind":"task","target":{target}}}"#)
        } else {
            format!(r#"{{"topic":{{"kind":"landings","branch":"{branch}"}},"paths":{paths}}}"#)
        };
        let answer = subscription_json(&mut world, &goal, tool, 94, input.as_bytes());
        subscription_feedback(&answer, expected);
        world.close();
    }
}

#[test]
fn forge_subscription_selection_and_duplicate_feedback_are_actionable() {
    for repository in ["\"org/repo\"", "\"\"", "{\"forge\":1,\"repository\":2}"] {
        let (mut world, goal, _) = subscription_goal();
        let input = format!(r#"{{"topic":{{"kind":"landings","repository":{repository},"branch":"main"}}}}"#);
        let first = subscription_json(&mut world, &goal, b"subscribe_forge", 94, input.as_bytes());
        assert!(matches!(first, engine::CallAnswer::Subscribed { .. }), "selected repository: {first:?}");
        let replay = subscription_json(&mut world, &goal, b"subscribe_forge", 94, input.as_bytes());
        assert_eq!(replay, first, "the named call replays its subscription number");
        let duplicate = subscription_json(&mut world, &goal, b"subscribe_forge", 95, input.as_bytes());
        subscription_feedback(&duplicate, "duplicate topic");
        subscription_feedback(&duplicate, "landings branch main repository {forge: 1, repository: 2}");
        world.close();
    }
    subscription_repository_feedback();
    for (repository, expected) in [
        (r#"{"forge":1}"#, temper_engine_smith::Problem::Missing(b"repository")),
        (r#"{"owner":"org","name":"repo"}"#, temper_engine_smith::Problem::Missing(b"forge")),
        ("[]", temper_engine_smith::Problem::Type(b"repository")),
        ("1", temper_engine_smith::Problem::Type(b"repository")),
    ] {
        let json = format!(r#"{{"topic":{{"kind":"landings","repository":{repository},"branch":"main"}}}}"#);
        let input = smith_domain_run::HostInput::attested(json.into_bytes().into()).expect("bounded JSON");
        let Err(problem) = temper_engine_smith::call(
            smith_domain_run::CallName { activation: 1, completion: 1, position: 1 },
            b"subscribe_forge",
            &input,
        ) else {
            panic!("malformed repository must fail decoding")
        };
        assert_eq!(problem, expected);
    }
}

fn subscription_repository_feedback() {
    let (mut world, goal) = change_tool::chat_world_with_words(false, b"Watch main");
    let answer = subscription_json(
        &mut world,
        &goal,
        b"subscribe_forge",
        94,
        br#"{"topic":{"kind":"landings","branch":"main"}}"#,
    );
    subscription_feedback(&answer, "this project has no repository to read");
    world.adopt();
    let replay = subscription_json(
        &mut world,
        &goal,
        b"subscribe_forge",
        94,
        br#"{"topic":{"kind":"landings","branch":"main"}}"#,
    );
    assert_eq!(replay, answer, "a refused named subscription stays refused after adoption");
    world.until(Until::Ready);
    assert!(
        !world.store.rows.keys().any(|key| matches!(key,
        Key::Forge(forge_top::Key::Subscription { task, .. }) if *task == goal.task)),
        "replaying a refused subscription must not install an interest after adoption"
    );
    let corrected = subscription_json(
        &mut world,
        &goal,
        b"subscribe_forge",
        95,
        br#"{"topic":{"kind":"landings","branch":"main"}}"#,
    );
    assert!(
        matches!(corrected, engine::CallAnswer::Subscribed { .. }),
        "a new call can use the adoption: {corrected:?}"
    );
    world.close();
    let (mut world, goal) = change_tool::several_repositories();
    let answer = subscription_json(
        &mut world,
        &goal,
        b"subscribe_forge",
        94,
        br#"{"topic":{"kind":"landings","branch":"main"}}"#,
    );
    subscription_feedback(&answer, "name the repository: org/repo, org/extra");
    let answer = subscription_json(
        &mut world,
        &goal,
        b"subscribe_forge",
        95,
        br#"{"topic":{"kind":"landings","repository":"org/missing","branch":"main"}}"#,
    );
    subscription_feedback(&answer, "no repository org/missing here; this project has:");
    world.close();
}

fn subscription_feedback(answer: &engine::CallAnswer, expected: &str) {
    let rendered = temper_engine_smith::answer(answer);
    let words = std::str::from_utf8(rendered.text()).expect("UTF-8 feedback");
    assert!(rendered.error(), "subscription must be refused for {expected}: {words}");
    assert!(words.contains(expected), "expected {expected}: {words}");
    assert_ne!(words, "Refused: task 1 subscription");
}

fn subscription_goal() -> (World, engine::Assignment, Box<[u8]>) {
    subscription_goal_bound("")
}

fn subscription_goal_bound(bound: &str) -> (World, engine::Assignment, Box<[u8]>) {
    let mut world = World::configured_limits(
        true,
        true,
        true,
        1000,
        Gate::default(),
        None,
        StoryLimits { tools: Some(1967), ..StoryLimits::default() },
    );
    match bound {
        "forge" => world.env.limits.forge.subscriptions = 1,
        "resources" => world.env.limits.forge.resources_per_task = 1,
        "task" => world.env.limits.tasks.subscriptions = 1,
        "paths" => world.env.limits.forge.paths_per_subscription = 0,
        _ => {}
    }
    let client = world.env.limits.forge.client;
    world = world.with_forge_client(client);
    world.adopt();
    world.send(protocol::hello_slots(Token::new(7), 2, Box::new([])));
    let people::Ask::Create { project, mut batch } = goal_branch::goal_request(&world, true) else { panic!("goal") };
    batch[0].authority.tools = 1967;
    let goal = assigned_chat(&mut world, 92, people::Ask::Create { project, batch });
    assert_eq!(live_task(&world, goal.task).authority.tools.0, 1967);
    let branch = format!("temper/{}/g{}", goal.task, goal.task).into_bytes().into();
    world.until(Until::Ready);
    (world, goal, branch)
}

fn subscription_json(
    world: &mut World,
    goal: &engine::Assignment,
    tool: &[u8],
    number: u32,
    json: &[u8],
) -> engine::CallAnswer {
    let input = smith_domain_run::HostInput::attested(json.into()).expect("bounded subscription JSON");
    let temper_engine_smith::Decoded::Call(call) = temper_engine_smith::call(
        smith_domain_run::CallName { activation: goal.attempt, completion: number, position: 1 },
        tool,
        &input,
    )
    .expect("subscription repository is selectable") else {
        panic!("decoded subscription")
    };
    let before = world.answers.len();
    let event = world.protocol.call(Token::new(7), goal.task, goal.attempt, u64::from(number), call);
    world.send(event);
    for _ in 0..200 {
        world.tick();
        if let Some(answer) = world.answers.get(before) {
            return answer.clone();
        }
    }
    panic!("subscription unanswered: {:?}", world.call_errors);
}

#[test]
fn a_goal_subscription_receives_the_connectors_landing_news() {
    let mut world = World::new();
    world.adopt();
    world.send(protocol::hello_slots(Token::new(7), 1, Box::new([])));
    world.send(engine::Event::Party(engine::PartyInput::Ask {
        models: Box::new([]),
        reply_to: ReplyTo::new(Token::new(92)),
        sign_in: world.signed_in.expect("owner session"),
        key: [92; 16],
        ask: world.chat(1, &b"watch the landing"[..]),
    }));
    world.until(Until::Assigned);
    let assignment = world.assigned[0].clone();
    let event = world.protocol.call(
        Token::new(7),
        assignment.task,
        assignment.attempt,
        93,
        engine::Call {
            completion: 1,
            position: 1,
            tool: engine::Tool::SubscribeForge {
                topic: forge_top::Topic::Landings { repository: forge_world::REPO, branch: Box::from(&b"main"[..]) },
                own_change: None,
                paths: Box::new([Box::from(&b"file"[..])]),
            },
        },
    );
    world.send(event);
    world.until(Until::Subscribed);
    for _ in 0..10 {
        world.tick();
    }
    let result = fake::advance(&mut world.fake, &world.fake_env, b"org/repo", b"main", b"file", b"changed", 1);
    assert!(result.is_ok(), "external landing in the fake forge");
    world.send(engine::Event::Forge(Box::new(temper_engine_domain_forge::Event::Hint {
        hint: skein_forge::api::Hint {
            repository: forge_world::REPO,
            change: skein_forge::api::Change::Branch(Box::from(&b"main"[..])),
            key: None,
        },
    })));
    world.until(Until::News);
    let Some(Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Live(row)))) =
        world.store.rows.get(&Key::Core(jig_domain_core::Key::Tasks(tasks::Key::Live(assignment.task))))
    else {
        panic!("subscriber task remains live")
    };
    assert!(
        row.inbox
            .iter()
            .any(|word| matches!(word.kind, tasks::MessageKind::News { class: tasks::NewsClass::Wakes, .. }))
    );
    world.close();
}

fn subscribe_chat(world: &mut World, topic: forge_top::Topic, key: u8) -> u64 {
    world.send(protocol::hello_slots(Token::new(7), 1, Box::new([])));
    world.send(engine::Event::Party(engine::PartyInput::Ask {
        models: Box::new([]),
        reply_to: ReplyTo::new(Token::new(u64::from(key))),
        sign_in: world.signed_in.expect("owner session"),
        key: [key; 16],
        ask: world.chat(1, &b"watch the forge"[..]),
    }));
    world.until(Until::Assigned);
    let assignment = world.assigned[0].clone();
    let event = world.protocol.call(
        Token::new(7),
        assignment.task,
        assignment.attempt,
        u64::from(key) + 1,
        engine::Call {
            completion: 1,
            position: 1,
            tool: engine::Tool::SubscribeForge { topic, own_change: None, paths: Box::new([]) },
        },
    );
    world.send(event);
    world.until(Until::Subscribed);
    assignment.task
}

fn news_count(world: &World, task: u64) -> usize {
    match world.store.rows.get(&Key::Core(jig_domain_core::Key::Tasks(tasks::Key::Live(task)))) {
        Some(Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Live(row)))) => {
            let mut hits = 0;
            for word in &row.inbox {
                if matches!(word.kind, tasks::MessageKind::News { .. }) {
                    hits += usize::try_from(word.hits).expect("bounded news hits");
                }
            }
            hits
        }
        Some(_) | None => 0,
    }
}

fn await_new_forge_news(world: &mut World, task: u64, before: usize) {
    for _ in 0..120 {
        world.tick();
        if news_count(world, task) > before {
            return;
        }
    }
    panic!("connector news did not reach the subscribed root task");
}

#[test]
fn a_pull_subscription_receives_the_connectors_state_news() {
    let mut world = World::new();
    world.adopt();
    assert!(matches!(
        world.external(2, raw::Op::Git(raw::Git::Create { branch: Box::from(&b"work"[..]), commit: 1 })),
        raw::Answer::Branch(_)
    ));
    fake::advance(&mut world.fake, &world.fake_env, b"org/repo", b"work", b"file", b"change", 1).expect("head");
    let raw::Answer::Created(pull) = world.external(
        2,
        raw::Op::Write(raw::Write::OpenPull {
            opens_empty: false,
            title: Box::from(&b"change"[..]),
            body: Box::new([]),
            head: Box::from(&b"work"[..]),
            base: Box::from(&b"main"[..]),
        }),
    ) else {
        panic!("pull opened")
    };
    let task = subscribe_chat(&mut world, forge_top::Topic::Pull { repository: forge_world::REPO, number: pull }, 94);
    for _ in 0..10 {
        world.tick();
    }
    let before = news_count(&world, task);
    assert!(matches!(world.external(2, raw::Op::Write(raw::Write::Close { number: pull })), raw::Answer::Done));
    world.send(engine::Event::Forge(Box::new(temper_engine_domain_forge::Event::Hint {
        hint: skein_forge::api::Hint {
            repository: forge_world::REPO,
            change: skein_forge::api::Change::Item(pull),
            key: None,
        },
    })));
    await_new_forge_news(&mut world, task, before);
    world.close();
}

fn ci_subscription_story(restart: bool) {
    let mut world = World::new();
    world.adopt();
    assert!(matches!(
        world.external(2, raw::Op::Git(raw::Git::Create { branch: Box::from(&b"work"[..]), commit: 1 })),
        raw::Answer::Branch(_)
    ));
    let head =
        fake::advance(&mut world.fake, &world.fake_env, b"org/repo", b"work", b"file", b"change", 1).expect("head");
    let raw::Answer::Created(_pull) = world.external(
        2,
        raw::Op::Write(raw::Write::OpenPull {
            opens_empty: false,
            title: Box::from(&b"change"[..]),
            body: Box::new([]),
            head: Box::from(&b"work"[..]),
            base: Box::from(&b"main"[..]),
        }),
    ) else {
        panic!("pull opened")
    };
    let task = subscribe_chat(
        &mut world,
        forge_top::Topic::Ci { repository: forge_world::REPO, head: translate::commit(head) },
        96,
    );
    for _ in 0..10 {
        world.tick();
    }
    if restart {
        world.restart(false, false);
        for _ in 0..20 {
            world.tick();
        }
    }
    let before = news_count(&world, task);
    assert!(matches!(
        world.external(
            1,
            raw::Op::Write(raw::Write::Status {
                commit: head,
                context: Box::from(&b"build"[..]),
                state: raw::Check::Failed,
                description: Box::new([]),
                link: Box::new([])
            })
        ),
        raw::Answer::Done
    ));
    world.send(engine::Event::Forge(Box::new(temper_engine_domain_forge::Event::Hint {
        hint: skein_forge::api::Hint {
            repository: forge_world::REPO,
            change: skein_forge::api::Change::Commit(translate::commit(head)),
            key: None,
        },
    })));
    await_new_forge_news(&mut world, task, before);
    world.close();
}

#[test]
fn a_ci_subscription_receives_the_connectors_verdict_news() {
    ci_subscription_story(false);
}

#[test]
fn a_durable_ci_subscription_receives_verdict_news_after_a_cold_restart() {
    ci_subscription_story(true);
}

#[test]
fn a_participation_subscription_receives_the_connectors_comment_news() {
    let mut world = World::new();
    world.adopt();
    assert!(matches!(
        world.external(2, raw::Op::Git(raw::Git::Create { branch: Box::from(&b"work"[..]), commit: 1 })),
        raw::Answer::Branch(_)
    ));
    fake::advance(&mut world.fake, &world.fake_env, b"org/repo", b"work", b"file", b"change", 1).expect("head");
    let raw::Answer::Created(pull) = world.external(
        2,
        raw::Op::Write(raw::Write::OpenPull {
            opens_empty: false,
            title: Box::from(&b"change"[..]),
            body: Box::new([]),
            head: Box::from(&b"work"[..]),
            base: Box::from(&b"main"[..]),
        }),
    ) else {
        panic!("pull opened")
    };
    let task =
        subscribe_chat(&mut world, forge_top::Topic::Participation { repository: forge_world::REPO, number: pull }, 98);
    for _ in 0..10 {
        world.tick();
    }
    let before = news_count(&world, task);
    assert!(matches!(
        world.external(2, raw::Op::Write(raw::Write::Comment { number: pull, body: Box::from(&b"please check"[..]) })),
        raw::Answer::Commented(_)
    ));
    world.send(engine::Event::Forge(Box::new(temper_engine_domain_forge::Event::Hint {
        hint: skein_forge::api::Hint {
            repository: forge_world::REPO,
            change: skein_forge::api::Change::Item(pull),
            key: None,
        },
    })));
    await_new_forge_news(&mut world, task, before);
    world.close();
}

#[test]
fn an_authorized_forge_read_returns_a_bounded_typed_answer() {
    let mut world = World::configured(true, false);
    world.adopt();
    world.send(protocol::hello_slots(Token::new(7), 1, Box::new([])));
    world.send(engine::Event::Party(engine::PartyInput::Ask {
        models: Box::new([]),
        reply_to: ReplyTo::new(Token::new(92)),
        sign_in: world.signed_in.expect("owner session"),
        key: [92; 16],
        ask: world.chat(1, &b"read forge"[..]),
    }));
    world.until(Until::Assigned);
    let assignment = world.assigned[0].clone();
    let event = world.protocol.call(
        Token::new(7),
        assignment.task,
        assignment.attempt,
        93,
        engine::Call {
            completion: 1,
            position: 1,
            tool: engine::Tool::ReadForge {
                repository: forge_world::REPO,
                read: skein_forge::api::Read::Branches { prefix: Box::from(&b"temper/"[..]) },
            },
        },
    );
    world.send(event);
    decoded(&mut world);
    assert!(world.store.pending.is_empty(), "an agent read crosses the door without a commit");
    world.until(Until::Read);
    assert!(
        matches!(world.answers.as_slice(), [temper_engine_domain::CallAnswer::ForgeRead(result)] if result.is_ok())
    );
    world.answers.clear();
    let file_head = fake::advance(&mut world.fake, &world.fake_env, b"org/repo", b"main", b"file", b"contents", 1)
        .expect("file head");
    let event = world.protocol.call(
        Token::new(7),
        assignment.task,
        assignment.attempt,
        94,
        engine::Call {
            completion: 2,
            position: 1,
            tool: engine::Tool::ReadForge {
                repository: forge_world::REPO,
                read: skein_forge::api::Read::File {
                    head: translate::commit(file_head),
                    path: Box::from(&b"file"[..]),
                    max_bytes: 2,
                },
            },
        },
    );
    world.send(event);
    decoded(&mut world);
    assert!(world.store.pending.is_empty(), "an agent read crosses the door without a commit");
    world.until(Until::Read);
    assert!(matches!(world.answers.as_slice(), [temper_engine_domain::CallAnswer::ForgeRead(result)]
        if matches!(result.as_ref(), Ok(skein_forge::api::Answer::File { head, bytes, truncated: true, .. })
            if *head == translate::commit(file_head) && bytes.len() == 2)));
    for completion in [1, 2] {
        let key = temper_engine_domain::CallKey {
            task: assignment.task,
            attempt: assignment.attempt,
            completion,
            position: 1,
        };
        assert!(
            !world.store.rows.contains_key(&Key::Core(jig_domain_core::Key::Core(jig_domain_core::CoreKey::Call(key)))),
            "read answers are never durable call records"
        );
        let named = forge_top::NamedCall {
            task: key.task,
            attempt: key.attempt,
            completion: key.completion,
            position: key.position,
        };
        assert!(
            !world.store.rows.contains_key(&Key::Forge(forge_top::Key::Call(named))),
            "read answers have no durable connector Call either"
        );
    }
    world.close();
}

#[test]
fn a_named_forge_effect_is_committed_before_its_write_and_made_once() {
    let mut world = World::configured(true, false);
    world.adopt();
    world.send(protocol::hello_slots(Token::new(7), 1, Box::new([])));
    world.send(engine::Event::Party(engine::PartyInput::Ask {
        models: Box::new([]),
        reply_to: ReplyTo::new(Token::new(92)),
        sign_in: world.signed_in.expect("owner session"),
        key: [92; 16],
        ask: world.chat(1, &b"create an issue"[..]),
    }));
    world.until(Until::Assigned);
    let assignment = world.assigned[0].clone();
    let event = world.protocol.call(
        Token::new(7),
        assignment.task,
        assignment.attempt,
        93,
        engine::Call {
            completion: 1,
            position: 1,
            tool: engine::Tool::EffectForge {
                repository: forge_world::REPO,
                resource: forge_top::What::Repository,
                write: Box::new(skein_forge::api::Write::CreateIssue {
                    key: Box::new([]),
                    title: Box::from(&b"Small finding"[..]),
                    body: Box::from(&b"Fix this"[..]),
                }),
            },
        },
    );
    world.send(event);
    world.until(Until::Effect);
    let entry = match world.answers[0] {
        temper_engine_domain::CallAnswer::ForgeEffect {
            entry,
            outcome: Some(skein_forge_client::Outcome::Made { .. }),
            ..
        } => entry,
        ref other => panic!("unexpected effect answer: {other:?}"),
    };
    for _ in 0..30 {
        world.tick();
    }
    let key = temper_engine_domain::CallKey {
        task: assignment.task,
        attempt: assignment.attempt,
        completion: 1,
        position: 1,
    };
    assert!(
        matches!(world.store.rows.get(&Key::Core(jig_domain_core::Key::Core(jig_domain_core::CoreKey::Call(key)))), Some(Record::Core(jig_domain_core::Record::Core(jig_domain_core::CoreRecord::Call(row)))) if matches!(row.part, jig_domain_core::CallPart::Effect { entry: observed, outcome: Some(jig_domain_core::effects::connector::OutboxOutcome::Made), .. } if observed == entry))
    );
    let event = world.protocol.call(
        Token::new(7),
        assignment.task,
        assignment.attempt,
        94,
        engine::Call {
            completion: 1,
            position: 1,
            tool: engine::Tool::EffectForge {
                repository: forge_world::REPO,
                resource: forge_top::What::Repository,
                write: Box::new(skein_forge::api::Write::CreateIssue {
                    key: Box::new([]),
                    title: Box::from(&b"Small finding"[..]),
                    body: Box::from(&b"Fix this"[..]),
                }),
            },
        },
    );
    world.send(event);
    for _ in 0..10 {
        world.tick();
    }
    assert!(
        matches!(world.answers.last(), Some(temper_engine_domain::CallAnswer::ForgeEffect { entry: observed, outcome: Some(skein_forge_client::Outcome::Made { .. }), .. }) if *observed == entry)
    );
    assert_named_call_writer_oracle(&world);
    world.close();
}

#[test]
fn an_explicit_forge_effect_proposal_is_retained_until_its_holder_accepts() {
    let mut world = World::configured(true, false);
    world.adopt();
    world.send(protocol::hello_slots(Token::new(7), 1, Box::new([])));
    world.send(engine::Event::Party(engine::PartyInput::Ask {
        models: Box::new([]),
        reply_to: ReplyTo::new(Token::new(92)),
        sign_in: world.signed_in.expect("owner session"),
        key: [92; 16],
        ask: world.chat(1, &b"propose an issue"[..]),
    }));
    world.until(Until::Assigned);
    let assignment = world.assigned[0].clone();
    let event = world.protocol.call(
        Token::new(7),
        assignment.task,
        assignment.attempt,
        93,
        engine::Call {
            completion: 1,
            position: 1,
            tool: engine::Tool::Propose {
                action: engine::ProposedAction::Effect {
                    repository: forge_world::REPO,
                    resource: forge_top::What::Repository,
                    write: Box::new(skein_forge::api::Write::CreateIssue {
                        key: Box::new([]),
                        title: b"Proposed finding".as_slice().into(),
                        body: b"Fix this".as_slice().into(),
                    }),
                },
                reason: b"please approve".as_slice().into(),
                as_holder: false,
            },
        },
    );
    world.send(event);
    world.until(Until::Ready);
    let proposal = match world.answers.last().expect("proposal answer") {
        temper_engine_domain::CallAnswer::Proposed { proposal, .. } => *proposal,
        other => panic!("unexpected proposal answer: {other:?}"),
    };
    assert!(world.store.rows.values().any(|record| matches!(record, Record::Forge { row, .. } if matches!(row.as_ref(), forge_top::Stored::ProposedEffect(effect) if effect.number == proposal))));
    assert_eq!(issue_count(&mut world), 0);
    world.send(engine::Event::Party(engine::PartyInput::Ask {
        models: Box::new([]),
        reply_to: ReplyTo::new(Token::new(94)),
        sign_in: world.signed_in.expect("owner session"),
        key: [94; 16],
        ask: people::Ask::DecideProposal {
            project: 1,
            proposer: people::Proposer::Task(assignment.task),
            proposal,
            decision: people::ProposalDecision::Accept,
            running: None,
        },
    }));
    world.until(Until::Ready);
    assert!(!world.store.rows.values().any(|record| matches!(record, Record::Forge { row, .. } if matches!(row.as_ref(), forge_top::Stored::ProposedEffect(effect) if effect.number == proposal))));
    for _ in 0..30 {
        world.tick();
    }
    assert_eq!(issue_count(&mut world), 1);
    world.send(engine::Event::Party(engine::PartyInput::Ask {
        models: Box::new([]),
        reply_to: ReplyTo::new(Token::new(95)),
        sign_in: world.signed_in.expect("owner session"),
        key: [94; 16],
        ask: people::Ask::DecideProposal {
            project: 1,
            proposer: people::Proposer::Task(assignment.task),
            proposal,
            decision: people::ProposalDecision::Accept,
            running: None,
        },
    }));
    world.until(Until::Ready);
    for _ in 0..30 {
        world.tick();
    }
    assert_eq!(issue_count(&mut world), 1);
    world.close();
}

fn issue_effect_call(world: &mut World, assignment: &engine::Assignment) -> engine::Event {
    world.protocol.call(
        Token::new(7),
        assignment.task,
        assignment.attempt,
        93,
        engine::Call {
            completion: 1,
            position: 1,
            tool: engine::Tool::EffectForge {
                repository: forge_world::REPO,
                resource: forge_top::What::Repository,
                write: Box::new(skein_forge::api::Write::CreateIssue {
                    key: Box::new([]),
                    title: Box::from(&b"Small finding"[..]),
                    body: Box::from(&b"Fix this"[..]),
                }),
            },
        },
    )
}

fn pending_issue_effect() -> (World, engine::Assignment) {
    let mut world = World::configured(true, false);
    world.adopt();
    world.send(protocol::hello_slots(Token::new(7), 1, Box::new([])));
    world.send(engine::Event::Party(engine::PartyInput::Ask {
        models: Box::new([]),
        reply_to: ReplyTo::new(Token::new(92)),
        sign_in: world.signed_in.expect("owner session"),
        key: [92; 16],
        ask: world.chat(1, &b"create an issue"[..]),
    }));
    world.until(Until::Assigned);
    let assignment = world.assigned[0].clone();
    let event = issue_effect_call(&mut world, &assignment);
    world.send(event);
    decoded(&mut world);
    assert!(!world.store.pending.is_empty(), "effect decision awaits durability");
    (world, assignment)
}

/// Step the protocol layer's decode of the call just sent, before any other input.
fn decoded(world: &mut World) {
    temper_engine_domain_world::poll(&mut world.root, &world.env, &mut world.out);
    world.collect();
    assert!(
        world
            .immediate
            .iter()
            .any(|event| matches!(event, engine::Event::Worker(engine::WorkerInput::DecodedHostCall { .. }))),
        "the root asked to decode the call"
    );
    world.flush();
}

fn issue_count(world: &mut World) -> usize {
    let raw::Answer::Items { items, more: false, .. } = world.external(
        1,
        raw::Op::Read(raw::Read::Items {
            order: raw::Order::LeastUpdated,
            state: None,
            kind: Some(raw::Kind::Issue),
            labels: Box::new([]),
            author: None,
            since: Time::ZERO,
            page: 1,
            limit: 10,
        }),
    ) else {
        panic!("one bounded issue page")
    };
    items.len()
}

// From the stop on, the engine takes nothing that would make a commit, and a
// committed forge effect stays with the connector (jig's `domain/engine/restart.md`, 1).
#[test]
fn a_stop_with_a_committed_forge_effect_makes_no_commit_and_no_write() {
    let (mut world, assignment) = pending_issue_effect();
    let calls = world.forge_calls;
    world.send(engine::Event::Stop);
    let number = world.store.apply();
    world.send(engine::Event::Store(engine::StoreInput::Committed { number, fill: 0 }));
    let commits = world.store.header().commits;
    for _ in 0..30 {
        world.tick();
    }
    assert_eq!(world.store.header().commits, commits);
    assert!(world.store.pending.is_empty());
    assert_eq!(world.forge_calls, calls, "no forge call after the stop");
    assert!(world.events.is_empty() && world.immediate.is_empty(), "the drain leaves no continuation to retry");
    assert_eq!(issue_count(&mut world), 0);
    world.restart(true, false);
    for _ in 0..40 {
        world.tick();
    }
    let event = issue_effect_call(&mut world, &assignment);
    world.send(event);
    for _ in 0..40 {
        world.tick();
    }
    assert_eq!(issue_count(&mut world), 1, "the retried call settles its durable outbox entry once");
}

fn goal_branch_survives_cut(cut: u8) {
    let mut world = goal_branch::adopted_world();
    let request = goal_branch::goal_request(&world, true);
    world.send(engine::Event::Party(engine::PartyInput::Ask {
        models: Box::new([]),
        reply_to: ReplyTo::new(Token::new(92)),
        sign_in: world.signed_in.expect("owner"),
        key: [92; 16],
        ask: request,
    }));
    let mut making = None;
    for _ in 0..1000 {
        // Handle the tip answer outside tick: tick also commits its first
        // decision, which would skip the before-entry cut.
        if let Some(index) = world.events.iter().position(|event| {
            matches!(event,
                engine::Event::Forge(event) if matches!(event.as_ref(), forge_top::Event::Client(
                    skein_forge_client::Event::Answered { result: Ok(skein_forge::api::Answer::Commit(_)), .. }
                ))
            )
        }) {
            let event = world.events.remove(index).expect("queued tip answer");
            world.send(event);
        }
        if let Some(found) = world.store.pending.front().into_iter().flat_map(|(_, writes)| writes.iter()).find_map(
            |write| match write {
                engine::Write::Save(Record::Forge { row }) => match row.as_ref() {
                    forge_top::Stored::Entry { entry, .. } => match &entry.effect.write {
                        skein_forge::api::Write::CreateBranch { branch, .. } if entry.attempt.is_none() => {
                            Some((entry.task, branch.clone()))
                        }
                        _ => None,
                    },
                    _ => None,
                },
                _ => None,
            },
        ) {
            making = Some(found);
            break;
        }
        world.tick();
    }
    let (goal, branch) = making.expect("making awaits its first commit");
    if cut == 0 {
        world.store.pending.clear();
    } else {
        let number = world.store.apply();
        if cut >= 2 {
            world.send(engine::Event::Store(engine::StoreInput::Committed { number, fill: 0 }));
            for _ in 0..1000 {
                if world.events.iter().any(goal_branch_answer) {
                    break;
                }
                world.tick();
            }
            assert_eq!(goal_branch_creations(&world, &branch), 1, "external making before cut {cut}");
            if cut == 3 {
                let index = world.events.iter().position(goal_branch_answer).expect("queued making answer");
                let event = world.events.remove(index).expect("queued making answer");
                world.send(event);
                assert!(!world.store.pending.is_empty());
                world.store.pending.clear();
            }
        }
    }
    world.restart(true, true);
    world.send(protocol::hello_slots(Token::new(7), 2, Box::new([])));
    for _ in 0..200 {
        world.tick();
    }
    assert_eq!(
        goal_branch_creations(&world, &branch),
        1,
        "cut {cut} makes the goal branch once: {:?}",
        live_task(&world, goal).phase
    );
    assert_eq!(goal_branch::branch_tip(&mut world, &branch), goal_branch::branch_tip(&mut world, b"main"));
    assert!(
        !matches!(live_task(&world, goal).phase, tasks::Phase::Held { .. }),
        "cut {cut}: {:?}",
        live_task(&world, goal).phase
    );
    world.close();
}
fn goal_branch_answer(event: &engine::Event) -> bool {
    matches!(event, engine::Event::Forge(event) if matches!(event.as_ref(),
        forge_top::Event::Client(skein_forge_client::Event::Answered { result: Ok(skein_forge::api::Answer::Branch(_)), .. })
    ))
}
fn goal_branch_creations(world: &World, expected: &[u8]) -> usize {
    world.writes.iter().filter(|write| matches!(write, skein_forge::api::Write::CreateBranch { branch, .. } if branch.as_ref() == expected)).count()
}

#[test]
fn effect_survives_each_commit_and_outbox_cut_without_a_second_issue() {
    for cut in 0..4 {
        goal_branch_survives_cut(cut);
        let (mut world, assignment) = pending_issue_effect();
        if cut == 0 {
            // The decision was sent, but no record became durable.
            world.store.pending.clear();
        } else {
            let number = world.store.apply();
            if cut >= 2 {
                world.send(engine::Event::Store(engine::StoreInput::Committed { number, fill: 0 }));
                for _ in 0..60 {
                    if world.events.iter().any(|event| {
                        matches!(
                            event,
                            engine::Event::Forge(event) if matches!(event.as_ref(),
                                temper_engine_domain_forge::Event::Client(
                                    skein_forge_client::Event::Answered {
                                        result: Ok(skein_forge::api::Answer::Created(_)), ..
                                    }
                                ))
                        )
                    }) {
                        break;
                    }
                    world.tick();
                }
                assert_eq!(
                    issue_count(&mut world),
                    1,
                    "the external write completed before the process cut: cut={cut} pending={:?} events={:?} answers={:?}",
                    world.pending,
                    world.events,
                    world.answers
                );
                if cut == 3 {
                    let index = world
                        .events
                        .iter()
                        .position(|event| {
                            matches!(
                                event,
                                engine::Event::Forge(event) if matches!(event.as_ref(),
                                    temper_engine_domain_forge::Event::Client(
                                        skein_forge_client::Event::Answered {
                                            result: Ok(skein_forge::api::Answer::Created(_)), ..
                                        }
                                    ))
                            )
                        })
                        .expect("external outcome queued for the old process");
                    let event = world.events.remove(index).expect("queued outcome");
                    world.send(event);
                    assert!(!world.store.pending.is_empty(), "outcome waits for its commit");
                    world.store.pending.clear();
                }
            }
        }
        world.restart(true, false);
        world.until(Until::Ready);
        let event = issue_effect_call(&mut world, &assignment);
        world.send(event);
        for _ in 0..40 {
            world.tick();
        }
        assert_eq!(issue_count(&mut world), 1, "cut {cut} created one keyed issue");
        let key = temper_engine_domain::CallKey {
            task: assignment.task,
            attempt: assignment.attempt,
            completion: 1,
            position: 1,
        };
        assert!(
            matches!(
                world.store.rows.get(&Key::Core(jig_domain_core::Key::Core(jig_domain_core::CoreKey::Call(key)))),
                Some(Record::Core(jig_domain_core::Record::Core(jig_domain_core::CoreRecord::Call(row))))
                    if matches!(row.part, jig_domain_core::CallPart::Effect {
                        outcome: Some(jig_domain_core::effects::connector::OutboxOutcome::Made), ..
                    })
            ),
            "cut {cut} committed the settled named answer"
        );
        assert_named_call_writer_oracle(&world);
    }
}

#[test]
fn a_tracked_goals_changes_install_their_landing_topic_alone_without_agent_calls() {
    let (mut world, goal, producer, branch) = change_world_with_policy(true, 1000, None, false, false, false, true);
    world.env.limits.tasks.subscriptions = 2;
    let subscriptions = |world: &World| -> Vec<forge_top::Subscriber> {
        world
            .store
            .rows
            .values()
            .filter_map(|stored| match stored {
                Record::Forge { row, .. } => match row.as_ref() {
                    forge_top::Stored::Subscription(subscriber) if subscriber.task == goal.task => {
                        Some(subscriber.clone())
                    }
                    _ => None,
                },
                Record::Core(_) => None,
            })
            .collect()
    };
    assert!(
        subscriptions(&world).iter().any(|subscriber| matches!(&subscriber.topic,
        forge_top::Topic::Landings { branch, .. } if branch.as_ref() == b"main")
            && subscriber.goal_tasks.as_ref().is_some_and(|tasks| !tasks.is_empty())),
        "goal subscribed as its plan names the landing branch"
    );
    world.external(2, raw::Op::Git(raw::Git::Create { branch: branch.clone(), commit: 1 }));
    let pushed = fake::advance(&mut world.fake, &world.fake_env, b"org/repo", &branch, b"file", b"fixed", 1)
        .expect("producer pushed");
    let event = world.protocol.answer(
        Token::new(7),
        producer.task,
        producer.attempt,
        5,
        tasks::End::Finished {
            result: tasks::TaskResult::Change {
                title: jig_domain_tasks::ChangeTitle::new(b"Small fix".as_slice().into()),
                connector: 0,
                kind: 2,
                resource: temper_engine_domain_forge::tag(forge_world::REPO.forge, forge_world::REPO.repository),
                words: Box::from(&b"pushed"[..]),
            },
            cancel_delegates: false,
        },
        engine::Parts {
            saved: None,
            pushed: Box::new([forge_top::Pushed {
                tag: forge_top::tag(forge_world::REPO.forge, forge_world::REPO.repository),
                commit: translate::commit(pushed),
            }]),
            preparation: None,
        },
    );
    world.send(event);
    for _ in 0..180 {
        world.tick();
    }
    assert!(world.store.rows.values().any(|stored| matches!(stored,
        Record::Forge { row, .. } if matches!(row.as_ref(), forge_top::Stored::BranchHead(head) if head.commit == translate::commit(pushed)))));
    let current = subscriptions(&world);
    assert!(
        !current.iter().any(|subscriber| matches!(subscriber.topic, forge_top::Topic::Ci { .. })),
        "the core keeps only landing topics for a goal"
    );
    assert_eq!(live_task(&world, goal.task).subscriptions.len(), 1, "one jig row for the landing branch");
    let before = world.answers.len();
    let event = world.protocol.call(
        Token::new(7),
        goal.task,
        goal.attempt,
        95,
        engine::Call {
            completion: 2,
            position: 1,
            tool: engine::Tool::Subscribe {
                kind: tasks::SubscriptionKind::Timer { at: skein_lib::Wall::from_nanos(u64::MAX), period: None },
            },
        },
    );
    world.send(event);
    for _ in 0..200 {
        world.tick();
        if world.answers.len() > before {
            break;
        }
    }
    let answer = world.answers.get(before).expect("timer subscription answered");
    assert!(matches!(answer, engine::CallAnswer::Subscribed { .. }), "room for the goal's timer: {answer:?}");
    world.restart(true, true);
    for _ in 0..100 {
        world.tick();
    }
    assert_eq!(
        subscriptions(&world)
            .iter()
            .filter(|subscriber| matches!(subscriber.topic, forge_top::Topic::Landings { .. }))
            .count(),
        1,
        "one durable interest for the whole subtree"
    );
    world.close();
}

#[test]
fn a_delegated_goal_is_woken_by_a_landing_over_its_planned_path() {
    for (path, class, wakes) in
        [(b"file".as_slice(), tasks::NewsClass::Wakes, true), (b"other".as_slice(), tasks::NewsClass::Kept, false)]
    {
        let (mut world, goal, producer, _) = change_world_with_policy(true, 1000, None, false, false, false, true);
        let owner = live_task(&world, goal.task);
        assert_ne!(owner.root, goal.task, "the tracked goal is delegated");
        assert!(matches!(owner.requester, tasks::Party::Task(_)));
        let tasks::Party::Task(change) = live_task(&world, producer.task).requester else {
            panic!("producer belongs to its planned change")
        };
        assert_eq!(live_task(&world, change).requester, tasks::Party::Task(goal.task));
        assert!(live_task(&world, change).spec.parameters.iter().any(
            |parameter| matches!(parameter, tasks::Parameter::Bytes { name: 6, value } if value.as_ref() == b"file")
        ));
        assert!(
            world.store.rows.values().all(|stored| !matches!(stored,
            Record::Forge { row, .. } if matches!(row.as_ref(), forge_top::Stored::Change(row)
                if row.task == change && row.pull.is_some()))),
            "change has no produced pull"
        );
        let event =
            world.protocol.answer(Token::new(7), goal.task, goal.attempt, 0, tasks::End::Parked, engine::Parts::none());
        world.send(event);
        for _ in 0..30 {
            world.tick();
        }
        let assigned = world.assigned.len();
        assert!(fake::advance(&mut world.fake, &world.fake_env, b"org/repo", b"main", path, b"changed", 1).is_ok());
        world.send(engine::Event::Forge(Box::new(forge_top::Event::Hint {
            hint: skein_forge::api::Hint {
                repository: forge_world::REPO,
                change: skein_forge::api::Change::Branch(Box::from(&b"main"[..])),
                key: None,
            },
        })));
        for _ in 0..200 {
            world.tick();
        }
        assert!(
            live_task(&world, goal.task)
                .inbox
                .iter()
                .any(|word| matches!(word.kind, tasks::MessageKind::News { class: actual, .. } if actual == class)),
            "landing classification for {path:?}: {:?}",
            live_task(&world, goal.task).inbox
        );
        assert_eq!(
            world.assigned[assigned..].iter().any(|run| run.task == goal.task),
            wakes,
            "only an overlapping landing makes a new goal run due: {path:?}"
        );
        world.close();
    }
}

/// The task's current specification revision, as its durable row shows it.
fn spec_revision(world: &World, task: u64) -> u64 {
    match world.store.rows.get(&Key::Core(jig_domain_core::Key::Tasks(tasks::Key::Live(task)))) {
        Some(Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Live(row)))) => row.revisions.spec,
        _ => panic!("live task {task}"),
    }
}

fn held_issue_write() -> (World, engine::Assignment, u64) {
    let (mut world, assignment) = pending_issue_effect();
    world.hold_write_answers = Some(HoldWrite::All);
    for _ in 0..80 {
        world.tick();
        if !world.held_write_answers.is_empty() {
            break;
        }
    }
    assert_eq!(world.held_write_answers.len(), 1, "one sent write’s answer is held");
    let entry = world
        .store
        .rows
        .values()
        .find_map(|record| match record {
            Record::Forge { row } => match row.as_ref() {
                forge_top::Stored::Entry { entry, .. } if entry.task == assignment.task => Some(entry.number),
                _ => None,
            },
            Record::Core(_) => None,
        })
        .expect("sent entry is durable");
    (world, assignment, entry)
}

fn drain_stop(world: &mut World) {
    for _ in 0..80 {
        world.tick();
        if world.drained.is_some() {
            return;
        }
    }
    panic!("stop drains after the write’s terminal");
}

#[test]
fn a_stop_commits_the_answer_of_a_forge_write_already_sent() {
    let (mut world, assignment, entry) = held_issue_write();
    world.send(engine::Event::Stop);
    for _ in 0..20 {
        world.tick();
    }
    assert!(world.drained.is_none(), "the stop waits for the held answer");
    assert!(world.store.rows.contains_key(&Key::Forge(forge_top::Key::Entry(entry))));
    world.release_write_answers();
    drain_stop(&mut world);
    assert!(
        !world.store.rows.contains_key(&Key::Forge(forge_top::Key::Entry(entry))),
        "outcome committed before Drained"
    );
    assert_eq!(world.unsettled.len(), 0);
    assert!(flights::rows(&world).is_empty(), "a settled stop leaves no flight");
    assert_eq!(issue_count(&mut world), 1);
    world.restart(true, false);
    for _ in 0..40 {
        world.tick();
    }
    let event = issue_effect_call(&mut world, &assignment);
    world.send(event);
    for _ in 0..40 {
        world.tick();
    }
    assert_eq!(issue_count(&mut world), 1);
    assert!(!world.store.rows.contains_key(&Key::Forge(forge_top::Key::Entry(entry))), "nothing to find after restart");
    world.close();
}

#[test]
fn settling_places_no_run_and_answers_a_call_busy() {
    let (mut world, assignment, _entry) = held_issue_write();
    // Another task becomes due, with a slot available, immediately before Stop.
    world.send(protocol::hello_slots(Token::new(7), 2, Box::new([])));
    world.send(engine::Event::Party(engine::PartyInput::Ask {
        models: Box::new([]),
        reply_to: ReplyTo::new(Token::new(992)),
        sign_in: world.signed_in.expect("owner session"),
        key: [99; 16],
        ask: world.chat(1, b"due during settlement"),
    }));
    let assigned = world.assigned.len();
    world.send(engine::Event::Stop);
    let event = world.protocol.call(
        Token::new(7),
        assignment.task,
        assignment.attempt,
        994,
        engine::Call {
            completion: 2,
            position: 1,
            tool: engine::Tool::ReadForge {
                repository: forge_world::REPO,
                read: skein_forge::api::Read::Branches { prefix: Box::from(b"temper/".as_slice()) },
            },
        },
    );
    world.send(event);
    for _ in 0..30 {
        world.tick();
    }
    assert!(world.drained.is_none());
    assert_eq!(world.assigned.len(), assigned, "the due task gets no placement");
    assert_eq!(world.busy_calls, 1, "a run’s call is answered busy");
    world.release_write_answers();
    drain_stop(&mut world);
    world.restart(true, false);
    world.close();
}

#[test]
fn a_write_unanswered_past_its_lifetime_stops_unsettled_naming_it() {
    let (mut world, assignment, entry) = held_issue_write();
    world.send(engine::Event::Stop);
    let until = world.env.now.saturating_add(
        world
            .env
            .limits
            .forge
            .client
            .lifetime
            .saturating_add(world.env.limits.forge.client.clock_margin)
            .saturating_mul(2),
    );
    while world.env.now < until {
        world.tick();
    }
    drain_stop(&mut world);
    assert_eq!(world.unsettled.as_slice(), &[engine::Unsettled { entry, task: assignment.task }]);
    assert_eq!(flights::rows(&world), [(assignment.task, entry)]);
    assert!(world.store.rows.contains_key(&Key::Forge(forge_top::Key::Entry(entry))));
    assert_eq!(issue_count(&mut world), 1);
    world.restart(true, false);
    for _ in 0..80 {
        world.tick();
    }
    let event = issue_effect_call(&mut world, &assignment);
    world.send(event);
    for _ in 0..40 {
        world.tick();
    }
    assert_eq!(issue_count(&mut world), 1, "the next start finds the issue rather than writing twice");
    assert!(flights::rows(&world).is_empty(), "restart settles and erases the flight");
    assert!(!world.store.rows.contains_key(&Key::Forge(forge_top::Key::Entry(entry))));
    world.close();
}

#[test]
fn a_stop_waits_for_a_sent_merge_to_be_proved() {
    let (mut world, _chat, producer, branch) = change_world(false, 1000, None);
    world.external(2, raw::Op::Git(raw::Git::Create { branch: branch.clone(), commit: 1 }));
    fake::advance(&mut world.fake, &world.fake_env, b"org/repo", &branch, b"file", b"fixed", 1)
        .expect("producer pushed");
    world.hold_write_answers = Some(HoldWrite::Merge);
    let event = world.protocol.answer(
        Token::new(7),
        producer.task,
        producer.attempt,
        5,
        tasks::End::Finished {
            result: tasks::TaskResult::Change {
                title: jig_domain_tasks::ChangeTitle::new(b"Change".as_slice().into()),
                connector: 0,
                kind: 2,
                resource: forge_top::tag(forge_world::REPO.forge, forge_world::REPO.repository),
                words: Box::from(b"pushed".as_slice()),
            },
            cancel_delegates: false,
        },
        engine::Parts::none(),
    );
    world.send(event);
    for _ in 0..300 {
        world.tick();
        if !world.held_write_answers.is_empty() {
            break;
        }
    }
    assert_eq!(world.held_write_answers.len(), 1, "merge answer held");
    assert_eq!(world.merges, 1);
    let entry = world
        .store
        .rows
        .values()
        .find_map(|record| match record {
            Record::Forge { row } => match row.as_ref() {
                forge_top::Stored::Entry { entry, .. }
                    if matches!(entry.effect.write, skein_forge::api::Write::Merge { .. }) =>
                {
                    Some(entry.number)
                }
                _ => None,
            },
            Record::Core(_) => None,
        })
        .expect("durable merge attempt");
    world.send(engine::Event::Stop);
    for _ in 0..10 {
        world.tick();
    }
    assert!(world.drained.is_none());
    let calls = world.forge_calls;
    world.release_write_answers();
    // The answer schedules a proving read; even an accepted write has no outcome yet.
    world.tick();
    assert!(world.drained.is_none());
    drain_stop(&mut world);
    assert!(world.forge_calls > calls, "the proving read crosses the paused write fence");
    assert!(
        !world.store.rows.contains_key(&Key::Forge(forge_top::Key::Entry(entry))),
        "the proved outcome is durable before Drained"
    );
    assert_eq!(world.unsettled.len(), 0);
    assert_eq!(world.merges, 1);
    world.restart(true, true);
    world.close();
}

// The oracle reads real root-written rows; identity legality belongs to their writer.
fn call_writer_oracle(world: &World, key: forge_top::NamedCall) -> bool {
    if key.task == 0 || key.attempt == 0 {
        return false;
    }
    let core =
        engine::CallKey { task: key.task, attempt: key.attempt, completion: key.completion, position: key.position };
    matches!(world.store.rows.get(&Key::Core(jig_domain_core::Key::Core(jig_domain_core::CoreKey::Call(core)))), Some(Record::Core(jig_domain_core::Record::Core(jig_domain_core::CoreRecord::Call(row)))) if row.key == core)
}
fn assert_named_call_writer_oracle(world: &World) {
    let mut count = 0_u32;
    for row in world.store.rows.values() {
        if let Record::Forge { row } = row
            && let forge_top::Stored::Call { key, .. } = row.as_ref()
        {
            assert!(call_writer_oracle(world, *key));
            assert!(!call_writer_oracle(world, forge_top::NamedCall { task: 0, ..*key }));
            assert!(!call_writer_oracle(world, forge_top::NamedCall { attempt: 0, ..*key }));
            count += 1;
        }
    }
    assert!(count > 0, "real writer retained connector Calls");
}

#[path = "forge_routes/decided.rs"]
mod decided;

#[path = "forge_routes/live_delegate.rs"]
mod live_delegate;

#[path = "forge_routes/close_change.rs"]
mod close_change;

#[path = "forge_routes/follow_changes.rs"]
pub(crate) mod follow_changes;
