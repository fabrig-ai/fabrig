//! Harness contracts run once in focused (skein/docs/foundation/testing-strategy.md, 8).
use crate::support;

#[test]
fn fixture_limits_are_applied_before_the_subprocess_restart() {
    let scratch = support::Scratch::startup("small-limits-settings");
    let mut configuration = support::lay_configuration(scratch.path(), &support::Forge::Unreached);
    configuration.jig_over("tasks.project_goals", 2);
    configuration.temper_over("checkout.remote_timeout_ms", 7_000);
    support::limit_settings::remember(scratch.path(), &configuration);
    support::limit_settings::install(scratch.path());
    configuration.jig_over("tasks.project_goals", 1);
    configuration.temper_over("checkout.remote_timeout_ms", 8_000);
    support::limit_settings::remember(scratch.path(), &configuration);
    support::limit_settings::install(scratch.path());
    let seen = support::ctl::run(support::ctl::Story::new(
        scratch.path(),
        vec![
            support::ctl::Step::Run(support::ctl::Call::sign_in()),
            support::ctl::Step::Run(support::ctl::Call::new(&["settings", "show", "limits", "--json"])),
            support::ctl::Step::Run(support::ctl::Call::new(&["settings", "show", "temper-limits", "--json"])),
        ],
    ));
    assert_eq!(
        support::ctl::json(&seen.calls[1])["value"]
            .as_array()
            .expect("jig limits")
            .iter()
            .find(|item| item["name"] == "tasks.project_goals")
            .expect("project goal capacity")["value"],
        1
    );
    assert_eq!(
        support::ctl::json(&seen.calls[2])["value"]
            .as_array()
            .expect("temper limits")
            .iter()
            .find(|item| item["name"] == "checkout.remote_timeout_ms")
            .expect("worker checkout limit")["value"],
        8_000
    );
    scratch.remove();
}

#[test]
fn ctl_a_failed_ctl_check_reaches_the_world_after_cleanup() {
    support::ctl::read_condition_tests::a_failed_ctl_check_reaches_the_world_after_cleanup();
}

#[test]
fn ctl_a_returned_visit_drains_without_another_process_or_a_poll() {
    support::ctl::read_condition_tests::a_returned_visit_drains_without_another_process_or_a_poll();
}
