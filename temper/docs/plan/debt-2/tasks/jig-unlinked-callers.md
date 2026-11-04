+++
id = "jig-unlinked-callers"
part = "jig-client-modes"
kind = "migration"
needs = ["jig-client-unlinked-requests"]
files = [
  "jig/testing/jig-client-test-domain/**",
  "jig/examples/ops/client-domain/**",
  "jig/testing/jig-client-test-view/**",
  "jig/testing/jig-client-test-shell-web/src/host.rs",
  "jig/crates/jig-client-view/src/tests/**",
  "jig/tests/client/**",
  "jig/tests/conformance/tests/suite/client_settings.rs",
  "jig/tests/engine-clients/tests/requests/**",
  "jig/tests/ops/src/client/**",
  "jig/tests/ops/tests/**",
]
effort = "high"
minutes = 110
+++

# jig D018, step 2: jig's unlinked callers step their clients by mode

## Goal

Every jig caller that steps an unlinked `jig_client::Client` steps it with `step_unlinked`
and `fire_unlinked` and matches `jig_client::UnlinkedRequest`; jig's two application roots
that run either mode carry the unlinked mode's requests under their own `Unlinked` variant
(`jig/docs/design/client/domain/client.md`, 4.4; `temper/docs/plan/debt-2/notes/contracts.md`).
Behaviour is unchanged. After this task and `temper-unlinked-callers`, nothing in jig or
temper names `jig_client::Request::Down` or `SignOut` but the linked paths' `unreachable!`
arms, which `jig-linked-requests` deletes with the variants.

Acceptance:

- `rg -n 'Request::(Down|SignOut)\b' jig --type rust | rg 'jig_client::|client::Request'`
  lists only: `jig/crates/jig-client/src/**`, `jig/crates/jig-client-shell-native/src/**`,
  `jig/tests/client-protocol/src/client.rs` (the linked world's own arm), and one
  `unreachable!` arm in each root's linked path. Report the list.
- No jig code calls `step` or `fire` on a client made with `Client::unlinked`.
- The tests in Steps pass, each count above zero, with no assertion changed but for the
  request's type path (`jig_client::Request::Down { .. }` read as
  `jig_client::UnlinkedRequest::Down { .. }`, or the root's `Client { .. }` read as
  `Unlinked { .. }`, in the unlinked mode).
- clippy is clean; `heavy cargo check -p temper-client-world --tests` passes.

## Inputs

- `temper/docs/plan/debt-2/notes/common.md` and `temper/docs/plan/debt-2/notes/contracts.md`
  (its jig section binds the names and the roots' variant).
- `jig/AGENTS.md`.
- `jig/docs/development/debt/D018-native-shell-unlinked-panics.md`, whole.
- `jig/docs/design/client/domain/client.md`, 4.4.

## Owns and may change

- The roots: `jig/testing/jig-client-test-domain/src/{domain,boundary,route}.rs` and
  `jig/examples/ops/client-domain/src/{domain,boundary,route}.rs`: the `Unlinked { request:
  jig_client::UnlinkedRequest }` variant; the unlinked mode stepped by the new pair (one
  scratch `Queue<jig_client::UnlinkedRequest>` beside today's `Queue<jig_client::Request>`,
  each used by its mode); `Down::Save` and `Down::Erase` turned into the root's `Save` and
  `Erase` as today (`jig-client-test-domain/src/route.rs`, ~188), in the same order; the
  linked path's `Down | SignOut` arm as the contract's one `unreachable!`.
- Their tests (`src/tests*`, `tests_client.rs`, `tests_support.rs`) and every test kit and
  world in `files` that steps an unlinked client or matches what it asks.
- `jig/testing/jig-client-test-shell-web/src/host.rs`: its catch-all for the root's
  `Client { .. }` gains `Unlinked { .. }` beside it, ignored as today.
- Reader edits the compiler asks for in other jig files, an arm or a type path each.
- temper: if `temper-client-world` (`temper/tests/client`) matches
  `jig_client_test_domain::Request` exhaustively, add its `Unlinked { .. }` arm, behaving as
  its `Client { .. }` arm did for the unlinked mode's requests; nothing else in temper.

Not yours: `jig/crates/jig-client/**`, `jig/crates/jig-client-shell-native/**`,
`jig/tests/client-protocol/**` (all linked, or `jig-linked-requests`'), and temper's own
roots (`temper-unlinked-callers`).

Pre-approved:

- A test helper that steps either mode may take the mode from the client it is given
  (`client.link()` or the root's own flag) and call the right pair; no trait, no generic.
- Where a test matched `Client { request: jig_client::Request::Told { .. } }` from an
  unlinked root: the roots handle `Told` and `Authorized` themselves today
  (`jig-client-test-domain/src/route.rs`, `told`), and keep doing so; if a test finds one
  forwarded, it reads `Unlinked { request: UnlinkedRequest::Told { .. } }`.
- If a test's expected sequence moves (an order, a count), the root's mapping changed
  behaviour: fix the mapping, never the test; if it cannot be kept, stop and ask with the
  test and both sequences.

## Steps

1. Sketch (test-domain's route; ops' is the same):

   ```rust
   // the unlinked mode
   match request {
       jig_client::UnlinkedRequest::Told { told: value } => told(root, env, value, out),
       jig_client::UnlinkedRequest::Down { down: jig_client::Down::Save { record } } => {
           out.push(Request::Save { record: Record::Unlinked(record) });
       }
       jig_client::UnlinkedRequest::Down { down: jig_client::Down::Erase { key } } => {
           out.push(Request::Erase { key: Key::Unlinked(key) });
       }
       request @ (jig_client::UnlinkedRequest::Down { .. }
       | jig_client::UnlinkedRequest::Authorized(_)
       | jig_client::UnlinkedRequest::SignOut) => out.push(Request::Unlinked { request }),
   }
   ```

   (as today's arms do for each request; where today a root answers `Authorized` itself, it
   still does).
2. Narrow tests, one at a time, each count above zero:
   - `heavy cargo nextest run -p jig-client-test-domain --lib`
   - `heavy cargo nextest run -p jig-ops-client-domain --lib`
   - `heavy cargo nextest run -p jig-client-test-view --lib`
   - `heavy cargo check -p jig-client-test-shell-web --target wasm32-unknown-unknown` (the
     crate has no test target and compiles only for wasm32: a check, no count)
   - `heavy cargo nextest run -p jig-client-view --lib`
   - `heavy cargo nextest run -p jig-client-world --test suite`, then `--lib` and each other
     binary holding a file you changed (`--test stories`, `--test golden`, `--test b_shared`)
   - `heavy cargo nextest run -p jig-conformance-world --test suite -E 'test(/client_settings/)'`
   - `heavy cargo nextest run -p jig-engine-clients-world --test requests`
   - `heavy cargo nextest run -p jig-ops-world --test suite`, and `--test client_stories` if
     it holds a file you changed
   - `heavy cargo check -p temper-client-world --tests`
3. `cargo fmt`; `python3 -I -B scripts/old_forms.py` from `jig/`; `heavy cargo clippy -p
   jig-client-test-domain -p jig-ops-client-domain -p jig-client-test-view -p
   jig-client-test-shell-web -p jig-client-view -p jig-client-world -p jig-conformance-world
   -p jig-engine-clients-world -p jig-ops-world --all-targets -- -D warnings`.
4. `python3 -I -B tools/citations/citations.py --base origin/main`.
5. Commits: the roots and their tests; the kits and worlds. Pushed.

## Report

As the implementer's text says, and also: the acceptance search's output; the files changed
by package; any test whose expected requests you had to re-read beyond its type path.

## Answers

2026-11-04. Q: may the web shell's nextest run (it selects 0 tests: `[lib] test = false`,
no test target, `src/lib.rs` gated to wasm32) be replaced by a wasm32 check? A: yes. Step 2's
`heavy cargo nextest run -p jig-client-test-shell-web` was the plan's error; it now reads
`heavy cargo check -p jig-client-test-shell-web --target wasm32-unknown-unknown`, which must
pass, and is exempt from "each count above zero". Add no test target to the crate. Also run
`heavy cargo clippy -p jig-client-test-shell-web --target wasm32-unknown-unknown -- -D warnings`
(the native clippy of step 3 compiles none of its code). Keep the work already pushed and finish
the remaining steps.
