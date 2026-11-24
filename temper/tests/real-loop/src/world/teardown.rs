//! Startup failure is terminal after the unreleased caller closes its descriptors.
use super::{ForgePeer, Judge, Kind, Llm, Proc, cluster_judge, host};
use crate::{agent, llm, service::ServiceHost};
use skein_lib::Time;
use skein_shell::Host;
use skein_world::{Referee, real::World};
use std::{cell::Cell, net::TcpListener, rc::Rc};

#[derive(Default)]
pub struct Receipt {
    pub settled: Cell<bool>,
    pub calls: Cell<usize>,
    pub passed: Cell<bool>,
}
struct Recording {
    judge: Judge,
    receipt: Rc<Receipt>,
}
impl Referee<Proc> for Recording {
    fn act(&mut self, now: Time, procs: &mut [Proc]) {
        self.judge.act(now, procs);
    }
    fn watch(&mut self, now: Time, procs: &[Proc]) -> Result<(), String> {
        let finding = self.judge.watch(now, procs);
        if self.judge.failure.is_some() {
            self.receipt.settled.set(procs.iter().all(Host::is_empty));
            self.receipt.passed.set(self.judge.passed());
            let Kind::Caller(caller) = &procs[2].kind else { return Err("caller root".into()) };
            self.receipt.calls.set(caller.calls.len());
        }
        finding
    }
    fn next_deadline(&self) -> Option<Time> {
        self.judge.next_deadline()
    }
    fn overdue(&self, now: Time) -> Option<String> {
        self.judge.overdue(now)
    }
    fn passed(&self) -> bool {
        self.judge.passed()
    }
}

pub fn startup(
    scratch: &skein_scratch::Scratch,
    occupied: Option<&TcpListener>,
    receipt: Rc<Receipt>,
) -> Result<skein_world::real::Outcome<Proc>, String> {
    let patience = crate::PATIENCE;
    let (state, peer) = crate::github::State::new();
    let mut world = World::new_controlled(|controls| {
        let mut judge = cluster_judge(controls, &[], None, patience);
        judge.github = Some(state);
        Recording { judge, receipt }
    });
    world.spawn(|| {
        Proc::new(Kind::Llm(Box::new(Llm {
            pending_streak: crate::patience::PendingStreak::default(),
            peer: llm::peer(llm::scripted(Box::new([]))).expect("deferred LLM"),
            scripts: Some(|_| Box::new([])),
            rules: None,
            cue: false,
            placement_hold: true,
            holds: Vec::new(),
            release: None,
            released: None,
        })))
    });
    world.spawn_signals(|signals| {
        let mut service = ServiceHost::github(scratch.path().into(), signals);
        if let Some(listener) = occupied {
            service.clients_occupied_by(listener);
        }
        Proc::new(Kind::Service(Box::new(service)))
    });
    host(&mut world, scratch.path(), || vec![crate::calls::sign_in(crate::calls::D1)], true, 1, agent::agent);
    world.spawn(|| Proc::new(Kind::Forge(Box::new(ForgePeer::new(peer)))));
    world.try_run(&skein_shell::Clock::new(), patience)
}
