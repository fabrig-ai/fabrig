//! Issued names precede live account rows in jig's restart walk
//! (jig's protocol/store.md, 4.4 and 7.2; domain/root.md, 9.1).
use jig_domain_accounts as accounts;
use jig_domain_core as core;
use jig_domain_people as people;
use skein_lib::{Queue, ReplyTo, Token};
use temper_engine_domain::{self as engine, Key, Record};
use temper_engine_domain_world::{direct::Driver, store::Store, walking};

const SEED: u64 = 91;
const CLAUDE: u32 = 2;
const CHATGPT: u32 = 3;
const LOGIN: &[u8] = b"login";
// One declared program account and one imported sign-in account.
const ACCOUNTS: u32 = 2;
// The walking world's normal page holds the reservation and counter together.
const PAGE: u32 = 2;
// A one-row page crosses that same boundary through an exclusive `after`.
const SINGLE_ROW: u32 = 1;
// Six stored rows, read two at a time.
const PAGES: u32 = 3;
// The direct world's existing settlement step budget, with an earlier exit
// on the restore refusal so a stopped root need not become quiescent.
const RESTORE_STEPS: u32 = 200;

fn config() -> engine::Config {
    let mut config = walking::config(SEED);
    config.core.settings.llm_providers = accounts::Providers {
        providers: Box::new([
            accounts::Provider {
                number: CLAUDE,
                name: b"Fake Claude".as_slice().into(),
                signed_in_plan: true,
                manage_usage: None,
                models: Box::new([]),
                offers: accounts::Offers {
                    sign_in: false,
                    key: false,
                    programs: Box::new([b"claude_code".as_slice().into()]),
                },
            },
            accounts::Provider {
                number: CHATGPT,
                name: b"Fake ChatGPT".as_slice().into(),
                signed_in_plan: true,
                manage_usage: None,
                models: Box::new([]),
                offers: accounts::Offers { sign_in: true, key: false, programs: Box::new([]) },
            },
        ]),
    };
    config
}

fn limits() -> engine::Limits {
    let mut limits = walking::limits();
    limits.accounts.accounts = ACCOUNTS;
    limits.accounts.holder_accounts = ACCOUNTS;
    // The core reserves per-run slots for configured credentials and accounts.
    limits.fleet.grants = limits.accounts.credentials + ACCOUNTS;
    limits
}

fn kept_store(pending_export: bool) -> Store {
    // Start empty: Driver::configured seeds deployment key accounts, which
    // would consume both slots before either operation in this story.
    let mut driver = Driver::cold(Store::new(), config(), &limits());
    driver.settle();
    driver.sign_in();
    driver.settle();
    driver.send(engine::Event::Party(engine::PartyInput::Ask {
        models: Box::new([]),
        reply_to: ReplyTo::new(Token::new(101)),
        sign_in: driver.session(),
        key: [1; 16],
        ask: people::Ask::AccountChange {
            holder: people::AccountHolder::Own,
            change: people::AccountChange::DeclareProgram { provider: CLAUDE, join: None, name: LOGIN.into() },
        },
    }));
    driver.settle();
    // Play only the imported record's component boundary, with no real login
    // or credential bytes. The root and accounts child write the durable rows.
    driver.send(engine::Event::Party(engine::PartyInput::Key {
        reply_to: ReplyTo::new(Token::new(102)),
        sign_in: driver.session(),
        key: [2; 16],
        ask: Box::new(people::Ask::AccountChange {
            holder: people::AccountHolder::Own,
            change: people::AccountChange::MoveIn { digest: [7; 32] },
        }),
        carried: core::Carried { number: 1_u64 << 63, digest: [7; 32] },
    }));
    driver.settle();
    let source = driver
        .protocol
        .accounts
        .iter()
        .find_map(|request| {
            if let accounts::Request::Signing(accounts::Signing::Open { source, .. }) = request {
                Some(*source)
            } else {
                None
            }
        })
        .expect("the fake component is asked to open the imported record");
    driver.send(engine::Event::Account(accounts::Event::Opened {
        source,
        provider: CHATGPT,
        subject: b"fake-chatgpt-owner".as_slice().into(),
        display: b"Fake owner".as_slice().into(),
    }));
    driver.settle();
    driver.protocol.sign_in_sources.insert(source);
    driver.send(engine::Event::Account(accounts::Event::Adopted { source, kept: true }));
    driver.settle();
    if pending_export {
        change(&mut driver, 103, people::AccountChange::MoveOut { account: 3 });
        assert!(driver.protocol.accounts.iter().any(|request| matches!(request,
            accounts::Request::Signing(accounts::Signing::Withdraw { source: got, .. }) if *got == source)));
        driver.send(engine::Event::Account(accounts::Event::Withdrawn { source, readable: true }));
        driver.settle();
        assert!(driver.delivered.iter().any(|delivery| matches!(delivery,
            engine::Delivery::WebReply { reply: people::Reply::Outcome(people::Outcome::AccountMovingOut {
                account: 3, number: 4, source: got }), .. } if *got == source)));
        let document = account_status(&mut driver);
        let source = document
            .accounts
            .iter()
            .find(|row| row.number == 3)
            .expect("imported account")
            .sources
            .iter()
            .next()
            .expect("imported source");
        assert_eq!(source.state, core::documents::AccountState::Moving);
        assert!(!source.usable);
    }
    // Replay the real durable commits into a fresh store component.
    driver.store.fork()
}

fn change(driver: &mut Driver, key: u8, change: people::AccountChange) {
    driver.send(engine::Event::Party(engine::PartyInput::Ask {
        models: Box::new([]),
        reply_to: ReplyTo::new(Token::new(u64::from(key))),
        sign_in: driver.session(),
        key: [key; 16],
        ask: people::Ask::AccountChange { holder: people::AccountHolder::Own, change },
    }));
    driver.settle();
}

fn account_status(driver: &mut Driver) -> Box<core::documents::Accounts> {
    let watcher = Token::new(1000 + u64::try_from(driver.delivered.len()).expect("small delivery count"));
    driver.send(engine::Event::Party(engine::PartyInput::Watch {
        watcher,
        sign_in: driver.session(),
        subject: core::documents::Watched::Accounts,
        window: core::documents::Window { rows: ACCOUNTS, after: None },
    }));
    driver.settle();
    driver
        .delivered
        .iter()
        .rev()
        .find_map(|delivery| {
            if let engine::Delivery::Document { watcher: got, document } = delivery
                && *got == watcher
                && let core::documents::Document::Accounts(accounts) = &**document
            {
                Some(accounts.clone())
            } else {
                None
            }
        })
        .expect("authenticated accounts read")
}

#[test]
fn an_imported_pending_export_restores_and_answers_reads() {
    let store = kept_store(true);
    let account = store.rows.get(&account_key(accounts::Key::Account { number: 3 })).expect("imported account").clone();
    let mut driver = Driver::cold(store, config(), &limits());
    driver.protocol.sign_in_sources.insert(2);
    // Hold only the existing protocol stub's grant, so refresh can be observed
    // before it makes the restored source usable.
    let mut grant = None;
    for _ in 0..RESTORE_STEPS {
        driver.advance(true);
        if let Some(at) = driver.events.iter().position(|event| {
            matches!(
                event,
                engine::Event::Account(accounts::Event::Component(accounts::FromComponent::Granted { account: 2, .. }))
            )
        }) {
            assert!(grant.is_none(), "one refresh grant");
            grant = driver.events.remove(at);
        }
        if driver.root.quiescent() && driver.events.is_empty() && driver.store.pending.is_empty() {
            break;
        }
    }
    assert_eq!(driver.root.restore_load_failure(), None);
    assert!(driver.root.ready());
    driver.sign_in();
    driver.settle();
    let refresh = driver
        .protocol
        .accounts
        .iter()
        .position(|request| matches!(request, accounts::Request::Signing(accounts::Signing::Refresh { source: 2 })))
        .expect("restart refreshes the moving source");
    let want = driver
        .protocol
        .accounts
        .iter()
        .position(|request| matches!(request, accounts::Request::Component(accounts::ToComponent::Want { account: 2 })))
        .expect("grant requested");
    assert!(refresh < want, "refresh precedes the grant request");
    let before = account_status(&mut driver);
    let imported = before.accounts.iter().find(|row| row.number == 3).expect("same imported account");
    assert_eq!(imported.provider, Some(CHATGPT));
    let source = imported.sources.iter().next().expect("imported source");
    assert_eq!(source.number, 2);
    assert!(!source.usable, "refresh has not answered yet");
    // The old transfer cannot complete after restart, even before refresh answers.
    change(&mut driver, 106, people::AccountChange::Written { number: 4 });
    assert!(driver.delivered.iter().any(|delivery| matches!(
        delivery,
        engine::Delivery::WebReply {
            reply: people::Reply::Outcome(people::Outcome::Refused(people::Refusal::AccountMove {
                why: people::AccountMoveWhy::Cancelled
            })),
            ..
        }
    )));
    driver.send(grant.expect("the existing component supplied a refresh grant"));
    driver.settle();
    let after = account_status(&mut driver);
    assert!(after.usable);
    let imported = after.accounts.iter().find(|row| row.number == 3).expect("same imported account");
    let source = imported.sources.iter().next().expect("imported source");
    assert_eq!(source.number, 2);
    assert_eq!(source.state, core::documents::AccountState::Granted);
    assert!(source.usable);
    assert_eq!(driver.store.rows.get(&account.key()), Some(&account), "imported identity unchanged");
    driver.send(engine::Event::Party(engine::PartyInput::Read {
        reply_to: ReplyTo::new(Token::new(107)),
        sign_in: driver.session(),
        read: core::reads::Read::Settings,
    }));
    driver.settle();
    assert!(driver.delivered.iter().any(|delivery| matches!(delivery,
        engine::Delivery::Read { answer: core::reads::Answer::Document(document), .. }
            if matches!(&**document, core::documents::Document::Settings(_)))));
}

fn account_key(key: accounts::Key) -> Key {
    Key::Core(core::Key::Accounts(key))
}

#[test]
fn declared_and_imported_rows_walk_reservations_before_the_counter() {
    let mut store = kept_store(false);
    let range = engine::Range::Core(engine::CoreRange::Accounts);
    let mut after = None;
    let mut keys = Vec::new();
    for _ in 0..PAGES {
        let (rows, next) = store.page(&range, after.as_ref(), PAGE);
        temper_engine_domain_world::store::page_rules(&range, after.as_ref(), &rows, next.as_ref())
            .expect("real account writer walk");
        keys.extend(rows.iter().map(Record::key));
        after = next;
    }
    assert_eq!(
        keys,
        vec![
            account_key(accounts::Key::Issued { number: 1 }),
            account_key(accounts::Key::Next),
            account_key(accounts::Key::Account { number: 0 }),
            account_key(accounts::Key::Account { number: 3 }),
            account_key(accounts::Key::Source { account: 0, number: 1 }),
            account_key(accounts::Key::Source { account: 3, number: 2 }),
        ]
    );
}

#[test]
fn restart_accepts_issued_and_counter_in_one_page() {
    let mut driver = Driver::cold(kept_store(false), config(), &limits());
    for _ in 0..RESTORE_STEPS {
        driver.advance(true);
        if driver.root.restore_load_failure().is_some() || driver.root.ready() {
            break;
        }
    }
    assert_eq!(driver.root.restore_load_failure(), None, "restart must accept the store's accounts walk");
    assert!(driver.root.ready(), "restored accounts permit the startup switch");
}

#[test]
fn accounts_load_accepts_counter_after_an_issued_page_cursor() {
    use engine::loads::{self, Loads, Request};
    let mut store = kept_store(false);
    let range = engine::Range::Core(engine::CoreRange::Accounts);
    let (_, after) = store.page(&range, None, SINGLE_ROW);
    let (rows, next) = store.page(&range, after.as_ref(), SINGLE_ROW);
    let mut loads = Loads::new(&limits().loads);
    // Each loads entry point emits at most one request or terminal.
    let mut out = Queue::with_capacity(1);
    let waiter = Token::new(u64::MAX);
    let owner = loads::begin(&mut loads, waiter, range, after, SINGLE_ROW, &mut out).expect("load admitted");
    let _ = out.pop().expect("store request");
    loads::loaded(&mut loads, owner, rows, next, &mut out);
    let terminal = out.pop();
    assert_eq!(
        terminal,
        Some(Request::Loaded {
            waiter,
            rows: Box::new([Record::Core(core::Record::Accounts(accounts::Stored::Next { number: 4 }))]),
            next: Some(account_key(accounts::Key::Next)),
            cut: None,
        }),
        "accounts continuation must accept the counter after Issued without skipping it"
    );
}

fn issued(number: u32) -> accounts::Stored {
    accounts::Stored::Issued { number, holder: accounts::Holder::Party(1), name: LOGIN.into(), signed_out: false }
}

fn account(number: u32) -> accounts::Stored {
    accounts::Stored::Account {
        number,
        provider: CLAUDE,
        holder: accounts::Holder::Party(1),
        place: 0,
        uses: None,
        subject: Box::new([]),
        display: Box::new([]),
    }
}

fn checked_accounts(after: Option<accounts::Key>, rows: Vec<accounts::Stored>) -> Option<engine::loads::Request> {
    use engine::loads::{self, Loads};
    let mut loads = Loads::new(&limits().loads);
    // One load request or terminal per entry point.
    let mut out = Queue::with_capacity(1);
    let owner = loads::begin(
        &mut loads,
        Token::new(104),
        engine::Range::Core(engine::CoreRange::Accounts),
        after.map(account_key),
        PAGE,
        &mut out,
    )
    .expect("load admitted");
    let _ = out.pop().expect("store request");
    let rows = rows.into_iter().map(|row| Record::Core(core::Record::Accounts(row))).collect();
    loads::loaded(&mut loads, owner, rows, None, &mut out);
    out.pop()
}

#[test]
fn the_first_account_follows_the_counter_cursor() {
    assert_eq!(
        checked_accounts(Some(accounts::Key::Next), vec![account(0)]),
        Some(engine::loads::Request::Loaded {
            waiter: Token::new(104),
            rows: Box::new([Record::Core(core::Record::Accounts(account(0)))]),
            next: None,
            cut: None,
        })
    );
}

#[test]
fn ordinary_accounts_pages_still_refuse_duplicates_and_backwards_rows() {
    use accounts::Key;
    for (after, rows) in [
        (None, vec![issued(1), issued(1)]),
        (Some(Key::Issued { number: 2 }), vec![issued(1)]),
        (Some(Key::Account { number: 3 }), vec![account(0)]),
        (Some(Key::Next), vec![issued(1)]),
        (None, vec![accounts::Stored::Next { number: 4 }, issued(1)]),
    ] {
        assert_eq!(
            checked_accounts(after, rows),
            Some(engine::loads::Request::Unloaded { waiter: Token::new(104), failure: engine::loads::Failure::Order })
        );
    }
}

#[test]
fn real_account_writer_pages_reject_duplicate_backward_and_before_cursor_copies_in_the_oracle() {
    use temper_engine_domain_world::store::page_rules;
    let mut store = kept_store(false);
    let range = engine::Range::Core(engine::CoreRange::Accounts);
    let (rows, next) = store.page(&range, None, PAGE);
    page_rules(&range, None, &rows, next.as_ref()).expect("valid bounded fixture");
    assert_eq!(rows.len(), 2, "issued reservation before the counter");
    for altered in [vec![rows[0].clone(), rows[0].clone()], vec![rows[1].clone(), rows[0].clone()]] {
        assert!(page_rules(&range, None, &altered, None).is_err());
    }
    assert!(page_rules(&range, Some(&rows[1].key()), &rows[..1], None).is_err());
    assert!(
        page_rules(&range, None, &rows, Some(&rows[0].key())).is_err(),
        "cursor must agree with the last account row"
    );
}
