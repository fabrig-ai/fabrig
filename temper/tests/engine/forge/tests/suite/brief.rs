//! Resolver Pull sections against client admission (temper/docs/design/domain/forge/change/repairs.md).
use skein_fake_forge_domain::api as raw;
use skein_forge::api;
use skein_lib::Token;
use temper_engine_domain_forge as top;
use temper_engine_domain_forge_change as change;
use temper_engine_forge_world::{REPO, World, changes, translate};

const SECTION: Token = Token::new(245);
const BYTES: u32 = 2048;

fn tree(files: u32) -> Box<[raw::File]> {
    let mut tree = vec![raw::File { path: Box::from(b"shared".as_slice()), content: Box::from(b"landed".as_slice()) }];
    for number in 1..files {
        tree.push(raw::File {
            path: format!("base-{number}").into_bytes().into_boxed_slice(),
            content: Box::from(b"landed".as_slice()),
        });
    }
    tree.into_boxed_slice()
}

fn resolver(files: u32, extra_commits: u32, missing: bool) -> (World, u64, api::Commit, api::Commit) {
    let mut world = World::with_comparison_room(245);
    world.adopt();
    world.produce_file(b"g", b"shared", b"base");
    world.produce_on(b"temper/51", b"g");
    world.push_file(b"temper/51", b"shared", b"ours");
    let pull = world.open_pull(b"temper/51", b"g");
    let head = translate::commit(world.branch(b"temper/51"));
    for (branch, count) in [(b"landing-one".as_slice(), 2), (b"landing-two".as_slice(), files)] {
        world.produce_on(branch, b"g");
        world.push_tree(branch, tree(count));
        world.status(branch, raw::Check::Passed);
        let landed = world.open_pull(branch, b"g");
        world.merge(landed, branch);
        world.delete_branch(branch);
    }
    for _ in 0..extra_commits {
        world.push_tree(b"g", tree(files));
    }
    let base = if missing { translate::commit(u64::MAX) } else { translate::commit(world.branch(b"g")) };
    let mut row = changes::change_row();
    row.base = Box::from(b"g".as_slice());
    row.pull = Some(pull);
    row.change.last_head = Some(head);
    row.change.resolutions = 2;
    row.change.state = change::State::Resolving { requested: true, base };
    world.event(top::Event::Change { row });
    (world, pull, head, base)
}

fn gather(world: &mut World, pull: u64, head: api::Commit) {
    start_gather(world, pull, head);
    world.run_for(5);
}

fn start_gather(world: &mut World, pull: u64, head: api::Commit) {
    world.take_seen();
    world.take_calls();
    world.event(top::Event::GatherBriefHeld {
        section: SECTION,
        source: top::BriefSource::Pull {
            item: top::BriefItem { repository: REPO.repository, number: pull },
            head: top::BriefCommit(head),
        },
        parts: 1,
        bytes: BYTES,
        max_job_bytes: 0,
    });
}

#[test]
fn a_resolvers_files_read_at_a_moved_head_pauses_and_wakes_its_change() {
    let (mut world, pull, head, _) = resolver(3, 0, false);
    start_gather(&mut world, pull, head);
    world.run_until_read(&api::Read::PullFiles { number: pull, head, page: 1, skip: 0, sides: true });
    world.push_file(b"temper/51", b"shared", b"pushed meanwhile");
    assert_ne!(translate::commit(world.branch(b"temper/51")), head);
    world.run_for(5);
    assert!(
        world.seen().contains(&top::Request::BriefSized {
            section: SECTION,
            size: Err(top::BriefUnread {
                fate: top::BriefFate::Capacity,
                words: Box::from(b"the change's head moved".as_slice()),
            }),
        }),
        "the resolution pauses without a permanent try: {:?}",
        world.seen()
    );
    assert!(world.seen().contains(&top::Request::BriefReadRefused {
        section: SECTION,
        stage: "files",
        reason: "the change's head moved",
        heads: None,
        error: Some(api::Error::Stale),
    }));
    assert!(world.seen().contains(&top::Request::WakeProcedure { task: changes::change_row().change.task }));
}

fn completed(world: &mut World, pull: u64, head: api::Commit, files: u32, commits: usize) {
    let sizes: Vec<_> = world
        .seen()
        .iter()
        .filter_map(|request| {
            if let top::Request::BriefSized { section: SECTION, size } = request { Some(size.clone()) } else { None }
        })
        .collect();
    assert_eq!(sizes.len(), 1, "one terminal sizing response");
    let size = sizes[0].clone().expect("required Pull section survives comparison admission");
    assert!(size <= BYTES);
    world.take_seen();
    world.event(top::Event::TakeBrief { section: SECTION });
    let [top::Request::BriefTaken { section: SECTION, bytes: Some(words) }] = world.seen() else {
        panic!("the required Pull section must be taken: {:?}", world.seen())
    };
    assert_eq!(words.len(), usize::try_from(size).expect("bounded size"));
    let text = std::str::from_utf8(words).expect("brief text");
    let pinned: String = head[..20].iter().flat_map(|byte| top::hex(*byte)).map(char::from).collect();
    for context in [
        format!("Pull request: #{pull}"),
        "Branch: temper/51".into(),
        "Base: g".into(),
        format!("Head: {pinned}"),
        "Title: Change".into(),
        "Description".into(),
        "File: file".into(),
        "change".into(),
    ] {
        assert!(text.contains(&context), "missing pinned context {context}: {text}");
    }
    assert!(text.contains(&format!("What landed in the base: {commits} commits")), "{text}");
    assert_eq!(text.lines().filter(|line| line.starts_with("Base file: ")).count(), files.min(8) as usize, "{text}");
    assert!(text.contains("Base file: shared\nConflicting file: shared\n"), "{text}");
    assert!(text.contains("Base file: base-1"), "{text}");
    if files > 8 {
        assert!(
            text.contains(&format!(
                "[{} more base files not listed: past forge.client.rows (8); read them in your checkout]",
                files - 8
            )),
            "{text}"
        );
    }
    assert!(!text.contains("comparison omitted"), "{text}");
    world.take_calls();
    world.run_for(40);
    assert!(!world.take_calls().iter().any(|op| matches!(op, api::Op::Read(api::Read::Compare { .. }))));
}

#[test]
fn resolver_compare_over_rows_keeps_required_pull_section() {
    for (files, extra, commits) in [(8, 0, 2), (9, 0, 2), (16, 0, 2), (3, 6, 8), (3, 7, 9), (12, 20, 22)] {
        let (mut world, pull, head, base) = resolver(files, extra, false);
        let raw::Answer::Comparison { files: changed, commits: history, .. } = world.inspect(&raw::Read::Compare {
            base: world.branch(b"temper/51"),
            head: world.branch(b"g"),
            page: 1,
            limit: 8,
        }) else {
            panic!("whole fake comparison")
        };
        assert_eq!(changed.len(), usize::try_from(files).expect("file count"));
        assert_eq!(history.len(), commits);
        gather(&mut world, pull, head);
        let calls = world.take_calls();
        assert!(
            !calls.iter().any(|op| matches!(op, api::Op::Read(api::Read::File { head, .. }) if *head == base)),
            "no base contents are read"
        );
        assert_eq!(
            calls.iter().filter(|op| **op == api::Op::Read(api::Read::Compare { before: head, after: base })).count(),
            1,
            "one pinned comparison"
        );
        completed(&mut world, pull, head, files, commits);
    }
}

#[test]
fn resolver_comparison_read_errors_still_fail_the_section() {
    let (mut world, pull, head, absent) = resolver(9, 0, true);
    gather(&mut world, pull, head);
    assert!(world.seen().contains(&top::Request::BriefSized {
        section: SECTION,
        size: Err(top::BriefUnread {
            fate: top::BriefFate::Permanent,
            words: Box::from(b"the forge has no such object".as_slice())
        })
    }));
    assert!(world.take_calls().contains(&api::Op::Read(api::Read::Compare { before: head, after: absent })));
    world.take_seen();
    world.event(top::Event::TakeBrief { section: SECTION });
    assert_eq!(world.seen(), &[top::Request::BriefTaken { section: SECTION, bytes: None }]);
}

#[test]
#[expect(
    clippy::too_many_lines,
    reason = "the review table checks failure and kept-section terminals with their shared setup"
)]
fn a_reviews_lone_pull_read_failure_fails_or_keeps_its_section() {
    type MakePull = fn(&mut World) -> (u64, api::Commit);
    let rows: [(MakePull, &str, api::Error, &str, Option<top::BriefUnread>); 2] = [
        (
            // The fake has no such pull; its Item read answers Missing.
            |_| (17, translate::commit(1)),
            "item",
            api::Error::Missing,
            "the forge has no such object",
            Some(top::BriefUnread {
                fate: top::BriefFate::Permanent,
                words: Box::from(b"the forge has no such object".as_slice()),
            }),
        ),
        (
            |world| {
                let path = b"notes/contracts.md";
                let before = b"contract\n".repeat(256);
                let mut after = before.clone();
                after.extend_from_slice(b"one new clause\n");
                assert!(before.len() < 4096 && after.len() < 4096 && before.len() + after.len() > 4096);
                world.reject_file_sides(api::Excess::at_least(api::Bound::AnswerBytes, 65_536, 65_537));
                world.push_file(b"main", path, &before);
                world.external(
                    raw::Op::Git(raw::Git::Create {
                        branch: b"temper/51".as_slice().into(),
                        commit: world.branch(b"main"),
                    }),
                    2,
                );
                world.push_file(b"temper/51", path, &after);
                let pull = world.open_pull(b"temper/51", b"main");
                (pull, translate::commit(world.branch(b"temper/51")))
            },
            "files",
            api::Error::TooLarge(api::Excess::at_least(api::Bound::AnswerBytes, 65_536, 65_537)),
            "the answer is past",
            None,
        ),
    ];
    for (make_pull, stage, error, reason, failure) in rows {
        let mut world = World::with_goal_room(246, 8, 65_536);
        world.adopt();
        let (pull, head) = make_pull(&mut world);
        let mut row = changes::change_row();
        row.pull = Some(pull);
        row.change.last_head = Some(head);
        row.change.state = change::State::Gating { head, asked: Box::new([]) };
        world.event(top::Event::Change { row });
        gather(&mut world, pull, head);
        let refused = world
            .seen()
            .iter()
            .position(|request| {
                *request
                    == top::Request::BriefReadRefused {
                        section: SECTION,
                        stage,
                        reason,
                        heads: None,
                        error: Some(error),
                    }
            })
            .unwrap_or_else(|| panic!("the refused read is logged: {:?}", world.seen()));
        let sizes: Vec<_> = world
            .seen()
            .iter()
            .enumerate()
            .filter_map(|(index, request)| match request {
                top::Request::BriefSized { section: SECTION, size } => Some((index, size.clone())),
                _ => None,
            })
            .collect();
        assert_eq!(sizes.len(), 1, "one terminal sizing response for {stage}");
        let (terminal_at, size) = sizes[0].clone();
        assert!(refused < terminal_at, "the refusal precedes sizing");
        if let Some(failure) = failure {
            assert_eq!(size, Err(failure));
            world.take_seen();
            world.event(top::Event::TakeBrief { section: SECTION });
            assert_eq!(world.seen(), &[top::Request::BriefTaken { section: SECTION, bytes: None }]);
        } else {
            let size = size.expect("the review keeps its pull section after oversized files");
            assert!(size <= BYTES);
            let calls = world.take_calls();
            assert_eq!(
                calls.iter().filter(|op| matches!(op, api::Op::Read(api::Read::PullFiles { .. }))).count(),
                1,
                "the refused page ends files reads"
            );
            assert!(calls.contains(&api::Op::Read(api::Read::Compare {
                before: translate::commit(world.branch(b"main")),
                after: head,
            })));
            world.take_seen();
            world.event(top::Event::TakeBrief { section: SECTION });
            let [top::Request::BriefTaken { section: SECTION, bytes: Some(words) }] = world.seen() else {
                panic!("the review's pull section must be taken: {:?}", world.seen())
            };
            assert_eq!(words.len(), usize::try_from(size).expect("bounded size"));
            let text = std::str::from_utf8(words).expect("brief text");
            let pinned: String = head[..20].iter().flat_map(|byte| top::hex(*byte)).map(char::from).collect();
            for context in [
                format!("Pull request: #{pull}"),
                "Branch: temper/51".into(),
                "Base: main".into(),
                format!("Head: {pinned}"),
                "Title: Change".into(),
                "Description".into(),
                "[files omitted from page 1: past forge.client.answer_bytes (65536): at least 65537 bytes; read_pull_files from page 2 lists the rest, and read_file reads a path at the head or the base]".into(),
            ] {
                assert!(text.contains(&context), "missing context {context}: {text}");
            }
        }
    }
}
