//! Temper's prepare in the whole worker world (temper/docs/design/domain/worker/jobs.md).
use super::jobs::{Report, World};
use jig_domain_link::{JobBound, JobKind};
use skein_lib::{Duration, Token};
use temper_fake_checkout::git::Remote;
use temper_worker_checkout_world::translate;
use temper_worker_domain::{Event, host, tooling, wire};

const REMOTE: &[u8] = b"org/jobs";
const BRANCH: &[u8] = b"temper/7/l81";
const MERGE: &[u8] = b"feature\n\nAdds the feature.\n\nChange: 12\nPull-request: 12\nHead: ";
const SQUASH: &[u8] = b"feature\n\nAdds the feature.\n\nChange: 12\nPull-request: 12\nHead: ";

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
    StackMerge,
    StackSquash,
    KeptPrefix,
    KeptUpstream,
    BranchConflict,
    MemberConflict,
    FirstConflict,
    AllConflict,
    ExistingPrefix,
    WrongPrefix,
    FewerPrefixes,
    MorePrefixes,
    MissingPrefix,
    MultipleWritable,
}

/// One typed assignment, no process, its branch/result, acknowledgement and shutdown.
#[must_use]
#[expect(clippy::too_many_lines, reason = "one prepare story checks its full lifecycle")]
pub fn run(seed: u64, case: Case) -> Report {
    if !matches!(
        case,
        Case::Merge
            | Case::ContainedMerge
            | Case::Squash
            | Case::EmptySquash
            | Case::Upstream
            | Case::NoContribution
            | Case::ConflictMerge
            | Case::ConflictSquash
            | Case::Existing
    ) {
        return run_stack(seed, case);
    }
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
    let squash = !matches!(case, Case::Merge | Case::ContainedMerge | Case::ConflictMerge | Case::Existing);
    let by_squash = squash && case != Case::Upstream;
    let merge_message = [MERGE, temper_fake_id(head).as_slice(), b"\nGoal: 7"].concat();
    let squash_message = [SQUASH, temper_fake_id(head).as_slice(), b"\nGoal: 7"].concat();
    world.title = if by_squash { squash_message.clone().into() } else { merge_message.clone().into() };
    if case == Case::Existing {
        world.forge.create_branch(REMOTE, BRANCH, base).expect("existing landing");
    }
    let id = |commit| String::from_utf8(temper_fake_id(commit)).expect("hex");
    let member = format!(
        "{{\"number\":12,\"branch\":\"topic\",\"head\":\"{}\",\"message\":\"{}\",\"squash\":\"{}\"}}",
        id(head),
        escaped(&merge_message),
        escaped(&squash_message)
    );
    let members = member;
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
    if case == Case::Existing {
        let host::JobEnding::Failed { failure: host::JobFailure::Undelivered(_), .. } = answer else {
            panic!("undelivered: {answer:?}")
        };
        assert_eq!(world.forge.branch(REMOTE, BRANCH), Some(base));
    } else {
        let host::JobEnding::Finished { result, work } = answer else { panic!("finished: {answer:?}") };
        if conflict {
            assert_eq!(
                result.as_ref(),
                br#"{"conflicts":[{"member":12,"with":"branch","files":["code"],"cut":false}]}"#
            );
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
            assert_eq!(
                world.forge.message(candidate),
                if by_squash { squash_message.as_slice() } else { merge_message.as_slice() }
            );
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
    world.event(Event::Shutdown { drain: true });
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

#[expect(clippy::too_many_lines, reason = "a stack story checks the request, every prefix and the final lifecycle")]
fn run_stack(seed: u64, case: Case) -> Report {
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
    let topics = [b"topic".as_slice(), b"topic2", b"topic3"];
    for topic in topics.iter().skip(1) {
        world.forge.create_branch(REMOTE, topic, original).expect("member branch");
    }
    let mut heads = Vec::new();
    for (index, topic) in topics.iter().enumerate() {
        let path: &[u8] = if case == Case::AllConflict || (case == Case::FirstConflict && index == 0) {
            b"code"
        } else if index < 2 && matches!(case, Case::MemberConflict | Case::BranchConflict) {
            b"member-conflict"
        } else {
            topic
        };
        heads.push(world.forge.advance(REMOTE, topic, path, topic));
    }
    if case == Case::BranchConflict {
        heads[1] = world.forge.advance(REMOTE, topics[1], b"branch-conflict", b"head2");
    }
    let base_path: &[u8] = if matches!(case, Case::FirstConflict | Case::AllConflict) {
        b"code".as_slice()
    } else if case == Case::BranchConflict {
        b"branch-conflict"
    } else {
        b"base"
    };
    let base = world.forge.advance(REMOTE, b"main", base_path, b"base\n");
    let mut from = base;
    let mut upstream = None;
    if matches!(case, Case::KeptPrefix | Case::KeptUpstream) {
        let mut tree = world.forge.tree(base);
        tree.insert(b"kept".to_vec(), b"kept prefix".to_vec());
        let mut second = None;
        if case == Case::KeptUpstream {
            world.forge.create_branch(REMOTE, b"upstream", original).expect("upstream");
            let tip = world.forge.advance(REMOTE, b"upstream", b"upstream", b"upstream\n");
            upstream = Some(tip);
            second = Some(tip);
            tree.insert(b"upstream".to_vec(), b"upstream\n".to_vec());
            let mut head_tree = world.forge.tree(heads[0]);
            head_tree.insert(b"upstream".to_vec(), b"upstream\n".to_vec());
            heads[0] = world.forge.store(heads[0], Some(tip), true, head_tree, b"catch up").expect("catch up");
            world.forge.push(REMOTE, topics[0], heads[0], None).expect("publish head");
        }
        from = world.forge.store(base, second, true, tree, b"kept prefix").expect("kept prefix");
        world.forge.push(REMOTE, b"kept", from, None).expect("fetched kept prefix");
    }
    let squash = matches!(case, Case::StackSquash | Case::KeptPrefix | Case::KeptUpstream);
    let branches = [BRANCH, b"temper/7/l81-2", b"temper/7/l81-3"];
    if case == Case::ExistingPrefix {
        world.forge.create_branch(REMOTE, branches[1], base).expect("existing b2");
    }
    let id = |commit| String::from_utf8(temper_fake_id(commit)).expect("hex");
    let messages: Vec<_> =
        topics.iter().enumerate().map(|(i, topic)| format!("Merge member {} from {}", i + 1, escaped(topic))).collect();
    let squashes: Vec<_> = heads
        .iter()
        .enumerate()
        .map(|(i, head)| {
            format!(
                "member {}\n\nTemper-Change: {}\nTemper-Pull: {}\nTemper-Head: {}",
                i + 1,
                i + 10,
                i + 20,
                id(*head)
            )
        })
        .collect();
    let members: Vec<_> = heads
        .iter()
        .enumerate()
        .map(|(i, head)| {
            let prefix = if case == Case::MissingPrefix && i == 1 {
                String::new()
            } else {
                format!(
                    ",\"prefix\":\"{}\"",
                    if case == Case::WrongPrefix && i == 1 { "different".to_owned() } else { escaped(branches[i]) }
                )
            };
            format!(
                "{{\"number\":{},\"branch\":\"{}\",\"head\":\"{}\",\"message\":\"{}\",\"squash\":\"{}\"{prefix}}}",
                i + 1,
                escaped(topics[i]),
                id(*head),
                escaped(messages[i].as_bytes()),
                escaped(squashes[i].as_bytes())
            )
        })
        .collect();
    let upstream_text = upstream.map_or_else(String::new, |tip| format!(",\"train_upstream\":\"{}\"", id(tip)));
    let request = format!(
        "{{\"version\":1,\"repository\":\"work\",\"main\":\"{}\",\"from\":\"{}\",\"combine\":\"{}\"{upstream_text},\"members\":[{}],\"upstreams\":[],\"gates\":[]}}",
        id(base),
        id(from),
        if squash { "squash" } else { "merge" },
        members.join(",")
    );
    let mut granted: Vec<Box<[u8]>> = branches.iter().map(|branch| Box::from(*branch)).collect();
    if case == Case::FewerPrefixes {
        granted.pop();
    }
    if case == Case::MorePrefixes {
        granted.push(b"extra".as_slice().into());
    }
    let access = if case == Case::MultipleWritable {
        wire::Access::Writable { push: BRANCH.into(), expected: None, landing: None }
    } else {
        wire::Access::Prefixes { branches: granted.into_boxed_slice() }
    };
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
                start: wire::Start::Commit { commit: translate::commit(from).raw() },
                access,
                identity: 7,
                account: None,
            }]),
        }),
        spec: Ok(wire::JobSpec { command: Box::new([]), request: request.into_bytes().into() }),
    });
    assert!(world.operations.is_empty(), "own prepare runs no tooling process");
    let answer = world.report.answer.as_ref().expect("prepare answer");
    let invalid = matches!(
        case,
        Case::WrongPrefix | Case::FewerPrefixes | Case::MorePrefixes | Case::MissingPrefix | Case::MultipleWritable
    );
    if invalid {
        let host::JobEnding::Refused(host::JobRefusal::Invalid(host::JobInvalid::Start { .. })) = answer else {
            panic!("invalid: {answer:?}")
        };
        assert_eq!(world.report.pushes, 0);
        for branch in branches {
            assert_eq!(world.forge.branch(REMOTE, branch), None);
        }
    } else {
        let mut combined = Vec::new();
        let mut conflicts = Vec::new();
        let mut parent = from;
        let mut tree = world.forge.tree(from);
        for i in 0..3 {
            let conflicted = case == Case::AllConflict
                || (case == Case::FirstConflict && i == 0)
                || (matches!(case, Case::BranchConflict | Case::MemberConflict) && i == 1);
            if conflicted {
                assert_eq!(world.forge.branch(REMOTE, branches[i]), None);
                let with = if case == Case::MemberConflict { "member" } else { "branch" };
                let file = if case == Case::MemberConflict {
                    "member-conflict"
                } else if case == Case::BranchConflict {
                    "branch-conflict"
                } else {
                    "code"
                };
                conflicts
                    .push(format!("{{\"member\":{},\"with\":\"{with}\",\"files\":[\"{file}\"],\"cut\":false}}", i + 1));
                continue;
            }
            let by_squash = squash && !(case == Case::KeptUpstream && i == 0);
            combined.push(format!(
                "{{\"member\":{},\"by\":\"{}\"}}",
                i + 1,
                if by_squash { "squash" } else { "merge" }
            ));
            if case == Case::ExistingPrefix && i >= 1 {
                assert_eq!(world.forge.branch(REMOTE, branches[i]), if i == 1 { Some(base) } else { None });
                continue;
            }
            let prefix = world.forge.branch(REMOTE, branches[i]).expect("combined member's branch");
            assert_eq!(world.forge.parent(prefix), Some(parent), "rolling prefix parent");
            assert_eq!(world.forge.merge_parent(prefix), if by_squash { None } else { Some(heads[i]) });
            assert_eq!(
                world.forge.message(prefix),
                if by_squash { squashes[i].as_bytes() } else { messages[i].as_bytes() }
            );
            for (path, value) in world.forge.tree(heads[i]) {
                if world.forge.tree(original).get(&path) != Some(&value) {
                    tree.insert(path, value);
                }
            }
            assert_eq!(world.forge.tree(prefix), tree, "each prefix has its merge tree");
            parent = prefix;
        }
        if case == Case::ExistingPrefix {
            let host::JobEnding::Failed { failure: host::JobFailure::Undelivered(_), .. } = answer else {
                panic!("undelivered: {answer:?}")
            };
            assert_eq!(world.report.pushes, 2, "stop at b2, preserving b1");
        } else {
            let host::JobEnding::Finished { result, work } = answer else { panic!("finished: {answer:?}") };
            let mut fields = Vec::new();
            if !combined.is_empty() {
                fields.push(format!("\"combined\":[{}]", combined.join(",")));
            }
            if !conflicts.is_empty() {
                fields.push(format!("\"conflicts\":[{}]", conflicts.join(",")));
            }
            assert_eq!(core::str::from_utf8(result).expect("JSON"), format!("{{{}}}", fields.join(",")));
            assert_eq!(world.report.pushes as usize, combined.len());
            assert_eq!(work.left.is_some(), !combined.is_empty());
        }
        if case == Case::FirstConflict || case == Case::AllConflict {
            let merges = world
                .report
                .trace
                .iter()
                .filter(|line| line.starts_with("<- Io") && line.contains("op: Merge {"))
                .count();
            assert_eq!(merges, 3, "a conflict directly on main needs no second merge");
        }
        world.report.tree = tree;
    }
    assert_eq!(world.domain.workspaces(), 0);
    assert_eq!(world.domain.agent().agents(), 0);
    world.event(Event::Link(host::LinkDown::Acknowledge { run: Token::new(81), attempt: Token::new(13) }));
    assert_eq!(world.domain.kept_answers(), 0);
    world.event(Event::Shutdown { drain: true });
    assert_eq!(world.stops, 1);
    world.report
}
