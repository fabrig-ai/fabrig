mod fetch;
mod files;
mod fixtures;
mod given;
mod golden;
mod limits;
mod machine;
mod machine_config;
mod parent;
mod prepare;
mod run;
mod setup_file;
mod sign_in;
mod spec;
mod upgrade;

#[test]
fn starter_workflow_is_pinned_to_the_setup_design() {
    assert_eq!(
        crate::starter_workflow(b"temper-host").unwrap().as_ref(),
        b"on:\n  push:\n    branches: [main]\n  pull_request:\njobs:\n  check:\n    runs-on: temper-host\n    steps:\n      - run: echo \"temper's starter check passed; replace this step with your own checks\"\n"
    );
    assert_eq!(crate::STARTER_PATH, b".forgejo/workflows/ci.yml");
}

#[test]
fn sample_files_are_pinned_to_the_setup_design() {
    assert_eq!(crate::SAMPLE_HELLO_PATH, b"hello");
    assert_eq!(
        crate::SAMPLE_HELLO,
        br#"#!/bin/sh
# Prints a greeting, to the name given or to the world: sh hello [NAME]
printf 'Hello, %s!\n' "${1:-world}"
"#
    );
    assert_eq!(crate::SAMPLE_TEST_PATH, b"test.sh");
    assert_eq!(
        crate::SAMPLE_TEST,
        br#"#!/bin/sh
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
"#
    );
    assert_eq!(
        crate::sample_workflow(b"temper-host").unwrap().as_ref(),
        br#"on:
  push:
    branches: [main]
  pull_request:
jobs:
  check:
    runs-on: temper-host
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
"#
    );
}

#[test]
fn sample_workflow_checks_the_runner_label() {
    for label in [b"".as_slice(), b"two labels", &[b'x'; 65], b"line\nbreak", b"bad:label"] {
        assert_eq!(crate::sample_workflow(label), None);
        assert_eq!(crate::starter_workflow(label), None);
    }
    for label in [b"docker".as_slice(), b"host_1.test-2", &[b'x'; 64]] {
        let sample = crate::sample_workflow(label).unwrap();
        assert!(skein_lib::bytes::find(&sample, label).is_some());
        assert!(skein_lib::bytes::find(&sample, b"LABEL").is_none());
    }
}
