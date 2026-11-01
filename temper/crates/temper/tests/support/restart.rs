//! Executable restart boundaries on minimal deployments.
//! temper/docs/design/domain/testing.md, 2.6 and 6.1.
use super::{
    Ended, Scratch, boxed,
    ctl::{Call, Step},
    forge_land::{self, cache_forgejo_binaries, deployment_flags, forgejo_setup_steps},
};
use boxed::BOX_DEADLINE;
use skein_io::kernel::Exit;

fn drained(exit: Option<Exit>, left: Option<&[u32]>, stderr: &[u8]) {
    assert_eq!(exit, Some(Exit::Code(0)), "{}", String::from_utf8_lossy(stderr));
    assert_eq!(left, Some([].as_slice()), "no process outlived a box: {}", String::from_utf8_lossy(stderr));
}

fn answered(calls: &[Ended], count: usize) {
    assert_eq!(calls.len(), count, "every ordinary ctl call completed");
    for call in calls {
        assert_eq!(
            call.exit,
            Some(Exit::Code(0)),
            "stdout: {}; stderr: {}",
            String::from_utf8_lossy(&call.stdout),
            String::from_utf8_lossy(&call.stderr)
        );
    }
}

fn reads() -> Vec<Step> {
    vec![
        Step::Run(Call::new(&["settings", "show", "--json"])),
        Step::Run(Call::new(&["account", "list", "--json"])),
        Step::Run(Call::new(&["forge", "list", "--json"])),
    ]
}

/// Both owned services use the pinned cache while the first deployment restarts.
pub fn shared_cache_restart(case: &str) {
    let scratch = Scratch::new(case);
    let paths = ["config", "data", "cache"].map(|name| scratch.path().join(name));
    forge_land::initialize(&paths);
    super::own_listener_files(&paths[0]);
    let _cached = cache_forgejo_binaries(&paths[2]);
    let flags = boxed::echo_steps(deployment_flags(&paths));
    let first = boxed::run_ctl(
        scratch.path(),
        flags.clone(),
        scratch.path(),
        forgejo_setup_steps("first"),
        BOX_DEADLINE,
        |_| (),
    );
    drained(first.exit, first.left.as_deref(), &first.stderr);
    answered(&first.calls, 7);
    assert!(first.checked.is_some(), "initial owned deployment completed setup");

    let other = scratch.path().join("other");
    std::fs::create_dir(&other).expect("second deployment");
    let mut other_paths = ["config", "data", "cache"].map(|name| other.join(name));
    forge_land::initialize(&other_paths);
    super::own_listener_files(&other_paths[0]);
    other_paths[2].clone_from(&paths[2]);
    let other = boxed::run_ctl(
        &other,
        boxed::echo_steps(deployment_flags(&other_paths)),
        &other,
        forgejo_setup_steps("first"),
        BOX_DEADLINE,
        |_| {
            let mut steps = forgejo_setup_steps("again");
            steps.extend(reads());
            boxed::run_ctl(scratch.path(), flags, scratch.path(), steps, BOX_DEADLINE, |_| ())
        },
    );
    drained(other.exit, other.left.as_deref(), &other.stderr);
    answered(&other.calls, 7);
    let restarted = other.checked.expect("the second owned service stayed live throughout the first's restart");
    drained(restarted.exit, restarted.left.as_deref(), &restarted.stderr);
    answered(&restarted.calls, 7);
    assert!(restarted.checked.is_some(), "all restart reads completed");
    scratch.remove();
}
