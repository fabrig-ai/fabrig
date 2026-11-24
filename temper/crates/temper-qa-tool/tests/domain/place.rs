//! Places use scripted programs and directory copies in the QA tree (temper/docs/design/testing/qa/places.md, 8).
use std::{
    ffi::{OsStr, OsString},
    fmt::Write as _,
    io,
    os::unix::ffi::{OsStrExt, OsStringExt},
    path::{Path, PathBuf},
};
use temper_qa_tool::{
    Exit, Home, Kind, Refusal,
    faces::{
        ArchiveFace, DirectoryMode, FilesFace, LockMode, OutputCommand, PathKind, ProcessOutput, Processes,
        RealArchive, RealDrive, RemoveMode, WriteMode, WriteOptions, copy_members,
    },
    place::{PassPlace, PlacePaths, RecipeInput, place_busy},
    testing::{Agents, Tree, passes},
};

const OLD: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const BUILD: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
const WELCOME: &str = "1111111111111111111111111111111111111111111111111111111111111111";
const PROJECT: &str = "2222222222222222222222222222222222222222222222222222222222222222";
const CHAT: &str = "3333333333333333333333333333333333333333333333333333333333333333";
const ADDED: &str = "4444444444444444444444444444444444444444444444444444444444444444";
const CATALOGUE: &str =
    "chat-draft (after project-landing): A draft.\nproject-landing (after welcome): A project.\nwelcome: Welcome!\n";

fn write(tree: &mut Tree, path: impl AsRef<Path>, bytes: impl AsRef<[u8]>) -> io::Result<()> {
    tree.write(path.as_ref(), bytes.as_ref(), WriteOptions::REPLACE, false)
}

fn text(tree: &Tree, path: impl AsRef<Path>) -> io::Result<String> {
    String::from_utf8(tree.clone().read(path.as_ref())?).map_err(|why| io::Error::new(io::ErrorKind::InvalidData, why))
}

fn answer(code: u8, stdout: impl Into<Vec<u8>>, stderr: impl Into<Vec<u8>>) -> ProcessOutput {
    ProcessOutput {
        exit: Exit::Code(code.into()),
        status: format!("exit status: {code}"),
        stdout: stdout.into(),
        stderr: stderr.into(),
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Journeys {
    Box,
    Ctl,
    CheckCopy,
    Deployment,
    DeploymentSeed,
}

impl Journeys {
    fn run(self, command: &OutputCommand<'_>, tree: &mut Tree) -> io::Result<ProcessOutput> {
        assert!(command.environment.iter().any(|(name, value)| &**name == b"QA_RECIPE_ENV" && &**value == b"captured"));
        if self == Self::Ctl {
            assert!(!command.environment.iter().any(|(name, _)| &**name == b"SKEIN_TEST_CHROMIUM"));
        }
        let directory = command.directory.expect("the place's directory");
        assert_eq!(Path::new(command.program), directory.join("bin/temper-client-journeys"));
        let mut tree = tree.at(directory)?;
        let arguments: Vec<_> = command.arguments.iter().map(|arg| arg.to_str().expect("recipe argument")).collect();
        append_arguments(&mut tree, Path::new("arguments"), &arguments)?;
        match arguments.as_slice() {
            ["recipe", "list", "--setups", setups] => {
                assert_eq!(tree.kind(setups), Some(PathKind::Directory));
                Ok(answer(0, tree.read(Path::new("catalogue"))?, []))
            }
            ["recipe", "hash", name, "--setups", setups] => {
                let refusal = PathBuf::from(format!("refuse-{name}"));
                if tree.exists(&refusal) {
                    return Ok(answer(1, [], tree.read(&refusal)?));
                }
                let recipe = Path::new(setups).join(name);
                check_recipe_files(&mut tree, name, &recipe)?;
                if self == Self::Ctl {
                    assert_eq!(tree.read(Path::new(&format!("expected-{name}")))?, tree.read(&recipe.join("recipe"))?);
                }
                Ok(answer(0, tree.read(Path::new(&format!("hash-{name}")))?, []))
            }
            ["recipe", "walk", name, options @ ..] => self.walk(directory, name, options, &mut tree),
            _ => panic!("unexpected journeys arguments: {arguments:?}"),
        }
    }

    fn walk(self, directory: &Path, name: &str, options: &[&str], tree: &mut Tree) -> io::Result<ProcessOutput> {
        let (from, options) = match options {
            ["--from", from, rest @ ..] => (*from, rest),
            _ => ("fresh", options),
        };
        let ["--bin", bin, "--bundle", bundle, "--scratch", scratch, "--setups", setups, "--strict", ports @ ..] =
            options
        else {
            panic!("unexpected walk options: {options:?}");
        };
        assert!(matches!(ports, [] | ["--ports", _]), "unexpected ports option: {ports:?}");
        let deployment = matches!(self, Self::Deployment | Self::DeploymentSeed);
        let scratch_name = if deployment { "deployment" } else { "box" };
        let scratch = Path::new(scratch);
        assert_eq!(Path::new(bin), directory.join("bin"));
        assert_eq!(Path::new(bundle), directory.join("bundle"));
        assert_eq!(scratch, directory.join(scratch_name));
        assert_eq!(tree.kind(setups), Some(PathKind::Directory));
        check_recipe_files(tree, name, &Path::new(setups).join(name))?;
        for member in ["config", "data"] {
            let path = scratch.join(member);
            if from == "fresh" {
                assert!(!tree.exists(path));
            } else {
                assert_eq!(tree.kind(path), Some(PathKind::Directory));
            }
        }
        let failure = PathBuf::from(format!("walk-{name}-{from}"));
        if tree.exists(&failure) {
            let mut log = String::new();
            for n in 1..=45 {
                writeln!(log, "answer {n}").expect("log line");
            }
            writeln!(log, "stopped at {name}:7: missing Start").expect("stop line");
            let code = text(tree, &failure)?.parse().expect("scripted walk exit");
            return Ok(answer(code, log, []));
        }
        for member in ["config", "data"] {
            tree.directory(&scratch.join(member), DirectoryMode::Parents)?;
        }
        if deployment {
            let receipt = scratch.join("config/receipt");
            if from == "fresh" {
                write(tree, receipt, "saved config")?;
            } else {
                assert_eq!(text(tree, receipt)?, "saved config");
            }
        }
        if self != Self::CheckCopy {
            write(tree, scratch.join("data/state"), format!("{name}\n"))?;
            let agent = scratch.join(format!("data/{scratch_name}/fake-backends/data/workspaces/agent"));
            tree.directory(&agent, DirectoryMode::Parents)?;
            write(tree, agent.join("temper-box"), b"pinned\n")?;
        }
        if self == Self::DeploymentSeed {
            copy(tree, &directory.join("seed"), scratch, Path::new("data"), &[])?;
        }
        Ok(answer(0, format!("walked {name}\n"), []))
    }
}

fn append_arguments(tree: &mut Tree, path: &Path, arguments: &[&str]) -> io::Result<()> {
    let line = arguments.join(" ") + "\n";
    let options = WriteOptions { way: WriteMode::Append, ..WriteOptions::REPLACE };
    tree.write(path, line.as_bytes(), options, false)
}

fn check_recipe_files(tree: &mut Tree, name: &str, recipe: &Path) -> io::Result<()> {
    let expected = Path::new("expected-files");
    if name == "added" && tree.kind(expected) == Some(PathKind::Directory) {
        assert_same_files(tree, expected, &recipe.join("files"))?;
    }
    Ok(())
}

fn assert_same_files(tree: &mut Tree, expected: &Path, actual: &Path) -> io::Result<()> {
    let entries = tree.list(expected)?;
    assert_eq!(tree.list(actual)?, entries);
    for entry in entries {
        let name = OsStr::from_bytes(&entry.name);
        let from = expected.join(name);
        let to = actual.join(name);
        if entry.kind == PathKind::Directory {
            assert_same_files(tree, &from, &to)?;
        } else {
            assert_eq!(tree.read(&from)?, tree.read(&to)?, "{}", to.display());
        }
    }
    Ok(())
}

fn copy(
    files: &mut impl FilesFace,
    source: &Path,
    destination: &Path,
    relative: &Path,
    left_out: &[&str],
) -> io::Result<()> {
    if left_out.iter().any(|path| relative == Path::new(path)) {
        return Ok(());
    }
    let from = source.join(relative);
    let to = destination.join(relative);
    files.directory(&to, DirectoryMode::Parents)?;
    for entry in files.list(&from)? {
        let name = OsStr::from_bytes(&entry.name);
        if entry.kind == PathKind::Directory {
            copy(files, source, destination, &relative.join(name), left_out)?;
        } else {
            files.copy(&from.join(name), &to.join(name))?;
        }
    }
    Ok(())
}

fn assert_mode(tree: &Tree, path: &Path, expected: u32) {
    assert_eq!(tree.mode(path), Some(expected), "{}", path.display());
}

fn assert_copy_mode(files: &mut impl FilesFace, path: &Path) {
    assert_eq!(files.facts(path, true).expect("copy mode").mode, 0o600, "{}", path.display());
}

fn archive_marker(files: &mut impl FilesFace, archive: &Path) -> PathBuf {
    PathBuf::from(OsString::from_vec(files.read(archive).expect("archive marker")))
}

#[derive(Default)]
struct Archive {
    saved: Vec<PathBuf>,
    restored: Vec<PathBuf>,
    removed_before_retry: Option<PathBuf>,
    store: PathBuf,
}
impl ArchiveFace for Archive {
    fn create(
        &mut self,
        scratch: &Path,
        archive: &Path,
        left_out: &[&str],
        files: &mut impl FilesFace,
        _processes: &mut impl Processes,
    ) -> Result<(), Refusal> {
        assert_copy_mode(files, archive);
        self.saved.push(archive.to_owned());
        let snapshot = self.store.join(self.saved.len().to_string());
        for member in ["config", "data"] {
            copy(files, scratch, &snapshot, Path::new(member), left_out).expect("save member");
        }
        files.write(archive, snapshot.as_os_str().as_bytes(), WriteOptions::REPLACE, false).expect("archive marker");
        Ok(())
    }
    fn extract(
        &mut self,
        archive: &Path,
        scratch: &Path,
        left_out: &[&str],
        files: &mut impl FilesFace,
        _processes: &mut impl Processes,
    ) -> Result<(), Refusal> {
        if let Some(refused) = &self.removed_before_retry
            && archive != refused
        {
            assert!(
                files.facts(refused, false).is_err_and(|why| why.kind() == io::ErrorKind::NotFound),
                "refused copy must be removed before trying an ancestor"
            );
        }
        self.restored.push(archive.to_owned());
        let snapshot = archive_marker(files, archive);
        copy(files, &snapshot, scratch, Path::new(""), left_out).expect("restore snapshot");
        Ok(())
    }
}
struct PlaceWorld {
    tree: Tree,
    home: Home,
    session: u64,
    place: PassPlace<RealDrive, Archive, Tree, Agents<Tree>>,
}

impl PlaceWorld {
    fn new() -> Self {
        Self::with_ports(false)
    }

    fn with_ports(fixed: bool) -> Self {
        Self::with_kind(fixed, Kind::Box, Tree::default())
    }

    fn with_kind(fixed: bool, kind: Kind, mut tree: Tree) -> Self {
        let directory = Path::new("/qa");
        tree.directory(directory, DirectoryMode::One).expect("home directory");
        let mut config = serde_json::json!({"owner":"qa","git":{"directory":"unused"},"agent":{"command":["unused"],"triage":"low","pass":"high"},"request":{"command":["unused"]}});
        if fixed {
            config["places"] = serde_json::json!([{"directory":"places/1", "ports":[31001,31002,31003,31004,31005,31006,31007,31008,31009]}]);
        }
        if kind == Kind::Deployment {
            config["places"][0]["kind"] = serde_json::json!("deployment");
        }
        write(&mut tree, directory.join("qa.json"), serde_json::to_vec(&config).expect("JSON")).expect("configuration");
        let home = Home::open(directory, &mut tree).expect("home");
        let started = home
            .transact(&mut tree, |ledger, _| {
                if kind == Kind::Box {
                    passes::queue(ledger, "work", "web");
                    passes::queue(ledger, "home", "web");
                } else {
                    passes::queue_kinds(ledger);
                }
                passes::build(ledger, OLD, &[1], 1);
                passes::build(ledger, BUILD, if kind == Kind::Box { &[1, 2] } else { &[1] }, 2);
                ledger.mark_build_deleted(OLD, 3)?;
                Ok(ledger.start_pass("places/1", kind, &["ctl".into(), "web".into()], "high", 4).expect("pass"))
            })
            .expect("pass through ledger operations");
        let drive =
            RealDrive { environment: vec![(b"QA_RECIPE_ENV".as_slice().into(), b"captured".as_slice().into())] };
        let place = PassPlace::for_session(
            &home,
            started.session,
            drive,
            Archive { store: directory.join("archive-face"), ..Archive::default() },
            tree.clone(),
            Agents::new(tree.clone()),
        )
        .expect("session place");
        place.paths.ensure(&mut tree).expect("place layout");
        tree.directory(&place.setups, DirectoryMode::Parents).expect("pass setups");
        for (name, hash) in [("welcome", WELCOME), ("project-landing", PROJECT), ("chat-draft", CHAT), ("added", ADDED)]
        {
            write(&mut tree, place.paths.directory.join(format!("hash-{name}")), format!("{hash}\n"))
                .expect("hash answer");
        }
        for (name, parent) in
            [("welcome", None), ("project-landing", Some("welcome")), ("chat-draft", Some("project-landing"))]
        {
            let directory = place.setups.join(name);
            tree.directory(&directory, DirectoryMode::Parents).expect("recipe directory");
            write(
                &mut tree,
                directory.join("recipe"),
                format!(
                    "recipe 1\n{}web see \"Ready\"\n",
                    parent.map_or(String::new(), |parent| format!("after \"{parent}\"\n"))
                ),
            )
            .expect("recipe");
            write(&mut tree, directory.join("README.md"), "A state.\n").expect("readme");
        }
        write(&mut tree, place.paths.directory.join("catalogue"), CATALOGUE).expect("catalogue");
        let mut world = Self { tree, home, session: started.session, place };
        world.script(if kind == Kind::Deployment { Journeys::Deployment } else { Journeys::Box });
        world
    }

    fn script(&mut self, journeys: Journeys) {
        self.place.processes.register(self.place.paths.bin.join("temper-client-journeys"), move |command, tree| {
            journeys.run(command, tree)
        });
    }

    fn copy(&self, name: &str, hash: &str, commit: &str) -> PathBuf {
        let mut tree = self.tree.clone();
        let directory = self.place.paths.copies.join(name).join(format!("{hash}-{commit}.tar"));
        tree.directory(directory.parent().expect("copy parent"), DirectoryMode::Parents).expect("copy directory");
        let snapshot = self.home.directory.join("test-copies").join(name).join(format!("{hash}-{commit}"));
        tree.directory(&snapshot.join("config"), DirectoryMode::Parents).expect("copy config");
        tree.directory(&snapshot.join("data"), DirectoryMode::Parents).expect("copy data");
        write(&mut tree, snapshot.join("data/state"), format!("{name}\n")).expect("copied state");
        write(&mut tree, &directory, snapshot.as_os_str().as_bytes()).expect("archive marker");
        directory
    }

    fn setup(&mut self, name: &str) -> (u8, String) {
        let mut output = Vec::new();
        let code = self.place.setup(name, &mut output).expect("setup command");
        (code, String::from_utf8(output).expect("output"))
    }

    fn walks(&self) -> Vec<String> {
        text(&self.tree, self.place.paths.directory.join("arguments"))
            .unwrap_or_default()
            .lines()
            .filter(|line| line.starts_with("recipe walk "))
            .map(str::to_owned)
            .collect()
    }

    fn fail(&mut self, name: &str, from: &str, code: u8) {
        write(&mut self.tree, self.place.paths.directory.join(format!("walk-{name}-{from}")), code.to_string())
            .expect("walk exit");
    }

    fn setup_file(&self) -> String {
        text(&self.tree, &self.place.paths.setup).expect("setup file")
    }

    fn recipe(&mut self, after: &str) -> (u8, String) {
        let input = RecipeInput::decode(&serde_json::to_vec(&serde_json::json!({"after":after,"lines":["web see \"Added\""],"readme":"An added state.","scripts":{"otherwise":"ok"}})).expect("JSON")).expect("input");
        let mut output = Vec::new();
        let code = self
            .place
            .recipe("added", &input, &["welcome".into(), "project-landing".into(), "chat-draft".into()], &mut output)
            .expect("recipe command");
        (code, String::from_utf8(output).expect("output"))
    }
}

fn ctl_world() -> PlaceWorld {
    let mut world = PlaceWorld::new();
    for (name, recipe) in [
        ("welcome", "recipe 1\ncheck\nctl project list\n"),
        (
            "project-landing",
            "recipe 1\nafter \"welcome\"\nctl project make Hello\ncheck\nctl project list --until made:1\nctl project show Hello\n",
        ),
    ] {
        write(&mut world.tree, world.place.setups.join(name).join("recipe"), recipe).expect("ctl recipe");
        write(&mut world.tree, world.place.paths.directory.join(format!("expected-{name}")), recipe)
            .expect("expected recipe");
    }
    world.script(Journeys::Ctl);
    world.home.configuration.surfaces = vec!["ctl".into()];
    world.place = PassPlace::for_session(
        &world.home,
        world.session,
        world.place.drive,
        world.place.archive,
        world.tree.clone(),
        world.place.processes,
    )
    .expect("ctl place");
    world
}

fn deployment_world() -> PlaceWorld {
    deployment_world_with(Tree::default())
}

fn deployment_world_with(tree: Tree) -> PlaceWorld {
    let mut world = PlaceWorld::with_kind(true, Kind::Deployment, tree);
    world.tree.directory(&world.place.setups.join("welcome/config"), DirectoryMode::Parents).expect("root config");
    world
}

#[test]
fn ctl_only_setup_walks_and_checks_its_copy_without_chromium() {
    let mut world = ctl_world();
    let (code, output) = world.setup("project-landing");
    assert_eq!(code, 0, "{output}");
    assert_eq!(world.place.archive.saved.len(), 2);
    assert!(world.walks()[0].starts_with("recipe walk welcome --bin "));
    assert!(world.walks()[1].starts_with("recipe walk project-landing --from welcome "));
    let (code, output) = world.setup("project-landing");
    assert_eq!(code, 0, "{output}");
    assert_eq!(world.place.archive.saved.len(), 2, "restoring only checks the saved state");
    assert_eq!(world.place.archive.restored.len(), 1);
    assert!(world.walks()[2].starts_with("recipe walk project-landing --from project-landing "));
    assert_eq!(world.setup_file(), format!("project-landing {PROJECT}\n"));
}

#[test]
fn ctl_only_recipe_preserves_its_check_and_proves_and_restores_without_chromium() {
    for after in ["fresh", "project-landing"] {
        let mut world = ctl_world();
        let lines = ["ctl project make Added", "check", "ctl project list --until made:1", "ctl project show Added"];
        let input = RecipeInput::decode(
            &serde_json::to_vec(&serde_json::json!({"after":after,"lines":lines,"readme":"Added is ready."}))
                .expect("JSON"),
        )
        .expect("ctl recipe input");
        let parent = if after == "fresh" { String::new() } else { format!("after \"{after}\"\n") };
        let expected = format!("recipe 1\n{parent}{}\n", lines.join("\n"));
        write(&mut world.tree, world.place.paths.directory.join("expected-added"), &expected)
            .expect("expected ctl recipe");
        let mut output = Vec::new();
        let code = world.place.recipe("added", &input, &[], &mut output).expect("write and prove");
        let output = String::from_utf8(output).expect("output");
        assert_eq!(code, 0, "{output}");
        assert!(output.contains(&format!("recipe added walked from {after}:")), "{output}");
        assert_eq!(text(&world.tree, world.place.setups.join("added/recipe")).expect("written recipe"), expected);
        let walks = world.walks();
        let proving = walks.last().expect("proving walk");
        if after == "fresh" {
            assert!(proving.starts_with("recipe walk added --bin "), "{proving}");
        } else {
            assert!(proving.starts_with("recipe walk added --from project-landing "), "{proving}");
        }
        let catalogue = format!(
            "{CATALOGUE}added{}: Added is ready.\n",
            if after == "fresh" { String::new() } else { format!(" (after {after})") }
        );
        write(&mut world.tree, world.place.paths.directory.join("catalogue"), catalogue).expect("updated catalogue");
        let saved = world.place.archive.saved.len();
        let (code, output) = world.setup("added");
        assert_eq!(code, 0, "{output}");
        assert_eq!(world.place.archive.saved.len(), saved);
        assert!(world.walks().last().expect("copy check").starts_with("recipe walk added --from added "));
        assert_eq!(world.setup_file(), format!("added {ADDED}\n"));
    }
}

#[test]
fn setups_and_proving_walks_receive_only_their_places_ports() {
    for fixed in [false, true] {
        let mut world = PlaceWorld::with_ports(fixed);
        assert_eq!(world.setup("chat-draft").0, 0);
        assert_eq!(world.recipe("welcome").0, 0);
        let walks = world.walks();
        assert_eq!(walks.len(), 5, "three setup walks, parent check and proving walk");
        for walk in walks {
            assert_eq!(walk.contains("--ports 31001,31002,31003,31004,31005,31006,31007,31008,31009"), fixed, "{walk}");
            assert_eq!(walk.contains("--ports"), fixed, "{walk}");
        }
    }
}

#[test]
fn a_matching_copy_is_checked_without_saving_and_uses_the_places_build() {
    let mut world = PlaceWorld::new();
    let copy = world.copy("chat-draft", CHAT, OLD);
    let (code, output) = world.setup("chat-draft");
    assert_eq!(code, 0);
    assert!(output.contains("setup chat-draft: restored from its copy made at aaaaaaaaaaaa"), "{output}");
    assert_eq!(world.setup_file(), format!("chat-draft {CHAT}\n"));
    assert_eq!(world.place.archive.restored, [copy]);
    assert_eq!(world.place.archive.saved.len(), 0);
    assert_eq!(
        world.walks(),
        [format!(
            "recipe walk chat-draft --from chat-draft --bin {} --bundle {} --scratch {} --setups {} --strict",
            world.place.paths.bin.display(),
            world.place.paths.bundle.display(),
            world.place.paths.scratch.display(),
            world.place.setups.display()
        )]
    );
    assert_eq!(text(&world.tree, world.place.paths.walk_log).expect("log"), "walked chat-draft\n");
}

#[test]
fn a_refused_copy_rebuilds_from_the_nearest_ancestor_and_prunes_every_saved_copy() {
    let mut world = PlaceWorld::new();
    world.copy("welcome", WELCOME, OLD);
    let refused = world.copy("chat-draft", CHAT, OLD);
    world.place.archive.removed_before_retry = Some(refused.clone());
    let stale = world.copy("project-landing", WELCOME, OLD);
    world.fail("chat-draft", "chat-draft", 3);
    let (code, output) = world.setup("chat-draft");
    assert_eq!(code, 0);
    assert!(!world.tree.exists(&refused));
    assert!(!world.tree.exists(&stale));
    assert!(output.contains("setup chat-draft: made again from welcome's copy"), "{output}");
    assert!(output.contains("copy project-landing saved: 222222222222 at bbbbbbbbbbbb"), "{output}");
    assert!(output.contains("copy chat-draft saved: 333333333333 at bbbbbbbbbbbb"), "{output}");
    assert!(
        output.find("setup chat-draft:").expect("summary")
            < output.find("copy project-landing saved:").expect("copy line")
    );
    let walks = world.walks();
    assert_eq!(walks.len(), 3);
    assert!(walks[0].starts_with("recipe walk chat-draft --from chat-draft "));
    assert!(walks[1].starts_with("recipe walk project-landing --from welcome "));
    assert!(walks[2].starts_with("recipe walk chat-draft --from project-landing "));
    for (name, hash) in [("project-landing", PROJECT), ("chat-draft", CHAT)] {
        let files = world
            .tree
            .list(&world.place.paths.copies.join(name))
            .expect("saved directory")
            .into_iter()
            .map(|entry| OsString::from_vec(entry.name.into_vec()))
            .collect::<Vec<_>>();
        assert_eq!(files, [std::ffi::OsString::from(format!("{hash}-{BUILD}.tar"))]);
    }
    assert_eq!(world.place.archive.saved.len(), 2);
    assert_eq!(world.setup_file(), format!("chat-draft {CHAT}\n"));
    let log = text(&world.tree, &world.place.paths.walk_log).expect("whole walk log");
    assert!(log.contains("answer 1\n"));
    assert!(log.ends_with("walked chat-draft\n"));
}

#[test]
fn no_copy_empties_the_box_then_walks_and_saves_each_member_root_first() {
    let mut world = PlaceWorld::new();
    for directory in ["config", "data", "cache"] {
        world.tree.directory(&world.place.paths.scratch.join(directory), DirectoryMode::Parents).expect("old box");
    }
    write(&mut world.tree, world.place.paths.scratch.join("cache/kept"), "cache").expect("cache");
    let (code, output) = world.setup("chat-draft");
    assert_eq!(code, 0);
    assert!(output.contains("setup chat-draft: made from a fresh box"), "{output}");
    let walks = world.walks();
    assert_eq!(walks.len(), 3);
    assert!(walks[0].starts_with("recipe walk welcome --bin "));
    assert!(walks[1].starts_with("recipe walk project-landing --from welcome "));
    assert!(walks[2].starts_with("recipe walk chat-draft --from project-landing "));
    assert_eq!(world.place.archive.saved.len(), 3);
    assert!(world.tree.exists(world.place.paths.scratch.join("cache/kept")));
}

#[test]
fn box_and_other_walk_failures_stop_with_only_the_last_forty_lines() {
    for (exit, expected) in [(3, 4), (4, 4), (1, 1), (9, 1)] {
        let mut world = PlaceWorld::new();
        write(&mut world.tree, &world.place.paths.setup, "old setup").expect("prior setup");
        world.fail("welcome", "fresh", exit);
        let (code, output) = world.setup("chat-draft");
        assert_eq!(code, expected);
        if expected == 4 {
            assert!(output.contains("setup chat-draft: the box failed walking welcome"), "{output}");
        }
        assert!(!output.contains("answer 6\n"), "{output}");
        assert!(output.contains("answer 7\n"), "{output}");
        assert_eq!(world.walks().len(), 1);
        assert!(text(&world.tree, &world.place.paths.walk_log).expect("log").starts_with("answer 1\n"));
        assert!(!world.tree.exists(&world.place.paths.setup));
        assert_eq!(world.place.archive.saved.len(), 0);
    }
}

#[test]
fn a_line_stop_reports_the_copy_build_and_only_landings_since_its_deleted_build() {
    let mut world = PlaceWorld::new();
    world.copy("chat-draft", CHAT, OLD);
    world.fail("chat-draft", "chat-draft", 5);
    let (code, output) = world.setup("chat-draft");
    assert_eq!(code, 5);
    assert!(output.contains("setup chat-draft: stopped at chat-draft:7: missing Start"), "{output}");
    assert!(output.contains(&format!("copy started from: chat-draft made at {OLD}")), "{output}");
    assert!(output.contains(&format!("recipe chat-draft last walked on: {OLD}")), "{output}");
    assert!(output.contains("landing 2: goal 12: a landing"), "{output}");
    assert!(!output.contains("landing 1:"), "{output}");
    assert!(!world.tree.exists(&world.place.paths.setup));
    let mut fresh = PlaceWorld::new();
    fresh.fail("welcome", "fresh", 5);
    let (code, output) = fresh.setup("welcome");
    assert_eq!(code, 5);
    assert!(output.contains("recipe welcome last walked on: no copy"));
    assert!(output.contains("landing 1: goal 11: a landing"));
    assert!(output.contains("landing 2: goal 12: a landing"));
}

#[test]
fn fresh_removes_only_config_and_data_and_writes_its_single_word() {
    let mut world = PlaceWorld::new();
    for directory in ["config", "data", "cache"] {
        world.tree.directory(&world.place.paths.scratch.join(directory), DirectoryMode::Parents).expect("box");
    }
    assert_eq!(world.setup("fresh"), (0, "setup fresh: an empty box\n".into()));
    assert!(!world.tree.exists(world.place.paths.scratch.join("config")));
    assert!(!world.tree.exists(world.place.paths.scratch.join("data")));
    assert_eq!(world.tree.kind(world.place.paths.scratch.join("cache")), Some(PathKind::Directory));
    assert_eq!(world.setup_file(), "fresh\n");
    assert_eq!(world.walks().len(), 0);
}

#[test]
fn unknown_unread_and_nonpass_setups_refuse_before_touching_the_box() {
    let mut world = PlaceWorld::new();
    let mut output = Vec::new();
    write(&mut world.tree, &world.place.paths.setup, "fresh\n").expect("prior setup");
    assert_eq!(
        world.place.setup("missing", &mut output).expect_err("unknown").to_string(),
        "setup missing is not in pass 1's setups; setups: fresh, chat-draft, project-landing, welcome"
    );
    write(&mut world.tree, world.place.paths.directory.join("refuse-chat-draft"), "chat-draft:7: no check\n")
        .expect("refusal");
    assert_eq!(
        world.place.setup("chat-draft", &mut output).expect_err("unread").to_string(),
        "recipe chat-draft: chat-draft:7: no check"
    );
    assert_eq!(world.setup_file(), "fresh\n");
    write(
        &mut world.tree,
        world.place.paths.directory.join("catalogue"),
        "welcome: refused: welcome/README.md: unread\n",
    )
    .expect("catalogue refusal");
    assert_eq!(
        world.place.setup("welcome", &mut output).expect_err("catalogue refused").to_string(),
        "recipe welcome: welcome/README.md: unread"
    );
    assert_eq!(world.walks().len(), 0);
    assert!(
        PassPlace::for_session(
            &world.home,
            1,
            RealDrive { environment: vec![] },
            Archive::default(),
            world.tree.clone(),
            Agents::new(world.tree.clone())
        )
        .is_err()
    );
    world
        .home
        .transact(&mut world.tree, |ledger, _| ledger.end_pass_session(world.session, Exit::Bound, None, 9, "qa", &[]))
        .expect("end session");
    assert!(
        PassPlace::for_session(
            &world.home,
            world.session,
            RealDrive { environment: vec![] },
            Archive::default(),
            world.tree.clone(),
            Agents::new(world.tree.clone())
        )
        .is_err()
    );
}

#[test]
fn a_new_recipe_is_written_and_proven_from_its_parent_even_when_its_own_copy_exists() {
    let mut world = PlaceWorld::new();
    world.copy("welcome", WELCOME, OLD);
    world.copy("added", ADDED, OLD);
    let (code, output) = world.recipe("welcome");
    assert_eq!(code, 0);
    assert!(output.contains("recipe added walked from welcome: 444444444444"), "{output}");
    let directory = world.place.setups.join("added");
    assert_eq!(
        text(&world.tree, directory.join("recipe")).expect("recipe"),
        "recipe 1\nafter \"welcome\"\nweb see \"Added\"\n"
    );
    assert_eq!(text(&world.tree, directory.join("README.md")).expect("readme"), "An added state.\n");
    assert_eq!(
        serde_json::from_slice::<serde_json::Value>(
            &world.tree.read(&directory.join("scripts.json")).expect("scripts")
        )
        .expect("JSON"),
        serde_json::json!({"otherwise":"ok"})
    );
    let walks = world.walks();
    assert_eq!(walks.len(), 2);
    assert!(walks[0].starts_with("recipe walk welcome --from welcome "));
    assert!(walks[1].starts_with("recipe walk added --from welcome "));
    assert_eq!(world.setup_file(), format!("added {ADDED}\n"));
    assert_eq!(world.place.archive.saved.len(), 1);
    // The pass's own name may be replaced and a fresh parent needs no --from.
    assert_eq!(world.recipe("fresh").0, 0);
    assert!(world.walks().last().expect("last walk").starts_with("recipe walk added --bin "));
    assert_eq!(text(&world.tree, directory.join("recipe")).expect("recipe"), "recipe 1\nweb see \"Added\"\n");
    let input =
        RecipeInput::decode(br#"{"after":"fresh","lines":["web see \"Ready\""],"readme":"Ready."}"#).expect("input");
    let mut output = Vec::new();
    assert_eq!(
        world.place.recipe("welcome", &input, &["welcome".into()], &mut output).expect_err("build name").to_string(),
        "recipe welcome is among the build's setups"
    );
}

fn commit_input() -> RecipeInput {
    RecipeInput {
        after: "fresh".into(),
        lines: vec![
            r#"forge commit forgejo playground/widget "m" "a.txt" "dir/b.txt""#.into(),
            "forge checks forgejo playground/widget".into(),
            r#"web see "Ready""#.into(),
        ],
        readme: "Committed files.".into(),
        scripts: None,
    }
}

#[test]
fn a_new_recipe_copies_commit_files_before_hash_and_walk() {
    let mut world = PlaceWorld::new();
    let files = world.home.directory.join("passes/1/files");
    let expected = world.place.paths.directory.join("expected-files");
    for root in [&files, &expected] {
        world.tree.directory(&root.join("dir"), DirectoryMode::Parents).expect("nested files");
        write(&mut world.tree, root.join("a.txt"), b"first\n").expect("first file");
        write(&mut world.tree, root.join("dir/b.txt"), b"second\0\xff").expect("second file");
    }
    write(&mut world.tree, files.join("unmentioned"), b"do not copy").expect("unmentioned file");
    let code = world.place.recipe("added", &commit_input(), &[], &mut Vec::new()).expect("copied before hash");
    assert_eq!(code, 0);
    let copied = world.place.setups.join("added/files");
    assert_eq!(world.tree.read(&copied.join("a.txt")).expect("first copied"), b"first\n");
    assert_eq!(world.tree.read(&copied.join("dir/b.txt")).expect("second copied"), b"second\0\xff");
    assert!(!world.tree.exists(copied.join("unmentioned")));
    assert_eq!(world.walks().len(), 1, "proven from fresh");
}

#[test]
fn a_new_recipe_reads_quoted_commit_paths_as_bytes() {
    let mut world = PlaceWorld::new();
    let files = world.home.directory.join("passes/1/files");
    world.tree.directory(&files, DirectoryMode::Parents).expect("pass files");
    let paths = ["a space.txt", "a\"quote.txt", "a\\slash.txt"];
    for path in paths {
        write(&mut world.tree, files.join(path), path).expect("commit file");
    }
    let mut input = commit_input();
    input.lines[0] =
        r#"forge commit forgejo playground/widget "m with \"quotes\"" "a space.txt" "a\"quote.txt" "a\\slash.txt""#
            .into();
    input.lines.insert(1, r#"forge commit forgejo playground/widget "again" "a space.txt""#.into());
    assert_eq!(world.place.recipe("added", &input, &[], &mut Vec::new()).expect("quoted paths"), 0);
    let copied = world.place.setups.join("added/files");
    assert_eq!(world.tree.list(&copied).expect("copied files").len(), paths.len());
    for path in paths {
        assert_eq!(world.tree.read(&copied.join(path)).expect("copied bytes"), path.as_bytes());
    }
}

#[test]
fn commit_paths_cannot_escape_the_pass_or_recipe() {
    for path in ["../escape", "/escape", "dir/../escape", "dir//escape", "./escape"] {
        let mut world = PlaceWorld::new();
        let files = world.home.directory.join("passes/1/files");
        world.tree.directory(&files.join("dir"), DirectoryMode::Parents).expect("pass files");
        write(&mut world.tree, files.join("escape"), b"inside").expect("inside");
        write(&mut world.tree, files.parent().expect("pass directory").join("escape"), b"outside").expect("outside");
        let mut input = commit_input();
        input.lines[0] = format!("forge commit forgejo playground/widget \"m\" \"{path}\"");
        let why = world.place.recipe("added", &input, &[], &mut Vec::new()).expect_err("invalid path");
        assert!(why.to_string().contains("invalid path"), "{why}");
        assert!(!world.tree.exists(world.place.setups.join("added")));
    }
}

#[test]
fn a_missing_commit_file_refuses_by_name_and_removes_the_recipe() {
    let mut world = PlaceWorld::new();
    let files = world.home.directory.join("passes/1/files");
    world.tree.directory(&files, DirectoryMode::Parents).expect("pass files");
    write(&mut world.tree, files.join("a.txt"), b"first").expect("first file");
    let why = world.place.recipe("added", &commit_input(), &[], &mut Vec::new()).expect_err("missing file");
    assert!(why.to_string().contains("dir/b.txt"), "{why}");
    assert!(!world.tree.exists(world.place.setups.join("added")), "no partially written recipe");
    assert_eq!(world.walks().len(), 0);
}

#[test]
fn a_recipe_without_commits_writes_no_files_directory() {
    let mut world = PlaceWorld::new();
    assert_eq!(world.recipe("fresh").0, 0);
    assert!(!world.tree.exists(world.place.setups.join("added/files")));
}

#[test]
fn an_unread_or_failed_new_recipe_is_removed_with_the_walks_exit() {
    let mut unread = PlaceWorld::new();
    write(&mut unread.tree, unread.place.paths.directory.join("refuse-added"), "added:2: no check")
        .expect("hash refusal");
    let input =
        RecipeInput::decode(br#"{"after":"fresh","lines":["ctl project list"],"readme":"A state."}"#).expect("input");
    assert!(
        unread
            .place
            .recipe("added", &input, &[], &mut Vec::new())
            .expect_err("hash refused")
            .to_string()
            .starts_with("recipe added: ")
    );
    assert!(!unread.tree.exists(unread.place.setups.join("added")));
    for (code, expected) in [(3, 4), (4, 4), (5, 5), (1, 1)] {
        let mut world = PlaceWorld::new();
        world.fail("added", "fresh", code);
        let (exit, output) = world.recipe("fresh");
        assert_eq!(exit, expected, "{output}");
        assert!(!world.tree.exists(world.place.setups.join("added")));
        assert!(!world.tree.exists(&world.place.paths.setup));
        assert_eq!(world.place.archive.saved.len(), 0);
    }
}

#[test]
fn recipe_input_and_names_reject_paths_and_multiline_records() {
    for bytes in [
        br#"{"after":"fresh","lines":["web see x\nctl x"],"readme":"A state."}"#.as_slice(),
        br#"{"after":"fresh","lines":[""],"readme":"A state."}"#,
        br#"{"after":"fresh","lines":["web see x"],"readme":" "}"#,
        br#"{"after":"fresh","lines":[],"readme":"A state.","scripts":[]}"#,
    ] {
        assert!(RecipeInput::decode(bytes).is_err());
    }
    let mut world = PlaceWorld::new();
    let input =
        RecipeInput::decode(br#"{"after":"fresh","lines":["web see \"Ready\""],"readme":"Ready."}"#).expect("input");
    for name in ["../escape", "fresh", "A", "1bad", "", &"a".repeat(65)] {
        assert!(
            world
                .place
                .recipe(name, &input, &[], &mut Vec::new())
                .expect_err("name refused")
                .to_string()
                .contains("invalid recipe name")
        );
    }
    let paths = PlacePaths::new(&world.place.paths.directory, Kind::Box);
    assert_eq!(paths.scratch, world.place.paths.directory.join("box"));
}

const PINNED_AGENT: &str = "data/box/fake-backends/data/workspaces/agent";
const CHECKOUT_AGENT: &str = "data/box/fake-backends/data/workspaces/1/repo/agent/kept";

fn write_copy_state(tree: &mut Tree, scratch: &Path, pinned: bool) {
    let kept = scratch.join(CHECKOUT_AGENT);
    tree.directory(kept.parent().expect("checkout agent parent"), DirectoryMode::Parents)
        .expect("checkout agent directory");
    write(tree, kept, "checkout state").expect("checkout agent state");
    write(tree, scratch.join("data/state"), "setup state").expect("setup state");
    if pinned {
        let agent = scratch.join(PINNED_AGENT);
        tree.directory(&agent, DirectoryMode::Parents).expect("pinned agent directory");
        write(tree, agent.join("temper-box"), "pinned binary").expect("pinned binary");
        write(tree, agent.join("agent.json"), "{}").expect("pinned document");
    }
}

fn assert_copy_state(tree: &mut Tree, scratch: &Path) {
    assert!(!tree.exists(scratch.join(PINNED_AGENT)), "the pinned agent directory is left out");
    assert_eq!(text(tree, scratch.join("data/state")).expect("setup state"), "setup state");
    assert_eq!(text(tree, scratch.join(CHECKOUT_AGENT)).expect("checkout state"), "checkout state");
}

fn assert_pinned_agent(tree: &mut Tree, scratch: &Path) {
    let agent = scratch.join(PINNED_AGENT);
    assert_eq!(tree.read(&agent.join("temper-box")).expect("pinned binary"), b"pinned binary");
    assert_eq!(tree.read(&agent.join("agent.json")).expect("pinned document"), b"{}");
}

#[test]
fn a_place_saves_state_without_the_walks_pinned_agent() {
    let mut world = PlaceWorld::new();
    let (code, output) = world.setup("welcome");
    assert_eq!(code, 0, "{output}");
    assert!(
        world.tree.exists(world.place.paths.scratch.join(PINNED_AGENT).join("temper-box")),
        "the walk pinned its agent"
    );
    assert_eq!(world.place.archive.saved.len(), 1);
    let saved = world.place.paths.copies.join("welcome").join(format!("{WELCOME}-{BUILD}.tar"));
    let snapshot = text(&world.tree, saved).expect("archive marker");
    assert!(!world.tree.exists(Path::new(&snapshot).join(PINNED_AGENT)), "save leaves out the pinned directory");
    assert_eq!(text(&world.tree, Path::new(&snapshot).join("data/state")).expect("saved state"), "welcome\n");
}

#[test]
fn a_place_restores_an_older_copy_without_its_pinned_agent() {
    let mut world = PlaceWorld::new();
    let archive = world.copy("welcome", WELCOME, OLD);
    let snapshot = text(&world.tree, &archive).expect("archive marker");
    write_copy_state(&mut world.tree, Path::new(&snapshot), true);
    // Inspect the restored state without simulating another worker start in the check.
    world.script(Journeys::CheckCopy);
    let (code, output) = world.setup("welcome");
    assert_eq!(code, 0, "{output}");
    assert_eq!(world.place.archive.restored, [archive]);
    assert_copy_state(&mut world.tree, &world.place.paths.scratch);
    assert!(world.tree.exists(Path::new(&snapshot).join(PINNED_AGENT)), "the existing copy is not rewritten");
}

#[test]
fn a_copy_takes_the_members_its_box_made() {
    // A box with fake backends keeps its configuration under data/, so its config/ may never be made.
    let mut tree = Tree::default();
    let box_directory = Path::new("/box");
    tree.directory(box_directory, DirectoryMode::One).expect("box");
    assert!(copy_members(box_directory, &mut tree).is_err(), "an empty box directory has nothing to copy");
    tree.directory(&box_directory.join("data"), DirectoryMode::One).expect("data");
    assert_eq!(copy_members(box_directory, &mut tree).expect("members"), ["data"]);
    tree.directory(&box_directory.join("config"), DirectoryMode::One).expect("config");
    assert_eq!(copy_members(box_directory, &mut tree).expect("members"), ["config", "data"]);
}

#[test]
fn ctl_only_pass_refuses_web_and_tui_in_a_setups_parent_before_restore() {
    for surface in ["web", "tui"] {
        let mut world = PlaceWorld::new();
        world.home.configuration.surfaces = vec!["ctl".into()];
        world.place = PassPlace::for_session(
            &world.home,
            world.session,
            RealDrive {
                environment: vec![(Box::from(b"QA_RECIPE_ENV".as_slice()), Box::from(b"captured".as_slice()))],
            },
            Archive::default(),
            world.tree.clone(),
            world.place.processes,
        )
        .expect("ctl pass place");
        write(&mut world.tree, &world.place.paths.setup, "fresh\n").expect("prior setup");
        write(
            &mut world.tree,
            world.place.setups.join("chat-draft/recipe"),
            "recipe 1\nafter project-landing\ncheck\nctl task list\n",
        )
        .expect("ctl child");
        write(
            &mut world.tree,
            world.place.setups.join("project-landing/recipe"),
            format!("recipe 1\n{surface} see Ready\n"),
        )
        .expect("parent");
        let why = world.place.setup("chat-draft", &mut Vec::new()).expect_err("surface refused").to_string();
        assert!(why.contains(&format!("recipe project-landing:2 has a {surface} line")), "{why}");
        assert!(why.contains("the pass's surfaces take ctl and forge lines alone"), "{why}");
        assert_eq!(world.setup_file(), "fresh\n");
        assert_eq!(world.walks().len(), 0);
        assert_eq!(world.place.archive.restored.len(), 0);
        assert_eq!(world.setup("fresh").0, 0);
    }
}

#[test]
fn a_new_recipe_copies_push_files_and_keeps_the_push_line() {
    let mut world = PlaceWorld::new();
    let files = world.home.directory.join("passes/1/files");
    world.tree.directory(&files, DirectoryMode::Parents).expect("files");
    write(&mut world.tree, files.join("a.txt"), b"pushed").expect("file");
    let mut input = commit_input();
    input.lines[0] = r#"forge push forgejo playground/widget work/topic "m" "a.txt""#.into();
    assert_eq!(world.place.recipe("added", &input, &[], &mut Vec::new()).expect("recipe"), 0);
    assert_eq!(world.tree.read(&world.place.setups.join("added/files/a.txt")).expect("kept file"), b"pushed");
    assert!(text(&world.tree, world.place.setups.join("added/recipe")).expect("recipe").contains(&input.lines[0]));
}

#[test]
fn setups_refuse_fresh_and_chains_of_the_other_places_kind_before_walking() {
    let mut deployment = deployment_world();
    let before = deployment.walks();
    assert_eq!(
        deployment.place.setup("fresh", &mut Vec::new()).expect_err("no fresh").to_string(),
        "a deployment's place has no fresh setup: its setups start from their root's config/"
    );
    deployment.tree.remove(&deployment.place.setups.join("welcome/config"), RemoveMode::Tree).expect("box root");
    assert_eq!(
        deployment.place.setup("chat-draft", &mut Vec::new()).expect_err("box chain").to_string(),
        "setup chat-draft is a box's; places/1 is a deployment's place"
    );
    assert_eq!(deployment.walks(), before);
    let mut boxed = PlaceWorld::new();
    boxed.tree.directory(&boxed.place.setups.join("welcome/config"), DirectoryMode::One).expect("deployment root");
    assert_eq!(
        boxed.place.setup("chat-draft", &mut Vec::new()).expect_err("deployment chain").to_string(),
        "setup chat-draft is a deployment's; places/1 is a box's place"
    );
    assert_eq!(boxed.walks().len(), 0);
}

#[test]
fn deployment_recipes_copy_patch_files_and_inherit_the_parents_kind_without_config() {
    let mut world = deployment_world();
    let files = world.home.directory.join("passes/1/files");
    world.tree.directory(&files.join("patches"), DirectoryMode::Parents).expect("patch directory");
    write(&mut world.tree, files.join("patches/engine.toml"), b"public = \"http://127.0.0.1:7100\"\n").expect("patch");
    let input = RecipeInput::decode(br#"{"after":"welcome","lines":["deployment patch engine \"patches/engine.toml\"","check","ctl task list"],"readme":"A patched deployment."}"#).expect("recipe input");
    assert_eq!(world.place.recipe("added", &input, &[], &mut Vec::new()).expect("recipe"), 0);
    let added = world.place.setups.join("added");
    assert_eq!(
        world.tree.read(&added.join("files/patches/engine.toml")).expect("recipe patch"),
        b"public = \"http://127.0.0.1:7100\"\n"
    );
    assert!(!world.tree.exists(added.join("config")));
    assert!(
        text(&world.tree, added.join("recipe"))
            .expect("recipe")
            .starts_with("recipe 1\nafter \"welcome\"\ndeployment patch engine")
    );
    write(
        &mut world.tree,
        world.place.paths.directory.join("catalogue"),
        format!("{CATALOGUE}added (after welcome): Patched.\n"),
    )
    .expect("catalogue");
    assert_eq!(world.setup("added").0, 0, "the new recipe is a deployment's through its root");
    let missing = RecipeInput { lines: vec!["deployment patch worker missing.toml".into()], ..input };
    let why = world.place.recipe("added", &missing, &[], &mut Vec::new()).expect_err("missing patch").to_string();
    assert!(why.contains("missing.toml"), "{why}");
    assert!(!world.tree.exists(&added), "a refused recipe leaves nothing");
}

/// The two tar handlers consume the same portable command. Neither applies exclusions.
enum Tar<'a> {
    Create(&'a [OsString]),
    Extract,
}

fn tar_command<'a>(command: &OutputCommand<'a>) -> (&'a Path, &'a Path, Tar<'a>) {
    assert_eq!(command.program, "tar");
    assert!(command.directory.is_none());
    assert!(command.log.is_none());
    let [operation, file_option, archive, directory_option, source, members @ ..] = command.arguments else {
        panic!("tar needs an archive and a directory: {:?}", command.arguments);
    };
    assert_eq!(file_option, "--file");
    assert_eq!(directory_option, "--directory");
    let operation = match operation.to_str() {
        Some("--create") if !members.is_empty() => Tar::Create(members),
        Some("--extract") if members.is_empty() => Tar::Extract,
        _ => panic!("unexpected tar arguments: {:?}", command.arguments),
    };
    (Path::new(archive), Path::new(source), operation)
}

fn scripted_archive(tree: &Tree, directory: &Path, code: u8, blocks_move_back: bool) -> (RealArchive, Agents<Tree>) {
    let mut processes = Agents::new(tree.clone());
    let environment = vec![
        (b"PATH".as_slice().into(), directory.as_os_str().as_bytes().into()),
        (b"QA_TAR_PINNED".as_slice().into(), PINNED_AGENT.as_bytes().into()),
        (b"QA_TAR_EXIT".as_slice().into(), code.to_string().into_bytes().into()),
    ];
    let expected_environment = environment.clone();
    processes.register("tar", move |command, tree| {
        assert_eq!(command.environment, expected_environment);
        let (archive, source, _) = tar_command(command);
        let mut arguments = Vec::new();
        for argument in command.arguments {
            arguments.extend_from_slice(argument.as_os_str().as_bytes());
            arguments.push(b'\n');
        }
        write(tree, archive.with_extension("tar.arguments"), arguments)?;
        let pinned = source.join(PINNED_AGENT);
        write(tree, archive.with_extension("tar.pinned"), if tree.exists(&pinned) { "present\n" } else { "absent\n" })?;
        if blocks_move_back {
            tree.directory(&pinned.join("occupied"), DirectoryMode::Parents)?;
        }
        Ok(answer(code, [], b"fake tar stderr\n"))
    });
    (RealArchive { environment }, processes)
}

#[test]
fn archive_commands_use_only_portable_options_and_hide_the_pinned_path() {
    for config in [true, false] {
        let mut tree = Tree::default();
        let source = Path::new("/source");
        let archive = Path::new("/copy.tar");
        write_copy_state(&mut tree, source, true);
        if config {
            tree.directory(&source.join("config"), DirectoryMode::One).expect("configuration");
        }
        let (mut face, mut programs) = scripted_archive(&tree, Path::new("/"), 0, false);
        face.create(source, archive, &[PINNED_AGENT], &mut tree, &mut programs).expect("create");
        let members = if config { "config\ndata\n" } else { "data\n" };
        assert_eq!(
            text(&tree, archive.with_extension("tar.arguments")).expect("create arguments"),
            format!("--create\n--file\n{}\n--directory\n{}\n{members}", archive.display(), source.display())
        );
        assert_eq!(text(&tree, archive.with_extension("tar.pinned")).expect("pinned while creating"), "absent\n");
        assert_pinned_agent(&mut tree, source);
        assert!(!tree.exists(archive.with_extension("tar.left-out")), "nothing remains set aside");
        face.extract(archive, source, &[PINNED_AGENT], &mut tree, &mut programs).expect("extract");
        assert_eq!(
            text(&tree, archive.with_extension("tar.arguments")).expect("extract arguments"),
            format!("--extract\n--file\n{}\n--directory\n{}\n", archive.display(), source.display())
        );
        assert_eq!(text(&tree, archive.with_extension("tar.pinned")).expect("pinned while extracting"), "present\n");
        assert_copy_state(&mut tree, source);
    }
}

#[test]
fn a_failed_archive_save_restores_the_pinned_bytes_and_removes_the_set_aside_directory() {
    let mut tree = Tree::default();
    let source = Path::new("/source");
    let archive = Path::new("/copy.tar");
    write_copy_state(&mut tree, source, true);
    let (mut face, mut programs) = scripted_archive(&tree, Path::new("/"), 2, false);
    let why = face.create(source, archive, &[PINNED_AGENT], &mut tree, &mut programs).expect_err("tar failed");
    assert_eq!(why.to_string(), "tar --create: exit status: 2: fake tar stderr");
    assert_eq!(text(&tree, archive.with_extension("tar.pinned")).expect("pinned while creating"), "absent\n");
    assert_pinned_agent(&mut tree, source);
    assert!(!tree.exists(archive.with_extension("tar.left-out")), "nothing remains set aside after failure");
}

#[test]
fn a_failed_archive_save_reports_tar_then_move_back_and_keeps_private_set_aside_bytes() {
    let mut tree = Tree::default();
    let source = Path::new("/source");
    let archive = Path::new("/copy.tar");
    write_copy_state(&mut tree, source, true);
    let (mut face, mut programs) = scripted_archive(&tree, Path::new("/"), 2, true);
    let why = face
        .create(source, archive, &[PINNED_AGENT], &mut tree, &mut programs)
        .expect_err("tar and move back failed")
        .to_string();
    assert!(
        why.starts_with(&format!(
            "tar --create: exit status: 2: fake tar stderr; {}: ",
            source.join(PINNED_AGENT).display()
        )),
        "{why}"
    );
    let aside = archive.with_extension("tar.left-out");
    assert_mode(&tree, &aside, 0o700);
    assert_eq!(tree.read(&aside.join("0/temper-box")).expect("saved binary"), b"pinned binary");
    assert_eq!(tree.read(&aside.join("0/agent.json")).expect("saved document"), b"{}");
}

#[test]
fn an_interrupted_saves_set_aside_is_removed_before_moving_the_pinned_path() {
    let mut tree = Tree::default();
    let source = Path::new("/source");
    let archive = Path::new("/copy.tar");
    let aside = archive.with_extension("tar.left-out");
    tree.directory(&aside, DirectoryMode::One).expect("interrupted save");
    write(&mut tree, aside.join("0"), "interrupted pinned file").expect("stale entry");
    write_copy_state(&mut tree, source, true);
    let (mut face, mut programs) = scripted_archive(&tree, Path::new("/"), 0, false);
    face.create(source, archive, &[PINNED_AGENT], &mut tree, &mut programs).expect("next save");
    assert_pinned_agent(&mut tree, source);
    assert!(!tree.exists(&aside), "the interrupted save and this save leave nothing set aside");
}

#[test]
fn archive_restore_removes_a_pinned_symlink_without_following_it() {
    for dangling in [false, true] {
        let mut tree = Tree::default();
        let source = Path::new("/source");
        let archive = Path::new("/copy.tar");
        let target = Path::new("/target");
        if !dangling {
            tree.directory(target, DirectoryMode::One).expect("symlink target");
            write(&mut tree, target.join("kept"), b"outside the scratch directory").expect("target state");
        }
        write_copy_state(&mut tree, source, false);
        tree.symlink(target, source.join(PINNED_AGENT)).expect("pinned symlink");
        let (mut face, mut programs) = scripted_archive(&tree, Path::new("/"), 0, false);
        face.extract(archive, source, &[PINNED_AGENT], &mut tree, &mut programs).expect("extract");
        assert!(tree.facts(&source.join(PINNED_AGENT), false).is_err(), "even a dangling symlink is removed");
        assert_copy_state(&mut tree, source);
        if !dangling {
            assert_eq!(tree.read(&target.join("kept")).expect("symlink target kept"), b"outside the scratch directory");
        }
    }
}

#[test]
fn a_failed_archive_restore_leaves_the_pinned_path_untouched() {
    let mut tree = Tree::default();
    let source = Path::new("/source");
    let archive = Path::new("/copy.tar");
    write_copy_state(&mut tree, source, true);
    let (mut face, mut programs) = scripted_archive(&tree, Path::new("/"), 2, false);
    let why = face.extract(archive, source, &[PINNED_AGENT], &mut tree, &mut programs).expect_err("tar failed");
    assert_eq!(why.to_string(), "tar --extract: exit status: 2: fake tar stderr");
    assert_pinned_agent(&mut tree, source);
}

#[test]
fn archive_restore_refuses_an_unremovable_path_by_name() {
    let mut tree = Tree::default();
    let source = Path::new("/source");
    let archive = Path::new("/copy.tar");
    write_copy_state(&mut tree, source, true);
    let path = source.join("data/state/agent");
    let expected = tree.facts(&path, false).expect_err("state is a file");
    let (mut face, mut programs) = scripted_archive(&tree, Path::new("/"), 0, false);
    let why =
        face.extract(archive, source, &["data/state/agent"], &mut tree, &mut programs).expect_err("cannot remove");
    assert_eq!(why.to_string(), format!("{}: {expected}", path.display()));
}

fn copying_tar(processes: &mut Agents<Tree>, store: PathBuf) -> RealArchive {
    let mut next = 0;
    processes.register("tar", move |command, tree| {
        let (archive, scratch, operation) = tar_command(command);
        match operation {
            Tar::Create(members) => {
                next += 1;
                let snapshot = store.join(next.to_string());
                for member in members {
                    copy(tree, scratch, &snapshot, Path::new(member), &[])?;
                }
                // The marker follows the archive when QA renames it from copy.tmp.
                write(tree, archive, snapshot.as_os_str().as_bytes())?;
            }
            Tar::Extract => {
                let snapshot = archive_marker(tree, archive);
                copy(tree, &snapshot, scratch, Path::new(""), &[])?;
            }
        }
        Ok(answer(0, [], []))
    });
    RealArchive { environment: vec![] }
}

const DEPLOYMENT_PINNED_AGENT: &str = "data/worker/workspaces/agent";
const DEPLOYMENT_WORKSPACE: &str = "data/worker/workspaces/w1/file";
const DEPLOYMENT_OTHER_AGENT: &str = "data/other/data/worker/workspaces/agent/kept";

fn deployment_copy_state(tree: &mut Tree, scratch: &Path, pinned: bool) {
    for path in [DEPLOYMENT_WORKSPACE, DEPLOYMENT_OTHER_AGENT] {
        let file = scratch.join(path);
        tree.directory(file.parent().expect("state directory"), DirectoryMode::Parents).expect("state directory");
        write(tree, file, "kept state").expect("state");
    }
    if pinned {
        let agent = scratch.join(DEPLOYMENT_PINNED_AGENT);
        tree.directory(&agent, DirectoryMode::Parents).expect("agent directory");
        write(tree, agent.join("temper-agent"), "pinned binary").expect("pinned binary");
    }
}

fn assert_deployment_copy_state(tree: &Tree, scratch: &Path) {
    assert!(!tree.exists(scratch.join(DEPLOYMENT_PINNED_AGENT)), "the deployment's pinned agent is left out");
    for path in [DEPLOYMENT_WORKSPACE, DEPLOYMENT_OTHER_AGENT] {
        assert_eq!(text(tree, scratch.join(path)).expect("kept state"), "kept state");
    }
}

#[test]
fn deployment_setup_saves_real_copies_without_its_pinned_agent_and_allows_its_absence() {
    for pinned in [true, false] {
        let mut world = deployment_world();
        let paths = world.place.paths.clone();
        deployment_copy_state(&mut world.tree, &paths.directory.join("seed"), pinned);
        world.script(Journeys::DeploymentSeed);
        let archive = copying_tar(&mut world.place.processes, world.home.directory.join("tar-copies"));
        let mut place = PassPlace::for_session(
            &world.home,
            world.session,
            world.place.drive,
            archive,
            world.tree.clone(),
            world.place.processes,
        )
        .expect("real archive face");
        assert_eq!(place.setup("welcome", &mut Vec::new()).expect("save setup"), 0);
        assert_eq!(world.tree.exists(paths.scratch.join(DEPLOYMENT_PINNED_AGENT)), pinned, "the walk's state");
        let restored = paths.directory.join("inspect");
        world.tree.directory(&restored, DirectoryMode::One).expect("inspection directory");
        // Inspect the saved tar without extract's exclusion masking a faulty save.
        place
            .archive
            .extract(
                &paths.copies.join("welcome").join(format!("{WELCOME}-{BUILD}.tar")),
                &restored,
                &[],
                &mut world.tree,
                &mut place.processes,
            )
            .expect("inspect copy");
        assert_deployment_copy_state(&world.tree, &restored);
        assert_eq!(text(&world.tree, restored.join("config/receipt")).expect("configuration"), "saved config");
    }
}

#[test]
fn deployment_setup_restores_an_older_real_copy_without_its_pinned_agent() {
    let mut world = deployment_world();
    let paths = world.place.paths.clone();
    let source = paths.directory.join("older");
    deployment_copy_state(&mut world.tree, &source, true);
    world.tree.directory(&source.join("config"), DirectoryMode::Parents).expect("older config");
    write(&mut world.tree, source.join("config/receipt"), "saved config").expect("older config");
    let directory = paths.copies.join("welcome");
    world.tree.directory(&directory, DirectoryMode::Parents).expect("copy directory");
    let archive = directory.join(format!("{WELCOME}-{OLD}.tar"));
    let mut face = copying_tar(&mut world.place.processes, world.home.directory.join("tar-copies"));
    face.create(&source, &archive, &[], &mut world.tree, &mut world.place.processes)
        .expect("older copy including pinned agent");
    let mut place = PassPlace::for_session(
        &world.home,
        world.session,
        world.place.drive,
        face,
        world.tree.clone(),
        world.place.processes,
    )
    .expect("real archive face");
    let mut output = Vec::new();
    assert_eq!(place.setup("welcome", &mut output).expect("restore older copy"), 0);
    assert!(String::from_utf8(output).expect("output").contains("restored from its copy made at aaaaaaaaaaaa"));
    assert_deployment_copy_state(&world.tree, &paths.scratch);
    assert!(world.tree.exists(archive), "checking a copy keeps it");
}

#[test]
fn setup_tightens_existing_place_directories_and_files() {
    let mut world = PlaceWorld::new();
    let paths = &world.place.paths;
    for path in [&paths.directory, &paths.scratch, &paths.copies] {
        world.tree.permissions(path, 0o755).expect("old directory mode");
    }
    for path in [&paths.walk_log, &paths.directory.join("setup.tmp")] {
        write(&mut world.tree, path, "old").expect("old file");
        world.tree.permissions(path, 0o666).expect("old file mode");
    }
    assert_eq!(world.setup("welcome").0, 0);
    let paths = &world.place.paths;
    for path in [&paths.directory, &paths.scratch, &paths.copies] {
        assert_mode(&world.tree, path, 0o700);
    }
    for path in [&paths.setup, &paths.walk_log] {
        assert_mode(&world.tree, path, 0o600);
    }
}

fn assert_install_modes(tree: &mut Tree, source: &Path, installed: &Path) {
    assert_mode(tree, installed, 0o700);
    for entry in tree.list(source).expect("build directory") {
        let name = OsStr::from_bytes(&entry.name);
        let path = source.join(name);
        let target = installed.join(name);
        let metadata = tree.facts(&path, true).expect("build mode");
        if metadata.kind == PathKind::Directory {
            assert_install_modes(tree, &path, &target);
        } else {
            assert_mode(tree, &target, metadata.mode);
        }
    }
}

#[test]
fn installation_privatizes_directories_but_keeps_build_file_modes() {
    use sha2::{Digest, Sha256};
    use std::io::Write as _;
    use temper_qa_tool::testing::{Agent, World, baseline, land};

    let world = World::new();
    let mut tree = world.files();
    tree.default_modes(0o644, 0o755); // Files and directories made under umask 022.
    let mut run = world.start(baseline(), &[Agent::Case, Agent::Never]);
    run.tick();
    land(&mut run, 2, 7);
    let build = world.installable_build(2);
    for (path, mode) in [("bin/nested/deep/tool", 0o711), ("bundle/assets/deep/note", 0o640)] {
        let full = build.join(path);
        tree.directory(full.parent().expect("parent"), DirectoryMode::Parents).expect("build directories");
        write(&mut tree, &full, path).expect("build file");
        tree.permissions(&full, mode).expect("build mode");
        let options = WriteOptions { way: WriteMode::Append, ..WriteOptions::REPLACE };
        writeln!(
            tree.open_write(&build.join("manifest"), options).expect("manifest"),
            "{:x}  {path}",
            Sha256::digest(path.as_bytes())
        )
        .expect("manifest row");
    }
    let place = world.home.directory.join("places/1");
    for path in [&place, &place.join("box"), &place.join("copies")] {
        tree.directory(path, DirectoryMode::Parents).expect("existing place");
        tree.permissions(path, 0o755).expect("old directory mode");
    }
    run.tick();
    run.tick();
    assert_eq!(world.ledger().passes.len(), 1, "installed pass started");
    for name in ["bin", "bundle"] {
        assert_install_modes(&mut tree, &build.join(name), &place.join(name));
    }
    for path in [&place, &place.join("box"), &place.join("copies")] {
        assert_mode(&tree, path, 0o700);
    }
    assert_mode(&tree, &place.join("drive.lock"), 0o600);
}

#[test]
fn another_holder_of_the_drive_lock_refuses_setup_and_recipe_and_probe_releases() {
    let mut world = PlaceWorld::new();
    assert!(!place_busy(&world.place.paths, &mut world.tree).expect("free"));
    let holder = world.tree.hold_lock(&world.place.paths.drive_lock).expect("holder takes lock");
    assert!(place_busy(&world.place.paths, &mut world.tree).expect("busy"));
    let mut output = Vec::new();
    let expected = "the drive session is open at places/1; close it first: temper-qa pass close";
    assert_eq!(world.place.setup("fresh", &mut output).expect_err("busy").to_string(), expected);
    let input =
        RecipeInput::decode(br#"{"after":"fresh","lines":["web see \"Ready\""],"readme":"Ready."}"#).expect("recipe");
    assert_eq!(world.place.recipe("added", &input, &[], &mut output).expect_err("busy").to_string(), expected);
    drop(holder);
    assert!(!place_busy(&world.place.paths, &mut world.tree).expect("free again"));
    let _lock = world
        .tree
        .lock(&world.place.paths.drive_lock, LockMode::Try, false)
        .expect("lock")
        .expect("probe did not keep it");
    assert_eq!(output.len(), 0);
}

#[test]
fn a_new_recipe_reads_a_non_utf8_commit_path_as_its_bytes() {
    let mut world = PlaceWorld::new();
    let files = world.home.directory.join("passes/1/files");
    world.tree.directory(&files, DirectoryMode::Parents).expect("pass files");
    let path = OsString::from_vec(b"a\xff.txt".to_vec());
    write(&mut world.tree, files.join(&path), b"non-UTF-8 path").expect("commit file");
    let mut input = commit_input();
    input.lines[0] = r#"forge commit forgejo playground/widget "m" "a\xff.txt""#.into();
    assert_eq!(world.place.recipe("added", &input, &[], &mut Vec::new()).expect("non-UTF-8 path"), 0);
    let copied = world.place.setups.join("added/files");
    assert_eq!(world.tree.list(&copied).expect("copied files").len(), 1);
    assert_eq!(world.tree.read(&copied.join(path)).expect("copied bytes"), b"non-UTF-8 path");
}

#[test]
fn deployment_walks_save_and_restore_their_private_scratch_with_bundle() {
    // Creation modes under umask 022 and 000, respectively.
    for (file, directory) in [(0o644, 0o755), (0o666, 0o777)] {
        let tree = Tree::default();
        tree.default_modes(file, directory);
        let mut world = deployment_world_with(tree);
        let paths = world.place.paths.clone();
        assert_eq!(paths.scratch, paths.directory.join("deployment"));
        assert!(!world.tree.exists(paths.directory.join("box")));
        assert_mode(&world.tree, &paths.scratch, 0o700);
        let (code, output) = world.setup("project-landing");
        assert_eq!(code, 0, "{output}");
        assert_eq!(world.place.archive.saved.len(), 2);
        assert_eq!(
            world.walks()[0],
            format!(
                "recipe walk welcome --bin {} --bundle {} --scratch {} --setups {} --strict --ports 31001,31002,31003,31004,31005,31006,31007,31008,31009",
                paths.bin.display(),
                paths.bundle.display(),
                paths.scratch.display(),
                world.place.setups.display()
            )
        );
        // Save the next state with both data and the configuration receipt.
        assert_eq!(world.setup("chat-draft").0, 0);
        write(&mut world.tree, paths.scratch.join("data/state"), "changed").expect("changed state");
        write(&mut world.tree, paths.scratch.join("config/receipt"), "changed").expect("changed config");
        assert_eq!(world.setup("chat-draft").0, 0);
        assert_eq!(text(&world.tree, paths.scratch.join("config/receipt")).expect("restored config"), "saved config");
        assert_eq!(text(&world.tree, paths.scratch.join("data/state")).expect("checked state"), "chat-draft\n");
        assert_eq!(world.place.archive.saved.len(), 3, "checking a copy does not replace it");
    }
}
