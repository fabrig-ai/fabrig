+++
id = "temper-unlinked-callers"
part = "jig-client-modes"
kind = "migration"
needs = ["jig-client-unlinked-requests"]
files = [
  "temper/crates/temper-client-domain/**",
  "temper/crates/temper-client-tools/src/root.rs",
  "temper/crates/temper-client-tools/src/recovery.rs",
  "temper/crates/temper-client-tools/src/tests/**",
  "temper/crates/temper-client-shell-cli/src/host.rs",
  "temper/crates/temper-client-shell-terminal/src/**",
  "temper/crates/temper-client-shell-web/src/host.rs",
  "temper/crates/temper-client-shell-web/src/host_tests.rs",
  "temper/crates/temper-client-view/src/tests/**",
  "temper/crates/temper-client-view/tests/**",
  "temper/crates/temper-service/src/tests/setup_github_render.rs",
  "temper/crates/temper/tests/support/ctl/member.rs",
  "temper/tests/client/src/app.rs",
  "temper/tests/client/src/front.rs",
  "temper/tests/client/src/front/**",
  "temper/tests/client/tests/**",
  "temper/tests/engine/domain/tests/suite/forge_routes/tier_task_show.rs",
  "temper/tools/gallery/src/**",
]
effort = "high"
minutes = 120
+++

# jig D018, step 2 in temper: temper's unlinked callers step their clients by mode

## Goal

Every temper caller that steps an unlinked `jig_client::Client` steps it with
`step_unlinked` and `fire_unlinked` and matches `jig_client::UnlinkedRequest`; temper's two
roots that run either mode (`temper-client-domain` and `temper-client-tools`'s `Root`) carry
the unlinked mode's requests under their own `Unlinked` variant, and the linked hosts match
it as they matched the client's `Down` and `SignOut` (jig's
`jig/docs/design/client/domain/client.md`, 4.4; `temper/docs/plan/debt-2/notes/contracts.md`).
Behaviour is unchanged. This is temper's half of jig's
`jig/docs/development/debt/D018-native-shell-unlinked-panics.md`; `jig-linked-requests`
pays it.

Acceptance:

- `rg -n 'Request::(Down|SignOut)\b' temper --type rust | rg 'jig_client::|client::Request'`
  lists only one `unreachable!` arm in each root's linked path (the contract's). Report it.
- No temper code calls `step` or `fire` on a client made with `Client::unlinked` (through
  `Root::unlinked`, `domain::unlinked`, `root::unlinked`, `client::unlinked` or directly).
- The tests in Steps pass, each count above zero, with no assertion changed but for the
  request's type path (`jig_client::Request::Down { .. }` read as
  `jig_client::UnlinkedRequest::Down { .. }`, or the root's `Client { .. }` read as
  `Unlinked { .. }`, in the unlinked mode).
- clippy is clean.

## Inputs

- `temper/docs/plan/debt-2/notes/common.md` and `temper/docs/plan/debt-2/notes/contracts.md`
  (its jig section binds the names, the roots' variant and the linked hosts' arms).
- `temper/AGENTS.md`.
- `jig/docs/development/debt/D018-native-shell-unlinked-panics.md`, whole.
- `jig/docs/design/client/domain/client.md`, 4.4.

## Owns and may change

- The roots: `temper-client-domain/src/{domain,boundary,route}.rs` (`route.rs`, ~215: the
  linked path keeps `Client { request }`, its `Down | SignOut` as one `unreachable!` arm; the
  unlinked path steps with the new pair and pushes `Unlinked { request }`; `Told` and
  `Authorized` handled as today in both modes) and `temper-client-tools/src/root.rs` (`Root::new`,
  `Root::unlinked`, ~198-212) with `recovery.rs` (reads `Down::Save` and `Down::Send` of the
  unlinked mode: from `UnlinkedRequest::Down`).
- The linked hosts, each one arm for the root's `Unlinked { .. }`:
  - `temper-client-shell-cli/src/host.rs` (~449-466) and `temper-client-shell-terminal`
    (`src/route.rs` ~22-36, `src/client.rs` ~398-417): today the client's `Down` and
    `SignOut` reach `shell.down` and panic; the arm is the contract's
    `unreachable!("a linked root asks no unlinked request (jig's client.md, 4.4)")`;
  - `temper-client-shell-web/src/host.rs` (~262-298): today it ignores them (`=> {}`); it
    ignores `Unlinked { .. }`;
  - `temper/crates/temper/tests/support/ctl/member.rs` (~118-126): as the CLI.
- The tests and tools in `files` that step an unlinked client or match what it asks
  (`temper-client-domain`'s `src/tests/**` and `tests/suite`, `temper-client-tools`'
  `src/tests/**`, the terminal's and web shell's unlinked fixtures and `host_tests.rs`,
  `temper-client-view`'s tests, `temper-service/src/tests/setup_github_render.rs`,
  `temper-client-world`'s `app.rs`, `front.rs`, `front/**` and suites, the engine domain
  world's `tier_task_show.rs`, the gallery's `driver.rs`).
- Reader edits the compiler asks for in other temper files, an arm or a type path each.
  `temper/tests/client/src/fixtures.rs` and the rest of `temper-service/src/tests/` are
  another task's (`temper-fixtures-records`, `temper-file-credentials-gone`): if one needs
  an edit, make the one line and say so in the report.

Not yours: anything in jig (`jig-client-unlinked-requests`, `jig-unlinked-callers`,
`jig-linked-requests` own it).

Pre-approved:

- A test helper that steps either mode may take the mode from its root and call the right
  pair; no trait, no generic.
- If a test's expected sequence moves (an order, a count), the root's mapping changed
  behaviour: fix the mapping, never the test; if it cannot be kept, stop and ask with the
  test and both sequences.

## Steps

1. Sketch (`temper-client-domain/src/route.rs`):

   ```rust
   // the linked mode, as today but for one arm
   request @ (jig_client::Request::Save { .. }
   | jig_client::Request::Erase { .. }
   | jig_client::Request::Exchange { .. }
   | jig_client::Request::OpenStream { .. }
   | jig_client::Request::CloseStream
   | jig_client::Request::ServiceCredential { .. }) => out.push(Request::Client { request }),
   jig_client::Request::Down { .. } | jig_client::Request::SignOut => {
       unreachable!("a linked client asks no Down or SignOut (jig's client.md, 4.4)")
   }
   // the unlinked mode
   jig_client::UnlinkedRequest::Told { told: value } => told(root, env, value, out),
   jig_client::UnlinkedRequest::Authorized(answer) => crate::authorize::answered(root, answer, out),
   request @ (jig_client::UnlinkedRequest::Down { .. } | jig_client::UnlinkedRequest::SignOut) => {
       out.push(Request::Unlinked { request });
   }
   ```
2. Narrow tests, one at a time, each count above zero:
   - `heavy cargo nextest run -p temper-client-domain --lib`, then `--test suite`
   - `heavy cargo nextest run -p temper-client-tools --lib`
   - `heavy cargo nextest run -p temper-client-shell-cli`
   - `heavy cargo nextest run -p temper-client-shell-terminal`
   - `heavy cargo nextest run -p temper-client-shell-web --lib`
   - `heavy cargo nextest run -p temper-client-view --lib`, then `--test suite`
   - `heavy cargo nextest run -p temper-service --lib -E 'test(/setup_github_render/)'`
   - `heavy cargo nextest run -p temper-client-world --test suite`, then `--lib`
   - `heavy cargo nextest run -p temper-engine-domain-world --test suite -E 'test(/tier_task_show/)'`
   - `heavy cargo check -p temper --tests` and `heavy cargo check -p temper-client-gallery --tests`
3. `cargo fmt`; `python3 -I -B scripts/old_forms.py` from `temper/`; `heavy cargo clippy -p
   temper-client-domain -p temper-client-tools -p temper-client-shell-cli -p
   temper-client-shell-terminal -p temper-client-shell-web -p temper-client-view -p
   temper-client-world -p temper-client-gallery --all-targets -- -D warnings`.
4. `python3 -I -B tools/citations/citations.py --base origin/main`.
5. Commits: the roots and their tests; the hosts; the other tests and tools. Pushed.

## Report

As the implementer's text says, and also: the acceptance search's output; the files changed
by package; any test whose expected requests you had to re-read beyond its type path.
