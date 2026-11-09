#[test]
fn each_box_adoption_refusal_renders_one_line_naming_what_it_lacks() {
    use temper_service::system::BoxRefusal;
    for (why, expected) in [
        (BoxRefusal::Sockets { limit: 1 }, "the box's pipes need two free socket slots; the service socket limit is 1"),
        (BoxRefusal::Adopted, "the box's pipes were already adopted"),
        (BoxRefusal::NoEngine, "the box's pipes need an engine"),
        (BoxRefusal::ZeroLine, "the box's pipes need a nonzero application.limits.overrides.box.line_bytes"),
    ] {
        assert_eq!(
            super::prefixed("temper serve", &super::box_refusal(why)),
            format!("temper serve: --box: {expected}\n")
        );
    }
}

#[test]
fn a_boxed_serve_reads_none_or_a_whole_secret() {
    use std::io::Cursor;
    use temper_service::clients::LINK_SECRET_BYTES;

    let mut empty = Cursor::new([]);
    let (printed, local) = super::given_secret(&mut empty).expect("empty input");
    assert!(printed.is_none() && local.is_none());
    let bytes = [23; LINK_SECRET_BYTES];
    let mut whole = Cursor::new(bytes);
    let (printed, local) = super::given_secret(&mut whole).expect("whole secret");
    assert!(printed.expect("given secret").expose() == bytes, "the secret is taken whole");
    assert!(local.is_none());
    let mut both = Cursor::new([vec![23; LINK_SECRET_BYTES], vec![37; LINK_SECRET_BYTES]].concat());
    let (printed, local) = super::given_secret(&mut both).expect("both given secrets");
    assert!(printed.expect("printed secret").expose() == bytes, "the printed secret comes first");
    assert!(local.expect("local secret").expose() == [37; LINK_SECRET_BYTES], "the local secret comes second");
    for length in [LINK_SECRET_BYTES - 1, LINK_SECRET_BYTES + 1, 63, 65] {
        let mut input = Cursor::new(vec![23; length]);
        assert_eq!(
            super::given_secret(&mut input).expect_err("partial or oversized secret"),
            "--box: stdin: expected the first link's secret"
        );
    }
    let mut oversized = Cursor::new(vec![23; LINK_SECRET_BYTES * 3]);
    assert!(super::given_secret(&mut oversized).is_err(), "oversized input is refused");
    assert_eq!(oversized.position(), 65);
}

#[test]
fn a_refused_start_is_its_binarys_serve_line() {
    use std::os::unix::ffi::OsStrExt;
    let scratch = skein_scratch::Scratch::new("serve-refused-start");
    let arguments = [
        Box::from(b"--config".as_slice()),
        scratch.path().join("config").as_os_str().as_bytes().into(),
        Box::from(b"--data".as_slice()),
        scratch.path().join("data").as_os_str().as_bytes().into(),
        Box::from(b"--cache".as_slice()),
        scratch.path().join("cache").as_os_str().as_bytes().into(),
    ];
    let (directories, _) = temper_shell::directories::resolve(&arguments, &[], b"/").expect("fixture directories");
    std::fs::create_dir(scratch.path().join("config")).expect("configuration directory");
    std::fs::write(directories.document(), b"{}").expect("document");
    assert_eq!(
        super::run("temper-worker", &directories, false, |_, _, _| Err("refused".into())),
        Err("temper-worker serve: refused\n".into())
    );
}
