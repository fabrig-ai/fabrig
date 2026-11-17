//! The production host's close on the store world's simulated disk.
use jig_protocol_store::{
    Event, Request,
    host::{Hosted, Lane, NotReady},
};
use jig_store_world::root;
use skein_fake_machine::{Machine, serve};
use skein_io::kernel::{Complete, Op, Submit};
use skein_lib::{Queue, Token};
use skein_sim::{Config, Handle, Pid, Sim};
struct World {
    host: Hosted,
    machine: Machine,
    sim: Sim,
    pid: Pid,
    pending: Queue<Submit>,
}
impl World {
    fn new() -> Self {
        let mut machine = Machine::for_user(0);
        let root = machine.lay(&[]);
        let mut sim = Sim::new(44, Config::calm());
        let pid = sim.spawn_process();
        let fd = sim.root(pid, Handle::new(root.raw()));
        let mut host = Hosted::new(&root::limits(), root::APPLICATION, &[], 44, NotReady::Above);
        host.adopt(fd, 0, [1; 16]).expect("adopt");
        let mut world = Self { host, machine, sim, pid, pending: Queue::uncapped() };
        world.settle();
        world.drain();
        world.host.ask(Request::Commit {
            number: 1,
            writes: Box::new([
                jig_protocol_store::Write::Save(root::header(1)),
                jig_protocol_store::Write::Wires,
                jig_protocol_store::Write::Save(root::run_proof(root::LIVE)),
                jig_protocol_store::Write::Save(root::run_proof(root::LIVE + 1)),
            ]),
        });
        world.settle();
        let events = world.drain();
        assert!(events.iter().any(|event| matches!(event, Event::Committed { number: 1, .. })));
        world.host.ask(Request::Checkpoint { owner: Token::new(99), draw: [0; 16] });
        world
    }
    fn prepare(&mut self) {
        self.host.iterate();
        while let Some(jig_protocol_store::host::Submission { lane, mut submit }) = self.host.pop_submission() {
            let tag = u8::try_from(lane.index() + 1).expect("four lanes");
            submit.op = Token::tagged(tag, submit.op.raw()).expect("local operation");
            if let Op::Cancel { target } = &mut submit.kind {
                *target = Token::tagged(tag, target.raw()).expect("local cancel");
            }
            self.pending.push(submit);
        }
    }
    fn complete(&mut self) {
        self.sim.submit(self.pid, &mut self.pending);
        serve(&mut self.machine, &mut self.sim);
        let mut completed = Queue::<Complete>::uncapped();
        self.sim.reap(self.pid, &mut completed);
        while let Some(mut complete) = completed.pop() {
            let index = usize::from(complete.op.tag()).checked_sub(1).expect("one based tag");
            complete.op = Token::new(complete.op.untagged());
            if let Op::Cancel { target } = &mut complete.kind {
                *target = Token::new(target.untagged());
            }
            self.host.complete(*Lane::ALL.get(index).expect("lane"), complete);
        }
    }
    fn settle(&mut self) {
        for _ in 0..10000 {
            self.prepare();
            self.complete();
            if self.host.closing() && self.host.closed()
                || !self.host.closing() && !self.host.work_pending(skein_lib::Time::ZERO)
            {
                return;
            }
            if !self.host.closing()
                && self.pending.is_empty()
                && self.host.submissions() == 0
                && !self.host.component.work_pending()
                && !self.host.above.is_empty()
            {
                return;
            }
        }
        panic!("host did not settle");
    }
    fn drain(&mut self) -> Vec<Event> {
        let mut events = Vec::new();
        while let Some(event) = self.host.above.pop() {
            events.push(event);
        }
        events
    }
    fn unavailable(&mut self) {
        let events = self.drain();
        assert_eq!(events.iter().filter(|event| matches!(event, Event::CheckpointRefused { owner, why: jig_protocol_store::CheckpointRefusal::Unavailable } if *owner == Token::new(99))).count(), 1, "{events:?}");
        assert!(!events.iter().any(|event| matches!(event, Event::Checkpointed { .. })));
        assert!(self.host.closed());
    }
}
#[test]
fn close_waits_for_the_checkpoint_copy_in_flight_and_answers_unavailable_once() {
    let mut world = World::new();
    let mut found = false;
    for _ in 0..10000 {
        world.prepare();
        if world.pending.iter().any(|request| {
            request.op.tag() == u8::try_from(Lane::Checkpoints.index() + 1).expect("lane")
                && matches!(request.kind, Op::Read { .. })
        }) {
            found = true;
            break;
        }
        world.complete();
    }
    assert!(found, "copy reaches an in-flight read");
    world.host.close();
    assert!(world.host.above.is_empty(), "no terminal before its in-flight file request finishes");
    world.complete();
    world.settle();
    world.unavailable();
}
#[test]
fn close_at_the_checkpoint_barrier_answers_unavailable_and_closes_the_store() {
    let mut world = World::new();
    let mut found = false;
    for _ in 0..10000 {
        world.prepare();
        if world.pending.is_empty() && world.host.component.work_pending() {
            found = true;
            break;
        }
        world.complete();
    }
    assert!(found, "selection yields with no file request in flight");
    world.host.close();
    assert!(world.host.above.iter().any(|event| matches!(
        event,
        Event::CheckpointRefused { why: jig_protocol_store::CheckpointRefusal::Unavailable, .. }
    )));
    world.settle();
    world.unavailable();
}
