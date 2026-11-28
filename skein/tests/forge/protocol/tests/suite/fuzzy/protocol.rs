//! Forgejo cuts and accounting (skein/docs/design/testing/sweeps.md, 4.2 to 4.5).

use skein_fake_forge_forgejo::Faults;
use skein_forge::api::{Error, Op, Read};
use skein_forge_kit::joint::{self, Joint};
use skein_forge_protocol_world::{World, fixtures};
use skein_lib::{Duration, Time};
use skein_world::{
    sweep::{Test, cases, gate},
    this_test,
};

fn mixed_cases(test: Test, check: fn(&World, u64)) {
    let protect_start = !gate();
    cases(test, (0..16).chain([26]), |seed| {
        let mut world = fixtures::mixed_calls(seed, protect_start);
        world.finish();
        check(&world, seed);
        Ok::<(), String>(())
    })
    .unwrap_or_else(|case| panic!("{case}"));
}

#[test]
fn each_call_has_one_answer_under_stream_cuts() {
    mixed_cases(this_test!("each_call_has_one_answer_under_stream_cuts"), check_answers);
}

fn check_answers(world: &World, seed: u64) {
    let (calls, answers) = world.calls_and_answers();
    assert!(calls > 0, "seed {seed}");
    assert_eq!(calls, answers, "seed {seed}");
}

#[test]
fn reported_costs_account_for_served_and_cut_requests() {
    mixed_cases(this_test!("reported_costs_account_for_served_and_cut_requests"), check_accounting);
}

fn check_accounting(world: &World, seed: u64) {
    let a = world.accounting();
    assert_eq!(a.reported, a.written, "seed {seed}: {a:?}");
    assert_eq!(
        a.served,
        a.written.checked_sub(a.cut_in_head).expect("cuts are written requests"),
        "seed {seed}: {a:?}"
    );
}

#[test]
fn cut_request_cost_includes_the_write_before_timeout() {
    let mut joint = Joint::cutting(26, 300, Faults { reset: 0, drop: 0, server_error: 0 });
    joint.run_for(Duration::from_secs(20));
    joint.call(1, joint::REPO, Op::Read(Read::Items { single: false, since: Time::ZERO, page: 1, kind: None }));
    joint.run_for(Duration::from_secs(11));
    let answer = joint.answered(1).expect("calibration terminal");
    let requests = joint.request_counts();
    let startup = joint.start_counts();
    assert_eq!(answer.cost, 2);
    assert_eq!(requests.written, 2);
    assert_eq!(joint.tally().total.checked_sub(startup.served), Some(1));
    assert_eq!(requests.cut_in_head, 1);
    assert_eq!(startup.served, 2);
    assert_eq!(answer.result, Err(Error::Timeout));
}

#[test]
fn no_credential_reaches_a_record_request_line_or_fact() {
    mixed_cases(this_test!("no_credential_reaches_a_record_request_line_or_fact"), check_credentials);
}

fn check_credentials(world: &World, seed: u64) {
    assert!(world.secrets_absent(), "seed {seed}");
}

#[test]
fn mixed_calls_accept_full_width_seed_without_cutting_startup() {
    // Seed 20 draws a reset 39 bytes into its first startup connection.
    for seed in [u64::MAX, 20] {
        let mut world = fixtures::mixed_calls(seed, true);
        world.finish();
        check_answers(&world, seed);
        check_accounting(&world, seed);
        check_credentials(&world, seed);
        assert_eq!(world.startup_head_cuts(), 0, "seed {seed}");
    }
}
