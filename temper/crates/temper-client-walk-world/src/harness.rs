//! Pipe admission before the service's first iteration (journey-walks.md, 4.4).
use crate::box_peer::{BoxPeer, LINE_BYTES, PROGRAM};
use skein_io::kernel::{Complete, Done, Fd, Op, Pipe, Spawn, Submit, Way};
use skein_lib::{Time, Token, Wall};
use skein_sim_world::Inherited;
use temper_playground::{programs::Harness, service::Observe};

const ATTACH: Token = Token::new(u64::MAX);

/// The walk world's harness: the service and a pipe-only box peer.
pub struct Journey;
impl Harness for Journey {
    type Service = Attach;
    type Process = BoxPeer;
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
