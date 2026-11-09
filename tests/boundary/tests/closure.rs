//! Each binary's normal dependency closure (temper/docs/design/protocol/README.md, 4.4).
use std::{
    path::Path,
    process::{Command, Stdio},
};

struct Row {
    package: &'static str,
    temper: &'static [&'static str],
    trees: &'static [&'static str],
    exceptions: &'static [(&'static str, &'static str)],
}

const ROWS: &[Row] = &[Row {
    package: "temper-agent",
    temper: &[
        "temper-agent",
        "temper-shell",
        "temper-docs",
        "temper-launcher",
        "temper-fetch",
        "temper-fetch-shell",
        "temper-version",
    ],
    trees: &["jig"],
    exceptions: &[],
}];

fn closure(root: &Path, package: &str) -> Vec<(String, Option<String>)> {
    let output = Command::new("cargo")
        .args(["tree", "--locked", "--offline", "-p", package, "-e", "normal", "--prefix", "none", "--format", "{p}"])
        .current_dir(root)
        .stdin(Stdio::null())
        .output()
        .expect("cargo tree starts");
    assert!(
        output.status.success(),
        "cargo tree {package}: {}\n{}",
        output.status,
        String::from_utf8_lossy(&output.stderr)
    );
    let text = String::from_utf8(output.stdout).expect("cargo tree is UTF-8");
    let mut packages: Vec<_> = text
        .lines()
        .map(|line| {
            let name = line.split_whitespace().next().expect("cargo tree package name").to_owned();
            let tree = line
                .split_once(" (")
                .and_then(|(_, tail)| tail.split_once(')'))
                .and_then(|(path, _)| Path::new(path).strip_prefix(root).ok())
                .and_then(|path| path.components().next())
                .map(|tree| tree.as_os_str().to_string_lossy().into_owned());
            (name, tree)
        })
        .collect();
    packages.sort();
    packages.dedup();
    packages
}

fn breaches(row: &Row, closure: &[(String, Option<String>)]) -> Vec<String> {
    let mut failures = Vec::new();
    if closure.is_empty() {
        failures.push(format!("{}: empty closure", row.package));
    }
    if !closure.iter().any(|(package, _)| package == row.package) {
        failures.push(format!("{}: closure lacks its own package", row.package));
    }
    for (package, tree) in closure {
        if row.exceptions.iter().any(|(exception, _)| package == exception) {
            continue;
        }
        if tree.as_deref().is_some_and(|tree| row.trees.contains(&tree))
            || tree.as_deref() == Some("temper") && !row.temper.contains(&package.as_str())
        {
            failures.push(format!("{} excludes {package}", row.package));
        }
    }
    failures
}

#[test]
fn each_binary_links_what_its_row_allows() {
    let root = skein_scratch::paths::workspace();
    for row in ROWS {
        let failures = breaches(row, &closure(&root, row.package));
        assert!(failures.is_empty(), "{}", failures.join("\n"));
    }
}

#[test]
fn the_check_names_what_a_row_excludes() {
    let row = &ROWS[0];
    let mut allowed: Vec<_> = row.temper.iter().map(|package| ((*package).into(), Some("temper".into()))).collect();
    allowed.extend([
        ("smith-agent-shell".into(), Some("smith".into())),
        ("skein-shell".into(), Some("skein".into())),
        ("serde".into(), None),
    ]);
    assert!(breaches(row, &allowed).is_empty(), "the row's allowed crates pass");
    let mut excluded = allowed.clone();
    excluded
        .extend([("jig-protocol-config".into(), Some("jig".into())), ("temper-service".into(), Some("temper".into()))]);
    assert_eq!(
        breaches(row, &excluded),
        ["temper-agent excludes jig-protocol-config", "temper-agent excludes temper-service"]
    );
    assert_eq!(breaches(row, &[]), ["temper-agent: empty closure", "temper-agent: closure lacks its own package"]);
    allowed.retain(|(package, _)| package != row.package);
    assert_eq!(breaches(row, &allowed), ["temper-agent: closure lacks its own package"]);
    let exception = Row {
        exceptions: &[
            ("jig-protocol-config", "checker fixture's tree exception"),
            ("temper-service", "checker fixture's package exception"),
        ],
        ..*row
    };
    assert!(breaches(&exception, &excluded).is_empty(), "each named exception passes");
}
