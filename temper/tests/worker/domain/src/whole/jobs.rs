//! Whole job stories with the real root, hub, capability and checkout, and
//! the fake disk and forge (temper/docs/design/domain/worker.md, 4.4).

use std::collections::VecDeque;

use skein_lib::{Duration, Env, Queue, Time, Token, Wall};
use temper_fake_checkout::{
    Checkout,
    git::{Remote, Tree},
};
use temper_worker_checkout_world::{forge::Forge, translate};
use temper_worker_domain::{Domain, Event, Request, checkout::git, host, tooling, wire};

const RUN: Token = Token::new(81);
const ATTEMPT: Token = Token::new(13);
const REMOTE: &[u8] = b"org/jobs";
const RESULT: &[u8] = b"{\"checked\":true}";

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Case {
    Hello,
    Finished,
    Delivered,
    Stale,
    Cancelled,
    Undecodable,
    GoneBeforeRead,
}

#[derive(PartialEq, Eq, Debug)]
pub struct Report {
    pub kinds: Box<[host::JobBound]>,
    pub answer: Option<host::JobEnding>,
    pub tree: Tree,
    pub pushes: u32,
    pub kills: u32,
    pub trace: Vec<String>,
}

pub(super) struct World {
    kind: tooling::Kind,
    pub(super) repository: Box<[u8]>,
    pub(super) domain: Domain,
    env: Env<temper_worker_domain::Limits>,
    out: Queue<Request>,
    owed: VecDeque<Event>,
    pub(super) operations: VecDeque<(Token, tooling::Request)>,
    pub(super) disk: Checkout,
    pub(super) forge: Forge,
    pub(super) workspace: Option<Token>,
    held_push: Option<(Token, git::Op)>,
    pub(super) report: Report,
    pub(super) stops: u32,
}

fn kind(delivers: bool) -> tooling::Kind {
    tooling::Kind {
        bound: host::JobBound { kind: host::JobKind(9), spec: 128, result: 128, deadline: Duration::from_secs(10) },
        verb: Box::from(b"check".as_slice()),
        delivers,
    }
}

impl World {
    pub(super) fn new(seed: u64, kind: tooling::Kind, limits: temper_worker_domain::Limits) -> Self {
        let mut forge = Forge::new(seed);
        let first = forge.repository(
            REMOTE,
            b"main",
            [
                (b"code".to_vec(), b"original\n".to_vec()),
                ([b"tools/", kind.verb.as_ref()].concat(), b"command\n".to_vec()),
            ]
            .into(),
        );
        forge.create_branch(REMOTE, b"topic", first).expect("delivery branch");
        Self {
            domain: Domain::new(&limits, Box::new([kind.clone()])),
            kind,
            repository: b"work".as_slice().into(),
            env: Env { now: Time::ZERO, wall: Wall::EPOCH, limits },
            out: Queue::uncapped(),
            owed: VecDeque::new(),
            operations: VecDeque::new(),
            disk: Checkout::new(),
            forge,
            workspace: None,
            held_push: None,
            stops: 0,
            report: Report {
                kinds: Box::new([]),
                answer: None,
                tree: Tree::new(),
                pushes: 0,
                kills: 0,
                trace: Vec::new(),
            },
        }
    }

    pub(super) fn event(&mut self, event: Event) {
        self.owed.push_back(event);
        for _ in 0..128 {
            while let Some(request) = self.out.pop() {
                self.report.trace.push(format!("<- {request:?}"));
                self.take(request);
            }
            self.domain.reclaim();
            assert_eq!(self.domain.hosting().len(), self.domain.host().unanswered());
            if self.domain.is_ready() {
                temper_worker_domain::resume(&mut self.domain, &self.env, &mut self.out);
                continue;
            }
            let Some(event) = self.owed.pop_front() else { return };
            self.report.trace.push(format!("-> {event:?}"));
            temper_worker_domain::step(&mut self.domain, &self.env, event, &mut self.out);
        }
        panic!("the job story settles");
    }

    fn take(&mut self, request: Request) {
        match request {
            Request::Hello(hello) => self.report.kinds = hello.jobs,
            Request::Link(host::LinkUp::JobAnswer { run, attempt, answer }) => {
                assert_eq!((run, attempt), (RUN, ATTEMPT));
                assert_eq!(self.domain.checkout().idle(), self.domain.checkout().workspaces(), "release before answer");
                assert!(self.held_push.is_none(), "push settles before answer");
                assert!(self.report.answer.replace(answer).is_none(), "one answer before acknowledgement");
            }
            Request::Io { owner, op, deadline } => {
                assert!(deadline > self.env.now);
                match &op {
                    git::Op::Make { workspace } => self.workspace = Some(*workspace),
                    git::Op::Commit { at, title, body, .. } => {
                        assert_eq!(at.repository, self.repository);
                        assert_eq!(title.as_ref(), format!("job {}", self.kind.bound.kind.0).as_bytes());
                        assert!(body.is_empty());
                    }
                    git::Op::Push { at, .. } => {
                        assert_eq!(at.repository, self.repository);
                        self.report.pushes += 1;
                        assert!(self.held_push.replace((owner, op)).is_none());
                        return;
                    }
                    git::Op::Clone { .. }
                    | git::Op::Fetch { .. }
                    | git::Op::Create { .. }
                    | git::Op::CheckOut { .. }
                    | git::Op::IsAncestor { .. }
                    | git::Op::Merge { .. } => {}
                }
                let done = translate::perform(&mut self.forge, &mut self.disk, op);
                self.owed.push_back(Event::Done { owner, done });
            }
            Request::Job { owner, op } => self.operations.push_back((owner, op)),
            Request::Stop => self.stops += 1,
            Request::Source { .. }
            | Request::SourceFault
            | Request::Link(_)
            | Request::Answer { .. }
            | Request::CancelRelay { .. }
            | Request::Process(_)
            | Request::Decode { .. }
            | Request::Encode { .. }
            | Request::Fact { .. }
            | Request::CancelIo { .. } => panic!("unscripted request"),
        }
    }

    fn assign(&mut self, decoded: bool, delivers: bool) {
        let repositories = [b"home".as_slice(), b"work".as_slice()]
            .into_iter()
            .enumerate()
            .map(|(index, name)| wire::Repository {
                tag: u64::try_from(index).expect("repository tag"),
                name: name.into(),
                remote: REMOTE.into(),
                save: None,
                git: None,
                start: wire::Start::Base { branch: b"main".as_slice().into() },
                access: if name == b"work" && delivers {
                    wire::Access::Writable { push: b"topic".as_slice().into(), landing: None, expected: None }
                } else {
                    wire::Access::ReadOnly
                },
                identity: 7,
                account: None,
            })
            .collect();
        let spec = if decoded {
            Ok(wire::JobSpec { command: b"tools/check".as_slice().into(), request: b"request".as_slice().into() })
        } else {
            Err(wire::Undecodable)
        };
        self.event(Event::AssignJob {
            job: host::JobAssignment {
                run: RUN,
                attempt: ATTEMPT,
                kind: host::JobKind(9),
                spec: b"encoded spec".as_slice().into(),
                deadline: Duration::from_secs(5),
                grants: Box::new([]),
                workspace: Some(host::Workspace { workstream: 81, items: Token::new(0) }),
            },
            items: Ok(wire::Workspace { repositories }),
            spec,
        });
    }

    pub(super) fn start(&mut self) -> Token {
        let (owner, op) = self.operations.pop_front().expect("write request");
        let tooling::Request::Write { owner: inner, path, bytes } = op else { panic!("write before spawn") };
        assert_eq!(owner, inner);
        let workspace = self.workspace.expect("checkout prepared");
        let directory = format!("workspaces/{}", workspace.raw()).into_bytes();
        assert_eq!(path.as_ref(), [directory.as_slice(), b"/request.json"].concat());
        assert_eq!(bytes.as_ref(), b"request");
        self.disk.write(&[translate::dir(workspace).as_slice(), b"/request.json"].concat(), &bytes);
        self.event(Event::Job { owner, event: tooling::Event::Written { owner, written: true } });
        let (spawn_owner, op) = self.operations.pop_front().expect("spawn after write");
        assert_eq!(spawn_owner, owner);
        let tooling::Request::Spawn { owner: inner, program, args, env, dir, deadline } = op else { panic!("spawn") };
        assert_eq!(inner, owner);
        assert_eq!(dir.as_ref(), directory);
        assert_eq!(program.as_ref(), [directory.as_slice(), b"/home/tools/", self.kind.verb.as_ref()].concat());
        assert_eq!(args.as_ref(), [program.clone(), self.kind.verb.clone(), path]);
        assert_eq!(
            env.as_ref(),
            [
                (b"PATH".as_slice().into(), b"/usr/bin:/bin".as_slice().into()),
                (b"HOME".as_slice().into(), dir.clone()),
                (b"TEMPER_JOB".as_slice().into(), dir),
                (b"LANG".as_slice().into(), b"C.UTF-8".as_slice().into()),
            ]
        );
        assert_eq!(deadline, self.env.now.saturating_add(Duration::from_secs(5)));
        assert!(
            self.disk
                .exists(&[translate::dir(workspace).as_slice(), b"/home/tools/", self.kind.verb.as_ref()].concat())
        );
        self.event(Event::Job { owner, event: tooling::Event::Spawned { owner } });
        owner
    }

    fn finish(&mut self, owner: Token, gone_first: bool) {
        let workspace = self.workspace.expect("prepared checkout");
        let fake_dir = translate::dir(workspace);
        // The command leaves its edit uncommitted: the checkout commits it.
        self.disk.write(&[fake_dir.as_slice(), b"/work/code"].concat(), b"checked\n");
        self.disk.write(&[fake_dir.as_slice(), b"/result.json"].concat(), RESULT);
        self.event(Event::Job {
            owner,
            event: tooling::Event::Exited { owner, exit: tooling::Exit::Code(0), tail: Box::new([]) },
        });
        let (read_owner, op) = self.operations.pop_front().expect("result read after exit");
        assert_eq!(read_owner, owner);
        let tooling::Request::Read { path, most, .. } = op else { panic!("read result") };
        assert_eq!(path.as_ref(), format!("workspaces/{}/result.json", workspace.raw()).as_bytes());
        assert_eq!(most, 128);
        if gone_first {
            self.event(Event::Job { owner, event: tooling::Event::Gone { owner } });
            assert!(self.report.answer.is_none(), "the read still owns the workspace");
        }
        self.event(Event::Job { owner, event: tooling::Event::Read { owner, bytes: Some(RESULT.into()) } });
    }

    pub(super) fn pushed(&mut self, stale: bool) {
        let (owner, op) = self.held_push.take().expect("job delivery began");
        if stale {
            self.forge.advance(REMOTE, b"topic", b"code", b"another writer\n");
        }
        let done = translate::perform(&mut self.forge, &mut self.disk, op);
        self.event(Event::Done { owner, done });
    }
}

/// Drives a complete job through its checkout and process terminals.
#[must_use]
pub fn run(seed: u64, case: Case) -> Report {
    let delivers = matches!(case, Case::Delivered | Case::Stale | Case::Cancelled);
    let mut world = World::new(seed, kind(delivers), crate::limits());
    world.event(Event::Ready);
    assert_eq!(world.report.kinds.as_ref(), [kind(delivers).bound]);
    if case != Case::Hello {
        world.assign(case != Case::Undecodable, delivers);
        if case != Case::Undecodable {
            let owner = world.start();
            if case == Case::Cancelled {
                world.event(Event::Link(host::LinkDown::Cancel { run: RUN, attempt: ATTEMPT }));
                let (kill_owner, op) = world.operations.pop_front().expect("kill on cancel");
                assert_eq!(kill_owner, owner);
                assert_eq!(op, tooling::Request::Kill { owner });
                world.report.kills += 1;
                world.event(Event::Job {
                    owner,
                    event: tooling::Event::Exited { owner, exit: tooling::Exit::Signal, tail: Box::new([]) },
                });
            } else {
                world.finish(owner, case == Case::GoneBeforeRead);
                if delivers {
                    assert!(world.report.answer.is_none(), "wait for the push");
                    world.pushed(case == Case::Stale);
                }
            }
            if case != Case::GoneBeforeRead {
                assert!(world.report.answer.is_none(), "wait for Gone");
                assert_eq!(world.domain.workspaces(), 1);
                world.event(Event::Job { owner, event: tooling::Event::Gone { owner } });
            }
            assert!(world.operations.is_empty());
            world.event(Event::Job { owner, event: tooling::Event::Gone { owner } });
        }
        assert_eq!(world.domain.workspaces(), 0);
        assert_eq!(world.domain.agent().agents(), 0);
        let answer = world.report.answer.as_ref().expect("one answer");
        match case {
            Case::Finished | Case::Delivered | Case::GoneBeforeRead => {
                let host::JobEnding::Finished { result, work } = answer else { panic!("job finished: {answer:?}") };
                assert_eq!(result.as_ref(), RESULT);
                assert_eq!(work.left.is_some(), delivers);
                assert!(work.saved.is_none());
            }
            Case::Stale | Case::Cancelled => {
                let host::JobEnding::Failed { failure, work, .. } = answer else { panic!("job failed") };
                let expected = if case == Case::Stale {
                    host::JobFailure::Undelivered(host::DeliveryOutcome::Stale)
                } else {
                    host::JobFailure::Cancelled(host::Reason::Engine)
                };
                assert_eq!(*failure, expected);
                assert_eq!(*work, host::Work { left: None, saved: None });
            }
            Case::Undecodable => assert!(matches!(
                answer,
                host::JobEnding::Refused(host::JobRefusal::Invalid(host::JobInvalid::Start { .. }))
            )),
            Case::Hello => unreachable!("hello only has no job"),
        }
        world.event(Event::Link(host::LinkDown::Acknowledge { run: RUN, attempt: ATTEMPT }));
    }
    let commit = world.forge.branch(REMOTE, b"topic").expect("delivery branch");
    world.report.tree = world.forge.tree(commit);
    world.event(Event::Shutdown);
    assert_eq!(world.stops, 1);
    assert_eq!(world.domain.kept_answers(), 0);
    world.report
}
