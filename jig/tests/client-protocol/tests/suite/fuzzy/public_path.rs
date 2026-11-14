//! Canonical public targets over seeded bytes (jig/docs/design/protocol/client/default-http.md, 5 and jig/docs/design/protocol/client/public-reads.md
//! ).
use jig_client_protocol_http::{PublicPath, public_target};
use jig_protocol_engine_clients::PublicRead;
use jig_protocol_engine_clients_http::public_path;
use skein_lib::{List, Rng};
use std::fmt::Write as _;

const FIRST_SEED: u64 = 0x7075_7a74;
const SEEDS: u64 = 4;
const ARBITRARY: u32 = 2_000;
const CANONICAL: u32 = 512;
const EDGES: [u16; 8] = [0, 1, 9, 10, 255, 256, 65_534, u16::MAX];

fn index(rng: &mut Rng, length: usize) -> usize {
    usize::try_from(rng.below(u64::try_from(length).expect("bounded length"))).expect("bounded index")
}

fn byte(rng: &mut Rng) -> u8 {
    u8::try_from(rng.below(256)).expect("byte range")
}

fn path_of(read: &PublicRead) -> PublicPath {
    let mut ids = List::with_capacity(2);
    for id in &read.ids {
        ids.push(*id).expect("at most two ids");
    }
    PublicPath { owner: read.owner, version: read.version, kind: read.kind, ids }
}

fn inspect(seed: u64, bytes: &[u8], canonical: bool) -> Option<PublicRead> {
    eprintln!("seed {seed:#x}, input {bytes:?}");
    let read = public_path(bytes);
    if canonical && let Some(read) = &read {
        assert_eq!(public_target(&path_of(read)).as_ref(), bytes, "accepted target prints byte for byte");
    }
    read
}

fn number(rng: &mut Rng, sample: u32, offset: u32) -> u16 {
    if sample < 8 {
        let position = usize::try_from(sample.checked_add(offset).expect("small sample") % 8).expect("edge index");
        *EDGES.get(position).expect("edge range")
    } else {
        u16::try_from(rng.below(65_536)).expect("whole u16 range")
    }
}

fn drawn(rng: &mut Rng, sample: u32) -> PublicPath {
    let mut ids = List::with_capacity(2);
    for id in 0..sample % 3 {
        let value = match sample % 4 {
            0 => 0,
            1 => u64::MAX,
            2 => u64::from(id),
            3 => rng.next_u64(),
            _ => unreachable!("bounded choice"),
        };
        ids.push(value).expect("at most two ids");
    }
    PublicPath { owner: number(rng, sample, 0), version: number(rng, sample, 3), kind: number(rng, sample, 5), ids }
}

fn target(path: &PublicPath) -> Vec<u8> {
    let mut text = format!("/v1/public/{}/{}/{}", path.owner, path.version, path.kind);
    for id in &path.ids {
        write!(text, "/{id:016x}").expect("writing to a string");
    }
    text.into_bytes()
}

fn printed(seed: u64, path: &PublicPath, bytes: &[u8]) {
    eprintln!("printer seed {seed:#x}, input {bytes:?}");
    assert_eq!(public_target(path).as_ref(), bytes, "printer seed {seed:#x}, input {bytes:?}");
    let read =
        inspect(seed, bytes, true).unwrap_or_else(|| panic!("canonical refused, seed {seed:#x}, input {bytes:?}"));
    assert_eq!(read.owner, path.owner, "owner seed {seed:#x}, input {bytes:?}");
    assert_eq!(read.version, path.version, "version seed {seed:#x}, input {bytes:?}");
    assert_eq!(read.kind, path.kind, "kind seed {seed:#x}, input {bytes:?}");
    let ids: Vec<_> = path.ids.iter().copied().collect();
    assert_eq!(read.ids.as_ref(), ids, "ids seed {seed:#x}, input {bytes:?}");
}

fn mutations(seed: u64, rng: &mut Rng, bytes: &[u8], canonical: bool) {
    let position = index(rng, bytes.len());
    let mut changed = bytes.to_vec();
    *changed.get_mut(position).expect("canonical byte") ^= 1;
    let _ = inspect(seed, &changed, canonical);
    changed = bytes.to_vec();
    changed.insert(position, byte(rng));
    let _ = inspect(seed, &changed, canonical);
    changed = bytes.to_vec();
    changed.remove(position);
    let _ = inspect(seed, &changed, canonical);

    let text = std::str::from_utf8(bytes).expect("canonical ascii");
    let segments: Vec<_> = text.split('/').collect();
    for (position, segment) in segments.iter().enumerate().skip(1) {
        for replacement in [
            format!("{segment}/{segment}"),
            String::new(),
            ".".to_owned(),
            "..".to_owned(),
            format!(
                "%{:02x}{}",
                segment.as_bytes().first().expect("nonempty segment"),
                segment.get(1..).expect("ascii")
            ),
            segment.to_ascii_uppercase(),
            format!("0{segment}"),
            "65536".to_owned(),
        ] {
            let mut changed = segments.clone();
            *changed.get_mut(position).expect("segment position") = &replacement;
            let _ = inspect(seed, changed.join("/").as_bytes(), canonical);
        }
    }
    for suffix in ["?q=1", "#fragment", "/", "/0000000000000000/0000000000000000/0000000000000000"] {
        let _ = inspect(seed, format!("{text}{suffix}").as_bytes(), canonical);
    }
}

fn sweep(canonical: bool) {
    for seed in FIRST_SEED..FIRST_SEED.checked_add(SEEDS).expect("seed range") {
        let mut rng = Rng::new(seed);
        let mut bytes = [0; 256];
        for _ in 0..ARBITRARY {
            let length = index(&mut rng, bytes.len().checked_add(1).expect("small buffer"));
            let input = bytes.get_mut(..length).expect("bounded length");
            for value in &mut *input {
                *value = byte(&mut rng);
            }
            let _ = inspect(seed, input, canonical);
        }
        for sample in 0..CANONICAL {
            let path = drawn(&mut rng, sample);
            let bytes = target(&path);
            if canonical {
                printed(seed, &path, &bytes);
            } else {
                let _ = inspect(seed, &bytes, false);
            }
            mutations(seed, &mut rng, &bytes, canonical);
        }
    }
}

#[test]
fn the_public_path_parser_never_panics() {
    sweep(false);
}

#[test]
fn the_parser_accepts_exactly_the_canonical_paths() {
    sweep(true);
}
