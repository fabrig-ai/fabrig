+++
id = "prepare-row"
part = "prepare-row"
kind = "feature"
needs = []
files = ["temper/crates/temper-forge-setup/src/prepare.rs", "temper/crates/temper-forge-setup/src/runner.rs", "temper/crates/temper-forge-setup/src/tests/**", "temper/crates/temper-forge-setup-shell/src/prepare.rs", "temper/crates/temper-forge-setup-shell/tests/suite/prepare.rs", "temper/docs/design/protocol/setup.status.md"]
effort = "high"
minutes = 45
+++

# A later preparation leaves the row of a runner that has declared alone

## Goal

`temper forge prepare` runs `forgejo-cli actions register` at every later preparation. That writes the runner's row over: raw labels (`NAME:host`) and no version. A runner that has declared doesn't declare again, so jobs asking `runs-on: NAME` wait forever with no error. Register only where the row may not exist; everywhere else, render the configuration from the `uuid` the present configuration names.

Acceptance:

- A failing test came first: the shell story below, run on the unchanged code, failed because the row went back to raw. The report quotes that failure.
- **No `register`** when the runner's `runner/config.yaml` names a `uuid` and no secret is drawn: a later preparation at the same versions, with the configuration identical or changed; an upgrade of Forgejo; an upgrade of the runner; a runner added again with its secret kept.
- **`register` as today** for a first preparation (a cut-short one too), a runner added with a secret drawn (even over a stale configuration), and a later preparation or upgrade whose configuration is missing or names no `uuid`. A missing `runner/secret` then fails naming `register`, as today.
- A changed capacity or labels still rewrites `runner/config.yaml` under the runner's lock, with the kept `uuid`, and a running runner still refuses it. An identical configuration still takes no runner lock and keeps its inode.
- `runner/config.yaml`'s text, `state`, the commands' arguments and prepare's output do not change for any setup. The goldens pass unchanged.

## Inputs

- `temper/docs/design/protocol/setup.md`, 6.3: items 2 and 4, and the bullet **The runner's row is the runner's once it is made**, which this cut wrote. 13.1's preparation decisions, and 13.3's runner bullet.
- `temper/docs/design/protocol/setup.status.md`: the last bullet, **Not yet:** 6.3, 13.1.
- The request: `temper/docs/plan/runner-labels/inputs/request.md`.

## Owns and may change

Owns: `temper-forge-setup`'s `src/prepare.rs`, `src/runner.rs` and `src/tests/**`; `temper-forge-setup-shell`'s `src/prepare.rs` and `tests/suite/prepare.rs`.

May also change, and how:

- `temper/docs/design/protocol/setup.status.md`: remove the last bullet (**Not yet:** 6.3, 13.1) and nothing else. This departs from your text's "not yours: any `docs/design/`", because the bullet is what this task closes. Another draft edits this file's other lines, so leave them as they are.

Not touched: `temper/crates/temper/tests/support/owned.rs` and `tests/suite/forge.rs`. Their `finished` fixture writes no configuration, so it takes the `register` path as before. `docs/help/` is not touched either: no page says a later preparation registers, and `temper/docs/help/tier.md`'s "Prepared runners keep their registration and secrets" stays true. No test reaches a real host, and fabrig/ui's files stay as they are.

Options, each pre-approved:

- If a `Found` literal outside the files above needs the new field, add it (compiler-forced).
- If reading the configuration in `found` fails with anything but absent, refuse as `write` does today: `Failed::Io("runner configuration", io::ErrorKind::PermissionDenied)`. Do not fall through to `register`.
- If anything here seems to need a change to a stored format, a protocol or a public interface, it's a `Question:`. None is expected.

## Steps

1. **The failing test first,** in `temper-forge-setup-shell/tests/suite/prepare.rs`. The scripted Forgejo's `actions register` arm in `owned()` also writes its arguments to a `row` file beside the script (`"$(dirname "$0")/row"`), as Forgejo writes its row, and still prints `kept-runner`. Then:

   ```rust
   #[test]
   fn a_later_preparation_leaves_the_row_a_runner_declared() {
       // with_runner; run once: `row` holds the raw `temper-host:host`.
       // Write "temper-host" over `row`, as the runner's start declares it.
       // Run again: `row` is still "temper-host".
       // Capacity 2, run again: `row` is still "temper-host"; config.yaml says capacity 2 and uuid "kept-runner".
       // Remove config.yaml, run again: `row` is raw again; config.yaml names "kept-runner".
   }
   ```

   Run it on the unchanged code and keep the failure for the report.

2. **The host reads, the machine decides.** `Found` gains:

   ```rust
   pub runner_configuration: Option<Secret>, // The runner's configuration, whole; absent if it has none.
   ```

   `found` in the shell reads it with `read_private_file`, at the bound `write` uses (`1 << 20`). `Secret` because the `uuid` is the runner secret's first sixteen characters in hexadecimal.

3. **`runner::registered`,** beside `configuration`, sharing its `uuid` line's text with it through a private const:

   ```rust
   /// The `uuid` a configuration of `configuration` names, as `register` printed it; `None` for any other text,
   /// which the preparation registers again.
   #[must_use]
   pub fn registered(configuration: &[u8]) -> Option<Secret>
   ```

4. **The rule in `Prepare`** (`temper/docs/design/protocol/setup.md`, 6.3). In the `self.state.prepared` branch of the found step, keep `runner::registered` of `found.runner_configuration` in a new field. Don't set it on the cut-short path: a first preparation never consults it. In `after_migrate`, between the draw and `self.run(Command::Register)`, a kept `uuid` goes straight to the configuration's write. That write and `Command::Register`'s arm in `ran` share one function taking the `uuid`, and it moves to `Phase::RunnerConfiguration` as today.

5. **Step tests,** in `src/tests/prepare.rs` and `upgrade.rs`, with `fixtures::later()` naming a configuration, so `older` does too, and `runner_added()` naming none:
   - Update the tests that pin `Register` on a path that no longer runs it: `refreshed`, `a_later_capacity_change_rewrites_the_runner_without_replacing_its_registration` (its `uuid` now comes from the fixture's configuration), `the_upgrade_dumps_an_older_forgejo_before_it_migrates`, `the_upgrade_prepares_an_older_runner_again_without_a_dump`, and any other test that drives a later preparation through it. Each keeps what it checks; only the `Register` step goes.
   - `the_upgrade_stops_where_register_failed_and_repeats_next_time` keeps its meaning with a found that names no configuration.
   - New tests: a later preparation with no configuration registers; one whose configuration names no `uuid` registers; a runner added again with its secret and configuration writes the configuration with no `register`; a runner added with a drawn secret over a stale configuration registers and writes the printed `uuid`.
   - `runner::registered` reads back what `configuration` wrote, and refuses text with no `uuid` line and an empty `uuid`.

6. **Checks.** This changes what a later preparation does, so the readers' tests run too:

   ```
   heavy cargo nextest run -p temper-forge-setup --lib
   heavy cargo nextest run -p temper-forge-setup-shell --test suite
   heavy cargo nextest run -p temper --test suite -E 'test(forge::)'
   ```

   Then your text's `cargo fmt`, `old_forms.py` and `clippy` for both packages, and `python3 -I -B tools/citations/citations.py --base origin/main`. Commit the test and the fix apart, the test first, and push.

## Report

Your text's report, and:

- the failure of step 1 on the unchanged code;
- the rule you broke for each new test and the failure seen;
- the counts of the three runs in step 6.
