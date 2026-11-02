# playground-decisions: the plan

From `inputs/request.md`: the user decided, on 2026-11-02, the five questions the playground
reset's close-out left. The design already states four of them; this plan brings the code to the
design and the status files to the decisions. One part per decision that needs code; they share
no crate in the first wave. The shapes that cross tasks are `notes/contracts.md` (C1, C2); every
task names `notes/common.md`.

## Parts and tasks

1. **playground-restart** (temper, `temper/docs/design/protocol/setup.md, 8.4`), slice 2.
   - `tasks/playground-restart-resets.md`, high, ~60 minutes, needs nothing: `restart` resets a
     running row as it resets a starting one, and asks the box for no group; the forge unit test
     that pinned the departure is inverted, the engine world gains the story, and a service
     reader test is adjusted if it breaks. Closes `setup.status.md`'s Departs "8.4, 16" and
     `STATUS.md`'s clause. The design: setup.md 16's open question removed (this push).
2. **kept-names** (jig and temper, `temper/docs/design/domain/forge.md, 4.1`), slice 4:
   decision 2, to C1. Its designer wrote the design first: forge.md 4.1 (a task's names kept
   until it is erased, their bound, a project's release), 5.1, 16, 18, and 20's open question
   removed; temper's store.md 2 (the `Names` row, live or kept); jig's connectors.md 2 and 3.4
   (`Unname` asked at a task's erasure). Kept rows hold the repositories alone, at most
   `forge.repositories` of them beside the live ones, the oldest a later task covers going first.
   - `tasks/kept-names.md`, high, ~150 minutes, needs `playground-restart-resets` (the same
     forge crate): the release keeps the row, `Unnamed` is the erasure, a project's release
     clears its repositories from kept rows, a start splits live and kept rows, temper's root
     maps `Ask::Unname`; the engine world's joined story. temper-engine-domain's first writer.
   - `tasks/unname-at-erasure.md`, medium, ~45 minutes, needs `kept-names` (temper's root holds
     `Ask::Unname` in an `unreachable!` arm until then) and `deletion-grace` (jig-domain-core's
     first writer): the erase walk's record decision and a forget ask every connector `Unname`;
     jig's test domain routes it to nothing; the readers' worlds, temper's among them, run.
     The lead added to jig's people.md 5.2 and 6 that an erasure tells every connector.
3. **deletion-grace** (jig and temper, `jig/docs/design/domain/people.md, 5.2`), slice 3:
   decision 3, to C2. Its designer wrote the design first: people.md 1, 2, 5.1 to 5.3, 7 and 11
   (the request's grace, refused past the tasks child's bound, kept with the deletion, each
   conversation closed as the deployment's `Close` closes one), tasks.md 7, client.md 19
   (`shapes::remove_project`), contract.md 3.2, jig's store.md 4.4 and temper's setup.md 8.4
   (the reset's grace, template 1's, which temper's root reads); the lead placed temper's
   client/README.md 3.5 and jig's configuration.md 3.1 to match.
   - `tasks/remove-project-grace.md`, interface, high, ~120 minutes, needs nothing: the field
     carried everywhere with no change of meaning: client-v19 and people-v14 in place, goldens
     regenerated, the people child keeping it, the core's party route matching the project
     alone, temper's two literals `grace: None`.
   - `tasks/remove-project-shape.md`, medium, ~25 minutes, needs `remove-project-grace`:
     jig-client's `shapes::remove_project`.
   - `tasks/deletion-grace.md`, high, ~120 minutes, needs `remove-project-grace`:
     jig-domain-core checks the grace at admission and closes each conversation with it; the
     core world's story. jig-domain-core's first writer in this plan.
   - `reset-grace` (temper's root fills the reset's grace from template 1), high, ~45 minutes,
     written and held until `kept-names` is in the plan: it needs `remove-project-grace`,
     `kept-names` (temper-engine-domain's first writer) and `playground-restart-resets`.
4. **Decision 4 needs no change:** `setup.md` 8.4 already runs the start's acts under the
   sign-in of the person who sent the start, and no status file leaves it open (checked
   `rg` over every `*.status.md` and `STATUS.md` of temper and jig on 2026-11-02).
5. **box-words** (temper, `temper/docs/design/client/forge.md, 5.4`), slice 1.
   - `tasks/box-words.md`, medium, ~60 minutes, needs nothing: the Details, the Forges row and
     ctl's refusal say to start temper with `temper box`; `set_up_commands` goes; the view keeps
     its three words' numbers and counts; `ctl-forge.md` is regenerated from the table. Closes
     the 5.5 and 5.4 Departs of `client/tools.status.md` and `client/forge.status.md`. The
     mockup's `pages.js` is brought to the words in this push.

## Critical path

`box-words` and `playground-restart-resets` landed. Now: `remove-project-grace` (~120) →
`deletion-grace` (~120) → `unname-at-erasure`; beside it `kept-names` (~150) → `reset-grace`
(~45) and `unname-at-erasure`. About five hours of implementer time on the longest chain (~285
minutes), up to ten with the buffer.

## Overlaps

- `box-words` and `reference-m8/panel-steps`: the same `temper-client-view/src/words.rs` and
  `temper-client-domain/src/forge/words.rs`; this branch keeps the phrase counts so it stays off
  the `PHRASES` line that branch changes, and edits `set_up_commands`, far from its hunks. Both
  edit `client/tools.status.md` and `client/forge.status.md`, in different entries.
- `playground-restart-resets` and the `reference-m8` branches: `domain.rs`'s `read_afresh`
  gains a line there; this task changes `playground.rs` alone.
- `setup.status.md`: `v0.1.0-ladder/finding-5` adds a line after its Departs; this plan edits the
  Departs line (`playground-restart-resets`) and the Not yet line (`reset-grace`, after it).
- `remove-project-grace` and `reference-m1/goal-jig-store`: both change `people-v14.schema` in
  place; the generated `people_v14.rs`, `layouts.rs` and the people goldens conflict wherever the
  two meet, mended by regenerating, never by hand. `client-v19.schema` likewise with the
  `reference-m1` settings branches.
- jig's `tasks.md` 7 and `tasks.status.md`: edited away from the `reference-m1` hunks
  (854-927, 1306-1346, the status's first lines).
- `kept-names` and the `reference-m8` branches: both change `temper-engine-domain-forge`'s
  `domain.rs` in `unnamed` and `names` (their tier watches) and near `retire_ended`; unavoidable,
  since the release keeps its row there.

## Known readers outside a spec

- `temper/tests/engine/domain/tests/suite/removal.rs:277` builds `people::Ask::RemoveProject {
  project: 1 }`, which `remove-project-grace`'s spec does not list. It needs `grace: None`, a
  compiler-forced field the implementer's text allows in its tree: `kept-names` owns the file and
  `reset-grace` builds that world, so whichever temper task meets it adds it; else the trunk's
  end check.
- After `unname-at-erasure`, every erased task reaches the forge connector as `Unnamed`, most
  with no row: `unnamed` then sends the client its unchanged watch set, which changes nothing.

## Calls made

- **jig-domain-core's order changed in flight:** `deletion-grace` first, `unname-at-erasure`
  after `kept-names`, since jig may send `Unname` only once temper's root maps it.
- **Decision 2's and 3's mechanisms are the lead's contracts** (`notes/contracts.md`), since the
  design states the rule and not how the connector learns of an erasure or how the grace reaches
  the deletion. C2 carries the grace in the request because the engine reads no template (jig's
  `domain/engine.md`, 11.1); the schemas change in place before the baseline (jig's
  `protocol/README.md`, 7).
