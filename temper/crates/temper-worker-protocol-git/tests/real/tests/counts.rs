//! The run's counts blob through both sets of alternates (protocol/git-operations.md, 1.3 and 1.13).
use std::{
    ffi::OsStr,
    fs,
    io::Write as _,
    os::unix::ffi::OsStrExt,
    path::Path,
    process::{Command, Output, Stdio},
};

use skein_lib::{Token, bytes};
use temper_worker_domain_checkout::git;
use temper_worker_protocol_git::{
    invocation,
    sequence::{self, Next},
    testing::fixtures,
};

const REF: &[u8] = b"refs/temper/counts";
const ABSENT: &[u8] = b"fatal: couldn't find remote ref refs/temper/counts\n";

fn git(at: &Path, words: &[&[u8]], input: &[u8]) -> Output {
    let mut command = Command::new("git");
    command.current_dir(at).env_clear().args(["-c", "user.name=Temper", "-c", "user.email=temper@example.test"]);
    for word in words {
        command.arg(OsStr::from_bytes(word));
    }
    let mut child = command.stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::piped()).spawn().unwrap();
    child.stdin.take().unwrap().write_all(input).unwrap();
    child.wait_with_output().unwrap()
}

fn ok(out: Output) -> Box<[u8]> {
    assert!(out.status.success(), "git stderr: {:?}", out.stderr);
    out.stdout.into_boxed_slice()
}

fn id(out: Output) -> Box<[u8]> {
    Box::from(ok(out).strip_suffix(b"\n").expect("object id line"))
}

fn publish(forge: &Path, reference: &[u8], name: &[u8], bytes: &[u8]) -> git::Commit {
    let blob = id(git(forge, &[b"hash-object", b"-w", b"--stdin"], bytes));
    let entry = bytes::concat(&[b"100644 blob ", &blob, b"\t", name, b"\n"]);
    let tree = id(git(forge, &[b"mktree"], &entry));
    let commit = id(git(forge, &[b"commit-tree", &tree, b"-m", b"publish"], b""));
    ok(git(forge, &[b"update-ref", reference, &commit], b""));
    invocation::object(&commit).unwrap()
}

fn invoke(root: &Path, op: &git::Op, command: &invocation::Command) -> Output {
    // A scratch file transport replaces HTTP; production argv is unchanged.
    let mut git = Command::new("git");
    git.current_dir(root).env_clear().env("LC_ALL", "C").args(["-c", "protocol.file.allow=always"]);
    for arg in invocation::arguments_for(op, command) {
        git.arg(OsStr::from_bytes(&arg));
    }
    git.output().unwrap()
}

fn after(op: &git::Op, command: &invocation::Command, out: &Output) -> Next {
    sequence::after(
        op,
        command,
        u8::try_from(out.status.code().unwrap()).unwrap(),
        &out.stdout,
        &out.stderr,
        0,
        &fixtures::checkout(),
    )
}

fn counts(root: &Path, at: &git::Place, op: &git::Op, expected: Option<git::Commit>) {
    let Next::Run(fetch) = sequence::first(op) else { panic!("counts fetch") };
    let fetched = invoke(root, op, &fetch);
    let done = match expected {
        Some(commit) => {
            assert!(fetched.status.success(), "counts fetch stderr: {:?}", fetched.stderr);
            git::Done::Fetched { commit }
        }
        None => {
            assert_eq!(fetched.status.code(), Some(128_i32));
            assert_eq!(fetched.stderr, ABSENT);
            git::Done::Succeeded
        }
    };
    let Next::Run(update) = after(op, &fetch, &fetched) else { panic!("counts update: {:?}", fetched.stderr) };
    assert_eq!(after(op, &update, &invoke(root, op, &update)), Next::End(done));
    let name = OsStr::from_bytes(REF);
    let private = root.join(OsStr::from_bytes(&invocation::git_dir(at))).join(name);
    let view = root.join(OsStr::from_bytes(&invocation::place(at))).join(".git").join(name);
    if let Some(commit) = expected {
        fs::create_dir_all(view.parent().unwrap()).unwrap();
        let pointed = fs::read(&private).unwrap();
        assert_eq!(pointed.strip_suffix(b"\n").unwrap(), &*invocation::hex(commit));
        fs::write(view, pointed).unwrap();
    } else {
        assert!(!private.exists());
        if view.exists() {
            fs::remove_file(view).unwrap();
        }
    }
}

#[test]
fn counts_can_be_read_offline_after_publish_replacement_and_removal() {
    let scratch = skein_scratch::Scratch::new_in(&skein_scratch::paths::target_tmp(), "worker-counts");
    let root = scratch.path();
    let forge = root.join("forge");
    let at = git::Place { workspace: Token::new(7), repository: b"specimen".as_slice().into(), git: None };
    let repo = root.join(OsStr::from_bytes(&invocation::place(&at)));
    let private = root.join(OsStr::from_bytes(&invocation::git_dir(&at)));
    let store = Token::new(9);
    let remote = bytes::concat(&[b"file://", forge.as_os_str().as_bytes()]);
    ok(git(root, &[b"init", b"--bare", b"-q", b"--initial-branch=main", b"forge"], b""));
    ok(git(&forge, &[b"config", b"uploadpack.allowFilter", b"true"], b""));
    publish(&forge, b"refs/heads/main", b"README", b"the main branch\n");
    let op = git::Op::Counts { at: at.clone(), store, remote: remote.clone(), identity: 1, account: None };
    let store_command = invocation::Command::Store { directory: b"stores/9".as_slice().into(), remote: remote.clone() };
    ok(invoke(root, &op, &store_command));
    let main_blob = id(git(&forge, &[b"rev-parse", b"main:README"], b""));
    assert!(
        !Command::new("git")
            .current_dir(root.join("stores/9"))
            .env_clear()
            .env("GIT_NO_LAZY_FETCH", "1")
            .args([OsStr::new("cat-file"), OsStr::new("-e"), OsStr::from_bytes(&main_blob)])
            .output()
            .unwrap()
            .status
            .success(),
        "the store begins blob-less",
    );
    fs::create_dir_all(repo.parent().unwrap()).unwrap();
    let clone = git::Op::Clone { at: at.clone(), remote: remote.clone(), store };
    let format = invocation::Format::Sha1;
    ok(invoke(root, &clone, &invocation::Command::Clone { directory: invocation::git_dir(&at), format, bare: true }));
    fs::write(private.join("objects/info/alternates"), b"../../../../stores/9/objects\n").unwrap();
    ok(invoke(root, &clone, &invocation::Command::Remote { remote, format }));
    ok(invoke(root, &clone, &invocation::Command::Clone { directory: invocation::place(&at), format, bare: false }));
    fs::write(repo.join(".git/objects/info/alternates"), b"../../../../../gitdirs/7/specimen/objects\n").unwrap();

    counts(root, &at, &op, None);
    for bytes in [b"{\"first\":1}\n".as_slice(), b"{\"second\":2}\n"] {
        counts(root, &at, &op, Some(publish(&forge, REF, b"counts.json", bytes)));
        let offline = forge.with_extension("offline");
        fs::rename(&forge, &offline).unwrap();
        let shown = Command::new("git")
            .current_dir(&repo)
            .env_clear()
            .env("GIT_NO_LAZY_FETCH", "1")
            .args(["show", "refs/temper/counts:counts.json"])
            .output()
            .unwrap();
        assert!(shown.status.success(), "offline git show stderr: {:?}", shown.stderr);
        assert_eq!(shown.stdout, bytes);
        fs::rename(offline, &forge).unwrap();
    }
    ok(git(&forge, &[b"update-ref", b"-d", REF], b""));
    counts(root, &at, &op, None);
    assert!(!repo.join(".git/refs/temper/counts").exists());
}
