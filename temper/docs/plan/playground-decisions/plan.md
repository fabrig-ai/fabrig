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
2. **kept names** (jig and temper, `temper/docs/design/domain/forge.md, 4.1`): decision 2. Its
   designer is writing the design and specs to C1: jig asks every connector `Unname { task }` at
   a task's erasure; temper's forge keeps a released task's `Names` row until then, so a removed
   forge's repository rows stay while any kept task names them. Expected tasks:
   `unname-at-erasure` (jig-domain-core) and `kept-names` (temper-engine-domain-forge and the
   root's mapping, after `playground-restart-resets`).
3. **deletion grace** (jig and temper, `jig/docs/design/domain/people.md, 5.2`): decision 3.
   Its designer is writing the design and specs to C2: `RemoveProject` carries a grace within the
   tasks child's bound, the deletion keeps it and cancels each live conversation as a close does;
   clients fill it from `me`; temper's root fills the reset's from its chat template. Expected:
   an interface task (wire, contract, people, store row, in place), jig-domain-core's admission
   and cancel, temper's root.
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

As it stands: `playground-restart-resets` (~60 minutes) → `kept-names` (designing) → the grace
family's temper task. The grace family's interface task (wire and store) is likely the longest.

## Overlaps

- `box-words` and `reference-m8/panel-steps`: the same `temper-client-view/src/words.rs` and
  `temper-client-domain/src/forge/words.rs`; this branch keeps the phrase counts so it stays off
  the `PHRASES` line that branch changes, and edits `set_up_commands`, far from its hunks. Both
  edit `client/tools.status.md` and `client/forge.status.md`, in different entries.
- `playground-restart-resets` and the `reference-m8` branches: `domain.rs`'s `read_afresh`
  gains a line there; this task changes `playground.rs` alone.
- `setup.status.md`: `v0.1.0-ladder/finding-5` adds a line after its Departs; this plan edits the
  Departs line (`playground-restart-resets`) and the Not yet line (the grace family).

## Calls made

- **Decision 2's and 3's mechanisms are the lead's contracts** (`notes/contracts.md`), since the
  design states the rule and not how the connector learns of an erasure or how the grace reaches
  the deletion. C2 carries the grace in the request because the engine reads no template (jig's
  `domain/engine.md`, 11.1); the schemas change in place before the baseline (jig's
  `protocol/README.md`, 7).
