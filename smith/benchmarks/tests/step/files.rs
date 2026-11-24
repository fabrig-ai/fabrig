//! The benchmark filesystem face and the harness decisions it drives offline.

use std::io::ErrorKind;
use std::path::Path;

use skein_fake_machine::{How, Is, Item, Refusal as MachineRefusal};
use smith_bench::arm::{ArmBinary, BuiltArm, arm_name, binary_digest_with, read_build_with};
use smith_bench::files::{Facts, Files, Kind};
use smith_bench::formats::read_document_with;
use smith_bench::guard::Guard;
use smith_bench::isolate::{Private, working_directory_with};
use smith_bench::testing::task::parse_task;
use smith_bench::{
    Design, Fault, FaultKind, RESULT_VERSION, RunSummary, SUMMARY_VERSION, Task, read_configuration_with,
    read_summary_with, write_summary_with,
};
use smith_bench_fixtures::files::{Operation, Read, Tree, Write};

#[test]
fn facts_distinguish_a_link_from_its_target() {
    const OWNER: u32 = 1234;
    const BYTES: &[u8] = b"data";
    const TARGET: &[u8] = b"data-file";
    let tree = Tree::new(&[
        Item::file(TARGET, BYTES).owner(OWNER).mode(0o740),
        Item::link(b"shortcut", TARGET).owner(OWNER).mode(0o755),
    ]);
    let shortcut = Path::new("/shortcut");
    assert_eq!(
        tree.symlink_metadata(shortcut).expect("link facts"),
        Facts { kind: Kind::Link, mode: 0o755, size: skein_lib::len_u64(TARGET.len()), owner: OWNER },
    );
    assert_eq!(
        tree.metadata(shortcut).expect("target facts"),
        Facts { kind: Kind::File, mode: 0o740, size: skein_lib::len_u64(BYTES.len()), owner: OWNER },
    );
}

#[test]
fn canonicalization_resolves_links_before_taking_parent_steps() {
    let tree = Tree::new(&[
        Item::directory(b"workspace"),
        Item::directory(b"workspace/deep"),
        Item::directory(b"alternate"),
        Item::link(b"alternate/relative", b"../workspace/deep"),
        Item::link(b"absolute", b"/workspace/deep"),
    ]);
    for (input, output) in [
        ("/alternate/relative", "/workspace/deep"),
        ("/absolute", "/workspace/deep"),
        ("/alternate/relative/..", "/workspace"),
    ] {
        assert_eq!(tree.canonicalize(Path::new(input)).expect("resolved link"), Path::new(output), "{input}");
    }
}

#[test]
fn impossible_lookups_preserve_their_io_error_kinds() {
    let tree = Tree::new(&[
        Item::link(b"dangling", b"/absent"),
        Item::file(b"file", b"file"),
        Item::directory(b"sealed").mode(0o600),
        Item::file(b"sealed/log", b"log"),
    ]);
    for (path, expected) in [
        ("/dangling", ErrorKind::NotFound),
        ("/file/child", ErrorKind::NotADirectory),
        ("/file/.", ErrorKind::NotADirectory),
        ("/sealed/log", ErrorKind::PermissionDenied),
    ] {
        assert_eq!(tree.canonicalize(Path::new(path)).expect_err("cannot resolve").kind(), expected, "{path}");
    }
}

#[test]
fn canonicalization_refuses_a_link_cycle_as_an_error() {
    let tree = Tree::new(&[Item::link(b"first", b"second"), Item::link(b"second", b"/first")]);
    let error = tree.canonicalize(Path::new("/first")).expect_err("cannot follow a cycle");
    assert_ne!(error.kind(), ErrorKind::NotFound, "a cycle is not a missing target");
}

#[test]
fn canonicalization_allows_forty_links_but_not_more() {
    const MOST_LINKS: u32 = 40; // The host's maximum number of follows in one lookup.
    let mut items = vec![Item::file(b"0", b"target")];
    for number in 1..=MOST_LINKS + 1 {
        items.push(Item::link(number.to_string().as_bytes(), format!("/{}", number - 1).as_bytes()));
    }
    let tree = Tree::new(&items);
    for (number, expected) in [(MOST_LINKS, Some(Path::new("/0"))), (MOST_LINKS + 1, None)] {
        let path = format!("/{number}");
        assert_eq!(tree.canonicalize(Path::new(&path)).ok().as_deref(), expected, "{path}");
    }
}

#[test]
fn directory_children_keep_the_listed_spelling_and_kind() {
    let tree = Tree::new(&[
        Item::directory(b"workspace"),
        Item::file(b"workspace/file", b"data"),
        Item::fifo(b"workspace/fifo"),
        Item::link(b"workspace/link", b"file"),
        Item::directory(b"workspace/subdirectory"),
        Item::link(b"alias", b"/workspace"),
    ]);
    let mut children: Vec<_> = tree
        .read_dir(Path::new("/alias"))
        .expect("open alias directory")
        .map(|child| {
            let child = child.expect("child");
            (child.path(), child.file_type().expect("no-follow kind"))
        })
        .collect();
    children.sort_by(|left, right| left.0.cmp(&right.0));
    assert_eq!(
        children,
        [
            ("/alias/fifo", Kind::Other),
            ("/alias/file", Kind::File),
            ("/alias/link", Kind::Link),
            ("/alias/subdirectory", Kind::Directory),
        ]
        .map(|(path, kind)| (path.into(), kind))
    );
}

#[test]
fn prefix_reads_distinguish_the_bound_and_one_past() {
    const BOUND: u64 = 3; // b"a\nc", before the extra d.
    let tree = Tree::new(&[Item::file(b"at-bound", b"a\nc"), Item::file(b"over-bound", b"a\ncd")]);
    for (path, expected) in [("/at-bound", b"a\nc".as_slice()), ("/over-bound", b"a\ncd".as_slice())] {
        assert_eq!(tree.read_bounded(Path::new(path), BOUND + 1).expect("bounded prefix"), expected, "{path}");
    }
    assert_eq!(tree.read_bounded(Path::new("/over-bound"), BOUND).expect("strict bound"), b"a\nc");
    assert_eq!(tree.read_until(Path::new("/over-bound"), b'\n', BOUND).expect("first line"), b"a\n");
}

#[test]
fn whole_reads_and_sha256_observe_the_same_bytes() {
    const BYTES: &[u8] = b"a\ncd";
    const SHA256: &str = "5dd6adc362b72583a3499941e6ee35f059c837f62e99d2d14967120d763afc5c";
    let tree = Tree::new(&[Item::file(b"file", BYTES)]);
    assert_eq!(tree.read(Path::new("/file")).expect("whole file"), BYTES);
    assert_eq!(tree.sha256(Path::new("/file")).expect("hash"), SHA256);
}

fn read_back(tree: &Tree, relative: &[u8]) -> Result<(Is, u32, Vec<u8>), MachineRefusal> {
    let root = tree.root();
    let mut machine = tree.machine();
    let file = machine.open(root, relative, How::PathNoFollow)?;
    let facts = machine.stat(file);
    let bytes = if facts.is == Is::File { machine.read(file, 0, u32::MAX) } else { Ok(Vec::new()) };
    machine.close(file);
    Ok((facts.is, facts.mode, bytes?))
}

#[test]
fn writes_are_ordered_and_change_the_machine_tree() {
    use Operation::{
        Copy, CreateDir, CreateDirAll, CreateNew, RemoveDirAll, RemoveFile, Rename, SetPermissions, Write as Replace,
    };

    let tree = Tree::new(&[
        Item::file(b"source", b"longer than new").mode(0o600),
        Item::file(b"obsolete", b"remove me"),
        Item::file(b"outside", b"keep"),
        Item::directory(b"gone"),
        Item::directory(b"gone/deeper"),
        Item::link(b"gone/deeper/outside", b"/outside"),
    ]);
    let files: &dyn Files = &tree;
    let source = Path::new("/source");
    let copied = Path::new("/parents/child/copy");
    files.create_dir(Path::new("/single"), 0o700).expect("make one directory");
    files.create_dir_all(Path::new("/parents/child"), 0o750).expect("make missing parents");
    files.create_new(Path::new("/single/created"), 0o600, b"created", true).expect("make and sync a new file");
    files.write(source, b"new").expect("replace the contents");
    files.set_permissions(source, 0o640).expect("change the mode");
    let copied_bytes = files.copy(source, copied).expect("copy bytes and mode");
    files.rename(copied, Path::new("/published")).expect("move copy");
    files.remove_file(Path::new("/obsolete")).expect("remove only the named file");
    files.remove_dir_all(Path::new("/gone")).expect("remove directory and its contents without following the link");

    assert_eq!(
        tree.writes(),
        [
            (CreateDir, "/single", None),
            (CreateDirAll, "/parents/child", None),
            (CreateNew, "/single/created", None),
            (Replace, "/source", None),
            (SetPermissions, "/source", None),
            (Copy, "/parents/child/copy", Some("/source")),
            (Rename, "/published", Some("/parents/child/copy")),
            (RemoveFile, "/obsolete", None),
            (RemoveDirAll, "/gone", None),
        ]
        .map(|(operation, path, source)| Write {
            operation,
            path: path.into(),
            source: source.map(Into::into)
        })
    );
    for (name, kind, mode, bytes) in [
        (b"single".as_slice(), Is::Directory, 0o700, b"".as_slice()),
        (b"parents", Is::Directory, 0o750, b""),
        (b"parents/child", Is::Directory, 0o750, b""),
        (b"single/created", Is::File, 0o600, b"created"),
        (b"source", Is::File, 0o640, b"new"),
        (b"published", Is::File, 0o640, b"new"),
        (b"outside", Is::File, 0o644, b"keep"),
    ] {
        assert_eq!(read_back(&tree, name), Ok((kind, mode, bytes.to_vec())));
    }
    for name in [b"obsolete".as_slice(), b"parents/child/copy", b"gone"] {
        assert_eq!(read_back(&tree, name), Err(MachineRefusal::NotFound));
    }
    assert_eq!(copied_bytes, skein_lib::len_u64(b"new".len()));
}

#[test]
fn a_scripted_read_refusal_names_one_operation_and_path() {
    let tree = Tree::new(&[Item::file(b"chosen", b"a"), Item::file(b"other", b"b")]);
    tree.refuse(Operation::Read, "/chosen", ErrorKind::PermissionDenied);
    assert_eq!(tree.read(Path::new("/chosen")).expect_err("scripted refusal").kind(), ErrorKind::PermissionDenied);
    assert_eq!(tree.read_to_string(Path::new("/chosen")).expect("different operation"), "a");
    assert_eq!(tree.read(Path::new("/other")).expect("different path"), b"b");
}

#[test]
fn never_read_paths_record_alias_reads_and_refused_lookups() {
    let tree = Tree::new(&[Item::file(b"protected", b"synthetic"), Item::link(b"alias", b"/protected")]);
    tree.never_read("/protected");
    tree.never_read("/missing");
    let _bytes = tree.read(Path::new("/alias")).expect("read through alias");
    let absent = tree.try_exists(Path::new("/missing")).expect("missing target is not an error");
    assert!(!absent);
    assert_eq!(
        tree.unexpected_reads(),
        [
            Read { operation: Operation::Read, path: "/protected".into() },
            Read { operation: Operation::TryExists, path: "/missing".into() },
        ]
    );
}

fn guard(tree: &Tree, tool_homes: &[std::path::PathBuf]) -> Guard {
    Guard::new_with(tree, Path::new("/user"), Path::new("/config"), Path::new("/state"), tool_homes).expect("guard")
}

#[test]
fn the_guard_refuses_an_alias_into_a_protected_directory() {
    let tree = Tree::new(&[
        Item::directory(b"protected"),
        Item::link(b"tool-home", b"/protected"),
        Item::link(b"alias", b"/protected"),
    ]);
    let guard = guard(&tree, &["/tool-home".into()]);
    tree.remove_file(Path::new("/tool-home")).expect("tool link removed after constructing guard");
    let error = guard.write_path_with(&tree, Path::new("/alias/new/record")).expect_err("alias into a protected home");
    assert_eq!(error, Fault::new(FaultKind::Input, "write refused inside the user's tool directories"));
}

#[test]
fn the_guard_resolves_missing_suffixes_at_existing_ancestors() {
    let tree = Tree::new(&[Item::directory(b"workspace"), Item::link(b"alias", b"workspace")]);
    let guard = guard(&tree, &[]);
    assert_eq!(
        guard.write_path_with(&tree, Path::new("/alias/output/missing")).expect("resolve existing link"),
        Path::new("/workspace/output/missing"),
    );
}

#[test]
fn the_guard_refuses_relative_and_parent_destinations() {
    let tree = Tree::new(&[]);
    let guard = guard(&tree, &[]);
    for path in ["relative", "/scratch/../output"] {
        let error = guard.write_path_with(&tree, Path::new(path)).expect_err("unsafe destination spelling");
        assert_eq!(error, Fault::new(FaultKind::Input, "write destination must be absolute and contain no '..'"));
    }
}

fn workspace(children: impl IntoIterator<Item = Item>) -> (Tree, Private) {
    let items: Vec<_> =
        [Item::directory(b"attempt"), Item::directory(b"attempt/workspace")].into_iter().chain(children).collect();
    let tree = Tree::new(&items);
    let private = Private {
        root: "/attempt".into(),
        workspace: "/attempt/workspace".into(),
        home: "/attempt/home".into(),
        temporary: "/attempt/tmp".into(),
        cargo_home: "/attempt/cargo".into(),
        target: "/attempt/target".into(),
        warm_cargo: "/attempt/warm-cargo".into(),
        warm_target: "/attempt/warm-target".into(),
        config: "/attempt/config".into(),
        state: "/attempt/state".into(),
    };
    (tree, private)
}

fn task_at(path: Option<&str>) -> Task {
    let mut task =
        parse_task(Path::new("/task.toml"), include_str!("../recorded/empty-seed-probe.toml")).expect("task");
    task.working_directory = path.map(Into::into);
    task
}

#[test]
fn working_directories_stay_inside_the_private_workspace() {
    let (tree, private) =
        workspace([Item::directory(b"attempt/workspace/project"), Item::link(b"attempt/workspace/inside", b"project")]);
    let guard = guard(&tree, &[]);
    for (path, expected) in [(None, "/attempt/workspace"), (Some("inside"), "/attempt/workspace/project")] {
        assert_eq!(
            working_directory_with(&tree, &guard, &private, &task_at(path)).expect("working directory"),
            Path::new(expected)
        );
    }
}

#[test]
fn unsafe_working_directories_keep_their_refusal_words() {
    const TRAVERSAL: &str = "task working_directory must be a seed-relative path without traversal";
    const OUTSIDE_OR_NOT_DIR: &str = "task working_directory must be an existing directory inside the private seed";
    let (tree, private) = workspace([
        Item::file(b"attempt/workspace/not-directory", b"file"),
        Item::directory(b"outside"),
        Item::link(b"attempt/workspace/escape", b"/outside"),
    ]);
    let guard = guard(&tree, &[]);
    for (path, kind, detail) in [
        ("../outside", FaultKind::Input, TRAVERSAL),
        ("/outside", FaultKind::Input, TRAVERSAL),
        ("missing", FaultKind::Io, "No such file or directory (os error 2)"),
        ("not-directory", FaultKind::Input, OUTSIDE_OR_NOT_DIR),
        ("escape", FaultKind::Input, OUTSIDE_OR_NOT_DIR),
    ] {
        let error =
            working_directory_with(&tree, &guard, &private, &task_at(Some(path))).expect_err("bad working directory");
        assert_eq!(error, Fault::new(kind, detail), "{path}");
    }
}

const PIN: &str = "/agents/current.pin.toml";
const PIN_BYTES: &[u8] = b"name = \"offline\"\nagent = \"smith\"\nprovider = \"anthropic\"\nversion = \"1\"\nconfiguration = \"settings.toml\"\n";
const CONFIGURATION: &[u8] = b"agents/settings.toml";
const CONFIGURATION_BYTES: &[u8] = b"[model]\nname = \"offline\"\n";

fn pin_tree(configuration: Item) -> Tree {
    Tree::new(&[Item::directory(b"agents"), Item::file(b"agents/current.pin.toml", PIN_BYTES), configuration])
}

#[test]
fn a_pin_digests_its_exact_metadata_and_configuration_bytes() {
    let tree = pin_tree(Item::file(CONFIGURATION, CONFIGURATION_BYTES));
    let pinned = read_configuration_with(&tree, Path::new(PIN)).expect("pinned config");
    assert_eq!(pinned.pin.configuration, Path::new("settings.toml"));
    assert_eq!(pinned.sha256, "4be008ed90b9bea49c078345e8ccf4163dd9d543847d788eca64ba42c4eef283");
}

#[test]
fn a_pin_refuses_links_and_other_non_regular_configurations() {
    for configuration in [Item::link(CONFIGURATION, b"current.pin.toml"), Item::directory(CONFIGURATION)] {
        let tree = pin_tree(configuration);
        let error = read_configuration_with(&tree, Path::new(PIN)).expect_err("not a regular configuration file");
        assert_eq!(error.key, "configuration");
        assert_eq!(error.reason, "expected a committed regular configuration file");
    }
}

const BINARY: &[u8] = b"synthetic executable";
const BINARY_SHA256: &str = "eb4ffe43c9edcf310dc3513998df4fa1700fc6570c6c46e32dd3af9e028fe720";

#[test]
fn a_binary_digest_hashes_the_regular_files_bytes() {
    let tree = Tree::new(&[Item::file(b"smith", BINARY).mode(0o700)]);
    assert_eq!(binary_digest_with(&tree, Path::new("/smith")).expect("binary digest"), BINARY_SHA256);
}

#[test]
fn a_binary_digest_refuses_links_and_directories() {
    let tree = Tree::new(&[Item::file(b"smith", BINARY), Item::link(b"alias", b"smith"), Item::directory(b"dir")]);
    for path in ["/alias", "/dir"] {
        let error = binary_digest_with(&tree, Path::new(path)).expect_err("not a regular executable");
        assert_eq!(error, Fault::new(FaultKind::Input, "arm executable must be a regular file"));
    }
}

#[test]
fn a_built_arm_checks_every_binary_in_its_inventory() {
    let arm = BuiltArm {
        version: 1,
        commit: "a".repeat(40),
        sha256: BINARY_SHA256.into(),
        toolchain: "offline".into(),
        binaries: ["smith", "helper"]
            .into_iter()
            .map(|name| ArmBinary { name: name.into(), sha256: BINARY_SHA256.into() })
            .collect(),
    };
    let directory = arm_name(&arm.commit, &arm.sha256).expect("arm name");
    let program = format!("{directory}/smith");
    let tree = Tree::new(&[
        Item::directory(directory.as_bytes()),
        Item::file(format!("{directory}/arm.json").as_bytes(), &serde_json::to_vec(&arm).expect("manifest")),
        Item::file(program.as_bytes(), BINARY),
        Item::file(format!("{directory}/helper").as_bytes(), b"different bytes"),
    ]);
    let fault = read_build_with(&tree, &Path::new("/").join(program)).expect_err("helper's digest differs");
    assert_eq!(fault, Fault::new(FaultKind::Mismatch, "existing arm binary digest mismatch"));
}

#[test]
fn a_document_is_decoded_from_the_supplied_filesystem() {
    let tree = Tree::new(&[Item::file(b"document.toml", b"name = 'offline'\n")]);
    let table: toml::Table = read_document_with(&tree, Path::new("/document.toml")).expect("laid TOML");
    assert_eq!(table["name"].as_str(), Some("offline"));
}

#[test]
fn summary_publication_does_not_overwrite_a_published_summary() {
    let tree = Tree::new(&[Item::directory(b"summaries")]);
    let path = Path::new("/summaries/offline.json");
    let summary = RunSummary {
        version: SUMMARY_VERSION,
        result_version: RESULT_VERSION,
        priced_at: "offline".into(),
        run: "first".into(),
        suite: "sample".into(),
        tier: "offline".into(),
        design: Design::Single,
        seed: 0,
        date: "2026-01-01".into(),
        attempts: Vec::new(),
        parity: None,
        change: None,
        groups: Vec::new(),
        comparisons: Vec::new(),
        audits: Vec::new(),
        drift_notices: Vec::new(),
    };
    let different = RunSummary { run: "second".into(), ..summary.clone() };
    write_summary_with(&tree, path, &summary).expect("exclusive publication");
    let refusal = write_summary_with(&tree, path, &different).expect_err("a summary at the same path is refused");
    assert_eq!(refusal.kind, FaultKind::Io);
    assert_eq!(read_summary_with(&tree, path).expect("summary in the tree"), summary);
}
