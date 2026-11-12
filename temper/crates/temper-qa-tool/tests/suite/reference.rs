//! Rendered references are the table byte for byte (testing/tools.md, 4.4).
use temper_qa_tool::TABLE;
#[test]
fn the_reference_documents_match_the_table() {
    let write = temper_test_env::rewrite();
    let directory = skein_scratch::paths::tree().join("docs/design/cli");
    for (name, rendered) in
        [("temper-qa-args.md", TABLE.arguments_document()), ("temper-qa-env.md", TABLE.environment_document(&[&TABLE]))]
    {
        let path = directory.join(name);
        if write {
            std::fs::write(&path, &rendered).expect("write table reference");
        } else {
            assert_eq!(
                std::fs::read_to_string(&path).expect("table reference"),
                rendered,
                "{name}; regenerate with SKEIN_CODEGEN_WRITE=1 and this test"
            );
        }
    }
}
