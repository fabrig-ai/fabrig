use super::*;
use crate::drive::{self, Answer, Exit};
use std::{
    collections::BTreeMap,
    ffi::{OsStr, OsString},
    path::Path,
};

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
fn forge_steps_walk_before_resets_and_checks_from_fresh_or_copy() {
    let mut files = Files::new();
    put(&mut files, "base", "recipe", b"recipe 1\nweb see \"Base\"\n");
    put(&mut files, "gate", "recipe", b"recipe 1\nafter \"base\"\nforge commit forgejo playground/widget \"gate\" \"a.txt\"\nforge checks forgejo playground/widget\nweb see \"Green\"\ntui see \"Green\"\n");
    put(&mut files, "gate", "files/a.txt", b"x\n");
    let chain = read(&files, "gate").expect("forge recipe");
    let fresh = [
        "walk: web reload \"/\"",
        "walk: tui stop",
        "base:2 web see \"Base\"",
        "gate:3 forge commit forgejo playground/widget \"gate\" \"a.txt\"",
        "gate:4 forge checks forgejo playground/widget",
        "walk: web reload \"/\"",
        "walk: tui stop",
        "gate:5 web see \"Green\"",
        "gate:6 tui see \"Green\"",
    ];
    assert_eq!(shown(chain.lines(None).expect("fresh")), fresh);
    assert_eq!(shown(chain.lines(Some("base")).expect("base copy")), &fresh[2..]);
    assert_eq!(shown(chain.lines(Some("gate")).expect("gate copy")), &fresh[7..]);
}

#[test]
fn reads_chain_and_walks_fresh_or_from_a_copy() {
    let files = three();
    let chain = read(&files, "last").expect("chain");
    assert_eq!(chain.names().collect::<Vec<_>>(), ["project-landing", "middle", "last"]);
    let middle = &chain.recipes[1];
    assert_eq!(middle.after.as_deref(), Some("project-landing"));
    assert_eq!(
        middle.steps.iter().map(|s| s.line.print()).collect::<Vec<_>>(),
        [br#"web see "Ready""#.as_slice(), b"ctl project list"]
    );
    assert_eq!(chain.recipes[0].steps[0].line.print(), br#"web press button "Start""#);
    assert_eq!(chain.recipes[0].check[0].line.print(), br#"web see "Hello""#);
    let fresh = [
        "project-landing:2 web press button \"Start\"",
        "walk: web reload \"/\"",
        "walk: tui stop",
        "project-landing:3 web see \"Hello\"",
        "middle:3 web see \"Ready\"",
        "middle:4 ctl project list",
        "walk: web reload \"/\"",
        "walk: tui stop",
        "middle:5 web see \"Middle\"",
        "middle:6 tui see \"Middle\"",
        "last:3 tui key enter",
        "walk: web reload \"/\"",
        "walk: tui stop",
        "last:4 web see \"Last\"",
    ];
    assert_eq!(shown(chain.lines(None).expect("fresh")), fresh);
    assert_eq!(shown(chain.lines(Some("middle")).expect("copy")), &fresh[8..]);
    assert_eq!(shown(chain.lines(Some("last")).expect("self")), ["last:4 web see \"Last\""]);
    let refusal = chain.lines(Some("elsewhere")).expect_err("not an ancestor");
    assert!(refusal.to_string().contains("elsewhere/recipe: --from"));
}
/// The landed QA catalogue reads the kept recipes (client/recipe.md, 2–3, 5.1).
#[test]
fn committed_qa_recipes_read_chain_and_end_on_their_own_checks() {
    let setups = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs/development/qa/setups");
    let mut names = vec![];
    for entry in std::fs::read_dir(&setups).expect("committed QA setups") {
        let entry = entry.expect("setup entry");
        if !entry.file_type().expect("setup file type").is_dir() {
            continue;
        }
        let name = entry.file_name().into_string().expect("ASCII recipe name");
        let chain = Chain::read(&setups, &name).unwrap_or_else(|why| panic!("{name}: {why}"));
        let recipe = chain.recipes.last().expect("recipe itself");
        let check: Vec<_> = recipe.check.iter().map(|line| recipe.walk_line(line)).collect();
        assert!(!check.is_empty(), "{name} has a closing check");
        assert!(chain.lines(None).expect("fresh walk").ends_with(&check), "{name} ends on its own check");
        assert_ne!(chain.hash(), [0; 32], "{name} has a chain hash");
        names.push(name);
    }
    let listed = command::catalogue(&Reader::Directory(&setups), names.clone());
    for name in names {
        assert!(
            listed.iter().any(|line| line.starts_with(&format!("{name} ")) || line.starts_with(&format!("{name}:")))
        );
    }
    assert!(
        listed.iter().all(|line| !line.contains(": refused:")),
        "recipe list reads every committed setup: {listed:?}"
    );

    let chain = Chain::read(&setups, "chat-draft").expect("chat-draft chain");
    assert_eq!(chain.names().collect::<Vec<_>>(), ["welcome", "project-landing", "chat-draft"]);
    let welcome = chain.recipes.first().expect("welcome");
    assert!(welcome.steps.is_empty(), "welcome is already reached on a fresh box");

    let project = chain.recipes.iter().find(|recipe| recipe.name == "project-landing").expect("project-landing");
    assert!(
        project.steps.iter().any(|line| line.line.print() == b"ctl forge connect forgejo-admin {forgejo.admin} --first work/hello --project Hello --api {forgejo.api} --git {forgejo.git} --git-program {git} --key qa-hello"),
        "project-landing connects the fake Forgejo as its admin through the box's values"
    );
    assert!(
        project.steps.iter().any(|line| line.line.print() == br#"web see "work/hello is in Hello""#),
        "Hello's home is waited on before the walk resets to the root"
    );
    assert_eq!(
        project.check.iter().map(|line| line.line.print()).collect::<Vec<_>>(),
        [br#"web see "Nothing needs you.""#.as_slice()]
    );
}

/// QA uses the fake Forgejo elsewhere, never the owned one (testing/qa/setups.md, 2, 5).
#[test]
fn committed_qa_recipes_never_set_up_the_owned_forge() {
    let setups = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs/development/qa/setups");
    for entry in std::fs::read_dir(&setups).expect("committed QA setups") {
        let entry = entry.expect("setup entry");
        if !entry.file_type().expect("setup file type").is_dir() {
            continue;
        }
        let name = entry.file_name().into_string().expect("ASCII recipe name");
        let chain = Chain::read(&setups, &name).expect("QA chain");
        let recipe = chain.recipes.last().expect("recipe itself");
        for line in recipe.steps.iter().chain(&recipe.check) {
            if let Record::Ctl(args) = line.line.record() {
                assert_ne!(
                    args.iter().take(2).map(AsRef::as_ref).collect::<Vec<_>>(),
                    [b"forge".as_slice(), b"setup"],
                    "{} sets up the owned Forgejo",
                    recipe.walk_line(line).source
                );
            }
            if let Record::Web(Web::Act { name, act: skein_ui::Act::Press, .. }) = line.line.record() {
                assert_ne!(
                    name.as_ref(),
                    b"Run on this machine",
                    "{} presses the owned setup",
                    recipe.walk_line(line).source
                );
            }
        }
    }
}

/// Widget starts with the twin's B0 tree on the fake Forgejo (testing/qa/setups.md, 4.3, 5).
#[test]
fn widget_commits_the_b0_tree_before_waiting_on_checks() {
    use crate::line::Forge;
    use serde_json::Value;

    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let chain = Chain::read(&root.join("docs/development/qa/setups"), "widget").expect("widget chain");
    let recipe = chain.recipes.last().expect("widget");
    let fixture: Value = serde_json::from_slice(
        &std::fs::read(root.join("tests/real-loop/tests/fixtures/b0/forge.json")).expect("B0 forge fixture"),
    )
    .expect("B0 forge JSON");
    let tree = fixture["repositories"]
        .as_array()
        .expect("repositories")
        .iter()
        .find(|repository| repository["name"] == "widget")
        .expect("widget")["tree"]
        .as_array()
        .expect("widget tree");
    let expected: CommitFiles = tree
        .iter()
        .map(|entry| {
            (
                entry["path"].as_str().expect("tree path").as_bytes().into(),
                entry["content"].as_str().expect("tree content").as_bytes().into(),
            )
        })
        .collect();
    assert_eq!(recipe.files, expected, "widget files equal the complete B0 tree byte for byte");
    let forge: Vec<_> = recipe
        .steps
        .iter()
        .filter_map(|line| if let Record::Forge(record) = line.line.record() { Some(record) } else { None })
        .collect();
    assert!(
        matches!(forge.as_slice(), [Forge::Commit { repository, paths, .. }, Forge::Checks { repository: checked }]
            if repository.as_ref() == b"playground/widget"
                && checked == repository
                && paths.iter().map(AsRef::as_ref).collect::<Vec<_>>() == [b"README.md".as_slice(), b"src/main.rs", b".forgejo/workflows/check.yml"]),
        "widget commits the three B0 files once, then waits on playground/widget's checks: {forge:?}"
    );
}

/// B1's gate is the one its twin met, not today's acceptance text (testing/qa/setups.md, 4.4–5).
#[test]
fn b1_plan_commits_exactly_the_twins_gate_before_waiting_on_checks() {
    use crate::line::Forge;
    use serde_json::Value;

    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let chain = Chain::read(&root.join("docs/development/qa/setups"), "b1-plan").expect("B1 plan chain");
    let recipe = chain.recipes.last().expect("B1 plan");
    assert_eq!(recipe.after.as_deref(), Some("widget"));
    let mut steps = recipe.steps.iter();
    let Record::Forge(Forge::Commit { repository, message, paths }) =
        steps.next().expect("gate is the first step").line.record()
    else {
        panic!("B1 starts by committing its gate");
    };
    assert_eq!(repository.as_ref(), b"playground/widget");
    assert_eq!(message.as_ref(), b"CI: each pull request adds changes/<its number>.md");
    let fixture: Value = serde_json::from_slice(
        &std::fs::read(root.join("tests/real-loop/tests/fixtures/b1/forge.json")).expect("B1 forge fixture"),
    )
    .expect("B1 forge JSON");
    let tree = fixture["repositories"]
        .as_array()
        .expect("repositories")
        .iter()
        .find(|repository| repository["name"] == "widget")
        .expect("widget")["tree"]
        .as_array()
        .expect("widget tree");
    assert_eq!(
        paths.iter().map(AsRef::as_ref).collect::<Vec<_>>(),
        [b".forgejo/workflows/ci.yml".as_slice(), b"CONTRIBUTING.md", b".fake-forge/ci"]
    );
    assert_eq!(recipe.files.len(), 3);
    assert_eq!(
        recipe.files.get(b".fake-forge/ci".as_slice()).expect("CI declaration"),
        b"pull-file changes/{pull}.md\n"
    );
    for path in [".forgejo/workflows/ci.yml", "CONTRIBUTING.md"] {
        let entry = tree.iter().find(|entry| entry["path"] == path).expect("gate file in fixture");
        let content = recipe.files.get(path.as_bytes()).expect("gate file in recipe");
        assert_eq!(
            content.as_slice(),
            entry["content"].as_str().expect("gate content").as_bytes(),
            "{path} equals the twin's tree"
        );
    }
    assert_eq!(
        steps.next().expect("wait on the gate's checks before requesting work").line.record(),
        &Record::Forge(Forge::Checks { repository: b"playground/widget".as_slice().into() })
    );
}

#[test]
fn tui_chat_waits_for_the_answer_but_checks_the_fresh_launchpad() {
    let setups = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs/development/qa/setups");
    let chain = Chain::read(&setups, "tui-chat").expect("tui-chat chain");
    assert_eq!(chain.names().collect::<Vec<_>>(), ["welcome", "project-landing", "tui-chat"]);
    let recipe = chain.recipes.last().expect("tui-chat");
    assert_eq!(
        recipe.steps.iter().rev().take(2).map(|line| line.line.print()).collect::<Vec<_>>(),
        [br#"web reload "/""#.as_slice(), br#"tui see "Hello from the local fake model.""#.as_slice()]
    );
    assert_eq!(
        recipe.check.iter().map(|line| line.line.print()).collect::<Vec<_>>(),
        [br#"tui see "/chats 1""#.as_slice()]
    );
    assert_ne!(chain.hash(), [0; 32]);
    assert!(!setups.join("tui-chat/scripts.json").exists());
}

#[test]
fn refuses_forbidden_records_and_requires_a_trailing_check() {
    let mut files = Files::new();
    for (record, why) in [
        ("web look", "not recipe acts"),
        ("web look 800", "not recipe acts"),
        ("tui look", "not recipe acts"),
        ("end", "not recipe acts"),
        ("tui stop", "not recipe acts"),
        ("", "not recipe acts"),
        ("web press button", "expected control name"),
        ("after \"root\"", "unknown client"),
    ] {
        put(
            &mut files,
            "bad",
            "recipe",
            format!("recipe 1\nweb see \"Start\"\n{record}\nweb see \"Done\"\n").as_bytes(),
        );
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
    let refusal = read(&files, "bounded").map(|_| ()).expect_err("1025 records must be refused");
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
    assert!(read(&files, "bounded").map(|_| ()).expect_err("64 KiB + 1 must be refused").why.contains("64 KiB"));
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
        put(
            &mut files,
            &format!("r{number}"),
            "recipe",
            format!("recipe 1\nafter \"r{}\"\nweb see \"Yes\"\n", number - 1).as_bytes(),
        );
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
    assert_eq!(hex(hash), "02c50b7858ce88c7d9487dca8454438358f600beedd6dca6224d54a93b2b6b47");
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
    let [Action::Finished(exit)] = <[_; 1]>::try_from(actions).expect("finished action") else {
        panic!("not finished")
    };
    exit
}
#[test]
fn walk_stops_on_first_failure_and_ends_before_status_five() {
    let chain = read(&three(), "last").expect("chain");
    let (mut walk, first) = Walk::start(&chain, None).expect("walk");
    assert!(matches!(first.as_slice(), [Action::Line(line)] if line.line.as_ref() == br#"web press button "Start""#));
    let actions =
        walk.step(Event::Answer(Answer { shown: vec!["a".into(), "b".into()], failed: Some("missing Start".into()) }));
    let [Action::Print(a), Action::Print(b), Action::Print(failed), Action::End] = actions.as_slice() else {
        panic!("{actions:?}")
    };
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
    assert_eq!(
        finished(walk.step(Event::Ended(clean()))).shown.last().expect("last"),
        "stopped at walk: reload failed"
    );
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
    let arguments = Arguments::parse(
        Mode::Walk,
        &args(&[
            "last",
            "--bin",
            "/build",
            "--bundle",
            "/bundle",
            "--scratch",
            "/scratch",
            "--ports",
            "1,2,3,4,5,6,7,8,9",
            "--from",
            "middle",
            "--strict",
            "--setups",
            "/recipes",
        ]),
    )
    .expect("arguments");
    assert_eq!(arguments.name.as_deref(), Some("last"));
    assert_eq!(arguments.from.as_deref(), Some("middle"));
    let options = session_options(&chain, &arguments.setups, arguments.drive.expect("drive"));
    let plan = drive::Plan::new(options, "/chromium".into(), OsStr::new("/bin"));
    let (machine, actions) = drive::Machine::start(plan, skein_lib::Time::ZERO);
    assert!(!machine.ready());
    let [drive::Action::Start { process: drive::Process::Box, program }] = actions.as_slice() else {
        panic!("{actions:?}")
    };
    assert!(
        program
            .environment
            .iter()
            .any(|(name, value)| name == "TEMPER_INTERNAL_FAKE_BOX_PORTS" && value == "1,2,3,4,5,6,7,8,9")
    );
    let script_args = program.args.iter().skip_while(|arg| *arg != "--scripts").cloned().collect::<Vec<_>>();
    assert_eq!(
        script_args,
        ["--scripts", "/recipes/project-landing/scripts.json", "--scripts", "/recipes/last/scripts.json", "--strict"]
    );
    for (from, config, data, accepted) in [
        (None, false, false, true),
        (None, true, false, false),
        (None, false, true, false),
        (None, true, true, false),
        (Some("middle"), false, false, false),
        (Some("middle"), true, false, false),
        (Some("middle"), false, true, false),
        (Some("middle"), true, true, true),
    ] {
        let result = walk::check_scratch(from, config, data);
        assert_eq!(result.is_ok(), accepted, "{from:?} {config} {data}");
        if let Err(why) = result {
            assert!(why.contains("config/") && why.contains("data/"));
        }
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
    let lines =
        command::catalogue(&Reader::Memory(&files), vec!["project-landing".into(), "middle".into(), "last".into()]);
    assert_eq!(lines.len(), 3);
    assert!(lines[0].starts_with("last: refused: last:2: "), "{lines:?}");
    assert_eq!(lines[1], "middle (after project-landing): The middle state!");
    assert_eq!(lines[2], "project-landing: A project.");
}

fn committing_files(count: usize) -> Files {
    use std::fmt::Write;
    let mut files = Files::new();
    let mut recipe = "recipe 1\n".to_owned();
    for start in (0..count).step_by(16) {
        recipe.push_str("forge commit forgejo playground/widget \"files\"");
        for n in start..(start + 16).min(count) {
            let path = format!("{n:02}.txt");
            write!(recipe, " \"{path}\"").expect("recipe field");
            put(&mut files, "root", &format!("files/{path}"), b"bytes");
        }
        recipe.push('\n');
    }
    recipe.push_str("web see \"Ready\"\n");
    put(&mut files, "root", "recipe", recipe.as_bytes());
    files
}

#[test]
fn recipe_files_require_exactly_the_named_regular_files_with_inclusive_bounds() {
    let mut files = committing_files(64);
    put(&mut files, "root", "files/00.txt", &vec![b'x'; 65536]);
    let chain = read(&files, "root").expect("64 files, 64 KiB inclusive");
    assert_eq!(chain.recipes[0].files.len(), 64);
    assert_eq!(chain.recipes[0].files[b"00.txt".as_slice()].len(), 65536);
    for (case, file, expected) in [
        ("missing", "files/00.txt", "missing"),
        ("unused", "files/unmentioned.txt", "no forge commit"),
        ("count", "files/extra.txt", "64 files"),
        ("large", "files/00.txt", "64 KiB"),
    ] {
        let mut files = if case == "unused" { committing_files(1) } else { files.clone() };
        match case {
            "missing" => {
                files.get_mut("root").expect("root").remove(OsStr::new(file));
            }
            "unused" | "count" => put(&mut files, "root", file, b"extra"),
            "large" => put(&mut files, "root", file, &vec![b'x'; 65537]),
            _ => unreachable!(),
        }
        let refusal = read(&files, "root").map(|_| ()).expect_err(case);
        assert_eq!(refusal.file.as_deref(), Some(file), "{case}: {refusal}");
        assert!(refusal.why.contains(expected), "{case}: {refusal}");
    }
}

#[test]
fn recipe_files_hash_frames_sorted_paths_and_bytes_after_scripts() {
    let mut files = committing_files(2);
    put(&mut files, "child", "recipe", b"recipe 1\nafter \"root\"\nweb see \"Child\"\n");
    let mut chain = read(&files, "child").expect("chain with parent files");
    let hash = chain.hash();
    assert_eq!(hex(hash), "19177e75929c5647896a56acff95b2915436bc8aeda9aa57007657f5fbd4c3c7");
    chain.recipes[0].files.insert(b"00.txt".as_slice().into(), b"changed".to_vec());
    assert_ne!(chain.hash(), hash, "a parent's file bytes count");
    chain.recipes[0].files.insert(b"00.txt".as_slice().into(), b"bytes".to_vec());
    let bytes = chain.recipes[0].files.remove(b"00.txt".as_slice()).expect("old path");
    chain.recipes[0].files.insert(b"renamed.txt".as_slice().into(), bytes);
    assert_ne!(chain.hash(), hash, "path alone counts, without changing recipe bytes");
}

#[test]
fn walk_commits_each_recipes_own_files_even_when_paths_are_the_same() {
    use crate::drive::Action as DriveAction;
    use crate::drive::tests::forge::{begin, fixture, response, select_files};
    let scratch = skein_scratch::Scratch::new("walk-recipe-files");
    let setups = scratch.path();
    for (name, after, content) in [("root", "", "parent bytes"), ("child", "after \"root\"\n", "child bytes")] {
        std::fs::create_dir_all(setups.join(name).join("files")).expect("files directory");
        std::fs::write(
            setups.join(name).join("recipe"),
            format!(
                "recipe 1\n{after}forge commit forgejo playground/widget \"commit\" \"a.txt\"\nweb see \"Ready\"\n"
            ),
        )
        .expect("recipe");
        std::fs::write(setups.join(name).join("files/a.txt"), content).expect("content");
    }
    let chain = Chain::read(setups, "child").expect("chain");
    let (_session_scratch, mut machine) = fixture();
    let (mut walk, mut actions) = Walk::start(&chain, None).expect("walk");
    let mut committed = vec![];
    loop {
        match actions.pop().expect("walk action") {
            Action::Line(line) => {
                if line.line.starts_with(b"forge commit") {
                    select_files(&mut machine, line.files(setups));
                    let actions = begin(&mut machine, std::str::from_utf8(&line.line).expect("line"));
                    assert!(matches!(actions.as_slice(), [DriveAction::ForgeRequest { .. }]));
                    let actions = response(
                        &mut machine,
                        &actions,
                        "/repos/playground/widget",
                        200,
                        br#"{"default_branch":"main"}"#,
                        0,
                    );
                    let actions = response(
                        &mut machine,
                        &actions,
                        "/repos/playground/widget/contents/a.txt?ref=main",
                        404,
                        b"",
                        0,
                    );
                    let [DriveAction::ForgeRequest { body, .. }] = actions.as_slice() else { panic!("POST") };
                    let body: serde_json::Value = serde_json::from_slice(body).expect("commit JSON");
                    committed.push(body["files"][0]["content"].as_str().expect("base64 content").to_owned());
                    machine.step(drive::Event::ForgeFailure("finish fake line".into()), skein_lib::Time::ZERO);
                }
                actions = walk.step(Event::Answer(Answer { shown: vec![], failed: None }));
            }
            Action::End => break,
            other => panic!("{other:?}"),
        }
    }
    assert_eq!(committed, ["cGFyZW50IGJ5dGVz", "Y2hpbGQgYnl0ZXM="]);
}
