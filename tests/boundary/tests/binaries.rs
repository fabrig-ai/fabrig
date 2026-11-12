//! Each binary's normal dependency closure (temper/docs/design/protocol/binaries.md, 4).

use std::path::Path;
use std::process::{Command, Stdio};

#[derive(Debug)]
enum Rule {
    Only(&'static [&'static str]),
    Not(&'static [&'static str]),
}

#[derive(Debug)]
struct Row {
    package: &'static str,
    rules: &'static [(&'static str, Rule)],
    exceptions: &'static [(&'static str, &'static str)],
}

const ROWS: &[Row] = &[
    Row {
        package: "temper",
        rules: &[
            (
                "temper",
                Rule::Only(&[
                    "temper",
                    "temper-client-shell",
                    "temper-client-domain",
                    "temper-client-protocol-wire",
                    "temper-client-shell-cli",
                    "temper-client-shell-terminal",
                    "temper-client-shell-tui",
                    "temper-client-redaction",
                    "temper-client-tools",
                    "temper-client-tui",
                    "temper-client-view",
                    "temper-fetch",
                    "temper-fetch-shell",
                    "temper-shell",
                    "temper-docs",
                    "temper-version",
                ]),
            ),
            (
                "jig",
                Rule::Only(&[
                    "jig-client",
                    "jig-client-domain",
                    "jig-client-protocol-http",
                    "jig-client-protocol-link",
                    "jig-client-protocol-wire",
                    "jig-client-shell-native",
                    "jig-client-view",
                ]),
            ),
            ("smith", Rule::Only(&[])),
        ],
        exceptions: &[
            (
                "temper-settings-form",
                "The no_std settings form and its words are rendered by ctl's tools and decoded by the engine.",
            ),
            (
                "temper-protocol-records",
                "The pasted forge bundle's codec is encoded by the client and decoded by the engine, without its forge feature.",
            ),
            (
                "temper-protocol-reaches",
                "The no_std subjects of what reaches a person are shared by the client's forge pages and the engine.",
            ),
            (
                "jig-protocol-transcript",
                "Jig's bounded translation of smith's turns into client vocabulary is read by jig-client-protocol-link; cutting it is jig's change.",
            ),
            (
                "smith-transcript",
                "The sans-io turns beneath jig-protocol-transcript are shared vocabulary; cutting them removes no io or process from the client.",
            ),
        ],
    },
    Row {
        package: "temper-engine",
        rules: &[
            ("temper", Rule::Not(&["temper-worker-", "temper-service"])),
            ("jig", Rule::Not(&["jig-protocol-worker-agents", "jig-protocol-worker-link"])),
            (
                "smith",
                Rule::Not(&[
                    "smith-host-",
                    "smith-agent-",
                    "smith-local-",
                    "smith-inline-agent",
                    "smith-domain",
                    "smith-domain-",
                    "smith-protocol-channel",
                    "smith-protocol-events",
                    "smith-protocol-llm",
                    "smith-protocol-machine",
                    "smith-protocol-program",
                    "smith-events",
                ]),
            ),
        ],
        exceptions: &[
            (
                "smith-domain",
                "Jig's charter and the engine's translation speak smith's sans-io agent vocabulary; extracting it would remove no io or process.",
            ),
            (
                "smith-domain-run",
                "The charter's run is shared sans-io vocabulary; extracting it would remove no io or process.",
            ),
            (
                "smith-domain-session",
                "Start, Event and Transcript are shared sans-io vocabulary; extracting them would remove no io or process.",
            ),
            (
                "smith-domain-tools",
                "Smith's agent tools are shared sans-io vocabulary; extracting them would remove no io or process.",
            ),
        ],
    },
    Row {
        package: "temper-worker",
        rules: &[
            (
                "temper",
                Rule::Not(&[
                    "temper-engine-",
                    "temper-box-",
                    "temper-fake",
                    "temper-fake-",
                    "temper-init",
                    "temper-client-bundle",
                    "temper-client-shell-web",
                    "temper-forge-setup",
                    "temper-forge-setup-shell",
                    "temper-protocol-records-forge",
                    "temper-service",
                    "temper-agent-shell",
                ]),
            ),
            ("smith", Rule::Not(&["smith-agent-shell", "smith-agent-service", "smith-local-", "smith-inline-agent"])),
        ],
        exceptions: &[],
    },
    Row {
        package: "temper-agent",
        rules: &[
            (
                "temper",
                Rule::Only(&[
                    "temper-agent",
                    "temper-agent-shell",
                    "temper-shell",
                    "temper-docs",
                    "temper-launcher",
                    "temper-fetch",
                    "temper-fetch-shell",
                    "temper-version",
                ]),
            ),
            ("jig", Rule::Only(&[])),
        ],
        exceptions: &[],
    },
];

#[derive(Debug, PartialEq, Eq)]
enum Finding<'a> {
    Empty,
    LacksPackage,
    Excluded(&'a str),
    MissingException(&'a str),
    NeedlessException(&'a str),
}

fn pattern_matches(pattern: &str, name: &str) -> bool {
    if pattern.ends_with('-') { name.starts_with(pattern) } else { name == pattern }
}

fn excluded(row: &Row, name: &str, tree: Option<&str>) -> bool {
    row.rules.iter().any(|(ruled_tree, rule)| {
        tree == Some(*ruled_tree)
            && match rule {
                Rule::Only(patterns) => !patterns.iter().any(|pattern| pattern_matches(pattern, name)),
                Rule::Not(patterns) => patterns.iter().any(|pattern| pattern_matches(pattern, name)),
            }
    })
}

fn findings<'a>(row: &'a Row, closure: &'a [(String, Option<String>)]) -> Vec<Finding<'a>> {
    let mut findings = Vec::new();
    if closure.is_empty() {
        findings.push(Finding::Empty);
    }
    if !closure.iter().any(|(name, _)| name == row.package) {
        findings.push(Finding::LacksPackage);
    }
    for (name, tree) in closure {
        if excluded(row, name, tree.as_deref()) && !row.exceptions.iter().any(|(exception, _)| name == exception) {
            findings.push(Finding::Excluded(name));
        }
    }
    for &(name, _) in row.exceptions {
        match closure.iter().find(|(package, _)| package == name) {
            None => findings.push(Finding::MissingException(name)),
            Some((_, tree)) => {
                if !excluded(row, name, tree.as_deref()) {
                    findings.push(Finding::NeedlessException(name));
                }
            }
        }
    }
    findings
}

fn cargo_tree(root: &Path, package: &str, inverse: Option<&str>) -> Result<String, String> {
    let mut command = Command::new("cargo");
    command.args(["tree", "--locked", "--offline", "-p", package, "-e", "normal"]);
    if let Some(name) = inverse {
        command.args(["-i", name]);
    } else {
        command.args(["--prefix", "none", "--format", "{p}"]);
    }
    command.current_dir(root).stdin(Stdio::null());
    let output = command.output().map_err(|why| format!("{command:?}: {why}"))?;
    if !output.status.success() || output.stdout.trim_ascii().is_empty() {
        return Err(format!(
            "{command:?}: {} (nonempty output required)\n{}\n{}",
            output.status,
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr),
        ));
    }
    String::from_utf8(output.stdout).map_err(|why| format!("{command:?}: output is not UTF-8: {why}"))
}

fn closure(root: &Path, output: &str) -> Vec<(String, Option<String>)> {
    let mut packages: Vec<_> = output
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

#[test]
fn deployment_binaries_hold_their_closures() {
    let root = skein_scratch::paths::workspace();
    let mut failures = Vec::new();
    for row in ROWS {
        let output = match cargo_tree(&root, row.package, None) {
            Ok(output) => output,
            Err(why) => {
                failures.push(why);
                continue;
            }
        };
        let closure = closure(&root, &output);
        println!("{}: {} crates, {} exceptions", row.package, closure.len(), row.exceptions.len());
        for finding in findings(row, &closure) {
            let detail = match finding {
                Finding::Empty => "empty closure".to_owned(),
                Finding::LacksPackage => "closure lacks its own package".to_owned(),
                Finding::Excluded(name) => {
                    let path = cargo_tree(&root, row.package, Some(name)).unwrap_or_else(|why| why);
                    format!("excluded crate {name}\n{path}")
                }
                Finding::MissingException(name) => format!("{name}: an exception the closure no longer holds: drop it"),
                Finding::NeedlessException(name) => format!("{name}: an exception no rule excludes: drop it"),
            };
            failures.push(format!("{}: {detail}", row.package));
        }
    }
    assert!(
        failures.is_empty(),
        "Deployment binaries' dependency boundaries (temper/docs/design/protocol/binaries.md, 4):\n{}",
        failures.join("\n"),
    );
}

#[test]
fn closure_matching() {
    use Finding::{Empty, Excluded, LacksPackage, MissingException, NeedlessException};

    struct Case {
        label: &'static str,
        rules: &'static [(&'static str, Rule)],
        exceptions: &'static [(&'static str, &'static str)],
        closure: &'static [(&'static str, Option<&'static str>)],
        expected: &'static [Finding<'static>],
    }

    for case in [
        Case {
            label: "only names; external crates are never excluded",
            rules: &[("temper", Rule::Only(&["temper-agent", "temper-shell"]))],
            exceptions: &[],
            closure: &[
                ("temper-agent", Some("temper")),
                ("temper-shell", Some("temper")),
                ("temper-service", Some("temper")),
                ("serde", None),
                ("temper-outside", None),
            ],
            expected: &[Excluded("temper-service")],
        },
        Case {
            label: "not a prefix; unruled trees are held whole",
            rules: &[("smith", Rule::Not(&["smith-agent-"]))],
            exceptions: &[],
            closure: &[
                ("temper-agent", Some("temper")),
                ("smith-agent-shell", Some("smith")),
                ("smith-agents", Some("smith")),
                ("smith-agent-outside", None),
            ],
            expected: &[Excluded("smith-agent-shell")],
        },
        Case {
            label: "not a name is not a prefix",
            rules: &[("smith", Rule::Not(&["smith-domain"]))],
            exceptions: &[],
            closure: &[
                ("temper-agent", Some("temper")),
                ("smith-domain", Some("smith")),
                ("smith-domain-run", Some("smith")),
            ],
            expected: &[Excluded("smith-domain")],
        },
        Case {
            label: "only a prefix does not admit a similar name",
            rules: &[("temper", Rule::Only(&["temper-agent-"]))],
            exceptions: &[],
            closure: &[("temper-agent", Some("temper")), ("temper-agent-shell", Some("temper"))],
            expected: &[Excluded("temper-agent")],
        },
        Case {
            label: "only none excludes the named tree alone",
            rules: &[("jig", Rule::Only(&[]))],
            exceptions: &[],
            closure: &[("temper-agent", Some("temper")), ("jig-client", Some("jig")), ("jig-external", None)],
            expected: &[Excluded("jig-client")],
        },
        Case {
            label: "exceptions are exact, held and excluded",
            rules: &[("smith", Rule::Not(&["smith-domain-"]))],
            exceptions: &[("smith-domain-run", "shared vocabulary")],
            closure: &[
                ("temper-agent", Some("temper")),
                ("smith-domain-run", Some("smith")),
                ("smith-domain-run-extra", Some("smith")),
            ],
            expected: &[Excluded("smith-domain-run-extra")],
        },
        Case {
            label: "missing exception",
            rules: &[("smith", Rule::Not(&["smith-domain-"]))],
            exceptions: &[("smith-domain-run", "shared vocabulary")],
            closure: &[("temper-agent", Some("temper"))],
            expected: &[MissingException("smith-domain-run")],
        },
        Case {
            label: "needless exception",
            rules: &[("smith", Rule::Not(&["smith-agent-"]))],
            exceptions: &[("smith-domain-run", "shared vocabulary")],
            closure: &[("temper-agent", Some("temper")), ("smith-domain-run", Some("smith"))],
            expected: &[NeedlessException("smith-domain-run")],
        },
        Case { label: "empty closure", rules: &[], exceptions: &[], closure: &[], expected: &[Empty, LacksPackage] },
        Case {
            label: "lacks its own package",
            rules: &[],
            exceptions: &[],
            closure: &[("serde", None)],
            expected: &[LacksPackage],
        },
    ] {
        let row = Row { package: "temper-agent", rules: case.rules, exceptions: case.exceptions };
        let closure: Vec<_> =
            case.closure.iter().map(|(name, tree)| ((*name).to_owned(), tree.map(str::to_owned))).collect();
        assert_eq!(findings(&row, &closure), case.expected, "{}: {row:?}, {closure:?}", case.label);
    }
}
