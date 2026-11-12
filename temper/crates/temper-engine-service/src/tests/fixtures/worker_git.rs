use crate::Worker;
use skein_io::{self as io, kernel};
use temper_worker_protocol_git_world::io::{Below, Input, Invocation, Script};

pub struct WorkerStand {
    pub worker: Worker,
    pub below: Below,
    pub seed: u64,
}
impl WorkerStand {
    pub fn new() -> Self {
        let mut below = Below::new(Script::default());
        below.out.io = skein_lib::Queue::with_capacity(128);
        below.out.kernel = skein_lib::Queue::with_capacity(32);
        below.adopt_root(kernel::Fd::new(9));
        below.script.invocations.push_back(Invocation::captured("version"));
        Self { worker: super::worker(), below, seed: 7 }
    }
    pub fn turn(&mut self) -> bool {
        self.worker.iterate(super::NOW, skein_lib::Wall::EPOCH);
        self.drain();
        self.input() || self.worker.work_pending(super::NOW)
    }
    pub fn drain(&mut self) {
        for _ in 0..self.worker.io().len() {
            let request = self.worker.io().pop().unwrap();
            if let io::Request::Connect { .. } = request {
                continue;
            }
            self.below.out.io.push(request);
        }
        for _ in 0..self.worker.submitted().len() {
            self.below.out.kernel.push(self.worker.submitted().pop().unwrap());
        }
        self.below.drain();
    }
    pub fn input(&mut self) -> bool {
        match self.below.next(&mut self.seed) {
            Some(Input::Io(event)) => self.worker.up(super::NOW, event),
            Some(Input::Kernel(complete)) => self.worker.complete(super::NOW, complete).unwrap(),
            None => return false,
        }
        true
    }
}
pub fn started_worker() -> WorkerStand {
    let mut stand = WorkerStand::new();
    for _ in 0_u16..1000 {
        if !stand.turn() {
            assert!(stand.worker.git_ready());
            return stand;
        }
    }
    panic!("git start finishes");
}
