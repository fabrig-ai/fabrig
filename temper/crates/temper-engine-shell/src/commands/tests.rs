use super::ENGINE;

#[test]
fn arguments_document_is_the_engine_table() {
    let rendered = ENGINE.arguments_document();
    let path = skein_scratch::paths::tree().join("docs/design/cli/temper-engine-args.md");
    if temper_test_env::rewrite() {
        std::fs::write(&path, &rendered).expect("write engine arguments document");
        return;
    }
    assert_eq!(
        std::fs::read_to_string(&path).expect("engine arguments document"),
        rendered,
        "regenerate with SKEIN_CODEGEN_WRITE=1 and this test"
    );
}
