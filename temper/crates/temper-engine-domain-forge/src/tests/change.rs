use super::*;

/// Reverse ancestry is read once per pair, including uncertain answers (keeping-up.md, 3).
#[test]
fn a_head_contained_in_its_base_ends_without_rereading_the_pair() {
    use skein_forge::api::{Answer, CommitMessage, Op, Read};
    use temper_engine_domain_forge_change as change;
    let mut env = env();
    env.limits.client.read_attempts = 1;
    for (answer, status) in fixtures::ancestry_cases() {
        // Connection failures have their own retry chain; the ancestry fold covers them separately.
        if matches!(
            answer,
            Err(skein_forge::api::Error::Unavailable
                | skein_forge::api::Error::Forbidden
                | skein_forge::api::Error::Missing)
        ) {
            continue;
        }
        let mut d = fixtures::connected_domain();
        outputs(&mut d, Event::Restore { record: Stored::Repository(adopted()) });
        let mut row = fixtures::opening_row();
        row.pending = None;
        row.effect = change::EffectResult::None;
        row.pull = Some(1);
        row.change.state = change::State::Checking { head: fixtures::HEAD };
        // A previously updated head waits for CI instead of updating before it concludes.
        row.change.clean = Box::new([fixtures::HEAD]);
        outputs(&mut d, Event::Restore { record: Stored::Change(Box::new(row)) });
        outputs(&mut d, Event::Restored { clock: skein_forge_client::RecoveryClock::Monotonic });
        outputs(&mut d, Event::ReadAfresh);
        outputs(&mut d, Event::SettleOutbox);
        for (tip, read, live) in [
            (fixtures::BASE, true, true),
            (fixtures::BASE, false, true),
            ([5; 32], true, true),
            ([5; 32], false, false),
        ] {
            seen_base(&mut d, tip);
            outputs(&mut d, waiting_step(live, false));
            let mut pull = fixtures::open_pull();
            pull.base_commit = Some(tip);
            pull.ci = skein_forge::api::Ci::Pending;
            for answer in [
                Answer::Commit(fixtures::HEAD),
                Answer::Commit(tip),
                Answer::Pull(pull),
                fixtures::checks(skein_forge::api::Check::Passed),
            ] {
                let (call, _) = fixtures::next_at(&mut d, &env);
                fixtures::answer_at(&mut d, &env, call, Ok(answer));
            }
            let (call, op) = fixtures::next_at(&mut d, &env);
            assert_eq!(op, Op::Read(Read::Commits { before: fixtures::HEAD, after: tip, page: 1 }));
            fixtures::answer_at(
                &mut d,
                &env,
                call,
                Ok(Answer::Commits {
                    commits: Box::new([CommitMessage { id: tip, parents: Box::new([]), message: Box::new([]) }]),
                    more: false,
                }),
            );
            if read {
                let (call, op) = fixtures::next_at(&mut d, &env);
                assert_eq!(op, Op::Read(Read::Commits { before: tip, after: fixtures::HEAD, page: 1 }));
                fixtures::answer_at(&mut d, &env, call, answer.clone());
            }
            let (call, op) = fixtures::next_at(&mut d, &env);
            assert_eq!(op, Op::Read(Read::Reviews { number: 1, page: 1, skip: 0 }), "no second ancestry read");
            fixtures::answer_at(
                &mut d,
                &env,
                call,
                Ok(Answer::Reviews { reviews: Box::new([]), more: false, cut: false }),
            );
            let state = &d.change(fixtures::CHANGE).expect("change").change.state;
            if !live && status == change::Status::Passed {
                assert_eq!(state, &change::State::NoChange { words: Box::from(b"The landing branch already contains its head 0303030303030303030303030303030303030303030303030303030303030303.".as_slice()) });
            } else {
                assert_eq!(state, &change::State::Checking { head: fixtures::HEAD });
            }
        }
    }
}

/// Keep workflow discovery outside the change's fresh-read chain.
fn seen_base(d: &mut Domain, commit: skein_forge::api::Commit) {
    let name = Name {
        forge: REPO.forge,
        repository: REPO.repository,
        what: What::Branch(Box::new([Box::from(b"main".as_slice())])),
    };
    d.heads.insert(name.clone(), BranchHead { name, commit, owned: false }).expect("fixture head room");
}

// env() starts at 100 s; the change policy stalls for another 30 s.
const WAIT_UNTIL: Wall = Wall::from_nanos(130_000_000_000);

const LATER_CHANGE: u64 = fixtures::CHANGE + 1;
const FIRST_ENTRY: u64 = fixtures::OPEN + 1;
const LATER_ENTRY: u64 = FIRST_ENTRY + 1;
const LATER_BRANCH: &[u8] = b"temper/52";

#[test]
fn a_ci_outage_excludes_unanswered_time_but_keeps_prior_stall_time() {
    use skein_forge::api::{Answer, Error};
    use temper_engine_domain_forge_change as c;
    let mut d = fixtures::connected_domain();
    let mut env = env();
    env.limits.client.read_attempts = 1;
    outputs(&mut d, Event::Restore { record: Stored::Repository(adopted()) });
    let mut row = fixtures::opening_row();
    row.pending = None;
    row.effect = c::EffectResult::None;
    row.pull = Some(1);
    row.change.state = c::State::Checking { head: fixtures::HEAD };
    row.change.since = Wall::from_nanos(env.wall.as_nanos() - Duration::from_secs(10).as_nanos());
    outputs(&mut d, Event::Restore { record: Stored::Change(Box::new(row)) });
    outputs(&mut d, Event::Restored { clock: skein_forge_client::RecoveryClock::Monotonic });
    outputs(&mut d, Event::ReadAfresh);
    outputs(&mut d, Event::SettleOutbox);
    fixtures::step_change(&mut d);
    let mut pull = fixtures::open_pull();
    pull.ci = skein_forge::api::Ci::Pending;
    for answer in [Answer::Commit(fixtures::HEAD), Answer::Commit(fixtures::BASE)] {
        fixtures::answer_next(&mut d, Ok(answer));
    }
    let (call, checks) = fixtures::next_at(&mut d, &env);
    fixtures::answer_at(&mut d, &env, call, Err(Error::Unavailable));
    fixtures::advance(&mut d, &mut env, Duration::from_secs(100));
    let (again, op) = fixtures::next_at(&mut d, &env);
    assert_eq!(op, checks);
    fixtures::answer_at(&mut d, &env, again, Ok(Answer::Pull(pull)));
    let since = d.change(fixtures::CHANGE).expect("checking").change.since;
    assert_eq!(
        env.wall.as_nanos() - since.as_nanos(),
        Duration::from_secs(10).as_nanos(),
        "only ten answered seconds count"
    );
    let mut out = Queue::uncapped();
    // The pull and checks agree that CI is pending.
    for answer in [
        fixtures::checks(skein_forge::api::Check::Pending),
        Answer::Commits { commits: Box::new([]), more: false },
        Answer::Reviews { reviews: Box::new([]), more: false, cut: false },
    ] {
        let (call, _) = fixtures::next_at(&mut d, &env);
        out = fixtures::answer_at(&mut d, &env, call, Ok(answer));
    }
    let until = Wall::from_nanos(env.wall.as_nanos() + Duration::from_secs(20).as_nanos());
    let mut waiting = false;
    for request in &out {
        assert!(
            !matches!(request, Request::ChangeDecision { decision: c::Decision::Hold(c::Hold::Stalled), .. }),
            "{out:?}"
        );
        if let Request::ChangeDecision { decision: c::Decision::Wait { until: Some(deadline) }, .. } = request {
            waiting |= *deadline == until;
        }
    }
    assert!(waiting, "twenty answered seconds remain before the stall: {out:?}");
    assert_eq!(d.change(fixtures::CHANGE).expect("still waiting").change.since, since);
}

#[test]
fn a_connection_wait_excludes_unanswered_time_but_keeps_prior_stall_time() {
    use skein_forge::api::{Answer, Ci, Error};
    use temper_engine_domain_forge_change as c;
    for (titled, restart_stall) in [(false, false), (true, false), (false, true)] {
        let mut d = fixtures::connected_domain();
        let mut env = env();
        outputs(&mut d, Event::Restore { record: Stored::Repository(adopted()) });
        let mut row = fixtures::opening_row();
        row.pending = None;
        row.effect = c::EffectResult::None;
        row.pull = Some(1);
        row.change.state = if titled {
            row.title = Box::new([]);
            c::State::Producing { requested: true }
        } else {
            c::State::Checking { head: fixtures::HEAD }
        };
        row.change.since = Wall::from_nanos(env.wall.as_nanos() - Duration::from_secs(10).as_nanos());
        outputs(&mut d, Event::Restore { record: Stored::Change(Box::new(row)) });
        outputs(&mut d, Event::Restored { clock: skein_forge_client::RecoveryClock::Monotonic });
        outputs(&mut d, Event::ReadAfresh);
        outputs(&mut d, Event::SettleOutbox);
        outputs(&mut d, Event::Connection { forge: 1, state: Connection::Connecting });
        fixtures::step_change(&mut d);
        fixtures::answer_next(&mut d, Ok(Answer::Commit(fixtures::HEAD)));
        fixtures::answer_next(&mut d, Err(Error::NotConnected));
        fixtures::advance(&mut d, &mut env, Duration::from_secs(50));
        if restart_stall {
            // A delegate ending during the wait starts a later stall.
            d.changes.get_mut(&fixtures::CHANGE).expect("change").change.since = env.wall;
        }
        let mut out = Queue::uncapped();
        step(&mut d, &env, fixtures::change_step(), &mut out);
        fixtures::advance(&mut d, &mut env, Duration::from_secs(50));
        step(
            &mut d,
            &env,
            Event::Connection { forge: 1, state: Connection::Connected { id: 7, login: b"bot".as_slice().into() } },
            &mut out,
        );
        let since = d.change(fixtures::CHANGE).expect("waiting change").change.since;
        let counted = if restart_stall { 0 } else { 10 };
        assert_eq!(env.wall.as_nanos() - since.as_nanos(), Duration::from_secs(counted).as_nanos());
        let mut saved = false;
        for request in &out {
            if let Request::Save { record: Stored::Change(row), .. } = request {
                saved |= row.change.since == since;
            }
        }
        assert!(saved, "the excluded wait is saved: {out:?}");
        let remaining = 30 - counted;
        let until = Wall::from_nanos(env.wall.as_nanos() + Duration::from_secs(remaining).as_nanos());
        for (elapsed, decision) in
            [(remaining - 1, c::Decision::Wait { until: Some(until) }), (1, c::Decision::Hold(c::Hold::Stalled))]
        {
            fixtures::advance(&mut d, &mut env, Duration::from_secs(elapsed));
            step(&mut d, &env, fixtures::change_step(), &mut out);
            let mut pull = fixtures::open_pull();
            pull.ci = Ci::Pending;
            for answer in [
                Answer::Commit(fixtures::HEAD),
                Answer::Commit(fixtures::BASE),
                Answer::Pull(pull),
                fixtures::checks(skein_forge::api::Check::Passed),
                Answer::Commits { commits: Box::new([]), more: false },
                Answer::Reviews { reviews: Box::new([]), more: false, cut: false },
            ] {
                let (call, _) = fixtures::next_at(&mut d, &env);
                out = fixtures::answer_at(&mut d, &env, call, Ok(answer));
            }
            let mut decided = false;
            for request in &out {
                decided |= *request
                    == Request::ChangeDecision { task: fixtures::CHANGE, decision, entry: None, evidence: None };
            }
            assert!(decided, "{out:?}");
            assert_eq!(d.change(fixtures::CHANGE).expect("change").change.since, since);
        }
    }
}

fn producing_with_history(known: bool, pushed: bool, merged: bool) -> Domain {
    use skein_forge::api::{Answer, State};
    use temper_engine_domain_forge_change as change;
    let mut d = fixtures::connected_domain();
    outputs(&mut d, Event::Restore { record: Stored::Repository(adopted()) });
    let mut row = fixtures::opening_row();
    row.pending = None;
    row.effect = change::EffectResult::None;
    row.pull = if known { Some(1) } else { None };
    row.change.state = change::State::Producing { requested: false };
    row.change.last_head = if pushed { Some(fixtures::HEAD) } else { None };
    outputs(&mut d, Event::Restore { record: Stored::Change(Box::new(row)) });
    outputs(&mut d, Event::Restored { clock: skein_forge_client::RecoveryClock::Monotonic });
    outputs(&mut d, Event::ReadAfresh);
    outputs(&mut d, Event::SettleOutbox);
    fixtures::step_change(&mut d);
    fixtures::answer_next(&mut d, Err(skein_forge::api::Error::Missing));
    // With neither a recorded pull nor a pushed head, no history is read.
    if !known && !pushed {
        return d;
    }
    fixtures::answer_next(&mut d, Ok(Answer::Commit(fixtures::BASE)));
    let mut pull = fixtures::open_pull();
    pull.state = State::Closed;
    pull.merged = if merged { Some(fixtures::BASE) } else { None };
    let (call, op) = fixtures::next_call(&mut d);
    let expected = if known {
        skein_forge::api::Read::Pull { number: 1, ci: true }
    } else {
        skein_forge::api::Read::PullFor {
            head: Box::from(b"temper/51".as_slice()),
            base: Box::from(b"main".as_slice()),
        }
    };
    assert_eq!(op, skein_forge::api::Op::Read(expected));
    fixtures::answer(&mut d, call, Ok(Answer::Pull(pull)));
    fixtures::answer_next(&mut d, Ok(fixtures::checks(skein_forge::api::Check::Passed)));
    d
}

#[test]
fn foreign_pull_history_does_not_land_or_name_a_producing_change() {
    use skein_forge::api::{Answer, Op, Read};
    use temper_engine_domain_forge_change as change;
    for merged in [true, false] {
        let mut d = producing_with_history(false, false, merged);
        let row = d.change(fixtures::CHANGE).expect("producing change");
        assert_eq!(row.pull, None, "another deployment's number is never recorded");
        assert_eq!(row.change.state, change::State::Producing { requested: true });
        assert_eq!(row.drift, None, "foreign history is missing, not drift");
        // The producer pushes its new head on the reused branch name.
        fixtures::step_change(&mut d);
        fixtures::answer_next(&mut d, Ok(Answer::Commit(fixtures::HEAD)));
        fixtures::answer_next(&mut d, Ok(Answer::Commit(fixtures::BASE)));
        let (call, op) = fixtures::next_call(&mut d);
        assert_eq!(
            op,
            Op::Read(Read::PullFor { head: Box::from(b"temper/51".as_slice()), base: Box::from(b"main".as_slice()) })
        );
        let mut pull = fixtures::open_pull();
        pull.state = skein_forge::api::State::Closed;
        pull.commit = [9; 32];
        pull.merged = if merged { Some(fixtures::BASE) } else { None };
        fixtures::answer(&mut d, call, Ok(Answer::Pull(pull)));
        let finished = fixtures::finish_missing(&mut d);
        let mut opens = false;
        for request in &finished {
            if let Request::ChangeDecision { decision: change::Decision::Effect(change::Effect::Open), .. } = request {
                opens = true;
            }
        }
        assert!(opens, "the pushed change opens its own pull: {finished:?}");
        assert_eq!(d.change(fixtures::CHANGE).expect("opening change").pull, None);
    }
}

#[test]
fn a_recorded_merged_pull_lands_without_a_matching_pushed_head() {
    let d = producing_with_history(true, false, true);
    assert_eq!(
        d.change(fixtures::CHANGE).expect("known change").change.state,
        temper_engine_domain_forge_change::State::Landed { merge: fixtures::BASE, since: env().wall }
    );
}

#[test]
fn a_merged_pull_at_the_pushed_head_is_recovered_after_restart() {
    let d = producing_with_history(false, true, true);
    let row = d.change(fixtures::CHANGE).expect("recovered change");
    assert_eq!(row.pull, Some(1));
    assert_eq!(
        row.change.state,
        temper_engine_domain_forge_change::State::Landed { merge: fixtures::BASE, since: env().wall }
    );
}

#[test]
fn a_pending_prepare_rebase_is_not_branch_drift() {
    use skein_forge::api::{Answer, Hint, Op, Read, UpdateStyle, Write};
    use temper_engine_domain_forge_change as change;
    for pending in [true, false] {
        let mut d = fixtures::connected_domain();
        outputs(&mut d, Event::Restore { record: Stored::Repository(adopted()) });
        let name = run_branch(b"51");
        outputs(&mut d, Event::Hold { entry: 0, task: fixtures::CHANGE, resource: name.clone(), from: None });
        outputs(&mut d, Event::Restored { clock: skein_forge_client::RecoveryClock::Monotonic });
        outputs(&mut d, Event::ReadAfresh);
        outputs(&mut d, Event::SettleOutbox);
        outputs(&mut d, Event::Names { task: fixtures::CHANGE, resources: Box::new([name.clone()]) });
        fixtures::answer_next(&mut d, Ok(Answer::Commit(fixtures::HEAD)));
        outputs(
            &mut d,
            Event::Client(skein_forge_client::Event::Pushed {
                resource: skein_forge_client::Resource {
                    repository: REPO,
                    what: skein_forge_client::What::Branch(Box::from(b"temper/51".as_slice())),
                },
                commit: fixtures::HEAD,
            }),
        );
        let mut row = fixtures::opening_row();
        row.change.way = change::Way::Turn;
        row.change.state =
            change::State::Preparing { from: fixtures::HEAD, main: fixtures::BASE, ready_since: env().wall };
        row.pending = if pending { Some(FIRST_ENTRY) } else { None };
        outputs(&mut d, Event::Change { row });
        outputs(
            &mut d,
            Event::Enqueue {
                entry: skein_forge_client::Entry {
                    number: FIRST_ENTRY,
                    task: fixtures::CHANGE,
                    repository: REPO,
                    effect: skein_forge_client::Effect {
                        write: Write::Update { number: 1, style: UpdateStyle::Rebase },
                        condition: skein_forge_client::Condition::Update { head: fixtures::HEAD, base: fixtures::BASE },
                    },
                    start: None,
                    attempt: None,
                    failures: 0,
                    queued: false,
                },
            },
        );
        outputs(
            &mut d,
            Event::Hint {
                hint: Hint {
                    repository: REPO,
                    change: skein_forge::api::Change::Branch(Box::from(b"temper/51".as_slice())),
                    key: None,
                },
            },
        );
        let mut later = env();
        later.now = later.now.saturating_add(LIMITS.client.hinted);
        let mut out = Queue::uncapped();
        fire(&mut d, &later, &mut out);
        let (call, op) = fixtures::next_call(&mut d);
        assert_eq!(op, Op::Read(Read::Branch { branch: Box::from(b"temper/51".as_slice()) }));
        fixtures::answer(&mut d, call, Ok(Answer::Commit(fixtures::BASE)));
        assert_eq!(d.hold(&name).expect("held branch").drift.is_none(), pending);
        assert_eq!(d.change(fixtures::CHANGE).expect("preparing change").drift.is_none(), pending);
    }
}

fn start_opening(d: &mut Domain, task: u64, entry: u64) {
    start_opening_kept(d, task, entry, false);
}

fn start_opening_kept(d: &mut Domain, task: u64, entry: u64, kept: bool) {
    use temper_engine_domain_forge_change as change;
    outputs(
        d,
        Event::StepChange {
            merge_job: false,
            task,
            entry,
            heard: change::Heard {
                delegate_running: false,
                delegate_live: false,
                review_request: None,
                start_conflict: None,
                lacking: None,
                spend: None,
                delegate: change::Status::Passed,
                effect: change::EffectResult::None,
                released: false,
                cancelled: false,
            },
            gates: Box::new([]),
            queue_repair_active: false,
            pending: Box::new([]),
            kept,
        },
    );
}

/// Leave the lower chain's last read in flight while the higher entry commits.
fn reversed_openings(lower_requested: bool) -> (Domain, Token) {
    use temper_engine_domain_forge_change as change;
    let mut d = fixtures::connected_domain();
    outputs(&mut d, Event::Restore { record: Stored::Repository(adopted()) });
    outputs(&mut d, Event::Restored { clock: skein_forge_client::RecoveryClock::Monotonic });
    outputs(&mut d, Event::ReadAfresh);
    outputs(&mut d, Event::SettleOutbox);
    let mut row = fixtures::opening_row();
    row.pending = None;
    row.pull = None;
    row.effect = change::EffectResult::None;
    row.change.state = change::State::Opening { requested: lower_requested };
    outputs(&mut d, Event::Change { row: row.clone() });
    row.task = LATER_CHANGE;
    row.change.task = LATER_CHANGE;
    row.change.state = change::State::Opening { requested: false };
    row.branch = Box::from(LATER_BRANCH);
    outputs(&mut d, Event::Change { row });
    start_opening(&mut d, fixtures::CHANGE, FIRST_ENTRY);
    fixtures::missing_pull_reads(&mut d);
    let green = fixtures::checks(skein_forge::api::Check::Passed);
    fixtures::answer_next(&mut d, Ok(green.clone()));
    fixtures::answer_next(&mut d, Ok(green));
    let (lower, _) = fixtures::next_call(&mut d);
    start_opening(&mut d, LATER_CHANGE, LATER_ENTRY);
    fixtures::missing_pull_reads(&mut d);
    let built = fixtures::finish_missing(&mut d);
    assert!(built_entry(&built, LATER_ENTRY), "higher effect: {built:?}");
    outputs(&mut d, Event::Committed { entry: LATER_ENTRY });
    (d, lower)
}

fn answer_entry_clock(d: &mut Domain) {
    fixtures::answer_next(
        d,
        Ok(skein_forge::api::Answer::Items { cut: false, items: Box::new([]), more: false, now: env().now }),
    );
}

fn built_entry(requests: &[Request], number: u64) -> bool {
    for request in requests {
        if let Request::ChangeDecision { entry: Some(entry), .. } = request
            && *entry == number
        {
            return true;
        }
    }
    false
}

#[test]
fn changes_finishing_in_reverse_order_reach_the_client_in_number_order() {
    use skein_forge::api::{Answer, Op, Write};
    let (mut d, lower) = reversed_openings(false);
    let built = finish_lower(&mut d, lower);
    assert!(built_entry(&built, FIRST_ENTRY), "lower effect: {built:?}");
    outputs(&mut d, Event::Committed { entry: FIRST_ENTRY });
    answer_entry_clock(&mut d);
    let (first, op) = fixtures::next_call(&mut d);
    let Op::Write(Write::OpenPull { head, .. }) = op else { panic!("first opening") };
    assert_eq!(head.as_ref(), b"temper/51");
    fixtures::answer(&mut d, first, Ok(Answer::Created(2)));
    answer_entry_clock(&mut d);
    let (_, op) = fixtures::next_call(&mut d);
    let Op::Write(Write::OpenPull { head, .. }) = op else { panic!("later opening") };
    assert_eq!(head.as_ref(), LATER_BRANCH);
}

fn finish_lower(d: &mut Domain, lower: Token) -> Box<[Request]> {
    fixtures::answer(d, lower, Ok(skein_forge::api::Answer::Commits { commits: Box::new([]), more: false }))
}

#[test]
fn a_woken_chain_restarts_with_its_newest_entry_number() {
    use skein_forge::api::{Answer, Op, Read, Write};
    let (mut d, lower) = reversed_openings(false);
    let newest = LATER_ENTRY + 2;
    start_opening(&mut d, fixtures::CHANGE, newest - 1);
    start_opening(&mut d, fixtures::CHANGE, newest);
    assert!(finish_lower(&mut d, lower).is_empty(), "the woken chain restarts");
    let (branch, op) = fixtures::next_call(&mut d);
    assert_eq!(op, Op::Read(Read::Branch { branch: Box::from(b"temper/51".as_slice()) }));
    // Leave the restarted chain's first read unanswered while B writes.
    answer_entry_clock(&mut d);
    let (later, op) = fixtures::next_call(&mut d);
    let Op::Write(Write::OpenPull { head, .. }) = op else { panic!("later opening before restarted reads finish") };
    assert_eq!(head.as_ref(), LATER_BRANCH);
    fixtures::answer(&mut d, later, Ok(Answer::Created(2)));
    fixtures::answer(&mut d, branch, Ok(Answer::Commit(fixtures::HEAD)));
    fixtures::answer_next(&mut d, Ok(Answer::Commit(fixtures::BASE)));
    fixtures::answer_next(&mut d, Err(skein_forge::api::Error::Missing));
    let built = fixtures::finish_missing(&mut d);
    assert!(built_entry(&built, newest), "restarted effect uses the newest reservation: {built:?}");
}

#[test]
fn a_wake_that_unkeeps_an_inflight_chain_is_not_lost() {
    use skein_forge::api::Answer;
    use temper_engine_domain_forge_change as change;
    let mut d = fixtures::connected_domain();
    outputs(&mut d, Event::Restore { record: Stored::Repository(adopted()) });
    outputs(&mut d, Event::Restored { clock: skein_forge_client::RecoveryClock::Monotonic });
    outputs(&mut d, Event::ReadAfresh);
    outputs(&mut d, Event::SettleOutbox);
    let mut row = fixtures::opening_row();
    row.pending = None;
    row.effect = change::EffectResult::None;
    row.pull = Some(1);
    row.change.state = change::State::Gating { head: fixtures::HEAD, asked: Box::new([]) };
    outputs(&mut d, Event::Change { row });
    start_opening_kept(&mut d, fixtures::CHANGE, FIRST_ENTRY, true);
    fixtures::missing_pull_reads(&mut d);
    start_opening(&mut d, fixtures::CHANGE, LATER_ENTRY);
    assert!(fixtures::finish_missing(&mut d).is_empty(), "the kept chain restarts");
    fixtures::answer_next(&mut d, Ok(Answer::Commit(fixtures::HEAD)));
    fixtures::answer_next(&mut d, Ok(Answer::Commit(fixtures::BASE)));
    fixtures::answer_next(&mut d, Ok(Answer::Pull(fixtures::open_pull())));
    fixtures::finish_open(&mut d);
    let finished =
        fixtures::answer_next(&mut d, Ok(Answer::Reviews { reviews: Box::new([]), more: false, cut: false }));
    assert!(
        finished.contains(&Request::ChangeDecision {
            task: fixtures::CHANGE,
            decision: change::Decision::Ready,
            entry: None,
            evidence: None,
        }),
        "the newest wake unkeeps the change: {finished:?}"
    );
}

#[test]
fn a_lower_chain_waiting_without_an_entry_releases_the_later_entry() {
    let (mut d, lower) = reversed_openings(true);
    assert!(ready(&mut d).is_empty(), "the later entry waits for the lower chain");
    let waited = finish_lower(&mut d, lower);
    assert!(
        waited.contains(&Request::ChangeDecision {
            task: fixtures::CHANGE,
            decision: temper_engine_domain_forge_change::Decision::Wait { until: Some(WAIT_UNTIL) },
            entry: None,
            evidence: None,
        }),
        "lower wait: {waited:?}"
    );
    assert!(d.is_ready(), "the committed entry keeps the connector ready");
    answer_entry_clock(&mut d);
    let (_, op) = fixtures::next_call(&mut d);
    let skein_forge::api::Op::Write(skein_forge::api::Write::OpenPull { head, .. }) = op else {
        panic!("later opening")
    };
    assert_eq!(head.as_ref(), LATER_BRANCH);
}

#[test]
fn a_committed_entry_waiting_for_a_lower_chain_can_be_withdrawn() {
    let (mut d, _) = reversed_openings(false);
    let withdrawn = outputs(&mut d, Event::Withdraw { entry: LATER_ENTRY });
    let mut found = false;
    for request in withdrawn {
        if let Request::Outcome {
            entry: LATER_ENTRY,
            task: LATER_CHANGE,
            outcome: skein_forge_client::Outcome::Withdrawn,
            ..
        } = request
        {
            found = true;
        }
    }
    assert!(found, "withdrawal reports the committed entry's outcome");
    assert!(ready(&mut d).is_empty(), "withdrawal releases no forge call");
}

#[test]
fn an_overtaking_open_outcome_restarts_the_fresh_chain() {
    use skein_forge::api::{Answer, Op, Read};
    use temper_engine_domain_forge_change as change;
    let mut d = fixtures::opened_during_chain();
    fixtures::step_change(&mut d);
    let overtaken = fixtures::finish_missing(&mut d);
    let (branch, branch_read) = fixtures::next_call(&mut d);
    fixtures::answer(&mut d, branch, Ok(Answer::Commit(fixtures::HEAD)));
    let (base, base_read) = fixtures::next_call(&mut d);
    fixtures::answer(&mut d, base, Ok(Answer::Commit(fixtures::BASE)));
    let (pull, pull_read) = fixtures::next_call(&mut d);
    fixtures::answer(&mut d, pull, Ok(Answer::Pull(fixtures::open_pull())));
    fixtures::finish_open(&mut d);
    let finished =
        fixtures::answer_next(&mut d, Ok(Answer::Reviews { reviews: Box::new([]), more: false, cut: false }));
    assert_eq!(overtaken.as_ref(), &[]);
    assert_eq!(
        [branch_read, base_read, pull_read],
        [
            Op::Read(Read::Branch { branch: Box::from(b"temper/51".as_slice()) }),
            Op::Read(Read::Branch { branch: Box::from(b"main".as_slice()) }),
            Op::Read(Read::PullFor { head: Box::from(b"temper/51".as_slice()), base: Box::from(b"main".as_slice()) }),
        ]
    );
    let [Request::Save { .. }, Request::Marked { tasks }, Request::ChangeDecision { task, decision, entry, evidence }] =
        finished.as_ref()
    else {
        panic!("one saved decision")
    };
    assert_eq!(tasks.as_ref(), &[fixtures::CHANGE, changes_subject(5), status_subject(5), APPROVALS_SUBJECT]);
    assert_eq!((*task, decision, entry, evidence), (fixtures::CHANGE, &change::Decision::Ready, &None, &None));
    assert_eq!(ready(&mut d).as_ref(), &[]);
}

#[test]
fn an_unwoken_chain_finishes_with_its_single_decision() {
    use temper_engine_domain_forge_change as change;
    let mut d = fixtures::opened_during_chain();
    let finished = fixtures::finish_missing(&mut d);
    let [Request::Save { .. }, Request::Marked { tasks }, decision] = finished.as_ref() else {
        panic!("one saved decision")
    };
    assert_eq!(tasks.as_ref(), &[fixtures::CHANGE]);
    assert_eq!(
        decision,
        &Request::ChangeDecision {
            task: fixtures::CHANGE,
            decision: change::Decision::Wait { until: Some(WAIT_UNTIL) },
            entry: None,
            evidence: None,
        }
    );
    assert_eq!(ready(&mut d).as_ref(), &[]);
}

fn merging_change(protection: Protection, pull_ci: skein_forge::api::Ci) -> Domain {
    first_change(protection, pull_ci, true, false)
}

fn first_change(protection: Protection, pull_ci: skein_forge::api::Ci, mergeable: bool, queued: bool) -> Domain {
    use skein_forge::api::Answer;
    use temper_engine_domain_forge_change as change;
    let mut d = fixtures::connected_domain();
    let mut repository = adopted();
    repository.protection = protection;
    outputs(&mut d, Event::Restore { record: Stored::Repository(repository) });
    outputs(&mut d, Event::Restored { clock: skein_forge_client::RecoveryClock::Monotonic });
    outputs(&mut d, Event::ReadAfresh);
    outputs(&mut d, Event::SettleOutbox);
    let mut row = fixtures::opening_row();
    row.pending = None;
    row.effect = change::EffectResult::None;
    row.pull = Some(1);
    row.change.state = if queued {
        change::State::Queued { head: fixtures::HEAD, ready_since: env().wall }
    } else {
        change::State::First { head: fixtures::HEAD, ready_since: env().wall }
    };
    row.change.owns_turn = !queued;
    outputs(&mut d, Event::Change { row });
    fixtures::step_change(&mut d);
    fixtures::answer_next(&mut d, Ok(Answer::Commit(fixtures::HEAD)));
    fixtures::answer_next(&mut d, Ok(Answer::Commit(fixtures::BASE)));
    let mut pull = fixtures::open_pull();
    pull.ci = pull_ci;
    pull.mergeable = mergeable;
    fixtures::answer_next(&mut d, Ok(Answer::Pull(pull)));
    d
}

fn merge_after_base_reads(d: &mut Domain) {
    use skein_forge::api::{Answer, Check, Op, Read};
    use temper_engine_domain_forge_change as change;
    let (call, op) = fixtures::next_call(d);
    assert_eq!(op, Op::Read(Read::Checks { commit: fixtures::BASE }));
    fixtures::answer(d, call, Ok(fixtures::checks(Check::Passed)));
    fixtures::answer_next(d, Ok(Answer::Commits { commits: Box::new([]), more: false }));
    let finished = fixtures::answer_next(d, Ok(Answer::Reviews { reviews: Box::new([]), more: false, cut: false }));
    let mut merging = false;
    for request in &finished {
        if let Request::ChangeDecision {
            decision: change::Decision::Effect(change::Effect::Merge { head, .. }), ..
        } = request
        {
            assert_eq!(*head, fixtures::HEAD);
            merging = true;
        }
    }
    assert!(merging, "merge at the checked head: {finished:?}");
}

#[test]
fn passed_pull_ci_reads_the_head_contexts_for_its_landed_result() {
    use skein_forge::api::{Answer, Check, Ci, Op, Read};
    use temper_engine_domain_forge_change as change;
    for queued in [false, true] {
        let mut d = first_change(Protection::Absent, Ci::Passed, true, queued);
        let (call, op) = fixtures::next_call(&mut d);
        assert_eq!(op, Op::Read(Read::Checks { commit: fixtures::HEAD }));
        fixtures::answer(&mut d, call, Ok(fixtures::checks(Check::Passed)));
        let (call, op) = fixtures::next_call(&mut d);
        assert_eq!(op, Op::Read(Read::Checks { commit: fixtures::BASE }));
        fixtures::answer(&mut d, call, Ok(fixtures::checks(Check::Passed)));
        fixtures::answer_next(&mut d, Ok(Answer::Commits { commits: Box::new([]), more: false }));
        let finished =
            fixtures::answer_next(&mut d, Ok(Answer::Reviews { reviews: Box::new([]), more: false, cut: false }));
        let mut proposed = None;
        for request in &finished {
            if let Request::ChangeDecision {
                decision: change::Decision::Effect(change::Effect::Merge { head, .. }),
                entry: Some(entry),
                evidence: Some(evidence),
                ..
            } = request
            {
                assert_eq!(*head, fixtures::HEAD);
                proposed = Some((*entry, evidence.prior.clone()));
            }
        }
        let (entry, prior) = proposed.expect("merge at the checked head");
        outputs(&mut d, Event::VetoChange { task: fixtures::CHANGE, entry, prior });
        let row = d.change(fixtures::CHANGE).expect("change waiting for approval");
        assert_eq!(
            row.change.state,
            change::State::First { head: fixtures::HEAD, ready_since: env().wall },
            "an approval-vetoed merge keeps its proposed state, queued: {queued}"
        );
        assert!(row.change.owns_turn, "the approval wait keeps its turn, queued: {queued}");
        assert_eq!(row.pending, None);
        assert_eq!(row.effect, change::EffectResult::None);
        assert!(!d.entries.contains_key(&entry), "a veto removes the uncommitted merge");
    }
}

#[test]
fn a_persons_request_is_taken_by_one_step_and_not_restored() {
    use skein_forge::api::{Answer, Ci};
    use temper_engine_domain_forge_change as change;
    for restart in [false, true] {
        let mut d = first_change(Protection::Absent, Ci::Passed, true, false);
        let row = d.change(fixtures::CHANGE).expect("first change").clone();
        let requested = outputs(&mut d, Event::ReviewRequested { task: fixtures::CHANGE, head: fixtures::BASE });
        assert!(requested.is_empty(), "no durable record carries the request");
        assert!(d.procedure_wakes.contains_key(&fixtures::CHANGE));
        if restart {
            // The same durable change starts without the volatile request.
            d = first_change(Protection::Absent, Ci::Passed, true, false);
        }
        fixtures::finish_open(&mut d);
        let finished =
            fixtures::answer_next(&mut d, Ok(Answer::Reviews { reviews: Box::new([]), more: false, cut: false }));
        let expected = if restart {
            change::Decision::Effect(change::Effect::Merge { head: fixtures::HEAD, base: fixtures::BASE })
        } else {
            change::Decision::Delegate(change::Delegate::Repair(change::Repair::Review { head: fixtures::BASE }))
        };
        let mut decided = false;
        for request in &finished {
            if let Request::ChangeDecision { decision, .. } = request {
                assert_eq!(*decision, expected, "restart {restart}");
                decided = true;
            }
        }
        assert!(decided, "{restart}: {finished:?}");
        if !restart {
            // Put the procedure at First again: the previous step already took its input.
            d.changes.insert(fixtures::CHANGE, row).expect("retained change");
            fixtures::step_change(&mut d);
            fixtures::answer_next(&mut d, Ok(Answer::Commit(fixtures::HEAD)));
            fixtures::answer_next(&mut d, Ok(Answer::Commit(fixtures::BASE)));
            fixtures::answer_next(&mut d, Ok(Answer::Pull(fixtures::open_pull())));
            fixtures::finish_open(&mut d);
            let finished =
                fixtures::answer_next(&mut d, Ok(Answer::Reviews { reviews: Box::new([]), more: false, cut: false }));
            let mut merging = false;
            for request in &finished {
                if let Request::ChangeDecision {
                    decision: change::Decision::Effect(change::Effect::Merge { .. }),
                    ..
                } = request
                {
                    merging = true;
                }
            }
            assert!(merging, "a request is read once: {finished:?}");
        }
    }
}

#[test]
fn required_head_checks_ignore_a_failed_non_required_row() {
    use skein_forge::api::{Answer, Check, Op, Read};
    let protection = Protection::Rule(skein_forge::api::Protection {
        queue: false,
        signatures: false,
        linear: false,
        code_owners: false,
        bypass: skein_forge::api::Bypass::Never,
        branch: Box::from(b"main".as_slice()),
        contexts: Box::new([Box::from(b"ci".as_slice())]),
        approvals: 0,
        dismiss_stale: false,
    });
    // The failed lint row makes the pull's aggregate fail, too.
    let mut d = merging_change(protection, skein_forge::api::Ci::Failed);
    let (call, op) = fixtures::next_call(&mut d);
    assert_eq!(op, Op::Read(Read::Checks { commit: fixtures::HEAD }));
    let Answer::Checks(rows) = fixtures::checks(Check::Passed) else { unreachable!() };
    let mut lint = rows[0].clone();
    lint.context = Box::from(b"lint".as_slice());
    lint.check = Check::Failed;
    fixtures::answer(&mut d, call, Ok(Answer::Checks(Box::new([lint, rows[0].clone()]))));
    merge_after_base_reads(&mut d);
}

fn named_pull_verdict(number: u64) -> Box<[Request]> {
    use skein_forge::api::{Answer, Ci, Kind, Op, Read, State, Summary};
    let mut d = fixtures::connected_domain();
    outputs(&mut d, Event::Restore { record: Stored::Repository(adopted()) });
    let mut row = fixtures::opening_row();
    row.pull = Some(1);
    outputs(&mut d, Event::Change { row: row.clone() });
    // Another change in the same repository must not wake with this pull.
    row.task = LATER_CHANGE;
    row.change.task = LATER_CHANGE;
    row.pull = Some(2);
    row.branch = Box::from(LATER_BRANCH);
    outputs(&mut d, Event::Change { row });
    outputs(&mut d, Event::Restored { clock: skein_forge_client::RecoveryClock::Monotonic });
    outputs(&mut d, Event::ReadAfresh);
    outputs(
        &mut d,
        Event::Names {
            task: fixtures::CHANGE,
            resources: Box::new([Name { forge: REPO.forge, repository: REPO.repository, what: What::Pull(number) }]),
        },
    );
    let mut seen = Queue::uncapped();
    // At most a repository listing, the item, and the pull itself.
    for _ in 0_u32..3 {
        let (call, op) = fixtures::next_call(&mut d);
        if op == Op::Read(Read::Pull { number, ci: true }) {
            let mut pull = fixtures::open_pull();
            pull.number = number;
            pull.ci = Ci::Passed;
            for request in fixtures::answer(&mut d, call, Ok(Answer::Pull(pull))) {
                seen.push(request);
            }
            for request in ready(&mut d) {
                seen.push(request);
            }
            let mut values = skein_lib::List::with_capacity(seen.len());
            for _ in 0..seen.len() {
                values.push(seen.pop().expect("counted request")).expect("sized from observed passes");
            }
            return values.into_boxed();
        }
        let answer = if let Op::Read(Read::Items { .. }) = op {
            Answer::Items { cut: false, items: Box::new([]), more: false, now: env().now }
        } else {
            let Op::Read(Read::Item { number: read_number, .. }) = op else { panic!("{op:?}") };
            assert_eq!(read_number, number);
            Answer::Item {
                item: Summary {
                    labels: Box::new([]),
                    number,
                    kind: Kind::Pull,
                    state: State::Open,
                    author: 7,
                    key: None,
                    title: Box::new([]),
                    body: Box::new([]),
                    updated: env().now,
                },
                comments: Box::new([]),
                more: false,
            }
        };
        for request in fixtures::answer(&mut d, call, Ok(answer)) {
            seen.push(request);
        }
    }
    panic!("the named pull must be read");
}

#[test]
fn a_named_pulls_ci_changed_wakes_only_its_own_change() {
    let seen = named_pull_verdict(1);
    let mut wakes = 0_u32;
    for request in &seen {
        if let Request::WakeProcedure { task } = request {
            assert_eq!(*task, fixtures::CHANGE, "CI Changed: {seen:?}");
            wakes += 1;
        }
    }
    assert_eq!(wakes, 1, "one owning change wakes: {seen:?}");
}

#[test]
fn a_named_pulls_changed_without_an_owning_change_wakes_nothing() {
    let seen = named_pull_verdict(99);
    for request in &seen {
        if let Request::WakeProcedure { .. } = request {
            panic!("unowned Changed: {seen:?}");
        }
    }
}

#[test]
fn a_settling_change_relies_on_its_own_pull_watch_but_keeps_its_deadline() {
    use skein_forge::api::{Answer, Ci, Op, Read};
    use temper_engine_domain_forge_change as change;
    for (watch, compared, delay) in [
        (Some(1), true, LIMITS.client.settle),
        (Some(1), false, LIMITS.client.hinted),
        (Some(2), true, LIMITS.client.hinted),
        (None, true, LIMITS.client.hinted),
    ] {
        let mut d = first_change(Protection::Absent, Ci::Passed, false, false);
        fixtures::answer_next(&mut d, Ok(fixtures::checks(skein_forge::api::Check::Passed)));
        fixtures::answer_next(&mut d, Ok(fixtures::checks(skein_forge::api::Check::Passed)));
        fixtures::answer_next(
            &mut d,
            if compared {
                Ok(Answer::Commits { commits: Box::new([]), more: false })
            } else {
                Err(skein_forge::api::Error::Missing)
            },
        );
        let (reviews, op) = fixtures::next_call(&mut d);
        assert_eq!(op, Op::Read(Read::Reviews { number: 1, page: 1, skip: 0 }));
        if let Some(number) = watch {
            outputs(
                &mut d,
                Event::Names {
                    task: fixtures::CHANGE,
                    resources: Box::new([Name {
                        forge: REPO.forge,
                        repository: REPO.repository,
                        what: What::Pull(number),
                    }]),
                },
            );
        }
        let finished =
            fixtures::answer(&mut d, reviews, Ok(Answer::Reviews { reviews: Box::new([]), more: false, cut: false }));
        let until = Wall::from_nanos(env().wall.as_nanos() + delay.as_nanos());
        assert!(
            finished.contains(&Request::ChangeDecision {
                task: fixtures::CHANGE,
                decision: change::Decision::Wait { until: Some(until) },
                entry: None,
                evidence: None,
            }),
            "watch {watch:?}, compared {compared}: the pull wake replaces complete gathers, with a deadline: {finished:?}"
        );
    }
}

// Exercise admission, retained title and body, and the operation handed to the client.
fn projected_words(explicit_title: &[u8], words: &[u8]) -> skein_forge::api::Write {
    projected_words_at(explicit_title, words, CLIENT.op_bytes)
}

fn projected_words_at(explicit_title: &[u8], words: &[u8], op_bytes: u32) -> skein_forge::api::Write {
    let mut configured = env();
    configured.limits.client.op_bytes = op_bytes;
    let mut d = domain_with_limits(&configured.limits);
    fixtures::connect(&mut d);
    outputs(&mut d, Event::Restore { record: Stored::Repository(adopted()) });
    outputs(&mut d, Event::Restored { clock: skein_forge_client::RecoveryClock::Monotonic });
    outputs(&mut d, Event::ReadAfresh);
    outputs(&mut d, Event::SettleOutbox);
    let mut row = fixtures::opening_row();
    row.pending = None;
    row.effect = temper_engine_domain_forge_change::EffectResult::None;
    row.change.state = temper_engine_domain_forge_change::State::Opening { requested: false };
    row.title = explicit_title.into();
    row.body = words.into();
    let registered = outputs(&mut d, Event::Change { row });
    for request in &registered {
        if let Request::Refused { .. } | Request::ResourceRefused { .. } = request {
            panic!("change registration refused: {request:?}");
        }
    }
    assert_eq!(d.change(fixtures::CHANGE).unwrap().title.as_ref(), explicit_title);
    assert_eq!(d.change(fixtures::CHANGE).unwrap().body.as_ref(), words);
    start_opening(&mut d, fixtures::CHANGE, FIRST_ENTRY);
    fixtures::missing_pull_reads(&mut d);
    let green = fixtures::checks(skein_forge::api::Check::Passed);
    fixtures::answer_next(&mut d, Ok(green.clone()));
    fixtures::answer_next(&mut d, Ok(green));
    let (call, _) = fixtures::next_call(&mut d);
    let mut built = Queue::uncapped();
    // The final fresh read builds the effect using this row's operation bound.
    step(
        &mut d,
        &configured,
        Event::Client(skein_forge_client::Event::Answered {
            free: 0,
            rate: None,
            call,
            cost: 1,
            result: Ok(skein_forge::api::Answer::Commits { commits: Box::new([]), more: false }),
        }),
        &mut built,
    );
    let mut entry_built = false;
    for _ in 0..built.len() {
        if let Request::ChangeDecision { entry: Some(entry), .. } = built.pop().expect("counted output") {
            entry_built |= entry == FIRST_ENTRY;
        }
    }
    assert!(entry_built);
    step(&mut d, &configured, Event::Committed { entry: FIRST_ENTRY }, &mut Queue::uncapped());
    let (call, _) = fixtures::next_call(&mut d);
    step(
        &mut d,
        &configured,
        Event::Client(skein_forge_client::Event::Answered {
            free: 0,
            rate: None,
            call,
            cost: 1,
            result: Ok(skein_forge::api::Answer::Items {
                cut: false,
                items: Box::new([]),
                more: false,
                now: configured.now,
            }),
        }),
        &mut Queue::uncapped(),
    );
    let mut calls = Queue::uncapped();
    resume(&mut d, &configured, &mut calls);
    for _ in 0..calls.len() {
        if let Request::Call { op, .. } = calls.pop().expect("counted output") {
            let skein_forge::api::Op::Write(write) = op else { panic!("open write") };
            assert!(skein_forge::measure::write(&write).unwrap() <= u64::from(op_bytes));
            return write;
        }
    }
    panic!("projected forge call");
}

#[test]
fn change_words_past_operation_bound_register_whole_and_open_with_marked_cuts() {
    let words = "界".repeat(700);
    let skein_forge::api::Write::OpenPull { title, body, .. } = projected_words(words.as_bytes(), words.as_bytes())
    else {
        panic!("open pull")
    };
    assert!(title.len() <= 256);
    let title_prefix = title.strip_suffix("…".as_bytes()).expect("marked title");
    assert_eq!(title_prefix, "界".repeat(title_prefix.len() / 3).as_bytes());
    let body_prefix =
        body.strip_suffix("…\nThe change's full words are on its task in temper.".as_bytes()).expect("marked body");
    assert_eq!(body_prefix, "界".repeat(body_prefix.len() / 3).as_bytes());
}

#[test]
fn pull_bodies_fit_the_forge_bound_without_cutting_retained_words() {
    const OP_BYTES: u32 = 131_070; // call-room's reference payload less its separator.
    const BODY_BYTES: usize = 65_536; // temper/docs/design/domain/forge/change/README.md, 1.
    const MARKER: &[u8] = "…\nThe change's full words are on its task in temper.".as_bytes();
    // All three rows, including the reference-sized body, measured 5.9 ms of CPU together.
    for (bytes, cut) in [(BODY_BYTES, false), (BODY_BYTES + 1, true), (131_070, true)] {
        let mut words = skein_lib::bytes::zeroed(bytes);
        words.fill(b'r');
        let skein_forge::api::Write::OpenPull { body, .. } = projected_words_at(b"Small fix", &words, OP_BYTES) else {
            panic!("open pull")
        };
        assert_eq!(body.len(), BODY_BYTES, "{bytes} supplied bytes");
        if cut {
            let prefix = body.strip_suffix(MARKER).expect("marked body cut");
            assert_eq!(prefix, &words[..BODY_BYTES - MARKER.len()], "{bytes} supplied bytes");
        } else {
            assert_eq!(body, words, "{bytes} supplied bytes");
        }
    }
}

#[test]
fn change_words_within_operation_bound_keep_the_explicit_title() {
    let words = b"Small fix\nKeep the full specification.";
    let skein_forge::api::Write::OpenPull { title, body, .. } = projected_words(b"Small fix", words) else {
        panic!("open pull")
    };
    assert_eq!(title.as_ref(), b"Small fix");
    assert_eq!(body.as_ref(), words);
}

#[test]
fn change_paragraph_keeps_title_punctuation_and_whole_body() {
    let words = b"Add CSV export to the widget. Read src/main.rs to see how the widget works. Keep the export simple.";
    let skein_forge::api::Write::OpenPull { title, body, .. } = projected_words(b"Add CSV v1.2. export!", words) else {
        panic!("open pull")
    };
    assert_eq!(title.as_ref(), b"Add CSV v1.2. export!");
    assert_eq!(body.as_ref(), words);
}

#[test]
fn change_long_explicit_title_fits_the_byte_bound() {
    const WORDS: usize = 50; // Eight bytes per word exceed the pull's 256-byte title bound.
    let words = "Exports ".repeat(WORDS) + "widgets. Keep the full specification.";
    let skein_forge::api::Write::OpenPull { title, .. } = projected_words(words.as_bytes(), words.as_bytes()) else {
        panic!("open pull")
    };
    assert_eq!(title.len(), 256);
    assert_eq!(title.strip_suffix("…".as_bytes()).expect("marked cut"), &words.as_bytes()[..253]);
}

#[test]
fn oversized_gate_remarks_are_marked_before_they_are_saved() {
    use temper_engine_domain_forge_change as change;
    let mut d = fixtures::connected_domain();
    outputs(&mut d, Event::Restore { record: Stored::Repository(adopted()) });
    let mut row = fixtures::opening_row();
    row.change.last_head = Some(fixtures::HEAD);
    row.delegate = Some((52, change::Delegate::Gate { number: 1, head: fixtures::HEAD }));
    outputs(&mut d, Event::Change { row });
    let words = "Please include the pull request's number and address. ".repeat(100);
    outputs(
        &mut d,
        Event::DelegateResult {
            task: fixtures::CHANGE,
            child: 52,
            status: change::Status::Failed,
            no_change: false,
            failed: false,
            cancelled: false,
            words: words.as_bytes().into(),
        },
    );
    let saved = &d.change(fixtures::CHANGE).expect("test fixture").gate_remarks[0].words;
    let mut kept = None;
    for (at, pair) in saved.windows(2).enumerate() {
        if pair == b"\n[" {
            kept = Some(at);
        }
    }
    let kept = kept.expect("marked stored cut");
    assert!(words.as_bytes().starts_with(&saved[..kept]));
    let count = skein_lib::Decimal::of(u64::try_from(words.len() - kept).expect("test fixture"));
    assert_eq!(&saved[kept + 2..saved.len() - b" bytes cut]\n".len()], count.as_bytes());
    assert!(saved.ends_with(b" bytes cut]\n"));
    assert!(saved.len() <= usize::try_from(LIMITS.client.answer_bytes).expect("test fixture"));
}

#[test]
fn awaiting_a_producer_title_keeps_the_changes_stall_deadline() {
    use temper_engine_domain_forge_change as change;
    for expired in [false, true] {
        let mut d = fixtures::connected_domain();
        outputs(&mut d, Event::Restore { record: Stored::Repository(adopted()) });
        outputs(&mut d, Event::Restored { clock: skein_forge_client::RecoveryClock::Monotonic });
        outputs(&mut d, Event::ReadAfresh);
        outputs(&mut d, Event::SettleOutbox);
        let mut row = fixtures::opening_row();
        row.pending = None;
        row.effect = change::EffectResult::None;
        row.title = Box::new([]);
        row.delegate = Some((52, change::Delegate::Produce));
        row.change.state = change::State::Producing { requested: true };
        row.change.since = if expired {
            Wall::from_nanos(env().wall.as_nanos().saturating_sub(LIMITS.change_policy.stall.as_nanos()))
        } else {
            env().wall
        };
        outputs(&mut d, Event::Change { row });
        fixtures::missing_pull(&mut d);
        let finished = fixtures::finish_missing(&mut d);
        let decision = if expired {
            change::Decision::Hold(change::Hold::Stalled)
        } else {
            change::Decision::Wait { until: Some(WAIT_UNTIL) }
        };
        assert!(
            finished.contains(&Request::ChangeDecision {
                task: fixtures::CHANGE,
                decision,
                entry: None,
                evidence: None,
            }),
            "title wait: {finished:?}"
        );
        if expired {
            assert_eq!(
                d.change(fixtures::CHANGE).expect("change").change.state,
                change::State::Held {
                    was: Box::new(change::State::Producing { requested: true }),
                    why: change::Hold::Stalled,
                }
            );
        }
    }
}

#[test]
fn only_the_active_successful_repair_supplies_an_account() {
    use temper_engine_domain_forge_change as change;
    for (child, status, no_change, failed, cancelled, retained) in [
        (52, change::Status::Passed, false, false, false, true),
        (52, change::Status::Failed, false, true, false, false),
        (52, change::Status::Failed, false, false, true, false),
        (52, change::Status::Passed, true, false, false, false),
        (52, change::Status::Passed, false, true, false, false),
        (52, change::Status::Passed, false, false, true, false),
        (53, change::Status::Passed, false, false, false, false),
    ] {
        let mut d = fixtures::connected_domain();
        outputs(&mut d, Event::Restore { record: Stored::Repository(adopted()) });
        let mut row = fixtures::opening_row();
        row.delegate = Some((52, change::Delegate::Repair(change::Repair::Ci)));
        row.result.repaired(change::Repair::Ci, Some(fixtures::HEAD), LIMITS.client.answer_bytes);
        row.report = b"exports CSV".as_slice().into();
        outputs(&mut d, Event::Restore { record: Stored::Change(Box::new(row)) });
        outputs(
            &mut d,
            Event::ProducedTitle {
                task: fixtures::CHANGE,
                child,
                title: b"repair title".as_slice().into(),
                report: b"ignored title account".as_slice().into(),
            },
        );
        outputs(
            &mut d,
            Event::DelegateResult {
                task: fixtures::CHANGE,
                child,
                status,
                no_change,
                failed,
                cancelled,
                words: b"Adds changes/2.md".as_slice().into(),
            },
        );
        let row = d.change(fixtures::CHANGE).expect("retained change");
        let account =
            skein_lib::bytes::find(&row.result.repairs.bytes, b"successful repair task 52: Adds changes/2.md");
        assert_eq!(account.is_some(), retained);
        assert_eq!(row.title.as_ref(), b"Small fix");
        assert_eq!(row.report.as_ref(), b"exports CSV");
    }
}

fn delegate_waiting(
    state: temper_engine_domain_forge_change::State,
    kind: temper_engine_domain_forge_change::Delegate,
) -> Domain {
    use temper_engine_domain_forge_change as change;
    let mut d = fixtures::connected_domain();
    outputs(&mut d, Event::Restore { record: Stored::Repository(adopted()) });
    outputs(&mut d, Event::Restored { clock: skein_forge_client::RecoveryClock::Monotonic });
    outputs(&mut d, Event::ReadAfresh);
    outputs(&mut d, Event::SettleOutbox);
    let mut row = fixtures::opening_row();
    row.pending = None;
    row.effect = change::EffectResult::None;
    row.delegate = Some((52, kind));
    row.delegate_status = change::Status::Pending;
    row.title = Box::new([]);
    row.change.state = state;
    row.change.since = Wall::EPOCH;
    outputs(&mut d, Event::Change { row });
    d
}

fn producer_terminal(d: &mut Domain, no_change: bool) -> Box<[Request]> {
    outputs(
        d,
        Event::DelegateResult {
            task: fixtures::CHANGE,
            child: 52,
            status: temper_engine_domain_forge_change::Status::Passed,
            no_change,
            failed: false,
            cancelled: false,
            words: b"Nothing applies".as_slice().into(),
        },
    )
}

fn waiting_step(live: bool, released: bool) -> Event {
    use temper_engine_domain_forge_change as change;
    Event::StepChange {
        task: fixtures::CHANGE,
        entry: fixtures::OPEN + 1,
        heard: change::Heard {
            review_request: None,
            start_conflict: None,
            lacking: None,
            spend: None,
            delegate: change::Status::Pending,
            delegate_running: false,
            delegate_live: live,
            effect: change::EffectResult::None,
            released,
            cancelled: false,
        },
        gates: Box::new([]),
        queue_repair_active: false,
        kept: false,
        merge_job: false,
        pending: Box::new([]),
    }
}

#[test]
fn a_live_producer_title_wait_rearms_the_stall() {
    use temper_engine_domain_forge_change as change;
    let mut d = delegate_waiting(change::State::Producing { requested: true }, change::Delegate::Produce);
    // Refresh a step already reading, so restart must retain the live-task fact.
    fixtures::step_change(&mut d);
    outputs(&mut d, waiting_step(true, false));
    fixtures::missing_pull_reads(&mut d);
    fixtures::finish_missing(&mut d);
    // The overlapping wake restarts its reads before deciding.
    fixtures::missing_pull_reads(&mut d);
    let finished = fixtures::finish_missing(&mut d);
    assert!(
        finished.contains(&Request::ChangeDecision {
            task: fixtures::CHANGE,
            decision: change::Decision::Wait { until: Some(WAIT_UNTIL) },
            entry: None,
            evidence: None
        }),
        "{finished:?}"
    );
    assert_eq!(d.change(fixtures::CHANGE).expect("change").change.since, env().wall);
    assert_eq!(d.change(fixtures::CHANGE).expect("change").change.state, change::State::Producing { requested: true });
}

#[test]
fn a_delegate_result_starts_the_stall_at_its_end_but_a_gate_does_not() {
    use temper_engine_domain_forge_change as change;
    for kind in [
        change::Delegate::Produce,
        change::Delegate::Repair(change::Repair::Ci),
        change::Delegate::Resolve { base: fixtures::BASE },
        change::Delegate::Merge { base: fixtures::BASE, from: Some(fixtures::HEAD) },
        change::Delegate::Gate { number: 1, head: fixtures::HEAD },
    ] {
        let mut d = delegate_waiting(change::State::Producing { requested: true }, kind);
        producer_terminal(&mut d, false);
        let expected =
            if kind == (change::Delegate::Gate { number: 1, head: fixtures::HEAD }) { Wall::EPOCH } else { env().wall };
        assert_eq!(d.change(fixtures::CHANGE).expect("change").change.since, expected, "{kind:?}");
    }
    let mut d = delegate_waiting(change::State::Producing { requested: true }, change::Delegate::Produce);
    producer_terminal(&mut d, false);
    fixtures::missing_pull(&mut d);
    let finished = fixtures::finish_missing(&mut d);
    assert!(finished.contains(&Request::ChangeDecision {
        task: fixtures::CHANGE,
        decision: change::Decision::Wait { until: Some(WAIT_UNTIL) },
        entry: None,
        evidence: None
    }));
    fixtures::missing_pull(&mut d);
    fixtures::answer_next(&mut d, Ok(fixtures::checks(skein_forge::api::Check::Passed)));
    fixtures::answer_next(&mut d, Ok(fixtures::checks(skein_forge::api::Check::Passed)));
    let (call, _) = fixtures::next_call(&mut d);
    let mut late = env();
    late.wall = WAIT_UNTIL;
    late.now = Time::from_nanos(WAIT_UNTIL.as_nanos());
    let mut out = Queue::uncapped();
    step(
        &mut d,
        &late,
        Event::Client(skein_forge_client::Event::Answered {
            call,
            free: 0,
            rate: None,
            cost: 1,
            result: Ok(skein_forge::api::Answer::Commits { commits: Box::new([]), more: false }),
        }),
        &mut out,
    );
    assert_eq!(
        d.change(fixtures::CHANGE).expect("change").change.state,
        change::State::Held { was: Box::new(change::State::Producing { requested: true }), why: change::Hold::Stalled }
    );
}

#[test]
fn a_result_ends_only_a_stall_at_its_delegate_wait() {
    use temper_engine_domain_forge_change as change;
    for (was, kind, ends) in [
        (change::State::Producing { requested: true }, change::Delegate::Produce, true),
        (
            change::State::Repairing { requested: true, why: change::Repair::Ci },
            change::Delegate::Repair(change::Repair::Ci),
            true,
        ),
        (
            change::State::Resolving { requested: true, base: fixtures::BASE },
            change::Delegate::Resolve { base: fixtures::BASE },
            true,
        ),
        (
            change::State::Resolving { requested: true, base: fixtures::BASE },
            change::Delegate::Merge { base: fixtures::BASE, from: None },
            true,
        ),
        (change::State::Checking { head: fixtures::HEAD }, change::Delegate::Produce, false),
        (
            change::State::Gating { head: fixtures::HEAD, asked: Box::new([1]) },
            change::Delegate::Gate { number: 1, head: fixtures::HEAD },
            false,
        ),
        (change::State::Producing { requested: false }, change::Delegate::Produce, false),
    ] {
        let held = change::State::Held { was: Box::new(was.clone()), why: change::Hold::Stalled };
        let mut d = delegate_waiting(held.clone(), kind);
        let result = producer_terminal(&mut d, false);
        assert_eq!(d.change(fixtures::CHANGE).expect("change").change.state, if ends { was } else { held });
        assert_eq!(
            result.contains(&Request::ChangeDecision {
                task: fixtures::CHANGE,
                decision: change::Decision::EndHold,
                entry: None,
                evidence: None
            }),
            ends
        );
        if ends {
            assert!(ready(&mut d).contains(&Request::WakeProcedure { task: fixtures::CHANGE }));
        }
    }
}

#[test]
fn a_stalled_producers_title_and_result_open_its_pull_without_release() {
    use temper_engine_domain_forge_change as change;
    let mut d = delegate_waiting(
        change::State::Held { was: Box::new(change::State::Producing { requested: true }), why: change::Hold::Stalled },
        change::Delegate::Produce,
    );
    outputs(
        &mut d,
        Event::ProducedTitle {
            task: fixtures::CHANGE,
            child: 52,
            title: b"Delivered fix".as_slice().into(),
            report: b"Whole report".as_slice().into(),
        },
    );
    producer_terminal(&mut d, false);
    assert_eq!(d.change(fixtures::CHANGE).expect("change").title.as_ref(), b"Delivered fix");
    assert_eq!(d.change(fixtures::CHANGE).expect("change").report.as_ref(), b"Whole report");
    fixtures::missing_pull(&mut d);
    let finished = fixtures::finish_missing(&mut d);
    assert!(opens_pull(&finished), "{finished:?}");
}

#[test]
fn a_no_change_result_ends_a_stalled_producer_with_its_words() {
    use temper_engine_domain_forge_change as change;
    let mut d = delegate_waiting(
        change::State::Held { was: Box::new(change::State::Producing { requested: true }), why: change::Hold::Stalled },
        change::Delegate::Produce,
    );
    let result = producer_terminal(&mut d, true);
    assert!(result.contains(&Request::ChangeDecision {
        task: fixtures::CHANGE,
        decision: change::Decision::EndHold,
        entry: None,
        evidence: None
    }));
    assert_eq!(
        d.change(fixtures::CHANGE).expect("change").change.state,
        change::State::NoChange { words: b"Nothing applies".as_slice().into() }
    );
    let finished = outputs(&mut d, waiting_step(false, false));
    assert!(finished.contains(&Request::ChangeDecision {
        task: fixtures::CHANGE,
        decision: change::Decision::NoChange,
        entry: None,
        evidence: None
    }));
}

#[test]
fn a_questions_hold_keeps_the_result_until_release_and_says_no_change_words() {
    use temper_engine_domain_forge_change as change;
    for no_change in [false, true] {
        let held = change::State::Held {
            was: Box::new(change::State::Producing { requested: true }),
            why: change::Hold::Questions,
        };
        let mut d = delegate_waiting(held.clone(), change::Delegate::Produce);
        outputs(
            &mut d,
            Event::ProducedTitle {
                task: fixtures::CHANGE,
                child: 52,
                title: b"Delivered fix".as_slice().into(),
                report: b"Whole report".as_slice().into(),
            },
        );
        let result = producer_terminal(&mut d, no_change);
        assert!(!result.contains(&Request::ChangeDecision {
            task: fixtures::CHANGE,
            decision: change::Decision::EndHold,
            entry: None,
            evidence: None
        }));
        let row = d.change(fixtures::CHANGE).expect("change");
        assert_eq!(row.change.state, held);
        assert_eq!(row.title.as_ref(), b"Delivered fix");
        assert_eq!(row.report.as_ref(), b"Whole report");
        if no_change {
            assert_eq!(row.producer_failure.as_deref(), Some(b"Nothing applies".as_slice()));
        } else {
            outputs(&mut d, waiting_step(false, true));
            fixtures::missing_pull_reads(&mut d);
            let finished = fixtures::finish_missing(&mut d);
            assert!(opens_pull(&finished), "{finished:?}");
        }
    }
}

fn opens_pull(requests: &[Request]) -> bool {
    use temper_engine_domain_forge_change as change;
    for request in requests {
        if let Request::ChangeDecision { decision: change::Decision::Effect(change::Effect::Open), .. } = request {
            return true;
        }
    }
    false
}

// A settled update is interpreted by the top (temper/docs/design/domain/forge/README.md, 6).
fn updating_entry(preparing: bool) -> (Domain, skein_forge_client::Entry) {
    use skein_forge::api::{UpdateStyle, Write};
    use temper_engine_domain_forge_change as change;
    let mut d = fixtures::connected_domain();
    outputs(&mut d, Event::Restore { record: Stored::Repository(adopted()) });
    outputs(&mut d, Event::Restored { clock: skein_forge_client::RecoveryClock::Monotonic });
    outputs(&mut d, Event::ReadAfresh);
    outputs(&mut d, Event::SettleOutbox);
    let mut row = fixtures::opening_row();
    row.pull = Some(1);
    row.change.updates = 1;
    row.change.state = if preparing {
        row.change.way = change::Way::Turn;
        change::State::Preparing { from: fixtures::HEAD, main: fixtures::BASE, ready_since: env().wall }
    } else {
        change::State::Updating { from: fixtures::HEAD, base: fixtures::BASE, ready_since: env().wall }
    };
    let mut entry = status(fixtures::OPEN, fixtures::CHANGE);
    entry.effect.write =
        Write::Update { number: 1, style: if preparing { UpdateStyle::Rebase } else { UpdateStyle::Merge } };
    entry.effect.condition = skein_forge_client::Condition::Update { head: fixtures::HEAD, base: fixtures::BASE };
    outputs(&mut d, Event::Change { row });
    outputs(&mut d, Event::Enqueue { entry: entry.clone() });
    (d, entry)
}

fn settled_change(d: &mut Domain, outcome: skein_forge_client::Outcome) -> Box<[Request]> {
    let mut out = Queue::uncapped();
    domain::change_outcome(d, &env(), fixtures::OPEN, outcome, &mut out);
    let mut requests = skein_lib::List::uncapped();
    for _ in 0..out.len() {
        requests.push(out.pop().expect("counted output")).expect("uncapped test output");
    }
    requests.into_boxed()
}

fn assert_settled_account(d: &Domain, entry: &skein_forge_client::Entry, outcome: skein_forge_client::Outcome) {
    let row = d.change(fixtures::CHANGE).expect("settled change");
    assert_eq!(row.pending, None);
    assert_eq!(
        row.effect_account,
        Some(EffectAccount {
            entry: entry.number,
            words: effect_words::account(entry, Some(&adopted()), outcome).expect("failure's words"),
        })
    );
}

#[test]
fn an_update_lost_or_overtaken_is_let_go_with_its_account() {
    use skein_forge::api::Error;
    use skein_forge_client::Outcome;
    use temper_engine_domain_forge_change::EffectResult;
    for preparing in [false, true] {
        for error in [Error::Timeout, Error::Unavailable, Error::Stale] {
            let (mut d, entry) = updating_entry(preparing);
            settled_change(&mut d, Outcome::Failed(error));
            let row = d.change(fixtures::CHANGE).expect("settled update");
            assert_eq!(row.effect, EffectResult::None, "preparing={preparing}, {error:?}");
            assert!(row.effect_account.as_ref().expect("account").words.starts_with(b"update pull request: "));
            assert_settled_account(&d, &entry, Outcome::Failed(error));
        }
    }
}

fn decide_update_again(d: &mut Domain, preparing: bool, updates: u32) {
    use skein_forge::api::{Answer, CommitMessage, Op, Read};
    use temper_engine_domain_forge_change as change;
    fixtures::step_change(d);
    fixtures::answer_next(d, Ok(Answer::Commit(fixtures::HEAD)));
    fixtures::answer_next(d, Ok(Answer::Commit(fixtures::BASE)));
    fixtures::answer_next(d, Ok(Answer::Pull(fixtures::open_pull())));
    fixtures::answer_next(d, Ok(fixtures::checks(skein_forge::api::Check::Passed)));
    let (call, op) = fixtures::next_call(d);
    assert_eq!(op, Op::Read(Read::Commits { before: fixtures::HEAD, after: fixtures::BASE, page: 1 }));
    fixtures::answer(
        d,
        call,
        Ok(Answer::Commits {
            commits: Box::new([CommitMessage {
                id: fixtures::BASE,
                parents: Box::new([fixtures::HEAD]),
                message: b"base ahead".as_slice().into(),
            }]),
            more: false,
        }),
    );
    let (call, op) = fixtures::next_call(d);
    assert_eq!(op, Op::Read(Read::Commits { before: fixtures::BASE, after: fixtures::HEAD, page: 1 }));
    fixtures::answer(d, call, Err(skein_forge::api::Error::TooLarge(skein_forge::api::Excess::unnamed())));
    let decided = fixtures::answer_next(d, Ok(Answer::Reviews { reviews: Box::new([]), more: false, cut: false }));
    let effect = if preparing {
        change::Effect::Prepare { head: fixtures::HEAD, main: fixtures::BASE }
    } else {
        change::Effect::Update { head: fixtures::HEAD, base: fixtures::BASE }
    };
    let mut found = false;
    for request in &decided {
        if let Request::ChangeDecision {
            task: fixtures::CHANGE,
            decision: change::Decision::Effect(chosen),
            entry: Some(FIRST_ENTRY),
            ..
        } = request
        {
            assert_eq!(*chosen, effect);
            found = true;
        }
    }
    assert!(found, "new effect {effect:?} under a fresh entry");
    let row = d.change(fixtures::CHANGE).expect("new update");
    assert_eq!(row.pending, Some(FIRST_ENTRY));
    assert_eq!(row.change.updates, updates);
    let state = if preparing {
        change::State::Preparing { from: fixtures::HEAD, main: fixtures::BASE, ready_since: env().wall }
    } else {
        change::State::Updating { from: fixtures::HEAD, base: fixtures::BASE, ready_since: env().wall }
    };
    assert_eq!(row.change.state, state);
    assert!(built_entry(&decided, FIRST_ENTRY));
}

#[test]
fn a_let_go_update_reads_the_base_it_asked_for_after_main_moves_on() {
    use skein_forge::api::{Answer, Error, Op, Read};
    use temper_engine_domain_forge_change as change;
    const NEW: skein_forge::api::Commit = [5; 32];
    const BASE2: skein_forge::api::Commit = [6; 32];
    for error in [Error::Timeout, Error::Unavailable, Error::Stale] {
        let (mut d, _) = updating_entry(false);
        settled_change(&mut d, skein_forge_client::Outcome::Failed(error));
        let base = Name {
            forge: REPO.forge,
            repository: REPO.repository,
            what: What::Branch(Box::new([b"main".as_slice().into()])),
        };
        d.heads.insert(base.clone(), BranchHead { name: base, commit: BASE2, owned: false }).unwrap();
        fixtures::step_change(&mut d);
        fixtures::answer_next(&mut d, Ok(Answer::Commit(NEW)));
        fixtures::answer_next(&mut d, Ok(Answer::Commit(BASE2)));
        let mut pull = fixtures::open_pull();
        pull.commit = NEW;
        pull.ci = skein_forge::api::Ci::Pending;
        fixtures::answer_next(&mut d, Ok(Answer::Pull(pull)));
        fixtures::answer_next(&mut d, Ok(fixtures::checks(skein_forge::api::Check::Passed)));
        let (call, op) = fixtures::next_call(&mut d);
        assert_eq!(op, Op::Read(Read::Commits { before: NEW, after: fixtures::BASE, page: 1 }), "{error:?}");
        fixtures::answer(&mut d, call, Ok(Answer::Commits { commits: Box::new([]), more: false }));
        fixtures::answer_next(&mut d, Ok(Answer::Reviews { reviews: Box::new([]), more: false, cut: false }));
        let row = d.change(fixtures::CHANGE).unwrap();
        assert_eq!(row.change.state, change::State::Checking { head: NEW });
        assert_eq!(row.change.clean.as_ref(), &[fixtures::HEAD]);
        assert_eq!(
            d.branch_head(&Name {
                forge: REPO.forge,
                repository: REPO.repository,
                what: What::Branch(Box::new([b"temper".as_slice().into(), b"51".as_slice().into()]))
            }),
            Some(NEW)
        );
    }
}

#[test]
fn a_let_go_update_is_decided_again() {
    for preparing in [false, true] {
        let (mut d, _) = updating_entry(preparing);
        let settled = settled_change(&mut d, skein_forge_client::Outcome::Failed(skein_forge::api::Error::Timeout));
        assert!(settled.contains(&Request::ChangeDecision {
            task: fixtures::CHANGE,
            decision: temper_engine_domain_forge_change::Decision::None,
            entry: None,
            evidence: None,
        }));
        decide_update_again(&mut d, preparing, 2);
    }
}

#[test]
fn a_let_go_update_restored_is_decided_again() {
    for preparing in [false, true] {
        let (mut d, _) = updating_entry(preparing);
        let settled = settled_change(&mut d, skein_forge_client::Outcome::Failed(skein_forge::api::Error::Stale));
        let mut saved = None;
        for request in settled {
            if let Request::Save { record: Stored::Change(row), .. } = request {
                saved = Some(row);
            }
        }
        let saved = saved.expect("saved let-go");
        assert_eq!(saved.pending, None);
        assert_eq!(saved.effect, temper_engine_domain_forge_change::EffectResult::None);
        let mut restored = fixtures::connected_domain();
        outputs(&mut restored, Event::Restore { record: Stored::Repository(adopted()) });
        outputs(&mut restored, Event::Restore { record: Stored::Change(saved) });
        assert_eq!(restored.restore_fit_failure(), None);
        outputs(&mut restored, Event::Restored { clock: skein_forge_client::RecoveryClock::Monotonic });
        outputs(&mut restored, Event::ReadAfresh);
        outputs(&mut restored, Event::SettleOutbox);
        decide_update_again(&mut restored, preparing, 2);
    }
}

#[test]
fn an_update_refused_for_good_still_fails() {
    use skein_forge::api::Error;
    use temper_engine_domain_forge_change::EffectResult;
    for (error, expected) in [
        (Error::Protected, EffectResult::Failed),
        (Error::Forbidden, EffectResult::Failed),
        (Error::Closed, EffectResult::Failed),
        (Error::Refused, EffectResult::Failed),
        (Error::Conflict, EffectResult::Conflict),
    ] {
        for preparing in [false, true] {
            let (mut d, entry) = updating_entry(preparing);
            let outcome = skein_forge_client::Outcome::Failed(error);
            settled_change(&mut d, outcome);
            assert_eq!(d.change(fixtures::CHANGE).expect("refused update").effect, expected, "{error:?}");
            assert_settled_account(&d, &entry, outcome);
        }
    }
}

#[test]
fn only_an_update_is_let_go() {
    use skein_forge::api::{Error, Write};
    for write in [
        Write::Merge { number: 1, head: fixtures::HEAD, how: skein_forge::api::MergeHow::Style },
        Write::OpenPull {
            title: b"fix".as_slice().into(),
            body: Box::new([]),
            head: b"temper/51".as_slice().into(),
            base: b"main".as_slice().into(),
        },
    ] {
        for error in [Error::Stale, Error::Timeout] {
            let (mut d, mut entry) = updating_entry(false);
            entry.effect.write = write.clone();
            // Keep the pending identity while choosing another operation.
            d.entries.get_mut(&entry.number).expect("pending entry").entry = entry.clone();
            let outcome = skein_forge_client::Outcome::Failed(error);
            settled_change(&mut d, outcome);
            assert_eq!(
                d.change(fixtures::CHANGE).expect("failed write").effect,
                temper_engine_domain_forge_change::EffectResult::Failed
            );
            assert_settled_account(&d, &entry, outcome);
        }
    }
}

#[test]
fn a_release_of_a_failed_update_drops_its_result() {
    use temper_engine_domain_forge_change as change;
    for preparing in [false, true] {
        for (effect, update_state, expected) in [
            (change::EffectResult::Failed, true, change::EffectResult::None),
            (change::EffectResult::Conflict, true, change::EffectResult::Conflict),
            (change::EffectResult::Failed, false, change::EffectResult::Failed),
        ] {
            let (mut d, _) = updating_entry(preparing);
            let mut row = d.change(fixtures::CHANGE).expect("update").clone();
            row.pending = None;
            row.effect = effect;
            let was = if update_state { row.change.state } else { change::State::Opening { requested: true } };
            row.change.state = change::State::Held { was: Box::new(was), why: change::Hold::Failed };
            d.changes.insert(row.task, row).expect("existing change slot");
            outputs(&mut d, Event::ReleaseChange { task: fixtures::CHANGE });
            assert_eq!(d.change(fixtures::CHANGE).expect("released").effect, expected);
            if expected == change::EffectResult::None {
                decide_update_again(&mut d, preparing, 1);
            }
        }
    }
}

#[test]
fn a_stored_held_update_keeps_its_hold_and_its_words() {
    use temper_engine_domain_forge_change as change;
    for preparing in [false, true] {
        let (mut d, entry) = updating_entry(preparing);
        settled_change(&mut d, skein_forge_client::Outcome::Held);
        assert_eq!(d.change(fixtures::CHANGE).expect("held entry").effect, change::EffectResult::Failed);
        assert_settled_account(&d, &entry, skein_forge_client::Outcome::Held);
        assert!(
            d.change(fixtures::CHANGE)
                .expect("held entry")
                .effect_account
                .as_ref()
                .expect("account")
                .words
                .starts_with(b"update pull request: uncertain: it may or may not have been made; ")
        );
        // A fresh snapshot puts the failed update on hold.
        let was = d.change(fixtures::CHANGE).expect("update").change.state.clone();
        fixtures::step_change(&mut d);
        fixtures::answer_next(&mut d, Ok(skein_forge::api::Answer::Commit(fixtures::HEAD)));
        fixtures::answer_next(&mut d, Ok(skein_forge::api::Answer::Commit(fixtures::BASE)));
        fixtures::answer_next(&mut d, Ok(skein_forge::api::Answer::Pull(fixtures::open_pull())));
        fixtures::finish_open(&mut d);
        fixtures::answer_next(
            &mut d,
            Ok(skein_forge::api::Answer::Reviews { reviews: Box::new([]), more: false, cut: false }),
        );
        assert_eq!(
            d.change(fixtures::CHANGE).expect("held change").change.state,
            change::State::Held { was: Box::new(was), why: change::Hold::Failed }
        );
        outputs(&mut d, Event::ReleaseChange { task: fixtures::CHANGE });
        decide_update_again(&mut d, preparing, 1);
    }
}

#[test]
fn a_release_keeps_the_repair_and_resolution_tasks_made_before_it() {
    use temper_engine_domain_forge_change as change;
    let mut d = delegate_waiting(
        change::State::Repairing { requested: true, why: change::Repair::Gate(1) },
        change::Delegate::Repair(change::Repair::Gate(1)),
    );
    outputs(
        &mut d,
        Event::Delegated {
            task: fixtures::CHANGE,
            child: 90,
            kind: change::Delegate::Resolve { base: fixtures::BASE },
        },
    );
    outputs(
        &mut d,
        Event::Delegated { task: fixtures::CHANGE, child: 52, kind: change::Delegate::Repair(change::Repair::Gate(1)) },
    );
    let row = d.changes.get_mut(&fixtures::CHANGE).expect("live change");
    row.change.repairs = 1;
    row.change.resolutions = 1;
    row.change.state = change::State::Held { was: Box::new(row.change.state.clone()), why: change::Hold::Failed };
    outputs(&mut d, Event::ReleaseChange { task: fixtures::CHANGE });
    assert_eq!(d.change(fixtures::CHANGE).expect("released").change.repairs, 0);
    outputs(&mut d, waiting_step(false, true));
    fixtures::missing_pull_reads(&mut d);
    let made = fixtures::finish_missing(&mut d);
    assert!(
        made.contains(&Request::ChangeDecision {
            task: fixtures::CHANGE,
            decision: change::Decision::Delegate(change::Delegate::Repair(change::Repair::Gate(1))),
            entry: None,
            evidence: None
        }),
        "{made:?}"
    );
    outputs(
        &mut d,
        Event::Delegated { task: fixtures::CHANGE, child: 53, kind: change::Delegate::Repair(change::Repair::Gate(1)) },
    );
    let row = d.change(fixtures::CHANGE).expect("second repair");
    assert_eq!(row.change.repairs, 1);
    assert_eq!(row.repair_tasks.as_ref(), &[52, 53]);
    assert_eq!(row.resolution_tasks.as_ref(), &[90]);
}

/// One required context gives the gather a reason to read rows, not the pull's verdict.
fn ci_protection(required: bool) -> Protection {
    if !required {
        return Protection::Absent;
    }
    Protection::Rule(skein_forge::api::Protection {
        queue: false,
        signatures: false,
        linear: false,
        code_owners: false,
        bypass: skein_forge::api::Bypass::Never,
        branch: Box::from(b"main".as_slice()),
        contexts: Box::new([Box::from(b"ci".as_slice())]),
        approvals: 0,
        dismiss_stale: false,
    })
}

#[test]
fn head_ci_contexts_are_read_once_per_gather() {
    use skein_forge::api::{Answer, Check, Ci, Op, Read};
    use temper_engine_domain_forge_change as change;
    for (name, (required, numbered, at_head, same_base, head_checks)) in [
        ("known pull serves head", (false, true, true, false, false)),
        ("required context reads head", (true, true, true, false, true)),
        ("found by branches reads head", (false, false, true, false, true)),
        ("pull at another commit reads head", (false, true, false, false, true)),
        ("head is base uses pull once", (false, true, true, true, false)),
        ("required head is base reads checks once", (true, true, true, true, true)),
    ] {
        let mut d = fixtures::connected_domain();
        let mut repository = adopted();
        repository.protection = ci_protection(required);
        outputs(&mut d, Event::Restore { record: Stored::Repository(repository) });
        outputs(&mut d, Event::Restored { clock: skein_forge_client::RecoveryClock::Monotonic });
        outputs(&mut d, Event::ReadAfresh);
        outputs(&mut d, Event::SettleOutbox);
        let mut row = fixtures::opening_row();
        row.pending = None;
        row.effect = change::EffectResult::None;
        row.pull = if numbered { Some(1) } else { None };
        row.change.state = change::State::Checking { head: fixtures::HEAD };
        outputs(&mut d, Event::Change { row });
        if same_base {
            seen_base(&mut d, fixtures::HEAD);
        }
        fixtures::step_change(&mut d);
        fixtures::answer_next(&mut d, Ok(Answer::Commit(fixtures::HEAD)));
        let base = if same_base { fixtures::HEAD } else { fixtures::BASE };
        fixtures::answer_next(&mut d, Ok(Answer::Commit(base)));
        let (call, op) = fixtures::next_call(&mut d);
        let expected = if numbered {
            Read::Pull { number: 1, ci: !required }
        } else {
            Read::PullFor { head: Box::from(b"temper/51".as_slice()), base: Box::from(b"main".as_slice()) }
        };
        assert_eq!(op, Op::Read(expected), "{name}");
        let mut pull = fixtures::open_pull();
        pull.ci = if numbered && !required { Ci::Pending } else { Ci::None };
        if !at_head {
            pull.commit = [8; 32];
        }
        fixtures::answer(&mut d, call, Ok(Answer::Pull(pull)));
        if head_checks {
            let (call, op) = fixtures::next_call(&mut d);
            assert_eq!(op, Op::Read(Read::Checks { commit: fixtures::HEAD }));
            fixtures::answer(&mut d, call, Ok(fixtures::checks(Check::Pending)));
        }
        if !same_base {
            let (call, op) = fixtures::next_call(&mut d);
            assert_eq!(op, Op::Read(Read::Checks { commit: base }));
            fixtures::answer(&mut d, call, Ok(fixtures::checks(Check::Passed)));
        }
        assert_eq!(d.panel_ci.get(&fixtures::CHANGE).unwrap().status, change::Status::Pending, "{name}: head verdict");
        if !same_base {
            let (call, op) = fixtures::next_call(&mut d);
            assert_eq!(op, Op::Read(Read::Commits { before: fixtures::HEAD, after: base, page: 1 }));
            fixtures::answer(&mut d, call, Ok(Answer::Commits { commits: Box::new([]), more: false }));
        }
        let (call, op) = fixtures::next_call(&mut d);
        assert_eq!(
            op,
            Op::Read(Read::Reviews { number: 1, page: 1, skip: 0 }),
            "{name}: equality needs no ancestry read"
        );
        let finished =
            fixtures::answer(&mut d, call, Ok(Answer::Reviews { reviews: Box::new([]), more: false, cut: false }));
        let mut decided = false;
        for request in &finished {
            if let Request::ChangeDecision { decision: change::Decision::NoChange, .. } = request {
                decided |= same_base;
            }
            if let Request::ChangeDecision { decision: change::Decision::Wait { .. }, .. } = request {
                decided |= !same_base;
            }
        }
        assert!(decided, "{name}: {finished:?}");
    }
}
