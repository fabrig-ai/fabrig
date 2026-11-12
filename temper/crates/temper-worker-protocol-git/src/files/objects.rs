//! Shared objects and private workspace references (protocol/git.md, 4.1–4.3, 4.11).
use super::{Action, Directories, close, fd, mkdir, open};
use crate::{
    Settings,
    invocation::{self, Command},
    sequence::Next,
};
use alloc::boxed::Box;
use skein_io::kernel::{Done, Error, Fd, Op, OpenHow};
use skein_lib::{Decimal, Token};
use temper_worker_domain_checkout::git;
fn broken() -> Action {
    Action::Operation(Next::End(git::Done::Failed { fault: git::Fault::Broken }))
}
fn name(token: Token) -> Box<[u8]> {
    Box::from(Decimal::of(token.raw()).as_bytes())
}
#[derive(Debug)]
pub(crate) struct Make {
    workspace: Box<[u8]>,
    stores: Box<[Token]>,
    index: usize,
    phase: MakePhase,
    fd: Option<Fd>,
    failed: bool,
}
#[derive(Debug)]
enum MakePhase {
    Aside,
    Create,
    Open,
    Move,
    Close,
}
impl Make {
    pub fn new(workspace: Token, stores: Box<[Token]>) -> Self {
        Self { workspace: name(workspace), stores, index: 0, phase: MakePhase::Aside, fd: None, failed: false }
    }
    pub fn first(&self, dirs: &mut Directories) -> Action {
        Action::Run(dirs.rename(dirs.workspaces.expect("layout"), self.workspace.clone()))
    }
    fn next(&mut self, settings: &Settings, stop: bool) -> Action {
        if self.failed {
            return broken();
        }
        let Some(store) = self.stores.get(self.index) else {
            return Action::Operation(Next::End(git::Done::Succeeded));
        };
        if stop {
            return broken();
        }
        self.phase = MakePhase::Open;
        Action::Run(open(
            settings.root,
            invocation::joined(&[b"stores/", &name(*store), b"/refs/temper"]),
            OpenHow::DirectoryNoFollow,
        ))
    }
    pub fn complete(
        &mut self,
        result: Result<Done, Error>,
        dirs: &mut Directories,
        settings: &Settings,
        stop: bool,
    ) -> Action {
        match self.phase {
            MakePhase::Aside => {
                if result.is_err() && result != Err(Error::NotFound) {
                    return broken();
                }
                if stop {
                    return broken();
                }
                self.phase = MakePhase::Create;
                Action::Run(mkdir(dirs.workspaces.expect("layout"), &self.workspace))
            }
            MakePhase::Create => {
                if result.is_err() {
                    return broken();
                }
                self.next(settings, stop)
            }
            MakePhase::Open => {
                if result == Err(Error::NotFound) {
                    self.index = self.index.checked_add(1).expect("spec length");
                    return self.next(settings, stop);
                }
                let Ok(fd) = fd(result) else {
                    return broken();
                };
                self.fd = Some(fd);
                if stop {
                    self.phase = MakePhase::Close;
                    return close(fd);
                }
                self.phase = MakePhase::Move;
                Action::Run(dirs.rename(fd, self.workspace.clone()))
            }
            MakePhase::Move => {
                self.failed = result.is_err() && result != Err(Error::NotFound);
                self.phase = MakePhase::Close;
                close(self.fd.expect("reference directory"))
            }
            MakePhase::Close => {
                self.fd = None;
                self.failed |= result.is_err();
                self.index = self.index.checked_add(1).expect("spec length");
                self.next(settings, stop)
            }
        }
    }
}
#[derive(Debug)]
pub(crate) struct Store {
    token: Token,
    temporary: Box<[u8]>,
    remote: Box<[u8]>,
    phase: StorePhase,
    done: Box<git::Done>,
}
#[derive(Debug)]
enum StorePhase {
    Aside(Token),
    Find,
    Close(Fd),
    Create,
    Install,
    Discard,
}
impl Store {
    pub fn new(store: Token, aside: Option<Token>, remote: Box<[u8]>) -> Self {
        let phase = match aside {
            Some(aside) => StorePhase::Aside(aside),
            None => StorePhase::Find,
        };
        Self { token: store, temporary: Box::new([]), remote, phase, done: Box::new(git::Done::Succeeded) }
    }
    pub fn install(store: Token, directory: &[u8], done: Box<git::Done>) -> Self {
        let phase = if *done == git::Done::Succeeded { StorePhase::Install } else { StorePhase::Discard };
        Self { token: store, temporary: Box::from(directory), remote: Box::new([]), phase, done }
    }
    pub fn first(&self, dirs: &mut Directories) -> Action {
        let stores = dirs.stores.expect("layout");
        Action::Run(match self.phase {
            StorePhase::Aside(aside) => dirs.rename(stores, name(aside)),
            StorePhase::Find => open(stores, name(self.token), OpenHow::DirectoryNoFollow),
            StorePhase::Close(fd) => Op::Close { fd },
            StorePhase::Create => mkdir(dirs.making.expect("layout"), &self.temporary),
            StorePhase::Install => Op::Rename {
                from_dir: dirs.making.expect("layout"),
                from: self.temporary.clone(),
                to_dir: stores,
                to: name(self.token),
                no_replace: true,
            },
            StorePhase::Discard => dirs.rename(dirs.making.expect("layout"), self.temporary.clone()),
        })
    }
    pub fn complete(&mut self, result: Result<Done, Error>, dirs: &mut Directories, stop: bool) -> Action {
        match self.phase {
            StorePhase::Aside(_) => {
                if result.is_err() && result != Err(Error::NotFound) {
                    return broken();
                }
                if stop {
                    return broken();
                }
                self.phase = StorePhase::Find;
                self.first(dirs)
            }
            StorePhase::Find => match fd(result) {
                Ok(fd) => {
                    self.phase = StorePhase::Close(fd);
                    self.first(dirs)
                }
                Err(Error::NotFound) => {
                    if stop {
                        return broken();
                    }
                    self.temporary = temporary(dirs);
                    self.phase = StorePhase::Create;
                    self.first(dirs)
                }
                Err(_) => broken(),
            },
            StorePhase::Close(_) => {
                if result.is_err() || stop {
                    return broken();
                }
                Action::Operation(Next::End(git::Done::Succeeded))
            }
            StorePhase::Create => {
                if result.is_err() {
                    return broken();
                }
                if stop {
                    *self.done = git::Done::Failed { fault: git::Fault::Broken };
                    self.phase = StorePhase::Discard;
                    return self.first(dirs);
                }
                Action::Operation(Next::Run(Command::Store {
                    directory: self.temporary.clone(),
                    remote: self.remote.clone(),
                }))
            }
            StorePhase::Install => {
                if result.is_ok() {
                    return Action::Operation(Next::End(git::Done::Succeeded));
                }
                if result != Err(Error::Exists) {
                    *self.done = git::Done::Failed { fault: git::Fault::Broken };
                }
                self.phase = StorePhase::Discard;
                self.first(dirs)
            }
            StorePhase::Discard => {
                if result.is_err() {
                    return broken();
                }
                Action::Operation(Next::End(*self.done.clone()))
            }
        }
    }
}
#[derive(Debug)]
pub(crate) struct Alternates {
    path: Box<[u8]>,
    bytes: Box<[u8]>,
    remote: Box<[u8]>,
    fd: Option<Fd>,
    written: u32,
    closing: bool,
    failed: bool,
}
impl Alternates {
    pub fn new(at: &git::Place, store: Token, remote: Box<[u8]>) -> Self {
        Self {
            path: invocation::joined(&[&invocation::place(at), b"/.git/objects/info/alternates"]),
            bytes: invocation::joined(&[b"../../../../../stores/", &name(store), b"/objects\n"]),
            remote,
            fd: None,
            written: 0,
            closing: false,
            failed: false,
        }
    }
    pub fn first(&self, settings: &Settings) -> Action {
        Action::Run(open(settings.root, self.path.clone(), OpenHow::CreateNoFollow { mode: Some(0o600) }))
    }
    pub fn complete(&mut self, result: Result<Done, Error>, stop: bool) -> Action {
        if self.closing {
            self.fd = None;
            return if self.failed || result.is_err() || stop {
                broken()
            } else {
                Action::Operation(Next::Run(Command::Remote { remote: self.remote.clone() }))
            };
        }
        if self.fd.is_none() {
            let Ok(fd) = fd(result) else {
                return broken();
            };
            self.fd = Some(fd);
        } else if let Ok(count) = super::count(result) {
            self.written = self.written.checked_add(count).expect("alternates length");
        } else {
            self.failed = true;
        }
        let fd = self.fd.expect("alternates opened");
        if stop || self.failed || usize::try_from(self.written).expect("small file") == self.bytes.len() {
            self.closing = true;
            return close(fd);
        }
        Action::Run(Op::Write { fd, bytes: self.bytes.clone(), from: self.written, at: u64::from(self.written) })
    }
}

fn temporary(dirs: &mut Directories) -> Box<[u8]> {
    let result = Box::from(Decimal::of(dirs.serial).as_bytes());
    dirs.serial = dirs.serial.checked_add(1).expect("serial lifetime");
    result
}
