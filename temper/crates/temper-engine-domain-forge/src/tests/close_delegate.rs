//! Passing the two tips of a close attempt (temper/docs/design/domain/forge/change/repairs-releases.md, 1).
use super::*;
use skein_forge::api::{Answer, Error};
use temper_engine_domain_forge_change as change;

const CHILD: u64 = 90;
const TIPS: change::CloseAt =
    change::CloseAt { branch: fixtures::HEAD, landing: fixtures::BASE, asked: true, refused: None };
const PREVIOUS: change::CloseAt = change::CloseAt {
    branch: fixtures::BASE,
    landing: fixtures::BASE,
    asked: true,
    refused: Some(change::CloseRefused { counts: None }),
};

fn close_change(close_at: Option<change::CloseAt>) -> Domain {
    let mut d = fixtures::adopted_domain();
    let mut row = fixtures::opening_row();
    row.base = b"goal".as_slice().into();
    row.change.state = change::State::Producing { requested: false };
    row.change.last_head = None;
    row.effect = change::EffectResult::None;
    row.pending = None;
    row.delegate_status = change::Status::Unknown;
    row.close = Some(br#"{"version":1,"goal":7}"#.as_slice().into());
    row.close_at = close_at;
    outputs(&mut d, Event::Change { row });
    d
}

#[test]
fn a_close_decision_carries_both_tips_it_read() {
    let mut d = close_change(None);
    fixtures::step_change(&mut d);
    fixtures::answer_next(&mut d, Err(Error::Missing));
    fixtures::answer_next(&mut d, Ok(Answer::Commit(fixtures::HEAD)));
    let read = fixtures::answer_next(&mut d, Ok(Answer::Commit(fixtures::BASE)));
    let mut decisions = skein_lib::List::uncapped();
    for request in &read {
        if let Request::ChangeDecision { .. } = request {
            decisions.push(request).expect("uncapped");
        }
    }
    assert_eq!(
        decisions.as_slice(),
        &[&Request::ChangeDecision {
            task: fixtures::CHANGE,
            decision: change::Decision::Delegate(change::Delegate::Close),
            entry: None,
            evidence: None,
            close_at: Some(TIPS),
        }]
    );
}

fn admit(kind: change::Delegate, close_at: Option<change::CloseAt>) -> Domain {
    let mut d = close_change(Some(PREVIOUS));
    outputs(&mut d, Event::Delegated { task: fixtures::CHANGE, child: CHILD, kind, close_at });
    d
}

#[test]
fn admitting_a_close_keeps_its_tips_and_delegate() {
    let d = admit(change::Delegate::Close, Some(TIPS));
    let row = d.change(fixtures::CHANGE).expect("change");
    assert_eq!((row.close_at, row.delegate), (Some(TIPS), Some((CHILD, change::Delegate::Close))));
}

#[test]
fn admitting_a_producer_leaves_previous_close_tips_alone() {
    let d = admit(change::Delegate::Produce, None);
    assert_eq!(d.change(fixtures::CHANGE).expect("change").close_at, Some(PREVIOUS));
}
