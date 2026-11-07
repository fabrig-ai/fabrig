+++
id = "reference-room"
part = "room"
kind = "docs"
needs = []
files = [
  "temper/docs/examples/reference/room.json",
  "temper/docs/examples/reference/README.md",
  "temper/crates/temper-service/src/tests/root_checks.rs",
  "temper/crates/temper/tests/support/owned.rs",
  "temper/docs/design/protocol/configuration.status.md",
]
effort = "medium"
minutes = 35
+++

# The reference's room carries the journal's run bound

## Goal

A box set up from the reference refuses a charter of 31 KB, since the box's `root.journal.run_bytes` is 20,480 (`temper/crates/temper-service/src/limits.rs`) and that bound holds a run's stored policy, a charter's whole policy among it. `temper/crates/temper-service/src/tests/room.rs` already starts with 65,536. Give the reference's `room.json` an `application` member carrying `root.journal.run_bytes` 65,536, merged over the document's `application` as `limits.json` is (`temper/docs/design/protocol/configuration.md`, 13.9, `room.json`), have its README's commands and the tests that merge the room apply it, and say in "A call's room" what it bounds and what else a charter must fit.

This task departs from the implementer's text in one way: one status entry is yours to remove (acceptance 5), since the design already says what this task builds.

Acceptance:

1. `room.json` is

   ```json
   {
     "agent": { ... as now ... },
     "worker": { ... as now ... },
     "application": {
       "limits": {
         "overrides": {
           "root.journal.run_bytes": 65536
         }
       }
     }
   }
   ```

2. `README.md`, "A call's room": the `temper.json` command also merges the room's `application` member over the document's, keeping the document's other members and overrides (default: `jq --slurpfile room room.json '(.workers[] |= (. * $room[0].worker)) | .application = (.application * $room[0].application)' temper.json > temper.next.json`). Two sentences beside it: `root.journal.run_bytes` bounds a run's stored policy, a charter's whole policy among it, and the box's 20,480 refuses a charter of 31 KB, so the room sets 65,536; and a charter's `turns`, its `time` and the run's spend must fit the bounds a worker advertises, or no host takes its tasks, citing `jig/crates/jig-domain-fleet/src/needs.rs` by a path relative to the README, as the README cites the design. Lose no fact of the section, and keep every phrase the tests below assert.
3. `temper/crates/temper-service/src/tests/root_checks.rs`: `reference_document` merges the room's `application` member over the document as it merges `worker` over each worker; `the_reference_room_starts_and_holds_its_relations` gains the row `("root.journal.run_bytes", 65_536)` in its table of the application's overrides, and still starts.
4. `temper/crates/temper/tests/support/owned.rs`, `bounded_template_profile`: with a room, merges its `application` member over the document's, as `merge_profile` merges a worker; `startup::the_reference_profile_passes_startup_admission` still passes.
5. The entry *13.9: `room.json` has no `application` member...* is gone from `temper/docs/design/protocol/configuration.status.md`.
6. Focused tests pass: `temper-service` (`--lib`), `temper-docs`, `temper-engine-smith` (`--lib`), `temper-engine-domain-world` (`host_calls::`), and `temper --test suite -E 'test(/^startup::/)'`; `python3 -I -B tools/citations/citations.py --base origin/main` lists nothing.

## Inputs

- `temper/docs/plan/replay-repairs/notes/common.md`.
- `temper/docs/design/protocol/configuration.md`, 13.9 (the `room.json` item, the paragraph on what `limits.json` and `room.json` hold, and *Checked as it is read*) and 13.8 (a call's room).
- `jig/crates/jig-domain-fleet/src/needs.rs`: `charter_shortfall` and `shortfall` compare a charter's `turns`, `time` and the run's `spend` with a worker's `Bounds`.

## Owns and may change

**Owned:** the `files` above.

**Readers of the README** (`.config/gate/read-by-tests.txt` lists it), each kept passing, none edited: `temper/crates/temper-docs/src/tests/rules.rs` (its phrases), `temper/crates/temper-service/src/tests/stock.rs` and `temper/tests/engine/domain/tests/suite/host_calls.rs` (the `reference-authority.json` recipe, which this task leaves as it is). **Readers of `room.json`:** `temper/crates/temper-engine-smith/src/tests.rs` (reads `agent` alone; unchanged).

**Options, each with its default:**

- The README's `temper.json` command: the default above, or any `jq` that keeps the document's other application members and overrides; the test's merge is what binds, the command says it.
- The two sentences' words are yours, within the facts above.
- If the start refuses 65,536, naming another limit that must move with it: stop with `Question:`, the figure and the relation; moving a figure beyond the request is the user's.
- If `limits.json` already carries an override the room's merge would replace: none does today; if one appears on the trunk, stop and ask.

## Steps

1. `room.json` (acceptance 1) and the two merges (acceptances 3 and 4); run `heavy cargo nextest run -p temper-service --lib -E 'test(=tests::root_checks::the_reference_room_starts_and_holds_its_relations)'`, then break the room's figure once (20480), see the new row fail, restore.
2. The README (acceptance 2) and the status entry (acceptance 5).
3. `cargo fmt`, `python3 -I -B temper/scripts/old_forms.py`, `heavy cargo clippy -p temper-service -p temper --all-targets -- -D warnings`, acceptance 6, commit and push.

## Report

As the implementer's text says; also the README's command as written.
