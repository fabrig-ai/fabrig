# skein-limits-rows: a margin under skein's focused limit

The request is [inputs/request.md](inputs/request.md): a margin of at least 2.5 CPU s under skein's focused limit, by two causes the last repairs left. `plan.toml` holds the parts and slices.

## Parts and tasks

- **`judge`:** `judge-parses`. The connection's access judge parses a key, and the fake verifies a token: neither signs to check.
- **`rows`:** `rows-codec` makes the `skein-rows` kit and folds `skein-codec`. `rows-forge` folds `skein-forge-setup` and `skein-forge`. `rows-worlds` folds the rest of the request's table, and every other `valid_refuses_*` family.

## Critical path

`judge-parses` and `rows-codec` start at once. `rows-codec` carries the kit, and is small for that reason. `rows-forge` starts when it has merged. `rows-worlds` starts when both have, since it folds `skein-forge-connection`'s and the fake's tests, which `judge-parses` changes. The critical path is `rows-codec`, then `rows-worlds`: about 290 implementer-minutes, by the specs' estimates.

## Where this plan lands against the request

**The honest folds fall well short of the request's estimate.** Six read-only surveys read the request's whole table against section 10's rule that rows carry data and one body checks them alike. They find **534 tests that fold into 188 tables, removing 346 processes: about 0.64 CPU s by the request's floors,** against its estimate of about 1,000 tests and 2 CPU s. These are readings of the code, unmeasured, and an implementer may find a little more or less.

| Binary | Tests | Fold | Tables | Processes removed | Floor, ms | Saved, ms | Task |
|---|---:|---:|---:|---:|---:|---:|---|
| `skein-forge-machine-world::suite`, `hand` | 387 | 85 | 29 | 56 | 2.6 | 145.6 | `rows-worlds` |
| `skein-forge-setup` | 552 | 125 | 49 | 76 | 1.9 | 144.4 | `rows-forge` |
| `skein-forge-client` | 257 | 64 | 13 | 51 | 1.5 | 76.5 | `rows-worlds` |
| `skein-forge` | 510 | 73 | 30 | 43 | 1.7 | 73.1 | `rows-forge` |
| `skein-oauth-accounts` | 163 | 34 | 5 | 29 | 1.6 | 46.4 | `rows-worlds` |
| `skein-codec` | 204 | 45 | 21 | 24 | 1.5 | 36.0 | `rows-codec` |
| `skein-fake-forge-forgejo` | 227 | 30 | 13 | 17 | 1.8 | 30.6 | `rows-worlds` |
| `skein-oauth` | 111 | 27 | 8 | 19 | 1.6 | 30.4 | `rows-worlds` |
| `skein-http` | 216 | 19 | 6 | 13 | 1.5 | 19.5 | `rows-worlds` |
| `skein-fake-forge-domain` | 246 | 17 | 7 | 10 | 1.6 | 16.0 | `rows-worlds` |
| `skein-forge-setup-world::suite` | 153 | 10 | 5 | 5 | 2.5 | 12.5 | `rows-worlds` |
| `skein-fake-forge-github` | 198 | 5 | 2 | 3 | 1.9 | 5.7 | `rows-worlds` |
| **Total** | **3,224** | **534** | **188** | **346** | | **636.7** | |

`rows-worlds` also folds about 44 `valid_refuses_*` tests in five crates outside the table, whose floors aren't known. Most of the rest are different behaviours: a state machine's path, a recovery, a race, each with its own assertions. Where they aren't, they're already tables, whose rows cost no process.

So item 2 gives about 0.7 CPU s, and item 1 some tenths more, which `judge-parses` measures in instructions. **The margin of 2.5 CPU s probably isn't reached by these two causes,** and only the gate's read after landing can say by how much. This plan doesn't loosen a fold to close the gap: the request says a fold is honest or it isn't done.

## Overlaps

- `skein-oauth`'s tests with `account-email/oauth-profile`, on its trunk and not yet on main, and `skein-forge-client` with `limits-fuzzy/state-hash`. Both come late in `rows-worlds`, whose folds touch those crates' tests alone.
- `skein-llm` with `limits-fuzzy/under-causes` and `model-catalogue-3/list-parse`: `rows-worlds` touches one test file, `src/client/tests.rs`.
- `skein-forge-setup` with `ui-forge-setup`, whose cut is in flight and whose files aren't known.
- `skein/docs/design/testing.status.md`: `rows-codec` and `rows-worlds` each remove their own line. The two lines are apart so that their merges don't meet.

## What was decided

- **The access judge is `limits::access`,** in `skein-forge-connection`, not a fake: the request's heading says "the fake's", but the figures behind it (`skein-limits-heavy`'s `machine-costs`) name `limits::access` and `Jwt::bearer` as the costliest hand test's two signings, and the hand harness speaks to a scripted wire. The fake pays by a second site, `verified_claims`, which signs a token's input again to compare it. `judge-parses` fixes both with one function, `rs256_public_pkcs1`.
- **A key that parses and won't sign is now caught at its first signing.** As recalled from ring's source, which the cut didn't read here, ring checks `d`, `dP`, `dQ` and `e` against the rest only when it signs, so a parsing judge passes such a key; `judge-parses` holds that with a test, and has a decided option if it's wrong. No request goes out on it: `Jwt::bearer` fails and the forge stops `Invalid`, naming the key, as `skein/docs/design/forge-connection.md`, 5.2 now says, with a test. The price is that such a key stops the forge after a connect rather than before it. Everything else that was refused at connect still is.
- **The fake's two design documents disagreed:** `tls.md`, 4 said a fake forge checks with `verify_rs256`, and `fake-forge.md`, 10.1 said it signs again. The code followed the second. `fake-forge.md` now says the first.
- **The fold kit is new.** `skein-codegen`'s `table` is the tree's only row helper, private to generated code, and takes function rows that return a `bool`, which `skein/docs/foundation/testing-strategy.md`, 10 rules out by hand. It doesn't catch a row's panic either, so a body would have to give up `assert_eq!`. `skein-rows` has no dependencies, so a binary's floor doesn't grow.
- **Tables already in these files stay as they are,** though they stop at their first failure: rewriting them saves no floor.
- **`rows-worlds` adds `skein_rows::panics`.** The largest honest families are `#[should_panic]` tests of limits, which `notes/rows.md` keeps apart for other tasks. They're section 8's own example of a fold, so `panics` holds the same check inside a row. `rows-codec`'s kit doesn't change.
- **Groups left alone are named by each task,** in its subsection below, after reading them.

## Before and after

Each task fills its own subsection. Suite totals aren't quoted here: the gate reads them after landing.

### rows-worlds

To be filled by the task.

### rows-forge

To be filled by the task.

### rows-codec

To be filled by the task.

### judge-parses

To be filled by the task.
