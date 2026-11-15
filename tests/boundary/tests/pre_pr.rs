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
assert 'TIERS_HOLD' not in os.environ, 'hold setting leaked to child'
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
if name == 'hold':
    assert Path.cwd() == root
    spend(0, ['hold-start', args])
    if args == ['fail']:
        sys.exit(3)
    print('held', flush=True)
    sys.stdin.read()
    state = json.loads(state_path.read_text())
    spend(0, ['hold-end', (git_dir / 'pre-pr-passed').exists()])
    sys.exit(0)
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
    if args[:1] == ['--again']:
        original = Path(args[1])
        assert (original / 'results.json').is_file(), 'first run still exists'
        assert args[2] == '--out'
        directory = Path(args[3])
        assert not directory.exists() and directory.parent == original.parent
        binary, *names = args[4:]
        ledger = state.get('head', {})
        target = next(t for t in ledger['tests'] if t['binary'] == binary and t['name'] == names[-1])
        assert names == target.get('before', []) + [target['name']]
        spend(100, ['again', args])
        directory.mkdir(mode=0o700)
        outcome = ledger['again'][target['name']]
        (directory / 'results.json').write_text(json.dumps({'tests': [
            dict(binary_id=binary, test_name=target['name'], outcome=outcome)]}))
        sys.exit(0 if outcome == 'passed' else 100)
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
    spend(0, ['one-process-end'])
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
    command_with_hold(root, program, args, None)
}

fn command_with_hold(root: &Path, program: &str, args: &[&str], hold: Option<&str>) -> Output {
    let mut command = Command::new(program);
    command
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
        .env("GIT_CONFIG_GLOBAL", "/dev/null");
    if let Some(prefix) = hold {
        command.env("TIERS_HOLD", prefix);
    }
    command.output().expect("start fixture command")
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
    for path in [
        ".temper/pre-pr",
        "tools/pre-pr/pre_pr.py",
        "tools/tiers/run",
        "tools/citations/citations.py",
        "tools/citations/sized.txt",
    ] {
        if let Ok(source) = fs::read_to_string(workspace.join(path)) {
            write(root, path, &source);
        }
    }
    for name in ["cargo", "rustfmt", "git", "hold"] {
        write(root, &format!(".git/bin/{name}"), PROGRAM);
        fs::set_permissions(root.join(format!(".git/bin/{name}")), fs::Permissions::from_mode(0o700))
            .expect("executable check");
    }
    write(root, "tools/tiers/packages.toml", "[test-binaries]\n");
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
    write(root, "temper/docs/design/first.md", "# Design addition\n");
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

#[test]
fn failed_or_interrupted_checks_never_pass() {
    let scratch = fixture();
    let root = scratch.path();
    for fault in ["format", "interrupt", "temper"] {
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
    let cached = deliver(root);
    assert!(cached.status.success(), "identical passed tree");
    assert!(text(&cached).contains("passed before"));
    assert_eq!(budget(root)["events"], json!([]));
    assert_eq!(budget(root)["ms"], 0);
    write(root, "temper/source-3.rs", "fn changed() {}\n");
    assert!(!deliver(root).status.success(), "changed source needs a new check");
    coverage(root);
}

#[test]
fn a_flake_reports_its_first_log_and_observed_reds() {
    let scratch = fixture();
    let root = scratch.path();
    pin(root);
    write(root, "a/src/lib.rs", "fn changed() {}\n");
    write(root, ".git/temper/reds-wait", "temper-reds-wait 1 1\n");
    write(root, ".git/temper/rerun", "temper-rerun 1 4\n");
    write(root, ".git/temper/reds", "temper-reds 1\n- test A::rules red\n- test A::rules green\n");
    let flake = changed_field(test_failure("flake", "first log\n\nunder load"), "before", json!(["setup"]));
    ledger(
        root,
        &json!({"head": {"tests": [flake, test_failure("red", "inherited"),
            changed_field(test_failure("green", ""), "outcome", json!("passed"))],
            "again": {"flake": "passed"}}}),
    );
    let output = deliver(root);
    let log = text(&output);
    assert!(output.status.success(), "flake refuses nothing: {log}");
    assert!(
        log.contains("flake: test A::rules flake (failed under load; passed when its process ran again: 2 tests,"),
        "{log}"
    );
    assert!(log.contains("pre-pr: tests: inherited,"), "{log}");
    let state = budget(root);
    let again: Vec<_> = state["events"].as_array().expect("events").iter().filter(|e| e[0] == "again").collect();
    assert_eq!(again.len(), 1, "goal reds must not run again: {state}");
    let args = again.first().expect("one again")[1].as_array().expect("arguments");
    assert_eq!(&args[4..], &[json!("A::rules"), json!("setup"), json!("flake")]);
    let record = fs::read_to_string(root.join(".git/pre-pr-passed")).expect("passed record");
    let fields: Vec<_> = record.split_whitespace().collect();
    let tree = fields.first().expect("tree");
    let base = fields.get(1).expect("base");
    let results = fs::read_to_string(root.join(".git/temper/check-results")).expect("check results");
    assert_eq!(
        results,
        format!(
            "temper-check 1\ncheck {tree} {base} passed\npassed test A::rules green\nfailed test A::rules red\nflake test A::rules flake\n|\n| first log\n|\n| under load\n"
        )
    );
    assert!(!root.join(".git/temper/check-results.tmp").exists());
    fs::remove_file(root.join(".git/temper/check-results")).expect("worker consumes results");
    let cached = deliver(root);
    assert!(cached.status.success(), "{}", text(&cached));
    assert!(text(&cached).contains("passed before"));
    assert!(!root.join(".git/temper/check-results").exists(), "cache hit observes nothing");
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
    let scratch = fixture();
    let root = scratch.path();
    let ancestor = String::from_utf8(git(root, &["rev-parse", "HEAD"])).expect("commit").trim().to_owned();
    let other = String::from_utf8(git(root, &["commit-tree", "HEAD^{tree}", "-m", "pin", "-p", &ancestor]))
        .expect("other commit")
        .trim()
        .to_owned();
    git(root, &["update-ref", "refs/temper/landing", &other]);
    git(root, &["commit", "--allow-empty", "-qm", "head"]);
    write(root, ".git/MERGE_HEAD", &other);
    write(root, "temper/docs/addition.md", "# Delivery\n");
    let output = deliver(root);
    let log = text(&output);
    assert!(output.status.success(), "{log}");
    assert!(build_events(root).is_empty(), "docs build nothing");
    assert!(log.contains("docs only: citations"), "{log}");
    let state = budget(root);
    let events = state["events"].as_array().expect("events");
    assert!(events.iter().all(|event| event[0] != "format"));
    assert!(log.contains(&format!("base {other}, merge base of refs/temper/landing and HEAD")), "{log}");
    assert!(
        events.contains(&json!(["git", ["merge-base", "refs/temper/landing", "HEAD", "MERGE_HEAD"]])),
        "merge command: {state}"
    );
}

#[test]
fn the_sets_are_changed_packages_dependents_and_path_readers() {
    let scratch = fixture();
    let root = scratch.path();
    let metadata_path = root.join(".git/metadata.json");
    let mut metadata: Value = serde_json::from_slice(&fs::read(&metadata_path).expect("metadata")).expect("JSON");
    metadata["packages"][1]["dependencies"] = json!([{"name": "A", "path": root.join("a"), "kind": null}]);
    metadata["packages"][2]["dependencies"] = json!([{"name": "B", "path": root.join("b"), "kind": null}]);
    fs::write(metadata_path, metadata.to_string()).expect("dependencies");
    pin(root);
    write(root, "a/src/lib.rs", "fn changed() {}\n");
    let output = deliver(root);
    let log = text(&output);
    assert!(output.status.success(), "{log}");
    let events = build_events(root);
    let lints: Vec<_> = events.iter().filter(|e| e[0] == "clippy").map(|e| e[2].clone()).collect();
    let tests: Vec<_> = events.iter().filter(|e| e[0] == "one-process").map(|e| e[2].clone()).collect();
    assert_eq!(lints, vec![json!(["A"])], "{log}");
    assert_eq!(tests, vec![json!(["A", "B"])], "{log}");
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
fn only_what_the_delivery_broke_refuses() {
    let scratch = fixture();
    let root = scratch.path();
    pin(root);
    write(root, "a/src/lib.rs", "fn changed() {}\n");
    write(root, ".git/temper/reds", "temper-reds 1\n- lint A clippy::rule a/src/lib.rs\n");
    ledger(
        root,
        &json!({"head": {"clippy": [changed_field(lint_failure(20, "lint fault"), "code", json!("clippy::other"))]}}),
    );
    let output = deliver(root);
    let log = text(&output);
    assert_eq!(output.status.code(), Some(1), "{log}");
    assert!(log.contains("clippy A clippy::other a/src/lib.rs"), "{log}");
    assert!(!passed(root), "cache only passes");
    assert!(
        build_events(root).iter().any(|e| e[0] == "one-process" && e[1] == "head"),
        "tests run even after clippy fails"
    );
    for event in budget(root)["events"].as_array().expect("events") {
        let side = if event[0] == "metadata" { &event[2] } else { &event[1] };
        assert_ne!(side, "base", "no base cargo: {event}");
    }
    assert!(!root.join(".git/temper-pre-pr/base").exists(), "no base worktree");
    assert!(!log.contains("pre-pr: known reds"), "no tree list read: {log}");
    assert!(!log.contains("inherited: build"), "build always refuses: {log}");
    assert!(log.find("last streamed line:").expect("streamed tools") < log.rfind("pre-pr: refused").expect("summary"));
}

#[test]
fn one_hold_spans_the_check_but_not_a_cache_hit() {
    let scratch = fixture();
    let root = scratch.path();
    pin(root);
    write(root, "a/src/lib.rs", "fn changed() {}\n");
    let output = command_with_hold(root, "sh", &[".temper/pre-pr"], Some(".git/bin/hold"));
    assert!(output.status.success(), "{}", text(&output));
    let state = budget(root);
    let events: Vec<_> =
        state["events"].as_array().expect("events").iter().filter(|e| e[0] != "git").cloned().collect();
    assert_eq!(events.first(), Some(&json!(["hold-start", []])));
    assert_eq!(events.get(1).expect("first cargo")[0], "metadata");
    assert_eq!(events.last(), Some(&json!(["hold-end", true])));
    assert!(
        events.iter().position(|e| e[0] == "one-process-end").expect("tests returned") < events.len().saturating_sub(1)
    );
    assert_eq!(events.iter().filter(|e| e[0] == "hold-start").count(), 1);
    reset(root, "format");
    let cached = command_with_hold(root, "sh", &[".temper/pre-pr"], Some(".git/bin/hold"));
    assert!(cached.status.success(), "{}", text(&cached));
    assert!(budget(root)["events"].as_array().expect("cached events").iter().all(|e| e[0] == "git"));
}

#[test]
fn an_early_hold_exit_warns_without_changing_the_verdict() {
    let scratch = fixture();
    let root = scratch.path();
    pin(root);
    write(root, "a/src/lib.rs", "fn changed() {}\n");
    let output = command_with_hold(root, "sh", &[".temper/pre-pr"], Some(".git/bin/hold fail"));
    assert!(output.status.success(), "{}", text(&output));
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(stdout.matches("pre-pr: hold:").count(), 1, "{stdout}");
    assert!(stdout.contains("; going on unheld"), "{stdout}");
    let state = budget(root);
    let events: Vec<_> =
        state["events"].as_array().expect("events").iter().filter(|e| e[0] != "git").cloned().collect();
    assert_eq!(events.first(), Some(&json!(["hold-start", ["fail"]])));
    assert_eq!(events.get(1).expect("cargo after failed hold")[0], "metadata");
}
