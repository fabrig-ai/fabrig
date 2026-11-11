#[test]
fn the_handed_link_is_one_to_4096_bytes() {
    // client/tui/box.md bounds the handed local link at 4,096 bytes.
    const MOST: usize = 4096;
    for (row, bytes, accepted) in [
        ("empty", Vec::new(), false),
        (
            "a link",
            b"http://127.0.0.1:8367/jig/v1/sign-in/link?secret=AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA".to_vec(),
            true,
        ),
        ("at the bound", vec![b'x'; MOST], true),
        ("past the bound", vec![b'x'; MOST + 1], false),
    ] {
        let link = crate::tui_front::handed(&bytes);
        assert_eq!(link.is_some(), accepted, "{row}: handed link admission");
        if let Some(link) = link {
            assert!(link.expose() == bytes, "{row}: the handed link stays whole");
        }
    }
}
