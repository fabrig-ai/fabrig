//! Temper's prepare in the whole worker world (temper/docs/design/domain/worker/jobs.md).
use super::jobs::{Report, World};
use jig_domain_link::{JobBound, JobKind};
use skein_lib::{Duration, Token};
use temper_fake_checkout::git::Remote;
use temper_worker_checkout_world::translate;
use temper_worker_domain::{Event, host, tooling, wire};

const REMOTE: &[u8] = b"org/jobs";
const BRANCH: &[u8] = b"temper/7/l81";
const MERGE: &[u8] = b"Merge pull request 'feature' (#12) from topic into main\n\nLanded by landing 81 for change 12.";
const SQUASH: &[u8] = b"feature\n\nTemper-Change: 12\nTemper-Pull: 12\nTemper-Head: ";

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Case {
    Merge,
    ContainedMerge,
    Squash,
    EmptySquash,
    Upstream,
    NoContribution,
    ConflictMerge,
    ConflictSquash,
    Existing,
    TwoMembers,
}

/// One typed assignment, no process, its branch/result, acknowledgement and shutdown.
#[must_use]
#[expect(clippy::too_many_lines, reason = "one prepare story checks its full lifecycle")]
pub fn run(seed: u64, case: Case) -> Report {
    let kind = tooling::Kind {
        bound: JobBound { kind: JobKind(1), spec: 16_384, result: 4_096, deadline: Duration::from_secs(600) },
        verb: b"prepare".as_slice().into(),
        delivers: true,
    };
    let mut limits = crate::limits();
    limits.tooling.file_bytes = 16_384;
    limits.checkout.conflicts = 8;
    limits.checkout.path_bytes = 64;
    let mut world = World::new(seed, kind.clone(), limits);
    world.repository = b"work".as_slice().into();
    let original = world.forge.branch(REMOTE, b"main").expect("main");
    let conflict = matches!(case, Case::ConflictMerge | Case::ConflictSquash);
    let mut head = world.forge.advance(REMOTE, b"topic", if conflict { b"code" } else { b"feature" }, b"head\n");
    let base = world.forge.advance(REMOTE, b"main", if conflict { b"code" } else { b"base" }, b"base\n");
    let upstream = if matches!(case, Case::Upstream | Case::NoContribution) {
        world.forge.create_branch(REMOTE, b"upstream", original).expect("upstream");
        let upstream = world.forge.advance(REMOTE, b"upstream", b"upstream", b"upstream\n");
        if case == Case::Upstream {
            let mut tree = world.forge.tree(head);
            tree.insert(b"upstream".to_vec(), b"upstream\n".to_vec());
            head = world.forge.store(head, Some(upstream), true, tree, b"catch up").expect("catch up commit");
            world.forge.push(REMOTE, b"topic", head, None).expect("catch up published for fetching");
        }
        Some(upstream)
    } else {
        None
    };
    if matches!(case, Case::EmptySquash | Case::ContainedMerge) {
        head = original;
    }
    let squash =
        !matches!(case, Case::Merge | Case::ContainedMerge | Case::ConflictMerge | Case::Existing | Case::TwoMembers);
    let by_squash = squash && case != Case::Upstream;
    let squash_message = [SQUASH, temper_fake_id(head).as_slice()].concat();
    world.title = if by_squash { squash_message.clone().into() } else { MERGE.into() };
    if case == Case::Existing {
        world.forge.create_branch(REMOTE, BRANCH, base).expect("existing landing");
    }
    let id = |commit| String::from_utf8(temper_fake_id(commit)).expect("hex");
    let member = format!(
        "{{\"number\":12,\"branch\":\"topic\",\"head\":\"{}\",\"message\":\"{}\",\"squash\":\"{}\"}}",
        id(head),
        escaped(MERGE),
        escaped(&squash_message)
    );
    let members = if case == Case::TwoMembers { format!("{member},{member}") } else { member };
    let upstream_text =
        upstream.map_or_else(String::new, |upstream| format!(",\"train_upstream\":\"{}\"", id(upstream)));
    let request = format!(
        "{{\"version\":1,\"repository\":\"work\",\"main\":\"{}\",\"combine\":\"{}\"{upstream_text},\"members\":[{members}],\"upstreams\":[],\"gates\":[]}}",
        id(base),
        if squash { "squash" } else { "merge" }
    );
    world.event(Event::Ready);
    assert_eq!(world.report.kinds.as_ref(), [kind.bound]);
    world.event(Event::AssignJob {
        job: host::JobAssignment {
            run: Token::new(81),
            attempt: Token::new(13),
            kind: JobKind(1),
            spec: b"encoded spec".as_slice().into(),
            deadline: Duration::from_secs(600),
            grants: Box::new([]),
            workspace: Some(host::Workspace { workstream: 81, items: Token::new(0) }),
        },
        items: Ok(wire::Workspace {
            repositories: Box::new([wire::Repository {
                tag: 0,
                name: b"work".as_slice().into(),
                remote: REMOTE.into(),
                save: None,
                git: None,
                start: wire::Start::Commit { commit: translate::commit(base).raw() },
                access: wire::Access::Writable { push: BRANCH.into(), expected: None, landing: None },
                identity: 7,
                account: None,
            }]),
        }),
        spec: Ok(wire::JobSpec { command: Box::new([]), request: request.into_bytes().into() }),
    });
    assert!(world.operations.is_empty(), "temper's prepare runs no file/process tooling operation");
    let answer = world.report.answer.as_ref().expect("prepare answer");
    if case == Case::TwoMembers {
        let host::JobEnding::Refused(host::JobRefusal::Invalid(_)) = answer else { panic!("invalid: {answer:?}") };
        assert_eq!(world.report.pushes, 0);
        assert_eq!(world.forge.branch(REMOTE, BRANCH), None);
    } else if case == Case::Existing {
        let host::JobEnding::Failed { failure: host::JobFailure::Undelivered(_), .. } = answer else {
            panic!("undelivered: {answer:?}")
        };
        assert_eq!(world.forge.branch(REMOTE, BRANCH), Some(base));
    } else {
        let host::JobEnding::Finished { result, work } = answer else { panic!("finished: {answer:?}") };
        if conflict {
            assert_eq!(result.as_ref(), br#"{"conflict":{"member":12,"with":"branch","files":["code"],"cut":false}}"#);
            assert_eq!(world.report.pushes, 0);
            assert_eq!(world.forge.branch(REMOTE, BRANCH), None);
            assert!(work.left.is_none());
        } else {
            assert_eq!(
                result.as_ref(),
                if by_squash {
                    br#"{"combined":[{"member":12,"by":"squash"}]}"#.as_slice()
                } else {
                    br#"{"combined":[{"member":12,"by":"merge"}]}"#
                }
            );
            let candidate = world.forge.branch(REMOTE, BRANCH).expect("new landing branch");
            assert_ne!(candidate, base, "even an empty squash commits");
            assert_eq!(world.forge.parent(candidate), Some(base));
            assert_eq!(world.forge.merge_parent(candidate), if by_squash { None } else { Some(head) });
            assert_eq!(world.forge.message(candidate), if by_squash { squash_message.as_slice() } else { MERGE });
            let mut expected = world.forge.tree(base);
            if !matches!(case, Case::EmptySquash | Case::ContainedMerge) {
                expected.insert(b"feature".to_vec(), b"head\n".to_vec());
            }
            if case == Case::Upstream {
                expected.insert(b"upstream".to_vec(), b"upstream\n".to_vec());
            }
            assert_eq!(world.forge.tree(candidate), expected, "merge and squash have the merged tree");
            assert_eq!(world.report.pushes, 1);
            assert!(work.left.is_some());
            world.report.tree = expected;
        }
    }
    assert_eq!(world.domain.workspaces(), 0);
    assert_eq!(world.domain.agent().agents(), 0);
    world.event(Event::Link(host::LinkDown::Acknowledge { run: Token::new(81), attempt: Token::new(13) }));
    assert_eq!(world.domain.kept_answers(), 0);
    world.event(Event::Shutdown);
    assert_eq!(world.stops, 1);
    world.report
}
fn temper_fake_id(commit: u64) -> Vec<u8> {
    let raw = translate::commit(commit).raw();
    raw[..20].iter().flat_map(|byte| format!("{byte:02x}").into_bytes()).collect()
}
fn escaped(bytes: &[u8]) -> String {
    String::from_utf8(bytes.to_vec()).expect("fixture").replace('\\', "\\\\").replace('"', "\\\"").replace('\n', "\\n")
}
