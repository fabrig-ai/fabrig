use crate::address::{Address, Place, parse, print};

#[test]
fn project_settings_prints_and_parses_back() {
    for (project, path) in [(1, b"/projects/1/settings".as_slice()), (u32::MAX, b"/projects/4294967295/settings")] {
        let address = Address::ProjectSettings { project };
        assert_eq!(print(&address).as_ref(), path);
        for suffix in [b"".as_slice(), b"/", b"?ignored=1", b"/?ignored=1"] {
            let mut input = skein_lib::Writer::new(path.len().checked_add(suffix.len()).expect("path fits"));
            input.put(path).expect("path fits");
            input.put(suffix).expect("suffix fits");
            assert_eq!(parse(&input.finish()), Place::Address(address.clone()));
        }
    }
}

#[test]
fn a_project_settings_path_without_a_number_is_missing() {
    for path in [b"/projects/x/settings".as_slice(), b"/projects//settings", b"/projects/4294967296/settings"] {
        assert_eq!(parse(path), Place::Missing, "{path:?}");
    }
}
