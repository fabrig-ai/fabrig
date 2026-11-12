use super::*;
use crate::drive::{self, Answer, Exit};
use std::{collections::BTreeMap, ffi::{OsStr, OsString}, path::Path};

type Files = BTreeMap<String, BTreeMap<OsString, Vec<u8>>>;
fn put(files: &mut Files, name: &str, file: &str, bytes: &[u8]) {
    files.entry(name.into()).or_default().insert(file.into(), bytes.into());
}
fn read(files: &Files, name: &str) -> Result<Chain, Refusal> {
    Chain::read_with(&Reader::Memory(files), name)
}
fn three() -> Files {
    let mut files = Files::new();
    put(&mut files, "project-landing", "recipe", b"recipe 1\nweb press button \"Start\"\nweb see \"Hello\"\n");
    put(&mut files, "middle", "recipe", b"recipe 1\nafter \"project-landing\"\nweb see \"Ready\"\nctl project list\nweb see \"Middle\"\ntui see \"Middle\"\n");
    put(&mut files, "last", "recipe", b"recipe 1\nafter \"middle\"\ntui key enter\nweb see \"Last\"\n");
    files
}
fn shown(lines: Vec<WalkLine>) -> Vec<String> {
    lines.into_iter().map(|line| format!("{} {}", line.source, String::from_utf8_lossy(&line.line))).collect()
}
#[test]
fn reads_chain_and_walks_fresh_or_from_a_copy() {
    let files = three();
    let chain = read(&files, "last").expect("chain");
    assert_eq!(chain.names().collect::<Vec<_>>(), ["project-landing", "middle", "last"]);
    let middle = &chain.recipes[1];
    assert_eq!(middle.after.as_deref(), Some("project-landing"));
    assert_eq!(middle.steps.iter().map(|s| s.line.print()).collect::<Vec<_>>(), [br#"web see "Ready""#.as_slice(), b"ctl project list"]);
    assert_eq!(chain.recipes[0].steps[0].line.print(), br#"web press button "Start""#);
    assert_eq!(chain.recipes[0].check[0].line.print(), br#"web see "Hello""#);
    let fresh = [
        "project-landing:2 web press button \"Start\"",
        "walk: web reload \"/\"", "walk: tui stop", "project-landing:3 web see \"Hello\"",
        "middle:3 web see \"Ready\"", "middle:4 ctl project list",
        "walk: web reload \"/\"", "walk: tui stop", "middle:5 web see \"Middle\"", "middle:6 tui see \"Middle\"",
        "last:3 tui key enter", "walk: web reload \"/\"", "walk: tui stop", "last:4 web see \"Last\"",
    ];
    assert_eq!(shown(chain.lines(None).expect("fresh")), fresh);
    assert_eq!(shown(chain.lines(Some("middle")).expect("copy")), &fresh[8..]);
    assert_eq!(shown(chain.lines(Some("last")).expect("self")), ["last:4 web see \"Last\""]);
    let refusal = chain.lines(Some("elsewhere")).expect_err("not an ancestor");
    assert!(refusal.to_string().contains("elsewhere/recipe: --from"));
}
#[test]
fn refuses_forbidden_records_and_requires_a_trailing_check() {
    let mut files = Files::new();
    for (record, why) in [
        ("web look", "not recipe acts"), ("web look 800", "not recipe acts"),
        ("tui look", "not recipe acts"), ("end", "not recipe acts"), ("tui stop", "not recipe acts"),
        ("", "not recipe acts"), ("web press button", "expected control name"),
        ("after \"root\"", "unknown client"),
    ] {
        put(&mut files, "bad", "recipe", format!("recipe 1\nweb see \"Start\"\n{record}\nweb see \"Done\"\n").as_bytes());
        let refusal = read(&files, "bad").expect_err(record);
        assert_eq!((refusal.name.as_str(), refusal.line), ("bad", Some(3)), "{refusal}");
        assert!(refusal.why.contains(why), "{refusal}");
    }
    for bytes in [b"recipe 1\nweb press button \"Start\"\n".as_slice(), b"recipe 1\n", b"recipe 1\nafter \"root\"\n"] {
        put(&mut files, "bad", "recipe", bytes);
        let refusal = read(&files, "bad").expect_err("no closing see");
        assert!(refusal.why.contains("must end"), "{refusal}");
        assert_eq!(refusal.line, Some(bytes.split(|b| *b == b'\n').count() - 1));
    }
    put(&mut files, "bad", "recipe", b"recipe 2\nweb see \"Done\"\n");
    assert_eq!(read(&files, "bad").expect_err("version").line, Some(1));
    put(&mut files, "bad", "recipe", b"recipe 1\nafter root\nweb see \"Done\"\n");
    assert!(read(&files, "bad").expect_err("quoted parent").why.contains("quoted parent"));
    put(&mut files, "bad", "recipe", b"recipe 1\nafter \"root\" extra\nweb see \"Done\"\n");
    assert!(read(&files, "bad").expect_err("extra parent field").why.contains("extra field"));
}
#[test]
fn refuses_names_and_unexpected_files() {
    let mut files = Files::new();
    let valid = format!("a{}", "0".repeat(63));
    put(&mut files, &valid, "recipe", b"recipe 1\nweb see \"Yes\"\n");
    read(&files, &valid).expect("64 byte name");
    for name in [String::new(), "A".into(), "0a".into(), "a/b".into(), "a_b".into(), "é".into(), format!("{valid}a")] {
        let refusal = read(&files, &name).expect_err("invalid name");
        assert_eq!(refusal.name, name);
        assert_eq!(refusal.file.as_deref(), Some("recipe"));
        assert!(refusal.why.contains("1–64 bytes"));
    }
    put(&mut files, "bad", "recipe", b"recipe 1\nafter \"Bad\"\nweb see \"Yes\"\n");
    assert_eq!(read(&files, "bad").expect_err("parent name").line, Some(2));
    put(&mut files, &valid, "extra", b"not allowed");
    let refusal = read(&files, &valid).expect_err("unexpected file");
    assert_eq!(refusal.file.as_deref(), Some("extra"));
    assert!(refusal.to_string().contains("/extra: unexpected file"));
}
#[test]
fn file_and_line_limits_are_inclusive() {
    let mut files = Files::new();
    let mut bytes = b"recipe 1\n".to_vec();
    bytes.extend(b"web see \"Yes\"\n".repeat(1023));
    put(&mut files, "bounded", "recipe", &bytes);
    read(&files, "bounded").expect("1024 records");
    bytes.extend_from_slice(b"web see \"One too many\"\n");
    put(&mut files, "bounded", "recipe", &bytes);
    let refusal = read(&files, "bounded").expect_err("1025 records");
    assert_eq!(refusal.line, Some(1025));
    assert!(refusal.why.contains("1024"));
    let mut bytes = b"recipe 1\nweb see \"".to_vec();
    bytes.extend(vec![b'x'; 65536 - bytes.len() - 2]);
    bytes.extend_from_slice(b"\"\n");
    assert_eq!(bytes.len(), 65536);
    put(&mut files, "bounded", "recipe", &bytes);
    read(&files, "bounded").expect("64 KiB");
    bytes.pop();
    put(&mut files, "bounded", "recipe", &bytes);
    read(&files, "bounded").expect("no final newline");
    bytes.extend_from_slice(b"\n\n");
    put(&mut files, "bounded", "recipe", &bytes);
    assert!(read(&files, "bounded").expect_err("64 KiB + 1").why.contains("64 KiB"));
}
#[test]
fn refuses_cycle_missing_parent_and_ninth_parent_at_the_after() {
    let mut files = Files::new();
    put(&mut files, "a", "recipe", b"recipe 1\nafter \"b\"\nweb see \"A\"\n");
    put(&mut files, "b", "recipe", b"recipe 1\nafter \"a\"\nweb see \"B\"\n");
    let refusal = read(&files, "a").expect_err("cycle");
    assert_eq!((refusal.name.as_str(), refusal.line), ("b", Some(2)));
    assert!(refusal.why.contains("cycle"));
    files.remove("b");
    let refusal = read(&files, "a").expect_err("missing parent");
    assert_eq!((refusal.name.as_str(), refusal.line), ("a", Some(2)));
    assert!(refusal.why.contains("after \"b\""));
    put(&mut files, "b", "README.md", b"Missing its recipe.");
    assert_eq!(read(&files, "a").expect_err("missing parent file").line, Some(2));
    put(&mut files, "r0", "recipe", b"recipe 1\nweb see \"Root\"\n");
    for number in 1..=9 {
        put(&mut files, &format!("r{number}"), "recipe", format!("recipe 1\nafter \"r{}\"\nweb see \"Yes\"\n", number - 1).as_bytes());
    }
    assert_eq!(read(&files, "r8").expect("eight parents").names().count(), 9);
    let refusal = read(&files, "r9").expect_err("ninth parent");
    assert_eq!((refusal.name.as_str(), refusal.line), ("r1", Some(2)));
    assert!(refusal.why.contains("ninth parent"));
}
#[test]
fn hash_frames_root_first_bytes_and_excludes_readme() {
    let mut files = Files::new();
    put(&mut files, "root", "recipe", b"recipe 1\nweb see \"Root\"\n");
    put(&mut files, "child", "recipe", b"recipe 1\nafter \"root\"\nweb see \"Child\"\n");
    put(&mut files, "root", "scripts.json", b"{}");
    let hash = read(&files, "child").expect("chain").hash();
    // Independently framed SHA-256 vector, not the production list of fields.
    assert_eq!(hex(hash), "fecb4ec1759344cfd094415c06e5f5d9a1b13525eae3f21539802009888be5a5");
    put(&mut files, "root", "README.md", b"Reworded.");
    assert_eq!(read(&files, "child").expect("README ignored").hash(), hash);
    let before = files.clone();
    put(&mut files, "root", "recipe", b"recipe 1\nweb see \"Changed\"\n");
    assert_ne!(read(&files, "child").expect("parent recipe").hash(), hash);
    files = before.clone();
    put(&mut files, "root", "scripts.json", b"{\"otherwise\": []}");
    assert_ne!(read(&files, "child").expect("parent scripts").hash(), hash);
    let mut files = before;
    let root = files.get("root").expect("root").clone();
    files.insert("renamed".into(), root);
    assert_ne!(read(&files, "root").expect("root").hash(), read(&files, "renamed").expect("name only changes").hash());
}
fn clean() -> Exit {
    Exit { status: 0, shown: vec!["ended".into()], error: None }
}
fn finished(actions: Vec<Action>) -> Exit {
    let [Action::Finished(exit)] = <[_; 1]>::try_from(actions).expect("finished action") else { panic!("not finished") };
    exit
}
#[test]
fn walk_stops_on_first_failure_and_ends_before_status_five() {
    let chain = read(&three(), "last").expect("chain");
    let (mut walk, first) = Walk::start(&chain, None).expect("walk");
    assert!(matches!(first.as_slice(), [Action::Line(bytes)] if bytes.as_ref() == br#"web press button "Start""#));
    let actions = walk.step(Event::Answer(Answer { shown: vec!["a\nb".into()], failed: Some("missing Start".into()) }));
    let [Action::Print(a), Action::Print(b), Action::Print(failed), Action::End] = actions.as_slice() else { panic!("{actions:?}") };
    assert_eq!(a, "project-landing:2 | a");
    assert_eq!(b, "project-landing:2 | b");
    assert_eq!(failed, "project-landing:2 failed: missing Start");
    let exit = finished(walk.step(Event::Ended(clean())));
    assert_eq!(exit.status, 5);
    assert_eq!(exit.shown.last().expect("last"), "stopped at project-landing:2: missing Start");
    let (mut walk, _) = Walk::start(&chain, None).expect("walk");
    walk.step(Event::Answer(Answer { shown: vec![], failed: None }));
    let actions = walk.step(Event::Answer(Answer { shown: vec![], failed: Some("reload failed".into()) }));
    assert!(matches!(actions.as_slice(), [Action::Print(line), Action::End] if line == "walk: failed: reload failed"));
    assert_eq!(finished(walk.step(Event::Ended(clean()))).shown.last().expect("last"), "stopped at walk: reload failed");
}
#[test]
fn walk_success_and_session_failures_keep_their_statuses() {
    let chain = read(&three(), "last").expect("chain");
    let (mut walk, mut actions) = Walk::start(&chain, Some("middle")).expect("walk");
    let mut count = 0;
    loop {
        match actions.pop().expect("next action") {
            Action::Line(_) => {
                count += 1;
                actions = walk.step(Event::Answer(Answer { shown: vec![], failed: None }));
            }
            Action::End => break,
            other => panic!("{other:?}"),
        }
    }
    assert_eq!(count, 6);
    let exit = finished(walk.step(Event::Ended(clean())));
    assert_eq!(exit.status, 0);
    assert_eq!(exit.shown.last().expect("last"), &format!("walked last {}", hex(chain.hash())));
    for status in [1, 3, 4] {
        let (mut walk, _) = Walk::start(&chain, None).expect("walk");
        walk.step(Event::Answer(Answer { shown: vec![], failed: Some("failed line".into()) }));
        let own = Exit { status, shown: vec!["session diagnosis".into()], error: Some("session error".into()) };
        assert_eq!(finished(walk.step(Event::Ended(own.clone()))), own);
    }
}
fn args(words: &[&str]) -> Vec<Box<[u8]>> {
    words.iter().map(|word| word.as_bytes().into()).collect()
}
#[test]
fn walk_plan_orders_scripts_and_strict_and_checks_scratch() {
    use command::{Arguments, Mode, session_options};
    let mut files = three();
    put(&mut files, "project-landing", "scripts.json", b"{}");
    put(&mut files, "last", "scripts.json", b"{}");
    let chain = read(&files, "last").expect("chain");
    let arguments = Arguments::parse(Mode::Walk, &args(&["last", "--bin", "/build", "--bundle", "/bundle", "--scratch", "/scratch", "--from", "middle", "--strict", "--setups", "/recipes"])).expect("arguments");
    assert_eq!(arguments.name.as_deref(), Some("last"));
    assert_eq!(arguments.from.as_deref(), Some("middle"));
    let options = session_options(&chain, &arguments.setups, arguments.drive.expect("drive"));
    let plan = drive::Plan::new(options, "/chromium".into(), OsStr::new("/bin"));
    let (machine, actions) = drive::Machine::start(plan, skein_lib::Time::ZERO);
    assert!(!machine.ready());
    let [drive::Action::Start { process: drive::Process::Box, program }] = actions.as_slice() else { panic!("{actions:?}") };
    let script_args = program.args.iter().skip_while(|arg| *arg != "--scripts").cloned().collect::<Vec<_>>();
    assert_eq!(script_args, ["--scripts", "/recipes/project-landing/scripts.json", "--scripts", "/recipes/last/scripts.json", "--strict"]);
    for (from, config, data, accepted) in [
        (None, false, false, true), (None, true, false, false), (None, false, true, false), (None, true, true, false),
        (Some("middle"), false, false, false), (Some("middle"), true, false, false), (Some("middle"), false, true, false), (Some("middle"), true, true, true),
    ] {
        let result = walk::check_scratch(from, config, data);
        assert_eq!(result.is_ok(), accepted, "{from:?} {config} {data}");
        if let Err(why) = result { assert!(why.contains("config/") && why.contains("data/")); }
    }
    assert!(Arguments::parse(Mode::Walk, &args(&["last"])).is_err());
    assert!(Arguments::parse(Mode::Walk, &args(&["last", "--scripts", "external"])).is_err());
    for mode in [Mode::Hash, Mode::List] {
        let words = if mode == Mode::Hash { args(&["last"]) } else { vec![] };
        let parsed = Arguments::parse(mode, &words).expect("defaults");
        assert_eq!(parsed.setups, Path::new(SETUPS));
        assert!(parsed.drive.is_none());
    }
    assert!(Arguments::parse(Mode::Hash, &args(&["last", "--from", "middle"])).is_err());
}
#[test]
fn catalogue_sorts_parents_and_continues_after_a_refusal() {
    let mut files = three();
    put(&mut files, "project-landing", "README.md", b"A project. Another sentence.");
    put(&mut files, "middle", "README.md", b"The middle\nstate! More.");
    put(&mut files, "last", "recipe", b"recipe 1\nweb look\n");
    let lines = command::catalogue(&Reader::Memory(&files), vec!["project-landing".into(), "middle".into(), "last".into()]);
    assert_eq!(lines.len(), 3);
    assert!(lines[0].starts_with("last: refused: last:2: "), "{lines:?}");
    assert_eq!(lines[1], "middle (after project-landing): The middle state!");
    assert_eq!(lines[2], "project-landing: A project.");
}
