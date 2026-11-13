//! A change on its requester's held branch (temper/docs/design/domain/forge/change/README.md, 1 and 3).
use super::*;
use skein_forge::api::{Answer, Hint, Op, Read, UpdateStyle, Write};
use temper_engine_domain_forge_change as change;

const GOAL: u64 = 7;
const DELEGATE: u64 = 90;
const GOAL_BRANCH: &[u8] = b"temper/7/g7";
const UPDATE: u64 = fixtures::OPEN + 1;
// Both observed change heads have work beyond this landing tip.
const LANDING_TIP: skein_forge::api::Commit = [2; 32];

fn name() -> Name {
    Name {
        forge: REPO.forge,
        repository: REPO.repository,
        what: What::Branch(Box::new([b"temper".as_slice().into(), b"7".as_slice().into(), b"g7".as_slice().into()])),
    }
}

fn landing_row() -> ChangeRow {
    let mut row = fixtures::opening_row();
    row.branch = GOAL_BRANCH.into();
    row.pending = None;
    row.effect = change::EffectResult::None;
    row.change.state = change::State::Checking { head: fixtures::HEAD };
    row
}

fn held_goal() -> Domain {
    held_by(Some(GOAL))
}

fn held_by(holder: Option<u64>) -> Domain {
    held_by_name(holder, name())
}

fn held_by_name(holder: Option<u64>, resource: Name) -> Domain {
    let mut d = fixtures::connected_domain();
    outputs(&mut d, Event::Restore { record: Stored::Repository(adopted()) });
    outputs(&mut d, Event::Restored { clock: skein_forge_client::RecoveryClock::Monotonic });
    outputs(&mut d, Event::ReadAfresh);
    outputs(&mut d, Event::SettleOutbox);
    if let Some(task) = holder {
        outputs(&mut d, Event::Hold { entry: 0, task, resource: resource.clone(), from: None });
        if resource == name() {
            fixtures::answer_next(&mut d, Ok(Answer::Commit(fixtures::HEAD)));
        } else {
            d.heads
                .insert(resource.clone(), BranchHead { name: resource, commit: fixtures::HEAD, owned: true })
                .unwrap();
        }
    }
    d
}

fn watched_goal() -> Domain {
    let mut d = held_goal();
    outputs(&mut d, Event::Names { task: GOAL, resources: Box::new([name()]) });
    fixtures::answer_next(&mut d, Ok(Answer::Commit(fixtures::HEAD)));
    outputs(
        &mut d,
        Event::Client(skein_forge_client::Event::Pushed {
            resource: skein_forge_client::Resource {
                repository: REPO,
                what: skein_forge_client::What::Branch(GOAL_BRANCH.into()),
            },
            commit: fixtures::HEAD,
        }),
    );
    d
}

fn moved(d: &mut Domain, observed: Option<skein_forge::api::Commit>) -> Box<[Request]> {
    outputs(
        d,
        Event::Hint {
            hint: Hint { repository: REPO, change: skein_forge::api::Change::Branch(GOAL_BRANCH.into()), key: None },
        },
    );
    let mut later = env();
    later.now = later.now.saturating_add(LIMITS.client.hinted);
    fire(d, &later, &mut Queue::uncapped());
    let (call, op) = fixtures::next_call(d);
    assert_eq!(op, Op::Read(Read::Branch { branch: GOAL_BRANCH.into() }));
    let result = match observed {
        Some(commit) => Ok(Answer::Commit(commit)),
        None => Err(skein_forge::api::Error::Missing),
    };
    let requests = fixtures::answer(d, call, result);
    if observed.is_some()
        && let Some(row) = d.change(fixtures::CHANGE)
        && row.pending == Some(UPDATE)
    {
        let (call, op) = fixtures::next_call(d);
        assert_eq!(op, Op::Read(Read::Commits { before: fixtures::BASE, after: fixtures::HEAD, page: 1 }));
        return fixtures::answer(d, call, Ok(Answer::Commits { commits: Box::new([]), more: false }));
    }
    requests
}

#[test]
fn drift_on_a_goal_branch_reaches_the_change_landing_it() {
    for observed in [Some(fixtures::BASE), None] {
        let mut d = watched_goal();
        outputs(&mut d, Event::Change { row: landing_row() });
        let requests = moved(&mut d, observed);
        let drift = DriftChange::Branch { expected: fixtures::HEAD, observed };
        assert_eq!(d.hold(&name()).unwrap().drift, Some(Drift { at: env().wall, change: drift }));
        assert_eq!(d.change(fixtures::CHANGE).unwrap().drift, Some(change::Hold::BranchMoved));
        for task in [GOAL, fixtures::CHANGE] {
            assert!(requests.contains(&Request::Drift { task, resource: name() }), "{requests:?}");
        }
        let row = d.change(fixtures::CHANGE).unwrap().clone();
        assert!(requests.contains(&Request::Save {
            read_afresh: None,
            release: None,
            record: Stored::Change(Box::new(row))
        }));
    }
}

#[test]
fn drift_only_reaches_live_changes_of_the_held_branch_in_the_same_repository() {
    let mut other_forge = landing_row();
    other_forge.repository.forge = 3;
    let mut other_repository = landing_row();
    other_repository.repository.repository = 3;
    let mut other_branch = landing_row();
    other_branch.branch = b"temper/7/c51".as_slice().into();
    let mut ended = landing_row();
    ended.change.state = change::State::Ended;
    for row in [other_forge, other_repository, other_branch, ended] {
        let mut d = watched_goal();
        d.changes.insert(row.task, row).expect("fixture change capacity");
        let requests = moved(&mut d, Some(fixtures::BASE));
        assert!(d.hold(&name()).unwrap().drift.is_some());
        assert_eq!(d.change(fixtures::CHANGE).unwrap().drift, None);
        assert!(requests.contains(&Request::Drift { task: GOAL, resource: name() }));
        assert!(!requests.contains(&Request::Drift { task: fixtures::CHANGE, resource: name() }));
    }
}

fn update_entry(preparing: bool) -> skein_forge_client::Entry {
    skein_forge_client::Entry {
        number: UPDATE,
        task: fixtures::CHANGE,
        repository: REPO,
        effect: skein_forge_client::Effect {
            write: Write::Update { number: 1, style: if preparing { UpdateStyle::Rebase } else { UpdateStyle::Merge } },
            condition: skein_forge_client::Condition::Update { head: fixtures::HEAD, base: fixtures::BASE },
        },
        start: None,
        attempt: None,
        failures: 0,
        queued: false,
    }
}

#[test]
fn an_updates_writer_owns_only_forward_moves_by_its_identity() {
    for (actor, forward, drifted) in [(7, true, false), (8, true, true), (7, false, true)] {
        let mut d = watched_goal();
        let mut row = landing_row();
        row.change.state =
            change::State::Updating { from: fixtures::HEAD, base: fixtures::BASE, ready_since: env().wall };
        row.pending = Some(UPDATE);
        outputs(&mut d, Event::Change { row });
        outputs(&mut d, Event::Enqueue { entry: update_entry(false) });
        design_move(&mut d, actor, forward);
        assert_eq!(
            d.change(fixtures::CHANGE).unwrap().drift,
            if drifted { Some(change::Hold::BranchMoved) } else { None }
        );
    }
}

#[test]
fn the_landing_changes_own_update_of_the_goal_branch_is_not_drift() {
    for preparing in [false, true] {
        for pending in [false, true] {
            let mut d = watched_goal();
            let mut row = landing_row();
            row.change.state = if preparing {
                change::State::Preparing { from: fixtures::HEAD, main: fixtures::BASE, ready_since: env().wall }
            } else {
                change::State::Updating { from: fixtures::HEAD, base: fixtures::BASE, ready_since: env().wall }
            };
            row.pending = if pending { Some(UPDATE) } else { None };
            outputs(&mut d, Event::Change { row });
            outputs(&mut d, Event::Enqueue { entry: update_entry(preparing) });
            let requests = moved(&mut d, Some(fixtures::BASE));
            assert_eq!(d.hold(&name()).unwrap().drift.is_none(), pending);
            assert_eq!(d.change(fixtures::CHANGE).unwrap().drift.is_none(), pending);
            for task in [GOAL, fixtures::CHANGE] {
                assert_eq!(requests.contains(&Request::Drift { task, resource: name() }), !pending);
            }
        }
    }
    for holder in [GOAL, fixtures::CHANGE] {
        let resource = if holder == GOAL {
            name()
        } else {
            Name {
                forge: REPO.forge,
                repository: REPO.repository,
                what: What::Branch(Box::new([b"temper".as_slice().into(), b"51".as_slice().into()])),
            }
        };
        let mut d = held_by_name(Some(holder), resource.clone());
        let mut row = landing_row();
        if holder == fixtures::CHANGE {
            row.branch = b"temper/51".as_slice().into();
        }
        row.change.state = change::State::Updating { from: fixtures::HEAD, base: LANDING_TIP, ready_since: env().wall };
        row.pending = Some(UPDATE);
        outputs(&mut d, Event::Change { row });
        outputs(&mut d, Event::Enqueue { entry: update_entry(false) });
        let mut out = Queue::uncapped();
        domain::change_outcome(
            &mut d,
            &env(),
            UPDATE,
            skein_forge_client::Outcome::Made { made: skein_forge_client::Made::Updated(fixtures::BASE), found: false },
            &mut out,
        );
        assert_eq!(d.branch_head(&resource), Some(fixtures::BASE), "holder={holder}");
        let advanced = BranchHead { name: resource, commit: fixtures::BASE, owned: true };
        let saved = Request::Save { read_afresh: None, release: None, record: Stored::BranchHead(advanced) };
        let mut recorded = false;
        for request in &out {
            recorded |= *request == saved;
        }
        assert!(recorded);
        step_at_head(&mut d, fixtures::BASE);
        let row = d.change(fixtures::CHANGE).unwrap();
        assert_eq!(row.change.state, change::State::Checking { head: fixtures::BASE });
        assert_eq!(row.change.last_head, Some(fixtures::BASE));
        assert_eq!(row.change.clean.as_ref(), &[fixtures::HEAD]);
        step_at_head(&mut d, fixtures::BASE);
        assert_eq!(d.change(fixtures::CHANGE).unwrap().change.state, change::State::Checking { head: fixtures::BASE });
    }
}

fn preparing_domain(goal_branch: bool) -> (Domain, Name) {
    let resource = if goal_branch {
        name()
    } else {
        Name {
            forge: REPO.forge,
            repository: REPO.repository,
            what: What::Branch(Box::new([b"temper".as_slice().into(), b"51".as_slice().into()])),
        }
    };
    let branch: Box<[u8]> = if goal_branch { GOAL_BRANCH.into() } else { b"temper/51".as_slice().into() };
    let holder = if goal_branch { GOAL } else { fixtures::CHANGE };
    let mut d = held_by_name(Some(holder), resource.clone());
    outputs(&mut d, Event::Names { task: holder, resources: Box::new([resource.clone()]) });
    fixtures::answer_next(&mut d, Ok(Answer::Commit(fixtures::HEAD)));
    outputs(
        &mut d,
        Event::Client(skein_forge_client::Event::Pushed {
            resource: skein_forge_client::Resource {
                repository: REPO,
                what: skein_forge_client::What::Branch(branch.clone()),
            },
            commit: fixtures::HEAD,
        }),
    );
    assert!(d.project_landing(
        5,
        change::Way::Turn,
        Box::new([]),
        temper_engine_domain_forge_landing::Project::DEFAULT,
        AfterLanding {
            verdict: temper_engine_domain_forge_tiers::Settings {
                retries: 2,
                suspects: temper_engine_domain_forge_tiers::Suspects::Bisect
            },
            bound: Duration::from_millis(7_200_000),
        },
    ));
    let mut row = landing_row();
    row.branch = branch;
    row.pull = Some(1);
    row.goal = Some(GOAL);
    row.change.way = change::Way::Turn;
    row.change.owns_turn = true;
    row.change.updates = 1;
    row.change.state = change::State::Preparing { from: fixtures::HEAD, main: LANDING_TIP, ready_since: env().wall };
    row.pending = Some(UPDATE);
    outputs(&mut d, Event::Change { row });
    let mut entry = update_entry(true);
    entry.effect.condition = skein_forge_client::Condition::Update { head: fixtures::HEAD, base: LANDING_TIP };
    outputs(&mut d, Event::Enqueue { entry });
    (d, resource)
}

fn prepare_at_head(d: &mut Domain, head: skein_forge::api::Commit) -> Box<[Request]> {
    let base = Name {
        forge: REPO.forge,
        repository: REPO.repository,
        what: What::Branch(Box::new([b"main".as_slice().into()])),
    };
    d.heads.insert(base.clone(), BranchHead { name: base, commit: LANDING_TIP, owned: false }).unwrap();
    let mut requests = skein_lib::List::uncapped();
    for request in outputs(d, fixtures::change_step()) {
        requests.push(request).expect("uncapped test output");
    }
    for _ in 0_u32..12 {
        if d.steps.is_empty() {
            return requests.into_boxed();
        }
        let (call, op) = fixtures::next_call(d);
        let answer = match op {
            Op::Read(Read::Branch { branch }) => {
                Answer::Commit(if branch.as_ref() == b"main" { LANDING_TIP } else { head })
            }
            Op::Read(Read::Pull { .. } | Read::PullFor { .. }) => {
                let mut pull = fixtures::open_pull();
                pull.head = d.change(fixtures::CHANGE).unwrap().branch.clone();
                pull.base_commit = Some(LANDING_TIP);
                pull.commit = head;
                Answer::Pull(pull)
            }
            Op::Read(Read::Checks { .. }) => fixtures::checks(skein_forge::api::Check::Passed),
            Op::Read(Read::Commits { before, after, .. }) => {
                assert_eq!((before, after), (head, LANDING_TIP));
                Answer::Commits { commits: Box::new([]), more: false }
            }
            Op::Read(Read::Reviews { .. }) => Answer::Reviews { reviews: Box::new([]), more: false, cut: false },
            Op::Read(Read::Compare { before, after }) => {
                Answer::Compare { before, after, files: Box::new([]), ahead: 0, behind: 0, changed: 0 }
            }
            Op::Read(_) | Op::Write(_) => panic!("prepare read: {op:?}"),
        };
        for request in fixtures::answer(d, call, Ok(answer)) {
            requests.push(request).expect("uncapped test output");
        }
    }
    panic!("prepare read did not settle")
}

#[test]
fn prepares_rebase_answer_is_its_candidate_not_drift() {
    for goal_branch in [true, false] {
        let (mut d, resource) = preparing_domain(goal_branch);
        let mut out = Queue::uncapped();
        domain::change_outcome(
            &mut d,
            &env(),
            UPDATE,
            skein_forge_client::Outcome::Made { made: skein_forge_client::Made::Updated(fixtures::BASE), found: false },
            &mut out,
        );
        assert_eq!(d.branch_head(&resource), Some(fixtures::BASE));
        // A keep's queued observation can follow the outcome that confirmed this head.
        domain::drift(
            &mut d,
            &env(),
            skein_forge_client::Resource {
                repository: REPO,
                what: skein_forge_client::What::Branch(if goal_branch {
                    GOAL_BRANCH.into()
                } else {
                    b"temper/51".as_slice().into()
                }),
            },
            DriftChange::Branch { expected: fixtures::HEAD, observed: Some(fixtures::BASE) },
            &mut out,
        );
        assert_eq!(d.change(fixtures::CHANGE).unwrap().drift, None);
        for request in &out {
            if let Request::Drift { .. } = request {
                panic!("{request:?}");
            }
        }
        let requests = prepare_at_head(&mut d, fixtures::BASE);
        assert_eq!(d.hold(&resource).unwrap().drift, None);
        let row = d.change(fixtures::CHANGE).unwrap();
        assert_eq!(row.drift, None);
        assert_eq!(row.change.state, change::State::Ready { head: fixtures::BASE, ready_since: env().wall });
        assert_eq!(row.change.last_head, Some(fixtures::BASE));
        assert_eq!(row.change.clean.as_ref(), &[fixtures::HEAD]);
        for request in &requests {
            if let Request::Drift { .. } = request {
                panic!("{request:?}");
            }
        }
    }
}

fn prepare_move(d: &mut Domain, actor: u64, forward: bool) -> Box<[Request]> {
    let branch = d.change(fixtures::CHANGE).unwrap().branch.clone();
    outputs(
        d,
        Event::Hint {
            hint: Hint {
                repository: REPO,
                change: skein_forge::api::Change::BranchMoved { branch: branch.clone(), head: fixtures::BASE, actor },
                key: None,
            },
        },
    );
    let mut later = env();
    later.now = later.now.saturating_add(LIMITS.client.hinted);
    fire(d, &later, &mut Queue::uncapped());
    let (call, op) = fixtures::next_call(d);
    assert_eq!(op, Op::Read(Read::Branch { branch }));
    fixtures::answer(d, call, Ok(Answer::Commit(fixtures::BASE)));
    assert_eq!(d.change(fixtures::CHANGE).unwrap().drift, None, "the update's writer compares its move before drift");
    let (call, op) = fixtures::next_call(d);
    assert_eq!(op, Op::Read(Read::Commits { before: fixtures::BASE, after: fixtures::HEAD, page: 1 }));
    fixtures::answer(d, call, Ok(design_ancestry(forward)))
}

fn rebase_made(d: &mut Domain) {
    domain::change_outcome(
        d,
        &env(),
        UPDATE,
        skein_forge_client::Outcome::Made { made: skein_forge_client::Made::Updated(fixtures::BASE), found: false },
        &mut Queue::uncapped(),
    );
}

fn observed_rebase(d: &mut Domain, actor: Option<u64>) -> Box<[Request]> {
    let branch = d.change(fixtures::CHANGE).unwrap().branch.clone();
    let change = match actor {
        Some(actor) => skein_forge::api::Change::BranchMoved { branch: branch.clone(), head: fixtures::BASE, actor },
        None => skein_forge::api::Change::Branch(branch.clone()),
    };
    if actor.is_some() {
        outputs(d, Event::Hint { hint: Hint { repository: REPO, change, key: None } });
    }
    let mut later = env();
    later.now = later.now.saturating_add(if actor.is_some() { LIMITS.client.hinted } else { LIMITS.client.slow });
    fire(d, &later, &mut Queue::uncapped());
    let (call, op) = fixtures::next_call(d);
    assert_eq!(op, Op::Read(Read::Branch { branch }));
    fixtures::answer(d, call, Ok(Answer::Commit(fixtures::BASE)));
    let (call, op) = fixtures::next_call(d);
    assert_eq!(op, Op::Read(Read::Commits { before: fixtures::BASE, after: fixtures::HEAD, page: 1 }));
    // Forgejo's rebase contains main, not the original change commit.
    fixtures::answer(d, call, Ok(design_ancestry(false)))
}

fn assert_rebase_ready(d: &Domain, resource: &Name) {
    assert_eq!(d.hold(resource).unwrap().drift, None);
    let row = d.change(fixtures::CHANGE).unwrap();
    assert_eq!(row.drift, None);
    assert_eq!(row.change.state, change::State::Ready { head: fixtures::BASE, ready_since: env().wall });
    assert_eq!(row.change.last_head, Some(fixtures::BASE));
    assert_eq!(row.change.clean.as_ref(), &[fixtures::HEAD]);
    assert_eq!(d.branch_head(resource), Some(fixtures::BASE));
}

#[test]
fn prepares_writer_owns_its_rebase_before_the_answer() {
    for goal_branch in [false, true] {
        let (mut d, resource) = preparing_domain(goal_branch);
        observed_rebase(&mut d, Some(7));
        assert_eq!(d.hold(&resource).unwrap().drift, None);
        let (call, op) = fixtures::next_call(&mut d);
        assert_eq!(op, Op::Read(Read::Commits { before: fixtures::BASE, after: LANDING_TIP, page: 1 }));
        fixtures::answer(&mut d, call, Ok(design_ancestry(true)));
        assert_eq!(d.branch_head(&resource), Some(fixtures::BASE));
        prepare_at_head(&mut d, fixtures::BASE);
        assert_eq!(d.change(fixtures::CHANGE).unwrap().change.last_head, Some(fixtures::HEAD));
        rebase_made(&mut d);
        prepare_at_head(&mut d, fixtures::BASE);
        assert_rebase_ready(&d, &resource);
    }
}

#[test]
fn prepares_unattributed_rebase_waits_for_its_exact_answer() {
    for goal_branch in [false, true] {
        let (mut d, resource) = preparing_domain(goal_branch);
        observed_rebase(&mut d, None);
        prepare_at_head(&mut d, fixtures::BASE);
        assert_eq!(
            d.change(fixtures::CHANGE).unwrap().change.state,
            change::State::Preparing { from: fixtures::HEAD, main: LANDING_TIP, ready_since: env().wall }
        );
        assert_eq!(d.hold(&resource).unwrap().drift, None);
        assert_eq!(d.branch_head(&resource), Some(fixtures::HEAD), "a gather is not confirmation");
        assert_eq!(d.change(fixtures::CHANGE).unwrap().change.last_head, Some(fixtures::HEAD));
        rebase_made(&mut d);
        prepare_at_head(&mut d, fixtures::BASE);
        assert_rebase_ready(&d, &resource);
    }
}

#[test]
fn prepares_unattributed_rebase_is_drift_on_any_other_completion() {
    use skein_forge_client::{Made, Outcome};
    for goal_branch in [false, true] {
        for outcome in [
            Outcome::Made { made: Made::Updated([8; 32]), found: false },
            Outcome::Failed(skein_forge::api::Error::Timeout),
            Outcome::Raced { made: Made::Updated(fixtures::BASE), why: skein_forge::api::Error::Stale },
            Outcome::Withdrawn,
            Outcome::Held,
        ] {
            let (mut d, resource) = preparing_domain(goal_branch);
            observed_rebase(&mut d, None);
            assert_eq!(d.hold(&resource).unwrap().drift, None);
            domain::change_outcome(&mut d, &env(), UPDATE, outcome, &mut Queue::uncapped());
            assert_eq!(
                d.hold(&resource).unwrap().drift.as_ref().unwrap().change,
                DriftChange::Branch { expected: fixtures::HEAD, observed: Some(fixtures::BASE) },
                "{outcome:?}"
            );
            prepare_at_head(&mut d, fixtures::BASE);
            assert_eq!(d.change(fixtures::CHANGE).unwrap().drift, Some(change::Hold::BranchMoved));
            assert!(matches!(
                d.change(fixtures::CHANGE).unwrap().change.state,
                change::State::Held { why: change::Hold::BranchMoved, .. }
            ));
        }
    }
}

#[test]
fn prepares_rebase_without_main_is_drift() {
    for goal_branch in [false, true] {
        let (mut d, resource) = preparing_domain(goal_branch);
        observed_rebase(&mut d, Some(7));
        assert_eq!(d.hold(&resource).unwrap().drift, None);
        let (call, op) = fixtures::next_call(&mut d);
        assert_eq!(op, Op::Read(Read::Commits { before: fixtures::BASE, after: LANDING_TIP, page: 1 }));
        fixtures::answer(&mut d, call, Ok(design_ancestry(false)));
        assert_eq!(
            d.hold(&resource).unwrap().drift.as_ref().unwrap().change,
            DriftChange::Branch { expected: fixtures::HEAD, observed: Some(fixtures::BASE) }
        );
        rebase_made(&mut d);
        prepare_at_head(&mut d, fixtures::BASE);
        assert_eq!(d.change(fixtures::CHANGE).unwrap().drift, Some(change::Hold::BranchMoved));
    }
}

#[test]
fn only_prepares_pending_rebase_can_explain_a_rewrite() {
    for goal_branch in [false, true] {
        for (actor, preparing, rebase, pending) in
            [(8, true, true, true), (7, true, false, true), (7, false, true, true), (7, true, true, false)]
        {
            let (mut d, resource) = preparing_domain(goal_branch);
            if !preparing {
                d.changes.get_mut(&fixtures::CHANGE).unwrap().change.state =
                    change::State::Updating { from: fixtures::HEAD, base: LANDING_TIP, ready_since: env().wall };
            }
            if !rebase {
                d.entries.get_mut(&UPDATE).unwrap().entry.effect.write =
                    Write::Update { number: 1, style: UpdateStyle::Merge };
            }
            if !pending {
                domain::change_outcome(
                    &mut d,
                    &env(),
                    UPDATE,
                    skein_forge_client::Outcome::Failed(skein_forge::api::Error::Timeout),
                    &mut Queue::uncapped(),
                );
            }
            observed_rebase(&mut d, Some(actor));
            assert_eq!(
                d.hold(&resource).unwrap().drift.as_ref().unwrap().change,
                DriftChange::Branch { expected: fixtures::HEAD, observed: Some(fixtures::BASE) }
            );
            assert_eq!(d.change(fixtures::CHANGE).unwrap().drift, Some(change::Hold::BranchMoved));
        }
    }
}

#[test]
fn prepares_pending_rebase_does_not_explain_a_third_head_or_a_late_original() {
    for goal_branch in [false, true] {
        for head in [[8; 32], fixtures::HEAD] {
            let (mut d, resource) = preparing_domain(goal_branch);
            observed_rebase(&mut d, Some(7));
            let (call, _) = fixtures::next_call(&mut d);
            fixtures::answer(&mut d, call, Ok(design_ancestry(true)));
            let branch = d.change(fixtures::CHANGE).unwrap().branch.clone();
            outputs(
                &mut d,
                Event::Hint {
                    hint: Hint {
                        repository: REPO,
                        change: skein_forge::api::Change::BranchMoved { branch: branch.clone(), head, actor: 7 },
                        key: None,
                    },
                },
            );
            let mut later = env();
            later.now = later.now.saturating_add(LIMITS.client.hinted);
            fire(&mut d, &later, &mut Queue::uncapped());
            let (call, op) = fixtures::next_call(&mut d);
            assert_eq!(op, Op::Read(Read::Branch { branch }));
            fixtures::answer(&mut d, call, Ok(Answer::Commit(head)));
            let (call, op) = fixtures::next_call(&mut d);
            assert_eq!(op, Op::Read(Read::Commits { before: head, after: fixtures::BASE, page: 1 }));
            fixtures::answer(&mut d, call, Ok(design_ancestry(false)));
            assert_eq!(
                d.hold(&resource).unwrap().drift.as_ref().unwrap().change,
                DriftChange::Branch { expected: fixtures::BASE, observed: Some(head) }
            );
            rebase_made(&mut d);
            prepare_at_head(&mut d, head);
            assert_eq!(d.change(fixtures::CHANGE).unwrap().drift, Some(change::Hold::BranchMoved));
        }
    }
}

#[test]
fn prepares_deferred_rebase_is_rebuilt_after_restart() {
    for goal_branch in [false, true] {
        for confirms in [false, true] {
            let (mut d, resource) = preparing_domain(goal_branch);
            let requests = observed_rebase(&mut d, None);
            assert_eq!(d.hold(&resource).unwrap().drift, None);
            let mut live = None;
            for request in requests {
                if let Request::Save { record: Stored::Client(skein_forge_client::Stored::Live(row)), .. } = request {
                    live = Some(row);
                }
            }
            let live = live.expect("keep saves the observed head");
            assert_eq!(live.cached.tip, Some(fixtures::BASE));
            let row = d.change(fixtures::CHANGE).unwrap().clone();
            let entry = d.entries.get(&UPDATE).unwrap().entry.clone();
            let hold = d.hold(&resource).unwrap().clone();
            let mut restored = fixtures::connected_domain();
            let records = [
                Stored::Repository(adopted()),
                Stored::Change(Box::new(row)),
                Stored::Hold(hold),
                Stored::Names {
                    task: if goal_branch { GOAL } else { fixtures::CHANGE },
                    resources: Box::new([resource.clone()]),
                },
                Stored::Entry { entry, held: false },
                Stored::Client(skein_forge_client::Stored::Live(live)),
                Stored::Client(skein_forge_client::Stored::Repository(skein_forge_client::RepositoryRecord {
                    repository: REPO,
                    mark: None,
                    clock: env().now,
                })),
                Stored::BranchHead(BranchHead { name: resource.clone(), commit: fixtures::HEAD, owned: true }),
            ];
            for record in records {
                outputs(&mut restored, Event::Restore { record });
            }
            assert_eq!(restored.restore_fit_failure(), None);
            outputs(&mut restored, Event::Restored { clock: skein_forge_client::RecoveryClock::Monotonic });
            outputs(&mut restored, Event::ReadAfresh);
            // The keep's unchanged first read cannot rediscover drift on its own.
            let (call, op) = fixtures::next_call(&mut restored);
            assert_eq!(
                op,
                Op::Read(Read::Branch { branch: restored.change(fixtures::CHANGE).unwrap().branch.clone() })
            );
            fixtures::answer(&mut restored, call, Ok(Answer::Commit(fixtures::BASE)));
            assert_eq!(restored.change(fixtures::CHANGE).unwrap().drift, None);
            let outcome = if confirms {
                skein_forge_client::Outcome::Made {
                    made: skein_forge_client::Made::Updated(fixtures::BASE),
                    found: true,
                }
            } else {
                skein_forge_client::Outcome::Failed(skein_forge::api::Error::Timeout)
            };
            domain::change_outcome(&mut restored, &env(), UPDATE, outcome, &mut Queue::uncapped());
            assert_eq!(
                restored.change(fixtures::CHANGE).unwrap().drift,
                if confirms { None } else { Some(change::Hold::BranchMoved) }
            );
        }
    }
}

#[test]
fn prepares_writer_owns_its_forward_head_before_the_answer() {
    for goal_branch in [false, true] {
        let (mut d, resource) = preparing_domain(goal_branch);
        let requests = prepare_move(&mut d, 7, true);
        assert_eq!(d.hold(&resource).unwrap().drift, None);
        let row = d.change(fixtures::CHANGE).unwrap();
        assert_eq!(row.drift, None);
        assert_eq!(
            row.change.state,
            change::State::Preparing { from: fixtures::HEAD, main: LANDING_TIP, ready_since: env().wall }
        );
        assert_eq!(d.branch_head(&resource), Some(fixtures::HEAD), "an observation is not a confirmed head");
        for request in &requests {
            if let Request::Drift { .. } = request {
                panic!("{request:?}");
            }
        }
        rebase_made(&mut d);
        prepare_at_head(&mut d, fixtures::BASE);
        let row = d.change(fixtures::CHANGE).unwrap();
        assert_eq!(row.drift, None);
        assert_eq!(row.change.state, change::State::Ready { head: fixtures::BASE, ready_since: env().wall });
        assert_eq!(row.change.clean.as_ref(), &[fixtures::HEAD]);
        assert_eq!(d.branch_head(&resource), Some(fixtures::BASE));
    }
}

#[test]
fn prepares_writer_does_not_own_another_identity_or_a_backwards_move() {
    for goal_branch in [false, true] {
        for (actor, forward) in [(8, true), (7, false)] {
            let (mut d, resource) = preparing_domain(goal_branch);
            let mut requests = prepare_move(&mut d, actor, forward);
            if actor == 7 && !forward {
                let (call, op) = fixtures::next_call(&mut d);
                assert_eq!(op, Op::Read(Read::Commits { before: fixtures::BASE, after: LANDING_TIP, page: 1 }));
                requests = fixtures::answer(&mut d, call, Ok(design_ancestry(false)));
            }
            assert!(requests.contains(&Request::Drift { task: fixtures::CHANGE, resource: resource.clone() }));
            assert_eq!(
                d.hold(&resource).unwrap().drift.as_ref().unwrap().change,
                DriftChange::Branch { expected: fixtures::HEAD, observed: Some(fixtures::BASE) }
            );
            prepare_at_head(&mut d, fixtures::BASE);
            let row = d.change(fixtures::CHANGE).unwrap();
            assert_eq!(row.drift, Some(change::Hold::BranchMoved));
            assert_eq!(
                row.change.state,
                change::State::Held {
                    was: Box::new(change::State::Preparing {
                        from: fixtures::HEAD,
                        main: LANDING_TIP,
                        ready_since: env().wall
                    }),
                    why: change::Hold::BranchMoved,
                }
            );
            assert_eq!(d.branch_head(&resource), Some(fixtures::HEAD));
        }
    }
}

#[test]
fn prepares_settled_answer_does_not_explain_a_third_head() {
    const OUTSIDE: skein_forge::api::Commit = [8; 32];
    for goal_branch in [false, true] {
        let (mut d, resource) = preparing_domain(goal_branch);
        rebase_made(&mut d);
        let requests = prepare_at_head(&mut d, OUTSIDE);
        assert!(requests.contains(&Request::Drift { task: fixtures::CHANGE, resource: resource.clone() }));
        assert_eq!(
            d.hold(&resource).unwrap().drift.as_ref().unwrap().change,
            DriftChange::Branch { expected: fixtures::BASE, observed: Some(OUTSIDE) }
        );
        let row = d.change(fixtures::CHANGE).unwrap();
        assert_eq!(row.drift, Some(change::Hold::BranchMoved));
        assert_eq!(
            row.change.state,
            change::State::Held {
                was: Box::new(change::State::Preparing {
                    from: fixtures::HEAD,
                    main: LANDING_TIP,
                    ready_since: env().wall
                }),
                why: change::Hold::BranchMoved,
            }
        );
        assert_eq!(d.branch_head(&resource), Some(fixtures::BASE));
    }
}

#[test]
fn prepares_candidate_does_not_explain_a_late_copy_of_the_original_head() {
    for goal_branch in [false, true] {
        let (mut d, resource) = preparing_domain(goal_branch);
        rebase_made(&mut d);
        prepare_at_head(&mut d, fixtures::BASE);
        let branch = d.change(fixtures::CHANGE).unwrap().branch.clone();
        outputs(
            &mut d,
            Event::Hint {
                hint: Hint { repository: REPO, change: skein_forge::api::Change::Branch(branch.clone()), key: None },
            },
        );
        let mut later = env();
        later.now = later.now.saturating_add(LIMITS.client.hinted);
        fire(&mut d, &later, &mut Queue::uncapped());
        let (call, op) = fixtures::next_call(&mut d);
        assert_eq!(op, Op::Read(Read::Branch { branch }));
        let requests = fixtures::answer(&mut d, call, Ok(Answer::Commit(fixtures::HEAD)));
        assert!(requests.contains(&Request::Drift { task: fixtures::CHANGE, resource: resource.clone() }));
        assert_eq!(
            d.hold(&resource).unwrap().drift.as_ref().unwrap().change,
            DriftChange::Branch { expected: fixtures::BASE, observed: Some(fixtures::HEAD) }
        );
        prepare_at_head(&mut d, fixtures::HEAD);
        let row = d.change(fixtures::CHANGE).unwrap();
        assert_eq!(row.drift, Some(change::Hold::BranchMoved));
        assert_eq!(
            row.change.state,
            change::State::Held {
                was: Box::new(change::State::Ready { head: fixtures::BASE, ready_since: env().wall }),
                why: change::Hold::BranchMoved,
            }
        );
        assert_eq!(d.branch_head(&resource), Some(fixtures::BASE));
    }
}

#[test]
fn prepares_let_go_update_takes_its_forward_head_after_the_slot_frees() {
    for goal_branch in [false, true] {
        let (mut d, resource) = preparing_domain(goal_branch);
        domain::change_outcome(
            &mut d,
            &env(),
            UPDATE,
            skein_forge_client::Outcome::Failed(skein_forge::api::Error::Timeout),
            &mut Queue::uncapped(),
        );
        let requests = prepare_move(&mut d, 7, true);
        for request in &requests {
            if let Request::Drift { .. } = request {
                panic!("{request:?}");
            }
        }
        prepare_at_head(&mut d, fixtures::BASE);
        let row = d.change(fixtures::CHANGE).unwrap();
        assert_eq!(d.hold(&resource).unwrap().drift, None);
        assert_eq!(row.drift, None);
        assert_eq!(row.change.state, change::State::Ready { head: fixtures::BASE, ready_since: env().wall });
        assert_eq!(row.change.clean.as_ref(), &[fixtures::HEAD]);
        assert_eq!(d.branch_head(&resource), Some(fixtures::BASE));
    }
}

#[test]
fn a_let_go_updates_forward_head_is_not_drift_before_its_containment_is_read() {
    use skein_forge::api::Error;
    for error in [Error::Timeout, Error::Unavailable, Error::Stale] {
        let mut d = watched_goal();
        let mut row = landing_row();
        row.change.state =
            change::State::Updating { from: fixtures::HEAD, base: fixtures::BASE, ready_since: env().wall };
        row.pending = Some(UPDATE);
        outputs(&mut d, Event::Change { row });
        outputs(&mut d, Event::Enqueue { entry: update_entry(false) });
        domain::change_outcome(
            &mut d,
            &env(),
            UPDATE,
            skein_forge_client::Outcome::Failed(error),
            &mut Queue::uncapped(),
        );
        design_move(&mut d, 7, true);
        assert_eq!(d.change(fixtures::CHANGE).unwrap().drift, None, "{error:?}");
        step_at_head(&mut d, fixtures::BASE);
        let row = d.change(fixtures::CHANGE).unwrap();
        assert_eq!(row.change.state, change::State::Checking { head: fixtures::BASE });
        assert_eq!(row.change.clean.as_ref(), &[fixtures::HEAD]);
        assert_eq!(d.branch_head(&name()), Some(fixtures::BASE));
        moved(&mut d, Some(fixtures::HEAD));
        assert_eq!(
            d.change(fixtures::CHANGE).unwrap().drift,
            Some(change::Hold::BranchMoved),
            "a late copy is still drift"
        );
    }
}

#[test]
fn a_settled_update_does_not_explain_another_head() {
    const OUTSIDE: skein_forge::api::Commit = [8; 32];
    for holder in [GOAL, fixtures::CHANGE] {
        let mut d = held_by(Some(holder));
        let mut row = landing_row();
        row.change.state = change::State::Updating { from: fixtures::HEAD, base: LANDING_TIP, ready_since: env().wall };
        row.pending = Some(UPDATE);
        outputs(&mut d, Event::Change { row });
        domain::change_outcome(
            &mut d,
            &env(),
            UPDATE,
            skein_forge_client::Outcome::Made { made: skein_forge_client::Made::Updated(fixtures::BASE), found: false },
            &mut Queue::uncapped(),
        );
        step_at_head(&mut d, OUTSIDE);
        assert_eq!(
            d.change(fixtures::CHANGE).unwrap().change.state,
            change::State::Held {
                was: Box::new(change::State::Updating {
                    from: fixtures::HEAD,
                    base: LANDING_TIP,
                    ready_since: env().wall
                }),
                why: change::Hold::BranchMoved,
            }
        );
        assert_eq!(
            d.hold(&name()).unwrap().drift.as_ref().unwrap().change,
            DriftChange::Branch { expected: fixtures::BASE, observed: Some(OUTSIDE) }
        );
    }
}

#[test]
fn a_landing_changes_repair_writes_the_goal_branch_under_its_goals_hold() {
    for kind in [
        change::Delegate::Repair(change::Repair::Gate(1)),
        change::Delegate::Resolve { base: fixtures::BASE },
        change::Delegate::Merge { base: fixtures::BASE, from: Some(fixtures::HEAD) },
    ] {
        for holder in [GOAL, fixtures::CHANGE] {
            let mut d = held_by(Some(holder));
            let mut row = landing_row();
            row.delegate = Some((DELEGATE, kind));
            outputs(&mut d, Event::Change { row });
            let item = d
                .workspace_item(items::Seed {
                    project: 5,
                    modify: false,
                    task: DELEGATE,
                    parent: Some(fixtures::CHANGE),
                    repository: REPO,
                    run_branch: b"temper/7/r90".as_slice().into(),
                    saved_branch: None,
                })
                .unwrap();
            assert_eq!(item.holder, Some(holder));
            assert_eq!(item.push.as_deref(), Some(GOAL_BRANCH));
            let start = match kind {
                change::Delegate::Repair(_) => items::Start::Branch(GOAL_BRANCH.into()),
                change::Delegate::Resolve { base } | change::Delegate::Merge { base, .. } => {
                    items::Start::Merge { branch: GOAL_BRANCH.into(), base }
                }
                change::Delegate::Produce | change::Delegate::Close | change::Delegate::Gate { .. } => {
                    unreachable!("repair or resolution")
                }
            };
            assert_eq!(item.start, start);
            let requests = outputs(
                &mut d,
                Event::Claim {
                    designs: Box::new([]),
                    task: DELEGATE,
                    attempt: 1,
                    writes: Box::new([name()]),
                    holders: Box::new([item.holder.unwrap()]),
                },
            );
            assert_eq!(
                d.hold(&name()).unwrap().writer,
                Some(Writer::Run { task: DELEGATE, attempt: 1 }),
                "{requests:?}"
            );
            outputs(
                &mut d,
                Event::Answered { task: DELEGATE, attempt: 1, pushed: Box::new([(name(), fixtures::BASE, true)]) },
            );
            assert_eq!(d.hold(&name()).unwrap().writer, None);
            assert_eq!(d.branch_head(&name()), Some(fixtures::BASE));
            assert_eq!(d.hold(&name()).unwrap().task, holder);
        }
    }
}

#[test]
fn facts_say_whether_a_changes_branch_is_its_requesters() {
    for holder in [Some(GOAL), Some(fixtures::CHANGE), None] {
        let mut d = held_by(holder);
        outputs(&mut d, Event::Change { row: landing_row() });
        if holder == Some(GOAL) {
            confirm_head(&mut d, fixtures::BASE);
        }
        let requests = step_at_head(&mut d, fixtures::BASE);
        let state = &d.change(fixtures::CHANGE).unwrap().change.state;
        let expected = if holder == Some(GOAL) {
            change::State::Checking { head: fixtures::BASE }
        } else {
            change::State::Held {
                was: Box::new(change::State::Checking { head: fixtures::HEAD }),
                why: change::Hold::BranchMoved,
            }
        };
        assert_eq!(state, &expected, "holder={holder:?}: {requests:?}");
    }
}

fn step_at_head(d: &mut Domain, head: skein_forge::api::Commit) -> Box<[Request]> {
    step_at_head_for(d, fixtures::CHANGE, head)
}

pub(super) fn step_at_head_for(d: &mut Domain, task: u64, head: skein_forge::api::Commit) -> Box<[Request]> {
    let base = Name {
        forge: REPO.forge,
        repository: REPO.repository,
        what: What::Branch(Box::new([b"main".as_slice().into()])),
    };
    d.heads
        .insert(base.clone(), BranchHead { name: base, commit: LANDING_TIP, owned: false })
        .expect("fixture head room");
    let Event::StepChange { entry, heard, gates, queue_repair_active, kept, merge_job, pending, .. } =
        fixtures::change_step()
    else {
        panic!("step fixture")
    };
    outputs(d, Event::StepChange { task, entry, heard, gates, queue_repair_active, kept, merge_job, pending });
    fixtures::answer_next(d, Ok(Answer::Commit(head)));
    fixtures::answer_next(d, Ok(Answer::Commit(LANDING_TIP)));
    let mut pull = fixtures::open_pull();
    pull.base_commit = Some(LANDING_TIP);
    pull.head = d.change(task).unwrap().branch.clone();
    pull.commit = head;
    pull.ci = skein_forge::api::Ci::Pending;
    fixtures::answer_next(d, Ok(Answer::Pull(pull)));
    for _ in 0_u32..3 {
        let (call, op) = fixtures::next_call(d);
        if let Op::Read(Read::Checks { commit }) = op {
            let check = if commit == head { skein_forge::api::Check::Pending } else { skein_forge::api::Check::Passed };
            fixtures::answer(d, call, Ok(fixtures::checks(check)));
        } else {
            let Op::Read(Read::Commits { .. }) = op else { panic!("{op:?}") };
            fixtures::answer(d, call, Ok(Answer::Commits { commits: Box::new([]), more: false }));
            break;
        }
    }
    fixtures::answer_next(d, Ok(Answer::Reviews { reviews: Box::new([]), more: false, cut: false }))
}

#[test]
fn a_merge_entry_into_the_goal_branch_stops_its_landings_head_decision() {
    for design_writer in [false, true] {
        let mut d = held_goal();
        if design_writer {
            design_claim(&mut d);
        }
        outputs(&mut d, Event::Change { row: landing_row() });
        outputs(
            &mut d,
            Event::Enqueue {
                entry: skein_forge_client::Entry {
                    number: UPDATE,
                    task: DELEGATE,
                    repository: REPO,
                    effect: skein_forge_client::Effect {
                        write: Write::Merge { number: 2, head: fixtures::BASE, how: skein_forge::api::MergeHow::Style },
                        condition: skein_forge_client::Condition::Merge { base: GOAL_BRANCH.into() },
                    },
                    start: None,
                    attempt: None,
                    failures: 0,
                    queued: false,
                },
            },
        );
        step_at_head(&mut d, fixtures::BASE);
        assert_eq!(d.change(fixtures::CHANGE).unwrap().change.state, change::State::Checking { head: fixtures::HEAD });
    }
}

#[test]
fn an_unconfirmed_move_of_a_goal_branch_is_drift_before_the_keep_reads_it() {
    let mut d = held_goal();
    outputs(&mut d, Event::Change { row: landing_row() });
    let requests = step_at_head(&mut d, fixtures::BASE);
    assert_eq!(
        d.change(fixtures::CHANGE).unwrap().change.state,
        change::State::Held {
            was: Box::new(change::State::Checking { head: fixtures::HEAD }),
            why: change::Hold::BranchMoved,
        },
        "{requests:?}"
    );
}

fn design_claim(d: &mut Domain) {
    outputs(
        d,
        Event::Claim {
            task: GOAL,
            attempt: 1,
            writes: Box::new([]),
            designs: Box::new([name()]),
            holders: Box::new([]),
        },
    );
}

#[test]
fn a_design_writer_takes_no_slot_and_its_answer_reads_the_goal_afresh() {
    let mut d = held_goal();
    design_claim(&mut d);
    let hold = d.hold(&name()).unwrap();
    assert_eq!(hold.writer, None);
    assert_eq!(hold.design_writers.as_ref(), &[DesignWriter { task: GOAL, attempt: 1, rereading: false }]);
    outputs(&mut d, Event::Answered { task: GOAL, attempt: 1, pushed: Box::new([(name(), fixtures::BASE, true)]) });
    assert_eq!(d.branch_head(&name()), Some(fixtures::HEAD), "the report is not the head");
    let (call, op) = fixtures::next_call(&mut d);
    assert_eq!(op, Op::Read(Read::Branch { branch: GOAL_BRANCH.into() }));
    fixtures::answer(&mut d, call, Ok(Answer::Commit(fixtures::BASE)));
    let (call, op) = fixtures::next_call(&mut d);
    assert_eq!(op, Op::Read(Read::Commits { before: fixtures::BASE, after: fixtures::HEAD, page: 1 }));
    fixtures::answer(&mut d, call, Ok(Answer::Commits { commits: Box::new([]), more: false }));
    assert_eq!(d.branch_head(&name()), Some(fixtures::BASE));
    assert!(d.hold(&name()).unwrap().design_writers.is_empty());
    assert_eq!(d.hold(&name()).unwrap().drift, None);
}

fn design_ancestry(forward: bool) -> Answer {
    let commits: Box<[_]> = if forward {
        Box::new([])
    } else {
        Box::new([skein_forge::api::CommitMessage { id: fixtures::HEAD, parents: Box::new([]), message: Box::new([]) }])
    };
    Answer::Commits { commits, more: false }
}

fn design_move(d: &mut Domain, actor: u64, forward: bool) -> Box<[Request]> {
    outputs(
        d,
        Event::Hint {
            hint: Hint {
                repository: REPO,
                change: skein_forge::api::Change::BranchMoved {
                    branch: GOAL_BRANCH.into(),
                    head: fixtures::BASE,
                    actor,
                },
                key: None,
            },
        },
    );
    let mut later = env();
    later.now = later.now.saturating_add(LIMITS.client.hinted);
    fire(d, &later, &mut Queue::uncapped());
    let (call, op) = fixtures::next_call(d);
    assert_eq!(op, Op::Read(Read::Branch { branch: GOAL_BRANCH.into() }));
    fixtures::answer(d, call, Ok(Answer::Commit(fixtures::BASE)));
    assert_eq!(d.hold(&name()).unwrap().drift, None, "a live writer's move is compared before judgement");
    let (call, op) = fixtures::next_call(d);
    assert_eq!(op, Op::Read(Read::Commits { before: fixtures::BASE, after: fixtures::HEAD, page: 1 }));
    fixtures::answer(d, call, Ok(design_ancestry(forward)))
}

#[test]
fn design_pushes_are_owned_but_other_identities_and_rewrites_still_drift() {
    for (actor, forward, drifted) in [(7, true, false), (8, true, true), (7, false, true)] {
        let mut d = watched_goal();
        design_claim(&mut d);
        // A slot writer ending leaves the design writer beside it.
        confirm_head(&mut d, fixtures::HEAD);
        let requests = design_move(&mut d, actor, forward);
        assert_eq!(d.hold(&name()).unwrap().drift.is_some(), drifted, "{actor}/{forward}: {requests:?}");
        assert_eq!(d.branch_head(&name()), Some(fixtures::HEAD), "an observation keeps the baseline");
    }
}

#[test]
fn a_restored_design_writer_survives_until_loss_and_its_fresh_read_detects_rewrites() {
    for forward in [true, false] {
        let mut d = held_goal();
        design_claim(&mut d);
        let hold = d.hold(&name()).unwrap().clone();
        let mut d = domain();
        outputs(&mut d, Event::Restore { record: Stored::Repository(adopted()) });
        outputs(&mut d, Event::Restore { record: Stored::Hold(hold.clone()) });
        outputs(
            &mut d,
            Event::Restore {
                record: Stored::BranchHead(BranchHead { name: name(), commit: fixtures::HEAD, owned: true }),
            },
        );
        outputs(&mut d, Event::Restore { record: Stored::Names { task: GOAL, resources: Box::new([name()]) } });
        outputs(&mut d, Event::Restored { clock: skein_forge_client::RecoveryClock::Monotonic });
        outputs(&mut d, Event::ReadAfresh);
        outputs(&mut d, Event::SettleOutbox);
        fixtures::answer_next(&mut d, Ok(Answer::Commit(fixtures::HEAD)));
        assert_eq!(d.hold(&name()).unwrap(), &hold);
        design_move(&mut d, 7, true);
        assert_eq!(d.hold(&name()).unwrap().drift, None, "restart restores the keep's written flag");
        outputs(&mut d, Event::Lost { task: GOAL, attempt: 1 });
        assert!(d.hold(&name()).unwrap().design_writers[0].rereading, "loss reads the restored design writer afresh");
        let tip = [9; 32];
        fixtures::answer_next(&mut d, Ok(Answer::Commit(tip)));
        fixtures::answer_next(&mut d, Ok(design_ancestry(forward)));
        assert!(d.hold(&name()).unwrap().design_writers.is_empty());
        assert_eq!(d.hold(&name()).unwrap().drift.is_none(), forward);
        assert_eq!(d.branch_head(&name()), Some(if forward { tip } else { fixtures::HEAD }));
    }
}

fn confirm_head(d: &mut Domain, head: skein_forge::api::Commit) {
    outputs(
        d,
        Event::Claim {
            designs: Box::new([]),
            task: DELEGATE,
            attempt: 1,
            writes: Box::new([name()]),
            holders: Box::new([GOAL]),
        },
    );
    outputs(d, Event::Answered { task: DELEGATE, attempt: 1, pushed: Box::new([(name(), head, true)]) });
}

#[test]
fn a_branch_read_before_a_confirmed_move_still_checks_the_new_head_without_drift() {
    let mut d = held_goal();
    outputs(&mut d, Event::Change { row: landing_row() });
    confirm_head(&mut d, fixtures::BASE);
    for head in [fixtures::HEAD, fixtures::BASE] {
        let requests = step_at_head(&mut d, head);
        let row = d.change(fixtures::CHANGE).unwrap();
        assert_eq!(row.change.state, change::State::Checking { head }, "{requests:?}");
        assert_eq!(row.change.repairs, 0);
    }
}

const RUN_BRANCH: &[u8] = b"temper/7/r7";

fn run_name() -> Name {
    Name {
        forge: REPO.forge,
        repository: REPO.repository,
        what: What::Branch(Box::new([b"temper".as_slice().into(), b"7".as_slice().into(), b"r7".as_slice().into()])),
    }
}

fn held_run(holder: u64, writer: Option<Writer>) -> Domain {
    let mut d = held_by(None);
    outputs(&mut d, Event::Hold { entry: 0, task: holder, resource: run_name(), from: None });
    if let Some(Writer::Run { task, attempt }) = writer {
        outputs(
            &mut d,
            Event::Claim {
                designs: Box::new([]),
                task,
                attempt,
                writes: Box::new([run_name()]),
                holders: Box::new([holder]),
            },
        );
    } else {
        d.holds.get_mut(&run_name()).unwrap().writer = writer;
    }
    let mut row = landing_row();
    row.branch = RUN_BRANCH.into();
    outputs(&mut d, Event::Change { row });
    d
}

#[test]
fn a_change_checks_deliveries_beside_its_requesters_run_or_design_writer() {
    let writer = Writer::Run { task: GOAL, attempt: 1 };
    let mut goal = held_goal();
    design_claim(&mut goal);
    outputs(&mut goal, Event::Change { row: landing_row() });
    for (mut d, name, slot) in [(held_run(GOAL, Some(writer)), run_name(), Some(writer)), (goal, name(), None)] {
        let requests = step_at_head(&mut d, fixtures::BASE);
        let row = d.change(fixtures::CHANGE).unwrap();
        assert_eq!(row.change.state, change::State::Checking { head: fixtures::BASE }, "{requests:?}");
        assert_eq!(row.change.last_head, Some(fixtures::BASE));
        assert_eq!(row.drift, None);
        assert_eq!(d.hold(&name).unwrap().writer, slot);
    }
}

#[test]
fn a_run_change_waits_for_every_other_writer() {
    for (holder, writer) in [
        (GOAL, Writer::Run { task: DELEGATE, attempt: 1 }),
        (GOAL, Writer::Entry(UPDATE)),
        (GOAL, Writer::Lost { task: GOAL, attempt: 1 }),
        (fixtures::CHANGE, Writer::Run { task: fixtures::CHANGE, attempt: 1 }),
    ] {
        let mut goal = held_goal();
        design_claim(&mut goal);
        goal.holds.get_mut(&name()).unwrap().writer = Some(writer);
        outputs(&mut goal, Event::Change { row: landing_row() });
        for mut d in [held_run(holder, Some(writer)), goal] {
            let requests = step_at_head(&mut d, fixtures::BASE);
            assert_eq!(
                d.change(fixtures::CHANGE).unwrap().change.state,
                change::State::Checking { head: fixtures::HEAD },
                "{holder}: {writer:?}: {requests:?}"
            );
        }
    }
}
