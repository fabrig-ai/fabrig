//! Remove a killed git's locks before a marked workspace can be used
//! (temper/docs/design/domain/worker/checkouts.md, Found at the next start).
use super::{Action, Directories, Settings, fd, git, invocation, page};
use alloc::boxed::Box;
use skein_io::kernel::{self, Error, Fd, Op, OpenHow};
use skein_lib::{Decimal, List, Queue, bytes};

#[derive(Clone, Copy, Debug)]
enum Scope {
    Git,
    Refs,
    Stores,
}

#[derive(Clone, Copy, Debug)]
enum Phase {
    Open,
    Index,
    Head,
    List,
    Remove,
    Close,
}

#[derive(Debug)]
pub(super) struct Locks {
    pending: Queue<(Box<[u8]>, Scope)>,
    path: Box<[u8]>,
    scope: Scope,
    phase: Phase,
    directory: Option<Fd>,
    name: Box<[u8]>,
}

impl Locks {
    pub(super) fn new(marks: &List<git::Kept>) -> Self {
        let mut pending = Queue::uncapped();
        for mark in marks.as_slice() {
            let token = Decimal::of(mark.workspace.raw());
            for repository in &mark.repositories {
                pending.push((
                    invocation::joined(&[b"workspaces/", token.as_bytes(), b"/", &repository.name, b"/.git"]),
                    Scope::Git,
                ));
                pending
                    .push((invocation::joined(&[b"gitdirs/", token.as_bytes(), b"/", &repository.name]), Scope::Git));
            }
        }
        // Fetch writes per-workspace refs in shared stores too. No git from the
        // previous process remains, so all store ref locks are stale.
        pending.push((Box::from(b"stores".as_slice()), Scope::Stores));
        let (path, scope) = pending.pop().expect("store scan");
        Self { pending, path, scope, phase: Phase::Open, directory: None, name: Box::new([]) }
    }

    pub(super) fn first(&self, settings: &Settings) -> Action {
        let op = match self.phase {
            Phase::Open => Op::Open { root: settings.root, path: self.path.clone(), how: OpenHow::DirectoryNoFollow },
            Phase::Index | Phase::Head | Phase::Remove => Op::Remove {
                dir: self.directory.expect("open lock directory"),
                name: match self.phase {
                    Phase::Index => Box::from(b"index.lock".as_slice()),
                    Phase::Head => Box::from(b"HEAD.lock".as_slice()),
                    Phase::Remove => self.name.clone(),
                    Phase::Open | Phase::List | Phase::Close => unreachable!("removal phase"),
                },
                directory: false,
            },
            Phase::List => Op::List {
                fd: self.directory.expect("open ref directory"),
                entries: Box::new([kernel::Entry::BLANK]),
                names: bytes::zeroed(kernel::LONGEST_NAME),
            },
            Phase::Close => Op::Close { fd: self.directory.expect("open directory") },
        };
        Action::Run(op)
    }

    fn next(&mut self) -> bool {
        if let Some((path, scope)) = self.pending.pop() {
            self.path = path;
            self.scope = scope;
            self.phase = Phase::Open;
            false
        } else {
            true
        }
    }

    pub(super) fn complete(&mut self, complete: kernel::Complete, dirs: &mut Directories) -> Result<bool, Error> {
        match self.phase {
            Phase::Open => {
                self.directory = match fd(complete.result) {
                    Ok(fd) => Some(fd),
                    Err(Error::NotFound | Error::NotADirectory) => return Ok(self.next()),
                    Err(error) => return Err(error),
                };
                dirs.mark_file = self.directory;
                self.phase = match self.scope {
                    Scope::Git => Phase::Index,
                    Scope::Refs | Scope::Stores => Phase::List,
                };
            }
            Phase::Index | Phase::Head | Phase::Remove => {
                if let Err(error) = complete.result
                    && error != Error::NotFound
                {
                    return Err(error);
                }
                self.phase = match self.phase {
                    Phase::Index => Phase::Head,
                    Phase::Head => {
                        self.pending.push((invocation::joined(&[&self.path, b"/refs"]), Scope::Refs));
                        Phase::Close
                    }
                    Phase::Remove => Phase::List,
                    Phase::Open | Phase::List | Phase::Close => unreachable!("removal phase"),
                };
            }
            Phase::List => {
                let directory = if let Op::List { entries, .. } = &complete.kind {
                    entries.first().expect("one entry").kind == kernel::Kind::Directory
                } else {
                    unreachable!("list completion")
                };
                self.name = page(complete)?;
                if self.name.is_empty() {
                    self.phase = Phase::Close;
                } else if directory {
                    let suffix = match self.scope {
                        Scope::Stores => b"/refs/temper".as_slice(),
                        Scope::Refs => b"".as_slice(),
                        Scope::Git => unreachable!("git roots are not listed"),
                    };
                    self.pending.push((invocation::joined(&[&self.path, b"/", &self.name, suffix]), Scope::Refs));
                } else if self.name.ends_with(b".lock") && matches_refs(self.scope) {
                    self.phase = Phase::Remove;
                }
            }
            Phase::Close => {
                complete.result?;
                self.directory = None;
                dirs.mark_file = None;
                return Ok(self.next());
            }
        }
        Ok(false)
    }
}

fn matches_refs(scope: Scope) -> bool {
    match scope {
        Scope::Refs => true,
        Scope::Git | Scope::Stores => false,
    }
}
