use hearth::Form;
use hearth_builder_world::fixtures::{self, Repository};
use skein_io::kernel::Exit;

#[test]
#[expect(clippy::disallowed_methods, reason = "hearth's round: reads PATH for the real git repository fixture")]
fn a_written_repository_is_the_one_git_makes() {
    use fixtures::File;
    let files = fixtures::thin_with(&[
        File { path: "executable", body: b"#!/bin/sh\nexit 0\n", mode: 0o755 },
        File::link("link", b"a.b"),
        File { path: "a.b", body: b"before directory\n", mode: 0o644 },
        File { path: "a/c", body: b"last in directory\n", mode: 0o644 },
    ]);
    let path = std::env::var_os("PATH").expect("git is required on PATH");
    let real = Repository::make_on(&path, "git-written", &files);
    let written = Repository::make("kit-written", &files);
    assert_eq!(written.git(&["rev-parse", "HEAD"]), real.git(&["rev-parse", "HEAD"]));
    drop(written.git(&["fsck", "--strict"]));
    let changed = [File { path: "a.b", body: b"changed\n", mode: 0o644 }];
    drop(real.next(&changed, &["a/c"]));
    drop(real.git(&["add", "-A"]));
    drop(real.git(&["commit", "--allow-empty", "--amend", "-q", "-m", "next"]));
    drop(written.next(&changed, &["a/c"]));
    assert_eq!(written.git(&["rev-parse", "HEAD"]), real.git(&["rev-parse", "HEAD"]));
    drop(written.git(&["fsck", "--strict"]));
}

#[test]
fn hearth_render_prints_the_thin_repositorys_spec_through_real_git() {
    let repository = Repository::make("thin", fixtures::THIN);
    let rendered = fixtures::render(&repository, &[], Form::Text);
    assert_eq!(
        String::from_utf8_lossy(&rendered.stdout),
        fixtures::THIN_TEXT,
        "hearth/docs/design/domain/render/README.md 6.2's lines"
    );
    assert_eq!(rendered.exit, Exit::Code(0), "hearth/docs/design/client/README.md 3.3: done");
}
#[test]
fn a_scratch_repository_without_git_on_the_path_fails_by_name_not_by_skip() {
    let scratch = skein_scratch::Scratch::new("without-git");
    let error = Repository::try_make_on(scratch.path().as_os_str(), "no-git", fixtures::THIN)
        .expect_err("missing git refuses the scratch repository");
    let cause = error.strip_prefix("git is required on the supplied PATH: ").expect("the refusal names git");
    assert!(!cause.is_empty(), "the refusal includes the spawn error: {error}");
}

const CACHED: &[(&str, &[u8])] = &[
    ("775324fcea7f2cdcbf11df25550e362669caceb6e69c8370374d2d436bb63123", b"Welcome to x1.\n"),
    ("f487088ec8b18ff569dfede6f12e5fa91983f9f6dd2f3e0b6cee4349a231e745", b"#!/bin/sh\nexec cat /etc/motd\n"),
];
#[test]
fn static_files_enter_the_cache_by_sha256_and_no_temporary_stays() {
    use std::{fs, os::unix::fs::PermissionsExt};
    let repository = Repository::make("cache", fixtures::THIN);
    hearth::umask(0o077);
    let rendered = fixtures::render(&repository, &[], Form::Text);
    assert_eq!(rendered.exit, Exit::Code(0));
    let cache = repository.scratch().path().join("store/cache");
    assert_eq!(fs::metadata(&cache).expect("cache").permissions().mode() & 0o7777, 0o700);
    let mut names: Vec<_> =
        fs::read_dir(&cache).expect("cache listing").map(|entry| entry.expect("cache entry").file_name()).collect();
    names.sort();
    assert_eq!(
        names,
        [
            "42ecbc10b6791c87aa0ca833e522946a40a7eece2297a745c5c10cea07baea96",
            "42ecbc10b6791c87aa0ca833e522946a40a7eece2297a745c5c10cea07baea96.record",
            "775324fcea7f2cdcbf11df25550e362669caceb6e69c8370374d2d436bb63123",
            "775324fcea7f2cdcbf11df25550e362669caceb6e69c8370374d2d436bb63123.record",
            "f487088ec8b18ff569dfede6f12e5fa91983f9f6dd2f3e0b6cee4349a231e745",
            "f487088ec8b18ff569dfede6f12e5fa91983f9f6dd2f3e0b6cee4349a231e745.record"
        ]
    );
    for &(name, body) in CACHED {
        assert_eq!(fs::read(cache.join(name)).expect("cached bytes"), body, "{name}");
        assert_eq!(fs::metadata(cache.join(name)).expect("cache file").permissions().mode() & 0o7777, 0o644, "{name}");
    }
}

#[test]
fn a_store_inside_a_work_tree_diffs_as_outside_any_repository() {
    let rendered = fixtures::diff_beneath_work_tree("diff-config", false);
    assert_eq!(
        fixtures::lines_starting(&rendered.stdout, &["               @@"]),
        "               @@ -1,5 +1,5 @@\n               @@ -9,7 +9,7 @@ line 8\n"
    );
}
#[test]
fn a_work_trees_gitattributes_do_not_make_a_file_binary() {
    let rendered = fixtures::diff_beneath_work_tree("diff-attributes", true);
    assert_eq!(
        fixtures::lines_starting(&rendered.stdout, &["               @@"]),
        "               @@ -1,5 +1,5 @@\n               @@ -9,7 +9,7 @@ line 8\n"
    );
}

#[test]
fn a_tree_git_cannot_read_prints_git_s_tail_and_exits_75() {
    let repository = Repository::make("broken-tree", fixtures::THIN);
    repository.remove_object(&repository.git(&["rev-parse", "HEAD^{tree}"]));
    let rendered = fixtures::render(&repository, &[], Form::Text);
    assert_eq!(String::from_utf8_lossy(&rendered.stdout), "git ls-tree exited 128\n  fatal: not a tree object\n");
    assert_eq!(rendered.exit, Exit::Code(75));
}

mod files;

mod boot;

mod accounts;

mod additions;
mod data;
mod scripts;

mod software;

mod collisions;
mod ids;
