+++
id = "replace-run"
part = "replace"
kind = "feature"
needs = ["check-connect"]
files = [
  "temper/crates/temper-service/src/route.rs",
  "temper/crates/temper-service/src/setup.rs",
  "temper/crates/temper-service/src/setup/public.rs",
  "temper/crates/temper-service/src/setup/lifecycle.rs",
  "temper/crates/temper-service/src/tests/setup.rs",
  "temper/crates/temper-service/src/tests/setup_replace.rs",
  "temper/docs/design/protocol/forge.status.md",
  "temper/docs/design/protocol/setup.status.md",
  "temper/docs/design/protocol/credentials.status.md",
]
effort = "high"
minutes = 120
+++

# The routing runs a Replace: Check on a connected Forgejo, Paste with the kept App id on GitHub

## Goal

A `Replace` admitted by the forge domain (`forge-connect`) reaches the routing as
`root::Request::Replace { forge, request, carried }`, refused `not taken` there until now. Stage
it by the forge's kind: a connected Forgejo's `Check`, its pasted tokens read before the kept
ones, so that one token alone is checked with the other kept; GitHub's `Paste` with the kept
`AppId`. On `Done` each pasted value is kept over the old and the forge, connected or stopped,
connects again with them; on `Stop` nothing is kept and the forge stays as it was.

Acceptance:

- `route.rs`'s `root::Request::Replace` arm calls a new `setup::public::replace(engine, forge,
  request, carried)`: `full()` answers `Busy`; else `ConnectAnswered::Connecting`, the carried
  row's `Get` and digest check, then by the forge's kind:
  - a connected Forgejo: the bundle's items a subset of `ApiToken`, `GitToken` and
    `WebhookSecret`, each nonempty, with at least one token, else stopped before any plan, `the
    pasted values are not a Forgejo's tokens`; a `Pasted::Tokens` caller and `Plan::Check {
    forge }`, as `tokens_connect` stages it;
  - GitHub: one `Pasted` item, else `the pasted value is not a key`; the App's id read from the
    forge's kept `AppId` row (as `temper-protocol-records`' `forge.rs` reads it for the access),
    none or unreadable stopping the run before any plan, `its records are unreadable`; a
    `Pasted::App` caller and `Plan::Paste { forge, app }`, as `connect` stages it.
- `Done`: as `check-connect`'s for a Forgejo (each pasted item kept, `Origin`, `BotLogin`,
  `ConnectEnded`, the erase, then the connect from records with the `GitToken` value at its next
  generation when one was pasted), and as a paste's for GitHub; on a forge connected already,
  `Request::Connect` hands the component the new access (skein's `forge-connection.md`, 3.5),
  and the second `Connected` this makes is taken as a first is: nothing in the routing expects
  one connect or one reconcile per forge for a lifetime (`lifecycle.rs:678`'s
  `reconcile.insert(..).expect`, `public.rs`' `pastes` and `erases` inserts: each takes a
  replacement as a fresh entry, or is shown never to meet one).
- `Stop`: nothing kept, the carried row erased, `ConnectEnded { connected: false, why }` in C5's
  words for a Forgejo and a paste's words for GitHub; the forge domain restores its connection.
- Tests (a new `src/tests/setup_replace.rs`, registered in `src/tests/setup.rs` as `#[path =
  "setup_replace.rs"] mod replace_tests;`), on `check-connect`'s fixtures for Forgejo and
  `tests/setup_lifecycle.rs`'s `Face` for GitHub:
  - a connected Forgejo, connected: a Replace of the git token alone, checked with the kept API
    token; the `GitToken` row kept over the old, the `ApiToken` row unchanged; the identity's
    value at the next generation before `Request::Connect`; `Connected` again as the same login;
  - the same forge stopped, then a Replace of both tokens: connected;
  - a Replace of another user's tokens: stopped `OtherUser` in C5's words, nothing kept, the
    forge still connected;
  - a Replace bundle holding an `AdminToken`, and one of the secret alone: stopped before any
    plan, nothing staged;
  - GitHub, connected: a Replace with a new key of the same App: `Paste` with the kept id,
    `AppKey` kept over the old, connected again, its reconcile run once after; a key of another
    App: stopped `OtherApp`, nothing kept, still connected;
  - a Replace's request bytes through the service's client path (as `check-connect`'s story
    sends a connect's) answered `connecting`;
  - no token, key or secret in any event, note or log line.
- Status: remove the entries `check-connect` left, word for word: in `forge.status.md` the
  sentence that begins `2.2, 2.6: \`Replace\` of a connected Forgejo's tokens`, in
  `setup.status.md` the one that begins `4: a connected Forgejo's \`Replace\``, and in
  `credentials.status.md` the one that begins `2.2, 5: a connected Forgejo's tokens and a GitHub
  App's key, replaced`.

## Inputs

- `temper/docs/plan/forgejo-connect/notes/common.md`; `notes/contracts.md`, C1, C4 and C5.
- `temper/docs/design/protocol/forge.md`, 2.2 (`Replace`; a replacement), 2.5 and 2.6 (`Paste`'s
  row: Replace on GitHub, with the kept id).
- `temper/docs/design/protocol/setup.md`, section 4 (*Replaced*) and 17.3 (*Replace*).
- skein's `docs/design/forge-connection.md`, 3.5 (a replacement's access).

## Owns and may change

- `temper-service/src/route.rs`: the `root::Request::Replace` arm alone.
- `temper-service/src/setup/public.rs`: `replace`; `ended` and `erase` for a replacement.
- `temper-service/src/setup/lifecycle.rs` and `setup.rs`: what a second connect of a running
  forge needs, nothing else.
- `temper-service/src/tests/setup.rs` (the module's line) and the new `tests/setup_replace.rs`;
  helpers it shares with `setup_tokens.rs` or `setup_lifecycle.rs` moved to `fixtures.rs` or
  made `pub(super)`, nothing else of them changed.
- The three status files, as the acceptance says.

Pre-approved: a GitHub paste's existing test that assumed a forge is connected once
(`tests/setup_github.rs:361`, `paste_replay_and_refused_replacement_keep_the_connection_and_write_counts`,
whose refusal is the forge domain's and stays) is left as it is; if one breaks, its assertion
moves to what the design says and the report names it. A skein fault found here (the component
refusing a second `Connect` of a connected forge) is a question.

## Steps

1. `replace` and its refusals: `heavy cargo nextest run -p temper-service --lib -E
   'test(replace_tests)'`.
2. `Done` on a connected forge; the second `Connected`; the Forgejo and GitHub stories, the same
   run; then `-E 'test(tokens_tests) | test(lifecycle_tests) | test(github_tests)'`.
3. The status edits; the citations check.
4. `cargo fmt`; old forms for temper; `heavy cargo clippy -p temper-service --all-targets -- -D
   warnings`; `heavy cargo nextest run -p temper-service`; `heavy cargo check --tests` for
   `temper` and `temper-one-process-world`. Commit the code and tests, then the status text;
   push.

## Report

As the implementer's text says, with what a second connect of a connected forge needed.

## Rebase (2026-11-03, step merge, foreman forgejo-connect/replace-run/foreman-1)

resolve setup.status.md by retaining the playground retention clarification and removing the completed Replace gap.
The merge conflicts only in `temper/docs/design/protocol/setup.status.md`, on the single-line `Not yet` paragraph.
Branch commit `4d671f2928` removes the Replace gap; main commit `74d7c626a5`, incorporated into the integration trunk by `f236af0705`, expands the playground removal gap.
Resolve by preserving both changes:
- Remove exactly: “4: a connected Forgejo's `Replace`, and 17.3: a pasted App's (`temper-service/src/route.rs` refuses it `not taken`).”
- Preserve the trunk's 8.4 clarification beginning “; the record stays while any kept task names one of its repositories” through its citation to jig's `protocol/store.md`, 11.1.
Keep all other status text unchanged. No Rust changes are needed for this refusal.
Run the citations check after resolving.
