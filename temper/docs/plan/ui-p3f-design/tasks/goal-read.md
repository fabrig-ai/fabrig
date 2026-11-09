+++
id = "goal-read"
part = "path"
kind = "feature"
needs = ["goal-path", "goal-value"]
files = [
  "temper/crates/temper-engine-domain-ways/src/ways.rs",
  "temper/crates/temper-engine-domain-ways/src/tests.rs",
  "temper/crates/temper-engine-domain/src/route/ways.rs",
  "temper/crates/temper-engine-domain/src/route/ways/tests.rs",
  "temper/docs/design/domain/forge.status.md",
]
effort = "high"
minutes = 60
+++

# The root reads the goal template's work for the path

## Goal

The root gives the derivation what the goal's template says: when a project's
work starts as a goal, `Start::Goal`'s `design` and `close_out` are that
template's kind 11 as the forge holds it, with a decision's items laid over it,
so that the project's own path, each stock way's path over it and a proposal's
"after" draw "design" and "close-out" from the settings. With none held, the
coordinator plans and makes no close-out.

Acceptance, by `temper/docs/design/domain/forge.md`, 8.13 (the inputs: *that
template's work*; *The ways' paths and a proposal's "after"*) and
`temper/docs/design/protocol/configuration.md`, 13.2 (*A goal template's work,
kind 11*, *with none*):

1. **`laid_goal`,** exactly `temper/docs/plan/ui-p3f-design/notes/contracts.md`,
   section 6, in `temper/crates/temper-engine-domain-ways/src/ways.rs` beside
   `laid_ways`, exported as it is: an item of kind 11, scope 0, the template's
   number sets the value; its removal leaves neither; items of any other kind,
   scope or number change nothing; with no item, the held value, else neither.
2. **The root's `start`** (`temper/crates/temper-engine-domain/src/route/ways.rs`)
   gives the chosen template's `laid_goal(domain.forge.template_goal(number),
   number, items)` to `Start::Goal`, in place of `goal-path`'s two falses. Only
   the template the start chose: a value of another template changes nothing.
3. **Rows,** no new test function:
   - the ways crate's `laid_ways_sets_removes_and_ignores_other_settings`
     (`src/tests.rs`), or a table of `laid_goal` beside it in the same test
     where the shapes differ: an item setting template 4's work, its removal, an
     item of template 2's or of kind 7 ignored, none held and none laid;
   - the root's `inputs_gather_the_repository_start_and_readers_roles`
     (`route/ways/tests.rs`): its expected starts carry `design` and
     `close_out`; rows "goal designs" (template 4's work set to both by an item
     laid), "goal held" (both held by the forge, no item: build the forge's
     state as the forge's own tests do, a kept setting, and name how in the
     report), "goal plans" (none held and none laid: both false) and "another
     template's" (template 2's work held, the project starting by template 4:
     both false). Its derived path asserts, as it asserts the catch-up: "design"
     drawn iff the expected start designs, "close-out" iff it closes out.
4. **Bite:** read template 1's work in place of the chosen template's, see the
   "another template's" row fail; restore.
5. **Status:** delete the bullet naming `goal-read` in
   `temper/docs/design/domain/forge.status.md` (8.13) (`notes/common.md`,
   *Status files*).

## Inputs

- `temper/docs/design/domain/forge.md`, 8.13: its first two bullets, what the
  path reads and how items are laid.
- `temper/docs/design/protocol/configuration.md`, 13.2, *A goal template's
  work, kind 11*.
- `temper/docs/plan/ui-p3f-design/notes/contracts.md`, sections 3, 5 and 6.
- `temper/docs/plan/ui-p3f-design/notes/common.md`.
- The code: `route/ways.rs`' `start`, `project_ways` and `inputs`; the ways
  crate's `laid_ways` and its test.

## Owns and may change

- Owns: `laid_goal` and its export, the root's `start`, the rows and the status
  bullet.
- Nothing else of the derivation (`goal-path`'s) or of the forge's value
  (`goal-value`'s). If `Domain::template_goal` or `Start::Goal` isn't as the
  contracts note has it on the trunk, stop and ask.
- Where a stock way's items or a proposal's carry a kind 11 item, it reaches
  `start` through `items` already: no other caller changes.

## Steps

1. Read the inputs.
2. `laid_goal` and its rows: `heavy cargo nextest run -p temper-engine-domain-ways`.
3. The root's `start` and its rows; the bite: `heavy cargo nextest run -p
   temper-engine-domain --lib -E 'test(/inputs_gather/)'`.
4. Runs, counts in the report: `heavy cargo nextest run -p
   temper-engine-domain-ways -p temper-engine-domain -p temper-service`; `heavy
   cargo check --tests -p temper-engine-domain-world`.
5. `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `heavy cargo clippy
   -p temper-engine-domain-ways -p temper-engine-domain --all-targets -- -D
   warnings`.
6. Commit (why, in the message); `python3 -I -B tools/citations/citations.py --base
   origin/main`; push.

## Report

As the implementer's text says, and: `laid_goal` as written, how the "goal held"
row holds its setting, the rows and the bite, the runs' counts.
