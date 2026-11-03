+++
id = "charter-close"
part = "charter"
kind = "docs"
needs = []
files = ["temper/docs/examples/reference/settings.json", "temper/crates/temper/src/tests/reference_settings.rs"]
effort = "medium"
minutes = 30
+++

# The coordinator's charter ends the plan with its tier settled; its landing names the draft

## Goal

Charter 5's instructions in the reference's settings file make the close-out wait until no red
of the goal branch's tier is left unowned, and make the goal's landing's words name the draft,
as `temper/docs/design/protocol/configuration.md`, 13.9 ("Charter 5's plan done, and its
landing's words") has them. The text is given whole below; place it as given.

Acceptance:

- In `temper/docs/examples/reference/settings.json`, charter 5's `instructions` (paragraphs
  joined by `\n\n`): the paragraph that starts `When every change of the plan has landed or
  been cancelled,` is replaced whole by paragraph **P** below, in the same place. Nothing else
  in the file changes; the last paragraph stays the last.
- `temper/crates/temper/src/tests/reference_settings.rs`, in
  `the_reference_settings_file_decodes`, asserts beside its other `instructions.contains`
  checks that the coordinator's instructions contain `a live repair owning its failing tests`
  and `then the draft it was built from, by path,`.
- `heavy cargo nextest run -p temper --lib -E 'test(=tests::reference_settings::the_reference_settings_file_decodes)'`
  passes, one test selected (find its exact path with `--list` if that name selects nothing).

**P**, exactly (in the JSON string, each `"` escaped as `\"`):

> When every change of the plan has landed or been cancelled, every Left list is made into a
> change or accepted as left, and no red of your goal branch's tier that you have heard is
> left without a repair that landed after it or a live repair owning its failing tests, make
> the close-out: a designer's change, "producer": 7, its words what the goal leaves for the
> next one: the status files where the code still lags or departs, the designs' Later
> sections, and the draft's fate (folded into the design, kept, or removed). A green run of
> the tier tells you nothing, and a red still failing reaches your next run, so the reds you
> have heard are what you know: for one left unowned, make its repair first, and the close-out
> once that repair has landed. Wait for no run of the tier in flight: the landing's CI checks
> the whole. When the close-out has landed and nothing is left that the plan needs, land the
> goal: make one change with `"branch": "mine"` and no `landing`, its words the goal's title
> in one line, then the draft it was built from, by path, then what the goal built, by design
> section. It lands your goal branch into main whole: CI at its head runs the project's full
> gate, then it is queued, brought up to date with main if it lacks main's tip, and merged.
> Then end your run: you wake on its result or its hold.

## Inputs

- `temper/docs/plan/reference-m10/notes/common.md`.
- `temper/docs/design/protocol/configuration.md`, 13.9 ("Charter 5's plan done, and its
  landing's words").

## Owns and may change

- Owns the two files of this spec's header, for what acceptance names.
- If the settings form refuses the longer instructions for their size, stop and ask, with the
  bound the form gives and the instructions' new size.

## Steps

1. Replace the paragraph; check the file with `python3 -I -c 'import json,sys;
   json.load(open(sys.argv[1]))' temper/docs/examples/reference/settings.json`.
2. The test's two assertions; run it as acceptance says.
3. `cargo fmt`; `heavy cargo clippy -p temper --all-targets -- -D warnings`; commit and push.

## Report

As the implementer's text asks.
