+++
id = "charter-triage"
part = "charter"
kind = "docs"
needs = []
files = ["temper/docs/examples/reference/settings.json", "temper/crates/temper/src/tests/reference_settings.rs"]
effort = "medium"
minutes = 30
+++

# The coordinator's charter triages reds and notes what it learns

## Goal

Charter 5's instructions in the reference's settings file gain the triage of a red of the goal
branch's tier, the triage and repair of a red after landing (`repairs`), and notes on what the
coordinator learns, as `temper/docs/design/protocol/configuration.md`, 13.9 (charter 5's
triage) has them. The text is given whole below; place it as given.

Acceptance:

- In `temper/docs/examples/reference/settings.json`, charter 5's `instructions` (a JSON string
  whose paragraphs are joined by `\n\n`) holds three new paragraphs, each its own paragraph:
  - **A** right after the paragraph that starts `On a question or an escalation,`;
  - **B** right after the paragraph that starts `When it is held past its bounds`;
  - **C** right after B.
- The paragraph that starts `When its result reaches you, finish:` ends, in place of `and the
  landing into main by its pull request and merge commit.`, with `and the landing into main by
  its pull request and merge commit, with the repair that cleared it if it was red after
  landing.`
- Nothing else in the file changes; the last paragraph stays the last.
- `temper/crates/temper/src/tests/reference_settings.rs` asserts, beside its `"branch": "mine"`
  check, that the coordinator's instructions contain `"repairs"`, `a red of your goal branch's
  tier` and `with \`note\``, and still end as they do.
- `heavy cargo nextest run -p temper --lib -E 'test(=tests::reference_settings::the_reference_settings_file_decodes)'`
  passes (find the test's exact path with `--list` if that name selects nothing).

The paragraphs, exactly (in the JSON string, each `"` escaped as `\"`):

**A:**

> When a red of your goal branch's tier reaches you as news, triage it: read your notes, the
> failing tests, the log's end, the tree and the design, and find which change owns the failing
> files and whether it has landed. A failure in a live change's files is that change's: amend
> its words to say so. For the rest, make one repair change into your goal branch, with
> `"landing": "goal"`, its words the failing tests, the log's end, and the live changes with
> the files they own, to leave alone. Other changes keep merging meanwhile. A test marked still
> failing that a live repair already owns needs nothing more, and a test your notes name as a
> flake needs no repair.

**B:**

> Once your landing has landed, main's tiers verify it. When a red after landing reaches you as
> news, it names your landing's commit, the failing tests blamed on it, the run and the log's
> end. Triage it the same way, reading your goal's history, now on main. Then make one small
> repair change into main, with no `landing` and with `"repairs"` naming the red landing's
> commit as the news gives it, its words the failing tests and what your triage found. It lands
> by main's queue, and the next tier run that passes those tests clears your landing, repaired
> by it; your landing's result then reaches you. A failure you find to be a flake, or failing
> before your landing, gets no repair: say so with `note`, in the repository's scope, naming
> the tier and the test, where the next triage finds it.

**C:**

> Note what you learn as you go with `note`, in your goal's scope or the repository's: a slow
> check, a dead end, a trap in the code, a flaky test. Read your notes before each triage.

## Inputs

- `temper/docs/plan/reference-m9/notes/common.md`.
- `temper/docs/design/protocol/configuration.md`, 13.9 (charter 5's triage; the rules' notes
  family is applied on the box, not in this file).

## Owns and may change

- Owns the two files of this spec's header, for what acceptance names.
- reference-m5 (`reference-relay`) and reference-m7b may add their own paragraphs to the same
  instructions at the same time: keep theirs as they are, and place A, B and C by the
  paragraphs named above wherever theirs land.
- If the settings form refuses the longer instructions for their size, stop and ask, with the
  bound the form gives and the instructions' new size.

## Steps

1. Edit the JSON string; check it with `python3 -I -c 'import json,sys; json.load(open(sys.argv[1]))' temper/docs/examples/reference/settings.json`.
2. The test's assertions; run it as acceptance says.
3. `cargo fmt`; `heavy cargo clippy -p temper --all-targets -- -D warnings`; commit and push.

## Report

As the implementer's text asks.
