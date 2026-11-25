//! Private request files beside an otherwise simulated agent process.
use skein_scratch::Scratch;
use smith_agent_process_world::{
    PRIVATE_GRANT, World, program_world,
    testing::dump::{BOUND, SEED, TURNS, accepted, dump_errors, files, finish, policy, world},
    wire,
};
use smith_channel::RunResult;
use std::{fs, os::unix::fs::PermissionsExt, path::PathBuf};

fn named<'a>(files: &'a [(PathBuf, Vec<u8>)], suffix: &str) -> &'a [u8] {
    &files
        .iter()
        .find(|(path, _)| path.file_name().expect("dump filename").to_str().expect("UTF-8 dump path").ends_with(suffix))
        .expect("dump file")
        .1
}

#[test]
fn a_request_s_file_holds_the_bytes_the_provider_read() {
    for (anthropic, suffix) in [(false, "-openai.json"), (true, "-anthropic.json")] {
        let scratch = Scratch::new("request-dump-body");
        let directory = scratch.path().join("private/dumps");
        let mut world = world(directory.clone(), BOUND, anthropic, true);
        assert!(world.settle());
        accepted(&world);
        let files = files(&directory);
        let requests: Vec<_> =
            files.iter().filter(|(path, _)| path.to_str().expect("UTF-8 dump path").ends_with(suffix)).collect();
        assert_eq!(requests.len(), usize::try_from(TURNS).expect("valid dump fixture"));
        assert_eq!(world.bodies().len(), requests.len());
        for ((path, bytes), body) in requests.into_iter().zip(world.bodies()) {
            assert_eq!(bytes.as_slice(), body.as_ref(), "{}", path.display());
        }
        for (path, bytes) in files {
            assert!(skein_lib::bytes::find(&bytes, PRIVATE_GRANT).is_none());
            assert_eq!(fs::metadata(&path).expect("dump file metadata").permissions().mode() & 0o777, 0o600);
            assert!(!path.file_name().expect("dump filename").to_str().expect("UTF-8 dump path").starts_with(".dump-"));
        }
        assert_eq!(fs::metadata(&directory).expect("dump directory metadata").permissions().mode() & 0o777, 0o700);
        assert_eq!(
            fs::metadata(directory.parent().expect("nested dump directory"))
                .expect("valid dump fixture")
                .permissions()
                .mode()
                & 0o777,
            0o700
        );
        assert_eq!(dump_errors(&world).len(), 0);
    }
}

#[test]
fn a_run_s_start_file_says_what_it_was_given() {
    let scratch = Scratch::new("request-dump-start");
    let directory = scratch.path().join("dumps");
    let charter = wire::edit_charter(&policy(), |parts| {
        let mut tools = parts.tools.clone();
        let mut families = tools.families;
        families.inspect = true;
        tools.families = families;
        parts.tools = tools;
        parts.brief = skein_lib::List::with_capacity(1);
        parts
            .brief
            .push(smith_charter::Section {
                title: b"Context".as_slice().into(),
                body: b"literal brief".as_slice().into(),
            })
            .expect("valid dump fixture");
        parts.models = skein_lib::List::with_capacity(1);
        parts.models.push(parts.main.clone()).expect("valid dump fixture");
    });
    let mut world = World::new(SEED, &charter);
    world.script(Box::new([finish()]));
    world.dump_to(directory.clone(), BOUND);
    assert!(world.settle());
    accepted(&world);
    let files = files(&directory);
    let start = std::str::from_utf8(named(&files, "-start.txt")).expect("valid dump fixture");
    for line in [
        "mode: finite",
        "resume: false",
        "waiting: 0 ns",
        "budget turns: 2",
        "budget spend: 20",
        "budget time: 60000000000 ns",
        "reserve turns: 0",
        "reserve time: 0 ns",
        "main model: fake",
        "main effort: default",
        "inspect: true",
        "modify: false",
        "shell: false",
        "agents: false",
        "wait: false",
        "deliver: false",
        "brief title: Context",
        "sub-agent model: fake",
        "sub-agent effort: default",
    ] {
        assert!(start.lines().any(|found| found == line), "missing {line}: {start}");
    }
}

#[test]
fn a_directory_that_cannot_be_made_leaves_the_run_answering() {
    let scratch = Scratch::new("request-dump-blocked");
    let blocker = scratch.path().join("file");
    fs::write(&blocker, b"regular file").expect("valid dump fixture");
    let mut world = world(blocker.join("dumps"), BOUND, false, false);
    assert!(world.settle());
    accepted(&world);
    // Started and the single model request each attempted one write.
    assert_eq!(dump_errors(&world).len(), 2);
    assert_eq!(fs::read(&blocker).expect("valid dump fixture"), b"regular file");
}

#[test]
fn a_directory_open_to_others_is_refused() {
    let scratch = Scratch::new("request-dump-mode");
    let directory = scratch.path().join("dumps");
    fs::create_dir(&directory).expect("valid dump fixture");
    fs::set_permissions(&directory, fs::Permissions::from_mode(0o755)).expect("valid dump fixture");
    let mut world = world(directory.clone(), BOUND, false, false);
    assert!(world.settle());
    accepted(&world);
    assert_eq!(files(&directory).len(), 0);
    assert_eq!(dump_errors(&world).len(), 2);
}

#[test]
fn the_bound_stops_the_writing_with_one_line() {
    let scratch = Scratch::new("request-dump-bound");
    let probe = scratch.path().join("probe");
    let mut first = world(probe.clone(), BOUND, false, false);
    assert!(first.settle());
    let measured = files(&probe);
    let size = u64::try_from(named(&measured, "-start.txt").len()).expect("valid dump fixture");
    let directory = scratch.path().join("bounded");
    let mut world = world(directory.clone(), size, false, true);
    assert!(world.settle());
    accepted(&world);
    let lines = dump_errors(&world);
    assert_eq!(lines.len(), 1);
    assert!(lines[0].contains("byte bound"));
    let files = files(&directory);
    assert_eq!(files.len(), 2, "start and dump.log alone");
    assert_eq!(named(&files, "-start.txt"), named(&measured, "-start.txt"));
    assert_eq!(named(&files, "dump.log"), format!("{}\n", lines[0]).as_bytes());
}

#[test]
fn a_claude_code_run_keeps_what_it_was_given() {
    use smith_fake_claude::{Step, Story, Turn};
    let scratch = Scratch::new("request-dump-program");
    let directory = scratch.path().join("dumps");
    let charter = wire::edit_charter(&policy(), |parts| {
        parts.instructions = b"Private dump instruction proof.".as_slice().into();
        parts.brief = skein_lib::List::with_capacity(1);
        parts
            .brief
            .push(smith_charter::Section {
                title: b"Request".as_slice().into(),
                body: b"Dump prompt proof.".as_slice().into(),
            })
            .expect("valid dump fixture");
        let mut main = parts.main.clone();
        main.effort = Some(b"high".as_slice().into());
        parts.main = main;
    });
    let story = Story {
        turns: vec![Turn::new(vec![Step::Outcome {
            input: serde_json::json!({"outcome":{"form":"report","text":"done","fields":{}}}),
        }])],
        ..Story::default()
    };
    let run = program_world::run_dumping(&story, &charter, directory.clone(), BOUND);
    assert!(
        matches!(&run.answer().expect("answer").result, RunResult::Accepted(_)),
        "{:?}: {}",
        run.answer(),
        String::from_utf8_lossy(&run.errors)
    );
    let files = files(&directory);
    let arguments = named(&files, "-claude-code-argv.txt");
    let invocation = run
        .invocations
        .iter()
        .find(|invocation| invocation.arguments.first().is_some_and(|arg| arg == "-p"))
        .expect("valid dump fixture");
    let argv: Vec<_> = std::str::from_utf8(arguments).expect("valid dump fixture").lines().collect();
    assert_eq!(argv, invocation.arguments);
    for (flag, value) in [("--effort", "high"), ("--model", "fake")] {
        assert_eq!(argv.iter().copied().skip_while(|argument| *argument != flag).nth(1), Some(value));
    }
    assert!(
        std::str::from_utf8(named(&files, "-claude-code-instructions.md"))
            .expect("valid dump fixture")
            .contains("Private dump instruction proof.")
    );
    let tools: serde_json::Value =
        serde_json::from_slice(named(&files, "-claude-code-tools.json")).expect("valid dump fixture");
    assert!(tools.get("tools").is_some());
    let input = std::str::from_utf8(named(&files, "-claude-code-stdin.json")).expect("valid dump fixture");
    assert!(input.contains("Begin the work your brief describes."), "{input}");
    let init_line = named(&files, "-claude-code-init.json");
    assert!(init_line.ends_with(b"\n"));
    let init: serde_json::Value = serde_json::from_slice(init_line).expect("valid dump fixture");
    assert_eq!(init["subtype"], "init");
    for (path, bytes) in files {
        assert_ne!(path.file_name().expect("dump filename"), "mcp.json");
        assert!(skein_lib::bytes::find(&bytes, b"SMITH_MCP_SECRET").is_none());
    }
}
