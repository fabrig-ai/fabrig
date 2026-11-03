# bugs-security: the plan

From `inputs/request.md` (the debt backfill's "Bugs", routed by the director, 2026-11-03): bugs
that leak a secret, each fixed with a test in the lowest world that shows it, failing on the old
code. Every task names `notes/common.md`; the `Debug` tasks also name `notes/debug.md`.

## The design

Written by this cut, each with its status entry, closed by the task that builds it:

- `skein/docs/design/lib.md`, 8: `CryptoRng` "is neither `Copy` nor `Clone`, since a copy would
  yield its bytes again"; its owner lends it by `&mut`, and a component that keeps one is made
  one on a stream of its own. `lib.status.md`: **Not yet** 8.
- `skein/docs/design/oauth.md`, 6.7: `Component::check` checks accounts as making the component
  does, taking no generator. `oauth.status.md`: **Not yet** 6.7.
- `skein/docs/design/io.md`, 5: "A file's contents print as their length" (`Store`, `WriteAt`,
  `Loaded`, `Read`), and, added in the third push as compatible detail, so does `FileIo`'s own
  `Debug` for the contents it holds in flight. `io.status.md`: **Not yet** 5, a clause.

Item 4's other fixes add no design text: each follows a rule the design already states,
`skein/docs/design/lib.md`, 6 (a record holding a secret "prints, logs and traces without it")
or, for the issuer, `skein/docs/design/oauth.md`, 4 ("every type carrying a token, authorization
code, verifier, state, nonce, ID token or client secret omits its bytes from `Debug`").

`skein/docs/design/STATUS.md` is not edited: its summary names oauth.md among the documents
with no gap, and so is stale for oauth.md until `oauth-accounts-check` lands. It is
`public-reading-build/public-status`'s file now, and the window is one task long.

## Parts and tasks

1. **jig-debug** (`jig/docs/design/domain/hosts.md, 5.1`), slice 1.
   - `tasks/jig-resources-debug.md`, medium, ~30 minutes, needs nothing (request item 1):
     `jig-test-service`'s `program_sources::Resources` gets temper's count-only `Debug`; tests in
     a new `src/tests/program_sources.rs` format it, and a configured `Component`, with a canary.
     Landed.
2. **crypto-rng** (`skein/docs/design/lib.md, 8`), slice 2 (request item 3).
   - `tasks/oauth-accounts-check.md`, medium, ~40 minutes, needs nothing:
     `skein-oauth-accounts`' `Component::check(accounts, limits)`, the check half of `new`, no
     generator. Done.
   - `tasks/jig-accounts-check.md`, medium, ~30 minutes, needs `oauth-accounts-check` and
     `jig-resources-debug` (one writer for `jig-test-service`): `provider_templates` checks with
     `Component::check`, no `crypto.clone()`.
   - `tasks/temper-accounts-check.md`, medium, ~30 minutes, needs `oauth-accounts-check`: the
     same in `temper-service`.
   - `tasks/crypto-rng-not-clone.md`, high, ~50 minutes, needs both: skein-lib drops the derive,
     an inherent-constant trait check holds `CryptoRng: !Clone` (failing on the old code), and
     skein-forge-setup's two oracle clones replay a fresh fixture generator.
3. **skein-debug** (`skein/docs/design/io.md, 5`), slice 3 (items 2 and 4), all needing nothing:
   - `tasks/io-file-debug.md`, high, ~90 minutes: skein-io's `file::Request` and `file::Event`
     print `Store`'s, `WriteAt`'s, `Loaded`'s and `Read`'s bytes as `len`, every other variant as
     derived; `FileIo` prints its in-flight store, read and load contents as lengths.
   - `tasks/issuer-debug.md`, medium, ~40 minutes: skein-oauth-issuer's `TokenRequest` prints
     `verifier_len`, its form's `Field` `value_len`.
   - `tasks/channel-debug.md`, medium, ~45 minutes: skein-channel's `Request::Open`,
     `Event::Opening` and `Control::Open` print `credential_len`.
   - `tasks/playground-cookie-debug.md`, medium, ~25 minutes: skein-playground's `Cookie` prints
     `value_len`.
4. **jig-holders** (`skein/docs/design/lib.md, 6`), slice 4 (item 4):
   - `tasks/jig-runner-debug.md`, medium, ~25 minutes, needs nothing: jig-test's
     `SourceSettings` prints `environment_entries`.
   - `tasks/jig-links-debug.md`, medium, ~35 minutes, needs `jig-accounts-check` (one writer for
     `jig-test-service`): `Clients` prints its printed links as `links_entries`.
   - `tasks/jig-web-taken-debug.md`, medium, ~25 minutes, needs nothing: jig-client-shell-web's
     `Taken` prints `secret_len`.
   - `tasks/jig-http-call-debug.md`, medium, ~35 minutes, needs nothing: jig-client-protocol-http's
     `Call` prints each header's name and `value_len`, and `body_len`.
5. **temper-holders** (`skein/docs/design/lib.md, 6`), slice 5 (item 4):
   - `tasks/box-settings-debug.md`, medium, ~30 minutes, needs nothing: temper-box's `Settings`
     prints `environment_entries`.

Critical path, from the third push: `io-file-debug` (~90 minutes), alone. Beside it,
`jig-accounts-check` or `temper-accounts-check` (~30, `oauth-accounts-check` done) then
`crypto-rng-not-clone` (~50), and `jig-accounts-check` then `jig-links-debug` (~35). Everything
else is under 45 minutes and needs nothing.

## Item 4's search

Two read-only surveys searched all four trees for a `Debug` printing a token, key, password or
credential's bytes. Fixed here (beside items 1 to 3): skein-io's `FileIo`; skein-oauth-issuer's
`TokenRequest` and `form::Field`; skein-channel's three opening credentials; skein-playground's
`Cookie`; jig-test's `SourceSettings`; jig-test-service's `Clients.links`; jig-client-shell-web's
`Taken`; jig-client-protocol-http's `Call`; temper-box's `Settings`.

Left for a question (`plan.toml`, none blocking a task): `byte-carriers` (skein-channel's
`Frame`, skein-http's `Header` and the other generic carriers), `issuer-authorization`
(`Authorization`'s state and nonce, where oauth-issuer.md 4 and oauth.md 4 disagree),
`link-sign-in-secret` (jig client-wire 19's `LinkSignIn.secret` declared `bytes`, a schema
change on files other drafts are changing) and `machine-environment` (smith-protocol-machine's
environment, which the design says holds no credential).

Not fixed, and why:

- **Test worlds and peers that hold only made-up credentials:** smith/tests/channel's `Meeting`,
  jig/tests/engine-clients' `Peer` and `Exchange`, jig/tests/store's `Records`, temper-box's test
  `Launch`. No real secret reaches a world (`AGENTS.md`, 3); the fixes are for code that can hold
  one at run time, the reference service and its runner among them.
- **temper-client-shell-web's journey `Line`:** journeys record the fake box's dummy link
  secret only, by design (`temper/docs/design/client/journeys.md`, 3.2 and 3.5).
- **temper-service's `startup::application::{Forge, Identity}`:** re-exported from
  `startup.rs` but never made or read anywhere; their running counterparts in `settings.rs` hold
  `Secret`. Fixing a `Debug` nothing can reach is no fix; removing them is not this request's.
- Already hidden: every `Secret`-typed field, the types that opt out of `Debug`
  (skein-oauth, skein-oauth-accounts, skein-llm, smith-local-service), and the hand-written ones
  (smith-protocol-translate's `GrantValue`, smith-protocol-program's `Launch`, skein-io's `Spawn`).
  Paths to a secret, digests, public keys and map keys hold no secret.

## What was decided, and why

- **Released first:** item 1 needs no survey: `Resources` is built through `from_paths` and
  `new` only, and nothing reads its `Debug` text (no test, trace or golden file holds it).
- **The check-only copies get a check, not a fresh stream.** The survey found four clones of a
  `CryptoRng`: jig-test-service's and temper-service's `provider_templates` give a throwaway
  accounts component `crypto.clone()` to check one provider's account, and two skein-forge-setup
  tests clone the host's generator as an oracle. `Component::new` never draws: it only checks
  and stores the generator. So the owners need no generator for the check at all; drawing a
  fresh one per provider would need an undocumented stream number for each, in processes whose
  streams are already chosen by hand (1, 5, the seed itself), for generators that never draw.
  `Component::check` sits beside `skein_oauth_accounts::valid` (oauth.md, 6.7). The two test
  oracles replay the fixture generator the host is made with, skipping what it drew.
- **The order keeps the trunk building:** the owners stop cloning before skein-lib drops the
  derive, so `crypto-rng-not-clone` needs both; each branch stays in its tree.
- **The trait check is an inherent-constant probe** in skein-lib's step tests, a control type
  beside it: nextest runs no doctests, so a `compile_fail` doctest would not run in the gate, and
  no repository has a compile-fail mechanism. The spec allows the doctest only if the probe does
  not compile, saying so.
- **One printed form:** a hand-written `Debug` keeps the derived one for every field without a
  secret, and prints a secret field as `<field>_len` or a list as `<field>_entries`, temper's
  `Resources` form (`environment_entries`); skein-io's file records print `len` in `bytes`'
  place, as io.md 5 says. A test asserts the canary absent both as text and as its bytes'
  `Debug` (a derived `Box<[u8]>` prints numbers), `notes/debug.md`.
- **`FileIo` joins item 2:** it holds the same file contents in flight, and smith-local-service's
  `HostIo` derives `Debug` over one, so fixing the records alone would leave the stored record
  printable; io.md 5 gained the sentence, compatible with its first push.
- **`Call` hides its body too:** jig-client-protocol-http's `Call` holds the bearer in a header
  and the sign-in link's secret in its encoded body; it is the record that holds both, so its own
  `Debug` hides both. Other encoded bodies are `byte-carriers`'.
- **No answer waits on the user.** The four questions shape follow-up work only.

## Overlaps with the drafts beside this one

- `jig-resources-debug`, `jig-accounts-check` and `jig-links-debug` edit files inside
  `fabrig-scale/standard-scale`'s glob `jig/testing/jig-test-service/src/**`
  (`program_sources.rs`, a new `src/tests/program_sources.rs` and one `mod` line in
  `src/tests.rs`; `accounts.rs`'s `provider_templates`; `clients.rs`' `Debug` and a test
  appended to `src/tests/clients.rs`). That task's work is the standard profile's scale, not
  these, so only the `mod` line and the appended test could meet a textual conflict.
- `jig-web-taken-debug` appends to `jig/crates/jig-client-shell-web/src/tests.rs`, and
  `jig-http-call-debug` to `jig/crates/jig-client-protocol-http/src/tests.rs`, files
  `public-reading-build/fetch-headers` and `public-reading-build/public-http` (both merged into
  their trunk) also changed; an append at the end may meet theirs.
- `temper-accounts-check` edits `temper/crates/temper-service/src/accounts.rs`, which no draft
  beside this one names. No other task touches a file another draft names.
