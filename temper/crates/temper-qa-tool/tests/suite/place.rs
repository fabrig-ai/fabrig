//! Real place modes and the system archive (temper/docs/design/testing/qa/README.md, 6).
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
    place::PassPlace,
};

const OLD: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const BUILD: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
const WELCOME: &str = "1111111111111111111111111111111111111111111111111111111111111111";
const TOOL: &str = r#"#!/bin/sh
set -eu
test "$QA_RECIPE_ENV" = captured
case "$2" in
list) printf 'welcome: Welcome!\n' ;;
hash) cat "hash-$3" ;;
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
    saved: usize,
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
        self.saved += 1;
        let snapshot = self.store.join(self.saved.to_string());
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
    place: PassPlace<RealDrive, Archive, RealFiles, RealProcesses>,
}
impl World {
    fn new() -> Self {
        let scratch = Scratch::new("qa-place");
        let config = serde_json::json!({"owner":"qa","git":{"directory":"unused"},"agent":{"command":["unused"],"triage":"low","pass":"high"},"request":{"command":["unused"]}});
        fs::write(scratch.path().join("qa.json"), serde_json::to_vec(&config).expect("JSON")).expect("configuration");
        let home = Home::open(scratch.path(), &mut RealFiles).expect("home");
        let started = home
            .transact(&mut RealFiles, |ledger, _| {
                super::passes::queue(ledger, "work", "web");
                super::passes::queue(ledger, "home", "web");
                super::passes::build(ledger, OLD, &[1], 1);
                super::passes::build(ledger, BUILD, &[1, 2], 2);
                ledger.mark_build_deleted(OLD, 3)?;
                Ok(ledger
                    .start_pass("places/1", temper_qa_tool::Kind::Box, &["ctl".into(), "web".into()], "high", 4)
                    .expect("pass"))
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
        let recipe = place.setups.join("welcome");
        fs::create_dir_all(&recipe).expect("recipe directory");
        fs::write(recipe.join("recipe"), "recipe 1\nweb see \"Ready\"\n").expect("recipe");
        fs::write(recipe.join("README.md"), "A state.\n").expect("readme");
        fs::write(place.paths.directory.join("hash-welcome"), format!("{WELCOME}\n")).expect("hash answer");
        let tool = place.paths.bin.join("temper-client-journeys");
        fs::write(&tool, TOOL).expect("fake build");
        fs::set_permissions(&tool, fs::Permissions::from_mode(0o700)).expect("executable");
        Self { _scratch: scratch, place }
    }
    fn setup(&mut self, name: &str) -> (u8, String) {
        let mut output = Vec::new();
        let code = self.place.setup(name, &mut output).expect("setup command");
        (code, String::from_utf8(output).expect("output"))
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
