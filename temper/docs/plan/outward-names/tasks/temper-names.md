+++
id = "temper-names"
part = "temper"
kind = "feature"
needs = []
files = [
  "temper/scripts/proof-live.sh",
  "temper/scripts/proof-live-github.sh",
  "temper/scripts/proof-live-github-managed.sh",
  "temper/tests/real-loop/tests/fixtures/b0/regenerate.sh",
  "temper/tests/real-loop/tests/fixtures/b1/regenerate.sh",
  "temper/crates/temper/tests/suite/forge_land.rs",
  "temper/crates/temper/tests/suite/startup.rs",
  "temper/crates/temper/tests/support/live.rs",
  "temper/crates/temper/tests/support/cluster.rs",
  "temper/tests/client-browser/src/lib.rs",
  "temper/tests/codecs/tests/suite/drift.rs",
  "temper/tools/twin/README.md",
  "temper/docs/development/debt/D008-pipeline-names-in-code.md",
  "temper/docs/development/playground-memory.md",
  "temper/docs/development/pre-pr-numbers.md",
]
effort = "high"
minutes = 80
+++

# temper's scripts, tests and documents stop naming the lab

## Goal

fabrig names nothing of whoever runs its development (`AGENTS.md`, 1). temper's live-proof
scripts look for the lab's build tool on `PATH` and default to paths under its owner's home; the
fixtures' regenerators default to a scratch path; tests stub the tool by name; hint strings,
messages, a README and three development notes show its commands, machine or pipeline. The
scripts run plain cargo, or the wrapper their caller names in `TEMPER_WRAPPER`; the regenerators
take their data directory as a required argument; the rest says it plainly. Nothing else changes
what it does.

Acceptance:

- The five scripts follow `notes/contracts.md`, 1 (`TEMPER_WRAPPER`) and the two regenerators
  `notes/contracts.md`, 2. No script holds `command -v heavy`, `$heavy` or a `/home/free/` path.
  `sh -n` passes on each; `shellcheck` too, with no new finding, where installed.
- `forge_land.rs`'s launcher tests stub `cargo` where they stubbed `heavy`, assert cargo's own
  arguments, and one new test shows a wrapper from `TEMPER_WRAPPER` runs the builds (`## Steps`,
  3); `heavy cargo nextest run -p temper --test suite -E 'test(/^forge_land::.*launcher/)'`
  passes, its count at least the base's plus one.
- Every other file of `files` changes only the words of `## Owns and may change`.
- The check of `notes/common.md` ("Checking your files are clean"), with the 13 entries of
  `## Steps`, 5 deleted, passes (1 selected); the inventory is then restored and not committed.
- `heavy cargo clippy -p temper -p temper-client-browser-world -p temper-codecs-world
  --all-targets -- -D warnings` passes; `python3 -I -B temper/scripts/old_forms.py` raises no
  count; `python3 -I -B tools/citations/citations.py --all` prints `citations: 0 broken in HEAD`.

## Inputs

- `temper/docs/plan/outward-names/notes/common.md`: the standing rules.
- `temper/docs/plan/outward-names/notes/contracts.md`, 1 (the wrapper), 2 (`regenerate.sh DATA`)
  and 3 (the wording).
- `temper/docs/plan/outward-names/inputs/request.md`: the request (How, Bounds).
- `temper/docs/design/domain/testing.md`, 2.7: the person's sitting the launcher runs (read
  only; it names no build tool).
- `tests/boundary/tests/outward.rs`: the check.

## Owns and may change

- **`temper/scripts/proof-live.sh`:** ~l.6-7 (the "build queue" comment and `heavy=$(command -v
  heavy || true)`) → the comment of `notes/contracts.md`, 1 for `TEMPER_WRAPPER`. The
  `rung-b0-offline` branch (~l.12-13) loses its "heavy is required" refusal and execs
  `${TEMPER_WRAPPER:+"$TEMPER_WRAPPER"} cargo nextest run ...` with the same arguments. ~l.266
  and ~l.281: `$heavy cargo` → `${TEMPER_WRAPPER:+"$TEMPER_WRAPPER"} cargo`, the `TMPDIR=$scratch`
  prefix and every argument as they are.
- **`temper/scripts/proof-live-github.sh`:** the same at ~l.6-7, ~l.18 and ~l.187.
- **`temper/scripts/proof-live-github-managed.sh`:** the same at ~l.8-9, ~l.56, ~l.74 and ~l.79;
  ~l.16 `CARGO_TARGET_DIR=/home/free/src/rust/tmp/owners/github-managed/target` →
  `CARGO_TARGET_DIR="$HOME/src/rust/tmp/owners/github-managed/target"` and ~l.45's live evidence
  default `/home/free/src/rust/tmp/owners/github-managed/reports/m3-conversion.json` →
  `$HOME/src/rust/tmp/owners/github-managed/reports/m3-conversion.json` (the same paths for their
  owner: behaviour holds; `notes/contracts.md`, 3).
- **`temper/tests/real-loop/tests/fixtures/{b0,b1}/regenerate.sh`:** `notes/contracts.md`, 2:
  a usage check before anything runs, `--data "$1"`, and the command as
  `${TEMPER_WRAPPER:+"$TEMPER_WRAPPER"} env TEMPER_BUNDLE=none NEXTEST_DOUBLE_SPAWN=0 cargo run -p
  temper-twin -- ...`; every `--base`, `--out`, `--cue` and `--replace` byte for byte. Their
  header comments (b1's "box3 on main 44230b8fc" included) stay.
- **`temper/crates/temper/tests/suite/forge_land.rs`:** `stubbed_story_launcher` (~l.465) and
  `held_launcher` (~l.659) write their stub as `cargo` where they wrote `heavy` (the local's name
  and the `expect` texts follow: "offline build stub", "executable build stub"); the
  `the_launcher_gives_each_landing_story_its_own_store` expectation (~l.550-575) loses its two
  `"cargo"` lines; comments that say "queue" for the stub say "build" or "cargo". `launcher()`
  (~l.244) already clears the environment: leave it. One new test, `## Steps`, 3.
- **`temper/crates/temper/tests/support/live.rs`** ~l.533: "before starting heavy" → "before its
  first build".
- **`temper/crates/temper/tests/support/cluster.rs`** ~l.26, `CGROUPS`: ": run them through
  heavy" → ": run them where it delegates them" (or words to that effect, naming no tool).
- **`temper/crates/temper/tests/suite/startup.rs`** ~l.1435,
  **`temper/tests/client-browser/src/lib.rs`** ~l.37 and
  **`temper/tests/codecs/tests/suite/drift.rs`** ~l.19: the hint's `heavy cargo` → `cargo`,
  the rest byte for byte. Other drafts change other lines of `startup.rs` and the browser shell's
  `lib.rs`: touch only these lines.
- **`temper/tools/twin/README.md`** ~l.8, ~l.76, ~l.77: `heavy env ...` → `env ...`.
- **`temper/docs/development/debt/D008-pipeline-names-in-code.md`** ~l.17: "the pre-fab
  pipeline's tasks" → "the former pipeline's tasks".
- **`temper/docs/development/playground-memory.md`** ~l.3: "on `hetzner9` (`hostname`)" → "on a
  builder"; ~l.14-15: `heavy cargo nextest` → `cargo nextest`. The date, commit and figures stay.
- **`temper/docs/development/pre-pr-numbers.md`** ~l.5: "a plan's shared branch (fab's
  `<draft>/integration`)" → "a plan's shared branch (`<draft>/integration`)".
- **If a line moved** at your base, the name is changed where it stands; a name this spec does
  not list is changed by `notes/contracts.md`, 3, and the report says where. A test elsewhere
  that turns out to stub or assert the tool's name (`rg -n heavy temper/`) follows the scripts
  the same way: change it and say so.
- Nothing else: not `tests/boundary/outward.txt`, not any `docs/design/` file (the two status
  files that name the gate list's runner are `status-words`'), not `docs/help/`.

## Steps

1. The scripts (`notes/contracts.md`, 1 and 2); `sh -n` each; `shellcheck` where installed.
   Run each regenerator with no argument and with two: both print the usage line to stderr and
   exit 2, building nothing. Run no live proof.
2. The stubs and expectations in `forge_land.rs`; then
   `heavy cargo nextest run -p temper --test suite -E 'test(/^forge_land::.*launcher/)'` and
   read its count before your change and after.
3. The new test, beside the launcher tests, in their style:

   ```rust
   /// TEMPER_WRAPPER names one command that runs each of the launcher's builds, cargo's
   /// own arguments following it.
   #[test]
   fn the_launcher_runs_its_builds_through_the_wrapper_it_is_given() {
       // In the scratch: `cargo`, a stub that exits 99 (reached only if the wrapper is
       // skipped), and `wrap`, a stub that appends "$@" to "$HOME/seen" and exits 0.
       // launcher(root, "forge-land", None) with PATH = scratch:/usr/bin:/bin and
       // TEMPER_WRAPPER=wrap.
       // Status 0; "seen" holds two runs, each starting "cargo", "nextest", "run".
   }
   ```

   Extending `stubbed_story_launcher` with an optional wrapper, or writing the setup inline, is
   your call. Run it alone, then step 2's filter again.
4. The other files' words, as `## Owns and may change`.
5. Check the files clean, as `notes/common.md` says, deleting these 13 entries of
   `tests/boundary/outward.txt` in the working tree: `temper/crates/temper/tests/suite/forge_land.rs`,
   `temper/crates/temper/tests/suite/startup.rs`,
   `temper/docs/development/debt/D008-pipeline-names-in-code.md`,
   `temper/docs/development/playground-memory.md`, `temper/docs/development/pre-pr-numbers.md`,
   the three `temper/scripts/proof-live*.sh`, `temper/tests/client-browser/src/lib.rs`,
   `temper/tests/codecs/tests/suite/drift.rs`, the two `regenerate.sh` and
   `temper/tools/twin/README.md`. Restore it.
6. Tests: step 2's filter; `heavy cargo nextest run -p temper --test suite -E
   'test(=startup::turn_bounds)'`; `heavy cargo nextest run -p temper-codecs-world --test suite
   -E 'test(/^drift::/)'`; `heavy cargo check -p temper-client-browser-world --tests` (its tests
   need a browser). `cluster.rs` and `live.rs` compile with the suite. Check each count is above
   zero.
7. `cargo fmt`; clippy as in Acceptance; old forms; commit; citations; push.

## Report

As the implementer's text says, and: each script's diff in a line or two; the regenerators'
usage runs and exit codes; `sh -n` and `shellcheck` (or that it is not installed); the launcher
filter's counts before and after; the check of step 5 and its selected count; the other test
counts; anything under `## Left`.
