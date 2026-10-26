# Supplement temper-04.s1: the repin onto jig's whole authorities and busy watches, and a stream's room

Added by ruling 319 (the designer's answers of 2026-10-18T05:52:56Z and
06:20:06Z). Read first: temper's `AGENTS.md`,
`docs/development/workflow.md`; this plan's README, "Rules of this
plan" and "Pins"; `temper/04-engine.md`, the corrections after the
designer's answers of 2026-10-18T05:52:56Z and 06:20:06Z; temper's
`configuration.md`, 7; `client.md`, 5.4.

Scope: temper repins to jig-17.s1 (an authority crosses whole) and
jig-17.s2 (a busy watch is opened again), and gives a client's stream
the watch room configuration.md, 7 asks, 16.
Needs: jig-17.s1's and jig-17.s2's landings (README, "Pins": its pin is
the later of the two, or temper's present pin if it holds both).

Shape:
- **The client's authority bound, 12** (the process tier's
  `authority.grants` and `tasks.authority_grants`,
  `crates/temper/tests/support/forgejo.rs:186`, `:190`), reaches the
  `authority` jig-17.s1 adds to `jig_client_domain::Limits`, where
  temper builds it: `crates/temper-client-domain/src/limits.rs:56-79`
  (`client()`, `segments: 9` at `:67`). It is set in the terminal's
  `native()` (`crates/temper-client-shell-terminal/src/limits.rs:24`):
  in the view's `jig_client_view::Limits` (`:28`) if jig-17.s1 gives it
  an `authority`, else in `temper_client_domain::Limits`, beside `jig`.
  Each further literal of the struct that gains it takes a value at
  least its engine's grants: `crates/temper-client-view/src/tests/fixtures.rs:10`,
  `crates/temper-client-domain/src/tests/fixtures.rs:13`,
  `tests/client/src/fixtures.rs:25`.
- `crates/temper/tests/configuration.rs`: three assertions after the
  paths' (`:29`): the engine's `documents.grants`,
  `documents.executors` and `documents.implications` each at most the
  native client's `authority`. They must see the process tier's 12:
  with `Forge::Unreached` (`:6`), `support::configuration` returns
  before those overrides (`forgejo.rs:142`), so the test takes them on
  its own document or they move above the return; the commit body says
  which.
- **The engine's document bounds, copied in temper's worlds:** jig-17.s1
  adds `grants`, `executors` and `implications` to
  `jig_domain_core::documents::DocumentLimits`, derived. The literal at
  `tests/engine/domain/src/walking.rs:124-133` takes them from
  `authority_limits()` and its tasks' bounds, as jig derives them;
  `tests/engine/domain/tests/forge_routes/paths.rs:85-92`
  (`document_limits`, which copies jig's `segments` derivation; called
  at `forge_routes.rs:302`, whose world sets grants 10 and executors 2)
  derives the three the same way. Refresh any further `DocumentLimits`
  literal or copy of the derivation (`git grep -n -e DocumentLimits -e
  'documents.segments' -- crates tests`; `tests/one-process/src/fixtures.rs:56-99`
  is its own tier), and list it in the commit body.
- `tests/engine/domain/tests/forge_routes/paths.rs`:
  `a_change_on_the_forges_deepest_branch_is_served_in_its_task_document`
  first reads the producer's task document as the world's owner
  (`policy.roles[0]`), before `sign_in_reader` (`:38`) takes its
  session, and asserts it served with `Act::Amend`'s admission
  `Admission::Bounded`; the reader's read stays.
- If the start refuses memory, `engine.memory_bytes` rises by what
  the refusal names.
- **One watch kept for `me`** (jig-17.s2): `temper-client-domain/src/limits.rs:61-62`
  give the client `handles: jig.watches` and `watches: jig.watches`;
  each takes one more, `checked_add(1)?`, as jig's view fixture does
  (`jig-client-view/src/tests/fixtures/client.rs:35`, `:47`, `:68`), so
  that the application keeps its 16; the terminal's
  `temper-client-shell-terminal/src/limits.rs:29` derives from it.
- **A stream's room:** a temper constant of 16 beside `VIEWS`
  (`crates/temper-engine-domain/src/limits.rs:103`), both exported from
  `lib.rs:39` (`mod limits` is private, `:15`), set as the clients
  component's `watches_per_stream` once, where temper reads its
  settings, so that the component's `Env`
  (`crates/temper-service/src/clients.rs:112`, from
  `config.limits.clients`) and its memory sum (`system.rs:576`, through
  `clients::worst_case`, `clients.rs:331`, which feeds
  `startup/checks.rs:305`) both see it; the service fixture's
  `watches_per_stream = 4` (`src/tests/fixtures.rs:436`) goes. In
  `crates/temper/tests/configuration.rs`, three assertions beside the
  documents' (`:4`): that constant, `VIEWS.watchers_per_party` and
  `VIEWS.watchers` each at least `native.watches`.
- jig-17.s2 changes no temper code: its repin repairs are what jig
  landed, as README, "Pins" says.

Proof: the world layer, once: `cargo nextest run -p
temper-engine-domain-world -E 'test(a_change_on_the_forges_deepest_branch_is_served_in_its_task_document)'`
(the owner's read served), and the gate.
Size: at most 20 production lines; the repin's repairs counted apart
(README, "Pins": at most 200).
Don't: work around a refusal or a busy `me` in temper's code; set
`documents.lines` for an authority.
