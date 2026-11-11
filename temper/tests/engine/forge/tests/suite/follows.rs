//! Source hearing and durable lag (temper/docs/design/domain/forge/follows.md, 3 and 9).
use skein_fake_forge_domain::api as raw;
use skein_forge::api::{self, Commit};
use temper_engine_domain_forge::{self as top, settings as s};
use temper_engine_forge_world::{REPO, World, translate};
const POLL: u64 = 40;
fn setting(paths: &[&[u8]]) -> s::Follow {
    s::Follow {
        source: s::FollowSource {
            repository: REPO,
            branch: b"main".as_slice().into(),
            paths: paths.iter().copied().map(Box::from).collect(),
        },
        target: s::FollowTarget {
            repository: api::Repository { forge: REPO.forge, repository: 3 },
            branch: b"main".as_slice().into(),
        },
        owner: s::FollowOwner::Person(4),
        start: s::FollowStart::Owner,
        producer: None,
        check: s::FollowCheck {
            workflow: b"tests.yml".as_slice().into(),
            artifact: b"results".as_slice().into(),
            format: temper_engine_domain_forge_tiers::Format::Junit,
        },
    }
}
fn world(queue: u32) -> World {
    let mut world = World::following(401, queue);
    world.keep_follow(1, Some(setting(&[b"src", b"include"])));
    world.run_for(3);
    assert!(row(&world).heard.is_some(), "first watched read establishes the baseline");
    world.take_calls();
    world
}
fn row(world: &World) -> &top::FollowRow {
    let Some(top::Stored::Follow(row)) = world.stored().get(&top::Key::Follow { project: 5, follow: 1 }) else {
        panic!("durable follow")
    };
    row
}
fn push(world: &mut World, path: &[u8], content: &[u8]) -> Commit {
    world.push_as(b"main", path, content, 2);
    let after = translate::commit(world.branch(b"main"));
    world.run_for(POLL);
    assert_eq!(row(world).heard, Some(after));
    after
}
fn ranges(calls: &[api::Op]) -> Vec<(Commit, Commit, u32)> {
    calls
        .iter()
        .filter_map(|op| match op {
            api::Op::Read(api::Read::Commits { before, after, page }) => Some((*before, *after, *page)),
            _ => None,
        })
        .collect()
}
fn files(paths: &[&[u8]], content: &[u8]) -> Box<[raw::File]> {
    paths.iter().map(|path| raw::File { path: Box::from(*path), content: Box::from(content) }).collect()
}
#[test]
fn accepted_follow_watches_baselines_filters_paths_and_removes_its_watch() {
    let mut world = world(16);
    let baseline = row(&world).heard.expect("baseline");
    assert_eq!(world.follow_lag(1).count, 0);
    assert!(row(&world).queue.is_empty());
    let message = b"Fix handoff\n\nThe body.";
    let after = world.push_message(files(&[b"src/a.c"], b"fix"), message);
    world.run_for(POLL);
    let entry = row(&world).queue.first().expect("matching move queued");
    assert_eq!((entry.before, entry.after), (baseline, after));
    assert_eq!(entry.commits.as_ref(), &[top::FollowCommit { id: after, message: message.as_slice().into() }]);
    assert_eq!(entry.files.as_deref(), Some([Box::from(b"src/a.c".as_slice())].as_slice()));
    assert_eq!(entry.more_commits, 0);
    assert_eq!(entry.more_files, 0);
    assert_eq!(entry.state, top::FollowState::NotMade);
    assert_eq!(row(&world).heard, Some(after));
    // Keep the source tree intact: deleting a source path would itself overlap.
    push(&mut world, b"docs/x.md", b"docs");
    push(&mut world, b"src-old/a.c", b"not src");
    assert_eq!(world.follow_lag(1).count, 1);
    assert_eq!(world.follow_lag(1).entries, row(&world).queue);
    // Keeping an accepted setting preserves its queue and cursor.
    let saved = row(&world).clone();
    world.keep_follow(1, Some(setting(&[b"src", b"include"])));
    assert_eq!(row(&world), &saved);
    world.followed(1, &[after]);
    assert_eq!(world.follow_lag(1).count, 0);
    world.keep_follow(1, None);
    world.run_for(2);
    assert!(!world.stored().contains_key(&top::Key::Follow { project: 5, follow: 1 }));
    world.take_calls();
    world.push_as(b"main", b"src/a.c", b"after removal", 2);
    world.run_for(POLL);
    assert!(
        world
            .take_calls()
            .iter()
            .all(|op| !matches!(op, api::Op::Read(api::Read::Branch { branch }) if branch.as_ref() == b"main"))
    );
    assert_eq!(world.follow_lag(1).count, 0);
}
#[test]
fn a_follow_on_an_existing_watch_takes_a_fresh_baseline() {
    let mut world = world(16);
    world.push_as(b"main", b"src/a.c", b"before second acceptance", 2);
    let after = translate::commit(world.branch(b"main"));
    world.keep_follow(2, Some(setting(&[b"src"])));
    world.run_for(POLL);
    let Some(top::Stored::Follow(second)) = world.stored().get(&top::Key::Follow { project: 5, follow: 2 }) else {
        panic!("second record")
    };
    assert_eq!(second.heard, Some(after));
    assert_eq!(world.follow_lag(2).count, 0, "a move before acceptance is only the fresh baseline");
    // Either follow owns the shared source watch until the last removal.
    world.keep_follow(1, None);
    world.push_as(b"main", b"src/a.c", b"second follows", 2);
    world.run_for(POLL);
    assert_eq!(world.follow_lag(2).count, 1);
}
#[test]
fn unknown_comparisons_failed_reads_and_empty_paths_keep_the_move() {
    let mut world = world(16);
    world.follow_read_limits(1, 8, 32, 512);
    world.push_message(files(&[b"docs/a", b"docs/b"], b"outside"), b"wide");
    world.run_for(POLL);
    assert_eq!(row(&world).queue.len(), 1, "incomplete comparison is unknown even outside paths");
    assert!(row(&world).queue.first().expect("unknown move").files.is_none());
    world.reject_comparisons();
    push(&mut world, b"docs/a", b"failed comparison");
    assert_eq!(row(&world).queue.len(), 2);
    assert!(row(&world).queue.last().expect("failed read").files.is_none());
    // A commits failure also makes overlap unknown rather than silently missing work.
    world.allow_comparisons();
    world.script_ancestry(Some(Err(api::Error::Forbidden)));
    push(&mut world, b"docs/a", b"failed commits");
    assert_eq!(row(&world).queue.len(), 3);
    assert!(row(&world).queue.last().expect("failed commits").files.is_none());
    let mut all = World::following(402, 16);
    let mut follow = setting(&[]);
    follow.source.branch = Box::new([]); // Adopted default branch.
    all.keep_follow(1, Some(follow));
    all.run_for(3);
    let after = push(&mut all, b"docs/alone", b"no paths");
    assert_eq!(all.follow_lag(1).count, 1);
    assert_eq!(row(&all).queue.first().expect("all moves").after, after);
}
#[test]
fn a_second_move_during_the_read_starts_at_the_cursor_the_first_leaves() {
    let mut world = world(16);
    let baseline = row(&world).heard.expect("baseline");
    world.pause_comparisons();
    world.push_as(b"main", b"src/a.c", b"first", 2);
    let first = translate::commit(world.branch(b"main"));
    world.run_for(POLL);
    assert!(world.comparisons_held() > 0);
    assert_eq!(row(&world).heard, Some(baseline));
    world.push_as(b"main", b"include/a.h", b"second", 2);
    let second = translate::commit(world.branch(b"main"));
    world.run_for(POLL);
    assert_eq!(world.follow_lag(1).count, 0);
    assert!(
        world
            .stored()
            .iter()
            .any(|(_, record)| matches!(record, top::Stored::BranchHead(head) if head.commit == second)),
        "second move is observed while the first read is held"
    );
    world.release_comparisons();
    world.run_for(POLL);
    let entries = world.follow_lag(1).entries;
    assert_eq!(
        entries.iter().map(|entry| (entry.before, entry.after)).collect::<Vec<_>>(),
        [(baseline, first), (first, second)]
    );
    assert_eq!(ranges(&world.take_calls()), [(baseline, first, 1), (first, second, 1)]);
}
#[test]
fn restart_keeps_three_entries_and_reasks_the_uncommitted_range() {
    let mut world = world(16);
    for content in [b"one".as_slice(), b"two", b"three"] {
        push(&mut world, b"src/a.c", content);
    }
    let saved = row(&world).clone();
    assert_eq!(saved.queue.len(), 3);
    world.pause_comparisons();
    world.push_as(b"main", b"include/a.h", b"four", 2);
    let after = translate::commit(world.branch(b"main"));
    world.run_for(POLL);
    assert!(world.comparisons_held() > 0);
    assert_eq!(row(&world), &saved);
    world.restart();
    assert_eq!(row(&world), &saved);
    world.release_comparisons();
    world.take_calls();
    world.run_for(POLL);
    let lag = world.follow_lag(1);
    assert_eq!(lag.count, 4);
    assert_eq!(lag.entries.get(..3), Some(saved.queue.as_ref()));
    assert_eq!(lag.entries.last().expect("reasked range").before, saved.heard.expect("cursor"));
    assert_eq!(lag.entries.last().expect("reasked range").after, after);
    assert_eq!(ranges(&world.take_calls()), [(saved.heard.expect("cursor"), after, 1)]);
    world.restart();
    world.run_for(POLL);
    assert_eq!(world.follow_lag(1), lag, "no entry queued twice");
}
#[test]
fn full_queue_stops_reads_until_followed_then_reads_one_accumulated_range() {
    let mut world = world(2);
    let first = push(&mut world, b"src/a.c", b"one");
    let second = push(&mut world, b"src/a.c", b"two");
    world.take_calls();
    world.push_as(b"main", b"src/a.c", b"three", 2);
    world.run_for(POLL);
    world.push_as(b"main", b"include/a.h", b"four", 2);
    let after = translate::commit(world.branch(b"main"));
    world.run_for(POLL);
    assert_eq!(row(&world).heard, Some(second));
    assert_eq!(world.follow_lag(1).count, 2);
    assert!(ranges(&world.take_calls()).is_empty(), "a full follow reads no commits");
    world.followed(1, &[first]);
    world.run_for(2);
    assert_eq!(row(&world).heard, Some(after));
    let last = row(&world).queue.last().expect("resumed range");
    assert_eq!((last.before, last.after), (second, after));
    assert_eq!(last.commits.len(), 2);
    assert_eq!(ranges(&world.take_calls()), [(second, after, 1)]);
}
#[test]
fn a_temper_merge_is_heard_with_its_pull_number() {
    use temper_engine_domain_forge_change as change;
    use temper_engine_forge_world::landings;
    let mut world = world(16);
    let before = row(&world).heard.expect("baseline");
    assert!(world.project_landing(5, change::Way::Turn, Box::new([])));
    world.produce_file(b"temper/51", b"src/a.c", b"own merge");
    world.status(b"temper/51", raw::Check::Passed);
    landings::ready_change(&mut world, 51, b"temper/51");
    let pull = world.change(51).pull.expect("change pull");
    let mut landing = landings::landing_row(world.change(51));
    landing.landing.task = 71;
    landing.landing.members.first_mut().expect("source member").gates = Box::new([]);
    world.event(top::Event::Landing { row: landing });
    landings::drive_landing(&mut world, 71);
    // The fake records the merge, and the connector hears its own outcome, not only a poll.
    world.run_for(2);
    let after = translate::commit(world.branch(b"main"));
    assert!(world.stored().contains_key(&top::Key::Landed(after)));
    let entry = row(&world).queue.first().expect("own merge queued");
    assert_eq!((entry.before, entry.after, entry.pull), (before, after, Some(pull)));
    assert_eq!(world.follow_lag(1).count, 1);
    assert_eq!(row(&world).heard, Some(after));
}
#[test]
fn paged_commits_keep_the_newest_and_count_and_cut_the_rest() {
    let mut world = world(1);
    let first = push(&mut world, b"src/a.c", b"fill queue");
    world.follow_read_limits(2, 2, 1, 4);
    let mut newest = Vec::new();
    for number in 0_u8..5 {
        let after = world.push_message(files(&[b"src/a.c", b"include/a.h"], &[number]), b"message body");
        newest.push(after);
        world.run_for(POLL);
    }
    let after = *newest.last().expect("five pushes");
    world.take_calls();
    world.followed(1, &[first]);
    world.run_for(3);
    let entry = row(&world).queue.first().expect("accumulated move");
    assert_eq!(entry.after, after);
    assert_eq!(entry.commits.iter().map(|commit| commit.id).collect::<Vec<_>>(), newest.get(3..).expect("newest two"));
    assert!(entry.commits.iter().all(|commit| commit.message.as_ref() == b"mess"));
    assert_eq!(entry.more_commits, 3);
    assert_eq!(entry.files.as_ref().expect("complete comparison").len(), 1);
    assert_eq!(entry.more_files, 1);
    assert_eq!(ranges(&world.take_calls()), [(first, after, 1), (first, after, 2), (first, after, 3)]);
}
