//! A deployment binary's dependency closure holds its boundary
//! (temper/docs/design/protocol/README.md, 4.1).

use std::collections::BTreeSet;
use std::path::Path;
use std::process::Command;

/// A binary's package, the crates its closure must not hold, and its exceptions.
struct Binary {
    package: &'static str,
    excluded: &'static [&'static str],
    exceptions: &'static [(&'static str, &'static str)],
}

const BINARIES: &[Binary] = &[Binary {
    package: "temper-engine",
    excluded: &[
        "temper-worker-",
        "jig-protocol-worker-agents",
        "jig-protocol-worker-link",
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
}];

#[derive(Debug, PartialEq, Eq)]
enum Finding<'a> {
    Excluded(&'a str),
    MissingException(&'a str),
}

fn findings<'a>(excluded: &[&str], exceptions: &[(&'a str, &str)], closure: &BTreeSet<&'a str>) -> Vec<Finding<'a>> {
    let mut findings = Vec::new();
    for &name in closure {
        let excluded =
            excluded.iter().any(|entry| if entry.ends_with('-') { name.starts_with(entry) } else { name == *entry });
        if excluded && !exceptions.iter().any(|(exception, _)| *exception == name) {
            findings.push(Finding::Excluded(name));
        }
    }
    for &(name, _) in exceptions {
        if !closure.contains(name) {
            findings.push(Finding::MissingException(name));
        }
    }
    findings
}

fn cargo_tree(root: &Path, package: &str, inverse: Option<&str>) -> Result<String, String> {
    let mut command = Command::new("cargo");
    command.args(["tree", "-p", package, "-e", "normal"]);
    if let Some(name) = inverse {
        command.args(["-i", name]);
    } else {
        command.args(["--prefix", "none", "-f", "{p}"]);
    }
    command.args(["--locked", "--offline"]).current_dir(root);
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

#[test]
fn deployment_binaries_hold_their_closures() {
    let root = skein_scratch::paths::workspace();
    let mut failures = Vec::new();
    for binary in BINARIES {
        let output = match cargo_tree(&root, binary.package, None) {
            Ok(output) => output,
            Err(why) => {
                failures.push(why);
                continue;
            }
        };
        let closure: BTreeSet<_> = output.lines().filter_map(|line| line.split_whitespace().next()).collect();
        println!("{}: {} crates, {} exceptions", binary.package, closure.len(), binary.exceptions.len());
        for finding in findings(binary.excluded, binary.exceptions, &closure) {
            match finding {
                Finding::Excluded(name) => {
                    let path = cargo_tree(&root, binary.package, Some(name)).unwrap_or_else(|why| why);
                    failures.push(format!("{}: excluded crate {name}\n{path}", binary.package));
                }
                Finding::MissingException(name) => {
                    failures
                        .push(format!("{}: {name}: an exception the closure no longer holds: drop it", binary.package));
                }
            }
        }
    }
    assert!(
        failures.is_empty(),
        "Deployment binaries' dependency boundaries (temper/docs/design/protocol/README.md, 4.1):\n{}",
        failures.join("\n"),
    );
}

#[test]
fn closure_matching() {
    use Finding::{Excluded, MissingException};

    for (excluded, exceptions, closure, expected) in [
        (
            &["smith-agent-"][..],
            &[][..],
            &["smith-agent-shell", "smith-agents"][..],
            &[Excluded("smith-agent-shell")][..],
        ),
        (&["smith-domain"][..], &[][..], &["smith-domain", "smith-domain-run"][..], &[Excluded("smith-domain")][..]),
        (
            &["smith-domain"][..],
            &[("smith-domain", "The engine speaks the domain's sans-io vocabulary.")][..],
            &["smith-domain"][..],
            &[][..],
        ),
        (
            &["smith-domain"][..],
            &[("smith-domain", "The engine speaks the domain's sans-io vocabulary.")][..],
            &["skein-lib"][..],
            &[MissingException("smith-domain")][..],
        ),
        (&["smith-agent-"][..], &[][..], &["skein-lib", "temper-engine"][..], &[][..]),
    ] {
        let closure = closure.iter().copied().collect();
        assert_eq!(findings(excluded, exceptions, &closure), expected, "{excluded:?}, {exceptions:?}, {closure:?}");
    }
}
