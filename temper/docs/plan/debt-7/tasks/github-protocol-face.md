+++
id = "github-protocol-face"
part = "fake-github"
kind = "migration"
needs = ["github-fake-surface"]
files = ["skein/tests/forge/protocol/src/github.rs", "skein/tests/forge/protocol/src/github_fixture.rs", "skein/tests/forge/protocol/src/world.rs"]
effort = "high"
minutes = 120
+++

# fake-github: the protocol world's face drives the fake's server

## Goal

`skein/tests/forge/protocol/src/github.rs` stops playing `skein_http::server` by hand: each `Peer` holds a
`server::Server`, each tick feeds it from the pipe and runs `inline::pass`. `github_fixture.rs` loses its `service`
tables to `fixture`, and with them the comment that says the machine world's service and domain are private: a
fixture in the fake is what every world reads. Every GitHub story answers as at base, with the same accounting.

Acceptance:

- `skein/tests/forge/protocol/src/github.rs` has no `Outcome`, `http::Server`, `http::Event`, `fn serve`, `fn response`
  or `fn continue_call`.
- `skein/tests/forge/protocol/src/github_fixture.rs` has no `Row {`, `Account {`, `App {` or the comment
  "The machine world's service and domain are private".
- No file of `skein/tests/forge/protocol/tests` changes.
- A lost creation, an ambiguous update and every minted token behave as the steps below say, shown by the
  stories that read them: `a_github_creation_with_a_lost_answer_is_found_after_restart`,
  `github_ambiguous_update_conflict_reaches_the_owner_on_both_attempts`,
  `github_calls_costs_and_credentials_survive_cuts_renewal_and_replacement`.
- Request lines compared (the note, 3) for `github_listings_follow_another_partys_changes_across_pages`,
  `github_watched_absent_branch_resolves_repository_once` and
  `github_ambiguous_update_conflict_reaches_the_owner_on_both_attempts`: equal.
- Net lines at most -160.

## Inputs

- `temper/docs/plan/debt-7/notes/common.md`; `temper/docs/plan/debt-7/notes/fake-github.md`, whole.
- `skein/docs/design/fake-forge.md`, 10.6.
- `skein/docs/development/debt/D029-fake-github-http-faces.md`.
- `skein/tests/forge/protocol/src/world.rs`, `driver.rs`, `referee.rs`: `Accounting`, `github_ambiguous_updates`,
  `secrets_absent`, `github_credentials`.
- `skein/tests/forge/machine/src/pipe.rs`: `Pipe`'s rules for `Down`.

## Owns and may change

- Owns: `github.rs`, `github_fixture.rs`.
- `world.rs`: `github.ambiguous_updates = true;` becomes a call of `Github::ambiguous_updates`, that line.
- **Keeps** `access`, `settings`, `limits(log_poll)`, `PATH`, `USER` and `key()` in `github_fixture.rs`. `key()`'s body
  is `skein_fake_forge_github::fixture::key()`, so this crate stops reading `skein_forge_machine_world::github`. If
  `fixture` is ambiguous with `crate::github_fixture as fixture`, alias the fake's.
- **`service(seed, key)`** stays a function here, a few lines: `Config { app: App { permissions: wanted(), key,
  ..fixture::app() }, ..fixture::config(fixture::widgets(2)) }`, over a new `service_limits()`, `install_one` at
  `Wall::EPOCH`. Its values stay: `http` and documents from `fixtures::fake_limits()`, installations `{1, 8, 1}`,
  pages `{1, 2048}`, page 2, body 4096, response 16384, the rest as the fixture's. The person and a second
  account come from the fixture; no story reads them.
- **The `Server`'s `Limits`** are `service_limits()`: the same `http`, so its demands are the base's.
- **`Close` and `Closed`:** `pass` ignores them, as this face ignored `http::Event::Failed`: the pipe carries the
  fault to the client, and `end` is the world's.

## Steps

1. `Peer` loses `server: http::Server`, `head`, `body`, `reply`, `offset`, `action`; gains `server: Server`, `above:
   Queue<Event>`, `lost: bool`. `io`'s `Connect`: `Server::new(owner, &limits)`, `server::start`. `busy` adds
   `server.has_work()`.

2. `tick`, per peer, in the base's order: drain `below` into the pipe; one `server_up`, through `peer.received`
   then `server::up`; then, in place of `server_events`:

   ```rust
   for _ in 0..HEADS {
       inline::pass(&mut peer.server, &mut self.service,
           &mut Lent { domain: &mut self.domain, env: &self.fake_env, hooks: true }, env, &mut peer.above, &mut peer.below);
       self.take_minted();
       let Some(call) = peer.server.take_head() else { break };
       // trace.served and peer.entrance, as at `http::Event::Call`; `delayed`, below.
   }
   ```

3. **`release_hooks`** is `self.service`, `inline::release`, and keeps its two call sites' timing: the head of `tick`,
   and ahead of each call, which `hooks: true` does. **`direct`** is `inline::release` then `inline::call`.

4. **A lost creation.** The face withheld `Respond`; the server sends it. At the head: `peer.delayed =
   self.lose_creation && method is POST && target ends with /issues`, `self.lose_creation &= !peer.delayed`, as at
   base. The request runs, so the write lands. Draining `below`, from the first `Down::Send` of a delayed peer on,
   every `Down` is dropped (`peer.lost`): demands for its body pass, and the client hears nothing, as at base.
   Don't hold the pipe instead: `has_work` would keep the world turning at `TURN` to the call's deadline.

5. **An ambiguous update.** `Github::ambiguous_updates` calls `Service::ambiguous_updates`; the rewrite of a `422`
   goes.

6. **Minted tokens.** `take_minted` runs after each pass: while the store's `minted()` passes the count seen, push
   `minted_token(n)` to `secrets` and add one to `mints`. So a token is held the turn it is minted, its reply cut or
   not, as at base. `rotate` resets the count seen, with the new service. Don't scan response bytes: a cut would
   lose it.

7. **`trace.own`.** The face refined it at `serve`, from the whole body: `/graphql` is the component's own only for
   the viewer query. Do it in `Peer::received`, on the delivered bytes, when a `/graphql` body's last byte arrives:
   that is the tick `Body(End)` fired, and the index is `delivered_heads - 1`. A body cut short leaves `own` as
   `sent` set it, as at base.

8. `cargo fmt`; `python3 -I -B skein/scripts/old_forms.py`; `heavy cargo clippy -p skein-forge-protocol-world
   --all-targets -- -D warnings`; `heavy cargo nextest run -p skein-forge-protocol-world`; `heavy cargo nextest run
   --profile fuzzy -p skein-forge-protocol-world`, whose
   `github_calls_costs_and_credentials_survive_cuts_renewal_and_replacement` is the sweep the note warns of.
   Citations. Commit and push.

## Report

The common note's items; the request lines' comparison; the sweep's `cuts` at base and on the branch; the package's
nextest totals, base and branch; net lines. No new test, so no break.
