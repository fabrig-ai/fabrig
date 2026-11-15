//! Frozen facts leave live owners judging their own resources (keeping-up.md, 3).
use super::*;
use crate::tests::{adopted, env, fixtures, outputs};

const GOAL: u64 = 7;
const LIVE: u64 = 52;

fn name() -> Name {
    Name {
        forge: fixtures::opening_row().repository.forge,
        repository: fixtures::opening_row().repository.repository,
        what: What::Branch(Box::new([b"temper".as_slice().into(), b"51".as_slice().into()])),
    }
}

fn held() -> Domain {
    let mut d = fixtures::connected_domain();
    outputs(&mut d, Event::Restore { record: Stored::Repository(adopted()) });
    outputs(&mut d, Event::Restored { clock: skein_forge_client::RecoveryClock::Monotonic });
    outputs(&mut d, Event::ReadAfresh);
    outputs(&mut d, Event::SettleOutbox);
    outputs(&mut d, Event::Hold { entry: 0, task: GOAL, resource: name(), from: None });
    d.heads.insert(name(), BranchHead { name: name(), commit: fixtures::HEAD, owned: true }).unwrap();
    d
}

#[test]
fn frozen_pulls_neither_drift_the_shared_holder_and_live_change_nor_wake_their_owner() {
    for mut frozen in fixtures::frozen_rows() {
        let mut d = held();
        frozen.pull = Some(1);
        let repository = frozen.repository;
        d.changes.insert(frozen.task, frozen).unwrap();
        let mut live = fixtures::opening_row();
        live.task = LIVE;
        live.change.task = LIVE;
        live.pending = None;
        live.pull = Some(2);
        live.change.state = change::State::Checking { head: fixtures::HEAD };
        d.changes.insert(LIVE, live).unwrap();
        let resource = skein_forge_client::Resource { repository, what: skein_forge_client::What::Pull(1) };
        for (state, base) in [
            (skein_forge::api::State::Closed, b"main".as_slice()),
            (skein_forge::api::State::Open, b"main".as_slice()),
            (skein_forge::api::State::Open, b"other".as_slice()),
        ] {
            let mut pull = fixtures::open_pull();
            pull.state = state;
            pull.base = base.into();
            let mut out = Queue::uncapped();
            pull_changed(&mut d, &env(), resource.clone(), pull, &mut out);
            while let Some(request) = out.pop() {
                assert!(!matches!(request, Request::Drift { .. } | Request::WakeProcedure { .. }), "{request:?}");
            }
            assert!(d.procedure_wakes.is_empty());
            assert!(d.hold(&name()).unwrap().drift.is_none());
            assert!(d.change(LIVE).unwrap().drift.is_none());
        }
        // A branch observation wakes its live owner, not the retained ended row.
        wake_head_dependents(&mut d, &name(), Some(fixtures::HEAD), fixtures::BASE);
        assert!(!d.procedure_wakes.contains_key(&fixtures::CHANGE));
        assert!(d.procedure_wakes.contains_key(&LIVE));
        d.procedure_wakes.remove(&LIVE);
        // The live change's own pull still reaches it and the goal.
        let mut pull = fixtures::open_pull();
        pull.number = 2;
        pull.state = skein_forge::api::State::Closed;
        let mut out = Queue::uncapped();
        pull_changed(
            &mut d,
            &env(),
            skein_forge_client::Resource { repository, what: skein_forge_client::What::Pull(2) },
            pull,
            &mut out,
        );
        let mut drifted = List::uncapped();
        while let Some(request) = out.pop() {
            if let Request::Drift { task, .. } = request {
                drifted.push(task).unwrap();
            }
        }
        assert_eq!(drifted.as_slice(), [GOAL, LIVE]);
    }
}

#[test]
fn frozen_hints_and_head_attribution_skip_each_row_not_its_branch() {
    for mut frozen in fixtures::frozen_rows() {
        let mut d = held();
        frozen.pull = Some(1);
        let repository = frozen.repository;
        d.changes.insert(frozen.task, frozen.clone()).unwrap();
        let hint = skein_forge::api::Hint {
            repository,
            change: skein_forge::api::Change::Branch(frozen.branch.clone()),
            key: None,
        };
        assert!(!hints::concerns(&d, &frozen, &hint));
        assert!(d.changes_for(repository, 10).is_empty());
        assert!(completed_branch(&d, repository, &frozen.branch));
        crate::settle::hinted(&mut d, &env(), &hint, &[fixtures::CHANGE]);
        assert!(!d.settling.contains_key(&fixtures::CHANGE));
        assert_eq!(heads::concerning(&d, &name(), Some(GOAL)), Some(GOAL));
        outputs(&mut d, Event::Hint { hint: hint.clone() });
        assert!(!d.procedure_wakes.contains_key(&fixtures::CHANGE));
        let mut live = fixtures::opening_row();
        live.task = LIVE;
        live.change.task = LIVE;
        d.changes.insert(LIVE, live).unwrap();
        assert_eq!(d.changes_for(repository, 10).as_ref(), [LIVE]);
        assert!(!completed_branch(&d, repository, &frozen.branch));
        assert_eq!(heads::concerning(&d, &name(), Some(GOAL)), Some(LIVE));
        outputs(&mut d, Event::Hint { hint });
        assert!(d.procedure_wakes.contains_key(&LIVE));
        assert!(!d.procedure_wakes.contains_key(&fixtures::CHANGE));
    }
}

#[test]
fn client_drift_of_a_frozen_watch_is_ignored_but_a_goals_own_is_not() {
    for mut frozen in fixtures::frozen_rows() {
        let mut d = held();
        frozen.pull = Some(1);
        let repository = frozen.repository;
        d.changes.insert(frozen.task, frozen).unwrap();
        d.names.insert(GOAL, Box::new([name()])).unwrap();
        let resource = skein_forge_client::Resource {
            repository,
            what: skein_forge_client::What::Branch(b"temper/51".as_slice().into()),
        };
        d.client_out.push(skein_forge_client::Request::Drift {
            resource: resource.clone(),
            expected: fixtures::HEAD,
            observed: Some(fixtures::BASE),
        });
        // The goal owns this watch, so its own branch still drifts.
        let mut out = Queue::uncapped();
        drain(&mut d, &env(), &mut out);
        assert_eq!(out.pop(), Some(Request::Drift { task: GOAL, resource: name() }));
        // A pull-origin drift cannot reach that goal through the branch either.
        let mut out = Queue::uncapped();
        drift(&mut d, &env(), resource.clone(), crate::DriftChange::PullClosed { number: 1 }, &mut out);
        assert!(out.is_empty());
        // A watch belonging only to the frozen row cannot drift the live goal's hold.
        d.names.remove(&GOAL);
        d.names.insert(fixtures::CHANGE, Box::new([name()])).unwrap();
        d.holds.get_mut(&name()).unwrap().drift = None;
        d.client_out.push(skein_forge_client::Request::Drift {
            resource: resource.clone(),
            expected: fixtures::HEAD,
            observed: None,
        });
        let mut out = Queue::uncapped();
        drain(&mut d, &env(), &mut out);
        assert!(out.is_empty());
        let hold = d.holds.get_mut(&name()).unwrap();
        hold.task = fixtures::CHANGE;
        hold.drift = None;
        d.client_out.push(skein_forge_client::Request::Drift { resource, expected: fixtures::HEAD, observed: None });
        let mut out = Queue::uncapped();
        drain(&mut d, &env(), &mut out);
        assert!(out.is_empty());
    }
}
