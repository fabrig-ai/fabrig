use crate::Worker;
use skein_io::{self as io, kernel};
use temper_worker_protocol_git_world::io::{Below, Input, Invocation, Script};

pub struct WorkerStand {
    pub worker: Worker,
    pub below: Below,
    pub seed: u64,
}
impl WorkerStand {
    #[expect(clippy::new_without_default, reason = "the fixture explicitly constructs an owned worker")]
    #[must_use]
    pub fn new() -> Self {
        Self::with_worker(super::worker())
    }
    #[must_use]
    pub fn with_worker(worker: Worker) -> Self {
        let mut below = Below::new(Script::default());
        below.out.io = skein_lib::Queue::with_capacity(128);
        below.out.kernel = skein_lib::Queue::with_capacity(32);
        below.adopt_root(kernel::Fd::new(9));
        below.script.invocations.push_back(Invocation::captured("version"));
        Self { worker, below, seed: 7 }
    }
    pub fn turn(&mut self) -> bool {
        self.worker.iterate(super::NOW, skein_lib::Wall::EPOCH);
        self.drain();
        self.input() || self.worker.work_pending(super::NOW)
    }
    pub fn drain(&mut self) {
        for _ in 0..self.worker.io().len() {
            let request = self.worker.io().pop().expect("worker fixture values and IO fit their declared bounds");
            if let io::Request::Connect { .. } = request {
                continue;
            }
            self.below.out.io.push(request);
        }
        for _ in 0..self.worker.submitted().len() {
            self.below
                .out
                .kernel
                .push(self.worker.submitted().pop().expect("worker fixture values and IO fit their declared bounds"));
        }
        self.below.drain();
    }
    pub fn input(&mut self) -> bool {
        match self.below.next(&mut self.seed) {
            Some(Input::Io(event)) => self.worker.up(super::NOW, event),
            Some(Input::Kernel(complete)) => self
                .worker
                .complete(super::NOW, complete)
                .expect("worker fixture values and IO fit their declared bounds"),
            None => return false,
        }
        true
    }
}
#[must_use]
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

#[must_use]
pub fn socket_stand_with(settings: &crate::settings::Worker, agent: &crate::settings::Agent) -> WorkerStand {
    socket_stand_with_limits(settings, agent, &super::worker_limits())
}

#[must_use]
pub fn socket_stand_with_limits(
    settings: &crate::settings::Worker,
    agent: &crate::settings::Agent,
    own: &crate::worker_settings::Limits,
) -> WorkerStand {
    let worker = Worker::new(settings, agent, own, &super::worker_files(), super::worker_runtime(), 7)
        .expect("worker fixture values and IO fit their declared bounds");
    let mut stand = WorkerStand::with_worker(worker);
    for _ in 0_u16..1000 {
        if !stand.turn() {
            return stand;
        }
    }
    panic!("socket worker starts");
}
