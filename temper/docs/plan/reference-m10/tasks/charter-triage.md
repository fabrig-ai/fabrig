+++
id = "charter-triage"
part = "triager"
kind = "docs"
needs = ["charter-close"]
files = ["temper/docs/examples/reference/settings.json", "temper/crates/temper/src/tests/reference_settings.rs"]
effort = "medium"
minutes = 40
+++

# The reference's triage charter; the coordinator delegates a triage and lets designers ask

## Goal

The reference's settings file gains charter 8, the triage's, read-only, and charter 5's
instructions may delegate a red's triage to it and give each designer's change `"questions":
"up"`, as `temper/docs/design/protocol/configuration.md`, 13.9 ("A triage delegate") has it.
The texts are given whole below; place them as given.

Acceptance:

- In `temper/docs/examples/reference/settings.json`, a new last item, charter 8, whose `set`
  is charter 6's (the foreman's item) member for member but: `"number": 8`; `instructions`
  **T** below; `"waiting": 0` and `"resume": false`. So: gpt-6.1-sol as charter 6 has it,
  `inspect` true, `modify`, `shell` and `agents` false, the workers, charter 6's turns, time,
  reserve, call timeout and spend, no documents, the default wake policy.
- Charter 5's `instructions` (paragraphs joined by `\n\n`):
  - the paragraph that starts `When a red of your goal branch's tier reaches you as news,`
    gains, at its end, after `needs no repair.`, one space and sentence **D** below;
  - in the paragraph that starts `On your first run, read the draft,`, the text `Give every
    designer's change "llm":` becomes `Give every designer's change "questions": "up" and
    "llm":`, nothing else of it changing.
  Nothing else in the file changes.
- `temper/crates/temper/src/tests/reference_settings.rs`: `ITEMS` is 8, its comment saying
  five charters; a `TRIAGE: u32 = 8` beside the other numbers; in
  `the_reference_settings_file_decodes`, charter 8 is asserted inspect-only (`inspect()`, not
  `modify()`, `shell()` or `agents()`), its model charter 6's, its wake the default, no
  documents, its instructions containing `You never edit, push or run anything.`; and the
  coordinator's instructions containing `delegate its triage to charter 8` and `"questions":
  "up"`. Its other assertions stay.
- `heavy cargo nextest run -p temper --lib -E 'test(/tests::reference_settings::/)'` passes,
  every test it selects (find the module's path with `--list` if that selects nothing).

**T**, exactly (one paragraph; in the JSON string, each `"` escaped as `\"`):

> You triage a red for a goal's coordinator. Your words carry the red's news: the tier, the
> failing tests, the run and the end of each failed job's log. Read the tree, the design, the
> goal's history and the notes with your reads, only what the failures need. Finish with a
> report: for each failing test, the change or landing whose files it fails in; whether it is
> new, failing before (inherited) or a flake, with what shows it; and the repair you would
> make, with the files that repair would own and the live changes it must leave alone. You
> never edit, push or run anything. Where nothing you read settles a test, say so for it.

**D**, exactly:

> Where the failures span files no one change owns, you may delegate its triage to charter 8
> with `delegate`, read-only, its words the news, and act on its report as on your own triage.

## Inputs

- `temper/docs/plan/reference-m10/notes/common.md`.
- `temper/docs/design/protocol/configuration.md`, 13.9 ("A triage delegate"; charter 5's
  triage). The rules' and policy 1's delegation to charter 8 is applied on the box (13.9's
  rules and policy 1), not in this file.

## Owns and may change

- Owns the two files of this spec's header, for what acceptance names. Task `charter-checks`
  may edit charter 2's item and the coder's assertions at the same time: leave those alone; a
  conflict at fab's merge is fab's.
- If the settings form refuses a ninth charter number or the item count (a bound it names),
  stop and ask, quoting the bound.

## Steps

1. Edit the JSON; check it with `python3 -I -c 'import json,sys;
   json.load(open(sys.argv[1]))' temper/docs/examples/reference/settings.json`.
2. The test's assertions; run it as acceptance says.
3. `cargo fmt`; `heavy cargo clippy -p temper --all-targets -- -D warnings`; commit and push.

## Report

As the implementer's text asks.
