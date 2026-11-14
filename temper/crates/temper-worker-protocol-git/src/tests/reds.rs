//! Typed git commands and kernel file steps against scratch repositories.
use super::fixtures;
use crate::{
    files::{Action, Directories, Task},
    invocation,
    sequence::{self, Next},
};
use skein_io::kernel::{self, Error, Fd, Op};
use skein_lib::{List, Token};
use std::{
    ffi::OsStr,
    fs,
    io::{Seek, SeekFrom, Write},
    os::unix::ffi::OsStrExt,
    path::{Path, PathBuf},
    process::Command,
};
use temper_worker_domain_checkout::git;
fn git(at: &Path, args: &[&[u8]]) -> Box<[u8]> {
    let mut command = Command::new("git");
    command.current_dir(at).env_clear().args(["-c", "user.name=Temper", "-c", "user.email=temper@example.test"]);
    for arg in args {
        command.arg(OsStr::from_bytes(arg));
    }
    let out = command.output().expect("scratch git");
    assert!(out.status.success(), "git failed: {:?}", out.stderr);
    out.stdout.into_boxed_slice()
}
fn commit(at: &Path, message: &str) -> git::Commit {
    git(at, &[b"commit", b"--quiet", b"--allow-empty", b"-m", message.as_bytes()]);
    invocation::object(git(at, &[b"rev-parse", b"HEAD"]).strip_suffix(b"\n").expect("line")).expect("commit")
}
fn path(root: &Path, bytes: &[u8]) -> PathBuf {
    root.join(OsStr::from_bytes(bytes))
}
fn error(error: std::io::Error) -> Error {
    if error.kind() == std::io::ErrorKind::NotFound {
        Error::NotFound
    } else if error.kind() == std::io::ErrorKind::AlreadyExists {
        Error::Exists
    } else {
        Error::Permission
    }
}
fn completed(result: std::io::Result<()>) -> Result<kernel::Done, Error> {
    match result {
        Ok(()) => Ok(kernel::Done::Count(0)),
        Err(why) => Err(error(why)),
    }
}
fn under(roots: &List<(Fd, PathBuf)>, fd: Fd, name: &[u8]) -> PathBuf {
    for (key, root) in roots {
        if *key == fd {
            return path(root, name);
        }
    }
    panic!("open directory");
}
fn file_steps(root: &Path, directory: Box<[u8]>, content: Option<Box<[u8]>>, marker: bool, fail: bool) -> git::Done {
    let settings = fixtures::settings();
    let mut dirs = Directories::new(1);
    let mut task = Task::reds(directory, content, marker);
    let mut action = task.first(&mut dirs, &settings);
    let mut file = None;
    let mut roots = List::uncapped();
    roots.push((settings.root, root.to_path_buf())).unwrap();
    let mut next_fd = 10_i32;
    for _ in 0..4096_u32 {
        match action {
            Action::Run(op) => {
                assert!(op.is_valid(), "valid kernel operation: {op:?}");
                let result = if let Op::Remove { dir, name, .. } = &op {
                    completed(fs::remove_file(under(&roots, *dir, name)))
                } else if let Op::MakeDirectory { dir, name, .. } = &op {
                    completed(fs::create_dir(under(&roots, *dir, name)))
                } else if let Op::Open { root: dir, path: name, how } = &op {
                    assert!(!name.starts_with(b"/"));
                    let place = under(&roots, *dir, name);
                    let fd = Fd::new(next_fd);
                    next_fd = next_fd.checked_add(1).unwrap();
                    if let kernel::OpenHow::DirectoryNoFollow = how {
                        assert!(place.is_dir());
                        roots.push((fd, place)).unwrap();
                        Ok(kernel::Done::Fd(fd))
                    } else {
                        match fs::OpenOptions::new().write(true).create_new(true).open(place) {
                            Ok(opened) => {
                                file = Some(opened);
                                Ok(kernel::Done::Fd(fd))
                            }
                            Err(why) => Err(error(why)),
                        }
                    }
                } else if let Op::Write { bytes, from, at, .. } = &op {
                    if fail {
                        Err(Error::Permission)
                    } else {
                        let opened = file.as_mut().expect("opened file");
                        opened.seek(SeekFrom::Start(*at)).unwrap();
                        let from = usize::try_from(*from).unwrap();
                        let end = from.saturating_add(7).min(bytes.len());
                        let count = opened.write(bytes.get(from..end).unwrap()).unwrap();
                        Ok(kernel::Done::Count(u32::try_from(count).unwrap()))
                    }
                } else if let Op::Close { .. } = &op {
                    drop(file.take());
                    Ok(kernel::Done::Count(0))
                } else if let Op::Rename { from_dir, from, to_dir, to, .. } = &op {
                    completed(fs::rename(under(&roots, *from_dir, from), under(&roots, *to_dir, to)))
                } else {
                    panic!("unexpected file operation: {op:?}")
                };
                action = task.complete(
                    kernel::Complete { op: Token::new(1), kind: op, result },
                    &mut dirs,
                    &settings,
                    &fixtures::limits(),
                    false,
                );
            }
            Action::Operation(Next::End(done)) => return done,
            Action::Operation(_) | Action::Layout(_) | Action::Swept(_) | Action::Closed => {
                panic!("check files finish")
            }
        }
    }
    panic!("file operation settles");
}
fn perform(root: &Path, at: &git::Place, action: git::Reds, fail: bool) -> git::Done {
    let op = git::Op::Reds { at: at.clone(), action };
    let mut next = sequence::first(&op);
    for _ in 0..512_u32 {
        next = match next {
            Next::Run(command) => {
                let mut process = Command::new("git");
                process.current_dir(path(root, &invocation::place(at))).env_clear();
                for arg in invocation::arguments(&command) {
                    process.arg(OsStr::from_bytes(&arg));
                }
                let out = process.output().expect("typed invocation");
                sequence::after(
                    &op,
                    &command,
                    u8::try_from(out.status.code().unwrap()).unwrap(),
                    &out.stdout,
                    &out.stderr,
                    0,
                    &fixtures::checkout(),
                )
            }
            Next::Reds { directory, content, marker } => return file_steps(root, directory, content, marker, fail),
            Next::End(done) => return done,
            Next::Snapshot { .. }
            | Next::Make
            | Next::Store
            | Next::InstallStore { .. }
            | Next::Alternates
            | Next::Scan { .. } => panic!("check operation"),
        };
    }
    panic!("check settles");
}
fn red(key: &[u8], head: git::Commit) -> git::Red {
    git::Red { key: key.into(), head: Box::from(head.raw().get(..20).unwrap()) }
}
#[test]
fn real_git_filters_the_pinned_base_and_installs_whole_files_in_git_paths() {
    let scratch = skein_scratch::Scratch::new_in(&skein_scratch::paths::target_tmp(), "worker-reds");
    let root = scratch.path();
    let at = git::Place { workspace: Token::new(7), repository: b"specimen".as_slice().into(), git: None };
    let repo = path(root, &invocation::place(&at));
    fs::create_dir_all(&repo).unwrap();
    git(&repo, &[b"init", b"-q"]);
    let parent = commit(&repo, "parent");
    let base = commit(&repo, "base");
    let after = commit(&repo, "after");
    git(&repo, &[b"update-ref", b"refs/temper/landing", &invocation::hex(base)]);
    fs::create_dir_all(repo.join(".git/temper")).unwrap();
    fs::write(repo.join(".git/temper/reds"), b"stale").unwrap();
    assert_eq!(perform(root, &at, git::Reds::Prepare { wait_ms: 6000 }, false), git::Done::Succeeded);
    assert!(!repo.join(".git/temper/reds").exists());
    assert_eq!(fs::read(repo.join(".git/temper/reds-wait")).unwrap(), b"temper-reds-wait 1 6000\n");
    let action = git::Reds::Write {
        reds: Box::new([
            red(b"test d::t base", base),
            red(b"test b::t after", after),
            red(b"test a::t parent", parent),
            red(b"test c::t missing", git::Commit::new([99; 32])),
            git::Red { key: b"test c::t wrong-width".as_slice().into(), head: Box::new(parent.raw()) },
            red(b"test a::t parent", parent),
        ]),
    };
    assert_eq!(perform(root, &at, action.clone(), false), git::Done::Succeeded);
    assert_eq!(
        fs::read(repo.join(".git/temper/reds")).unwrap(),
        b"temper-reds 1\n- test a::t parent\n- test d::t base\n"
    );
    assert!(!repo.join(".git/temper/reds.tmp").exists());
    assert_eq!(perform(root, &at, git::Reds::Clear, false), git::Done::Succeeded);
    assert_eq!(perform(root, &at, action.clone(), true), git::Done::Failed { fault: git::Fault::Broken });
    assert!(!repo.join(".git/temper/reds").exists());
    assert!(!repo.join(".git/temper/reds.tmp").exists());
    let linked = root.join("workspaces/7/linked");
    git(&repo, &[b"worktree", b"add", b"-q", b"--detach", linked.as_os_str().as_bytes(), b"HEAD"]);
    let linked_at = git::Place { repository: b"linked".as_slice().into(), ..at.clone() };
    assert_eq!(perform(root, &linked_at, action, false), git::Done::Succeeded);
    let git_path = git(&linked, &[b"rev-parse", b"--git-path", b"temper/reds"]);
    assert_eq!(
        fs::read(path(&linked, git_path.strip_suffix(b"\n").unwrap())).unwrap(),
        b"temper-reds 1\n- test a::t parent\n- test d::t base\n"
    );
    git(&repo, &[b"checkout", b"-q", b"--detach", &invocation::hex(parent)]);
    let side = commit(&repo, "side");
    let landing = commit(&repo, "landing");
    git(&repo, &[b"update-ref", b"refs/temper/landing", &invocation::hex(landing)]);
    git(&repo, &[b"checkout", b"-q", b"--detach", &invocation::hex(after)]);
    fs::write(repo.join(".git/MERGE_HEAD"), invocation::joined(&[&invocation::hex(side), b"\n"])).unwrap();
    assert_eq!(
        perform(
            root,
            &at,
            git::Reds::Write { reds: Box::new([red(b"test a::t side", side), red(b"test b::t base", base)]) },
            false
        ),
        git::Done::Succeeded
    );
    assert_eq!(fs::read(repo.join(".git/temper/reds")).unwrap(), b"temper-reds 1\n- test a::t side\n");
    fs::remove_file(repo.join(".git/MERGE_HEAD")).unwrap();
    git(&repo, &[b"checkout", b"-q", b"--orphan", b"unrelated"]);
    assert_ne!(commit(&repo, "unrelated"), parent);
    assert_eq!(
        perform(root, &at, git::Reds::Write { reds: Box::new([red(b"test a::t parent", parent)]) }, false),
        git::Done::Succeeded
    );
    assert_eq!(fs::read(repo.join(".git/temper/reds")).unwrap(), b"temper-reds 1\n");
    perform(root, &at, git::Reds::Clear, false);
    git(&repo, &[b"update-ref", b"-d", b"refs/temper/landing"]);
    fs::remove_file(repo.join(".git/temper/reds-wait")).unwrap();
    assert_eq!(perform(root, &at, git::Reds::Prepare { wait_ms: 6000 }, false), git::Done::Succeeded);
    assert!(!repo.join(".git/temper/reds-wait").exists());
    assert!(!repo.join(".git/temper/reds").exists());
    let wide_at = git::Place { repository: b"sha256".as_slice().into(), ..at };
    let wide = path(root, &invocation::place(&wide_at));
    fs::create_dir_all(&wide).unwrap();
    git(&wide, &[b"init", b"-q", b"--object-format=sha256"]);
    let wide_head = commit(&wide, "wide");
    git(&wide, &[b"update-ref", b"refs/temper/landing", &invocation::hex(wide_head)]);
    let mut missing = wide_head.raw();
    *missing.last_mut().unwrap() ^= 1;
    let action = git::Reds::Write {
        reds: Box::new([
            git::Red { key: b"test a::t wide".as_slice().into(), head: Box::new(wide_head.raw()) },
            git::Red { key: b"test b::t missing-tail".as_slice().into(), head: Box::new(missing) },
        ]),
    };
    assert_eq!(perform(root, &wide_at, action, false), git::Done::Succeeded);
    assert_eq!(fs::read(wide.join(".git/temper/reds")).unwrap(), b"temper-reds 1\n- test a::t wide\n");
}
