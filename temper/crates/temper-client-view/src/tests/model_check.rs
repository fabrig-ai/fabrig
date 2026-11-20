//! Model’s line, its states and controls (temper/docs/design/client/pages/README.md).
#![expect(clippy::disallowed_methods, reason = "tests inspect semantic trees and decoded controls")]
use super::{connect_model as model, fixtures as f};
use crate::pages::model_check::{self, Answer, First, State};
use crate::{accounts::Bindings, words::Words};
use jig_client::contract as c;
use skein_lib::Writer;
use skein_lib::{List, Token};
use skein_ui::Role;
use skein_ui::{Builder, Tree};
use temper_client_domain::{
    self as d, Address, Step,
    pages::{accounts::Way, connect_model::Act},
};
use temper_client_protocol_wire as wire;
use temper_client_protocol_wire::v1::Record;

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
        let why = v::TrialWhy::Ended(v::TrialEnded { class: class.clone() });
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

fn bindings() -> Bindings {
    let limits = f::limits();
    Bindings::new(limits.client.domain.accounts, limits.row_uses, 1)
}

#[expect(clippy::disallowed_types, reason = "tests print the missing semantic node’s name")]
pub(crate) fn position(tree: &Tree, role: Role, name: &[u8]) -> usize {
    tree.nodes()
        .iter()
        .position(|n| n.role == role && n.name.as_deref() == Some(name))
        .unwrap_or_else(|| panic!("missing {role:?}: {}", String::from_utf8_lossy(name)))
}

#[expect(clippy::disallowed_types, reason = "tests inspect the semantic tree’s printed form")]
pub(crate) fn print(tree: &Tree) -> String {
    let mut print = skein_ui_text::Print::new(&f::tree_limits()).unwrap();
    skein_ui_text::render(tree, b"temper", None, &mut print);
    String::from_utf8(print.text.as_slice().to_vec()).unwrap()
}

fn event(root: &mut d::Domain, event: d::Event) -> Queue<d::Request> {
    let mut out = Queue::uncapped();
    d::step(root, &skein_lib::Env { now: skein_lib::Time::ZERO, wall: f::NOW, limits: f::limits() }, event, &mut out);
    out
}

fn up(root: &mut d::Domain, up: jig_client::Up) -> Queue<d::Request> {
    event(root, d::Event::Client { event: jig_client::Event::Up { up } })
}

fn opening(out: &Queue<d::Request>, subject: jig_client::Watchable) -> Token {
    if subject == jig_client::Watchable::Jig(c::Subject::Me) {
        return Token::new(0);
    }
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

pub(crate) fn fed(kind: d::WayKind) -> (d::Domain, Token, Token) {
    let (offered, _) = f::welcome::model_with_forge(kind);
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
    let mut more = up(
        &mut root,
        jig_client::Up::Watched { watch: me_watch, event: c::WatchEvent::Delivered(c::Delivery::Me(Box::new(me))) },
    );
    while let Some(r) = more.pop() {
        out.push(r);
    }
    let accounts = opening(&out, jig_client::Watchable::Jig(c::Subject::Accounts));
    let trials = opening(&out, d::forge::trials::watch(&f::limits().forge).unwrap());
    for request in &out {
        if let d::Request::Unlinked {
            request:
                jig_client::UnlinkedRequest::Down {
                    down: jig_client::Down::Read { read, what: jig_client::Readable::Family { record, .. } },
                },
        } = request
            && wire::decode_reading(&record.bytes, &v::CEILINGS).unwrap().read == v::Read::Deployment
        {
            f::deliver_deployment(&mut root, *read, true, true);
        }
    }
    let forges = opening(&out, d::forge::forges::watch(&f::limits().forge).unwrap());
    let forges_doc = v::Forges {
        forges: f::connect_forge::first_rows(f::connect_forge::connected(), f::connect_forge::made()),
        linked: false,
    };
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
    let mut doc = f::accounts::card_document(
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
    f::accounts::deliver(&mut root, accounts, doc);
    assert!(d::pages::accounts::document(root.client()).is_some(), "accounts delivered");
    event(&mut root, d::Event::Act { action: d::Action::Forge(d::forge::Act::TryAccount { account: 1, again: true }) });
    (root, trials, accounts)
}

pub(crate) fn row(step: v::TrialStep, state: v::TrialState) -> v::TrialRow {
    v::TrialRow { run_wait: None, account: 1, task: Some(91), took: None, step, state }
}

pub(crate) fn rows(root: &mut d::Domain, watch: Token, rows: &[v::TrialRow]) -> Queue<d::Request> {
    let doc = v::Trials { rows: List::from_boxed(rows.into()) };
    let mut writer = Writer::new(
        usize::try_from(doc.measure(&temper_client_protocol_wire::v1::CEILINGS).expect("within ceilings")).unwrap(),
    );
    doc.write(&temper_client_protocol_wire::v1::CEILINGS, &mut writer).unwrap();
    up(
        root,
        jig_client::Up::FamilyDelivered {
            watch,
            record: jig_client::Family { family: wire::FORGE, version: wire::CLIENT_VERSION, bytes: writer.finish() },
        },
    )
}

pub(crate) fn failed(why: v::TrialWhy) -> v::TrialState {
    v::TrialState::Failed(v::TrialFailed { why })
}

pub(crate) fn from_root(root: &d::Domain, w: &Words) -> (Tree, Bindings) {
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
fn a_trial_row_draws_its_wait_and_keeps_it_after_timeout() {
    let (mut root, watch, _) = fed(d::WayKind::Program);
    let mut parts = row(v::TrialStep::Answers, v::TrialState::Running);
    let wait = jig_client_protocol_wire::TaskRunWait::Credential;
    let mut writer = Writer::new(
        usize::try_from(wait.measure(&jig_client_protocol_wire::CEILINGS).expect("bounded fixture")).unwrap(),
    );
    wait.write(&jig_client_protocol_wire::CEILINGS, &mut writer).unwrap();
    parts.run_wait = Some(writer.finish());
    let out = rows(&mut root, watch, &[parts]);
    for request in &out {
        if let d::Request::Unlinked {
            request:
                jig_client::UnlinkedRequest::Down {
                    down: jig_client::Down::Open { subject: jig_client::Watchable::Jig(c::Subject::Task { .. }), .. },
                },
        } = request
        {
            panic!("a trial follows its row, not a task watch");
        }
    }
    for state in [v::TrialState::Running, failed(v::TrialWhy::Waited)] {
        if state != v::TrialState::Running {
            rows(&mut root, watch, &[row(v::TrialStep::Answers, state)]);
        }
        for words in [f::words(), f::browser_words()] {
            let trial = model_check::trial(&root, &words).unwrap();
            let State::Stopped { at, why, .. } = trial.state else { panic!("the row's wait stops the trial line") };
            assert_eq!(at, 2);
            assert_eq!(why.as_ref(), b"This work needs a credential from its forge.");
            let tree = from_root(&root, &words).0;
            position(&tree, Role::Paragraph, &why);
        }
    }
}
