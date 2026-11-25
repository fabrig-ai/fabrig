//! Cleanup shares the outbox line, without keeping an uncommitted number (forge/room.md, 3–4).
use super::*;
use skein_forge::api::{Answer, Write};
use temper_engine_domain_forge_change as change;

const TASK: u64 = fixtures::CHANGE;
const OTHER: u64 = 52;
const DISCARDED: u64 = 90;
const FRESH: u64 = 100;

fn pull(d: &mut Domain) {
    let mut row = outbox_room::row(TASK);
    row.pull = Some(1);
    row.change.state = change::State::Checking { head: fixtures::HEAD };
    outputs(d, Event::Change { row });
}

fn cancel(d: &mut Domain, reason: Option<&[u8]>, entry: u64) -> Box<[Request]> {
    outputs(
        d,
        Event::Release {
            task: TASK,
            root: TASK,
            ending: ReleaseEnding::Cancelled,
            reason: match reason {
                Some(reason) => Some(Box::from(reason)),
                None => None,
            },
            entry,
        },
    )
}

fn entries(out: &[Request]) -> Box<[skein_forge_client::Entry]> {
    let mut entries = skein_lib::List::uncapped();
    for request in out {
        if let Request::Save { record: Stored::Entry { entry, .. }, .. } = request {
            entries.push(entry.clone()).unwrap();
        }
    }
    entries.into_boxed()
}

fn asks(out: &[Request]) -> Box<[u64]> {
    let mut asks = skein_lib::List::uncapped();
    for request in out {
        if let Request::ContinueRelease { task } | Request::WakeProcedure { task } = request {
            asks.push(*task).unwrap();
        }
    }
    asks.into_boxed()
}

fn waited(d: &Domain, out: &[Request], place: u64) {
    assert!(!release_failed(out, TASK), "a full outbox waits: {out:?}");
    assert!(entries(out).is_empty(), "a wait keeps no entry");
    let row = d.releases.get(&TASK).unwrap();
    assert_eq!(row.room, Some(place));
    assert_eq!(row.pending, None);
    assert!(!row.failed);
    assert!(
        out.contains(&Request::Save { read_afresh: None, release: None, record: Stored::Release(row.clone()) }),
        "the place is durable: {out:?}"
    );
    assert!(!d.entries.contains_key(&DISCARDED));
}

fn freed(d: &mut Domain, number: u64) -> Box<[Request]> {
    // Withdrawal's terminal outcome removes a committed entry and frees its room.
    outputs(d, Event::Withdraw { entry: number })
}

#[test]
fn a_cleanup_entry_keeps_the_end_learned_at_settlement() {
    let mut d = refusals::adopted_domain();
    pull(&mut d);
    let closed = outputs(&mut d, Event::SettleEffects { task: TASK, root: TASK, ending: ReleaseEnding::Cancelled });
    assert!(closed.contains(&Request::EffectsSettled { task: TASK }));
    let later = outbox_room::later(&env(), Duration::from_secs(5));
    step(
        &mut d,
        &later,
        Event::Release { task: TASK, root: TASK, ending: ReleaseEnding::Cancelled, reason: None, entry: FRESH },
        &mut Queue::uncapped(),
    );
    assert_eq!(d.releases.get(&TASK).unwrap().pending, Some((FRESH, None)));
    assert_eq!(d.inspection_entry_owner_ended(FRESH), Some(env().wall));
}

#[test]
fn a_room_bound_fails_release_without_a_cleanup_write() {
    let mut d = outbox_room::full();
    pull(&mut d);
    let waiting = cancel(&mut d, None, DISCARDED);
    waited(&d, &waiting, 1);
    let clock = outbox_room::later(&env(), Duration::from_secs(5));
    let writer = refusals::entry(1).task;
    let mut out = Queue::uncapped();
    step(&mut d, &clock, Event::ChangeEnding { task: writer, ending: Ending::Cancelled }, &mut out);
    let due = outbox_room::later(&clock, clock.limits.waits.room);
    assert_eq!(d.next_wall_deadline(), Some(due.wall));
    out = Queue::uncapped();
    fire(&mut d, &due, &mut out);
    let words = format!(
        "no progress at release for 30 minutes: waiting for room in the forge's outbox, held by entry 1 of ended task {writer}"
    );
    let failed = collected(out);
    assert!(
        failed.contains(&Request::ReleaseFailed {
            task: TASK,
            entry: None,
            words: words.into_bytes().into_boxed_slice(),
        }),
        "{failed:?}"
    );
    assert!(d.releases.get(&TASK).unwrap().failed);
    assert_eq!(d.releases.get(&TASK).unwrap().room, None);
    assert!(entries(&failed).is_empty());
    assert_eq!(d.entries.len(), clock.limits.entries);
}

#[test]
fn a_waiting_release_keeps_its_room_bound_without_reporting_it() {
    let mut d = outbox_room::full();
    pull(&mut d);
    let settling = outputs(&mut d, Event::SettleEffects { task: TASK, root: TASK, ending: ReleaseEnding::Cancelled });
    assert!(settling.contains(&Request::EffectsSettled { task: TASK }));
    assert_eq!(wait_report(&settling, TASK), Some([].as_slice()), "settling clears the change's wait");
    let waiting = cancel(&mut d, None, DISCARDED);
    waited(&d, &waiting, 1);
    assert_eq!(wait_report(&waiting, TASK), None, "a release behind a live writer does not report");
    assert_eq!(d.inspection_room_wait(TASK), None, "a live writer gives the release no timed bound");

    let writer = refusals::entry(1).task;
    let ended = outbox_room::later(&env(), Duration::from_secs(5));
    let mut out = Queue::uncapped();
    step(&mut d, &ended, Event::ChangeEnding { task: writer, ending: Ending::Cancelled }, &mut out);
    assert_eq!(wait_report(&collected(out), TASK), None, "the ended writer's deadline stays internal");
    let due = advanced_wall(ended.wall, ended.limits.waits.room);
    assert_eq!(d.inspection_room_wait(TASK), Some((due, true)));
}

#[test]
fn a_release_comment_waits_then_keeps_one_fresh_entry_before_its_close() {
    let mut d = outbox_room::full();
    pull(&mut d);
    let waiting = cancel(&mut d, Some(b"cancelled"), DISCARDED);
    waited(&d, &waiting, 1);
    assert_eq!(wait_report(&waiting, TASK), Some([].as_slice()), "closing clears the change's wait");
    let asked = freed(&mut d, 1);
    assert_eq!(asks(&asked).as_ref(), &[TASK]);
    assert_eq!(wait_report(&asked, TASK), None, "the release ask does not report a wait");
    assert_eq!(room::reserved(&d), 1);
    let kept = outputs(&mut d, Event::ContinueRelease { task: TASK, entry: FRESH });
    assert_eq!(wait_report(&kept, TASK), None, "keeping the cleanup write is silent");
    let saved = entries(&kept);
    assert_eq!(saved.len(), 1);
    let entry = saved.first().unwrap();
    assert_eq!(entry.number, FRESH);
    assert!(matches!(&entry.effect.write, Write::Post { number: 1, body, .. } if body.as_ref() == b"cancelled"));
    assert_eq!(d.releases.get(&TASK).unwrap().pending, Some((FRESH, None)));
    assert_eq!(d.releases.get(&TASK).unwrap().room, None);
    assert_eq!(room::reserved(&d), 0);
    assert_eq!(entries(&outputs(&mut d, Event::ContinueRelease { task: TASK, entry: FRESH + 1 })).len(), 0);
    outputs(&mut d, Event::Committed { entry: FRESH });
    // Retire the unrelated writes; cleanup's comment is now the next lane.
    for number in 2..=u64::from(env().limits.entries) {
        freed(&mut d, number);
    }

    outputs(&mut d, Event::Client(skein_forge_client::Event::SettleOutbox));
    fixtures::answer_next(&mut d, Ok(Answer::Items { cut: false, items: Box::new([]), more: false, now: env().now }));
    let (call, op) = fixtures::next_call(&mut d);
    assert_eq!(op, skein_forge::api::Op::Write(entry.effect.write.clone()));
    let posted = fixtures::answer(&mut d, call, Ok(Answer::Commented(9)));
    assert!(posted.contains(&Request::ContinueRelease { task: TASK }));
    let closed = outputs(&mut d, Event::ContinueRelease { task: TASK, entry: FRESH + 2 });
    assert_eq!(entries(&closed).first().unwrap().effect.write, Write::Close { number: 1 });
}

#[test]
fn a_release_close_and_branch_deletion_wait_for_room() {
    for branch in [false, true] {
        let mut d = outbox_room::full();
        if branch {
            outputs(&mut d, Event::Hold { entry: 0, task: TASK, resource: run_branch(b"51"), from: None });
        } else {
            pull(&mut d);
        }
        let waiting = cancel(&mut d, None, DISCARDED);
        waited(&d, &waiting, 1);
        assert_eq!(asks(&freed(&mut d, 1)).as_ref(), &[TASK]);
        let kept = outputs(&mut d, Event::ContinueRelease { task: TASK, entry: FRESH });
        let saved = entries(&kept);
        assert_eq!(saved.len(), 1);
        let entry = saved.first().unwrap();
        assert_eq!(entry.number, FRESH);
        assert_eq!(
            entry.effect.write,
            if branch {
                Write::DeleteBranch { branch: b"temper/51".as_slice().into() }
            } else {
                Write::Close { number: 1 }
            }
        );
        assert_eq!(d.releases.get(&TASK).unwrap().room, None);
        assert_eq!(room::reserved(&d), 0);
    }
}

#[test]
fn a_change_ahead_of_release_takes_the_first_room() {
    let mut d = outbox_room::full();
    outputs(&mut d, Event::Change { row: outbox_room::row(OTHER) });
    outbox_room::open(&mut d, OTHER, DISCARDED - 1);
    pull(&mut d);
    let waiting = cancel(&mut d, None, DISCARDED);
    waited(&d, &waiting, 2);
    assert_eq!(asks(&freed(&mut d, 1)).as_ref(), &[OTHER]);
    let kept = outbox_room::open(&mut d, OTHER, FRESH);
    assert_eq!(entries(&kept).first().unwrap().number, FRESH);
    assert_eq!(asks(&freed(&mut d, 2)).as_ref(), &[TASK]);
    let kept = outputs(&mut d, Event::ContinueRelease { task: TASK, entry: FRESH + 1 });
    assert_eq!(entries(&kept).first().unwrap().number, FRESH + 1);
}

#[test]
fn release_places_restart_in_order_without_repeating_the_write() {
    let mut before = outbox_room::full();
    pull(&mut before);
    cancel(&mut before, Some(b"cancelled"), DISCARDED);
    let mut other = settling_release(None);
    other.task = OTHER;
    other.root = OTHER;
    other.effects_settled = true;
    other.releasing = true;
    other.room = Some(2);
    let mut d = fixtures::connected_domain();
    outputs(&mut d, Event::Restore { record: Stored::Repository(adopted()) });
    outputs(&mut d, Event::Restore { record: Stored::Change(Box::new(before.change(TASK).unwrap().clone())) });
    outputs(&mut d, Event::Restore { record: Stored::Release(other) });
    outputs(&mut d, Event::Restore { record: Stored::Release(before.releases.get(&TASK).unwrap().clone()) });
    for (_, entry) in &before.entries {
        outputs(&mut d, Event::Restore { record: Stored::Entry { entry: entry.entry.clone(), held: true } });
    }
    outputs(&mut d, Event::Restored { clock: skein_forge_client::RecoveryClock::Monotonic });
    outputs(&mut d, Event::CatchUp);

    assert_eq!(room::reserved(&d), 0);
    assert_eq!(d.releases.get(&TASK).unwrap().room, Some(1));
    assert_eq!(asks(&outputs(&mut d, Event::Resolve { entry: 1, again: false })).as_ref(), &[TASK]);
    let kept = outputs(&mut d, Event::ContinueRelease { task: TASK, entry: FRESH });
    assert_eq!(entries(&kept).len(), 1);
    assert_eq!(d.releases.get(&TASK).unwrap().pending, Some((FRESH, None)));
    let again = outputs(&mut d, Event::ContinueRelease { task: TASK, entry: FRESH + 1 });
    assert_eq!(entries(&again).len(), 0);
    assert_eq!(asks(&outputs(&mut d, Event::Resolve { entry: 2, again: false })).as_ref(), &[OTHER]);
}

#[test]
fn permanent_release_refusals_are_not_hidden_by_a_full_outbox() {
    for refusal in ["duplicate", "read-only", "kind", "missing", "oversize"] {
        let mut d = outbox_room::full();
        pull(&mut d);
        let mut clock = env();
        let entry = if refusal == "duplicate" { 1 } else { DISCARDED };
        match refusal {
            "read-only" => d.repositories.get_mut(&REPO).unwrap().role = Role::Context,
            "kind" => {
                let repository = d.repositories.get_mut(&REPO).unwrap();
                repository.kinds.open = false;
                repository.kinds.issue = false;
            }
            "missing" => {
                d.repositories.remove(&REPO);
            }
            "oversize" => {
                clock.limits.client.op_bytes = 1;
                d.changes.get_mut(&TASK).unwrap().pull = None;
                outputs(&mut d, Event::Hold { entry: 0, task: TASK, resource: run_branch(b"51"), from: None });
            }
            _ => {}
        }
        let mut out = Queue::uncapped();
        step(
            &mut d,
            &clock,
            Event::Release { task: TASK, root: TASK, ending: ReleaseEnding::Cancelled, reason: None, entry },
            &mut out,
        );
        let mut sent = skein_lib::List::uncapped();
        while let Some(request) = out.pop() {
            sent.push(request).unwrap();
        }
        let out = sent.into_boxed();
        let (op, cause, resource) = match refusal {
            "duplicate" => ("close item", "outbox entry 1 is already kept", "org/repo #1"),
            "read-only" => ("close item", "the repository is context", "org/repo #1"),
            "kind" => ("close item", "its adoption does not take kind close", "org/repo #1"),
            "missing" => ("close item", "the repository is not adopted", "repository 2 #1"),
            "oversize" => ("delete branch", "past forge.client.op_bytes (1)", "org/repo branch"),
            _ => unreachable!("each refusal has its own account"),
        };
        release_account(&out, TASK, None, &[op, "not sent: ", cause, resource]);
        assert!(entries(&out).is_empty());
        assert!(d.releases.get(&TASK).unwrap().failed);
        assert_eq!(d.releases.get(&TASK).unwrap().room, None);
        assert_eq!(d.releases.get(&TASK).unwrap().pending, None);
        assert_eq!(room::reserved(&d), 0);
    }
}

#[test]
fn release_comment_bound_counts_the_key_and_body() {
    // Deployment frame (3), namespace (4), release identity (11 + 6), body (2).
    const SIZE: u32 = 26;
    const BODY: &[u8] = b"no";
    for limit in [1, SIZE - 1, SIZE] {
        let mut d = refusals::adopted_domain();
        pull(&mut d);
        let mut clock = env();
        clock.limits.client.op_bytes = limit;
        let mut out = Queue::uncapped();
        step(
            &mut d,
            &clock,
            Event::Release {
                task: TASK,
                root: TASK,
                ending: ReleaseEnding::Cancelled,
                reason: Some(BODY.into()),
                entry: FRESH,
            },
            &mut out,
        );
        let out = collected(out);
        if limit == SIZE {
            assert!(!release_failed(&out, TASK));
            let entries = entries(&out);
            let [entry] = entries.as_ref() else { panic!("one keyed comment: {out:?}") };
            assert!(matches!(&entry.effect.write, Write::Post { number: 1, body, .. } if body.as_ref() == BODY));
        } else {
            let bound = format!("past forge.client.op_bytes ({limit}): {SIZE} bytes");
            release_account(&out, TASK, None, &["post comment: not sent: ", &bound, "resource org/repo #1"]);
            assert!(entries(&out).is_empty());
            assert!(d.releases.get(&TASK).unwrap().failed);
        }
    }
}

#[test]
fn an_asked_release_with_no_write_returns_room_to_the_next_waiter() {
    for facts_pending in [false, true] {
        let mut d = outbox_room::full();
        pull(&mut d);
        cancel(&mut d, None, DISCARDED);
        outputs(&mut d, Event::Change { row: outbox_room::row(OTHER) });
        outbox_room::open(&mut d, OTHER, DISCARDED + 1);
        assert_eq!(asks(&freed(&mut d, 1)).as_ref(), &[TASK]);
        if facts_pending {
            d.releases.get_mut(&TASK).unwrap().releasing = false;
        } else {
            d.changes.get_mut(&TASK).unwrap().pull = None;
        }
        let passed = outputs(&mut d, Event::ContinueRelease { task: TASK, entry: FRESH });
        assert_eq!(entries(&passed).len(), 0);
        assert_eq!(asks(&passed).as_ref(), &[OTHER]);
        assert_eq!(room::reserved(&d), 1);
        if facts_pending {
            assert_eq!(d.releases.get(&TASK).unwrap().room, None);
        } else {
            assert!(passed.contains(&Request::Released { task: TASK }));
        }
    }
}

#[test]
fn a_held_release_is_passed_over_and_rejoins_in_its_place() {
    let mut d = outbox_room::full();
    pull(&mut d);
    cancel(&mut d, None, DISCARDED);
    outputs(&mut d, Event::Change { row: outbox_room::row(OTHER) });
    outbox_room::open(&mut d, OTHER, DISCARDED + 1);
    assert_eq!(asks(&freed(&mut d, 1)).as_ref(), &[TASK]);
    let passed = outputs(&mut d, Event::RoomUnavailable { task: TASK });
    assert_eq!(asks(&passed).as_ref(), &[OTHER]);
    assert_eq!(d.releases.get(&TASK).unwrap().room, Some(1));
    // A person's release steps cleanup again; it may not take OTHER's reservation.
    let waiting = cancel(&mut d, None, DISCARDED);
    waited(&d, &waiting, 1);
    assert_eq!(asks(&freed(&mut d, 2)).as_ref(), &[TASK]);
}

#[test]
fn removing_a_forge_takes_waiting_cleanup_out_of_the_line() {
    for branch in [false, true] {
        let mut d = outbox_room::full();
        outputs(&mut d, Event::Restore { record: Stored::Forge(removal::record(1, Mode::Connected)) });
        if branch {
            outputs(&mut d, Event::Hold { entry: 0, task: TASK, resource: run_branch(b"51"), from: None });
        } else {
            pull(&mut d);
        }
        cancel(&mut d, None, DISCARDED);
        let removed = outputs(
            &mut d,
            Event::Application {
                request: Token::new(4),
                token: Token::new(104),
                retention: Duration::from_secs(60),
                ask: Ask::RemoveForge { forge: 1 },
                role: Some(jig_domain_people::Role::Owner),
                carried: None,
            },
        );
        assert_eq!(d.releases.get(&TASK).unwrap().room, None);
        assert!(removed.contains(&Request::ContinueRelease { task: TASK }));
        assert_eq!(room::reserved(&d), 0);
        let continued = outputs(&mut d, Event::ContinueRelease { task: TASK, entry: FRESH });
        assert_eq!(entries(&continued).len(), 0);
        assert!(continued.contains(&Request::Released { task: TASK }));
    }
}

#[test]
fn cancelling_a_closing_release_keeps_its_room_place() {
    let mut d = outbox_room::full();
    pull(&mut d);
    // A failed child's branch cleanup can already be waiting when cancel replaces its ending.
    outputs(&mut d, Event::Hold { entry: 0, task: TASK, resource: run_branch(b"51"), from: None });
    outputs(
        &mut d,
        Event::Release { task: TASK, root: TASK, ending: ReleaseEnding::Failed, reason: None, entry: DISCARDED },
    );
    assert_eq!(d.releases.get(&TASK).unwrap().room, Some(1));
    outputs(&mut d, Event::Change { row: outbox_room::row(OTHER) });
    outbox_room::open(&mut d, OTHER, DISCARDED + 1);
    let waiting = cancel(&mut d, None, DISCARDED + 2);
    waited(&d, &waiting, 1);
}
