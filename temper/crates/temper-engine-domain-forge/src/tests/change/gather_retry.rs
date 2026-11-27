//! A gather short of a fact reads again at the client's pace (temper/docs/design/domain/forge/keeping-up.md, 3).
use super::*;
use fixtures::{GATHER_BACKOFF, assert_gather_wait, gather_clock, wall_after};
use skein_forge::api::{Answer, Error, Op, Read};
use temper_engine_domain_forge_change as c;

// Two polls would cross this bound, so the second wait must be shortened.
const STALL: Duration = Duration::from_secs(5);
const SOURCE: &[u8] = b"docs/x";
const GOAL: &[u8] = b"temper/goal";

fn start(clock: &Env<Limits>, row: ChangeRow) -> Domain {
    let mut d = fixtures::adopted_domain();
    let mut out = Queue::uncapped();
    step(&mut d, clock, Event::Change { row }, &mut out);
    gather(&mut d, clock);
    d
}

fn gather(d: &mut Domain, clock: &Env<Limits>) {
    step(d, clock, fixtures::change_step(), &mut Queue::uncapped());
}

fn unstarted(state: c::State, since: Wall) -> ChangeRow {
    let mut row = fixtures::opening_row();
    row.change.state = state;
    row.change.since = since;
    row.change.last_head = None;
    row.pending = None;
    row.effect = c::EffectResult::None;
    row
}

fn taking(clock: &Env<Limits>) -> Domain {
    let mut row = unstarted(c::State::Taking { requested: false }, clock.wall);
    row.source = Some(ChangeSource { branch: SOURCE.into(), at: None });
    start(clock, row)
}

#[test]
fn an_unrequested_close_rereads_both_tips_at_backoff() {
    let mut clock = gather_clock();
    let mut row = unstarted(c::State::Producing { requested: false }, clock.wall);
    row.base = GOAL.into();
    row.close = Some(b"close the goal".as_slice().into());
    let retry_branch = Op::Read(Read::Branch { branch: row.branch.clone() });
    let retry_goal = Op::Read(Read::Branch { branch: row.base.clone() });
    let landing = Op::Read(Read::Branch { branch: adopted().landing });
    let mut d = start(&clock, row);
    fixtures::answer_next_at(&mut d, &clock, Err(Error::Missing));
    fixtures::answer_next_at(&mut d, &clock, Ok(Answer::Commit(fixtures::BASE)));
    let (call, _) = fixtures::next_at(&mut d, &clock);
    // Another wake arrives while the landing tip is out, rather than after the wait.
    gather(&mut d, &clock);
    let answered = collected(fixtures::answer_at(&mut d, &clock, call, Err(Error::Missing)));
    assert_gather_wait(&answered, wall_after(&clock, GATHER_BACKOFF));

    fixtures::advance(&mut d, &mut clock, GATHER_BACKOFF);
    gather(&mut d, &clock);
    for (read, answer) in [(retry_branch, Err(Error::Missing)), (retry_goal, Ok(Answer::Commit(fixtures::BASE)))] {
        let (call, op) = fixtures::next_at(&mut d, &clock);
        assert_eq!(op, read);
        fixtures::answer_at(&mut d, &clock, call, answer);
    }
    assert_eq!(fixtures::next_at(&mut d, &clock).1, landing);
}

#[test]
fn a_taking_timeout_retries_at_fixed_backoff() {
    let mut clock = gather_clock();
    let mut d = taking(&clock);
    let first = fixtures::answer_next_at(&mut d, &clock, Err(Error::Timeout));
    assert_gather_wait(&first, wall_after(&clock, GATHER_BACKOFF));

    fixtures::advance(&mut d, &mut clock, GATHER_BACKOFF);
    gather(&mut d, &clock);
    let again = fixtures::answer_next_at(&mut d, &clock, Err(Error::Timeout));
    assert_gather_wait(&again, wall_after(&clock, GATHER_BACKOFF));
}

#[test]
fn the_read_bound_caps_the_next_backoff_retry() {
    let mut clock = gather_clock();
    clock.limits.change_policy.stall = STALL;
    let bound = wall_after(&clock, STALL);
    let mut d = taking(&clock);
    fixtures::answer_next_at(&mut d, &clock, Err(Error::Timeout));

    fixtures::advance(&mut d, &mut clock, GATHER_BACKOFF);
    gather(&mut d, &clock);
    let again = fixtures::answer_next_at(&mut d, &clock, Err(Error::Timeout));
    assert_gather_wait(&again, bound);
}
