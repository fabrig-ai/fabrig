//! Component-owned key generators.
use super::number;
use crate::keys::Own;
use skein_lib::Rng;

pub fn own(rng: &mut Rng) -> Own {
    match rng.below(8) {
        7 => Own::Floor,
        4 => Own::History { task: number(rng) },
        5 => Own::TurnLog { task: number(rng) },
        6 => Own::LogAttempt { task: number(rng), attempt: number(rng) },
        0 => Own::Format,
        3 => Own::Wires,
        1 => Own::Transcript { task: number(rng) },
        _ => Own::Attempt { task: number(rng), attempt: number(rng) },
    }
}
