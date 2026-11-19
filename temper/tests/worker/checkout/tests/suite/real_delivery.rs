//! Delivery and isolation from a run's git, against real git and a throwaway HTTP remote.
//! temper/docs/design/protocol/git.md, 3, 4.7, 9; domain/worker/checkouts.md, Two git directories.
use skein_lib::{Env, Queue, Secret, Time, Token, Wall};
use skein_throwaway_forgejo::{Forgejo, Init, Settings};
use std::{
    collections::BTreeMap,
    ffi::OsStr,
    fs,
    io::Write,
    os::unix::{ffi::OsStrExt, fs::PermissionsExt},
    path::{Path, PathBuf},
    process::{Command, Stdio},
};
const LIMITS: checkout::Limits =
    checkout::Limits { conflicts: 4, path_bytes: 128, ..temper_worker_checkout_world::LIMITS };
use temper_worker_domain_checkout::{self as checkout, git};
use temper_worker_protocol_git::{
    Identity, invocation,
    sequence::{self, Next},
};

fn store_token(remote: &[u8]) -> Token {
    let hash = skein_lib::sha256::digest(remote);
    Token::new(u64::from_be_bytes(hash[..8].try_into().expect("eight bytes")))
}

fn delivery_spec(remote: &[u8], push: &[u8], start: checkout::Start) -> checkout::Spec {
    checkout::Spec {
        key: b"delivery".as_slice().into(),
        repositories: Box::new([checkout::Repository {
            prefixes: Box::new([]),
            name: b"specimen".as_slice().into(),
            remote: remote.into(),
            git: None,
            start,
            identity: 1,
            account: None,
            push: Some(push.into()),
            expected: None,
            delivered: None,
            landing: Some(b"goal".as_slice().into()),
            save: None,
        }]),
    }
}

fn directory_bytes(directory: &Path) -> BTreeMap<PathBuf, Vec<u8>> {
    let mut files = BTreeMap::new();
    for entry in fs::read_dir(directory).expect("git directory") {
        let entry = entry.expect("git entry");
        if entry.file_type().expect("entry kind").is_dir() {
            for (path, bytes) in directory_bytes(&entry.path()) {
                files.insert(PathBuf::from(entry.file_name()).join(path), bytes);
            }
        } else {
            files.insert(PathBuf::from(entry.file_name()), fs::read(entry.path()).expect("git file"));
        }
    }
    files
}

struct World {
    root: PathBuf,
    identity: Identity,
    domain: checkout::Domain,
    hold: Option<Token>,
}

impl World {
    fn run_git(&self, directory: &str, args: &[&str]) -> Vec<u8> {
        let mut command = Command::new("/usr/bin/git");
        command.current_dir(self.root.join(directory)).env_clear();
        let kind = if args.first() == Some(&"push") { git::Kind::Push } else { git::Kind::Commit };
        for entry in
            invocation::environment(self.root.join("home").as_os_str().as_bytes(), Some(kind), Some(&self.identity))
        {
            let at = entry.iter().position(|b| *b == b'=').expect("environment entry");
            command.env(OsStr::from_bytes(&entry[..at]), OsStr::from_bytes(&entry[at + 1..]));
        }
        let output = command.args(args).output().expect("run's git ran");
        assert!(output.status.success(), "run git {args:?}: {:?}", output.stderr);
        output.stdout
    }

    fn topic_head(&self, workspace: Token, remote: &[u8]) -> git::Commit {
        let git::Done::Fetched { commit } = self.perform(git::Op::Fetch {
            at: git::Place { workspace, repository: b"specimen".as_slice().into(), git: None },
            store: store_token(remote),
            remote: remote.into(),
            want: git::Want::Branch { branch: b"topic".as_slice().into() },
            into: git::Into::Fetched,
            identity: 1,
            account: None,
        }) else {
            panic!("remote topic fetched")
        };
        commit
    }

    fn inspect(&self, directory: &str, args: &[&str]) -> String {
        let mut command = Command::new("/usr/bin/git");
        command.current_dir(&self.root).env_clear();
        if let Some(repository) = directory.strip_prefix("workspaces/") {
            command.args([format!("--git-dir=gitdirs/{repository}"), format!("--work-tree={directory}")]);
        } else {
            command.arg(format!("--git-dir={directory}"));
        }
        let output = command.args(args).output().expect("inspection git ran");
        assert!(output.status.success(), "inspection: {:?}", output.stderr);
        String::from_utf8(output.stdout).expect("git ids and settings")
    }

    fn plant_run_configuration(&self, directory: &str, moved: &str, unrelated: &str) {
        let run = self.root.join(directory).join(".git");
        for reference in ["HEAD", "refs/temper/landing", "refs/temper/fetched"] {
            // This is the run's tampering, not a worker operation. Suppress the hooks
            // already planted by the preceding story while moving its references.
            let output = Command::new("/usr/bin/git")
                .current_dir(self.root.join(directory))
                .env_clear()
                .args(["-c", "core.hooksPath=/dev/null", "update-ref", "--no-deref", reference, moved])
                .output()
                .expect("run's git");
            assert!(output.status.success(), "move run's {reference}: {:?}", output.stderr);
        }
        fs::write(run.join("MERGE_HEAD"), format!("{unrelated}\n")).expect("plant merge parent");
        let hooks = self.root.join("planted-hooks");
        let markers = self.root.join("planted-markers");
        fs::create_dir_all(&hooks).expect("hook directory");
        fs::create_dir_all(&markers).expect("marker directory");
        for name in [
            "clean",
            "smudge",
            "pre-commit",
            "post-checkout",
            "reference-transaction",
            "post-index-change",
            "pre-push",
            "alias",
        ] {
            let script = hooks.join(name);
            let marker = markers.join(name);
            fs::write(&script, format!("#!/bin/sh\nprintf planted > '{}'\ncat\n", marker.display()))
                .expect("planted driver");
            fs::set_permissions(script, fs::Permissions::from_mode(0o700)).expect("executable driver");
        }
        let config = run.join("config");
        fs::OpenOptions::new()
            .append(true)
            .open(config)
            .expect("run config")
            .write_all(
                format!(
                    "\n[filter \"planted\"]\nclean = {}\nsmudge = {}\n[core]\nhooksPath = {}\n[alias]\nplanted = !{}\n",
                    hooks.join("clean").display(),
                    hooks.join("smudge").display(),
                    hooks.display(),
                    hooks.join("alias").display(),
                )
                .as_bytes(),
            )
            .expect("plant configuration");
        fs::write(self.root.join(directory).join(".gitattributes"), b"* filter=planted\n").expect("attributes");
    }

    fn no_planted_program_ran(&self) {
        let markers = self.root.join("planted-markers");
        if markers.exists() {
            let programs = fs::read_dir(markers)
                .expect("markers")
                .map(|entry| entry.expect("marker").file_name())
                .collect::<Vec<_>>();
            assert!(programs.is_empty(), "run's programs ran: {programs:?}");
        }
    }

    fn delivered_files(&self, directory: &str, commit: git::Commit, parent: &str) {
        self.no_planted_program_ran();
        let delivered = String::from_utf8(invocation::hex(commit).into_vec()).expect("id");
        assert_eq!(
            self.inspect(directory, &["show", "-s", "--format=%P", &delivered]).trim(),
            parent,
            "delivery has exactly the worker's parent, not the run's HEAD or MERGE_HEAD"
        );
        assert_eq!(self.inspect(directory, &["rev-parse", "HEAD"]).trim(), delivered, "worker advances its HEAD");
        let delivered_paths = self.inspect(directory, &["ls-tree", "-r", "--name-only", &delivered]);
        let mut work_tree_paths = Vec::new();
        for entry in fs::read_dir(self.root.join(directory)).expect("work tree") {
            let entry = entry.expect("work tree entry");
            let name = entry.file_name().into_string().expect("fixture filename");
            if name != ".git" {
                assert!(entry.file_type().expect("entry kind").is_file(), "fixture files are flat");
                work_tree_paths.push(name);
            }
        }
        work_tree_paths.sort();
        assert_eq!(delivered_paths.lines().collect::<Vec<_>>(), work_tree_paths, "exact work tree paths");
        for name in delivered_paths.lines() {
            assert_eq!(
                self.inspect(directory, &["show", &format!("{delivered}:{name}")]).as_bytes(),
                fs::read(self.root.join(directory).join(name)).expect("work tree file"),
                "delivered {name}"
            );
        }
        assert_eq!(self.inspect(directory, &["show", &format!("{delivered}:.gitattributes")]), "* filter=planted\n");
    }

    fn redirected_delivery(&mut self, workspace: Token, remote: &[u8]) {
        let hold = self.hold.take().expect("held");
        assert!(matches!(self.finish(checkout::Event::Release { hold }), checkout::Request::Released { .. }));
        let answer = self.finish(checkout::Event::Prepare {
            client: Token::new(1),
            spec: delivery_spec(remote, b"change", checkout::Start::Branch { branch: b"change".as_slice().into() }),
        });
        assert!(
            matches!(answer, checkout::Request::Prepared {
            prepared: checkout::Prepared::Ready { workspace: reused, .. }, ..
        } if reused == workspace),
            "prepare before redirecting the run's git: {answer:?}"
        );
        self.no_planted_program_ran();
        let directory = format!("workspaces/{}/specimen", workspace.raw());
        let parent = self.inspect(&directory, &["rev-parse", "HEAD"]);
        let moved = self.inspect(&directory, &["rev-parse", "HEAD^"]);
        let store = format!("stores/{}", store_token(remote).raw());
        let unrelated = self.unrelated(&store, &directory);
        self.plant_run_configuration(&directory, moved.trim(), unrelated.trim());
        let elsewhere = self.root.join("elsewhere");
        let run = self.root.join(&directory).join(".git");
        fs::rename(&run, &elsewhere).expect("move run's git outside workspace");
        // Moving the run's directory changes the base of its borrowed object path.
        fs::write(
            elsewhere.join("objects/info/alternates"),
            format!("{}/gitdirs/{}/specimen/objects\n", self.root.display(), workspace.raw()),
        )
        .expect("run's borrowed objects");
        fs::write(&run, format!("gitdir: {}\n", elsewhere.display())).expect("redirect run's git");
        let before = directory_bytes(&elsewhere);
        let landing = self.inspect(&directory, &["rev-parse", "refs/temper/landing"]);
        fs::write(self.root.join(&directory).join("design0.md"), b"# Files, not the run's git\n").expect("edit");
        let answer = self.finish(checkout::Event::Push {
            hold: self.hold.expect("held"),
            message: checkout::Message { title: b"Redirected git".as_slice().into(), body: Box::default() },
        });
        let checkout::Request::Pushed { outcome: checkout::Outcome::Pushed { landings }, .. } = answer else {
            panic!("redirected delivery: {answer:?}")
        };
        let [checkout::Landing::Landed { commit }] = landings.as_ref() else { panic!("delivered: {landings:?}") };
        assert_eq!(
            self.perform(git::Op::Fetch {
                store: store_token(remote),
                at: git::Place { workspace, repository: b"specimen".as_slice().into(), git: None },
                remote: remote.into(),
                want: git::Want::Branch { branch: b"change".as_slice().into() },
                into: git::Into::Fetched,
                identity: 1,
                account: None,
            }),
            git::Done::Fetched { commit: *commit },
            "forge holds the redirected delivery"
        );
        self.delivered_files(&directory, *commit, parent.trim());
        let after = directory_bytes(&elsewhere);
        assert_eq!(after.keys().collect::<Vec<_>>(), before.keys().collect::<Vec<_>>(), "same redirected files");
        for (path, bytes) in before {
            assert_eq!(after.get(&path), Some(&bytes), "redirected file {} is untouched", path.display());
        }
        // Fetch updates only the worker's fetched reference; its landing pin stays put.
        assert_eq!(self.inspect(&directory, &["rev-parse", "refs/temper/landing"]), landing);
        assert_eq!(
            self.inspect(&directory, &["rev-parse", "refs/temper/fetched"]).trim(),
            String::from_utf8(invocation::hex(*commit).into_vec()).expect("id")
        );
    }

    fn unrelated(&self, store: &str, directory: &str) -> String {
        let tree = self.inspect(directory, &["rev-parse", "HEAD^{tree}"]);
        self.inspect(
            store,
            &[
                "-c",
                "user.name=Writer",
                "-c",
                "user.email=writer@example.invalid",
                "commit-tree",
                tree.trim(),
                "-m",
                "Unrelated root",
            ],
        )
    }

    fn missing(&self, directory: &str, revision: &str) -> Vec<String> {
        self.inspect(directory, &["rev-list", "--objects", "--missing=print", revision])
            .lines()
            .filter_map(|line| line.strip_prefix('?').map(str::to_owned))
            .collect()
    }

    fn advance_goal(&self, workspace: Token, remote: &[u8]) -> git::Commit {
        let at = git::Place { workspace, repository: b"peer".as_slice().into(), git: None };
        assert_eq!(
            self.perform(git::Op::Clone { at: at.clone(), remote: remote.into(), store: store_token(remote) }),
            git::Done::Succeeded
        );
        let git::Done::Fetched { commit: parent } = self.perform(git::Op::Fetch {
            store: store_token(remote),
            at: at.clone(),
            remote: remote.into(),
            want: git::Want::Branch { branch: b"goal".as_slice().into() },
            into: git::Into::Fetched,
            identity: 1,
            account: None,
        }) else {
            panic!("goal fetched")
        };
        assert_eq!(
            self.perform(git::Op::CheckOut { at: at.clone(), commit: parent, clean: true, identity: 1, account: None }),
            git::Done::Succeeded
        );
        fs::write(self.root.join(format!("workspaces/{}/peer/code.rs", workspace.raw())), b"inherited code\n")
            .expect("peer edit");
        let git::Done::Committed { commit } = self.perform(git::Op::Commit {
            at: at.clone(),
            parent,
            merging: None,
            always: false,
            title: b"Code".as_slice().into(),
            body: Box::default(),
            identity: 1,
            snapshot: Some(git::Snapshot { remote: remote.into(), account: None }),
        }) else {
            panic!("peer committed")
        };
        assert_eq!(
            self.perform(git::Op::Push {
                at,
                remote: remote.into(),
                commit,
                branch: b"goal".as_slice().into(),
                expected: Some(parent),
                identity: 1,
                account: None,
            }),
            git::Done::Succeeded
        );
        commit
    }

    fn command(
        &self,
        op: &git::Op,
        command: &invocation::Command,
        kind: Option<git::Kind>,
        message: &[u8],
        index: Option<&std::path::Path>,
    ) -> std::process::Output {
        let kind = if matches!(command, invocation::Command::UpdateRef { .. } | invocation::Command::Clean)
            || (matches!(command, invocation::Command::Unmerged) && kind == Some(git::Kind::Merge))
        {
            None
        } else {
            kind
        };
        let mut process = Command::new("/usr/bin/git");
        process.current_dir(&self.root).env_clear();
        for entry in invocation::environment(self.root.as_os_str().as_bytes(), kind, Some(&self.identity)) {
            let at = entry.iter().position(|b| *b == b'=').expect("environment entry");
            process.env(OsStr::from_bytes(&entry[..at]), OsStr::from_bytes(&entry[at + 1..]));
        }
        if let Some(index) = index {
            process.env("GIT_INDEX_FILE", index);
        }
        process.args(invocation::arguments_for(op, command).iter().map(|arg| OsStr::from_bytes(arg)));
        process.stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::piped());
        let mut child = process.spawn().expect("git ran");
        child.stdin.take().expect("stdin").write_all(message).expect("message");
        let output = child.wait_with_output().expect("git output");
        if matches!(command, invocation::Command::CheckOut { .. } | invocation::Command::Merge { .. }) {
            assert!(
                output.status.success()
                    || (matches!(command, invocation::Command::Merge { .. }) && output.status.code() == Some(1)),
                "lazy-fetch invocation failed: {}",
                String::from_utf8_lossy(&output.stderr)
            );
        }
        output
    }

    fn reds(&self, directory: &[u8], content: Option<Box<[u8]>>, marker: bool, rerun: Option<u32>) -> git::Done {
        let directory = self.root.join(OsStr::from_bytes(directory));
        fs::create_dir_all(&directory).expect("check directory");
        let file = directory.join("reds");
        if marker || content.is_none() {
            let _ = fs::remove_file(&file);
            let _ = fs::remove_file(directory.join("check-results"));
        }
        if let Some(content) = content {
            let name = if marker { "reds-wait" } else { "reds" };
            let temporary = directory.join(format!("{name}.tmp"));
            fs::write(&temporary, content).expect("check file");
            fs::rename(temporary, directory.join(name)).expect("whole check file");
        }
        if let Some(rerun) = rerun {
            let temporary = directory.join("rerun.tmp");
            fs::write(&temporary, format!("temper-rerun 1 {rerun}\n")).expect("rerun marker");
            fs::rename(temporary, directory.join("rerun")).expect("whole marker");
        }
        git::Done::Succeeded
    }

    #[expect(clippy::too_many_lines, reason = "the real-git harness translates process and file steps together")]
    fn perform(&self, mut op: git::Op) -> git::Done {
        if let git::Op::Make { workspace, .. } = &op {
            for tree in ["workspaces", "gitdirs"] {
                fs::create_dir_all(self.root.join(format!("{tree}/{}", workspace.raw()))).expect("checkout tree");
            }
            return git::Done::Succeeded;
        }
        if let git::Op::Store { store, remote, .. } = &op {
            let directory = format!("stores/{}", store.raw());
            fs::create_dir_all(self.root.join(&directory)).expect("store directory");
            let output = self.command(
                &op,
                &invocation::Command::Store { directory: directory.into_bytes().into(), remote: remote.clone() },
                Some(git::Kind::Store),
                b"",
                None,
            );
            assert!(output.status.success(), "store init: {:?}", output.stderr);
            return git::Done::Succeeded;
        }
        let at = match &op {
            git::Op::Clone { at, .. }
            | git::Op::Reds { at, .. }
            | git::Op::Reset { at, .. }
            | git::Op::Point { at, .. }
            | git::Op::Fetch { at, .. }
            | git::Op::Create { at, .. }
            | git::Op::CheckOut { at, .. }
            | git::Op::Look { at, .. }
            | git::Op::DiffPaths { at, .. }
            | git::Op::Brings { at, .. }
            | git::Op::IsAncestor { at, .. }
            | git::Op::Merge { at, .. }
            | git::Op::Commit { at, .. }
            | git::Op::Push { at, .. } => at,
            git::Op::Store { .. } | git::Op::Make { .. } => unreachable!(),
        };
        let repository = String::from_utf8(invocation::place(at).into_vec()).expect("fixture directory");
        let git_directory = self.root.join(OsStr::from_bytes(&invocation::git_dir(at)));
        let mut index: Option<PathBuf> = None;
        let mut next = sequence::first(&op);
        for _ in 0..20 {
            next = match next {
                Next::Run(command) => {
                    if matches!(command, invocation::Command::Add)
                        && let Some(path) = index.take()
                    {
                        fs::remove_file(path).expect("scan index removed before staging");
                    }
                    let message = match (&command, &op) {
                        (invocation::Command::UpdateIndex { paths }, _) => paths.clone(),
                        (invocation::Command::CommitTree { .. }, git::Op::Commit { title, body, .. }) => {
                            invocation::message(title, body)
                        }
                        _ => Box::default(),
                    };
                    let kind = if matches!(command, invocation::Command::SnapshotFetch { .. }) {
                        Some(git::Kind::Fetch)
                    } else {
                        Some(op.kind())
                    };
                    let private =
                        matches!(command, invocation::Command::Unmerged | invocation::Command::UpdateIndex { .. });
                    let output =
                        self.command(&op, &command, kind, &message, if private { index.as_deref() } else { None });
                    sequence::after(
                        &op,
                        &command,
                        u8::try_from(output.status.code().expect("exit")).expect("code"),
                        &output.stdout,
                        &output.stderr,
                        0,
                        &LIMITS,
                    )
                }
                Next::Snapshot { parent, merging } => {
                    let git::Op::Commit { parent: target, merging: second, .. } = &mut op else { unreachable!() };
                    *target = parent;
                    *second = merging;
                    Next::Run(if merging.is_some() { invocation::Command::Unmerged } else { invocation::Command::Add })
                }
                Next::Reds { directory, content, marker, rerun } => {
                    return self.reds(&directory, content, marker, rerun);
                }
                Next::Results { .. } => panic!("this delivery has no check results read"),
                Next::End(done) => {
                    if let Some(path) = index.take() {
                        fs::remove_file(path).expect("scan index removed on refusal");
                    }
                    return done;
                }
                Next::Alternates { format, run } => {
                    let git::Op::Clone { at, store, remote } = &op else { unreachable!() };
                    let directory = String::from_utf8(invocation::git_dir(at).into_vec()).expect("metadata path");
                    let (file, bytes, next) = if run {
                        (
                            self.root.join(&repository).join(".git/objects/info/alternates"),
                            format!("../../../../../{directory}/objects\n"),
                            Next::End(git::Done::Succeeded),
                        )
                    } else {
                        (
                            self.root.join(&directory).join("objects/info/alternates"),
                            format!("../../../../stores/{}/objects\n", store.raw()),
                            Next::Run(invocation::Command::Remote { remote: remote.clone(), format }),
                        )
                    };
                    fs::write(file, bytes).expect("alternates");
                    next
                }
                Next::Scan { files, cut } => {
                    if cut && index.is_none() {
                        let path = git_directory.join("temper-scan");
                        fs::copy(git_directory.join("index"), &path).expect("whole private index");
                        index = Some(path);
                    }
                    let mut marked = Vec::new();
                    for path in &files {
                        let mut markers = temper_worker_protocol_git::output::Markers::START;
                        let bytes = fs::read(self.root.join(&repository).join(OsStr::from_bytes(path)))
                            .expect("conflicted file");
                        markers.feed(&bytes);
                        if markers.finish() {
                            marked.push(path.clone());
                        }
                    }
                    if marked.is_empty() {
                        Next::Run(if cut {
                            invocation::Command::UpdateIndex {
                                paths: temper_worker_protocol_git::output::path_input(&files),
                            }
                        } else {
                            invocation::Command::Add
                        })
                    } else {
                        Next::End(git::Done::Conflicted { files: marked.into_boxed_slice(), cut: false })
                    }
                }
                Next::Make | Next::Store | Next::InstallStore { .. } => {
                    panic!("no unresolved paths")
                }
            };
        }
        panic!("git operation settles")
    }

    fn finish(&mut self, mut event: checkout::Event) -> checkout::Request {
        let env = Env { now: Time::ZERO, wall: Wall::from_nanos(0), limits: LIMITS };
        let mut out = Queue::uncapped();
        for _ in 0..80 {
            checkout::step(&mut self.domain, &env, event, &mut out);
            let mut pending = None;
            while let Some(request) = out.pop() {
                match request {
                    checkout::Request::Held { hold, .. } => self.hold = Some(hold),
                    checkout::Request::Io { owner, op, .. } => pending = Some((owner, op)),
                    terminal => return terminal,
                }
            }
            while self.domain.pop_fact().is_some() {}
            self.domain.reclaim();
            let (owner, op) = pending.expect("one operation outstanding");
            event = checkout::Event::Done { owner, done: self.perform(op) };
        }
        panic!("delivery settles")
    }
}

#[test]
#[expect(
    clippy::too_many_lines,
    reason = "one Forgejo scenario shares startup for lazy fetches and both run-git tampering stories"
)]
fn a_design_delivery_reaches_its_landing_branch_through_real_git() {
    let binary = temper_test_env::value(&temper_test_env::FORGEJO).expect("pinned Forgejo");
    let forgejo = Forgejo::from_path(std::path::Path::new(&binary)).expect("Forgejo");
    let scratch = skein_scratch::Scratch::new_in(&skein_scratch::paths::target_tmp(), "design-delivery");
    let mut settings = Settings::small();
    settings.actions = false;
    let mut instance = forgejo.start(&scratch.path().join("forge"), &settings).expect("started");
    let writer = instance.admin().clone();
    let repository = instance.create_repository(&writer, "specimen", Init::Readme).expect("repository");
    let target = format!("/api/v1/repos/{}/{}", writer.login, repository.name);
    let response = instance
        .exchange(&skein_throwaway_forgejo::Outgoing::json(
            "PATCH",
            &target,
            Some(&writer.token),
            br#"{"private":true}"#,
        ))
        .expect("private repository");
    assert_eq!(response.status, 200);
    let readme = instance
        .exchange(&skein_throwaway_forgejo::Outgoing::get(
            &format!("{target}/raw/README.md?ref=main"),
            Some(&writer.token),
        ))
        .expect("remote README");
    assert_eq!(readme.status, 200);
    let root = scratch.path().join("worker");
    fs::create_dir_all(&root).expect("worker root");
    let mut world = World {
        root,
        identity: Identity {
            token: Secret::new(writer.token.value().as_bytes().into()),
            login: writer.login.as_bytes().into(),
            name: b"Writer".as_slice().into(),
            email: b"writer@example.invalid".as_slice().into(),
        },
        domain: checkout::Domain::new(&LIMITS),
        hold: None,
    };
    let remote: Box<[u8]> = format!("{}/{}/{}.git", instance.base(), writer.login, repository.name).into_bytes().into();
    for design in [true, false] {
        let push = if design { b"goal".as_slice() } else { b"change".as_slice() };
        let spec = || {
            delivery_spec(
                &remote,
                push,
                if design {
                    checkout::Start::Base { branch: push.into() }
                } else {
                    checkout::Start::Branch { branch: b"goal".as_slice().into() }
                },
            )
        };
        let checkout::Request::Prepared { prepared: checkout::Prepared::Ready { workspace, .. }, .. } =
            world.finish(checkout::Event::Prepare { client: Token::new(1), spec: spec() })
        else {
            panic!("prepared")
        };
        let directory = format!("workspaces/{}/specimen", workspace.raw());
        let store = format!("stores/{}", store_token(&remote).raw());
        if design {
            let readme_blob = world.inspect(&directory, &["rev-parse", "HEAD:README.md"]);
            assert!(
                world.missing(&store, "--all").contains(&readme_blob.trim().to_owned()),
                "store lacks README contents"
            );
            assert_eq!(world.inspect(&store, &["config", "--get", "remote.origin.promisor"]), "true\n");
            assert_eq!(
                fs::read(world.root.join(&directory).join("README.md")).expect("checked-out README"),
                readme.body
            );
            let checked_out = world.inspect(&directory, &["rev-parse", "HEAD"]);
            assert!(world.missing(&directory, checked_out.trim()).is_empty(), "checkout owns all missing contents");
        }
        let parent = world.inspect(&directory, &["rev-parse", "HEAD"]);
        if !design {
            let moved = world.inspect(&directory, &["rev-parse", "HEAD^"]);
            let unrelated = world.unrelated(&store, &directory);
            let references = world.inspect(&directory, &["show-ref"]);
            world.plant_run_configuration(&directory, moved.trim(), unrelated.trim());
            assert_eq!(world.inspect(&directory, &["rev-parse", "HEAD"]), parent, "worker's HEAD is separate");
            assert_eq!(world.inspect(&directory, &["show-ref"]), references, "worker's references are separate");
            fs::remove_file(world.root.join(&directory).join("README.md")).expect("force checkout to write a blob");
            let hold = world.hold.take().expect("held");
            assert!(matches!(world.finish(checkout::Event::Release { hold }), checkout::Request::Released { .. }));
            let answer = world.finish(checkout::Event::Prepare { client: Token::new(1), spec: spec() });
            assert!(
                matches!(answer, checkout::Request::Prepared {
                prepared: checkout::Prepared::Ready { workspace: reused, .. }, ..
            } if reused == workspace),
                "same workstream prepares its existing checkout: {answer:?}"
            );
            world.no_planted_program_ran();
            assert_eq!(world.inspect(&directory, &["rev-parse", "HEAD"]), parent, "prepare keeps the worker's start");
            assert_eq!(world.inspect(&directory, &["show-ref"]), references, "prepare restores worker's references");
            fs::write(world.root.join(&directory).join(".gitattributes"), b"* filter=planted\n").expect("attributes");
        }
        for index in 0..=LIMITS.conflicts {
            fs::write(
                world.root.join(&directory).join(format!("design{index}.md")),
                if design { b"# Design\n" } else { b"# Change\n" },
            )
            .expect("edit");
        }
        let answer = world.finish(checkout::Event::Push {
            hold: world.hold.expect("held"),
            message: checkout::Message { title: b"Design".as_slice().into(), body: Box::default() },
        });
        world.no_planted_program_ran();
        let checkout::Request::Pushed { outcome: checkout::Outcome::Pushed { landings }, .. } = answer else {
            panic!("push terminal")
        };
        let [checkout::Landing::Landed { commit }] = landings.as_ref() else { panic!("delivered: {landings:?}") };
        assert_eq!(
            world.perform(git::Op::Fetch {
                store: store_token(&remote),
                at: git::Place { workspace, repository: b"specimen".as_slice().into(), git: None },
                remote: remote.clone(),
                want: git::Want::Branch { branch: push.into() },
                into: git::Into::Fetched,
                identity: 1,
                account: None,
            }),
            git::Done::Fetched { commit: *commit },
            "remote branch reached the delivered commit"
        );
        if !design {
            world.delivered_files(&directory, *commit, parent.trim());
            world.redirected_delivery(workspace, &remote);
        }
        if design {
            let moved = world.advance_goal(workspace, &remote);
            fs::write(world.root.join(&directory).join("later.md"), b"# More design\n").expect("later edit");
            let answer = world.finish(checkout::Event::Push {
                hold: world.hold.expect("held"),
                message: checkout::Message { title: b"More design".as_slice().into(), body: Box::default() },
            });
            assert!(
                matches!(answer, checkout::Request::Pushed { outcome: checkout::Outcome::Pushed { ref landings }, .. }
                if matches!(landings.as_ref(), [checkout::Landing::DesignMerged { tip, .. }] if *tip == moved)),
                "{answer:?}"
            );
            assert_eq!(
                fs::read(world.root.join(&directory).join("code.rs")).expect("merged code"),
                b"inherited code\n"
            );
            let code_blob = world.inspect(
                &directory,
                &[
                    "rev-parse",
                    &format!("{}:code.rs", String::from_utf8(invocation::hex(moved).into_vec()).expect("id")),
                ],
            );
            assert!(
                world.missing(&store, "--all").contains(&code_blob.trim().to_owned()),
                "merge contents never enter store"
            );
        }
        let hold = world.hold.take().expect("held");
        assert!(matches!(world.finish(checkout::Event::Release { hold }), checkout::Request::Released { .. }));
    }
    world.no_planted_program_ran();
    instance.stop().expect("stopped");
}

#[expect(
    clippy::too_many_lines,
    reason = "one merge story holds preparation, resolution, delivery and its index oracle together"
)]
fn merge_past_the_bound(marked: bool) {
    let binary = temper_test_env::value(&temper_test_env::FORGEJO).expect("pinned Forgejo");
    let forgejo = Forgejo::from_path(std::path::Path::new(&binary)).expect("Forgejo");
    let scratch = skein_scratch::Scratch::new_in(&skein_scratch::paths::target_tmp(), "paged-merge");
    let mut settings = Settings::small();
    settings.actions = false;
    let mut instance = forgejo.start(&scratch.path().join("forge"), &settings).expect("started");
    let writer = instance.admin().clone();
    let repository = instance.create_repository(&writer, "specimen", Init::Empty).expect("repository");
    let remote_text = format!("{}/{}/{}.git", instance.base(), writer.login, repository.name);
    let root = scratch.path().join("worker");
    fs::create_dir_all(root.join("seed")).expect("seed directory");
    let mut world = World {
        root,
        identity: Identity {
            token: Secret::new(writer.token.value().as_bytes().into()),
            login: writer.login.as_bytes().into(),
            name: b"Worker".as_slice().into(),
            email: b"worker@example.invalid".as_slice().into(),
        },
        domain: checkout::Domain::new(&LIMITS),
        hold: None,
    };
    world.run_git("seed", &["init", "--initial-branch=main"]);
    let names: Vec<_> = (0..LIMITS.conflicts + 2).map(|n| format!("file{n}")).collect();
    let write_files = |content: &[u8]| {
        for name in &names {
            fs::write(world.root.join("seed").join(name), content).expect("seed file");
        }
    };
    write_files(b"original\n");
    world.run_git("seed", &["add", "--all"]);
    world.run_git("seed", &["commit", "-m", "Initial"]);
    world.run_git("seed", &["checkout", "-b", "topic"]);
    write_files(b"ours\n");
    world.run_git("seed", &["add", "--all"]);
    world.run_git("seed", &["commit", "-m", "Ours"]);
    let ours = invocation::object(world.run_git("seed", &["rev-parse", "HEAD"]).trim_ascii()).expect("ours");
    world.run_git("seed", &["push", &remote_text, "topic"]);
    world.run_git("seed", &["checkout", "main"]);
    write_files(b"theirs\n");
    world.run_git("seed", &["add", "--all"]);
    world.run_git("seed", &["commit", "-m", "Theirs"]);
    let base = invocation::object(world.run_git("seed", &["rev-parse", "HEAD"]).trim_ascii()).expect("base");
    world.run_git("seed", &["push", &remote_text, "main"]);
    let remote: Box<[u8]> = remote_text.as_bytes().into();
    let spec = checkout::Spec {
        key: b"paged-merge".as_slice().into(),
        repositories: Box::new([checkout::Repository {
            prefixes: Box::new([]),
            name: b"specimen".as_slice().into(),
            remote: remote.clone(),
            git: None,
            start: checkout::Start::Merge { branch: b"topic".as_slice().into(), base },
            identity: 1,
            account: None,
            push: Some(b"topic".as_slice().into()),
            expected: Some(ours),
            delivered: None,
            landing: None,
            save: None,
        }]),
    };
    let prepared = world.finish(checkout::Event::Prepare { client: Token::new(1), spec });
    let checkout::Request::Prepared { prepared: checkout::Prepared::Ready { workspace, conflicts }, .. } = prepared
    else {
        panic!("merge prepared: {prepared:?}")
    };
    assert_eq!(conflicts.len(), 1);
    assert_eq!(conflicts[0].files.len(), usize::try_from(LIMITS.conflicts).expect("small bound"));
    let directory = format!("workspaces/{}/specimen", workspace.raw());
    let git_directory = world.root.join(format!("gitdirs/{}/specimen", workspace.raw()));
    let unmerged_args = ["diff-files", "--name-only", "-z", "--diff-filter=U"];
    let unmerged = names.join("\0") + "\0";
    assert_eq!(world.inspect(&directory, &unmerged_args), unmerged);
    let last = names.last().expect("last page path");
    for name in &names {
        if !marked || name != last {
            fs::write(world.root.join(&directory).join(name), b"resolved\n").expect("settled file");
        }
    }
    // The command stages into its own .git, not the worker's still-unmerged index.
    world.run_git(&directory, &["add", "--", &names[0], &names[1]]);
    assert_eq!(world.inspect(&directory, &unmerged_args), unmerged);
    let answer = world.finish(checkout::Event::Push {
        hold: world.hold.expect("held"),
        message: checkout::Message { title: b"Resolve merge".as_slice().into(), body: Box::default() },
    });
    let checkout::Request::Pushed { outcome: checkout::Outcome::Pushed { landings }, .. } = answer else {
        panic!("push terminal")
    };
    if marked {
        assert_eq!(landings.as_ref(), &[checkout::Landing::Conflicted { files: Box::new([last.as_bytes().into()]) }]);
        assert_eq!(world.inspect(&directory, &unmerged_args), unmerged, "every worker index conflict survives refusal");
        assert_eq!(world.topic_head(workspace, &remote), ours);
    } else {
        let [checkout::Landing::Landed { commit }] = landings.as_ref() else { panic!("merge delivered: {landings:?}") };
        let head = String::from_utf8(invocation::hex(*commit).into_vec()).expect("commit id");
        assert_eq!(world.topic_head(workspace, &remote), *commit);
        let parents = world.inspect(&directory, &["show", "--no-patch", "--format=%P", &head]);
        assert_eq!(
            parents.split_whitespace().map(|id| invocation::object(id.as_bytes()).expect("parent")).collect::<Vec<_>>(),
            [ours, base]
        );
        for name in &names {
            assert_eq!(world.inspect(&directory, &["show", &format!("{head}:{name}")]), "resolved\n");
        }
    }
    assert!(!git_directory.join("temper-scan").exists(), "no scan copy remains");
    let hold = world.hold.take().expect("held");
    assert!(matches!(world.finish(checkout::Event::Release { hold }), checkout::Request::Released { .. }));
    instance.stop().expect("stopped");
}

#[test]
fn a_merge_delivery_pages_past_the_bound_after_the_run_stages_two_files() {
    merge_past_the_bound(false);
}

#[test]
fn a_last_page_marker_refuses_real_delivery_and_preserves_every_unmerged_path() {
    merge_past_the_bound(true);
}
