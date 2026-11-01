+++
id = "sk-conn-add"
part = "skein-forges"
kind = "feature"
needs = []
files = [
  "skein/crates/skein-forge-connection/src/component.rs",
  "skein/crates/skein-forge-connection/src/limits.rs",
  "skein/crates/skein-forge-connection/src/tests.rs",
  "skein/crates/skein-forge-connection/src/tests/runtime_add.rs",
]
effort = "high"
minutes = 50
+++

## Goal

skein's forge component (`skein-forge-connection`) takes a forge added
at runtime: `Component::add` checks one new forge as `Component::new`
checks each configured one, keeps it beside the others, and starts it
alone when it comes with access. Nothing of the forges already kept
changes. temper's forges become records made at runtime (the draft,
`temper/docs/design/draft/playground-reset.md`, section 3), so the component
can no longer hold only the forges it was built with.

Acceptance:

- A component built with no forge, or with some, adds a Forgejo and a
  GitHub forge; each added forge's calls, connect, start and
  `Component::start(forge)` behave as a configured forge's do.
- An add is refused, with the `Unusable` `new` gives the same settings,
  for: a number already kept, one forge past `limits.forges`, and each
  per-forge check `limits::check` makes (transport, address, access,
  log origins, the dialect's derived bounds when the dialect is new).
  A refused add changes nothing.
- An added GitHub forge has its reserved own connection, and the
  dialect's repository and user tables have their room when it is the
  first of its dialect.
- An add after `Close` or `Abort` is refused (`Added::Closed`, the
  second option's default).
- The crate's tests and the skein worlds that build the component pass
  unchanged.

## Inputs

- `temper/docs/plan/playground-reset-r4/notes/common.md` and
  `temper/docs/plan/playground-reset-r4/notes/contracts.md` (section 1, D1:
  why forges are added at runtime).
- `skein/docs/design/forge-connection.md`, sections 2.1, 3 (3.1 to 3.5),
  7.2 and 7.4.
- `temper/docs/plan/playground-reset-r4/notes/surveys/s6-skein-forges.md`,
  section 1 whole (settings, positions, limits, "What runtime
  addition/removal needs").
- skein's `programming-model.md`, section 10 (step code's rules), and
  `testing-strategy.md`, section 1.

## Owns and may change

Owned: `skein/crates/skein-forge-connection/src/component.rs` (the
constructor's capacities, the new `add`), `src/limits.rs` (a per-forge
check `add` shares with `check`), `src/tests.rs` (one `mod` line) and
the new `src/tests/runtime_add.rs`.

Neighbours: `src/lib.rs` (a re-export, if `add`'s refusal is a new
type); `src/component/*.rs` only where a helper must take a position
it now computes the same way (no behaviour change).

Pre-approved options:

- If sharing the per-forge part of `limits::check` would mean
  reshaping it, write `limits::check_added(l, s, settings, forge) ->
  Result<(), Unusable>` beside it, calling the same per-forge helpers,
  and leave `check` as it is.
- If the add's refusal for a closed component cannot be an existing
  `Unusable` variant, return `Result<(), Added>` with `pub enum Added {
  Unusable(Unusable), Closed }`; default: that enum.
- Adding no `Request` variant is binding: temper's protocol layer
  matches `Request`, and this branch stays in skein.

Not here: removal (`sk-conn-remove`), temper's callers, any design text.

## Steps

1. **The method** (`component.rs`, cited to forge-connection.md, 3, and
   the draft, 3):

   ```rust
   /// Adds a forge at runtime, checked as `new` checks a configured
   /// one, and starts it when it comes with access (3.3, 3.5).
   pub fn add(
       &mut self,
       env: &Env<Limits>,
       forge: settings::Forge,
       up: &mut Queue<Event>,
       io: &mut Queue<skein_io::Request>,
   ) -> Result<(), Added>;
   ```

   It refuses at a closing or closed lifecycle; checks the forge
   (number unique among `settings.forges`, count below
   `env.limits.forges`, the per-forge checks of `limits::check`, and the
   dialect's derived checks when no kept forge has its dialect);
   appends the forge and its four parallel entries together (`starts`,
   `github`, `own_tokens`, `own_plans`) exactly as `new` makes them;
   makes the dialect's tables with `new`'s capacities when they were
   built with none; and, with access, sets the forge's start to its
   `initial` state and drives (`self.drive(env, up, io)`), starting no
   other forge again. `begin`'s listener is not begun again.

2. **Construction's room** (`new`): `settings.forges` is held in a list
   of capacity `limits.forges` (re-made from the owner's list when its
   capacity is smaller; temper's is the length of its configured list);
   the connections table and slab reserve one own connection per forge
   the component may keep that is GitHub, in place of per configured
   GitHub forge: `limits.connections + limits.forges`, unless the
   reserved-slot accounting in `component/calls.rs` (`assign`, its
   reserved-slot check) counts the configured GitHub forges, in which
   case keep the capacity at the larger bound and leave that count
   live (computed from `settings.forges` each time). Say which in the
   report.

3. **Tests**, `src/tests/runtime_add.rs`, with `tests/fixtures.rs`'
   `limits()`, `shared()`, `settings()` and its driver (`new`, `with`,
   `call`, `send`, `run`, `answered`, `start`, `connected`):
   - `an_added_forgejo_forge_answers_calls_as_a_configured_one`: built
     with no forge, add a Forgejo forge with access, run its start to
     ready, a call on it is answered as on the fixture's configured
     forge.
   - `a_forge_added_without_access_answers_not_connected_until_its_connect`.
   - `an_added_github_forge_takes_its_reserved_connection` (after
     `github_start.rs`' reserved-connection test).
   - `adding_leaves_the_kept_forges_starts_alone`: two configured
     forges ready, add a third: their starts and connections unchanged,
     no request on their connections.
   - `an_add_is_refused_as_new_refuses`: duplicate number, one past
     `limits.forges`, an in-process address, a plaintext name refused,
     a GitHub forge whose derived bounds fail; each `Err` equal to
     `Component::new`'s for the same settings, and the component's
     forges unchanged (`start(number)` is `None`).
   - `an_add_after_close_is_refused`.
   Narrow: `heavy cargo nextest run -p skein-forge-connection --lib -E
   'test(runtime_add::)'`, then the crate's lib tests.

4. `cargo fmt`; `heavy cargo clippy -p skein-forge-connection
   --all-targets -- -D warnings`; `heavy cargo nextest run -p
   skein-forge-connection`; the worlds that build the component:
   `heavy cargo nextest run -p skein-forge-machine-world --test suite`,
   `-p skein-forge-protocol-world --test suite`; `heavy cargo check
   --tests` for `temper-service`, `temper-engine-protocol-forge-world`
   and `forgejo-conformance` (package names as in their `Cargo.toml`).
   Commit and push.

## Report

fab's sections, and `## Interfaces`: `Component::add`'s signature, the
refusal type, any `limits` function added, and the construction's new
capacities (step 2's choice).
