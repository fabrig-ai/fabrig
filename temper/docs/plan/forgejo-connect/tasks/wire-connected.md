+++
id = "wire-connected"
part = "wire"
kind = "interface"
needs = ["pasted-list", "forge-surface"]
files = [
  "temper/crates/temper-client-protocol-wire/**",
  "temper/crates/temper-service/src/family.rs",
  "temper/crates/temper-service/src/tests/family.rs",
  "temper/crates/temper-client-domain/src/forge/forges.rs",
  "temper/crates/temper-client-domain/src/tests/form.rs",
  "temper/crates/temper-client-domain/src/tests/connect.rs",
  "temper/crates/temper-client-tools/src/forges.rs",
  "temper/crates/temper-client-tools/src/forges/lifecycle.rs",
  "temper/crates/temper-client-tools/src/forges/connection.rs",
  "temper/crates/temper-client-tools/src/tests/forge_connection.rs",
  "temper/crates/temper-client-tools/src/tests/fixtures/adopt.rs",
  "temper/crates/temper-client-tools/src/tests/forges.rs",
  "temper/crates/temper-client-shell-terminal/src/tests/family.rs",
  "temper/crates/temper-client-shell-cli/src/tests/fixtures/setup.rs",
  "temper/docs/design/client/README.status.md",
]
effort = "high"
minutes = 90
+++

# The wire carries a connected Forgejo's tokens connect and a Replace

## Goal

Put `notes/contracts.md`, C2 on the wire, in place at `CLIENT_VERSION` 11, and map it to the forge
domain's C3 in the service, so that a client's tokens connect or Replace reaches the forge
domain's admission (which refuses both `not taken` until `forge-connect` lands). Give the client
domain the one bundle helper its two consumers (ctl, the web page) share, and the refusal words a
Replace needs. Every client crate keeps sending what it sends today.

Acceptance:

- `client-wire-v1.schema` holds C2's four changes, each new variant last in its enum; the
  generated code and goldens are regenerated; the drift test passes; `CLIENT_VERSION` stays 11.
- `decode_requesting` takes a `connected` new forge on the same checks as `managed`'s, a
  `forgejo_tokens` connect as it is, and a `replace` as it is.
- The service's `family.rs` maps `ForgeAsk::Connected(m)` to the forge domain's
  `ForgeAsk::Connected { place, git_program }` as `Managed`'s, `How::ForgejoTokens` to
  `How::ForgejoTokens`, and `Request::Replace(r)` to `Ask::Replace { forge: r.forge() }` with its
  carried value and the deployment's scope, as a `Connect`'s; a Replace's answer goes out as a
  `Connect`'s (`connecting`, `connect_refused`, `unknown`).
- `temper-client-domain`'s `forge/forges.rs` gains

  ```rust
  /// A connected Forgejo's pasted tokens and hooks' secret, one bundle
  /// (protocol/credentials.md, 2.2): ApiToken, GitToken, WebhookSecret, each when given.
  #[must_use]
  pub fn pasted_tokens(api: Option<&[u8]>, git: Option<&[u8]>, secret: Option<&[u8]>) -> Option<skein_lib::Secret>;
  ```

  (codes 2, 3 and 5 in that order, an empty value as none; `None` when nothing is given or the
  bundle is past its bound), and `connect_refusal`'s words become: `connected`, `connected
  already: forge replace replaces its key or tokens`; `not_connected`, `not connected: connect it
  first`.
- Tests: the wire's own (`src/tests/`), as `forgejo_admin.rs` does for the admin's connect: a
  tokens connect of a new `connected` forge and of a kept one, and a Replace, each round-trips,
  with a golden of each (`connect_forgejo_tokens_new.bin`, `connect_forgejo_tokens_kept.bin`,
  `replace.bin`); a `connected` forge with an empty host or port 0 is refused as `managed`'s is.
  The service's `tests/family.rs`: `new_forge_places_reach_both_setup_and_connect_whole`'s table
  gains `connected`, and a tokens connect and a Replace reach the root as their asks with their
  carried values. The client domain: `pasted_tokens` of (api, git), of (api, git, secret), of
  git alone and of nothing, decoded back with `decode_pasted`.
- `temper/docs/design/client/README.status.md`'s entry that begins `3.5, 4.6, 8: a connected
  Forgejo's \`Connect\`` reads, word for word: `3.5, 4.6, 8: a connected Forgejo's \`Connect\`
  with its tokens and Replace, on the Forges page (\`temper-client-domain\` sends neither); and
  Forget, on the Forges page and as the FORGE request \`Forget\`: no \`forget\` in
  \`temper-client-protocol-wire\`'s schema or in \`temper-service\`'s routing.`

## Inputs

- `temper/docs/plan/forgejo-connect/notes/common.md`; `notes/contracts.md`, C1 to C3.
- `temper/docs/design/client/README.md`, 4.6 (`Making`, `Connect`, `Replace`, `ForgeRefusal`)
  and section 8 (the FORGE requests' table).
- `temper/docs/design/protocol/credentials.md`, 2.2 (which items a tokens connect carries).

## Owns and may change

- `temper/crates/temper-client-protocol-wire/**`: the schema, its regeneration
  (`notes/common.md`), `src/lib.rs`'s `decode_requesting` and `valid_making`, and `src/tests/`
  (a `forgejo_tokens.rs` beside `forgejo_admin.rs`, its goldens under `golden/`). If skein's
  codegen refuses `record ForgejoTokens { }`, make `forgejo_tokens` a unit variant, as C2
  pre-approves, and say so in the report.
- `temper-service/src/family.rs`: `to_root`'s request match (`:47`, `:91-173`), its `How` match
  (`:95`), `making` (`:1125-1146`); `answer` (`:527`) only if a Replace's answer needs an arm.
  `tests/family.rs` as the acceptance says.
- `temper-client-domain/src/forge/forges.rs`: `pasted_tokens` and the two refusal words;
  `tests/connect.rs` (or the test file that holds `connect_refusal`'s tests) for the helper and
  any assertion of those words; `tests/form.rs:87-97` and `:200-210`, the forced arms.
- Forced arms, each as the nearest arm of its kind and nothing more: `temper-client-tools`'
  `src/forges.rs` (`restore`, `:~575-605`), `forges/lifecycle.rs:70-75`, `forges/connection.rs`
  (`:300-313`, the `Request` match, and `told`, `:304-307`, `How::ForgejoTokens` told as
  `ForgejoAdmin` without `first` is), `tests/forge_connection.rs:860-864`,
  `tests/fixtures/adopt.rs:~80-90`, `tests/forges.rs:~1316`;
  `temper-client-shell-terminal/src/tests/family.rs:19-29`;
  `temper-client-shell-cli/src/tests/fixtures/setup.rs:25-50`. A `Replace` is treated as a
  `Connect` whose value is carried. ctl offers neither yet: its own task does.
- Readers of the two refusal words: any test or golden in `temper-client-tools` that asserts
  `connected already: forge-replace replaces its key` or `not connected: connect the App first`
  takes the new words (a golden regenerated with `SKEIN_CODEGEN_WRITE=1` on its test); list
  them in the report.

Pre-approved: any other exhaustive match over the wire's `Request`, `How` or `ForgeAsk` the
compiler names in temper's crates or worlds gets its forced arm and is listed in the report.
Nothing in `temper-client-view` changes; `temper/tests/client/src/front.rs`'s `matches!` needs
none.

## Steps

1. The schema, its regeneration and `lib.rs`; the wire's tests: `heavy cargo nextest run -p
   temper-client-protocol-wire` and `-p temper-codecs-world --test suite -E 'test(drift::) |
   test(client_wire)'`.
2. The service's mapping: `heavy cargo nextest run -p temper-service --lib -E 'test(family)'`.
3. The client domain's helper and words; the forced arms: `heavy cargo nextest run -p
   temper-client-domain --lib`, then `-p temper-client-tools --lib`, `-p
   temper-client-shell-terminal --lib`, `-p temper-client-shell-cli --lib`.
4. The status edit; the citations check.
5. `cargo fmt`; old forms for temper; `heavy cargo clippy` naming every package changed,
   `--all-targets -- -D warnings`; their focused tests; `heavy cargo check --tests` for
   `temper-client-view`, `temper-client-world` and `temper-client-shell-web`. Commit the code and
   tests, then the status text; push.

## Report

As the implementer's text says, with the regenerated files' `git diff --stat`, the
`ForgejoTokens` shape the schema took, and every forced arm and words' reader.
