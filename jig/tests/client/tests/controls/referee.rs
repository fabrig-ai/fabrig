//! Referee controls over observations, without a world
//! (jig/docs/design/domain/testing.md, 9).
use jig_client_world::harness::*;
use skein_lib::{Duration, Time, Token};
use skein_ui::{NodeId, Tree};

#[test]
fn me_cannot_unpark_a_client_running_outdated_code() {
    let mut referee = skein_world::domain::Referee::new(referee::Rules::new(
        Duration::from_millis(10),
        Duration::from_millis(10),
        Duration::from_millis(10),
    ));
    let client = ClientId(0);
    let mut out = Vec::new();
    referee.observe(Time::ZERO, Observation::Client { client }, &mut out);
    referee.observe(Time::ZERO, Observation::Truth(Observed::Reachable { live: true }), &mut out);
    referee.observe(Time::ZERO, Observation::Delivered { client, receipt: Receipt::OutOfDate }, &mut out);
    referee.observe(
        Time::ZERO,
        Observation::Delivered { client, receipt: Receipt::Party { party: 7, watch: Some(Token::new(1)) } },
        &mut out,
    );
    referee.fire(Time::from_nanos(20_000_000), &mut out);
    assert_eq!(referee.verdict(), skein_world::domain::Verdict::Passed);
    referee.observe(
        Time::from_nanos(20_000_000),
        Observation::Delivered { client, receipt: Receipt::Started },
        &mut out,
    );
    referee.observe(
        Time::from_nanos(20_000_000),
        Observation::Delivered { client, receipt: Receipt::Party { party: 7, watch: Some(Token::new(1)) } },
        &mut out,
    );
    assert_eq!(referee.next_deadline(), Some(Time::from_nanos(30_000_000)));
    referee.observe(
        Time::from_nanos(20_000_000),
        Observation::Shown { client, seen: vec![Seen::Link { live: true }] },
        &mut out,
    );
    assert_eq!(referee.verdict(), skein_world::domain::Verdict::Passed);
}

/// A referee of 10 ms heartbeats whose link comes back within 40 ms, with one
/// tab admitted at zero while the engine is `live` or not.
fn reloading(live: bool) -> (skein_world::domain::Referee<referee::Rules>, ClientId) {
    let span = Duration::from_millis(10);
    let mut referee = skein_world::domain::Referee::new(referee::Rules::new(span, span, Duration::from_millis(40)));
    let client = ClientId(0);
    let mut out = Vec::new();
    referee.observe(Time::ZERO, Observation::Client { client }, &mut out);
    referee.observe(Time::ZERO, Observation::Truth(Observed::Reachable { live }), &mut out);
    referee.observe(Time::ZERO, Observation::Shown { client, seen: vec![Seen::Link { live }] }, &mut out);
    (referee, client)
}

#[test]
fn a_reloaded_tab_owes_its_link_afresh() {
    let (mut referee, client) = reloading(false);
    let mut out = Vec::new();
    referee.observe(Time::from_nanos(5_000_000), Observation::Reloaded { client }, &mut out);
    assert_eq!(referee.next_deadline(), Some(Time::from_nanos(45_000_000)));
    referee.fire(Time::from_nanos(20_000_000), &mut out);
    referee.observe(
        Time::from_nanos(30_000_000),
        Observation::Shown { client, seen: vec![Seen::Link { live: false }] },
        &mut out,
    );
    referee.fire(Time::from_nanos(50_000_000), &mut out);
    assert_eq!(referee.verdict(), skein_world::domain::Verdict::Passed);
}

#[test]
fn a_reloaded_tab_never_told_its_link_is_caught() {
    for live in [false, true] {
        let (mut referee, client) = reloading(live);
        let mut out = Vec::new();
        referee.observe(Time::from_nanos(5_000_000), Observation::Reloaded { client }, &mut out);
        referee.fire(Time::from_nanos(50_000_000), &mut out);
        assert!(
            matches!(referee.verdict(), skein_world::domain::Verdict::Failed(_)),
            "the earlier page's link stood for the reloaded one, live: {live}"
        );
    }
}

#[test]
fn a_tree_breaking_the_outline_fails_the_referee() {
    let mut referee = skein_world::domain::Referee::new(referee::Rules::new(
        Duration::from_millis(10),
        Duration::from_millis(10),
        Duration::from_millis(10),
    ));
    let mut out = Vec::new();
    referee.observe(
        Time::ZERO,
        Observation::Shown {
            client: ClientId(0),
            seen: vec![Seen::Outline {
                fault: skein_ui::outline::Fault { rule: skein_ui::outline::Rule::OneMain, node: 2 },
            }],
        },
        &mut out,
    );
    assert_eq!(
        referee.verdict(),
        skein_world::domain::Verdict::Failed(skein_world::domain::Failure {
            at: Time::ZERO,
            why: "the page's outline: a tree broke skein's ui/README.md, 12".to_owned(),
            pending: vec![],
        })
    );
}

const WATCH_ENDS: &[jig_client_domain::contract::WatchEnd] = &[
    jig_client_domain::contract::WatchEnd::Closed,
    jig_client_domain::contract::WatchEnd::Dropped,
    jig_client_domain::contract::WatchEnd::Busy,
    jig_client_domain::contract::WatchEnd::Stopping,
    jig_client_domain::contract::WatchEnd::Unknown,
    jig_client_domain::contract::WatchEnd::NotAllowed,
    jig_client_domain::contract::WatchEnd::SubjectEnded,
    jig_client_domain::contract::WatchEnd::SignedOut,
];
#[test]
fn me_after_its_watch_ends_is_no_evidence() {
    use jig_client::Up;
    use jig_client_domain::contract::{Delivery, WatchEvent};
    use jig_client_world::observe::up_received;
    const WATCH: Token = Token::new(1);
    for end in WATCH_ENDS {
        let span = Duration::from_millis(10);
        let mut referee = skein_world::domain::Referee::new(referee::Rules::new(span, span, span));
        let client = ClientId(0);
        let mut out = Vec::new();
        referee.observe(Time::ZERO, Observation::Client { client }, &mut out);
        referee.observe(Time::ZERO, Observation::Truth(Observed::Reachable { live: true }), &mut out);
        referee.observe(Time::ZERO, Observation::Delivered { client, receipt: Receipt::Parked }, &mut out);
        for event in [
            Up::WatchEnded { watch: WATCH, end: *end },
            Up::Watched {
                watch: WATCH,
                event: WatchEvent::Delivered(Delivery::Me(Box::new(jig_client_world::stories::frame(7)))),
            },
        ] {
            for receipt in up_received(&event) {
                referee.observe(Time::ZERO, Observation::Delivered { client, receipt }, &mut out);
            }
        }
        referee.fire(Time::from_nanos(20_000_000), &mut out);
        assert_eq!(referee.verdict(), skein_world::domain::Verdict::Passed, "{end:?}");
    }
}

#[test]
fn a_sign_out_shown_done_before_the_engine_received_it_is_caught() {
    let span = Duration::from_millis(10);
    let mut referee = skein_world::domain::Referee::new(referee::Rules::new(span, span, span));
    let client = ClientId(0);
    let mut out = Vec::new();
    referee.observe(Time::ZERO, Observation::Client { client }, &mut out);
    referee.observe(
        Time::ZERO,
        Observation::Delivered { client, receipt: Receipt::Party { party: 7, watch: None } },
        &mut out,
    );
    referee.observe(Time::ZERO, Observation::SignOutAsked { client }, &mut out);
    referee.observe(Time::ZERO, Observation::Shown { client, seen: vec![Seen::SignedOut] }, &mut out);
    assert_eq!(
        referee.verdict(),
        skein_world::domain::Verdict::Failed(skein_world::domain::Failure {
            at: Time::ZERO,
            why: "a sign-out told: the engine did not receive sign-out".to_owned(),
            pending: vec![]
        })
    );
}

#[test]
fn a_sign_in_the_engine_ended_shows_the_sign_in_page_after_a_lost_sign_out() {
    let span = Duration::from_millis(10);
    let mut referee = skein_world::domain::Referee::new(referee::Rules::new(span, span, span));
    let client = ClientId(0);
    let mut out = Vec::new();
    referee.observe(Time::ZERO, Observation::Truth(Observed::SignInEnded { client }), &mut out);
    referee.observe(Time::ZERO, Observation::SignOutAsked { client }, &mut out);
    referee.observe(Time::ZERO, Observation::Shown { client, seen: vec![Seen::SignedOut] }, &mut out);
    assert_eq!(referee.verdict(), skein_world::domain::Verdict::Passed);
}

const DRAFT_SPAN: Duration = Duration::from_millis(10);
const DRAFT_CLIENT: ClientId = ClientId(0);
const DRAFT_KEY: jig_client::Key = jig_client::Key { asker: jig_client::Asker::JIG, number: 47 };
const DRAFT_ATTEMPT: Token = Token::new(DRAFT_KEY.number);
fn draft_answer(offer: jig_client::shapes::Fall) -> skein_world::domain::Verdict {
    let mut referee = skein_world::domain::Referee::new(referee::Rules::new(DRAFT_SPAN, DRAFT_SPAN, DRAFT_SPAN));
    let mut out = Vec::new();
    referee.observe(
        Time::ZERO,
        Observation::Shown { client: DRAFT_CLIENT, seen: vec![Seen::Draft { key: DRAFT_KEY, offer }] },
        &mut out,
    );
    referee.observe(
        Time::ZERO,
        Observation::Delivered {
            client: DRAFT_CLIENT,
            receipt: Receipt::DraftAnswer {
                attempt: DRAFT_ATTEMPT,
                outcome: DraftOutcome::Proposed(jig_client_world::stories::proposal().key),
            },
        },
        &mut out,
    );
    referee.observe(
        Time::ZERO,
        Observation::Delivered {
            client: DRAFT_CLIENT,
            receipt: Receipt::Document {
                value: Box::new(jig_client::contract::ReadValue::Proposal(Box::new(
                    jig_client_world::stories::proposal(),
                ))),
            },
        },
        &mut out,
    );
    referee.verdict()
}
#[test]
fn a_draft_within_its_offer_answered_proposed_fails_offers_kept() {
    assert_eq!(
        draft_answer(jig_client::shapes::Fall::Within),
        skein_world::domain::Verdict::Failed(skein_world::domain::Failure {
            at: Time::ZERO,
            why: "offers kept: draft answer differs from its offer".to_owned(),
            pending: vec![],
        })
    );
}
#[test]
fn a_goal_past_the_first_bound_answered_proposed_keeps_the_offer() {
    assert_eq!(
        draft_answer(jig_client::shapes::Fall::Proposable(jig_client::contract::Recipient::Party(7))),
        skein_world::domain::Verdict::Passed
    );
}
#[test]
fn a_draft_proposed_to_another_recipient_fails_offers_kept() {
    assert_eq!(
        draft_answer(jig_client::shapes::Fall::Proposable(jig_client::contract::Recipient::Party(8))),
        skein_world::domain::Verdict::Failed(skein_world::domain::Failure {
            at: Time::ZERO,
            why: "offers kept: proposed to a different recipient".to_owned(),
            pending: vec![],
        })
    );
}

fn budget_refusal(
    budget: jig_client::contract::NewBudget,
    refusal: jig_client::contract::Refusal,
) -> skein_world::domain::Verdict {
    use jig_client::contract::{Ask, ReadValue};
    let mut referee = skein_world::domain::Referee::new(referee::Rules::new(DRAFT_SPAN, DRAFT_SPAN, DRAFT_SPAN));
    let mut out = Vec::new();
    let me = jig_client_world::stories::frame(7);
    let mut ask = jig_client::shapes::chat(&me, 1, jig_client_world::stories::text(b"chat")).expect("chat offered");
    let Ask::Create { tasks, .. } = &mut ask else { panic!("chat creates a task") };
    tasks.get_mut(0).expect("one chat").budget = budget;
    referee.observe(
        Time::ZERO,
        Observation::Delivered {
            client: DRAFT_CLIENT,
            receipt: Receipt::Document { value: Box::new(ReadValue::Me(Box::new(me))) },
        },
        &mut out,
    );
    referee.observe(Time::ZERO, Observation::Shown { client: DRAFT_CLIENT, seen: vec![] }, &mut out);
    referee.observe(
        Time::ZERO,
        Observation::Decision {
            client: DRAFT_CLIENT,
            key: DRAFT_KEY,
            ask: Box::new(ask),
            scene: Some(jig_client_test_kit::check::scene(&Tree::new(0), NodeId(0))),
        },
        &mut out,
    );
    let answer = jig_client::Up::Answer {
        attempt: DRAFT_ATTEMPT,
        envelope: skein_lib::RequestEnvelope::Final,
        answer: Some(jig_client::Answered::Jig(jig_client::contract::Answer::Refused { refusal, at: None })),
        why: None,
    };
    for receipt in jig_client_world::observe::up_received(&answer) {
        referee.observe(Time::ZERO, Observation::Delivered { client: DRAFT_CLIENT, receipt }, &mut out);
    }
    referee.verdict()
}

#[test]
fn an_as_allowed_draft_can_be_refused_when_its_live_pool_is_empty() {
    use jig_client::contract::{Lacking, NewBudget, Refusal};
    assert_eq!(
        budget_refusal(
            NewBudget::AsAllowed { cap: None },
            Refusal::Authority { lacking: Lacking::Budget { needed: 1, left: 0 }, proposable: None },
        ),
        skein_world::domain::Verdict::Passed,
    );
}

#[test]
fn only_an_empty_as_allowed_allotment_excuses_a_refused_draft() {
    use jig_client::contract::{Lacking, NewBudget, Refusal};
    for (budget, lacking, proposable) in [
        (NewBudget::Named, Lacking::Budget { needed: 1, left: 0 }, None),
        (NewBudget::AsAllowed { cap: None }, Lacking::Funding, None),
        (NewBudget::AsAllowed { cap: None }, Lacking::Budget { needed: 2, left: 0 }, None),
        (NewBudget::AsAllowed { cap: None }, Lacking::Budget { needed: 1, left: 0 }, Some(true)),
    ] {
        assert_eq!(
            budget_refusal(budget, Refusal::Authority { lacking, proposable }),
            skein_world::domain::Verdict::Failed(skein_world::domain::Failure {
                at: Time::ZERO,
                why: "offers kept: draft answer differs from its offer".to_owned(),
                pending: vec![],
            }),
        );
    }
}
