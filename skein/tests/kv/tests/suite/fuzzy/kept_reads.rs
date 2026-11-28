//! Seeded agreement between kept and uncached reads (skein/docs/design/data/kv.md, 6.5).
//! Cases take their seeds as in skein/docs/design/testing/sweeps.md, 4.

use super::super::logs::{OWNER, limits};
use super::logs::draw;
use skein_kv::{Direction, Event, Limits, LogRule, Op, Request, Store};
use skein_lib::{Env, Queue, Time, Wall};
use skein_world::{sweep::cases, this_test};
use std::convert::Infallible;

const SEEDS: u64 = 6;
const STEPS: u64 = 48;
const LOGS: usize = 3;
const RECORD_BYTES: u64 = 48; // Both short stored and longer repeated records.
const PAGE_BYTES: u64 = 24; // Short enough to meet a record that must be cut.
const PAGE_ROWS: u64 = 3;

fn step(store: &mut Store, env: &Env<Limits>, request: Request) -> Event {
    let mut above = Queue::uncapped();
    skein_kv::down(store, env, request, &mut above, &mut Queue::uncapped());
    above.pop().expect("memory answers inline")
}

#[test]
fn generated_payload_reads_match_with_keeping_on_and_off() {
    let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits: limits() };
    let test = this_test!("generated_payload_reads_match_with_keeping_on_and_off");
    cases::<_, Infallible, _>(test, 1..=SEEDS, |initial| {
        let mut seed = initial;
        let mut on = Store::memory(&env.limits);
        let mut off = Store::memory(&env.limits);
        for twin in [&mut on, &mut off] {
            twin.logs(LogRule { prefix: Box::from(&b"e/"[..]) });
        }
        off.keep_reads(false);
        // One boundary for each frame end, plus the first frame start at zero.
        let mut boundaries: [Vec<u64>; LOGS] = std::array::from_fn(|_| vec![0]);
        for turn in 0..STEPS {
            let log = draw(&mut seed) % u64::try_from(LOGS).expect("log count");
            let places = boundaries.get_mut(usize::try_from(log).expect("log index")).expect("chosen log");
            let choice = draw(&mut seed) % 8;
            if choice < 3 {
                let op = if choice == 0 {
                    places.truncate(1);
                    Op::Drop { log }
                } else {
                    let length = usize::try_from(1 + draw(&mut seed) % RECORD_BYTES).expect("length");
                    let byte = u8::try_from(draw(&mut seed) % 256).expect("byte");
                    let frame = on.frame(&vec![byte; length]).expect("record fits");
                    let end = places.last().expect("zero boundary") + frame.len();
                    places.push(end);
                    Op::Append { log, frame }
                };
                for (twin, op) in [(&mut on, op.clone()), (&mut off, op)] {
                    let commit = step(twin, &env, Request::Commit { owner: OWNER, ops: Box::new([op]) });
                    assert!(matches!(commit, Event::Committed { .. }), "seed {initial}, turn {turn}: {commit:?}");
                }
            } else {
                let dir = if draw(&mut seed).is_multiple_of(2) { Direction::Forward } else { Direction::Backward };
                let choices = u64::try_from(places.len()).expect("boundary count");
                let index = usize::try_from(draw(&mut seed) % choices).expect("boundary index");
                let at = places[index];
                let max = u32::try_from(1 + draw(&mut seed) % PAGE_BYTES).expect("page bytes");
                let rows = u32::try_from(1 + draw(&mut seed) % PAGE_ROWS).expect("page rows");
                let request = || Request::ReadLog { owner: OWNER, log, at, direction: dir, max, rows };
                let expected = step(&mut off, &env, request());
                for _ in 0..2 {
                    assert_eq!(
                        step(&mut on, &env, request()),
                        expected,
                        "seed {initial}, turn {turn}: {:?}",
                        request()
                    );
                }
            }
        }
        Ok(())
    })
    .unwrap_or_else(|case| panic!("{case}"));
}
