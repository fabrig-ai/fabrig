use super::fixtures::{self, acts as f};
use crate::{acts, pages::Act};
use jig_client::contract::{self as c, ActKind, Admission, ObjectKey};
const FIRST_CARD: u32 = 0;
const OTHER_PROPOSAL: u64 = 4;
const PROPOSAL: ObjectKey = ObjectKey::Proposal(f::PROPOSAL);
const HELD: ObjectKey = ObjectKey::Escalation(f::TASK);
const PERSON: ObjectKey = ObjectKey::PersonTask(f::TASK);
#[test]
fn accepting_sends_nothing_until_the_confirmation_is_confirmed() {
    let mut pages = f::pages(PROPOSAL, f::proposal(ActKind::Accept, Admission::Allowed));
    assert_eq!(f::sent(&mut f::act(&mut pages, f::decide(PROPOSAL, ActKind::Accept))).len(), 0);
    assert_eq!(
        f::sent(&mut f::act(&mut pages, Act::Confirm)).as_ref(),
        [c::Ask::Decide {
            project: fixtures::PROJECT,
            on: c::DecideOn::Proposal { proposal: f::PROPOSAL, decision: c::Decision::Accepted, running: None }
        }]
    );
}
#[test]
fn a_second_press_in_flight_sends_nothing() {
    let mut pages = f::pages(PROPOSAL, f::proposal(ActKind::Accept, Admission::Allowed));
    f::act(&mut pages, f::decide(PROPOSAL, ActKind::Accept));
    assert_eq!(f::sent(&mut f::act(&mut pages, Act::Confirm)).len(), 1);
    let key = pages.confirmation().unwrap().sent;
    assert_eq!(f::sent(&mut f::act(&mut pages, Act::Confirm)).len(), 0);
    assert_eq!(pages.confirmation().expect("second press leaves the sheet").sent, key);
    assert_eq!(f::sent(&mut f::act(&mut pages, f::decide(PROPOSAL, ActKind::Accept))).len(), 0);
}
#[test]
fn a_page_holds_at_most_one_open_confirmation() {
    let mut pages = f::pages(PROPOSAL, f::proposal(ActKind::Accept, Admission::Allowed));
    f::act(&mut pages, f::decide(PROPOSAL, ActKind::Accept));
    let other = ObjectKey::Proposal(c::ProposalKey { proposer: c::Proposer::Task(f::TASK), number: OTHER_PROPOSAL });
    let mut second = pages.inbox().expect("inbox").cards.get(FIRST_CARD).expect("first card").clone();
    second.object = other;
    pages.inbox.as_mut().expect("inbox").cards.push(second).expect("second card fits");
    f::act(&mut pages, f::decide(other, ActKind::Accept));
    assert_eq!(
        pages.confirmation(),
        Some(&acts::Confirm {
            object: other,
            act: ActKind::Accept,
            revision: Some(f::REVISION),
            again: None,
            sent: None,
            failed: None
        })
    );
}
#[test]
fn a_decision_names_the_revision_its_card_showed() {
    let mut pages = f::pages(HELD, f::held(ActKind::Release));
    f::act(&mut pages, f::decide(HELD, ActKind::Release));
    assert_eq!(
        f::sent(&mut f::act(&mut pages, Act::Confirm)).as_ref(),
        [c::Ask::Release { project: fixtures::PROJECT, task: f::TASK, revision: f::REVISION, again: None }]
    );
}
#[derive(Clone, Copy)]
enum Direct {
    Answer,
    LeaveHeld,
    Take,
    HandBack,
}
impl Direct {
    const fn act(self) -> ActKind {
        match self {
            Self::Answer => ActKind::Answer,
            Self::LeaveHeld => ActKind::LeaveHeld,
            Self::Take => ActKind::Take,
            Self::HandBack => ActKind::HandBack,
        }
    }
}
const CASES: [(Direct, ObjectKey); 4] =
    [(Direct::Answer, f::QUESTION), (Direct::LeaveHeld, HELD), (Direct::Take, PERSON), (Direct::HandBack, PERSON)];
#[test]
fn each_inbox_act_is_one_keyed_request() {
    for (case, object) in CASES {
        let act = case.act();
        let page = match case {
            Direct::Answer => f::question(),
            Direct::LeaveHeld => f::held(act),
            Direct::Take | Direct::HandBack => f::person(act),
        };
        let mut pages = f::pages(object, page);
        f::act(&mut pages, Act::CardWords { object, words: Box::from(*b"Because") });
        let expected = match case {
            Direct::Answer => c::Ask::Answer {
                project: fixtures::PROJECT,
                on: c::AnswerOn::Question { task: f::TASK, message: f::MESSAGE, words: f::text(b"Because") },
            },
            Direct::LeaveHeld => c::Ask::Decide {
                project: fixtures::PROJECT,
                on: c::DecideOn::Escalation {
                    task: f::TASK,
                    revision: f::REVISION,
                    decision: c::Decision::LeftHeld { reason: f::text(b"Because") },
                },
            },
            Direct::Take => c::Ask::Take { project: fixtures::PROJECT, task: f::TASK },
            Direct::HandBack => c::Ask::HandBack { project: fixtures::PROJECT, task: f::TASK },
        };
        let mut out = f::act(&mut pages, f::decide(object, act));
        assert_eq!(f::sent(&mut out).as_ref(), [expected], "{act:?}");
        assert_eq!(f::sent(&mut f::act(&mut pages, f::decide(object, act))).len(), 0, "{act:?}");
    }
}
#[test]
fn an_act_made_offline_shows_parked_until_the_link_is_live() {
    let mut client = f::live_client();
    let mut out = skein_lib::Queue::uncapped();
    client.step(skein_lib::Time::ZERO, skein_lib::Wall::EPOCH, jig_client::Event::StreamGone, &mut out);
    assert_eq!(client.link().state, jig_client::Link::Offline);
    let mut pages = f::pages(PROPOSAL, f::proposal(ActKind::Accept, Admission::Allowed));
    f::act(&mut pages, f::decide(PROPOSAL, ActKind::Accept));
    let mut asks = f::act(&mut pages, Act::Confirm);
    while let Some(request) = asks.pop() {
        if let crate::pages::Request::Client(event) = request {
            client.step(skein_lib::Time::ZERO, skein_lib::Wall::EPOCH, event, &mut out);
        }
    }
    let card = pages.inbox().expect("inbox").cards.get(FIRST_CARD).expect("card");
    assert_eq!(acts::pending(card, &client), Some(jig_client::Progress::Parked));
}
fn terminal(outcome: jig_client::Outcome) -> crate::pages::Pages {
    let mut pages = f::pages(PROPOSAL, f::proposal(ActKind::Accept, Admission::Allowed));
    f::act(&mut pages, f::decide(PROPOSAL, ActKind::Accept));
    f::act(&mut pages, Act::Confirm);
    let key = pages.inbox().expect("inbox").cards.get(FIRST_CARD).expect("card").key.expect("key");
    assert!(acts::told(&mut pages, &jig_client::Told::Answered { key, outcome }));
    pages
}
#[test]
fn a_request_not_stored_says_not_sent_on_its_card() {
    let pages = terminal(jig_client::Outcome::NotKept);
    assert_eq!(
        pages.inbox().expect("inbox").cards.get(FIRST_CARD).expect("card").outcome,
        Some(jig_client::Outcome::NotKept)
    );
}
#[test]
fn an_answer_unknown_past_the_retention_says_so_on_the_card() {
    let pages = terminal(jig_client::Outcome::Unknown);
    assert_eq!(
        pages.inbox().expect("inbox").cards.get(FIRST_CARD).expect("card").outcome,
        Some(jig_client::Outcome::Unknown)
    );
}
#[test]
fn a_refused_act_shows_jigs_words_and_sends_nothing_more() {
    let outcome = jig_client::Outcome::Answered(jig_client::Answered::Jig(c::Answer::Refused {
        refusal: c::Refusal::NotOffered,
        at: None,
    }));
    let mut pages = terminal(outcome.clone());
    assert_eq!(pages.inbox().expect("inbox").cards.get(FIRST_CARD).expect("card").outcome, Some(outcome));
    assert_eq!(f::sent(&mut f::act(&mut pages, f::decide(PROPOSAL, ActKind::Accept))).len(), 0);
}
#[test]
fn a_deciding_card_survives_its_removal_from_inbox() {
    let mut pages = f::pages(HELD, f::held(ActKind::Release));
    f::act(&mut pages, f::decide(HELD, ActKind::Release));
    f::act(&mut pages, Act::Confirm);
    let entries = pages.inbox().expect("inbox").entries;
    let limits = fixtures::limits();
    let mut out = skein_lib::Queue::uncapped();
    crate::pages::step(
        &mut pages,
        &fixtures::env(&limits),
        crate::unlinked(&limits, fixtures::SEED).client(),
        crate::pages::Event::Told { told: jig_client::Told::Changed { handle: entries } },
        &mut out,
    );
    assert_eq!(pages.inbox().expect("inbox").cards.len(), 1);
}

#[test]
fn cancelling_drops_the_pages_open_confirmation_without_sending() {
    let mut pages = f::pages(PROPOSAL, f::proposal(ActKind::Accept, Admission::Allowed));
    f::act(&mut pages, f::decide(PROPOSAL, ActKind::Accept));
    assert!(f::sent(&mut f::act(&mut pages, Act::Cancel)).is_empty());
    assert_eq!(pages.confirmation(), None);
}

#[test]
fn card_words_keep_a_persons_whole_reply_to_the_receiving_bound() {
    let mut limits = fixtures::limits();
    limits.client.domain.text = 2048;
    limits.client.domain.long_text = 2048;
    limits.client.domain.turn_text = 2048;
    let mut pages = f::pages(PROPOSAL, f::proposal(ActKind::Reject, Admission::Allowed));
    let mut out = skein_lib::Queue::with_capacity(32);
    let bound = usize::try_from(limits.client.domain.text).unwrap();
    let mut words = skein_lib::bytes::zeroed(bound);
    words.fill(b'a');
    let mut over = skein_lib::bytes::zeroed(bound.checked_add(1).unwrap());
    over.fill(b'b');
    acts::step(
        &mut pages,
        &fixtures::env(&limits),
        Act::CardWords { object: PROPOSAL, words: words.clone() },
        &mut out,
    );
    acts::step(&mut pages, &fixtures::env(&limits), Act::CardWords { object: PROPOSAL, words: over }, &mut out);
    assert_eq!(pages.inbox().unwrap().cards.get(FIRST_CARD).unwrap().words.as_ref().unwrap().bytes(), &*words);
}

fn pressed_sheet() -> crate::pages::Pages {
    let mut pages = f::pages(PROPOSAL, f::proposal(ActKind::Accept, Admission::Allowed));
    f::act(&mut pages, f::decide(PROPOSAL, ActKind::Accept));
    f::act(&mut pages, Act::Confirm);
    pages
}
fn sheet_answer(pages: &mut crate::pages::Pages, key: jig_client::Key, outcome: jig_client::Outcome) {
    let limits = fixtures::limits();
    crate::pages::step(
        pages,
        &fixtures::env(&limits),
        crate::unlinked(&limits, fixtures::SEED).client(),
        crate::pages::Event::Told { told: jig_client::Told::Answered { key, outcome } },
        &mut skein_lib::Queue::uncapped(),
    );
}
#[test]
fn the_sheet_stays_until_its_ask_is_answered() {
    let mut pages = pressed_sheet();
    let key = pages.confirmation().expect("sheet stays").sent.expect("sent");
    let other = jig_client::Key { asker: key.asker, number: key.number.saturating_add(1) };
    sheet_answer(&mut pages, other, jig_client::Outcome::Unknown);
    assert_eq!(pages.confirmation().expect("another answer leaves the sheet").sent, Some(key));
    sheet_answer(
        &mut pages,
        key,
        jig_client::Outcome::Answered(jig_client::Answered::Jig(c::Answer::Done(c::Outcome::Document(
            c::DocumentOutcome::Amended { task: f::TASK },
        )))),
    );
    assert!(pages.confirmation().is_none());
}
#[test]
fn a_failed_ask_keeps_the_sheet_and_presses_send_again() {
    let mut pages = pressed_sheet();
    let key = pages.confirmation().expect("sheet").sent.expect("sent");
    sheet_answer(&mut pages, key, jig_client::Outcome::NotKept);
    let open = pages.confirmation().expect("failed sheet stays");
    assert_eq!(open.failed, Some(jig_client::Outcome::NotKept));
    assert_eq!(open.sent, None);
    assert_eq!(f::sent(&mut f::act(&mut pages, Act::Confirm)).len(), 1);
    let open = pages.confirmation().expect("retry stays");
    assert_ne!(open.sent, Some(key));
    assert!(open.failed.is_none());
}
#[test]
fn the_keep_closes_it_with_an_ask_in_flight() {
    let mut pages = pressed_sheet();
    let key = pages.confirmation().expect("sheet").sent.expect("sent");
    assert!(f::sent(&mut f::act(&mut pages, Act::Cancel)).is_empty());
    assert!(pages.confirmation().is_none());
    sheet_answer(&mut pages, key, jig_client::Outcome::NotKept);
    assert_eq!(
        pages.inbox().expect("inbox").cards.get(FIRST_CARD).expect("card").outcome,
        Some(jig_client::Outcome::NotKept)
    );
}
#[test]
fn a_press_that_sends_nothing_closes_the_sheet() {
    for (revision, effect) in [(None, c::ReleaseEffect::Judged), (Some(f::REVISION), c::ReleaseEffect::Uncertain)] {
        let mut page = f::held(ActKind::Release);
        let c::ReadValue::Escalation(held) = &mut page else { panic!("held fixture") };
        held.acts.get_mut(0).unwrap().effect = Some(c::Effect::Release(effect));
        let mut pages = f::pages(HELD, page);
        f::act(&mut pages, Act::Decide { object: HELD, act: ActKind::Release, revision });
        assert!(f::sent(&mut f::act(&mut pages, Act::Confirm)).is_empty());
        assert!(pages.confirmation().is_none());
    }
}
