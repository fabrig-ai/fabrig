//! Canonical public targets over seeded bytes
//! (jig/docs/design/protocol/client/default-http.md, 5 and jig/docs/design/protocol/client/public-reads.md).
use jig_client_protocol_http::{PublicPath, public_target};
use jig_protocol_engine_clients::PublicRead;
use jig_protocol_engine_clients_http::public_path;
use skein_checks::{MutationBatch, count, mutation};
use skein_lib::{List, Rng};
use std::fmt::{Arguments, Write as _};

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
    PublicPath { owner: read.owner, layout: read.layout.unwrap_or_default(), kind: read.kind, ids }
}

fn inspect(seed: u64, bytes: &[u8], canonical: bool, case: Arguments<'_>) -> Option<PublicRead> {
    eprintln!("seed {seed:#x}, {case}, input length {}", bytes.len());
    let read = public_path(bytes);
    if canonical && let Some(read) = &read {
        let path = path_of(read);
        if read.layout.is_some() {
            assert_eq!(public_target(&path).as_ref(), bytes, "accepted target prints byte for byte");
        } else {
            let version = std::str::from_utf8(bytes)
                .expect("accepted decimal path is ASCII")
                .split('/')
                .nth(4)
                .expect("decimal version segment")
                .parse::<u16>()
                .expect("decimal version is a u16");
            assert_eq!(target(&path, Some(version)), bytes, "accepted target uses canonical decimal version");
        }
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
    let mut layout = [0; 32];
    for value in &mut layout {
        *value = byte(rng);
    }
    PublicPath { owner: number(rng, sample, 0), layout, kind: number(rng, sample, 5), ids }
}

fn target(path: &PublicPath, version: Option<u16>) -> Vec<u8> {
    let mut text = format!("/v1/public/{}/", path.owner);
    if let Some(version) = version {
        write!(text, "{version}").expect("writing to a string");
    } else {
        for byte in path.layout {
            write!(text, "{byte:02x}").expect("writing to a string");
        }
    }
    write!(text, "/{}", path.kind).expect("writing to a string");
    for id in &path.ids {
        write!(text, "/{id:016x}").expect("writing to a string");
    }
    text.into_bytes()
}

fn printed(seed: u64, path: &PublicPath, bytes: &[u8], sample: u32) {
    eprintln!("printer seed {seed:#x}, sample {sample}, original, input length {}", bytes.len());
    assert_eq!(public_target(path).as_ref(), bytes, "printer seed {seed:#x}, input {bytes:?}");
    let read = inspect(seed, bytes, true, format_args!("sample {sample}, original"))
        .unwrap_or_else(|| panic!("canonical refused, seed {seed:#x}, input {bytes:?}"));
    assert_eq!(read.owner, path.owner, "owner seed {seed:#x}, input {bytes:?}");
    assert_eq!(read.layout, Some(path.layout), "layout seed {seed:#x}, input {bytes:?}");
    assert_eq!(read.kind, path.kind, "kind seed {seed:#x}, input {bytes:?}");
    let ids: Vec<_> = path.ids.iter().copied().collect();
    assert_eq!(read.ids.as_ref(), ids, "ids seed {seed:#x}, input {bytes:?}");
}

fn mutations(batch: &mut MutationBatch, seed: u64, rng: &mut Rng, bytes: &[u8], canonical: bool, sample: u32) {
    let position = index(rng, bytes.len());
    let mut changed = bytes.to_vec();
    *changed.get_mut(position).expect("canonical byte") ^= 1;
    let _ = batch.judge(mutation(&changed), |input| {
        inspect(seed, input, canonical, format_args!("sample {sample}, flipped byte {position}"))
    });
    changed = bytes.to_vec();
    changed.insert(position, byte(rng));
    let _ = batch.judge(mutation(&changed), |input| {
        inspect(seed, input, canonical, format_args!("sample {sample}, inserted byte {position}"))
    });
    changed = bytes.to_vec();
    changed.remove(position);
    let _ = batch.judge(mutation(&changed), |input| {
        inspect(seed, input, canonical, format_args!("sample {sample}, removed byte {position}"))
    });

    let text = std::str::from_utf8(bytes).expect("canonical ascii");
    let segments: Vec<_> = text.split('/').collect();
    for (position, segment) in segments.iter().enumerate().skip(1) {
        for (replacement_index, replacement) in [
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
        ]
        .into_iter()
        .enumerate()
        {
            let mut changed = segments.clone();
            *changed.get_mut(position).expect("segment position") = &replacement;
            let changed = changed.join("/");
            let variant = if changed == text { changed.as_bytes().into() } else { mutation(changed.as_bytes()) };
            let _ = batch.judge(variant, |input| {
                inspect(
                    seed,
                    input,
                    canonical,
                    format_args!("sample {sample}, segment {position} replacement {replacement_index}"),
                )
            });
        }
    }
    for (suffix_index, suffix) in
        ["?q=1", "#fragment", "/", "/0000000000000000/0000000000000000/0000000000000000"].into_iter().enumerate()
    {
        let changed = format!("{text}{suffix}");
        let _ = batch.judge(mutation(changed.as_bytes()), |input| {
            inspect(seed, input, canonical, format_args!("sample {sample}, suffix {suffix_index}"))
        });
    }
}

/// Check arbitrary and generated public targets for each seed, optionally asserting canonical forms.
pub fn sweep(first_seed: u64, seeds: u64, arbitrary: u32, canonical_samples: u32, canonical: bool) {
    let mut batch = MutationBatch::default();
    for seed in first_seed..first_seed.checked_add(seeds).expect("seed range") {
        let mut rng = Rng::new(seed);
        let mut bytes = [0; 256];
        for sample in 0..arbitrary {
            let length = index(&mut rng, bytes.len().checked_add(1).expect("small buffer"));
            let input = bytes.get_mut(..length).expect("bounded length");
            for value in &mut *input {
                *value = byte(&mut rng);
            }
            let _ = inspect(seed, input, canonical, format_args!("arbitrary {sample}"));
        }
        for sample in 0..canonical_samples {
            let path = drawn(&mut rng, sample);
            let bytes = target(&path, None);
            if canonical {
                printed(seed, &path, &bytes, sample);
            } else {
                let _ = inspect(seed, &bytes, false, format_args!("sample {sample}, original"));
            }
            mutations(&mut batch, seed, &mut rng, &bytes, canonical, sample);
            let decimal_target = target(&path, Some(number(&mut rng, sample, 3)));
            let read = inspect(seed, &decimal_target, canonical, format_args!("sample {sample}, decimal version"));
            if canonical {
                let read = read.expect("canonical decimal target is routed");
                let expected = PublicRead {
                    owner: path.owner,
                    layout: None,
                    kind: path.kind,
                    ids: Box::from(path.ids.as_slice()),
                };
                assert_eq!(read, expected, "decimal path names no layout, seed {seed:#x}, input {decimal_target:?}");
            }
        }
    }
    batch.finish();
    count("seed", seeds);
}
