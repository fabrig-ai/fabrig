//! Makes or copies a repository, sharing org, access and file steps (skein/docs/design/forge/setup/forgejo.md, 1.9 and 1.10).
use crate::forgejo::admin::{self, Admin};
use crate::forgejo::{Reply, reply, unexpected};
use crate::{
    call::Auth,
    forgejo::access::{Access, Fail, Move, Shared},
    given::{self, Given, Refused},
    host::File,
    item::Item,
    plan::{self, Event, Need, Outcome, Stage, Why},
    reconcile,
};
use alloc::boxed::Box;
use core::num::NonZeroU64;
use skein_forge::forgejo::{
    document::Document,
    request::{Author, Repo, Request},
};
use skein_lib::{Queue, Secret, Writer};

#[derive(Debug)]
enum State {
    Admin(Admin),
    Org,
    MakeOrg,
    Repository,
    MakeRepository,
    CopyMark,
    Mark,
    Migrate,
    CopyBranch { restart: bool },
    Delete,
    Access,
    Branch,
    Raw,
    File,
    Ended,
}
#[derive(Debug)]
enum Source {
    Empty,
    Copy { from: Box<[u8]> },
}
/// One creation or copy, holding its administrator only until the run ends.
#[derive(Debug)]
pub struct Create {
    source: Source,
    forge: u16,
    admin_name: Box<[u8]>,
    admin: Option<Secret>,
    admin_login: Box<[u8]>,
    owner: Option<Box<[u8]>>,
    at: Repo,
    subject: Box<[u8]>,
    author: Author,
    files: Box<[File]>,
    next: usize,
    id: Option<NonZeroU64>,
    branch: Box<[u8]>,
    access: Access,
    shared: Shared,
    state: State,
}
impl Create {
    /// The host has checked the plan against its admitted bounds before constructing it.
    pub fn new(given: &Given, org: Box<[u8]>, repository: Box<[u8]>, files: Box<[File]>) -> Result<Self, Refused> {
        given::check(given)?;
        given::create_fields(&given.limits, u32::MAX, &org, &repository, &files)?;
        let mut subject = Writer::new(
            org.len().checked_add(repository.len()).expect("checked names").checked_add(1).expect("subject slash"),
        );
        subject.put(&org).expect("org capacity");
        subject.put(b"/").expect("slash capacity");
        subject.put(&repository).expect("name capacity");
        Ok(Self {
            source: Source::Empty,
            forge: given.forge,
            admin_name: given.admin_name.clone(),
            admin: None,
            admin_login: Box::new([]),
            owner: given.org_owner.clone(),
            at: Repo { owner: org, name: repository },
            subject: subject.finish(),
            author: Author { name: given.bot_login.clone(), email: given.bot_email.clone() },
            files,
            next: 0,
            id: None,
            branch: Box::new([]),
            access: Access::new(
                0,
                &given.bot_login,
                match given.listener {
                    Some(address) => reconcile::hook_url(address, given.forge),
                    None => None,
                },
            ),
            shared: Shared { cap: None, secret: None },
            state: State::Admin(Admin::Begin),
        })
    }
    pub(crate) fn copying(
        given: &Given,
        org: Box<[u8]>,
        repository: Box<[u8]>,
        from: Box<[u8]>,
        files: Box<[File]>,
    ) -> Result<Self, Refused> {
        let plan = crate::host::Plan::Copy { forge: given.forge, org, repository, from, files };
        given::check_copy(&given.limits, u32::MAX, &plan)?;
        let crate::host::Plan::Copy { org, repository, from, files, .. } = plan else {
            unreachable!("copy plan constructed above")
        };
        let mut machine = Self::new(given, org, repository, files)?;
        machine.source = Source::Copy { from };
        Ok(machine)
    }
    fn stage(&self) -> Stage {
        match self.state {
            State::Admin(_) => Stage::Admin,
            State::Org | State::MakeOrg => Stage::Org,
            State::Repository
            | State::MakeRepository
            | State::CopyMark
            | State::Mark
            | State::Migrate
            | State::CopyBranch { .. }
            | State::Delete
            | State::Access => Stage::Repository,
            State::Branch | State::Raw | State::File => Stage::File,
            State::Ended => unreachable!("ended creation has no outstanding need"),
        }
    }
    fn stop(&mut self, why: Why) -> Need {
        let step = self.stage();
        let subject = match step {
            Stage::Admin => self.admin_name.clone(),
            Stage::Org => self.at.owner.clone(),
            Stage::Repository => self.subject.clone(),
            Stage::File => self.file().path.clone(),
            Stage::Bot | Stage::Token | Stage::App | Stage::Installation | Stage::State | Stage::Person => {
                unreachable!("creation stages")
            }
        };
        self.state = State::Ended;
        self.admin = None;
        self.shared.secret = None;
        plan::stop(self.forge, step, &subject, why)
    }
    fn send(&self, request: Request) -> Need {
        Need::Send { request: request.into(), auth: Auth::Token(self.admin.clone().expect("admin before writes")) }
    }
    fn repository(&mut self) -> Need {
        self.state = State::Repository;
        self.send(Request::Repository { at: self.at.clone() })
    }
    fn remember(&mut self, id: u64, branch: Box<[u8]>) -> Result<(), Why> {
        self.id = Some(NonZeroU64::new(id).ok_or(Why::Unexpected)?);
        if branch.is_empty() {
            return Err(Why::Unexpected);
        }
        self.branch = branch;
        Ok(())
    }
    fn access(&mut self) -> Need {
        self.state = State::Access;
        let request = self.access.known(self.at.clone());
        self.send(request)
    }
    fn file(&self) -> &File {
        self.files.get(self.next).expect("current file")
    }
    fn next_file(&mut self) -> Need {
        if self.next == self.files.len() {
            self.state = State::Ended;
            self.admin = None;
            self.shared.secret = None;
            let id = self.id.expect("repository read or made");
            let branch = self.branch.clone();
            Need::Done(match self.source {
                Source::Empty => Outcome::Created { id, branch },
                Source::Copy { .. } => Outcome::Copied { id, branch },
            })
        } else {
            self.state = State::Branch;
            self.send(Request::Branch { at: self.at.clone(), name: self.branch.clone() })
        }
    }
    fn file_done(&mut self) -> Need {
        self.next = self.next.checked_add(1).expect("bounded files");
        self.next_file()
    }
    fn put_file(&mut self) -> Need {
        let file = self.file();
        let mut message = Writer::new(file.path.len().checked_add(4).expect("bounded path"));
        message.put(b"Add ").expect("message prefix");
        message.put(&file.path).expect("message path");
        let request = Request::CreateFile {
            at: self.at.clone(),
            path: file.path.clone(),
            branch: self.branch.clone(),
            content: file.content.clone(),
            message: message.finish(),
            author: self.author.clone(),
        };
        self.state = State::File;
        self.send(request)
    }
    fn advance(&mut self, event: Event) -> Result<Need, Why> {
        match self.state {
            State::Admin(ref mut admin) => match admin::step(admin, event)? {
                admin::Move::Read => Ok(Need::Read(Item::AdminToken)),
                admin::Move::Token(token) => {
                    self.admin = Some(token);
                    Ok(self.send(Request::User))
                }
                admin::Move::Passed(user) => {
                    self.admin_login = user.login;
                    self.state = State::Org;
                    Ok(self.send(Request::Org { name: self.at.owner.clone() }))
                }
            },
            State::CopyMark => match event.read() {
                Ok(Some(_)) => Ok(self.copy_branch(true)),
                Ok(None) => Err(Why::Taken),
                Err(why) => Err(why),
            },
            State::Mark => match event.written() {
                Ok(()) => {
                    let from = match &self.source {
                        Source::Copy { from } => from.clone(),
                        Source::Empty => unreachable!("only a copy writes its mark"),
                    };
                    self.state = State::Migrate;
                    Ok(self.send(Request::Migrate { at: self.at.clone(), from }))
                }
                Err(why) => Err(why),
            },
            State::Access => match self.access.event(&mut self.shared, event) {
                Ok(Move::Send(request)) => Ok(self.send(request)),
                Ok(Move::Read(item)) => Ok(Need::Read(item)),
                Ok(Move::Write { item, bytes }) => Ok(Need::Write { item, bytes }),
                Ok(Move::Draw(count)) => Ok(Need::Draw(count)),
                Ok(Move::Done) => Ok(self.next_file()),
                Err(Fail::Gone) => Err(Why::Missing),
                Err(Fail::Why(why)) => Err(why),
            },
            State::Org
            | State::MakeOrg
            | State::Repository
            | State::MakeRepository
            | State::Migrate
            | State::CopyBranch { .. }
            | State::Delete
            | State::Branch
            | State::Raw
            | State::File => self.answer(reply(event)?),
            State::Ended => unreachable!("terminal creation ignores events"),
        }
    }
    fn answer(&mut self, answer: Reply) -> Result<Need, Why> {
        match self.state {
            State::Org => match answer {
                Reply::Document(document) => {
                    let Document::Org(_) = *document else { return Err(Why::Unexpected) };
                    Ok(self.repository())
                }
                Reply::Status(404) => {
                    let owner = match &self.owner {
                        Some(owner) => owner.clone(),
                        None => self.admin_login.clone(),
                    };
                    self.state = State::MakeOrg;
                    Ok(self.send(Request::CreateOrg { person: owner, name: self.at.owner.clone() }))
                }
                other @ (Reply::Rows(_) | Reply::Empty | Reply::Status(_) | Reply::Down) => {
                    Err(unexpected(other))
                }
            },
            State::MakeOrg => match answer {
                Reply::Document(document) => {
                    let Document::Org(_) = *document else { return Err(Why::Unexpected) };
                    Ok(self.repository())
                }
                other @ (Reply::Rows(_) | Reply::Empty | Reply::Status(_) | Reply::Down) => {
                    Err(unexpected(other))
                }
            },
            State::Repository => match answer {
                Reply::Document(document) => {
                    let Document::Repository(repository) = *document else { return Err(Why::Unexpected) };
                    self.remember(repository.id, repository.default_branch)?;
                    Ok(match self.source {
                        Source::Empty => self.access(),
                        Source::Copy { .. } => {
                            self.state = State::CopyMark;
                            Need::Read(Item::Copying)
                        }
                    })
                }
                Reply::Status(404) => Ok(match self.source {
                    Source::Empty => {
                        self.state = State::MakeRepository;
                        self.send(Request::CreateRepository {
                            org: self.at.owner.clone(),
                            name: self.at.name.clone(),
                            private: true,
                        })
                    }
                    Source::Copy { .. } => self.mark(),
                }),
                other @ (Reply::Rows(_) | Reply::Empty | Reply::Status(_) | Reply::Down) => {
                    Err(unexpected(other))
                }
            },
            State::MakeRepository => match answer {
                Reply::Document(document) => {
                    let Document::CreatedRepository(repository) = *document else { return Err(Why::Unexpected) };
                    self.remember(repository.id, repository.default_branch)?;
                    Ok(self.access())
                }
                Reply::Status(409) => Ok(self.repository()),
                other @ (Reply::Rows(_) | Reply::Empty | Reply::Status(_) | Reply::Down) => {
                    Err(unexpected(other))
                }
            },
            State::Migrate | State::CopyBranch { .. } | State::Delete => self.copy_answer(answer),
            State::Branch | State::Raw | State::File => self.file_answer(answer),
            State::Admin(_) | State::CopyMark | State::Mark | State::Access | State::Ended => {
                unreachable!("creation reply phases")
            }
        }
    }
    fn mark(&mut self) -> Need {
        self.state = State::Mark;
        Need::Write { item: Item::Copying, bytes: Secret::new(Box::new([1])) }
    }
    fn copy_branch(&mut self, restart: bool) -> Need {
        self.state = State::CopyBranch { restart };
        self.send(Request::Branch { at: self.at.clone(), name: self.branch.clone() })
    }
    fn copy_answer(&mut self, answer: Reply) -> Result<Need, Why> {
        match self.state {
            State::Migrate => match answer {
                Reply::Document(document) => {
                    let Document::CreatedRepository(repository) = *document else { return Err(Why::Unexpected) };
                    self.remember(repository.id, repository.default_branch)?;
                    Ok(self.copy_branch(false))
                }
                Reply::Status(409) => Err(Why::Taken),
                other @ (Reply::Rows(_) | Reply::Empty | Reply::Status(_) | Reply::Down) => {
                    Err(unexpected(other))
                }
            },
            State::CopyBranch { restart } => match answer {
                Reply::Document(document) => {
                    let Document::Branch(_) = *document else { return Err(Why::Unexpected) };
                    Ok(self.access())
                }
                Reply::Status(404) => {
                    if restart {
                        self.state = State::Delete;
                        Ok(self.send(Request::DeleteRepository { at: self.at.clone() }))
                    } else {
                        Err(Why::Empty)
                    }
                }
                other @ (Reply::Rows(_) | Reply::Empty | Reply::Status(_) | Reply::Down) => {
                    Err(unexpected(other))
                }
            },
            State::Delete => match answer {
                Reply::Empty | Reply::Status(404) => Ok(self.mark()),
                other @ (Reply::Document(_) | Reply::Rows(_) | Reply::Status(_) | Reply::Down) => {
                    Err(unexpected(other))
                }
            },
            State::Admin(_)
            | State::Org
            | State::MakeOrg
            | State::Repository
            | State::MakeRepository
            | State::CopyMark
            | State::Mark
            | State::Access
            | State::Branch
            | State::Raw
            | State::File
            | State::Ended => unreachable!("copy reply phases"),
        }
    }
    fn file_answer(&mut self, answer: Reply) -> Result<Need, Why> {
        match self.state {
            State::Branch => match answer {
                Reply::Document(document) => {
                    let Document::Branch(branch) = *document else { return Err(Why::Unexpected) };
                    self.state = State::Raw;
                    Ok(self.send(Request::Raw {
                        at: self.at.clone(),
                        path: self.file().path.clone(),
                        reference: branch.commit,
                    }))
                }
                other @ (Reply::Rows(_) | Reply::Empty | Reply::Status(_) | Reply::Down) => {
                    Err(unexpected(other))
                }
            },
            State::Raw => match answer {
                Reply::Empty => Ok(self.file_done()),
                Reply::Status(404) => Ok(self.put_file()),
                other @ (Reply::Document(_) | Reply::Rows(_) | Reply::Status(_) | Reply::Down) => {
                    Err(unexpected(other))
                }
            },
            State::File => match answer {
                Reply::Document(document) => {
                    let Document::CreatedFile { .. } = *document else { return Err(Why::Unexpected) };
                    Ok(self.file_done())
                }
                other @ (Reply::Rows(_) | Reply::Empty | Reply::Status(_) | Reply::Down) => {
                    Err(unexpected(other))
                }
            },
            State::Admin(_)
            | State::Org
            | State::MakeOrg
            | State::Repository
            | State::MakeRepository
            | State::CopyMark
            | State::Mark
            | State::Migrate
            | State::CopyBranch { .. }
            | State::Delete
            | State::Access
            | State::Ended => unreachable!("file reply phases"),
        }
    }
}
/// One event in, at most one need out; an ended run ignores later events.
pub fn step(machine: &mut Create, event: Event, out: &mut Queue<Need>) {
    if let State::Ended = machine.state {
        return;
    }
    let need = match machine.advance(plan::normal(event)) {
        Ok(need) => need,
        Err(why) => machine.stop(why),
    };
    out.push(need);
}
/// Stops at the outstanding stage, releasing every held credential.
pub fn halt(machine: &mut Create, why: Why, out: &mut Queue<Need>) {
    if let State::Ended = machine.state {
        return;
    }
    out.push(machine.stop(why));
}
