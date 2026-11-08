# client-binary: the deployment's client, `temper-client`, M1 of deployment-binaries

The spec is `temper/docs/design/draft/deployment-binaries.md`, sections 4 and 9.1, carried out as `deployment-binaries.execution.md`, M1 says: read the draft whole, then M1's row, which is this plan's input. It designs and builds together and lands whole. It's unmarked: it yields to v0.1.0's critical pieces.

## What it builds

- **A package of its own, `temper-client`,** with a `temper-client` binary target. It holds the TUI's start (`temper [ADDRESS]` as the box's `temper` runs it today), `ctl`'s entry (`temper-client ctl …`, the same code `temper ctl` runs), and their help. Whatever the user names the published file (section 5, open), the Cargo target is `temper-client` under options A to C.
- **The TUI's address and link-file helpers** move out of the `temper` library into a client crate both binaries link.
- **Its records without the `forge` feature,** which its own invocation keeps (section 8).
- **`--version`,** as `temper/docs/design/release.md`, 3 says the binaries print it.
- **What it may link** (section 4): the client crates except the web shell and the bundle, jig's client crates, and skein's. The boundary test is M5's: name any crate it can't yet drop in `plan.md`, with the reason.
- **The box's `temper` keeps every command it has,** linking the moved code, so the box and the tests don't change.

## Tests

As section 10 says for each binary, in step tests of its package: `--version` and its help print, and an unknown command is refused. Section 10's story, `temper ctl` from `temper-client` signed in to an engine and reading `me`, is written and compiled, not run. If `temper-engine` (M2, temper/ui's) isn't on main when that task runs, write it against the box's `temper serve` as the engine, and leave a status entry for M5 to switch it.

## Design

Write the client's binary in `temper/docs/design/protocol/README.md`, section 4 (the commands' rows by binary), and `temper/docs/design/client/README.md`, section 2 (its crates) and 6.11, and close those lags in `deployment-binaries.status.md`. Set M1's state in the execution file. Don't touch section 5's open name choice: leave the published file's name to the release job and the user.

## Overlaps

- **`tui-box-front`** (temper/tui) is in flight in `temper`'s `tui.rs`, `lib.rs`, `box_command.rs` and `commands.rs`, adding `tui_front.rs`. Move the TUI's helpers as one task, small, and expect a catch-up. If that plan has landed, move its front's helpers too.
- **`temper ctl`** is temper/migration's. Its entry is linked into the client unchanged; no behaviour of `ctl` changes.
- M2 (temper/ui) splits `temper-service` beside this plan: keep off the service.

## Walk

`walk = false`. The story runs after landing.

## For every task of this plan

- **A plan is complete when it lands.** No walk, render check or throwaway driver in it.
- **Only focused and fuzzy run in a plan.** Never `tools/tiers/run`, a browser, process or live tier, one of their stories, or a tier's CPU figure. A new story for those tiers is written and compiled (`cargo check --tests`), not run. A slow-tier test whose expected output the change makes stale is updated in the same task, compiled, and named in the report.
- **Tests stay cheap:** a new case of one behaviour is a row of an existing table test. Budgets and timeouts are the user's, never raised.
- **Decide, don't ask.** A stored format, a protocol or a public interface may change freely before v0.1.0: no compatibility, no migration, no `Question:`; note it in `plan.md`. A small edit a task needs just outside its file list is made and named in the report. A `Question:` is only for what truly blocks the task.
- **Generated files** (`docs/help/reference/*`) that a change makes stale are that task's to regenerate.
- **Before calling a focused or fuzzy failure "outside the change",** run it once on the pristine base. If main passes and the branch fails, it's the branch's.
