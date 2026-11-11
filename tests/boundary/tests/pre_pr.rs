//! Delivery selection, inherited failures and bounded output (tools/pre-pr/design.md, 1–9).
//! Formatting and baselines retain their budget (temper/docs/design/protocol/configuration/reference.md).

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::process::{Command, Output};

use serde_json::{Value, json};

const TREES: [&str; 5] = ["skein", "smith", "jig", "temper", "hearth"];

// Twelve formatter launches cost 69,758 ms; the baselines cost 17,000 ms.
// Each launch pays 5,500 ms plus its share of source work. No real time passes.
const PROGRAM: &str = r"#!/usr/bin/python3
import json
import os
from pathlib import Path
import signal
import subprocess
import sys
import xml.etree.ElementTree as ET

name = Path(sys.argv[0]).name
args = sys.argv[1:]
common = subprocess.check_output(['/usr/bin/git', 'rev-parse', '--git-common-dir'], text=True).strip()
git_dir = Path(common).resolve()
root = git_dir.parent
state_path = git_dir / 'budget.json'
if name == 'git':
    if state_path.exists():
        state = json.loads(state_path.read_text())
        if args[0] in ('merge-base', 'worktree', 'checkout', 'clean'):
            state['events'].append(['git', args])
            state_path.write_text(json.dumps(state))
    os.execv('/usr/bin/git', ['git'] + args)
state = json.loads(state_path.read_text())
side = 'base' if 'temper-pre-pr/base' in str(Path.cwd()) else 'head'
if name == 'one-process' or (name == 'cargo' and args[0] == 'clippy'):
    assert 'GIT_INDEX_FILE' not in os.environ, 'private index leaked to build or tests'
    assert os.environ.get('TEMPER_INTERNAL_BUNDLE') == 'none'
fail = state.get('fail', '')
def spend(cost, event):
    state['ms'] += cost
    state['events'].append(event)
    state_path.write_text(json.dumps(state))
    if state['ms'] > 60000:
        print('timed-out: given_ms=60000', file=sys.stderr)
        sys.exit(124)
def format_files(paths):
    spend(5500 + sum(315 if path.endswith('11.rs') else 313 for path in paths),
          ['format', args, paths])
    if fail == 'format':
        print('format fault', file=sys.stderr)
        sys.exit(1)
    if fail == 'interrupt':
        os.kill(os.getpid(), signal.SIGTERM)
metadata = json.loads((git_dir / 'metadata.json').read_text())
if name == 'cargo' and args[0] == 'metadata':
    spend(1000, ['metadata', args, side])
    print(json.dumps(metadata))
elif name == 'cargo' and args == ['fmt', '--all', '--check']:
    for package in metadata['packages']:
        format_files([target['src_path'] for target in package['targets']])
elif name == 'rustfmt':
    assert '--check' in args
    paths = [arg for arg in args if arg.endswith('.rs')]
    for package in metadata['packages']:
        for target in package['targets']:
            if target['src_path'] in paths:
                assert args[args.index('--edition') + 1] == target['edition']
                directory = Path(target['src_path']).parent
                while not (directory / 'rustfmt.toml').exists():
                    directory = directory.parent
                assert args[args.index('--config-path') + 1] == str(directory / 'rustfmt.toml')
    format_files(paths)
elif name == 'cargo':
    assert '--workspace' not in args and args[0] != 'clean'
    packages = [args[i + 1] for i, arg in enumerate(args) if arg == '-p']
    kind = args[0]
    spend(100, [kind, side, packages, args])
    ledger = state.get('head', {})
    if kind == 'clippy':
        assert '--all-targets' in args and '--message-format=json' in args
        assert args[-3:] == ['--', '-D', 'warnings']
        errors = [e for e in ledger.get('clippy', []) if e['package'] in packages]
        for error in errors:
            package = next(p for p in metadata['packages'] if p['name'] == error['package'])
            print(json.dumps({'reason': 'compiler-message', 'package_id': package['id'], 'message': {
                'level': 'error', 'code': {'code': error['code']}, 'message': error['message'],
                'spans': [{'is_primary': True, 'file_name': str(Path.cwd() / error['file']),
                           'line_start': error['line']}], 'rendered': error['rendered']}}))
        sys.exit(101 if errors else ledger.get('clippy_exit', 0))
    assert kind != 'nextest', 'pre-pr must not run cargo nextest'
    raise AssertionError('unexpected cargo command: ' + str(args))
elif name == 'one-process':
    assert Path.cwd() == root, 'tests start at the root'
    assert args[:1] == ['--out']
    directory = Path(args[1])
    assert directory.name == 'tests' and not directory.exists()
    packages = [args[i + 1] for i, arg in enumerate(args) if arg == '-p']
    assert args[2:] == ['default'] + [arg for p in packages for arg in ('-p', p)] + ['--no-tests=warn']
    spend(100, ['one-process', side, packages, args])
    ledger = state.get('head', {})
    directory.mkdir(mode=0o700)
    tests = [t for t in ledger.get('tests', []) if t['package'] in packages]
    document = ET.Element('testsuites')
    for test in tests:
        suite = ET.SubElement(document, 'testsuite', name=test['binary'])
        case = ET.SubElement(suite, 'testcase', name=test['name'])
        if test['outcome'] in ('failed', 'crashed', 'cancelled'):
            ET.SubElement(case, 'failure', message='ledger failure')
        ET.SubElement(case, 'system-out').text = test['output']
    (directory / 'junit.xml').write_bytes(ET.tostring(document))
    (directory / 'results.json').write_text(json.dumps({
        'tests': [dict(binary_id=t['binary'], test_name=t['name'], outcome=t['outcome'],
                       alone=t.get('alone'), before=t.get('before', [])) for t in tests],
        'unobserved': ledger.get('unobserved', [])}))
    (directory / 'run.json').write_text(json.dumps({'starved': ledger.get('starved', False)}))
    print('last streamed line: ' + side, flush=True)
    if ledger.get('output'):
        print(ledger['output'], flush=True)
    sys.exit(ledger.get('one_process_exit', 100 if any(t['outcome'] in ('failed', 'crashed') for t in tests) else 0))
else:
    tree = Path.cwd().name
    if Path('delivery.rs').exists():
        tracked = subprocess.check_output(['git', 'ls-files', '-z'])
        assert b'delivery.rs' in tracked.split(b'\0'), 'baseline missed new Rust source'
    spend(3400, ['baseline', tree])
    if fail == tree:
        print(tree + ' baseline fault', file=sys.stderr)
        sys.exit(1)
";

fn write(root: &Path, path: &str, text: &str) {
    let path = root.join(path);
    fs::create_dir_all(path.parent().expect("a parent")).expect("create directory");
    fs::write(path, text).expect("write fixture");
}

fn command(root: &Path, program: &str, args: &[&str]) -> Output {
    Command::new(program)
        .args(args)
        .current_dir(root)
        .env_clear()
        .env("HOME", root.join(".git/home"))
        .env("PATH", format!("{}:/usr/bin:/bin", root.join(".git/bin").display()))
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_AUTHOR_NAME", "Fixture")
        .env("GIT_AUTHOR_EMAIL", "fixture@example.invalid")
        .env("GIT_COMMITTER_NAME", "Fixture")
        .env("GIT_COMMITTER_EMAIL", "fixture@example.invalid")
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .output()
        .expect("start fixture command")
}

fn git(root: &Path, args: &[&str]) -> Vec<u8> {
    let output = command(root, "git", args);
    assert!(output.status.success(), "git {args:?}: {}", String::from_utf8_lossy(&output.stderr));
    output.stdout
}

fn fixture() -> skein_scratch::Scratch {
    let scratch = skein_scratch::Scratch::new("pre-pr");
    let root = scratch.path();
    git(root, &["init", "-q"]);
    fs::create_dir_all(root.join(".git/home")).expect("temporary HOME");
    let workspace = skein_scratch::paths::workspace();
    for path in
        [".temper/pre-pr", "tools/pre-pr/pre_pr.py", "tools/citations/citations.py", "tools/citations/sized.txt"]
    {
        if let Ok(source) = fs::read_to_string(workspace.join(path)) {
            write(root, path, &source);
        }
    }
    for name in ["cargo", "rustfmt", "git"] {
        write(root, &format!(".git/bin/{name}"), PROGRAM);
        fs::set_permissions(root.join(format!(".git/bin/{name}")), fs::Permissions::from_mode(0o700))
            .expect("executable check");
    }
    write(root, "tools/tiers/one-process", PROGRAM);
    fs::set_permissions(root.join("tools/tiers/one-process"), fs::Permissions::from_mode(0o700))
        .expect("executable one-process");
    for tree in TREES {
        write(root, &format!("{tree}/scripts/old_forms.py"), PROGRAM);
        write(root, &format!("{tree}/rustfmt.toml"), "edition = \"2024\"\n");
    }
    write(root, "rustfmt.toml", "edition = \"2024\"\n");
    let packages: Vec<_> = (0..12)
        .map(|n| {
            let tree = TREES.get(n % 6).copied().unwrap_or("root");
            let path = format!("{tree}/source-{n}.rs");
            write(root, &path, "fn main() {}\n");
            let edition = if n == 5 || n == 11 { "2021" } else { "2024" };
            let name = match n {
                0 => "A".to_owned(),
                1 => "B".to_owned(),
                _ => format!("P{n}"),
            };
            let directory = name.to_lowercase();
            write(root, &format!("{directory}/Cargo.toml"), "[package]\n");
            write(root, &format!("{directory}/src/lib.rs"), "fn source() {}\n");
            json!({"name": name, "id": format!("path+fixture#{name}@0.1.0"),
                "manifest_path": root.join(directory).join("Cargo.toml"), "dependencies": [],
                "edition": "2024", "targets": [{
                "src_path": root.join(path), "edition": edition, "kind": ["bin"]
            }]})
        })
        .collect();
    let members: Vec<_> = packages.iter().map(|p| p["id"].clone()).collect();
    write(
        root,
        ".git/metadata.json",
        &json!({"packages": packages, "workspace_members": members,
        "target_directory": root.join("target")})
        .to_string(),
    );
    reset(root, "");
    git(root, &["add", "."]);
    git(root, &["commit", "-qm", "fixture"]);
    scratch
}

fn reset(root: &Path, failure: &str) {
    write(root, ".git/budget.json", &json!({"ms": 0, "events": [], "fail": failure}).to_string());
}

fn budget(root: &Path) -> Value {
    serde_json::from_slice(&fs::read(root.join(".git/budget.json")).expect("budget ledger")).expect("JSON ledger")
}

fn deliver(root: &Path) -> Output {
    command(root, "sh", &[".temper/pre-pr"])
}

fn passed(root: &Path) -> bool {
    root.join(".git/pre-pr-passed").exists()
}

fn coverage(root: &Path) {
    let state = budget(root);
    let events = state["events"].as_array().expect("events");
    for tree in TREES {
        assert!(events.contains(&json!(["baseline", tree])), "missing {tree} baseline: {state}");
    }
    for n in 0..12 {
        let suffix = format!("/source-{n}.rs");
        assert!(
            events.iter().any(|event| event[0] == "format"
                && event[2]
                    .as_array()
                    .expect("formatted paths")
                    .iter()
                    .any(|path| path.as_str().expect("source path").ends_with(&suffix))),
            "missing {suffix}: {state}"
        );
    }
}

#[test]
fn initial_design_delivery_fits_check_budget() {
    let scratch = fixture();
    let root = scratch.path();
    assert!(!passed(root), "first delivery has no success marker");
    let index = fs::read(root.join(".git/index")).expect("real index");
    for addition in ["first", "second"] {
        write(root, &format!("temper/docs/design/{addition}.md"), "# Design addition\n");
        reset(root, "");
        let output = deliver(root);
        assert!(
            output.status.success(),
            "initial design delivery exceeds its unchanged 60000 ms budget: {}\n{}",
            String::from_utf8_lossy(&output.stderr),
            budget(root)
        );
        assert!(budget(root)["ms"].as_u64().expect("milliseconds") <= 60_000);
        coverage(root);
        assert!(passed(root));
        assert_eq!(fs::read(root.join(".git/index")).expect("index after delivery"), index);
    }
}

#[test]
fn failed_or_interrupted_checks_never_pass() {
    let scratch = fixture();
    let root = scratch.path();
    for fault in ["format", "interrupt", "skein", "smith", "jig", "temper", "hearth"] {
        reset(root, fault);
        let output = deliver(root);
        assert!(!output.status.success(), "accepted {fault}");
        assert!(!passed(root), "cached {fault} as passed");
        if fault != "interrupt" {
            assert!(String::from_utf8_lossy(&output.stderr).contains(fault), "fault propagated: {output:?}");
        }
    }
    reset(root, "");
    assert!(deliver(root).status.success(), "failed checks can be retried");
    coverage(root);
}

#[test]
fn only_the_exact_passed_tree_skips_checks() {
    let scratch = fixture();
    let root = scratch.path();
    assert!(deliver(root).status.success());
    reset(root, "format");
    assert!(deliver(root).status.success(), "identical passed tree");
    assert_eq!(budget(root)["ms"], 0);
    write(root, "temper/source-3.rs", "fn changed() {}\n");
    assert!(!deliver(root).status.success(), "changed source needs a new check");
    reset(root, "");
    assert!(deliver(root).status.success());
    coverage(root);
    for message in ["first pin", "moved pin"] {
        git(root, &["commit", "--allow-empty", "-qm", message]);
        git(root, &["update-ref", "refs/temper/landing", "HEAD"]);
        reset(root, "");
        assert!(deliver(root).status.success(), "new base must be checked");
        coverage(root);
    }
}

#[test]
fn private_index_checks_new_sources_without_staging_them() {
    let scratch = fixture();
    let root = scratch.path();
    pin(root);
    let index = fs::read(root.join(".git/index")).expect("real index");
    write(root, "temper/delivery.rs", "fn delivered() {}\n");
    write(root, "a/src/new.rs", "fn delivered() {}\n");
    let output = deliver(root);
    assert!(output.status.success(), "checks must see new source: {}", text(&output));
    assert!(fs::read(root.join(".git/index")).expect("index after checks") == index, "real index changed");
}

fn text(output: &Output) -> String {
    format!("{}{}", String::from_utf8_lossy(&output.stdout), String::from_utf8_lossy(&output.stderr))
}

fn pin(root: &Path) {
    git(root, &["update-ref", "refs/temper/landing", "HEAD"]);
}

fn ledger(root: &Path, additions: &Value) {
    let mut state = budget(root);
    for (key, value) in additions.as_object().expect("ledger fields") {
        state[key] = value.clone();
    }
    write(root, ".git/budget.json", &state.to_string());
}

fn build_events(root: &Path) -> Vec<Value> {
    budget(root)["events"]
        .as_array()
        .expect("events")
        .iter()
        .filter(|event| event[0] == "clippy" || event[0] == "one-process")
        .cloned()
        .collect()
}

#[test]
fn the_base_is_the_pins_merge_base() {
    for row in ["no pin", "behind", "merge", "unrelated", "unchecked save"] {
        let scratch = fixture();
        let root = scratch.path();
        let ancestor = String::from_utf8(git(root, &["rev-parse", "HEAD"])).expect("commit").trim().to_owned();
        let mut expected = String::new();
        let mut merge_args = vec!["merge-base", "refs/temper/landing", "HEAD"];
        match row {
            "no pin" => {}
            "behind" => {
                pin(root);
                git(root, &["commit", "--allow-empty", "-qm", "head"]);
                expected.clone_from(&ancestor);
            }
            "unchecked save" => {
                git(root, &["commit", "--allow-empty", "-qm", "Unchecked: work in progress, saved before a question"]);
                pin(root);
                expected.clone_from(&ancestor);
            }
            "merge" | "unrelated" => {
                let mut args = vec!["commit-tree", "HEAD^{tree}", "-m", "pin"];
                if row == "merge" {
                    args.extend(["-p", &ancestor]);
                }
                let other = String::from_utf8(git(root, &args)).expect("other commit").trim().to_owned();
                git(root, &["update-ref", "refs/temper/landing", &other]);
                if row == "merge" {
                    git(root, &["commit", "--allow-empty", "-qm", "head"]);
                    write(root, ".git/MERGE_HEAD", &other);
                    expected = other;
                    merge_args.push("MERGE_HEAD");
                }
            }
            _ => unreachable!("table row"),
        }
        write(root, "temper/docs/addition.md", "# Delivery\n");
        let output = deliver(root);
        let log = text(&output);
        assert!(output.status.success(), "{row}: {log}");
        assert!(build_events(root).is_empty(), "{row}: docs build nothing");
        if expected.is_empty() {
            coverage(root);
            let reason = if row == "no pin" {
                "no refs/temper/landing"
            } else {
                "no merge base of refs/temper/landing and HEAD"
            };
            assert!(
                log.contains(&format!("pre-pr: no base ({reason}): the format check and the counted baselines alone")),
                "{row}: {log}"
            );
        } else {
            assert!(log.contains("docs only: citations"), "{row}: {log}");
            assert!(budget(root)["events"].as_array().expect("events").iter().all(|event| event[0] != "format"));
            assert!(
                log.contains(&format!("base {expected}, merge base of refs/temper/landing and HEAD")),
                "{row}: {log}"
            );
            assert!(
                budget(root)["events"].as_array().expect("events").contains(&json!(["git", merge_args])),
                "merge command: {}",
                budget(root)
            );
        }
    }
}

#[test]
fn the_sets_are_changed_packages_dependents_and_path_readers() {
    for (row, path, lints, tests) in [
        ("normal", "a/src/lib.rs", vec!["A"], vec!["A", "B"]),
        ("dev", "a/src/lib.rs", vec!["A"], vec!["A", "B"]),
        ("untracked", "a/src/new.rs", vec!["A"], vec!["A"]),
        ("deleted", "a/src/lib.rs", vec!["A"], vec!["A"]),
        ("root manifest", "Cargo.toml", vec![], vec![]),
        ("docs", "temper/docs/selection.md", vec![], vec![]),
        ("reader", "a/src/lib.rs", vec!["A"], vec!["A", "B"]),
        ("reader only", "temper/docs/selection.md", vec![], vec!["B"]),
        ("prefix-less", "temper/docs/selection.md", vec![], vec![]),
        ("component boundary", "a-sibling/new.rs", vec![], vec![]),
        ("unknown reader", "temper/docs/selection.md", vec![], vec![]),
    ] {
        let scratch = fixture();
        let root = scratch.path();
        let metadata_path = root.join(".git/metadata.json");
        let mut metadata: Value = serde_json::from_slice(&fs::read(&metadata_path).expect("metadata")).expect("JSON");
        if row == "normal" || row == "dev" {
            metadata["packages"][1]["dependencies"] = json!([{
                "name": "A", "path": root.join("a"), "kind": if row == "dev" { Some("dev") } else { None }
            }]);
            // Dependents are direct, not transitive.
            metadata["packages"][2]["dependencies"] = json!([{"name": "B", "path": root.join("b"), "kind": null}]);
            fs::write(metadata_path, metadata.to_string()).expect("dependencies");
        }
        let always = match row {
            "reader" => "B a/\n",
            "reader only" => "B temper/docs/\n",
            "unknown reader" => "missing shared/\n",
            _ => "B\n",
        };
        write(root, ".config/gate/always.txt", always);
        if row == "reader only" {
            write(root, ".config/gate/read-by-tests.txt", "temper/docs/selection.md\n");
        }
        write(root, "Cargo.toml", "# root manifest\n");
        git(root, &["add", "."]);
        git(root, &["commit", "-qm", "reader configuration"]);
        pin(root);
        let path = if row == "unknown reader" { "shared/input.txt" } else { path };
        if row == "deleted" {
            fs::remove_file(root.join(path)).expect("deleted source");
        } else {
            write(root, path, "# changed\n");
        }
        let output = deliver(root);
        let log = text(&output);
        assert!(output.status.success(), "{row}: {log}");
        let events = build_events(root);
        let actual_lints: Vec<_> = events.iter().filter(|e| e[0] == "clippy").map(|e| e[2].clone()).collect();
        let actual_tests: Vec<_> = events.iter().filter(|e| e[0] == "one-process").map(|e| e[2].clone()).collect();
        assert_eq!(actual_lints, if lints.is_empty() { vec![] } else { vec![json!(lints)] }, "{row}: {log}");
        assert_eq!(actual_tests, if tests.is_empty() { vec![] } else { vec![json!(tests)] }, "{row}: {log}");
        if lints.is_empty() && tests.is_empty() && !log.contains("docs only:") {
            assert!(log.contains("pre-pr: no package changed"), "{row}: {log}");
        }
        if row == "unknown reader" {
            assert!(log.contains("missing"), "unknown reader must be printed: {log}");
        }
    }
}

fn test_failure(name: &str, output: &str) -> Value {
    json!({"package": "A", "binary": "A::rules", "name": name, "output": output, "outcome": "failed"})
}

fn lint_failure(line: u32, message: &str) -> Value {
    json!({"package": "A", "code": "clippy::rule", "file": "a/src/lib.rs", "line": line,
        "message": message, "rendered": format!("{message} at line {line}\n")})
}

fn changed_field(mut value: Value, field: &str, replacement: Value) -> Value {
    value[field] = replacement;
    value
}

#[test]
#[expect(clippy::too_many_lines, reason = "One ledger table compares failure and starvation verdicts")]
fn only_what_the_delivery_broke_refuses() {
    for (row, head, known_reds, accepted, named) in [
        (
            "inherited test",
            json!({"tests": [test_failure("rule", "head junit tail")]}),
            Some("- test A::rules rule\n"),
            true,
            "inherited: test A::rules rule (known red, .temper/known-reds.md)",
        ),
        (
            "different test",
            json!({"tests": [test_failure("rule", "head junit tail")]}),
            Some("- test A::rules other\n"),
            false,
            "test A::rules rule",
        ),
        ("no file", json!({"tests": [test_failure("rule", "head junit tail")]}), None, false, "test A::rules rule"),
        (
            "different binary",
            json!({"tests": [test_failure("rule", "head junit tail")]}),
            Some("- test A::other rule\n"),
            false,
            "test A::rules rule",
        ),
        (
            "inherited clippy",
            json!({"clippy": [lint_failure(20, "any lint message")]}),
            Some("- lint A clippy::rule a/src/lib.rs\n"),
            true,
            "inherited: clippy A clippy::rule a/src/lib.rs (known red, .temper/known-reds.md)",
        ),
        (
            "different lint code",
            json!({"clippy": [changed_field(lint_failure(20, "lint fault"), "code", json!("clippy::other"))]}),
            Some("- lint A clippy::rule a/src/lib.rs\n"),
            false,
            "clippy A clippy::other",
        ),
        (
            "different lint file",
            json!({"clippy": [changed_field(lint_failure(20, "lint fault"), "file", json!("a/src/other.rs"))]}),
            Some("- lint A clippy::rule a/src/lib.rs\n"),
            false,
            "clippy A clippy::rule a/src/other.rs",
        ),
        (
            "build with entries",
            json!({"one_process_exit": 2, "output": "one-process: nextest listing exited with 101"}),
            Some("- test A::rules rule\n- lint A clippy::rule a/src/lib.rs\n"),
            false,
            "build\nno tests ran",
        ),
        (
            "lint test and build failures",
            json!({"clippy": [lint_failure(2, "lint fault")],
                "tests": [test_failure("rule", "head junit tail")], "one_process_exit": 2, "output": "one-process: nextest listing exited with 101"}),
            Some("- test A::rules rule\n- lint A clippy::rule a/src/lib.rs\n"),
            false,
            "build\nno tests ran",
        ),
        (
            "head check error",
            json!({"one_process_exit": 2}),
            Some("- test A::rules rule\n"),
            false,
            "check one-process",
        ),
        (
            "starved",
            json!({"starved": true, "one_process_exit": 0,
                "tests": [changed_field(test_failure("rule", "cancelled"), "outcome", json!("cancelled"))],
                "unobserved": [{"binary_id": "A::rules", "test_name": "later"},
                    {"binary_id": "A::rules", "test_name": "last"}]}),
            Some("- test A::rules rule\n"),
            true,
            "pre-pr: tests starved: 1 starved or cancelled, 2 unobserved, a warning (skein/docs/foundation/testing-strategy.md, 8.1)",
        ),
    ] {
        let scratch = fixture();
        let root = scratch.path();
        pin(root);
        write(root, "a/src/lib.rs", "fn changed() {}\n");
        if let Some(entries) = known_reds {
            write(root, ".temper/known-reds.md", entries);
        }
        ledger(root, &json!({"head": head}));
        let output = deliver(root);
        let log = text(&output);
        assert_eq!(output.status.code(), Some(i32::from(!accepted)), "{row}: {log}");
        assert!(log.contains(named), "{row}: missing {named}: {log}");
        assert_eq!(passed(root), accepted, "{row}: cache only passes");
        let events = build_events(root);
        assert!(
            events.iter().any(|e| e[0] == "one-process" && e[1] == "head"),
            "tests run even after clippy fails: {row}"
        );
        let state = budget(root);
        for event in state["events"].as_array().expect("events") {
            let side = if event[0] == "metadata" { &event[2] } else { &event[1] };
            assert_ne!(side, "base", "no base cargo: {row}: {event}");
        }
        assert!(!root.join(".git/temper-pre-pr/base").exists(), "no base worktree: {row}");
        assert!(!log.contains("inherited: build"), "build always refuses: {row}: {log}");
        if row == "different test" || row == "no file" {
            assert!(log.contains("head junit tail"), "JUnit output in summary: {log}");
        }
        if row == "inherited test" {
            assert!(log.contains("pre-pr: tests: inherited,"), "inherited check timed: {log}");
        }
        if row == "inherited clippy" {
            assert!(log.contains("pre-pr: clippy: inherited,"), "inherited check timed: {log}");
        }
        if row == "starved" {
            assert_eq!(log.matches(named).count(), 2, "warning after run and in summary: {log}");
            assert!(log.contains("pre-pr: tests: passed,"), "starvation passes: {log}");
            assert!(!log.contains("inherited: test"), "cancelled tests are not failures: {log}");
        }
    }
}

#[test]
fn the_summary_fits_the_kept_tail() {
    let scratch = fixture();
    let root = scratch.path();
    pin(root);
    write(root, "a/src/lib.rs", "fn changed() {}\n");
    let tail = format!("{}\n", "x".repeat(63)).repeat(64);
    assert_eq!(tail.len(), 4096);
    let failures: Vec<_> = (0..40).map(|n| test_failure(&format!("rule_{n:02}"), &tail)).collect();
    ledger(root, &json!({"head": {"tests": failures}}));
    let output = deliver(root);
    let log = String::from_utf8(output.stdout).expect("UTF-8 log");
    assert_eq!(output.status.code(), Some(1), "{log}");
    let (_, after_tools) = log.rsplit_once("last streamed line:").expect("streamed tools");
    let (_, summary) = after_tools.split_once("pre-pr: refused").expect("summary last");
    assert!(summary.len() + "pre-pr: refused".len() <= 8192, "{} bytes", summary.len());
    assert!(summary.contains("test A::rules rule_00"), "first failure kept: {summary}");
    assert!(summary.contains(" bytes cut]"), "cut marked: {summary}");
    assert!(summary.contains(" more failures]"), "omitted failures counted: {summary}");
    let entries: Vec<_> = summary.split("test A::rules ").skip(1).collect();
    let kept = entries.len();
    assert!(summary.contains(&format!("[\u{2026} {} more failures]", 40 - kept)), "omission count: {summary}");
    for entry in entries {
        let (_, output) = entry.split_once('\n').expect("failure output");
        let output = output.lines().take_while(|line| !line.contains("more failures")).collect::<Vec<_>>().join("\n");
        assert!(output.len() < 1536, "failure tail including marker: {} bytes", output.len() + 1);
        let (marker, content) = output.split_once('\n').expect("marker and tail");
        assert!(marker.starts_with("[\u{2026} ") && marker.ends_with(" bytes cut]"), "{marker}");
        assert!(content.lines().all(|line| line.len() == 63), "tail starts at a whole line: {content}");
        assert!(content.ends_with(&"x".repeat(63)), "last output line retained");
    }
    assert!(log.find("last streamed line:").expect("streamed tools") < log.rfind("pre-pr: refused").expect("summary"));
}
