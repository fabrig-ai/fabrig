use crate::restart::{Action, Restart};
use skein_lib::{Duration, Wall};

// Three reloads in a minute, then failure (ui/web/README.md, 4).
const PANIC_SECONDS: [u64; 4] = [1, 2, 3, 4];

fn wall_at(seconds: u64) -> Wall {
    Wall::from_nanos(Duration::from_secs(seconds).as_nanos())
}

fn restart() -> Restart {
    Restart::new(3, Duration::from_secs(60), "skein-ui-restarts", "The application failed.")
}

#[test]
fn a_fourth_panic_in_a_minute_says_the_application_failed() {
    let restart = restart();
    let mut dates = Vec::new();
    let actions = PANIC_SECONDS.map(|seconds| restart.panicked(wall_at(seconds), &mut dates));
    assert_eq!(actions, [Action::Reload, Action::Reload, Action::Reload, Action::Failed]);
}

#[test]
fn a_later_start_is_refused_until_the_restart_window_expires() {
    const DURING_WINDOW: u64 = 5;
    const AFTER_WINDOW: u64 = 64; // One minute since the fourth panic.
    let restart = restart();
    let mut dates = Vec::new();
    for seconds in PANIC_SECONDS {
        restart.panicked(wall_at(seconds), &mut dates);
    }
    assert!(restart.failed(wall_at(DURING_WINDOW), &dates));
    assert!(!restart.failed(wall_at(AFTER_WINDOW), &dates));
}

#[test]
fn a_panic_past_the_minute_reloads_again() {
    let restart = restart();
    let mut dates = vec![Wall::EPOCH; 4];
    assert_eq!(restart.panicked(Wall::from_nanos(60_000_000_001), &mut dates), Action::Reload);
}
