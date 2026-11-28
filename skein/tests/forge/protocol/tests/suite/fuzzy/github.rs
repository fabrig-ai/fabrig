//! GitHub cases through the sweep seam (skein/docs/design/testing/sweeps.md, 4.2 to 4.5).

use skein_forge::api::Read;
use skein_forge_protocol_world::{World, fixtures};
use skein_lib::{Duration, Time};
use skein_world::{
    sweep::{cases, gate},
    this_test,
};

#[test]
fn github_calls_costs_and_credentials_survive_cuts_renewal_and_replacement() {
    let at_gate = gate();
    let mut cuts = 0;
    cases(this_test!("github_calls_costs_and_credentials_survive_cuts_renewal_and_replacement"), 0..8, |seed| {
        let cut = github_case(seed, !at_gate);
        if at_gate {
            cuts += cut;
        }
        Ok::<(), String>(())
    })
    .unwrap_or_else(|case| panic!("{case}"));
    if at_gate {
        assert!(cuts > 0, "sweep includes charged requests cut before a whole head");
    }
}

#[test]
fn github_story_accepts_full_width_seed() {
    github_case(u64::MAX, true);
}

fn github_case(seed: u64, protect_setup: bool) -> u64 {
    // Minute polls exercise the log throughout the shipped one-hour token life.
    const LOG_POLL: Duration = Duration::from_secs(60);
    let mut world = World::github_with_log_poll(seed, 300, LOG_POLL);
    if protect_setup {
        world.protect_github_setup();
    }
    world.run_for(Duration::from_secs(20));
    let number = world.person_issue(b"before");
    let _ = world.read(Read::Items { single: false, since: Time::ZERO, page: 1, kind: None });
    world.person_edit(number, b"after");
    let _ = world.read(Read::Item { number, after: 0, since: Time::ZERO });
    fixtures::github_close(&mut world, number);
    world.restart();
    let mints = world.github_mints();
    // Tokens keep their shipped one-hour life. Only simulated time advances.
    world.run_for(Duration::from_secs(3600));
    assert!(world.github_mints() > mints, "renewal, seed {seed}");
    // An owner read is outstanding when the new key/access is handed in.
    world.replace_github_access(number);
    world.run_for(Duration::from_secs(20));
    let _ = world.read(Read::Items { single: false, since: Time::ZERO, page: 1, kind: None });
    world.finish();
    let (calls, answers) = world.calls_and_answers();
    assert!(calls > 0, "seed {seed}");
    assert_eq!(calls, answers, "seed {seed}");
    assert_eq!(world.owner_counts(), (0, 0), "seed {seed}");
    assert!(world.github_log_reads() > 0, "puller reads exercised, seed {seed}");
    let a = world.accounting();
    assert_eq!(a.reported, a.written, "seed {seed}: {a:?}");
    assert_eq!(a.served + a.cut_in_head, a.written, "seed {seed}: {a:?}");
    assert!(world.secrets_absent(), "seed {seed}");
    let (keys, jwts, tokens) = world.github_credentials();
    assert_eq!(keys, 2, "both handed keys, seed {seed}");
    assert!(jwts >= 2 && tokens >= 4, "renewed credentials, seed {seed}");
    assert_eq!(
        u64::try_from(tokens).expect("token count"),
        world.github_mints(),
        "every minted token retained, seed {seed}"
    );
    a.cut_in_head
}
