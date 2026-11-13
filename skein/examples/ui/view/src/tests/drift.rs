//! The family and golden bytes cannot drift (data/codec/README.md, 6).
#[test]
#[expect(clippy::disallowed_methods, reason = "skein's round: reads SKEIN_CODEGEN_WRITE, the codegen rewrite switch")]
fn records_match_their_generated_code() {
    let root = std::path::Path::new(".");
    let source = std::fs::read_to_string(root.join("schema/notes-v1.schema")).expect("schema readable");
    let schema = skein_codegen::parse(&source).expect("schema valid");
    let output = skein_codegen::generate(&schema);
    let write = std::env::var_os("SKEIN_CODEGEN_WRITE").as_deref() == Some(std::ffi::OsStr::new("1"));
    let codec = root.join("../view-codec");
    check(&codec.join("src/generated/v1.rs"), output.records.as_bytes(), write);
    check(
        &codec.join("tests/generated/v1.rs"),
        output.tests.text("skein_ui_example_view_codec::records").as_bytes(),
        write,
    );
    check(&codec.join("tests/generated/main.rs"), skein_codegen::tests_main(&["v1"]).as_bytes(), write);
    assert_eq!(output.goldens.len(), 12);
    for golden in output.goldens {
        check(&root.join("golden/v1").join(golden.name), &golden.bytes, write);
    }
}

fn check(path: &std::path::Path, bytes: &[u8], write: bool) {
    if write {
        std::fs::create_dir_all(path.parent().expect("output has a parent")).expect("output directory");
        std::fs::write(path, bytes).expect("write generated output");
    } else {
        assert!(
            std::fs::read(path).ok().as_deref() == Some(bytes),
            "{} drifted; regenerate notes family with SKEIN_CODEGEN_WRITE=1",
            path.display()
        );
    }
}
