//! Pushes cross the root's durability boundary, not the browser's (attention.md, 7.2).
use jig_domain_authority as authority;
use jig_domain_people as people;
use skein_lib::{ReplyTo, Token, Wall};
use temper_engine_domain::{self as engine, Delivery, Event, Key, Record, Write};
use temper_engine_domain_forge as forge;
use temper_engine_domain_world::{direct::Driver, fixtures, walking};

fn configuration() -> engine::Config {
    let mut config = walking::config(599);
    config.core.owners =
        Box::new([7, 8].map(|user| people::InitialOwner { project: 1, identity: forge::user_key(1, user) }));
    config.core.connectors = Box::new([1]);
    config.forge.push = Some(Box::new([b"http://push.test:4000".as_slice().into()]));
    config.core.defaults.rules.ceiling.delegation.kinds =
        Box::new([authority::Executor::Charter(1), authority::Executor::Role(0)]);
    let policy = &mut config.core.seed_projects[0].policy.policy;
    policy.ceiling.delegation.kinds.clone_from(&config.core.defaults.rules.ceiling.delegation.kinds);
    policy.roles[0].authority.delegation.kinds.clone_from(&policy.ceiling.delegation.kinds);
    config
}

fn device(party: u64) -> forge::Device {
    forge::Device {
        party,
        number: 1,
        endpoint: format!("http://push.test:4000/push/{party}").into_bytes().into_boxed_slice(),
        p256dh: [4; 65],
        auth: [7; 16],
        label: b"phone".as_slice().into(),
        at: Wall::EPOCH,
    }
}

#[test]
fn a_role_need_pushes_each_holder_after_commit_gone_commits_its_erase_and_restart_pushes_nothing() {
    let mut limits = walking::limits();
    limits.people.initial_owners = 2;
    limits.people.holdings = 4;
    limits.authority.executors = 2;
    limits.documents.executors = 2;
    let mut store = fixtures::restored_store(configuration(), &limits);
    let parties: Vec<_> = store
        .rows
        .values()
        .filter_map(|row| {
            if let Record::Core(jig_domain_core::Record::People(people::Stored::Person { number, .. })) = row {
                Some(*number)
            } else {
                None
            }
        })
        .collect();
    assert_eq!(parties.len(), 2);
    let devices: Vec<_> = parties.iter().copied().map(device).collect();
    store.seed(
        devices.iter().cloned().map(|row| Record::Forge { row: Box::new(forge::Stored::Device(row)) }).collect(),
        Box::new([]),
    );
    let mut driver = Driver::configured(store, [configuration(), configuration()], &limits);
    driver.settle();
    driver.sign_in();
    driver.settle();
    let people::Ask::Create { project, mut batch } = driver.chat_ask(1, b"Choose an implementation") else {
        panic!("task template")
    };
    batch[0].executor = people::TaskExecutor::Person(people::PersonAddress::Role(0));
    batch[0].flags.conversation = false;
    batch[0].flags.listed = true;
    batch[0].budget = people::Budget::Named;
    batch[0].authority.spend = 32;
    batch[0].authority.delegation = people::Delegation { kinds: Box::new([]), tasks: 0, depth: 0 };
    driver.push_answers.push_back(engine::PushAnswer::Gone);
    driver.send(Event::Party(engine::PartyInput::Ask {
        models: Box::new([]),
        reply_to: ReplyTo::new(Token::new(501)),
        sign_in: driver.session(),
        key: [59; 16],
        ask: people::Ask::Create { project, batch },
    }));
    for _ in 0..32 {
        driver.advance(false);
    }
    assert!(!driver.store.pending.is_empty(), "the need's decision awaits durability");
    assert!(driver.pushes.is_empty(), "no push before the decision commits");
    let number = driver.store.apply();
    for _ in 0..32 {
        driver.advance(false);
    }
    assert!(driver.pushes.is_empty(), "no push before the store's terminal");
    driver.send(Event::Store(engine::StoreInput::Committed { number, fill: 0 }));
    driver.settle();
    assert!(
        driver.delivered.iter().any(|delivery| matches!(
            delivery,
            Delivery::WebReply { reply: people::Reply::Outcome(people::Outcome::Created { .. }), .. }
        )),
        "the role task was admitted"
    );
    assert_eq!(driver.pushes.len(), 2);
    for (push, device) in driver.pushes.iter().zip(&devices) {
        assert_eq!((push.party, push.device), (device.party, 1));
        assert_eq!(push.topic.as_ref(), b"p1");
        assert_eq!(push.payload.as_ref(), br#"{"title":"temper","body":"1 thing needs you in Project 1","project":1}"#);
    }
    let gone = Key::Forge(forge::Key::Device { party: parties[0], number: 1 });
    assert!(!driver.store.rows.contains_key(&gone));
    assert!(driver.store.rows.contains_key(&Key::Forge(forge::Key::Device { party: parties[1], number: 1 })));
    let erase = driver
        .transactions
        .iter()
        .find(|writes| writes.iter().any(|write| matches!(write, Write::Erase(key) if key == &gone)))
        .expect("Gone's own decision");
    assert!(
        !erase.iter().any(|write| matches!(write, Write::Save(Record::Core(jig_domain_core::Record::Tasks(_))))),
        "the arrival writes nothing for push; Gone is a later decision"
    );
    driver.reopen(configuration());
    driver.settle();
    assert_eq!(driver.pushes.len(), 2, "standing needs are not arrivals on restart");
}
