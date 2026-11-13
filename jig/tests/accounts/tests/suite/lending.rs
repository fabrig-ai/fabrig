//! A party lends watches through the core; requests and documents cross the same seam.
use jig_accounts_world::{LIMITS, PROVIDER, provider, providers};
use jig_core_kit::{
    client_peer::ClientPeer,
    fixtures::ask_as,
    world::{Setup, World},
};
use jig_domain_accounts::{self as accounts, LentEnd, LoginEnd};
use jig_domain_core::{
    self as core,
    documents::{self as d, Document, Watched, Window},
};
use jig_domain_people as people;
use jig_test_domain as root;
use skein_lib::Token;

const ACCOUNTS: u64 = 100;
const TERMINAL: u64 = 107;
const PARTY: u64 = 1;
const SEED: u64 = 1937;

fn configuration() -> root::Config {
    let (mut config, _) = jig_core_kit::world::fixture(SEED, false);
    let mut anthropic = provider(PROVIDER, false, true);
    anthropic.name = b"Anthropic".as_slice().into();
    anthropic.offers.programs = Box::new([b"claude_code".as_slice().into()]);
    config.core.settings.llm_providers = providers(vec![anthropic]);
    config
}
fn fixture() -> World {
    fixture_with_names(LIMITS.program_names)
}
fn fixture_with_names(names: u32) -> World {
    let config = configuration();
    let (_, mut limits) = jig_core_kit::world::fixture(SEED, false);
    limits.core.accounts = accounts::Limits { program_names: names, ..LIMITS };
    limits.core.views.watchers = 16;
    limits.core.people.requests = 32;
    limits.core.people.sign_ins = 4;
    let mut world = World::new(Setup {
        seed: SEED,
        configuration: Some(config),
        limits,
        rows: Some(Box::new([])),
        initial: Some(people::Ask::MarkSeen { seen: people::Seen::Results { through: 0 } }),
        require_assignment: false,
        ..Setup::default()
    });
    world.client_peer = Some(ClientPeer::routed());
    open(&mut world, ACCOUNTS, Watched::Accounts);
    world
}
fn send(world: &mut World, event: core::Event) {
    world.send(root::Event::Core(event));
}
fn open(world: &mut World, watcher: u64, subject: Watched) {
    send(
        world,
        core::Event::Watch {
            watcher: Token::new(watcher),
            sign_in: world.session(),
            subject,
            window: Window { rows: 4, after: None },
        },
    );
}
fn terminal(world: &mut World, watcher: u64, number: u64) {
    open(world, watcher, Watched::Terminal { terminal: number });
}
fn close(world: &mut World, watcher: u64) {
    send(world, core::Event::Unwatch { watcher: Token::new(watcher) });
}
fn document(world: &mut World, watcher: u64) -> Document {
    // Consume the previous delivery so the newest marked document can arrive.
    send(world, core::Event::Delivered { watcher: Token::new(watcher) });
    world
        .client_peer
        .as_ref()
        .expect("client peer")
        .delivered
        .iter()
        .rev()
        .find_map(|(_, delivery)| match delivery {
            root::Delivery::Core(core::Held::Delivery { watcher: found, document }) if found.raw() == watcher => {
                Some(*document.clone())
            }
            _ => None,
        })
        .expect("watch delivered")
}
fn account_doc(world: &mut World) -> d::Accounts {
    let Document::Accounts(doc) = document(world, ACCOUNTS) else { panic!("accounts document") };
    *doc
}
fn terminal_doc(world: &mut World, watcher: u64) -> Option<accounts::TerminalLogin> {
    let Document::Terminal { login } = document(world, watcher) else { panic!("terminal document") };
    login
}
fn change_as(
    world: &mut World,
    sign_in: u64,
    key: u8,
    holder: people::AccountHolder,
    change: people::AccountChange,
) -> people::Outcome {
    ask_as(world, sign_in, key, people::Ask::AccountChange { holder, change });
    match world.people_answers.last().expect("account answer") {
        people::Reply::Outcome(outcome) => outcome.clone(),
        people::Reply::Refused(refusal) => people::Outcome::Refused(refusal.clone()),
        reply => panic!("account outcome: {reply:?}"),
    }
}
fn change(world: &mut World, key: u8, change: people::AccountChange) -> people::Outcome {
    change_as(world, world.session(), key, people::AccountHolder::Own, change)
}
fn ask(world: &mut World, key: u8, name: &[u8]) -> people::Outcome {
    change(world, key, people::AccountChange::LoginProgram { provider: PROVIDER, name: name.into() })
}
fn report(world: &mut World, key: u8, number: u32, exit: people::LoginExit) -> people::Outcome {
    change(world, key, people::AccountChange::LoginEnded { number, exit })
}
fn end(world: &mut World, number: u32, expected: LoginEnd) {
    let doc = account_doc(world);
    assert!(doc.settled);
    assert_eq!(doc.lending, None);
    assert_eq!(doc.lent_end, Some(LentEnd { number, end: expected }));
}

#[test]
fn terminal_watches_offer_withdraw_and_refuse_the_fifth() {
    let mut world = fixture();
    assert!(!account_doc(&mut world).lendable);
    for number in 1..=4 {
        terminal(&mut world, TERMINAL + number, number);
    }
    assert!(account_doc(&mut world).lendable);
    terminal(&mut world, TERMINAL + 5, 5);
    assert!(world.client_peer.as_ref().expect("client peer").now.iter().any(|now| matches!(now, core::Now::WatchRefused { watcher, why: core::WatchRefusal::Busy } if watcher.raw() == TERMINAL + 5)));
    for number in 1..=3 {
        close(&mut world, TERMINAL + number);
    }
    assert!(account_doc(&mut world).lendable);
    close(&mut world, TERMINAL + 4);
    assert!(!account_doc(&mut world).lendable);
    terminal(&mut world, TERMINAL, 7);
    assert!(account_doc(&mut world).lendable, "withdrawal releases the bound");
}

#[test]
fn the_newest_terminal_gets_the_login_and_exit_zero_declares_work() {
    let mut world = fixture();
    terminal(&mut world, TERMINAL - 1, 6);
    terminal(&mut world, TERMINAL, 7);
    assert_eq!(ask(&mut world, 21, b"work"), people::Outcome::AccountLoginAsked { number: 1 });
    assert_eq!(terminal_doc(&mut world, TERMINAL - 1), None);
    let pending =
        accounts::TerminalLogin { number: 1, provider: PROVIDER, name: b"work".as_slice().into(), cancelled: false };
    assert_eq!(terminal_doc(&mut world, TERMINAL), Some(pending.clone()));
    let doc = account_doc(&mut world);
    assert_eq!(doc.lending, Some(pending));
    assert!(!doc.settled);
    assert_eq!(report(&mut world, 22, 1, people::LoginExit::Exited { code: 0 }), people::Outcome::AccountChanged);
    end(&mut world, 1, LoginEnd::SignedIn { source: 1 });
    let doc = account_doc(&mut world);
    assert_eq!(doc.accounts.len(), 1, "joining no account");
    let account = doc.accounts.get(0).expect("declared account");
    assert_eq!(account.holder, accounts::Holder::Party(PARTY));
    let source = account.sources.get(0).expect("declared source");
    assert_eq!(source.kind, accounts::Kind::Program);
    assert_eq!(source.name.as_deref(), Some(b"work".as_slice()));
    assert_eq!(terminal_doc(&mut world, TERMINAL), None);
}

#[test]
fn login_refusals_precede_no_terminal_then_login_under_way() {
    let mut world = fixture();
    let missing = people::Outcome::Refused(people::Refusal::NoTerminal);
    assert_eq!(ask(&mut world, 21, b"work"), missing);
    assert_eq!(
        ask(&mut world, 22, b"Upper"),
        people::Outcome::Refused(people::Refusal::Limit { which: people::LimitKind::Words })
    );
    assert_eq!(
        change(&mut world, 23, people::AccountChange::LoginProgram { provider: 99, name: b"work".as_slice().into() }),
        people::Outcome::Refused(people::Refusal::NotOffered)
    );
    let session = world.session();
    assert_eq!(
        change_as(
            &mut world,
            session,
            24,
            people::AccountHolder::Deployment,
            people::AccountChange::LoginProgram { provider: PROVIDER, name: b"work".as_slice().into() }
        ),
        people::Outcome::Refused(people::Refusal::NotOffered)
    );
    terminal(&mut world, TERMINAL, 7);
    assert_eq!(ask(&mut world, 25, b"work"), people::Outcome::AccountLoginAsked { number: 1 });
    assert_eq!(
        ask(&mut world, 26, b"Upper"),
        people::Outcome::Refused(people::Refusal::Limit { which: people::LimitKind::Words })
    );
    assert_eq!(ask(&mut world, 27, b"work"), people::Outcome::Refused(people::Refusal::LoginUnderWay));
}

#[test]
fn exit_zero_checks_a_live_source_instead_of_declaring_again() {
    let mut world = fixture();
    terminal(&mut world, TERMINAL, 7);
    ask(&mut world, 21, b"work");
    report(&mut world, 22, 1, people::LoginExit::Exited { code: 0 });
    let before = world.domain.accounts().next_number();
    assert_eq!(ask(&mut world, 23, b"work"), people::Outcome::AccountLoginAsked { number: 2 });
    report(&mut world, 24, 2, people::LoginExit::Exited { code: 0 });
    end(&mut world, 2, LoginEnd::SignedIn { source: 1 });
    assert_eq!(account_doc(&mut world).accounts.len(), 1);
    assert_eq!(world.domain.accounts().next_number(), before + 1, "a check advances the grant epoch");
}

#[test]
fn unsuccessful_exits_record_not_finished_without_declaring() {
    let mut world = fixture();
    terminal(&mut world, TERMINAL, 7);
    for (index, (exit, expected)) in [
        (people::LoginExit::Exited { code: 1 }, accounts::LoginExit::Exited { code: 1 }),
        (people::LoginExit::Killed { signal: 15 }, accounts::LoginExit::Killed { signal: 15 }),
        (people::LoginExit::NotStarted, accounts::LoginExit::NotStarted),
    ]
    .into_iter()
    .enumerate()
    {
        let number = u32::try_from(index + 1).expect("small table");
        let key = u8::try_from(21 + index * 2).expect("small table");
        assert_eq!(ask(&mut world, key, b"work"), people::Outcome::AccountLoginAsked { number });
        assert_eq!(report(&mut world, key + 1, number, exit), people::Outcome::AccountChanged);
        end(&mut world, number, LoginEnd::NotFinished { exit: Some(expected) });
    }
    assert!(account_doc(&mut world).accounts.is_empty());
}

#[test]
fn cancel_marks_both_documents_and_dominates_every_report() {
    for exit in [
        people::LoginExit::Exited { code: 0 },
        people::LoginExit::Exited { code: 1 },
        people::LoginExit::Killed { signal: 9 },
        people::LoginExit::NotStarted,
    ] {
        let mut world = fixture();
        terminal(&mut world, TERMINAL, 7);
        ask(&mut world, 21, b"work");
        assert_eq!(
            change(&mut world, 22, people::AccountChange::CancelLogin { number: 1 }),
            people::Outcome::AccountChanged
        );
        assert!(terminal_doc(&mut world, TERMINAL).expect("cancelled login").cancelled);
        assert!(account_doc(&mut world).lending.expect("cancelled login").cancelled);
        report(&mut world, 23, 1, exit);
        end(&mut world, 1, LoginEnd::Cancelled);
        assert!(account_doc(&mut world).accounts.is_empty());
    }
}

#[test]
fn withdrawing_the_selected_watch_ends_its_login_and_late_or_foreign_reports_refuse() {
    let mut world = fixture();
    let owner = world.session();
    let other = jig_core_kit::fixtures::second_party(&mut world, b"other", people::Role::Member);
    world.sign_in = Some(owner);
    terminal(&mut world, TERMINAL - 1, 6);
    terminal(&mut world, TERMINAL, 7);
    ask(&mut world, 21, b"work");
    for request in [
        people::AccountChange::CancelLogin { number: 1 },
        people::AccountChange::LoginEnded { number: 1, exit: people::LoginExit::Exited { code: 0 } },
    ] {
        let key = if matches!(request, people::AccountChange::CancelLogin { .. }) { 22 } else { 23 };
        assert_eq!(
            change_as(&mut world, other, key, people::AccountHolder::Own, request),
            people::Outcome::Refused(people::Refusal::NotLending)
        );
    }
    close(&mut world, TERMINAL - 1);
    assert!(account_doc(&mut world).lending.is_some(), "an older offer going leaves the login");
    close(&mut world, TERMINAL);
    end(&mut world, 1, LoginEnd::NotFinished { exit: None });
    assert_eq!(
        report(&mut world, 24, 1, people::LoginExit::Exited { code: 0 }),
        people::Outcome::Refused(people::Refusal::NotLending)
    );
    assert_eq!(
        change(&mut world, 25, people::AccountChange::CancelLogin { number: 1 }),
        people::Outcome::Refused(people::Refusal::NotLending)
    );
    terminal(&mut world, TERMINAL + 1, 8);
    ask(&mut world, 26, b"work");
    change(&mut world, 27, people::AccountChange::CancelLogin { number: 2 });
    close(&mut world, TERMINAL + 1);
    end(&mut world, 2, LoginEnd::Cancelled);
}

#[test]
fn restart_forgets_offers_and_logins_without_recording_an_end() {
    let mut world = fixture();
    terminal(&mut world, TERMINAL, 7);
    ask(&mut world, 21, b"work");
    world.restart_with(configuration());
    open(&mut world, ACCOUNTS, Watched::Accounts);
    let doc = account_doc(&mut world);
    assert!(!doc.lendable);
    assert!(doc.settled);
    assert_eq!(doc.lending, None);
    assert_eq!(doc.lent_end, None);
    terminal(&mut world, TERMINAL, 7);
    assert_eq!(ask(&mut world, 22, b"work"), people::Outcome::AccountLoginAsked { number: 1 });
}

#[test]
fn a_name_taken_before_the_report_refuses_the_end_and_the_next_ask() {
    let mut world = fixture();
    let owner = world.session();
    let other = jig_core_kit::fixtures::second_party(&mut world, b"other", people::Role::Member);
    world.sign_in = Some(owner);
    terminal(&mut world, TERMINAL, 7);
    ask(&mut world, 21, b"work");
    assert!(matches!(
        change_as(
            &mut world,
            other,
            22,
            people::AccountHolder::Own,
            people::AccountChange::DeclareProgram { provider: PROVIDER, join: None, name: b"work".as_slice().into() }
        ),
        people::Outcome::AccountProgramDeclared { .. }
    ));
    assert_eq!(
        ask(&mut world, 23, b"work"),
        people::Outcome::Refused(people::Refusal::NameTaken),
        "declaration before LoginUnderWay"
    );
    assert_eq!(report(&mut world, 24, 1, people::LoginExit::Exited { code: 0 }), people::Outcome::AccountChanged);
    end(&mut world, 1, LoginEnd::Refused { refusal: accounts::Refusal::ProgramNameReserved });
    close(&mut world, TERMINAL);
    assert_eq!(
        ask(&mut world, 25, b"work"),
        people::Outcome::Refused(people::Refusal::NameTaken),
        "declaration before NoTerminal"
    );
}
#[test]
fn lifetime_names_spent_before_the_report_refuse_its_declaration() {
    let mut world = fixture_with_names(1);
    terminal(&mut world, TERMINAL, 7);
    ask(&mut world, 21, b"work");
    assert!(matches!(
        change(
            &mut world,
            22,
            people::AccountChange::DeclareProgram { provider: PROVIDER, join: None, name: b"other".as_slice().into() }
        ),
        people::Outcome::AccountProgramDeclared { .. }
    ));
    report(&mut world, 23, 1, people::LoginExit::Exited { code: 0 });
    end(&mut world, 1, LoginEnd::Refused { refusal: accounts::Refusal::Limit { bound: accounts::Bound::Sources } });
    close(&mut world, TERMINAL);
    assert_eq!(
        ask(&mut world, 24, b"work"),
        people::Outcome::Refused(people::Refusal::Limit { which: people::LimitKind::Sources })
    );
}
