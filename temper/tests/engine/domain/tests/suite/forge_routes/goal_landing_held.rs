//! A goal's landing exhausts its repairs, stays held through a change into g,
//! then checks and lands g's released head (temper/docs/design/domain/forge/change/README.md, 1 and temper/docs/design/domain/forge/change/repairs.md and temper/docs/design/domain/forge/change/README.md, 3
//! ; temper/docs/design/draft/archive/reference-landing.md, 3 and 9).
use super::goal_branch::{branch_tip, goal_request, writer};
use super::goal_landing::{
    assert_goal_holds, ended, finish_run, land_into_goal, mine, producing_into_goal, row, status,
};
use super::*;
use temper_engine_domain_forge_change::{Delegate, Hold, Repair, State};

const STEPS: u32 = 1000; // The forge world's bound for a full landing and release.
const REPAIRS: u32 = 3;
const LANDING_CALL: u64 = 95;

fn world() -> World {
    world_with_brief(None)
}

fn world_with_brief(brief: Option<engine::BriefLimits>) -> World {
    world_with_settings(brief, None)
}

pub(super) fn world_with_bound(bound: Option<Duration>) -> World {
    world_with_settings(None, bound)
}

fn world_with_settings(brief: Option<engine::BriefLimits>, bound: Option<Duration>) -> World {
    let mut world = World::configured_limits(
        true,
        true,
        true,
        STEPS,
        Gate::default(),
        None,
        StoryLimits { brief, ..StoryLimits::default() },
    )
    .with_check_timeout(bound);
    world.env.limits.forge.change_policy.repairs = REPAIRS;
    let segments = [b"forge".as_slice(), b"forge.example", b"org", b"repo", b"branch"].map(Box::from);
    world = world.with_landing_deployment(Box::new([
        engine::LandingRule {
            sets: Box::new([]),
            connector: 0,
            kind: 4,
            enforce: true,
            pattern: authority::Pattern {
                segments: segments.clone().into(),
                last: authority::Last::Exact(b"main".as_slice().into()),
            },
            ci: true,
            up_to_date: true,
            gates: Box::new([]),
            approvals: Box::new([]),
        },
        engine::LandingRule {
            sets: Box::new([]),
            connector: 0,
            kind: 4,
            enforce: true,
            pattern: authority::Pattern {
                segments: segments.into(),
                last: authority::Last::Open(b"temper".as_slice().into()),
            },
            ci: false,
            up_to_date: false,
            gates: Box::new([]),
            approvals: Box::new([]),
        },
    ]));
    world.adopt();
    world.send(protocol::hello_slots(Token::new(7), 2, Box::new([])));
    world
}

fn checking(world: &mut World, landing: u64, head: u64) {
    let mut read_ci = false;
    for _ in 0..STEPS {
        world.tick();
        read_ci |= world.pending.values().any(|op| {
            matches!(op,
            skein_forge::api::Op::Read(skein_forge::api::Read::Checks { commit })
                if *commit == translate::commit(head))
                || matches!(op, skein_forge::api::Op::Read(skein_forge::api::Read::Pull { number, ci: true })
                    if row(world, landing).pull == Some(*number))
        });
        if read_ci && row(world, landing).change.state == (State::Checking { head: translate::commit(head) }) {
            break;
        }
    }
    assert!(read_ci, "CI is read at g's head {head}");
    assert_eq!(row(world, landing).change.state, State::Checking { head: translate::commit(head) });
    assert_eq!(row(world, landing).change.last_head, Some(translate::commit(head)));
}

fn repair(world: &mut World, goal: u64, branch: &[u8], landing: u64, head: u64, number: u32) -> u64 {
    let assigned = world.assigned.len();
    status(world, head, raw::Check::Failed);
    let repair = reviews::next_assignment(world, assigned);
    assert_eq!(live_task(world, repair.task).requester, tasks::Party::Task(landing));
    assert_eq!(row(world, landing).delegate, Some((repair.task, Delegate::Repair(Repair::Ci))));
    assert_eq!(row(world, landing).change.state, State::Repairing { why: Repair::Ci, requested: true });
    assert_eq!(row(world, landing).change.last_head, Some(translate::commit(head)));
    assert_eq!(row(world, landing).change.repairs, number);
    assert_eq!(repair.workspace.repositories.len(), 1);
    assert_eq!(repair.workspace.repositories[0].start, engine::ForgeStart::Branch(branch.into()));
    assert_eq!(repair.workspace.repositories[0].push.as_deref(), Some(branch));
    assert_eq!(writer(world, branch), Some(tasks::Writer::Run { task: repair.task, attempt: repair.attempt }));
    assert_goal_holds(world, goal, branch);
    assert_eq!(branch_tip(world, branch), head, "the repair starts at g's current head");
    let next =
        fake::advance(&mut world.fake, &world.fake_env, b"org/repo", branch, b"repair", &number.to_be_bytes(), 1)
            .expect("L's repair pushes g under G's hold");
    assert_ne!(next, head);
    assert_goal_holds(world, goal, branch);
    assert_eq!(writer(world, branch), Some(tasks::Writer::Run { task: repair.task, attempt: repair.attempt }));
    status(world, next, raw::Check::Pending);
    finish_run(
        world,
        &repair,
        tasks::TaskResult::Change {
            title: tasks::ChangeTitle::new(b"Repair CI".as_slice().into()),
            connector: 0,
            kind: 2,
            resource: forge_top::tag(forge_world::REPO.forge, forge_world::REPO.repository),
            words: b"pushed a repair".as_slice().into(),
        },
    );
    checking(world, landing, next);
    assert_goal_holds(world, goal, branch);
    assert_eq!(writer(world, branch), None, "the repair's answer frees g's slot");
    next
}

fn exhausted(world: &mut World, goal: &engine::Assignment, landing: u64, head: u64) {
    let assigned = world.assigned.len();
    status(world, head, raw::Check::Failed);
    for _ in 0..STEPS {
        world.tick();
        if matches!(live_task(world, landing).phase, tasks::Phase::Held { .. })
            && matches!(live_task(world, landing).escalation, tasks::Escalation::Waiting { .. })
        {
            break;
        }
    }
    world.until(Until::Ready);
    assert_eq!(
        row(world, landing).change.state,
        State::Held { was: Box::new(State::Checking { head: translate::commit(head) }), why: Hold::Repairs }
    );
    assert_eq!(row(world, landing).change.repairs, REPAIRS);
    assert!(matches!(live_task(world, landing).phase, tasks::Phase::Held { why: tasks::Hold::Procedure, .. }));
    assert_eq!(world.assigned.len(), assigned, "the fourth CI failure makes no fourth repair");
    let tasks::Escalation::Waiting { holder, entry, revision, .. } = live_task(world, landing).escalation else {
        panic!("L's hold escalates: {:?}", live_task(world, landing).escalation)
    };
    assert_eq!(holder, tasks::EscalationHolder::Task(goal.task));
    assert_eq!(revision, live_task(world, landing).revisions.hold);
    assert!(
        world.delivered_messages.get(&(goal.task, goal.attempt)).is_some_and(|messages| messages.contains(&entry)),
        "G's running worker receives L's escalation entry"
    );
}

fn change_while_held(world: &mut World, goal: &engine::Assignment, branch: &[u8], landing: u64) -> u64 {
    let held = row(world, landing).change.clone();
    let read = Some(live_task(world, goal.task).last_message);
    world.send(protocol::turn(
        Token::new(7),
        goal.task,
        goal.attempt,
        2,
        0,
        read,
        b"The landing holds for its repairs. Rework the goal before releasing it.",
    ));
    world.until(Until::Ready);
    let (change, producer) = producing_into_goal(world, goal, branch, LANDING_CALL + 2, b"rework");
    assert_eq!(row(world, landing).change, held);
    assert_eq!(row(world, change).base.as_ref(), branch, "G makes a change with landing goal");
    finish_run(
        world,
        &producer,
        tasks::TaskResult::Change {
            title: tasks::ChangeTitle::new(b"Rework the goal".as_slice().into()),
            connector: 0,
            kind: 2,
            resource: forge_top::tag(forge_world::REPO.forge, forge_world::REPO.repository),
            words: b"reworked".as_slice().into(),
        },
    );
    for _ in 0..STEPS {
        world.tick();
        assert_eq!(row(world, landing).change, held, "L decides nothing while held");
        assert!(matches!(live_task(world, landing).phase, tasks::Phase::Held { .. }));
        assert_goal_holds(world, goal.task, branch);
        if ended(world, change).is_some() {
            break;
        }
    }
    assert!(ended(world, change).is_some(), "G's rework ends");
    assert!(matches!(row(world, change).change.state, State::Landed { .. }));
    assert_eq!(world.merges, 2, "G's plan and rework landed into g; L stayed held");
    branch_tip(world, branch)
}

fn release(world: &mut World, goal: &engine::Assignment, landing: u64) {
    let revision = live_task(world, landing).revisions.hold;
    let before = world.answers.len();
    let event = world.protocol.call(
        Token::new(7),
        goal.task,
        goal.attempt,
        LANDING_CALL + 3,
        engine::Call {
            completion: 3,
            position: 1,
            tool: engine::Tool::Release { target: landing, revision, again: None },
        },
    );
    world.send(event);
    for _ in 0..STEPS {
        world.tick();
        if world.answers.len() > before {
            break;
        }
    }
    assert_eq!(
        world.answers.get(before),
        Some(&engine::CallAnswer::Revised { task: landing, revision: live_task(world, landing).revisions.spec }),
        "G releases L through its running worker: {:?}",
        world.call_errors
    );
}

/// temper/docs/design/domain/agent/README.md, 5: a save also pins a goal landing's repair.
#[expect(clippy::too_many_lines, reason = "one story follows the save, question, writer reread and resumed pull")]
fn saving_repair_rebriefs(reported: bool) {
    // The pull's identity, saved content and CI reason share the resumed brief.
    let small = temper_engine_service::limits::profile(temper_engine_service::limits::Profile::Small);
    let mut world = world_with_brief(Some(small.root.brief));
    let people::Ask::Create { project, mut batch } = goal_request(&world, true) else { panic!("goal template") };
    batch[0].authority.tools |= jig_domain_core::ToolFamilies::JIG.control.0;
    let goal = assigned_chat(&mut world, 92, people::Ask::Create { project, batch });
    let branch = format!("temper/{}/g{}", goal.task, goal.task).into_bytes().into_boxed_slice();
    world.until(Until::Ready);
    let head = land_into_goal(&mut world, &goal, &branch, LANDING_CALL + 1, b"plan");
    world.until(Until::Ready);
    let read = live_task(&world, goal.task).inbox.iter().map(|word| word.number).max();
    world.send(protocol::turn(Token::new(7), goal.task, goal.attempt, 1, 0, read, b"The plan landed."));
    world.until(Until::Ready);
    status(&mut world, head, raw::Check::Pending);
    let answer =
        mine(&mut world, &goal, LANDING_CALL, br#"{"words":"Land the goal","branch":"mine","questions":"up"}"#);
    let Some(engine::CallAnswer::Delegated(made)) = answer else { panic!("goal landing admitted: {answer:?}") };
    let landing = made[0].task;
    checking(&mut world, landing, head);
    let assigned = world.assigned.len();
    status(&mut world, head, raw::Check::Failed);
    let repair = reviews::next_assignment(&mut world, assigned);
    let delegate = Some((repair.task, Delegate::Repair(Repair::Ci)));
    let state = State::Repairing { why: Repair::Ci, requested: true };
    assert_eq!(row(&world, landing).delegate, delegate);
    assert_eq!(row(&world, landing).change.state, state);
    assert_eq!(repair.workspace.repositories[0].push.as_deref(), Some(branch.as_ref()));
    assert_eq!(writer(&world, &branch), Some(tasks::Writer::Run { task: repair.task, attempt: repair.attempt }));
    assert_goal_holds(&world, goal.task, &branch);
    let mut tree: Vec<_> = world
        .fake
        .object(head)
        .expect("goal head")
        .tree
        .iter()
        .map(|(path, content)| raw::File { path: path.clone(), content: content.clone() })
        .collect();
    tree.push(raw::File { path: b"repair".as_slice().into(), content: b"saved repair in progress".as_slice().into() });
    let saved = fake::commit(
        &mut world.fake,
        &world.fake_env.limits,
        head,
        tree.into_boxed_slice(),
        b"Unchecked: work in progress, saved before a question",
    )
    .expect("the worker commits its unchecked repair")
    .expect("the repair changed the tree");
    assert_eq!(
        world.external(2, raw::Op::Git(raw::Git::Push { branch: branch.clone(), commit: saved, expected: Some(head) })),
        raw::Answer::Pushed(raw::Pushed::Pushed)
    );
    let question = questions::ask(&mut world, &repair, landing);
    let pull_number = row(&world, landing).pull;
    let writes = world.writes.len();
    let merges = world.merges;
    let parts = if reported {
        engine::Parts {
            pushed: Box::new([forge_top::Pushed {
                tag: forge_top::tag(forge_world::REPO.forge, forge_world::REPO.repository),
                commit: translate::commit(saved),
            }]),
            ..engine::Parts::none()
        }
    } else {
        engine::Parts::none()
    };
    let event = world.protocol.answer(Token::new(7), repair.task, repair.attempt, 0, tasks::End::Parked, parts);
    world.send(event);
    for _ in 0..STEPS {
        world.tick();
        if writer(&world, &branch).is_none() && world.store.pending.is_empty() {
            break;
        }
    }
    assert_eq!(row(&world, landing).change.state, state, "a save completes no repair");
    assert_eq!(row(&world, landing).delegate, delegate);
    assert_eq!(row(&world, landing).pull, pull_number);
    assert_eq!(world.writes.len(), writes, "a save makes no forge effect");
    assert_eq!(world.merges, merges);
    assert_eq!(writer(&world, &branch), None, "the answer or reread frees the writer");
    assert_goal_holds(&world, goal.task, &branch);
    let assigned = world.assigned.len();
    questions::answer(&mut world, &goal, landing, LANDING_CALL + 2);
    assert_eq!(
        row(&world, landing).change.last_head,
        Some(translate::commit(saved)),
        "the accepted save pins the next brief"
    );
    let resumed = reviews::next_assignment(&mut world, assigned);
    assert_eq!(resumed.task, repair.task, "the same repair continues");
    assert!(resumed.attempt > repair.attempt);
    assert!(resumed.messages.iter().any(|word| word.kind == tasks::MessageKind::Answer { question }));
    let pull = resumed
        .sections
        .iter()
        .find_map(|section| {
            if section.kind == engine::BriefKind::Forge(engine::ForgeBriefKind::Pull)
                && let engine::BriefBody::Text(words) = &section.body
            {
                Some(words.as_ref())
            } else {
                None
            }
        })
        .expect("the resumed repair reads its pull section");
    let head = forge_top::commit_text(&translate::commit(saved));
    assert!(contains(pull, &head), "{}", String::from_utf8_lossy(pull));
    assert!(contains(pull, b"saved repair in progress"));
    assert_eq!(row(&world, landing).change.last_head, Some(translate::commit(saved)));
    assert_eq!(live_task(&world, repair.task).tries, tasks::Tries::NONE);
    assert_eq!(row(&world, landing).change.state, state);
    assert_eq!(row(&world, landing).delegate, delegate);
    assert_eq!(row(&world, landing).pull, pull_number);
    assert_eq!(world.writes.len(), writes);
    assert_eq!(world.merges, merges);
    assert_goal_holds(&world, goal.task, &branch);
    world.close();
}

#[test]
fn a_goal_landing_repair_that_reports_its_save_rebriefs_at_the_saved_head() {
    saving_repair_rebriefs(true);
}

#[test]
fn a_goal_landing_repair_whose_save_is_not_reported_rebriefs_at_the_reread_head() {
    saving_repair_rebriefs(false);
}

#[test]
fn a_goals_landing_held_past_its_repairs_lands_after_a_change_into_its_branch() {
    // The reference's main asks CI and up to date, with no gates; g asks neither.
    let mut world = world();
    let people::Ask::Create { project, mut batch } = goal_request(&world, true) else { panic!("goal template") };
    batch[0].authority.tools |= jig_domain_core::ToolFamilies::JIG.control.0;
    let goal = assigned_chat(&mut world, 92, people::Ask::Create { project, batch });
    let branch = format!("temper/{}/g{}", goal.task, goal.task).into_bytes().into_boxed_slice();
    world.until(Until::Ready);
    assert_goal_holds(&world, goal.task, &branch);
    let mut head = land_into_goal(&mut world, &goal, &branch, LANDING_CALL + 1, b"plan");
    world.until(Until::Ready);
    let read = live_task(&world, goal.task).inbox.iter().map(|word| word.number).max();
    world.send(protocol::turn(Token::new(7), goal.task, goal.attempt, 1, 0, read, b"The plan landed."));
    world.until(Until::Ready);
    status(&mut world, head, raw::Check::Pending);
    let answer = mine(&mut world, &goal, LANDING_CALL, br#"{"words":"Land the goal","branch":"mine"}"#);
    let Some(engine::CallAnswer::Delegated(made)) = answer else { panic!("goal landing admitted: {answer:?}") };
    let landing = made[0].task;
    checking(&mut world, landing, head);
    assert_eq!(world.merges, 1, "the plan landed into g; L still checks");
    assert_goal_holds(&world, goal.task, &branch);
    assert_eq!(writer(&world, &branch), None);
    for number in 1..=REPAIRS {
        head = repair(&mut world, goal.task, &branch, landing, head, number);
        assert_eq!(world.merges, 1, "only the plan has landed");
    }
    exhausted(&mut world, &goal, landing, head);
    let released_head = change_while_held(&mut world, &goal, &branch, landing);
    assert_ne!(released_head, head);
    assert_eq!(row(&world, landing).change.last_head, Some(translate::commit(head)));
    status(&mut world, released_head, raw::Check::Pending);
    release(&mut world, &goal, landing);
    checking(&mut world, landing, released_head);
    assert_eq!(row(&world, landing).change.repairs, 0, "release resets the repair allowance");
    assert_eq!(row(&world, landing).drift, None);
    assert_eq!(world.merges, 2, "L waits for CI on the released head");
    status(&mut world, released_head, raw::Check::Passed);
    for _ in 0..STEPS {
        world.tick();
        assert_goal_holds(&world, goal.task, &branch);
        if ended(&world, landing).is_some() {
            break;
        }
    }
    assert!(ended(&world, landing).is_some(), "L ends after the released head passes CI");
    assert!(matches!(row(&world, landing).change.state, State::Landed { .. }));
    assert_eq!(row(&world, landing).change.last_head, Some(translate::commit(released_head)));
    let number = row(&world, landing).pull.expect("L's pull request");
    assert!(world.writes.iter().any(|write| matches!(write,
        skein_forge::api::Write::Merge { number: pull, head: merged, .. }
            if *pull == number && *merged == translate::commit(released_head))));
    let raw::Answer::Pull(pull) = world.external(1, raw::Op::Read(raw::Read::Pull { number })) else {
        panic!("L's landed pull")
    };
    assert_eq!(pull.commit, released_head, "L lands exactly the head checked after release");
    assert!(pull.merged.is_some());
    assert_eq!(branch_tip(&mut world, &branch), released_head);
    assert_eq!(world.merges, 3);
    assert_eq!(writer(&world, &branch), None);
    let before = super::news_count(&world, goal.task);
    world.send(engine::Event::Forge(Box::new(forge_top::Event::Hint {
        hint: skein_forge::api::Hint {
            repository: forge_world::REPO,
            change: skein_forge::api::Change::Branch(b"main".as_slice().into()),
            key: None,
        },
    })));
    super::await_new_forge_news(&mut world, goal.task, before);
    world.close();
}
