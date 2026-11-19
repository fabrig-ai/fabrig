//! Many names against a whole-file model (after-landing.md, section 4).
use crate::{Format, Limits, ManyTests, TestId, many_tests};
use alloc::collections::BTreeSet;
use skein_lib::{List, Rng};

#[test]
#[expect(
    clippy::disallowed_methods,
    reason = "the naive test model uses whole-file iterators independently of the streaming reader"
)]
fn many_lines_match_the_lowest_ids_and_unkept_listings_model() {
    let limits = Limits { failing_tests: 4, test_id_bytes: 3 };
    for seed in 0..512 {
        let mut rng = Rng::new(seed);
        let count = usize::try_from(rng.between(1, 12)).unwrap();
        let mut file = List::with_capacity(u32::try_from(count * 7).unwrap());
        for _ in 0..count {
            for _ in 0..rng.below(6) {
                file.push(b'a' + u8::try_from(rng.below(3)).unwrap()).unwrap();
            }
            if rng.chance(500) {
                file.push(b'\r').unwrap();
            }
            file.push(b'\n').unwrap();
        }
        let file = file.into_boxed();
        let cut = rng.chance(500);
        let end =
            if cut { usize::try_from(rng.below(u64::try_from(file.len()).unwrap() + 1)).unwrap() } else { file.len() };
        let file = file.get(..end).unwrap();
        let whole = if cut {
            file.get(..file.iter().rposition(|byte| *byte == b'\n').map_or(0, |at| at + 1)).unwrap()
        } else {
            file
        };
        let lines: Box<[&[u8]]> = whole
            .split(|byte| *byte == b'\n')
            .map(|line| line.strip_suffix(b"\r").unwrap_or(line))
            .filter(|line| !line.is_empty())
            .collect();
        let unique: BTreeSet<TestId> = lines.iter().filter(|id| id.len() <= 3).map(|id| Box::from(*id)).collect();
        let tests: Box<[TestId]> = unique.into_iter().take(4).collect();
        let more =
            u32::try_from(lines.iter().filter(|id| !tests.iter().any(|kept| kept.as_ref() == **id)).count()).unwrap();
        let expected = if more != 0 || cut { Some(ManyTests { tests, more, cut }) } else { None };
        assert_eq!(many_tests(Format::Lines, file, &limits, cut), expected, "seed {seed}, bytes {file:?}");
    }
}
