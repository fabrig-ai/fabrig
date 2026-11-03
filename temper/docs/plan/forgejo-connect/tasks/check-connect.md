+++
id = "check-connect"
part = "service"
kind = "feature"
needs = ["pasted-list", "wire-connected", "forge-connect"]
files = [
  "temper/crates/temper-service/src/route.rs",
  "temper/crates/temper-service/src/setup.rs",
  "temper/crates/temper-service/src/setup/public.rs",
  "temper/crates/temper-service/src/setup/lifecycle.rs",
  "temper/crates/temper-service/src/forges.rs",
  "temper/crates/temper-service/src/tests/setup.rs",
  "temper/crates/temper-service/src/tests/setup_tokens.rs",
  "temper/docs/design/protocol/forge.status.md",
  "temper/docs/design/protocol/setup.status.md",
  "temper/docs/design/protocol/credentials.status.md",
]
effort = "high"
minutes = 150
+++

# The routing runs a connected Forgejo's Check, keeps its tokens and connects as its user

## Goal

The meeting point: a `Connect` with a Forgejo's pasted tokens, admitted by the forge domain
(`forge-connect`), runs skein's `Check` in the setup host; on `Done` the routing keeps the tokens,
the secret and the `Origin`, tells the forge domain the login, erases the carried row and connects
the forge from its records as that user, as `notes/contracts.md`, C4 orders it; on `Stop` it keeps
nothing and names why in C5's words. One story runs it from the wire's bytes to `Connected`.

Acceptance:

- `route.rs`'s `How::ForgejoTokens` arm (the interim refusal since `forge-connect`) calls a new
  `setup::public::tokens_connect(engine, forge, request, carried)`, modelled on `connect`
  (`public.rs:117-156`): `full()` answers `Busy`; else `ConnectAnswered::Connecting`, the carried
  row's `Get` and digest check ("the pasted value changed"), then the bundle decoded with
  `decode_pasted`: exactly `ApiToken` and `GitToken`, and `WebhookSecret` or not, each nonempty,
  else the run stops before any plan, `the pasted values are not a Forgejo's tokens` (another
  version, `PastedError::Version`'s words); then a `Caller::Connect` holding the items (a new
  `Pasted::Tokens` variant) and `Plan::Check { forge }` staged.
- `read_item` (`setup.rs:782-830`) serves `ApiToken`, `GitToken` and `WebhookSecret` from a
  `Pasted::Tokens` caller's items first, else from the forge's rows (never the legacy file
  token); `User` from the rows. `lifecycle`'s `Kind` gains the Check's kind, so `kind()`
  (`:267`), `done`, `stopped` and `writes_failed` take it.
- `Done(Outcome::Identity { id, login })` (C4, steps 2 to 5): each pasted item kept in the order
  `ApiToken`, `GitToken`, `WebhookSecret`, each `Kept` before the next, then `Origin` (Forgejo,
  the record's `api` address); then `Event::BotLogin { forge, login }` and
  `Event::ConnectEnded { connected: true }`; at the root's `EraseCarried`, the erase, and at its
  `Erased` the connect: `lifecycle::connect` (`:602-616`, GitHub's alone today) builds a
  Forgejo's access from its records (`records::access(Kind::Forgejo, ..)` with the webhooks'
  listener's setting), puts the `GitToken` value in the identities' table at its next
  generation (`forge::tokens::identity`), then `forge::connect`. The component's `Connected`
  reaches the root as `Connected { id, login }`, the login the record's bot's.
- `forges::record` (`forges.rs:91`): a kept forge's record that differs from the one held only in
  its bot's login takes the new login into the identities' table in place, and neither replaces
  the forge's endpoint nor connects it (today a changed identity re-adds the forge and connects it
  from its records at once, which would connect before the carried row's erase); every other
  change goes on as today.
- `Stop` (C4, step 6): nothing more kept, the carried row erased, `ConnectEnded { connected:
  false, why, stop: None }`, `why` in C5's words for a tokens caller (the record's bot login in
  the `api`/`OtherUser` words).
- Tests (a new `src/tests/setup_tokens.rs`, registered in `src/tests/setup.rs` as `#[path =
  "setup_tokens.rs"] mod tokens_tests;`, beside `admin_tests`), against skein's fake
  Forgejo as `tests/setup_create.rs`'s `Forge` runs it, with a forge made by the forge domain's
  admission as `tests/setup_admin.rs`'s `ask` makes one, `mode: Connected`:
  - **the story through the wire:** a tokens connect of a new `connected` forge, its request's
    bytes made with `temper-client-protocol-wire` and its value with
    `temper_client_domain::forge::forges::pasted_tokens`, sent through the service's client path
    as `tests/setup_github.rs`'s `http_paste` and `connect_request_with_policy` send a paste:
    `Check` against the fake user; `User`, `ApiToken`, `GitToken`, `WebhookSecret` then `Origin`
    kept in that order; the record's bot login the fake user's; the carried row erased;
    `Request::Connect` with the Forgejo access; the identity's value at generation 1 before it;
    the root told `Connected { id, login }` with the fake user's id and login; and no token or
    secret in any event, note or log line (`setup_admin.rs`'s `scan`);
  - a kept connected Forgejo, stopped, connected again by tokens of the same user: connected,
    each pasted row kept over the old;
  - tokens of two users: stopped, `the git token answers as another user than the API token`,
    nothing kept, the carried row erased, the forge as it was;
  - a site admin's API token: stopped with the `BotIsAdmin` words, nothing kept;
  - tokens of another user than the kept `User` on a kept forge: stopped with the `api`,
    `OtherUser` words naming the record's login, nothing kept, the forge as it was;
  - a refused API token (the fake answering `401`): `Forgejo refused the API token`;
  - a bundle without the git token, and one holding an `AdminToken`: stopped before any plan, `the
    pasted values are not a Forgejo's tokens`, nothing staged;
  - a restart after the connect: the forge connects from its records at start as the record's
    login (`tests/forge_runtime.rs:87`'s pattern).
- Status, as `notes/common.md` says: in `temper/docs/design/protocol/forge.status.md`, the entry
  that begins `2.2, 2.6: a connected Forgejo:` reads `2.2, 2.6: \`Replace\` of a connected
  Forgejo's tokens or of a GitHub App's key (\`temper-service/src/route.rs\` refuses it \`not
  taken\`).`; in `setup.status.md`, `4: a connected Forgejo: ... (no \`Replace\` request).`
  (through the 17.3 sentence) reads `4: a connected Forgejo's \`Replace\`, and 17.3: a pasted
  App's (\`temper-service/src/route.rs\` refuses it \`not taken\`).`; in
  `credentials.status.md`, `2.1, 2.2, 5: a connected Forgejo's tokens, connected, connected again
  by its kept user alone, and replaced; and a GitHub App's key replaced (no request carries
  them).` reads `2.2, 5: a connected Forgejo's tokens and a GitHub App's key, replaced
  (\`temper-service/src/route.rs\` refuses \`Replace\` \`not taken\`).`

## Inputs

- `temper/docs/plan/forgejo-connect/notes/common.md`; `notes/contracts.md`, C1, C4 and C5.
- `temper/docs/design/protocol/forge.md`, 2.1 (the records' load and the access), 2.2 (a
  connected Forgejo's `Check`; a forge's user stays), 2.5 (Forgejo's value and its generations)
  and 2.6 (the plans' table, pasted values, a run's `Done`).
- `temper/docs/design/protocol/setup.md`, section 4; `protocol/credentials.md`, 2.1 and 2.2.
- skein's `docs/design/forge-setup.md`, 4.6 (the `Check` plan); its code,
  `skein/crates/skein-forge-setup/src/check.rs`.

## Owns and may change

- `temper-service/src/route.rs`: the `How::ForgejoTokens` arm alone; `Request::Replace` keeps its
  interim refusal (`replace-run` stages it).
- `temper-service/src/setup/public.rs`: `Pasted::Tokens`, `tokens_connect`, the tokens caller's
  stop words, and `ended`'s handling of a tokens caller (`matches_app` and `pastes`, `:248` and
  `:305-319`, taking it as they take an App's, so that the connect follows the erase).
- `temper-service/src/setup.rs`: `read_item`, `item_record` if it needs an arm, and the run's
  gates only as the Check's kind needs.
- `temper-service/src/setup/lifecycle.rs`: the kind, `done`'s `Identity` arm (`:433-437`), the
  keeps' purposes, `stopped`, `writes_failed`, `connect` made kind-generic, `connected` only if
  the login needs it.
- `temper-service/src/forges.rs`: `record`, as the acceptance says.
- `temper-service/src/tests/setup.rs` (the module's line) and the new `tests/setup_tokens.rs`. Helpers in
  `tests/setup_admin.rs`, `setup_create.rs`, `setup_github.rs` or `fixtures.rs` that the new
  file needs are made `pub(super)` or moved to `fixtures.rs`, nothing else of them changed.
- The three status files, as the acceptance says.

Pre-approved:
- If skein's fake Forgejo cannot hold a user that is not an admin with two tokens, or answer `401`
  for one, script those answers as `tests/setup_lifecycle.rs`'s `Face` scripts GitHub's, for the
  cases that need it, and say which in the report.
- If a GitHub paste's tests (`setup_lifecycle.rs`, `setup_github.rs`) need a call site's edit
  for `connect` made kind-generic, edit only that call.
- A skein fault found here is a question, never a skein edit.

## Steps

1. `tokens_connect`, `Pasted::Tokens`, `read_item`, the kind: the bundle's refusals' tests:
   `heavy cargo nextest run -p temper-service --lib -E 'test(tokens_tests)'`.
2. `Done`'s keeps, `BotLogin`, the erase and the generic connect; `forges::record`; the story
   through the wire and the stops, the same run; then `-E 'test(lifecycle_tests) |
   test(admin_tests) | test(github_tests) | test(forge_runtime) | test(family)'` (the setup
   tests nest under `setup::tests`: `lifecycle_tests`, `admin_tests`, `github_tests` inside
   `lifecycle_tests`).
3. The status edits; the citations check.
4. `cargo fmt`; old forms for temper; `heavy cargo clippy -p temper-service --all-targets -- -D
   warnings`; `heavy cargo nextest run -p temper-service`; `heavy cargo check --tests` for
   `temper` and `temper-one-process-world` (`temper/tests/one-process`), which run the service.
   Commit the code and tests, then the status text; push.

## Report

As the implementer's text says, with how the fake Forgejo was given its users and tokens, and
the order of the keeps as the story observed it.
