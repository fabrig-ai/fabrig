//! Public reading's world (jig/docs/design/domain/people/public-reading.md, 6–10).
//!
//! Core identifiers are internal; the clients' component replaces them at its boundary.
//! Move under another task is not built (domain/people/README.status.md); these stories only move out.
use crate::{
    fixtures,
    world::{Setup, World},
};
use jig_domain_core as core;
use jig_domain_people as people;
use jig_domain_tasks as tasks;
use jig_test_domain as root;
use skein_lib::{ReplyTo, Token};

pub const SEEDS: &[u64] = &[3101, 3102];
pub const PROJECT_NAME: &str = "Visible project marker";
pub const GOAL_TITLE: &[u8] = b"Visible goal marker";
pub const CHILD_TITLE: &[u8] = b"Visible delegate marker";
pub const PRIVATE: &[&[u8]] = &[
    b"hidden-spec-marker",
    b"hidden-delegate-spec",
    b"hidden-conversation",
    b"hidden-words-marker",
    b"hidden-instructions-marker",
    b"hidden-result-marker",
];

#[must_use]
pub fn world(seed: u64, conversation: bool) -> World {
    bounded_world(seed, conversation, 4, 2)
}

#[must_use]
pub fn bounded_world(seed: u64, conversation: bool, watermark: u32, reads: u32) -> World {
    let (mut config, mut limits) = crate::faults::plan_fixture(seed);
    eprintln!("public story seed {seed}");
    config.core.defaults.charters[0].policy.instructions = b"hidden-instructions-marker".as_slice().into();
    config.core.seed_projects[0].name = PROJECT_NAME.into();
    config.core.grants = Box::new([people::ActGrant { connector: 1, may: Box::new([people::ActKind::RemoveProject]) }]);
    limits.core.public = core::PublicIndexLimits { goals: 2, ended: 2, lines: 4, depth: 2 };
    limits.public_reads = reads;
    limits.public_watermark = watermark;
    let creating = jig_test_profile::fixtures::Creating::new(&config.core, limits.core.tasks.tree_tasks);
    let people::Ask::Create { project, mut batch } = jig_test_profile::fixtures::create_chat(
        &creating,
        1,
        0,
        if conversation {
            b"hidden-conversation".as_slice().into()
        } else {
            b"Visible goal marker\nhidden-spec-marker".as_slice().into()
        },
    ) else {
        unreachable!()
    };
    if !conversation {
        batch[0].flags.conversation = false;
        batch[0].flags.listed = true;
        batch[0].tracked = Some(17);
    }
    let mut world = World::new(Setup {
        seed,
        configuration: Some(config),
        limits,
        initial: Some(people::Ask::Create { project, batch }),
        ..Setup::default()
    });
    world.turn(1, 7, None, b"hidden-words-marker");
    world
}

pub fn ask(world: &mut World, key: u8, ask: people::Ask) -> people::Reply {
    fixtures::ask_as(world, world.session(), key, ask);
    if world.manual {
        let index = world
            .released
            .iter()
            .rposition(|out| {
                matches!(out, root::Request::Deliver(root::Delivery::Core(core::Held::PeopleReply { .. })))
            })
            .expect("manual party terminal");
        let root::Request::Deliver(root::Delivery::Core(core::Held::PeopleReply { reply, .. })) =
            world.released.remove(index)
        else {
            unreachable!()
        };
        reply
    } else {
        world.people_answers.last().expect("party terminal").clone()
    }
}

pub fn visibility(world: &mut World, key: u8, public: bool) {
    assert_eq!(
        ask(world, key, people::Ask::SetVisibility { project: 1, public }),
        people::Reply::Outcome(people::Outcome::VisibilitySet { project: 1, public })
    );
}

#[must_use]
pub fn delegate(world: &mut World, tracked: bool) -> u64 {
    let run = world.assignments[0];
    let mut child = fixtures::member(
        world,
        tasks::Executor::Agent { charter: 1 },
        b"Visible delegate marker\nhidden-delegate-spec",
    );
    child.tracked = tracked.then_some(23);
    fixtures::delegate(world, run, 1, child)
}

#[must_use]
pub fn read(kind: u16, task: Option<u64>) -> core::PublicRead {
    core::PublicRead { owner: 0, version: 1, kind, project: Some(1), task }
}

#[must_use]
pub fn event(reply: u64, read: core::PublicRead) -> root::Event {
    root::Event::Core(core::Event::PublicRead { reply: ReplyTo::new(Token::new(reply)), read })
}

/// The ordinary root loop, retaining the whole public boundary output.
#[must_use]
pub fn answer(world: &mut World, reply: u64, read: core::PublicRead) -> core::PublicAnswer {
    world.manual = true;
    let before = world.commits.len();
    world.send(event(reply, read));
    assert_eq!(world.commits.len(), before, "public reading commits nothing");
    let requests = std::mem::take(&mut world.released);
    world.manual = false;
    let mut answers = requests.into_iter().map(|request| {
        let root::Request::Deliver(root::Delivery::Core(core::Held::PublicRead { reply: to, answer })) = request else {
            panic!("public read's entire output: {request:?}");
        };
        assert_eq!(to.into_token(), Token::new(reply));
        answer
    });
    let answer = answers.next().expect("one public answer");
    assert!(answers.next().is_none(), "one terminal");
    answer
}

#[must_use]
pub fn document(answer: core::PublicAnswer) -> core::PublicDocument {
    let core::PublicAnswer::Document { document, .. } = answer else { panic!("public document: {answer:?}") };
    document
}

pub fn move_out(world: &mut World, task: u64) {
    assert!(matches!(
        ask(world, 61, people::Ask::Move { project: 1, task, reason: b"hidden-move-reason".as_slice().into() }),
        people::Reply::Outcome(people::Outcome::Moved { .. })
    ));
}

pub fn untrack(world: &mut World, task: u64) {
    let revision = fixtures::live_row(world, task).revisions.spec;
    let answer = ask(
        world,
        62,
        people::Ask::Amend {
            project: 1,
            task,
            revision,
            release: None,
            amendment: people::Amendment {
                tracked: Some(people::Tracking::Untracked),
                ..jig_test_profile::fixtures::person_unchanged(b"hidden-untrack-reason")
            },
        },
    );
    assert!(matches!(answer, people::Reply::Outcome(people::Outcome::Amended { .. })), "{answer:?}");
}

pub fn delete(world: &mut World) {
    world.send(root::Event::Core(core::Event::Act {
        reply_to: ReplyTo::new(Token::new(63)),
        connector: 1,
        key: [63; 16],
        act: Box::new(people::Act::RemoveProject { project: 1, grace: None }),
        carried: None,
    }));
    let answer = if world.manual {
        let index = world
            .released
            .iter()
            .rposition(|out| matches!(out, root::Request::Deliver(root::Delivery::Core(core::Held::Acted { .. }))))
            .expect("manual act terminal");
        let root::Request::Deliver(root::Delivery::Core(core::Held::Acted { outcome, .. })) =
            world.released.remove(index)
        else {
            unreachable!()
        };
        outcome
    } else {
        world.act_answers.last().expect("act terminal").1.clone()
    };
    assert_eq!(answer, people::Outcome::ProjectRemoved { project: 1 });
}

pub mod races;
