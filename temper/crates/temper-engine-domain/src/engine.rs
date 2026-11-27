//! The root's pass-through to jig's engine (jig/docs/design/domain/engine/vocabulary.md, 2).

use crate::limits::{Limits, environment_core};
use jig_domain_core::Timer;
use jig_domain_core::engine::{self, Engine, In, Out};
use skein_lib::{Env, Queue};

pub(crate) fn step(engine: &mut Engine, env: &Env<Limits>, input: In) -> Queue<Out> {
    let mut out = Queue::uncapped();
    engine::step(engine, &environment_core(env), input, &mut out);
    out
}

pub(crate) fn fire(engine: &mut Engine, env: &Env<Limits>, timer: Timer) -> Queue<Out> {
    let mut out = Queue::uncapped();
    engine::fire(engine, &environment_core(env), timer, &mut out);
    out
}
