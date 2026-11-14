//! Check report bounds and the v8/v9 bridge (domain/forge/reds.md, 5.6).
use super::fixtures;
use crate::v1;
use skein_lib::{List, Reader, Writer};

fn keys(count: u32, bytes: u32) -> List<Box<[u8]>> {
    let mut keys = List::with_capacity(count);
    for _ in 0..count {
        keys.push(skein_lib::bytes::zeroed(usize::try_from(bytes).unwrap())).unwrap();
    }
    keys
}

fn largest_report(limits: &v1::Limits) -> v1::CheckResults {
    let mut flakes = List::with_capacity(limits.check_results_flakes);
    for _ in 0..limits.check_results_flakes {
        flakes.push(v1::Flake::new(limits, v1::FlakeParts {
            key: skein_lib::bytes::zeroed(usize::try_from(limits.flake_key).unwrap()),
            log: skein_lib::bytes::zeroed(usize::try_from(limits.flake_log).unwrap()),
        }).unwrap()).unwrap();
    }
    v1::CheckResults::new(limits, v1::CheckResultsParts {
        tag: u64::MAX,
        check_number: u64::MAX,
        tree: Box::new([1; 32]),
        base: Some(Box::new([2; 32])),
        verdict: v1::Verdict::Refused,
        passed: keys(limits.check_results_passed, limits.check_results_passed_item),
        failed: keys(limits.check_results_failed, limits.check_results_failed_item),
        flakes,
    }).unwrap()
}

#[test]
fn reports_at_each_sides_bounds_fill_the_largest_ask() {
    for limits in [fixtures::limits(), v1::CEILINGS] {
        let ask = v1::Ask::Results(largest_report(&limits));
        let bytes = crate::ask(&ask);
        assert_eq!(v1::Ask::decode(&limits, &mut Reader::new(&bytes)).unwrap(), ask);
        let largest = crate::largest(&limits).unwrap();
        assert_eq!(usize::try_from(largest.ask).unwrap(), bytes.len());
        assert_eq!(u64::from(largest.ask), v1::Ask::encoded_bound(&limits).unwrap());
        assert!(largest.ask <= jig_protocol_worker_wire::v14::CEILINGS.ask_bytes);
    }
}

#[test]
fn reports_decode_only_within_the_engines_key_bounds_and_flake_ceilings() {
    let limits = fixtures::limits();
    for (passed, path) in [(true, v1::Path::CheckResultsPassed), (false, v1::Path::CheckResultsFailed)] {
        for (count, length) in [(3, 12), (1, 13)] {
            let mut parts = largest_report(&limits).into_parts();
            parts.flakes = List::with_capacity(0);
            if passed {
                parts.passed = keys(count, length);
            } else {
                parts.failed = keys(count, length);
            }
            let ask = v1::Ask::Results(v1::CheckResults::new(&v1::CEILINGS, parts).unwrap());
            let bytes = crate::ask(&ask);
            let Err(error) = v1::Ask::decode(&limits, &mut Reader::new(&bytes)) else {
                panic!("over-bound report decoded");
            };
            assert_eq!(error.path, path);
        }
    }
    // Length prefixes past a schema ceiling must be refused before reading their payload.
    for (count, key, log, path) in [
        (17_u32, 0_u32, 0_u32, v1::Path::CheckResultsFlakes),
        (1, 513, 0, v1::Path::FlakeKey),
        (1, 0, 2049, v1::Path::FlakeLog),
    ] {
        let mut bytes = Writer::new(63);
        bytes.put(&[1]).unwrap(); // Ask::Results
        bytes.put(&1_u64.to_be_bytes()).unwrap();
        bytes.put(&2_u64.to_be_bytes()).unwrap();
        bytes.put(&20_u32.to_be_bytes()).unwrap();
        bytes.put(&[3; 20]).unwrap();
        bytes.put(&[0, 0]).unwrap(); // no base, passed
        bytes.put(&0_u32.to_be_bytes()).unwrap();
        bytes.put(&0_u32.to_be_bytes()).unwrap();
        bytes.put(&count.to_be_bytes()).unwrap();
        bytes.put(&key.to_be_bytes()).unwrap();
        bytes.put(&log.to_be_bytes()).unwrap();
        let bytes = bytes.finish();
        for limits in [limits, v1::CEILINGS] {
            let error = v1::Ask::decode(&limits, &mut Reader::new(&bytes)).unwrap_err();
            assert_eq!((error.path, error.reason), (path, skein_codec::Reason::Bound));
        }
    }
}

#[test]
fn reds_cross_v8_and_v9_but_results_and_kept_cannot_go_back() {
    let limits = fixtures::limits();
    let mut application = crate::application(&limits).unwrap();
    let bridge = application.bridges.get_mut(0).unwrap();
    for (root, bytes) in [
        (b"Ask".as_slice(), crate::ask(&v1::Ask::Reds)),
        (b"Answer".as_slice(), crate::answer(&v1::Answer::Reds(
            v1::Reds::new(&limits, v1::RedsParts { reds: List::with_capacity(0) }).unwrap(),
        ))),
    ] {
        assert_eq!(bridge.translate(8, 9, root, bytes.clone()).unwrap(), bytes);
        assert_eq!(bridge.translate(9, 8, root, bytes.clone()).unwrap(), bytes);
        assert_eq!(bridge.total(8, root), Some(false));
    }
    for (root, bytes) in [
        (b"Ask".as_slice(), crate::ask(&v1::Ask::Results(largest_report(&limits)))),
        (b"Answer".as_slice(), crate::answer(&v1::Answer::Kept)),
    ] {
        bridge.translate(9, 8, root, bytes).expect_err("no v8 form");
    }
}
