//! Pipe admission before the service's first iteration (journey-walks.md, 4.4).
use crate::box_peer::{BoxPeer, LINE_BYTES, PROGRAM};
use skein_io::kernel::{Complete, Done, Exit, Fd, Op, Pipe, Spawn, Submit, Way};
use skein_lib::{Queue, Time, Token, Wall};
use skein_shell::Host;
use skein_sim_world::Inherited;
use temper_playground::{programs::Harness, service::Observe};

const ATTACH: Token = Token::new(u64::MAX);

/// The walk world's harness: the service, box peer, terminal connection and tab.
pub enum Clients {}
impl Harness for Clients {
    type Service = Attach;
    type Process = Native;
    fn prepare(
        disk: &mut skein_fake_machine::Machine,
        document: &temper_playground::documents::Configuration,
        _seed: u64,
    ) {
        crate::facts::prepare(disk, document);
    }
    fn files() -> temper_service::settings::Files {
        crate::facts::files()
    }
}

/// The walk's native processes, scheduled by the composition's one loop.
pub enum Native {
    Box(Box<BoxPeer>),
    Terminal(Box<crate::tui::Connection>),
    Tab(Box<skein_playground::tab::Tab>),
}
impl Native {
    fn host(&self) -> &dyn Host {
        match self {
            Self::Box(host) => host.as_ref(),
            Self::Terminal(host) => host.as_ref(),
            Self::Tab(host) => host.as_ref(),
        }
    }
    fn host_mut(&mut self) -> &mut dyn Host {
        match self {
            Self::Box(host) => host.as_mut(),
            Self::Terminal(host) => host.as_mut(),
            Self::Tab(host) => host.as_mut(),
        }
    }
}
impl Host for Native {
    fn state_digest(&self) -> Option<[u8; 32]> {
        self.host().state_digest()
    }
    fn drain(&mut self) {
        self.host_mut().drain();
    }
    fn iterate(&mut self, now: Time, wall: Wall) {
        self.host_mut().iterate(now, wall);
    }
    fn completions(&mut self) -> &mut Queue<Complete> {
        self.host_mut().completions()
    }
    fn submissions(&mut self) -> &mut Queue<Submit> {
        self.host_mut().submissions()
    }
    fn work_pending(&self, now: Time) -> bool {
        self.host().work_pending(now)
    }
    fn next_mechanism_deadline(&self) -> Option<Time> {
        self.host().next_mechanism_deadline()
    }
    fn next_policy_deadline(&self) -> Option<Time> {
        self.host().next_policy_deadline()
    }
    fn is_empty(&self) -> bool {
        self.host().is_empty()
    }
    fn operations(&self) -> u32 {
        self.host().operations()
    }
    fn exit(&self) -> Option<Exit> {
        self.host().exit()
    }
}

/// The service spawns the peer before doing work, retaining composition indices
/// and its stdout printer. The pipes have the same directions as under a box.
pub struct Attach {
    root: Fd,
    queued: bool,
    attached: bool,
}
impl Observe for Attach {
    fn started(root: Fd, _: &Inherited) -> Self {
        Self { root, queued: false, attached: false }
    }
    fn before_iterate(&mut self, service: &mut temper_service::Service, _: Time, _: Wall) -> bool {
        // The native harness is the diagnostic sink, as the service shell is.
        while service.notes().pop().is_some() {}
        if !self.queued {
            self.queued = true;
            service.submissions().push(Submit {
                op: ATTACH,
                kind: Op::Spawn {
                    spawn: Box::new(Spawn {
                        reach: None,
                        program: PROGRAM.into(),
                        args: Box::new([]),
                        env: Box::new([]),
                        root: self.root,
                        dir: Box::new([]),
                        pipes: [(3, Way::In), (4, Way::Out)]
                            .map(|(child, way)| Pipe { child, way, parent: None })
                            .into(),
                    }),
                },
            });
        }
        !self.attached
    }
    fn complete(&mut self, complete: Complete, service: &mut temper_service::Service) -> Option<Complete> {
        if complete.op != ATTACH {
            return Some(complete);
        }
        match (complete.kind, complete.result.expect("peer startup")) {
            (Op::Spawn { spawn }, Done::Spawned { pidfd, .. }) => {
                let pipe = |child| {
                    spawn
                        .pipes
                        .iter()
                        .find(|pipe| pipe.child == child)
                        .and_then(|pipe| pipe.parent)
                        .expect("parent pipe")
                };
                service.adopt_box(pipe(3), pipe(4), LINE_BYTES).expect("world box pipes");
                service.submissions().push(Submit { op: ATTACH, kind: Op::Close { fd: pidfd } });
            }
            (Op::Close { .. }, Done::Nothing) => self.attached = true,
            (op, done) => panic!("peer startup {op:?}: {done:?}"),
        }
        None
    }
}
