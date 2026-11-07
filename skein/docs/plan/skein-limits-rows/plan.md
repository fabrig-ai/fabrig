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

| Binary | Module | Tests before | Tests after | Tables | Rows | Floor saved, ms |
|---|---|---:|---:|---:|---:|---:|
| `skein-rows` | `tests` | 5 | 8 | 0 | 0 | unknown |
| `skein-forge-machine-world::suite` | `hand::github::*` | 291 | 238 | 24 | 78 | 137.8 |
| `skein-forge-machine-world::suite` | `hand::forgejo::*` | 96 | 92 | 4 | 16 | 10.4 |
| `skein-forge-client` | `tests` | 257 | 206 | 13 | 90 | 76.5 |
| `skein-oauth-accounts` | `tests` | 161 | 132 | 5 | 34 | 46.4 |
| `skein-fake-forge-forgejo` | `tests` | 227 | 210 | 13 | 30 | 30.6 |
| `skein-oauth` | `machine_tests` | 74 | 61 | 4 | 23 | 20.8 |
| `skein-oauth` | `tests` | 33 | 28 | 3 | 17 | 8.0 |
| `skein-oauth` | `pkce::tests` | 2 | 1 | 1 | 3 | 1.6 |
| `skein-http` | `tests::server` | 73 | 68 | 2 | 15 | 7.5 |
| `skein-http` | `tests` | 143 | 135 | 4 | 31 | 12.0 |
| `skein-fake-forge-domain` | `tests` | 244 | 227 | 7 | 39 | 27.2 |

| `skein-forge-setup-world::suite` | `host` | 24 | 22 | 2 | 4 | 5.0 |
| `skein-forge-setup-world::suite` | `create` | 8 | 7 | 1 | 2 | 2.5 |
| `skein-forge-setup-world::suite` | `reconcile` | 15 | 14 | 1 | 2 | 2.5 |
| `skein-forge-setup-world::suite` | `github` | 13 | 12 | 1 | 2 | 2.5 |
| `skein-fake-forge-github` | `server::tests` | 2 | 1 | 1 | 2 | 1.9 |
| `skein-fake-forge-github` | `tests` | 191 | 189 | 1 | 3 | 3.8 |
| `skein-llm` | `client::tests` | 38 | 24 | 1 | 15 | unknown |
| `skein-llm-connection` | `tests` | 38 | 31 | 1 | 8 | unknown |
| `skein-forge-connection` | `limits::tests` | 19 | 14 | 2 | 7 | unknown |

Left apart:

- `skein-rows::tests`, 8 tests: independent runner admission, continuation, diagnostics and panic-message contracts.
- `hand::github::*` and `hand::forgejo::*`: heap/process meters, distinct call, renewal, discovery, pull, read and write contracts and existing tables remain separate. Four transport-start wrappers keep their reset IO body; the four HTTP-start wrappers share response IO. Six stale-token wrappers share one loss helper rather than the survey’s two tables. Shared bodies strengthen the common revision, probe-count, range and empty-log checks.
- `skein-forge-client::tests`: all 13 surveyed groups fold; other lifecycle, cache, outbox, queue, recovery, settlement and bound tests retain distinct operations or extra checks. Existing tables remain. The byte-case fixtures now carry Answer values rather than callbacks, and the unused bad_live wrapper is removed.
- `skein-oauth-accounts::tests`: all five surveyed groups fold. Sign-in, persistence, grant, refresh, cancellation, listener and shared-owner paths with distinct assertions, heap checks and existing tables stay apart.
- `skein-fake-forge-forgejo::tests`: all 13 surveyed groups fold. Push signing/head, failed-job and deleted-branch hooks have different checks and stay separate; so do other authentication, route, pagination, write and response contracts and existing tables.
- `skein-oauth`, other client exchanges, OpenID claims, registrations, byte encodings and generated golden tests: distinct validation and lifecycle contracts or existing tables. All eight surveyed groups fold; generated code stays untouched.
- `skein-http::tests` and `tests::server`: all six surveyed groups fold. Other framing failures, gzip algorithms, stream lifecycle, demand ordering and misuse panics have different assertions and remain separate. Request-body rows all check the final 204 suffix.
- `skein-fake-forge-domain::tests`: surveyed verdict, log and report groups fold. Three forced-failure tests and the compatible foreign label, edit, deletion and queue-name failures also fold as profile/observation/decision data. Marker parsing, successful permission streams, verdict precedence, checkpoints, exact rebase exemptions and other domain operations retain separate contracts or existing tables.
- `skein-forge-setup-world::suite`: all five surveyed groups fold. Heap measurements, fuzzy sweeps and distinct host, creation, reconciliation, permissions and GitHub lifecycle paths stay separate; common page reads, connection facts and mutation-route absence checks hold for every folded row.
- `skein-forge-setup-world::suite`: all five surveyed groups fold. Heap measurements, fuzzy sweeps and distinct host, creation, reconciliation, permissions and GitHub lifecycle paths stay separate; common page reads, connection facts and mutation-route absence checks hold for every folded row.
- `skein-fake-forge-github::tests::service_app`, 4 JWT route tests: attempted combined table read 24 ms, then 46 ms after restoring mutations, above the 40 ms guide; all four tests and the RSA comment stay apart. One original row profiled first at 54,137,815 Ir, including 45,031,053 signing Ir. The six app-check tests and other distinct fake contracts stay apart. Both other surveyed groups fold; the write table consumes request data from all three source modules.
- `skein-llm::client::tests`: all 15 limit-refusal wrappers fold; the other client and translation contracts retain their different checks and existing tables. Floor is unknown for this binary.
- `skein-llm-connection::tests`, 3 limit tests: too many endpoint records, a measured request-head failure and a computed largest-call reservation have additional setup or assertions beyond a limit literal, so stay apart. Eight helper-equivalent refusals fold; the after-Closed panic is a separate lifecycle rule. Floor is unknown.
- `skein-forge-connection::limits::tests`: all seven refusal tests fold into two tables, separating plain constructor refusals from refusals that also assert capacity_bound is absent. Other limit, key, platform and component lifecycle contracts stay apart. Floor is unknown.

### rows-forge

To be filled by the task.

### rows-codec

Counts are from the package's before and after nextest listings. Tables and
rows count the folds made here, including inner cases taken from existing
tables. Floor saved uses the request's 1.5 ms codec floor, not a suite timing.

| Binary | Module | Tests before | Tests after | Tables | Rows | Floor saved, ms |
|---|---|---:|---:|---:|---:|---:|
| `skein-codec` | `patch::run` | 64 | 57 | 5 | 39 | 10.5 |
| `skein-codec` | `patch::plan` | 39 | 31 | 6 | 77 | 12.0 |
| `skein-codec` | `patch::text` | 15 | 14 | 1 | 18 | 1.5 |
| `skein-codec` | `patch` | 12 | 10 | 2 | 5 | 3.0 |
| `skein-codec` | `tests::piece` | 19 | 14 | 4 | 14 | 7.5 |
| `skein-codec` | `tests` | 5 | 5 | 0 | 0 | 0.0 |
| `skein-codec` | `value` | 16 | 15 | 1 | 2 | 1.5 |
| `skein-codec` | `schema` | 20 | 18 | 2 | 6 | 3.0 |
| `skein-codec` | `schema::fingerprint` | 14 | 13 | 1 | 2 | 1.5 |
| `skein-rows` | `tests` | 0 | 5 | 0 | 0 | — |

All 21 listed groups fold. Three further folds take the rename-then-widen
admission into the layout admission table, missing written sites into the
missing-operation table, and byte-sum overflow refusals into the piece bound
table. Together these replace 49 tests with 22 tables holding 163 rows,
removing 27 codec processes, or 40.5 ms by the request's floor. The new kit
has five tests of its own; no floor for its new binary is assumed.

Every removed test name is retained: 49 removed, 49 found in row names.
Former inner cases use `old_name: label`.

Left apart:

- `patch::run`, 52 tests: distinct builder operations, stops, recovery paths,
  capacity lifecycles, and process-local allocation checks; existing tables
  with those contracts remain.
- `patch::plan`, 25 tests: matching and evidence, value encoding, pieces,
  flags, retained tables and ordering each have their own assertions;
  existing tables remain.
- `patch::text`, 13 tests: scanning, grammar admission, numeric refusals,
  statement kinds, numbering and header-only parsing are different phases
  or contracts, many already tables.
- `patch`, 8 tests: lookup, family admission, plan ownership, handles and
  shared plan capacity have different checks.
- `tests::piece`, 10 tests: writing, counting, bounded and counted splitting
  have different operations or extra assertions; the two panic tests stay
  separate.
- `tests`, 5 tests: length, count, text, version and reason checks show
  different operations.
- `value`, 14 tests: accessors, refusal paths, index ownership, iteration,
  capacities and secret access have different checks or are already tables.
- `schema`, 16 tests: grammar and layout phases, encoded limits, nesting,
  widths and source lines have different assertions or are already tables.
- `schema::fingerprint`, 12 tests: headers, type productions, child and
  family digests, layout invariance and ceiling walks are different
  contracts or existing tables.
- `skein-rows::tests`, 5 tests: continuation, failure diagnostics, success
  and the two admission rules each pin their own invariant.

The wrong-scalar rows both check the expected halt, the next call's order
error and a fresh start at their own root. Writer refusals retain the
written-count checks, then fill untouched room before finishing: a sized
writer must finish full. Missing-operation rows all check the complete
family-block diagnostic. No assertion is weakened.

The final focused package run passes all 177 codec tests and five kit
tests. With one test process at a time, the 22 folded tables take 5–14 ms
on their PASS lines, each within the 40 ms unit-test guide. Each changed
codec test file has a two-row expected-value mutation that reports both
failures; all five kit tests also reject a broken rule.

### judge-parses

Each named test ran alone under Callgrind through nextest's runner, as
`notes/common.md` says: x86-64, unoptimized test builds, default profile,
the same builder before and after. `Collected` is the test's instruction
count (also the profile's summary); the signing column includes
`sign_rs256` and all its callees. No test name moved.

| Package and test | Collected Ir before | Collected Ir after | `sign_rs256` calls before → after | Inclusive signing Ir before → after |
|---|---:|---:|---:|---:|
| `skein-forge-machine-world::suite`, `hand::github::github_installation_loss::stale_full_after_deleted_is_discarded` | 28,168,371 | 17,682,754 | 2 → 1 | 22,580,773 → 11,290,102 |
| `skein-fake-forge-github`, `tests::app::github_fake_jwt_accepted` | 126,199,178 | 58,107,517 | 11 → 4 | 123,832,976 → 45,034,019 |
| `github-conformance`, `tests::github_conformance_names_a_token_expiring_within_30_days` | 61,539,809 | 51,804,727 | 2 → 1 | 22,519,550 → 11,261,726 |

The two checks have different callers, found by searching their uses:

- **Access admission:** `skein-forge-connection`'s `Component::connect`
  calls `limits::access` (`src/component.rs`). Its GitHub path now calls
  `rs256_public_pkcs1`, never signs. `skein-forge-machine-world`'s hand
  harness (`src/hand.rs`) and `skein-forge-kit`'s machine and joint
  harnesses (`src/machine.rs`, `src/joint.rs`) pay for admission through
  that component. The measured hand test loses the judge's signing;
  its one remaining call is `Jwt::bearer`'s product signing.
- **Fake authentication:** `skein-fake-forge-github`'s `verified_claims`
  is called by `app::check` and `Verified::check`. The latter is used by
  that package's service (`src/service.rs`) and `skein-forge-kit`'s joint
  GitHub harness (`src/joint/github.rs`). `skein-fake-peers` wraps the
  service (`src/forge.rs`), so its GitHub consumers, including
  `github-conformance` (`src/fake.rs`) and `skein-fake-peers-tests`, pay
  for that check too. `skein-forge-setup-world` also constructs the service
  directly (`src/host.rs`, `src/plans.rs`). The measured fake test keeps
  four fixture signings
  and replaces seven checking signings with public-key verification;
  conformance keeps its runner's one signing. The exact-byte/key cache
  and the checks of claims on every request remain.

The new public-half function and signing share the portable-target,
byte-bound and ring-parse refusals in `app_key`. The 2050-bit public test
fixture is 1190 bytes, within `RS256_KEY_BYTES`; ring refuses it during
parsing. Removing the modulus-size guard therefore leaves its row green:
no such parsed key reaches that guard. The guard and row stay, under the
spec's decided option. Bypassing ring parsing instead makes the refusal
test fail on a trailing byte. Changing `e` to 65539 still parses and fails
at signing, so no design amendment is needed. Both damaged-key admission
rows, the stop-before-request test, and the fake's wrong-key, changed-claims
and changed-cache-key checks fail when their respective checks are removed.
