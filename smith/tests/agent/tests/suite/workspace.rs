//! Optional workspace and mixed mounts on the typed fake LLM world.
//! Contract: smith/docs/design/domain/run-start.md, section 1, and smith/docs/design/domain/run-delivery.md, section 1; smith/docs/design/domain/tools.md, sections 2–4.

use skein_fake_checkout::Checkout;
use skein_fake_llm_domain::api::{Line, Part, Query, Script, Turn};
use skein_io::kernel;
use skein_lib::{Duration, Time, Token};
use smith_agent_world::backend::{Backend, Typed};
use smith_agent_world::{Boundary, Job, Settings, World, translate};
use smith_domain::{Below, Event, Lower, Request, llm, run, tools};
use smith_protocol_machine::{FromDomain, ToDomain};
use smith_protocol_machine_fixtures::{CheckProcess, STDERR, STDOUT};

fn call(name: &[u8], arguments: &[u8]) -> Line {
    Line::Call { name: name.into(), arguments: arguments.into() }
}

fn calls(lines: Vec<Line>, tokens: u64) -> Turn {
    smith_agent_world::fixture::calls(lines.into(), tokens)
}

fn says(text: &[u8], tokens: u64) -> Turn {
    smith_agent_world::fixture::says(Box::new([Line::Text { text: text.into() }]), tokens)
}

fn contains(bytes: &[u8], part: &[u8]) -> bool {
    skein_lib::bytes::find(bytes, part).is_some()
}

#[test]
fn workspace_offers_share_session_schemas_without_changing_served_tools() {
    let names = [b"read".as_slice(), b"list", b"search", b"write", b"edit", b"remove", b"shell", b"job"];
    let mut served = Vec::new();
    for (writable, count) in [(true, names.len()), (false, 3)] {
        let settings = Settings { job: Job::HostTools, writable, ..Settings::calm(0) };
        let mut world = World::new(settings);
        world.run(10_000);
        let query = &world.prompts()[0];
        let grants = tools::Grants { inspect: true, modify: writable, shell: writable };
        let (workspace, host) = query.tools.split_at(count);
        assert_eq!(workspace.iter().map(|tool| tool.name.as_ref()).collect::<Vec<_>>(), names[..count]);
        assert_eq!(workspace, &*smith_session_world::translate::offer(grants));
        let custom = host.iter().find(|tool| tool.name.as_ref() == b"host_action").expect("host tool still offered");
        assert_eq!(custom.description.as_ref(), b"Opaque host write");
        assert_eq!(custom.parameters.as_ref(), br#"{"type":"object"}"#);
        assert!(host.iter().any(|tool| tool.name.as_ref() == b"wait" && tool.parameters.as_ref() == b"{}"));
        if writable {
            served = host.to_vec();
        } else {
            assert_eq!(host, served);
        }
    }
}

#[test]
fn remove_and_offset_read_decode_like_the_session_on_both_paths() {
    const INPUT_BYTES: usize = 128;
    let grants = tools::Grants { inspect: true, modify: true, shell: true };
    let calls = [(b"remove".as_slice(), br#"{"path":"old"}"#.as_slice()), (b"read", br#"{"path":"notes","offset":3}"#)];
    let expected: Vec<_> = calls
        .iter()
        .map(|(name, input)| {
            let smith_domain::session::llm::Decoded::Owned { call } =
                smith_session_world::translate::decode(name, input)
            else {
                panic!("session decodes the fixture's call")
            };
            assert_eq!(
                translate::decode(name, input, grants, &[], llm::Asker::Main),
                llm::Decoded::Owned { call: call.clone() }
            );
            call
        })
        .collect();
    assert!(matches!(&*expected, [tools::Call::Remove { .. }, tools::Call::Read { offset: 3, .. }]));

    let mut settings = Settings::calm(0);
    settings.limits.session.answer_bytes = u64::try_from(INPUT_BYTES).expect("call input bound");
    let mut oversized = calls[0].1.to_vec();
    oversized.resize(INPUT_BYTES + 1, b' ');
    assert!(matches!(
        translate::decode(b"remove", &oversized, grants, &[], llm::Asker::Main),
        llm::Decoded::Owned { .. }
    ));
    let lines = calls.into_iter().map(|(name, input)| call(name, input)).chain([call(b"remove", &oversized)]).collect();
    let scripts =
        Box::new([Script { cue: Box::from(b"@decode".as_slice()), turns: Box::new([self::calls(lines, 1)]) }]);
    let mut typed = Typed::scripted(&settings, scripts);
    let owner = Token::new(1);
    let request = Request::Lower(Lower::Complete {
        owner,
        name: llm::CompletionName { sequence: 1, retry: 0 },
        grant: None,
        prompt: llm::Prompt {
            purpose: llm::Purpose::Ordinary { guard: None },
            asker: llm::Asker::Main,
            endpoint: llm::Endpoint(0),
            affinity: llm::Affinity { key: [0; 16], thread: 0 },
            model: Box::from(b"fake-1".as_slice()),
            effort: None,
            system: Box::from(b"@decode".as_slice()),
            tools: grants,
            namespace: tools::Namespace::One,
            served: Box::default(),
            choice: llm::ToolChoice::Auto,
            messages: Box::new([llm::Message { role: llm::Role::User, content: Box::default() }]),
            max_tokens: 100,
        },
        deadlines: smith_domain::session::Deadlines {
            head: Duration::from_secs(1),
            idle: Duration::from_secs(1),
            whole: Time::from_nanos(1_000_000_000),
        },
        max_completion_bytes: 4096,
        max_completion_blocks: 3,
        max_failure_bytes: 128,
        decoded_call_bytes: 4096,
    });
    let wall = skein_lib::Wall::from_nanos(0);
    typed.start(request, Time::ZERO, wall);
    let mut progress = typed.advance(Time::ZERO, wall, false);
    if let Some(due) = typed.next_deadline() {
        progress.terminals.extend(typed.advance(due, wall, false).terminals);
    }
    typed.assert_settled();
    let [(reply, Event::Below(Below::Completed { owner: completion_owner, completion }))] = &*progress.terminals else {
        panic!("one typed completion: {progress:?}");
    };
    assert_eq!((*reply, *completion_owner), (owner, owner));
    let [
        llm::Said::ToolCall { call: remove, .. },
        llm::Said::ToolCall { call: read, .. },
        llm::Said::ToolCall { call: too_large, .. },
    ] = &*completion.content
    else {
        panic!("three tool calls");
    };
    for (decoded, call) in [remove, read].into_iter().zip(expected) {
        assert_eq!(*decoded, llm::Decoded::Owned { call });
    }
    assert_eq!(*too_large, llm::Decoded::Invalid { problem: llm::Problem::TooLarge });
}

fn mixed_disk() -> (Checkout, run::Workspace) {
    let mut disk = Checkout::new();
    let mut directories = Vec::new();
    for (name, writable, git) in [
        (b"work".as_slice(), true, true),
        (b"archive", false, true),
        (b"notes", true, false),
        (b"reference", false, false),
    ] {
        disk.mkdir(name);
        disk.write(&[name, b"/AGENTS.md"].concat(), &[b"guide-", name].concat());
        disk.write(&[name, b"/.temper/pre-pr"].concat(), b"#!checks");
        disk.write(&[name, b"/src/lib.rs"].concat(), b"pub fn answer() -> u32 { 43 }\n");
        disk.write(&[name, b"/data.txt"].concat(), b"old");
        if git {
            disk.write(&[name, b"/.git/HEAD"].concat(), b"host-owned");
        }
        let root = Token::new(disk.root(name));
        directories.push(run::Directory {
            results: None,
            hooks: None,
            host: Box::from(b"/host".as_slice()),
            name: name.into(),
            root,
            writable,
            git,
            conflicts: Box::new([]),
        });
    }
    (disk, run::Workspace { directories: directories.into() })
}

fn mixed_scripts() -> Box<[Script]> {
    Box::new([
        Script {
            cue: b"@midreport".as_slice().into(),
            turns: vec![
                calls(vec![call(b"write", br#"{"path":"../notes/data.txt","content":"premature"}"#)], 1),
                calls(vec![call(b"read", br#"{"path":"../notes/data.txt"}"#)], 1),
                calls(vec![call(b"edit", br#"{"path":"../notes/data.txt","edits":[{"old":"old","new":"main"}]}"#)], 1),
                calls(
                    vec![call(b"edit", br#"{"path":"../archive/data.txt","edits":[{"old":"old","new":"forbidden"}]}"#)],
                    1,
                ),
                calls(vec![call(b"write", br#"{"path":".git/HEAD","content":"forbidden"}"#)], 1),
                calls(vec![call(b"sub_agent", br#"{"brief":"@mixed-read","tools":["inspect"]}"#)], 1),
                calls(vec![call(b"sub_agent", br#"{"brief":"@mixed-write","tools":["inspect","modify"]}"#)], 1),
                calls(vec![call(b"deliver", br#"{"ticket":"mixed-tree"}"#)], 1),
                calls(vec![call(b"finish", br#"{"report":"Mixed directories handled.","source":"workspace"}"#)], 1),
            ]
            .into(),
        },
        Script {
            cue: b"@mixed-read".as_slice().into(),
            turns: Box::new([
                calls(vec![call(b"read", br#"{"path":"../reference/data.txt"}"#)], 1),
                calls(
                    vec![call(b"edit", br#"{"path":"../notes/data.txt","edits":[{"old":"main","new":"forbidden"}]}"#)],
                    1,
                ),
                says(b"Read-only child done.", 1),
            ]),
        },
        Script {
            cue: b"@mixed-write".as_slice().into(),
            turns: Box::new([
                calls(vec![call(b"write", br#"{"path":"../notes/data.txt","content":"unread-child"}"#)], 1),
                calls(
                    vec![call(b"read", br#"{"path":"../notes/data.txt"}"#), call(b"read", br#"{"path":"data.txt"}"#)],
                    1,
                ),
                calls(
                    vec![call(b"edit", br#"{"path":"../notes/data.txt","edits":[{"old":"main","new":"child"}]}"#)],
                    1,
                ),
                calls(vec![call(b"edit", br#"{"path":"data.txt","edits":[{"old":"old","new":"child-git"}]}"#)], 1),
                calls(
                    vec![call(
                        b"edit",
                        br#"{"path":"../reference/data.txt","edits":[{"old":"old","new":"forbidden"}]}"#,
                    )],
                    1,
                ),
                says(b"Writable child done.", 1),
            ]),
        },
    ])
}

fn feedback_has(query: &Query, part: &[u8]) -> bool {
    query
        .messages
        .iter()
        .flat_map(|message| &message.parts)
        .any(|item| matches!(item, Part::ToolOutput { output, is_error: true, .. } if contains(output, part)))
}

const CHECK_BOUND: Duration = Duration::from_secs(1);
const CHECK_TAIL: u32 = 64;
const MARKER: &[u8] = b"smith-check: not judged\n";

#[derive(Clone, Copy)]
enum CheckEnd {
    Exit(kernel::Exit),
    SplitThenExit,
    Deadline,
    Unstarted,
}

fn check_attempt(reads: &[(Token, &[u8])], end: CheckEnd, bound: Duration) -> run::Ran {
    let request = FromDomain::Check {
        owner: Token::new(1),
        program: run::Place { root: Token::new(1), path: b".temper/pre-pr".as_slice().into() },
        results: None,
        bound,
        cap: Time::ZERO.saturating_add(bound),
        tail: CHECK_TAIL,
    };
    let terminal = if let CheckEnd::Unstarted = end {
        CheckProcess::unstarted(request, Time::ZERO)
    } else {
        let mut process = CheckProcess::start(request, Time::ZERO);
        for &(pipe, bytes) in reads {
            process.bytes(pipe, bytes);
        }
        match end {
            CheckEnd::Exit(exit) => process.exited(exit),
            CheckEnd::SplitThenExit => {
                process.bytes(STDERR, b"smith-check: not ju");
                process.exited(kernel::Exit::Code(1));
                process.bytes(STDERR, b"dged\n");
                process.bytes(STDOUT, &[b'x'; CHECK_TAIL as usize]);
            }
            CheckEnd::Deadline => {
                process.at(process.deadline());
                process.fire();
                process.assert_stopped();
                process.exited(kernel::Exit::Signal(15));
            }
            CheckEnd::Unstarted => unreachable!("spawned"),
        }
        process.finish()
    };
    let ToDomain::Checked { ran, .. } = terminal else { panic!("check terminal: {terminal:?}") };
    ran
}

fn delivery_check_story(finishing: bool, attempts: Vec<run::Ran>, feedback: Option<(&str, &str, u64)>) {
    let (disk, mut workspace) = mixed_disk();
    workspace.directories = workspace.directories.into_vec().into_iter().take(1).collect();
    let mut settings = Settings { job: if finishing { Job::Coding } else { Job::MidReport }, ..Settings::calm(1011) };
    settings.limits.run.check_tail = CHECK_TAIL;
    settings.limits.run.check_timeout = CHECK_BOUND;
    let delivery = || {
        if finishing {
            call(b"finish", br#"{"title":"Answer","body":"Deliver the checked tree."}"#)
        } else {
            call(b"deliver", br#"{"ticket":"checked tree"}"#)
        }
    };
    let mut turns = vec![calls(vec![delivery()], 1)];
    if feedback.is_some() {
        turns.push(calls(vec![delivery()], 1));
    }
    if !finishing {
        turns.push(calls(vec![call(b"finish", br#"{"report":"Done.","source":"workspace"}"#)], 1));
    }
    let cue = if finishing { b"@coding".as_slice() } else { b"@midreport" };
    let scripts = Box::new([Script { cue: cue.into(), turns: turns.into() }]);
    let count = attempts.len();
    let mut world = World::with_workspace_scripts(settings, None, Some(workspace), disk, scripts);
    world.script_checks(attempts);
    if let Some((kind, exit, given_ms)) = feedback {
        let expected =
            format!("{kind} repository=\"work\" check=\".temper/pre-pr\" exit={exit} given_ms={given_ms} cut=");
        let mut answered = false;
        for _ in 0..10_000 {
            assert!(!world.drive(1), "must return feedback before it can deliver");
            assert!(world.pushes().is_empty(), "no delivery before check feedback: {expected}");
            let returned = world
                .prompts()
                .iter()
                .flat_map(|query| &query.messages)
                .flat_map(|message| &message.parts)
                .find_map(|part| match part {
                    Part::ToolOutput { output, is_error: true, .. } if output.starts_with(b"checks-") => Some(output),
                    _ => None,
                });
            if let Some(output) = returned {
                assert!(
                    output.starts_with(expected.as_bytes()),
                    "expected {expected}, got {}",
                    String::from_utf8_lossy(output)
                );
                assert_eq!(world.checked(), vec![false; count], "only deadline expiry retries automatically");
                answered = true;
                break;
            }
        }
        assert!(answered, "missing feedback: {expected}");
    }
    world.run(10_000);
    assert_eq!(world.pushes(), [smith_agent_world::delivered()], "one delivery after a passing check");
    assert_eq!(world.checked().len(), count + usize::from(feedback.is_some()));
    assert!(world.checked().last().copied().expect("a check ran"));
    assert!(matches!(world.answer(), run::Answer::Accepted { .. }));
}

#[test]
fn delivery_and_finish_distinguish_a_check_refusal_from_no_verdict() {
    use kernel::Exit::{Code, Signal};
    let empty: &[(Token, &[u8])] = &[];
    let stderr_marker = [(STDERR, MARKER)];
    let refused = Some(("checks-failed", "code:1", 1000));
    let not_judged = Some(("checks-not-judged", "code:1", 1000));
    for finishing in [false, true] {
        for (reads, end, expected) in [
            (&stderr_marker[..], CheckEnd::Exit(Code(1)), not_judged),
            (empty, CheckEnd::SplitThenExit, not_judged),
            (empty, CheckEnd::Exit(Code(1)), refused),
            ([(STDOUT, MARKER)].as_slice(), CheckEnd::Exit(Code(1)), refused),
            ([(STDERR, b"xsmith-check: not judged\n".as_slice())].as_slice(), CheckEnd::Exit(Code(1)), refused),
            ([(STDERR, b"smith-check: not judged x\n".as_slice())].as_slice(), CheckEnd::Exit(Code(1)), refused),
            ([(STDERR, b"smith-check: not judged\r\n".as_slice())].as_slice(), CheckEnd::Exit(Code(1)), refused),
            ([(STDERR, b"smith-check: not judged".as_slice())].as_slice(), CheckEnd::Exit(Code(1)), refused),
            (&stderr_marker[..], CheckEnd::Exit(Code(0)), None),
            (empty, CheckEnd::Exit(Signal(9)), Some(("checks-not-judged", "signalled", 1000))),
            (empty, CheckEnd::Unstarted, Some(("checks-not-judged", "unstarted", 1000))),
        ] {
            delivery_check_story(finishing, vec![check_attempt(reads, end, CHECK_BOUND)], expected);
        }
        let twice = [CHECK_BOUND, CHECK_BOUND.saturating_add(CHECK_BOUND)]
            .map(|bound| check_attempt(empty, CheckEnd::Deadline, bound));
        delivery_check_story(finishing, Vec::from(twice), Some(("checks-not-judged", "timed-out", 2000)));
    }
}

#[test]
fn a_delivery_to_three_directories_failing_in_the_second_names_that_directory() {
    let mut disk = Checkout::new();
    let mut directories = Vec::new();
    for name in [b"one".as_slice(), b"two", b"three"] {
        disk.mkdir(name);
        disk.write(&[name, b"/.temper/pre-pr"].concat(), b"#!checks");
        disk.write(&[name, b"/src/lib.rs"].concat(), b"pub fn answer() -> u32 { 43 }\n");
        directories.push(run::Directory {
            results: None,
            hooks: None,
            host: Box::from(b"/host".as_slice()),
            name: name.into(),
            root: Token::new(disk.root(name)),
            writable: true,
            git: false,
            conflicts: Box::new([]),
        });
    }
    let workspace = run::Workspace { directories: directories.into() };
    let mut settings = Settings { job: Job::MidReport, ..Settings::calm(1003) };
    settings.limits.run.directories = 3;
    settings.limits.session.tools.repos = 3;
    let scripts = Box::new([Script {
        cue: b"@midreport".as_slice().into(),
        turns: Box::new([
            calls(vec![call(b"deliver", br#"{"ticket":"three-directory-tree"}"#)], 1),
            calls(vec![call(b"finish", br#"{"report":"Delivery failed.","source":"workspace"}"#)], 1),
        ]),
    }]);
    let mut world = World::with_workspace_scripts(settings, None, Some(workspace), disk, scripts);
    world.enable_parent_deliveries();
    for _ in 0..10_000 {
        assert!(!world.drive(1));
        if !world.delivery_submissions().is_empty() {
            break;
        }
    }
    assert_eq!(world.delivery_submissions().len(), 1);
    let owner = world.delivery_submissions()[0].owner;
    let failure = run::DeliveryFailure::new(1, run::DeliveryReason::RefusedByTarget);
    world.return_delivery(owner, run::Delivery::Failed(failure)).expect("one failed delivery terminal");
    world.run(10_000);
    assert!(world.prompts().iter().any(|query| feedback_has(query, b"delivery-failed directory=1")));
    assert!(matches!(world.answer(), run::Answer::Accepted { outcome: run::outcome::Declared::Report(_), .. }));
}

#[test]
fn mixed_git_plain_and_readonly_mounts_share_discovery_and_child_authority() {
    let (disk, workspace) = mixed_disk();
    let readonly_git = disk.tree(b"archive");
    let readonly_plain = disk.tree(b"reference");
    let git_head = disk.content(b"work/.git/HEAD").expect("host-owned git metadata").to_vec();
    let roots = workspace.directories.iter().map(|directory| directory.root).collect::<Vec<_>>();
    let mut settings = Settings { job: Job::MidReport, ..Settings::calm(1002) };
    settings.limits.run.directories = 4;
    settings.limits.session.tools.repos = 4;
    let mut world = World::with_workspace_scripts(settings, None, Some(workspace), disk, mixed_scripts());
    world.enable_parent_deliveries();
    for _ in 0..10_000 {
        assert!(!world.drive(1));
        if !world.delivery_submissions().is_empty() {
            break;
        }
    }
    assert_eq!(world.delivery_submissions().len(), 1, "parent delivery reached");
    assert_eq!(world.checked(), [true, true]);
    assert_eq!(world.disk().content(b"work/data.txt"), Some(b"child-git".as_slice()));
    assert_eq!(world.disk().content(b"notes/data.txt"), Some(b"child".as_slice()));
    assert_eq!(world.disk().tree(b"archive"), readonly_git);
    assert_eq!(world.disk().tree(b"reference"), readonly_plain);
    assert_eq!(world.disk().content(b"work/.git/HEAD"), Some(git_head.as_slice()));
    assert!(!world.disk().exists(b"notes/.git") && !world.disk().exists(b"reference/.git"));
    let owner = world.delivery_submissions()[0].owner;
    let receipts = run::Delivered::new(Box::new([
        run::Receipt::new(0, b"work: checked git files kept".as_slice().into()).expect("git receipt"),
        run::Receipt::new(2, b"notes: changed plain files kept".as_slice().into()).expect("plain receipt"),
    ]))
    .expect("exact changed writable ordinals");
    world.return_delivery(owner, run::Delivery::Delivered(receipts)).expect("one terminal");
    world.run(10_000);
    assert!(matches!(world.answer(), run::Answer::Accepted { outcome: run::outcome::Declared::Report(_), .. }));
    let discovery = world
        .boundaries()
        .iter()
        .filter_map(|(_, boundary)| match boundary {
            Boundary::Read { at } => Some((b"read".as_slice(), at.root, at.path.as_ref())),
            Boundary::Probe { at } => Some((b"probe".as_slice(), at.root, at.path.as_ref())),
            Boundary::Check { .. } | Boundary::Io { .. } => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(
        discovery,
        [
            (b"read".as_slice(), roots[0], b"AGENTS.md".as_slice()),
            (b"probe", roots[0], b".temper/pre-pr"),
            (b"read", roots[1], b"AGENTS.md"),
            (b"read", roots[2], b"AGENTS.md"),
            (b"probe", roots[2], b".temper/pre-pr"),
            (b"read", roots[3], b"AGENTS.md"),
        ]
    );
    let checks = world
        .boundaries()
        .iter()
        .filter_map(|(_, boundary)| match boundary {
            Boundary::Check { program } => Some(program.root),
            Boundary::Read { .. } | Boundary::Probe { .. } | Boundary::Io { .. } => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(checks, [roots[0], roots[2]], "readonly executable files never become checks");
    for query in world.prompts() {
        for name in [b"work".as_slice(), b"archive", b"notes", b"reference"] {
            assert!(contains(&query.system, &[b"guide-", name].concat()), "same guides in main and children");
        }
        for metadata in [
            b"`work`, which you may change, a git working tree".as_slice(),
            b"`archive`, which you may only read, a git working tree",
            b"`notes`, which you may change, a plain directory",
            b"`reference`, which you may only read, a plain directory",
        ] {
            assert!(contains(&query.system, metadata), "identical kind and write metadata in main and children");
        }
    }
    let main =
        world.prompts().iter().filter(|query| query.system.starts_with(b"## Task\n\n@midreport")).collect::<Vec<_>>();
    assert!(main.iter().any(|query| feedback_has(query, b"NotRead")));
    assert!(main.iter().any(|query| feedback_has(query, b"ReadOnly")));
    assert!(main.iter().any(|query| feedback_has(query, b"Protected")));
    let readonly = world.prompts().iter().filter(|query| query.system.starts_with(b"@mixed-read")).collect::<Vec<_>>();
    assert_eq!(readonly.len(), 3);
    assert!(readonly.iter().all(|query| query.tools.iter().all(|tool| {
        ![b"edit".as_slice(), b"write", b"finish", b"wait", b"host_action"].contains(&tool.name.as_ref())
    })));
    assert!(readonly.iter().any(|query| feedback_has(query, b"UnknownTool")));
    let writable = world.prompts().iter().filter(|query| query.system.starts_with(b"@mixed-write")).collect::<Vec<_>>();
    assert!(writable.iter().any(|query| feedback_has(query, b"NotRead")), "main's knowledge is not child's");
    assert!(writable.iter().any(|query| feedback_has(query, b"ReadOnly")), "child cannot widen mount authority");
}
