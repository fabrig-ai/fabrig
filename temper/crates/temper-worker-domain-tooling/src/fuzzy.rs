use crate::{CONFIG_BYTES, Times, times};
use skein_lib::{Decimal, Rng};

const SEEDS: u64 = 1024;

#[test]
fn arbitrary_bytes_never_report_lines_outside_the_file() {
    const MAX_BYTES: usize = 2 * CONFIG_BYTES;
    let mut storage = [0; MAX_BYTES];
    for seed in 0..SEEDS {
        let mut rng = Rng::new(seed);
        let len = usize::try_from(rng.between(0, u64::try_from(MAX_BYTES).expect("byte bound"))).expect("drawn length");
        let file = storage.get_mut(..len).expect("length within storage");
        for byte in &mut *file {
            *byte = u8::try_from(rng.between(0, u64::from(u8::MAX))).expect("drawn byte");
        }
        if let Err(fault) = times(file) {
            let lines = file.split_inclusive(|&byte| byte == b'\n').count();
            assert!(usize::try_from(fault.line).expect("line fits") <= lines, "seed {seed}: {fault:?}, {lines} lines");
        }
    }
}

#[test]
#[expect(clippy::disallowed_types, reason = "the test builds varied line fixtures in a byte vector")]
fn drawn_time_files_read_back_the_seconds_written() {
    for seed in 0..SEEDS {
        let mut rng = Rng::new(seed);
        let mut file = Vec::new();
        let mut expected = Times { deliver: None, finish: None, merge: None, close: None };
        let mut tables: [(&[u8], &mut Option<u32>); 4] = [
            (b"deliver", &mut expected.deliver),
            (b"finish", &mut expected.finish),
            (b"merge", &mut expected.merge),
            (b"close", &mut expected.close),
        ];
        for last in (1..tables.len()).rev() {
            let other =
                usize::try_from(rng.between(0, u64::try_from(last).expect("table index"))).expect("drawn index");
            if other < last {
                let (before, tail) = tables.split_at_mut_checked(last).expect("last table is in the array");
                core::mem::swap(
                    before.get_mut(other).expect("drawn index is before last"),
                    tail.first_mut().expect("last table is in the tail"),
                );
            }
        }
        for (name, seconds) in tables {
            if !rng.chance(500) {
                continue;
            }
            let value = rng.between(1, u64::from(u32::MAX));
            *seconds = Some(u32::try_from(value).expect("drawn seconds"));
            let spaces: &[&[u8]] = &[b"", b" ", b"\t", b" \t "];
            let comments: &[&[u8]] = &[b"", b"#", b"# a check", b"# d\xc3\xa9lais"];
            let newlines: &[&[u8]] = &[b"\n", b"\r\n"];
            let digits = Decimal::of(value);
            for part in [
                draw(&mut rng, spaces),
                draw(&mut rng, comments),
                draw(&mut rng, newlines),
                draw(&mut rng, spaces),
                b"[",
                draw(&mut rng, spaces),
                name,
                draw(&mut rng, spaces),
                b"]",
                draw(&mut rng, spaces),
                draw(&mut rng, comments),
                draw(&mut rng, newlines),
                draw(&mut rng, spaces),
                b"timeout_seconds",
                draw(&mut rng, spaces),
                b"=",
                draw(&mut rng, spaces),
                digits.as_bytes(),
                draw(&mut rng, spaces),
                draw(&mut rng, comments),
                draw(&mut rng, newlines),
            ] {
                file.extend_from_slice(part);
            }
        }
        assert_eq!(times(&file), Ok(expected), "seed {seed}: {file:?}");
    }
}

fn draw<'a>(rng: &mut Rng, choices: &[&'a [u8]]) -> &'a [u8] {
    let at = rng.below(u64::try_from(choices.len()).expect("length"));
    choices.get(usize::try_from(at).expect("drawn index")).expect("choice")
}
