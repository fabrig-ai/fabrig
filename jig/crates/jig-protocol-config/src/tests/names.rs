use crate::{Profile, What, names, overrides, profile};

#[test]
fn every_row_is_a_name_the_parser_takes() {
    let small = profile(Profile::Small);
    for row in &names::NAMES {
        let mut limits = small;
        let value = (row.value)(&small);
        let other = value.checked_add(1).expect("small limits leave room for a replacement");
        assert_eq!(overrides::apply(&mut limits, row.name.as_bytes(), other), Ok(()), "{}", row.name);
        assert_eq!((row.value)(&limits), other, "{}", row.name);
        assert_eq!(overrides::apply(&mut limits, row.name.as_bytes(), value), Ok(()), "{}", row.name);
        assert_eq!((row.value)(&limits), value, "{}", row.name);
        assert_eq!(limits, small, "{}", row.name);
    }
}

#[test]
fn every_name_the_parser_took_is_a_row() {
    let small = profile(Profile::Small);
    let mut seen = skein_lib::Set::with_capacity(u32::try_from(names::NAMES.len()).unwrap());
    for row in super::overrides::SMALL {
        let name = row.name.as_bytes();
        let found = names::find(name).expect("every fixture name has a row");
        assert_eq!(found.name, row.name);
        assert_eq!(found.unit, row.unit, "{}", row.name);
        assert_eq!((found.value)(&small), (row.value)(&small), "{}", row.name);
        assert_eq!(seen.insert(row.name), Ok(true), "duplicate name: {}", row.name);
    }
    assert_eq!(usize::try_from(seen.len()).unwrap(), names::NAMES.len());
}

#[test]
fn each_line_is_within_its_bound() {
    // C3 bounds one sentence in a reference row to 120 bytes.
    const LINE_BYTES: usize = 120;
    for row in &names::NAMES {
        assert!(!row.line.is_empty() && row.line.len() <= LINE_BYTES, "{}: {}", row.name, row.line);
        assert!(row.line.ends_with('.'), "{}: {}", row.name, row.line);
        assert_eq!(skein_lib::bytes::count(row.line.as_bytes(), b".", u32::MAX), 1, "{}: {}", row.name, row.line);
        assert!(!row.line.contains(['\n', '\r']), "{}", row.name);
    }
}

#[test]
fn an_unknown_name_is_refused() {
    let small = profile(Profile::Small);
    for name in
        [b"".as_slice(), b"absent", b"tasks.absent", b"tasks.retries", b"accounts.token.absent", b"clients_http.absent"]
    {
        let mut limits = small;
        assert!(names::find(name).is_none(), "{name:?}");
        assert_eq!(overrides::apply(&mut limits, name, 1), Err(What::Override), "{name:?}");
        assert_eq!(limits, small, "{name:?}");
    }
}
