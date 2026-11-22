//! Lagging views at the step boundary, without a world.
use super::*;
use crate::{Fault, Fell, Lag};

const SPAN: Duration = Duration::from_secs(10);

fn fixture() -> (Harness, u64, u64) {
    let mut setup = setup();
    setup.checks.contexts = Box::new([]);
    let config = Config {
        latency_min: Duration::ZERO,
        latency_max: Duration::ZERO,
        hook_min: Duration::ZERO,
        hook_max: Duration::ZERO,
        skew: Skew::Ahead(Duration::from_secs(200)),
        ..calm()
    };
    let mut h = Harness::with(config, setup);
    let head = h.commit(FIRST, &[(b"work", b"one")]);
    h.push(ENGINE, b"work", head).unwrap();
    let Answer::Created(number) = h.open(b"work").unwrap() else { panic!("pull") };
    h.settle();
    h.heard();
    h.env.now = Time::from_nanos(100_000_000_000);
    h.ok(PERSON, read(Read::Locate));
    (h, number, head)
}

fn enable(h: &mut Harness) {
    crate::lag(&mut h.domain, Lag { chance: 1000, min: SPAN, max: SPAN });
}

fn tick(h: &mut Harness, at: Time) {
    h.env.now = at;
    h.ok(PERSON, read(Read::Locate));
}

fn listed(h: &mut Harness) -> Box<[Pull]> {
    let Answer::Pulls(pulls) = h.ok(PERSON, read(Read::OpenPulls { head: None, base: None, page: 1, limit: 0 })) else {
        panic!("pulls")
    };
    pulls
}

fn record(at: Time, fault: Fault, number: u64, commit: u64) -> Fell {
    Fell {
        at,
        until: Some(at.saturating_add(SPAN)),
        fault,
        repository: Some(copy_of(REPOSITORY)),
        user: None,
        call: None,
        reply_to: None,
        pull: Some(number),
        commit: Some(commit),
        run: None,
        runner: None,
    }
}

#[test]
fn head_behind_preserves_mergeability_in_reads_and_listings_until_its_end() {
    let (mut h, number, head) = fixture();
    let before = h.pull(number);
    assert!(before.mergeable);
    enable(&mut h);
    let next = h.commit(head, &[(b"work", b"two")]);
    let at = h.env.now;
    h.push(ENGINE, b"work", next).unwrap();
    assert_eq!(h.branch(b"work"), Some(next));
    assert_eq!(h.pull(number), before);
    assert_eq!(listed(&mut h).as_ref(), core::slice::from_ref(&before));
    let Answer::PullFiles { head: files_head, .. } = h.ok(PERSON, read(Read::PullFiles { number, page: 1, limit: 0 }))
    else {
        panic!("files")
    };
    assert_eq!(files_head, head);
    tick(&mut h, at.saturating_add(Duration::from_secs(9)));
    assert_eq!(h.pull(number), before);
    assert_eq!(h.domain.pop_fell(), Some(record(at, Fault::HeadBehind, number, next)));
    assert_eq!(h.domain.pop_fell(), None);
    tick(&mut h, at.saturating_add(SPAN));
    assert_eq!(h.pull(number).commit, next);
    assert_eq!(listed(&mut h).first().unwrap().commit, next);
}

#[test]
fn base_behind_is_recorded_for_each_open_pull() {
    let (mut h, first, head) = fixture();
    let other = h.commit(FIRST, &[(b"other", b"one")]);
    h.push(ENGINE, b"other", other).unwrap();
    let Answer::Created(second) = h.open(b"other").unwrap() else { panic!("pull") };
    h.settle();
    enable(&mut h);
    let at = h.env.now;
    let next = h.commit(FIRST, &[(b"base", b"two")]);
    h.push(ENGINE, MAIN, next).unwrap();
    assert_eq!(h.branch(MAIN), Some(next));
    let pulls = listed(&mut h);
    assert_eq!(pulls.len(), 2);
    for pull in &pulls {
        assert_eq!(pull.base_commit, Some(FIRST));
        assert!(pull.mergeable);
    }
    assert_eq!(h.pull(first).commit, head);
    for number in [first, second] {
        assert_eq!(h.domain.pop_fell(), Some(record(at, Fault::BaseBehind, number, next)));
    }
    assert_eq!(h.domain.pop_fell(), None);
    tick(&mut h, at.saturating_add(SPAN));
    for pull in listed(&mut h) {
        assert_eq!(pull.base_commit, Some(next));
    }
}

#[test]
fn merge_late_keeps_reads_open_but_moves_the_base_and_sends_the_hook() {
    let (mut h, number, head) = fixture();
    let before = h.pull(number);
    enable(&mut h);
    let at = h.env.now;
    let Answer::Merged(commit) = h.ok(ENGINE, merge(number, head)) else { panic!("merged") };
    h.settle();
    assert_eq!(h.branch(MAIN), Some(commit));
    assert!(h.heard().as_slice().contains(&(Change::Pull, Some(number))));
    assert_eq!(h.pull(number), before);
    assert_eq!(listed(&mut h).as_ref(), &[before]);
    let (item, _) = h.item(number);
    assert_eq!((item.state, item.merged), (State::Open, false));
    assert_eq!(h.list(PERSON, Some(State::Open), &[], 1).0.as_slice(), &[number]);
    assert!(h.list(PERSON, Some(State::Closed), &[], 1).0.is_empty());
    tick(&mut h, at.saturating_add(Duration::from_secs(2)));
    let later = h.commit(commit, &[(b"later", b"write")]);
    h.push(ENGINE, MAIN, later).unwrap();
    assert_eq!(h.branch(MAIN), Some(later));
    assert_eq!(h.pull(number).merged, None);
    assert_eq!(h.domain.pop_fell(), Some(record(at, Fault::MergeLate, number, commit)));
    assert_eq!(h.domain.pop_fell(), None);
    tick(&mut h, at.saturating_add(SPAN));
    assert_eq!((h.pull(number).state, h.pull(number).merged), (State::Closed, Some(commit)));
    assert!(listed(&mut h).is_empty());
    assert_eq!(h.list(PERSON, Some(State::Closed), &[], 1).0.as_slice(), &[number]);
}

#[test]
fn pull_lookup_prefers_a_lagging_open_view_to_an_older_closed_pull() {
    let (mut h, first, head) = fixture();
    h.ok(ENGINE, merge(first, head));
    let next = h.commit(head, &[(b"second", b"two")]);
    h.push(ENGINE, b"work", next).unwrap();
    let Answer::Created(second) = h.open(b"work").unwrap() else { panic!("pull") };
    enable(&mut h);
    let at = h.env.now;
    h.ok(ENGINE, merge(second, next));
    let lookup = read(Read::PullFor { head: copy_of(b"work"), base: copy_of(MAIN) });
    let Answer::Pull(shown) = h.ok(PERSON, lookup.clone()) else { panic!("pull") };
    assert_eq!((shown.number, shown.state), (second, State::Open));
    tick(&mut h, at.saturating_add(SPAN));
    let Answer::Pull(shown) = h.ok(PERSON, lookup) else { panic!("pull") };
    assert_eq!((shown.number, shown.state), (first, State::Closed));
}

#[test]
fn a_second_move_does_not_extend_the_view_and_merges_use_the_truth() {
    let (mut h, number, head) = fixture();
    enable(&mut h);
    let at = h.env.now;
    let next = h.commit(head, &[(b"work", b"two")]);
    h.push(ENGINE, b"work", next).unwrap();
    tick(&mut h, at.saturating_add(Duration::from_secs(5)));
    let last = h.commit(next, &[(b"work", b"three")]);
    h.push(ENGINE, b"work", last).unwrap();
    assert_eq!(h.pull(number).commit, head);
    assert_eq!(h.call(ENGINE, merge(number, head)), Err(Error::Stale));
    assert_eq!(h.domain.pop_fell(), Some(record(at, Fault::HeadBehind, number, next)));
    assert_eq!(h.domain.pop_fell(), None);
    tick(&mut h, at.saturating_add(SPAN));
    assert_eq!(h.pull(number).commit, last);
    let next_at = h.env.now;
    let newest = h.commit(last, &[(b"work", b"four")]);
    h.push(ENGINE, b"work", newest).unwrap();
    assert_eq!(h.pull(number).commit, last);
    assert_eq!(h.domain.pop_fell(), Some(record(next_at, Fault::HeadBehind, number, newest)));
}

#[test]
fn zero_chance_matches_the_default_views_and_random_stream() {
    let (mut default, number, head) = fixture();
    let (mut zero, zero_number, zero_head) = fixture();
    assert_eq!((number, head), (zero_number, zero_head));
    enable(&mut zero);
    crate::lag(&mut zero.domain, Lag { chance: 0, min: SPAN, max: SPAN });
    for h in [&mut default, &mut zero] {
        let rng = h.domain.rng.state();
        let id = h.domain.id(REPOSITORY);
        crate::lag::start(&mut h.domain, &h.env, id, number, Some(FIRST), head, Fault::HeadBehind);
        assert_eq!(h.domain.rng.state(), rng, "a disabled view draws nothing");
        let next = h.commit(head, &[(b"work", b"two")]);
        h.push(ENGINE, b"work", next).unwrap();
        assert_eq!(h.pull(number).commit, next);
        h.ok(ENGINE, merge(number, next));
        assert_eq!(h.pull(number).state, State::Closed);
        assert_eq!(h.domain.pop_fell(), None);
    }
    assert_eq!(default.domain.rng.state(), zero.domain.rng.state());
    assert_eq!(default.pull(number), zero.pull(number));
    assert_eq!(default.domain.image().unwrap(), zero.domain.image().unwrap());
}

#[test]
fn an_image_keeps_current_state_but_neither_views_nor_the_setting() {
    for merging in [false, true] {
        let (mut h, number, head) = fixture();
        enable(&mut h);
        let current = if merging {
            let Answer::Merged(commit) = h.ok(ENGINE, merge(number, head)) else { panic!("merged") };
            commit
        } else {
            let next = h.commit(head, &[(b"work", b"two")]);
            h.push(ENGINE, b"work", next).unwrap();
            next
        };
        assert_eq!(h.pull(number).commit, head);
        assert_eq!(h.pull(number).state, State::Open);
        let restored = Domain::from_image(&h.env.limits, &h.domain.image().unwrap()).unwrap();
        let Answer::Pull(pull) = restored.inspect(&h.env.limits, REPOSITORY, &Read::Pull { number }).unwrap() else {
            panic!("pull")
        };
        if merging {
            assert_eq!((pull.state, pull.merged), (State::Closed, Some(current)));
        } else {
            assert_eq!(pull.commit, current);
        }
        assert_eq!(restored.lag, None);
    }
}
