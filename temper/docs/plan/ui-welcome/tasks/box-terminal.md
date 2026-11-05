+++
id = "box-terminal"
part = "box-terminal"
kind = "feature"
needs = []
files = [
  "temper/crates/temper-box/src/**",
  "temper/crates/temper-forge-setup/src/step.rs",
  "temper/crates/temper/src/box_command.rs",
  "temper/docs/help/box.md",
  "temper/docs/development/qa.md",
]
effort = "high"
minutes = 80
+++

# The box's terminal

## Goal

The box's terminal says `Forge N (Forgejo): starting`, then `started`, as the mock's `boxPage()` prints; a
forge's hands go to its Forgejo step's log. Request, section 9.

Acceptance:

- Every `Group*` fact of an owned forge is labelled `Forge N (Forgejo)`, N the forge's number. The fake kind's
  stays `Playground`.
- A real run's terminal holds `HH:MM:SSZ Forge 1 (Forgejo): starting` and `HH:MM:SSZ Forge 1 (Forgejo): started`,
  and no `api` or `person` line. `<data>/box/logs/forgejo-1.log` holds them, each after its time.
- Everything else the terminal says is unchanged. `box.html`'s page around it is the mock's story: nothing
  of it is built.

## Inputs

- `temper/docs/plan/ui-welcome/notes/common.md`.
- `temper/docs/plan/ui-welcome/inputs/request.md`, section 9.
- `temper/docs/design/protocol/setup.md`, 7.2 (*Its terminal*, *Its logs*) and 7.3 (`label`).
- The mock: `temper/docs/design/client/greenfield/app.js`, `boxPage()`.

## Owns and may change

Owns the files in `files`.

May also change, each for these lines alone: `temper/crates/temper/tests/process.rs` (`assert_box_streams`,
`owned_hands_line`), `tests/suite/startover.rs:136`, `tests/suite/box_kinds.rs`,
`tests/support/first_repository.rs` (`live_from_hands`) and `tests/support/forge_land/rung.rs`
(`setup_observer`), which read these lines from stderr and now read the terminal's `started` and the log's
hands. The last two are real-forge tiers: check them with `heavy cargo check -p temper --tests`, and say
that they did not run. `temper/docs/design/protocol/setup.status.md`: delete "7.2, 7.3".

`temper/docs/help/**` is read by tests: run `heavy cargo nextest run -p temper-client-tools --lib -E 'test(help)'`
and `-p temper --lib -E 'test(help)'`.

Options, pre-approved:

- Nothing but tests sets a notice on an always step. If so, `Fact::Started` and the `notice` it rides on in
  `machine.rs` go for what they served; say so. If something does, it keeps its line.
- The words stay as `owned_notices` writes them (`Forgejo: api ...`, `Forgejo: person LOGIN`). 7.2's "read
  header PATH, write header PATH" is not this task's: leave it if it's a recorded gap, else add it to
  `setup.status.md`'s **Departs**.

## Steps

1. **The label.** `temper-forge-setup/src/step.rs`: the owned group's `label` is `b"Forge $forge (Forgejo)"`
   (`FORGE_ARGUMENT`). `temper-box/src/host.rs`'s binding, where it sets `asked.forge` and the step, also sets
   `placed.label` with the number in its `$forge`, by a function of `layout.rs`'s beside `bound`. A `label`
   with no `$forge` is unchanged. `terminal.rs`'s bound on a line still holds: the longest is `Forge 65535
   (Forgejo)`; test it.
2. **The terminal.** `Formatter::fact`: `Fact::GroupStarted` always says `<label>: started\n`.
3. **The log.** The host, at a group's `started`, writes the step's `started_notice` to that step's log with
   `Logs::put` as a `log_only` piece. `Logs` holds one piece at a time: wait for `takes()` as its other
   writers do, so that the notice is lost by nothing but a failed log, and nothing else is held up. `owned_notices`
   in `box_command.rs` no longer writes `Forgejo: started`.
4. **Tests,** `temper-box/src/tests/{terminal,machine,host}.rs`: a label bound to its number on each `Group*`
   fact; the fake kind's `Playground` unchanged; the notice in the step's log and not on the terminal, with its
   time, once; the largest line within the bound. `temper/tests/process.rs`'s streams test: the terminal's two
   lines, and the log's hands.
5. **Docs.** `docs/help/box.md`: where the two lines are, `DATA/box/logs/forgejo-FORGE.log`, and the terminal's
   new words. `docs/development/qa.md` likewise.
6. **Check,** `notes/common.md`: `temper-box`, `temper-forge-setup`, `temper`'s `process` and `suite` for
   `box_kinds` and `startover`. Commit and push.

## Report

A sample of the terminal's lines and of the log's, from a run; what you did not run.
