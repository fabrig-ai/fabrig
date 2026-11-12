#![allow(
    clippy::disallowed_methods,
    clippy::disallowed_types,
    clippy::disallowed_macros,
    reason = "test assertions inspect semantic trees and diagnostic redaction"
)]
use super::{connect_model::text, fixtures};
use jig_client::contract as c;
use skein_lib::{List, Token};
use skein_ui::{Act, Role};
use temper_client_domain::{
    self as d,
    pages::accounts::{Act as A, Operation, Row},
};
use temper_client_protocol_wire::{self as wire, v1 as v};
const PROVIDER: u32 = 17;
const SOURCE: u32 = 43;
const ADDRESS: &[u8] = b"https://issuer.example/visit";
const TRIED_ACCOUNT: u32 = 1;
const OTHER_ACCOUNT: u32 = 2;
const TRIAL_TASK: u64 = 91;

fn trial_event(root: &mut d::Domain, event: d::Event) -> skein_lib::Queue<d::Request> {
    let mut out = skein_lib::Queue::uncapped();
    d::step(
        root,
        &skein_lib::Env { now: skein_lib::Time::ZERO, wall: fixtures::NOW, limits: fixtures::limits() },
        event,
        &mut out,
    );
    out
}

fn opened(out: &skein_lib::Queue<d::Request>, subject: jig_client::Watchable) -> Token {
    out.iter()
        .find_map(|request| {
            if let d::Request::Unlinked {
                request: jig_client::UnlinkedRequest::Down { down: jig_client::Down::Open { watch, subject: found } },
            } = request
            {
                (*found == subject).then_some(*watch)
            } else {
                None
            }
        })
        .expect("fixture watch opened")
}

fn trial_root(kind: c::AccountKind) -> (d::Domain, Token, Token) {
    let (frame, _) = fixtures::account_root();
    let me = d::pages::accounts::current_me(frame.client()).unwrap().clone();
    let mut root = d::unlinked(&fixtures::limits(), fixtures::SEED);
    let mut out = skein_lib::Queue::uncapped();
    for event in [
        d::Event::Client { event: jig_client::Event::StartUnlinked { kept: List::with_capacity(0) } },
        d::Event::Start {
            address: d::Address::Accounts,
            kept: List::with_capacity(0),
            with: Some(jig_client::With::Transport),
        },
        d::Event::Client {
            event: jig_client::Event::Up { up: jig_client::Up::Served { served: true, versions: None } },
        },
    ] {
        d::step(
            &mut root,
            &skein_lib::Env { now: skein_lib::Time::ZERO, wall: fixtures::NOW, limits: fixtures::limits() },
            event,
            &mut out,
        );
    }
    let me_watch = opened(&out, jig_client::Watchable::Jig(c::Subject::Me));
    let accounts = opened(&out, jig_client::Watchable::Jig(c::Subject::Accounts));
    let out = trial_event(
        &mut root,
        d::Event::Client {
            event: jig_client::Event::Up {
                up: jig_client::Up::Watched {
                    watch: me_watch,
                    event: c::WatchEvent::Delivered(c::Delivery::Me(Box::new(me))),
                },
            },
        },
    );
    let trials = opened(&out, d::forge::trials::watch(&fixtures::limits().forge).unwrap());
    let provider = if kind == c::AccountKind::Program { b"anthropic".as_slice() } else { b"openai" };
    deliver(&mut root, accounts, card_document(provider, b"Pat", kind));
    (root, trials, accounts)
}

fn trial_row(step: v::TrialStep, state: v::TrialState) -> v::TrialRow {
    v::TrialRow::new(
        &v::CEILINGS,
        v::TrialRowParts { account: TRIED_ACCOUNT, task: Some(TRIAL_TASK), took: None, step, state },
    )
    .unwrap()
}

fn trial_failed(why: v::TrialWhy) -> v::TrialState {
    v::TrialState::Failed(v::TrialFailed::new(&v::CEILINGS, v::TrialFailedParts { why }).unwrap())
}

fn trial_rows(root: &mut d::Domain, watch: Token, rows: &[v::TrialRow]) {
    let doc = v::Trials::new(&v::CEILINGS, v::TrialsParts { rows: List::from_boxed(rows.into()) }).unwrap();
    let mut writer = skein_lib::Writer::new(usize::try_from(doc.measure()).unwrap());
    doc.encode(&mut writer).unwrap();
    trial_event(
        root,
        d::Event::Client {
            event: jig_client::Event::Up {
                up: jig_client::Up::FamilyDelivered {
                    watch,
                    record: jig_client::Family {
                        family: wire::FORGE,
                        version: wire::CLIENT_VERSION,
                        bytes: writer.finish(),
                    },
                },
            },
        },
    );
}

fn account_nodes(tree: &skein_ui::Tree) -> &[skein_ui::Node] {
    super::inbox::subtree(tree, super::inbox::styled(tree, crate::look::SETTINGS_ROW))
}

/// The account's trial is under its head and before its sources (temper/docs/design/domain/trial.md, 6.2).
#[test]
fn a_row_holds_its_trials_line() {
    let answered =
        v::TrialState::Answered(v::TrialAnswered::new(&v::CEILINGS, v::TrialAnsweredParts { seconds: 6 }).unwrap());
    for (kind, step, state, line, style, each_step) in [
        (
            c::AccountKind::Program,
            v::TrialStep::Starts,
            v::TrialState::Running,
            "Claude subscription · trying: Claude Code starts on this machine",
            None,
            true,
        ),
        (
            c::AccountKind::Program,
            v::TrialStep::Answers,
            v::TrialState::Running,
            "Claude subscription · trying: Claude answers",
            None,
            true,
        ),
        (
            c::AccountKind::SignIn,
            v::TrialStep::Starts,
            v::TrialState::Running,
            "ChatGPT plan · trying: ChatGPT answers",
            None,
            true,
        ),
        (
            c::AccountKind::Key,
            v::TrialStep::Answers,
            v::TrialState::Running,
            "OpenAI API key · trying: ChatGPT answers",
            None,
            true,
        ),
        (
            c::AccountKind::SignIn,
            v::TrialStep::Answers,
            answered,
            "ChatGPT plan · Pat · answered in 6 s",
            Some(crate::look::FLOW_DONE),
            false,
        ),
        (
            c::AccountKind::SignIn,
            v::TrialStep::Answers,
            trial_failed(v::TrialWhy::Waited),
            "ChatGPT plan · ChatGPT didn’t answer",
            Some(crate::look::FLOW_FAILED),
            true,
        ),
        (
            c::AccountKind::Key,
            v::TrialStep::Starts,
            trial_failed(v::TrialWhy::Refused),
            "OpenAI API key · ChatGPT didn’t answer",
            Some(crate::look::FLOW_FAILED),
            true,
        ),
    ] {
        let (mut root, trials, _) = trial_root(kind);
        trial_rows(&mut root, trials, &[trial_row(step, state)]);
        for words in [fixtures::words(), fixtures::browser_words()] {
            let mut view = fixtures::view_of(words);
            let tree = fixtures::build(&mut view, &root);
            let nodes = account_nodes(tree);
            let head = nodes.iter().position(|n| n.role == Role::Disclosure).unwrap();
            let at = nodes
                .iter()
                .position(|n| n.role == Role::Paragraph && n.name.as_deref() == Some(line.as_bytes()))
                .expect(line);
            let sources = nodes
                .iter()
                .position(|n| n.name.as_deref() == Some(b"Its sources in order: a run takes the first usable one."))
                .unwrap();
            assert!(head < at && at < sources, "{line}");
            if let Some(style) = style {
                assert!(nodes[at].styles.as_slice().contains(&style), "{line}");
            }
            assert_eq!(
                nodes.iter().any(|n| n.role == Role::Disclosure && n.name.as_deref() == Some(b"Each step")),
                each_step,
                "{line}"
            );
        }
    }
}

/// A trial follows `took`, leaving the account that started it alone (domain/trial.md, 6.2).
#[test]
fn a_row_answered_for_another_account_says_nothing() {
    // The fixture has the account that started the trial and the one that answered.
    const ACCOUNT_ROWS: usize = 2;
    let (mut root, trials, accounts) = trial_root(c::AccountKind::SignIn);
    let mut doc = card_document(b"openai", b"Pat", c::AccountKind::SignIn);
    let mut other = doc.accounts.get(0).unwrap().clone();
    other.number = OTHER_ACCOUNT;
    other.display = text(b"Other");
    doc.accounts = List::from_boxed(Box::new([doc.accounts.get(0).unwrap().clone(), other]));
    deliver(&mut root, accounts, doc);
    let mut row = trial_row(
        v::TrialStep::Answers,
        v::TrialState::Answered(v::TrialAnswered::new(&v::CEILINGS, v::TrialAnsweredParts { seconds: 6 }).unwrap()),
    )
    .into_parts();
    row.took = Some(OTHER_ACCOUNT);
    trial_rows(&mut root, trials, &[v::TrialRow::new(&v::CEILINGS, row).unwrap()]);
    for words in [fixtures::words(), fixtures::browser_words()] {
        let mut view = fixtures::view_of(words);
        let tree = fixtures::build(&mut view, &root);
        let rows: Vec<_> = tree
            .nodes()
            .iter()
            .enumerate()
            .filter(|(_, n)| n.styles.as_slice().contains(&crate::look::SETTINGS_ROW))
            .collect();
        assert_eq!(rows.len(), ACCOUNT_ROWS);
        for ((at, _), answered) in rows.into_iter().zip([false, true]) {
            let nodes = super::inbox::subtree(tree, at);
            assert_eq!(
                nodes.iter().any(|n| n.name.as_deref() == Some("ChatGPT plan · Other · answered in 6 s".as_bytes())),
                answered
            );
            assert_eq!(nodes.iter().any(|n| n.styles.as_slice().contains(&crate::look::FLOW_DONE)), answered);
        }
    }
}

/// A row's answering act opens `ConnectModel`; no answering act offers no All ways (domain/trial.md, 6.2).
#[test]
fn sign_in_again_on_a_row_goes_to_the_ways_page() {
    for (kind, way, offered) in [
        (c::AccountKind::Program, d::WayKind::Program, true),
        (c::AccountKind::SignIn, d::WayKind::SignIn, true),
        (c::AccountKind::Key, d::WayKind::Key, false),
    ] {
        let (mut root, trials, _) = trial_root(kind);
        let why = v::TrialWhy::Ended(
            v::TrialEnded::new(&v::CEILINGS, v::TrialEndedParts { class: v::TrialClass::Permanent }).unwrap(),
        );
        trial_rows(&mut root, trials, &[trial_row(v::TrialStep::Answers, trial_failed(why))]);
        for words in [fixtures::words(), fixtures::browser_words()] {
            let mut view = fixtures::view_of(words);
            let tree = fixtures::build(&mut view, &root);
            assert!(fixtures::find(tree, Role::Button, b"Try again").is_some());
            assert!(fixtures::find(tree, Role::Link, b"All ways").is_none());
            let sign_in = fixtures::find(tree, Role::Link, b"Sign in again");
            assert_eq!(sign_in.is_some(), offered);
            if let Some(sign_in) = sign_in {
                assert_eq!(
                    crate::act(&mut view, sign_in, Act::Press),
                    Some(d::Action::Pages(d::pages::Act::Go {
                        address: d::Address::ConnectModel {
                            way: Some(d::ModelWay { source: None, provider: PROVIDER, kind: way })
                        }
                    }))
                );
            }
        }
    }
}

/// Try again targets the trial's account with `again: true` (domain/trial.md, 6).
#[test]
fn try_again_on_a_row_runs_another() {
    let (mut root, trials, _) = trial_root(c::AccountKind::SignIn);
    trial_rows(&mut root, trials, &[trial_row(v::TrialStep::Answers, trial_failed(v::TrialWhy::Refused))]);
    for words in [fixtures::words(), fixtures::browser_words()] {
        let mut view = fixtures::view_of(words);
        let tree = fixtures::build(&mut view, &root);
        let retry = fixtures::find(tree, Role::Button, b"Try again").unwrap();
        assert_eq!(
            crate::act(&mut view, retry, Act::Press),
            Some(d::Action::Forge(d::forge::Act::TryAccount { account: TRIED_ACCOUNT, again: true }))
        );
    }
}

/// Without a trial the existing row gains no nodes (domain/trial.md, 6.2).
#[test]
fn a_row_with_no_trial_reads_as_today() {
    let (mut root, trials, _) = trial_root(c::AccountKind::SignIn);
    let before: Vec<_> = [fixtures::words(), fixtures::browser_words()]
        .into_iter()
        .map(|words| {
            let mut view = fixtures::view_of(words);
            account_nodes(fixtures::build(&mut view, &root)).to_vec()
        })
        .collect();
    trial_rows(&mut root, trials, &[]);
    for (words, before) in [fixtures::words(), fixtures::browser_words()].into_iter().zip(before) {
        let mut view = fixtures::view_of(words);
        let nodes = account_nodes(fixtures::build(&mut view, &root));
        assert_eq!(nodes, before);
        assert!(!nodes.iter().any(|n| n.styles.as_slice().contains(&crate::look::FLOW_DONE)
            || n.name.as_deref() == Some(b"Each step")
            || n.styles.as_slice().contains(&crate::look::FLOW_FAILED)));
    }
}
pub(super) fn card_document(provider: &[u8], display: &[u8], kind: c::AccountKind) -> c::Accounts {
    let mut doc = document(1);
    let row = doc.accounts.get_mut(0).unwrap();
    row.provider_name = Some(text(provider));
    row.display = text(display);
    row.plan = kind != c::AccountKind::Key;
    let source = row.sources.get_mut(0).unwrap();
    source.kind = kind;
    if kind == c::AccountKind::Program {
        source.name = Some(Box::from(display));
    }
    doc
}
pub(super) fn email_facts(email: Option<&[u8]>) -> List<c::AccountFact> {
    match email {
        Some(email) => List::from_boxed(Box::new([c::AccountFact {
            kind: c::FACT_EMAIL,
            label: Box::new([]),
            value: c::AccountFactValue::Text(email.into()),
            seen: fixtures::NOW,
            source: Some(SOURCE),
            origin: 1,
        }])),
        None => List::with_capacity(0),
    }
}
fn card_heading(doc: c::Accounts) -> Vec<u8> {
    let (mut root, watch) = fixtures::account_root();
    deliver(&mut root, watch, doc);
    let mut view = fixtures::view();
    fixtures::build(&mut view, &root)
        .nodes()
        .iter()
        .find(|node| node.role == Role::Disclosure && node.styles.as_slice().contains(&crate::look::SETTINGS_ROW))
        .unwrap()
        .name
        .as_ref()
        .unwrap()
        .to_vec()
}
#[test]
fn a_row_names_the_account_without_an_inner_heading() {
    let (mut root, watch) = fixtures::account_root();
    deliver(&mut root, watch, card_document(b"openai", b"alice@example.com", c::AccountKind::SignIn));
    let mut view = fixtures::view();
    let tree = fixtures::build(&mut view, &root);
    let heading = tree
        .nodes()
        .iter()
        .find(|node| node.role == Role::Disclosure && node.styles.as_slice().contains(&crate::look::SETTINGS_ROW))
        .unwrap();
    assert_eq!(heading.name.as_deref(), Some("ChatGPT · alice@example.com · OpenAI · held by Pat".as_bytes()));
    assert!(!tree.nodes().iter().any(|node| node.role == Role::Heading { level: 3 }));
}
#[test]
fn a_plan_card_names_the_plan_display_brand_and_holder() {
    for (email, display, who) in [
        (None, b"Legacy".as_slice(), "Legacy"),
        (Some(b"alice@example.com".as_slice()), b"", "alice@example.com"),
        (Some(b"alice@example.com".as_slice()), b"Different display", "alice@example.com"),
    ] {
        let (mut root, me_watch, _, _, watch) =
            fixtures::root_at_full(d::Address::Accounts, &fixtures::domain_limits(), b"Shop");
        let mut me = d::pages::accounts::current_me(root.client()).unwrap().clone();
        me.party.name = text(b"Alice");
        fixtures::deliver_me(&mut root, me_watch, me);
        let mut doc = card_document(b"openai", display, c::AccountKind::SignIn);
        doc.accounts.get_mut(0).unwrap().facts = email_facts(email);
        deliver(&mut root, watch.unwrap(), doc);
        for view in [fixtures::page_view(&root), fixtures::browser_page_view(&root)] {
            let title = format!("ChatGPT · {who} · OpenAI · held by Alice");
            assert!(fixtures::find(view.tree(), Role::Disclosure, title.as_bytes()).is_some());
        }
    }
}
#[test]
fn a_key_only_card_names_the_brand_and_holder_once() {
    assert_eq!(
        card_heading(card_document(b"openai", b"", c::AccountKind::Key)),
        "OpenAI API key · held by Pat".as_bytes()
    );
}
/// The key account's row controls are view reads (temper/docs/design/client/web/testing.md, 6).
#[test]
fn a_key_account_offers_replacement_and_removal() {
    let (mut root, watch) = fixtures::account_root();
    deliver(&mut root, watch, card_document(b"openai", b"", c::AccountKind::Key));
    let view = fixtures::page_view(&root);
    let row = super::inbox::styled(view.tree(), crate::look::SETTINGS_ROW);
    let content = super::inbox::subtree(view.tree(), row);
    for name in [b"Replace".as_slice(), b"Remove account"] {
        assert!(content.iter().any(|node| node.role == Role::Button && node.name.as_deref() == Some(name)), "{name:?}");
    }
}
#[test]
fn a_program_card_names_the_subscription_and_login() {
    for (email, display, source, expected) in [
        (None, b"".as_slice(), Some(b"claude-alice".as_slice()), "claude-alice"),
        (None, b"Legacy", None, "Legacy"),
        (
            Some(b"alice@example.com".as_slice()),
            b"Different display",
            Some(b"claude-alice".as_slice()),
            "alice@example.com",
        ),
    ] {
        let mut doc = card_document(b"anthropic", display, c::AccountKind::Program);
        let row = doc.accounts.get_mut(0).unwrap();
        row.facts = email_facts(email);
        row.sources.get_mut(0).unwrap().name = source.map(Box::from);
        assert_eq!(card_heading(doc), format!("Claude subscription · {expected} · Anthropic · held by Pat").as_bytes());
    }
}
#[test]
fn a_deployment_card_names_its_holder_in_words() {
    for (email, expected) in [
        (None, "ChatGPT · Legacy · OpenAI · held by the deployment"),
        (Some(b"alice@example.com".as_slice()), "ChatGPT · alice@example.com · OpenAI · held by the deployment"),
    ] {
        let (mut root, watch) = owner_root();
        let mut doc = root.pages().accounts().unwrap().deployment(root.client()).unwrap().clone();
        let row = doc.accounts.get_mut(0).unwrap();
        row.provider_name = Some(text(b"openai"));
        row.plan = true;
        row.display = text(b"Legacy");
        row.facts = email_facts(email);
        let mut login = source(SOURCE);
        login.kind = c::AccountKind::SignIn;
        row.sources = List::from_boxed(Box::new([login]));
        deliver_deployment(&mut root, watch, doc);
        for view in [fixtures::page_view(&root), fixtures::browser_page_view(&root)] {
            assert!(fixtures::find(view.tree(), Role::Disclosure, expected.as_bytes()).is_some());
        }
    }
    let mut doc = card_document(b"openai", b"", c::AccountKind::Key);
    doc.accounts.get_mut(0).unwrap().holder = c::AccountHolder::Deployment;
    assert_eq!(card_heading(doc), "OpenAI API key · held by the deployment".as_bytes());
}
#[test]
fn an_empty_display_leaves_no_empty_heading_part() {
    assert_eq!(
        card_heading(card_document(b"openai", b"", c::AccountKind::SignIn)),
        "ChatGPT · OpenAI · held by Pat".as_bytes()
    );
}
#[test]
fn sources_name_their_kinds_in_the_providers_words() {
    for (provider, display, kind, expected) in [
        (b"openai".as_slice(), b"alice@example.com".as_slice(), c::AccountKind::SignIn, "ChatGPT sign-in · Ready"),
        (b"openai", b"", c::AccountKind::Key, "API key · Ready"),
        (b"anthropic", b"claude-alice", c::AccountKind::Program, "Claude Code login claude-alice · Ready"),
    ] {
        assert!(shown(card_document(provider, display, kind)).contains(expected));
    }
}
#[test]
fn account_management_keeps_both_account_lists() {
    let (root, _) = owner_root();
    let mut view = fixtures::view();
    let tree = fixtures::build(&mut view, &root);
    let headings: Vec<_> = tree
        .nodes()
        .iter()
        .filter(|node| node.role == Role::Heading { level: 2 })
        .map(|node| node.name.as_ref().unwrap().to_vec())
        .collect();
    assert_eq!(headings, [b"Your accounts".to_vec(), b"Deployment accounts".to_vec()]);
}
#[test]
fn the_tab_draws_your_accounts_with_its_count_and_act_in_its_head() {
    let (mut root, watch) = fixtures::account_root();
    deliver(&mut root, watch, card_document(b"openai", b"Pat", c::AccountKind::SignIn));
    for mut view in [fixtures::page_view(&root), fixtures::browser_page_view(&root)] {
        let tree = view.tree();
        let row = super::inbox::subtree(tree, super::inbox::styled(tree, crate::look::SETTINGS_SECTION_HEAD));
        assert!(row[0].styles.as_slice().contains(&crate::look::ACCOUNTS_PAGE));
        assert!(row.iter().any(|node| node.role == Role::Heading { level: 2 }
            && node.name.as_deref() == Some("Your accounts · 1".as_bytes())
            && node.styles.as_slice().contains(&crate::look::SETTINGS_SECTION_TITLE)));
        let act = row
            .iter()
            .find(|node| node.role == Role::Link && node.name.as_deref() == Some(b"+ Connect a model"))
            .unwrap();
        assert!(act.styles.as_slice().contains(&crate::look::SETTINGS_BTN));
        assert!(act.styles.as_slice().contains(&crate::look::SETTINGS_SMALL));
        let id = act.id;
        assert_eq!(
            crate::act(&mut view, id, Act::Press),
            Some(d::Action::Pages(d::pages::Act::Go { address: d::Address::ConnectModel { way: None } }))
        );
    }
}
#[test]
fn a_cut_provider_name_keeps_the_numbered_card_fallback() {
    let mut doc = card_document(b"openai", b"Pat", c::AccountKind::SignIn);
    doc.accounts.get_mut(0).unwrap().provider_name.as_mut().unwrap().cut = true;
    assert_eq!(card_heading(doc), "Account 1 · LLM provider 17 · Pat".as_bytes());
}
#[test]
fn an_unknown_provider_keeps_its_configured_name() {
    assert_eq!(
        card_heading(card_document(b"Custom", b"Pat", c::AccountKind::SignIn)),
        "Custom · Pat · Custom · held by Pat".as_bytes()
    );
}
fn source(number: u32) -> c::AccountSource {
    c::AccountSource {
        name: None,
        number,
        place: number,
        kind: c::AccountKind::Key,
        state: c::AccountState::Granted,
        usable: true,
        expiring: false,
        attention: false,
        spent_until: None,
    }
}
pub(super) fn document(count: u32) -> c::Accounts {
    let mut accounts = List::with_capacity(count);
    let mut signing = List::with_capacity(count);
    for number in 1..=count {
        let mut sources = List::with_capacity(count);
        for n in 1..=count {
            sources.push(source(n)).unwrap();
        }
        accounts
            .push(c::AccountLine {
                list: None,
                facts: List::with_capacity(0),
                number,
                place: number,
                uses: None,
                plan: true,
                provider: Some(PROVIDER),
                provider_name: Some(c::DocumentText::complete(
                    c::Text::new(Box::from(b"Scripted".as_slice()), 32).unwrap(),
                )),
                manage_usage: None,
                display: c::DocumentText::complete(c::Text::new(Box::from(b"Pat".as_slice()), 32).unwrap()),
                holder: c::AccountHolder::Party(73),
                sources,
                cut: true,
            })
            .unwrap();
        signing
            .push(c::AccountSignIn { source: number, provider: PROVIDER, holder: c::AccountHolder::Party(73) })
            .unwrap();
    }
    c::Accounts {
        program_hosts: List::with_capacity(0),
        readiness: List::with_capacity(0),
        readiness_cut: false,
        accounts,
        signing,
        last_end: None,
        usable: true,
        settled: false,
        cut: true,
    }
}
pub(super) fn deliver(root: &mut d::Domain, watch: Token, doc: c::Accounts) {
    fixtures::root_event(
        root,
        d::Event::Client {
            event: jig_client::Event::Up {
                up: jig_client::Up::Watched {
                    watch,
                    event: c::WatchEvent::Delivered(c::Delivery::Accounts(Box::new(doc))),
                },
            },
        },
    );
}
fn action(root: &mut d::Domain, act: A) {
    fixtures::root_event(root, d::Event::Act { action: d::Action::Pages(d::pages::Act::Accounts(act)) });
}
pub(super) fn press(view: &mut crate::View, root: &mut d::Domain, role: Role, name: &[u8], act: Act) {
    let node = fixtures::find(fixtures::build(view, root), role, name).expect("offered account control");
    let event = crate::event(view, node, act).expect("account control decodes");
    if let d::Event::Act { action: d::Action::Pages(d::pages::Act::Go { address }) } = event {
        fixtures::root_event(root, d::Event::Went { place: d::Place::Address(address) });
    } else {
        fixtures::root_event(root, event);
    }
}
fn shown(doc: c::Accounts) -> String {
    shown_at(doc, fixtures::limits().client.domain.accounts)
}
fn shown_at(doc: c::Accounts, counts: jig_client::AccountCounts) -> String {
    let (mut root, watch) = fixtures::account_root();
    deliver(&mut root, watch, doc);
    let mut view = bounded_view(counts);
    let tree = fixtures::build(&mut view, &root);
    let mut text = String::from_utf8(fixtures::text(tree).into_vec()).unwrap();
    for node in tree.nodes() {
        if node.role == Role::Disclosure && node.styles.as_slice().contains(&crate::look::SETTINGS_ROW) {
            for summary in [&node.name, &node.description].into_iter().flatten() {
                text.push_str(&String::from_utf8_lossy(summary));
            }
        }
    }
    text
}
fn bounded_view(counts: jig_client::AccountCounts) -> crate::View {
    let mut domain = fixtures::limits();
    let words = fixtures::words();
    let bounds = crate::Limits::of(&domain, &words).unwrap();
    domain.client.domain.accounts = counts;
    crate::View::new(&bounds, &domain, words, fixtures::FORGE)
}
#[test]
fn the_accounts_page_keeps_accounts_beyond_rows() {
    assert!(account_order(document(3)).contains(&3));
}
#[test]
fn the_accounts_page_keeps_sources_beyond_rows() {
    assert_eq!(shown(document(3)).matches("3. API key").count(), 3);
}
#[test]
fn the_accounts_page_keeps_sign_ins_beyond_rows() {
    assert!(shown(document(3)).contains("Sign-in in progress: source 3"));
}
#[test]
fn a_sign_in_without_an_account_row_stays_visible_at_page_level() {
    for count in [0, 1] {
        let (mut root, watch) = fixtures::account_root();
        let mut doc = document(count);
        doc.signing = List::with_capacity(1);
        doc.signing
            .push(c::AccountSignIn { source: SOURCE, provider: PROVIDER, holder: c::AccountHolder::Party(73) })
            .unwrap();
        deliver(&mut root, watch, doc);
        let mut view = fixtures::view();
        let tree = fixtures::build(&mut view, &root);
        let status = fixtures::person_find(
            tree,
            Role::Status,
            b"Sign-in in progress: source 43 \xc2\xb7 LLM provider 17 \xc2\xb7 Person 73",
        )
        .expect("another client's sign-in stays visible before it has an account row");
        let at = tree.nodes().iter().position(|node| node.id == status).unwrap();
        assert!(!tree.nodes().iter().enumerate().any(|(start, node)| {
            node.role == Role::Disclosure && start < at && at < start + usize::try_from(node.size).unwrap()
        }));
    }
}
#[test]
fn a_sign_in_in_an_omitted_row_stays_visible_without_duplicating_drawn_rows() {
    let doc = document(1);
    let counts = fixtures::limits().client.domain.accounts;
    assert_eq!(shown_at(doc.clone(), counts).matches("Sign-in in progress: source 1").count(), 1);
    let shown = shown_at(doc, jig_client::AccountCounts { accounts: 0, ..counts });
    assert!(shown.contains("Sign-in in progress: source 1 · LLM provider 17 · Person 73"));
}
#[test]
fn the_accounts_page_preserves_supplied_order() {
    let mut doc = document(3);
    let first = doc.accounts.get(0).unwrap().clone();
    let third = doc.accounts.get(2).unwrap().clone();
    *doc.accounts.get_mut(0).unwrap() = third;
    *doc.accounts.get_mut(2).unwrap() = first;
    assert_eq!(account_order(doc), [3, 2, 1]);
}
#[test]
fn the_accounts_page_names_cut_facts() {
    assert!(shown(document(1)).contains("Some account facts were omitted."));
}
#[test]
fn the_accounts_page_caps_uses() {
    let mut doc = document(1);
    let mut uses = List::with_capacity(300);
    for party in 1..=300 {
        uses.push(c::AccountUse::Party(party)).unwrap();
    }
    doc.accounts.get_mut(0).unwrap().uses = Some(uses);
    let text = shown(doc);
    assert!(text.contains("Uses left out: 292"));
    assert!(!text.contains("Person 9;"));
}
fn key_draft() -> (d::Domain, crate::View) {
    let (mut root, watch) = fixtures::account_root();
    deliver(&mut root, watch, card_document(b"openai", b"alice@example.com", c::AccountKind::SignIn));
    let mut view = fixtures::view();
    press(&mut view, &mut root, Role::Button, b"Add an API key", Act::Press);
    press(
        &mut view,
        &mut root,
        Role::TextField,
        b"Key (pasted once)",
        Act::Input { value: Box::from(b"private-once".as_slice()) },
    );
    (root, view)
}
fn key_field(view: &crate::View) -> &skein_ui::Node {
    view.tree().nodes().iter().find(|n| n.name.as_deref() == Some(b"Key (pasted once)")).unwrap()
}
#[test]
fn the_key_field_declares_a_secret_entry() {
    let (_, view) = key_draft();
    assert_eq!(key_field(&view).entry, Some(skein_ui::EntryKind::Secret));
}
#[test]
fn a_key_draft_never_enters_the_view() {
    let (_, view) = key_draft();
    assert!(!format!("{view:?}").contains("private-once"));
}
#[test]
fn a_held_key_has_an_empty_view_value() {
    let (root, view) = key_draft();
    assert!(root.pages().accounts().unwrap().request.has_key());
    assert!(key_field(&view).value.as_ref().unwrap().text.is_empty());
}
#[test]
fn sending_a_key_writes_an_empty_field() {
    let (mut root, mut view) = key_draft();
    let before = key_field(&view).value.as_ref().unwrap().written;
    press(&mut view, &mut root, Role::Button, b"Add key", Act::Press);
    fixtures::build(&mut view, &root);
    let value = key_field(&view).value.as_ref().unwrap();
    assert!(value.text.is_empty());
    assert_ne!(value.written, before);
}
pub(super) fn sign_in_address(address: &[u8]) -> d::Domain {
    let mut root = super::connect_model::offered_root();
    let limits = fixtures::limits();
    let env = skein_lib::Env { now: skein_lib::Time::ZERO, wall: skein_lib::Wall::EPOCH, limits };
    let mut out = skein_lib::Queue::uncapped();
    d::step(
        &mut root,
        &env,
        d::Event::Act {
            action: d::Action::Pages(d::pages::Act::ConnectModel(d::pages::connect_model::Act::SignIn {
                name: [97; 16],
                way: d::pages::accounts::Way::SignIn { provider: PROVIDER, plan: true },
            })),
        },
        &mut out,
    );
    fixtures::root_event(
        &mut root,
        d::Event::Went {
            place: d::Place::Address(d::Address::ConnectModel {
                way: Some(d::ModelWay { source: None, provider: PROVIDER, kind: d::WayKind::SignIn }),
            }),
        },
    );
    d::step(&mut root, &env, d::Event::Client { event: jig_client::Event::Stored }, &mut out);
    let attempt = out
        .iter()
        .find_map(|request| match request {
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
        .expect("sign-in attempt");
    fixtures::root_event(
        &mut root,
        d::Event::Client {
            event: jig_client::Event::Up {
                up: jig_client::Up::Answer {
                    attempt,
                    envelope: skein_lib::RequestEnvelope::Final,
                    answer: Some(jig_client::Answered::Jig(c::Answer::Done(c::Outcome::Document(
                        c::DocumentOutcome::AccountSignInStarted { source: SOURCE, address: Box::from(address) },
                    )))),
                    why: None,
                },
            },
        },
    );
    root
}
#[test]
fn a_sign_in_address_has_a_visible_link_word() {
    let root = sign_in_address(ADDRESS);
    let view = fixtures::browser_page_view(&root);
    assert!(String::from_utf8(fixtures::text(view.tree()).into_vec()).unwrap().contains("Open it again ›"));
}
#[test]
fn an_address_containing_a_space_is_refused() {
    unsafe_address(b"https://issuer.example/bad path");
}
#[test]
fn an_uppercase_scheme_is_refused() {
    unsafe_address(b"HTTPS://issuer.example/visit");
}
fn unsafe_address(address: &[u8]) {
    let root = sign_in_address(address);
    let mut view = fixtures::view();
    let tree = fixtures::build(&mut view, &root);
    assert!(!tree.nodes().iter().any(|n| n.external.is_some()));
    assert!(String::from_utf8(fixtures::text(tree).into_vec()).unwrap().contains("not an http(s) address"));
}
#[test]
fn a_long_sign_in_address_is_refused() {
    let address = format!("https://issuer.example/{}", "x".repeat(1000));
    let root = sign_in_address(address.as_bytes());
    let mut view = fixtures::view();
    let tree = fixtures::build(&mut view, &root);
    assert!(!tree.nodes().iter().any(|n| n.external.is_some()));
    let text = String::from_utf8(fixtures::text(tree).into_vec()).unwrap();
    assert!(text.contains(&format!("longer than this client shows; bytes: {}", address.len())));
}

#[test]
fn supplied_source_states_keep_their_words() {
    use c::AccountState as S;
    let (mut root, watch) = fixtures::account_root();
    let mut view = fixtures::view();
    for (state, expected) in [
        (S::Asking, b"Checking".as_slice()),
        (S::Granted, b"Ready"),
        (S::Later { until: None }, b"Limited. Tasks on it wait."),
        (S::SignInAgain, b"Needs sign-in"),
        (S::Misconfigured, b"Check the configuration's registration."),
        (S::Cancelled, b"Cancelled"),
        (S::Closing, b"Closing"),
        (S::Moving, b"Moving to another deployment"),
        (S::Unloaded { at: c::AccountPlace::File, why: c::AccountUnread::Stalled }, b"Cannot load File: Stalled"),
    ] {
        let mut doc = document(1);
        doc.accounts.get_mut(0).unwrap().sources.get_mut(0).unwrap().state = state;
        deliver(&mut root, watch, doc);
        let shown = fixtures::text(fixtures::build(&mut view, &root));
        assert!(skein_lib::bytes::find(&shown, expected).is_some(), "{state:?}");
        assert!(skein_lib::bytes::find(&shown, b"Limited until").is_none(), "no invented reset");
    }
}
#[test]
fn move_refusals_keep_their_words() {
    use c::{AccountEnd as E, AccountRefusal as R};
    for (refusal, expected) in [
        (R::NotSignedIn, b"Not signed in".as_slice()),
        (R::NotOurs, b"Not this application's record"),
        (R::Cancelled, b"The move was cancelled"),
        (R::Moving, b"Moving to another deployment"),
    ] {
        assert_eq!(sign_in_end_words(document(0), E::Refused { refusal }), expected, "{refusal:?}");
    }
}
#[test]
fn a_supplied_reset_is_visible() {
    let (mut root, watch) = fixtures::account_root();
    let mut view = fixtures::view();
    let mut doc = document(1);
    doc.accounts.get_mut(0).unwrap().sources.get_mut(0).unwrap().state =
        c::AccountState::Later { until: Some(skein_lib::Wall::from_nanos(1_791_828_240_000_000_000)) };
    deliver(&mut root, watch, doc);
    assert!(
        skein_lib::bytes::find(&fixtures::text(fixtures::build(&mut view, &root)), b"Limited until Oct 12, 18:04 UTC")
            .is_some()
    );
}
#[test]
fn sign_in_ends_preserve_their_typed_details() {
    use c::{AccountEnd as E, AccountFailure as F, AccountRefusal as R};
    for (end, expected) in [
        (E::Made { account: 29, declined: true }, b"You signed in without plan usage.".as_slice()),
        (E::Renewed { account: 29, declined: false }, b"same account again; account 29"),
        (E::Refused { refusal: R::NotKept { why: c::AccountKeyFailure::TooLong } }, b"The key is too long."),
        (E::Refused { refusal: R::Limit { bound: c::AccountBound::Sources } }, b"Source limit reached."),
        (E::Refused { refusal: R::Kind { kind: c::AccountKind::Program } }, b"Source kind refused: Program login"),
        (E::Failed { failure: F::MissingScope { scope: 19 } }, b"Missing required scope 19"),
        (E::Failed { failure: F::Later { after: Some(skein_lib::Duration::from_secs(5)) } }, b"Try again after 5"),
    ] {
        let mut doc = document(0);
        doc.last_end = Some(c::AccountLastEnd { source: SOURCE, end });
        assert!(skein_lib::bytes::find(&sign_in_end_words(doc, end), expected).is_some());
    }
}
fn sign_in_end_words(mut doc: c::Accounts, end: c::AccountEnd) -> Vec<u8> {
    doc.last_end = Some(c::AccountLastEnd { source: SOURCE, end });
    let mut b = skein_ui::Builder::new(&fixtures::tree_limits());
    b.open(Role::Main);
    crate::pages::accounts::end(&mut b, &fixtures::words(), &end, &doc.accounts);
    b.close();
    let tree = b.finish();
    for (index, status) in tree.nodes().iter().enumerate() {
        if status.role != Role::Status {
            continue;
        }
        let node = tree.nodes().get(index.checked_add(1).unwrap()).unwrap();
        let mut words = Vec::new();
        for run in &node.text {
            words.extend_from_slice(&run.text);
        }
        if words.starts_with(b"Signed in") || words.starts_with("That’s".as_bytes()) {
            return words;
        }
    }
    fixtures::text(&tree).into_vec()
}
#[test]
fn a_completed_sign_in_names_the_plan_and_display() {
    for (email, display) in [
        (None, b"alice@example.com".as_slice()),
        (Some(b"alice@example.com".as_slice()), b""),
        (Some(b"alice@example.com".as_slice()), b"Different display"),
    ] {
        let mut doc = card_document(b"openai", display, c::AccountKind::SignIn);
        doc.accounts.get_mut(0).unwrap().facts = email_facts(email);
        assert_eq!(
            sign_in_end_words(doc, c::AccountEnd::Made { account: 1, declined: false }),
            b"Signed in to ChatGPT as alice@example.com"
        );
    }
}
#[test]
fn a_renewed_sign_in_explains_how_to_add_another_account() {
    for (email, display) in [
        (None, b"alice@example.com".as_slice()),
        (Some(b"alice@example.com".as_slice()), b""),
        (Some(b"alice@example.com".as_slice()), b"Different display"),
    ] {
        let mut doc = card_document(b"openai", display, c::AccountKind::SignIn);
        doc.accounts.get_mut(0).unwrap().facts = email_facts(email);
        assert_eq!(sign_in_end_words(doc, c::AccountEnd::Renewed { account: 1, declined: false }), "That’s alice@example.com again. To add another account, sign out of ChatGPT in your browser first, then Continue with ChatGPT.".as_bytes());
    }
}
#[test]
fn a_sign_in_end_keeps_the_number_when_its_account_has_not_arrived() {
    for (end, expected) in [
        (c::AccountEnd::Made { account: 29, declined: false }, b"Signed in; account 29".as_slice()),
        (c::AccountEnd::Renewed { account: 29, declined: false }, b"Signed in to the same account again; account 29"),
    ] {
        assert_eq!(sign_in_end_words(document(0), end), expected);
    }
}
#[test]
fn a_sign_in_end_without_a_display_names_the_plan_unless_renewed() {
    for (end, expected) in [
        (c::AccountEnd::Made { account: 1, declined: false }, b"Signed in to ChatGPT".as_slice()),
        (c::AccountEnd::Renewed { account: 1, declined: false }, b"Signed in to the same account again; account 1"),
    ] {
        assert_eq!(sign_in_end_words(card_document(b"openai", b"", c::AccountKind::SignIn), end), expected);
    }
}
#[test]
fn a_named_sign_in_end_still_reports_declined_plan_usage() {
    assert!(
        sign_in_end_words(
            card_document(b"openai", b"alice@example.com", c::AccountKind::SignIn),
            c::AccountEnd::Made { account: 1, declined: true }
        )
        .ends_with(b"You signed in without plan usage.")
    );
}
fn answer_account_operation(root: &mut d::Domain, operation: Operation, outcome: c::DocumentOutcome) {
    let limits = fixtures::limits();
    let env = skein_lib::Env { now: skein_lib::Time::ZERO, wall: skein_lib::Wall::EPOCH, limits };
    let mut out = skein_lib::Queue::uncapped();
    for act in [
        A::KeyFor { row: Row { holder: c::AskAccountHolder::Own, account: 1 }, source: None },
        A::Submit { name: [98; 16], row: Row { holder: c::AskAccountHolder::Own, account: 1 }, operation },
        A::Confirm,
    ] {
        d::step(root, &env, d::Event::Act { action: d::Action::Pages(d::pages::Act::Accounts(act)) }, &mut out);
    }
    d::step(root, &env, d::Event::Client { event: jig_client::Event::Stored }, &mut out);
    let attempt = out
        .iter()
        .find_map(|request| match request {
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
        .expect("account operation attempt");
    fixtures::root_event(
        root,
        d::Event::Client {
            event: jig_client::Event::Up {
                up: jig_client::Up::Answer {
                    attempt,
                    envelope: skein_lib::RequestEnvelope::Final,
                    answer: Some(jig_client::Answered::Jig(c::Answer::Done(c::Outcome::Document(outcome)))),
                    why: None,
                },
            },
        },
    );
}

fn account_act_page(program: bool) -> (d::Domain, Token) {
    let (mut root, watch) =
        if program { super::connect_model::flow_fixture(d::WayKind::Program) } else { fixtures::account_root() };
    let mut doc = document(0);
    let control = if program {
        "I’ve signed in"
    } else {
        doc = card_document(b"openai", b"alice", c::AccountKind::SignIn);
        doc.accounts.get_mut(0).unwrap().sources.get_mut(0).unwrap().state = c::AccountState::Later { until: None };
        "Resume"
    };
    deliver(&mut root, watch, doc);
    let mut view = fixtures::view();
    let node = fixtures::find(fixtures::build(&mut view, &root), Role::Button, control.as_bytes())
        .expect("offered account control");
    let event = crate::event(&view, node, Act::Press).expect("account control decodes");
    let out = trial_event(&mut root, event);
    let stored = trial_event(&mut root, d::Event::Client { event: jig_client::Event::Stored });
    let attempt = out
        .iter()
        .chain(stored.iter())
        .find_map(|request| {
            if let d::Request::Unlinked {
                request: jig_client::UnlinkedRequest::Down { down: jig_client::Down::Send { attempt, .. } },
            } = request
            {
                Some(*attempt)
            } else {
                None
            }
        })
        .expect("the page sent its account act");
    (root, attempt)
}

#[test]
fn account_act_refusals_say_why_and_what_next() {
    for (program, refusal, expected) in [
        (true, c::Refusal::NameTaken, "The name claude-pat is taken. Choose another name for this login."),
        (true, c::Refusal::NotOffered, "This provider offers you no program login. Choose another way to connect."),
        (
            false,
            c::Refusal::NotOffered,
            "This source isn't limited, so there's nothing to resume. Its state above is current.",
        ),
        (true, c::Refusal::Unknown, "No visible object with that name"),
    ] {
        let (mut root, attempt) = account_act_page(program);
        fixtures::root_event(
            &mut root,
            d::Event::Client {
                event: jig_client::Event::Up {
                    up: jig_client::Up::Answer {
                        attempt,
                        envelope: skein_lib::RequestEnvelope::Final,
                        answer: Some(jig_client::Answered::Jig(c::Answer::Refused { refusal, at: None })),
                        why: None,
                    },
                },
            },
        );
        let view = fixtures::browser_page_view(&root);
        let lines: Vec<_> = view
            .tree()
            .nodes()
            .iter()
            .enumerate()
            .filter(|(_, node)| node.role == Role::Status)
            .map(|(index, _)| fixtures::node_text(view.tree(), index))
            .collect();
        assert!(lines.iter().any(|line| line.as_ref() == expected.as_bytes()), "expected {expected:?}, got {lines:?}");
    }
}
#[test]
fn only_settled_account_statuses_have_the_done_style() {
    use crate::accounts::words::{self, Word as W};
    for (outcome, word, done) in [
        (
            Some((
                Operation::SignOut { source: 1 },
                c::DocumentOutcome::AccountSignedOut { confirmed: true, failed_hosts: Box::new([]) },
            )),
            W::Confirmed,
            true,
        ),
        (
            Some((
                Operation::SignOut { source: 1 },
                c::DocumentOutcome::AccountSignedOut { confirmed: false, failed_hosts: Box::new([]) },
            )),
            W::Unconfirmed,
            false,
        ),
        (
            Some((
                Operation::RemoveKey { source: 1 },
                c::DocumentOutcome::AccountRemoved { confirmed: true, failed_hosts: Box::new([]) },
            )),
            W::Removed,
            true,
        ),
        (
            Some((
                Operation::RemoveKey { source: 1 },
                c::DocumentOutcome::AccountRemoved { confirmed: false, failed_hosts: Box::new([]) },
            )),
            W::RemovedUnconfirmed,
            false,
        ),
    ] {
        let (mut root, watch) = fixtures::account_root();
        let mut doc = document(1);
        doc.settled = outcome.is_none();
        deliver(&mut root, watch, doc);
        if let Some((operation, outcome)) = outcome {
            answer_account_operation(&mut root, operation, outcome);
        }
        let mut view = fixtures::view();
        let tree = fixtures::build(&mut view, &root);
        let name = words::say(&fixtures::words(), word, &[]);
        let status = tree
            .nodes()
            .iter()
            .find(|n| n.role == Role::Status && n.name.as_deref() == Some(name.as_ref()))
            .expect("account status line");
        assert_eq!(
            status.styles.as_slice().contains(&crate::look::STATUS_DONE),
            done,
            "{}",
            String::from_utf8_lossy(&name)
        );
    }
}
#[cfg(target_pointer_width = "64")]
use temper_fixtures::memory_literal;

fn owner_root() -> (d::Domain, Token) {
    let (root, deployment, _) = owner_root_with_accounts();
    (root, deployment)
}
fn owner_root_with_accounts() -> (d::Domain, Token, Token) {
    let limits = fixtures::limits();
    let (mut root, me_watch, _, _, accounts) = fixtures::root_at_full(d::Address::Accounts, &limits, b"Shop");
    let mut me = d::pages::accounts::current_me(root.client()).unwrap().clone();
    me.deployment_role = Some(c::HeldRole::Owner);
    let env = skein_lib::Env { now: skein_lib::Time::ZERO, wall: skein_lib::Wall::EPOCH, limits };
    let mut out = skein_lib::Queue::uncapped();
    d::step(
        &mut root,
        &env,
        d::Event::Client {
            event: jig_client::Event::Up {
                up: jig_client::Up::Watched {
                    watch: me_watch,
                    event: c::WatchEvent::Delivered(c::Delivery::Me(Box::new(me))),
                },
            },
        },
        &mut out,
    );
    let watch =
        out.iter()
            .find_map(|r| {
                if let d::Request::Unlinked {
                    request:
                        jig_client::UnlinkedRequest::Down {
                            down:
                                jig_client::Down::Open {
                                    watch,
                                    subject: jig_client::Watchable::Jig(c::Subject::Deployment),
                                },
                        },
                } = r
                {
                    Some(*watch)
                } else {
                    None
                }
            })
            .unwrap();
    let link = jig_client_protocol_link::Limits::new(limits.client.domain, limits.client.exchanges).unwrap();
    let deployment = jig_client_protocol_link::translate::documents::deployment(
        jig_client_test_records::deployment_rows_record(),
        &link,
    )
    .unwrap();
    deliver_deployment(&mut root, watch, deployment);
    (root, watch, accounts.expect("own accounts watch"))
}
#[test]
fn an_owner_sees_deployment_accounts_after_their_own() {
    let (root, _) = owner_root();
    let mut view = fixtures::view();
    let text = fixtures::text(fixtures::build(&mut view, &root));
    assert!(
        skein_lib::bytes::find(&text, b"Your accounts").unwrap()
            < skein_lib::bytes::find(&text, b"Deployment accounts").unwrap()
    );
    assert!(
        view.tree()
            .nodes()
            .iter()
            .any(|n| n.role == Role::Disclosure && n.name.as_ref().is_some_and(|name| name.starts_with(b"Account 41")))
    );
}
#[test]
fn the_rows_offer_no_number_fields() {
    let (root, mut view) = key_draft();
    let tree = fixtures::build(&mut view, &root);
    for name in [b"Provider number".as_slice(), b"Account number", b"Source number"] {
        assert!(!tree.nodes().iter().any(|n| n.name.as_deref() == Some(name)));
    }
}
#[test]
fn cancel_refreshes_the_rows_secret_field() {
    let (mut root, mut view) = key_draft();
    let before = root.pages().accounts().unwrap().request.key_written();
    // Cancel also belongs to an editor before its first request.
    action(&mut root, A::Cancel);
    fixtures::build(&mut view, &root);
    assert!(key_field(&view).value.as_ref().unwrap().written > before);
}

#[test]
fn expiring_has_a_separator() {
    let mut doc = document(1);
    doc.accounts.get_mut(0).unwrap().sources.get_mut(0).unwrap().expiring = true;
    assert!(shown(doc).contains("Ready · Expiring"));
}
#[test]
fn attention_has_a_separator() {
    let mut doc = document(1);
    doc.accounts.get_mut(0).unwrap().sources.get_mut(0).unwrap().attention = true;
    assert!(shown(doc).contains("Ready · Needs attention"));
}
#[test]
fn spent_has_its_own_time_word() {
    let mut doc = document(1);
    doc.accounts.get_mut(0).unwrap().sources.get_mut(0).unwrap().spent_until =
        Some(skein_lib::Wall::from_nanos(1_791_828_240_000_000_000));
    assert!(shown(doc).contains("Ready · Usage limit reached · Usage limit until Oct 12, 18:04 UTC"));
}
fn confirmation_words(operation: Operation) -> String {
    let (mut root, watch) = fixtures::account_root();
    deliver(&mut root, watch, document(1));
    action(&mut root, A::KeyFor { row: Row { holder: c::AskAccountHolder::Own, account: 1 }, source: None });
    action(
        &mut root,
        A::Submit { name: [72; 16], row: Row { holder: c::AskAccountHolder::Own, account: 1 }, operation },
    );
    let mut view = fixtures::view();
    String::from_utf8(fixtures::text(fixtures::build(&mut view, &root)).into_vec()).unwrap()
}
#[test]
fn sign_out_confirmation_names_its_source() {
    assert!(
        confirmation_words(Operation::SignOut { source: 1 })
            .contains("Sign this source out? · Your own accounts · Account 1 · Source 1")
    );
}
#[test]
fn key_removal_confirmation_names_its_source() {
    assert!(
        confirmation_words(Operation::RemoveKey { source: 1 })
            .contains("Remove this key? · Your own accounts · Account 1 · Source 1")
    );
}
#[test]
fn account_removal_confirmation_names_its_holder() {
    let (mut root, _) = owner_root();
    action(&mut root, A::KeyFor { row: Row { holder: c::AskAccountHolder::Deployment, account: 41 }, source: None });
    action(
        &mut root,
        A::Submit {
            name: [72; 16],
            row: Row { holder: c::AskAccountHolder::Deployment, account: 41 },
            operation: Operation::RemoveAccount,
        },
    );
    let mut view = fixtures::view();
    let text = String::from_utf8(fixtures::text(fixtures::build(&mut view, &root)).into_vec()).unwrap();
    assert!(text.contains("Remove this account and all its sources? · The deployment's accounts · Account 41"));
}
#[test]
fn a_deployment_row_offers_no_sign_in() {
    let (root, _) = owner_root();
    let mut view = fixtures::view();
    let tree = fixtures::build(&mut view, &root);
    assert!(!tree.nodes().iter().any(|n| n.name.as_deref() == Some(b"Use deployment accounts")));
    assert!(!tree.nodes().iter().any(|n| n.name.as_deref() == Some(b"Continue with ChatGPT")));
}
#[test]
fn an_account_without_a_provider_has_one_separator() {
    let mut doc = document(1);
    doc.accounts.get_mut(0).unwrap().provider = None;
    doc.accounts.get_mut(0).unwrap().provider_name = None;
    assert!(shown(doc).contains("Account 1 · Pat"));
}
#[test]
fn empty_uses_have_no_label() {
    let mut doc = document(1);
    doc.accounts.get_mut(0).unwrap().uses = Some(List::with_capacity(0));
    assert!(!shown(doc).contains("Uses:"));
}
#[test]
fn an_empty_deployment_leaves_out_its_section() {
    let (mut root, watch) = owner_root();
    let mut doc = root.pages().accounts().unwrap().deployment(root.client()).unwrap().clone();
    doc.accounts = List::with_capacity(0);
    deliver_deployment(&mut root, watch, doc);
    let mut view = fixtures::view();
    let text = String::from_utf8(fixtures::text(fixtures::build(&mut view, &root)).into_vec()).unwrap();
    assert!(!text.contains("Deployment accounts"));
}

#[test]
fn sign_out_keeps_its_element_when_deployment_accounts_arrive() {
    let (mut root, watch) = owner_root();
    let deployment = root.pages().accounts().unwrap().deployment(root.client()).unwrap().clone();
    let mut empty = deployment.clone();
    empty.accounts.clear();
    deliver_deployment(&mut root, watch, empty.clone());
    let mut view = fixtures::browser_page_view(&root);
    let sign_out = fixtures::find(view.tree(), Role::Button, b"Sign out").unwrap();
    for document in [deployment, empty] {
        deliver_deployment(&mut root, watch, document);
        fixtures::build(&mut view, &root);
        assert_eq!(fixtures::find(view.tree(), Role::Button, b"Sign out"), Some(sign_out));
        assert_eq!(crate::act(&mut view, sign_out, Act::Press), Some(d::Action::Pages(d::pages::Act::SignOut)));
    }
}
#[test]
fn the_menus_sign_out_keeps_its_element_when_deployment_accounts_arrive() {
    let (mut root, watch) = owner_root();
    let deployment = root.pages().accounts().unwrap().deployment(root.client()).unwrap().clone();
    let mut empty = deployment.clone();
    empty.accounts.clear();
    deliver_deployment(&mut root, watch, empty.clone());
    let mut view = fixtures::browser_page_view(&root);
    let sign_out = fixtures::find(view.tree(), Role::MenuItem, b"Sign out").unwrap();
    for document in [deployment, empty] {
        deliver_deployment(&mut root, watch, document);
        fixtures::build(&mut view, &root);
        assert_eq!(fixtures::find(view.tree(), Role::MenuItem, b"Sign out"), Some(sign_out));
        assert_eq!(crate::act(&mut view, sign_out, Act::Press), Some(d::Action::Pages(d::pages::Act::SignOut)));
    }
}
#[test]
fn accounts_past_the_count_are_named() {
    let mut counts = fixtures::limits().client.domain.accounts;
    counts.accounts = 2;
    assert!(shown_at(document(3), counts).contains("Accounts left out: 1"));
}
#[test]
fn sources_past_the_count_are_named() {
    let mut counts = fixtures::limits().client.domain.accounts;
    counts.sources = 2;
    assert!(shown_at(document(3), counts).contains("Sources left out: 1"));
}
#[test]
fn sign_ins_past_the_count_are_named() {
    let mut counts = fixtures::limits().client.domain.accounts;
    counts.sign_ins = 2;
    assert!(shown_at(document(3), counts).contains("Sign-ins left out: 1"));
}
#[test]
fn each_source_row_offers_its_rank_control() {
    let mut doc = document(16);
    for at in 0..doc.accounts.len() {
        let row = doc.accounts.get_mut(at).unwrap();
        row.number = 1;
        row.place = 2;
        for at in 0..row.sources.len() {
            let source = row.sources.get_mut(at).unwrap();
            source.number = 1;
            source.place = 2;
        }
    }
    let (mut root, watch) = fixtures::account_root();
    deliver(&mut root, watch, doc);
    action(&mut root, A::KeyFor { row: Row { holder: c::AskAccountHolder::Own, account: 1 }, source: None });
    let mut view = bounded_view(fixtures::limits().client.domain.accounts);
    let tree = fixtures::build(&mut view, &root);
    assert_eq!(tree.nodes().iter().filter(|n| n.name.as_deref() == Some(b"Move source up")).count(), 16 * 8);
}
#[test]
fn bindings_at_capacity_refuse_another_control() {
    let counts = fixtures::limits().client.domain.accounts;
    let mut bindings =
        crate::accounts::Bindings::new(counts, fixtures::limits().row_uses, fixtures::limits().client.domain.rows);
    for _ in 0..crate::accounts::binding_capacity(&counts, fixtures::limits().client.domain.rows) {
        assert!(bindings.insert(crate::accounts::Meaning::Key).is_some());
    }
    assert!(bindings.insert(crate::accounts::Meaning::Key).is_none());
}

#[test]
fn deployment_source_confirmation_names_its_account() {
    let (mut root, watch) = owner_root();
    let mut doc = root.pages().accounts().unwrap().deployment(root.client()).unwrap().clone();
    let mut sources = List::with_capacity(1);
    sources.push(source(43)).unwrap();
    doc.accounts.get_mut(0).unwrap().sources = sources;
    deliver_deployment(&mut root, watch, doc);
    action(
        &mut root,
        A::KeyFor { row: Row { holder: c::AskAccountHolder::Deployment, account: 41 }, source: Some(43) },
    );
    action(
        &mut root,
        A::Submit {
            name: [72; 16],
            row: Row { holder: c::AskAccountHolder::Deployment, account: 41 },
            operation: Operation::RemoveKey { source: 43 },
        },
    );
    let mut view = fixtures::view();
    let text = String::from_utf8(fixtures::text(fixtures::build(&mut view, &root)).into_vec()).unwrap();
    assert!(text.contains("Remove this key? · The deployment's accounts · Account 41 · Source 43"));
}
#[test]
fn deployment_accounts_past_the_count_are_named() {
    let (mut root, watch) = owner_root();
    let mut doc = root.pages().accounts().unwrap().deployment(root.client()).unwrap().clone();
    let row = doc.accounts.get(0).unwrap().clone();
    let mut rows = List::with_capacity(3);
    for _ in 0_u32..3 {
        rows.push(row.clone()).unwrap();
    }
    doc.accounts = rows;
    deliver_deployment(&mut root, watch, doc);
    let mut counts = fixtures::limits().client.domain.accounts;
    counts.accounts = 2;
    let mut view = bounded_view(counts);
    let tree = fixtures::build(&mut view, &root);
    let text = String::from_utf8(fixtures::text(tree).into_vec()).unwrap();
    assert_eq!(
        tree.nodes()
            .iter()
            .filter(|node| node.role == Role::Disclosure
                && node.name.as_ref().is_some_and(|name| name.starts_with(b"Account 41")))
            .count(),
        2
    );
    assert!(text.contains("Accounts left out: 1"));
}

#[test]
fn the_link_word_counts_towards_the_address_bound() {
    let address = format!("https://i/{}", "x".repeat(490));
    assert!(address.len() < usize::try_from(fixtures::tree_limits().text_per_node).unwrap());
    let root = sign_in_address(address.as_bytes());
    let mut view = fixtures::view();
    let tree = fixtures::build(&mut view, &root);
    assert!(!tree.nodes().iter().any(|n| n.external.is_some()));
}

#[test]
fn typing_a_key_keeps_the_write_revision() {
    let (mut root, mut view) = key_draft();
    let before = key_field(&view).value.as_ref().unwrap().written;
    press(
        &mut view,
        &mut root,
        Role::TextField,
        b"Key (pasted once)",
        Act::Input { value: Box::from(b"another-key".as_slice()) },
    );
    fixtures::build(&mut view, &root);
    assert_eq!(key_field(&view).value.as_ref().unwrap().written, before);
}

#[test]
fn accounts_show_the_configured_workers_program_observation() {
    let mut doc = card_document(b"anthropic", b"work-login", c::AccountKind::Program);
    let mut programs = List::with_capacity(1);
    programs
        .push(c::ProgramLogin {
            name: b"work-login".as_slice().into(),
            state: Some(c::ProgramState::SignedInOtherwise { method: b"other-method".as_slice().into() }),
            sign_out: None,
            cleanup: false,
            cut: false,
        })
        .unwrap();
    doc.program_hosts = List::with_capacity(1);
    doc.program_hosts
        .push(c::ProgramHost {
            name: b"configured-worker".as_slice().into(),
            channel: 17,
            connected: true,
            awaiting_logins: false,
            programs,
            cut: false,
        })
        .unwrap();
    let text = shown(doc);
    assert!(
        text.contains("configured-worker") && text.contains("work-login") && text.contains("other-method"),
        "{text}"
    );
}

#[test]
fn accounts_are_folded_disclosures_with_their_state_and_line() {
    let (mut root, watch) = fixtures::account_root();
    deliver(&mut root, watch, card_document(b"openai", b"alice", c::AccountKind::SignIn));
    let mut view = fixtures::view();
    let tree = fixtures::build(&mut view, &root);
    let row = tree.nodes().iter().find(|node| node.styles.as_slice().contains(&crate::look::SETTINGS_ROW)).unwrap();
    assert!(!row.states.as_slice().contains(&skein_ui::State::Expanded));
    let summary = core::str::from_utf8(row.description.as_ref().unwrap()).unwrap();
    assert!(summary.contains("Ready"));
    assert!(summary.contains("Your own work, and projects where you’re the only member"));
    assert!(!summary.contains("no task uses it now"));
    // Opening is native shell state; every row keeps its content in the semantic tree.
    let at = tree.nodes().iter().position(|n| n.id == row.id).unwrap();
    let content = &tree.nodes()[at..at + usize::try_from(row.size).unwrap()];
    for name in [b"Add an API key".as_slice(), b"Sign out", b"Remove account"] {
        assert!(content.iter().any(|n| n.name.as_deref() == Some(name)), "{name:?}");
    }
}
#[test]
fn a_limited_plan_offers_resume_and_moves_the_first_later_key() {
    let (mut root, watch) = fixtures::account_root();
    let mut doc = card_document(b"openai", b"alice", c::AccountKind::SignIn);
    let row = doc.accounts.get_mut(0).unwrap();
    row.sources.get_mut(0).unwrap().state = c::AccountState::Later { until: None };
    let sign_in = row.sources.get(0).unwrap().clone();
    row.sources = List::with_capacity(3);
    row.sources.push(sign_in).unwrap();
    row.sources.push(source(2)).unwrap();
    row.sources.push(source(3)).unwrap();
    deliver(&mut root, watch, doc);
    let mut view = fixtures::view();
    let text = String::from_utf8(fixtures::text(fixtures::build(&mut view, &root)).into_vec()).unwrap();
    assert!(text.contains("temper reached its usage limit on your ChatGPT plan. Tasks on it wait."));
    let node = fixtures::find(view.tree(), Role::Button, b"Use the API key first").unwrap();
    let Some(d::Event::Act { action }) = crate::event(&view, node, Act::Press) else { panic!("row act") };
    assert_eq!(
        action,
        d::Action::Pages(d::pages::Act::Accounts(A::Submit {
            name: root.pages().account_name(),
            row: Row { holder: c::AskAccountHolder::Own, account: 1 },
            operation: Operation::MoveSource { source: 2, place: 1 }
        }))
    );
}
#[test]
fn declined_plan_usage_belongs_to_the_ending_named_account() {
    let (mut root, watch) = fixtures::account_root();
    let mut doc = document(2);
    for index in 0..doc.accounts.len() {
        let row = doc.accounts.get_mut(index).unwrap();
        row.provider_name = Some(text(b"openai"));
        row.sources.get_mut(0).unwrap().kind = c::AccountKind::SignIn;
    }
    doc.last_end = Some(c::AccountLastEnd { source: 1, end: c::AccountEnd::Made { account: 2, declined: true } });
    deliver(&mut root, watch, doc);
    let mut view = fixtures::view();
    let tree = fixtures::build(&mut view, &root);
    let row = tree
        .nodes()
        .iter()
        .position(|n| n.role == Role::Disclosure && n.name.as_ref().unwrap().starts_with(b"ChatGPT"))
        .unwrap();
    let text = String::from_utf8(fixtures::text(tree).into_vec()).unwrap();
    assert!(text.contains("You signed in without plan usage."));
    assert!(tree.nodes().iter().skip(row).any(|n| n.name.as_deref() == Some(b"Use an API key")));
}
#[test]
fn none_of_your_own_offers_the_connect_model_link() {
    let (mut root, watch) = fixtures::account_root();
    deliver(&mut root, watch, document(0));
    let mut view = fixtures::view();
    let text = String::from_utf8(fixtures::text(fixtures::build(&mut view, &root)).into_vec()).unwrap();
    assert!(text.contains("No model of yours yet."));
    assert!(fixtures::find(view.tree(), Role::Link, b"Connect a model").is_some());
}
#[test]
#[expect(clippy::too_many_lines, reason = "one table covers account states, expiring renewal and their bound controls")]
fn each_account_state_draws_its_words_and_state_controls() {
    const EXPIRING: &str = "Expiring: sign in again before it ends.";
    for (kind, expiring, state, word, act) in [
        (c::AccountKind::SignIn, false, c::AccountState::Granted, "Ready", None),
        (
            c::AccountKind::SignIn,
            false,
            c::AccountState::Later { until: None },
            "temper reached its usage limit on your ChatGPT plan. Tasks on it wait.",
            Some(b"Resume".as_slice()),
        ),
        (
            c::AccountKind::SignIn,
            false,
            c::AccountState::Later { until: Some(skein_lib::Wall::EPOCH) },
            "Limited by OpenAI until",
            Some(b"Resume".as_slice()),
        ),
        (
            c::AccountKind::SignIn,
            false,
            c::AccountState::SignInAgain,
            "Signed out of ChatGPT.",
            Some(b"Continue with ChatGPT".as_slice()),
        ),
        (
            c::AccountKind::SignIn,
            false,
            c::AccountState::Misconfigured,
            "OpenAI refused temper’s sign-in. Check the configuration’s registration.",
            None,
        ),
        (
            c::AccountKind::SignIn,
            false,
            c::AccountState::Unloaded { at: c::AccountPlace::File, why: c::AccountUnread::Unreadable },
            "Cannot load File: Unreadable",
            None,
        ),
        (c::AccountKind::SignIn, true, c::AccountState::Granted, EXPIRING, Some(b"Continue with ChatGPT".as_slice())),
        (c::AccountKind::Program, true, c::AccountState::Granted, EXPIRING, Some(b"Check again".as_slice())),
        (
            c::AccountKind::SignIn,
            true,
            c::AccountState::SignInAgain,
            "Signed out of ChatGPT.",
            Some(b"Continue with ChatGPT".as_slice()),
        ),
    ] {
        let (mut root, watch) = fixtures::account_root();
        let mut doc = card_document(b"openai", b"claude-alice", kind);
        let source = doc.accounts.get_mut(0).unwrap().sources.get_mut(0).unwrap();
        source.state = state;
        source.expiring = expiring;
        deliver(&mut root, watch, doc);
        let mut view = fixtures::view();
        let tree = fixtures::build(&mut view, &root);
        let shown = String::from_utf8(fixtures::text(tree).into_vec()).unwrap();
        assert!(shown.contains(word), "{state:?}: {shown}");
        assert_eq!(
            tree.nodes()
                .iter()
                .filter(|n| n.role == Role::Paragraph && n.name.as_deref() == Some(EXPIRING.as_bytes()))
                .count(),
            usize::from(expiring && state != c::AccountState::SignInAgain),
            "{kind:?}, {expiring}, {state:?}: {shown}"
        );
        if kind == c::AccountKind::Program {
            let command = b"On each host, with its directories: temper program-login --config CONFIG --data DATA --cache CACHE claude-alice";
            assert_eq!(
                tree.nodes().iter().filter(|n| n.role == Role::Code && n.name.as_deref() == Some(command)).count(),
                1
            );
        }
        if let Some(act) = act {
            assert_eq!(
                tree.nodes().iter().filter(|n| n.role == Role::Button && n.name.as_deref() == Some(act)).count(),
                1,
                "{state:?}"
            );
            let node = fixtures::find(tree, Role::Button, act).unwrap();
            let operation = match kind {
                c::AccountKind::Program => Some(Operation::CheckProgram { source: 1 }),
                c::AccountKind::SignIn if act == b"Continue with ChatGPT" => {
                    Some(Operation::SignIn { provider: PROVIDER })
                }
                c::AccountKind::SignIn | c::AccountKind::Key | c::AccountKind::None => None,
            };
            if let Some(operation) = operation {
                let Some(d::Event::Act { action }) = crate::event(&view, node, Act::Press) else { panic!("row act") };
                assert_eq!(
                    action,
                    d::Action::Pages(d::pages::Act::Accounts(A::Submit {
                        name: root.pages().account_name(),
                        row: Row { holder: c::AskAccountHolder::Own, account: 1 },
                        operation,
                    }))
                );
            }
        } else {
            for act in [b"Continue with ChatGPT".as_slice(), b"Check again"] {
                assert!(fixtures::find(tree, Role::Button, act).is_none(), "{state:?}");
            }
        }
    }
}
#[test]
fn a_moving_account_shows_its_state_without_buttons() {
    let (mut root, watch) = fixtures::account_root();
    let mut doc = card_document(b"openai", b"alice", c::AccountKind::SignIn);
    deliver(&mut root, watch, doc.clone());
    // An editor opened before the move must not keep its buttons either.
    action(&mut root, A::KeyFor { row: Row { holder: c::AskAccountHolder::Own, account: 1 }, source: None });
    let row = doc.accounts.get_mut(0).unwrap();
    row.place = 2;
    let source = row.sources.get_mut(0).unwrap();
    source.state = c::AccountState::Moving;
    source.usable = false;
    source.place = 2;
    deliver(&mut root, watch, doc);
    let mut view = fixtures::view();
    let tree = fixtures::build(&mut view, &root);
    let row = tree.nodes().iter().find(|node| node.styles.as_slice().contains(&crate::look::SETTINGS_ROW)).unwrap();
    assert!(core::str::from_utf8(row.description.as_ref().unwrap()).unwrap().contains("Moving to another deployment"));
    let at = tree.nodes().iter().position(|node| node.id == row.id).unwrap();
    let content = super::inbox::subtree(tree, at);
    assert!(content.iter().all(|node| node.role != Role::Button));
    assert!(skein_lib::bytes::find(&fixtures::text(tree), b"Moving to another deployment").is_some());
}
#[test]
fn a_programs_missing_login_names_its_host_command_and_check() {
    let local =
        b"'/box/temper' program-login --config '/box/config' --data '/box/data' --cache '/box/cache'".as_slice();
    for (line, expiring) in [(None, false), (Some(local), false), (None, true), (Some(local), true)] {
        let (mut root, watch) = super::connect_model::program_root_with_line(d::Address::Accounts, line);
        let mut doc = card_document(b"anthropic", b"claude-alice", c::AccountKind::Program);
        let source = doc.accounts.get_mut(0).unwrap().sources.get_mut(0).unwrap();
        source.usable = false;
        source.expiring = expiring;
        let mut programs = List::with_capacity(1);
        programs
            .push(c::ProgramLogin {
                name: b"claude-alice".as_slice().into(),
                state: Some(c::ProgramState::NotSignedIn),
                sign_out: None,
                cleanup: false,
                cut: false,
            })
            .unwrap();
        doc.program_hosts = List::with_capacity(1);
        doc.program_hosts
            .push(c::ProgramHost {
                name: b"worker".as_slice().into(),
                channel: 1,
                connected: true,
                awaiting_logins: false,
                programs,
                cut: false,
            })
            .unwrap();
        deliver(&mut root, watch, doc);
        let mut view = fixtures::view();
        let tree = fixtures::build(&mut view, &root);
        let shown = String::from_utf8(fixtures::text(tree).into_vec()).unwrap();
        assert!(shown.contains("Claude Code isn't signed in for 'claude-alice' on worker."));
        let expected = match line {
        Some(line) => format!("On this machine, run: {} claude-alice", core::str::from_utf8(line).unwrap()),
        None => "On each host, with its directories: temper program-login --config CONFIG --data DATA --cache CACHE claude-alice".into(),
    };
        assert!(shown.contains(&expected), "{shown}");
        for (role, name) in [(Role::Code, expected.as_bytes()), (Role::Button, b"Check again".as_slice())] {
            assert_eq!(tree.nodes().iter().filter(|n| n.role == role && n.name.as_deref() == Some(name)).count(), 1);
        }
        assert!(fixtures::find(tree, Role::Button, b"Sign out on every machine").is_some());
    }
}

#[test]
fn an_outcome_is_in_the_disclosure_that_sent_the_act() {
    let (mut root, watch) = fixtures::account_root();
    deliver(&mut root, watch, document(2));
    answer_account_operation(
        &mut root,
        Operation::SignOut { source: 1 },
        c::DocumentOutcome::AccountSignedOut { confirmed: true, failed_hosts: Box::new([]) },
    );
    let mut view = fixtures::view();
    let tree = fixtures::build(&mut view, &root);
    let rows: Vec<_> = tree
        .nodes()
        .iter()
        .enumerate()
        .filter(|(_, n)| n.styles.as_slice().contains(&crate::look::SETTINGS_ROW))
        .collect();
    for (number, (at, row)) in rows.into_iter().enumerate() {
        let content = &tree.nodes()[at..at + usize::try_from(row.size).unwrap()];
        assert_eq!(
            content.iter().any(|n| n.name.as_deref() == Some(b"Signed out; revocation confirmed.")),
            number == 0
        );
    }
}

fn deliver_deployment(root: &mut d::Domain, watch: Token, doc: c::Deployment) {
    fixtures::root_event(
        root,
        d::Event::Client {
            event: jig_client::Event::Up {
                up: jig_client::Up::Watched {
                    watch,
                    event: c::WatchEvent::Delivered(c::Delivery::Deployment(Box::new(doc))),
                },
            },
        },
    );
}
#[test]
fn deployment_account_facts_exclude_waiting_tasks_from_its_users() {
    const RUNNING: u64 = 4;
    const WAITING: u64 = 9;
    let (mut root, watch) = owner_root();
    let mut doc = root.pages().accounts().unwrap().deployment(root.client()).unwrap().clone();
    let row = doc.accounts.get_mut(0).unwrap();
    let mut chip = fixtures::task::document().chip;
    row.runs = List::with_capacity(2);
    for task in [RUNNING, WAITING] {
        chip.task = task;
        row.runs.push(chip.clone()).unwrap();
    }
    row.waiting = List::with_capacity(1);
    row.waiting.push(chip).unwrap();
    deliver_deployment(&mut root, watch, doc);
    let mut view = fixtures::view();
    let tree = fixtures::build(&mut view, &root);
    for (name, value) in [(b"Used by".as_slice(), b"T4".as_slice()), (b"Waiting for it", b"T9")] {
        let at = tree.nodes().iter().position(|n| n.role == Role::Region && n.name.as_deref() == Some(name)).unwrap();
        let row = &tree.nodes()[at..at + usize::try_from(tree.nodes()[at].size).unwrap()];
        assert!(row.iter().any(|n| n.role == Role::Description && n.name.as_deref() == Some(value)), "{name:?}");
    }
}

#[test]
fn only_deployment_rows_show_usage_facts_and_their_use_summary() {
    let (mut root, deployment, accounts) = owner_root_with_accounts();
    let mut doc = root.pages().accounts().unwrap().deployment(root.client()).unwrap().clone();
    let row = doc.accounts.get_mut(0).unwrap();
    row.runs = List::with_capacity(0);
    row.waiting = List::with_capacity(0);
    deliver_deployment(&mut root, deployment, doc);
    deliver(&mut root, accounts, card_document(b"openai", b"alice", c::AccountKind::SignIn));
    let view = fixtures::page_view(&root);
    let mut rows: u32 = 0;
    for row in view.tree().nodes().iter().filter(|n| n.styles.as_slice().contains(&crate::look::SETTINGS_ROW)) {
        let at = view.tree().nodes().iter().position(|n| n.id == row.id).unwrap();
        let content = &view.tree().nodes()[at..at + usize::try_from(row.size).unwrap()];
        let deployment = rows == 1;
        for name in [b"Used by".as_slice(), b"Waiting for it"] {
            assert_eq!(content.iter().any(|n| n.name.as_deref() == Some(name)), deployment);
        }
        let summary = core::str::from_utf8(row.description.as_deref().unwrap()).unwrap();
        assert_eq!(summary.contains("no task uses it now"), deployment);
        rows += 1;
    }
    assert_eq!(rows, 2);
}

#[test]
#[expect(
    clippy::disallowed_methods,
    reason = "cli C4: SKEIN_CODEGEN_WRITE selects regeneration of the test's checked document or golden; the test table will pass it"
)]
fn english_phrase_declarations_match_the_vocabulary() {
    let path = skein_scratch::paths::package().join("src/words.rs");
    let source = std::fs::read_to_string(&path).unwrap();
    let variants = source.split("pub enum Word {\n").nth(1).unwrap().split("\n}").next().unwrap();
    let names: Vec<_> = variants.lines().map(str::trim).filter(|line| line.ends_with(',')).collect();
    let mut all = format!("const ALL: [Word; {}] = [\n", names.len());
    for name in names {
        all.push_str("    Word::");
        all.push_str(name);
        all.push('\n');
    }
    all.push_str("];");
    let start = source.find("const ALL: [Word;").unwrap();
    let end = start + source[start..].find("\n];").unwrap() + 3;
    if std::env::var_os("SKEIN_CODEGEN_WRITE").is_some_and(|value| value == "1") {
        let updated = format!("{}{}{}", &source[..start], all, &source[end..]);
        std::fs::write(&path, updated).unwrap();
    } else {
        assert_eq!(&source[start..end], all);
    }
    for (marker, expected, actual) in crate::words::english_declarations() {
        memory_literal::check(&path, marker, expected, actual);
    }
}

#[test]
fn an_accounts_rows_sign_in_offers_the_paste() {
    let (mut root, watch) = super::connect_model::flow_fixture(d::WayKind::SignIn);
    fixtures::root_event(&mut root, d::Event::Went { place: d::Place::Address(d::Address::Accounts) });
    deliver(&mut root, watch, card_document(b"openai", b"fixture", c::AccountKind::SignIn));
    answer_account_operation(
        &mut root,
        Operation::SignIn { provider: PROVIDER },
        c::DocumentOutcome::AccountSignInStarted {
            source: SOURCE,
            address: Box::from(b"https://issuer.example/visit".as_slice()),
        },
    );
    let mut view = fixtures::view();
    let tree = fixtures::build(&mut view, &root);
    let field = fixtures::find(tree, Role::TextField, b"Address of the page that couldn't be reached").unwrap();
    let field_at = tree.nodes().iter().position(|n| n.id == field).unwrap();
    let disclosure = tree
        .nodes()
        .iter()
        .position(|n| {
            n.role == Role::Disclosure
                && n.name.as_deref() == Some("Signed in, but the page couldn’t be reached?".as_bytes())
        })
        .unwrap();
    assert!(field_at > disclosure && field_at < disclosure + usize::try_from(tree.nodes()[disclosure].size).unwrap());
    let input = crate::event(&view, field, Act::Input { value: Box::from(b"fixture".as_slice()) }).unwrap();
    let d::Event::SignInAddress { value } = &input else { panic!("secret event") };
    assert!(value.expose() == b"fixture");
    fixtures::root_event(&mut root, input);
    let finish = fixtures::find(fixtures::build(&mut view, &root), Role::Button, b"Finish connecting").unwrap();
    assert_eq!(
        crate::act(&mut view, finish, Act::Press),
        Some(d::Action::Pages(d::pages::Act::Accounts(A::Finish { name: root.pages().account_name() })))
    );
}

fn account_order(doc: c::Accounts) -> Vec<u32> {
    let (mut root, watch) = fixtures::account_root();
    deliver(&mut root, watch, doc);
    let mut view = bounded_view(fixtures::limits().client.domain.accounts);
    fixtures::build(&mut view, &root);
    let ids: Vec<_> = view
        .tree()
        .nodes()
        .iter()
        .filter(|node| node.role == Role::Button && node.name.as_deref() == Some(b"Remove account"))
        .map(|node| node.id)
        .collect();
    let mut numbers = Vec::new();
    for id in ids {
        if let Some(d::Action::Pages(d::pages::Act::Accounts(A::Submit { row, .. }))) =
            crate::act(&mut view, id, Act::Press)
        {
            numbers.push(row.account);
        }
    }
    numbers
}
#[test]
fn an_accounts_row_keeps_its_header_sources_facts_and_acts_in_order() {
    let (mut root, me_watch, _, _, accounts_watch) =
        fixtures::root_at_full(d::Address::Accounts, &fixtures::limits(), b"Shop");
    let mut me = d::pages::accounts::current_me(root.client()).unwrap().clone();
    me.projects.get_mut(0).unwrap().models = super::choice::line_models(true);
    fixtures::root_event(
        &mut root,
        d::Event::Client {
            event: jig_client::Event::Up {
                up: jig_client::Up::Watched {
                    watch: me_watch,
                    event: c::WatchEvent::Delivered(c::Delivery::Me(Box::new(me))),
                },
            },
        },
    );
    deliver(&mut root, accounts_watch.unwrap(), card_document(b"openai", b"Pat", c::AccountKind::SignIn));
    let view = fixtures::page_view(&root);
    let tree = view.tree();
    assert!(fixtures::find(tree, Role::Paragraph, "The LLM accounts temper’s agents run on for you. A task keeps its account: while the account is limited, the task waits for it.".as_bytes()).is_some());
    let row = super::inbox::styled(tree, crate::look::SETTINGS_ROW);
    let nodes = super::inbox::subtree(tree, row);
    assert!(nodes[0].states.is_empty());
    let mut previous = 0;
    for (role, name) in [
        (Role::Paragraph, b"Its sources in order: a run takes the first usable one.".as_slice()),
        (Role::Status, b"Ready".as_slice()),
        (Role::Button, b"Remove account".as_slice()),
    ] {
        let at = nodes.iter().position(|node| node.role == role && node.name.as_deref() == Some(name)).unwrap();
        assert!(at > previous, "{name:?}");
        previous = at;
    }
    assert!(
        !nodes
            .iter()
            .any(|node| node.name.as_deref() == Some(b"Used by") || node.name.as_deref() == Some(b"Waiting for it"))
    );
}

#[test]
fn accounts_lists_the_scripted_model_with_nothing_to_add() {
    let root = fixtures::playground_root(d::Address::Accounts, fixtures::PROJECT, true);
    for words in [fixtures::words(), fixtures::browser_words()] {
        let mut view = fixtures::view_of(words);
        let tree = fixtures::build(&mut view, &root);
        let rows: Vec<_> =
            tree.nodes().iter().filter(|node| node.styles.as_slice().contains(&crate::look::SETTINGS_ROW)).collect();
        assert_eq!(rows.len(), 1);
        assert_eq!(
            rows[0].name.as_deref(),
            Some("Scripted model · the playground’s · held by the deployment".as_bytes())
        );
        for name in [b"Connect a model".as_slice(), b"+ Connect a model", b"Add an API key"] {
            assert!(fixtures::find(tree, Role::Link, name).is_none(), "{name:?}");
            assert!(fixtures::find(tree, Role::Button, name).is_none(), "{name:?}");
        }
    }
}

#[test]
fn a_real_account_added_before_is_ready_in_the_playground() {
    for playground in [false, true] {
        let (mut root, _, _, _, watch) = fixtures::root_at_with_playground(
            d::Address::Accounts,
            &fixtures::domain_limits(),
            b"Shop",
            playground.then_some((fixtures::PROJECT, true)),
        );
        let mut doc = card_document(b"openai", b"Pat", c::AccountKind::SignIn);
        doc.accounts.get_mut(0).unwrap().provider = Some(PROVIDER + 1);
        deliver(&mut root, watch.unwrap(), doc);
        let view = fixtures::page_view(&root);
        let tree = view.tree();
        let row = tree
            .nodes()
            .iter()
            .find(|node| node.name.as_deref() == Some("ChatGPT · Pat · OpenAI · held by Pat".as_bytes()))
            .unwrap();
        assert!(core::str::from_utf8(row.description.as_deref().unwrap()).unwrap().contains("Ready"));
        assert_eq!(
            tree.nodes().iter().filter(|node| node.styles.as_slice().contains(&crate::look::SETTINGS_ROW)).count(),
            if playground { 2 } else { 1 }
        );
        assert_eq!(
            crate::pages::accounts::first_usable_short(&root, &fixtures::words()).unwrap().as_ref(),
            "ChatGPT plan · Pat".as_bytes()
        );
    }
}

#[test]
fn no_account_has_one_primary_way_and_unread_facts_claim_nothing() {
    let (mut root, watch) = fixtures::account_root();
    for view in [fixtures::page_view(&root), fixtures::browser_page_view(&root)] {
        let tree = view.tree();
        assert!(fixtures::find(tree, Role::Heading { level: 2 }, b"Your accounts").is_some());
        assert!(!tree.nodes().iter().any(|n| n.styles.as_slice().contains(&crate::look::SETTINGS_BAND)));
        for name in [b"+ Connect a model".as_slice(), b"Connect a model"] {
            assert!(fixtures::find(tree, Role::Link, name).is_none());
        }
    }
    deliver(&mut root, watch, document(0));
    for mut view in [fixtures::page_view(&root), fixtures::browser_page_view(&root)] {
        let tree = view.tree();
        assert!(fixtures::find(tree, Role::Heading { level: 2 }, "Your accounts · 0".as_bytes()).is_some());
        let band = super::inbox::subtree(tree, super::inbox::styled(tree, crate::look::CONNECT_EMPTY));
        assert!(!tree.nodes().iter().any(|n| n.styles.as_slice().contains(&crate::look::SETTINGS_QUIET_BAND)));
        assert!(fixtures::find(tree, Role::Link, b"+ Connect a model").is_none());
        assert!(String::from_utf8_lossy(&fixtures::text(tree)).contains("No model of yours yet."));
        let act = band.iter().find(|n| n.role == Role::Link && n.name.as_deref() == Some(b"Connect a model")).unwrap();
        assert!(act.styles.as_slice().contains(&crate::look::CONNECT_PRIMARY));
        assert!(!act.styles.as_slice().contains(&crate::look::SETTINGS_SMALL));
        let id = act.id;
        assert_eq!(
            crate::act(&mut view, id, Act::Press),
            Some(d::Action::Pages(d::pages::Act::Go { address: d::Address::ConnectModel { way: None } }))
        );
    }
}

#[test]
fn deployment_accounts_and_the_playgrounds_are_sections_of_the_same_rows() {
    let (mut owner, _, accounts) = owner_root_with_accounts();
    deliver(&mut owner, accounts, card_document(b"openai", b"Pat", c::AccountKind::SignIn));
    let playground = fixtures::playground_root(d::Address::Accounts, fixtures::PROJECT, true);
    for (root, titles, count) in [
        (&owner, ["Your accounts · 1", "Deployment accounts"], 2),
        (&playground, ["Your accounts · 0", "The playground"], 1),
    ] {
        for view in [fixtures::page_view(root), fixtures::browser_page_view(root)] {
            let tree = view.tree();
            let sections: Vec<_> =
                tree.nodes().iter().filter(|n| n.styles.as_slice().contains(&crate::look::SETTINGS_SECTION)).collect();
            assert_eq!(sections.len(), 2);
            for (section, title) in sections.iter().zip(titles) {
                let at = tree.nodes().iter().position(|n| n.id == section.id).unwrap();
                let nodes = super::inbox::subtree(tree, at);
                assert!(nodes.iter().any(|n| n.role == Role::Heading { level: 2 }
                    && n.name.as_deref() == Some(title.as_bytes())
                    && n.styles.as_slice().contains(&crate::look::SETTINGS_SECTION_TITLE)));
                if title != "Your accounts · 0" {
                    assert!(nodes.iter().any(|n| n.styles.as_slice().contains(&crate::look::SETTINGS_LINES)));
                    assert!(nodes.iter().any(|n| n.role == Role::Disclosure
                        && n.styles.as_slice().contains(&crate::look::SETTINGS_ROW)
                        && n.states.is_empty()));
                }
            }
            assert_eq!(
                tree.nodes().iter().filter(|n| n.styles.as_slice().contains(&crate::look::SETTINGS_ROW)).count(),
                count
            );
        }
    }
}

fn assert_no_old_chrome(root: &d::Domain) {
    for view in [fixtures::page_view(root), fixtures::browser_page_view(root)] {
        for node in view.tree().nodes() {
            for style in [
                crate::look::CARD,
                crate::look::PAGEBAR,
                crate::look::HEADING,
                crate::look::SUBTLE_COUNT,
                crate::look::DETAILS,
                crate::look::EMPTY,
            ] {
                assert!(!node.styles.as_slice().contains(&style), "old chrome {style:?} on {:?}", node.name);
            }
            if node.role == Role::Region && node.name.as_deref() == Some(b"Pays for") {
                assert!(node.styles.as_slice().contains(&crate::look::SETTINGS_SETTING));
            }
        }
    }
}

#[test]
fn the_old_chrome_is_gone() {
    for state in [
        c::AccountState::Granted,
        c::AccountState::Asking,
        c::AccountState::Later { until: None },
        c::AccountState::SignInAgain,
        c::AccountState::Misconfigured,
        c::AccountState::Cancelled,
        c::AccountState::Closing,
        c::AccountState::Moving,
        c::AccountState::Unloaded { at: c::AccountPlace::File, why: c::AccountUnread::Unreadable },
    ] {
        let (mut root, watch) = fixtures::account_root();
        let mut doc = card_document(b"openai", b"Pat", c::AccountKind::SignIn);
        doc.accounts.get_mut(0).unwrap().sources.get_mut(0).unwrap().state = state;
        deliver(&mut root, watch, doc);
        assert_no_old_chrome(&root);
        action(&mut root, A::KeyFor { row: Row { holder: c::AskAccountHolder::Own, account: 1 }, source: None });
        assert_no_old_chrome(&root);
        let name = root.pages().account_name();
        action(
            &mut root,
            A::Submit {
                name,
                row: Row { holder: c::AskAccountHolder::Own, account: 1 },
                operation: Operation::RemoveAccount,
            },
        );
        assert_no_old_chrome(&root);
    }
    assert_no_old_chrome(&owner_root().0);
    assert_no_old_chrome(&fixtures::playground_root(d::Address::Accounts, fixtures::PROJECT, true));
}
