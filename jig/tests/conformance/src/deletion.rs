//! Store-family deletion stories (temper/docs/design/draft/playground-reset.md, 5 and 8).
use crate::{Config, Testing, actions::World};
use jig_conformance::{Cut, Harness};
use jig_domain_accounts as accounts;
use jig_domain_core as core;
use jig_domain_notes as notes;
use jig_domain_people as people;
use jig_domain_tasks as tasks;
use jig_test_connector as connector;
use jig_test_domain as root;
use skein_lib::{List, ReplyTo, Token};

/// Each durable store owner is exercised with its own project deletion.
#[derive(Clone, Copy, Debug)]
pub enum Family { Core, Tasks, People, Notes, Accounts, Application }

/// Every owner participating in the deletion contract.
pub const FAMILIES: [Family; 6] = [Family::Core, Family::Tasks, Family::People, Family::Notes, Family::Accounts, Family::Application];

fn drain(world: &mut World) {
    world.drain().unwrap_or_else(|error| panic!("deletion story: {error:?}"));
}
fn ask(world: &mut World, key: u8, ask: people::Ask) {
    world.send(root::Event::Core(core::Event::People(people::Event::Ask {
        models: Box::new([]),
        reply_to: ReplyTo::new(Token::new(9000 + u64::from(key))),
        sign_in: world.peers.peers.parties[0].sign_in.expect("owner signed in"),
        key: [key; 16], ask,
    })));
    drain(world);
}
fn act(world: &mut World, key: u8, act: people::Act) {
    world.send(root::Event::Core(core::Event::Act {
        reply_to: ReplyTo::new(Token::new(9500 + u64::from(key))),
        connector: 1, key: [key; 16], act: Box::new(act), carried: None,
    }));
    drain(world);
}
fn row(world: &World, task: u64) -> &tasks::TaskRecord {
    world.store.rows.values().find_map(|record| {
        if let root::Record::Core(core::Record::Tasks(tasks::Stored::Live(row) | tasks::Stored::Ended(row))) = record
            && row.number == task { Some(row.as_ref()) } else { None }
    }).expect("story task retained")
}
fn ready(world: &mut World) {
    drain(world);
    if world.peers.assignments.is_empty() {
        world.advance(6_000_000_000);
        drain(world);
        world.advance(1_000_000_000);
        drain(world);
    }
    assert!(!world.peers.assignments.is_empty(), "initial chat assigned");
}
fn adoption(world: &mut World) {
    world.send(root::Event::Connector { number: 1, event: connector::Event::Adopt {
        project: 1, resource: jig_test_connector_world::path(1, 1), role: connector::ResourceRole::Participating,
    }});
    drain(world);
    assert!(world.store.rows.keys().any(|key| matches!(key, root::Key::Connector { key: connector::RecordKey::Adoption { project: 1, .. }, .. })));
}
fn task_rows(world: &mut World) {
    let (task, attempt) = world.peers.assignments[0];
    let mut authority = row(world, task).authority.clone();
    authority.budget.spend = 10;
    authority.delegation = tasks::Delegation { kinds: Box::new([]), tasks: 0, depth: 0 };
    world.send(root::Event::Core(core::Event::DelegateValidated {
        to: ReplyTo::new(Token::new(9800)),
        key: core::CallKey { task, attempt, completion: 1, position: 0 },
        batch: Box::new([core::Delegate {
            deadline: tasks::Deadline::Named, label: 0, flags: tasks::Flags::PLAIN, tracked: None,
            budget: people::Budget::Named, executor: tasks::Executor::Agent { charter: 1 },
            spec: tasks::Spec { asks: false, words: b"delegate".as_slice().into(), parameters: Box::new([]), inputs: Box::new([]) },
            contract: tasks::Contract::Report { words: 128 }, authority, llm: None,
            symbolic_grants: Box::new([]), dependencies: Box::new([]), wake: tasks::WakePolicy::DEFAULT,
        }]), stubs: Box::new([]),
    }));
    drain(world);
    let child = world.store.rows.values().find_map(|record| {
        if let root::Record::Core(core::Record::Tasks(tasks::Stored::Live(row))) = record
            && row.number != task { Some(row.number) } else { None }
    }).expect("delegate created");
    ask(world, 40, people::Ask::Cancel { project: 1, task: child, reason: b"done".as_slice().into(), grace: None });
    ask(world, 41, people::Ask::SetPool { project: 1, person: 1, budget: 500 });
    ask(world, 42, people::Ask::Stop { project: 1, task });
    assert!(world.store.rows.keys().any(|key| matches!(key, root::Key::Core(core::Key::Tasks(tasks::Key::Ended(_))))));
    assert!(world.store.rows.keys().any(|key| matches!(key, root::Key::Core(core::Key::Tasks(tasks::Key::PlacedHold(_))))));
}
fn note_rows(world: &mut World) {
    let (task, attempt) = world.peers.assignments[0];
    world.send(root::Event::Core(core::Event::NamedAction {
        to: ReplyTo::new(Token::new(9801)),
        key: core::CallKey { task, attempt, completion: 1, position: 0 },
        action: core::NamedAction::Note { entry: notes::New {
            name: 0, scope: notes::Scope::Project { project: 1 }, description: b"project note".as_slice().into(),
            body: b"kept learning".as_slice().into(), references: List::with_capacity(1), author: notes::Author::Task { task, attempt },
        }, recalled: None },
    }));
    drain(world);
    assert!(world.store.rows.keys().any(|key| matches!(key, root::Key::Core(core::Key::Notes(notes::Key::Entry { .. })))));
    assert!(world.store.rows.keys().any(|key| matches!(key, root::Key::Core(core::Key::Notes(notes::Key::Line { .. })))));
}

/// Build the family's rows through admitted engine requests before arming deletion cuts.
#[must_use]
pub fn prepared(family: Family, seed: u64) -> World {
    let mut world = Harness::<Testing>::new(Config::new(seed, false, 2, None).deleting(), seed);
    ready(&mut world);
    match family {
        Family::Core => {
            crate::actions::say(&mut world, 30);
            drain(&mut world);
            assert!(world.store.rows.keys().any(|key| matches!(key, root::Key::Core(core::Key::Core(core::CoreKey::Turn { .. })))));
        }
        Family::Tasks => task_rows(&mut world),
        Family::People => {
            let task = world.peers.assignments[0].0;
            ask(&mut world, 31, people::Ask::MarkSeen { seen: people::Seen::Conversation { task, through: 1 } });
            adoption(&mut world);
        }
        Family::Notes => note_rows(&mut world),
        Family::Accounts => {
            ask(&mut world, 32, people::Ask::AccountChange { holder: people::AccountHolder::Deployment,
                change: people::AccountChange::SetUses { account: 1, uses: Some(Box::new([people::Use::Project(1)])) } });
            assert!(world.store.rows.values().any(|record| matches!(record,
                root::Record::Core(core::Record::Accounts(accounts::Stored::Account { uses: Some(uses), .. }))
                if uses.as_ref() == [accounts::Use::Project(1)])));
        }
        Family::Application => adoption(&mut world),
    }
    world
}

/// Close the project by its deployment act and wait for every kept deletion step.
pub fn remove(world: &mut World) {
    act(world, 50, people::Act::RemoveProject { project: 1 });
    assert!(world.peers.acts.contains(&people::Outcome::ProjectRemoved { project: 1 }), "deployment act answered");
    for _ in 0..4 {
        if !world.store.rows.keys().any(|key| matches!(key, root::Key::Core(core::Key::People(people::Key::Deletion(1))))) { break; }
        world.advance(6_000_000_000);
        drain(world);
    }
    assert!(!world.store.rows.keys().any(|key| matches!(key, root::Key::Core(core::Key::People(people::Key::Deletion(1))))), "deletion ended whole");
    for (key, record) in &world.store.rows {
        if let Some(name) = jig_core_world::observations::naming::naming(key, record) {
            assert!(name.project != Some(1) || name.outlives.is_some(), "deleted project row: {name:?}");
        }
    }
    assert!(!world.store.rows.keys().any(|key| matches!(key, root::Key::Connector { key: connector::RecordKey::Adoption { project: 1, .. }, .. })), "connector released the project");
    crate::actions::stop(world);
    drain(world);
    assert!(!world.peers.drained.is_empty(), "engine drained after deletion");
}

/// The focused family story, with its referee retained throughout.
#[must_use]
pub fn run(family: Family, seed: u64, cut: Cut) -> World {
    let mut world = prepared(family, seed);
    world.cut(cut);
    remove(&mut world);
    world
}
