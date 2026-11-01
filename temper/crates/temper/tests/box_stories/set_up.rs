//! An unfinished owned forge starts over by removal (draft/playground-reset.md, 6.4).
use super::{document, group, restart_until, rows};
use crate::{
    first_run_directories,
    support::{
        self,
        ctl::{Call, Step},
        forge_land,
    },
};
use std::{
    fs,
    os::unix::fs::MetadataExt,
    path::{Path, PathBuf},
};

#[test]
fn a_set_up_killed_at_fetch_is_started_over_from_the_cache() {
    started_over("fetch", |line| step_number(line, "Starting fetch "));
}

#[test]
fn a_set_up_killed_at_prepare_is_started_over_from_the_cache() {
    started_over("prepare", |line| step_number(line, "Starting prepare "));
}

#[test]
fn a_set_up_killed_at_run_is_started_over_from_the_cache() {
    started_over("run", |line| step_number(line, "Starting forgejo "));
}

#[test]
fn a_set_up_killed_after_started_is_started_over_from_the_cache() {
    started_over("started", |line| line.split("Forge ").nth(1)?.strip_suffix(" (Forgejo): started")?.parse().ok());
}

fn step_number(line: &str, prefix: &str) -> Option<u64> {
    line.split(prefix).nth(1)?.parse().ok()
}

// Inode and mtime complement the byte check: an identical replacement is a
// download too. Metadata belongs only to the pinned public executables.
fn identity(path: &Path) -> (u64, i64, i64) {
    let metadata = fs::metadata(path).expect("cached executable metadata");
    (metadata.ino(), metadata.mtime(), metadata.mtime_nsec())
}

fn cache_unchanged(cached: &[PathBuf; 2], sources: &[PathBuf; 2], identities: &[(u64, i64, i64); 2]) {
    for ((path, source), before) in cached.iter().zip(sources).zip(identities) {
        forge_land::assert_cached_binary_unchanged(path, source);
        assert_eq!(&identity(path), before, "no new download or replaced cache file");
        assert!(!path.with_extension("part").exists(), "no partial download");
    }
}

fn started_over(name: &str, number_at: impl Fn(&str) -> Option<u64>) {
    let scratch = support::Scratch::new(&format!("set-up-cut-{name}"));
    let (paths, flags) = first_run_directories(&scratch);
    let (cached, sources) = forge_land::cache_forgejo_binaries(&paths[2]);
    let identities = cached.each_ref().map(|path| identity(path));
    let cut = support::cut::at_line(
        scratch.path(),
        flags.clone(),
        scratch.path(),
        vec![Step::Run(Call::sign_in()), Step::Run(Call::new(&["forge", "setup", "--key", "cut-setup"]))],
        |line| number_at(line).is_some(),
    );
    let number = number_at(&cut.line).expect("cut names its forge");
    eprintln!("{name}: killed at {}", cut.line);
    let old_group = group(&paths[1], number);
    assert_eq!(old_group["setup"]["forge"], number);
    assert_eq!(old_group["playground"], false);
    let old_directory = paths[1].join(old_group["setup"]["directory"].as_str().expect("owned directory"));
    cache_unchanged(&cached, &sources, &identities);

    let data = paths[1].clone();
    let remove_data = data.clone();
    let remove_directory = old_directory.clone();
    let remove_cached = cached.clone();
    let remove_sources = sources.clone();
    let steps = vec![
        Step::Then(Box::new(move |calls| {
            let forges = rows(&calls[1]);
            assert_eq!(forges.len(), 1, "only the cut forge is kept");
            let forge = &forges[0];
            assert_eq!(forge["number"], number);
            assert_eq!(forge["finished"], false, "the cut forge is unfinished: {forge}");
            Call::new(&["forge", "remove", &number.to_string(), "--key", "start-over"])
        })),
        Step::Stderr(b" (Forgejo): removed\n"),
        Step::Until(Call::new(&["forge", "list", "--json"]), b"\"forges\":[]"),
        Step::Then(Box::new(move |calls| {
            assert_eq!(calls[2].stdout, b"removing\n", "RemoveForge answered");
            assert!(rows(calls.last().expect("empty forges read")).is_empty());
            for path in [
                remove_data.join(format!("box/groups/{number}")),
                remove_data.join(format!("box/started/{number}")),
                remove_directory,
            ] {
                assert!(!path.exists(), "removed {}", path.display());
            }
            cache_unchanged(&remove_cached, &remove_sources, &identities);
            Call::new(&["forge", "setup", "--key", "fresh-setup"])
        })),
        Step::Run(Call::new(&["forge", "list", "--json"])),
    ];
    let seen = restart_until(scratch.path(), flags, scratch.path(), b"\"finished\":false", steps, |_| {
        cache_unchanged(&cached, &sources, &identities);
    });
    let forges = rows(seen.calls.last().expect("fresh forge read"));
    assert_eq!(forges.len(), 1);
    let forge = &forges[0];
    let fresh = forge["number"].as_u64().expect("fresh forge number");
    assert!(fresh > number, "removed forge numbers are never reused");
    assert_eq!(forge["state"], "connected");
    assert_eq!(forge["finished"], true, "the first repository was adopted");
    assert_eq!(forge["first"]["state"], "made");
    assert_eq!(forge["first"]["org"], "work");
    assert_eq!(forge["first"]["repository"], "hello");
    assert!(data.join(format!("box/started/{fresh}")).is_file());
    let fresh_group = group(&data, fresh);
    assert_eq!(fresh_group["setup"]["forge"], fresh);
    assert_eq!(fresh_group["setup"]["mode"]["owned"]["forgejo"], "fetched");
    assert_eq!(fresh_group["setup"]["mode"]["owned"]["runner"]["binary"], "fetched");
    // Init's template and both incarnations use the fetched cache paths.
    let config = document(&paths[0]);
    assert_eq!(config["application"]["owned"]["mode"]["owned"]["forgejo"], "fetched");
    assert_eq!(config["application"]["owned"]["mode"]["owned"]["runner"]["binary"], "fetched");
    let stderr = String::from_utf8_lossy(&seen.stderr);
    assert!(stderr.contains(&format!("Forge {fresh} (Forgejo): started")), "{stderr}");
    cache_unchanged(&cached, &sources, &identities);
    scratch.remove();
}
