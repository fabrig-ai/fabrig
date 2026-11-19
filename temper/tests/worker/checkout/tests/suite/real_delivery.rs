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
    process::Command,
};
const LIMITS: checkout::Limits =
    checkout::Limits { conflicts: 4, path_bytes: 128, ..temper_worker_checkout_world::LIMITS };
use temper_worker_domain_checkout::{self as checkout, git};
use temper_worker_protocol_git::{Identity, invocation};
mod io;

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
            pin: None,
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
    git: std::cell::RefCell<io::Driver>,
    operations: Vec<git::Kind>,
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
        command.env_clear();
        if directory.starts_with("workspaces/") {
            command.current_dir(self.root.join(directory));
        } else {
            command.current_dir(&self.root).arg(format!("--git-dir={directory}"));
        }
        let output = command.args(args).output().expect("inspection git ran");
        assert!(output.status.success(), "inspection: {:?}", output.stderr);
        String::from_utf8(output.stdout).expect("git ids and settings")
    }

    fn worker(&self, directory: &str, args: &[&str]) -> String {
        let work_tree = format!("--work-tree={directory}");
        let mut worker_args = vec![work_tree.as_str()];
        worker_args.extend_from_slice(args);
        self.inspect(&directory.replacen("workspaces/", "gitdirs/", 1), &worker_args)
    }

    fn pre_pr_tree(&self, directory: &str) -> String {
        let root = self.root.join(directory);
        let cache = root.join(".git/temper-pre-pr");
        fs::create_dir_all(&cache).expect("check cache");
        let index = cache.join("index");
        fs::copy(root.join(".git/index"), &index).expect("private index");
        let mut tree = String::new();
        for args in [&["add", "-A"][..], &["write-tree"][..]] {
            let output = Command::new("/usr/bin/git")
                .current_dir(&root)
                .env_clear()
                .env("GIT_INDEX_FILE", &index)
                .args(args)
                .output()
                .expect("pre-pr git");
            assert!(output.status.success(), "pre-pr step {args:?}: {:?}", output.stderr);
            tree = String::from_utf8(output.stdout).expect("tree id");
        }
        let worktree = cache.join("base");
        self.inspect(
            directory,
            &[
                "worktree",
                "add",
                "--detach",
                "--force",
                worktree.to_str().expect("fixture path"),
                "refs/temper/landing",
            ],
        );
        tree
    }

    fn prepare_view(
        &mut self,
        start: checkout::Start,
        branch: &[u8],
        remote: &[u8],
    ) -> (Token, Box<[checkout::Conflicts]>) {
        let spec = checkout::Spec {
            key: b"view".as_slice().into(),
            repositories: Box::new([checkout::Repository {
                prefixes: Box::new([]),
                name: b"specimen".as_slice().into(),
                remote: remote.into(),
                git: None,
                start,
                identity: 1,
                account: None,
                push: Some(branch.into()),
                expected: None,
                delivered: None,
                landing: Some(b"main".as_slice().into()),
                save: None,
            }]),
        };
        let answer = self.finish(checkout::Event::Prepare { client: Token::new(1), spec });
        let checkout::Request::Prepared { prepared: checkout::Prepared::Ready { workspace, conflicts }, .. } = answer
        else {
            panic!("prepared: {answer:?}")
        };
        (workspace, conflicts)
    }

    fn push_view(&mut self) -> git::Commit {
        let answer = self.finish(checkout::Event::Push {
            hold: self.hold.expect("held"),
            message: checkout::Message { title: b"View".as_slice().into(), body: Box::default() },
        });
        let checkout::Request::Pushed { outcome: checkout::Outcome::Pushed { landings }, .. } = answer else {
            panic!("delivered: {answer:?}")
        };
        let [checkout::Landing::Landed { commit }] = landings.as_ref() else { panic!("landed: {landings:?}") };
        *commit
    }

    fn release(&mut self) {
        let hold = self.hold.take().expect("held");
        assert!(matches!(self.finish(checkout::Event::Release { hold }), checkout::Request::Released { .. }));
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
        self.plant_run_drivers(directory);
    }

    fn plant_run_drivers(&self, directory: &str) {
        let run = self.root.join(directory).join(".git");
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
            self.worker(directory, &["show", "-s", "--format=%P", &delivered]).trim(),
            parent,
            "delivery has exactly the worker's parent, not the run's HEAD or MERGE_HEAD"
        );
        assert_eq!(self.worker(directory, &["rev-parse", "HEAD"]).trim(), delivered, "worker advances its HEAD");
        let delivered_paths = self.worker(directory, &["ls-tree", "-r", "--name-only", &delivered]);
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
                self.worker(directory, &["show", &format!("{delivered}:{name}")]).as_bytes(),
                fs::read(self.root.join(directory).join(name)).expect("work tree file"),
                "delivered {name}"
            );
        }
        assert_eq!(self.worker(directory, &["show", &format!("{delivered}:.gitattributes")]), "* filter=planted\n");
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
        let parent = self.worker(&directory, &["rev-parse", "HEAD"]);
        let moved = self.worker(&directory, &["rev-parse", "HEAD^"]);
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
        let landing = self.worker(&directory, &["rev-parse", "refs/temper/landing"]);
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
        assert_eq!(self.worker(&directory, &["rev-parse", "refs/temper/landing"]), landing);
        assert_eq!(
            self.worker(&directory, &["rev-parse", "refs/temper/fetched"]).trim(),
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

    fn perform(&self, op: git::Op) -> git::Done {
        let looked_config = if let git::Op::Look { at, .. } = &op {
            let path = self.root.join(OsStr::from_bytes(&invocation::place(at))).join(".git/config");
            Some((path.clone(), fs::read(path).expect("kept config")))
        } else {
            None
        };
        let done = self.git.borrow_mut().perform(op, self.identity.clone());
        if let Some((path, config)) = looked_config {
            assert_eq!(
                fs::read(path).expect("run config after Look"),
                config,
                "Look leaves the run's config unchanged"
            );
        }
        done
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
            self.operations.push(op.kind());
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
        git: std::cell::RefCell::new(io::Driver::new(&root)),
        root,
        identity: Identity {
            token: Secret::new(writer.token.value().as_bytes().into()),
            login: writer.login.as_bytes().into(),
            name: b"Writer".as_slice().into(),
            email: b"writer@example.invalid".as_slice().into(),
        },
        domain: checkout::Domain::new(&LIMITS),
        hold: None,
        operations: Vec::new(),
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
            assert_eq!(world.inspect(&directory, &["status", "--porcelain"]), "", "fresh run is clean");
            for revision in ["HEAD", "refs/temper/landing"] {
                assert_eq!(
                    world.inspect(&directory, &["rev-parse", revision]),
                    world.worker(&directory, &["rev-parse", revision])
                );
            }
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
        let parent = world.worker(&directory, &["rev-parse", "HEAD"]);
        if !design {
            let moved = world.worker(&directory, &["rev-parse", "HEAD^"]);
            let unrelated = world.unrelated(&store, &directory);
            let references = world.worker(&directory, &["show-ref"]);
            world.plant_run_configuration(&directory, moved.trim(), unrelated.trim());
            assert_eq!(world.worker(&directory, &["rev-parse", "HEAD"]), parent, "worker's HEAD is separate");
            assert_eq!(world.worker(&directory, &["show-ref"]), references, "worker's references are separate");
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
            assert_eq!(world.worker(&directory, &["rev-parse", "HEAD"]), parent, "prepare keeps the worker's start");
            assert_eq!(world.worker(&directory, &["show-ref"]), references, "prepare restores worker's references");
            fs::write(world.root.join(&directory).join(".gitattributes"), b"* filter=planted\n").expect("attributes");
        }
        for index in 0..=LIMITS.conflicts {
            fs::write(
                world.root.join(&directory).join(format!("design{index}.md")),
                if design { b"# Design\n" } else { b"# Change\n" },
            )
            .expect("edit");
        }
        let tree = design.then(|| world.pre_pr_tree(&directory));
        let answer = world.finish(checkout::Event::Push {
            hold: world.hold.expect("held"),
            message: checkout::Message { title: b"Design".as_slice().into(), body: Box::default() },
        });
        world.no_planted_program_ran();
        let checkout::Request::Pushed { outcome: checkout::Outcome::Pushed { landings }, .. } = answer else {
            panic!("push terminal")
        };
        let [checkout::Landing::Landed { commit }] = landings.as_ref() else { panic!("delivered: {landings:?}") };
        if let Some(tree) = tree {
            assert_eq!(world.worker(&directory, &["rev-parse", "HEAD^{tree}"]), tree, "delivery commits pre-pr's tree");
            assert_eq!(
                world.inspect(&directory, &["rev-parse", "HEAD"]),
                world.worker(&directory, &["rev-parse", "HEAD"])
            );
            assert_eq!(world.inspect(&directory, &["status", "--porcelain"]), "", "delivered run is clean");
        }
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
    world.git.borrow_mut().close();
    instance.stop().expect("stopped");
}

fn view_fixture(name: &str) -> (skein_scratch::Scratch, skein_throwaway_forgejo::Instance, World, Box<[u8]>) {
    let binary = temper_test_env::value(&temper_test_env::FORGEJO).expect("pinned Forgejo");
    let forgejo = Forgejo::from_path(std::path::Path::new(&binary)).expect("Forgejo");
    let scratch = skein_scratch::Scratch::new_in(&skein_scratch::paths::target_tmp(), name);
    let mut settings = Settings::small();
    settings.actions = false;
    let mut instance = forgejo.start(&scratch.path().join("forge"), &settings).expect("started");
    let writer = instance.admin().clone();
    let repository = instance.create_repository(&writer, "specimen", Init::Readme).expect("repository");
    let root = scratch.path().join("worker");
    fs::create_dir_all(&root).expect("worker root");
    let world = World {
        git: std::cell::RefCell::new(io::Driver::new(&root)),
        root,
        identity: Identity {
            token: Secret::new(writer.token.value().as_bytes().into()),
            login: writer.login.as_bytes().into(),
            name: b"Writer".as_slice().into(),
            email: b"writer@example.invalid".as_slice().into(),
        },
        domain: checkout::Domain::new(&LIMITS),
        hold: None,
        operations: Vec::new(),
    };
    let remote = format!("{}/{}/{}.git", instance.base(), writer.login, repository.name).into_bytes().into();
    (scratch, instance, world, remote)
}

#[test]
fn a_producer_conflict_is_visible_to_plain_git_and_pre_pr_can_stage_its_resolution() {
    let (_scratch, instance, mut world, remote) = view_fixture("producer-view");
    let mut base = None;
    for branch in [b"topic".as_slice(), b"main"] {
        let (workspace, conflicts) =
            world.prepare_view(checkout::Start::Base { branch: branch.into() }, branch, &remote);
        assert!(conflicts.is_empty());
        let directory = format!("workspaces/{}/specimen", workspace.raw());
        assert_eq!(world.inspect(&directory, &["status", "--porcelain"]), "");
        fs::write(world.root.join(&directory).join("README.md"), branch).expect("opposing edit");
        base = Some(world.push_view());
        world.release();
    }
    let (workspace, conflicts) = world.prepare_view(
        checkout::Start::Merge { branch: b"topic".as_slice().into(), base: base.expect("main advanced") },
        b"topic",
        &remote,
    );
    assert_eq!(conflicts.len(), 1);
    let directory = format!("workspaces/{}/specimen", workspace.raw());
    assert_eq!(world.inspect(&directory, &["status", "--porcelain"]), "UU README.md\n");
    assert!(world.inspect(&directory, &["status"]).contains("unmerged paths"));
    for revision in ["HEAD", "MERGE_HEAD", "refs/temper/landing"] {
        assert_eq!(
            world.inspect(&directory, &["rev-parse", revision]),
            world.worker(&directory, &["rev-parse", revision])
        );
    }
    fs::write(world.root.join(&directory).join("README.md"), b"resolved\n").expect("resolution");
    let tree = world.pre_pr_tree(&directory);
    let commit = world.push_view();
    assert_eq!(world.worker(&directory, &["rev-parse", "HEAD^{tree}"]), tree);
    assert_eq!(world.inspect(&directory, &["rev-parse", "HEAD"]).trim().as_bytes(), invocation::hex(commit).as_ref());
    assert_eq!(world.inspect(&directory, &["status", "--porcelain"]), "");
    assert!(!world.root.join(&directory).join(".git/MERGE_HEAD").exists());
    world.release();
    world.git.borrow_mut().close();
    instance.stop().expect("stopped");
}

fn contents(root: &std::path::Path) -> std::collections::BTreeMap<PathBuf, Option<Vec<u8>>> {
    let mut result = std::collections::BTreeMap::new();
    let mut pending = vec![root.to_path_buf()];
    while let Some(directory) = pending.pop() {
        for entry in fs::read_dir(directory).expect("directory snapshot") {
            let entry = entry.expect("snapshot entry");
            let path = entry.path();
            let directory = entry.file_type().expect("entry kind").is_dir();
            let bytes = if directory {
                pending.push(path.clone());
                None
            } else {
                Some(fs::read(&path).expect("snapshot bytes"))
            };
            result.insert(path.strip_prefix(root).expect("beneath snapshot").to_path_buf(), bytes);
        }
    }
    result
}

#[test]
fn a_gitdir_file_is_remade_without_touching_the_directory_it_names() {
    let (_scratch, instance, mut world, remote) = view_fixture("remade-view");
    let (workspace, _) =
        world.prepare_view(checkout::Start::Base { branch: b"main".as_slice().into() }, b"main", &remote);
    let directory = format!("workspaces/{}/specimen", workspace.raw());
    let git = world.root.join(&directory).join(".git");
    let elsewhere = world.root.join("elsewhere");
    fs::rename(&git, &elsewhere).expect("move run metadata");
    fs::write(&git, format!("gitdir: {}\n", elsewhere.display())).expect("planted gitdir file");
    let before = contents(&elsewhere);
    let head = invocation::object(world.worker(&directory, &["rev-parse", "HEAD"]).trim().as_bytes()).expect("head");
    assert_eq!(
        world.perform(git::Op::Point {
            at: git::Place { workspace, repository: b"specimen".as_slice().into(), git: None },
            into: git::Into::Landing,
            commit: head
        }),
        git::Done::Succeeded
    );
    assert!(fs::symlink_metadata(&git).expect("remade metadata").is_dir());
    assert_eq!(world.inspect(&directory, &["rev-parse", "HEAD"]), world.worker(&directory, &["rev-parse", "HEAD"]));
    assert_eq!(contents(&elsewhere), before, "every byte and directory left alone");
    world.release();
    world.git.borrow_mut().close();
    instance.stop().expect("stopped");
}

/// A kept one-directory checkout crosses the two-directory redeploy without
/// trusting the configuration that is now the run's (checkouts.md, Two git directories).
#[test]
#[expect(
    clippy::too_many_lines,
    reason = "one real-git scenario seeds the kept checkout, holds it and delivers its files"
)]
fn a_kept_one_directory_checkout_is_taken_on_without_running_its_filters() {
    let binary = temper_test_env::value(&temper_test_env::FORGEJO).expect("pinned Forgejo");
    let forgejo = Forgejo::from_path(std::path::Path::new(&binary)).expect("Forgejo");
    let scratch = skein_scratch::Scratch::new_in(&skein_scratch::paths::target_tmp(), "take-on-checkout");
    let mut settings = Settings::small();
    settings.actions = false;
    let mut instance = forgejo.start(&scratch.path().join("forge"), &settings).expect("started");
    let writer = instance.admin().clone();
    let repository = instance.create_repository(&writer, "specimen", Init::Readme).expect("repository");
    let remote: Box<[u8]> = format!("{}/{}/{}.git", instance.base(), writer.login, repository.name).into_bytes().into();
    let root = scratch.path().join("worker");
    fs::create_dir_all(&root).expect("worker root");
    let mut world = World {
        git: std::cell::RefCell::new(io::Driver::new(&root)),
        root,
        identity: Identity {
            token: Secret::new(writer.token.value().as_bytes().into()),
            login: writer.login.as_bytes().into(),
            name: b"Writer".as_slice().into(),
            email: b"writer@example.invalid".as_slice().into(),
        },
        domain: checkout::Domain::new(&LIMITS),
        hold: None,
        operations: Vec::new(),
    };
    world.domain.reuse_workspaces();
    let store = store_token(&remote);
    assert_eq!(
        world.perform(git::Op::Store {
            store,
            aside: None,
            remote: remote.clone(),
            identity: 1,
            account: None,
            git: None,
        }),
        git::Done::Succeeded
    );
    let directory = "workspaces/0/specimen";
    let work_tree = world.root.join(directory);
    fs::create_dir_all(&work_tree).expect("kept work tree");
    // These commands make the checkout the one-directory way, before planting
    // the configuration. No worker command below discovers its directory here.
    for args in [vec!["init", "--quiet"], vec!["remote", "add", "origin", std::str::from_utf8(&remote).expect("URL")]] {
        let output = Command::new("/usr/bin/git")
            .current_dir(&work_tree)
            .env_clear()
            .args(["--git-dir=.git", "--work-tree=."])
            .args(args)
            .output()
            .expect("old checkout setup");
        assert!(output.status.success(), "old checkout: {:?}", output.stderr);
    }
    fs::write(
        work_tree.join(".git/objects/info/alternates"),
        format!("../../../../../stores/{}/objects\n", store.raw()),
    )
    .expect("old alternates");
    for args in [
        vec!["fetch", "--quiet", "origin", "main"],
        vec!["-c", "remote.origin.promisor=true", "checkout", "--quiet", "--detach", "FETCH_HEAD"],
    ] {
        let output = Command::new("/usr/bin/git")
            .current_dir(&work_tree)
            .env_clear()
            .args(["--git-dir=.git", "--work-tree=."])
            .args(args)
            .output()
            .expect("old checkout");
        assert!(output.status.success(), "old checkout: {:?}", output.stderr);
    }
    let original = fs::read(work_tree.join("README.md")).expect("old checkout wrote README");
    world.plant_run_drivers(directory);
    // A new worker index must cause every tracked file to be written, even
    // though the kept checkout's own index says that README is unchanged.
    fs::write(work_tree.join("README.md"), b"dirty kept README\n").expect("dirty tracked file");
    assert!(!world.root.join("gitdirs/0/specimen").exists());
    let spec = checkout::Spec {
        key: b"kept".as_slice().into(),
        repositories: Box::new([checkout::Repository {
            prefixes: Box::new([]),
            name: b"specimen".as_slice().into(),
            remote: remote.clone(),
            git: None,
            start: checkout::Start::Base { branch: b"main".as_slice().into() },
            identity: 1,
            account: None,
            push: Some(b"change".as_slice().into()),
            expected: None,
            delivered: None,
            landing: Some(b"main".as_slice().into()),
            save: None,
        }]),
    };
    assert!(matches!(world.finish(checkout::Event::Prepare { client: Token::new(1), spec }),
        checkout::Request::Prepared { prepared: checkout::Prepared::Ready { workspace, .. }, .. } if workspace == Token::new(0)));
    assert!(world.operations.contains(&git::Kind::Look));
    assert!(!world.operations.contains(&git::Kind::Make));
    assert!(!world.operations.contains(&git::Kind::Clone));
    assert_eq!(fs::read(work_tree.join("README.md")).expect("fresh checkout"), original);
    world.no_planted_program_ran();
    fs::write(work_tree.join("delivered.txt"), b"work tree files\n").expect("run edit");
    // The checkout removed the untracked attributes. Keep them for the delivery
    // too, so staging files would run a planted clean driver if it read .git.
    fs::write(work_tree.join(".gitattributes"), b"* filter=planted\n").expect("delivery attributes");
    let answer = world.finish(checkout::Event::Push {
        hold: world.hold.expect("held"),
        message: checkout::Message { title: b"Kept checkout".as_slice().into(), body: Box::default() },
    });
    let checkout::Request::Pushed { outcome: checkout::Outcome::Pushed { landings }, .. } = answer else {
        panic!("push: {answer:?}")
    };
    let [checkout::Landing::Landed { commit }] = landings.as_ref() else { panic!("landed: {landings:?}") };
    assert_eq!(
        world.perform(git::Op::Fetch {
            at: git::Place { workspace: Token::new(0), repository: b"specimen".as_slice().into(), git: None },
            store,
            remote,
            want: git::Want::Branch { branch: b"change".as_slice().into() },
            into: git::Into::Fetched,
            identity: 1,
            account: None,
        }),
        git::Done::Fetched { commit: *commit }
    );
    let delivered = String::from_utf8(invocation::hex(*commit).into_vec()).expect("id");
    for name in ["README.md", ".gitattributes", "delivered.txt"] {
        assert_eq!(
            world.inspect(directory, &["show", &format!("{delivered}:{name}")]).as_bytes(),
            fs::read(work_tree.join(name)).expect("work tree file")
        );
    }
    world.no_planted_program_ran();
    world.git.borrow_mut().close();
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
        git: std::cell::RefCell::new(io::Driver::new(&root)),
        root,
        identity: Identity {
            token: Secret::new(writer.token.value().as_bytes().into()),
            login: writer.login.as_bytes().into(),
            name: b"Worker".as_slice().into(),
            email: b"worker@example.invalid".as_slice().into(),
        },
        domain: checkout::Domain::new(&LIMITS),
        hold: None,
        operations: Vec::new(),
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
    assert_eq!(world.worker(&directory, &unmerged_args), unmerged);
    let last = names.last().expect("last page path");
    for name in &names {
        if !marked || name != last {
            fs::write(world.root.join(&directory).join(name), b"resolved\n").expect("settled file");
        }
    }
    // The command stages into its own .git, not the worker's still-unmerged index.
    world.run_git(&directory, &["add", "--", &names[0], &names[1]]);
    assert_eq!(world.worker(&directory, &unmerged_args), unmerged);
    let answer = world.finish(checkout::Event::Push {
        hold: world.hold.expect("held"),
        message: checkout::Message { title: b"Resolve merge".as_slice().into(), body: Box::default() },
    });
    let checkout::Request::Pushed { outcome: checkout::Outcome::Pushed { landings }, .. } = answer else {
        panic!("push terminal")
    };
    if marked {
        assert_eq!(landings.as_ref(), &[checkout::Landing::Conflicted { files: Box::new([last.as_bytes().into()]) }]);
        assert_eq!(world.worker(&directory, &unmerged_args), unmerged, "every worker index conflict survives refusal");
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
    world.git.borrow_mut().close();
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
