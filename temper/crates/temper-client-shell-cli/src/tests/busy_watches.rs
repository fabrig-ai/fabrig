//! client/ctl/results.md, 3: a deadline names watches refused busy.

use super::{DEADLINE_TICK, fixtures, limits};
use skein_io::kernel::Exit;
use skein_lib::Duration;

fn deadline_names_watches(mut f: fixtures::Engine) {
    f.advance_past(Duration::from_nanos(limits::DEADLINE.as_nanos() - DEADLINE_TICK.as_nanos()));
    assert_eq!(f.exit(), None);
    assert_eq!(f.stderr(), []);
    f.advance_past(DEADLINE_TICK);
    assert_eq!(f.exit(), Some(Exit::Code(1)));
    assert_eq!(f.stderr(), b"temper ctl: all the engine's watches are in use, by open pages and waits; close one, or wait and call again\n");
    assert!(f.stderr().len() <= 240);
    assert_eq!(f.stdout(), []);
}

#[test]
fn a_call_at_its_deadline_names_the_engines_watches() {
    for words in [&["me"][..], &["task", "show", "41"][..]] {
        deadline_names_watches(fixtures::Engine::busy_watches(words));
    }
}

#[test]
fn a_watch_room_that_frees_before_the_deadline_answers() {
    let mut f = fixtures::Engine::busy_watches(&["me", "--json"]);
    f.busy_watches = false;
    f.sign_in_then_deliver_me();
    f.advance_past(limits::native().root.domain.client.domain.backoff.first);
    assert_eq!(f.exit(), Some(Exit::Code(0)));
    assert!(f.stdout().starts_with(b"{\""));
    assert_eq!(f.stderr(), []);
    assert!(!f.host.root.client().link().watches_full);
}
