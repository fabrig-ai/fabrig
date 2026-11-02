+++
id = "pair-stories"
part = "live-pair"
kind = "feature"
needs = []
files = [
  "temper/crates/temper/tests/live.rs",
  "temper/crates/temper/tests/support/forge_land.rs",
  "temper/crates/temper/tests/suite/forge_land.rs",
  "temper/scripts/proof-live.sh",
  "temper/scripts/proof-live-github.sh",
]
effort = "high"
minutes = 70
+++

# The landing pair runs in one invocation: a store per story, Claude on haiku

## Goal

After every landing fab will run the two owned-forge landing stories in one nextest
invocation with one fresh `TEMPER_LIVE_STORE`. Today both stories take that directory as
their store, so the second would reuse the first's deployment; and the Claude story runs on
`sonnet`, not the cheapest model. Give each story its own store under `TEMPER_LIVE_STORE`,
run the Claude story on `claude-haiku-5-5`, and keep the owner's scripts working: they name
a nextest profile, `live`, that no longer exists.

Acceptance:

- The ChatGPT story's store is `$TEMPER_LIVE_STORE/forge-land`, the Claude story's
  `$TEMPER_LIVE_STORE/forge-land-claude`, by the rules of
  `temper/docs/plan/live-temper/notes/contracts.md`, 3; the rung's store is unchanged.
- `LiveModel::ClaudeCode`'s name is `claude-haiku-5-5`; `ChatGpt`'s stays `gpt-6-luna`.
- `scripts/proof-live.sh forge-land` and `forge-land-claude` export the stores' directory
  (default `$HOME/src/rust/tmp/live-store`) as `TEMPER_LIVE_STORE` and keep their own
  checks, give-back and `kept store` line on `<directory>/<argument>`; `rung-b0` unchanged.
- Both scripts build with `--profile live-temper --no-run` and run their story with
  `--profile live-temper --ignore-default-filter` and `-E 'binary(=B) & test(=NAME)'`.
- New suite tests (below) pass; every `forge_land::` test of `temper`'s `suite` binary passes;
  `temper`'s focused tests pass; clippy is clean; shellcheck finds nothing on changed lines.
- No live story is run (notes/common.md).

## Inputs

- `temper/docs/plan/live-temper/notes/common.md` and `notes/contracts.md` (2, 3, 4).
- `temper/docs/plan/live-temper/inputs/request.md`, "One fresh store per story", "The cheap
  model for Claude", "Without a person".
- `temper/docs/design/domain/testing.md, 2.7`: the owned-forge landing story's bullet (its
  store, a kept rerun, the held record) and the Claude Code bullet. Another task of this plan
  rewrites that text; do not edit it.

## Owns and may change

Owns, all in temper:

- `crates/temper/tests/support/forge_land.rs`: `live_store` (~765-779) and a new store
  function beside it; `LiveModel`'s `impl` (~1129-1136): `name`'s Claude arm and a new
  `story`. Nothing else of the file.
- `crates/temper/tests/live.rs`: `owned_forge_landing`'s store line (67) alone.
- `crates/temper/tests/suite/forge_land.rs`: the four `"sonnet"`/`b"sonnet"` literals
  (~381, 392, 397, 431); the launcher stubs' store paths (`held_launcher`'s callers and
  `the_launcher_records_a_failed_ordinary_return_hook`, ~611-671); the two new tests.
- `scripts/proof-live.sh`: the store block (~77-101) and the two nextest lines (~263, ~278).
- `scripts/proof-live-github.sh`: its two nextest lines (~19, ~188).

Read only: `crates/temper/tests/support/forge_land/rung.rs` (calls `live_store(true)`:
unchanged, so it compiles untouched), `support/held_login.rs` (the hook's contract),
`support/mod.rs` (`Scratch`), `.config/nextest.toml` (another task narrows
`live-temper`'s filter; your scripts ignore it).

Pre-approved options:

- If `live_store` and the new function share the held-account refusal through a private
  helper, that is yours; keep `live_store(account: bool) -> (PathBuf, bool)` as it is called.
- If a launcher test other than those named breaks on the store's new path, change its
  stub's or assertion's path alone, and list it.
- If shellcheck flags a line you did not change, leave it and list it under `## Left`.
- `jig/scripts/live-sign-in.sh` names the missing `live` profile too: jig's, not yours; list
  it under `## Left`.

## Steps

1. `support/forge_land.rs`. Sketch of the outer layer (bodies yours):

   ```rust
   impl LiveModel<'_> {
       /// The story's launcher argument, and its store's name under `TEMPER_LIVE_STORE`.
       #[must_use]
       pub const fn story(self) -> &'static str {
           match self {
               Self::ChatGpt { .. } => "forge-land",
               Self::ClaudeCode { .. } => "forge-land-claude",
           }
       }
   }

   /// The owned-forge landing story's own store under `TEMPER_LIVE_STORE`, so the pair runs
   /// in one invocation, each on its own deployment (domain/testing.md, 2.7).
   #[must_use]
   pub fn landing_store(model: LiveModel<'_>) -> (PathBuf, bool);

   /// Make the stores' directory (0700, with its parents) when absent, then reserve
   /// `stores/<story>` as `reserve_live_store` does; a held ChatGPT record moves into a fresh
   /// store only.
   pub fn story_store(stores: &std::path::Path, model: LiveModel<'_>) -> Result<(PathBuf, bool), String>;
   ```

   `landing_store` reads `TEMPER_LIVE_STORE` (same `expect` text as `live_store`) and, on an
   `Err`, notices and panics as `live_store` does. `name`'s Claude arm becomes
   `"claude-haiku-5-5"`.

2. `live.rs`, line 67: `let (store, kept) = forge_land::landing_store(model);`.

3. `suite/forge_land.rs`:
   - the four `sonnet` literals become `claude-haiku-5-5` (they pin `name()`'s value through
     `configure_live_model` and the stock charters);
   - a new test beside `a_held_chatgpt_record_is_admitted_only_as_a_private_regular_file_never_named`:

     ```rust
     /// domain/testing.md, 2.7: one TEMPER_LIVE_STORE holds a store per landing story.
     #[test]
     fn each_landing_story_takes_its_own_store_under_one_live_store() {
         // stores = scratch/stores, absent.
         // ChatGpt { account: None } -> Ok((stores/forge-land, false)); stores is a 0700 dir.
         // a file written into stores/forge-land (its deployment begun)
         // ClaudeCode { login: "/held/claude-login" } -> Ok((stores/forge-land-claude, false))
         // ChatGpt again -> Err naming stores/forge-land and what it lacks (a kept store
         //   is checked, never repaired)
         // ChatGpt { account: Some(..) } on stores/forge-land -> Err (not a fresh store)
     }
     ```

   - the launcher stubs that write `$TEMPER_LIVE_STORE/give-back` (and `runner`,
     `synthetic-import`) for `forge-land` write them in `$TEMPER_LIVE_STORE/forge-land`;
     `rung-b0`'s stay where they are. `the_launcher_records_a_failed_ordinary_return_hook`'s
     retained hook is `store/forge-land/give-back`.
   - a new launcher test beside `the_launcher_runs_only_the_claude_story_then_returns`:

     ```rust
     /// The launcher exports the stores' directory and keeps its story's store beneath it.
     #[test]
     fn the_launcher_gives_each_landing_story_its_own_store() {
         // for forge-land (no account: the stub exits 0 before any sign-in) and
         // forge-land-claude (stubbed_launcher's held directory):
         //   the stub heavy appends "$TEMPER_LIVE_STORE" and "$@" to $HOME/seen;
         //   TEMPER_LIVE_STORE seen is the launcher's directory (scratch/store, resolved);
         //   stdout's `kept store` line names <that>/<argument>;
         //   the run's arguments hold --profile live-temper, --ignore-default-filter and
         //   binary(=live) & test(=<the story>).
         // and once with TEMPER_LIVE_STORE removed: the directory is
         //   $HOME/src/rust/tmp/live-store, the store $HOME/src/rust/tmp/live-store/forge-land.
     }
     ```

     If `launcher()` or `stubbed_launcher` need a variant for this (an argument, an
     `env_remove`), add it beside them.

4. `scripts/proof-live.sh`, for `forge-land` and `forge-land-claude` (contracts.md, 3):
   `stores=${TEMPER_LIVE_STORE:-$HOME/src/rust/tmp/live-store}`; `mkdir -p -- "$stores"`;
   resolve it (`readlink -f`); `store=$stores/$1`; `export TEMPER_LIVE_STORE="$stores"`. The
   rerun check, the held-account refusal, `stop_owned`, the give-back and `kept store` keep
   reading `$store`. `rung-b0` keeps `store=${TEMPER_LIVE_STORE:-$PWD/../rung-b0-...}` and
   exports `$store`. The two nextest lines:

   ```sh
   $heavy cargo nextest run -p temper --profile live-temper --no-run
   ...
   TMPDIR=$scratch $heavy cargo nextest run -p temper --profile live-temper --ignore-default-filter \
       --run-ignored only --no-capture -E "binary(=live) & test(=$test_name)" &
   ```

5. `scripts/proof-live-github.sh`: `--profile live` becomes `--profile live-temper` on both
   lines; the run gains `--ignore-default-filter` (its `-E` already names `live_github`'s test).

6. Tests: the new tests alone
   (`heavy cargo nextest run -p temper --test suite -E 'test(=forge_land::NAME)'`), then
   `-E 'test(/^forge_land::/)'` (every reader of the store and the model). `cargo fmt`;
   `heavy cargo clippy -p temper --all-targets -- -D warnings` (compiles the `live` binary);
   `shellcheck temper/scripts/proof-live.sh temper/scripts/proof-live-github.sh`;
   `heavy cargo nextest run -p temper` (the focused profile: never the live binaries).

7. Commits: the store and its tests; the model; the scripts. Push your branch.

## Report

As the implementer's text says, and also: the stores' and the launcher's shapes chosen; the
suite counts; shellcheck's output; under `## Left`, `jig/scripts/live-sign-in.sh`'s
`--profile live`.

## Answers

2026-11-02. Q: shellcheck is not on hetzner9's PATH; install it, or supply one?
A: Neither: installing a package on a builder is the user's, and no gate step runs
shellcheck, so it does not block this task. Run `shellcheck` on the two scripts when
`command -v shellcheck` finds it; otherwise run `bash -n` on both (it must pass), keep each
`# shellcheck disable=...` directive directly above the nextest line it covers today, and
report `shellcheck: not on PATH, not run; bash -n clean`. That stands in for the acceptance's
shellcheck line and step 6's shellcheck command; everything else in the task is unchanged.
Go on with the implementation.
