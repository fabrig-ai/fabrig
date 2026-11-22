//! Per-account provider faults leave another party's goal running (checks.md, 6.3).
use skein_fake_llm_domain::api::Strike;
use skein_lib::Duration;
use skein_llm_kit::faults::{EXPIRED_SIGN_IN, RATE_WINDOW, SPENT_CREDITS};
use skein_world::{
    streams::{Stream, Streams},
    sweep::Sweep,
};
use temper_system_world::composed::{
    World,
    accounts::{FIRST_ACCOUNT, OTHER_ACCOUNT},
};

#[test]
fn an_account_fault_holds_only_its_goals() {
    const OTHER_GOAL_MINUTES: u64 = 3;
    const OTHER_GOAL_STEPS: u32 = 20_000;
    const RECOVERY_MINUTES: u64 = 3;
    const RECOVERY_STEPS: u32 = 20_000;
    let mut sweep = Sweep::new("system accounts");
    for kind in [SPENT_CREDITS, EXPIRED_SIGN_IN, RATE_WINDOW] {
        sweep.configure(kind);
    }
    for seed in 0..12 {
        eprintln!("seed {seed}");
        let streams = Streams::new(seed);
        let mut rng = streams.rng(Stream::Faults, "llm");
        let strike = match rng.below(3) {
            0 => Strike::Spent,
            1 => Strike::SignedOut,
            _ => Strike::Window { tokens: 1, length: Duration::from_millis(50 + rng.below(50)) },
        };
        let account = if rng.below(2) == 0 { FIRST_ACCOUNT } else { OTHER_ACCOUNT };
        let at_call = usize::try_from(rng.below(2)).expect("two query steps");
        let clear = (rng.below(2) == 0).then(|| u32::try_from(20 + rng.below(40)).expect("bounded clear step"));
        eprintln!("{strike:?}, account {account}, call {at_call}, clear after {clear:?}");
        let mut world = World::two_accounts(streams.seed_of(Stream::Scheduling, "system"));
        world.plan_account_fault(account, strike, at_call, clear);
        let outcome = world.run_account_case(
            account,
            clear.is_some(),
            (Duration::from_secs(60 * OTHER_GOAL_MINUTES), OTHER_GOAL_STEPS),
            (Duration::from_secs(60 * RECOVERY_MINUTES), RECOVERY_STEPS),
        );
        world.judge_account_runs().expect("blame and task tries follow the account fault");
        outcome.expect("bounded account isolation");
        sweep.ran(world.account_ledger());
    }
    sweep.check_fell().expect("each configured account fault fell");
}
