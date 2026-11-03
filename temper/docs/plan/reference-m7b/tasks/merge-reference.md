+++
id = "merge-reference"
part = "merge-reference"
kind = "feature"
needs = ["merge-settings"]
files = [".temper/merge", "tools/merge/**", "temper/docs/examples/reference/landing.json", "temper/docs/examples/reference/settings.json"]
effort = "high"
minutes = 100
+++

## Goal

The reference's merge job (`temper/docs/design/protocol/configuration.md, 13.9`): fabrig's own
merge command, `.temper/merge`, which recounts a counted baseline left in conflict and leaves
every other conflict (`temper/docs/design/protocol/tooling.md, 4.1`); the reference's
`landing.json` naming it as the project's `merge_job`; and charter 5's catch-ups in the
reference's `settings.json`.

Acceptance:

- `.temper/merge`, executable, run as `home/.temper/merge merge <request.json>` from a job's
  directory whose `home/` is a git work tree in a merge in progress (tooling.md 4.1: no
  environment but `PATH=/usr/bin:/bin`, `HOME`, `LANG`, `TEMPER_JOB`): for each unmerged
  `<tree>/scripts/old-forms.txt` (`<tree>` one of `skein`, `smith`, `jig`, `temper`), it writes
  the head's side (index stage 2) with each form's count the larger of stages 2 and 3, runs
  `python3 -I -B scripts/old_forms.py --write` in `<tree>`, and stages the file; a baseline
  whose stage 3 holds a `# Raised:` line stage 2 lacks, or whose recount fails, is left. It
  writes `$TEMPER_JOB/result.json` as tooling.md 4.1 has it (`settled`, and `left` for every
  other unmerged path and each baseline it left) and exits 0 when nothing is left, 3 otherwise;
  it never commits, aborts the merge or touches a file that is not unmerged;
- `tools/merge/test_merge.py` (unittest, run as `python3 -I -B tools/merge/test_merge.py`)
  builds scratch repositories in a temporary directory and shows: a conflict in
  `temper/scripts/old-forms.txt` alone is settled, exit 0, the file staged with the recount and
  no file left unmerged; that conflict beside a conflicting `temper/x.rs` settles the baseline,
  leaves `temper/x.rs`, exit 3; a conflict in another file alone is left, exit 3, the work tree
  unchanged; the result files are as specified;
- `temper/docs/examples/reference/landing.json`'s project gains `"merge_job": ".temper/merge"`,
  and the test that reads it (`temper/crates/temper-service/src/tests/reference_landing.rs`)
  passes, asserting the project's `merge_job`;
- charter 5's instructions in `temper/docs/examples/reference/settings.json` gain the paragraph
  below, and the test that decodes the file (`temper/crates/temper/src/tests/reference_settings.rs`)
  passes.

## Inputs

- `temper/docs/design/protocol/tooling.md, 4.1` and `2`; `temper/docs/design/protocol/configuration.md,
  13.9` and `3` (`merge_job`); `temper/docs/design/domain/forge.md`, section 10 ("A catch-up").
- `tools/old_forms/README.md` (how the baselines are counted and written).
- `temper/docs/plan/reference-m7b/notes/common.md`.

## Owns and may change

- `.temper/merge` (a POSIX `sh` script that `exec`s `python3 -I -B` on `tools/merge/merge.py`,
  found relative to its own path), `tools/merge/merge.py`, `tools/merge/test_merge.py` and
  `tools/merge/README.md` (a few lines: what it is, how it is run, how it is tested). Python's
  standard library and `git` alone; `subprocess` with argument lists, never a shell.
- `temper/docs/examples/reference/landing.json`, `settings.json` (the paragraph alone), and the
  two tests that read them, their assertions only.
- Charter 5's paragraph, verbatim, inserted after the paragraph that begins "On a change's
  result," (the JSON string's `\n\n` between paragraphs):

  > When main moves under your goal, you hear a landing into main that touches your goal's
  > open work as news, at most once in 30 minutes. Catch up with one `change`: `"landing":
  > "goal"`, `"merge"` the landing's merge commit, its words naming the landing and why your
  > goal needs it now. Keep one catch-up live at a time: a later landing's commit carries the
  > earlier ones, so on news while one is live, wait for its result. On your first run, once
  > your first batch is made, subscribe with `subscribe_forge` to the landings on main in your
  > repository.

  If the settings test shows that charter 5 is not offered `subscribe_forge`, keep the
  paragraph and list it under `## Left` (the charter's tools are the reference walk's).
- Not this task's: `.temper/pre-pr`, any Rust beyond the two tests' assertions.

## Steps

1. `tools/merge/merge.py` and `.temper/merge`; the unit tests (they run `git` in temporary
   directories alone, never in the worktree).
2. The two example files; their tests.
3. `heavy python3 -I -B tools/merge/test_merge.py`;
   `heavy cargo nextest run -p temper-service --lib -E 'test(/reference_landing/)'`;
   `heavy cargo nextest run -p temper --lib -E 'test(/reference_settings/)'`; `cargo fmt`;
   `python3 -I -B temper/scripts/old_forms.py`; clippy on `temper-service` and `temper` only if
   their Rust changed.
4. Commit and push (`.temper/merge` with its executable bit).

## Report

As the implementer's text says.
