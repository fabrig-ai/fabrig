//! Routing tests.
pub(in crate::tests) mod account_signing;
pub(in crate::tests) mod deadlines;
pub(in crate::tests) mod engine_timers;
pub(in crate::tests) mod flight;
pub(in crate::tests) mod route_orders;
pub(in crate::tests) mod sign_ins;

mod erase;

use crate::tests::{routing::engine_timers::timer_core, *};

#[test]
fn an_unknown_unwatch_answers_with_its_ending() {
    let (mut core, env) = timer_core(false);
    let watcher = Token::new(900);
    let mut out = step(&mut core, &env, Event::Unwatch { watcher });
    let Some(Request::Held(held)) = out.pop() else { panic!("watch ending") };
    let Held::WatchEnded { watcher: got, why } = *held else { panic!("watch ending") };
    assert_eq!(got, watcher);
    assert_eq!(why, views::End::Unwatched);
    assert!(out.is_empty());
}
