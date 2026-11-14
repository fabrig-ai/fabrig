//! Seeded public paths (temper/docs/design/client/web/public.md, 1 and 4).
use skein_lib::Rng;
use temper_client_domain::address::{Place, Public, parse, print_public};

// Match the codec world's 2,000 draws, each independently replayable.
const SEEDS: u64 = 2_000;
// client/web/public.md, 1: each public identifier is 16 lowercase hexadecimal digits.
const ID_DIGITS: usize = 16;
const PROJECT_AT: usize = b"/public/projects/".len();

fn identifier(bytes: &[u8]) -> Option<u64> {
    if bytes.len() != ID_DIGITS || !bytes.iter().all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(byte)) {
        return None;
    }
    u64::from_str_radix(std::str::from_utf8(bytes).ok()?, 16).ok()
}

// Read the five grammar productions independently of the address codec.
fn oracle(path: &[u8]) -> Option<Public> {
    let segments: Vec<_> = path.split(|byte| *byte == b'/').collect();
    match segments.as_slice() {
        [b"", b"public"] => Some(Public::Projects),
        [b"", b"public", b"projects", project] => Some(Public::Board { project: identifier(project)? }),
        [b"", b"public", b"projects", project, b"tasks", task] => {
            Some(Public::Task { project: identifier(project)?, task: identifier(task)? })
        }
        [b"", b"public", b"projects", project, b"landings"] => Some(Public::Landings { project: identifier(project)? }),
        [b"", b"public", b"projects", project, b"landings", task] => {
            Some(Public::Landing { project: identifier(project)?, task: identifier(task)? })
        }
        _ => None,
    }
}

fn check(path: &[u8], seed: u64) {
    eprintln!("seed {seed}, input {path:?}: parse and print public path");
    let Place::Public(page) = parse(path) else { panic!("path must read as public") };
    let printed = print_public(&page);
    let expected = oracle(path);
    assert_eq!(page, expected.unwrap_or(Public::Projects), "seed {seed}, input {path:?}");
    if expected.is_some() {
        assert_eq!(printed.as_ref(), path, "seed {seed}, input {path:?}: valid path prints verbatim");
    } else {
        assert_eq!(printed.as_ref(), b"/public", "seed {seed}, input {path:?}: malformed path prints projects");
    }
}

fn mutated_identifier(path: &[u8], start: usize, rng: &mut Rng, seed: u64) {
    let digit = usize::try_from(rng.below(u64::try_from(ID_DIGITS).expect("identifier length fits")))
        .expect("digit position fits");
    let at = start.checked_add(digit).expect("digit lies within path");
    for replacement in [b'A', b'G', 0xff] {
        let mut bytes = path.to_vec();
        *bytes.get_mut(at).expect("identifier digit") = replacement;
        check(&bytes, seed);
    }
    let mut short = path.to_vec();
    short.remove(at);
    check(&short, seed);
    let mut long = path.to_vec();
    long.insert(at, b'0');
    check(&long, seed);
    let mut percent = path.to_vec();
    percent.splice(at..=at, *b"%61");
    check(&percent, seed);
    let mut empty = path.to_vec();
    let end = start.checked_add(ID_DIGITS).expect("identifier end fits");
    empty.drain(start..end);
    check(&empty, seed);
}

fn sweep(seed: u64) {
    let mut rng = Rng::new(seed);
    let project = rng.next_u64();
    let task = rng.next_u64();
    let base = format!("/public/projects/{project:016x}");
    for (page, path) in [
        (Public::Projects, "/public".to_owned()),
        (Public::Board { project }, base.clone()),
        (Public::Task { project, task }, format!("{base}/tasks/{task:016x}")),
        (Public::Landings { project }, format!("{base}/landings")),
        (Public::Landing { project, task }, format!("{base}/landings/{task:016x}")),
    ] {
        assert_eq!(oracle(path.as_bytes()), Some(page), "seed {seed}, input {path:?}: generated grammar");
        check(path.as_bytes(), seed);
        for suffix in [b"/".as_slice(), b"?page=1", b"/extra", b"//", b"/.", b"/..", b"/%61", b"/\xff"] {
            let mut bytes = path.as_bytes().to_vec();
            bytes.extend_from_slice(suffix);
            check(&bytes, seed);
        }
        let task_at = match page {
            Public::Projects => continue,
            Public::Board { .. } | Public::Landings { .. } => None,
            Public::Task { .. } | Public::Landing { .. } => {
                Some(path.len().checked_sub(ID_DIGITS).expect("task suffix"))
            }
        };
        for start in [Some(PROJECT_AT), task_at].into_iter().flatten() {
            mutated_identifier(path.as_bytes(), start, &mut rng, seed);
        }
        for segment in [b"/".as_slice(), b"./", b"../", b"projects/"] {
            let mut bytes = path.as_bytes().to_vec();
            bytes.splice(PROJECT_AT..PROJECT_AT, segment.iter().copied());
            check(&bytes, seed);
        }
        // Arbitrary suffix bytes keep the public prefix while mixing separators,
        // queries, percent escapes and bytes outside ASCII without a valid template.
        let mut arbitrary = b"/public/".to_vec();
        for _ in 0..rng.between(0, skein_lib::len_u64(path.len())) {
            arbitrary.push(u8::try_from(rng.below(256)).expect("one byte"));
        }
        check(&arbitrary, seed);
    }
}

#[test]
fn public_paths_never_panic_and_print_back_what_they_read() {
    for seed in 1..=SEEDS {
        eprintln!("seed {seed}: public path sweep");
        sweep(seed);
    }
}
