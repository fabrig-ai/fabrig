//! Decision opening gates stale listings when the start opens (engine.md, 6, 8).
use jig_domain_core::{
    self as core,
    start::{Action, Phase, SweepInput},
};
use jig_domain_fleet as fleet;
use skein_lib::{Duration, Env, Queue, Time, Token, Wall};

fn start_to_open(core: &mut core::Core, env: &Env<core::Limits>) {
    let mut out = Queue::uncapped();
    let mut action = core.start_begin(false, env.wall);
    for _ in 0..32 {
        action = match action {
            Action::Step(Phase::Open) => return,
            Action::Step(Phase::Sweep) => core.start_sweep(env, SweepInput::Finished, &mut out),
            Action::Step(phase) => core.start_done(env, phase, &mut out),
            Action::Commit(token) => core.start_resume(env, token, &mut out),
            Action::Idle | Action::Refused(_) => panic!("start reaches Open"),
        };
        out.clear();
    }
    panic!("bounded start");
}

fn listing(core: &mut core::Core, env: &Env<core::Limits>, worker: u32) -> Vec<(Token, Token, Token)> {
    let channel = Token::new(7 + u64::from(worker));
    let run = Token::new(41 + u64::from(worker));
    let attempt = Token::new(11);
    let mut out = core::step(
        core,
        env,
        core::Event::Fleet(fleet::Event::Hello {
            worker,
            channel,
            hello: fleet::Hello {
                bounds: jig_test_profile::fixtures::bounds(),
                stop_bound: Duration::ZERO,
                slots: 1,
                workstreams: Box::new([]),
                hosting: Box::new([fleet::Hosting { run, attempt, phase: fleet::Phase::Active }]),
            },
        }),
    );
    let mut cancelled = Vec::new();
    while let Some(request) = out.pop() {
        if let core::Request::Held(held) = request
            && let core::Held::Cancel { channel, run, attempt } = *held
        {
            cancelled.push((channel, run, attempt));
        }
    }
    cancelled
}

#[test]
fn stale_worker_listings_cancel_only_after_either_start_opens() {
    {
        let mut limits = jig_test_profile::limits().core;
        limits.fleet.workers = 2;
        let mut configured = jig_test_profile::config(301).core;
        configured.worker_names = Box::new([Box::from(b"alpha".as_slice()), Box::from(b"beta".as_slice())]);
        let mut core = core::Core::new(configured, &limits).expect("configured core");
        let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits };
        {
            start_to_open(&mut core, &env);
        }
        assert!(!core.start_opened());
        assert!(!core.start_opened());
        assert!(listing(&mut core, &env, 0).is_empty());
        {
            assert!(matches!(core.start_done(&env, Phase::Open, &mut Queue::uncapped()), Action::Idle));
        }
        assert!(core.start_opened());
        assert!(core.start_opened());
        assert_eq!(listing(&mut core, &env, 1), [(Token::new(8), Token::new(42), Token::new(11))]);
    }
}
