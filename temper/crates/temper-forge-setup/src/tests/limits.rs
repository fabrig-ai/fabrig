// Fixture parsing without a deployment keeps written paths.
use crate::{check, limits, problem};
#[test]
fn the_small_profile_passes_the_setup_limit_checks() {
    assert_eq!(skein_forge_setup::limits::check(&limits::small()), Ok(()));
    assert_eq!(check::limits(&[]).unwrap().0, limits::small());
}
#[test]
fn a_backoff_past_its_maximum_names_both_limits() {
    for (name, value, expected) in [
        (b"setup.backoff_ms".as_slice(), 6000, b"application.limits.overrides.setup.backoff_ms: must agree with application.limits.overrides.setup.backoff_max_ms\n".as_slice()),
        (b"setup.backoff_max_ms", 120_000, b"application.limits.overrides.setup.backoff_max_ms: must agree with application.limits.overrides.setup.deadline_ms\n"),
    ] {
        let problems = check::limits(&[(name.into(), value)]).unwrap_err();
        assert_eq!(&*problem::lines(&problems), expected);
    }
}
#[test]
fn an_unknown_zero_or_overflowing_limit_is_refused() {
    for (name, value, expected) in [
        (b"box.restarts".as_slice(), 1, limits::Refused::Unknown),
        (b"setup.deadline_ms", u64::MAX, limits::Refused::Range),
    ] {
        assert_eq!(limits::apply(&mut limits::small(), name, value), Err(expected));
    }
}
#[test]
fn host_counts_are_owned_by_the_small_profile_and_overrides_refuse_zero() {
    let mut host = limits::host();
    assert_eq!((host.runs, host.states), (8, 8));
    assert_eq!(host.plan(), limits::small());
    for name in [b"setup.runs".as_slice(), b"setup.states"] {
        for value in [0, u64::MAX] {
            assert_eq!(limits::apply_host(&mut host, name, value), Err(limits::Refused::Range));
        }
        assert_eq!(limits::apply_host(&mut host, name, 2), Ok(()));
    }
    assert_eq!((host.runs, host.states), (2, 2));
    for value in [0, 3_600_001, u64::MAX] {
        assert_eq!(limits::apply_host(&mut host, b"setup.state_life_ms", value), Err(limits::Refused::Range));
    }
}

fn row_words(name: &str, line: &str) {
    assert!(!line.is_empty() && line.len() <= 120, "{name}");
    assert!(line.ends_with('.') && !line.contains(['\n', '\r']), "{name}");
    assert_eq!(line.matches('.').count(), 1, "{name}");
}

#[test]
fn every_row_is_a_name_the_parser_takes() {
    let small = limits::small();
    for row in &limits::NAMES {
        row_words(row.name, row.line);
        let mut changed = small.clone();
        let value = (row.value)(&small);
        assert_eq!(limits::apply(&mut changed, row.name.as_bytes(), value + 1), Ok(()), "{}", row.name);
        assert_eq!((row.value)(&changed), value + 1, "{}", row.name);
        assert_eq!(limits::apply(&mut changed, row.name.as_bytes(), value), Ok(()));
        assert_eq!(changed, small);
    }
    let small = limits::host();
    for row in &limits::HOST_NAMES {
        row_words(row.name, row.line);
        let mut changed = small;
        let value = (row.value)(&small);
        let other = if row.name == "setup.state_life_ms" { value - 1 } else { value + 1 };
        assert_eq!(limits::apply_host(&mut changed, row.name.as_bytes(), other), Ok(()), "{}", row.name);
        assert_eq!((row.value)(&changed), other, "{}", row.name);
        assert_eq!(limits::apply_host(&mut changed, row.name.as_bytes(), value), Ok(()));
        assert_eq!(changed, small);
    }
    for row in &limits::FETCH_NAMES {
        row_words(row.name, row.line);
        for value in [limits::FETCH_BYTES, limits::FETCH_BYTES + 1] {
            let (plan, fetch) = check::limits(&[(row.name.as_bytes().into(), value)]).expect("fetch override");
            assert_eq!(plan, limits::small());
            assert_eq!((row.value)(&fetch), value);
        }
    }
}

#[test]
fn an_unknown_name_is_refused() {
    for name in [b"".as_slice(), b"setup.absent", b"box.restarts", b"setup.deadline_ms.extra"] {
        let small = limits::small();
        let mut plan = small.clone();
        assert_eq!(limits::apply(&mut plan, name, 1), Err(limits::Refused::Unknown));
        assert_eq!(plan, small);
        let small = limits::host();
        let mut host = small;
        assert_eq!(limits::apply_host(&mut host, name, 1), Err(limits::Refused::Unknown));
        assert_eq!(host, small);
        check::limits(&[(name.into(), 1)]).expect_err("unknown name");
    }
}
