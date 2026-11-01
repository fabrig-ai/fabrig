+++
id = "sk-setup-forges"
part = "skein-forges"
kind = "feature"
needs = []
files = [
  "skein/crates/skein-forge-setup/src/host.rs",
  "skein/crates/skein-forge-setup/src/host/limits.rs",
  "skein/crates/skein-forge-setup/src/host/states.rs",
  "skein/crates/skein-forge-setup/src/tests/host_forges.rs",
]
effort = "high"
minutes = 45
+++

## Goal

skein's setup host (`skein-forge-setup`'s `host::Host`) adds and removes
a forge at runtime: `Host::add` checks one forge as `Host::new` checks
each, and keeps it; `Host::remove` ends the forge's runs unheard,
aborts their connections and clears its callback states. A run that
names a forge the host does not keep is stopped, never a panic. temper's
forges are records made and removed at runtime
(`temper/docs/design/draft/playground-reset.md`, sections 3 and 4: "the
setup host sets it up no more").

Acceptance:

- A host built with no forge adds a Forgejo and a GitHub forge, then
  runs each one's plans as for a configured forge.
- An add is refused with the `Unusable` that `Host::new` gives the same
  settings: a duplicate number, one past `limits.forges`, and each
  per-forge check (address, plaintext name, the dialect's `Given`
  checks); a refused add changes nothing.
- After `remove(forge)`: its live runs are gone with no event (as
  `Close` ends runs unheard), their connections aborted and settled as
  `Close` settles them; its issued states are cleared (a callback with
  one is refused as an unknown state); an `Answer` for one of its runs
  is ignored; a later `Run` naming it is answered `Event::Stop` with a
  `Failure` whose `why` is `Why::Unknown`; the other forges' runs go on.
- `Request::Run` for a number never kept is the same `Stop`, not the
  panic it is today (`host.rs`, `configured`).

## Inputs

- `temper/docs/plan/playground-reset-r4/notes/common.md` and
  `temper/docs/plan/playground-reset-r4/notes/contracts.md` (D3).
- `skein/docs/design/forge-setup.md`, sections 6.3 and 6.5 (the host;
  6.5 says its settings are fixed for its life, which this task
  departs from: say so in the commit, citing the draft, 3 and 4).
- `temper/docs/plan/playground-reset-r4/notes/surveys/s6-skein-forges.md`,
  section 2 whole.

## Owns and may change

Owned: `skein/crates/skein-forge-setup/src/host.rs` (`add`, `remove`,
the unknown-forge `Stop`), `src/host/limits.rs` (the per-forge check
shared with `check`), `src/host/states.rs` (clearing one forge's
states), the new `src/tests/host_forges.rs` and its `mod` line beside
`tests/host.rs`'s declaration.

Neighbours: `src/host/settings.rs`, `src/host/run.rs`,
`src/host/connection.rs`: a helper only, if the removal must find a
forge's runs' connections there.

Pre-approved options:

- If the per-forge part of `limits::check` cannot be shared without
  reshaping it, add `check_added(l, settings, forge) -> Result<(),
  Unusable>` calling the same helpers.
- `Host::new`'s `settings.forges` list is re-made with capacity
  `limits.forges` when the owner's is smaller (temper sizes it to its
  configured count); default.
- The unknown-forge `Failure`: `forge` the number, `step`
  `Stage::App` for a GitHub-only plan and `Stage::Admin` otherwise (by
  the plan's dialect), `subject` `b"forge"`, `why` `Why::Unknown`.
- No `Request`, `Event` or `Why` variant is added: temper matches them.

## Steps

1. **The methods** (`host.rs`, cited to forge-setup.md, 6.3, and the
   draft, 3 and 4):

   ```rust
   /// Adds a forge at runtime, checked as `new` checks a configured one.
   pub fn add(&mut self, limits: &Limits, forge: Forge) -> Result<(), Unusable>;
   /// Removes a forge at once: its runs end unheard, their connections
   /// abort, its states clear.
   pub fn remove(&mut self, forge: u16);
   ```

   On a closing host `add` keeps nothing and returns `Ok(())` (a
   closing host serves nothing), and `remove` does nothing. `remove` takes every run whose `forge` is the
   number out of `runs` and cuts it as `Abort` cuts a halted run (`cut`,
   its connections aborted and kept until settled), without pushing its
   event; clears `states` of the forge (`States::clear_forge(forge)`,
   new); removes the forge from `settings.forges` (rebuild the list
   without it: `skein_lib::List` has no `remove`).

2. **Unknown forges.** `configured` returns `Option<&Forge>`; `down`'s
   `Run` answers `Event::Stop` (the option's `Failure`) when it is
   none; the send path that looks the forge up again (`host.rs`, its
   second `configured` call) treats none as the run's end, unheard.

3. **Tests**, `src/tests/host_forges.rs`, with `tests/fixtures.rs` and
   `tests/github_fixtures.rs`' builders and `tests/host.rs`' driving
   helpers:
   - `an_added_forgejo_forge_runs_its_reconcile` (a host with no forge,
     add, `Run` a `Reconcile`, drive it to `Done` against the fixture's
     answers).
   - `an_added_github_forge_issues_and_takes_its_state`.
   - `an_add_is_refused_as_new_refuses` (duplicate, past `forges`, a
     bad address, a bad `Given`), the host unchanged after each.
   - `removing_a_forge_ends_its_runs_unheard_and_keeps_the_others`: two
     forges, a run on each awaiting an answer; remove the first: no
     event for its run, its connection aborted, an `Answer` for it
     ignored; the second's run answers `Done`.
   - `a_removed_forges_state_is_unknown`: two GitHub forges; the first
     issues a manifest state; remove it; a `Convert` with that state on
     the second is refused `Why::Unknown`.
   - `a_run_for_an_unknown_forge_stops_unknown`.
   Narrow: `heavy cargo nextest run -p skein-forge-setup --lib -E
   'test(host_forges::)'`.

4. `cargo fmt`; `heavy cargo clippy -p skein-forge-setup --all-targets
   -- -D warnings`; `heavy cargo nextest run -p skein-forge-setup`;
   `heavy cargo nextest run -p skein-forge-setup-world --test suite`;
   `heavy cargo check --tests -p temper-service -p temper-forge-setup
   -p temper-forge-setup-shell`. Commit and push.

## Report

fab's sections, and `## Interfaces`: `Host::add`, `Host::remove`,
`States::clear_forge`, and the unknown-forge `Failure` as built.
