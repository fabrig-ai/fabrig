//! The checkout's design delivery against real git and a throwaway HTTP remote.
use skein_lib::{Env, Queue, Secret, Time, Token, Wall};
use skein_throwaway_forgejo::{Forgejo, Init, Settings};
use std::{
    ffi::OsStr,
    fs,
    io::Write,
    os::unix::ffi::OsStrExt,
    path::PathBuf,
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

struct World {
    root: PathBuf,
    identity: Identity,
    domain: checkout::Domain,
    hold: Option<Token>,
}

impl World {
    fn inspect(&self, directory: &str, args: &[&str]) -> String {
        let output = Command::new("/usr/bin/git")
            .current_dir(self.root.join(directory))
            .env_clear()
            .args(args)
            .output()
            .expect("inspection git ran");
        assert!(output.status.success(), "inspection: {:?}", output.stderr);
        String::from_utf8(output.stdout).expect("git ids and settings")
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
        directory: &str,
        command: &invocation::Command,
        kind: Option<git::Kind>,
        message: &[u8],
    ) -> std::process::Output {
        let kind = if matches!(command, invocation::Command::UpdateRef { .. } | invocation::Command::Clean)
            || (matches!(command, invocation::Command::Unmerged) && kind == Some(git::Kind::Merge))
        {
            None
        } else {
            kind
        };
        let mut process = Command::new("/usr/bin/git");
        process.current_dir(self.root.join(directory)).env_clear();
        for entry in invocation::environment(self.root.as_os_str().as_bytes(), kind, Some(&self.identity)) {
            let at = entry.iter().position(|b| *b == b'=').expect("environment entry");
            process.env(OsStr::from_bytes(&entry[..at]), OsStr::from_bytes(&entry[at + 1..]));
        }
        process.args(invocation::arguments(command).iter().map(|arg| OsStr::from_bytes(arg)));
        process.stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::piped());
        let mut child = process.spawn().expect("git ran");
        child.stdin.take().expect("stdin").write_all(message).expect("message");
        let output = child.wait_with_output().expect("git output");
        if matches!(command, invocation::Command::CheckOut { .. } | invocation::Command::Merge { .. }) {
            assert!(
                output.status.success(),
                "lazy-fetch invocation failed: {}",
                String::from_utf8_lossy(&output.stderr)
            );
        }
        output
    }

    fn perform(&self, mut op: git::Op) -> git::Done {
        if let git::Op::Make { workspace, .. } = &op {
            fs::create_dir_all(self.root.join(format!("workspaces/{}", workspace.raw()))).expect("workspace");
            return git::Done::Succeeded;
        }
        if let git::Op::Store { store, remote, .. } = &op {
            let directory = format!("stores/{}", store.raw());
            fs::create_dir_all(self.root.join(&directory)).expect("store directory");
            let output = self.command(
                "stores",
                &invocation::Command::Store {
                    directory: store.raw().to_string().into_bytes().into(),
                    remote: remote.clone(),
                },
                Some(git::Kind::Store),
                b"",
            );
            assert!(output.status.success(), "store init: {:?}", output.stderr);
            return git::Done::Succeeded;
        }
        let at = match &op {
            git::Op::Clone { at, .. }
            | git::Op::Point { at, .. }
            | git::Op::Fetch { at, .. }
            | git::Op::Create { at, .. }
            | git::Op::CheckOut { at, .. }
            | git::Op::Look { at, .. }
            | git::Op::DiffPaths { at, .. }
            | git::Op::IsAncestor { at, .. }
            | git::Op::Merge { at, .. }
            | git::Op::Commit { at, .. }
            | git::Op::Push { at, .. } => at,
            git::Op::Store { .. } | git::Op::Make { .. } => unreachable!(),
        };
        let repository = String::from_utf8(invocation::place(at).into_vec()).expect("fixture directory");
        let workspace = at.workspace;
        let mut next = sequence::first(&op);
        for _ in 0..20 {
            next = match next {
                Next::Run(command) => {
                    let message = if let (invocation::Command::CommitTree { .. }, git::Op::Commit { title, body, .. }) =
                        (&command, &op)
                    {
                        invocation::message(title, body)
                    } else {
                        Box::default()
                    };
                    let kind = if matches!(command, invocation::Command::SnapshotFetch { .. }) {
                        Some(git::Kind::Fetch)
                    } else {
                        Some(op.kind())
                    };
                    let directory = match (&op, &command) {
                        (git::Op::Clone { .. }, invocation::Command::Clone { .. }) => {
                            format!("workspaces/{}", workspace.raw())
                        }
                        (git::Op::Clone { store, .. }, invocation::Command::Format)
                        | (git::Op::Fetch { store, .. }, invocation::Command::Fetch { .. }) => {
                            format!("stores/{}", store.raw())
                        }
                        _ => repository.clone(),
                    };
                    let output = self.command(&directory, &command, kind, &message);
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
                Next::End(done) => return done,
                Next::Alternates => {
                    let git::Op::Clone { store, remote, .. } = &op else { unreachable!() };
                    fs::write(
                        self.root.join(&repository).join(".git/objects/info/alternates"),
                        format!("../../../../../stores/{}/objects\n", store.raw()),
                    )
                    .expect("alternates");
                    Next::Run(invocation::Command::Remote { remote: remote.clone() })
                }
                Next::Make | Next::Store | Next::InstallStore { .. } | Next::Scan { .. } => {
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
    reason = "one Forgejo scenario proves blob-less stores and both authenticated lazy-fetch paths"
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
        let spec = checkout::Spec {
            key: b"delivery".as_slice().into(),
            repositories: Box::new([checkout::Repository {
                name: b"specimen".as_slice().into(),
                remote: remote.clone(),
                git: None,
                start: checkout::Start::Base { branch: push.into() },
                identity: 1,
                account: None,
                push: Some(push.into()),
                expected: None,
                landing: Some(b"goal".as_slice().into()),
                save: None,
            }]),
        };
        let checkout::Request::Prepared { prepared: checkout::Prepared::Ready { workspace, .. }, .. } =
            world.finish(checkout::Event::Prepare { client: Token::new(1), spec })
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
    instance.stop().expect("stopped");
}
