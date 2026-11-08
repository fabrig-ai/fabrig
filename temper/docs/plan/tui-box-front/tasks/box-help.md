+++
id = "box-help"
part = "box-front"
kind = "docs"
needs = ["binary-front"]
files = [
  "temper/docs/help/box.md",
  "temper/docs/help/tui.md",
  "temper/docs/design/protocol/setup.status.md",
]
effort = "high"
minutes = 45
+++

# `temper help box` and `temper help tui` say the TUI runs in front of a box

## Goal

After `binary-front`, `temper box` in a terminal runs the TUI in front of
it, and `--no-tui` runs it as before (`temper/docs/design/protocol/setup.md`,
7.6; `temper/docs/design/client/README.md`, 6.17). The walk `temper help
box` reads the printed link from the box's stdout in its steps 2 and 3, and
`temper help tui` says what Ctrl+C does: both must say so, the walk within
its bound, losing no fact.

Acceptance:

1. **`temper/docs/help/box.md`** (a Walk, at most 12,288 bytes,
   `temper-docs/src/lib.rs`' `Kind::bound`; 12,282 today):
   - step 2's two commands gain `--no-tui` (`temper box --no-tui --config
     CONFIG --data DATA --cache CACHE`, and the fake one), and step 2 says,
     in a sentence or two, that in a terminal without `--no-tui` the TUI
     runs in front instead (`temper help tui`): it shows the link and is
     signed in, the block, the fake backends' `box:` lines and errors go to
     `DATA/box/logs/box.log`, and quitting it stops the box; that this walk
     reads the link, so it keeps `--no-tui`, as on a pipe;
   - the page condensed elsewhere to stay within its bound, every fact
     kept: rewording, not removing; no fenced command line that a test
     reads changes.
2. **`temper/docs/help/tui.md`** (a Concept, 1,450 bytes today): a short
   paragraph or bullet: `temper box` in a terminal runs this TUI in front
   of it (`temper help box`), signed in with nothing pasted, its welcome
   showing the box's link for a browser on this machine; in front,
   Ctrl+C or `/exit` stops the box too, asking first while work runs;
   `temper box --no-tui` runs the box without it. Every sentence its test
   reads stays (`tui_help_explains_closing_and_ending_the_sign_in`).
3. **Tests read them and pass unchanged:** `temper/crates/temper/src/tests/help.rs`
   (`tui_help_explains_closing_and_ending_the_sign_in`,
   `box_help_configures_coding_run_tools`,
   `box_step_seven_names_every_ending_and_the_direct_report`,
   `box_walk_distinguishes_stock_policy_from_open_budgets`,
   `box_walk_creates_a_project_before_it_is_funded`,
   `box_walk_reads_the_conversation_after_waiting_for_a_reply`,
   `box_help_creates_the_first_project_before_using_its_number`,
   `box_and_tier_help_name_the_host_runners_node_prerequisite`,
   `box_help_names_delivery_checks_path_and_tools`,
   `box_and_tier_help_explain_shared_capacity_and_existing_boxes`,
   `box_and_change_help_locate_temper_review_gates_in_change_show`), and
   `temper/crates/temper-docs/src/tests/` (`checks.rs`' bounds,
   `rules.rs`' `box_help_locates_forgejo_lifecycle_and_hands` and its
   prose rules, `paging_search.rs`).
4. **Status:** delete the **Not yet** bullet naming `box-help` in
   `temper/docs/design/protocol/setup.status.md`. No other design text.

## Inputs

- `temper/docs/plan/tui-box-front/notes/common.md` (the help pages are
  closed but where a spec says otherwise: this one opens these two).
- `temper/docs/design/protocol/setup.md`, 7.6, *When*, *The terminal is
  the front's*, *Its end is the box's stop*; `temper/docs/design/client/README.md`,
  6.17 (its words for the welcome, quitting and `/log`).
- `binary-front`'s box row notes in `temper/crates/temper/src/commands.rs`
  (the words the reference page `commands` now uses: agree with them).
- The pages and their readers: `temper/docs/help/box.md` (step 2, about
  lines 102-148; step 3, about 160-170), `temper/docs/help/tui.md`;
  `temper/crates/temper-docs/src/lib.rs` (`Kind::bound`, about line 54);
  the tests of acceptance 3.

## Owns and may change

Owns the two pages and the status edit. Not `reference/commands.md`
(generated from `commands.rs`, `binary-front`'s), not any other page, not
the code. If a sentence a test reads must change to fit the bound, don't:
condense elsewhere; if nothing else can give, say so in the report with
the test's name and stop there.

## Steps

1. Edit `box.md`'s step 2, then condense to the bound, `wc -c` after each
   pass; edit `tui.md`.
2. Runs, through `heavy`, counts in the report: `heavy cargo nextest run -p
   temper-docs`; `heavy cargo nextest run -p temper --lib -E
   'test(/tests::help::/)'`.
3. The status edit; `python3 -I -B temper/scripts/old_forms.py`;
   `python3 -I -B tools/citations/citations.py --base origin/main`.
4. Commit (why: the help says that `temper box` in a terminal runs the TUI
   in front and how the walk keeps reading the link), with
   `Look-request: tui-box-front` directly above the other trailers; push.

## Report

As the implementer's text says: the pages' sizes before and after, each
condensation and the fact it keeps, the runs' counts.
