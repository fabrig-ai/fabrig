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

// The spawning thread, not just its process, is the parent-death owner on Linux.
#[cfg(target_os = "linux")]
#[test]
#[expect(clippy::disallowed_methods, reason = "this test exercises death on the spawning thread's return")]
fn a_fixture_started_on_a_thread_dies_when_that_thread_returns() {
    use skein_io::kernel::Signal;
    use skein_world::{Deadline, Expired};
    use std::{
        os::unix::process::ExitStatusExt,
        process::{Command, Stdio},
        thread,
    };

    let child = thread::spawn(|| {
        let mut command = Command::new("/bin/sleep");
        command.arg("60").stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::null());
        support::fixture::spawn(&mut command, Signal::Kill).expect("fixture starts")
    })
    .join()
    .expect("spawning thread returned");
    let mut deadline = Deadline::arm("the fixture ends with its spawning thread", skein_lib::Duration::from_secs(3));
    deadline.watch_process(child.id());
    let mut owner = support::parent::Guard::new(child);
    let status = loop {
        if let Some(status) = owner.child.as_mut().expect("fixture child").try_wait().expect("fixture status") {
            break status;
        }
        if deadline.passed() {
            match deadline.expired() {
                Expired::Starved(starved) => {
                    drop(owner);
                    starved.pass();
                }
                Expired::Failed(figures) => {
                    drop(owner);
                    panic!("fixture must end with its spawning thread: {figures}");
                }
            }
        }
        thread::sleep(std::time::Duration::from_millis(5));
    };
    assert_eq!(status.signal(), Some(9), "fixture dies by SIGKILL");
}
