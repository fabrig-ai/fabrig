//! Direct Core starts shared by step inputs and domain fixtures (jig/docs/design/domain/testing.md, 9).
use jig_domain_contract::{
    Declaration,
    effects::{EffectForm, EffectKind, Recovery},
};
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
    first_with_core(jig_test_profile::new_core(config, &env.limits).expect("configured core"), env)
}

#[must_use]
pub fn first_with_core(mut core: core::Core, env: &Env<core::Limits>) -> core::Core {
    let mut out = Queue::uncapped();
    let action = core.start_begin(false, env.wall);
    open(&mut core, env, &mut out, action);
    assert!(core.start_opened());
    core
}

#[must_use]
pub const fn effect_kind(kind: u16, waits_for_dependencies: bool) -> EffectKind {
    EffectKind { kind, form: EffectForm::Set, recovery: Recovery::Keyed, waits_for_dependencies }
}

#[must_use]
pub fn with_effects(config: core::Config, limits: &core::Limits, effects: &[EffectKind]) -> core::Core {
    let declarations: Box<[_]> = config
        .connectors
        .iter()
        .copied()
        .map(|connector| {
            let mut declaration = Declaration::empty(connector);
            if connector == crate::fixtures::CONNECTOR {
                declaration.effects = Box::from(effects);
            }
            declaration
        })
        .collect();
    core::Core::new(config, &declarations, limits).expect("configured core and its effect kinds")
}
