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
fn explicit_check_separates_steps_from_ctl_reads_waits_and_see_lines() {
    let mut files = Files::new();
    put(&mut files, "ctl-state", "recipe", b"recipe 1\nctl project make Hello\nforge checks forgejo work/hello\ncheck\nctl project list --until made:1\nctl project show Hello\n");
    let chain = read(&files, "ctl-state").expect("ctl check");
    let recipe = &chain.recipes[0];
    assert_eq!(recipe.steps.len(), 2);
    assert_eq!(recipe.check.iter().map(|line| line.number).collect::<Vec<_>>(), [5, 6]);
    assert_eq!(
        shown(chain.lines(Some("ctl-state")).expect("check from copy")),
        ["ctl-state:5 ctl project list --until made:1", "ctl-state:6 ctl project show Hello"]
    );
    put(
        &mut files,
        "mixed",
        "recipe",
        b"recipe 1\nweb see \"Wait\"\ncheck\nctl project show Hello\nweb see \"Hello\"\ntui see \"Hello\"\n",
    );
    let chain = read(&files, "mixed").expect("explicit mixed check");
    assert_eq!(chain.recipes[0].steps.len(), 1);
    assert_eq!(chain.recipes[0].check.len(), 3);
}

#[test]
fn explicit_check_refuses_repetition_fields_empty_checks_and_acts() {
    let mut files = Files::new();
    for (tail, line, why) in [
        ("check\nctl project list\ncheck\nctl project list\n", 5, "repeated check"),
        ("check extra\nctl project list\n", 3, "check takes no fields"),
        ("check\n", 3, "at least one line"),
        ("check\nforge checks forgejo work/hello\n", 4, "may follow check"),
        ("check\ndeployment patch engine landing.json\n", 4, "may follow check"),
        ("check\nweb press button \"Start\"\n", 4, "may follow check"),
        ("check\nweb reload \"/\"\n", 4, "may follow check"),
        ("check\ntui key enter\n", 4, "may follow check"),
        ("check\ntui stop\n", 4, "not recipe acts"),
        ("check\nweb look\n", 4, "not recipe acts"),
        ("check\nend\n", 4, "not recipe acts"),
        ("check\n\n", 4, "not recipe acts"),
        ("ctl project list\n", 3, "must end"),
    ] {
        put(&mut files, "bad", "recipe", format!("recipe 1\nctl project make Hello\n{tail}").as_bytes());
        let refusal = read(&files, "bad").expect_err(tail);
        assert_eq!((refusal.name.as_str(), refusal.line), ("bad", Some(line)), "{refusal}");
        assert!(refusal.why.contains(why), "{refusal}");
    }
}

fn ctl_chain() -> Files {
    let mut files = Files::new();
    put(&mut files, "root", "recipe", b"recipe 1\nctl project make Hello\ncheck\nctl project show Hello\n");
    put(&mut files, "child", "recipe", b"recipe 1\nafter \"root\"\nforge checks forgejo work/hello\ncheck\nctl project list --until made:1\nctl project show Hello\n");
    files
}

#[test]
fn ctl_chain_plans_a_ctl_session_without_resets_from_fresh_or_copy() {
    let chain = read(&ctl_chain(), "child").expect("ctl chain");
    let fresh = [
        "root:2 ctl project make Hello",
        "root:4 ctl project show Hello",
        "child:3 forge checks forgejo work/hello",
        "child:5 ctl project list --until made:1",
        "child:6 ctl project show Hello",
    ];
    assert_eq!(shown(chain.lines(None).expect("fresh")), fresh);
    assert_eq!(shown(chain.lines(Some("root")).expect("root copy")), &fresh[1..]);
    assert_eq!(shown(chain.lines(Some("child")).expect("child copy")), &fresh[3..]);
    let options = args(&["--bin", "/build", "--bundle", "/bundle", "--scratch", "/scratch"]);
    let options = command::session_options(&chain, Path::new("/recipes"), &options).expect("box options");
    assert!(options.ctl);
    assert_eq!(chain.kind(), Kind::Box);
    assert!(chain.config_dir(Path::new("/recipes")).is_none());
    assert!(options.deployment.is_none());
    assert_eq!(options.bundle.as_deref(), Some(Path::new("/bundle")));
}

#[test]
fn mixed_chain_keeps_browser_session_and_resets_even_from_a_ctl_copy() {
    for (recipe, contents) in [
        ("root", "recipe 1\ntui key enter\ncheck\nctl project show Hello\n"),
        ("root", "recipe 1\nweb see \"Hello\"\n"),
        ("child", "recipe 1\nafter \"root\"\nweb press button \"Start\"\nweb see \"Hello\"\n"),
    ] {
        let mut files = ctl_chain();
        put(&mut files, recipe, "recipe", contents.as_bytes());
        let chain = read(&files, "child").expect("mixed chain");
        let options = args(&["--bin", "/build", "--bundle", "/bundle", "--scratch", "/scratch"]);
        assert!(!command::session_options(&chain, Path::new("/recipes"), &options).expect("box options").ctl);
        let fresh = shown(chain.lines(None).expect("fresh"));
        assert_eq!(fresh.iter().filter(|line| *line == "walk: web reload \"/\"").count(), 2);
        assert_eq!(fresh.iter().filter(|line| *line == "walk: tui stop").count(), 2);
        let copy = shown(chain.lines(Some("root")).expect("root copy"));
        assert_eq!(copy.iter().filter(|line| *line == "walk: web reload \"/\"").count(), 1);
        assert_eq!(copy.iter().filter(|line| *line == "walk: tui stop").count(), 1);
        assert!(chain.lines(Some("child")).expect("child check").iter().all(|line| line.source != Source::Walk));
    }
}

#[test]
fn hash_includes_the_check_record_as_written() {
    let mut files = Files::new();
    put(&mut files, "root", "recipe", b"recipe 1\nweb see \"Hello\"\n");
    let implicit = read(&files, "root").expect("implicit check");
    put(&mut files, "root", "recipe", b"recipe 1\ncheck\nweb see \"Hello\"\n");
    let explicit = read(&files, "root").expect("explicit check");
    assert_eq!(implicit.recipes[0].check[0].line.print(), explicit.recipes[0].check[0].line.print());
    assert_eq!(implicit.recipes[0].check[0].number, 2);
    assert_eq!(explicit.recipes[0].check[0].number, 3);
    assert_ne!(implicit.hash(), explicit.hash(), "the delimiter counts even when the check has the same meaning");
    assert_eq!(hex(explicit.hash()), "1f770bfc911226bee546deb98d47e122f0acbac385999391cfa124bb0dd27fc1");
}

#[test]
fn ctl_check_walk_requires_ok_and_stops_at_its_first_failed_read() {
    let chain = read(&ctl_chain(), "child").expect("ctl chain");
    for failed in [None, Some("project absent")] {
        let (mut walk, first) = Walk::start(&chain, Some("child")).expect("walk check");
        assert!(
            matches!(first.as_slice(), [Action::Line(line)] if line.line.as_ref() == b"ctl project list --until made:1")
        );
        let actions =
            walk.step(Event::Answer(Answer { shown: vec!["project list".into()], failed: failed.map(str::to_owned) }));
        if failed.is_some() {
            assert!(matches!(actions.last(), Some(Action::End)));
        } else {
            assert!(
                matches!(actions.last(), Some(Action::Line(line)) if line.line.as_ref() == b"ctl project show Hello")
            );
            let actions = walk.step(Event::Answer(Answer { shown: vec![], failed: None }));
            assert!(matches!(actions.last(), Some(Action::End)));
        }
        let exit = finished(walk.step(Event::Ended(clean())));
        assert_eq!(exit.status, if failed.is_some() { 5 } else { 0 });
        assert_eq!(
            exit.shown.last().expect("result"),
            &failed.map_or_else(
                || format!("walked child {}", hex(chain.hash())),
                |why| format!("stopped at child:5: {why}"),
            )
        );
    }
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
    assert!(!chain.ctl_only());
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

    let turn = Chain::read(&setups, "turn-landing").expect("turn-landing chain");
    assert_eq!(turn.kind(), Kind::Deployment);
    assert_eq!(turn.names().collect::<Vec<_>>(), ["turn-landing"]);
    let recipe = turn.recipes.first().expect("deployment root");
    assert_eq!(recipe.config.len(), crate::drive::deployment::inputs().len());
    assert_eq!(
        recipe.check.iter().map(|line| line.line.print()).collect::<Vec<_>>(),
        [b"ctl project show Hello".as_slice(), b"ctl forge list --until connected:1", b"ctl forge repositories 1"]
    );

    let chain = Chain::read(&setups, "project-landing").expect("project-landing chain");
    assert_eq!(chain.names().collect::<Vec<_>>(), ["welcome", "project-landing"]);
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

/// The same setup reaches the same state at both kinds of place (testing/qa/setups.md, 5.2, 7).
#[test]
fn stock_landing_keeps_ctl_hellos_recipe_and_scripts_byte_for_byte() {
    let setups = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs/development/qa/setups");
    for file in ["recipe", "scripts.json"] {
        assert_eq!(
            std::fs::read(setups.join("stock-landing").join(file)).expect("stock-landing file"),
            std::fs::read(setups.join("ctl-hello").join(file)).expect("ctl-hello file"),
            "{file} is shared by the two kinds of place"
        );
    }
}

/// Stock landing starts the same deployment processes (testing/qa/setups.md, 5.2, 6).
#[test]
fn stock_landing_keeps_turn_landings_configuration_file_for_file() {
    let setups = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs/development/qa/setups");
    let stock = Chain::read(&setups, "stock-landing").expect("stock-landing chain");
    let turn = Chain::read(&setups, "turn-landing").expect("turn-landing chain");
    let stock = &stock.recipes.first().expect("stock root").config;
    let turn = &turn.recipes.first().expect("turn root").config;
    assert_eq!(stock.keys().collect::<Vec<_>>(), turn.keys().collect::<Vec<_>>());
    for (name, bytes) in stock {
        assert_eq!(Some(bytes), turn.get(name), "{} is shared", String::from_utf8_lossy(name));
    }
}

/// Ctl alone reaches Hello at a fresh deployment's place (testing/qa/setups.md, 5.2, 6).
#[test]
fn stock_landing_plans_its_steps_then_check_in_a_deployment_ctl_session() {
    let setups = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs/development/qa/setups");
    let chain = Chain::read(&setups, "stock-landing").expect("stock-landing chain");
    assert_eq!(chain.kind(), Kind::Deployment);
    assert_eq!(chain.names().collect::<Vec<_>>(), ["stock-landing"]);
    assert!(chain.ctl_only());
    assert_ne!(chain.hash(), [0; 32]);
    let ports = "1,2,3,4,5,6,7,8,9";
    let options = command::session_options(
        &chain,
        &setups,
        &args(&["--bin", "/build", "--scratch", "/scratch", "--ports", ports, "--bundle", "/bundle"]),
    )
    .expect("deployment ctl session");
    assert!(options.ctl);
    assert_eq!(options.deployment, Some(setups.join("stock-landing/config")));
    assert_eq!(options.ports.as_deref(), Some(ports));
    assert_eq!(options.bundle.as_deref(), Some(Path::new("/bundle")));
    assert_eq!(options.scripts, vec![setups.join("stock-landing/scripts.json")]);
    assert_eq!(
        shown(chain.lines(None).expect("fresh deployment ctl walk")),
        [
            "stock-landing:2 ctl project make Hello --key qa-hello-project",
            "stock-landing:3 ctl forge connect forgejo-admin {forgejo.admin} --first work/hello --project Hello --api {forgejo.api} --git {forgejo.git} --git-program {git} --key qa-hello",
            "stock-landing:4 ctl forge list --until made:1",
            "stock-landing:5 ctl account add-key 1 {model.key} --key qa-model",
            "stock-landing:6 ctl account list --until usable",
            "stock-landing:8 ctl project show Hello",
            "stock-landing:9 ctl account list --until usable",
        ]
    );
    assert_eq!(
        command::catalogue(&Reader::Directory(&setups), vec!["stock-landing".into()]),
        ["stock-landing: Hello lands by the stock rule on the deployment's fake Forgejo."]
    );
}

/// Both clients see Hello on the deployment's engine (testing/qa/setups.md, 5.1, 6).
#[test]
fn turn_landing_clients_plans_a_full_deployment_walk_from_the_turn_landing_copy() {
    let setups = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs/development/qa/setups");
    let chain = Chain::read(&setups, "turn-landing-clients").expect("deployment clients chain");
    assert_eq!(chain.kind(), Kind::Deployment);
    assert_eq!(chain.names().collect::<Vec<_>>(), ["turn-landing", "turn-landing-clients"]);
    assert!(!chain.ctl_only());
    let recipe = chain.recipes.last().expect("clients recipe");
    assert_eq!(recipe.check.iter().map(|line| line.number).collect::<Vec<_>>(), [7, 8, 9]);
    assert!(recipe.scripts.is_none());
    assert!(recipe.files.is_empty());
    assert!(recipe.config.is_empty());
    let turn = Chain::read(&setups, "turn-landing").expect("turn-landing chain");
    assert_ne!(chain.hash(), turn.hash());
    assert_ne!(chain.hash(), [0; 32]);

    let ports = "1,2,3,4,5,6,7,8,9";
    let mut arguments = args(&["--bin", "/build", "--scratch", "/scratch", "--ports", ports]);
    assert!(command::session_options(&chain, &setups, &arguments).expect_err("bundle required").contains("--bundle"));
    arguments.extend(args(&["--bundle", "/bundle"]));
    let options = command::session_options(&chain, &setups, &arguments).expect("full deployment session");
    assert!(!options.ctl);
    assert_eq!(options.deployment, Some(setups.join("turn-landing/config")));
    assert_eq!(options.ports.as_deref(), Some(ports));
    assert_eq!(options.bundle.as_deref(), Some(Path::new("/bundle")));
    assert_eq!(options.scripts, vec![setups.join("turn-landing/scripts.json")]);
    assert_eq!(
        shown(chain.lines(Some("turn-landing")).expect("from turn-landing copy")),
        [
            "turn-landing:9 ctl project show Hello",
            "turn-landing:10 ctl forge list --until connected:1",
            "turn-landing:11 ctl forge repositories 1",
            "turn-landing-clients:3 web reload \"/projects/1\"",
            "turn-landing-clients:4 web see \"work/hello is in Hello\"",
            "turn-landing-clients:5 tui see \"Ask temper\"",
            "walk: web reload \"/\"",
            "walk: tui stop",
            "turn-landing-clients:7 ctl project show Hello",
            "turn-landing-clients:8 web see \"Nothing needs you.\"",
            "turn-landing-clients:9 tui see \"Ask temper\"",
        ]
    );
    assert_eq!(
        command::catalogue(&Reader::Directory(&setups), vec!["turn-landing-clients".into()]),
        [
            "turn-landing-clients (after turn-landing): Hello landing by turn, as turn-landing leaves it, is seen through the web client and the TUI on the deployment's engine."
        ]
    );
}

/// Ctl alone reaches Hello and the person's model account (testing/qa/setups.md, 2, 6).
#[test]
fn ctl_hello_reads_hashes_and_walks_its_steps_and_checks_in_a_ctl_session() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let setups = root.join("docs/development/qa/setups");
    let chain = Chain::read(&setups, "ctl-hello").expect("ctl-hello chain");
    assert_eq!(chain.names().collect::<Vec<_>>(), ["ctl-hello"]);
    assert_ne!(chain.hash(), [0; 32]);
    let listed = command::catalogue(&Reader::Directory(&setups), vec!["ctl-hello".into()]);
    assert!(
        matches!(listed.as_slice(), [line] if line.starts_with("ctl-hello: ") && !line.contains(": refused:")),
        "{listed:?}"
    );
    let design = std::fs::read_to_string(root.join("docs/design/testing/qa/setups.md")).expect("setups design");
    let block = design.split("  ```text\n").nth(1).expect("recipe block").split("  ```").next().expect("block end");
    let documented = block.lines().fold(String::new(), |mut text, line| {
        text.push_str(line.strip_prefix("  ").expect("indented block"));
        text.push('\n');
        text
    });
    assert_eq!(
        std::str::from_utf8(&chain.recipes[0].bytes).expect("ASCII recipe"),
        documented,
        "recipe bytes are the design's block"
    );
    assert_eq!(chain.recipes[0].steps.len(), 5);
    assert_eq!(chain.recipes[0].check.len(), 2);
    let lines = chain.lines(None).expect("fresh ctl walk");
    assert_eq!(
        shown(lines.clone()),
        [
            "ctl-hello:2 ctl project make Hello --key qa-hello-project",
            "ctl-hello:3 ctl forge connect forgejo-admin {forgejo.admin} --first work/hello --project Hello --api {forgejo.api} --git {forgejo.git} --git-program {git} --key qa-hello",
            "ctl-hello:4 ctl forge list --until made:1",
            "ctl-hello:5 ctl account add-key 1 {model.key} --key qa-model",
            "ctl-hello:6 ctl account list --until usable",
            "ctl-hello:8 ctl project show Hello",
            "ctl-hello:9 ctl account list --until usable",
        ]
    );
    let scratch = skein_scratch::Scratch::new("recipe-ctl-hello");
    std::fs::create_dir(scratch.path().join("session")).expect("session");
    let log = b"box: effective data /box/data\nbox: fake forgejo http://127.0.0.1:4321\nbox: Forgejo elsewhere admin token file /box/admin.token\nbox: model key /box/model.key\n";
    std::fs::write(scratch.path().join("session/box.log"), log).expect("opening lines");
    let mut options = command::session_options(
        &chain,
        &setups,
        &args(&["--bin", "/build", "--bundle", "/bundle", "--scratch", "/scratch"]),
    )
    .expect("options");
    options.scratch = scratch.path().into();
    assert!(options.ctl);
    run_ctl_hello(options, lines, scratch.path());
}

fn run_ctl_hello(options: drive::Options, lines: Vec<WalkLine>, scratch: &Path) {
    use drive::{Action, Event, Input, Machine, Output, Plan, Process};
    use skein_lib::Time;

    let plan = Plan::new(options, std::path::PathBuf::new(), OsStr::new("/bin"));
    let (mut machine, _) = Machine::start(plan, Time::ZERO);
    machine.step(
        Event::BoxLine { output: Output::Stdout, line: b"http://127.0.0.1:9/link?secret=first".as_slice().into() },
        Time::ZERO,
    );
    let actions = machine.step(
        Event::Exited { process: Process::SignIn, code: 0, stdout: Box::new([]), stderr: Box::new([]) },
        Time::ZERO,
    );
    assert!(matches!(actions.as_slice(), [Action::RemoveLink, Action::Ready]));
    for line in lines {
        let mut actions = machine.step(Event::Line(line.line.clone()), Time::ZERO);
        if let [Action::ReadBoxLog { path }] = actions.as_slice() {
            assert_eq!(path, &scratch.join("session/box.log"));
            actions = machine.step(Event::BoxLog(Ok(std::fs::read(path).expect("box log").into())), Time::ZERO);
        }
        let [Action::Start { process: Process::Ctl, program }] = actions.as_slice() else {
            panic!("{} must start ctl, not refuse a surface: {actions:?}", line.source)
        };
        let mut resolved = String::from_utf8(line.line.to_vec()).expect("ASCII ctl line");
        for (name, value) in [
            ("{forgejo.admin}", "/box/admin.token"),
            ("{forgejo.api}", "127.0.0.1:4321"),
            ("{forgejo.git}", "http://127.0.0.1:4321"),
            ("{git}", "/box/data/demo/bin/git"),
            ("{model.key}", "/box/model.key"),
        ] {
            resolved = resolved.replace(name, value);
        }
        let mut expected: Vec<OsString> = vec!["ctl".into(), scratch.join("session/ctl").into()];
        expected.extend(resolved.split_whitespace().skip(1).map(OsString::from));
        assert_eq!(program.args, expected, "{}", line.source);
        assert_eq!(program.program, Path::new("/build/temper"));
        assert_eq!(program.input, Input::Null);
        let actions = machine.step(
            Event::Exited { process: Process::Ctl, code: 0, stdout: Box::new([]), stderr: Box::new([]) },
            Time::ZERO,
        );
        assert!(
            matches!(actions.as_slice(), [Action::Keep(kept), Action::Answer(answer)]
            if kept == &line.line && answer.failed.is_none()),
            "{}: {actions:?}",
            line.source
        );
        assert!(machine.ready());
    }
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
    let expected: RecipeFiles = tree
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
    let Record::Forge(Forge::Commit { repository, message, paths, .. }) =
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
    assert_eq!(hex(hash), "4594012646a0515f3245725de9d2c7e1869ba8cf53a6894d3d9ca2f4d827f0d5");
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
fn config(files: &mut Files, name: &str) {
    for document in ["engine.json", "worker.json", "agent.json", "forgejo.json", "model.json", "providers.json"] {
        put(files, name, &format!("config/{document}"), b"opaque, not parsed here");
    }
}
fn deployment_chain() -> Files {
    let mut files = ctl_chain();
    config(&mut files, "root");
    files
}

#[test]
fn deployment_config_requires_exactly_five_bounded_documents_without_parsing() {
    let mut files = deployment_chain();
    put(&mut files, "root", "config/engine.json", &vec![b'x'; 262_144]);
    let chain = read(&files, "child").expect("256 KiB inclusive, opaque documents");
    assert_eq!(chain.kind(), Kind::Deployment);
    assert_eq!(chain.names().collect::<Vec<_>>(), ["root", "child"]);
    assert_eq!(chain.config_dir(Path::new("/recipes")), Some("/recipes/root/config".into()));
    for document in ["engine.json", "worker.json", "agent.json", "forgejo.json", "model.json", "providers.json"] {
        let mut missing = files.clone();
        let path = format!("config/{document}");
        missing.get_mut("root").expect("root").remove(OsStr::new(&path));
        let refusal = read(&missing, "child").map(|_| ()).expect_err("every document required");
        assert_eq!((refusal.name.as_str(), refusal.file.as_deref()), ("root", Some(path.as_str())));
        assert!(refusal.why.contains("missing"), "{refusal}");
    }
    for (path, bytes, why) in [
        ("config/extra.json", b"extra".to_vec(), "unexpected document"),
        ("config/sub/engine.json", b"nested".to_vec(), "unexpected document"),
        ("config/engine.json", vec![b'x'; 262_145], "256 KiB"),
    ] {
        let mut invalid = files.clone();
        put(&mut invalid, "root", path, &bytes);
        let refusal = read(&invalid, "child").map(|_| ()).expect_err(path);
        assert_eq!((refusal.name.as_str(), refusal.file.as_deref()), ("root", Some(path)));
        assert!(refusal.why.contains(why), "{refusal}");
    }
}

#[test]
fn config_belongs_only_to_a_root_without_after() {
    for name in ["child", "root"] {
        let mut files = deployment_chain();
        if name == "child" {
            config(&mut files, name);
        } else {
            put(&mut files, name, "recipe", b"recipe 1\nafter \"parent\"\ncheck\nctl project list\n");
        }
        let refusal = read(&files, "child").expect_err("config and after refuse before following parent");
        assert_eq!((refusal.name.as_str(), refusal.file.as_deref()), (name, Some("config")));
        assert!(refusal.why.contains("after"), "{refusal}");
    }
}

#[test]
fn chain_kind_refuses_the_other_kinds_lines_and_deployment_checks() {
    for (deployment, after_check, why) in
        [(false, false, "a box recipe takes no deployment line"), (true, true, "may follow check")]
    {
        let mut files = if deployment { deployment_chain() } else { ctl_chain() };
        let source = if after_check {
            b"recipe 1\ncheck\ndeployment patch engine \"patch.json\"\n".as_slice()
        } else {
            b"recipe 1\ndeployment patch engine \"patch.json\"\ncheck\nctl project list\n"
        };
        put(&mut files, "root", "recipe", source);
        let refusal = read(&files, "child").expect_err(why);
        assert_eq!((refusal.name.as_str(), refusal.line), ("root", Some(if after_check { 3 } else { 2 })));
        assert!(refusal.why.contains(why), "{refusal}");
    }
}

#[test]
fn deployment_web_and_tui_chain_plans_a_full_session_with_bundle_ports_and_resets() {
    let mut files = deployment_chain();
    put(
        &mut files,
        "child",
        "recipe",
        b"recipe 1\nafter \"root\"\nweb reload \"/projects/1\"\ntui see \"Ask temper\"\n",
    );
    let chain = read(&files, "child").expect("deployment accepts web and tui");
    let setups = Path::new("/recipes");
    let base = ["--bin", "/build", "--scratch", "/scratch"];
    let no_ports = args(&base);
    assert!(command::session_options(&chain, setups, &no_ports).expect_err("ports required").contains("--ports"));
    let mut arguments = args(&base);
    arguments.extend(args(&["--ports", "1,2,3,4,5,6,7,8,9"]));
    assert!(command::session_options(&chain, setups, &arguments).expect_err("bundle required").contains("--bundle"));
    arguments.extend(args(&["--bundle", "/p/bundle"]));
    let options = command::session_options(&chain, setups, &arguments).expect("full deployment walk");
    assert!(!options.ctl);
    assert_eq!(options.deployment, Some("/recipes/root/config".into()));
    assert_eq!(options.bundle, Some("/p/bundle".into()));
    assert_eq!(options.ports.as_deref(), Some("1,2,3,4,5,6,7,8,9"));
    let lines = shown(chain.lines(Some("root")).expect("from root copy"));
    assert_eq!(
        &lines[lines.len() - 4..],
        [
            "child:3 web reload \"/projects/1\"",
            "walk: web reload \"/\"",
            "walk: tui stop",
            "child:4 tui see \"Ask temper\""
        ]
    );
}

#[test]
fn deployment_hash_frames_config_after_files_in_name_byte_order() {
    let files = deployment_chain();
    let mut chain = read(&files, "child").expect("deployment");
    let hash = chain.hash();
    assert_eq!(hex(hash), "f7b7c1ecb80b12c49145a1ebbfc9d9c06c5afc5bd0956846454203a6ac6946b2");
    let mut changed = files.clone();
    put(&mut changed, "root", "config/engine.json", b"Opaque, not parsed here");
    assert_ne!(read(&changed, "child").expect("one changed byte").hash(), hash);
    let bytes = chain.recipes[0].config.remove(b"engine.json".as_slice()).expect("engine");
    chain.recipes[0].config.insert(b"renamed.json".as_slice().into(), bytes);
    assert_ne!(chain.hash(), hash, "name alone counts, without changing recipe bytes");
}

#[test]
fn deployment_walk_passes_root_config_ports_and_each_recipes_patch_files_without_resets() {
    use command::{Arguments, Mode, session_options};
    let mut files = deployment_chain();
    for (name, after, bytes) in [("root", "", b"root patch".as_slice()), ("child", "after \"root\"\n", b"child patch")]
    {
        put(
            &mut files,
            name,
            "recipe",
            format!("recipe 1\n{after}deployment patch engine \"patch.json\"\ncheck\nctl project list\n").as_bytes(),
        );
        put(&mut files, name, "files/patch.json", bytes);
        put(&mut files, name, "README.md", format!("The {name}.").as_bytes());
    }
    let chain = read(&files, "child").expect("patches named by deployment lines");
    let setups = Path::new("/recipes");
    let ports = "1,2,3,4,5,6,7,8,9";
    let arguments = Arguments::parse(
        Mode::Walk,
        &args(&["child", "--bin", "/build", "--scratch", "/scratch", "--ports", ports, "--setups", "/recipes"]),
    )
    .expect("deployment needs no bundle");
    let options = session_options(&chain, &arguments.setups, &arguments.drive.expect("walk arguments"))
        .expect("deployment session");
    assert_eq!(options.deployment, chain.config_dir(setups));
    assert_eq!(options.ports.as_deref(), Some(ports));
    assert!(options.bundle.is_none());
    assert!(options.ctl, "no web or tui lines selects --ctl --deployment");
    let fresh = chain.lines(None).expect("fresh lines");
    assert_eq!(
        shown(fresh.clone()),
        [
            "root:2 deployment patch engine \"patch.json\"",
            "root:4 ctl project list",
            "child:3 deployment patch engine \"patch.json\"",
            "child:5 ctl project list"
        ]
    );
    assert_eq!(shown(chain.lines(Some("root")).expect("root copy")), shown(fresh.clone())[1..]);
    assert_eq!(shown(chain.lines(Some("child")).expect("child copy")), shown(fresh.clone())[3..]);
    for (line, name, bytes) in [(&fresh[0], "root", b"root patch".as_slice()), (&fresh[2], "child", b"child patch")] {
        let directory = line.files(setups).expect("own files");
        assert_eq!(directory, setups.join(name).join("files"));
        assert_eq!(
            chain.recipes.iter().find(|r| r.name == name).expect("recipe").files[b"patch.json".as_slice()],
            bytes
        );
    }
    let no_ports = args(&["--bin", "/build", "--scratch", "/scratch"]);
    assert!(
        session_options(&chain, setups, &no_ports)
            .expect_err("ports required before session start")
            .contains("--ports")
    );
    let with_bundle = args(&["--bin", "/build", "--scratch", "/scratch", "--ports", ports, "--bundle", "/bundle"]);
    assert_eq!(session_options(&chain, setups, &with_bundle).expect("optional bundle").bundle, Some("/bundle".into()));
    let box_chain = read(&ctl_chain(), "child").expect("box");
    assert!(session_options(&box_chain, setups, &no_ports).expect_err("box requires bundle").contains("--bundle"));
    assert_eq!(
        command::catalogue(&Reader::Memory(&files), vec!["root".into(), "child".into()]),
        ["child (after root): The child.", "root: The root."]
    );
    for name in ["root", "child"] {
        let mut missing = files.clone();
        missing.get_mut(name).expect("recipe").remove(OsStr::new("files/patch.json"));
        let refusal = read(&missing, "child").expect_err("patch must be in its own recipe");
        assert_eq!((refusal.name.as_str(), refusal.file.as_deref()), (name, Some("files/patch.json")));
        assert!(refusal.why.contains("missing"), "{refusal}");
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
    let options =
        session_options(&chain, &arguments.setups, &arguments.drive.expect("drive")).expect("session options");
    assert!(!options.ctl, "a mixed chain keeps its browser session");
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
        (Some("middle"), false, true, true),
        (Some("middle"), true, true, true),
    ] {
        let result = walk::check_scratch(from, config, data);
        assert_eq!(result.is_ok(), accepted, "{from:?} {config} {data}");
        if let Err(why) = result {
            assert!(why.contains("config/") && why.contains("data/"));
        }
    }
    let missing = Arguments::parse(Mode::Walk, &args(&["last"])).expect("kind checked after reading chain");
    assert!(session_options(&chain, &missing.setups, &missing.drive.expect("walk arguments")).is_err());
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
    assert_eq!(hex(hash), "9784e219ded22670cf0f3ea68d80bd3ca9760b33f247f7acb5683fae8caf3b11");
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

#[test]
fn recipe_keeps_a_push_line_and_its_files_for_a_ctl_walk() {
    let mut files = Files::new();
    let source = r#"forge push forgejo work/hello work/topic "message" "a.txt""#;
    put(&mut files, "pushed", "recipe", format!("recipe 1\n{source}\ncheck\nctl change list\n").as_bytes());
    put(&mut files, "pushed", "files/a.txt", b"pushed bytes");
    let chain = read(&files, "pushed").expect("push recipe");
    assert_eq!(chain.recipes[0].files[b"a.txt".as_slice()], b"pushed bytes");
    assert!(chain.lines(None).expect("walk lines").iter().any(|line| line.line.as_ref() == source.as_bytes()));
    files.get_mut("pushed").expect("recipe").remove(OsStr::new("files/a.txt"));
    assert!(read(&files, "pushed").map(|_| ()).expect_err("missing push file").why.contains("missing"));
}

#[test]
fn settings_files_belong_to_their_recipe_and_its_hash() {
    let mut files = Files::new();
    let source = b"recipe 1\nctl settings apply landing.json --against 1 --key landing\ncheck\nctl settings show\n";
    put(&mut files, "root", "recipe", source);
    let refusal = read(&files, "root").expect_err("settings input missing");
    assert!(refusal.to_string().contains("files/landing.json"), "{refusal}");
    put(&mut files, "root", "files/landing.json", b"first settings");
    let first = read(&files, "root").expect("settings file belongs to the recipe");
    put(&mut files, "root", "files/landing.json", b"changed settings");
    assert_ne!(first.hash(), read(&files, "root").expect("changed settings").hash());
    put(&mut files, "child", "recipe", b"recipe 1\nafter \"root\"\nctl settings propose landing.json --against 1 --key proposal\ncheck\nctl settings show\n");
    assert!(
        read(&files, "child").expect_err("parent's file is not child's").to_string().contains("files/landing.json")
    );
    put(&mut files, "child", "files/landing.json", b"child settings");
    let chain = read(&files, "child").expect("each recipe has its file");
    assert_ne!(chain.recipes[0].files, chain.recipes[1].files);
}
