//! A scratch repository of the test `IaC` and a run of the built binary (hearth/docs/design/domain/testing.md, 2.9).
use skein_scratch::Scratch;
use std::{
    ffi::OsStr,
    fs,
    os::unix::{
        ffi::OsStrExt,
        fs::{DirBuilderExt, PermissionsExt},
    },
    path::{Path, PathBuf},
    process::Command,
};

mod binary;
pub use binary::{Ended, binary};
mod resolution;
pub use resolution::*;
pub mod reserved;
pub use reserved::{MOTD_DIFF_TEXT, MOTD_KIND};

#[derive(Clone, Copy, Debug)]
pub struct File {
    pub path: &'static str,
    pub body: &'static [u8],
    pub mode: u32,
}
pub const DEBIAN: &[u8] = br#"{"debian":{"suite":"forky","architecture":"amd64","snapshot":{"archive":"20261001T030405Z"},"pockets":{"updates":false,"security":false}}}"#;
pub const THIN: &[File] = &[
    reserved::EMPTY_HEARTH,
    reserved::FLEET_JSON,
    reserved::PROD_ENV,
    File { path: "hosts/x1/x1.json", body: b"{\"motd\":{}}", mode: 0o644 },
    File { path: "hosts/x1/x1-prod.json", body: reserved::METAL_EFI, mode: 0o644 },
    File { path: "tasks/motd/task.json", body: b"{}", mode: 0o644 },
    File { path: "tasks/motd/tree/etc/motd", body: b"Welcome to x1.\n", mode: 0o644 },
    File { path: "tasks/motd/tree/usr/local/bin/motd", body: b"#!/bin/sh\nexec cat /etc/motd\n", mode: 0o755 },
];
#[derive(Debug)]
pub struct Repository {
    scratch: Scratch,
    work_tree: PathBuf,
}
impl Repository {
    #[must_use]
    pub fn make(label: &str, files: &[File]) -> Self {
        let path = reserved::git_path();
        Self::make_on(&path, label, files)
    }
    #[must_use]
    pub fn make_on(path: &OsStr, label: &str, files: &[File]) -> Self {
        let repository = Self::empty(label);
        git(&repository.work_tree, path, &["init", "--quiet", "--initial-branch=main", "--template="]);
        repository.write(files);
        git(&repository.work_tree, path, &["add", "-A"]);
        git(&repository.work_tree, path, &["commit", "--allow-empty", "-q", "-m", "thin"]);
        repository
    }
    fn write(&self, files: &[File]) {
        for file in files {
            let target = self.work_tree.join(file.path);
            fs::create_dir_all(target.parent().expect("a parent")).expect("fixture directories");
            if file.mode == 0o120_000 {
                if fs::symlink_metadata(&target).is_ok() {
                    fs::remove_file(&target).expect("replace link");
                }
                std::os::unix::fs::symlink(OsStr::from_bytes(file.body), &target).expect("fixture link");
            } else {
                fs::write(&target, file.body).expect("fixture file");
                fs::set_permissions(target, fs::Permissions::from_mode(file.mode)).expect("fixture mode");
            }
        }
    }
    #[must_use]
    pub fn head(&self) -> String {
        let path = reserved::git_path();
        String::from_utf8(git_output(&self.work_tree, &path, &["rev-parse", "HEAD"]))
            .expect("commit id")
            .trim()
            .to_owned()
    }
    #[must_use]
    pub fn next(&self, files: &[File], removed: &[&str]) -> String {
        self.write(files);
        for path in removed {
            fs::remove_file(self.work_tree.join(path)).expect("removed file");
        }
        let path = reserved::git_path();
        git(&self.work_tree, &path, &["add", "-A"]);
        git(&self.work_tree, &path, &["commit", "--allow-empty", "-q", "-m", "next"]);
        self.head()
    }
    #[must_use]
    pub fn empty(label: &str) -> Self {
        let scratch = Scratch::new(label);
        let work_tree = scratch.path().join("repo");
        fs::create_dir(&work_tree).expect("empty repository directory");
        Self { scratch, work_tree }
    }
    #[must_use]
    pub fn git(&self, args: &[&str]) -> String {
        let path = reserved::git_path();
        String::from_utf8(git_output(&self.work_tree, &path, args)).expect("git text").trim().to_owned()
    }
    pub fn remove_object(&self, oid: &str) {
        let (directory, name) = oid.split_at(2);
        fs::remove_file(self.work_tree.join(".git/objects").join(directory).join(name)).expect("loose object");
    }
    #[must_use]
    pub fn work_tree(&self) -> &Path {
        &self.work_tree
    }
    #[must_use]
    pub fn scratch(&self) -> &Scratch {
        &self.scratch
    }
}
pub fn git(directory: &Path, path: &OsStr, args: &[&str]) {
    drop(git_output(directory, path, args));
}
#[expect(clippy::must_use_candidate, reason = "fixture git commands also run for their side effects")]
pub fn git_output(directory: &Path, path: &OsStr, args: &[&str]) -> Vec<u8> {
    let result = Command::new("git")
        .current_dir(directory)
        .args(args)
        .env_clear()
        .env("PATH", path)
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .env("GIT_CONFIG_COUNT", "1")
        .env("GIT_CONFIG_KEY_0", "maintenance.auto")
        .env("GIT_CONFIG_VALUE_0", "false")
        .env("HOME", "/dev/null")
        .env("GIT_AUTHOR_NAME", "hearth")
        .env("GIT_AUTHOR_EMAIL", "hearth@hearth.invalid")
        .env("GIT_AUTHOR_DATE", "1700000000 +0000")
        .env("GIT_COMMITTER_NAME", "hearth")
        .env("GIT_COMMITTER_EMAIL", "hearth@hearth.invalid")
        .env("GIT_COMMITTER_DATE", "1700000000 +0000")
        .output()
        .expect("git is required on the supplied PATH");
    assert!(result.status.success(), "git {args:?} failed: {}", String::from_utf8_lossy(&result.stderr));
    result.stdout
}

pub const THIN_TEXT: &str = "target       x1-prod   base x1   environment prod\nplacement    metal efi 7th-gen\nspec         23656fdf46b7\nboot         part bc6cab417252\n  mutable    false\npackages     part 64b1b99b813a   resolution 4ee22520f418 needed\n  sources    forky amd64 main 20261001T030405Z\n  variant    apt\n  microcode  generic\n  apt        install_recommends false\nsoftware     part 656d135b92c3\nconfig       part 369415fd653c\n  /etc/motd                file 0644 root:root 775324fcea7f\n  /usr/local/bin/motd      file 0755 root:root f487088ec8b1\ndata         part 2d4765200bab\nsecrets      part ac870b5c968e\nlive         part 6f1b7fda0c21\nexports      part 970579fd41be\n";

#[must_use]
pub fn lines_starting(stdout: &[u8], prefixes: &[&str]) -> String {
    let mut lines = String::new();
    for line in String::from_utf8_lossy(stdout).lines() {
        if prefixes.iter().any(|prefix| line.starts_with(prefix)) {
            lines.push_str(line);
            lines.push('\n');
        }
    }
    lines
}

pub const MOTD: &[File] = &[
    reserved::EMPTY_HEARTH,
    reserved::FLEET_JSON,
    reserved::PROD_ENV,
    reserved::LAB_ENV,
    File { path: "hosts/west/west.json", body: b"{}", mode: 0o644 },
    File { path: "hosts/x1/x1.json", body: b"{}", mode: 0o644 },
    File {
        path: "hosts/west/west-prod.json",
        body: b"{\"placement\":{\"kind\":\"metal\",\"firmware\":\"efi\",\"hardware\":\"7th-gen\"},\"motd\":{}}",
        mode: 0o644,
    },
    File {
        path: "hosts/x1/x1-prod.json",
        body: b"{\"placement\":{\"kind\":\"metal\",\"firmware\":\"efi\",\"hardware\":\"7th-gen\"},\"motd\":{}}",
        mode: 0o644,
    },
    File { path: "hosts/west/west-lab.json", body: reserved::METAL_EFI, mode: 0o644 },
    File { path: "hosts/x1/x1-lab.json", body: reserved::METAL_EFI, mode: 0o644 },
    File { path: "tasks/motd/task.json", body: b"{}", mode: 0o644 },
    File { path: "tasks/motd/tree/etc/motd", body: b"Welcome.\n", mode: 0o644 },
];

#[derive(Clone, Debug)]
pub enum Laid {
    Resolution(hearth_protocol_tree::resolution::Resolution),
    File(&'static str),
    Record(&'static str, hearth_lib::encode::Digest),
    Layer(Box<hearth_protocol_tree::layer::Layer>),
}
pub fn lay(store: &Path, entries: &[Laid]) {
    fs::DirBuilder::new().recursive(true).mode(0o700).create(store.join("cache")).expect("cache directory");
    fs::DirBuilder::new().recursive(true).mode(0o700).create(store.join("resolutions")).expect("resolutions directory");
    if entries.iter().any(|entry| matches!(entry, Laid::Layer(_))) {
        fs::DirBuilder::new().recursive(true).mode(0o700).create(store.join("layers")).expect("layers directory");
    }
    for (name, bytes) in laid_files(entries) {
        fs::write(store.join(name), bytes).expect("laid file");
    }
}
#[must_use]
pub fn laid_files(entries: &[Laid]) -> Vec<(String, Vec<u8>)> {
    use hearth_protocol_tree::{
        cache_file::{self, CacheFile, Origin},
        limits,
    };
    entries
        .iter()
        .map(|entry| match entry {
            Laid::Resolution(record) => (
                format!("resolutions/{}.record", hex_digest(record.key.0)),
                hearth_protocol_tree::resolution::write(record, &limits::SHIPPED)
                    .expect("resolution record")
                    .into_vec(),
            ),
            Laid::File(name) => ((*name).to_owned(), b"fixture".to_vec()),
            Laid::Layer(record) => (
                format!("layers/{}.record", hex_digest(record.key.0)),
                hearth_protocol_tree::layer::write(record, &limits::SHIPPED).expect("layer record").into_vec(),
            ),
            Laid::Record(name, digest) => (
                (*name).to_owned(),
                cache_file::write(
                    &CacheFile {
                        at: skein_lib::Wall::EPOCH,
                        origin: Origin::Rendered { path: b"fixture".as_slice().into() },
                        sha256: *digest,
                        size: 7,
                        took: skein_lib::Duration::ZERO,
                    },
                    &limits::SHIPPED,
                )
                .expect("cache record")
                .into_vec(),
            ),
        })
        .collect()
}

impl File {
    #[must_use]
    pub const fn link(path: &'static str, target: &'static [u8]) -> Self {
        Self { path, body: target, mode: 0o120_000 }
    }
}
