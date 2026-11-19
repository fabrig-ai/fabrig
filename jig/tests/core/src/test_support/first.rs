//! Direct Core starts shared by step inputs and domain fixtures (jig/docs/design/domain/testing.md, 9).
use jig_domain_core::{
    self as core,
    start::{Action, Phase, SweepInput},
};
use skein_lib::{Env, Queue};

pub fn open(core: &mut core::Core, env: &Env<core::Limits>, out: &mut Queue<core::Request>, mut action: Action) {
    for _ in 0..32 {
        action = match action {
            Action::Step(Phase::Sweep) => core.start_sweep(env, SweepInput::Finished, out),
            Action::Step(done) => core.start_done(env, done, out),
            Action::Commit(token) => core.start_resume(env, token, out),
            Action::Idle => return,
            Action::Refused(failure) => panic!("stored start fits: {failure:?}"),
        };
        out.clear();
    }
    panic!("bounded start opens");
}

#[must_use]
pub fn first(config: core::Config, env: &Env<core::Limits>) -> core::Core {
    let mut core = core::Core::new(config, &env.limits).expect("configured core");
    let mut out = Queue::uncapped();
    let action = core.start_begin(false, env.wall);
    open(&mut core, env, &mut out, action);
    assert!(core.start_opened());
    core
}
