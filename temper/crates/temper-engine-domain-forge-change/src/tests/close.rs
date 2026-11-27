//! Close producers (temper/docs/design/domain/forge/change/README.md, 1, temper/docs/design/domain/forge/change/repairs.md).
use super::*;

const TIPS: crate::CloseAt = crate::CloseAt { branch: HEAD, landing: BASE, asked: true, refused: None, outdated: None };
const REFUSAL: crate::CloseAt = crate::CloseAt { refused: Some(crate::CloseRefused { counts: None }), ..TIPS };

fn close(requested: bool) -> (Change, Facts) {
    let mut c = change(State::Producing { requested });
    c.last_head = None;
    let mut f = facts();
    f.branch = None;
    f.pull = Pull::Missing;
    f.close = true;
    f.close_tips = Some(TIPS);
    f.delegate = requested.then_some(Delegate::Close);
    (c, f)
}

#[test]
fn a_close_changes_producer_is_the_close_job() {
    let (c, f) = close(false);
    let next = decide(&c, &f, &heard());
    assert_eq!(next.decision, Decision::Delegate(Delegate::Close));
    assert_eq!(next.change.state, State::Producing { requested: true });
}

#[test]
fn a_close_waits_while_its_closer_waits_for_work() {
    let (c, f) = close(false);
    let h = Heard { closer_waiting: true, ..heard() };
    let next = decide(&c, &f, &h);
    assert_eq!(next.decision, Decision::Wait { until: None });
    assert_eq!(next.change.state, c.state);
}

#[test]
fn a_close_with_unchanged_tips_holds_on_its_refusal() {
    let (c, mut f) = close(false);
    f.close_at = Some(REFUSAL);
    let h = Heard { delegate: Status::Failed, ..heard() };
    let next = decide(&c, &f, &h);
    assert_eq!(next.decision, Decision::Hold(Cause::CloseRefused));
    assert_eq!(
        next.change.state,
        State::Held {
            was: Box::new(State::Producing { requested: true }),
            hold: crate::Hold::new(Cause::CloseRefused, at(1))
        }
    );
}

#[test]
fn a_refused_close_restarts_if_either_tip_moved() {
    let (c, mut f) = close(false);
    f.close_at = Some(REFUSAL);
    let h = Heard { delegate: Status::Failed, ..heard() };
    for tips in [crate::CloseAt { branch: NEW, ..TIPS }, crate::CloseAt { landing: NEW, ..TIPS }] {
        f.close_tips = Some(tips);
        assert_eq!(decide(&c, &f, &h).decision, Decision::Delegate(Delegate::Close));
    }
}

#[test]
fn a_close_without_fresh_tips_waits() {
    let (c, mut f) = close(false);
    f.close_at = Some(TIPS);
    f.close_tips = None;
    let h = Heard { delegate: Status::Failed, ..heard() };
    assert_eq!(decide(&c, &f, &h).decision, Decision::Wait { until: None });
}

#[test]
fn a_close_with_a_head_opens_without_a_producer() {
    let (c, mut f) = close(false);
    f.branch = Some(NEW);
    let next = decide(&c, &f, &heard());
    assert_eq!(next.decision, Decision::None);
    assert_eq!(next.change.state, State::Opening { requested: false });
    assert_eq!(next.change.last_head, Some(NEW));
}

#[test]
fn a_close_job_that_pushed_nothing_ends_with_no_change() {
    let (c, f) = close(true);
    let h = Heard { delegate: Status::Passed, ..heard() };
    let next = decide(&c, &f, &h);
    assert_eq!(next.decision, Decision::NoChange);
    assert_eq!(next.change.state, State::NoChange { words: b"Nothing to settle.".as_slice().into() });
}

#[test]
fn an_outdated_close_job_ends_without_a_hold() {
    use crate::CloseOutdated;
    const BRANCH: [u8; 32] =
        [0xc0, 0xff, 0xee, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1];
    for (finding, words) in [
        (CloseOutdated::Window, b"The close check didn't judge the goal's branch at c0ffee000001: it holds none of the commits main's published counts cover; it asks for the goal's catch-up from main.".as_slice()),
        (CloseOutdated::Ancestry, b"The close check didn't judge the goal's branch at c0ffee000001: its ancestry was incomplete; it asks for the goal's catch-up from main.".as_slice()),
    ] {
        let (c, mut f) = close(true);
        f.close_at = Some(crate::CloseAt { branch: BRANCH, outdated: Some(finding), ..TIPS });
        let h = Heard { delegate: Status::Passed, ..heard() };
        let next = decide(&c, &f, &h);
        assert_eq!(next.decision, Decision::NoChange);
        assert_eq!(next.change, Change { state: State::NoChange { words: words.into() }, ..c });
    }
}

#[test]
fn a_close_job_that_pushed_moves_the_change_to_opening() {
    let (c, mut f) = close(true);
    f.branch = Some(NEW);
    assert_eq!(decide(&c, &f, &heard()).change.state, State::Producing { requested: true });
    let h = Heard { delegate: Status::Passed, ..heard() };
    let next = decide(&c, &f, &h);
    assert_eq!(next.change.state, State::Opening { requested: false });
    assert_eq!(next.change.last_head, Some(NEW));
}

#[test]
fn a_failed_close_job_holds_the_producer() {
    let (c, mut f) = close(true);
    f.branch = Some(NEW);
    let h = Heard { delegate: Status::Failed, ..heard() };
    let next = decide(&c, &f, &h);
    assert_eq!(next.decision, Decision::Hold(Cause::Failed));
    assert_eq!(next.change.state, State::Held { was: Box::new(c.state), hold: crate::Hold::new(Cause::Failed, at(1)) });
}
