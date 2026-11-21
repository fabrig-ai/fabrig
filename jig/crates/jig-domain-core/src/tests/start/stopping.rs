use super::*;

fn wires() -> (Core, Env<crate::Limits>, crate::start::Continuation) {
    let limits = limits();
    let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits };
    let mut configured = config(41);
    configured.grants = Box::new([]);
    configured.workers_in_process = false;
    let mut core = Core::new(configured, &limits).expect("core");
    let mut out = Queue::uncapped();
    core.start_begin(false, env.wall);
    through_link(&mut core, &env, &mut out);
    core.start_sweep(&env, SweepInput::Finished, &mut out);
    core.start_done(&env, Phase::Switch, &mut out);
    let Action::Commit(continuation) = core.start_done(&env, Phase::Wires, &mut out) else {
        panic!("wires continuation")
    };
    assert!(out.is_empty());
    (core, env, continuation)
}

#[test]
fn stopping_a_start_drains_without_a_mark_or_restart_word() {
    for held in [false, true] {
        let (mut core, env, continuation) = wires();
        if !held {
            // The loading boundary has no runtime input and no run adopted.
            let mut configured = config(41);
            configured.grants = Box::new([]);
            core = Core::new(configured, &env.limits).expect("loading core");
            core.start_begin(false, env.wall);
        }
        let mut out = crate::step(&mut core, &env, crate::Event::Stopping);
        assert!(core.stopping());
        assert!(!core.settling());
        assert_eq!(core.counters.deployment().restart_wait_ms, None);
        let Some(Request::Held(held_output)) = out.pop() else { panic!("journal-held drain") };
        let crate::Held::Drained = *held_output else { panic!("drained before runtime") };
        assert!(out.is_empty(), "no claims settled, mark committed, phase asked or restart word said");
        if held {
            core.start
                .held
                .push(Request::Held(Box::new(crate::Held::Restarting { wait: env.limits.fleet.restart_wait })));
            let Action::Idle = core.start_resume(&env, continuation, &mut out) else { panic!("stopped continuation") };
            assert!(core.start.held.is_empty(), "the released decision's continuation drops its held work");
        } else {
            let Action::Idle = core.start_done(&env, Phase::LoadCore, &mut out) else { panic!("stopped load") };
        }
        assert!(out.is_empty());
        assert!(crate::step(&mut core, &env, crate::Event::Stopping).is_empty(), "drained only once");
    }
}

#[test]
fn settling_a_marked_start_keeps_and_says_the_restart_word_before_drained() {
    let (mut core, env, continuation) = wires();
    let mut out = Queue::uncapped();
    step_phase(core.start_resume(&env, continuation, &mut out), Phase::AdoptRuns);
    assert!(crate::step(&mut core, &env, crate::Event::Settling).is_empty());
    let mut out = crate::step(&mut core, &env, crate::Event::Stopping);
    assert_eq!(core.counters.deployment().restart_wait_ms, Some(env.limits.fleet.restart_wait.as_millis()));
    let Some(Request::Held(word)) = out.pop() else { panic!("restart word") };
    let crate::Held::Restarting { wait } = *word else { panic!("restart word before drain") };
    assert_eq!(wait, env.limits.fleet.restart_wait);
    let Some(Request::Held(drained)) = out.pop() else { panic!("drained") };
    let crate::Held::Drained = *drained else { panic!("drained after restart word") };
    assert!(out.is_empty());
}
