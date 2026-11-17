//! A release uses the pull state its poll kept (forge/change/README.md, 4).
use super::*;
use crate::domain::{from_client, pull_changed};
use skein_forge::api::{Answer, Error, Op, State, Write};
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

fn seen(d: &mut Domain, closed: bool) {
    let mut pull = fixtures::open_pull();
    pull.head = d.change(TASK).unwrap().branch.clone();
    if closed {
        pull.state = State::Closed;
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

fn in_flight(d: &mut Domain, write: Write) -> Token {
    outputs(d, Event::Committed { entry: ENTRY });
    let clock = env();
    for _ in 0..clock.limits.client.pending.saturating_add(2) {
        let (call, op) = fixtures::next_at(d, &clock);
        match op {
            Op::Read(skein_forge::api::Read::Branch { .. }) => {
                fixtures::answer_at(d, &clock, call, Ok(Answer::Commit(fixtures::HEAD)));
            }
            Op::Read(skein_forge::api::Read::Items { .. }) => {
                fixtures::answer_at(
                    d,
                    &clock,
                    call,
                    Ok(Answer::Items { cut: false, items: Box::new([]), more: false, now: clock.now }),
                );
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

fn refused_with_account(out: &Queue<Request>) {
    let mut found = false;
    for request in out {
        if let Request::ReleaseFailed { task, account } = request {
            assert_eq!(*task, TASK);
            assert!(account.is_some(), "{out:?}");
            found = true;
        }
    }
    assert!(found, "{out:?}");
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
        seen(&mut d, true);
        // Close admission is account-free; a closed pull needs no such admission.
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
    seen(&mut d, false);
    release(&mut d, ReleaseEnding::Cancelled, None, ENTRY);
    let call = in_flight(&mut d, Write::Close { number: 1 });
    seen(&mut d, true);
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
    seen(&mut d, false);
    release(&mut d, ReleaseEnding::Cancelled, None, ENTRY);
    let call = in_flight(&mut d, Write::Close { number: 1 });
    let out = fixtures::answer_at(&mut d, &env(), call, Err(Error::Refused));
    refused_with_account(&out);
    assert!(d.releases.get(&TASK).unwrap().failed);
}

#[test]
fn retrying_a_refused_close_after_seen_merged_sends_no_more_pull_writes() {
    let mut d = setup(true);
    seen(&mut d, false);
    release(&mut d, ReleaseEnding::Cancelled, None, ENTRY);
    let call = in_flight(&mut d, Write::Close { number: 1 });
    fixtures::answer_at(&mut d, &env(), call, Err(Error::Refused));
    assert!(d.releases.get(&TASK).unwrap().failed);
    assert!(!d.entries.contains_key(&ENTRY));
    seen(&mut d, true);
    let out = release(&mut d, ReleaseEnding::Cancelled, Some(b"cancelled again"), NEXT);
    ended(&d, &out, true);
}

#[test]
fn a_parent_failing_after_cancelled_close_does_not_strand_its_closer() {
    let mut d = setup(true);
    seen(&mut d, false);
    outputs(&mut d, Event::SettleEffects { task: TASK, root: GOAL, ending: ReleaseEnding::Cancelled });
    release(&mut d, ReleaseEnding::Cancelled, None, ENTRY);
    let call = in_flight(&mut d, Write::Close { number: 1 });
    fixtures::answer_at(&mut d, &env(), call, Err(Error::Refused));
    seen(&mut d, true);
    let out = outputs(&mut d, Event::SettleEffects { task: TASK, root: GOAL, ending: ReleaseEnding::Failed });
    no_failure(&out);
    let out = release(&mut d, ReleaseEnding::Failed, None, NEXT);
    ended(&d, &out, true);
}

#[test]
fn a_held_cancelled_release_takes_failed_and_retains_its_branch_to_the_root() {
    for settle in [false, true] {
        let mut d = setup(false);
        seen(&mut d, false);
        release(&mut d, ReleaseEnding::Cancelled, None, ENTRY);
        let call = in_flight(&mut d, Write::Close { number: 1 });
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
    for (held, root) in [(false, GOAL), (true, GOAL + 1)] {
        let mut d = setup(true);
        release(&mut d, ReleaseEnding::Cancelled, None, ENTRY);
        let call = in_flight(&mut d, Write::Close { number: 1 });
        fixtures::answer_at(&mut d, &env(), call, if held { Err(Error::Refused) } else { Ok(Answer::Done) });
        for settling in [true, false] {
            let event = if settling {
                Event::SettleEffects { task: TASK, root, ending: ReleaseEnding::Failed }
            } else {
                Event::Release { task: TASK, root, ending: ReleaseEnding::Failed, reason: None, entry: NEXT }
            };
            let out = outputs(&mut d, event);
            assert!(out.contains(&Request::ReleaseFailed { task: TASK, account: None }), "{out:?}");
            assert_eq!(d.releases.get(&TASK).unwrap().ending, ReleaseEnding::Cancelled);
        }
    }
}

#[test]
fn a_merged_changes_branch_missing_or_refused_is_not_an_account_free_failure() {
    for error in [Error::Missing, Error::Refused] {
        let mut d = setup(false);
        seen(&mut d, true);
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
        let call = in_flight(&mut d, Write::DeleteBranch { branch: b"temper/51".as_slice().into() });
        let out = fixtures::answer_at(&mut d, &env(), call, Err(error));
        for request in &out {
            assert!(!matches!(request, Request::ReleaseFailed { account: None, .. }), "{out:?}");
        }
        if error == Error::Missing {
            let out = outputs(&mut d, Event::ContinueRelease { task: TASK, entry: NEXT });
            ended(&d, &out, false);
            assert!(d.hold(&branch_name(b"temper/51")).is_none());
        } else {
            refused_with_account(&out);
        }
    }
}
