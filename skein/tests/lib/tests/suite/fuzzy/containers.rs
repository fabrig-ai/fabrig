//! Map joins and caller-driven searches against naive ones (runtime/lib.md, 5.2, 5.3).

use skein_lib::{Bisection, Rng};
use skein_lib_tests::containers::{linear_bound, naive_join, random_maps, random_sorted_slice};

#[test]
fn seeded_joins_agree_with_key_union_and_lookups() {
    let mut rng = Rng::new(0xA01A_5EED);
    for round in 0..20_000 {
        let (first, second) = random_maps(&mut rng, round);
        let expected = naive_join(&first, &second);
        assert!(first.join(&second).eq(expected.iter().map(|(key, joined)| (key, *joined))), "round {round}");
    }
}

#[test]
fn seeded_bisections_agree_with_a_linear_scan() {
    let mut rng = Rng::new(0xB15E_5EED);
    let mut buffer = [0_i16; 64];
    for round in 0..20_000 {
        let values = random_sorted_slice(&mut rng, &mut buffer);
        for key in -1..=43 {
            let mut search = Bisection::over(values.len());
            for _ in 0..Bisection::PROBES {
                let Some(probe) = search.probe() else { break };
                let value = values.get(probe).expect("probe within the slice");
                search.narrow(value.cmp(&key));
            }
            assert_eq!(search.probe(), None, "within PROBES, round {round}, key {key}");
            assert_eq!(search.end(), linear_bound(values, key), "round {round}, slice {values:?}, key {key}");
        }
    }
}
