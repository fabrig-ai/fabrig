use crate::{Identity, Limits};
use alloc::boxed::Box;
use skein_lib::{Duration, Secret};
use temper_worker_domain_checkout::{self as checkout, git};
pub const HOME: &[u8] = b"/srv/worker/home";
pub const REMOTE: &[u8] = b"https://forge.example.test/acme/app.git";
pub const COMMIT: git::Commit = git::Commit::new([
    0xab, 0xab, 0xab, 0xab, 0xab, 0xab, 0xab, 0xab, 0xab, 0xab, 0xab, 0xab, 0xab, 0xab, 0xab, 0xab, 0xab, 0xab, 0xab,
    0xab, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
]);
pub const COMMIT_256: git::Commit = git::Commit::new([0xcd; 32]);
pub fn checkout() -> checkout::Limits {
    checkout::Limits {
        workspaces: 2,
        repositories: 2,
        name_bytes: 64,
        message_bytes: 256,
        conflicts: 4,
        path_bytes: 64,
        remote_timeout: Duration::from_secs(30),
        local_timeout: Duration::from_secs(5),
        facts: 8,
    }
}
pub fn limits() -> Limits {
    Limits { operations: 2, stdout_bytes: 512, stderr_bytes: 512, read_bytes: 64, sweep_entries: 4, sweep_bytes: 255 }
}
pub fn identity() -> Identity {
    Identity {
        token: Secret::new(Box::from(&b"tok-1"[..])),
        login: Box::from(&b"temper-bot"[..]),
        name: Box::from(&b"Temper Bot"[..]),
        email: Box::from(&b"bot@example.test"[..]),
    }
}

pub const OWNER: skein_lib::Token = skein_lib::Token::new(91);
pub const WORKSPACE: skein_lib::Token = skein_lib::Token::new(7);
pub const DEADLINE: skein_lib::Time = skein_lib::Time::from_nanos(1000);
pub const PARENT: git::Commit = COMMIT_256;
pub const EXPECTED: git::Commit = COMMIT_256;
pub const TRASH: skein_io::kernel::Fd = skein_io::kernel::Fd::new(4);
pub const WORKSPACES: skein_io::kernel::Fd = skein_io::kernel::Fd::new(5);
pub const SERIAL: u64 = 1000;
pub fn make(owner: skein_lib::Token, workspace: skein_lib::Token) -> crate::Request {
    crate::Request::Io { owner, op: git::Op::Make { workspace }, deadline: DEADLINE, identity: None }
}
pub fn at() -> git::Place {
    git::Place { git: None, workspace: WORKSPACE, repository: Box::from(&b"app"[..]) }
}
pub fn fetch(owner: skein_lib::Token, deadline: skein_lib::Time) -> crate::Request {
    crate::Request::Io {
        owner,
        op: git::Op::Fetch {
            at: at(),
            remote: Box::from(REMOTE),
            want: git::Want::Default,
            identity: 1,
            account: None,
        },
        deadline,
        identity: Some(identity()),
    }
}
pub fn commit_op(merging: Option<git::Commit>) -> git::Op {
    git::Op::Commit {
        at: at(),
        parent: PARENT,
        merging,
        title: Box::from(&b"Title"[..]),
        body: Box::from(&b"Body"[..]),
        identity: 1,
    }
}
pub fn push_op(expected: Option<git::Commit>) -> git::Op {
    git::Op::Push {
        at: at(),
        remote: Box::from(REMOTE),
        commit: COMMIT,
        branch: Box::from(&b"main"[..]),
        expected,
        identity: 1,
        account: None,
    }
}
pub fn settings() -> crate::Settings {
    crate::Settings {
        git: Some(Box::from(&b"/bin/git"[..])),
        home: Box::from(HOME),
        root: skein_io::kernel::Fd::new(3),
        prefix: 6_u64 << 60_u32,
        serial: SERIAL,
        largest_read: 64,
        largest_room: 16,
    }
}
#[derive(Debug)]
pub struct Below {
    pub out: crate::Outputs,
    pub io_owner: Option<skein_lib::Token>,
    pub serial: u64,
}
impl Below {
    pub fn new() -> Below {
        Below {
            out: crate::Outputs {
                events: skein_lib::Queue::with_capacity(2),
                io: skein_lib::Queue::with_capacity(20),
                kernel: skein_lib::Queue::with_capacity(2),
            },
            io_owner: None,
            serial: 0,
        }
    }
    pub fn spawned(
        &mut self,
        component: &mut crate::Component,
        now: skein_lib::Time,
    ) -> (skein_lib::Token, Box<[skein_lib::Token]>) {
        self.spawned_program(component, now, b"/bin/git")
    }
    pub fn spawned_program(
        &mut self,
        component: &mut crate::Component,
        now: skein_lib::Time,
        program: &[u8],
    ) -> (skein_lib::Token, Box<[skein_lib::Token]>) {
        let request = self.out.io.pop().expect("spawn requested");
        let (owner, count) = match request {
            skein_io::Request::Spawn { owner, spawn } => {
                assert_eq!(&*spawn.program, program);
                (owner, spawn.pipes.len())
            }
            other @ (skein_io::Request::Listen { .. }
            | skein_io::Request::Connect { .. }
            | skein_io::Request::Bind { .. }
            | skein_io::Request::Reject { .. }
            | skein_io::Request::Stream { .. }
            | skein_io::Request::Output { .. }
            | skein_io::Request::Signal { .. }
            | skein_io::Request::Usage { .. }
            | skein_io::Request::Window { .. }
            | skein_io::Request::Close { .. }
            | skein_io::Request::Abort { .. }) => panic!("expected Spawn, got {other:?}"),
        };
        self.serial = self.serial.checked_add(10).expect("test serial");
        let child = skein_lib::Token::new(self.serial);
        let pipes: Box<[skein_lib::Token]> = if count == 3 {
            Box::new([
                skein_lib::Token::new(self.serial.checked_add(1).expect("test pipe")),
                skein_lib::Token::new(self.serial.checked_add(2).expect("test pipe")),
                skein_lib::Token::new(self.serial.checked_add(3).expect("test pipe")),
            ])
        } else {
            Box::new([
                skein_lib::Token::new(self.serial.checked_add(1).expect("test pipe")),
                skein_lib::Token::new(self.serial.checked_add(2).expect("test pipe")),
            ])
        };
        self.io_owner = Some(owner);
        component.up(now, skein_io::Event::Spawned { owner, child, pipes: pipes.clone() }, &mut self.out);
        (child, pipes)
    }
}
#[derive(Debug)]
pub struct Started {
    pub component: crate::Component,
    pub below: Below,
    pub now: skein_lib::Time,
    pub file: Option<skein_io::kernel::Submit>,
}
impl Started {
    pub fn unstarted() -> Started {
        Started {
            component: crate::Component::new(&limits(), &checkout(), settings()).expect("valid limits"),
            below: Below::new(),
            now: skein_lib::Time::ZERO,
            file: None,
        }
    }
    pub fn new() -> Started {
        Self::laid_out()
    }
    pub fn version_checked() -> Started {
        let mut started = Self::unstarted();
        started.version();
        started
    }
    fn version(&mut self) {
        let started = self;
        started.component.start(started.now, &mut started.below.out);
        let (child, pipes) = started.spawned();
        started.stdout(pipes[0], b"git version 2.53.0\n");
        started.end(pipes[0]);
        started.end(pipes[1]);
        started.exited(child, 0);
        started.closed(child, &pipes);
        started.component.reclaim();
    }
    pub fn laid_out() -> Started {
        let mut started = Self::unstarted();
        started.component = crate::Component::new(
            &limits(),
            &checkout(),
            crate::Settings { serial: SERIAL.checked_sub(2).expect("layout names"), ..settings() },
        )
        .expect("valid");
        started.version();
        for _ in 0_u8..7 {
            started.file_operation();
            started.complete_ok();
        }
        assert_eq!(started.below.out.events.pop(), Some(crate::Event::Started));
        started.empty_sweep();
        started
    }
    pub fn file_operation(&mut self) -> skein_io::kernel::Op {
        assert!(self.file.is_none(), "previous file operation completed");
        let submit = self.below.out.kernel.pop().expect("kernel submission");
        let kind = clone_file_kind(&submit.kind);
        self.file = Some(submit);
        kind
    }
    pub fn complete(&mut self, result: Result<skein_io::kernel::Done, skein_io::kernel::Error>) {
        let submit = self.file.take().expect("file operation inspected");
        self.component.complete(
            self.now,
            skein_io::kernel::Complete { op: submit.op, kind: submit.kind, result },
            &mut self.below.out,
        );
    }
    pub fn complete_ok(&mut self) {
        let result = match &self.file.as_ref().expect("file inspected").kind {
            skein_io::kernel::Op::Open { path, .. } => {
                skein_io::kernel::Done::Fd(if &**path == b"trash" { TRASH } else { WORKSPACES })
            }
            skein_io::kernel::Op::Read { .. } | skein_io::kernel::Op::List { .. } => skein_io::kernel::Done::Count(0),
            skein_io::kernel::Op::Rename { .. }
            | skein_io::kernel::Op::MakeDirectory { .. }
            | skein_io::kernel::Op::Remove { .. }
            | skein_io::kernel::Op::Close { .. } => skein_io::kernel::Done::Nothing,
            skein_io::kernel::Op::Lock { .. }
            | skein_io::kernel::Op::Socket { .. }
            | skein_io::kernel::Op::Bind { .. }
            | skein_io::kernel::Op::Listen { .. }
            | skein_io::kernel::Op::Accept { .. }
            | skein_io::kernel::Op::Connect { .. }
            | skein_io::kernel::Op::Recv { .. }
            | skein_io::kernel::Op::Send { .. }
            | skein_io::kernel::Op::Shutdown { .. }
            | skein_io::kernel::Op::Write { .. }
            | skein_io::kernel::Op::Append { .. }
            | skein_io::kernel::Op::Truncate { .. }
            | skein_io::kernel::Op::Sync { .. }
            | skein_io::kernel::Op::Stat { .. }
            | skein_io::kernel::Op::Spawn { .. }
            | skein_io::kernel::Op::Wait { .. }
            | skein_io::kernel::Op::Signal { .. }
            | skein_io::kernel::Op::Usage
            | skein_io::kernel::Op::Window { .. }
            | skein_io::kernel::Op::ReadSignal { .. }
            | skein_io::kernel::Op::PipeRead { .. }
            | skein_io::kernel::Op::PipeWrite { .. }
            | skein_io::kernel::Op::Cancel { .. } => panic!("unexpected file request"),
        };
        self.complete(Ok(result));
    }
    pub fn complete_err(&mut self, error: skein_io::kernel::Error) {
        self.complete(Err(error));
    }
    pub fn read(&mut self, bytes: &[u8]) {
        use skein_io::kernel::Op;
        match &mut self.file.as_mut().expect("file inspected").kind {
            Op::Read { buf, .. } => {
                for (slot, byte) in buf.iter_mut().zip(bytes) {
                    *slot = *byte;
                }
            }
            Op::Lock { .. }
            | Op::Socket { .. }
            | Op::Bind { .. }
            | Op::Listen { .. }
            | Op::Accept { .. }
            | Op::Connect { .. }
            | Op::Recv { .. }
            | Op::Send { .. }
            | Op::Shutdown { .. }
            | Op::Close { .. }
            | Op::Open { .. }
            | Op::Write { .. }
            | Op::Append { .. }
            | Op::Truncate { .. }
            | Op::Sync { .. }
            | Op::Stat { .. }
            | Op::Rename { .. }
            | Op::Remove { .. }
            | Op::MakeDirectory { .. }
            | Op::List { .. }
            | Op::Spawn { .. }
            | Op::Wait { .. }
            | Op::Signal { .. }
            | Op::Usage
            | Op::Window { .. }
            | Op::ReadSignal { .. }
            | Op::PipeRead { .. }
            | Op::PipeWrite { .. }
            | Op::Cancel { .. } => panic!("expected Read"),
        }
        self.complete(Ok(skein_io::kernel::Done::Count(u32::try_from(bytes.len()).expect("test read"))));
    }
    pub fn listing(&mut self, values: &[(&[u8], skein_io::kernel::Kind)]) {
        use skein_io::kernel::Op;
        match &mut self.file.as_mut().expect("file inspected").kind {
            Op::List { entries, names, .. } => {
                let mut start = 0;
                for (i, (name, kind)) in values.iter().enumerate() {
                    for (slot, byte) in names.iter_mut().skip(start).zip(*name) {
                        *slot = *byte;
                    }
                    entries[i] = skein_io::kernel::Entry {
                        kind: *kind,
                        start: u32::try_from(start).expect("test start"),
                        len: u32::try_from(name.len()).expect("test name"),
                    };
                    start = start.checked_add(name.len()).expect("test names");
                }
            }
            Op::Lock { .. }
            | Op::Socket { .. }
            | Op::Bind { .. }
            | Op::Listen { .. }
            | Op::Accept { .. }
            | Op::Connect { .. }
            | Op::Recv { .. }
            | Op::Send { .. }
            | Op::Shutdown { .. }
            | Op::Close { .. }
            | Op::Open { .. }
            | Op::Read { .. }
            | Op::Write { .. }
            | Op::Append { .. }
            | Op::Truncate { .. }
            | Op::Sync { .. }
            | Op::Stat { .. }
            | Op::Rename { .. }
            | Op::Remove { .. }
            | Op::MakeDirectory { .. }
            | Op::Spawn { .. }
            | Op::Wait { .. }
            | Op::Signal { .. }
            | Op::Usage
            | Op::Window { .. }
            | Op::ReadSignal { .. }
            | Op::PipeRead { .. }
            | Op::PipeWrite { .. }
            | Op::Cancel { .. } => panic!("expected List"),
        }
        self.complete(Ok(skein_io::kernel::Done::Count(u32::try_from(values.len()).expect("test page"))));
    }
    pub fn empty_sweep(&mut self) {
        for _ in 0_u8..3 {
            self.file_operation();
            self.complete_ok();
        }
    }
    pub fn shutdown(&mut self) {
        self.down(crate::Request::Shutdown);
        for _ in 0_u8..2 {
            self.file_operation();
            self.complete_ok();
        }
    }
    pub fn down(&mut self, request: crate::Request) {
        self.component.down(self.now, request, &mut self.below.out);
    }
    pub fn spawned(&mut self) -> (skein_lib::Token, Box<[skein_lib::Token]>) {
        self.below.spawned(&mut self.component, self.now)
    }
    pub fn stdout(&mut self, pipe: skein_lib::Token, bytes: &[u8]) {
        for byte in bytes {
            while self.below.out.io.pop().is_some() {}
            self.component.up(
                self.now,
                skein_io::Event::Stream { owner: pipe, up: skein_lib::stream::Up::Bytes(Box::from([*byte])) },
                &mut self.below.out,
            );
        }
    }
    pub fn end(&mut self, pipe: skein_lib::Token) {
        while self.below.out.io.pop().is_some() {}
        self.component.up(
            self.now,
            skein_io::Event::Stream { owner: pipe, up: skein_lib::stream::Up::End },
            &mut self.below.out,
        );
    }
    pub fn exited(&mut self, _child: skein_lib::Token, code: u8) {
        self.component.up(
            self.now,
            skein_io::Event::Exited {
                owner: self.below.io_owner.expect("spawned"),
                exit: skein_io::kernel::Exit::Code(code),
            },
            &mut self.below.out,
        );
    }
    pub fn closed(&mut self, _child: skein_lib::Token, pipes: &[skein_lib::Token]) {
        while self.below.out.io.pop().is_some() {}
        for pipe in pipes {
            self.component.up(self.now, skein_io::Event::Closed { owner: *pipe }, &mut self.below.out);
        }
        self.component.up(
            self.now,
            skein_io::Event::Closed { owner: self.below.io_owner.expect("spawned") },
            &mut self.below.out,
        );
    }
    pub fn fire(&mut self, now: skein_lib::Time) {
        self.now = now;
        while self.below.out.io.pop().is_some() {}
        self.component.fire(now, &mut self.below.out);
    }
    pub fn signals(&self) -> Box<[(skein_lib::Token, skein_io::kernel::Signal, skein_io::kernel::Target)]> {
        let mut values = skein_lib::List::with_capacity(4);
        for request in &self.below.out.io {
            if let skein_io::Request::Signal { child, signal, to } = request {
                values.push((*child, *signal, *to)).expect("test-sized signals");
            }
        }
        values.into_boxed()
    }
    pub fn done(&mut self) -> Option<(skein_lib::Token, git::Done)> {
        match self.below.out.events.pop() {
            Some(crate::Event::Done { owner, done }) => Some((owner, done)),
            None => None,
            Some(crate::Event::Started | crate::Event::Unusable { .. }) => panic!("unexpected start terminal"),
        }
    }
}
fn clone_file_kind(kind: &skein_io::kernel::Op) -> skein_io::kernel::Op {
    use skein_io::kernel::Op;
    match kind {
        Op::Open { root, path, how } => Op::Open { root: *root, path: path.clone(), how: *how },
        Op::Read { fd, buf, at } => Op::Read { fd: *fd, buf: buf.clone(), at: *at },
        Op::Rename { from_dir, from, to_dir, to, no_replace } => Op::Rename {
            from_dir: *from_dir,
            from: from.clone(),
            to_dir: *to_dir,
            to: to.clone(),
            no_replace: *no_replace,
        },
        Op::Remove { dir, name, directory } => Op::Remove { dir: *dir, name: name.clone(), directory: *directory },
        Op::MakeDirectory { dir, name, mode } => Op::MakeDirectory { dir: *dir, name: name.clone(), mode: *mode },
        Op::List { fd, entries, names } => Op::List { fd: *fd, entries: entries.clone(), names: names.clone() },
        Op::Close { fd } => Op::Close { fd: *fd },
        Op::Lock { .. }
        | Op::Socket { .. }
        | Op::Bind { .. }
        | Op::Listen { .. }
        | Op::Accept { .. }
        | Op::Connect { .. }
        | Op::Recv { .. }
        | Op::Send { .. }
        | Op::Shutdown { .. }
        | Op::Write { .. }
        | Op::Append { .. }
        | Op::Truncate { .. }
        | Op::Sync { .. }
        | Op::Stat { .. }
        | Op::Spawn { .. }
        | Op::Wait { .. }
        | Op::Signal { .. }
        | Op::Usage
        | Op::Window { .. }
        | Op::ReadSignal { .. }
        | Op::PipeRead { .. }
        | Op::PipeWrite { .. }
        | Op::Cancel { .. } => panic!("unexpected file operation"),
    }
}
