+++
id = "fr-skein-connection"
part = "forge-records"
kind = "feature"
needs = []
files = [
  "skein/crates/skein-forge-connection/src/**",
  "skein/tests/forge/protocol/**",
  "skein/tests/forge/machine/**",
]
effort = "high"
minutes = 60
+++

## Goal

skein's forge connection component takes every forge at its start and
ignores a request for any other (`skein/docs/design/forge-connection.md,
3.1` and `3.5`). temper now makes forges at runtime
(`temper/docs/design/draft/playground-reset.md, 3`): add the generic
mechanism here first, a request that adds one forge to a running
component, checked as start checks a forge, then started as start
starts one.

Acceptance, new unit tests in `skein-forge-connection`
(`src/tests/add.rs`):

- `an_added_forge_serves_calls_once_connected`: a component started
  with forge 1 is asked `Add` for forge 2 (Forgejo, plaintext loopback,
  no access); it answers `Added { forge: 2, refused: None }`; a call to
  2 is answered `NotConnected` at once, cost 0; after `Connect { forge:
  2, access }` a call to 2 is served as 1's are.
- `an_add_past_the_limit_or_of_a_held_number_is_refused`: with
  `forges` = 1, an `Add` is refused naming the limit; an `Add` of
  forge 1 is refused naming the number; neither changes forge 1.
- `an_added_forge_with_access_starts_at_once`: `Add` with an access
  runs the forge's start (3.3) as a forge handed at start with access.
- `an_added_forges_hooks_are_heard` (if the webhooks' listener serves
  each forge's path from the configured set, 6.1): a delivery to forge
  2's path after its `Add` and `Connect` is verified and goes up.
- The component's existing tests pass unchanged.

## Inputs

- `temper/docs/plan/playground-reset-r2/notes/common.md` and
  `temper/docs/plan/playground-reset-r2/notes/scope.md`.
- `skein/AGENTS.md`; `skein/docs/foundation/programming-model.md`,
  section 1 and section 10 (step crates).
- `skein/docs/design/forge-connection.md`, sections 2.1 (what crosses),
  3.1 to 3.5 (start and access), 6.1 (each forge's webhook path), 7.2
  (limits).

## Owns and may change

Owns `skein/crates/skein-forge-connection/src/**` for this task. May
change: the skein forge worlds (`skein/tests/forge/protocol`,
`skein/tests/forge/machine`) and any owner's exhaustive match on
`skein_forge_connection::Event` or `Request` in any tree (temper's
`temper-service` matches `Event` in `src/engine.rs` and `src/forge.rs`:
an `Added` arm that does nothing, commented "fr-connections adds forges
at runtime"), as `notes/scope.md` says. The design document is not
yours: what the code taught goes in the report.

## Steps

1. In `src/boundary.rs`:

   ```rust
   // Request
   /// Adds one forge to the running component, checked as start checks
   /// each forge (3.2), then started as at start (3.3): with no access
   /// it runs no start until its `Connect`.
   Add { forge: settings::Forge },
   // Event
   /// `Add`'s one answer; `refused` names the value start would refuse.
   Added { forge: u16, refused: Option<Refused> },
   ```

   `Refused` is the type start's check refuses with (read `start.rs`;
   if start refuses with a `Problems`-like list, carry the first
   refusal's value as start names it, and say which in the report).
2. The check is start's for one forge, against the forges held: its
   number unique, the count at most `forges`, an address never
   in-process, plaintext only to loopback, its job log origins within
   their bounds. Factor start's per-forge check into one function both
   call, so they agree.
3. The added forge joins every per-forge table start sizes (its
   connection, its installations' room, its webhook path); tables sized
   at start by `forges` already have its room. After `Close` or
   `Abort`, an `Add` is ignored, as a `Connect` is (3.5).
4. Tests (Goal); `cargo fmt`; `heavy cargo clippy -p
   skein-forge-connection --all-targets -- -D warnings`; one at a time:
   `heavy cargo nextest run -p skein-forge-connection`, then `-p
   skein-forge-protocol-world -p skein-forge-machine-world`, then
   `heavy cargo check --tests -p temper-service -p temper-one-process-world`;
   `python3 -I -B scripts/old_forms.py` from `skein/`.
5. Commit and push.

## Report

The standard report, plus: the refusal type `Added` carries; which
start tables the added forge joins; what forge-connection.md 3 should
say of a forge added at runtime (for its designer).
