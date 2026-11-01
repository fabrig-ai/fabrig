//! Durable reset cuts (temper/docs/design/draft/playground-reset.md, 6.2, 8;
//! temper/docs/plan/playground-reset-r4/notes/contracts.md, D4, D5).
use jig_domain_accounts as accounts;
use jig_domain_core as core;
use jig_domain_people as people;
use skein_lib::{Env, Queue, ReplyTo, Token};
use temper_engine_domain::{self as engine, Delivery, Key, Record, Write};
use temper_engine_domain_forge as forge;
use temper_engine_domain_world::{direct::Driver, fixtures, forge_stand::ForgeStand, store::Store};

// The reset fixture's journal bound and request allowance, including the next start.
const RESET_WRITES: u32 = 2_160;
const RESET_REQUESTS: u32 = 16;
const STEPS: usize = 200;
const CHAT: u64 = 1;
const FORGE: u16 = 1;

#[derive(Clone, Copy, Debug)]
enum Point {
    Decision,
    ProjectAnswer,
    ForgeDecision,
    RemovalFinish,
    AccountDecision,
    RowErasure,
    Running,
}

fn config() -> engine::Config {
    let mut config = super::playground::config(false);
    config.under_box = true;
    config
}
fn limits() -> engine::Limits {
    let mut limits = fixtures::forge_limits();
    limits.tasks.depth = 2;
    limits.accounts.accounts = 2;
    limits.fleet.grants = limits.accounts.credentials + limits.accounts.accounts;
    limits.people.act_grants = 1;
    limits.people.pending = 4;
    limits.people.requests = RESET_REQUESTS;
    limits.authority.projects = 3;
    limits.forge.judge_projects = 3;
    limits.people.projects = 3;
    limits.journal.writes = RESET_WRITES;
    limits.journal.deliveries += 4;
    limits.journal.held += 12;
    limits
}
fn apply(d: &mut Driver, number: u8, ask: forge::Ask) {
    let carried =
        (ask == forge::Ask::StartPlayground).then_some(core::Carried { number: u64::from(number), digest: [7; 32] });
    d.send(engine::Event::Party(engine::PartyInput::Application {
        reply_to: ReplyTo::new(Token::new(u64::from(number))),
        sign_in: d.session(),
        key: [number; 16],
        digest: [9; 32],
        scope: people::ApplicationScope::Deployment,
        request: engine::Application::Forge(ask),
        carried,
    }));
}
fn start(d: &mut Driver, number: u8) {
    apply(d, number, forge::Ask::StartPlayground);
    d.pump();
    let forge = d.root.playground().expect("playground offered").forge.expect("forge made");
    d.forge = Some(ForgeStand::with(&[(b"playground", b"widget", 2)]).at_forge(forge));
    d.send(engine::Event::Forge(Box::new(forge::Event::Connection {
        forge,
        state: forge::Connection::Connected { id: 1, login: b"temper".as_slice().into() },
    })));
    d.settle();
    assert!(d.delivered.iter().any(|delivery| matches!(
        delivery,
        Delivery::Application {
            outcome: Some(engine::ApplicationAnswer::StartPlayground(forge::StartAnswer::Running)),
            ..
        }
    )));
}
fn running() -> Driver {
    let mut d = Driver::cold(Store::new(), config(), &limits());
    d.forge = Some(ForgeStand::with(&[(b"playground", b"widget", 2)]));
    d.settle();
    d.sign_in();
    d.settle();
    start(&mut d, 81);
    let row = *d.root.playground().expect("running playground");
    assert_eq!(row.state, forge::PlaygroundState::Running);
    assert_eq!(row.forge, Some(FORGE));
    let people::Ask::Create { batch, .. } = d.chat_ask(1, b"hello") else { unreachable!("chat fixture") };
    d.send(engine::Event::Party(engine::PartyInput::Ask {
        reply_to: ReplyTo::new(Token::new(90)),
        sign_in: d.session(),
        key: [90; 16],
        ask: people::Ask::Create { project: row.project.expect("project"), batch },
        models: Box::new([]),
    }));
    d.pump();
    assert!(d.task(CHAT).is_some(), "chat admitted before the crash");
    d
}
fn kept(d: &Driver, act: forge::KeptAct) -> bool {
    d.store.rows.values().any(|record| {
        matches!(record,
        Record::Forge { row } if matches!(row.as_ref(), forge::Stored::Kept(row) if row.act == act))
    })
}
fn decided(writes: &[Write], project: bool) -> bool {
    writes.iter().any(|write| matches!(write,
        Write::Save(Record::Core(core::Record::People(people::Stored::Acted { act, .. })))
        if matches!((project, act), (true, people::Act::RemoveProject { .. }) | (false, people::Act::RemoveAccount { .. }))))
}
fn reached(d: &Driver, point: Point, writes: &[Write]) -> bool {
    match point {
        Point::Decision => d.store.rows.values().any(|record| {
            matches!(record,
            Record::Forge { row } if matches!(row.as_ref(), forge::Stored::Playground(row)
            if row.state == forge::PlaygroundState::Resetting))
        }),
        // The answer and connector continuation share a root decision. Cut at
        // that decision's commit, without manufacturing a store boundary.
        Point::ProjectAnswer => decided(writes, true),
        Point::ForgeDecision => {
            kept(d, forge::KeptAct::RemoveForge { forge: FORGE })
                && kept(d, forge::KeptAct::BoxRemove { forge: FORGE })
                && d.store.rows.values().any(|record| {
                    matches!(record,
                    Record::Forge { row } if matches!(row.as_ref(), forge::Stored::Forge(row)
                    if row.number == FORGE && row.removing))
                })
        }
        Point::RemovalFinish => writes.contains(&Write::Erase(Key::Forge(forge::Key::Forge(FORGE)))),
        Point::AccountDecision => decided(writes, false),
        Point::RowErasure => writes.contains(&Write::Erase(Key::Forge(forge::Key::Playground))),
        Point::Running => unreachable!("running cut needs no reset"),
    }
}
fn answer_box(d: &mut Driver) {
    let requests = std::mem::take(&mut d.immediate);
    for request in requests {
        if let engine::Request::BoxRemove { forge } = request {
            d.send(engine::Event::Forge(Box::new(forge::Event::BoxRemoved { forge, removed: true })));
        }
    }
}
fn finish(d: &mut Driver) {
    for _ in 0..STEPS {
        answer_box(d);
        d.advance(true);
        if d.root.ready() && d.root.playground().is_none() && d.events.is_empty() && d.store.pending.is_empty() {
            d.settle();
            return;
        }
    }
    unreachable!("reset finishes within the direct world's step bound");
}
fn durable(d: &Driver) -> Vec<Vec<Write>> {
    let count = usize::try_from(d.store.applied).expect("small world commit count");
    // This story uses only root commits, never fixture seeds.
    assert!(count <= d.transactions.len());
    d.transactions[..count].to_vec()
}
fn cut_at(point: Point) -> Driver {
    let mut d = running();
    if !matches!(point, Point::Running) {
        apply(&mut d, 91, forge::Ask::EndPlayground);
        let mut cut = false;
        for _ in 0..STEPS {
            let before = d.store.applied;
            d.advance(true);
            if d.store.applied != before {
                let committed = durable(&d);
                if reached(&d, point, committed.last().expect("reset commit")) {
                    cut = true;
                    break;
                }
            }
            answer_box(&mut d);
        }
        assert!(cut, "durable cut reached: {point:?}");
    }
    let mut transactions = durable(&d);
    let mut restarted = Driver::cold(d.store.fork(), config(), &d.env.limits);
    restarted.forge = d.forge;
    finish(&mut restarted);
    transactions.extend(durable_after_restart(&restarted));
    assert_decided_once(&restarted, &transactions);
    restarted.sign_in();
    restarted.settle();
    restarted.send(engine::Event::Party(engine::PartyInput::Read {
        reply_to: ReplyTo::new(Token::new(92)),
        sign_in: restarted.session(),
        read: core::reads::Read::Frame,
    }));
    restarted.pump();
    restarted
}
fn durable_after_restart(d: &Driver) -> Vec<Vec<Write>> {
    let pending = d.store.pending.len();
    d.transactions[..d.transactions.len() - pending].to_vec()
}
fn assert_decided_once(d: &Driver, transactions: &[Vec<Write>]) {
    for project in [true, false] {
        let decisions: Vec<_> = transactions
            .iter()
            .flatten()
            .filter_map(|write| {
                if let Write::Save(Record::Core(core::Record::People(row @ people::Stored::Acted { act, .. }))) = write
                    && matches!(
                        (project, act),
                        (true, people::Act::RemoveProject { .. }) | (false, people::Act::RemoveAccount { .. })
                    )
                {
                    Some(row)
                } else {
                    None
                }
            })
            .collect();
        assert_eq!(decisions.len(), 1, "one durable deletion decision; project={project}");
        assert_replayed(d, decisions[0]);
    }
}
fn assert_replayed(d: &Driver, row: &people::Stored) {
    let env = Env { now: d.env.now, wall: d.env.wall, limits: d.env.limits.people };
    let mut people = people::Domain::new(&env.limits, Box::new([]), 1, Box::new([]));
    let mut out = Queue::uncapped();
    people::step(&mut people, &env, people::Event::Restore { record: row.clone() }, &mut out);
    people::step(&mut people, &env, people::Event::Restored, &mut out);
    out.clear();
    let people::Stored::Acted { key, act, outcome, .. } = row else { unreachable!("saved act") };
    people::step(
        &mut people,
        &env,
        people::Event::Act {
            reply_to: ReplyTo::new(Token::new(93)),
            connector: key.connector,
            key: key.key,
            act: Box::new(act.clone()),
        },
        &mut out,
    );
    assert_eq!(out.len(), 1, "retained answer emits no second deletion, save or erase");
    assert!(matches!(out.pop(), Some(people::Request::Acted { outcome: answer, .. }) if answer == *outcome));
}
fn assert_reset(d: &Driver) {
    assert!(d.root.ready());
    assert_eq!(d.root.playground(), None);
    assert!(!d.store.rows.values().any(|record| matches!(record,
        Record::Forge { row } if matches!(row.as_ref(), forge::Stored::Playground(_) | forge::Stored::Kept(_)))));
    assert!(!d.store.rows.contains_key(&Key::Forge(forge::Key::Forge(FORGE))));
    assert!(!d.store.rows.values().any(|record| matches!(
        record,
        Record::Core(core::Record::Accounts(accounts::Stored::Account {
            holder: accounts::Holder::Deployment,
            provider: 1,
            ..
        }))
    )));
    assert!(!d.store.rows.values().any(|record| matches!(record,
        Record::Core(core::Record::People(people::Stored::Policy { name, .. })) if name.as_ref() == "Playground")));
    assert!(d.task(CHAT).is_none());
    assert!(d.root.tasks().view_tasks().iter().all(|row| row.number != CHAT));
    let frame = d
        .delivered
        .iter()
        .find_map(|delivery| {
            if let Delivery::Read { reply, answer: core::reads::Answer::Document(document) } = delivery
                && *reply == ReplyTo::new(Token::new(92))
                && let core::documents::Document::Frame(frame) = document.as_ref()
            {
                Some(frame)
            } else {
                None
            }
        })
        .expect("me after reset");
    assert!(frame.projects.iter().all(|project| project.name.bytes.as_ref() != b"Playground"));
}
fn check(point: Point) {
    let mut d = cut_at(point);
    assert_reset(&d);
    start(&mut d, 82);
    let row = d.root.playground().expect("next playground offered");
    assert_eq!(row.state, forge::PlaygroundState::Running);
    assert!(row.forge.expect("next forge") > FORGE);
}

#[test]
fn a_reset_cut_after_its_decision_ends_whole() {
    check(Point::Decision);
}
#[test]
fn a_reset_cut_after_the_project_answer_ends_whole() {
    check(Point::ProjectAnswer);
}
#[test]
fn a_reset_cut_after_the_forge_decision_ends_whole() {
    check(Point::ForgeDecision);
}
#[test]
fn a_reset_cut_after_the_removal_finish_ends_whole() {
    check(Point::RemovalFinish);
}
#[test]
fn a_reset_cut_after_the_account_decision_ends_whole() {
    check(Point::AccountDecision);
}
#[test]
fn a_reset_cut_after_the_row_erasure_ends_whole() {
    check(Point::RowErasure);
}
#[test]
fn a_running_playground_found_at_a_start_is_reset() {
    check(Point::Running);
}
