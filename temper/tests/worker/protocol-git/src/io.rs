//! Scripted children and kernel files; reusable by the worker service world.
use crate::transcripts::{self, Transcript};
use skein_io::{self as records, kernel};
use skein_lib::{Queue, Token, stream};
use std::collections::{BTreeMap, VecDeque};
use temper_worker_protocol_git::{KERNEL_SUBMISSIONS, Outputs};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Fault {
    None,
    Spawn,
    Signal,
}
#[derive(Clone, Debug)]
pub struct Invocation {
    word: Box<[u8]>,
    pub exit: u8,
    pub stdout: Box<[u8]>,
    pub stderr: Box<[u8]>,
    pub fault: Fault,
}
impl Invocation {
    #[must_use]
    pub fn captured(case: &str) -> Self {
        let Transcript { argv, exit, stdout, stderr } = transcripts::read(case);
        let word = argv.split(|byte| *byte == b'\n').skip(27).find(|word| is_command(word)).expect("captured command");
        Self { word: word.into(), exit, stdout, stderr, fault: Fault::None }
    }
    #[must_use]
    pub fn scripted(word: &[u8], exit: u8, stdout: &[u8], stderr: &[u8]) -> Self {
        Self { word: word.into(), exit, stdout: stdout.into(), stderr: stderr.into(), fault: Fault::None }
    }
}
/// Identify the git command word after the common settings.
#[must_use]
pub fn is_command(word: &[u8]) -> bool {
    !word.starts_with(b"-") && word != b"remote.origin.promisor=true"
}
#[derive(Debug, Default)]
pub struct Script {
    pub invocations: VecDeque<Invocation>,
    pub exit_first: bool,
    /// The scripted command consuming stdin waits for EOF before it exits.
    pub read_stdin_to_end: bool,
    pub file_failure: Option<(u32, kernel::Error)>,
    /// Fail this step of the view on this many attempts.
    pub view_failure: Option<(u32, u8)>,
}
/// A spawn's private index override and the bytes it can read there.
#[derive(Debug, PartialEq, Eq)]
pub struct IndexOverride {
    pub spawn: u32,
    pub path: Box<[u8]>,
    pub contents: Option<Box<[u8]>>,
}

#[derive(Clone, Debug)]
pub enum Node {
    Directory,
    File(Box<[u8]>),
    Link,
}
#[derive(Debug)]
struct Pipe {
    bytes: Box<[u8]>,
    at: usize,
    ended: bool,
    input: bool,
    closed: bool,
}
#[derive(Debug)]
struct Child {
    owner: Token,
    pipes: Vec<Token>,
    exit: kernel::Exit,
    exited: bool,
    closing: bool,
}
#[derive(Debug)]
pub enum Input {
    Io(records::Event),
    Kernel(kernel::Complete),
}
#[derive(Debug, Default)]
struct OpenFile {
    path: Vec<u8>,
    /// The last name returned in the tree's sorted order, even if it is later removed.
    after: Option<Vec<u8>>,
}
/// Owns outputs and answers them from a script, one event or completion at a time.
#[derive(Debug)]
pub struct Below {
    pub out: Outputs,
    pub script: Script,
    pub tree: BTreeMap<Vec<u8>, Node>,
    pub remove_failure: Option<(Vec<u8>, kernel::Error)>,
    pub spawns: u32,
    pub file_calls: u32,
    pub view_steps: u32,
    pub view_attempts: u8,
    view_step: u32,
    pub closed_children: u32,
    pub signals: u32,
    pub arguments: Vec<Vec<Box<[u8]>>>,
    pub messages: Vec<Vec<u8>>,
    pub index_overrides: Vec<IndexOverride>,
    children: BTreeMap<Token, Child>,
    pipes: BTreeMap<Token, Pipe>,
    files: BTreeMap<kernel::Fd, OpenFile>,
    pending: VecDeque<Input>,
    serial: u64,
}
impl Below {
    #[must_use]
    pub fn new(script: Script) -> Self {
        Self {
            out: Outputs {
                events: Queue::uncapped(),
                io: Queue::uncapped(),
                kernel: Queue::with_capacity(KERNEL_SUBMISSIONS),
            },
            script,
            tree: BTreeMap::from([(Vec::new(), Node::Directory)]),
            remove_failure: None,
            spawns: 0,
            file_calls: 0,
            view_steps: 0,
            view_attempts: 0,
            view_step: 0,
            closed_children: 0,
            signals: 0,
            arguments: Vec::new(),
            messages: Vec::new(),
            index_overrides: Vec::new(),
            children: BTreeMap::new(),
            pipes: BTreeMap::new(),
            files: BTreeMap::from([(kernel::Fd::new(3), OpenFile::default())]),
            pending: VecDeque::new(),
            serial: 100,
        }
    }
    pub fn reset(&mut self, script: Script) {
        assert!(self.pending.is_empty() && self.children.is_empty());
        self.script = script;
        self.spawns = 0;
        self.file_calls = 0;
        self.view_steps = 0;
        self.view_attempts = 0;
        self.view_step = 0;
        self.closed_children = 0;
        self.arguments.clear();
        self.messages.clear();
        self.index_overrides.clear();
    }
    /// Use the directory descriptor supplied to the worker service.
    pub fn adopt_root(&mut self, root: kernel::Fd) {
        assert!(self.children.is_empty() && self.pending.is_empty());
        self.files.clear();
        self.files.insert(root, OpenFile::default());
    }
    fn token(&mut self) -> Token {
        self.serial += 1;
        Token::new(self.serial)
    }
    fn event(&mut self, event: records::Event) {
        self.pending.push_back(Input::Io(event));
    }
    /// Consume the outputs of one entry point before the next event is delivered.
    pub fn drain(&mut self) {
        while let Some(request) = self.out.io.pop() {
            self.request(request);
        }
        while let Some(mut submit) = self.out.kernel.pop() {
            assert!(submit.kind.is_valid(), "valid file operation: {:?}", submit.kind);
            self.file_calls += 1;
            if matches!(&submit.kind, kernel::Op::Open { path, how: kernel::OpenHow::DirectoryNoFollow, .. } if path.starts_with(b"workspaces/") && path.ends_with(b"/.git"))
            {
                self.view_attempts += 1;
                self.view_step = 0;
            }
            let view = self.in_view(&submit.kind);
            if view {
                self.view_step += 1;
                self.view_steps = self.view_steps.max(self.view_step);
            }
            let failed_view = view
                && self
                    .script
                    .view_failure
                    .is_some_and(|(at, attempts)| at == self.view_step && self.view_attempts <= attempts);
            let error = self
                .script
                .file_failure
                .filter(|(at, _)| *at == self.file_calls)
                .map(|(_, error)| error)
                .or(if failed_view { Some(kernel::Error::Permission) } else { None });
            let result = if let Some(error) = error {
                // A close consumes its descriptor even when its completion reports an error.
                if matches!(&submit.kind, kernel::Op::Close { .. }) {
                    let _ = self.file(&mut submit.kind);
                }
                Err(error)
            } else {
                self.file(&mut submit.kind)
            };
            self.pending.push_back(Input::Kernel(kernel::Complete { op: submit.op, kind: submit.kind, result }));
        }
    }
    fn in_view(&self, op: &kernel::Op) -> bool {
        use kernel::Op;
        let path = match op {
            Op::Open { root, path, .. } => self.path(*root, path),
            Op::MakeDirectory { dir, name, .. } | Op::Remove { dir, name, .. } => self.path(*dir, name),
            Op::Rename { from_dir, from, .. } if from.as_ref() == b"temper-view.tmp" => self.path(*from_dir, from),
            Op::Read { fd, .. } | Op::Write { fd, .. } | Op::Close { fd } => self.files[fd].path.clone(),
            _ => return false,
        };
        path.starts_with(b"workspaces/") && path.windows(5).any(|part| part == b"/.git")
    }
    pub fn next(&mut self, seed: &mut u64) -> Option<Input> {
        *seed = seed.wrapping_mul(6_364_136_223_846_793_005).wrapping_add(1);
        // Keep lifecycle events ordered; independent stdout/stderr demands may trade places.
        if self.pending.len() >= 2 && *seed & 1 != 0 {
            let a = matches!(self.pending[0], Input::Io(records::Event::Stream { .. }));
            let b = matches!(self.pending[1], Input::Io(records::Event::Stream { .. }));
            if a && b {
                self.pending.swap(0, 1);
            }
        }
        let input = self.pending.pop_front()?;
        if let Input::Io(records::Event::Closed { owner }) = &input
            && !self.pipes.contains_key(owner)
        {
            self.closed_children += 1;
        }
        Some(input)
    }
    #[must_use]
    pub fn file_completions_drained(&self) -> bool {
        self.out.kernel.is_empty() && !self.pending.iter().any(|input| matches!(input, Input::Kernel(_)))
    }
    #[must_use]
    pub fn open_files(&self) -> usize {
        self.files.len()
    }
    #[must_use]
    pub fn settled(&self) -> bool {
        self.pending.is_empty()
            && self.children.is_empty()
            && self.pipes.values().all(|pipe| pipe.closed)
            && self.files.len() == 1
            && self.out.io.is_empty()
            && self.out.kernel.is_empty()
    }
    fn request(&mut self, request: records::Request) {
        match request {
            records::Request::Spawn { owner, spawn } | records::Request::SpawnAlone { owner, spawn } => {
                self.spawn(owner, spawn);
            }
            records::Request::Stream { stream, down } => self.stream(stream, down),
            records::Request::Signal { child, signal, to, .. } => {
                assert!(self.children.contains_key(&child));
                assert_eq!(signal, kernel::Signal::Kill);
                assert_eq!(to, kernel::Target::Group);
                self.signals += 1;
            }
            records::Request::Close { entity }
            | records::Request::CloseAfterFlush { entity }
            | records::Request::Abort { entity } => self.close(entity),
            other @ (records::Request::Listen { .. }
            | records::Request::Connect { .. }
            | records::Request::Bind { .. }
            | records::Request::Reject { .. }
            | records::Request::Output { .. }
            | records::Request::Usage { .. }
            | records::Request::Window { .. }) => panic!("git requested {other:?}"),
        }
    }
    fn spawn(&mut self, owner: Token, spawn: kernel::Spawn) {
        self.spawns += 1;
        let invocation = self.script.invocations.pop_front().expect("script supplies each invocation");
        let word = spawn.args.iter().skip(26).map(AsRef::as_ref).find(|word| is_command(word));
        assert_eq!(word, Some(invocation.word.as_ref()), "command agrees with the script");
        assert!(spawn.dir.is_empty(), "every worker invocation runs at its root");
        for argument in &spawn.args {
            assert!(!argument.windows(11).any(|part| part == b"secret-git!"));
        }
        let init = word == Some(b"init".as_slice());
        self.observe_indexes(&spawn.env);
        self.arguments.push(spawn.args.into_vec());
        if invocation.fault == Fault::Spawn {
            self.event(records::Event::Failed { owner, error: records::Error::Other });
            self.event(records::Event::Closed { owner });
            return;
        }
        if invocation.exit == 0 && init {
            let path = self.arguments.last().expect("init argv").last().expect("init path").clone();
            let bare = self.arguments.last().expect("init argv").iter().any(|arg| arg.as_ref() == b"--bare");
            if !bare {
                self.tree.insert(path.to_vec(), Node::Directory);
            }
            let directory = if bare { path.into_vec() } else { [path.as_ref(), b"/.git"].concat() };
            for suffix in [b"".as_slice(), b"/objects", b"/objects/info", b"/refs", b"/refs/heads", b"/refs/tags"] {
                self.tree.insert([directory.as_slice(), suffix].concat(), Node::Directory);
            }
        }
        let child = self.token();
        let mut pipes = Vec::new();
        for spec in &spawn.pipes {
            let token = self.token();
            let bytes = match spec.child {
                0 => Box::from([]),
                1 => invocation.stdout.clone(),
                2 => invocation.stderr.clone(),
                _ => panic!("standard pipe"),
            };
            self.pipes.insert(token, Pipe { bytes, at: 0, ended: false, input: spec.child == 0, closed: false });
            pipes.push(token);
        }
        let exit = if invocation.fault == Fault::Signal {
            kernel::Exit::Signal(9)
        } else {
            kernel::Exit::Code(invocation.exit)
        };
        self.children.insert(child, Child { owner, pipes: pipes.clone(), exit, exited: false, closing: false });
        self.event(records::Event::Spawned { owner, child, pipes: pipes.into_boxed_slice() });
        self.messages.push(Vec::new());
        if self.script.exit_first {
            self.exit(child);
        }
    }
    fn observe_indexes(&mut self, env: &[Box<[u8]>]) {
        for path in env.iter().filter_map(|entry| entry.strip_prefix(b"GIT_INDEX_FILE=")) {
            let relative = env
                .iter()
                .find_map(|entry| entry.strip_prefix(b"HOME="))
                .and_then(|home| home.strip_suffix(b"/home"))
                .and_then(|root| path.strip_prefix(root))
                .and_then(|rest| rest.strip_prefix(b"/"));
            let contents = match relative.and_then(|path| self.tree.get(path)) {
                Some(Node::File(bytes)) => Some(bytes.clone()),
                Some(Node::Directory | Node::Link) | None => None,
            };
            self.index_overrides.push(IndexOverride { spawn: self.spawns, path: path.into(), contents });
        }
    }
    fn exit(&mut self, token: Token) {
        let child = self.children.get_mut(&token).expect("live child");
        if !child.exited {
            child.exited = true;
            let (owner, exit) = (child.owner, child.exit);
            self.event(records::Event::Exited { owner, exit });
        }
    }
    fn stream(&mut self, token: Token, down: stream::Down) {
        let pipe = self.pipes.get_mut(&token).expect("live pipe");
        if pipe.closed {
            return;
        }
        match down {
            stream::Down::Demand { read, room } => {
                if room > 0 {
                    assert!(pipe.input);
                    self.event(records::Event::Stream { owner: token, up: stream::Up::Room });
                } else if !pipe.ended {
                    let max = match read {
                        stream::Read::Arrived { max }
                        | stream::Read::Scan { max, .. }
                        | stream::Read::Line { max }
                        | stream::Read::Fill(max) => max,
                        stream::Read::Nothing => return,
                    };
                    let end = (pipe.at + usize::try_from(max).expect("max")).min(pipe.bytes.len());
                    let up = if end == pipe.at {
                        pipe.ended = true;
                        stream::Up::End
                    } else {
                        let bytes = Box::from(&pipe.bytes[pipe.at..end]);
                        pipe.at = end;
                        stream::Up::Bytes(bytes)
                    };
                    self.event(records::Event::Stream { owner: token, up });
                }
            }
            stream::Down::Send(bytes) => {
                assert!(pipe.input);
                self.messages.last_mut().expect("spawn").extend_from_slice(&bytes);
            }
            stream::Down::Finish => {
                assert!(pipe.input);
                self.close(token);
            }
        }
        let ready: Vec<Token> = self
            .children
            .iter()
            .filter(|(_, child)| {
                !child.exited
                    && child.pipes.iter().all(|pipe| {
                        let pipe = &self.pipes[pipe];
                        if pipe.input { !self.script.read_stdin_to_end || pipe.closed } else { pipe.ended }
                    })
            })
            .map(|(token, _)| *token)
            .collect();
        for child in ready {
            self.exit(child);
        }
    }
    fn close(&mut self, token: Token) {
        if let Some(pipe) = self.pipes.get_mut(&token) {
            if !pipe.closed {
                pipe.closed = true;
                self.event(records::Event::Closed { owner: token });
            }
        } else {
            let child = self.children.get_mut(&token).expect("closing live child");
            child.closing = true;
            self.exit(token);
        }
        let ended: Vec<Token> = self
            .children
            .iter()
            .filter(|(_, child)| child.closing && child.pipes.iter().all(|pipe| self.pipes[pipe].closed))
            .map(|(token, _)| *token)
            .collect();
        for token in ended {
            let child = self.children.remove(&token).expect("live child");
            self.event(records::Event::Closed { owner: child.owner });
        }
    }
    fn path(&self, dir: kernel::Fd, name: &[u8]) -> Vec<u8> {
        let mut path = self.files.get(&dir).expect("open directory").path.clone();
        if !path.is_empty() && !name.is_empty() {
            path.push(b'/');
        }
        path.extend_from_slice(name);
        path
    }
    /// The names immediately beneath trash, in the fake disk's listing order.
    #[must_use]
    pub fn trash(&self) -> Vec<Vec<u8>> {
        self.children_of(b"trash")
    }
    fn children_of(&self, path: &[u8]) -> Vec<Vec<u8>> {
        let mut prefix = path.to_vec();
        if !prefix.is_empty() {
            prefix.push(b'/');
        }
        self.tree
            .keys()
            .filter_map(|name| {
                name.strip_prefix(prefix.as_slice())
                    .filter(|tail| !tail.is_empty() && !tail.contains(&b'/'))
                    .map(<[u8]>::to_vec)
            })
            .collect()
    }
    fn moving(
        &mut self,
        from_dir: kernel::Fd,
        from: &[u8],
        to_dir: kernel::Fd,
        to: &[u8],
        no_replace: bool,
    ) -> Result<kernel::Done, kernel::Error> {
        let from = self.path(from_dir, from);
        let to = self.path(to_dir, to);
        if !self.tree.contains_key(&from) {
            return Err(kernel::Error::NotFound);
        }
        if no_replace && self.tree.contains_key(&to) {
            return Err(kernel::Error::Exists);
        }
        if self.tree.contains_key(&to) && !self.children_of(&to).is_empty() {
            return Err(kernel::Error::NotEmpty);
        }
        let mut prefix = from.clone();
        prefix.push(b'/');
        let names: Vec<Vec<u8>> =
            self.tree.keys().filter(|name| *name == &from || name.starts_with(&prefix)).cloned().collect();
        for name in names {
            let node = self.tree.remove(&name).expect("moving existing entry");
            let mut moved = to.clone();
            moved.extend_from_slice(&name[from.len()..]);
            self.tree.insert(moved, node);
        }
        Ok(kernel::Done::Nothing)
    }
    fn listing(&mut self, fd: kernel::Fd, entries: &mut [kernel::Entry], names: &mut [u8]) -> kernel::Done {
        let open = &self.files[&fd];
        let values = self.children_of(&open.path);
        let mut count = 0_usize;
        let mut start = 0_usize;
        let mut last = None;
        for name in
            values.into_iter().filter(|name| open.after.as_ref().is_none_or(|after| name > after)).take(entries.len())
        {
            if start + name.len() > names.len() {
                break;
            }
            names[start..start + name.len()].copy_from_slice(&name);
            let kind = match self.tree[&self.path(fd, &name)] {
                Node::Directory => kernel::Kind::Directory,
                Node::File(_) => kernel::Kind::File,
                Node::Link => kernel::Kind::Symlink,
            };
            entries[count] = kernel::Entry {
                kind,
                start: u32::try_from(start).expect("small name"),
                len: u32::try_from(name.len()).expect("small name"),
            };
            count += 1;
            start += name.len();
            last = Some(name);
        }
        if let Some(last) = last {
            self.files.get_mut(&fd).expect("open directory").after = Some(last);
        }
        kernel::Done::Count(u32::try_from(count).expect("listing size"))
    }
    #[expect(clippy::too_many_lines, reason = "the scripted file table includes creating and writing alternates")]
    fn file(&mut self, op: &mut kernel::Op) -> Result<kernel::Done, kernel::Error> {
        use kernel::{Done, Error, Op};
        match op {
            Op::Open { root, path, how } => {
                let path = self.path(*root, path);
                let mut prefix = Vec::new();
                for part in path.split(|byte| *byte == b'/') {
                    if !prefix.is_empty() {
                        prefix.push(b'/');
                    }
                    prefix.extend_from_slice(part);
                    if matches!(self.tree.get(&prefix), Some(Node::Link)) {
                        return Err(Error::TooManyLinks);
                    }
                }
                if matches!(how, kernel::OpenHow::CreateNoFollow { .. }) {
                    if self.tree.contains_key(&path) {
                        return Err(Error::Exists);
                    }
                    self.tree.insert(path.clone(), Node::File(Box::new([])));
                }
                let node = self.tree.get(&path).ok_or(Error::NotFound)?;
                if *how == kernel::OpenHow::DirectoryNoFollow && !matches!(node, Node::Directory) {
                    return Err(Error::NotADirectory);
                }
                let fd = kernel::Fd::new(i32::try_from(self.token().raw()).expect("small descriptor"));
                self.files.insert(fd, OpenFile { path, after: None });
                Ok(Done::Fd(fd))
            }
            Op::Write { fd, bytes, from, at } => {
                let Node::File(content) = self.tree.get_mut(&self.files[fd].path).expect("opened file") else {
                    return Err(Error::IsADirectory);
                };
                let from = usize::try_from(*from).expect("bounded write");
                let at = usize::try_from(*at).expect("bounded offset");
                let mut updated = content.to_vec();
                updated.resize(at + bytes.len() - from, 0);
                updated[at..].copy_from_slice(&bytes[from..]);
                *content = updated.into_boxed_slice();
                Ok(Done::Count(u32::try_from(bytes.len() - from).expect("small write")))
            }
            Op::Read { fd, buf, at } => {
                let node = self.tree.get(&self.files[fd].path).expect("open file");
                let Node::File(bytes) = node else {
                    return Err(Error::IsADirectory);
                };
                let at = usize::try_from(*at).expect("small offset");
                let bytes = bytes.get(at..).unwrap_or(&[]);
                let n = bytes.len().min(buf.len());
                buf[..n].copy_from_slice(&bytes[..n]);
                Ok(Done::Count(u32::try_from(n).expect("small read")))
            }
            Op::Sync { fd } => {
                assert!(self.files.contains_key(fd));
                Ok(Done::Nothing)
            }
            Op::Close { fd } => {
                assert!(self.files.remove(fd).is_some());
                Ok(Done::Nothing)
            }
            Op::MakeDirectory { dir, name, mode } => {
                assert_eq!(*mode, 0o700);
                let path = self.path(*dir, name);
                if self.tree.contains_key(&path) {
                    return Err(Error::Exists);
                }
                self.tree.insert(path, Node::Directory);
                Ok(Done::Nothing)
            }
            Op::Remove { dir, name, directory } => {
                let path = self.path(*dir, name);
                if let Some((blocked, error)) = &self.remove_failure
                    && blocked == &path
                {
                    return Err(*error);
                }
                let node = self.tree.get(&path).ok_or(Error::NotFound)?;
                if matches!(node, Node::Directory) {
                    if !*directory {
                        return Err(Error::IsADirectory);
                    }
                    if !self.children_of(&path).is_empty() {
                        return Err(Error::NotEmpty);
                    }
                } else if *directory {
                    return Err(Error::NotADirectory);
                }
                self.tree.remove(&path);
                Ok(Done::Nothing)
            }
            Op::Rename { from_dir, from, to_dir, to, no_replace } => {
                self.moving(*from_dir, from, *to_dir, to, *no_replace)
            }
            Op::List { fd, entries, names } => Ok(self.listing(*fd, entries, names)),
            other @ (Op::HardLink { .. }
            | Op::Lock { .. }
            | Op::Socket { .. }
            | Op::Bind { .. }
            | Op::Listen { .. }
            | Op::Accept { .. }
            | Op::Connect { .. }
            | Op::Recv { .. }
            | Op::Send { .. }
            | Op::Shutdown { .. }
            | Op::Append { .. }
            | Op::Truncate { .. }
            | Op::Stat { .. }
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
