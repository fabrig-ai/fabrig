//! Model’s line, its states and controls (temper/docs/design/client/README.md, 3.5).
#![expect(
    clippy::disallowed_methods,
    clippy::disallowed_types,
    reason = "tests inspect semantic trees and decoded controls"
)]
use super::{connect_model as model, fixtures as f};
use crate::{
    accounts::Bindings,
    look,
    pages::model_check::{self, Answer, First, State, Trial},
    words::Words,
};
use skein_ui::{Builder, Role, Tree};
use temper_client_domain::{
    self as d, Address, Step,
    pages::{accounts::Way, connect_model::Act},
};
const WAY: Way = Way::Program { provider: 17 };
fn claude(state: State) -> Trial {
    Trial {
        account: b"Claude subscription".as_slice().into(),
        who: b"Claude".as_slice().into(),
        detail: Some(b"claude-owner, signed in on this machine".as_slice().into()),
        first: First::Subscription,
        state,
    }
}
fn bindings() -> Bindings {
    let limits = f::limits();
    Bindings::new(limits.client.domain.accounts, limits.row_uses, 1)
}
fn ready_root() -> d::Domain {
    let (mut root, watch) = super::welcome::model_with_forge(d::WayKind::Program);
    let mut doc = d::pages::accounts::document(root.client()).unwrap().clone();
    doc.usable = false;
    doc.settled = false;
    super::accounts::deliver(&mut root, watch, doc);
    let name = root.pages().account_name();
    model::sent(&mut root, Act::SignedIn { name, way: WAY });
    f::root_event(&mut root, d::Event::Went { place: d::Place::Address(Address::Welcome { step: Step::Ready }) });
    root
}
fn drawn(w: &Words, trial: &Trial) -> Tree {
    let mut b = Builder::new(&f::tree_limits());
    model_check::line(&mut b, &ready_root(), w, &mut bindings(), trial);
    b.finish()
}
fn position(tree: &Tree, role: Role, name: &[u8]) -> usize {
    tree.nodes().iter().position(|n| n.role == role && n.name.as_deref() == Some(name)).unwrap()
}
fn styled(tree: &Tree, style: skein_ui::Style) -> bool {
    tree.nodes().iter().any(|n| n.styles.as_slice().contains(&style))
}
fn items(tree: &Tree) -> Vec<&skein_ui::Node> {
    tree.nodes().iter().filter(|n| n.role == Role::Item).collect()
}
fn print(tree: &Tree) -> String {
    let mut print = skein_ui_text::Print::new(&f::tree_limits()).unwrap();
    skein_ui_text::render(tree, b"temper", None, &mut print);
    String::from_utf8(print.text.as_slice().to_vec()).unwrap()
}
fn stopped(at: usize, answer: Option<Answer>) -> Trial {
    claude(State::Stopped { at, why: b"Sign-in expired. Sign in again, then try again.".as_slice().into(), answer })
}
#[test]
fn running_marks_each_step_and_only_the_browser_has_a_bar() {
    for at in 0..3 {
        for w in [f::words(), f::browser_words()] {
            let tree = drawn(&w, &claude(State::Running { at }));
            let step = ["Signed in to your subscription", "Claude Code starts on this machine", "Claude answers"][at];
            let mut name = String::from("Claude subscription · trying: ");
            name.push_str(step);
            let line = position(&tree, Role::Paragraph, name.as_bytes());
            let disclosure = position(&tree, Role::Disclosure, b"Each step");
            assert!(line < disclosure);
            assert_eq!(styled(&tree, look::FLOW_BUSY), w.shell == crate::words::Shell::Browser);
            assert_eq!(tree.nodes().iter().any(|n| n.role == Role::Progress), w.shell == crate::words::Shell::Browser);
            assert_eq!(items(&tree).len(), 3);
            for (index, item) in items(&tree).iter().enumerate() {
                assert_eq!(item.styles.as_slice().contains(&look::FLOW_DONE), index < at);
                assert_eq!(item.states.as_slice().contains(&skein_ui::State::Current), index == at);
            }
            if w.shell == crate::words::Shell::Terminal {
                assert!(print(&tree).contains(&name));
            }
        }
    }
}
#[test]
fn usable_and_answered_are_ticked_without_each_step_and_only_answered_has_a_time() {
    for answered in [false, true] {
        for w in [f::words(), f::browser_words()] {
            let tree = drawn(&w, &claude(if answered { State::Answered { seconds: 6 } } else { State::Usable }));
            let mut name = String::from("Claude subscription · claude-owner, signed in on this machine");
            if answered {
                name.push_str(" · answered in 6 s");
            }
            let line = position(&tree, Role::Paragraph, name.as_bytes());
            assert!(tree.nodes()[line].styles.as_slice().contains(&look::FLOW_DONE));
            assert!(!tree.nodes().iter().any(|n| n.role == Role::Disclosure || n.role == Role::Progress));
            assert_eq!(String::from_utf8_lossy(&f::text(&tree)).contains("answered in"), answered);
            if w.shell == crate::words::Shell::Terminal {
                let mut expected = String::from("✓ ");
                expected.push_str(&name);
                assert!(print(&tree).contains(&expected));
            }
        }
    }
}
#[test]
fn stopped_says_why_then_controls_then_each_step_in_both_shells() {
    for at in 0..3 {
        for w in [f::words(), f::browser_words()] {
            let tree = drawn(&w, &stopped(at, Some(Answer::SignIn { way: WAY })));
            let line = position(&tree, Role::Paragraph, "Claude subscription · Claude didn’t answer".as_bytes());
            let why = position(&tree, Role::Paragraph, b"Sign-in expired. Sign in again, then try again.");
            let retry = position(&tree, Role::Button, b"Try again");
            let sign_in = position(&tree, Role::Link, b"Sign in again");
            let disclosure = position(&tree, Role::Disclosure, b"Each step");
            assert!(line < why && why < retry && retry < sign_in && sign_in < disclosure);
            assert!(tree.nodes()[line].styles.as_slice().contains(&look::FLOW_FAILED));
            assert!(tree.nodes()[why].styles.as_slice().contains(&look::WELCOME_STATUS_WHY));
            assert!(tree.nodes()[retry].styles.as_slice().contains(&look::FLOW_SECONDARY));
            assert!(tree.nodes()[sign_in].styles.as_slice().contains(&look::FLOW_QUIET));
            assert!(styled(&tree, look::WELCOME_STATUS_ACTS));
            assert!(f::find(&tree, Role::Status, b"stopped").is_some());
            assert!(!tree.nodes().iter().any(|n| n.role == Role::Progress));
            for (index, item) in items(&tree).iter().enumerate() {
                assert_eq!(item.styles.as_slice().contains(&look::FLOW_DONE), index < at);
                assert_eq!(item.styles.as_slice().contains(&look::FLOW_FAILED), index == at);
            }
            if w.shell == crate::words::Shell::Terminal {
                assert!(print(&tree).contains("Claude subscription · Claude didn’t answer"));
            }
        }
    }
}
#[test]
fn a_plan_or_key_has_two_steps_with_the_first_already_done() {
    for (first, account, line) in
        [(First::Plan, "ChatGPT plan", "Signed in to your ChatGPT plan"), (First::Key, "OpenAI API key", "Key added")]
    {
        let trial = Trial {
            first,
            account: account.as_bytes().into(),
            who: b"ChatGPT".as_slice().into(),
            detail: None,
            state: State::Running { at: 1 },
        };
        for w in [f::words(), f::browser_words()] {
            let tree = drawn(&w, &trial);
            assert_eq!(items(&tree).len(), 2);
            assert!(items(&tree)[0].styles.as_slice().contains(&look::FLOW_DONE));
            assert!(items(&tree)[1].states.as_slice().contains(&skein_ui::State::Current));
            let text = String::from_utf8_lossy(&f::text(&tree)).into_owned();
            assert!(text.contains(line));
            assert!(text.contains("ChatGPT answers"));
            assert!(!text.contains("Code starts"));
        }
    }
}
#[test]
fn stopped_controls_retry_and_link_to_the_reason_or_all_ways() {
    let root = ready_root();
    for has_answer in [false, true] {
        let mut bindings = bindings();
        let mut b = Builder::new(&f::tree_limits());
        model_check::line(
            &mut b,
            &root,
            &f::browser_words(),
            &mut bindings,
            &stopped(1, if has_answer { Some(Answer::SignIn { way: WAY }) } else { None }),
        );
        let tree = b.finish();
        let controls: Vec<_> = tree.nodes().iter().filter(|n| n.role == Role::Button || n.role == Role::Link).collect();
        assert_eq!(controls.len(), 2);
        let expected = [
            d::pages::Act::ConnectModel(Act::SignedIn { name: root.pages().account_name(), way: WAY }),
            d::pages::Act::Go {
                address: Address::Welcome {
                    step: Step::Model {
                        way: if has_answer { Some(d::pages::connect_model::model_way(WAY)) } else { None },
                    },
                },
            },
        ];
        assert_eq!(
            controls[1].name.as_deref(),
            Some(if has_answer { b"Sign in again".as_slice() } else { b"All ways" })
        );
        for (node, expected) in controls.iter().zip(expected) {
            let binding = node.binding.or(node.link).unwrap();
            let Some(d::Event::Act { action }) = bindings.decode(binding.token, skein_ui::Act::Press) else {
                panic!("control acts")
            };
            assert_eq!(action, d::Action::Pages(expected));
        }
    }
}
#[test]
fn only_the_welcomes_host_check_supplies_a_trial_today() {
    let (mut root, watch) = super::welcome::model_with_forge(d::WayKind::Program);
    let mut doc = d::pages::accounts::document(root.client()).unwrap().clone();
    doc.usable = false;
    doc.settled = false;
    super::accounts::deliver(&mut root, watch, doc.clone());
    assert!(model_check::trial(&root, &f::words()).is_none());
    let name = root.pages().account_name();
    let attempt = model::sent(&mut root, Act::SignedIn { name, way: Way::Program { provider: 17 } });
    let trial = model_check::trial(&root, &f::words()).unwrap();
    let State::Running { at } = trial.state else { panic!("host check is running") };
    assert_eq!(at, 0);
    let limits = f::limits();
    let mut out = skein_lib::Queue::uncapped();
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
            | d::Request::Copy { .. }
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
}
