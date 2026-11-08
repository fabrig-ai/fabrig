//! Delivery checks keep their 60-second budget without dropping formatting or baselines
//! (temper/docs/design/protocol/configuration.md, 11 and 13.9).

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

root = Path.cwd()
while not (root / '.git').is_dir():
    root = root.parent
state_path = root / '.git/budget.json'
state = json.loads(state_path.read_text())
name = Path(sys.argv[0]).name
args = sys.argv[1:]
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
metadata = json.loads((root / '.git/metadata.json').read_text())
if name == 'cargo' and args[0] == 'metadata':
    spend(1000, ['metadata', args])
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
    for path in [".temper/pre-pr", "tools/pre-pr/pre_pr.py"] {
        if let Ok(source) = fs::read_to_string(workspace.join(path)) {
            write(root, path, &source);
        }
    }
    for name in ["cargo", "rustfmt"] {
        write(root, &format!(".git/bin/{name}"), PROGRAM);
        fs::set_permissions(root.join(format!(".git/bin/{name}")), fs::Permissions::from_mode(0o700))
            .expect("executable check");
    }
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
            json!({"edition": "2024", "targets": [{
                "src_path": root.join(path), "edition": edition, "kind": ["bin"]
            }]})
        })
        .collect();
    write(root, ".git/metadata.json", &json!({"packages": packages}).to_string());
    git(root, &["add", "."]);
    reset(root, "");
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
}

#[test]
fn private_index_checks_new_sources_without_staging_them() {
    let scratch = fixture();
    let root = scratch.path();
    let index = fs::read(root.join(".git/index")).expect("real index");
    write(root, "temper/delivery.rs", "fn delivered() {}\n");
    let output = deliver(root);
    assert!(output.status.success(), "baseline must see new source: {}", String::from_utf8_lossy(&output.stderr));
    assert!(fs::read(root.join(".git/index")).expect("index after checks") == index, "real index changed");
}
