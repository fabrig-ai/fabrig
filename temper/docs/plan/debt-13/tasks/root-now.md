+++
id = "root-now"
part = "root-now"
kind = "feature"
needs = ["root-charter"]
files = ["jig/crates/jig-domain-hub-link/src/**", "jig/examples/ops/domain/src/route.rs", "jig/examples/ops/domain/src/link.rs", "jig/testing/jig-test-domain/src/routes.rs"]
effort = "high"
minutes = 60
+++

# The link's `Now`s are the link's, and `ops`'s link reads shorter

## Goal

Each root turns six of the core's `Now`s into link events by hand. `jig-domain-hub-link` says which they are, as `down`
already does for `Held`. And `ops`'s `link.rs` is what an application reads and copies: it spells one piece twice and
one assert six times.

Acceptance:

- `rg -n 'Now::(DropPayload|TurnPayload|AnswerPayload|AcceptedTurn|RefusedPayload)\s*\{' jig/examples/ops/domain/src/route.rs
  jig/testing/jig-test-domain/src/routes.rs` shows no arm body: at most the pass-through list.
- `rg -c 'host::Piece \{' jig/examples/ops/domain/src/link.rs` reads 1 (2 at base).
- `rg -c "ops engine's permanent link" jig/examples/ops/domain/src/link.rs` reads 1.
- `route.rs` is at most 1,400 lines at the end, and `link.rs` at most 215 (246 at base).
- `heavy cargo nextest run -p jig-domain-hub-link -p jig-test-domain -p jig-ops-world -p jig-core-world -p
  jig-conformance -p jig-test-service` passes.

## Inputs

- `temper/docs/plan/debt-13/notes/common.md`
- `temper/docs/plan/debt-13/notes/contracts.md`, 4
- `jig/docs/development/debt/D004-ops-root-jig-wiring.md`
- `jig/docs/design/domain/root.md`, 5, 6 and 12
- `jig/docs/design/README.md`, 6.7

## Owns and may change

- **Owns:** `jig-domain-hub-link/src/`, `now` and its test.
- **May change:** `ops`'s `route.rs` (`core_now`) and `link.rs`; the test root's `routes.rs` (`route_now`).
- **Not to touch:** `Now::RelayBusy`, `Now::DropCall`, `Now::RunPrepared` and the `Held` arms, which differ between the
  roots: moving them changes what a test sees (`D004`'s rewrite names them). The test root's `link::answer`, which its
  tests call. Its `domain.now` queue.
- **Options:**
  - The test root's `DropPayload` at base calls `take_payload`, and its `AnswerPayload` passes `Time::ZERO`. The link's
    `step` does the same for `DropPayload`, and reads `now` only for a turn's backoff: both are the same through `now`.
    If a test says otherwise, stop and ask.
  - If an `ops` arm of `Now::Relayed` is held and not now, it stays. At base it's `false`, and so in `now`'s set.

## Steps

1. `jig-domain-hub-link`: `now` in `down.rs`, beside `down`, in its shape:

   ```rust
   /// The core's `Now`s that are the link's; any other comes back (domain/root.md, 5).
   /// # Errors
   /// Returns any other output without consuming it.
   #[expect(clippy::result_large_err, reason = "an untranslated output returns to its root without another allocation")]
   pub fn now(value: core::Now) -> Result<Event, core::Now>;
   ```

   `Relayed`, `DropPayload`, `TurnPayload`, `AnswerPayload`, `AcceptedTurn` and `RefusedPayload` become their `Event`s,
   and every other variant comes back. A test: each of the six, and one that comes back unchanged. Break a mapping by hand and
   see it red; restore.
2. `core_now` and `route_now` begin as `core_held` does with `down`:

   ```rust
   let value = match jig_domain_hub_link::now(value) {
       Ok(event) => {
           crate::link::route(domain, env, decision, event, false, work);
           return;
       }
       Err(value) => value,
   };
   ```

   The six arms go, and their variants join the exhaustive pass-through list, as `core_held`'s does for what `down` takes.
3. `ops`'s `link.rs`: `agent_content` builds its `link::Event::Up` in one small `fn piece(..)`, used for the empty text
   and for each chunk. `outputs` asserts by one `fn permanent(channel: Token)`, and `Token::new(0)` as a channel is one
   `const ENGINE: Token` in that file. It's the reference root, so a comment says why the engine has one channel,
   citing `jig/docs/design/domain/hosts.md`, 6.1.
4. `cargo fmt`, `heavy cargo clippy -p jig-domain-hub-link -p jig-ops-domain -p jig-test-domain --all-targets -- -D
   warnings`, the tests above, commit, push.

## Report

What changed, the commit, what ran with counts, the rule broken and what failed, and the net lines.
