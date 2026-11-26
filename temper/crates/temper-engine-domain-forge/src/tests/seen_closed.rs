//! A release uses the pull state its poll kept (forge/change/README.md, 4).
use super::*;
use crate::domain::{from_client, pull_changed};
use skein_forge::api::{Answer, Error, Op, Permission, Read, State, UserPermission, Write};
use temper_engine_domain_forge_change as change;

const TASK: u64 = fixtures::CHANGE;
const GOAL: u64 = 1;
const ENTRY: u64 = 90;
const NEXT: u64 = 91;
const GOAL_BRANCH: &[u8] = b"temper/1/g1";

fn setup(closer: bool) -> Domain {
    let mut d = refusals::adopted_domain();
    let mut row = outbox_room::row(TASK);
    row.pull = Some(1);
    row.change.state = change::State::Checking { head: fixtures::HEAD };
    if closer {
        row.branch = GOAL_BRANCH.into();
        row.goal = Some(GOAL);
    }
    let name = branch_name(&row.branch);
    outputs(&mut d, Event::Change { row });
    outputs(&mut d, Event::Hold { entry: 0, task: if closer { GOAL } else { TASK }, resource: name, from: None });
    d
}

fn branch_name(branch: &[u8]) -> Name {
    from_client(
        &skein_forge_client::Resource { repository: REPO, what: skein_forge_client::What::Branch(branch.into()) },
        &env().limits,
    )
    .unwrap()
}

fn seen(d: &mut Domain, state: State, merged: bool) {
    let mut pull = fixtures::open_pull();
    pull.head = d.change(TASK).unwrap().branch.clone();
    pull.state = state;
    if merged {
        pull.merged = Some(fixtures::HEAD);
    }
    pull_changed(
        d,
        &env(),
        skein_forge_client::Resource { repository: REPO, what: skein_forge_client::What::Pull(1) },
        pull,
        &mut Queue::uncapped(),
    );
}

fn release(d: &mut Domain, ending: ReleaseEnding, reason: Option<&[u8]>, entry: u64) -> Box<[Request]> {
    let reason = match reason {
        Some(reason) => Some(Box::from(reason)),
        None => None,
    };
    outputs(d, Event::Release { task: TASK, root: GOAL, ending, reason, entry })
}

fn in_flight(d: &mut Domain, entry: u64, write: Write) -> Token {
    outputs(d, Event::Committed { entry });
    let clock = env();
    for _ in 0..clock.limits.client.pending.saturating_add(2) {
        let (call, op) = fixtures::next_at(d, &clock);
        match op {
            Op::Read(Read::Branch { .. }) => {
                fixtures::answer_at(d, &clock, call, Ok(Answer::Commit(fixtures::HEAD)));
            }
            Op::Read(Read::Items { .. }) => {
                fixtures::answer_at(
                    d,
                    &clock,
                    call,
                    Ok(Answer::Items { cut: false, items: Box::new([]), more: false, now: clock.now }),
                );
            }
            Op::Read(Read::File { .. }) => {
                fixtures::answer_at(d, &clock, call, Err(Error::Missing));
            }
            op @ (Op::Read(_) | Op::Write(_)) => {
                assert_eq!(op, Op::Write(write));
                return call;
            }
        }
    }
    panic!("cleanup write never sent")
}

fn no_failure(out: &[Request]) {
    for request in out {
        assert!(!matches!(request, Request::ReleaseFailed { .. }), "{out:?}");
    }
}

fn no_pull_write(out: &[Request]) {
    no_failure(out);
    for request in out {
        if let Request::Save { record: Stored::Entry { entry, .. }, .. } = request {
            assert!(!matches!(entry.effect.write, Write::Post { .. } | Write::Close { .. }), "{out:?}");
        }
    }
}

fn no_write(out: &[Request]) {
    for request in out {
        assert!(
            !matches!(
                request,
                Request::Save { record: Stored::Entry { .. }, .. } | Request::Call { op: Op::Write(_), .. }
            ),
            "{out:?}"
        );
    }
}

fn refused_with_account(out: Queue<Request>, parts: &[&str]) {
    release_account(&collected(out), TASK, Some(ENTRY), parts);
}

fn ended(d: &Domain, out: &[Request], closer: bool) {
    no_pull_write(out);
    assert!(out.contains(&Request::Released { task: TASK }), "{out:?}");
    assert!(d.landed.is_empty(), "another hand's merge is not a landing");
    assert_eq!(d.change(TASK).unwrap().change.state, change::State::Ended);
    let holder = match d.hold(&branch_name(GOAL_BRANCH)) {
        Some(hold) => Some(hold.task),
        None => None,
    };
    assert_eq!(holder, if closer { Some(GOAL) } else { None });
}

#[test]
fn a_seen_closed_pull_skips_cleanup_even_when_admission_would_refuse_it() {
    for closer in [false, true] {
        let mut d = setup(closer);
        seen(&mut d, State::Closed, true);
        // A closed pull needs no write admission.
        let repository = d.repositories.get_mut(&REPO).unwrap();
        repository.kinds.comment = false;
        repository.kinds.open = false;
        repository.kinds.issue = false;
        let out = release(&mut d, ReleaseEnding::Cancelled, Some(b"cancelled"), ENTRY);
        no_pull_write(&out);
        if closer {
            ended(&d, &out, true);
        } else {
            assert_eq!(
                d.entries.get(&ENTRY).unwrap().entry.effect.write,
                Write::DeleteBranch { branch: b"temper/51".as_slice().into() }
            );
        }
    }
}

#[test]
fn a_close_refused_after_the_poll_saw_merged_releases_without_a_failure() {
    let mut d = setup(true);
    seen(&mut d, State::Open, false);
    release(&mut d, ReleaseEnding::Cancelled, None, ENTRY);
    let call = in_flight(&mut d, ENTRY, Write::Close { number: 1 });
    seen(&mut d, State::Closed, true);
    let out = fixtures::answer_at(&mut d, &env(), call, Err(Error::Refused));
    for request in &out {
        assert!(!matches!(request, Request::ReleaseFailed { .. }), "{out:?}");
    }
    let out = outputs(&mut d, Event::ContinueRelease { task: TASK, entry: NEXT });
    ended(&d, &out, true);
}

#[test]
fn a_close_refused_while_still_seen_open_keeps_its_account() {
    let mut d = setup(true);
    seen(&mut d, State::Open, false);
    release(&mut d, ReleaseEnding::Cancelled, None, ENTRY);
    let call = in_flight(&mut d, ENTRY, Write::Close { number: 1 });
    let out = fixtures::answer_at(&mut d, &env(), call, Err(Error::Refused));
    refused_with_account(out, &["close item: ", "forge policy refused the operation", "resource org/repo #1"]);
    assert!(d.releases.get(&TASK).unwrap().failed);
}

#[test]
fn retrying_a_refused_close_after_a_hand_close_sends_no_more_pull_writes() {
    let mut d = setup(true);
    seen(&mut d, State::Open, false);
    release(&mut d, ReleaseEnding::Cancelled, None, ENTRY);
    let call = in_flight(&mut d, ENTRY, Write::Close { number: 1 });
    fixtures::answer_at(&mut d, &env(), call, Err(Error::Refused));
    assert!(d.releases.get(&TASK).unwrap().failed);
    assert!(!d.entries.contains_key(&ENTRY));
    seen(&mut d, State::Closed, false);
    let out = release(&mut d, ReleaseEnding::Cancelled, Some(b"cancelled again"), NEXT);
    ended(&d, &out, true);
}

#[test]
fn a_parent_failing_after_cancelled_close_does_not_strand_its_closer() {
    let mut d = setup(true);
    seen(&mut d, State::Open, false);
    outputs(&mut d, Event::SettleEffects { task: TASK, root: GOAL, ending: ReleaseEnding::Cancelled });
    release(&mut d, ReleaseEnding::Cancelled, None, ENTRY);
    let call = in_flight(&mut d, ENTRY, Write::Close { number: 1 });
    fixtures::answer_at(&mut d, &env(), call, Err(Error::Refused));
    seen(&mut d, State::Closed, true);
    let out = outputs(&mut d, Event::SettleEffects { task: TASK, root: GOAL, ending: ReleaseEnding::Failed });
    no_failure(&out);
    let out = release(&mut d, ReleaseEnding::Failed, None, NEXT);
    ended(&d, &out, true);
}

#[test]
fn a_held_cancelled_release_takes_failed_and_retains_its_branch_to_the_root() {
    for settle in [false, true] {
        let mut d = setup(false);
        seen(&mut d, State::Open, false);
        release(&mut d, ReleaseEnding::Cancelled, None, ENTRY);
        let call = in_flight(&mut d, ENTRY, Write::Close { number: 1 });
        fixtures::answer_at(&mut d, &env(), call, Err(Error::Refused));
        assert!(d.releases.get(&TASK).unwrap().failed);
        if settle {
            let out = outputs(&mut d, Event::SettleEffects { task: TASK, root: GOAL, ending: ReleaseEnding::Failed });
            no_failure(&out);
        }
        let out = release(&mut d, ReleaseEnding::Failed, None, NEXT);
        ended(&d, &out, false);
        let name = branch_name(b"temper/51");
        assert!(out.contains(&Request::Retained { task: TASK, root: GOAL, resource: name.clone() }));
        assert_eq!(d.hold(&name).unwrap().task, GOAL);
        for request in &out {
            assert!(!matches!(request, Request::Save { record: Stored::Entry { .. }, .. }));
        }
    }
}

#[test]
fn another_ending_of_an_unheld_release_or_another_root_is_still_refused() {
    for (root, ending, asked) in [
        (TASK, ReleaseEnding::Failed, "asked by root 51 as failed"),
        (52, ReleaseEnding::Cancelled, "asked by root 52 as cancelled"),
    ] {
        let mut d = setup(false);
        outputs(
            &mut d,
            Event::Release { task: TASK, root: TASK, ending: ReleaseEnding::Done, reason: None, entry: ENTRY },
        );
        let prior = d.releases.get(&TASK).unwrap().clone();
        assert_eq!(prior.pending, Some((ENTRY, Some(branch_name(b"temper/51")))));
        for event in [
            Event::SettleEffects { task: TASK, root, ending },
            Event::Release { task: TASK, root, ending, reason: None, entry: NEXT },
        ] {
            let out = outputs(&mut d, event);
            release_account(&out, TASK, None, &["release of task 51: ", "under way for root 51 as done", asked]);
            assert_eq!(d.releases.get(&TASK), Some(&prior));
            assert!(d.entries.contains_key(&ENTRY), "a refused cancel does not withdraw the owner's write");
            no_write(&out);
        }
    }
}

#[test]
fn a_merged_changes_branch_missing_or_refused_is_not_an_account_free_failure() {
    for error in [Error::Missing, Error::Forbidden] {
        let mut d = setup(false);
        seen(&mut d, State::Closed, true);
        // Isolate deletion from the pull cleanup, which other tests cover.
        let row = ReleaseRow {
            task: TASK,
            root: GOAL,
            ending: ReleaseEnding::Cancelled,
            close_pull: None,
            comment: None,
            pending: None,
            failed: false,
            effects_settled: true,
            releasing: true,
            room: None,
        };
        d.releases.insert(TASK, row).unwrap();
        outputs(&mut d, Event::ContinueRelease { task: TASK, entry: ENTRY });
        let call = in_flight(&mut d, ENTRY, Write::DeleteBranch { branch: b"temper/51".as_slice().into() });
        let out = fixtures::answer_at(&mut d, &env(), call, Err(error));
        if error == Error::Missing {
            no_failure(&collected(out));
            let out = outputs(&mut d, Event::ContinueRelease { task: TASK, entry: NEXT });
            ended(&d, &out, false);
            assert!(d.hold(&branch_name(b"temper/51")).is_none());
        } else {
            refused_with_account(
                out,
                &["delete branch: ", "forge refused permission", "resource org/repo branch temper/51"],
            );
            assert!(d.releases.get(&TASK).unwrap().failed);
            assert_eq!(d.hold(&branch_name(b"temper/51")).unwrap().task, TASK);
            // A person deleted the branch meanwhile and restored the bot's access.
            for (read, answer) in [
                (
                    Read::Permission { user: 7 },
                    Answer::Permission(UserPermission { user: 7, login: Box::new([]), permission: Permission::Write }),
                ),
                (Read::Settings, Answer::Settings(adopted().settings)),
                (Read::Protection { branch: b"main".as_slice().into() }, Answer::Protection(None)),
            ] {
                let (call, op) = fixtures::next_at(&mut d, &env());
                assert_eq!(op, Op::Read(read));
                fixtures::answer_at(&mut d, &env(), call, Ok(answer));
            }
            no_failure(&release(&mut d, ReleaseEnding::Cancelled, None, NEXT));
            let call = in_flight(&mut d, NEXT, Write::DeleteBranch { branch: b"temper/51".as_slice().into() });
            no_failure(&collected(fixtures::answer_at(&mut d, &env(), call, Err(Error::Missing))));
            let out = outputs(&mut d, Event::ContinueRelease { task: TASK, entry: NEXT + 1 });
            ended(&d, &out, false);
            assert!(d.hold(&branch_name(b"temper/51")).is_none());
        }
    }
}

#[test]
fn a_stored_release_with_no_change_names_its_missing_pull_record() {
    let mut d = refusals::adopted_domain();
    let mut row = settling_release(None);
    row.task = TASK;
    row.root = TASK;
    row.ending = ReleaseEnding::Cancelled;
    row.close_pull = Some(7);
    row.releasing = true;
    row.effects_settled = true;
    outputs(&mut d, Event::Restore { record: Stored::Release(row) });
    let out = outputs(&mut d, Event::ContinueRelease { task: TASK, entry: ENTRY });
    release_account(&out, TASK, None, &["release of task 51: ", "no change record", "pull request #7"]);
    no_write(&out);
    assert!(d.releases.get(&TASK).unwrap().failed);
}

#[test]
fn release_refuses_a_branch_still_written_by_another_entry() {
    let mut d = setup(false);
    let name = branch_name(b"temper/51");
    let mut hold = d.hold(&name).unwrap().clone();
    hold.writer = Some(Writer::Entry(ENTRY));
    outputs(&mut d, Event::Restore { record: Stored::Entry { entry: status(ENTRY, GOAL), held: true } });
    outputs(&mut d, Event::Restore { record: Stored::Hold(hold) });
    let out = release(&mut d, ReleaseEnding::Done, None, NEXT);
    release_account(
        &out,
        TASK,
        None,
        &["delete branch: ", "still written by outbox entry 90", "resource org/repo branch temper/51"],
    );
    no_write(&out);
    assert!(d.releases.get(&TASK).unwrap().failed);
    assert_eq!(d.hold(&name).unwrap().writer, Some(Writer::Entry(ENTRY)));
}

#[test]
fn after_restart_only_the_unfinished_part_of_a_release_is_written() {
    let mut before = setup(false);
    release(&mut before, ReleaseEnding::Cancelled, None, ENTRY);
    let call = in_flight(&mut before, ENTRY, Write::Close { number: 1 });
    fixtures::answer_at(&mut before, &env(), call, Ok(Answer::Done));
    let name = branch_name(b"temper/51");
    let mut d = fixtures::connected_domain();
    for record in [
        Stored::Repository(adopted()),
        Stored::Change(Box::new(before.change(TASK).unwrap().clone())),
        Stored::Hold(before.hold(&name).unwrap().clone()),
        Stored::Release(before.releases.get(&TASK).unwrap().clone()),
    ] {
        outputs(&mut d, Event::Restore { record });
    }
    outputs(&mut d, Event::Restored { clock: skein_forge_client::RecoveryClock::Monotonic });
    outputs(&mut d, Event::CatchUp);

    // With no closed-pull observation to rely on, the stored release keeps
    // the close from being written again.
    let out = release(&mut d, ReleaseEnding::Cancelled, Some(b"cancelled again"), NEXT);
    no_pull_write(&out);
    let call = in_flight(&mut d, NEXT, Write::DeleteBranch { branch: b"temper/51".as_slice().into() });
    no_failure(&collected(fixtures::answer_at(&mut d, &env(), call, Ok(Answer::Done))));
    let out = outputs(&mut d, Event::ContinueRelease { task: TASK, entry: NEXT + 1 });
    ended(&d, &out, false);
}

#[test]
fn a_full_release_table_names_its_limit() {
    let mut d = refusals::adopted_domain();
    let mut clock = env();
    clock.limits.tasks = 1;
    let first = Event::SettleEffects { task: GOAL, root: GOAL, ending: ReleaseEnding::Done };
    let mut out = Queue::uncapped();
    step(&mut d, &clock, first, &mut out);
    assert!(collected(out).contains(&Request::EffectsSettled { task: GOAL }));
    for event in [
        Event::SettleEffects { task: TASK, root: TASK, ending: ReleaseEnding::Done },
        Event::Release { task: TASK, root: TASK, ending: ReleaseEnding::Done, reason: None, entry: ENTRY },
    ] {
        let mut out = Queue::uncapped();
        step(&mut d, &clock, event, &mut out);
        release_account(
            &collected(out),
            TASK,
            None,
            &["release of task 51: ", "forge.tasks is 1", "this needs 2", "a limit in temper-limits"],
        );
        assert!(!d.releases.contains_key(&TASK));
    }
}
