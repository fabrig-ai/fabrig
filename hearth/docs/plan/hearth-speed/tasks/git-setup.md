+++
id = "git-setup"
part = "git-setup"
kind = "feature"
needs = []
files = ["hearth/testing/hearth-test-repository/src/lib.rs"]
effort = "medium"
minutes = 15
+++

# A scratch repository skips what no test reads

## Goal

`Repository::make` spawns `git init`, `git add` and `git commit`, and `git commit` spawns `git maintenance run --auto` behind it. `git init` copies the template's hooks. No test reads either, and git's real work on the repository is the same without them. On one machine, in a scratch directory (`git` 2.47.3, 100 repositories of `THIN`'s size), the three cost 8.0 ms and without them 5.5: about 0.5 CPU s over the real-git binary's 183 tests, and more with `next`.

Acceptance:

- `Repository::make_on`'s `git init` passes `--template=`.
- `git_output`'s environment turns off automatic maintenance: `GIT_CONFIG_COUNT=1`, `GIT_CONFIG_KEY_0=maintenance.auto`, `GIT_CONFIG_VALUE_0=false`.
- Every test of `hearth-builder-world` and of `hearth` passes unchanged. Commit ids don't change, and `a_scratch_repository_without_git_on_the_path_fails_by_name_not_by_skip` still panics by that name.
- The commit gives the method's figures, or the real-git binary's alone if the suite's noise hides the saving (labelled as that), before and after, on one machine.

## Inputs

- `hearth/docs/plan/hearth-speed/notes/common.md`
- `hearth/docs/design/domain/testing.md`, 9.1: real git on tiny repositories
- `hearth/testing/hearth-test-repository/src/lib.rs`

## Owns and may change

- **Owns:** `hearth/testing/hearth-test-repository/src/lib.rs`.
- **Searched, for you:** nothing under `hearth/tests/`, `hearth/testing/` or `hearth/crates/hearth/` reads `hooks`, `info/exclude`, `.git/description`, `gc.auto` or `maintenance`. `hearth/tests/builder/src/fixtures.rs`'s `work_tree` also runs `git init`. It stands for a person's containing work tree, so it keeps its template; it gets the maintenance setting through `git_output`. `hearth/tests/conformance/` runs git by its own path, is outside the gate, and doesn't change.
- **If a test does need the template or maintenance:** drop that half, keep the other, and say which test.
- **What hearth spawns isn't touched:** this is the fixture's environment, and the code under test builds its own.
- **Departs from your text:** adds no test, so no bite: it changes no rule, and the tests it passes are the check.

## Steps

1. Before: the base commit's figures (`notes/common.md`).
2. The two edits.
3. After: the same figures, on the same machine.
4. `heavy cargo nextest run -p hearth-test-repository -p hearth-builder-world -p hearth`. The end of every task in `notes/common.md`.
5. Commit with the saving, and push.

## Report

The figures before and after, the machine, and the tests that ran.
