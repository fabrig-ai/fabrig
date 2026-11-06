//! The model try's box and acts (temper/docs/design/client/README.md, 3.5).
#![expect(
    clippy::disallowed_methods,
    clippy::disallowed_types,
    reason = "tests inspect semantic trees, typed acts and their terminal prints"
)]
use super::{connect_model as model, fixtures as f};
use crate::{
    accounts::Bindings,
    look,
    pages::model_check::{self, Answer, First, State, Trial},
    words::Words,
};
use skein_ui::{Builder, Node, Role, Tree};
use temper_client_domain::{
    self as d, Address,
    pages::{accounts::Way, connect_model::Act},
};

fn claude(state: State) -> Trial {
    Trial {
        account: b"Claude subscription".as_slice().into(),
        who: b"Claude".as_slice().into(),
        first: First::Subscription,
        state,
    }
}
fn drawn(w: &Words, trial: &Trial) -> Tree {
    let mut b = Builder::new(&f::tree_limits());
    model_check::status(&mut b, w, trial);
    b.finish()
}
fn position(tree: &Tree, role: Role, name: &[u8]) -> usize {
    tree.nodes().iter().position(|n| n.role == role && n.name.as_deref() == Some(name)).unwrap()
}
fn styled(tree: &Tree, style: skein_ui::Style) -> bool {
    tree.nodes().iter().any(|n| n.styles.as_slice().contains(&style))
}
fn items(tree: &Tree) -> Vec<&Node> {
    tree.nodes().iter().filter(|n| n.role == Role::Item).collect()
}
fn words(tree: &Tree) -> String {
    String::from_utf8(f::text(tree).into_vec()).unwrap()
}
fn print(tree: &Tree) -> String {
    let mut print = skein_ui_text::Print::new(&f::tree_limits()).unwrap();
    skein_ui_text::render(tree, b"temper", None, &mut print);
    String::from_utf8(print.text.as_slice().to_vec()).unwrap()
}
fn bindings() -> Bindings {
    let limits = f::limits();
    Bindings::new(limits.client.domain.accounts, limits.row_uses, 1)
}
fn stopped(at: usize, answer: Option<Answer>) -> Trial {
    claude(State::Stopped { at, why: b"Sign-in expired. Sign in again, then try again.".as_slice().into(), answer })
}
#[test]
fn running_marks_each_step_and_only_the_browser_spins() {
    for at in 0..3 {
        for w in [f::words(), f::browser_words()] {
            let tree = drawn(&w, &claude(State::Running { at, message: true }));
            assert_eq!(tree.nodes()[0].styles.as_slice(), &[look::WELCOME_FORGE_STATUS, look::WELCOME_CHECK]);
            let status = f::find(&tree, Role::Status, b"Trying your Claude subscription one short message").unwrap();
            assert_eq!(tree.nodes()[1].id, status);
            assert!(f::find(&tree, Role::List, b"Each step").is_some());
            assert_eq!(styled(&tree, look::WELCOME_SPIN), w.shell == crate::words::Shell::Browser);
            assert_eq!(items(&tree).len(), 3);
            assert!(
                words(&tree).find("Signed in to your subscription").unwrap()
                    < words(&tree).find("Claude Code starts").unwrap()
            );
            assert!(words(&tree).find("Claude Code starts").unwrap() < words(&tree).find("Claude answers").unwrap());
            for (index, item) in items(&tree).iter().enumerate() {
                assert_eq!(item.styles.as_slice().contains(&look::FLOW_DONE), index < at);
                assert_eq!(item.states.as_slice().contains(&skein_ui::State::Current), index == at);
                let offset = tree.nodes().iter().position(|n| core::ptr::eq(n, *item)).unwrap();
                let state = tree
                    .nodes()
                    .iter()
                    .skip(offset + 1)
                    .take(usize::try_from(item.size).unwrap() - 1)
                    .find(|n| n.role == Role::Status);
                assert_eq!(
                    state.and_then(|n| n.name.as_deref()),
                    match index.cmp(&at) {
                        core::cmp::Ordering::Less => Some(b"done".as_slice()),
                        core::cmp::Ordering::Equal => Some(b"under way".as_slice()),
                        core::cmp::Ordering::Greater => None,
                    }
                );
            }
        }
    }
    let tree = drawn(&f::words(), &claude(State::Running { at: 0, message: true }));
    assert!(print(&tree).contains("Trying your Claude subscription · one short message"));
    let tree = drawn(&f::browser_words(), &claude(State::Running { at: 0, message: false }));
    assert!(!words(&tree).contains("one short message"));
    assert!(f::find(&tree, Role::Status, b"Trying your Claude subscription").is_some());
}
#[test]
fn answered_folds_completed_steps_and_prints_its_elapsed_time() {
    for w in [f::words(), f::browser_words()] {
        let tree = drawn(&w, &claude(State::Answered { seconds: 6 }));
        assert_eq!(
            tree.nodes()[0].styles.as_slice(),
            &[look::WELCOME_FORGE_STATUS, look::FLOW_DONE, look::WELCOME_CHECK]
        );
        assert!(f::find(&tree, Role::Status, b"Claude subscription answered in 6 s").is_some());
        let disclosure = position(&tree, Role::Disclosure, b"Each step");
        let list = position(&tree, Role::List, b"Each step");
        assert!(disclosure < list);
        assert!(items(&tree).iter().all(|n| n.styles.as_slice().contains(&look::FLOW_DONE)));
        assert!(!styled(&tree, look::WELCOME_SPIN));
        assert_eq!(styled(&tree, look::WELCOME_OK), w.shell == crate::words::Shell::Browser);
        if w.shell == crate::words::Shell::Terminal {
            assert!(print(&tree).contains("✓ Claude subscription · answered in 6 s"));
        }
    }
}
#[test]
fn stopped_marks_the_failed_step_between_done_and_to_come_then_says_why() {
    for at in 0..3 {
        let tree = drawn(&f::browser_words(), &stopped(at, Some(Answer::SignIn)));
        assert_eq!(
            tree.nodes()[0].styles.as_slice(),
            &[look::WELCOME_FORGE_STATUS, look::WELCOME_CHECK, look::WELCOME_CHECK_STOPPED]
        );
        let alert = position(&tree, Role::Alert, "Claude didn’t answer".as_bytes());
        let list = position(&tree, Role::List, b"Each step");
        let why = position(&tree, Role::Paragraph, b"Sign-in expired. Sign in again, then try again.");
        assert!(alert < list && list < why);
        assert!(tree.nodes()[why].styles.as_slice().contains(&look::WELCOME_CHECK_WHY));
        for (index, item) in items(&tree).iter().enumerate() {
            assert_eq!(item.styles.as_slice().contains(&look::FLOW_DONE), index < at);
            assert_eq!(item.styles.as_slice().contains(&look::WELCOME_CHECK_FAILED), index == at);
            assert!(!item.states.as_slice().contains(&skein_ui::State::Current));
        }
        assert!(!styled(&tree, look::WELCOME_SPIN));
        assert!(f::find(&tree, Role::Disclosure, b"Each step").is_none());
    }
    assert!(print(&drawn(&f::words(), &stopped(2, None))).contains("Claude didn’t answer"));
}
#[test]
fn a_plan_or_key_has_two_steps_with_the_first_already_done() {
    for (first, account, line) in [
        (First::Plan, b"ChatGPT plan".as_slice(), b"Signed in to your ChatGPT plan".as_slice()),
        (First::Key, b"OpenAI API key".as_slice(), b"Key added".as_slice()),
    ] {
        let trial = Trial {
            first,
            account: account.into(),
            who: b"ChatGPT".as_slice().into(),
            state: State::Running { at: 1, message: true },
        };
        let tree = drawn(&f::browser_words(), &trial);
        assert_eq!(items(&tree).len(), 2);
        assert!(items(&tree)[0].styles.as_slice().contains(&look::FLOW_DONE));
        assert!(items(&tree)[1].states.as_slice().contains(&skein_ui::State::Current));
        assert!(f::text(&tree).windows(line.len()).any(|s| s == line));
        assert!(words(&tree).contains("ChatGPT answers"));
        assert!(!words(&tree).contains("Code starts"));
        let trial = Trial { state: State::Running { at: 0, message: true }, ..trial };
        assert!(items(&drawn(&f::browser_words(), &trial))[0].styles.as_slice().contains(&look::FLOW_DONE));
    }
}
#[test]
fn stopped_controls_offer_retry_then_the_reason_or_all_ways_then_continue() {
    let (root, _) = model::flow_fixture(d::WayKind::Program);
    let way = Way::Program { provider: 17 };
    for answer in [Some(Answer::SignIn), None] {
        let has_answer = answer.is_some();
        let mut bindings = bindings();
        let mut b = Builder::new(&f::tree_limits());
        model_check::stopped_acts(
            &mut b,
            &root,
            &f::browser_words(),
            &mut bindings,
            &stopped(1, answer),
            way,
            Address::NewChat { draft: false },
        );
        let tree = b.finish();
        assert!(tree.nodes()[0].styles.as_slice().contains(&look::WELCOME_ACTS));
        let controls: Vec<_> = tree.nodes().iter().filter(|n| n.role == Role::Button || n.role == Role::Link).collect();
        assert_eq!(controls.len(), 3);
        assert_eq!(
            controls.iter().map(|n| n.name.as_deref().unwrap()).collect::<Vec<_>>(),
            [b"Try again".as_slice(), if has_answer { b"Sign in again" } else { b"All ways" }, b"Continue anyway"]
        );
        assert_eq!(controls[0].styles.as_slice(), &[look::FLOW_PRIMARY]);
        assert_eq!(controls[1].styles.as_slice(), &[look::FLOW_SECONDARY]);
        assert_eq!(controls[2].styles.as_slice(), &[look::FLOW_QUIET]);
        let expected = [
            d::pages::Act::ConnectModel(Act::SignedIn { name: root.pages().account_name(), way }),
            if has_answer {
                d::pages::Act::ConnectModel(Act::SignInAgain)
            } else {
                d::pages::Act::Go { address: Address::ConnectModel { way: None } }
            },
            d::pages::Act::Go { address: Address::NewChat { draft: false } },
        ];
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
fn standalone_uses_the_box_in_its_card_and_keeps_its_primary() {
    let (root, _) = model::flow_fixture(d::WayKind::Program);
    for trial in
        [claude(State::Running { at: 1, message: true }), claude(State::Answered { seconds: 6 }), stopped(2, None)]
    {
        let mut b = Builder::new(&f::tree_limits());
        let mut bindings = bindings();
        crate::pages::connect_model::trial_body(
            &mut b,
            &root,
            &f::browser_words(),
            &mut bindings,
            &trial,
            Way::Program { provider: 17 },
        );
        let tree = b.finish();
        assert!(styled(&tree, look::FLOW_DETAILS));
        assert!(styled(&tree, look::WELCOME_CHECK));
        assert!(!styled(&tree, look::WELCOME));
        assert!(f::find(&tree, Role::Heading { level: 2 }, b"Ready").is_some());
        if let State::Answered { .. } = trial.state {
            let primary = position(&tree, Role::Link, b"Start a chat");
            let binding = tree.nodes()[primary].link.unwrap();
            let Some(d::Event::Act { action }) = bindings.decode(binding.token, skein_ui::Act::Press) else {
                panic!("primary acts")
            };
            assert_eq!(action, d::Action::Pages(d::pages::Act::Go { address: Address::NewChat { draft: false } }));
        }
    }
}
#[test]
fn ready_note_says_that_the_try_failed_in_muted_words() {
    let mut b = Builder::new(&f::tree_limits());
    model_check::ready_note(&mut b, &f::words());
    let tree = b.finish();
    assert_eq!(words(&tree), "didn’t answer when tried");
    assert!(styled(&tree, look::FLOW_MUTED));
    assert!(!model_check::last_try_failed());
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
    let State::Running { at, message } = trial.state else { panic!("host check is running") };
    assert_eq!(at, 0);
    assert!(!message);
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
    let State::Running { at, message } = trial.state else { panic!("host check is running") };
    assert_eq!(at, 0);
    assert!(!message);
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
            | d::Request::Tab { .. }
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
    let Some(Answer::SignIn) = answer else { panic!("sign-in answers the reason") };
}
