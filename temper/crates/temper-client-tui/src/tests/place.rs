//! The engine's place (temper/docs/design/client/README.md, 6.12).
#[test]
fn a_box_names_only_its_non_default_port() {
    for (address, boxed, expected) in [
        (b"127.0.0.1:8367".as_slice(), true, b"box".as_slice()),
        (b"127.0.0.1:40691", true, b"box:40691"),
        (b"127.0.0.1:40691", false, b"127.0.0.1:40691"),
        (b"[::1]:8367", true, b"box"),
        (b"[::1]:40691", true, b"box:40691"),
        (b"[::1]:8367", false, b"[::1]:8367"),
    ] {
        assert_eq!(crate::place(address, boxed).as_ref(), expected, "{address:?}");
    }
}
