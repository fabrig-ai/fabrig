use temper_worker_checkout_world::hosted::{self as fixtures, Fault};
use temper_worker_domain_checkout::git;
use temper_worker_protocol_git::output;
use temper_worker_protocol_git_world::transcripts;
const CASES: &[(&str, Fault)] = &[
    ("version", Fault::None),
    ("origin-no-repository", Fault::None),
    ("origin-no-remote", Fault::None),
    ("checkout-index-locked", Fault::None),
    ("clone", Fault::None),
    ("clone-sha256", Fault::None),
    ("object-format", Fault::None),
    ("object-format-sha256", Fault::None),
    ("init", Fault::None),
    ("init-sha256", Fault::None),
    ("remote-add", Fault::None),
    ("update-ref", Fault::None),
    ("clone-empty", Fault::None),
    ("fetch-branch", Fault::None),
    ("fetch-commit", Fault::None),
    ("fetch-default", Fault::None),
    ("fetch-up-to-date", Fault::None),
    ("fetch-sha256-branch", Fault::None),
    ("checkout-1", Fault::None),
    ("checkout-2", Fault::None),
    ("merge-clean", Fault::None),
    ("merge-in-progress", Fault::None),
    ("merge-up-to-date", Fault::None),
    ("merge-conflict", Fault::None),
    ("merge-conflict-2", Fault::None),
    ("commit-add", Fault::None),
    ("commit-unchanged", Fault::None),
    ("commit-changed", Fault::None),
    ("commit-write-tree", Fault::None),
    ("commit-tree-one-parent", Fault::None),
    ("commit-tree-two-parents", Fault::None),
    ("commit-unmerged", Fault::None),
    ("create-new-branch", Fault::None),
    ("create-exists-at-commit", Fault::None),
    ("create-exists-elsewhere", Fault::None),
    ("create-protected", Fault::ProtectedBranch),
    ("push-fast-forward", Fault::None),
    ("push-new-branch", Fault::None),
    ("push-up-to-date", Fault::None),
    ("push-non-fast-forward", Fault::None),
    ("push-lease-stale", Fault::None),
    ("push-ancestor-yes", Fault::None),
    ("push-ancestor-no", Fault::None),
    ("push-protected", Fault::ProtectedBranch),
    ("push-no-permission", Fault::NoPermission),
    ("clone-missing-repository", Fault::MissingRepository),
    ("clone-stranger-private", Fault::MissingRepository),
    ("fetch-missing-branch", Fault::MissingBranch),
    ("fetch-missing-commit", Fault::MissingCommit),
    ("clone-refused-connection", Fault::Unreachable),
    ("clone-wrong-token", Fault::WrongToken),
    ("fetch-wrong-token", Fault::WrongToken),
];
#[test]
fn masking_keeps_whole_and_short_fake_ids_equal_and_count_zero_distinct_from_zero_id() {
    let first = temper_fake_git_hosted::id::id(0);
    let second = temper_fake_git_hosted::id::id(1);
    let bytes =
        [first.as_slice(), b" ", &first[..16], b" ", &second, b" 0000000000000000000000000000000000000000"].concat();
    assert_eq!(fixtures::masked(&bytes), b"<id:1> <id:1> <id:2> <zero>");
}
#[test]
fn each_transcript_s_exit_and_masked_stdout_match_the_fake_s() {
    for &(case, fault) in CASES {
        let real = transcripts::read(case);
        let fake = fixtures::answer_case(case, fault);
        assert_eq!(fake.exit, real.exit, "{case}: exit");
        assert_eq!(fixtures::masked(&fake.stdout), fixtures::masked(&real.stdout), "{case}: stdout");
        for bytes in [&fake.stdout, &fake.stderr] {
            assert!(!bytes.windows(fixtures::TOKEN.len()).any(|value| value == fixtures::TOKEN), "{case}: token");
        }
    }
}
#[test]
fn each_failure_transcript_s_stderr_classifies_as_the_fake_s() {
    let rows = [
        ("checkout-index-locked", Fault::None, git::Kind::CheckOut),
        ("merge-in-progress", Fault::None, git::Kind::Merge),
        ("clone-missing-repository", Fault::MissingRepository, git::Kind::Store),
        ("clone-stranger-private", Fault::MissingRepository, git::Kind::Store),
        ("fetch-missing-branch", Fault::MissingBranch, git::Kind::Fetch),
        ("fetch-missing-commit", Fault::MissingCommit, git::Kind::Fetch),
        ("fetch-wrong-token", Fault::WrongToken, git::Kind::Store),
        ("clone-wrong-token", Fault::WrongToken, git::Kind::Store),
        ("clone-refused-connection", Fault::Unreachable, git::Kind::Store),
        ("push-protected", Fault::ProtectedBranch, git::Kind::Push),
    ];
    for (case, fault, kind) in rows {
        let real = transcripts::read(case);
        let fake = fixtures::answer_case(case, fault);
        assert_eq!(fake.exit, real.exit, "{case}: exit");
        assert_eq!(output::classify(&fake.stderr, kind), output::classify(&real.stderr, kind), "{case}: stderr");
    }
}
#[test]
fn a_wrong_token_is_answered_as_forgejo_answers_it() {
    let fake = fixtures::answer_case("fetch-wrong-token", Fault::WrongToken);
    assert_eq!(fake.exit, 128);
    assert!(fake.stderr.ends_with(b"The requested URL returned error: 401\n"));
    assert_eq!(output::classify(&fake.stderr, git::Kind::Fetch), git::Fault::Refused);
}
#[test]
fn cloning_an_unborn_repository_imports_no_local_objects() {
    use temper_fake_git_hosted::{answer, check};
    use temper_worker_protocol_git::invocation::Command;
    let mut scenario = fixtures::named(b"fixture/empty", None);
    let spawn = fixtures::spawn_of(&Command::Store {
        remote: scenario.remote.clone().into(),
        directory: b"empty".as_slice().into(),
    });
    let checked = check(&spawn).expect("empty clone spawn");
    let fake = answer(&checked, &mut scenario.checkout, &mut scenario.transport);
    assert_eq!(fake.exit, 0);
    assert!(scenario.checkout.exists(b"workspaces/1/app/empty"));
    assert!(scenario.checkout.tree(b"workspaces/1/app/empty/objects").is_empty());
    assert!(scenario.transport.forge.borrow().domain.object(1).is_none());
}

use skein_fake_machine::{How, Is, Item, Machine, Opened};
use skein_io::kernel::{self as k, Done, Exit, Fd, Op, OpenHow};
use skein_lib::{Time, Wall};
use skein_shell::Host;
use skein_sim_world::Inherited;
use std::collections::BTreeSet;
use temper_fake_git_hosted::Answer;
use temper_worker_protocol_git::invocation::{self, Command};

#[derive(Debug)]
struct Disk {
    machine: Machine,
    root: Opened,
}
impl Disk {
    fn new(scenario: &fixtures::Scenario) -> Self {
        let mut directories = BTreeSet::new();
        let mut files = Vec::new();
        for (path, bytes) in scenario.checkout.files() {
            let mut parent = path;
            while let Some(slash) = parent.iter().rposition(|b| *b == b'/') {
                parent = &parent[..slash];
                directories.insert(parent.to_vec());
            }
            files.push(Item::file(path, bytes));
        }
        let mut items: Vec<_> = directories.iter().map(|path| Item::directory(path)).collect();
        items.extend(files);
        let mut machine = Machine::new();
        let root = machine.lay(&items);
        Self { machine, root }
    }
    fn read(&mut self, at: &[u8], name: &[u8]) -> Vec<u8> {
        let file = self.machine.open(self.root, &[at, b"/", name].concat(), How::ReadNoFollow).expect("file");
        let bytes = self.machine.read(file, 0, 16_384).expect("read");
        self.machine.close(file);
        bytes
    }
    fn run(&mut self, scenario: &fixtures::Scenario, command: &Command, input: &[u8]) -> Answer {
        let mut spawn = fixtures::spawn_of(command);
        spawn.dir = scenario.at.clone().into();
        let inherited = Inherited {
            roots: vec![(b"worker".as_slice().into(), descriptor(self.machine.reopen_root(0)))],
            appends: Vec::new(),
            pipes: vec![(0, Fd::new(10_000)), (1, Fd::new(10_001)), (2, Fd::new(10_002))],
            signal: Fd::new(10_003),
        };
        let mut held: BTreeSet<_> = inherited.pipes.iter().map(|(_, fd)| *fd).collect();
        held.insert(inherited.signal);
        let mut host = temper_fake_git_hosted::git(&spawn, &inherited, scenario.transport.clone());
        let mut answer = Answer { exit: 0, stdout: Vec::new(), stderr: Vec::new() };
        let mut input = input;
        let now = scenario.transport.forge.borrow().now;
        for _ in 0..20_000 {
            host.iterate(now, Wall::EPOCH);
            if let Some(Exit::Code(code)) = host.exit() {
                answer.exit = code;
                assert!(held.is_empty(), "inherited pipes and signals closed");
                assert_eq!(self.machine.open_handles(), 1, "only the test's root remains");
                return answer;
            }
            let submit = host.submissions().pop().expect("a live host progresses");
            let mut kind = submit.kind;
            let result = match &mut kind {
                Op::PipeRead { fd, buf } => {
                    assert!(held.contains(fd));
                    let count = input.len().min(buf.len()).min(7);
                    buf[..count].copy_from_slice(&input[..count]);
                    input = &input[count..];
                    Ok(Done::Count(u32::try_from(count).expect("bounded")))
                }
                Op::PipeWrite { fd, bytes, from } => {
                    assert!(held.contains(fd));
                    let bytes = &bytes[usize::try_from(*from).expect("offset")..];
                    let count = bytes.len().min(7);
                    match fd.raw() {
                        10_001 => answer.stdout.extend(&bytes[..count]),
                        10_002 => answer.stderr.extend(&bytes[..count]),
                        other => panic!("output pipe {other}"),
                    }
                    Ok(Done::Count(u32::try_from(count).expect("bounded")))
                }
                Op::Close { fd } if fd.raw() >= 10_000 => {
                    assert!(held.remove(fd), "a descriptor closes once");
                    Ok(Done::Nothing)
                }
                op @ (Op::Open { .. }
                | Op::Read { .. }
                | Op::Write { .. }
                | Op::Truncate { .. }
                | Op::MakeDirectory { .. }
                | Op::Remove { .. }
                | Op::List { .. }
                | Op::Close { .. }) => self.file(op).map_err(skein_fake_machine::kernel_error),
                other @ (Op::Lock { .. }
                | Op::Socket { .. }
                | Op::Bind { .. }
                | Op::Listen { .. }
                | Op::Accept { .. }
                | Op::Connect { .. }
                | Op::Recv { .. }
                | Op::Send { .. }
                | Op::Shutdown { .. }
                | Op::Append { .. }
                | Op::Sync { .. }
                | Op::Stat { .. }
                | Op::Rename { .. }
                | Op::Spawn { .. }
                | Op::Wait { .. }
                | Op::Signal { .. }
                | Op::Usage
                | Op::Window { .. }
                | Op::ReadSignal { .. }
                | Op::Cancel { .. }) => panic!("git operation {other:?}"),
            };
            host.completions().push(k::Complete { op: submit.op, kind, result });
        }
        panic!("git settles within its operation bound");
    }
    fn file(&mut self, op: &mut Op) -> Result<Done, skein_fake_machine::Refusal> {
        match op {
            Op::Open { root, path, how } => {
                let how = match how {
                    OpenHow::DirectoryNoFollow => How::DirectoryNoFollow,
                    OpenHow::ReadNoFollow => How::ReadNoFollow,
                    OpenHow::WriteNoFollow => How::WriteNoFollow,
                    OpenHow::CreateNoFollow { mode } => How::CreateNoFollow { mode: mode.unwrap_or(0o600) },
                    other @ (OpenHow::PathNoFollow
                    | OpenHow::Read
                    | OpenHow::Directory
                    | OpenHow::Create { .. }
                    | OpenHow::Write) => panic!("git open {other:?}"),
                };
                self.machine.open(opened(*root), path, how).map(|fd| Done::Fd(descriptor(fd)))
            }
            Op::List { fd, entries, names } => {
                let listed = self.machine.list(opened(*fd), 8, 2040)?;
                let mut start = 0;
                for (slot, (kind, name)) in entries.iter_mut().zip(&listed) {
                    names[start..start + name.len()].copy_from_slice(name);
                    *slot = k::Entry {
                        kind: match kind {
                            Is::File => k::Kind::File,
                            Is::Directory => k::Kind::Directory,
                            Is::Link => k::Kind::Symlink,
                            Is::Fifo | Is::Device => k::Kind::Other,
                        },
                        start: u32::try_from(start).expect("bounded"),
                        len: u32::try_from(name.len()).expect("bounded"),
                    };
                    start += name.len();
                }
                Ok(Done::Count(u32::try_from(listed.len()).expect("bounded")))
            }
            Op::Read { fd, buf, at } => {
                let bytes = self.machine.read(opened(*fd), *at, 1024)?;
                buf[..bytes.len()].copy_from_slice(&bytes);
                Ok(Done::Count(u32::try_from(bytes.len()).expect("bounded")))
            }
            Op::Write { fd, bytes, from, at } => {
                let bytes = &bytes[usize::try_from(*from).expect("offset")..];
                let count = bytes.len().min(127);
                self.machine.write(opened(*fd), *at, &bytes[..count])?;
                Ok(Done::Count(u32::try_from(count).expect("bounded")))
            }
            Op::Truncate { fd, length } => self.machine.truncate(opened(*fd), *length).map(|()| Done::Nothing),
            Op::MakeDirectory { dir, name, mode } => {
                self.machine.make_directory(opened(*dir), name, *mode).map(|()| Done::Nothing)
            }
            Op::Remove { dir, name, directory } => {
                self.machine.remove(opened(*dir), name, *directory).map(|()| Done::Nothing)
            }
            Op::Close { fd } => {
                self.machine.close(opened(*fd));
                Ok(Done::Nothing)
            }
            other @ (Op::Lock { .. }
            | Op::Socket { .. }
            | Op::Bind { .. }
            | Op::Listen { .. }
            | Op::Accept { .. }
            | Op::Connect { .. }
            | Op::Recv { .. }
            | Op::Send { .. }
            | Op::Shutdown { .. }
            | Op::Append { .. }
            | Op::Sync { .. }
            | Op::Stat { .. }
            | Op::Rename { .. }
            | Op::Spawn { .. }
            | Op::Wait { .. }
            | Op::Signal { .. }
            | Op::Usage
            | Op::Window { .. }
            | Op::ReadSignal { .. }
            | Op::PipeRead { .. }
            | Op::PipeWrite { .. }
            | Op::Cancel { .. }) => panic!("git file operation {other:?}"),
        }
    }
}
impl Drop for Disk {
    fn drop(&mut self) {
        self.machine.close(self.root);
    }
}
fn commit(number: u64) -> temper_worker_domain_checkout::git::Commit {
    invocation::object(&temper_fake_git_hosted::id::id(number)).expect("counted id")
}

#[test]
fn hosted_git_stages_files_and_commits_the_requested_parents_without_moving_head() {
    let mut scenario = fixtures::scenario();
    scenario.transport.forge.borrow_mut().config.limits.body_bytes = 8192;
    let path = |name: &[u8]| [scenario.at.as_slice(), b"/", name].concat();
    scenario.checkout.write(&path(b"conflict.txt"), &vec![b'x'; 5000]);
    scenario.checkout.write(&path(b".git/MERGE_HEAD"), &2_u64.to_le_bytes());
    for number in 0..12 {
        scenario.checkout.write(&path(format!("new-{number}").as_bytes()), b"new\n");
    }
    let mut disk = Disk::new(&scenario);
    assert_eq!(disk.run(&scenario, &Command::Add, b"").exit, 0);
    let tree = disk.run(&scenario, &Command::WriteTree, b"");
    assert_eq!(tree.exit, 0);
    let tree = invocation::object(tree.stdout.strip_suffix(b"\n").expect("one id")).expect("tree id");
    let file = disk.machine.open(disk.root, &path(b"conflict.txt"), How::WriteNoFollow).expect("working file");
    disk.machine.truncate(file, 0).expect("truncate working file");
    disk.machine.write(file, 0, b"unstaged\n").expect("edit after staging");
    disk.machine.close(file);
    let command = Command::CommitTree { tree, parent: commit(1), merging: Some(commit(2)) };
    let message = vec![b'm'; 4100];
    let result = disk.run(&scenario, &command, &message);
    assert_eq!(result.exit, 0);
    assert_eq!(result.stdout, b"1d0b14e4db018fed000000000000000300000000\n");
    assert!(result.stderr.is_empty());
    let forge = scenario.transport.forge.borrow();
    let object = forge.domain.object(3).expect("new commit");
    assert_eq!((object.parent, object.merge_parent), (Some(1), Some(2)));
    assert_eq!(object.tree.get(b"conflict.txt".as_slice()).expect("staged file").as_ref(), vec![b'x'; 5000]);
    assert_eq!(object.tree.len(), 13);
    assert_eq!(object.message.as_ref(), message);
    assert_eq!(disk.read(&scenario.at, b".git/temper-head"), 1_u64.to_le_bytes());
    assert_eq!(disk.read(&scenario.at, b".git/MERGE_HEAD"), 2_u64.to_le_bytes());
    assert_eq!(disk.read(&scenario.at, b".git/objects/3"), b"");
}

#[test]
fn hosted_git_stages_no_hidden_log() {
    for (merging, new_work) in [(None, true), (Some(2), true), (None, false)] {
        let mut scenario = fixtures::scenario();
        let parent = scenario.transport.commit_tree(
            scenario.base,
            None,
            [(b"conflict.txt".to_vec(), b"base\n".to_vec()), (b"sub/.tracked.log".to_vec(), b"parent log\n".to_vec())]
                .into(),
            b"tracked log\n",
        );
        let path = |name: &[u8]| [scenario.at.as_slice(), b"/", name].concat();
        scenario.checkout.write(&path(format!(".git/objects/{parent}").as_bytes()), b"");
        scenario.checkout.write(&path(b".git/temper-index/sub/.tracked.log"), b"parent log\n");
        for name in [b".a.log".as_slice(), b"sub/.b.log", b"sub/.tracked.log"] {
            scenario.checkout.write(&path(name), b"tool output\n");
        }
        if new_work {
            scenario.checkout.write(&path(b"keep.log"), b"new log\n");
            scenario.checkout.write(&path(b"sub/work.txt"), b"new work\n");
            scenario.checkout.write(&path(b".log"), b"short name\n");
            scenario.checkout.write(&path(b".directory.log/work.txt"), b"nested work\n");
        }
        let mut disk = Disk::new(&scenario);
        assert_eq!(disk.run(&scenario, &Command::Add, b"").exit, 0);
        let difference = disk.run(&scenario, &Command::DiffIndex { parent: commit(parent) }, b"");
        if !new_work {
            assert_eq!(difference.exit, 0, "hidden logs alone leave the parent tree unchanged");
            let op = git::Op::Commit {
                always: false,
                snapshot: None,
                at: git::Place { git: None, workspace: skein_lib::Token::new(1), repository: b"app".as_slice().into() },
                parent: commit(parent),
                merging: None,
                title: b"work".as_slice().into(),
                body: Box::new([]),
                identity: 1,
            };
            assert_eq!(
                temper_worker_protocol_git::sequence::after(
                    &op,
                    &Command::DiffIndex { parent: commit(parent) },
                    difference.exit,
                    &difference.stdout,
                    &difference.stderr,
                    0,
                    &temper_worker_protocol_git_world::fixtures::checkout(),
                ),
                temper_worker_protocol_git::sequence::Next::End(git::Done::Unchanged),
                "the commit operation ends without write-tree or commit-tree",
            );
            continue;
        }
        assert_eq!(difference.exit, 1, "new work changes the tree");
        let tree = disk.run(&scenario, &Command::WriteTree, b"");
        assert_eq!(tree.exit, 0);
        let tree = invocation::object(tree.stdout.strip_suffix(b"\n").expect("one tree id")).expect("tree id");
        let result = disk.run(
            &scenario,
            &Command::CommitTree { tree, parent: commit(parent), merging: merging.map(commit) },
            b"work\n",
        );
        assert_eq!(result.exit, 0);
        let forge = scenario.transport.forge.borrow();
        let object = forge.domain.object(parent + 1).expect("new commit");
        assert_eq!((object.parent, object.merge_parent), (Some(parent), merging));
        assert!(!object.tree.contains_key(b".a.log".as_slice()), "root hidden log excluded");
        assert!(!object.tree.contains_key(b"sub/.b.log".as_slice()), "nested hidden log excluded");
        assert_eq!(object.tree.get(b"sub/.tracked.log".as_slice()).expect("tracked log").as_ref(), b"parent log\n");
        assert_eq!(object.tree.get(b"keep.log".as_slice()).expect("ordinary log").as_ref(), b"new log\n");
        assert_eq!(object.tree.get(b"sub/work.txt".as_slice()).expect("new file").as_ref(), b"new work\n");
        assert_eq!(object.tree.get(b".log".as_slice()).expect("short name").as_ref(), b"short name\n");
        assert_eq!(
            object.tree.get(b".directory.log/work.txt".as_slice()).expect("log directory").as_ref(),
            b"nested work\n"
        );
    }
}

#[test]
fn hosted_git_refuses_a_second_merge_until_a_checkout_clears_the_first() {
    let mut scenario = fixtures::scenario();
    let second = scenario.transport.commit_tree(
        scenario.base,
        None,
        [(b"conflict.txt".to_vec(), b"base\n".to_vec()), (b"other".to_vec(), b"second\n".to_vec())].into(),
        b"second head\n",
    );
    scenario.checkout.write(&[scenario.at.as_slice(), format!("/.git/objects/{second}").as_bytes()].concat(), b"");
    let mut disk = Disk::new(&scenario);
    let mut parent = commit(scenario.base);
    for (head, merged) in [(scenario.topic, 4_u64), (second, 5)] {
        let merge = Command::Merge { theirs: commit(head) };
        assert_eq!(disk.run(&scenario, &merge, b"").exit, 0);
        assert_eq!(disk.run(&scenario, &Command::Add, b"").exit, 0);
        let tree = disk.run(&scenario, &Command::WriteTree, b"");
        let tree = invocation::object(tree.stdout.strip_suffix(b"\n").expect("one tree id")).expect("tree id");
        let committed =
            disk.run(&scenario, &Command::CommitTree { tree, parent, merging: Some(commit(head)) }, b"merge head\n");
        assert_eq!(committed.exit, 0);
        let committed =
            invocation::object(committed.stdout.strip_suffix(b"\n").expect("one commit id")).expect("merge commit id");
        assert_eq!(committed, commit(merged));
        assert_eq!(disk.read(&scenario.at, b".git/MERGE_HEAD"), head.to_le_bytes());
        let refused = disk.run(&scenario, &Command::Merge { theirs: commit(second) }, b"");
        assert_eq!(refused.exit, 128);
        assert!(refused.stdout.is_empty());
        assert_eq!(
            refused.stderr,
            b"fatal: You have not concluded your merge (MERGE_HEAD exists).\nPlease, commit your changes before you merge.\n"
        );
        assert_eq!(disk.run(&scenario, &Command::CheckOut { commit: committed }, b"").exit, 0);
        assert_eq!(disk.run(&scenario, &Command::Clean, b"").exit, 0);
        assert_eq!(disk.read(&scenario.at, b".git/temper-head"), merged.to_le_bytes());
        assert_eq!(
            disk.machine.open(disk.root, &[scenario.at.as_slice(), b"/.git/MERGE_HEAD"].concat(), How::PathNoFollow),
            Err(skein_fake_machine::Refusal::NotFound)
        );
        parent = committed;
    }
}

#[test]
fn hosted_checkout_and_clean_replace_and_remove_machine_files() {
    let mut scenario = fixtures::scenario();
    let at = scenario.at.as_slice();
    scenario.checkout.write(&[at, b"/conflict.txt"].concat(), &vec![b'x'; 5000]);
    scenario.checkout.write(&[at, b"/untracked/deep/file"].concat(), b"remove\n");
    scenario.checkout.write(&[at, b"/.git/keep"].concat(), b"metadata\n");
    let mut disk = Disk::new(&scenario);
    assert_eq!(disk.run(&scenario, &Command::CheckOut { commit: commit(2) }, b"").exit, 0);
    assert_eq!(disk.run(&scenario, &Command::Clean, b"").exit, 0);
    assert_eq!(disk.read(at, b"conflict.txt"), b"topic\n");
    assert_eq!(disk.read(at, b".git/keep"), b"metadata\n");
    assert_eq!(
        disk.machine.open(disk.root, &[at, b"/untracked"].concat(), How::PathNoFollow),
        Err(skein_fake_machine::Refusal::NotFound)
    );
}

fn descriptor(fd: Opened) -> Fd {
    Fd::new(i32::try_from(fd.raw()).expect("fixture descriptor"))
}
fn opened(fd: Fd) -> Opened {
    Opened::new(u64::try_from(fd.raw()).expect("fixture descriptor"))
}

#[test]
fn hosted_clone_creates_only_the_destination_without_checking_out_files() {
    let scenario = fixtures::scenario();
    let mut disk = Disk::new(&scenario);
    let command = Command::Store { remote: scenario.remote.clone().into(), directory: b"copy".as_slice().into() };
    let result = disk.run(&scenario, &command, b"");
    assert_eq!(result.exit, 0);
    assert!(result.stdout.is_empty());
    assert!(result.stderr.is_empty());
    assert_eq!(disk.read(&scenario.at, b"copy/objects/1"), b"");
    assert_eq!(disk.read(&scenario.at, b"copy/remote"), scenario.remote);
    assert_eq!(disk.read(&scenario.at, b"conflict.txt"), b"base\n");
    let cloned =
        disk.machine.open(disk.root, &[scenario.at.as_slice(), b"/copy/conflict.txt"].concat(), How::PathNoFollow);
    assert_eq!(cloned, Err(skein_fake_machine::Refusal::NotFound));
}

#[test]
fn hosted_commit_tree_stores_unchanged_trees_with_exactly_the_requested_parents() {
    let scenario = fixtures::scenario();
    let mut disk = Disk::new(&scenario);
    let tree = disk.run(&scenario, &Command::WriteTree, b"");
    let tree = invocation::object(tree.stdout.strip_suffix(b"\n").expect("one tree id")).expect("tree id");
    let rows = [
        (None, b"1d0b14e4db018fed000000000000000300000000\n".as_slice(), 3, None),
        (Some(commit(2)), b"6e73e372e2338aca000000000000000400000000\n".as_slice(), 4, Some(2)),
    ];
    for (merging, expected, number, second) in rows {
        let result = disk.run(&scenario, &Command::CommitTree { tree, parent: commit(1), merging }, b"unchanged\n");
        assert_eq!(result.exit, 0, "commit {number}: exit");
        assert_eq!(result.stdout, expected, "commit {number}: id");
        let forge = scenario.transport.forge.borrow();
        let object = forge.domain.object(number).expect("new unchanged commit");
        assert_eq!((object.parent, object.merge_parent), (Some(1), second), "commit {number}: parents");
        assert_eq!(object.tree.len(), 1, "commit {number}: tree files");
        assert_eq!(
            object.tree.get(b"conflict.txt".as_slice()).expect("base file").as_ref(),
            b"base\n",
            "commit {number}: tree"
        );
        assert_eq!(disk.read(&scenario.at, b".git/temper-head"), 1_u64.to_le_bytes(), "commit {number}: HEAD");
    }
}

// A read left due before the hosted git process starts owns this reply.
const OLDER_READ: skein_lib::Token = skein_lib::Token::new(42);
// Only the older read remains after hosted git consumes its own reply.
const PENDING_READ_REPLIES: u32 = 1;

fn an_older_read_is_due() -> fixtures::Scenario {
    use skein_fake_forge_domain::{self as domain, api};
    use skein_lib::{Duration, Env, ReplyTo, Time, Wall};
    let scenario = fixtures::scenario();
    {
        let mut forge = scenario.transport.forge.borrow_mut();
        let mut config = forge.config.clone();
        config.limits.calls = 2;
        let mut state = domain::Domain::new(&config, 1);
        domain::repository(
            &mut state,
            &config,
            api::Setup {
                id: 2,
                name: b"acme/app".as_slice().into(),
                default: b"main".as_slice().into(),
                tree: Box::new([api::File {
                    path: b"conflict.txt".as_slice().into(),
                    content: b"base\n".as_slice().into(),
                }]),
                labels: Box::new([]),
                workflows: Box::new([]),
                checks: api::Checks {
                    cues: Box::new([]),
                    contexts: Box::new([]),
                    latency_min: Duration::ZERO,
                    latency_max: Duration::ZERO,
                    silent: 0,
                    passes: 1000,
                    reruns: 0,
                    cue: None,
                },
                protection: None,
                hooked: false,
            },
        );
        domain::grant(&mut state, b"acme/app", 1, api::Permission::Write);
        *forge = skein_fake_peers::forge::Forge::new(state, config.clone());
        let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits: config };
        let skein_fake_peers::forge::Forge { domain, pending, .. } = &mut *forge;
        domain::step(
            domain,
            &env,
            domain::Event::Call {
                reply_to: ReplyTo::new(OLDER_READ),
                user: 1,
                repository: b"acme/app".as_slice().into(),
                op: api::Op::Read(api::Read::Locate),
            },
            pending,
        );
        assert!(pending.is_empty(), "the older read is submitted but not fired");
        assert_eq!(domain.next_deadline(), Some(Time::ZERO));
        forge.now = Time::from_nanos(1);
    }
    scenario
}

#[test]
fn hosted_git_fires_past_an_older_due_read_and_keeps_its_reply_once() {
    use skein_fake_forge_domain as domain;
    let scenario = an_older_read_is_due();
    let mut disk = Disk::new(&scenario);
    let result = disk.run(
        &scenario,
        &Command::Fetch {
            workspace: skein_lib::Token::new(1),
            into: git::Into::Fetched,
            remote: scenario.remote.clone().into(),
            source: invocation::Source::Default,
        },
        b"",
    );
    assert_eq!(result.exit, 0, "{:?}", result.stderr);
    assert!(result.stderr.is_empty());
    let mut forge = scenario.transport.forge.borrow_mut();
    assert!(forge.now > Time::ZERO, "git's call follows the older read");
    assert_eq!(forge.pending.len(), PENDING_READ_REPLIES, "only the older read's reply remains");
    let domain::Request::Reply { to, result } = forge.pending.pop().expect("older reply") else {
        panic!("the pending request is the older read's reply");
    };
    assert_eq!(to, skein_lib::ReplyTo::new(OLDER_READ));
    assert!(result.is_ok());
    assert!(forge.pending.is_empty(), "the older reply is returned once");
    assert_eq!(forge.domain.next_deadline(), None, "neither call waits on a timer");
}

#[test]
fn a_clone_keeps_its_given_origin_url() {
    let mut scenario = fixtures::named(b"fixture/specimen", None);
    let clone = temper_worker_protocol_git::invocation::Command::Clone {
        directory: b"fresh".as_slice().into(),
        format: invocation::Format::Sha1,
    };
    let mut spawn = fixtures::spawn_of(&clone);
    spawn.dir = b"workspaces/1".as_slice().into();
    let cloned = temper_fake_git_hosted::check(&spawn).expect("clone invocation");
    let _cloned = temper_fake_git_hosted::answer(&cloned, &mut scenario.checkout, &mut scenario.transport);
    let mut remote = fixtures::spawn_of(&Command::Remote { remote: scenario.remote.clone().into() });
    remote.dir = b"workspaces/1/fresh".as_slice().into();
    let remote = temper_fake_git_hosted::check(&remote).expect("remote add invocation");
    let _answer = temper_fake_git_hosted::answer(&remote, &mut scenario.checkout, &mut scenario.transport);
    let command =
        temper_worker_protocol_git::invocation::Command::Origin { directory: b"workspaces/1/fresh".as_slice().into() };
    let checked = temper_fake_git_hosted::check(&fixtures::spawn_of(&command)).expect("local origin query");
    let answer = temper_fake_git_hosted::answer(&checked, &mut scenario.checkout, &mut scenario.transport);
    assert_eq!(answer.stdout, [scenario.remote.as_slice(), b"\n"].concat());
    assert_eq!(answer.exit, transcripts::read("origin-matches").exit);
}
