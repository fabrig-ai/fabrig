mod link_file;

use crate::commands::{Row, Table, Who};

const ROWS: [Row; 2] = [
    Row {
        words: &[],
        who: Who::Clients,
        forms: &["[ADDRESS]"],
        line: "Connect.",
        arguments: &[],
        settings: &[],
        notes: "",
    },
    Row {
        words: &["forge", "run"],
        who: Who::Deployment,
        forms: &["FORGE", "--all"],
        line: "Run.",
        arguments: &[],
        settings: &[],
        notes: "",
    },
];

#[test]
fn command_renderings_name_the_binary() {
    let mut failures = Vec::new();
    for (binary, words, expected) in [
        ("temper", vec![], "temper: no command; temper --help lists them\n"),
        ("temper-engine", vec![], "temper-engine: no command; temper-engine --help lists them\n"),
        (
            "temper-engine",
            vec!["forge", "unknown", "ignored"],
            "temper-engine: no command forge unknown; temper-engine --help lists them\n",
        ),
        ("other", vec!["unknown", "ignored"], "other: no command unknown; other --help lists them\n"),
    ] {
        let table = Table { binary, rows: &ROWS };
        let words: Vec<Box<[u8]>> = words.iter().map(|word| Box::from(word.as_bytes())).collect();
        if table.unknown(&words) != expected {
            failures.push(format!("{binary}: unknown {words:?}"));
        }
    }
    for (binary, row, expected) in [
        ("temper", &ROWS[0], "usage: temper [ADDRESS]\n"),
        ("temper-engine", &ROWS[0], "usage: temper-engine [ADDRESS]\n"),
        ("temper-engine", &ROWS[1], "usage: temper-engine forge run FORGE\nusage: temper-engine forge run --all\n"),
        ("other", &ROWS[1], "usage: other forge run FORGE\nusage: other forge run --all\n"),
    ] {
        if (Table { binary, rows: &ROWS }).forms(row) != expected {
            failures.push(format!("{binary}: forms {:?}", row.words));
        }
    }
    assert!(failures.is_empty(), "binary name rows: {failures:?}");
}
