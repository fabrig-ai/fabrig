use super::ENGINE;

#[test]
fn arguments_document_is_the_engine_table() {
    let overview = ENGINE.help();
    let (_, commands) = overview.split_once("\ncommands:\n").expect("commands section");
    let commands = commands.split("\n\n").next().expect("command lines");
    let entries: Vec<_> = commands
        .lines()
        .filter_map(|line| line.strip_prefix("  "))
        .filter(|line| !line.starts_with(' '))
        .filter_map(|line| line.split_whitespace().next())
        .collect();
    assert_eq!(
        entries,
        [
            "serve",
            "versions",
            "upgrade",
            "upgrade",
            "upgrade",
            "checkpoint",
            "checkpoint",
            "checkpoint",
            "restore",
            "help"
        ]
    );
    let words = commands.split_whitespace().collect::<Vec<_>>().join(" ");
    assert!(words.contains("upgrade --workers"), "{words}");
    assert!(words.contains("Judges the workers alone over links and small copies; switches nothing."), "{words}");

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
