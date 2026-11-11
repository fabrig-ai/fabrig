//! Rules cover a change's own diff at its judged head, and a sign-off waits outside the queue
//! (temper/docs/design/domain/forge/change/gates.md, 2; queues.md; change/questions.md, 2).
use super::approval_identity::{approvals_of, forge_with_only_temper, post_approval, post_approval_read, settle};
use super::light_rules::change_row;
use super::update_first::{finish, status};
use super::*;
use temper_engine_domain_forge::settings;
use temper_engine_domain_forge_change::State;
use temper_engine_domain_world::kinds;
use temper_protocol_reaches::{Classes, PathSet};

const WATCHER: Token = Token::new(990);
const SET_NAME: &[u8] = b"vsr's protocol core";

fn story() -> (World, engine::Assignment) {
    let mut world = World::configured_limits(
        true,
        true,
        false,
        1000,
        Gate::default(),
        None,
        StoryLimits { kinds: true, ..StoryLimits::default() },
    );
    forge_with_only_temper(&mut world);
    let (mut world, _, producer, _) = change_world_in(world, Change::default());
    fake::grant(&mut world.fake, b"org/repo", 2, raw::Permission::Write);
    let classes = settings::Value::Classes(Classes {
        items: Box::new([]),
        sets: Box::new([PathSet {
            number: 1,
            name: SET_NAME.into(),
            patterns: [b"include/vsr.h".as_slice(), b"src/protocol.*"].map(Box::from).into(),
        }]),
    });
    let rule = settings::Value::Rule(settings::LandingRule {
        sets: Box::new([1]),
        pattern: forge_top::GatePattern {
            segments: [b"forge".as_slice(), b"forge.example", b"org", b"repo", b"branch"].map(Box::from).into(),
            last: forge_top::GateLast::Exact(b"main".as_slice().into()),
        },
        enforce: true,
        ci: true,
        up_to_date: true,
        gates: Box::new([]),
        approvals: Box::new([settings::RuleApproval {
            role: people::Role::Owner.number(),
            people: 1,
            freshness: forge_top::JudgeFreshness::Clean,
        }]),
    });
    let revision = kinds::revision(&world.store.rows);
    world.send(kinds::change(
        world.signed_in.expect("owner session"),
        revision,
        96,
        Box::new([kinds::item(8, 1, 1, &classes), kinds::item(2, 1, 1, &rule)]),
    ));
    settle(&mut world);
    assert_eq!(kinds::revision(&world.store.rows), revision + 1, "{:?}", world.web_replies);
    (world, producer)
}

fn change_of(world: &World, producer: &engine::Assignment) -> u64 {
    let tasks::Party::Task(change) = live_task(world, producer.task).requester else { panic!("change producer") };
    change
}

fn next_change(world: &mut World) -> engine::Assignment {
    let before = world.assigned.len();
    let chat = world.assigned[0].clone();
    let engine::CallAnswer::Delegated(_) = change_tool::change(world, &chat, 94, None, b"Independent change") else {
        panic!("change delegated: {:?}", world.answers);
    };
    reviews::next_assignment(world, before)
}

// None makes an empty commit: a change whose own diff touches no path.
fn publish(world: &mut World, producer: &engine::Assignment, path: Option<&[u8]>, pending: bool) -> u64 {
    let change = change_of(world, producer);
    let branch = change_row(world, change).branch.clone();
    let base = world.fake.branch(b"org/repo", b"main").expect("main tip");
    let head = if let Some(path) = path {
        assert_eq!(
            world.external(2, raw::Op::Git(raw::Git::Create { branch: branch.clone(), commit: base })),
            raw::Answer::Branch(raw::Created::Created)
        );
        fake::advance(&mut world.fake, &world.fake_env, b"org/repo", &branch, path, b"added", 2)
            .expect("producer pushed")
    } else {
        let head =
            fake::commit_tree(&mut world.fake, &world.fake_env.limits, base, None, Box::new([]), b"Empty change")
                .expect("empty commit on the empty main tree");
        assert_eq!(
            world.external(2, raw::Op::Git(raw::Git::Push { branch, commit: head, expected: None })),
            raw::Answer::Pushed(raw::Pushed::Pushed)
        );
        head
    };
    if pending {
        // Let the fake's automatic build finish before replacing its verdict with pending.
        settle(world);
        status(world, head, raw::Check::Pending);
    }
    finish(world, producer);
    if pending {
        for _ in 0..200 {
            world.tick();
            if matches!(change_row(world, change).change.state, State::Checking { .. }) {
                break;
            }
        }
    } else {
        settle(world);
    }
    head
}

fn approvals(world: &mut World) -> engine::Approvals {
    // The world's short sessions expire while a sign-off waits; each read authenticates afresh.
    world.sign_in();
    world.approvals.clear();
    world.send(engine::Event::Party(engine::PartyInput::Watch {
        watcher: WATCHER,
        sign_in: world.signed_in.expect("approval watcher session"),
        subject: jig_domain_core::documents::Watched::Application {
            scope: people::ApplicationScope::Own,
            connector: 0,
            subject: Token::new(engine::APPROVALS_SUBJECT),
        },
        window: jig_domain_core::documents::Window { rows: world.env.limits.documents.rows, after: None },
    }));
    for _ in 0..200 {
        if !world.approvals.is_empty() {
            break;
        }
        world.tick();
    }
    let document = approvals_of(world, WATCHER);
    world.send(engine::Event::Party(engine::PartyInput::Unwatch { watcher: WATCHER }));
    document
}

fn assert_wait(world: &mut World, change: u64, head: u64) {
    let row = change_row(world, change);
    assert!(matches!(row.change.state, State::Queued { .. }), "{:?}", row.change);
    assert!(!row.change.owns_turn, "the sign-off does not own the queue");
    assert_eq!(row.change.last_head, Some(translate::commit(head)));
    let document = approvals(world);
    assert_eq!(document.rows.len(), 1, "{document:?}");
    let approval = &document.rows[0];
    assert_eq!(approval.task, change);
    assert_eq!(
        (approval.required, approval.counted, approval.yours, approval.whole),
        (1, 0, engine::YourReview::Counts, true)
    );
    assert_eq!(approval.sets.len(), 1);
    assert_eq!(approval.sets[0].number, 1);
    assert_eq!(approval.sets[0].name.as_ref(), SET_NAME);
    assert!(pull_of(world, change).merged.is_none(), "the fake forge has not merged the waiting change");
}

fn pull_of(world: &mut World, change: u64) -> raw::Pull {
    let number = change_row(world, change).pull.expect("opened pull");
    let raw::Answer::Pull(pull) = world.external(1, raw::Op::Read(raw::Read::Pull { number })) else {
        panic!("fake forge pull")
    };
    pull
}

fn assert_landed(world: &mut World, change: u64) {
    for _ in 0..200 {
        if matches!(change_row(world, change).change.state, State::Landed { .. }) {
            break;
        }
        world.tick();
    }
    assert!(matches!(change_row(world, change).change.state, State::Landed { .. }), "{:?}", change_row(world, change));
    let pull = change_row(world, change).pull.expect("opened pull");
    assert!(
        world
            .writes
            .iter()
            .any(|write| matches!(write, skein_forge::api::Write::Merge { number, .. } if *number == pull))
    );
    assert!(pull_of(world, change).merged.is_some(), "the fake forge merged it");
}

#[test]
fn a_protocol_signoff_waits_outside_the_queue_while_docs_land_without_approval() {
    let (mut world, producer) = story();
    let protocol = change_of(&world, &producer);
    let head = publish(&mut world, &producer, Some(b"src/protocol.c"), false);
    assert_wait(&mut world, protocol, head);
    assert_eq!(world.merges, 0);
    let since = change_row(&world, protocol).change.ready_since;

    world.env.now = world.env.now.saturating_add(Duration::from_secs(1));
    let docs_producer = next_change(&mut world);
    let docs = change_of(&world, &docs_producer);
    assert_eq!(change_row(&world, docs).base, change_row(&world, protocol).base);
    publish(&mut world, &docs_producer, Some(b"docs/a.md"), false);
    assert_landed(&mut world, docs);
    assert!(pull_of(&mut world, docs).reviews.is_empty(), "the docs-only change landed without a forge review");
    assert_eq!(world.merges, 1, "a docs-only change needs no approval and never waits behind the protocol edit");
    assert_wait(&mut world, protocol, head);
    assert_eq!(change_row(&world, protocol).change.ready_since, since);

    post_approval(&mut world, protocol, 7);
    assert_landed(&mut world, protocol);
    assert_eq!(world.merges, 2);
    assert!(approvals(&mut world).rows.is_empty());
    world.close();
}

#[test]
fn a_new_head_touching_the_header_waits_for_approval() {
    let (mut world, producer) = story();
    let change = change_of(&world, &producer);
    let earlier = publish(&mut world, &producer, Some(b"docs/a.md"), true);
    assert!(
        matches!(change_row(&world, change).change.state, State::Checking { .. }),
        "{:?}",
        change_row(&world, change)
    );
    assert!(approvals(&mut world).rows.is_empty());
    assert_eq!(world.merges, 0, "CI, not the path rule, keeps the docs head");

    let branch = change_row(&world, change).branch.clone();
    let head = fake::advance(&mut world.fake, &world.fake_env, b"org/repo", &branch, b"include/vsr.h", b"header", 2)
        .expect("new pushed head");
    assert_ne!(head, earlier);
    status(&mut world, head, raw::Check::Passed);
    let pull = change_row(&world, change).pull.expect("opened pull");
    world.send(engine::Event::Forge(Box::new(forge_top::Event::Hint {
        hint: skein_forge::api::Hint {
            repository: forge_world::REPO,
            change: skein_forge::api::Change::Item(pull),
            key: None,
        },
    })));
    settle(&mut world);
    assert!(matches!(
        change_row(&world, change).change.state,
        State::Held { why: temper_engine_domain_forge_change::Hold::BranchMoved, .. }
    ));
    // A person's push is drift until the owner accepts that new head.
    world.sign_in();
    world.send(engine::Event::Party(engine::PartyInput::Ask {
        models: Box::new([]),
        reply_to: ReplyTo::new(Token::new(97)),
        sign_in: world.signed_in.expect("owner session"),
        key: [97; 16],
        ask: people::Ask::Release {
            project: 1,
            task: change,
            revision: live_task(&world, change).revisions.hold,
            again: None,
        },
    }));
    settle(&mut world);
    assert_wait(&mut world, change, head);
    assert_eq!(world.merges, 0);
    post_approval(&mut world, change, 7);
    assert_landed(&mut world, change);
    assert_eq!(world.merges, 1);
    assert!(approvals(&mut world).rows.is_empty());
    world.close();
}

#[test]
fn taking_mains_protocol_edit_in_an_update_touches_nothing() {
    let (mut world, producer) = story();
    let protocol = change_of(&world, &producer);
    let protocol_head = publish(&mut world, &producer, Some(b"src/protocol.c"), false);
    assert_wait(&mut world, protocol, protocol_head);
    let empty_producer = next_change(&mut world);
    let empty = change_of(&world, &empty_producer);
    let head = publish(&mut world, &empty_producer, None, true);
    assert!(
        matches!(change_row(&world, empty).change.state, State::Checking { .. }),
        "{:?}",
        change_row(&world, empty)
    );

    post_approval_read(&mut world, protocol, 7);
    settle(&mut world);
    assert_landed(&mut world, protocol);
    assert_eq!(world.merges, 1);
    let main = world.fake.branch(b"org/repo", b"main").expect("protocol landing");
    status(&mut world, head, raw::Check::Passed);
    settle(&mut world);
    assert_landed(&mut world, empty);
    assert_eq!(world.merges, 2, "an update brings main's edit, not a new approval requirement");
    let row = change_row(&world, empty);
    assert_eq!(row.change.updates, 1);
    let updated = fake::commit_count(&row.change.last_head.expect("updated head")).expect("fake commit");
    assert_ne!(updated, head);
    assert_eq!(
        world.fake.object(updated).expect("updated tree").tree.get(b"src/protocol.c".as_slice()).map(AsRef::as_ref),
        Some(b"added".as_slice())
    );
    let raw::Answer::Comparison { files, .. } =
        world.external(1, raw::Op::Read(raw::Read::Compare { base: main, head: updated, page: 1, limit: 10 }))
    else {
        panic!("updated diff")
    };
    assert!(files.is_empty(), "only main's protocol edit was brought in");
    assert!(approvals(&mut world).rows.is_empty());
    world.close();
}

#[test]
fn unread_comparisons_make_an_untouching_change_wait_for_approval() {
    let (mut world, producer) = story();
    world.reject_comparisons();
    let change = change_of(&world, &producer);
    let head = publish(&mut world, &producer, None, false);
    assert_wait(&mut world, change, head);
    assert_eq!(world.merges, 0, "an unread diff conservatively touches every set");
    post_approval(&mut world, change, 7);
    assert_landed(&mut world, change);
    assert_eq!(world.merges, 1);
    assert!(approvals(&mut world).rows.is_empty());
    world.close();
}
