//! An independent counted-time model against random stderr lines and read splits.

use crate::checks::held::Check;
use skein_lib::{Duration, Rng, Time, Token};

#[test]
fn marker_sequences_and_read_splits_keep_the_held_bound_and_absolute_cap() {
    for seed in 0..96 {
        let mut rng = Rng::new(seed);
        let mut check = Check::new(500, 600);
        let mut counted = 0_u64;
        let mut waiting = false;
        let mut now = 0_u64;
        let mut line = Vec::new();
        let mut deadline = 60_u64;
        'sequence: for _ in 0..100 {
            let bytes = match rng.below(8) {
                0 | 1 => b"smith-check: waiting\n".to_vec(),
                2 | 3 => b"smith-check: admitted\n".to_vec(),
                4 => b"smith-check: waiting now\n".to_vec(),
                5 => b"smith-check: waiting".to_vec(),
                6 => b"\n".to_vec(),
                7 => vec![b'x'; 1024],
                _ => unreachable!("eight draws"),
            };
            let stderr = !rng.chance(200);
            let pipe = Token::new(if stderr { 42 } else { 41 });
            let mut from = 0;
            while from < bytes.len() {
                let to = (from + usize::try_from(rng.between(1, 24)).expect("small chunk")).min(bytes.len());
                let later = now + rng.below(5);
                if later >= deadline {
                    if !waiting {
                        counted += deadline - now;
                    }
                    assert!(counted <= 60, "seed {seed}");
                    break 'sequence;
                }
                if !waiting {
                    counted += later - now;
                }
                now = later;
                check.at(now);
                check.bytes(pipe, &bytes[from..to]);
                if stderr {
                    for byte in &bytes[from..to] {
                        line.push(*byte);
                        if *byte == b'\n' {
                            if line == b"smith-check: waiting\n" {
                                waiting = true;
                            } else if line == b"smith-check: admitted\n" {
                                waiting = false;
                            }
                            line.clear();
                        }
                    }
                }
                deadline = if waiting { 500 } else { (now + 60 - counted).min(500) };
                assert!(counted <= 60, "seed {seed}");
                assert!(deadline <= 500, "seed {seed}");
                assert_eq!(
                    check.deadline(),
                    Time::ZERO.saturating_add(Duration::from_secs(deadline)),
                    "seed {seed}, counted {counted}, waiting {waiting}"
                );
                from = to;
            }
        }
        check.settle_at_deadline(deadline);
    }
}
