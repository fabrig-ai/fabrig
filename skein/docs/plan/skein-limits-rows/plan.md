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

Counts are from before/after nextest listings; setup's `all` includes its
two fuzzy tests in those listings. Only focused tests ran. Tables and rows
count folds made here, including labelled cases from former inner tables.
Floors are the request's estimates; the five additional binaries' floors
and the kit's are unknown.

| Binary | Module | Tests before | Tests after | Tables | Rows | Floor saved, ms |
|---|---|---:|---:|---:|---:|---:|
| `skein-channel` | `tests` | 33 | 31 | 1 | 4 | unknown |
| `skein-fake-forge-domain` | `object::tests` | 2 | 2 | 0 | 0 | 0.0 |
| `skein-fake-forge-domain` | `tests` | 244 | 227 | 7 | 39 | 27.2 |
| `skein-fake-forge-forgejo` | `tests` | 227 | 210 | 13 | 30 | 30.6 |
| `skein-fake-forge-github` | `server::git_tests` | 2 | 2 | 0 | 0 | 0.0 |
| `skein-fake-forge-github` | `server::tests` | 2 | 1 | 1 | 2 | 1.9 |
| `skein-fake-forge-github` | `tests` | 194 | 192 | 1 | 3 | 3.8 |
| `skein-forge-client` | `tests` | 257 | 206 | 13 | 87 | 76.5 |
| `skein-forge-connection` | `limits::tests` | 19 | 14 | 2 | 7 | unknown |
| `skein-forge-connection` | `other modules (*)` | 157 | 157 | 0 | 0 | unknown |
| `skein-forge-machine-world::suite` | `hand::forgejo::*` | 96 | 92 | 4 | 16 | 10.4 |
| `skein-forge-machine-world::suite` | `hand::github::*` | 291 | 235 | 25 | 82 | 145.6 |
| `skein-forge-setup-world::suite` | `all` | 155 | 150 | 5 | 10 | 12.5 |
| `skein-http` | `tests` | 143 | 135 | 4 | 30 | 12.0 |
| `skein-http` | `tests::server` | 73 | 68 | 2 | 14 | 7.5 |
| `skein-llm` | `client::tests` | 38 | 24 | 1 | 15 | unknown |
| `skein-llm` | `other modules (*)` | 111 | 111 | 0 | 0 | unknown |
| `skein-llm-connection` | `component` | 3 | 3 | 0 | 0 | unknown |
| `skein-llm-connection` | `deadlines` | 5 | 5 | 0 | 0 | unknown |
| `skein-llm-connection` | `tests` | 38 | 31 | 1 | 8 | unknown |
| `skein-oauth` | `v3::golden_tests` | 2 | 2 | 0 | 0 | 0.0 |
| `skein-oauth` | `machine_tests` | 74 | 61 | 4 | 23 | 20.8 |
| `skein-oauth` | `pkce::tests` | 2 | 1 | 1 | 3 | 1.6 |
| `skein-oauth` | `tests` | 33 | 28 | 3 | 17 | 8.0 |
| `skein-oauth-accounts` | `component` | 1 | 1 | 0 | 0 | 0.0 |
| `skein-oauth-accounts` | `exchange` | 1 | 1 | 0 | 0 | 0.0 |
| `skein-oauth-accounts` | `tests` | 161 | 132 | 5 | 34 | 46.4 |
| `skein-rows` | `tests` | 5 | 8 | 0 | 0 | unknown |
| `skein-tls` | `client::native::tests` | 26 | 26 | 0 | 0 | unknown |
| `skein-tls` | `tests` | 54 | 50 | 1 | 5 | unknown |

(*) Other modules aggregate untouched provider/translation/derivation/replay tests
in LLM and untouched app/component/connection/installation tests in forge
connection; their focused test names remain unchanged.

The folds replace **340 tests with 94 tables holding 429 rows**, removing
246 processes. All 340 removed names occur verbatim as row names (with a
label suffix where an inner case becomes a row); a second audit of removed
declarations in `git diff` agrees with the compiled-name audit. The kit
adds three independent panic-helper tests. No suite-total timing is claimed.

All folded unit tables read 5–15 ms on their final PASS lines, and the
world tables read 12–41 ms, within the 40 ms and 200 ms guides. Each changed
test file has a two-row mutation failure; the shared GitHub write table's
three mutated source-module rows fail together.

Beyond the survey, four fixed transport/rate stage wrappers also fold;
their inner response set is the same for every stage. The six stale-token
loss wrappers share one body instead of the survey's two tables. The three
forced-push refusal streams and compatible foreign label, edit, deletion
and ordinary queue-name failures also fold as recorded data. Fixtures
retain each row's extra accounts, initial waits, profile and response
sequence; common true assertions are retained or strengthened.

`app_jwt_route` was profiled before attempting a fold: one route collected
54,137,815 Ir, including 45,031,053 signing Ir. The four-route table read
24 ms, then 46 ms after restoring its mutations, above the guide. The
decided option therefore keeps all four original tests and their RSA
comment. The six app-check tests also remain unchanged.

Left apart:

- `skein-channel::tests`, 30 tests: invalid schema and output-queue overflow assert different constructor errors; other lifecycle/schema contracts and existing tables remain.
- `skein-fake-forge-domain::object::tests`, 2 tests: independent object spelling and count-decoding contracts outside the owned test files.
- `skein-fake-forge-domain::tests`, 220 tests: marker parsing, successful permission streams, verdict precedence, checkpoints, exact rebase exemptions and other domain operations or existing tables.
- `skein-fake-forge-forgejo::tests`, 197 tests: different push-head/signature, failed-job and deleted-branch hook checks, authentication, pagination and write contracts, and existing tables; all 13 surveyed groups fold.
- `skein-fake-forge-github::server::git_tests`, 2 tests: outside the two targeted HTTP intake tests.
- `skein-fake-forge-github::tests`, 191 tests: different fake contracts and existing tables; the six app-check tests remain separate.
- `skein-forge-client::tests`, 193 tests: distinct lifecycle, cache, outbox, queue, recovery and settlement assertions, and existing tables; all 13 surveyed groups fold.
- `skein-forge-connection::limits::tests`, 12 tests: different key, platform and component lifecycle contracts and existing tables; all seven refusals fold into plain-constructor and capacity-assertion tables.
- `skein-forge-connection::other modules`, 157 tests: different key, platform and component lifecycle contracts and existing tables; all seven refusals fold into plain-constructor and capacity-assertion tables.
- `skein-forge-machine-world::hand::forgejo::*`, 88 tests: heap/process meters, existing tables and distinct call, discovery, renewal, pull, read and write contracts.
- `skein-forge-machine-world::hand::github::*`, 210 tests: heap/process meters, existing tables and distinct call, discovery, renewal, pull, read and write contracts.
- `skein-forge-setup-world::all`, 145 tests: heap measurements, two fuzzy sweeps, existing tables and distinct host, creation, permissions and lifecycle paths; all five surveyed groups fold.
- `skein-http::tests`, 131 tests: different framing failures, gzip algorithms, stream lifecycle, demand ordering, misuse panics and existing tables; all six surveyed groups fold.
- `skein-http::tests::server`, 66 tests: different framing failures, gzip algorithms, stream lifecycle, demand ordering, misuse panics and existing tables; all six surveyed groups fold.
- `skein-llm::client::tests`, 23 tests: different client and translation assertions and existing tables; all 15 refusal wrappers fold.
- `skein-llm::other modules`, 111 tests: different client and translation assertions and existing tables; all 15 refusal wrappers fold.
- `skein-llm-connection::component`, 3 tests: link and process-memory contracts outside the refusal family.
- `skein-llm-connection::deadlines`, 5 tests: deadline and process-memory contracts outside the refusal family.
- `skein-llm-connection::tests`, 30 tests: three refusals keep additional endpoint-record setup, measured request-head failure or computed largest-call reservation checks; after-Closed is a separate lifecycle panic; other contracts and existing tables remain.
- `skein-oauth::v3::golden_tests`, 2 tests: different exchange, claims, registration and byte-encoding checks, and existing tables; generated code stays untouched.
- `skein-oauth::machine_tests`, 57 tests: different exchange, claims, registration and byte-encoding checks, and existing tables; generated code stays untouched.
- `skein-oauth::tests`, 25 tests: different exchange, claims, registration and byte-encoding checks, and existing tables; generated code stays untouched.
- `skein-oauth-accounts::component`, 1 test: nonce process-memory check outside the owned test files.
- `skein-oauth-accounts::exchange`, 1 test: exchange-header check outside the owned test files.
- `skein-oauth-accounts::tests`, 127 tests: distinct sign-in, persistence, grant, refresh, listener and shared-owner paths, heap checks and existing tables; all five surveyed groups fold.
- `skein-rows::tests`, 8 tests: independent runner admission, continuation, diagnostics and panic-message contracts.
- `skein-tls::client::native::tests`, 26 tests: outside the targeted config refusal family.
- `skein-tls::tests`, 49 tests: native-only room refusal accepts portable limits, unlike the five folded refusals; other TLS and RSA contracts remain separate.

The three measured/extra-setup LLM-connection refusals, TLS's native-only
refusal and channel's two constructor/schema refusals remain tests under
step 11's decided option. No assertion is weakened, and no test changes
suite. Mutable harnesses are built per row; input records and transcripts
carry data, never callbacks.

### rows-forge

Counts are from before and after nextest package listings. Tables and rows
count these folds, including labelled inner cases; retained tables are not
recounted. Floor saved uses the request's 1.9 ms setup and 1.7 ms forge floors.

| Binary | Module | Tests before | Tests after | Tables | Rows | Floor saved, ms |
|---|---|---:|---:|---:|---:|---:|
| `skein-forge-setup` | `github::key` | 1 | 1 | 0 | 0 | 0.0 |
| `skein-forge-setup` | `host` | 51 | 43 | 5 | 17 | 15.2 |
| `skein-forge-setup` | `tests::adopt` | 19 | 19 | 0 | 0 | 0.0 |
| `skein-forge-setup` | `tests::call` | 4 | 4 | 0 | 0 | 0.0 |
| `skein-forge-setup` | `tests::check` | 16 | 13 | 3 | 9 | 5.7 |
| `skein-forge-setup` | `tests::copy` | 12 | 12 | 0 | 0 | 0.0 |
| `skein-forge-setup` | `tests::create` | 4 | 4 | 0 | 0 | 0.0 |
| `skein-forge-setup` | `tests::create_bounds` | 16 | 10 | 2 | 8 | 11.4 |
| `skein-forge-setup` | `tests::create_file` | 2 | 2 | 0 | 0 | 0.0 |
| `skein-forge-setup` | `tests::create_repository` | 4 | 4 | 0 | 0 | 0.0 |
| `skein-forge-setup` | `tests::github_accept` | 6 | 6 | 0 | 0 | 0.0 |
| `skein-forge-setup` | `tests::github_checks` | 3 | 3 | 0 | 0 | 0.0 |
| `skein-forge-setup` | `tests::github_convert` | 13 | 13 | 0 | 0 | 0.0 |
| `skein-forge-setup` | `tests::github_given` | 11 | 10 | 1 | 2 | 1.9 |
| `skein-forge-setup` | `tests::github_hook` | 10 | 7 | 1 | 15 | 5.7 |
| `skein-forge-setup` | `tests::github_install` | 2 | 2 | 0 | 0 | 0.0 |
| `skein-forge-setup` | `tests::github_installations` | 23 | 22 | 1 | 12 | 1.9 |
| `skein-forge-setup` | `tests::github_manifest` | 9 | 8 | 1 | 6 | 1.9 |
| `skein-forge-setup` | `tests::github_paste` | 11 | 11 | 0 | 0 | 0.0 |
| `skein-forge-setup` | `tests::github_picker` | 23 | 22 | 1 | 2 | 1.9 |
| `skein-forge-setup` | `tests::github_reconcile` | 24 | 14 | 1 | 11 | 19.0 |
| `skein-forge-setup` | `tests::github_send` | 19 | 19 | 0 | 0 | 0.0 |
| `skein-forge-setup` | `tests::given` | 24 | 20 | 3 | 7 | 7.6 |
| `skein-forge-setup` | `tests::hooks` | 24 | 23 | 1 | 2 | 1.9 |
| `skein-forge-setup` | `tests::host` | 94 | 76 | 9 | 54 | 34.2 |
| `skein-forge-setup` | `tests::item` | 13 | 13 | 0 | 0 | 0.0 |
| `skein-forge-setup` | `tests::limits` | 5 | 5 | 0 | 0 | 0.0 |
| `skein-forge-setup` | `tests::reachable` | 38 | 25 | 13 | 98 | 24.7 |
| `skein-forge-setup` | `tests::reconcile` | 30 | 25 | 5 | 18 | 9.5 |
| `skein-forge-setup` | `tests::steps` | 21 | 18 | 2 | 25 | 5.7 |
| `skein-forge-setup` | `tests::take_over` | 20 | 18 | 2 | 4 | 3.8 |
| `skein-forge` | `forgejo` | 247 | 219 | 24 | 81 | 47.6 |
| `skein-forge` | `github` | 231 | 227 | 4 | 25 | 6.8 |
| `skein-forge` | `tests` | 32 | 20 | 3 | 34 | 20.4 |

All 49 listed setup groups and 30 listed forge groups fold. Two additional
setup folds join GitHub hook failure diagnostics and timing relation bounds;
a further forge fold joins GitHub App name and collection bounds. Setup
replaces 131 tests with 51 tables holding 290 rows, saving 80 processes
(152.0 ms by its floor). Forge replaces 75 tests with 31 tables holding
140 rows, saving 44 processes (74.8 ms by its floor).

Every removed name remains a row label: 206 removed, 206 found. The explicit
out-of-order picker table keeps all 32 invalid pairs: 20 admin-path pairs
and 12 connected-path pairs, before and after. The host settings table
keeps its 13 existing cases and 11 further refusal cases. The halted-plan
table keeps all 23 phases, including seven hook phases.

The shared bodies preserve every assertion. Checks that also hold for all
rows are extended to them: stored identity completion, raw-file targets
and decoder absence, owner-wait completion and late answers, and zero
listing-row admission. Invalid etags carry their expected retained header
bytes separately: an empty etag is retained internally but is not a usable
validator. App bounds retain both the decoder result and error classification.

Left apart:

- `github::key`, 1 test: private-key framing has its own grammar checks.
- `host`, 38 tests: connection abort and capacity, callback state bindings,
  state allocation ceilings, runtime removal and GitHub lifecycle contracts
  have different assertions; process-local meters remain separate.
- `tests::host`, 67 tests: retries, redaction, shutdown, admission, runtime
  changes and bounds have distinct sequences or additional invariants;
  the eight runtime tests and four TLS fixture readers stay separate.
- `tests::reachable`, 12 tests: token-selection sequences, retained-token
  checks, dropped-row pagination and diagnostic redaction have different
  assertions or are existing tables.
- `tests::adopt`, 19 tests: admin, repository, permission, collaborator,
  routing, redaction and terminal-phase checks have separate contracts.
- `tests::call`, 4 tests: dialect pairing, authentication ownership and
  diagnostic redaction inspect different types and invariants.
- `tests::check`, 10 tests: token order, matching identity, user admission,
  reply failures and terminal phases have different checks or existing tables.
- `tests::copy`, 12 tests: migration, markers, organization and file
  transitions, admission and host integration have different sequences.
- `tests::create`, 4 tests: creation races, empty-file completion, halt
  and the initial owner request have different assertions.
- `tests::create_bounds`, 8 tests: path/name grammar, decoder limits,
  namespace rules and zero limits are existing tables; the heap meter
  remains a process of its own.
- `tests::create_file`, 2 tests, and `tests::create_repository`, 4 tests:
  HTTP refusals, successful documents, malformed replies and transport loss
  have different checks.
- `tests::github_accept`, 6 tests: accepted-set ordering, persistence,
  capacity, installation validation and terminal phases are separate contracts.
- `tests::github_checks`, 3 tests: missing grants, access strength and
  extra names each have their own assertions and existing rows.
- `tests::github_convert`, 13 tests: PEM bounds, identity, persistence,
  failures, admission, halt and redaction use different checks.
- `tests::github_given`, 9 tests: grammar, origins, URL bounds, limit
  precedence, state lifetime and redaction have different assertions.
- `tests::github_hook`, 6 tests: repair permission, URL generation, child
  forwarding, unasked events and redaction each show another contract.
- `tests::github_install`, 2 tests: state-link completion and refusal
  paths have different checks and existing rows.
- `tests::github_installations`, 21 tests: listing order, accepted-set
  pruning, age/capacity rules, wrong replies, persistence and end states
  each carry different assertions or existing tables.
- `tests::github_manifest`, 7 tests: JSON, target/state admission,
  grammar and terminal diagnostics have separate checks or existing rows.
- `tests::github_paste`, 11 tests: PEM, app/bot identity, item persistence,
  hook repair, failures and terminal redaction have different sequences.
- `tests::github_picker`, 21 tests: accepted installations, minting,
  listing validation, token release and end states have different checks.
- `tests::github_reconcile`, 13 tests: app/hook/install sequences,
  persistence, matching reconciliation and diagnostics have different checks.
- `tests::github_send`, 19 tests: authentication, signing, HTTP transport,
  timeout and redaction assertions differ across their exchange states.
- `tests::given`, 17 tests: names, ownership, ordering, duplicate ids,
  precedence and constructor admission have distinct checks or existing rows.
- `tests::hooks`, 22 tests: secret creation, hook selection, pagination,
  patching, redaction and end states have different sequences or existing tables.
- `tests::item`, 13 tests: each item has its own decoding, encoding,
  byte grammar, bound and diagnostic checks, many already tables.
- `tests::limits`, 5 tests: relation admission, pair diagnostics and
  precedence are different rules; existing tables remain.
- `tests::reconcile`, 20 tests: bot creation, identity and permission
  checks, organization/repository work, reply failures and redaction
  have different assertions or existing tables.
- `tests::steps`, 16 tests: reconciliation, access, hooks, terminal
  machines and unasked events have different sequences and checks.
- `tests::take_over`, 16 tests: admin checks, pagination, repeat deletion,
  token redaction and terminal states have separate checks or existing rows.
- `skein-forge::tests`, 17 tests: locate, marker, cache, vocabulary,
  credential and validator contracts have different assertions or existing rows.
- `forgejo`, 195 tests: wire grammars, routing, streamed decoding, plan
  calls, exchanges, recovery and bounds have different assertions, many
  already tables; the moved repository UTF8 row and removed-file spellings
  are retained in the shared tables.
- `github`, 223 tests: request/document contracts, delivery, App facts,
  pull/check/job resolution, pagination, writes, replay and redaction
  have different sequences, fields or additional assertions, many already tables.

One table for every changed test file is checked by deliberately wrong
expectations; the UTF8 table also checks its row moved from repositories.
Each mutation reports at least two failed row names in one run, then the
restored table passes. No listed group is left apart.

The focused package run passes all 472 setup tests and 466 forge tests.
With one process at a time, all 82 folded tables take 6–19 ms on their
PASS lines, within the 40 ms guide. The 24-case host settings table takes
14 ms, the eleven-phase secret-release table 14 ms and the 23-phase
halted-plan table 7 ms.

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
