use crate::{What, names, overrides};

#[test]
fn every_row_is_a_name_the_parser_takes() {
    let stock = crate::stock();
    for row in &names::NAMES {
        let mut limits = stock;
        let value = (row.value)(&stock);
        let other = value.checked_add(1).expect("stock limits leave room for a replacement");
        assert_eq!(overrides::apply(&mut limits, row.name.as_bytes(), other), Ok(()), "{}", row.name);
        assert_eq!((row.value)(&limits), other, "{}", row.name);
        assert_eq!(overrides::apply(&mut limits, row.name.as_bytes(), value), Ok(()), "{}", row.name);
        assert_eq!((row.value)(&limits), value, "{}", row.name);
        assert_eq!(limits, stock, "{}", row.name);
    }
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
    let stock = crate::stock();
    for name in [
        b"".as_slice(),
        b"absent",
        b"tasks.absent",
        b"tasks.retries",
        b"accounts.token.absent",
        b"clients_http.absent",
        b"io.input",
        b"io.sends",
    ] {
        let mut limits = stock;
        assert!(names::find(name).is_none(), "{name:?}");
        assert_eq!(overrides::apply(&mut limits, name, 1), Err(What::Override), "{name:?}");
        assert_eq!(limits, stock, "{name:?}");
    }
}
