//! Accounts sweeps use one case per seed (skein/docs/design/testing/sweeps.md, 4.2).

use hearth_accounts_world::{
    fixtures::{FILES, LIMITS},
    generate,
};
use hearth_protocol_accounts::{File, Ids, MAX_ID, Refused, limits::Limits, read};
use skein_lib::Rng;
use skein_world::{
    sweep::{Case, cases},
    this_test,
};
use std::{collections::BTreeSet, convert::Infallible};

const SEEDS: u64 = 400;
const PINNED: &[u64] = &[];
const GENERATED_OFFSET: u64 = 0x0a77_0000;
const MUTATED_OFFSET: u64 = 0x0a78_0000;
const ARBITRARY_OFFSET: u64 = 0x0a79_0000;

fn case_rng(seed: u64, offset: u64) -> Rng {
    Rng::new(seed.wrapping_add(offset))
}

fn size(n: u32) -> usize {
    usize::try_from(n).expect("u32 fits usize")
}
fn lines_of(bytes: &[u8]) -> usize {
    bytes.split(|b| *b == b'\n').count() - usize::from(bytes.is_empty() || bytes.ends_with(b"\n"))
}
/// Whatever a read gives satisfies the format, and refuses only a line that exists.
fn check(file: File, bytes: &[u8], limits: &Limits, note: &str) {
    match read(file, bytes, limits) {
        Ok(lines) => {
            assert_eq!(lines.len(), lines_of(bytes), "{note}");
            assert!(bytes.len() <= size(limits.file), "{note}");
            let mut names = BTreeSet::new();
            for line in &lines {
                assert!(!line.name.is_empty() && line.name.len() <= size(limits.name), "{note}");
                assert!(!line.name.contains(&b':') && !line.name.contains(&b'\n'), "{note}");
                assert!(names.insert(line.name.clone()), "{note}: a name twice");
                match (file, line.ids) {
                    (File::Passwd, Ids::User { uid, gid }) => assert!(uid <= MAX_ID && gid <= MAX_ID, "{note}"),
                    (File::Group, Ids::Group { gid }) => assert!(gid <= MAX_ID, "{note}"),
                    (File::Shadow | File::Gshadow, Ids::Shadow) => {}
                    (file, ids) => panic!("{note}: {file:?} read to {ids:?}"),
                }
            }
        }
        Err(Refused::File) => assert!(bytes.len() > size(limits.file), "{note}"),
        Err(Refused::Lines) => assert!(lines_of(bytes) > size(limits.lines), "{note}"),
        Err(Refused::Line { number, .. }) => {
            assert!(number >= 1 && size(number) <= lines_of(bytes), "{note}: line {number}");
        }
    }
}
#[test]
fn generated_files_read_to_their_entries() -> Result<(), Case<Infallible>> {
    cases(this_test!("generated_files_read_to_their_entries"), PINNED.iter().copied().chain(0..SEEDS), |seed| {
        for file in FILES {
            let mut rng = case_rng(seed, GENERATED_OFFSET);
            let generated = generate::file(&mut rng, file, &LIMITS);
            let lines = read(file, &generated.bytes, &LIMITS).unwrap_or_else(|e| panic!("seed {seed} {file:?}: {e:?}"));
            assert_eq!(lines.to_vec(), generated.lines, "seed {seed} {file:?}");
        }
        Ok(())
    })
}
#[test]
fn mutated_files_never_panic_and_give_entries_or_a_refusal() -> Result<(), Case<Infallible>> {
    cases(
        this_test!("mutated_files_never_panic_and_give_entries_or_a_refusal"),
        PINNED.iter().copied().chain(0..SEEDS),
        |seed| {
            for file in FILES {
                let mut rng = case_rng(seed, MUTATED_OFFSET);
                let generated = generate::file(&mut rng, file, &LIMITS);
                let mutated = generate::mutate(&mut rng, &generated.bytes);
                check(file, &mutated, &LIMITS, &format!("seed {seed} {file:?}"));
                let small = generate::small_limits(&mut rng);
                check(file, &mutated, &small, &format!("seed {seed} {file:?} {small:?}"));
                check(file, &generated.bytes, &small, &format!("seed {seed} {file:?} whole {small:?}"));
            }
            Ok(())
        },
    )
}
#[test]
fn bytes_of_any_kind_never_panic() -> Result<(), Case<Infallible>> {
    cases(this_test!("bytes_of_any_kind_never_panic"), 0..SEEDS, |seed| {
        let mut rng = case_rng(seed, ARBITRARY_OFFSET);
        let length = rng.between(0, 300);
        let bytes: Vec<u8> =
            (0..length).map(|_| if rng.chance(300) { b':' } else { u8::try_from(rng.below(256)).unwrap() }).collect();
        for file in FILES {
            check(file, &bytes, &LIMITS, &format!("seed {seed} {file:?}"));
        }
        Ok(())
    })
}

#[test]
fn account_draws_wrap_full_width_seeds() {
    assert_eq!(case_rng(u64::MAX, GENERATED_OFFSET).state(), 0x0a76_ffff);
    assert_eq!(case_rng(u64::MAX, MUTATED_OFFSET).state(), 0x0a77_ffff);
    assert_eq!(case_rng(u64::MAX, ARBITRARY_OFFSET).state(), 0x0a78_ffff);
}
