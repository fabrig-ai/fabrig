//! Playground reset through the engine (draft 6.2; contracts D4, D5).
use jig_domain_accounts as accounts;
use jig_domain_core as core;
use jig_domain_people as people;
use skein_lib::{ReplyTo, Token};
use temper_engine_domain::{self as engine, Delivery};
use temper_engine_domain_forge as forge;
use temper_engine_domain_world::{direct::Driver, fixtures, forge_stand::ForgeStand, store::Store};

// The playground start fixture's journal bound, including project and account writes.
const RESET_WRITES: u32 = 2_160;
// Two starts' five requests each, two resets' two acts each, a chat and an end.
const RESET_REQUESTS: u32 = 16;
fn config(granted: bool) -> engine::Config {
    let mut config = super::playground::config(false);
    if !granted {
        config.core.grants[0].may = Box::new([people::ActKind::Cancel, people::ActKind::RemoveAccount]);
    }
    config
}
fn signed_in(granted: bool) -> (Driver, u64) {
    let mut limits = fixtures::forge_limits();
    limits.tasks.depth = 2;
    limits.accounts.accounts = 2;
    limits.fleet.grants = limits.accounts.credentials + limits.accounts.accounts;
    limits.people.act_grants = 1;
    limits.people.pending = 4;
    limits.people.requests = RESET_REQUESTS;
    // The seeded project and two successive playground project numbers.
    limits.authority.projects = 3;
    limits.forge.judge_projects = 3;
    limits.people.projects = 3;
    limits.journal.writes = RESET_WRITES;
    limits.journal.deliveries += 4;
    limits.journal.held += 12;
    let mut d = Driver::cold(Store::new(), config(granted), &limits);
    d.forge = Some(ForgeStand::with(&[(b"playground", b"widget", 2)]));
    d.settle();
    d.sign_in();
    d.settle();
    let sign_in = d.session();
    (d, sign_in)
}
fn driver(granted: bool) -> (Driver, u64) {
    let (mut d, sign_in) = signed_in(granted);
    start(&mut d, sign_in, 81);
    (d, sign_in)
}
fn apply(d: &mut Driver, sign_in: u64, number: u8, ask: forge::Ask, carried: Option<core::Carried>) {
    d.send(engine::Event::Party(engine::PartyInput::Application {
        reply_to: ReplyTo::new(Token::new(u64::from(number))),
        sign_in,
        key: [number; 16],
        digest: [9; 32],
        scope: people::ApplicationScope::Deployment,
        request: engine::Application::Forge(ask),
        carried,
    }));
    d.pump();
}
fn start(d: &mut Driver, sign_in: u64, number: u8) {
    apply(
        d,
        sign_in,
        number,
        forge::Ask::StartPlayground,
        Some(core::Carried { number: u64::from(number), digest: [7; 32] }),
    );
    let number = d.root.playground().expect("playground").forge.expect("made forge");
    d.forge = Some(ForgeStand::with(&[(b"playground", b"widget", 2)]).at_forge(number));
    d.send(engine::Event::Forge(Box::new(forge::Event::Connection {
        forge: number,
        state: forge::Connection::Connected { id: 1, login: b"temper".as_slice().into() },
    })));
    d.settle();
}
fn end(d: &mut Driver, sign_in: u64) {
    d.delivered.clear();
    apply(d, sign_in, 91, forge::Ask::EndPlayground, None);
}
fn reset_answered(d: &Driver) -> bool {
    d.delivered.iter().any(|delivery| {
        matches!(
            delivery,
            Delivery::Application {
                outcome: Some(engine::ApplicationAnswer::EndPlayground(forge::EndAnswer::Reset)),
                ..
            }
        )
    })
}
fn assert_erased(d: &Driver, number: u16) {
    assert_eq!(d.root.playground(), None);
    assert!(!d.store.rows.values().any(|record| matches!(record,
        engine::Record::Forge { row } if matches!(row.as_ref(), forge::Stored::Playground(_) | forge::Stored::Kept(_)))));
    assert!(!d.store.rows.values().any(|record| matches!(record,
        engine::Record::Forge { row } if matches!(row.as_ref(), forge::Stored::Forge(row) if row.number == number))));
    assert!(!d.store.rows.values().any(|record| matches!(
        record,
        engine::Record::Core(core::Record::Accounts(accounts::Stored::Account {
            holder: accounts::Holder::Deployment,
            provider: 1,
            ..
        }))
    )));
    assert!(!d.store.rows.values().any(|record| matches!(record,
        engine::Record::Core(core::Record::People(people::Stored::Policy { name, .. })) if name.as_ref() == "Playground")));
}
#[test]
fn a_reset_deletes_the_project_removes_the_forge_and_the_account() {
    let (mut d, sign_in) = driver(true);
    let row = *d.root.playground().expect("running playground");
    let people::Ask::Create { batch, .. } = d.chat_ask(1, b"hello") else { unreachable!("chat fixture") };
    let ask = people::Ask::Create { project: row.project.expect("project"), batch };
    d.send(engine::Event::Party(engine::PartyInput::Ask {
        reply_to: ReplyTo::new(Token::new(90)),
        sign_in,
        key: [90; 16],
        ask,
        models: Box::new([]),
    }));
    d.pump();
    let task = d
        .delivered
        .iter()
        .find_map(|delivery| {
            if let Delivery::WebReply {
                reply: people::Reply::Outcome(people::Outcome::Created { first, .. }), ..
            } = delivery
            {
                Some(*first)
            } else {
                None
            }
        })
        .expect("chat admitted");
    end(&mut d, sign_in);
    d.settle();
    assert!(reset_answered(&d), "playground: {:?}; deliveries: {:?}", d.root.playground(), d.delivered);
    assert_erased(&d, row.forge.expect("forge"));
    assert!(!d.root.tasks().view_tasks().iter().any(|row| row.number == task));
    assert!(d.task(task).is_none());
    d.send(engine::Event::Party(engine::PartyInput::Read {
        reply_to: ReplyTo::new(Token::new(92)),
        sign_in,
        read: core::reads::Read::Frame,
    }));
    d.pump();
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
#[test]
fn the_playground_starts_again_after_its_reset_on_a_new_forge() {
    let (mut d, sign_in) = driver(true);
    let removed = d.root.playground().expect("running playground").forge.expect("forge");
    end(&mut d, sign_in);
    d.settle();
    d.delivered.clear();
    start(&mut d, sign_in, 82);
    let row = d.root.playground().expect("new playground");
    assert!(row.forge.expect("new forge") > removed);
    assert_eq!(row.state, forge::PlaygroundState::Running);
    assert!(d.delivered.iter().any(|delivery| matches!(
        delivery,
        Delivery::Application {
            outcome: Some(engine::ApplicationAnswer::StartPlayground(forge::StartAnswer::Running)),
            ..
        }
    )));
}
#[test]
fn an_ungranted_remove_project_keeps_the_reset_waiting() {
    let (mut d, sign_in) = driver(false);
    let row = *d.root.playground().expect("running playground");
    end(&mut d, sign_in);
    assert!(!reset_answered(&d));
    assert_eq!(d.root.playground().expect("resetting playground").state, forge::PlaygroundState::Resetting);
    assert!(d.store.rows.values().any(|record| matches!(record,
        engine::Record::Forge { row: stored } if matches!(stored.as_ref(), forge::Stored::Kept(kept)
        if kept.act == (forge::KeptAct::RemoveProject { project: row.project.expect("project") })))));
    assert!(d.delivered.iter().any(|delivery| matches!(delivery,
        Delivery::DemoNote { line } if line.windows(b"remove_project (engine.acts)".len())
            .any(|part| part == b"remove_project (engine.acts)"))));
    d.root = engine::Domain::new(config(true), &d.env.limits).expect("restart config");
    d.events.clear();
    d.delivered.clear();
    d.send(engine::Event::Restart { pending: false });
    d.settle();
    assert_erased(&d, row.forge.expect("forge"));
    assert!(!reset_answered(&d), "held requests are volatile across restart");
    d.sign_in();
    d.settle();
    let sign_in = d.session();
    end(&mut d, sign_in);
    assert!(reset_answered(&d));
}

#[test]
fn a_reset_with_a_start_waiting_for_connection_drops_its_route() {
    let (mut d, sign_in) = signed_in(true);
    apply(&mut d, sign_in, 81, forge::Ask::StartPlayground, Some(core::Carried { number: 81, digest: [7; 32] }));
    end(&mut d, sign_in);
    d.settle();
    assert!(d.delivered.iter().any(|delivery| matches!(
        delivery,
        Delivery::Application {
            outcome: Some(engine::ApplicationAnswer::StartPlayground(forge::StartAnswer::NotOffered)),
            ..
        }
    )));
    assert!(reset_answered(&d));
    assert_erased(&d, 1);
    d.delivered.clear();
    d.fire_at(skein_lib::Duration::from_secs(10));
    d.settle();
    assert!(!d.delivered.iter().any(|delivery| matches!(delivery, Delivery::Application { .. })));
}
