//! Seeded public decoders (jig/docs/design/client/domain/contract/README.md, section 10;
//! jig/docs/design/protocol/client/public-reads.md).
use jig_client_protocol_wire::{PublicIndexBounds, public_limits, public_v1::*};
use skein_lib::{Reader, Rng, Writer};
use std::fs;

const FIRST_SEED: u64 = 0x7075_b11c;
const SEEDS: u64 = 4;
const ARBITRARY: u32 = 2_000;
const MUTATIONS: u32 = 8;

macro_rules! check {
    ($record:ident, $limits:expr, $bytes:expr) => {
        if let Ok(record) = $record::decode($limits, &mut Reader::new($bytes)) {
            let mut writer = Writer::new(skein_lib::length::usize_of(record.measure()));
            record.encode(&mut writer).expect("measured room");
            assert_eq!(writer.finish().as_ref(), $bytes, stringify!($record));
        }
    };
}

fn decoders(seed: u64, bytes: &[u8], limits: &Limits) {
    eprintln!("seed {seed:#x}, limits {limits:?}, input {bytes:?}");
    check!(PublicProjects, limits, bytes);
    check!(PublicProject, limits, bytes);
    check!(PublicBoard, limits, bytes);
    check!(PublicGoal, limits, bytes);
    check!(PublicCard, limits, bytes);
    check!(PublicLine, limits, bytes);
    check!(PublicCounts, limits, bytes);
    check!(PublicApplication, limits, bytes);
    check!(PublicExecutor, limits, bytes);
    check!(PublicPhase, limits, bytes);
    check!(PublicHold, limits, bytes);
}

fn index(rng: &mut Rng, length: usize) -> usize {
    usize::try_from(rng.below(u64::try_from(length).expect("bounded length"))).expect("bounded index")
}

fn byte(rng: &mut Rng) -> u8 {
    u8::try_from(rng.below(256)).expect("byte range")
}

fn arbitrary(seed: u64, rng: &mut Rng, limits: &Limits) {
    let mut bytes = [0; 512];
    for _ in 0..ARBITRARY {
        let length = index(rng, bytes.len().checked_add(1).expect("small buffer"));
        let input = bytes.get_mut(..length).expect("bounded length");
        for value in &mut *input {
            *value = byte(rng);
        }
        decoders(seed, input, limits);
    }
}

fn mutated(seed: u64, rng: &mut Rng, limits: &Limits, golden: &[u8]) {
    decoders(seed, golden, limits);
    for mutation in 0..MUTATIONS {
        let mut bytes = golden.to_vec();
        let position = index(rng, bytes.len().checked_add(1).expect("golden length"));
        match mutation {
            0 | 1 => {
                if let Some(value) = bytes.get_mut(position) {
                    *value ^= 1_u8.checked_shl(u32::from(byte(rng) % 8)).expect("bit range");
                }
            }
            2 => bytes.insert(position, byte(rng)),
            3 => {
                if position < bytes.len() {
                    bytes.remove(position);
                }
            }
            4 => bytes.truncate(position),
            5 => bytes.extend_from_slice(&[0, 255]),
            6 => {
                // Length prefixes and tags see their largest byte value too.
                for value in bytes.iter_mut().skip(position).take(4) {
                    *value = 255;
                }
            }
            7 => {
                if let Some(value) = bytes.get_mut(position) {
                    *value = byte(rng);
                }
            }
            _ => unreachable!("bounded mutations"),
        }
        decoders(seed, &bytes, limits);
    }
}

#[test]
fn public_records_decode_without_panic_and_re_encode_what_they_decode() {
    let root = skein_scratch::paths::tree().join("crates/jig-client-protocol-wire/golden/public");
    let mut paths: Vec<_> = fs::read_dir(root)
        .expect("public golden directory")
        .map(|entry| entry.expect("public golden entry").path())
        .filter(|path| path.extension().is_some_and(|extension| extension == "bin"))
        .collect();
    paths.sort();
    assert!(!paths.is_empty(), "public golden corpus, seed {FIRST_SEED:#x}");
    let goldens: Vec<_> = paths.iter().map(|path| fs::read(path).expect("public golden bytes")).collect();
    for seed in FIRST_SEED..FIRST_SEED.checked_add(SEEDS).expect("seed range") {
        let mut rng = Rng::new(seed);
        for bounds in [
            PublicIndexBounds { goals: 0, ended: 0, lines: 0, depth: 0, projects: 0 },
            PublicIndexBounds { goals: 1, ended: 2, lines: 3, depth: 4, projects: 5 },
            PublicIndexBounds { goals: 20, ended: 20, lines: 50, depth: 16, projects: 64 },
            PublicIndexBounds { goals: 256, ended: 256, lines: 256, depth: 64, projects: 4096 },
        ] {
            let limits = public_limits(&bounds).expect("public bounds within ceilings");
            arbitrary(seed, &mut rng, &limits);
            for golden in &goldens {
                mutated(seed, &mut rng, &limits, golden);
            }
        }
    }
}
