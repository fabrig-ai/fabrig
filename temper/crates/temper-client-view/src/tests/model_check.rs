//! Model’s line, its states and controls (temper/docs/design/client/pages/README.md).
#![expect(clippy::disallowed_methods, reason = "tests inspect semantic trees and decoded controls")]
use super::{connect_model as model, fixtures as f};
use crate::pages::model_check::{self, Answer, First, State};
use skein_ui::Role;
use temper_client_domain::{
    self as d, Address, Step,
    pages::{accounts::Way, connect_model::Act},
};

#[test]
fn the_welcomes_host_check_precedes_the_trial() {
    let (mut root, trials, watch) = fed(d::WayKind::Program);
    rows(&mut root, trials, &[row(v::TrialStep::Answers, failed(v::TrialWhy::Refused))]);
    let mut doc = d::pages::accounts::document(root.client()).unwrap().clone();
    doc.usable = false;
    doc.settled = false;
    super::accounts::deliver(&mut root, watch, doc.clone());
    let State::Stopped { at: 2, .. } = model_check::trial(&root, &f::words()).unwrap().state else {
        panic!("trial stopped at the answer")
    };
    f::root_event(
        &mut root,
        d::Event::Went {
            place: d::Place::Address(Address::Welcome {
                step: Step::Model { way: Some(d::ModelWay { source: None, provider: 17, kind: d::WayKind::Program }) },
            }),
        },
    );
    let name = root.pages().account_name();
    let attempt = model::sent(&mut root, Act::SignedIn { name, way: Way::Program { provider: 17 } });
    f::root_event(&mut root, d::Event::Went { place: d::Place::Address(Address::Welcome { step: Step::Ready }) });
    let trial = model_check::trial(&root, &f::words()).unwrap();
    let State::Running { at } = trial.state else { panic!("host check is running") };
    assert_eq!(at, 0);
    let limits = f::limits();
    let mut out = Queue::uncapped();
    d::step(
        &mut root,
        &skein_lib::Env { now: skein_lib::Time::ZERO, wall: f::NOW, limits },
        d::Event::Client {
            event: jig_client::Event::Up {
                up: jig_client::Up::Answer {
                    attempt,
                    envelope: skein_lib::RequestEnvelope::Final,
                    why: None,
                    answer: Some(jig_client::Answered::Jig(jig_client::contract::Answer::Done(
                        jig_client::contract::Outcome::Document(
                            jig_client::contract::DocumentOutcome::AccountProgramDeclared {
                                source: 43,
                                name: b"claude-pat".as_slice().into(),
                            },
                        ),
                    ))),
                },
            },
        },
        &mut out,
    );
    let trial = model_check::trial(&root, &f::words()).unwrap();
    let State::Running { at } = trial.state else { panic!("host check is running") };
    assert_eq!(at, 0);
    // The check request is chained after the declaration's answer.
    d::step(
        &mut root,
        &skein_lib::Env { now: skein_lib::Time::ZERO, wall: f::NOW, limits },
        d::Event::Client { event: jig_client::Event::Stored },
        &mut out,
    );
    let attempt = out
        .iter()
        .find_map(|r| match r {
            d::Request::Unlinked {
                request: jig_client::UnlinkedRequest::Down { down: jig_client::Down::Send { attempt, .. } },
            } => Some(*attempt),
            d::Request::Client { .. }
            | d::Request::Unlinked { .. }
            | d::Request::Show { .. }
            | d::Request::Leave { .. }
            | d::Request::SignOut
            | d::Request::Dropped { .. }
            | d::Request::Screen { .. }
            | d::Request::KeepLook { .. }
            | d::Request::KeepSignIn { .. }
            | d::Request::Tab { .. }
            | d::Request::Copy { .. }
            | d::Request::Push(_)
            | d::Request::Fact { .. } => None,
        })
        .unwrap();
    model::answer(&mut root, attempt, jig_client::contract::DocumentOutcome::AccountChanged);
    doc.settled = true;
    super::accounts::deliver(&mut root, watch, doc);
    let trial = model_check::trial(&root, &f::words()).unwrap();
    let First::Subscription = trial.first else { panic!("subscription steps") };
    assert_eq!(trial.account.as_ref(), b"Claude subscription");
    assert_eq!(trial.who.as_ref(), b"Claude");
    let State::Stopped { at, why, answer } = trial.state else { panic!("settled not signed in") };
    assert_eq!(at, 0);
    assert_eq!(why.as_ref(), "Claude Code isn’t signed in here yet. Sign in again, then try again.".as_bytes());
    let Some(Answer::SignIn { way: Way::Program { provider: 17 } }) = answer else {
        panic!("sign-in answers the reason")
    };
    for view in [f::page_view(&root), f::browser_page_view(&root)] {
        position(view.tree(), Role::Heading { level: 1 }, b"Almost ready");
        let create = view
            .tree()
            .nodes()
            .iter()
            .find(|n| n.role == Role::Button && n.name.as_deref() == Some(b"Create your project"))
            .unwrap();
        assert!(view.tree().disabled(create.id));
    }
}

use skein_lib::Queue;
use temper_client_protocol_wire::v1 as v;

#[test]
fn trial_failures_say_why_and_retry_the_account() {
    let (mut root, watch, _) = fed(d::WayKind::Program);
    for (class, words) in [
        (v::TrialClass::Transient, "Claude was busy or out of reach just now."),
        (v::TrialClass::Infrastructure, "Claude was busy or out of reach just now."),
        (v::TrialClass::Permanent, "Claude refused the request. Its account may need attention."),
        (v::TrialClass::Run, "The run stopped before Claude answered."),
        (v::TrialClass::Agent, "The run stopped before Claude answered."),
        (v::TrialClass::Invalid, "The run stopped before Claude answered."),
        (v::TrialClass::Lost, "The machine running it went away."),
        (v::TrialClass::Refused, "The try couldn't start."),
    ] {
        let why =
            v::TrialWhy::Ended(v::TrialEnded::new(&v::CEILINGS, v::TrialEndedParts { class: class.clone() }).unwrap());
        rows(&mut root, watch, &[row(v::TrialStep::Answers, failed(why))]);
        for w in [f::words(), f::browser_words()] {
            let (tree, bindings) = from_root(&root, &w);
            position(&tree, Role::Paragraph, words.as_bytes());
            let mut view =
                if w.shell == crate::words::Shell::Browser { f::browser_page_view(&root) } else { f::page_view(&root) };
            position(view.tree(), Role::Heading { level: 1 }, b"Ready");
            let create = view
                .tree()
                .nodes()
                .iter()
                .find(|n| n.role == Role::Button && n.name.as_deref() == Some(b"Create your project"))
                .unwrap();
            let id = create.id;
            assert!(!view.tree().disabled(id));
            assert_eq!(
                crate::act(&mut view, id, skein_ui::Act::Press),
                Some(d::Action::Pages(d::pages::Act::MakeDefault))
            );
            if w.shell == crate::words::Shell::Terminal {
                assert!(print(&tree).contains(words));
            }
            let retry = &tree.nodes()[position(&tree, Role::Button, b"Try again")];
            let Some(d::Event::Act { action }) = bindings.decode(retry.binding.unwrap().token, skein_ui::Act::Press)
            else {
                panic!("retry acts")
            };
            assert_eq!(action, d::Action::Forge(d::forge::Act::TryAccount { account: 1, again: true }));
            assert_eq!(
                tree.nodes().iter().any(|n| n.name.as_deref() == Some(b"Sign in again")),
                class == v::TrialClass::Permanent
            );
        }
    }
    for (why, words) in [
        (v::TrialWhy::Refused, "The try couldn't start."),
        (v::TrialWhy::Waited, "Claude didn’t answer within a minute."),
    ] {
        rows(&mut root, watch, &[row(v::TrialStep::Starts, failed(why))]);
        for w in [f::words(), f::browser_words()] {
            position(&from_root(&root, &w).0, Role::Paragraph, words.as_bytes());
        }
    }
}

pub(super) use crate::setup_tests::model_check::{failed, fed, from_root, position, print, row, rows};
