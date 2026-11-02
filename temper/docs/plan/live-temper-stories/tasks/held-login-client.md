+++
id = "held-login-client"
part = "landing-story"
kind = "feature"
needs = []
files = [
  "temper/crates/temper/tests/support/forge_land.rs",
  "temper/crates/temper/tests/suite/forge_land.rs",
  "temper/docs/design/domain/testing.status.md",
]
effort = "high"
minutes = 40
+++

# The held record's import and give-back name the sitting's client directory

## Goal

The ChatGPT landing story's `account import FILE` failed after 6.6 s under fab's
after-landing pair. Its cause is the story's harness: every ctl call of a sitting runs with
the run's root (`LivePlan::client`, `<store>/first`) as its working directory and names
DIRECTORY `client` beneath it, but the held record's import guard and its give-back hook are
handed the run's root itself, so `temper ctl <store>/first account import ...` opens a
directory with no `address` and fails. Name the sitting's client directory there, as
`temper/docs/design/domain/testing.md, 2.7` says, with a test that is red before.

Acceptance:

- `LivePlan` gains `client_directory()`, `<client>/client`; the import guard and the return
  hook the landing story installs are built through two `LivePlan` methods that name it.
- The new suite test (below) fails with the methods naming the run's root, as the trunk's
  code does (the stub sees `<run>` as DIRECTORY), and passes with `client_directory()`; every `forge_land::` test of `temper`'s `suite` binary passes, and so does
  `startup::a_held_login_import_refuses_after_returning_without_starting_ctl` and
  `startup::a_held_login_return_hook_is_idempotent_and_refuses_unsafe_metadata_without_reading_the_file`;
  clippy is clean (it compiles the `live` binary).
- `temper/docs/design/domain/testing.status.md` no longer holds this half of the plan's 2.7
  entry.
- No live story is run (notes/common.md).

## Inputs

- `temper/docs/plan/live-temper-stories/notes/common.md`.
- `temper/docs/design/domain/testing.md, 2.7`: the owned-forge landing story's bullet, from
  "With `TEMPER_LIVE_ACCOUNT` naming a held ChatGPT record" to "names it.", and the runner's
  sub-bullets under "After every landing".
- `temper/docs/plan/live-temper-stories/inputs/request.md`, item 1.

## Owns and may change

Owns, in `temper/crates/temper/tests/`:

- `support/forge_land.rs`: `impl LivePlan<'_>` (~1324-1342): `install_return_hook` and the
  two new methods; the import guard's construction in `LiveJudge::drive` (~2191-2199) alone;
  `run_live`'s `plan.install_return_hook()` line (~2302) alone. Nothing else of the file
  (another task of this plan, `delegate-pages`, edits the judge after this one merges).
- `suite/forge_land.rs`: the new test, after
  `each_landing_story_takes_its_own_store_under_one_live_store`.
- `temper/docs/design/domain/testing.status.md`: in the **Not yet** entry that begins `2.7:
  the owned-forge landing story's held-record import and give-back`, remove the clause
  "held-record import and give-back name the run's root as DIRECTORY, not its `client`
  directory;" so that it reads `2.7: the owned-forge landing story reads each parent's ended
  delegates as one uncut page (...)`. Nothing else. This is the task's one departure from the
  implementer's text ("Not yours: any `docs/design/`").

Read only: `support/held_login.rs` (`return_hook(binary, store, client, account)` and
`import_guard(binary, store, client, arguments)` take `client` as ctl's DIRECTORY, as
`support/forge_land/rung.rs` ~613 passes it: unchanged), `support/ctl.rs` (`Calls` names
DIRECTORY `client` in its root; `alone`), `suite/startup.rs` (the hook's and guard's own
tests and their stub ctl), `live.rs` (~85-95 builds `LivePlan::client` as the run's root).

Pre-approved options:

- If the hook's and the guard's construction reads cleaner as free functions taking
  `&LivePlan`, that is yours; keep the names below.
- If the test needs `LivePlan`'s other fields filled (`tasks`, `baseline`, `roots` `None`;
  `words`, `line` empty), fill them; add no constructor.

## Steps

1. `support/forge_land.rs`. Sketch of the outer layer (bodies yours):

   ```rust
   impl LivePlan<'_> {
       /// The sitting's client directory, `client` beneath the run's root: every ctl call's
       /// DIRECTORY, the held record's import and give-back included (domain/testing.md, 2.7).
       #[must_use]
       pub fn client_directory(&self) -> PathBuf;

       /// Install the held record's give-back in the story's store; `binary` is the temper
       /// the hook runs.
       pub fn install_return_hook(&self, binary: &std::path::Path);

       /// The held record's import as the sitting runs it, behind the give-back's lock.
       #[must_use]
       pub fn held_import(&self, binary: &std::path::Path, words: &[&str]) -> super::ctl::Call;
   }
   ```

   `install_return_hook` passes `&self.client_directory()` as the hook's client;
   `held_import` calls `held_login::import_guard(binary, store, &self.client_directory(),
   words)`, `store` as `drive` takes it today (`self.paths[0].parent()`). `drive` calls
   `self.plan.held_import(&skein_scratch::paths::binary("temper"), &words)`; `run_live` calls
   `plan.install_return_hook(&skein_scratch::paths::binary("temper"))`.

2. `suite/forge_land.rs`:

   ```rust
   /// domain/testing.md, 2.7: the held record's import and give-back name the sitting's
   /// client directory, never the run's root.
   #[test]
   fn the_held_records_import_and_give_back_name_the_sittings_client_directory() {
       // scratch: store = stores/forge-land, its config/data/cache paths; run = store/first
       //   (0700) with run/client (0700); held/ (0700) with held/record (synthetic, 0600).
       // a stub ctl, 0700, as suite/startup.rs's: it appends "$@" (one per line) to a seen
       //   file; for `account list` prints {"accounts":{"party":[{"provider":1,"plan":true,"number":7}]}};
       //   for `export` writes a synthetic 0600 FILE.
       // plan = LivePlan { paths, client: &run, first: true, kept: false,
       //   model: ChatGpt { account: Some(&record) }, .. None / "" }.
       // ctl::alone(&run, plan.held_import(&stub, &["account", "import", record, "--key", "connect-plan"])):
       //   exit 0; the seen DIRECTORY (the word after `ctl`) == plan.client_directory() == run/client.
       // plan.install_return_hook(&stub); remove held/record; run store/give-back:
       //   it succeeds; the seen `list` and `export` calls name run/client; held/record is back, 0600.
   }
   ```

3. Tests: the new test alone (`heavy cargo nextest run -p temper --test suite -E
   'test(=forge_land::the_held_records_import_and_give_back_name_the_sittings_client_directory)'`),
   first with `held_import` and `install_return_hook` passing `self.client` as the trunk's
   code does, to see it red, then with `client_directory()`; then `-E
   'test(/^forge_land::/) | test(/^startup::a_held_login/)'`.

4. The status clause (Owns). Then `cargo fmt`; `heavy cargo clippy -p temper --all-targets --
   -D warnings`; `heavy cargo nextest run -p temper` (the focused profile: never the live
   binaries); `python3 -I -B tools/citations/citations.py --base origin/main` after
   committing.

5. Commits: the test and the fix; the status clause. Push your branch.

## Report

As the implementer's text says, and also: the test's red line on the trunk's code, the
`suite` counts, and whether any other caller of `held_login::return_hook` or `import_guard`
names a run's root (`rung.rs` names its `client` directory today).
