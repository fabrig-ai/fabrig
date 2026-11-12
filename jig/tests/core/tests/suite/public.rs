//! The public world stories (jig/docs/design/domain/people/public-reading.md, 6).
use jig_core_world::stories::public as story;
use jig_domain_core::{self as core, PublicAnswer, PublicDocument};
use jig_domain_people as people;
use jig_test_domain as root;
use skein_lib::{ReplyTo, Token};

#[test]
fn an_owner_makes_a_project_public_and_the_public_party_reads_its_board_and_a_delegates_card() {
    for &seed in story::SEEDS {
        let mut world = story::world(seed, false);
        let goal = world.assignments[0].0;
        let child = story::delegate(&mut world, false);
        story::visibility(&mut world, 51, true);
        let PublicDocument::Board(board) = story::document(story::answer(&mut world, 500, story::read(2, None))) else {
            panic!("board seed {seed}")
        };
        assert_eq!(board.name.as_ref(), story::PROJECT_NAME, "seed {seed}");
        assert_eq!(board.live.len(), 1, "seed {seed}");
        let goal_line = &board.live[0];
        assert_eq!(goal_line.line.task, goal, "core numbers seed {seed}");
        assert_eq!(goal_line.line.title.bytes.as_ref(), story::GOAL_TITLE);
        assert_eq!(goal_line.tree.iter().map(|line| line.task).collect::<Vec<_>>(), [child]);
        assert_eq!(goal_line.line.delegates.working, 1);
        let PublicDocument::Card(card) = story::document(story::answer(&mut world, 501, story::read(3, Some(child))))
        else {
            panic!("card seed {seed}")
        };
        assert_eq!(card.line.title.bytes.as_ref(), story::CHILD_TITLE);
        assert_eq!(card.line.parent, Some(goal));
        assert_eq!(card.lineage.as_ref(), &[goal]);
        assert!(card.delegates.is_empty());
        let shown = format!("{board:?} {card:?}");
        for marker in story::PRIVATE {
            assert!(!shown.contains(format!("{marker:?}").trim_matches(['[', ']'])), "private marker seed {seed}");
        }
    }
}

#[test]
fn a_members_set_visibility_is_refused() {
    let mut world = story::world(story::SEEDS[0], false);
    world.send(root::Event::Core(core::Event::People(people::Event::Roles {
        project: 1,
        holdings: Box::new([people::Holding { person: 1, role: people::Role::Member }]),
    })));
    let answer = story::ask(&mut world, 51, people::Ask::SetVisibility { project: 1, public: true });
    assert!(
        matches!(answer, people::Reply::Outcome(people::Outcome::Refused(people::Refusal::Role { .. }))),
        "{answer:?}"
    );
    assert_eq!(story::answer(&mut world, 500, story::read(2, None)), PublicAnswer::Unknown);
    world.send(root::Event::Core(core::Event::People(people::Event::Roles {
        project: 1,
        holdings: Box::new([people::Holding { person: 1, role: people::Role::Owner }]),
    })));
}

#[test]
fn a_goal_beneath_a_conversation_reads_as_a_public_goal_with_no_parent_and_the_conversation_is_never_named_or_counted()
{
    let mut world = story::world(story::SEEDS[0], true);
    let conversation = world.assignments[0].0;
    let goal = story::delegate(&mut world, true);
    story::visibility(&mut world, 51, true);
    let PublicDocument::Board(board) = story::document(story::answer(&mut world, 500, story::read(2, None))) else {
        panic!("board")
    };
    assert_eq!(board.live.len(), 1);
    let line = &board.live[0];
    assert_eq!(line.line.task, goal);
    assert_eq!(line.line.parent, None);
    assert!(line.tree.is_empty());
    assert_eq!(line.line.delegates.working, 0);
    assert_eq!(board.live_folded, 0);
    assert_eq!(story::answer(&mut world, 501, story::read(3, Some(conversation))), PublicAnswer::Unknown);
}

#[test]
fn a_conversations_delegates_card_is_unknown() {
    let mut world = story::world(story::SEEDS[0], true);
    let child = story::delegate(&mut world, false);
    story::visibility(&mut world, 51, true);
    assert_eq!(story::answer(&mut world, 500, story::read(3, Some(child))), PublicAnswer::Unknown);
}

/// Move beneath a task is not built (jig/docs/design/domain/people/README.status.md).
#[test]
fn a_delegate_moved_out_of_its_goals_tree_leaves_its_board_counts_and_card() {
    let mut world = story::world(story::SEEDS[0], false);
    let child = story::delegate(&mut world, false);
    story::visibility(&mut world, 51, true);
    story::move_out(&mut world, child);
    let PublicDocument::Board(board) = story::document(story::answer(&mut world, 500, story::read(2, None))) else {
        panic!("board")
    };
    assert!(board.live[0].tree.is_empty());
    assert_eq!(board.live[0].line.delegates.working, 0);
    assert_eq!(board.live[0].folded, 0);
    assert_eq!(story::answer(&mut world, 501, story::read(3, Some(child))), PublicAnswer::Unknown);
}

#[test]
fn private_never_made_and_deleted_projects_answer_unknown_with_the_same_work() {
    let mut world = story::world(story::SEEDS[0], false);
    let private = story::answer(&mut world, 500, story::read(2, None));
    let never = story::answer(&mut world, 501, core::PublicRead { project: Some(99), ..story::read(2, None) });
    story::visibility(&mut world, 51, true);
    story::delete(&mut world);
    let deleted = story::answer(&mut world, 502, story::read(2, None));
    assert_eq!(private, PublicAnswer::Unknown);
    assert_eq!(private, never);
    assert_eq!(private, deleted);
}

#[test]
fn a_deleted_public_project_is_unknown_from_its_deletions_decision() {
    let mut world = story::world(story::SEEDS[0], false);
    story::visibility(&mut world, 51, true);
    world.hold_project_releases.push(1);
    story::delete(&mut world);
    assert_eq!(story::answer(&mut world, 500, story::read(2, None)), PublicAnswer::Unknown);
    world.hold_project_releases.clear();
    world.send(root::Event::Core(core::Event::ProjectReleased { connector: 1, project: 1 }));
    super::deletion::finish(&mut world);
    assert!(world.domain.core().people.deletions().is_empty());
    assert!(world.domain.core().people.deleted(1));
    assert_eq!(story::answer(&mut world, 501, story::read(2, None)), PublicAnswer::Unknown);
}

#[test]
fn the_public_watermark_and_the_cores_public_reads_refuse_a_public_read_and_admit_a_signed_in_one() {
    for (watermark, reads) in [(1, 2), (4, 1)] {
        let mut world = story::bounded_world(story::SEEDS[0], false, watermark, reads);
        story::visibility(&mut world, 51, true);
        world.manual = true;
        world.public_offer(story::event(500, story::read(2, None)));
        world.public_offer(story::event(501, story::read(2, None)));
        let signed = world.session();
        world.public_offer(root::Event::Core(core::Event::Read {
            reply: ReplyTo::new(Token::new(502)),
            sign_in: signed,
            read: core::reads::Read::Project { project: 99 },
        }));
        world.drain();
        assert!(world.released.iter().any(|out| matches!(
            out,
            root::Request::Deliver(root::Delivery::Core(core::Held::PublicRead { answer: PublicAnswer::Busy, .. }))
        )));
        assert!(
            world
                .released
                .iter()
                .any(|out| matches!(out, root::Request::Deliver(root::Delivery::Core(core::Held::Read { .. }))))
        );
        world.released.clear();
        world.manual = false;
    }
}

#[test]
fn made_private_while_a_public_read_is_under_way() {
    story::races::play(story::races::Change::Private);
}
#[test]
fn deletion_admitted_while_a_public_read_is_under_way() {
    story::races::play(story::races::Change::Deleting);
}
#[test]
fn moved_out_while_a_public_read_is_under_way() {
    story::races::play(story::races::Change::Moved);
}
#[test]
fn untracked_while_a_public_read_is_under_way() {
    story::races::play(story::races::Change::Untracked);
}

#[test]
fn the_public_referee_catches_each_breach() {
    use jig_core_world::referee::public::{Broken, planted_breaches};
    let mut world = story::world(story::SEEDS[0], false);
    let child = story::delegate(&mut world, false);
    story::visibility(&mut world, 51, true);
    let read = story::read(3, Some(child));
    let document = story::document(story::answer(&mut world, 500, read));
    assert_eq!(
        planted_breaches(&world, read, &document),
        [
            Broken::PublicCommit,
            Broken::PublicRecord,
            Broken::Watermark,
            Broken::Membership,
            Broken::Private,
            Broken::Membership,
            Broken::Membership,
            Broken::Private,
            Broken::Membership,
            Broken::Membership,
            Broken::Fields
        ]
    );
}

#[test]
fn an_ended_public_goal_is_retained_by_the_public_referee() {
    let mut world = story::world(story::SEEDS[0], false);
    story::visibility(&mut world, 51, true);
    jig_core_world::fixtures::finish_chat(&mut world, 7, b"hidden-result-marker");
    let PublicDocument::Board(board) = story::document(story::answer(&mut world, 500, story::read(2, None))) else {
        panic!("board")
    };
    assert!(board.live.is_empty());
    assert_eq!(board.ended.len(), 1);
}
