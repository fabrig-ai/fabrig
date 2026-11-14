//! Naive map join and slice search, with seeded inputs (runtime/lib.md, 5.2, 5.3).

use skein_lib::{Joined, Map, Rng};

pub fn random_maps(rng: &mut Rng, round: u32) -> (Map<u8, u8>, Map<u8, u8>) {
    let mut first = Map::with_capacity(32);
    let mut second = Map::with_capacity(32);
    for key in 0_u8..32 {
        let in_first = rng.chance(500);
        let in_second = match round % 3 {
            0 => in_first, // All shared.
            1 => !in_first, // Disjoint and interleaved.
            _ => rng.chance(500), // A mixture of all three variants.
        };
        let value = u8::try_from(rng.below(8)).expect("small value");
        let other = if rng.chance(500) { value } else { value + 1 };
        if in_first {
            first.insert(key, value).expect("room for every key");
        }
        if in_second {
            second.insert(key, other).expect("room for every key");
        }
    }
    (first, second)
}

#[must_use]
pub fn naive_join<'a>(first: &'a Map<u8, u8>, second: &'a Map<u8, u8>) -> std::collections::BTreeMap<u8, Joined<'a, u8>> {
    let keys: std::collections::BTreeSet<_> = first.iter().chain(second).map(|(key, _)| *key).collect();
    keys.into_iter().map(|key| {
        let joined = match (first.get(&key), second.get(&key)) {
            (Some(a), Some(b)) => Joined::Both(a, b),
            (Some(a), None) => Joined::First(a),
            (None, Some(b)) => Joined::Second(b),
            (None, None) => unreachable!("key from either map"),
        };
        (key, joined)
    }).collect()
}

pub fn random_sorted_slice<'a>(rng: &mut Rng, buffer: &'a mut [i16; 64]) -> &'a [i16] {
    let len = usize::try_from(rng.between(0, 64)).expect("short slice");
    let values = &mut buffer[..len];
    for value in values.iter_mut() {
        *value = i16::try_from(rng.below(22)).expect("small key") * 2;
    }
    values.sort_unstable();
    values
}

/// The first item not below the key, comparing in slice order.
#[must_use]
pub fn linear_bound(values: &[i16], key: i16) -> usize {
    values.iter().position(|value| *value >= key).unwrap_or(values.len())
}
