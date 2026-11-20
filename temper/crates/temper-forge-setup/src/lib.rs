//! Forge setup, checked before any command acts (protocol/setup/init.md and protocol/setup/README.md, 7).
#![cfg_attr(not(test), no_std)]
#![forbid(unsafe_code)]
extern crate alloc;
use alloc::boxed::Box;
pub mod check;
pub mod fetch;
pub mod given;
pub mod ini;
pub mod limits;
pub mod machine;
pub mod name;
pub mod parent;
pub mod place;
pub mod prepare;
pub mod problem;
pub mod run;
pub mod runner;
pub mod setup;
pub mod setup_file;
pub mod sign_in;
pub mod spec;
pub mod state;
pub mod step;
pub mod upgrade;
pub use setup::Setup;
pub use step::{FORGE_ARGUMENT, Group, Kind, Stage, Step, Steps, kinds};
#[cfg(test)]
mod tests;

/// The first repository's starter CI (protocol/setup/owned.md, 7).
pub const STARTER_PATH: &[u8] = b".forgejo/workflows/ci.yml";
pub const STARTER_WORKFLOW: &[u8] = b"on:\n  push:\n    branches: [main]\n  pull_request:\njobs:\n  check:\n    runs-on: LABEL\n    steps:\n      - run: echo \"temper's starter check passed; replace this step with your own checks\"\n";

/// The sample's greeting and checks (protocol/setup/owned.md, 7).
pub const SAMPLE_HELLO_PATH: &[u8] = b"hello";
pub const SAMPLE_HELLO: &[u8] = br#"#!/bin/sh
# Prints a greeting, to the name given or to the world: sh hello [NAME]
printf 'Hello, %s!\n' "${1:-world}"
"#;
pub const SAMPLE_TEST_PATH: &[u8] = b"test.sh";
pub const SAMPLE_TEST: &[u8] = br#"#!/bin/sh
# Checks hello's greetings: sh test.sh
status=0
check() {
    expected=$1
    shift
    actual=$(sh ./hello "$@")
    if [ "$actual" = "$expected" ]; then
        echo "ok: $expected"
    else
        echo "failed: expected '$expected', got '$actual'"
        status=1
    fi
}
check 'Hello, world!'
check 'Hello, Ada!' Ada
exit "$status"
"#;
/// The sample's CI, rendered on the forge's runner label.
pub const SAMPLE_WORKFLOW: &[u8] = br#"on:
  push:
    branches: [main]
  pull_request:
jobs:
  check:
    runs-on: LABEL
    steps:
      - name: Check out
        env:
          TOKEN: ${{ github.token }}
        run: |
          git init -q .
          export GIT_CONFIG_COUNT=1 GIT_CONFIG_KEY_0=http.extraHeader GIT_CONFIG_VALUE_0="Authorization: token $TOKEN"
          git fetch -q --depth 1 "${GITHUB_SERVER_URL%/}/$GITHUB_REPOSITORY.git" "$GITHUB_SHA"
          git checkout -q FETCH_HEAD
      - name: Test
        run: sh test.sh
"#;

/// Render only a wire-sized runner label; its alphabet cannot add YAML syntax.
#[must_use]
pub fn starter_workflow(label: &[u8]) -> Option<Box<[u8]>> {
    workflow(STARTER_WORKFLOW, label)
}

/// Render the sample with the same checked runner label as the starter.
#[must_use]
pub fn sample_workflow(label: &[u8]) -> Option<Box<[u8]>> {
    workflow(SAMPLE_WORKFLOW, label)
}

fn workflow(template: &[u8], label: &[u8]) -> Option<Box<[u8]>> {
    if label.is_empty() || label.len() > 64 {
        return None;
    }
    for byte in label {
        if !byte.is_ascii_alphanumeric() && !b"._-".contains(byte) {
            return None;
        }
    }
    let at = skein_lib::bytes::find(template, b"LABEL").expect("workflow template label");
    let after = at.checked_add(5).expect("template offset");
    let size = template.len().checked_sub(5)?.checked_add(label.len())?;
    let mut writer = skein_lib::Writer::new(size);
    writer.put(template.get(..at)?).ok()?;
    writer.put(label).ok()?;
    writer.put(template.get(after..)?).ok()?;
    Some(writer.finish())
}

/// The playground source shared by its fake seed and its owned copy (protocol/setup/owned.md, 7).
pub const PLAYGROUND_PATH: &[u8] = b"src/main.rs";
/// Widget prints the playground table.
pub const PLAYGROUND_MAIN: &[u8] = b"const TABLE: [(&str, u32); 3] = [(\"apples\", 3), (\"pears\", 5), (\"plums\", 2)];\n\nfn main() {\n    for (name, count) in TABLE {\n        println!(\"{name:<8}{count}\");\n    }\n}\n";
