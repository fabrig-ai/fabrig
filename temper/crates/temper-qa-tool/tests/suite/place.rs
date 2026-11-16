//! Places use a scripted build and directory copies, never a box or browser (qa/places.md, 8).
use skein_scratch::Scratch;
use std::{
    fs,
    io::{BufRead, BufReader},
    os::unix::{ffi::OsStringExt, fs::PermissionsExt},
    path::{Path, PathBuf},
    process::{Command, Stdio},
};
use temper_qa_tool::{
    Home, Refusal,
    faces::{ArchiveFace, RealArchive, RealDrive, copy_members},
    place::{PassPlace, PlacePaths, RecipeInput, place_busy},
};

const OLD: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const BUILD: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
const WELCOME: &str = "1111111111111111111111111111111111111111111111111111111111111111";
const PROJECT: &str = "2222222222222222222222222222222222222222222222222222222222222222";
const CHAT: &str = "3333333333333333333333333333333333333333333333333333333333333333";
const ADDED: &str = "4444444444444444444444444444444444444444444444444444444444444444";
const CATALOGUE: &str =
    "chat-draft (after project-landing): A draft.\nproject-landing (after welcome): A project.\nwelcome: Welcome!\n";
const TOOL: &str = r#"#!/bin/sh
set -eu
test "$QA_RECIPE_ENV" = captured
printf '%s\n' "$*" >> arguments
case "$2" in
list) cat catalogue ;;
hash)
    if test -f "refuse-$3"; then cat "refuse-$3" >&2; exit 1; fi
    if test -d expected-files && test "$3" = added; then
        diff -r expected-files "$5/added/files" >&2
    fi
    cat "hash-$3" ;;
walk)
    name=$3; shift 3
    from=fresh
    while test "$#" -gt 0; do
        case "$1" in
        --from) from=$2; shift 2 ;;
        --bin) test "$2" = "$PWD/bin"; shift 2 ;;
        --bundle) test "$2" = "$PWD/bundle"; shift 2 ;;
        --scratch) test "$2" = "$PWD/box"; shift 2 ;;
        --setups)
            test -d "$2"
            if test -d expected-files && test "$name" = added; then
                diff -r expected-files "$2/added/files"
            fi
            shift 2 ;;
        --strict) shift ;;
        --ports) shift 2 ;;
        *) exit 1 ;;
        esac
    done
    if test "$from" = fresh; then
        test ! -e box/config; test ! -e box/data
    else
        test -d box/config; test -d box/data
    fi
    if test -f "walk-$name-$from"; then
        n=1
        while test "$n" -le 45; do printf 'answer %s\n' "$n"; n=$((n + 1)); done
        printf 'stopped at %s:7: missing Start\n' "$name"
        exit "$(cat "walk-$name-$from")"
    fi
    mkdir -p box/config box/data
    printf '%s\n' "$name" > box/data/state
    mkdir -p box/data/box/fake-backends/data/workspaces/agent
    printf 'pinned\n' > box/data/box/fake-backends/data/workspaces/agent/temper-box
    printf 'walked %s\n' "$name"
    ;;
*) exit 1 ;;
esac
"#;

#[derive(Default)]
struct Archive {
    saved: Vec<PathBuf>,
    restored: Vec<PathBuf>,
    removed_before_retry: Option<PathBuf>,
    store: PathBuf,
}
impl ArchiveFace for Archive {
    fn create(&mut self, scratch: &Path, archive: &Path, left_out: &[&str]) -> Result<(), Refusal> {
        assert_mode(archive, 0o600);
        self.saved.push(archive.to_owned());
        let snapshot = self.store.join(self.saved.len().to_string());
        for member in ["config", "data"] {
            copy(scratch, &snapshot, Path::new(member), left_out);
        }
        fs::write(archive, snapshot.to_str().expect("snapshot path")).expect("archive marker");
        Ok(())
    }
    fn extract(&mut self, archive: &Path, scratch: &Path, left_out: &[&str]) -> Result<(), Refusal> {
        if let Some(refused) = &self.removed_before_retry
            && archive != refused
        {
            assert!(!refused.exists(), "refused copy must be removed before trying an ancestor");
        }
        self.restored.push(archive.to_owned());
        let snapshot = fs::read_to_string(archive).expect("archive marker");
        copy(Path::new(&snapshot), scratch, Path::new(""), left_out);
        Ok(())
    }
}
fn copy(source: &Path, destination: &Path, relative: &Path, left_out: &[&str]) {
    if left_out.iter().any(|path| relative == Path::new(path)) {
        return;
    }
    let from = source.join(relative);
    let to = destination.join(relative);
    fs::create_dir_all(&to).expect("copy directory");
    for entry in fs::read_dir(&from).expect("source") {
        let entry = entry.expect("entry");
        if entry.file_type().expect("type").is_dir() {
            copy(source, destination, &relative.join(entry.file_name()), left_out);
        } else {
            fs::copy(entry.path(), to.join(entry.file_name())).expect("file copy");
        }
    }
}
struct World {
    _scratch: Scratch,
    home: Home,
    session: u64,
    place: PassPlace<RealDrive, Archive>,
}
impl World {
    fn new() -> Self {
        Self::with_ports(false)
    }
    fn with_ports(fixed: bool) -> Self {
        Self::with_kind(fixed, temper_qa_tool::Kind::Box)
    }
    fn with_kind(fixed: bool, kind: temper_qa_tool::Kind) -> Self {
        let scratch = Scratch::new("qa-place");
        let mut config = serde_json::json!({"owner":"qa","git":{"directory":"unused"},"agent":{"command":["unused"],"triage":"low","pass":"high"},"request":{"command":["unused"]}});
        if fixed {
            config["places"] = serde_json::json!([{"directory":"places/1", "ports":[31001,31002,31003,31004,31005,31006,31007,31008,31009]}]);
        }
        if kind == temper_qa_tool::Kind::Deployment {
            config["places"][0]["kind"] = serde_json::json!("deployment");
        }
        fs::write(scratch.path().join("qa.json"), serde_json::to_vec(&config).expect("JSON")).expect("configuration");
        let home = Home::open(scratch.path()).expect("home");
        let started = home
            .transact(|ledger| {
                if kind == temper_qa_tool::Kind::Box {
                    super::passes::queue(ledger, "work", "web");
                    super::passes::queue(ledger, "home", "web");
                } else {
                    super::passes::queue_kinds(ledger);
                }
                super::passes::build(ledger, OLD, &[1], 1);
                super::passes::build(ledger, BUILD, if kind == temper_qa_tool::Kind::Box { &[1, 2] } else { &[1] }, 2);
                ledger.mark_build_deleted(OLD, 3)?;
                Ok(ledger.start_pass("places/1", kind, &["ctl".into(), "web".into()], "high", 4).expect("pass"))
            })
            .expect("pass through ledger operations");
        let environment = vec![
            (Box::from(b"PATH".as_slice()), Box::from(b"/usr/bin:/bin".as_slice())),
            (Box::from(b"QA_RECIPE_ENV".as_slice()), Box::from(b"captured".as_slice())),
        ];
        let place = PassPlace::for_session(
            &home,
            started.session,
            RealDrive { environment },
            Archive { store: scratch.path().join("archive-face"), ..Archive::default() },
        )
        .expect("session place");
        place.paths.ensure().expect("place layout");
        fs::create_dir_all(&place.setups).expect("pass setups");
        for (name, hash) in [("welcome", WELCOME), ("project-landing", PROJECT), ("chat-draft", CHAT), ("added", ADDED)]
        {
            fs::write(place.paths.directory.join(format!("hash-{name}")), format!("{hash}\n")).expect("hash answer");
        }
        for (name, parent) in
            [("welcome", None), ("project-landing", Some("welcome")), ("chat-draft", Some("project-landing"))]
        {
            let directory = place.setups.join(name);
            fs::create_dir_all(&directory).expect("recipe directory");
            fs::write(
                directory.join("recipe"),
                format!(
                    "recipe 1\n{}web see \"Ready\"\n",
                    parent.map_or(String::new(), |parent| format!("after \"{parent}\"\n"))
                ),
            )
            .expect("recipe");
            fs::write(directory.join("README.md"), "A state.\n").expect("readme");
        }
        fs::write(place.paths.directory.join("catalogue"), CATALOGUE).expect("catalogue");
        let tool = place.paths.bin.join("temper-client-journeys");
        let tool_text =
            if kind == temper_qa_tool::Kind::Deployment { TOOL.replace("box", "deployment") } else { TOOL.into() };
        fs::write(&tool, tool_text).expect("fake build");
        fs::set_permissions(&tool, fs::Permissions::from_mode(0o700)).expect("executable");
        Self { _scratch: scratch, home, session: started.session, place }
    }
    fn copy(&self, name: &str, hash: &str, commit: &str) -> PathBuf {
        let directory = self.place.paths.copies.join(name).join(format!("{hash}-{commit}.tar"));
        fs::create_dir_all(directory.parent().expect("copy parent")).expect("copy directory");
        let snapshot = self.home.directory.join("test-copies").join(name).join(format!("{hash}-{commit}"));
        fs::create_dir_all(snapshot.join("config")).expect("copy config");
        fs::create_dir_all(snapshot.join("data")).expect("copy data");
        fs::write(snapshot.join("data/state"), format!("{name}\n")).expect("copied state");
        fs::write(&directory, snapshot.to_str().expect("snapshot path")).expect("archive marker");
        directory
    }
    fn setup(&mut self, name: &str) -> (u8, String) {
        let mut output = Vec::new();
        let code = self.place.setup(name, &mut output).expect("setup command");
        (code, String::from_utf8(output).expect("output"))
    }
    fn walks(&self) -> Vec<String> {
        fs::read_to_string(self.place.paths.directory.join("arguments"))
            .unwrap_or_default()
            .lines()
            .filter(|line| line.starts_with("recipe walk "))
            .map(str::to_owned)
            .collect()
    }
    fn fail(&self, name: &str, from: &str, code: u8) {
        fs::write(self.place.paths.directory.join(format!("walk-{name}-{from}")), code.to_string()).expect("walk exit");
    }
    fn setup_file(&self) -> String {
        fs::read_to_string(&self.place.paths.setup).expect("setup file")
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

fn ctl_world() -> World {
    let mut world = World::new();
    for (name, recipe) in [
        ("welcome", "recipe 1\ncheck\nctl project list\n"),
        (
            "project-landing",
            "recipe 1\nafter \"welcome\"\nctl project make Hello\ncheck\nctl project list --until made:1\nctl project show Hello\n",
        ),
    ] {
        fs::write(world.place.setups.join(name).join("recipe"), recipe).expect("ctl recipe");
        fs::write(world.place.paths.directory.join(format!("expected-{name}")), recipe).expect("expected recipe");
    }
    let tool = TOOL
        .replace(
            "test \"$QA_RECIPE_ENV\" = captured",
            "test \"$QA_RECIPE_ENV\" = captured\ntest \"${SKEIN_TEST_CHROMIUM+x}\" != x",
        )
        .replace("cat \"hash-$3\" ;;", "cmp \"expected-$3\" \"$5/$3/recipe\"\n    cat \"hash-$3\" ;;");
    fs::write(world.place.paths.bin.join("temper-client-journeys"), tool).expect("ctl fake build");
    world.home.configuration.surfaces = vec!["ctl".into()];
    world.place =
        PassPlace::for_session(&world.home, world.session, world.place.drive, world.place.archive).expect("ctl place");
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
        fs::write(world.place.paths.directory.join("expected-added"), &expected).expect("expected ctl recipe");
        let mut output = Vec::new();
        let code = world.place.recipe("added", &input, &[], &mut output).expect("write and prove");
        let output = String::from_utf8(output).expect("output");
        assert_eq!(code, 0, "{output}");
        assert!(output.contains(&format!("recipe added walked from {after}:")), "{output}");
        assert_eq!(fs::read_to_string(world.place.setups.join("added/recipe")).expect("written recipe"), expected);
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
        fs::write(world.place.paths.directory.join("catalogue"), catalogue).expect("updated catalogue");
        let saved = world.place.archive.saved.len();
        let (code, output) = world.setup("added");
        assert_eq!(code, 0, "{output}");
        assert_eq!(world.place.archive.saved.len(), saved);
        assert!(world.walks().last().expect("copy check").starts_with("recipe walk added --from added "));
        assert_eq!(world.setup_file(), format!("added {ADDED}\n"));
    }
}

/// Run the test in an isolated process; the shell restores its mask even on failure.
#[expect(
    clippy::disallowed_methods,
    reason = "the test harness reads its own isolated umask subprocess marker, not the tool's environment"
)]
pub(super) fn under_umasks(name: &str, masks: &[&str]) -> bool {
    if std::env::var_os("QA_TEST_UMASK").is_some() {
        return false;
    }
    for mask in masks {
        let status = Command::new("sh")
            .args(["-c", "old=$(umask); umask \"$1\"; \"$2\" --exact \"$3\" --nocapture; result=$?; umask \"$old\"; exit \"$result\"", "qa-umask", mask])
            .arg(std::env::current_exe().expect("test executable"))
            .arg(name)
            .env("QA_TEST_UMASK", mask)
            .stdin(Stdio::null())
            .status()
            .expect("test under umask");
        assert!(status.success(), "{name} under umask {mask}: {status}");
    }
    true
}

pub(super) fn assert_mode(path: &Path, expected: u32) {
    assert_eq!(fs::metadata(path).expect("mode").permissions().mode() & 0o7777, expected, "{}", path.display());
}

#[test]
fn restores_are_private_under_permissive_umasks() {
    if under_umasks("place::restores_are_private_under_permissive_umasks", &["022", "000"]) {
        return;
    }
    let mut world = World::new();
    assert_eq!(world.setup("welcome").0, 0);
    let paths = &world.place.paths;
    for path in [&paths.directory, &paths.scratch, &paths.copies, &paths.copies.join("welcome")] {
        assert_mode(path, 0o700);
    }
    for path in [&paths.setup, &paths.walk_log, &paths.copies.join("welcome").join(format!("{WELCOME}-{BUILD}.tar"))] {
        assert_mode(path, 0o600);
    }
}

#[test]
fn setup_tightens_existing_place_directories_and_files() {
    let mut world = World::new();
    let paths = &world.place.paths;
    for path in [&paths.directory, &paths.scratch, &paths.copies] {
        fs::set_permissions(path, fs::Permissions::from_mode(0o755)).expect("old directory mode");
    }
    for path in [&paths.walk_log, &paths.directory.join("setup.tmp")] {
        fs::write(path, "old").expect("old file");
        fs::set_permissions(path, fs::Permissions::from_mode(0o666)).expect("old file mode");
    }
    assert_eq!(world.setup("welcome").0, 0);
    let paths = &world.place.paths;
    for path in [&paths.directory, &paths.scratch, &paths.copies] {
        assert_mode(path, 0o700);
    }
    for path in [&paths.setup, &paths.walk_log] {
        assert_mode(path, 0o600);
    }
}

fn assert_install_modes(source: &Path, installed: &Path) {
    assert_mode(installed, 0o700);
    for entry in fs::read_dir(source).expect("build directory") {
        let entry = entry.expect("entry");
        let target = installed.join(entry.file_name());
        let metadata = entry.metadata().expect("build mode");
        if metadata.is_dir() {
            assert_install_modes(&entry.path(), &target);
        } else {
            assert_mode(&target, metadata.permissions().mode() & 0o7777);
        }
    }
}

#[test]
fn installation_privatizes_directories_but_keeps_build_file_modes() {
    use super::world::{Agent, World, baseline, land};
    use sha2::{Digest, Sha256};
    use std::io::Write;
    if under_umasks("place::installation_privatizes_directories_but_keeps_build_file_modes", &["022"]) {
        return;
    }
    let world = World::new();
    let mut run = world.start(baseline(), &[Agent::Case, Agent::Never]);
    run.tick();
    land(&mut run, 2, 7);
    let build = world.installable_build(2);
    for (path, mode) in [("bin/nested/deep/tool", 0o711), ("bundle/assets/deep/note", 0o640)] {
        let full = build.join(path);
        fs::create_dir_all(full.parent().expect("parent")).expect("build directories");
        fs::write(&full, path).expect("build file");
        fs::set_permissions(&full, fs::Permissions::from_mode(mode)).expect("build mode");
        writeln!(
            fs::OpenOptions::new().append(true).open(build.join("manifest")).expect("manifest"),
            "{:x}  {path}",
            Sha256::digest(path.as_bytes())
        )
        .expect("manifest row");
    }
    let place = world.home.directory.join("places/1");
    for path in [&place, &place.join("box"), &place.join("copies")] {
        fs::create_dir_all(path).expect("existing place");
        fs::set_permissions(path, fs::Permissions::from_mode(0o755)).expect("old directory mode");
    }
    run.tick();
    run.tick();
    assert_eq!(world.ledger().passes.len(), 1, "installed pass started");
    for name in ["bin", "bundle"] {
        assert_install_modes(&build.join(name), &place.join(name));
    }
    for path in [&place, &place.join("box"), &place.join("copies")] {
        assert_mode(path, 0o700);
    }
    assert_mode(&place.join("drive.lock"), 0o600);
}

#[test]
fn setups_and_proving_walks_receive_only_their_places_ports() {
    for fixed in [false, true] {
        let mut world = World::with_ports(fixed);
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
    let mut world = World::new();
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
    assert_eq!(fs::read_to_string(world.place.paths.walk_log).expect("log"), "walked chat-draft\n");
}

#[test]
fn a_refused_copy_rebuilds_from_the_nearest_ancestor_and_prunes_every_saved_copy() {
    let mut world = World::new();
    world.copy("welcome", WELCOME, OLD);
    let refused = world.copy("chat-draft", CHAT, OLD);
    world.place.archive.removed_before_retry = Some(refused.clone());
    let stale = world.copy("project-landing", WELCOME, OLD);
    world.fail("chat-draft", "chat-draft", 3);
    let (code, output) = world.setup("chat-draft");
    assert_eq!(code, 0);
    assert!(!refused.exists());
    assert!(!stale.exists());
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
        let files = fs::read_dir(world.place.paths.copies.join(name))
            .expect("saved directory")
            .map(|entry| entry.expect("copy").file_name())
            .collect::<Vec<_>>();
        assert_eq!(files, [std::ffi::OsString::from(format!("{hash}-{BUILD}.tar"))]);
    }
    assert_eq!(world.place.archive.saved.len(), 2);
    assert_eq!(world.setup_file(), format!("chat-draft {CHAT}\n"));
    let log = fs::read_to_string(&world.place.paths.walk_log).expect("whole walk log");
    assert!(log.contains("answer 1\n"));
    assert!(log.ends_with("walked chat-draft\n"));
}

#[test]
fn no_copy_empties_the_box_then_walks_and_saves_each_member_root_first() {
    let mut world = World::new();
    for directory in ["config", "data", "cache"] {
        fs::create_dir_all(world.place.paths.scratch.join(directory)).expect("old box");
    }
    fs::write(world.place.paths.scratch.join("cache/kept"), "cache").expect("cache");
    let (code, output) = world.setup("chat-draft");
    assert_eq!(code, 0);
    assert!(output.contains("setup chat-draft: made from a fresh box"), "{output}");
    let walks = world.walks();
    assert_eq!(walks.len(), 3);
    assert!(walks[0].starts_with("recipe walk welcome --bin "));
    assert!(walks[1].starts_with("recipe walk project-landing --from welcome "));
    assert!(walks[2].starts_with("recipe walk chat-draft --from project-landing "));
    assert_eq!(world.place.archive.saved.len(), 3);
    assert!(world.place.paths.scratch.join("cache/kept").exists());
}

#[test]
fn box_and_other_walk_failures_stop_with_only_the_last_forty_lines() {
    for (exit, expected) in [(3, 4), (4, 4), (1, 1), (9, 1)] {
        let mut world = World::new();
        fs::write(&world.place.paths.setup, "old setup").expect("prior setup");
        world.fail("welcome", "fresh", exit);
        let (code, output) = world.setup("chat-draft");
        assert_eq!(code, expected);
        if expected == 4 {
            assert!(output.contains("setup chat-draft: the box failed walking welcome"), "{output}");
        }
        assert!(!output.contains("answer 6\n"), "{output}");
        assert!(output.contains("answer 7\n"), "{output}");
        assert_eq!(world.walks().len(), 1);
        assert!(fs::read_to_string(&world.place.paths.walk_log).expect("log").starts_with("answer 1\n"));
        assert!(!world.place.paths.setup.exists());
        assert_eq!(world.place.archive.saved.len(), 0);
    }
}

#[test]
fn a_line_stop_reports_the_copy_build_and_only_landings_since_its_deleted_build() {
    let mut world = World::new();
    world.copy("chat-draft", CHAT, OLD);
    world.fail("chat-draft", "chat-draft", 5);
    let (code, output) = world.setup("chat-draft");
    assert_eq!(code, 5);
    assert!(output.contains("setup chat-draft: stopped at chat-draft:7: missing Start"), "{output}");
    assert!(output.contains(&format!("copy started from: chat-draft made at {OLD}")), "{output}");
    assert!(output.contains(&format!("recipe chat-draft last walked on: {OLD}")), "{output}");
    assert!(output.contains("landing 2: goal 12: a landing"), "{output}");
    assert!(!output.contains("landing 1:"), "{output}");
    assert!(!world.place.paths.setup.exists());
    let mut fresh = World::new();
    fresh.fail("welcome", "fresh", 5);
    let (code, output) = fresh.setup("welcome");
    assert_eq!(code, 5);
    assert!(output.contains("recipe welcome last walked on: no copy"));
    assert!(output.contains("landing 1: goal 11: a landing"));
    assert!(output.contains("landing 2: goal 12: a landing"));
}

#[test]
fn fresh_removes_only_config_and_data_and_writes_its_single_word() {
    let mut world = World::new();
    for directory in ["config", "data", "cache"] {
        fs::create_dir_all(world.place.paths.scratch.join(directory)).expect("box");
    }
    assert_eq!(world.setup("fresh"), (0, "setup fresh: an empty box\n".into()));
    assert!(!world.place.paths.scratch.join("config").exists());
    assert!(!world.place.paths.scratch.join("data").exists());
    assert!(world.place.paths.scratch.join("cache").is_dir());
    assert_eq!(world.setup_file(), "fresh\n");
    assert_eq!(world.walks().len(), 0);
}

#[test]
fn unknown_unread_and_nonpass_setups_refuse_before_touching_the_box() {
    let mut world = World::new();
    let mut output = Vec::new();
    fs::write(&world.place.paths.setup, "fresh\n").expect("prior setup");
    assert_eq!(
        world.place.setup("missing", &mut output).expect_err("unknown").to_string(),
        "setup missing is not in pass 1's setups; setups: fresh, chat-draft, project-landing, welcome"
    );
    fs::write(world.place.paths.directory.join("refuse-chat-draft"), "chat-draft:7: no check\n").expect("refusal");
    assert_eq!(
        world.place.setup("chat-draft", &mut output).expect_err("unread").to_string(),
        "recipe chat-draft: chat-draft:7: no check"
    );
    assert_eq!(world.setup_file(), "fresh\n");
    fs::write(world.place.paths.directory.join("catalogue"), "welcome: refused: welcome/README.md: unread\n")
        .expect("catalogue refusal");
    assert_eq!(
        world.place.setup("welcome", &mut output).expect_err("catalogue refused").to_string(),
        "recipe welcome: welcome/README.md: unread"
    );
    assert_eq!(world.walks().len(), 0);
    assert!(PassPlace::for_session(&world.home, 1, RealDrive { environment: vec![] }, Archive::default()).is_err());
    world
        .home
        .transact(|ledger| ledger.end_pass_session(world.session, temper_qa_tool::Exit::Bound, None, 9, "qa", &[]))
        .expect("end session");
    assert!(
        PassPlace::for_session(&world.home, world.session, RealDrive { environment: vec![] }, Archive::default())
            .is_err()
    );
}

#[test]
fn another_process_holding_the_drive_lock_refuses_setup_and_recipe_and_probe_releases() {
    let mut world = World::new();
    assert!(!place_busy(&world.place.paths).expect("free"));
    let mut holder = Command::new("flock")
        .args(["--exclusive"])
        .arg(&world.place.paths.drive_lock)
        .args(["/bin/sh", "-c", "printf 'ready\\n'; cat >/dev/null"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("holder");
    let mut line = String::new();
    BufReader::new(holder.stdout.take().expect("stdout")).read_line(&mut line).expect("ready");
    assert_eq!(line, "ready\n");
    assert!(place_busy(&world.place.paths).expect("busy"));
    let mut output = Vec::new();
    let expected = "the drive session is open at places/1; close it first: temper-qa pass close";
    assert_eq!(world.place.setup("fresh", &mut output).expect_err("busy").to_string(), expected);
    let input =
        RecipeInput::decode(br#"{"after":"fresh","lines":["web see \"Ready\""],"readme":"Ready."}"#).expect("recipe");
    assert_eq!(world.place.recipe("added", &input, &[], &mut output).expect_err("busy").to_string(), expected);
    drop(holder.stdin.take());
    assert!(holder.wait().expect("holder ended").success());
    assert!(!place_busy(&world.place.paths).expect("free again"));
    let lock = fs::OpenOptions::new().read(true).write(true).open(&world.place.paths.drive_lock).expect("lock");
    lock.try_lock().expect("probe did not keep it");
    assert_eq!(output.len(), 0);
}

#[test]
fn a_new_recipe_is_written_and_proven_from_its_parent_even_when_its_own_copy_exists() {
    let mut world = World::new();
    world.copy("welcome", WELCOME, OLD);
    world.copy("added", ADDED, OLD);
    let (code, output) = world.recipe("welcome");
    assert_eq!(code, 0);
    assert!(output.contains("recipe added walked from welcome: 444444444444"), "{output}");
    let directory = world.place.setups.join("added");
    assert_eq!(
        fs::read_to_string(directory.join("recipe")).expect("recipe"),
        "recipe 1\nafter \"welcome\"\nweb see \"Added\"\n"
    );
    assert_eq!(fs::read_to_string(directory.join("README.md")).expect("readme"), "An added state.\n");
    assert_eq!(
        serde_json::from_slice::<serde_json::Value>(&fs::read(directory.join("scripts.json")).expect("scripts"))
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
    assert_eq!(fs::read_to_string(directory.join("recipe")).expect("recipe"), "recipe 1\nweb see \"Added\"\n");
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
    let mut world = World::new();
    let files = world.home.directory.join("passes/1/files");
    let expected = world.place.paths.directory.join("expected-files");
    for root in [&files, &expected] {
        fs::create_dir_all(root.join("dir")).expect("nested files");
        fs::write(root.join("a.txt"), b"first\n").expect("first file");
        fs::write(root.join("dir/b.txt"), b"second\0\xff").expect("second file");
    }
    fs::write(files.join("unmentioned"), b"do not copy").expect("unmentioned file");
    let code = world.place.recipe("added", &commit_input(), &[], &mut Vec::new()).expect("copied before hash");
    assert_eq!(code, 0);
    let copied = world.place.setups.join("added/files");
    assert_eq!(fs::read(copied.join("a.txt")).expect("first copied"), b"first\n");
    assert_eq!(fs::read(copied.join("dir/b.txt")).expect("second copied"), b"second\0\xff");
    assert!(!copied.join("unmentioned").exists());
    assert_eq!(world.walks().len(), 1, "proven from fresh");
}

#[test]
fn a_new_recipe_reads_quoted_commit_paths_as_bytes() {
    let mut world = World::new();
    let files = world.home.directory.join("passes/1/files");
    fs::create_dir_all(&files).expect("pass files");
    let paths = [b"a space.txt".as_slice(), b"a\"quote.txt", b"a\\slash.txt", b"a\xff.txt"];
    for path in paths {
        fs::write(files.join(std::ffi::OsString::from_vec(path.to_vec())), path).expect("commit file");
    }
    let mut input = commit_input();
    input.lines[0] = r#"forge commit forgejo playground/widget "m with \"quotes\"" "a space.txt" "a\"quote.txt" "a\\slash.txt" "a\xff.txt""#.into();
    input.lines.insert(1, r#"forge commit forgejo playground/widget "again" "a space.txt""#.into());
    assert_eq!(world.place.recipe("added", &input, &[], &mut Vec::new()).expect("quoted paths"), 0);
    let copied = world.place.setups.join("added/files");
    assert_eq!(fs::read_dir(&copied).expect("copied files").count(), paths.len());
    for path in paths {
        assert_eq!(fs::read(copied.join(std::ffi::OsString::from_vec(path.to_vec()))).expect("copied bytes"), path);
    }
}

#[test]
fn commit_paths_cannot_escape_the_pass_or_recipe() {
    for path in ["../escape", "/escape", "dir/../escape", "dir//escape", "./escape"] {
        let mut world = World::new();
        let files = world.home.directory.join("passes/1/files");
        fs::create_dir_all(files.join("dir")).expect("pass files");
        fs::write(files.join("escape"), b"inside").expect("inside");
        fs::write(files.parent().expect("pass directory").join("escape"), b"outside").expect("outside");
        let mut input = commit_input();
        input.lines[0] = format!("forge commit forgejo playground/widget \"m\" \"{path}\"");
        let why = world.place.recipe("added", &input, &[], &mut Vec::new()).expect_err("invalid path");
        assert!(why.to_string().contains("invalid path"), "{why}");
        assert!(!world.place.setups.join("added").exists());
    }
}

#[test]
fn a_missing_commit_file_refuses_by_name_and_removes_the_recipe() {
    let mut world = World::new();
    let files = world.home.directory.join("passes/1/files");
    fs::create_dir_all(&files).expect("pass files");
    fs::write(files.join("a.txt"), b"first").expect("first file");
    let why = world.place.recipe("added", &commit_input(), &[], &mut Vec::new()).expect_err("missing file");
    assert!(why.to_string().contains("dir/b.txt"), "{why}");
    assert!(!world.place.setups.join("added").exists(), "no partially written recipe");
    assert_eq!(world.walks().len(), 0);
}

#[test]
fn a_recipe_without_commits_writes_no_files_directory() {
    let mut world = World::new();
    assert_eq!(world.recipe("fresh").0, 0);
    assert!(!world.place.setups.join("added/files").exists());
}

#[test]
fn an_unread_or_failed_new_recipe_is_removed_with_the_walks_exit() {
    let mut unread = World::new();
    fs::write(unread.place.paths.directory.join("refuse-added"), "added:2: no check").expect("hash refusal");
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
    assert!(!unread.place.setups.join("added").exists());
    for (code, expected) in [(3, 4), (4, 4), (5, 5), (1, 1)] {
        let mut world = World::new();
        world.fail("added", "fresh", code);
        let (exit, output) = world.recipe("fresh");
        assert_eq!(exit, expected, "{output}");
        assert!(!world.place.setups.join("added").exists());
        assert!(!world.place.paths.setup.exists());
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
    let mut world = World::new();
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
    let paths = PlacePaths::new(&world.place.paths.directory, temper_qa_tool::Kind::Box);
    assert_eq!(paths.scratch, world.place.paths.directory.join("box"));
}

const PINNED_AGENT: &str = "data/box/fake-backends/data/workspaces/agent";
const CHECKOUT_AGENT: &str = "data/box/fake-backends/data/workspaces/1/repo/agent/kept";

fn write_copy_state(scratch: &Path, pinned: bool) {
    let kept = scratch.join(CHECKOUT_AGENT);
    fs::create_dir_all(kept.parent().expect("checkout agent parent")).expect("checkout agent directory");
    fs::write(kept, "checkout state").expect("checkout agent state");
    fs::write(scratch.join("data/state"), "setup state").expect("setup state");
    if pinned {
        let agent = scratch.join(PINNED_AGENT);
        fs::create_dir_all(&agent).expect("pinned agent directory");
        fs::write(agent.join("temper-box"), "pinned binary").expect("pinned binary");
        fs::write(agent.join("agent.json"), "{}").expect("pinned document");
    }
}

fn assert_copy_state(scratch: &Path) {
    assert!(!scratch.join(PINNED_AGENT).exists(), "the pinned agent directory is left out");
    assert_eq!(fs::read_to_string(scratch.join("data/state")).expect("setup state"), "setup state");
    assert_eq!(fs::read_to_string(scratch.join(CHECKOUT_AGENT)).expect("checkout state"), "checkout state");
}

fn real_archive() -> RealArchive {
    RealArchive { environment: vec![(Box::from(b"PATH".as_slice()), Box::from(b"/usr/bin:/bin".as_slice()))] }
}

#[test]
fn real_copies_leave_out_only_the_pinned_agent_and_allow_its_absence() {
    for pinned in [true, false] {
        let scratch = Scratch::new("qa-real-copy");
        let source = scratch.path().join("source");
        let restored = scratch.path().join("restored");
        let archive = scratch.path().join("copy.tar");
        write_copy_state(&source, pinned);
        fs::create_dir(&restored).expect("restore directory");
        let mut face = real_archive();
        face.create(&source, &archive, &[PINNED_AGENT]).expect("create copy");
        // With a pin, inspect create's archive without masking it through extract's exclusion.
        // Without one, exercise extract's exclusion of an absent directory as well.
        let left_out: &[&str] = if pinned { &[] } else { &[PINNED_AGENT] };
        face.extract(&archive, &restored, left_out).expect("restore copy");
        assert_copy_state(&restored);
    }
}

#[test]
fn real_restore_leaves_out_the_pinned_agent_from_an_older_copy() {
    let scratch = Scratch::new("qa-real-restore");
    let source = scratch.path().join("source");
    let restored = scratch.path().join("restored");
    let archive = scratch.path().join("copy.tar");
    write_copy_state(&source, true);
    fs::create_dir(&restored).expect("restore directory");
    let output = Command::new("tar")
        .arg("--create")
        .arg("--file")
        .arg(&archive)
        .arg("--directory")
        .arg(&source)
        .arg("data")
        .stdin(Stdio::null())
        .output()
        .expect("plain tar");
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    real_archive().extract(&archive, &restored, &[PINNED_AGENT]).expect("restore older copy");
    assert_copy_state(&restored);
}

#[test]
fn a_place_saves_state_without_the_walks_pinned_agent() {
    let mut world = World::new();
    let (code, output) = world.setup("welcome");
    assert_eq!(code, 0, "{output}");
    assert!(world.place.paths.scratch.join(PINNED_AGENT).join("temper-box").exists(), "the walk pinned its agent");
    assert_eq!(world.place.archive.saved.len(), 1);
    let saved = world.place.paths.copies.join("welcome").join(format!("{WELCOME}-{BUILD}.tar"));
    let snapshot = fs::read_to_string(saved).expect("archive marker");
    assert!(!Path::new(&snapshot).join(PINNED_AGENT).exists(), "save leaves out the pinned directory");
    assert_eq!(fs::read_to_string(Path::new(&snapshot).join("data/state")).expect("saved state"), "welcome\n");
}

#[test]
fn a_place_restores_an_older_copy_without_its_pinned_agent() {
    let mut world = World::new();
    let archive = world.copy("welcome", WELCOME, OLD);
    let snapshot = fs::read_to_string(&archive).expect("archive marker");
    write_copy_state(Path::new(&snapshot), true);
    // Inspect the restored state without simulating another worker start in the check.
    let check = TOOL.replace(
        "    mkdir -p box/data/box/fake-backends/data/workspaces/agent\n    printf 'pinned\\n' > box/data/box/fake-backends/data/workspaces/agent/temper-box\n",
        "",
    ).replace("    printf '%s\\n' \"$name\" > box/data/state\n", "");
    fs::write(world.place.paths.bin.join("temper-client-journeys"), check).expect("check script");
    let (code, output) = world.setup("welcome");
    assert_eq!(code, 0, "{output}");
    assert_eq!(world.place.archive.restored, [archive]);
    assert_copy_state(&world.place.paths.scratch);
    assert!(Path::new(&snapshot).join(PINNED_AGENT).exists(), "the existing copy is not rewritten");
}

#[test]
fn a_copy_takes_the_members_its_box_made() {
    // A box with fake backends keeps its configuration under data/, so its config/ may never be made.
    let scratch = Scratch::new("qa-copy-members");
    let box_directory = scratch.path();
    assert!(copy_members(box_directory).is_err(), "an empty box directory has nothing to copy");
    fs::create_dir(box_directory.join("data")).expect("data");
    assert_eq!(copy_members(box_directory).expect("members"), ["data"]);
    fs::create_dir(box_directory.join("config")).expect("config");
    assert_eq!(copy_members(box_directory).expect("members"), ["config", "data"]);
}

#[test]
fn ctl_only_pass_refuses_web_and_tui_in_a_setups_parent_before_restore() {
    for surface in ["web", "tui"] {
        let mut world = World::new();
        world.home.configuration.surfaces = vec!["ctl".into()];
        world.place = PassPlace::for_session(
            &world.home,
            world.session,
            RealDrive {
                environment: vec![(Box::from(b"QA_RECIPE_ENV".as_slice()), Box::from(b"captured".as_slice()))],
            },
            Archive::default(),
        )
        .expect("ctl pass place");
        fs::write(&world.place.paths.setup, "fresh\n").expect("prior setup");
        fs::write(
            world.place.setups.join("chat-draft/recipe"),
            "recipe 1\nafter project-landing\ncheck\nctl task list\n",
        )
        .expect("ctl child");
        fs::write(world.place.setups.join("project-landing/recipe"), format!("recipe 1\n{surface} see Ready\n"))
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
    let mut world = World::new();
    let files = world.home.directory.join("passes/1/files");
    fs::create_dir_all(&files).expect("files");
    fs::write(files.join("a.txt"), b"pushed").expect("file");
    let mut input = commit_input();
    input.lines[0] = r#"forge push forgejo playground/widget work/topic "m" "a.txt""#.into();
    assert_eq!(world.place.recipe("added", &input, &[], &mut Vec::new()).expect("recipe"), 0);
    assert_eq!(fs::read(world.place.setups.join("added/files/a.txt")).expect("kept file"), b"pushed");
    assert!(fs::read_to_string(world.place.setups.join("added/recipe")).expect("recipe").contains(&input.lines[0]));
}

fn deployment_world() -> World {
    let world = World::with_kind(true, temper_qa_tool::Kind::Deployment);
    fs::create_dir_all(world.place.setups.join("welcome/config")).expect("root config");
    let tool = world.place.paths.bin.join("temper-client-journeys");
    let text = fs::read_to_string(&tool).expect("fake").replace("mkdir -p deployment/config deployment/data", "mkdir -p deployment/config deployment/data\n    if test \"$from\" = fresh; then printf 'saved config' > deployment/config/receipt; else test \"$(cat deployment/config/receipt)\" = 'saved config'; fi");
    fs::write(tool, text).expect("config-checking fake");
    world
}

#[test]
fn deployment_walks_save_and_restore_their_private_scratch_with_bundle() {
    if under_umasks("place::deployment_walks_save_and_restore_their_private_scratch_with_bundle", &["022", "000"]) {
        return;
    }
    let mut world = deployment_world();
    let paths = world.place.paths.clone();
    assert_eq!(paths.scratch, paths.directory.join("deployment"));
    assert!(!paths.directory.join("box").exists());
    assert_mode(&paths.scratch, 0o700);
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
    fs::write(paths.scratch.join("data/state"), "changed").expect("changed state");
    fs::write(paths.scratch.join("config/receipt"), "changed").expect("changed config");
    assert_eq!(world.setup("chat-draft").0, 0);
    assert_eq!(fs::read_to_string(paths.scratch.join("config/receipt")).expect("restored config"), "saved config");
    assert_eq!(fs::read_to_string(paths.scratch.join("data/state")).expect("checked state"), "chat-draft\n");
    assert_eq!(world.place.archive.saved.len(), 3, "checking a copy does not replace it");
}

#[test]
fn setups_refuse_fresh_and_chains_of_the_other_places_kind_before_walking() {
    let mut deployment = deployment_world();
    let before = deployment.walks();
    assert_eq!(
        deployment.place.setup("fresh", &mut Vec::new()).expect_err("no fresh").to_string(),
        "a deployment's place has no fresh setup: its setups start from their root's config/"
    );
    fs::remove_dir(deployment.place.setups.join("welcome/config")).expect("box root");
    assert_eq!(
        deployment.place.setup("chat-draft", &mut Vec::new()).expect_err("box chain").to_string(),
        "setup chat-draft is a box's; places/1 is a deployment's place"
    );
    assert_eq!(deployment.walks(), before);
    let mut boxed = World::new();
    fs::create_dir(boxed.place.setups.join("welcome/config")).expect("deployment root");
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
    fs::create_dir_all(files.join("patches")).expect("patch directory");
    fs::write(files.join("patches/engine.json"), b"{\"application\":{}}").expect("patch");
    let input = RecipeInput::decode(br#"{"after":"welcome","lines":["deployment patch engine \"patches/engine.json\"","check","ctl task list"],"readme":"A patched deployment."}"#).expect("recipe input");
    assert_eq!(world.place.recipe("added", &input, &[], &mut Vec::new()).expect("recipe"), 0);
    let added = world.place.setups.join("added");
    assert_eq!(fs::read(added.join("files/patches/engine.json")).expect("recipe patch"), b"{\"application\":{}}");
    assert!(!added.join("config").exists());
    assert!(
        fs::read_to_string(added.join("recipe"))
            .expect("recipe")
            .starts_with("recipe 1\nafter \"welcome\"\ndeployment patch engine")
    );
    fs::write(world.place.paths.directory.join("catalogue"), format!("{CATALOGUE}added (after welcome): Patched.\n"))
        .expect("catalogue");
    assert_eq!(world.setup("added").0, 0, "the new recipe is a deployment's through its root");
    let missing = RecipeInput { lines: vec!["deployment patch agent missing.json".into()], ..input };
    let why = world.place.recipe("added", &missing, &[], &mut Vec::new()).expect_err("missing patch").to_string();
    assert!(why.contains("missing.json"), "{why}");
    assert!(!added.exists(), "a refused recipe leaves nothing");
}
