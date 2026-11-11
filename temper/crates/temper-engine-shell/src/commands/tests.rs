use super::ENGINE;

#[test]
#[expect(
    clippy::disallowed_methods,
    reason = "cli C4: SKEIN_CODEGEN_WRITE selects regeneration of the arguments document; the test table will pass it"
)]
fn arguments_document_is_the_engine_table() {
    let rendered = ENGINE.arguments_document();
    let path = skein_scratch::paths::tree().join("docs/design/cli/temper-engine-args.md");
    if std::env::var_os("SKEIN_CODEGEN_WRITE").is_some_and(|value| value == "1") {
        std::fs::write(&path, &rendered).expect("write engine arguments document");
        return;
    }
    assert_eq!(
        std::fs::read_to_string(&path).expect("engine arguments document"),
        rendered,
        "regenerate with SKEIN_CODEGEN_WRITE=1 and this test"
    );
}
