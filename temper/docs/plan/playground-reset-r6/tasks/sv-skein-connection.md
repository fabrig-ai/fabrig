+++
id = "sv-skein-connection"
part = "fs"
kind = "interface"
needs = []
files = ["skein/crates/skein-forge-connection/**", "skein/tests/forge/machine/**"]
effort = "high"
minutes = 100
+++

## Goal

skein's forge connection component takes forges added and removed after its start
(`notes/contracts.md`, T7): `Request::Add { forge, settings }` and `Request::Remove { forge }`,
each answered by the component's existing events. Its start-time list stays the initial set.
skein only: nothing here names temper.

Acceptance:
- `skein_forge_connection::Request` gains `Add { forge: u16, settings: settings::Forge }` and
  `Remove { forge: u16 }`; their doc comments say what this spec says.
- An admitted `Add` holds the forge as a start-time forge is held: with an access its start
  begins at once and says `Connected` or `Stopped` once, as a `Connect` does (one access
  generation); with none it runs no start and its calls answer `NotConnected` at cost 0.
  An `Add` before `begin` is one generation too, not two (`begin` starts it as it starts a
  forge handed at start, and says `Connected` once).
- An `Add` whose access 3.5 refuses is admitted with no access and says
  `Stopped { forge, why: Stopped::Invalid { value } }`, as a refused `Connect` does.
- An `Add` the component cannot hold changes nothing and says nothing, as a `Connect` for a
  forge not configured: after `Close`/`Abort`; a number already held; `forge` differing from
  `settings.number`; `limits.forges` forges already held; or settings that start's per-forge
  checks (forge-connection.md, 3.2 and 7.4) refuse. `Component::admits` says why, by the same
  check, so the owner can check first.
- `Remove` of a held forge: every call open on it ends at once with its one
  `Event::Answered` (as at its deadline: `Unavailable` at cost 0, `Timeout` once it cost);
  its own connection and its calls' connections close; its start, installations, tokens,
  puller and job-log origins are dropped; afterwards no `Connected`, `Stopped`,
  `Installation`, `GitToken` or `Fact` names it, a call naming it answers `Unavailable` as a
  call for a forge never held does, a `Connect` or `Installations` naming it is ignored, and
  a webhook delivery to its path is answered `404`. A hint already verified before the
  removal may still go up (the owner drops it). `Remove` of a number not held is ignored.
  The number may be added again afterwards, as a new forge.
- Start-time behaviour is unchanged for every existing test, except the capacities below.

## Inputs

- `skein/docs/design/forge-connection.md`, sections 2.1, 3.1, 3.2, 3.5, 7.2, 7.4, 8.3, 8.5.
- `skein/docs/foundation/programming-model.md`, section 1 and 6.1 (containers grow as they
  fill: a `Map`/`List`/`Slab` of a capacity allocates nothing until used).
- `temper/docs/plan/playground-reset-r6/notes/common.md`.
- `temper/docs/plan/playground-reset-r6/notes/contracts.md`, T7.
- `skein/AGENTS.md`.

## Owns and may change

- Owns: `skein/crates/skein-forge-connection/` (all of it) and the machine world
  `skein/tests/forge/machine/` (package `skein-forge-machine-world`).
- May change, call sites only: any skein or temper file the compiler names because of a
  type this task changes (none is expected: no reader matches `Request` exhaustively; temper
  uses `if let` on it in `temper/crates/temper-service/src/forge.rs`).
- Not `skein/docs/design/`: the report proposes forge-connection.md's new text (3.1's "for
  each forge" gains "or added later"; 3.5 gains Add and Remove; 7.4 names the per-forge
  check), and the lead writes it.
- Pre-approved options:
  - Per-forge state lives today in parallel `List`s indexed by position (`settings.forges`,
    `starts`, `github`, `own_tokens`, `own_plans`). Default: keep them parallel and, on
    `Remove`, rebuild each list without the removed position (a fresh
    `List::with_capacity(limits.forges)` refilled in order), so that positions stay dense;
    every place that keeps a position across events must then find it again by number (rg
    `at: u32` and `Some(at)` in `src/component*`; today positions are found by number on
    each pass). If a slot type (`List<Option<...>>`, a `Map<u16, ...>`) is simpler for you,
    take it and say so in the report. Do not add methods to `skein-lib`.
  - Tables: today `forgejo_tables` and `github_tables` are sized 0 unless a forge of that
    kind is configured at start. Default: size both by `limits.repositories` and
    `limits.users` always (an unused `Map` allocates nothing). The connections' `Map` and
    `Slab` capacity becomes `limits.connections + limits.forges` (every forge may be a GitHub
    forge with its reserved own connection); `limits::check` checks that sum is
    representable instead of `connections + github`.
  - GitHub's limit checks (`check_github`: documents, installation bounds, `deadline` at
    most 270 s, `renew_before`, `key_bytes`) stay conditional at start on a GitHub forge
    being configured, and run in the per-forge check of a GitHub forge's `Add`. Default as
    said; do not make them unconditional at start (a Forgejo-only owner's limits would be
    refused).
  - The new per-forge check: factor the per-forge part of `limits::check_settings` (number
    unique, plaintext name, address, log origins, request head) and of `check_github` into
    `pub(crate) fn forge(l: &Limits, s: &Shared, settings: &Settings, forge: &Forge) ->
    Result<(), Unusable>` used by both start and `Add`, so they cannot drift. `Unusable`
    keeps its variants; reuse `Forges { forges }` for a full set and `Duplicate { forge }`
    for a held number; for `forge` differing from `settings.number`, `Duplicate { forge }`
    is wrong: add `Unusable::Number { forge: u16 }` and route it in temper's
    `temper/crates/temper-service/src/forge.rs::problems` and `::indexed` to the same names
    as `Duplicate` (compiler-forced).

## Steps

1. `src/boundary.rs`: add the two variants to `Request`:

   ```rust
   /// A forge held from now on, as one handed at start is (3.1); its access, when it has
   /// one, starts it as a `Connect` does (3.5). Ignored, with nothing said, when the
   /// component cannot hold it: see `Component::admits`.
   Add { forge: u16, settings: crate::settings::Forge },
   /// A forge held no more: its open calls end as at their deadlines, its connections
   /// close, and nothing it held is kept. Ignored for a forge not held.
   Remove { forge: u16 },
   ```

2. `src/component.rs`:
   - `pub fn admits(&self, limits: &Limits, forge: &settings::Forge) -> Result<(), Unusable>`:
     `Forges` when `limits.forges` are held; `Duplicate` when the number is held; else the
     per-forge check of 3.2/7.4 against the held settings (`webhooks`, `user_agent`, `word`)
     and the `Shared` the component was built with (keep a copy: it already keeps
     `shared`). The access is not checked here (it is `Add`'s, as `Connect`'s).
   - `down`: `Request::Add { forge, settings }`: when `Lifecycle::Live`, `forge ==
     settings.number` and `admits` is `Ok`, push the forge and its parallel state
     (`Start::Unconnected`, its `GitHub` installations state for a GitHub forge as `new`
     builds it, `None` own token and plan); then, when `settings.access` is `Some`, hand
     it through the same path `connect` takes (so a refused access says `Stopped
     Invalid`); then `drive`. Otherwise nothing.
   - `Request::Remove { forge }`: when held: for each call whose `repository.forge` is the
     number, end it as `expire` does (closing its connection); close the forge's own
     connection (as `connect` closes a replaced one); drop its position from every per-forge
     list; `drive`. Calls held in `held` (refused/returned answers) are the owner's and stay.
   - In `Closing`/`Aborting`/`Closed`, `Add` and `Remove` are ignored like every request
     but `Abort`.
3. `src/limits.rs`: the factored per-forge check (options above); `capacity_bound` checks
   `connections + forges`.
4. Unit tests (`src/tests/`, a new module `src/tests/runtime.rs` registered in
   `src/tests.rs`), each with the component built by the existing fixtures:
   - `an_add_past_forges_or_of_a_held_number_is_refused_and_changes_nothing`
   - `admits_names_the_same_refusal_as_start_for_a_bad_address_and_a_bad_plaintext_name`
   - `an_add_of_a_github_forge_checks_the_github_limits`
   - `an_add_before_begin_starts_once_and_says_connected_once`
   - `a_remove_drops_every_per_forge_slot_and_the_others_keep_theirs` (two forges, remove the
     first, the second's start and calls unaffected; `Component::start(n)` is `None` for
     the removed one)
5. Machine world (`skein/tests/forge/machine`, `tests/suite/runtime.rs`, registered in
   `tests/suite/main.rs`). Teach `Machine::action` that an admitted `Add` with an access
   starts a new access generation for its number, as `Connect` does (its
   `access_generations` map gains the number), and that `Remove` ends it. Tests:
   - `a_forge_added_after_start_starts_says_connected_and_answers_its_calls` (start with no
     forges: `Settings { forges: List::with_capacity(..) empty, .. }`; `Add` the fixture
     forge with `access(TOKEN, WRITER, None)`; `locate_script()`; call answered, cost 1)
   - `a_forge_added_with_no_access_answers_not_connected_until_a_connect`
   - `an_add_with_a_refused_access_says_stopped_invalid`
   - `a_removal_ends_a_waiting_call_unavailable_and_a_written_one_as_its_deadline_would`
     (`pause_at`/`Pause::Reading` as `access.rs` does; the connection's later `Closed`
     changes nothing and nothing names the forge afterwards)
   - `a_call_and_a_connect_after_a_removal_are_answered_as_for_a_forge_never_added`
   - `a_removed_forge_added_again_starts_a_new_generation`
   - `a_removed_github_forge_drops_its_installations_tokens_and_puller` (on the GitHub
     fixtures `Machine::github`, after `github_start`'s ready state: no `GitToken`,
     `Installation` or `Fact` event after the removal, `run_for` a renewal's span)
   - `a_webhook_delivery_to_a_removed_forge_is_answered_not_found` (on `hook_settings()`, as
     `tests/suite/hooks.rs` drives a delivery)
   - `close_after_a_removal_says_closed_once`
6. Narrow runs, one at a time:
   - `heavy cargo nextest run -p skein-forge-connection --lib -E 'test(/tests::runtime::/)'`
   - `heavy cargo nextest run -p skein-forge-machine-world --test suite -E 'test(/^runtime::/)'`
   Check the counts (5 and 9 tests). Then the packages whose meaning a capacity change could
   reach, whole: `heavy cargo nextest run -p skein-forge-connection`,
   `heavy cargo nextest run -p skein-forge-machine-world`,
   `heavy cargo nextest run -p skein-forge-protocol-world`; and
   `heavy cargo check -p temper-service --tests`.
7. `cargo fmt`; `heavy cargo clippy -p skein-forge-connection -p skein-forge-machine-world
   --all-targets`; `python3 -I -B scripts/old_forms.py` from `skein/` (no count rises).
8. Commit (why: forges added and removed at runtime, for temper's forges in its store) and
   push your branch.

## Report

- The positions option taken, and every place that kept a position across events.
- The proposed text for forge-connection.md 3.1, 3.5 and 7.4 (a short paragraph each).
- The memory world's ceilings (`tests/suite/memory.rs`): unchanged, or moved and why.
