//! Provider faults on the direct backend (smith/docs/design/faults.md, 3 and 5).
use skein_fake_llm_domain::api::{Line, Piece, Strike};
use skein_lib::Duration;
use skein_llm_kit::faults::{EXPIRED_SIGN_IN, HANG_AFTER_HEAD, IDLE_MID_STREAM, RATE_WINDOW, SPENT_CREDITS};
use skein_world::{
    streams::{Stream, Streams},
    sweep::{Exclusion, Sweep},
};
use smith_protocol_llm_world::fixture;

// A small finite report has enough calls to strike before or during its work.
const ROUNDS: u32 = 10_000;
const LONG_WINDOW: &str = "a window longer than the session's shortest first backoff";

#[test]
fn provider_faults_fall_on_the_direct_backend() {
    let mut sweep = Sweep::new("direct provider faults");
    for kind in [RATE_WINDOW, SPENT_CREDITS, EXPIRED_SIGN_IN, HANG_AFTER_HEAD, IDLE_MID_STREAM] {
        sweep.configure(kind);
    }
    // Not configured until provider failures fixes a_cut_stream_is_retried_not_failed:
    // CUT_BEFORE_OUTPUT, CUT_MID_TEXT, CUT_MID_CALL. UNSERVED_MODEL awaits
    // an_unserved_model_moves_the_task in temper's system world.
    sweep
        .exclude(Exclusion::new("a_rate_window_is_waited_out", "provider failures", RATE_WINDOW, &[LONG_WINDOW]))
        .expect("named exclusion");
    // One seed for each configured kind; key/bearer renewal policy also has focused coverage.
    for seed in [0, 3, 7, 10, 15] {
        eprintln!("direct provider faults seed {seed}");
        let streams = Streams::new(seed);
        let mut faults = streams.rng(Stream::Faults, "llm");
        let mut workload = streams.rng(Stream::Workload, "runs");
        let kind = faults.below(5);
        let count = 3 + workload.below(3);
        let target = usize::try_from(faults.below(count - 1)).expect("small work");
        let length = Duration::from_millis(if faults.below(2) == 0 { 50 } else { 10_000 });
        let mut turns = Vec::new();
        for index in 0..count {
            let line = if usize::try_from(index).expect("small work") == target && kind >= 3 {
                let mut pieces = Vec::new();
                if kind == 4 {
                    pieces.push(Piece { text: b"working".as_slice().into(), hold: None });
                }
                pieces.push(Piece { text: b"held".as_slice().into(), hold: Some(b"provider hold".as_slice().into()) });
                Line::Pieces { pieces: pieces.into() }
            } else {
                Line::Call {
                    name: b"read".as_slice().into(),
                    arguments: format!(r#"{{"path":"src/lib.rs","window":{}}}"#, 32 + workload.below(32))
                        .into_bytes()
                        .into(),
                }
            };
            turns.push(fixture::calls(Box::new([line]), 3));
        }
        turns.push(fixture::calls(
            Box::new([Line::Call {
                name: b"finish".as_slice().into(),
                arguments: br#"{"form":"report","text":"done","fields":{"source":"peer"}}"#.as_slice().into(),
            }]),
            7,
        ));
        let dialect = if workload.below(2) == 0 { fixture::Dialect::OpenAi } else { fixture::Dialect::Anthropic };
        eprintln!("seed {seed}: fault {kind}, dialect {dialect:?}, target {target}");
        let mut world = fixture::provider_fault_world(
            streams.seed_of(Stream::Scheduling, "world"),
            dialect.configuration(),
            turns.into(),
        );
        world.record_fault_runs("run 1", "a");
        let mut struck = false;
        let mut settled = false;
        let mut renewed = false;
        for _ in 0..ROUNDS {
            if !struck && world.protocol_calls().len() >= target {
                let strike = match kind {
                    0 => Some(Strike::Window { tokens: 1, length }),
                    1 => Some(Strike::Spent),
                    2 => Some(Strike::SignedOut),
                    _ => None,
                };
                if let Some(strike) = strike {
                    if kind == 0
                        && length > world.shortest_backoff()
                        && sweep.leaves_out(RATE_WINDOW, &[LONG_WINDOW]).is_some()
                    {
                        world.leave_out_fault("a_rate_window_is_waited_out", RATE_WINDOW);
                    } else {
                        world.strike(0, strike).expect("one account");
                    }
                }
                struck = true;
            }
            if kind == 2 && dialect == fixture::Dialect::OpenAi && !renewed && !world.unauthorized().is_empty() {
                world.renew_sign_in(2);
                renewed = true;
            }
            if world.drive(1) {
                settled = true;
                break;
            }
        }
        assert!(settled, "seed {seed}: finite world settles");
        world.judge_faults().unwrap_or_else(|finding| panic!("seed {seed}: {finding}; answer {:?}", world.answer()));
        eprintln!("seed {seed} fell {:?}", world.fault_ledger().fell());
        sweep.ran(world.fault_ledger());
    }
    sweep.check_fell().expect("every configured provider fault fell");
}
