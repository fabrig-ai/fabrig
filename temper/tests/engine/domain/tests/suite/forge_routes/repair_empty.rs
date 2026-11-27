//! Empty repairs recheck CI without spending another repair (change/repairs.md).
use super::questions::row;
use super::*;
use temper_engine_domain_forge_change::{Cause, Delegate, Repair, State};

const STEPS: u32 = 300;
const REPAIRS: u32 = 2;

fn repair() -> (World, engine::Assignment, u64) {
    let world = World::configured_limits(
        true,
        true,
        false,
        0,
        Gate::default(),
        None,
        StoryLimits { repairs: Some(REPAIRS), stall: Some(Duration::from_secs(30 * 60)), ..StoryLimits::default() },
    );
    let (mut world, _, producer, branch) = change_world_in(world, Change::default());
    world.external(
        1,
        raw::Op::Write(raw::Write::Status {
            commit: 1,
            context: b"build".as_slice().into(),
            state: raw::Check::Passed,
            description: Box::new([]),
            link: Box::new([]),
        }),
    );
    let repair = failed_ci_repair(&mut world, &producer, &branch, &[b"build"]);
    let tasks::Party::Task(change) = live_task(&world, repair.task).requester else { panic!("repair owner") };
    (world, repair, change)
}

fn answer(world: &mut World, repair: &engine::Assignment) {
    let event = world.protocol.answer(
        Token::new(7),
        repair.task,
        repair.attempt,
        5,
        tasks::End::Finished {
            result: tasks::TaskResult::NoChange {
                words: b"the test passed locally; no files changed".as_slice().into(),
            },
            cancel_delegates: false,
        },
        engine::Parts::none(),
    );
    world.send(event);
}

#[test]
fn an_empty_ci_repair_checks_its_head_again_without_a_hold() {
    let (mut world, repair, change) = repair();
    let head = row(&world, change).change.last_head.expect("repaired head");
    answer(&mut world, &repair);
    let mut checked = false;
    for _ in 0..STEPS {
        world.tick();
        let row = row(&world, change);
        assert!(!matches!(row.change.state, State::Held { .. }), "{:?}", row.change);
        assert!(!matches!(live_task(&world, change).phase, tasks::Phase::Held { .. }));
        if row.change.state == (State::Checking { head }) {
            checked = true;
            assert_eq!(row.change.repairs, 1);
            break;
        }
    }
    assert!(checked, "empty repair checks its unchanged head: {:?}", row(&world, change));
    // This world's translated failed checks include jobs at the head.
    let row = row(&world, change);
    assert!(row.effect_account.as_ref().is_some_and(|account| contains(&account.words, b"a repair changed nothing")));
    world.close();
}

#[test]
fn an_empty_ci_repairs_later_red_holds_without_spending_the_bound() {
    let (mut world, repair, change) = repair();
    let head = row(&world, change).change.last_head.expect("repaired head");
    assert_eq!(row(&world, change).delegate, Some((repair.task, Delegate::Repair(Repair::Ci))));
    let assigned = world.assigned.len();
    answer(&mut world, &repair);
    for _ in 0..STEPS {
        world.tick();
        if matches!(live_task(&world, change).phase, tasks::Phase::Held { .. }) {
            break;
        }
    }
    let row = row(&world, change);
    let State::Held { was, hold } = &row.change.state else { panic!("held change") };
    assert_eq!(**was, State::Repairing { requested: false, why: Repair::Ci });
    assert_eq!(hold.cause, Cause::RedStands);
    assert_eq!(row.change.last_head, Some(head));
    assert_eq!((row.change.repairs, row.change.updates), (1, 0));
    assert_eq!(world.assigned.len(), assigned, "no second repair was assigned");
    let measured = row.ci_recheck.expect("one rerun to measure the same content");
    assert_eq!(measured.job.head, head);
    assert!(measured.taken.is_some_and(|taken| taken > measured.job.attempt));
    let tasks::Phase::Held { source, why: tasks::Hold::Procedure, .. } = &live_task(&world, change).phase else {
        panic!("red stands at the head: {:?}", live_task(&world, change).phase)
    };
    let tasks::HoldSource::Account { words, .. } = source.as_ref().expect("hold account").as_ref() else {
        panic!("red stands account")
    };
    assert_eq!(hold.judged, Some(temper_engine_domain_forge_change::Judged { head, spec: row.spec_revision }));
    let head = forge_top::commit_text(&head);
    assert_eq!(words.as_ref(), format!(
        "its red at {} stands: repair task {} changed nothing, and the same content failed again: build; amend this change to ask a fresh repair, measure it again, or cancel it",
        std::str::from_utf8(&head[..12]).expect("hex head"),
        repair.task,
    ).as_bytes());
    let original = words.clone();
    let initial_revision = live_task(&world, change).revisions.hold;
    red_act(
        &mut world,
        [76; 16],
        766,
        people::Ask::Release { project: 1, task: change, revision: initial_revision, again: None },
    );
    wait_for_red_release(&mut world, change, initial_revision, false);
    world.crash_restart(766);
    world.until(Until::Ready);
    let revision = reheld(&mut world, change, initial_revision);
    let retained = questions::row(&world, change);
    assert_eq!(retained.ci_recheck, Some(measured), "ordinary release measures nothing");
    let State::Held { hold, .. } = &retained.change.state else { panic!("red re-held") };
    let prefix = format!(
        "not measured again: its red at {} stands at the same content and spec; amend this change to ask a fresh repair, measure it again, or cancel it.\n\n",
        std::str::from_utf8(&head[..12]).unwrap()
    );
    assert_eq!(hold.words.as_ref(), [prefix.as_bytes(), original.as_ref()].concat());

    let measure = people::Ask::Measure { project: 1, task: change, revision };
    red_act(&mut world, [77; 16], 767, measure.clone());
    wait_for_red_release(&mut world, change, revision, true);
    world.crash_restart(767);
    world.until(Until::Ready);
    red_act(&mut world, [77; 16], 768, measure);
    let new_revision = reheld(&mut world, change, revision);
    let fresh = questions::row(&world, change);
    let fresh_check = fresh.ci_recheck.expect("accepted measurement rechecks once");
    assert_eq!(Some(fresh_check.job.attempt), measured.taken);
    assert_eq!(fresh_check.taken, measured.taken.map(|old| old + 1));
    assert_eq!(fresh.change.repairs, 1);
    assert_eq!(world.assigned.len(), assigned);
    world.advance_ticks(STEPS);
    assert_eq!(live_task(&world, change).revisions.hold, new_revision);
    world.close();
}

#[test]
fn repeated_releases_keep_the_reds_own_account() {
    let (mut world, repair, change) = repair();
    let assigned = world.assigned.len();
    let initial_revision = live_task(&world, change).revisions.hold;
    answer(&mut world, &repair);
    let mut revision = reheld(&mut world, change, initial_revision);
    let first = row(&world, change);
    let State::Held { hold, .. } = &first.change.state else { panic!("red held") };
    let measured = first.ci_recheck.expect("one check of the same content");
    assert!(measured.taken.is_some_and(|taken| taken > measured.job.attempt));
    let head = forge_top::commit_text(&measured.job.head);
    let mut expected = format!(
        "not measured again: its red at {} stands at the same content and spec; amend this change to ask a fresh repair, measure it again, or cancel it.\n\n",
        std::str::from_utf8(&head[..12]).unwrap(),
    );
    expected.push_str(std::str::from_utf8(&hold.words).unwrap());
    let bound = usize::try_from(tasks::HOLD_ACCOUNT_BYTES).unwrap();
    expected.truncate(expected.floor_char_boundary(expected.len().min(bound)));

    for release in 0..4 {
        red_act(
            &mut world,
            [80 + release; 16],
            780 + u64::from(release),
            people::Ask::Release { project: 1, task: change, revision, again: None },
        );
        revision = reheld(&mut world, change, revision);
        let retained = row(&world, change);
        let State::Held { hold, .. } = &retained.change.state else { panic!("red re-held") };
        assert_eq!(hold.cause, Cause::RedStands);
        assert_eq!(std::str::from_utf8(&hold.words).unwrap(), expected, "saved hold after release {}", release + 1);
        let tasks::Phase::Held { source, .. } = &live_task(&world, change).phase else { panic!("task re-held") };
        let tasks::HoldSource::Account { words, .. } = source.as_deref().expect("hold account") else {
            panic!("red stands account")
        };
        assert_eq!(std::str::from_utf8(words).unwrap(), expected, "task account after release {}", release + 1);
        assert_eq!(retained.ci_recheck, Some(measured), "ordinary release measures nothing");
        assert_eq!(retained.change.repairs, first.change.repairs);
        assert_eq!(world.assigned.len(), assigned);
    }
    world.close();
}

fn red_act(world: &mut World, key: [u8; 16], token: u64, ask: people::Ask) {
    world.send(engine::Event::Party(engine::PartyInput::Ask {
        models: Box::new([]),
        reply_to: ReplyTo::new(Token::new(token)),
        sign_in: world.signed_in.expect("owner signed in"),
        key,
        ask,
    }));
}

fn wait_for_red_release(world: &mut World, change: u64, revision: u64, measure: bool) {
    for _ in 0..STEPS {
        world.tick();
        if row(world, change).task_release.is_some_and(|took| took.revision == revision && took.measure == measure) {
            return;
        }
    }
    panic!("release was not kept: {:?}", row(world, change).task_release);
}

fn reheld(world: &mut World, change: u64, release: u64) -> u64 {
    for _ in 0..STEPS {
        world.tick();
        let task = live_task(world, change);
        if task.revisions.hold > release
            && matches!(task.phase, tasks::Phase::Held { .. })
            && world.store.pending.is_empty()
        {
            return task.revisions.hold;
        }
    }
    panic!("expected a new hold after release: {:?}", row(world, change).change.state);
}
