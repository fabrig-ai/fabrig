//! The root's pass-through to jig's engine (jig/docs/design/domain/engine/vocabulary.md, 2).

use crate::limits::{Limits, environment_core};
use jig_domain_core::engine::{self, Engine, In, Out};
use jig_domain_core::{Event, Request, Timer};
use skein_lib::{Env, Queue};

pub(crate) fn step(engine: &mut Engine, env: &Env<Limits>, event: Event) -> Queue<Request> {
    let mut out = Queue::uncapped();
    engine::step(engine, &environment_core(env), In::Core(event), &mut out);
    core_requests(out)
}

pub(crate) fn fire(engine: &mut Engine, env: &Env<Limits>, timer: Timer) -> Queue<Request> {
    let mut out = Queue::uncapped();
    engine::fire(engine, &environment_core(env), timer, &mut out);
    core_requests(out)
}

fn core_requests(mut out: Queue<Out>) -> Queue<Request> {
    let mut requests = Queue::uncapped();
    for _ in 0..out.len() {
        match out.pop().expect("engine output count") {
            Out::Core(request) => requests.push(request),
        }
    }
    requests
}
