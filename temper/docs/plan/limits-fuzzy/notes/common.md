# limits-fuzzy: the rules every task names

The request is `temper/docs/plan/limits-fuzzy/inputs/request.md`: read *For every task of this plan*, which binds whatever your spec says. `notes/measuring.md` says how a sweep is profiled and written down, and `notes/contracts.md` holds the shape that crosses tasks. All three paths are under `temper/docs/plan/limits-fuzzy/`.

The repair's order is `skein/docs/foundation/testing-strategy.md`, 8.1: does the test belong here; if so, can it be made faster, at the cause; if neither, the measurements go in the report and the test stays. **What a test checks decides its place, never its cost.** Every sweep named in this plan belongs in the fuzzy suite: each settles many drawn worlds or inputs (`temper/docs/design/domain/testing.md`, 6.1). This plan moves none.

## Who owns what

One writer per crate. A cause the profile puts in a crate you don't own goes under `## Left` in your figures and your report, with its figures and its share. `under-causes` starts after the sweep tasks and reads those sections, and `close` carries what's still left to the user.

| Task | Owns |
|---|---|
| `public-wire` | `temper/tests/codecs/**` |
| `one-process-sweeps` | `temper/tests/one-process/**` |
| `state-hash` | the derive lines of skein's and jig's state types, in `notes/contracts.md` |
| `engine-sweeps` | `temper/tests/engine/domain/**`, and the derive lines of temper's state types, in `notes/contracts.md` |
| `under-causes` | the crates under the worlds and `temper/tests/real-loop/**`, once the three above have merged |
| `close` | the account in `temper/docs/development/limits-fuzzy.md` |

Each task also owns its own figures file, `temper/docs/plan/limits-fuzzy/figures/<task>.md`.

## Departures from the implementer's text

- **A tool the task lacks is otherwise a question.** Valgrind is the request's own method, so `notes/measuring.md` says how to get it with no root and no change to the machine. Only if that fails is it a `Question:`.
- **Questions:** the request says decide, don't ask. A `Question:` is only for what truly blocks your acceptance. A small edit just outside your file list is made and named in the report.
- **`docs/design/`:** where your spec names a design edit (a status entry to remove), make it, in the same task. No other design text.

## Every fix

- **Profile first, quote before and after.** No change without the figure that put the cause there.
- **Largest first; stop when the next cause's gain is under about 2% of a sweep.** Say what you stopped at, with its share. A footprint isn't a gain: say what could really go.
- **A sweep keeps what it checks.** The worlds or seeds it draws, each crash, each invariant and each ending stay. A sweep's rate of worlds per CPU second is what goes up. An input that doesn't vary with the seed isn't a draw: checked once, it's still checked.
- **Nothing is lost.** A test that goes names what holds its behaviour now, by test name, in `temper/docs/plan/limits-fuzzy/plan.md`, and that holder exists and passes first. A test past its guide isn't merged with another or named an exception (`temper/docs/design/domain/testing.md`, 6.1). No spec in this plan plans for a test to go.
- **A product change is fine where the profile puts the cause,** at the size the cause needs, with its own focused test and no change of behaviour. Keep that test cheap: the focused suite is over its own limit.
- **A change of meaning runs its readers' tests:** if a shared item returns something else or keeps another order, run the tests of each package that reads it.
- **No budget, timeout, deadline or build setting changes, and test builds stay unoptimized.** A cost no test can avoid goes in your report, and stays. This plan adds no `tools/tiers/budgets.toml` entry.
- **A stored format or protocol may change freely,** with no compatibility.
- **A figure that fell by less than two runs of the same code differ isn't a gain.** Say so.
- **Before calling a focused or fuzzy failure outside your change,** run it once on the pristine base. If main passes and your branch fails, it's yours.

## Not this plan

- Only focused and fuzzy run. No browser, process or live story, no `tools/tiers/run`, no walk, no render check, no throwaway driver. A slow-tier story you touch is compiled (`--no-run`), not run, and you say so.
- `temper-conformance-world`'s sweeps were `limits-worlds`' work: touch them only through a shared cause.
- Focused tests aren't this plan's to repair, though a fix under a sweep may pay there too. Say so when it does.

## Beside this plan

Other drafts hold files near here. Keep hunks small, don't reformat, and don't move code you needn't change. If a merge meets another draft in a file your spec names, keep their change, and say so in your report.

- `account-email/facts-record` holds `jig/crates/jig-domain-accounts/src/{facts,records,restore,changes,domain,lib}.rs`, `jig/crates/jig-domain-core/src/start.rs` and `jig/crates/jig-protocol-store/**` (its accounts schema, codec, keys and intervals).
- `account-email/oauth-profile` holds `skein/crates/skein-oauth/**`, `skein/crates/skein-oauth-accounts/**` and `skein/crates/skein-fake-oauth/src/lib.rs`.
- `account-email/status-email` holds `smith/crates/smith-protocol-program/**`, `smith/crates/smith-host-domain/src/program.rs` and `smith/crates/smith-host-protocol/src/program*`.
- `call-room/large-argument` holds `smith/tests/agent-process/**` and `smith/tests/fake-claude/src/**`.
- `jig-world-cost-2/naming-row` holds `jig/testing/jig-conformance/src/referee*` and the naming rows of `jig/testing/jig-conformance/src/core/naming.rs` and `jig/testing/jig-core-kit/src/conformance/naming.rs`.
- `diagrams`, on its way to main, holds `skein/crates/skein-ui*/**`, `temper/crates/temper-service/src/stock/**`, `temper/crates/temper-service/src/tests/stock.rs`, `temper/crates/temper-client-view/src/limits.rs` and `temper/crates/temper-client-shell-web/**`.
- `skein-gate-entries`, on its way to main, holds `tools/tiers/budgets.toml`.
