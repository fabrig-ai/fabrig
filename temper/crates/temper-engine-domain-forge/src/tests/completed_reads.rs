//! Completed changes keep their facts without reading their work again (forge/keeping-up.md, 3).
use super::*;
use skein_forge::api::{Answer, Op, Read};
use temper_engine_domain_forge_change as change;

fn checking() -> Domain {
    let mut d = fixtures::connected_domain();
    outputs(&mut d, Event::Restore { record: Stored::Repository(adopted()) });
    let mut row = fixtures::opening_row();
    row.pull = Some(1);
    row.pending = None;
    row.change.state = change::State::Checking { head: fixtures::HEAD };
    outputs(&mut d, Event::Change { row });
    d
}

#[test]
fn a_late_fresh_change_read_is_dropped_after_completion() {
    let mut d = checking();
    outputs(&mut d, Event::Restored { clock: skein_forge_client::RecoveryClock::Monotonic });
    outputs(&mut d, Event::ReadAfresh);
    outputs(&mut d, Event::SettleOutbox);
    fixtures::step_change(&mut d);
    let (call, op) = fixtures::next_call(&mut d);
    assert_eq!(op, Op::Read(Read::Branch { branch: b"temper/51".as_slice().into() }));
    d.changes.get_mut(&fixtures::CHANGE).expect("change").change.state =
        change::State::Landed { since: env().wall, merge: fixtures::BASE };
    let answered = fixtures::answer(&mut d, call, Ok(Answer::Commit(fixtures::HEAD)));
    let mut decided = false;
    for request in &answered {
        if let Request::Save { record: Stored::BranchHead(_), .. } = request {
            panic!("the completed change drops fresh facts: {answered:?}");
        }
        if let Request::ChangeDecision { task: fixtures::CHANGE, .. } = request {
            decided = true;
        }
    }
    assert!(decided, "verification still answers the core's step");
    assert!(ready(&mut d).is_empty(), "the completed change reads no base or brief");
    assert!(d.steps.is_empty());
}

#[test]
fn a_late_branch_comparison_for_completed_work_is_dropped() {
    for state in [change::State::Landed { since: env().wall, merge: fixtures::BASE }, change::State::Ended] {
        let mut d = checking();
        let name = Name {
            forge: REPO.forge,
            repository: REPO.repository,
            what: What::Branch(Box::new([b"temper".as_slice().into(), b"51".as_slice().into()])),
        };
        outputs(
            &mut d,
            Event::Restore {
                record: Stored::BranchHead(BranchHead { name: name.clone(), commit: fixtures::HEAD, owned: false }),
            },
        );
        outputs(&mut d, Event::Restored { clock: skein_forge_client::RecoveryClock::Monotonic });
        outputs(&mut d, Event::ReadAfresh);
        outputs(&mut d, Event::SettleOutbox);
        let named = outputs(&mut d, Event::Names { task: fixtures::CHANGE, resources: Box::new([name]) });
        assert!(!named.contains(&Request::Refused { task: fixtures::CHANGE }), "names accepted: {named:?}");
        outputs(
            &mut d,
            Event::Subscribe {
                subscription: Subscriber {
                    task: 7,
                    number: 1,
                    topic: Topic::Landings { repository: REPO, branch: b"temper/51".as_slice().into() },
                    own_change: None,
                    goal_tasks: None,
                    paths: Box::new([]),
                },
            },
        );
        let (call, op) = fixtures::next_call(&mut d);
        assert_eq!(op, Op::Read(Read::Branch { branch: b"temper/51".as_slice().into() }));
        fixtures::answer(&mut d, call, Ok(Answer::Commit(fixtures::BASE)));
        let (call, op) = fixtures::next_call(&mut d);
        assert_eq!(op, Op::Read(Read::Compare { before: fixtures::HEAD, after: fixtures::BASE }));
        d.changes.get_mut(&fixtures::CHANGE).expect("change").change.state = state;
        let answered = fixtures::answer(
            &mut d,
            call,
            Ok(Answer::Compare {
                before: fixtures::HEAD,
                after: fixtures::BASE,
                files: Box::new([]),
                ahead: 1,
                behind: 0,
                changed: 0,
            }),
        );
        assert!(answered.is_empty(), "no late facts or news for completed work: {answered:?}");
        assert!(ready(&mut d).is_empty(), "the dropped answer starts no read");
    }
}
