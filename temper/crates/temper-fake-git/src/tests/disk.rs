//! Files beneath the program, using the same machine face as the hosted transcripts.
use super::*;
use skein_fake_machine::{How, Is, Item, Machine, Opened};
use skein_io::kernel::OpenHow;
use std::collections::BTreeSet;
pub(super) struct Disk {
    pub machine: Machine,
    pub written: Vec<Vec<u8>>,
    root: Opened,
}
impl Disk {
    pub fn new(files: &[(Vec<u8>, Vec<u8>)]) -> Self {
        let mut directories = BTreeSet::from([b"repo".to_vec()]);
        let mut items = Vec::new();
        for (path, bytes) in files {
            let mut parent = path.as_slice();
            while let Some(slash) = parent.iter().rposition(|b| *b == b'/') {
                parent = &parent[..slash];
                directories.insert(parent.to_vec());
            }
            items.push(Item::file(path, bytes));
        }
        let mut all: Vec<_> = directories.iter().map(|path| Item::directory(path)).collect();
        all.extend(items);
        let mut machine = Machine::new();
        let root = machine.lay(&all);
        Self { machine, root, written: Vec::new() }
    }
    pub fn read(&mut self, path: &[u8]) -> Option<Vec<u8>> {
        let fd = self.machine.open(self.root, path, How::ReadNoFollow).ok()?;
        let value = self.machine.read(fd, 0, 65536).expect("read file");
        self.machine.close(fd);
        Some(value)
    }
    pub fn run(&mut self, mut spawn: k::Spawn, net: &mut Net, mut input: &[u8]) -> crate::Answer {
        let root = descriptor(self.machine.reopen_root(0));
        spawn.dir = b"repo".as_slice().into();
        let mut held: BTreeSet<_> = [Fd::new(10000), Fd::new(10001), Fd::new(10002), Fd::new(10003)].into();
        let mut owned: Vec<_> = held.iter().copied().collect();
        owned.push(root);
        let start = crate::Start {
            root,
            pipes: Box::new([(0, Fd::new(10000)), (1, Fd::new(10001)), (2, Fd::new(10002))]),
            owned: owned.into(),
        };
        let mut host = crate::Git::new(&spawn, start, crate::Route::new(limits()));
        let mut answer = crate::Answer { exit: 0, stdout: Vec::new(), stderr: Vec::new() };
        // The runs-bound row crosses 1024 tiny HTTP exchanges; normal rows settle in hundreds of jobs.
        for _ in 0..1_000_000 {
            host.iterate(Time::ZERO, Wall::EPOCH);
            if let Some(Exit::Code(exit)) = host.exit() {
                answer.exit = exit;
                assert!(held.is_empty());
                assert_eq!(self.machine.open_handles(), 1, "only test root remains");
                net.closed();
                return answer;
            }
            let Some(submit) = host.submissions().pop() else { continue };
            let mut op = submit.kind;
            let result = match &mut op {
                Op::PipeRead { buf, .. } => {
                    let count = input.len().min(buf.len()).min(7);
                    buf[..count].copy_from_slice(&input[..count]);
                    input = &input[count..];
                    Ok(Done::Count(u32::try_from(count).expect("input")))
                }
                Op::PipeWrite { fd, bytes, from } => {
                    assert!(held.contains(fd));
                    let bytes = &bytes[usize::try_from(*from).expect("offset")..];
                    let count = bytes.len().min(7);
                    if *fd == Fd::new(10001) {
                        answer.stdout.extend_from_slice(&bytes[..count]);
                    } else {
                        assert_eq!(*fd, Fd::new(10002));
                        answer.stderr.extend_from_slice(&bytes[..count]);
                    }
                    Ok(Done::Count(u32::try_from(count).expect("output")))
                }
                Op::Close { fd } if held.remove(fd) => Ok(Done::Nothing),
                op @ (Op::Socket { .. }
                | Op::Connect { .. }
                | Op::Send { .. }
                | Op::Recv { .. }
                | Op::Shutdown { .. }) => net.step(op),
                Op::Close { fd } if *fd == Net::FD => net.step(&mut op),
                op @ (Op::Bind { .. }
                | Op::Listen { .. }
                | Op::Accept { .. }
                | Op::Open { .. }
                | Op::Read { .. }
                | Op::Write { .. }
                | Op::Append { .. }
                | Op::Truncate { .. }
                | Op::Sync { .. }
                | Op::Lock { .. }
                | Op::Stat { .. }
                | Op::Rename { .. }
                | Op::Remove { .. }
                | Op::MakeDirectory { .. }
                | Op::List { .. }
                | Op::Spawn { .. }
                | Op::Wait { .. }
                | Op::Signal { .. }
                | Op::Window { .. }
                | Op::ReadSignal { .. }
                | Op::Cancel { .. }
                | Op::Usage
                | Op::Close { .. }) => self.file(op).map_err(skein_fake_machine::kernel_error),
            };
            host.completions().push(k::Complete { op: submit.op, kind: op, result });
        }
        panic!("git settles within scripted operation bound");
    }
    fn file(&mut self, op: &mut Op) -> Result<Done, skein_fake_machine::Refusal> {
        match op {
            Op::Open { root, path, how } => {
                if matches!(how, OpenHow::WriteNoFollow | OpenHow::CreateNoFollow { .. }) {
                    self.written.push(path.to_vec());
                }
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
fn descriptor(fd: Opened) -> Fd {
    Fd::new(i32::try_from(fd.raw()).expect("descriptor"))
}
fn opened(fd: Fd) -> Opened {
    Opened::new(u64::try_from(fd.raw()).expect("descriptor"))
}
