+++
id = "replacement-wait"
part = "replacement-wait"
kind = "feature"
needs = []
files = ["temper/tests/real-loop/tests/suite/github_repair.rs"]
effort = "medium"
minutes = 40
+++

# The replacement github_repair story waits for hold 2

## Goal

`github_repair::a_replacement_github_repair_after_release_receives_the_job_log_again`
(`temper-real-loop-world`, binary `suite`) ends its story on a `change show --until held`
that can match hold 1, which the panel still shows after `task release 2 --hold 1`, before
the replacement repair has run. Under load the story then ends with "story completion
obligations: undeclared engine task 1: Running". It has rejected trunks in the queue. Bring
M7a's fix (`063edf0d7e`) to main unchanged: the last wait becomes an `until` on
`task release 2 --hold 2;`, the hold the replacement's `no_change` verdict creates.

Acceptance:

- The branch holds one commit over its base, `063edf0d7e` cherry-picked: it changes only
  `temper/tests/real-loop/tests/suite/github_repair.rs`, 8 lines in and 2 out (the `until`
  import and `replacement_acts`), and the file's blob after it is `9610423b40`, as on
  `reference-m7a/repair-1`.
- The test passes alone five times, and in each of three runs of the `suite` binary, each
  run a separate `heavy` command (Steps 4 and 5); another test's failure there is handled
  as Steps says.
- `cargo fmt` changes nothing; clippy on `temper-real-loop-world` is clean; temper's
  old-form counts are unchanged.

## Inputs

- `temper/docs/plan/github-repair-race/notes/common.md`.
- `temper/docs/plan/github-repair-race/inputs/request.md` and `inputs/fix.patch` (the commit,
  as a patch).
- `temper/docs/design/domain/testing.md, 2.4`, the real loop: what the suite's stories run
  on. Nothing in it changes.
- `temper/tests/real-loop/src/calls.rs`: `until(at, argv, holds, early)`, which the fix
  imports; it exists on main unchanged.

## Owns and may change

Owns `temper/tests/real-loop/tests/suite/github_repair.rs`, and only the two hunks of
`063edf0d7e`. Nothing else, in any tree.

Departure from `AGENTS.md` 6, stated: the request asks for the same test to run several
times unchanged, to show the race gone under load; Steps 4 and 5 are those runs, and are not
unchanged reruns of a failure.

Pre-approved, with defaults:

- If `063edf0d7e` is not in the slot, `git fetch origin reference-m7a/repair-1` and pick it
  again. If that branch is gone from origin, apply `inputs/fix.patch` with `git apply` and
  commit it with the same subject line, `Wait for the replacement repair verdict before
  ending its story`; say so in the report.
- If the pick conflicts (the trunk's file moved after the cut, which saw its blob at
  `a2e050d61c`), resolve it so the import line and `replacement_acts` read exactly as in
  `063edf0d7e`, keep the trunk's other lines, and report the conflict and the blob you
  reached.
- If `cargo fmt` would change anything in `github_repair.rs` (at the cut the fix's file was
  rustfmt-clean), keep the picked bytes and report it; any other file it would change is not
  yours: leave it and list it under `## Left`.
- A run that exits 137 runs once more, and that run counts in its place.

## Steps

1. From the branch's base: `git cherry-pick -x 063edf0d7e`. Check with `git show --stat HEAD`
   (one file, +8 -2) and `git rev-parse --short=10
   HEAD:temper/tests/real-loop/tests/suite/github_repair.rs` (`9610423b40`).
2. `cargo fmt`, then `git status` shows nothing changed;
   `python3 -I -B temper/scripts/old_forms.py` shows temper's counts unchanged (no
   `--write`).
3. `heavy cargo clippy -p temper-real-loop-world --all-targets -- -D warnings`.
4. The test alone, five times, one command at a time:

   ```
   heavy cargo nextest run -p temper-real-loop-world --test suite \
     -E 'test(=github_repair::a_replacement_github_repair_after_release_receives_the_job_log_again)'
   ```

   Each run selects 1 test and passes (a run selecting 0 proves nothing: fix the filter).
5. The suite, three times, one command at a time:
   `heavy cargo nextest run -p temper-real-loop-world --test suite` (the focused profile:
   the binary less its `fuzzy` module). This test passes in each.
6. Push your branch: `git push origin HEAD:refs/heads/<your branch>`.

When a run fails, read its log (heavy prints its path) before anything else runs:

- This test fails, in any run, with any message: stop. Change the test no further; push
  the picked commit and report `Question:` with the failure's lines, the log's path, the
  machine and the run's number. Options: (a) land the pick as it is, as it narrows the race;
  (b) a further wait, which is a separate task agreed with M7a's trunk so that both sides keep
  one change. Recommend (a) unless the log shows the story ending before hold 2 again.
- Another test fails (one this change cannot reach): run it once alone
  (`-E 'test(=NAME)'`). Passing, it is a flake: under `## Left`, with both logs and the
  machine. Failing alone, it is an inherited red: under `## Left`, with the log; it does not
  stop this task's acceptance, which is Step 4 and this test passing in Step 5.
- A test marked SLOW or a starved warning is no failure: give its time in the report.

## Report

`Done:` (or `Question:`) first. Then: the commit pushed (hash) and its `--stat`, the blob
reached; for each of the eight runs its command, machine, tests selected and passed, and wall
time; any failure's log path; what fmt, old-forms and clippy said; implementation minutes
and check minutes apart.
