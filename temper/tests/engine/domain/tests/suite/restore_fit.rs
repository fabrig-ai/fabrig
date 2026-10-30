//! Own forge fit refuses before switch; expiry uses the root entry observation.
use skein_lib::{Time, Token, Wall};
use temper_engine_domain::{self as root, Range, Record, Write};
use temper_engine_domain_forge as forge;
use temper_engine_domain_world::{direct::Driver, store::Store, walking};

fn binding(number: u8) -> Record {
    Record::Forge { row: Box::new(forge::Stored::Binding { address: Box::new([number]), subject: Box::new([]) }) }
}
fn driver(pending: bool) -> Driver {
    let mut store = Driver::new(Store::new()).store;
    store.pending_generation = pending;
    let mut limits = walking::limits();
    limits.forge.bindings = 3;
    let mut driver = Driver::cold(store, walking::config(91), &limits);
    driver.env.wall = Wall::from_nanos(100);
    driver.env.now = Time::from_nanos(100);
    driver
}
fn forge_page(driver: &mut Driver, previous: Option<Token>) -> Token {
    for _ in 0_u32..300 {
        driver.advance(true);
        if let Some((owner, Range::Forge)) = driver.loads.last()
            && Some(*owner) != previous
        {
            let owner = *owner;
            driver.events.clear();
            return owner;
        }
    }
    panic!("forge restore load");
}
#[test]
fn a_forge_fit_failure_names_its_step_and_releases_no_wires() {
    for pending in [false, true] {
        let mut driver = driver(pending);
        driver.store.seed((1..=5).map(binding).collect(), Box::new([]));
        for _ in 0_u32..400 {
            driver.advance(true);
            if driver.stopped {
                break;
            }
        }
        assert!(driver.stopped);
        let first = forge::RestoreFit {
            family: Some(forge::RecordKind::Binding),
            set: forge::RestoreSet::Rows(forge::RecordKind::Binding),
            held: 3,
            additional: 1,
            limit: 3,
        };
        assert_eq!(driver.root.forge_fit_failure(), Some(&first));
        assert_eq!(
            driver.root.restart_failure(),
            Some(jig_domain_core::RestartStep::RestoreConnector { connector: 0 })
        );
        assert_eq!(driver.root.restore_load_failure(), None, "own set diagnosis stays distinct from load decoding");
        let loads = driver.loads.len();
        for _ in 0_u32..3 {
            driver.send(root::Event::Resume);
            root::fire(&mut driver.root, &driver.env);
            root::release(&mut driver.root, &driver.env, &mut driver.out);
            driver.collect();
        }
        let owner = driver.loads.last().expect("forge load was issued").0;
        driver.send(root::Event::Store(root::StoreInput::Unloaded { owner }));
        driver.send(root::Event::Store(root::StoreInput::Loaded {
            owner,
            rows: Box::new([binding(99)]),
            next: None,
            texts: None,
        }));
        assert_eq!(driver.root.forge_fit_failure(), Some(&first));
        assert_eq!(driver.loads.len(), loads, "no continuation after the failed page");
        assert!(driver.switches.is_empty());
        assert!(driver.delivered.is_empty());
        assert!(driver.host_requests.is_empty());
        assert!(driver.immediate.is_empty(), "no connection/setup outputs: {:?}", driver.immediate);
        assert!(!driver.transactions.iter().flatten().any(|write| matches!(write, Write::Wires)));
        assert!(!driver.root.connectors_restored());
        assert!(!driver.root.ready());
    }
}
fn outcome(token: u64, until: u64) -> Record {
    Record::Forge {
        row: Box::new(forge::Stored::Application {
            token,
            until: Wall::from_nanos(until),
            outcome: forge::ApplicationOutcome::ForgeRefused(forge::AdoptionRefusal::Refused),
        }),
    }
}
#[test]
fn application_pages_use_the_restart_entry_wall_observation() {
    for later in [50, 200] {
        for over_limit in [false, true] {
            let mut driver = driver(false);
            let owner = forge_page(&mut driver, None);
            // Equality expires; future-at-start rows stay, despite alternating clock movement between pages.
            let count = driver.env.limits.forge.applications;
            let mut rows = Vec::new();
            for token in 0..count + 10 {
                rows.push(outcome(u64::from(token), 100));
            }
            let mut page_owner = owner;
            for token in 0..count {
                rows.push(outcome(u64::from(count + 10 + token), 101));
            }
            if over_limit {
                rows.push(outcome(1000, 101));
            }
            let most = usize::try_from(driver.env.limits.loads.rows).expect("configured page count fits usize");
            let total = rows.len();
            for (index, page) in rows.chunks(most).enumerate() {
                driver.env.wall = Wall::from_nanos(if index % 2 == 0 { later } else { 300 - later });
                let done = (index + 1) * most >= total;
                let next = if done {
                    None
                } else {
                    Some(root::Key::Forge(forge::Key::Application(
                        u64::try_from((index + 1) * most).expect("bounded page cursor"),
                    )))
                };
                driver.send(root::Event::Store(root::StoreInput::Loaded {
                    owner: page_owner,
                    rows: page.to_vec().into_boxed_slice(),
                    next,
                    texts: None,
                }));
                if done && over_limit {
                    assert_eq!(
                        driver.root.forge_fit_failure(),
                        Some(&forge::RestoreFit {
                            family: Some(forge::RecordKind::Application),
                            set: forge::RestoreSet::Rows(forge::RecordKind::Application),
                            held: u64::from(count),
                            additional: 1,
                            limit: u64::from(count)
                        })
                    );
                } else {
                    assert_eq!(driver.root.forge_fit_failure(), None);
                }
                if !done {
                    page_owner = forge_page(&mut driver, Some(page_owner));
                }
            }
            assert_eq!(driver.stopped, over_limit);
        }
    }
}
