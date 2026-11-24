//! Kept builds through the command entry (temper/docs/design/testing/qa/builds.md, 3 and 4).
use super::{Reply, invoke, output, put};
use std::{
    cell::Cell,
    path::{Path, PathBuf},
    rc::Rc,
};
use temper_qa_tool::{
    BuildState,
    faces::{DirectoryMode, FilesFace, PathKind, RemoveMode},
    testing::{Agents, Git, World},
};

const COMMIT: &str = "0123456789abcdef0123456789abcdef01234567";
const SHORT: &str = "0123456789ab";
const BINARIES: [&str; 6] =
    ["temper-box", "temper", "temper-client-journeys", "temper-engine", "temper-worker", "temper-agent"];

fn executable(world: &World, path: &Path) {
    let mut files = world.files();
    put(&mut files, path, b"executable\n");
    files.permissions(path, 0o700).expect("executable");
}

fn stage(world: &World) -> PathBuf {
    let mut files = world.files();
    let path = world.home.directory.join("stage");
    for directory in ["bin", "bundle"] {
        files.directory(&path.join(directory), DirectoryMode::Parents).expect("stage directory");
    }
    for binary in BINARIES.into_iter().chain(["fake-claude"]) {
        executable(world, &path.join("bin").join(binary));
    }
    put(&mut files, &path.join("bundle/index.html"), b"a\n");
    path
}

fn versions(world: &World, calls: &Rc<Cell<usize>>) -> Agents {
    let mut agents = world.agents();
    for binary in BINARIES {
        let calls = Rc::clone(calls);
        agents.register(binary, move |command, _| {
            calls.set(calls.get() + 1);
            assert_eq!(command.arguments, ["--version"]);
            assert!(
                command
                    .environment
                    .iter()
                    .any(|(name, value)| name.as_ref() == b"QA_VERSION_WORD" && value.as_ref() == b"captured")
            );
            Ok(output(0, format!("{binary} 0.1.0 ({SHORT})\n")))
        });
    }
    agents
}

fn keep(world: &World, agents: Agents, commit: &str, stage: &Path) -> Reply {
    invoke(
        world,
        agents,
        Git::new(world.files()),
        &["build", "keep", commit, stage.to_str().expect("stage path")],
        None,
        None,
        &[("QA_VERSION_WORD", "captured")],
    )
}

fn success(reply: &Reply, files: usize) {
    assert_eq!(reply.code, 0, "{reply:?}");
    assert_eq!(reply.stdout, format!("build {SHORT} kept: {files} files\n").as_bytes());
    assert_eq!(reply.stderr, b"");
}

fn refusal(world: &World, agents: Agents, commit: &str, stage: &Path, words: &str) {
    let mut files = world.files();
    let home = &world.home;
    let before = files.read(&home.ledger).ok();
    let reply = keep(world, agents, commit, stage);
    assert_eq!(reply.code, 1, "{reply:?}");
    assert_eq!(reply.stderr, format!("temper-qa: {words}\n").as_bytes());
    assert_eq!(reply.stdout, b"");
    assert_eq!(files.kind(stage), Some(PathKind::Directory), "staging was moved");
    assert!(!files.exists(home.builds.join(commit)), "build was published");
    assert!(!files.exists(home.builds.join(format!(".{commit}.partial"))), "staging was moved to partial");
    assert_eq!(files.read(&home.ledger).ok(), before, "ledger was changed");
}

#[test]
fn commit_form_refuses_before_build_checks_and_accepts_sha256_ids() {
    let world = World::new();
    let stage = stage(&world);
    let calls = Rc::new(Cell::new(0));
    for commit in [
        "",
        "0123456789ab",
        "0123456789abcdef0123456789abcdef0123456G",
        "0123456789ABCDEF0123456789abcdef01234567",
        "0123456789abcdef0123456789abcdef012345678",
    ] {
        refusal(&world, versions(&world, &calls), commit, &stage, &format!("{commit} is no commit id"));
    }
    assert_eq!(calls.get(), 0);
    let long = format!("{COMMIT}0123456789abcdef01234567");
    success(&keep(&world, versions(&world, &calls), &long, &stage), 8);
    assert!(world.files().exists(world.home.builds.join(long).join("manifest")));
    assert_eq!(calls.get(), BINARIES.len());
}

#[test]
fn a_kept_commit_refuses_before_inspecting_the_stage() {
    let world = World::new();
    let mut files = world.files();
    world.home.transact(&mut files, |ledger, _| ledger.keep_build(COMMIT, 1)).expect("kept record");
    let stage = world.home.directory.join("empty");
    files.directory(&stage, DirectoryMode::One).expect("empty stage");
    refusal(&world, world.agents(), COMMIT, &stage, &format!("build {SHORT} is kept already"));
}

#[test]
fn every_missing_required_path_is_named_before_versions_are_run() {
    let world = World::new();
    let mut files = world.files();
    let calls = Rc::new(Cell::new(0));
    for missing in [
        &["bin/temper-box"][..],
        &["bundle/index.html"],
        &["bin/temper-worker"],
        &["bin/temper-engine", "bin/temper-agent"],
    ] {
        let stage = stage(&world);
        for path in missing {
            files.remove(&stage.join(path), RemoveMode::File).expect("remove required file");
        }
        let words =
            missing.iter().map(|path| format!("{} lacks {path}", stage.display())).collect::<Vec<_>>().join("; ");
        refusal(&world, versions(&world, &calls), COMMIT, &stage, &words);
        assert_eq!(calls.get(), 0, "a version handler ran on an incomplete stage");
        files.remove(&stage, RemoveMode::Tree).expect("reset stage");
    }
}

#[test]
fn symlinks_under_the_stage_are_refused_without_following_them() {
    let world = World::new();
    let stage = stage(&world);
    world.files().symlink("missing", stage.join("bundle/link")).expect("link");
    refusal(
        &world,
        world.agents(),
        COMMIT,
        &stage,
        &format!("{} holds bundle/link, neither a file nor a directory", stage.display()),
    );
}

#[test]
fn each_regular_file_directly_in_bin_must_be_executable() {
    let world = World::new();
    let stage = stage(&world);
    put(&mut world.files(), &stage.join("bin/data"), b"data");
    refusal(&world, versions(&world, &Rc::new(Cell::new(0))), COMMIT, &stage, "bin/data is not executable");
}

#[test]
fn every_executable_must_print_the_clean_commit_on_its_first_line() {
    let world = World::new();
    let stage = stage(&world);
    executable(&world, &stage.join("bin/other"));
    for line in [
        "other 0.1.0 (abcdef012345)",
        "other 0.1.0 (0123456789ab-dirty)",
        "wrong first line\nother 0.1.0 (0123456789ab)",
    ] {
        let mut agents = versions(&world, &Rc::new(Cell::new(0)));
        agents.register("other", move |_, _| Ok(output(0, format!("{line}\n"))));
        let first = line.lines().next().expect("line");
        refusal(&world, agents, COMMIT, &stage, &format!("bin/other prints {first}, not {SHORT}"));
    }
}

#[test]
fn a_deleted_build_is_kept_anew_and_an_abandoned_partial_is_removed() {
    let world = World::new();
    let mut files = world.files();
    let home = &world.home;
    home.transact(&mut files, |ledger, _| {
        ledger.keep_build(COMMIT, 1)?;
        ledger.set_build_holds(COMMIT, vec![], 1)?;
        ledger.mark_build_deleted(COMMIT, 2)?;
        Ok(())
    })
    .expect("deleted through operations");
    let partial = home.builds.join(format!(".{COMMIT}.partial"));
    files.directory(&partial, DirectoryMode::Parents).expect("abandoned partial");
    put(&mut files, &partial.join("abandoned"), b"old");
    let stage = stage(&world);
    success(&keep(&world, versions(&world, &Rc::new(Cell::new(0))), COMMIT, &stage), 8);
    assert!(!files.exists(partial));
    let kept = home.builds.join(COMMIT);
    assert!(!files.exists(kept.join("bundle-record")));
    let manifest = files.read_to_string(&kept.join("manifest")).expect("manifest");
    let paths: Vec<_> = manifest.lines().skip(1).map(|line| line.split_once("  ").expect("checksum row").1).collect();
    assert_eq!(
        paths,
        [
            "bin/fake-claude",
            "bin/temper",
            "bin/temper-agent",
            "bin/temper-box",
            "bin/temper-client-journeys",
            "bin/temper-engine",
            "bin/temper-worker",
            "bundle/index.html",
        ]
    );
    assert!(!files.exists(kept.join("abandoned")));
    assert!(!files.exists(stage));
    let ledger = world.ledger();
    assert_eq!(ledger.builds.len(), 1);
    let build = &ledger.builds[0];
    assert_eq!(build.state, BuildState::Kept);
    assert!(build.kept > 2);
    assert_eq!(build.holds, None);
    assert_eq!(build.order, None);
    assert_eq!(build.deleted, None);
}
