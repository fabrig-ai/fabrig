//! Places use a scripted build and directory copies, never a box or browser (qa/places.md, 8).
use skein_scratch::Scratch;
use std::{
    fs,
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
    process::{Command, Stdio},
};
use temper_qa_tool::faces::{FilesFace, Processes, RealFiles, RealProcesses};
use temper_qa_tool::{
    Home, Refusal,
    faces::{ArchiveFace, RealArchive, RealDrive},
    place::{PassPlace, RecipeInput},
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
    fn create(
        &mut self,
        scratch: &Path,
        archive: &Path,
        left_out: &[&str],
        _files: &mut impl FilesFace,
        _processes: &mut impl Processes,
    ) -> Result<(), Refusal> {
        assert_mode(archive, 0o600);
        self.saved.push(archive.to_owned());
        let snapshot = self.store.join(self.saved.len().to_string());
        for member in ["config", "data"] {
            copy(scratch, &snapshot, Path::new(member), left_out);
        }
        fs::write(archive, snapshot.to_str().expect("snapshot path")).expect("archive marker");
        Ok(())
    }
    fn extract(
        &mut self,
        archive: &Path,
        scratch: &Path,
        left_out: &[&str],
        _files: &mut impl FilesFace,
        _processes: &mut impl Processes,
    ) -> Result<(), Refusal> {
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
#[expect(dead_code, reason = "unused suite fixtures are removed in the goal’s final cleanup")]
struct World {
    _scratch: Scratch,
    home: Home,
    session: u64,
    place: PassPlace<RealDrive, Archive, RealFiles, RealProcesses>,
}
#[expect(dead_code, reason = "unused suite fixtures are removed in the goal’s final cleanup")]
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
        let home = Home::open(scratch.path(), &mut RealFiles).expect("home");
        let started = home
            .transact(&mut RealFiles, |ledger, _| {
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
            RealFiles,
            RealProcesses::default(),
        )
        .expect("session place");
        place.paths.ensure(&mut RealFiles).expect("place layout");
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

#[expect(dead_code, reason = "unused suite fixtures are removed in the goal’s final cleanup")]
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
    world.place = PassPlace::for_session(
        &world.home,
        world.session,
        world.place.drive,
        world.place.archive,
        RealFiles,
        RealProcesses::default(),
    )
    .expect("ctl place");
    world
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

#[expect(dead_code, reason = "unused suite fixtures are removed in the goal’s final cleanup")]
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

#[expect(dead_code, reason = "unused suite fixtures are removed in the goal’s final cleanup")]
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

fn assert_pinned_agent(scratch: &Path) {
    let agent = scratch.join(PINNED_AGENT);
    assert_eq!(fs::read(agent.join("temper-box")).expect("pinned binary"), b"pinned binary");
    assert_eq!(fs::read(agent.join("agent.json")).expect("pinned document"), b"{}");
}

#[expect(dead_code, reason = "unused suite fixtures are removed in the goal’s final cleanup")]
const TAR: &str = r#"#!/bin/sh
set -eu
printf '%s\n' "$@" > "$3.arguments"
if test -e "$5/$QA_TAR_PINNED" || test -L "$5/$QA_TAR_PINNED"; then
    printf 'present\n' > "$3.pinned"
else
    printf 'absent\n' > "$3.pinned"
fi
printf 'fake tar stderr\n' >&2
exit "$QA_TAR_EXIT"
"#;

#[expect(dead_code, reason = "unused suite fixtures are removed in the goal’s final cleanup")]
fn scripted_archive(directory: &Path, code: u8) -> RealArchive {
    let tar = directory.join("tar");
    fs::write(&tar, TAR).expect("fake tar");
    fs::set_permissions(tar, fs::Permissions::from_mode(0o700)).expect("executable tar");
    RealArchive {
        environment: vec![
            (Box::from(b"PATH".as_slice()), directory.as_os_str().as_encoded_bytes().into()),
            (Box::from(b"QA_TAR_PINNED".as_slice()), PINNED_AGENT.as_bytes().into()),
            (Box::from(b"QA_TAR_EXIT".as_slice()), code.to_string().into_bytes().into()),
        ],
    }
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
        face.create(&source, &archive, &[PINNED_AGENT], &mut RealFiles, &mut RealProcesses::default())
            .expect("create copy");
        assert_eq!(source.join(PINNED_AGENT).exists(), pinned, "save keeps the source's pinned directory");
        if pinned {
            assert_pinned_agent(&source);
        }
        // With a pin, inspect create's archive without masking it through extract's removal.
        // Without one, exercise extract's removal of an absent directory as well.
        let left_out: &[&str] = if pinned { &[] } else { &[PINNED_AGENT] };
        face.extract(&archive, &restored, left_out, &mut RealFiles, &mut RealProcesses::default())
            .expect("restore copy");
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
    real_archive()
        .extract(&archive, &restored, &[PINNED_AGENT], &mut RealFiles, &mut RealProcesses::default())
        .expect("restore older copy");
    assert_copy_state(&restored);
}

#[expect(dead_code, reason = "unused suite fixtures are removed in the goal’s final cleanup")]
fn deployment_world() -> World {
    let world = World::with_kind(true, temper_qa_tool::Kind::Deployment);
    fs::create_dir_all(world.place.setups.join("welcome/config")).expect("root config");
    let tool = world.place.paths.bin.join("temper-client-journeys");
    let text = fs::read_to_string(&tool).expect("fake").replace("mkdir -p deployment/config deployment/data", "mkdir -p deployment/config deployment/data\n    if test \"$from\" = fresh; then printf 'saved config' > deployment/config/receipt; else test \"$(cat deployment/config/receipt)\" = 'saved config'; fi");
    fs::write(tool, text).expect("config-checking fake");
    world
}

#[expect(dead_code, reason = "unused suite fixtures are removed in the goal’s final cleanup")]
const DEPLOYMENT_PINNED_AGENT: &str = "data/worker/workspaces/agent";
#[expect(dead_code, reason = "unused suite fixtures are removed in the goal’s final cleanup")]
const DEPLOYMENT_WORKSPACE: &str = "data/worker/workspaces/w1/file";
#[expect(dead_code, reason = "unused suite fixtures are removed in the goal’s final cleanup")]
const DEPLOYMENT_OTHER_AGENT: &str = "data/other/data/worker/workspaces/agent/kept";

#[expect(dead_code, reason = "unused suite fixtures are removed in the goal’s final cleanup")]
fn deployment_copy_state(scratch: &Path, pinned: bool) {
    for path in [DEPLOYMENT_WORKSPACE, DEPLOYMENT_OTHER_AGENT] {
        let file = scratch.join(path);
        fs::create_dir_all(file.parent().expect("state directory")).expect("state directory");
        fs::write(file, "kept state").expect("state");
    }
    if pinned {
        let agent = scratch.join(DEPLOYMENT_PINNED_AGENT);
        fs::create_dir_all(&agent).expect("agent directory");
        fs::write(agent.join("temper-agent"), "pinned binary").expect("pinned binary");
    }
}
#[expect(dead_code, reason = "unused suite fixtures are removed in the goal’s final cleanup")]
fn assert_deployment_copy_state(scratch: &Path) {
    assert!(!scratch.join(DEPLOYMENT_PINNED_AGENT).exists(), "the deployment's pinned agent is left out");
    for path in [DEPLOYMENT_WORKSPACE, DEPLOYMENT_OTHER_AGENT] {
        assert_eq!(fs::read_to_string(scratch.join(path)).expect("kept state"), "kept state");
    }
}
