//! Crash cuts through forge removal (temper/docs/design/domain/forge.md, 4.1 and 18).
use jig_domain_core as core;
use jig_domain_core::documents::{Watched, Window};
use jig_domain_people as people;
use jig_domain_tasks as tasks;
use skein_forge_client::{Condition, Effect, Entry};
use skein_lib::{Duration, ReplyTo, Token};
use temper_engine_domain::{self as engine, Delivery, Key, Record};
use temper_engine_domain_forge as forge;
use temper_engine_domain_world::{
    direct::Driver, forge_routes::change_world, forge_stand::ForgeStand, protocol, walking,
};

const FORGE: u16 = 1;
const ENTRY: u64 = 100;
const STEPS: usize = 400;

#[derive(Clone, Copy, Debug)]
enum Cut {
    Decision,
    SecretsErased,
    BoxRemoved,
    TaskEnded,
    ChatErased,
}

fn session(d: &Driver) -> u64 {
    d.store
        .rows
        .values()
        .find_map(|record| {
            if let Record::Core(core::Record::People(people::Stored::SignIn { number, .. })) = record {
                Some(*number)
            } else {
                None
            }
        })
        .expect("durable owner session")
}

fn application(d: &mut Driver, number: u8, ask: forge::Ask, carried: bool) {
    d.send(engine::Event::Party(engine::PartyInput::Application {
        reply_to: ReplyTo::new(Token::new(u64::from(number))),
        sign_in: session(d),
        key: [number; 16],
        digest: [number; 32],
        scope: people::ApplicationScope::Deployment,
        request: engine::Application::Forge(ask),
        carried: carried.then_some(core::Carried { number: u64::from(number), digest: [number; 32] }),
    }));
}

fn forge_rows(d: &Driver) -> Vec<&forge::Stored> {
    d.store
        .rows
        .values()
        .filter_map(|record| if let Record::Forge { row } = record { Some(row.as_ref()) } else { None })
        .collect()
}

fn reconnect(d: &mut Driver) {
    let hosting = d
        .store
        .rows
        .values()
        .filter_map(|record| {
            let Record::Core(core::Record::Tasks(tasks::Stored::Live(row))) = record else { return None };
            let attempt =
                if let tasks::Phase::Active(tasks::Active::Running { attempt } | tasks::Active::Claimed { attempt }) =
                    row.phase
                {
                    attempt
                } else if let tasks::Phase::Closing(tasks::Closing { stage: tasks::Stage::Run { attempt }, .. }) =
                    row.phase
                {
                    attempt
                } else {
                    return None;
                };
            Some(protocol::hosting(row.number, attempt, jig_domain_host::Phase::Active))
        })
        .collect();
    d.send(protocol::hello_slots(Token::new(7), 2, hosting));
    d.pump();
}

fn stop_cancelled_runs(d: &mut Driver) {
    for _ in 0..4 {
        let closing: Vec<_> = d
            .store
            .rows
            .values()
            .filter_map(|record| {
                if let Record::Core(core::Record::Tasks(tasks::Stored::Live(row))) = record
                    && let tasks::Phase::Closing(tasks::Closing { stage: tasks::Stage::Run { attempt }, .. }) =
                        row.phase
                {
                    Some((row.number, attempt))
                } else {
                    None
                }
            })
            .collect();
        if closing.is_empty() {
            return;
        }
        for (task, attempt) in closing {
            d.answer(Token::new(7), task, attempt, 0, tasks::End::Parked, engine::Parts::none());
        }
        d.pump();
    }
}

fn crash(d: &mut Driver, seed: u64) {
    d.store.crash(seed);
    let mut config = walking::config(71);
    config.forges = Box::new([]);
    config.identities = Box::new([]);
    config.forge.writers = Box::new([]);
    d.replace_root(config);
    d.events.clear();
    d.out.clear();
    d.delivered.clear();
    d.immediate.clear();
    d.forge = Some(ForgeStand::new());
    d.send(engine::Event::Restart { pending: d.store.pending_generation });
    d.pump();
    assert!(d.root.ready() && !d.stopped, "restored removal: {:?}", d.root.start_stop());
    reconnect(d);
}

fn fixture() -> (Driver, u64, u64) {
    let (mut world, chat, producer, _) = change_world(false, 1000, None);
    for _ in 0..20 {
        world.tick();
    }
    let Record::Core(core::Record::Tasks(tasks::Stored::Live(row))) =
        &world.store.rows[&Key::Core(core::Key::Tasks(tasks::Key::Live(producer.task)))]
    else {
        panic!("live producer")
    };
    let tasks::Party::Task(change) = row.requester else { panic!("change producer") };
    let mut d = Driver::cold(world.store, walking::config(71), &world.env.limits);
    d.env = world.env;
    d.root = world.root;
    d.protocol = world.protocol;
    d.period_sent = true;
    d.forge = Some(ForgeStand::new());
    let record = forge::ForgeRecord {
        git_program: None,
        number: FORGE,
        kind: forge::Kind::Forgejo,
        mode: forge::Mode::Owned,
        state: forge::ForgeState::Kept,
        place: forge::Place {
            api: forge::Address { host: b"forge.example".as_slice().into(), port: 3000, tls: None },
            git: b"http://forge.example:3000".as_slice().into(),
            web: None,
        },
        read_only: false,
        bot: forge::Bot { login: b"temper".as_slice().into(), name: b"Temper".as_slice().into(), email: Box::new([]) },
        asked: true,
        first: None,
    };
    d.store.seed(
        Box::new([
            Record::Forge { row: Box::new(forge::Stored::Forge(record)) },
            Record::Forge { row: Box::new(forge::Stored::ForgeNumber { last: FORGE }) },
            Record::Forge {
                row: Box::new(forge::Stored::Writer(skein_forge_client::Writer { forge: FORGE, author: 1 })),
            },
        ]),
        Box::new([]),
    );
    crash(&mut d, 71);
    d.send(protocol::hello_slots(
        Token::new(7),
        2,
        Box::new([
            protocol::hosting(chat.task, chat.attempt, jig_domain_host::Phase::Active),
            protocol::hosting(producer.task, producer.attempt, jig_domain_host::Phase::Active),
        ]),
    ));
    d.pump();
    // Keep the terminal in the old process while the durable flight is present.
    d.send(engine::Event::Forge(Box::new(forge::Event::Enqueue {
        entry: Entry {
            number: ENTRY,
            task: change,
            repository: skein_forge::api::Repository { forge: FORGE, repository: 2 },
            effect: Effect { write: skein_forge::api::Write::Close { number: 1 }, condition: Condition::None },
            start: None,
            attempt: None,
            failures: 0,
            queued: false,
        },
    })));
    for _ in 0..STEPS {
        if forge_rows(&d).iter().any(|row| matches!(row, forge::Stored::Flight { entry: ENTRY, .. })) {
            break;
        }
        d.advance(true);
    }
    assert!(forge_rows(&d).iter().any(|row| matches!(row, forge::Stored::Writer(_))));
    assert!(forge_rows(&d).iter().any(|row| matches!(row, forge::Stored::Flight { entry: ENTRY, .. })));
    assert!(
        forge_rows(&d).iter().any(|row| matches!(row, forge::Stored::Entry { entry, .. } if entry.number == ENTRY))
    );
    assert!(d.task(change).is_some());
    (d, chat.task, change)
}

fn commit(d: &mut Driver) {
    // Lose the commit terminal, as well as all outputs still in the process.
    assert!(!d.store.pending.is_empty(), "the cut follows a kept decision");
    while !d.store.pending.is_empty() {
        d.store.apply();
    }
}

fn removed(d: &mut Driver, event: forge::Event) {
    d.send(engine::Event::Forge(Box::new(event)));
    d.pump();
}

fn assert_chat_keeps_name(d: &mut Driver, chat: u64, change: u64) {
    let row = d.task(chat).expect("chat remains open");
    assert!(matches!(row.phase, tasks::Phase::Active(_)));
    assert!(forge_rows(d).iter().any(|row| matches!(row, forge::Stored::Repository(row)
        if row.provider.forge == FORGE && row.owner.as_ref() == b"org" && row.name.as_ref() == b"repo")));
    assert!(d.root.forges().rows.is_empty());
    assert!(!forge_rows(d).iter().any(|row| matches!(
        row,
        forge::Stored::Writer(_) | forge::Stored::Flight { .. } | forge::Stored::Entry { .. }
    )));
    assert_name_reads(d, change);
}

fn assert_name_reads(d: &mut Driver, change: u64) {
    d.delivered.clear();
    let watcher = Token::new(301);
    d.send(engine::Event::Party(engine::PartyInput::Watch {
        watcher,
        sign_in: session(d),
        subject: Watched::Application {
            scope: people::ApplicationScope::Project(1),
            connector: 0,
            subject: Token::new(change),
        },
        window: Window { rows: 1, after: None },
    }));
    d.pump();
    assert!(
        d.delivered.iter().any(|delivery| matches!(delivery, Delivery::Panel { watcher: got, panel }
        if *got == watcher && panel.owner.as_ref() == b"org" && panel.name.as_ref() == b"repo")),
        "retained-name watch: {:?}",
        d.immediate
    );
    d.send(engine::Event::Party(engine::PartyInput::Unwatch { watcher }));
    d.send(engine::Event::Party(engine::PartyInput::Delivered { watcher }));
    d.pump();
}

fn end_change(d: &mut Driver, change: u64) {
    stop_cancelled_runs(d);
    assert!(
        d.store.rows.values().any(|record| matches!(record,
        Record::Core(core::Record::Tasks(tasks::Stored::Ended(row))) if row.number == change
            && matches!(row.phase, tasks::Phase::Ended(tasks::Ending::Cancelled { .. })))),
        "change: {:?}",
        d.task(change)
    );
}

fn erase_chat(d: &mut Driver, chat: u64) {
    d.send(engine::Event::Party(engine::PartyInput::Ask {
        models: Box::new([]),
        reply_to: ReplyTo::new(Token::new(202)),
        sign_in: session(d),
        key: [202; 16],
        ask: people::Ask::RemoveProject { project: 1, grace: None },
    }));
    d.pump();
    d.fire_at(Duration::from_nanos(d.env.now.saturating_add(Duration::from_secs(5)).as_nanos()));
    d.pump();
    stop_cancelled_runs(d);
    assert!(!d.store.rows.contains_key(&Key::Core(core::Key::Tasks(tasks::Key::Live(chat)))));
    assert!(!d.store.rows.contains_key(&Key::Core(core::Key::Tasks(tasks::Key::Ended(chat)))));
}

fn assert_whole(d: &Driver) {
    assert!(d.root.forges().rows.is_empty());
    for row in forge_rows(d) {
        assert!(
            !matches!(row,
            forge::Stored::Forge(record) if record.number == FORGE),
            "forge record erased"
        );
        assert!(
            !matches!(row, forge::Stored::Writer(_) | forge::Stored::Flight { .. } | forge::Stored::Entry { .. }),
            "writer and outbox erased: {row:?}"
        );
        assert!(!matches!(row, forge::Stored::Repository(row) if row.provider.forge == FORGE), "repository erased");
    }
}

fn run(cut: Cut, seed: u64) -> std::collections::BTreeMap<Key, Record> {
    let (mut d, chat, change) = fixture();
    application(&mut d, 201, forge::Ask::RemoveForge { forge: FORGE }, false);
    commit(&mut d);
    if matches!(cut, Cut::Decision) {
        crash(&mut d, seed);
    }
    d.pump();
    assert_chat_keeps_name(&mut d, chat, change);
    removed(&mut d, forge::Event::SecretsErased { forge: FORGE });
    if matches!(cut, Cut::SecretsErased) {
        crash(&mut d, seed);
    }
    removed(&mut d, forge::Event::BoxRemoved { forge: FORGE });
    if matches!(cut, Cut::BoxRemoved) {
        crash(&mut d, seed);
    }
    assert_chat_keeps_name(&mut d, chat, change);
    end_change(&mut d, change);
    if matches!(cut, Cut::TaskEnded) {
        crash(&mut d, seed);
    }
    assert_chat_keeps_name(&mut d, chat, change);
    erase_chat(&mut d, chat);
    if matches!(cut, Cut::ChatErased) {
        crash(&mut d, seed);
    }
    assert_whole(&d);
    make_forge(&mut d, 203);
    assert!(forge_rows(&d).iter().any(|row| matches!(row, forge::Stored::Forge(row) if row.number > FORGE)));
    d.store.rows.clone()
}

fn make_forge(d: &mut Driver, number: u8) {
    application(
        d,
        number,
        forge::Ask::Connect {
            forge: forge::Making::New(forge::ForgeAsk::Managed {
                git_program: None,
                place: forge::Place {
                    api: forge::Address { host: b"other.example".as_slice().into(), port: 3000, tls: None },
                    git: b"http://other.example:3000".as_slice().into(),
                    web: None,
                },
            }),
            how: forge::How::ForgejoAdmin { first: None },
        },
        true,
    );
    d.pump();
    assert!(!d.delivered.iter().any(|delivery| matches!(delivery, Delivery::Application { outcome: None, .. })));
}

fn replay(cut: Cut) {
    for seed in [1, 7, 71] {
        assert_eq!(run(cut, seed), run(cut, seed), "cut {cut:?}, seed {seed}");
    }
}

#[test]
fn removal_cut_after_decision_ends_whole() {
    replay(Cut::Decision);
}
#[test]
fn removal_cut_after_secrets_erased_ends_whole() {
    replay(Cut::SecretsErased);
}
#[test]
fn removal_cut_after_box_removed_ends_whole() {
    replay(Cut::BoxRemoved);
}
#[test]
fn removal_cut_after_task_ended_ends_whole() {
    replay(Cut::TaskEnded);
}
#[test]
fn removal_cut_after_chat_erased_ends_whole() {
    replay(Cut::ChatErased);
}
#[test]
fn a_removed_forges_number_is_never_given_again() {
    let mut d = Driver::forge();
    d.pump();
    d.sign_in();
    d.pump();
    make_forge(&mut d, 201);
    let number = forge_rows(&d)
        .iter()
        .find_map(|row| if let forge::Stored::Forge(row) = row { Some(row.number) } else { None })
        .expect("new forge record");
    application(&mut d, 202, forge::Ask::RemoveForge { forge: number }, false);
    d.pump();
    removed(&mut d, forge::Event::SecretsErased { forge: number });
    assert!(!d.store.rows.contains_key(&Key::Forge(forge::Key::Forge(number))));
    d.store.crash(91);
    d.root = engine::Domain::new(walking::config(71), &d.env.limits).expect("number fixture");
    d.events.clear();
    d.send(engine::Event::Restart { pending: false });
    d.pump();
    make_forge(&mut d, 203);
    assert!(forge_rows(&d).iter().any(|row| matches!(row, forge::Stored::Forge(row) if row.number > number)));
}

#[test]
fn an_ended_chat_reads_its_removed_forges_repository_until_its_project_is_deleted() {
    let (mut d, chat, change) = fixture();
    application(&mut d, 201, forge::Ask::RemoveForge { forge: FORGE }, false);
    d.pump();
    removed(&mut d, forge::Event::SecretsErased { forge: FORGE });
    removed(&mut d, forge::Event::BoxRemoved { forge: FORGE });
    end_change(&mut d, change);
    d.send(engine::Event::Party(engine::PartyInput::Ask {
        models: Box::new([]),
        reply_to: ReplyTo::new(Token::new(204)),
        sign_in: session(&d),
        key: [204; 16],
        ask: people::Ask::Cancel { project: 1, task: chat, reason: b"close the chat".as_slice().into(), grace: None },
    }));
    d.pump();
    stop_cancelled_runs(&mut d);
    assert!(d.store.rows.contains_key(&Key::Core(core::Key::Tasks(tasks::Key::Ended(chat)))));
    assert!(!d.store.rows.contains_key(&Key::Core(core::Key::Tasks(tasks::Key::Live(chat)))));
    let names = forge_rows(&d)
        .into_iter()
        .find_map(|row| {
            if let forge::Stored::Names { task, resources } = row
                && *task == change
            {
                Some(resources.as_ref())
            } else {
                None
            }
        })
        .expect("the change keeps its repository name");
    assert_eq!(names, &[forge::Name { forge: FORGE, repository: 2, what: forge::What::Repository }]);
    assert_name_reads(&mut d, change);
    crash(&mut d, 71);
    assert_name_reads(&mut d, change);
    erase_chat(&mut d, chat);
    assert_whole(&d);
    assert!(!forge_rows(&d).iter().any(|row| matches!(row,
        forge::Stored::Names { task, .. } if *task == chat || *task == change)));
    crash(&mut d, 7);
}
