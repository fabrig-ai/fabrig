use super::*;

fn ms(millis: u64) -> Time {
    Time::from_nanos(millis * 1_000_000)
}
fn history(by: Party, at: u64) -> History {
    History {
        tasks: BTreeMap::from([
            (2, Task { requester: None, agent: true }),
            (3, Task { requester: None, agent: false }),
            (4, Task { requester: None, agent: false }),
            (5, Task { requester: Some(4), agent: true }),
            (6, Task { requester: None, agent: false }),
        ]),
        cancellations: BTreeMap::from([(4, Cancellation { by, at: ms(at) })]),
    }
}
fn kill(at: u64) -> Kill {
    Kill { root: Some(1), at: ms(at), reason: Reason::Cancelled, exit: Exit::Signal(9) }
}
fn note(at: u64) -> Note {
    Note { root: 1, at: ms(at), run: 5, attempt: 1 }
}

#[test]
fn only_the_latest_root_notes_stand_for_a_cancel() {
    let mut notes = vec![note(10)];
    // An older unrelated cancel, a future cancel, and another root do not stand for this kill.
    notes.push(Note { root: 1, at: ms(7), run: 6, attempt: 2 });
    notes.push(Note { root: 1, at: ms(11), run: 6, attempt: 3 });
    notes.push(Note { root: 9, at: ms(10), run: 6, attempt: 4 });
    let history = history(Party::Task(2), 9);
    let judged = judge(&[kill(10)], &notes, &history, 0);
    assert_eq!(judged.refusal, None);
    assert_eq!(judged.details, ["; stands for run 5 attempt 1, cancelled with task 4"]);
}

#[test]
fn every_attempt_in_a_cascade_must_be_the_storys() {
    let notes = [note(10), Note { root: 1, at: ms(10), run: 6, attempt: 2 }];
    let history = history(Party::Task(2), 9);
    let judged = judge(&[kill(10)], &notes, &history, 0);
    assert!(judged.refusal.expect("unasked member").starts_with("only requested kills"));
    assert_eq!(
        judged.details,
        ["; stands for run 5 attempt 1, cancelled with task 4; stands for run 6 attempt 2, not the story's cancel"]
    );
}

#[test]
fn a_story_cancel_accounts_for_its_delegates_kill() {
    for (by, at) in [(Party::Task(2), 9), (Party::Person(1), 10)] {
        let notes = [note(10), Note { root: 1, at: ms(10), run: 4, attempt: 2 }];
        let judged = judge(&[kill(10)], &notes, &history(by, at), 0);
        assert_eq!(judged.refusal, None);
        assert_eq!(
            judged.details,
            ["; stands for run 5 attempt 1, cancelled with task 4; stands for run 4 attempt 2, cancelled with task 4"]
        );
    }
}

#[test]
fn only_story_cancels_seen_by_the_kill_account_for_it() {
    for (by, at, kill_at) in [
        (Party::Deployment { project: 1 }, 9, 10),
        (Party::Task(3), 9, 10),
        (Party::Task(99), 9, 10),
        (Party::Person(1), 9, 8),
    ] {
        let judged = judge(&[kill(kill_at)], &[note(kill_at)], &history(by, at), 0);
        assert!(judged.refusal.expect("unasked cancel").starts_with("only requested kills"));
        assert_eq!(judged.details, ["; stands for run 5 attempt 1, not the story's cancel"]);
    }
}

#[test]
fn a_cancel_needs_a_root_note_and_a_kill_signal() {
    let history = history(Party::Task(2), 9);
    for (root, at, exit, expected) in [
        (Some(1), 10, Exit::Signal(9), "; no cancel noted"),
        (Some(2), 11, Exit::Signal(9), "; no cancel noted"),
        (None, 11, Exit::Signal(9), "; no cancel noted"),
        (Some(1), 11, Exit::Code(0), "; stands for run 5 attempt 1, cancelled with task 4"),
    ] {
        let mut kill = kill(at);
        kill.root = root;
        kill.exit = exit;
        let judged = judge(&[kill], &[note(11)], &history, 0);
        assert!(judged.refusal.expect("unasked kill").starts_with("only requested kills"));
        assert_eq!(judged.details, [expected]);
    }
    assert_eq!(judge(&[kill(10)], &[], &history, 0).details, ["; no cancel noted"]);
}

#[test]
fn an_explicit_kill_and_a_cancel_kill_are_both_requested() {
    let kills = [Kill { reason: Reason::Asked, ..kill(10) }, kill(10)];
    let judged = judge(&kills, &[note(10)], &history(Party::Task(2), 9), 1);
    assert_eq!(judged.refusal, None);
    assert_eq!(judged.details, ["", "; stands for run 5 attempt 1, cancelled with task 4"]);
}

#[test]
fn other_kills_still_need_exactly_the_requested_kill_steps() {
    let history = history(Party::Task(2), 9);
    for reason in [Reason::Deadline, Reason::Grace, Reason::Closed, Reason::Asked, Reason::Unknown] {
        let mut unaccounted = kill(10);
        unaccounted.reason = reason;
        let judged = judge(&[unaccounted], &[note(10)], &history, 0);
        assert!(judged.refusal.expect("needs a Kill step").starts_with("only requested kills"));
        assert_eq!(judged.details, [""]);
    }
    let kills = [kill(10), Kill { reason: Reason::Asked, ..kill(10) }, Kill { reason: Reason::Grace, ..kill(10) }];
    let judged = judge(&kills, &[note(10)], &history, 1);
    assert_eq!(
        judged.refusal.as_deref(),
        Some("only requested kills: 3 recorded, 1 accounted for by the story's cancels, for 1 Kill steps")
    );
    assert_eq!(judged.details, ["; stands for run 5 attempt 1, cancelled with task 4", "", ""]);
    assert!(judge(&[], &[], &history, 1).refusal.is_some(), "a missing requested kill still fails");
    assert!(
        judge(&[Kill { reason: Reason::Asked, exit: Exit::Code(0), ..kill(10) }], &[], &history, 1).refusal.is_some()
    );
}
