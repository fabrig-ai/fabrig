//! Pass places through the command entry and scripted programs (temper/docs/design/testing/qa/places.md, 4 and 6).
use super::{Reply, invoke, output, put, run, success};
use serde_json::json;
use std::{
    ffi::{OsStr, OsString},
    path::{Path, PathBuf},
};
use temper_qa_tool::{
    CaseInput, Exit, Kind,
    faces::{DirectoryMode, FilesFace},
    place::PlacePaths,
    testing::{Agents, Git, Node, World},
};

const COMMIT: &str = "0123456789abcdef0123456789abcdef01234567";
const HASH: &str = "1111111111111111111111111111111111111111111111111111111111111111";
const RECIPE: &[u8] = br#"{"after":"fresh","lines":["web see \"Added\""],"readme":"An added state."}"#;
const COPY: &[u8] = b"saved config and data";

fn pass_home(kind: Kind, commit: &str) -> (World, PlacePaths, PathBuf) {
    let world = World::new();
    let home = &world.home;
    let mut files = world.files();
    let (case, setups, deployment_setups) = if kind == Kind::Deployment {
        let path = home.directory.join("qa.json");
        let mut config: serde_json::Value = serde_json::from_slice(&files.read(&path).expect("config")).expect("JSON");
        config["places"] = json!([{"directory":"places/1","kind":"deployment","ports":[31001,31002,31003,31004,31005,31006,31007,31008,31009]}]);
        put(&mut files, &path, serde_json::to_vec(&config).expect("JSON"));
        (
            br#"{"surface":"ctl","page":"task","setup":"turn-landing","steps":[{"do":"ctl task list","expect":"listed"}]}"#.as_slice(),
            vec!["fresh".into(), "turn-landing".into()],
            vec!["turn-landing".into()],
        )
    } else {
        (
            br#"{"surface":"web","page":"work","setup":"fresh","steps":[{"do":"open","expect":"Work"}]}"#.as_slice(),
            vec!["fresh".into()],
            Vec::new(),
        )
    };
    home.transact(&mut files, |ledger, _| {
        ledger.record_landing(commit, 10, None, "Work", 0);
        let triage = ledger.start_session(
            1,
            "low",
            [("web".into(), vec!["work".into()]), ("ctl".into(), vec!["task".into()])].into(),
            setups,
            deployment_setups,
            0,
        )?;
        ledger.add_case(triage, &CaseInput::decode(case, "case")?, 1)?;
        ledger.end_session(triage, Exit::Code(0), None, 2, "qa")?;
        ledger.keep_build(commit, 3)?;
        ledger.set_build_holds(commit, vec![1], 1)?;
        let started = ledger.start_pass("places/1", kind, &["web".into(), "ctl".into()], "high", 4).expect("pass");
        assert_eq!((started.pass, started.session), (1, 2));
        Ok(())
    })
    .expect("place pass");
    let place = PlacePaths::new(&home.directory.join("places/1"), kind);
    place.ensure(&mut files).expect("place");
    let setups = home.directory.join("passes/1/setups");
    (world, place, setups)
}

fn shell_capture(environment: &temper_qa_tool::faces::EnvironmentEntry) -> bool {
    environment.0.as_ref() == b"PATH" && environment.1.as_ref() == b"/qa/test-bin"
}

fn programs(
    world: &World,
    place: &PlacePaths,
    setups: &Path,
    walk_exit: i32,
    recipe_file: Option<(&'static str, &'static [u8])>,
) -> Agents {
    let mut agents = world.agents();
    let paths = place.clone();
    let setups = setups.to_path_buf();
    agents.register("temper-client-journeys", move |request, files| {
        assert_eq!(request.program, paths.bin.join("temper-client-journeys"));
        assert_eq!(request.directory, Some(paths.directory.as_path()));
        assert!(request.environment.iter().any(shell_capture));
        let args: Vec<&OsStr> = request.arguments.iter().map(OsString::as_os_str).collect();
        assert_eq!(args[0], "recipe");
        let result = match args[1].to_str().expect("drive verb") {
            "list" => {
                assert_eq!(args[2..], [OsStr::new("--setups"), setups.as_os_str()]);
                let mut list = String::from("welcome: Welcome.\n");
                if files.exists(setups.join("added/recipe")) {
                    list.push_str("added: Added.\n");
                }
                output(0, list)
            }
            "hash" => {
                assert_eq!(args[3..], [OsStr::new("--setups"), setups.as_os_str()]);
                let recipe = setups.join(args[2]);
                assert!(files.exists(recipe.join("recipe")));
                if let Some((name, bytes)) = recipe_file
                    && !files.read(&recipe.join("files").join(name)).is_ok_and(|copied| copied == bytes)
                {
                    return Ok(temper_qa_tool::faces::ProcessOutput {
                        stderr: format!("files/{name}: missing or different settings file\n").into_bytes(),
                        ..output(1, b"")
                    });
                }
                output(0, format!("{HASH}\n"))
            }
            "walk" => {
                let name = args[2].to_str().expect("recipe name");
                assert_eq!(request.log, Some(paths.walk_log.as_path()));
                let flags = &args[3..];
                let flags = if flags.starts_with(&[OsStr::new("--from")]) {
                    assert_eq!(flags[1], "added");
                    assert_eq!(name, "added");
                    &flags[2..]
                } else {
                    flags
                };
                assert_eq!(
                    flags,
                    [
                        OsStr::new("--bin"),
                        paths.bin.as_os_str(),
                        OsStr::new("--bundle"),
                        paths.bundle.as_os_str(),
                        OsStr::new("--scratch"),
                        paths.scratch.as_os_str(),
                        OsStr::new("--setups"),
                        setups.as_os_str(),
                        OsStr::new("--strict"),
                    ]
                );
                if walk_exit != 0 {
                    output(walk_exit, format!("stopped at {name}:2: missing Work\n"))
                } else {
                    for member in ["config", "data"] {
                        files.directory(&paths.scratch.join(member), DirectoryMode::Parents)?;
                    }
                    output(0, format!("walked {name} {HASH}\n"))
                }
            }
            verb => panic!("unexpected drive verb {verb}"),
        };
        Ok(result)
    });
    let paths = place.clone();
    agents.register("tar", move |request, files| {
        assert!(request.environment.iter().any(shell_capture));
        let args = request.arguments;
        assert_eq!(args[1], "--file");
        assert_eq!(args[3], "--directory");
        assert_eq!(args[4], paths.scratch);
        let archive = Path::new(&args[2]);
        match args[0].to_str().expect("archive verb") {
            "--create" => {
                assert_eq!(archive, paths.copies.join("added/copy.tmp"));
                assert_eq!(args[5..], ["config", "data"]);
                put(files, archive, COPY);
            }
            "--extract" => {
                assert_eq!(archive, paths.copies.join("added").join(format!("{HASH}-{COMMIT}.tar")));
                assert_eq!(args.len(), 5);
                assert_eq!(files.read(archive)?, COPY);
                assert!(!files.exists(paths.scratch.join("data/stale")));
                for member in ["config", "data"] {
                    files.directory(&paths.scratch.join(member), DirectoryMode::Parents)?;
                }
            }
            verb => panic!("unexpected archive verb {verb}"),
        }
        Ok(output(0, b""))
    });
    agents
}

fn run_at(world: &World, agents: Agents, arguments: &[&str]) -> Reply {
    let mut git = Git::new(world.files());
    // The captured git program's empty build setup catalogue permits a pass-local rewrite.
    git.nodes.insert(COMMIT.into(), Node::default());
    invoke(world, agents, git, arguments, Some("2"), None, &[("PATH", "/qa/test-bin")])
}

#[test]
fn setup_and_recipe_preserve_walk_exits_and_use_the_pass_place_and_captured_processes() {
    let (world, place, setups) = pass_home(Kind::Box, COMMIT);
    let mut files = world.files();
    let before = files.read(&world.home.ledger).expect("ledger");
    let welcome = setups.join("welcome");
    files.directory(&welcome, DirectoryMode::Parents).expect("setups");
    put(&mut files, &welcome.join("recipe"), b"recipe 1\nweb see \"Welcome\"\n");

    let missing = run(&world, &["pass", "setup", "fresh"], None, None);
    assert_eq!(missing.code, 1);
    assert_eq!(
        missing.stderr,
        b"temper-qa: TEMPER_INTERNAL_QA_SESSION is not set: pass commands run in a pass session\n"
    );
    assert_eq!(missing.stdout, b"");
    let stopped = run_at(&world, programs(&world, &place, &setups, 5, None), &["pass", "setup", "welcome"]);
    assert_eq!(stopped.code, 5, "{stopped:?}");
    assert_eq!(stopped.stderr, b"", "{stopped:?}");
    assert!(String::from_utf8_lossy(&stopped.stdout).contains("stopped at welcome:2: missing Work"));

    let fresh = run(&world, &["pass", "setup", "fresh"], Some("2"), None);
    success(&fresh, b"setup fresh: an empty box\n");

    // The shipped stdin witness writes the recipe and its archive; stage that result for the restore.
    let added = setups.join("added");
    files.directory(&added, DirectoryMode::Parents).expect("pass recipe");
    put(&mut files, &added.join("recipe"), b"recipe 1\nweb see \"Added\"\n");
    let copies = place.copies.join("added");
    files.directory(&copies, DirectoryMode::One).expect("copies");
    let archive = copies.join(format!("{HASH}-{COMMIT}.tar"));
    put(&mut files, &archive, COPY);
    files.directory(&place.scratch.join("data"), DirectoryMode::One).expect("stale box");
    put(&mut files, &place.scratch.join("data/stale"), b"not in copy");

    let restored = run_at(&world, programs(&world, &place, &setups, 0, None), &["pass", "setup", "added"]);
    assert_eq!(restored.code, 0, "{restored:?}");
    assert_eq!(restored.stderr, b"", "{restored:?}");
    assert!(
        String::from_utf8_lossy(&restored.stdout).contains("setup added: restored from its copy made at 0123456789ab")
    );
    assert!(!files.exists(place.scratch.join("data/stale")));
    assert_eq!(files.read_to_string(&place.setup).expect("setup"), format!("added {HASH}\n"));

    put(&mut files, &world.home.directory.join("recipe.json"), RECIPE);
    let rewritten =
        run_at(&world, programs(&world, &place, &setups, 0, None), &["pass", "recipe", "added", "recipe.json"]);
    assert_eq!(rewritten.code, 0, "{rewritten:?}");
    assert_eq!(rewritten.stderr, b"", "{rewritten:?}");
    assert_eq!(files.read(&added.join("README.md")).expect("rewritten recipe"), b"An added state.\n");
    assert_eq!(files.read(&world.home.ledger).expect("unchanged ledger"), before);
}

#[test]
fn pass_recipe_keeps_the_file_named_by_settings_apply() {
    const PROVIDER: &str = "provider.json";
    const SETTINGS: &[u8] = br#"{"provider":"example"}"#;
    const UNNAMED: &str = "unnamed.json";
    const INPUT: &[u8] = br#"{"after":"fresh","lines":["ctl settings apply provider.json","check","ctl settings show"],"readme":"A provider setting."}"#;
    let (world, place, setups) = pass_home(Kind::Box, COMMIT);
    let mut files = world.files();
    let pass_files = world.home.directory.join("passes/1/files");
    files.directory(&pass_files, DirectoryMode::Parents).expect("pass files");
    put(&mut files, &pass_files.join(PROVIDER), SETTINGS);
    put(&mut files, &pass_files.join(UNNAMED), b"{}");
    put(&mut files, &world.home.directory.join("recipe.json"), INPUT);

    let agents = programs(&world, &place, &setups, 0, Some((PROVIDER, SETTINGS)));
    let added = run_at(&world, agents, &["pass", "recipe", "added", "recipe.json"]);
    assert_eq!(added.code, 0, "{added:?}");
    assert_eq!(added.stderr, b"", "{added:?}");
    assert!(
        String::from_utf8_lossy(&added.stdout)
            .lines()
            .any(|line| line == "recipe added walked from fresh: 111111111111"),
        "{added:?}"
    );
    let recipe = setups.join("added");
    assert_eq!(files.read(&recipe.join("files").join(PROVIDER)).expect("copied settings"), SETTINGS);
    assert!(!files.exists(recipe.join("files").join(UNNAMED)));
    assert_eq!(
        files.read(&recipe.join("recipe")).expect("recipe"),
        b"recipe 1\nctl settings apply provider.json\ncheck\nctl settings show\n"
    );
}

#[test]
fn a_deployment_place_refuses_fresh_and_box_setups_with_exit_one() {
    let (world, place, setups) = pass_home(Kind::Deployment, "build");
    let mut files = world.files();
    let before = files.read(&world.home.ledger).expect("ledger");
    let fresh = run(&world, &["pass", "setup", "fresh"], Some("2"), None);
    assert_eq!(fresh.code, 1);
    assert_eq!(
        fresh.stderr,
        b"temper-qa: a deployment's place has no fresh setup: its setups start from their root's config/\n"
    );
    assert_eq!(fresh.stdout, b"");
    files.directory(&setups.join("welcome"), DirectoryMode::Parents).expect("setup");
    put(&mut files, &setups.join("welcome/recipe"), b"recipe 1\ncheck\nctl task list\n");
    let mut agents = world.agents();
    let program = place.bin.join("temper-client-journeys");
    let directory = place.directory.clone();
    agents.register(program.clone(), move |request, _| {
        assert_eq!(request.program, program);
        assert_eq!(request.directory, Some(directory.as_path()));
        assert_eq!(
            request.arguments,
            ["recipe".into(), "list".into(), "--setups".into(), setups.clone().into_os_string()]
        );
        Ok(output(0, b"welcome: A box.\n"))
    });
    let boxed = run_at(&world, agents, &["pass", "setup", "welcome"]);
    assert_eq!(boxed.code, 1);
    assert_eq!(boxed.stderr, b"temper-qa: setup welcome is a box's; places/1 is a deployment's place\n");
    assert_eq!(boxed.stdout, b"");
    assert!(!files.exists(place.setup));
    assert_eq!(files.mode(place.scratch), Some(0o700));
    assert_eq!(files.read(&world.home.ledger).expect("unchanged ledger"), before);
}
