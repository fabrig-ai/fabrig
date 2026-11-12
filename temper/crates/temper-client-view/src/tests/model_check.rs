//! Model’s line, its states and controls (temper/docs/design/client/pages/README.md).
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
        account_number: None,
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
    model_check::line(&mut b, &ready_root(), w, &mut bindings(), trial, model_check::Place::Welcome);
    b.finish()
}
fn position(tree: &Tree, role: Role, name: &[u8]) -> usize {
    tree.nodes()
        .iter()
        .position(|n| n.role == role && n.name.as_deref() == Some(name))
        .unwrap_or_else(|| panic!("missing {role:?}: {}", String::from_utf8_lossy(name)))
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
            assert!(!String::from_utf8_lossy(&f::text(&tree)).contains("one short message"));
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
    for (key, account, first_line) in
        [(false, "ChatGPT plan", "Signed in to your ChatGPT plan"), (true, "OpenAI API key", "Key added")]
    {
        for state in [
            State::Running { at: 1 },
            State::Answered { seconds: 6 },
            State::Stopped { at: 1, why: b"Stopped".as_slice().into(), answer: None },
        ] {
            let running = match state {
                State::Running { .. } => true,
                State::Usable | State::Answered { .. } | State::Stopped { .. } => false,
            };
            let trial = Trial {
                first: if key { First::Key } else { First::Plan },
                account_number: None,
                account: account.as_bytes().into(),
                who: b"ChatGPT".as_slice().into(),
                detail: None,
                state,
            };
            for w in [f::words(), f::browser_words()] {
                let tree = drawn(&w, &trial);
                let text = String::from_utf8_lossy(&f::text(&tree)).into_owned();
                assert_eq!(text.contains("one short message"), key && running);
                if !running {
                    assert!(!print(&tree).contains("one short message"));
                    continue;
                }
                let mut name = String::from(account);
                name.push_str(" · trying: ChatGPT answers");
                if key {
                    name.push_str(" · one short message");
                }
                let at = position(&tree, Role::Paragraph, name.as_bytes());
                if w.shell == crate::words::Shell::Browser {
                    let paragraph = &tree.nodes()[at..at + usize::try_from(tree.nodes()[at].size).unwrap()];
                    let details: Vec<_> =
                        paragraph.iter().filter(|n| n.styles.as_slice().contains(&look::FLOW_MUTED)).collect();
                    let expected: &[&str] = if key {
                        &["Trying: ChatGPT answers", "one short message"]
                    } else {
                        &["Trying: ChatGPT answers"]
                    };
                    assert_eq!(details.len(), expected.len());
                    for (detail, expected) in details.iter().zip(expected) {
                        assert_eq!(f::nodes_text(core::slice::from_ref(*detail)).as_ref(), expected.as_bytes());
                    }
                } else {
                    assert!(print(&tree).contains(&name));
                }
                assert_eq!(items(&tree).len(), 2);
                assert!(items(&tree)[0].styles.as_slice().contains(&look::FLOW_DONE));
                assert!(items(&tree)[1].states.as_slice().contains(&skein_ui::State::Current));
                assert!(text.contains(first_line));
                assert!(text.contains("ChatGPT answers"));
                assert!(!text.contains("Code starts"));
            }
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
            model_check::Place::Welcome,
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

use jig_client::contract as c;
use skein_lib::{List, Queue, Token, Writer};
use temper_client_protocol_wire::{self as wire, v1 as v};
fn event(root: &mut d::Domain, event: d::Event) -> Queue<d::Request> {
    let mut out = Queue::uncapped();
    d::step(root, &skein_lib::Env { now: skein_lib::Time::ZERO, wall: f::NOW, limits: f::limits() }, event, &mut out);
    out
}
fn up(root: &mut d::Domain, up: jig_client::Up) -> Queue<d::Request> {
    event(root, d::Event::Client { event: jig_client::Event::Up { up } })
}
fn opening(out: &Queue<d::Request>, subject: jig_client::Watchable) -> Token {
    out.iter()
        .find_map(|r| {
            if let d::Request::Unlinked {
                request: jig_client::UnlinkedRequest::Down { down: jig_client::Down::Open { watch, subject: found } },
            } = r
            {
                (*found == subject).then_some(*watch)
            } else {
                None
            }
        })
        .expect("watch opened")
}
fn fed(kind: d::WayKind) -> (d::Domain, Token, Token) {
    let (offered, _) = super::welcome::model_with_forge(kind);
    let me = d::pages::accounts::current_me(offered.client()).unwrap().clone();
    let mut root = d::unlinked(&f::limits(), f::SEED);
    let mut out =
        event(&mut root, d::Event::Client { event: jig_client::Event::StartUnlinked { kept: List::with_capacity(0) } });
    let mut more = event(
        &mut root,
        d::Event::Start {
            address: Address::Welcome { step: Step::Ready },
            kept: List::with_capacity(0),
            with: Some(jig_client::With::Transport),
        },
    );
    while let Some(r) = more.pop() {
        out.push(r);
    }
    let mut more = up(&mut root, jig_client::Up::Served { served: true, versions: None });
    while let Some(r) = more.pop() {
        out.push(r);
    }
    let me_watch = opening(&out, jig_client::Watchable::Jig(c::Subject::Me));
    let accounts = opening(&out, jig_client::Watchable::Jig(c::Subject::Accounts));
    let mut more = up(
        &mut root,
        jig_client::Up::Watched { watch: me_watch, event: c::WatchEvent::Delivered(c::Delivery::Me(Box::new(me))) },
    );
    while let Some(r) = more.pop() {
        out.push(r);
    }
    let trials = opening(&out, d::forge::trials::watch(&f::limits().forge).unwrap());
    for request in &out {
        if let d::Request::Unlinked {
            request:
                jig_client::UnlinkedRequest::Down {
                    down: jig_client::Down::Read { read, what: jig_client::Readable::Family { record, .. } },
                },
        } = request
            && wire::decode_reading(&record.bytes, &v::CEILINGS).unwrap().read() == &v::Read::Deployment
        {
            f::deliver_deployment(&mut root, *read, true, true);
        }
    }
    let forges = opening(&out, d::forge::forges::watch(&f::limits().forge).unwrap());
    let forges_doc = v::Forges::new(
        &v::CEILINGS,
        v::ForgesParts {
            forges: super::connect_forge::first_rows(super::connect_forge::connected(), super::connect_forge::made()),
            linked: false,
        },
    )
    .unwrap();
    up(
        &mut root,
        jig_client::Up::FamilyDelivered {
            watch: forges,
            record: jig_client::Family {
                family: wire::FORGE,
                version: wire::CLIENT_VERSION,
                bytes: wire::forges(&forges_doc),
            },
        },
    );
    let mut doc = super::accounts::card_document(
        if kind == d::WayKind::Program { b"anthropic" } else { b"openai" },
        b"owner",
        match kind {
            d::WayKind::Program => c::AccountKind::Program,
            d::WayKind::SignIn => c::AccountKind::SignIn,
            d::WayKind::Key => c::AccountKind::Key,
        },
    );
    doc.cut = false;
    doc.accounts.get_mut(0).unwrap().cut = false;
    super::accounts::deliver(&mut root, accounts, doc);
    assert!(d::pages::accounts::document(root.client()).is_some(), "accounts delivered");
    event(&mut root, d::Event::Act { action: d::Action::Forge(d::forge::Act::TryAccount { account: 1, again: true }) });
    (root, trials, accounts)
}
fn row(step: v::TrialStep, state: v::TrialState) -> v::TrialRow {
    v::TrialRow::new(&v::CEILINGS, v::TrialRowParts { account: 1, task: Some(91), took: None, step, state }).unwrap()
}
fn rows(root: &mut d::Domain, watch: Token, rows: &[v::TrialRow]) -> Queue<d::Request> {
    let doc = v::Trials::new(&v::CEILINGS, v::TrialsParts { rows: List::from_boxed(rows.into()) }).unwrap();
    let mut writer = Writer::new(usize::try_from(doc.measure()).unwrap());
    doc.encode(&mut writer).unwrap();
    up(
        root,
        jig_client::Up::FamilyDelivered {
            watch,
            record: jig_client::Family { family: wire::FORGE, version: wire::CLIENT_VERSION, bytes: writer.finish() },
        },
    )
}
fn failed(why: v::TrialWhy) -> v::TrialState {
    v::TrialState::Failed(v::TrialFailed::new(&v::CEILINGS, v::TrialFailedParts { why }).unwrap())
}
fn from_root(root: &d::Domain, w: &Words) -> (Tree, Bindings) {
    let mut b = Builder::new(&f::tree_limits());
    let mut bindings = bindings();
    model_check::line(
        &mut b,
        root,
        w,
        &mut bindings,
        &model_check::trial(root, w).unwrap(),
        model_check::Place::Welcome,
    );
    (b.finish(), bindings)
}
#[test]
fn trial_rows_feed_the_steps_answer_and_ask_gap_in_both_shells() {
    for (kind, email, detail) in [
        (d::WayKind::Program, None, Some("owner, signed in on this machine")),
        (d::WayKind::SignIn, None, Some("owner")),
        (d::WayKind::Key, None, Some("billed to the key’s organization")),
        (
            d::WayKind::Program,
            Some(b"alice@example.com".as_slice()),
            Some("alice@example.com, signed in on this machine"),
        ),
        (d::WayKind::SignIn, Some(b"alice@example.com".as_slice()), Some("alice@example.com")),
    ] {
        let (mut root, watch, accounts) = fed(kind);
        let mut doc = d::pages::accounts::document(root.client()).unwrap().clone();
        doc.accounts.get_mut(0).unwrap().facts = super::accounts::email_facts(email);
        super::accounts::deliver(&mut root, accounts, doc);
        assert_eq!(root.forge().trial_asked(), Some(1));
        for (step, state, at) in [
            (v::TrialStep::Starts, v::TrialState::Running, 1),
            (v::TrialStep::Answers, v::TrialState::Running, if kind == d::WayKind::Program { 2 } else { 1 }),
        ] {
            for w in [f::words(), f::browser_words()] {
                let trial = model_check::trial(&root, &w).unwrap();
                let State::Running { .. } = trial.state else { panic!("ask and running never ticked") };
                rows(&mut root, watch, &[row(step.clone(), state.clone())]);
                let (tree, _) = from_root(&root, &w);
                let steps = items(&tree);
                assert_eq!(steps.len(), if kind == d::WayKind::Program { 3 } else { 2 });
                for (i, node) in steps.iter().enumerate() {
                    assert_eq!(node.states.as_slice().contains(&skein_ui::State::Current), i == at);
                    assert_eq!(node.styles.as_slice().contains(&look::FLOW_DONE), i < at);
                }
                assert!(!tree.nodes().iter().any(|n| n.name.as_deref() == Some(b"Try again")));
                if w.shell == crate::words::Shell::Terminal {
                    assert!(print(&tree).contains("trying:"));
                }
            }
        }
        rows(
            &mut root,
            watch,
            &[row(
                v::TrialStep::Answers,
                v::TrialState::Answered(
                    v::TrialAnswered::new(&v::CEILINGS, v::TrialAnsweredParts { seconds: 7 }).unwrap(),
                ),
            )],
        );
        for w in [f::words(), f::browser_words()] {
            let trial = model_check::trial(&root, &w).unwrap();
            assert_eq!(trial.detail.as_deref(), detail.map(str::as_bytes));
            let (tree, _) = from_root(&root, &w);
            assert!(String::from_utf8_lossy(&f::text(&tree)).contains("answered in 7 s"));
            assert!(!tree.nodes().iter().any(|n| n.role == Role::Disclosure));
        }
    }
}
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

fn wait_delivery(root: &mut d::Domain, watch: Token, wait: Option<c::TaskRunWait>) {
    let mut task = f::task::document();
    task.chip.task = 91;
    task.llm = None;
    task.run_wait = wait;
    up(root, jig_client::Up::Watched { watch, event: c::WatchEvent::Delivered(c::Delivery::Task(Box::new(task))) });
}
#[test]
fn a_running_trials_wait_stops_the_line_until_it_clears_and_waited_keeps_its_words() {
    use c::{TaskRunWait as R, TaskSourceState as S, TaskWaitWhy as W};
    let (mut root, watch, _) = fed(d::WayKind::Program);
    let out = rows(&mut root, watch, &[row(v::TrialStep::Starts, v::TrialState::Running)]);
    let task_watch = opening(&out, jig_client::Watchable::Jig(c::Subject::Task { task: 91 }));
    for (why, expected) in [
        (W::Source(S::NeedsSignIn), "Needs sign-in"),
        (W::Source(S::Limited { until: None }), "Limited. Tasks on it wait."),
        (W::Source(S::Refused), "Refused"),
        (W::Source(S::Misconfigured), "The provider refused temper's sign-in. Check the configuration's registration."),
        (
            W::Source(S::Unloaded { at: c::AccountPlace::File, why: c::AccountUnread::Unreadable }),
            "Cannot load File: Unreadable",
        ),
        (W::Host, "owner's login isn't signed in on a host that can run this."),
        (W::Empty, "No account can run this chat yet."),
        (W::Invalid(c::AccountWhy::Uses), "The requested account cannot serve this work"),
        (W::Source(S::Later { until: None }), "Try later"),
        (W::Source(S::Cancelled), "Cancelled"),
        (W::Source(S::Closing), "Closing"),
    ] {
        wait_delivery(&mut root, task_watch, Some(R::Account { account: Some(1), source: Some(1), why }));
        for w in [f::words(), f::browser_words()] {
            let (tree, _) = from_root(&root, &w);
            position(&tree, Role::Paragraph, expected.as_bytes());
            assert!(!tree.nodes().iter().any(|n| n.name.as_deref() == Some(b"Try again")));
            assert!(!String::from_utf8_lossy(&f::text(&tree)).contains("temper program-login"));
        }
    }
    for wait in [
        R::Credential,
        R::Host,
        R::Refused(c::TaskWaitRefused::Bounds {
            kind: c::HostKind::Worker,
            why: Some(c::AgentShortfall::Turns { charter: 1 }),
            figures: Some(c::Figures { required: 100_000, offered: 800 }),
        }),
        R::Refused(c::TaskWaitRefused::Hello {
            kind: c::HostKind::Worker,
            at: skein_lib::Wall::from_nanos(1_791_828_240_000_000_000),
            why: c::HelloRefusal::JobKinds,
            figures: None,
        }),
    ] {
        wait_delivery(&mut root, task_watch, Some(wait));
        for w in [f::words(), f::browser_words()] {
            let trial = model_check::trial(&root, &w).unwrap();
            let State::Stopped { why, .. } = trial.state else { panic!("lasting wait stops line") };
            let mut b = Builder::new(&f::tree_limits());
            let mut task = f::task::document();
            task.llm = None;
            task.run_wait = Some(wait);
            let now = if let R::Refused(c::TaskWaitRefused::Hello { at, .. }) = wait { at } else { f::NOW };
            crate::choice::waiting(
                &mut b,
                &root,
                &task,
                &w,
                &mut crate::frame::Bindings::new(32),
                &mut crate::words::Times::new(now),
            );
            assert!(skein_lib::bytes::find(&f::text(&b.finish()), &why).is_some(), "same words as a chat");
        }
    }
    for why in [W::Source(S::Asking), W::Refreshing, W::Source(S::Granted), W::Source(S::Moving)] {
        wait_delivery(&mut root, task_watch, Some(R::Account { account: Some(1), source: Some(1), why }));
        for w in [f::words(), f::browser_words()] {
            let State::Running { at: 1 } = model_check::trial(&root, &w).unwrap().state else {
                panic!("a moment settles this wait")
            };
        }
    }
    wait_delivery(&mut root, task_watch, None);
    let State::Running { at: 1 } = model_check::trial(&root, &f::words()).unwrap().state else {
        panic!("cleared wait leaves the row running")
    };
    wait_delivery(
        &mut root,
        task_watch,
        Some(R::Account { account: Some(1), source: Some(1), why: W::Source(S::NeedsSignIn) }),
    );
    rows(&mut root, watch, &[row(v::TrialStep::Starts, failed(v::TrialWhy::Waited))]);
    for w in [f::words(), f::browser_words()] {
        let (tree, _) = from_root(&root, &w);
        position(&tree, Role::Paragraph, b"Needs sign-in");
        position(&tree, Role::Button, b"Try again");
        position(&tree, Role::Link, b"Sign in again");
    }
}
#[test]
fn took_names_the_account_that_answered_and_the_newest_row_wins() {
    let (mut root, watch, accounts) = fed(d::WayKind::Program);
    let mut doc = d::pages::accounts::document(root.client()).unwrap().clone();
    let mut other =
        super::accounts::card_document(b"openai", b"other", c::AccountKind::SignIn).accounts.get(0).unwrap().clone();
    other.number = 2;
    doc.accounts = List::from_boxed(Box::new([doc.accounts.get(0).unwrap().clone(), other]));
    super::accounts::deliver(&mut root, accounts, doc);
    let mut took = row(v::TrialStep::Answers, failed(v::TrialWhy::Waited)).into_parts();
    took.took = Some(2);
    let mut older = row(v::TrialStep::Starts, v::TrialState::Running).into_parts();
    older.account = 2;
    rows(
        &mut root,
        watch,
        &[v::TrialRow::new(&v::CEILINGS, took).unwrap(), v::TrialRow::new(&v::CEILINGS, older).unwrap()],
    );
    for w in [f::words(), f::browser_words()] {
        let (tree, _) = from_root(&root, &w);
        assert!(model_check::of_account(&root, &w, 1).is_none());
        assert!(model_check::of_account(&root, &w, 2).is_some());
        assert_eq!(model_check::stopped(&root, &w), Some((Box::from(b"ChatGPT".as_slice()), 1)));
        position(&tree, Role::Paragraph, "ChatGPT plan · ChatGPT didn’t answer".as_bytes());
        position(&tree, Role::Paragraph, "ChatGPT didn’t answer within a minute.".as_bytes());
        assert_eq!(items(&tree).len(), 2);
    }
}

#[test]
fn sign_in_again_follows_the_place_and_the_selected_source() {
    for kind in [d::WayKind::Program, d::WayKind::SignIn, d::WayKind::Key] {
        let (mut root, watch, accounts) = fed(kind);
        let mut doc = d::pages::accounts::document(root.client()).unwrap().clone();
        let account = doc.accounts.get_mut(0).unwrap();
        let source = account.sources.get(0).unwrap().clone();
        let mut unused = source.clone();
        unused.kind = c::AccountKind::Key;
        unused.number = 99;
        unused.usable = false;
        account.sources = List::from_boxed(Box::new([unused, source]));
        super::accounts::deliver(&mut root, accounts, doc);
        let why = v::TrialWhy::Ended(
            v::TrialEnded::new(&v::CEILINGS, v::TrialEndedParts { class: v::TrialClass::Permanent }).unwrap(),
        );
        rows(&mut root, watch, &[row(v::TrialStep::Answers, failed(why))]);
        for w in [f::words(), f::browser_words()] {
            for welcome in [false, true] {
                let mut b = Builder::new(&f::tree_limits());
                let mut bindings = bindings();
                let trial = model_check::of_account(&root, &w, 1).unwrap();
                model_check::line(
                    &mut b,
                    &root,
                    &w,
                    &mut bindings,
                    &trial,
                    if welcome { model_check::Place::Welcome } else { model_check::Place::Row },
                );
                let tree = b.finish();
                let links: Vec<_> = tree.nodes().iter().filter(|n| n.role == Role::Link).collect();
                if kind == d::WayKind::Key && !welcome {
                    assert_eq!(links.len(), 0);
                    continue;
                }
                assert_eq!(links.len(), 1);
                let link = links[0];
                assert_eq!(
                    link.name.as_deref(),
                    Some(if kind == d::WayKind::Key { b"All ways".as_slice() } else { b"Sign in again" })
                );
                let way =
                    if kind == d::WayKind::Key { None } else { Some(d::ModelWay { source: None, provider: 17, kind }) };
                let address = if welcome {
                    Address::Welcome { step: Step::Model { way } }
                } else {
                    Address::ConnectModel { way }
                };
                let Some(d::Event::Act { action }) = bindings.decode(link.link.unwrap().token, skein_ui::Act::Press)
                else {
                    panic!("sign in acts")
                };
                assert_eq!(action, d::Action::Pages(d::pages::Act::Go { address }));
            }
        }
    }
}
