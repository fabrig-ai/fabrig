//! Release fences watched answers before gathering again (keeping-up.md, 3).
use super::*;
use skein_forge::api::{self, Answer, Op, Read};
use temper_engine_domain_forge_change as change;

fn held_pull() -> (Domain, Name) {
    let mut d = fixtures::connected_domain();
    outputs(&mut d, Event::Restore { record: Stored::Repository(adopted()) });
    let branch = Name {
        forge: REPO.forge,
        repository: REPO.repository,
        what: What::Branch(Box::new([b"temper".as_slice().into(), b"51".as_slice().into()])),
    };
    outputs(
        &mut d,
        Event::Restore {
            record: Stored::Hold(Hold {
                name: branch.clone(),
                task: fixtures::CHANGE,
                writer: None,
                design_writers: Box::new([]),
                drift: Some(Drift { at: env().wall, change: DriftChange::PullClosed { number: 1 } }),
            }),
        },
    );
    let mut row = fixtures::opening_row();
    row.pending = None;
    row.effect = change::EffectResult::None;
    row.pull = Some(1);
    row.drift = Some(change::Hold::PullClosed);
    row.change.state = change::State::Held {
        was: Box::new(change::State::Checking { head: fixtures::HEAD }),
        why: change::Hold::PullClosed,
    };
    outputs(&mut d, Event::Restore { record: Stored::Change(Box::new(row)) });
    outputs(&mut d, Event::Restored { clock: skein_forge_client::RecoveryClock::Monotonic });
    outputs(&mut d, Event::ReadAfresh);
    outputs(&mut d, Event::SettleOutbox);
    outputs(
        &mut d,
        Event::Names { task: fixtures::CHANGE, resources: Box::new([Name { what: What::Pull(1), ..branch.clone() }]) },
    );
    (d, branch)
}

#[test]
fn a_late_watched_closed_pull_cannot_hold_a_released_change() {
    let (mut d, branch) = held_pull();
    let (listing, op) = fixtures::next_call(&mut d);
    assert!(matches!(op, Op::Read(Read::Items { .. })));
    fixtures::answer(
        &mut d,
        listing,
        Ok(Answer::Items { cut: false, items: Box::new([]), more: false, now: env().now }),
    );
    let (item, op) = fixtures::next_call(&mut d);
    assert!(matches!(op, Op::Read(Read::Item { number: 1, .. })));
    fixtures::answer(
        &mut d,
        item,
        Ok(Answer::Item {
            item: api::Summary {
                number: 1,
                kind: api::Kind::Pull,
                state: api::State::Open,
                author: 7,
                key: None,
                title: Box::new([]),
                body: Box::new([]),
                labels: Box::new([]),
                updated: env().now,
            },
            comments: Box::new([]),
            more: false,
        }),
    );
    let (old, op) = fixtures::next_call(&mut d);
    assert_eq!(op, Op::Read(Read::Pull { number: 1, ci: true }));
    outputs(
        &mut d,
        Event::ReleaseChange { task: fixtures::CHANGE, release: Some(TaskRelease { revision: 1, at: env().wall }) },
    );
    let mut closed = fixtures::open_pull();
    closed.state = api::State::Closed;
    let late = fixtures::answer(&mut d, old, Ok(Answer::Pull(closed.clone())));
    for request in &late {
        assert!(!matches!(request, Request::Drift { .. }), "{late:?}");
    }
    assert_eq!(d.hold(&branch).expect("branch held").drift, None);
    let row = d.change(fixtures::CHANGE).expect("released change");
    assert_eq!(row.drift, None);
    assert_eq!(row.change.state, change::State::Checking { head: fixtures::HEAD });

    fixtures::step_change(&mut d);
    for (read, answer) in [
        (Read::Branch { branch: b"temper/51".as_slice().into() }, Answer::Commit(fixtures::HEAD)),
        (Read::Branch { branch: b"main".as_slice().into() }, Answer::Commit(fixtures::BASE)),
        (Read::Pull { number: 1, ci: true }, Answer::Pull(fixtures::open_pull())),
    ] {
        let (call, op) = fixtures::next_call(&mut d);
        assert_eq!(op, Op::Read(read), "the release decides from fresh reads");
        fixtures::answer(&mut d, call, Ok(answer));
    }
    fixtures::finish_open(&mut d);
    let decided = fixtures::answer_next(&mut d, Ok(Answer::Reviews { reviews: Box::new([]), more: false, cut: false }));
    let mut continued = false;
    for request in &decided {
        if let Request::ChangeDecision { task, decision, .. } = request {
            assert_eq!(*task, fixtures::CHANGE);
            assert!(!matches!(decision, change::Decision::Hold(_)), "{decided:?}");
            continued = true;
        }
    }
    assert!(continued, "fresh reads continue recovery: {decided:?}");
    assert_eq!(d.change(fixtures::CHANGE).expect("recovering change").drift, None);
    assert!(!matches!(d.change(fixtures::CHANGE).unwrap().change.state, change::State::Held { .. }));
}
