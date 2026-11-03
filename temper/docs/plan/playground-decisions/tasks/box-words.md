+++
id = "box-words"
part = "box-words"
kind = "feature"
needs = []
files = [
  "temper/crates/temper-client-domain/src/forge/words.rs",
  "temper/crates/temper-client-domain/src/tests/forges.rs",
  "temper/crates/temper-client-view/src/words.rs",
  "temper/crates/temper-client-view/src/forge/connect.rs",
  "temper/crates/temper-client-view/src/tests/connect_forge.rs",
  "temper/crates/temper-client-tools/src/forges.rs",
  "temper/crates/temper-client-tools/src/tests/forges.rs",
  "temper/tests/client/tests/suite/forges.rs",
  "temper/tests/client/tests/suite/forges_edges.rs",
  "temper/tests/client/tests/suite/first_repository.rs",
  "temper/docs/help/reference/ctl-forge.md",
  "temper/docs/design/client/tools.status.md",
  "temper/docs/design/client/forge.status.md",
]
effort = "medium"
minutes = 60
+++

# Without the box, the page and ctl say to start temper with `temper box`

## Goal

Without the box, an owned Forgejo is never set up, since its group is the box's
(`temper/docs/design/protocol/setup.md`, 7.4). The page, the Forges row and ctl still print the
four `temper forge` commands and a restart, which find no group's file for a forge the box did
not run. The user accepted the design's words for now (`inputs/request.md`, decision 5): bring
the code to them.

Acceptance:

- Forgejo on this machine's Details, not under the box, show the heading "Set it up under temper
  box", then one paragraph, "temper was not started by temper box, so it does not start a
  Forgejo itself. Start temper with temper box to set one up here.", and no action, no code
  block and no other paragraph (`temper/docs/design/client/forge.md`, 5.4, *Forgejo on this
  machine*).
- A not set up owned row without the box reads "Not set up: start temper with temper box"
  (`client/forge.md`, 5.4, the connection table's *not set up*), on the page and in `temper ctl
  forge list`.
- `temper ctl forge setup` refused without the box prints `refused: not under the box`, then the
  line ``Start temper with `temper box` to set one up here.``, exit 2; `--json` carries the same
  words in its `next` (`temper/docs/design/client/tools.md`, 5.5, `forge setup`: "saying to start
  temper with `temper box`").
- `forge setup`'s notes say so, and `temper/docs/help/reference/ctl-forge.md` is regenerated
  from the table to match.
- No `temper forge fetch|prepare|run` command is named by any of these; `set_up_commands` is
  gone.
- The two status entries below are gone; the tests below pass.

## Inputs

- `temper/docs/plan/playground-decisions/notes/common.md`.
- `temper/docs/design/client/forge.md`, 5.4: *Forgejo on this machine*'s "Not under the box"
  sentence (line ~600) and the connection table's *not set up* row (line ~771).
- `temper/docs/design/client/tools.md`, 5.5: `forge setup`'s refusals.
- `temper/docs/design/protocol/setup.md`, 7.4 (why: the commands find no group's file).
- The mockup already draws the words: `temper/docs/design/client/mockups/pages.js`,
  `ownedNotUnderTheBox` (read only).

## Owns and may change

- `temper-client-domain/src/forge/words.rs`: `set_up_commands` is removed. In its place:

  ```rust
  /// ctl's line after a set up refused "not under the box" (client/tools.md, 5.5).
  pub const UNDER_THE_BOX: &[u8] = b"Start temper with `temper box` to set one up here.";
  ```

  and `forge_connection`'s `C::NotSetUp` arm with `Offer::NotUnderTheBox` returns
  `b"Not set up: start temper with temper box"`. `join_words` stays if other callers use it.
- `temper-client-view/src/words.rs`: the three words keep their names, numbers and shapes (one
  literal segment each, no holes), so `PHRASES` and `SEGMENTS` do not change; only their texts:
  `ForgeTerminal` "Set it up under temper box"; `ForgeTerminalDetails` "temper was not started by
  temper box, so it does not start a Forgejo itself."; `ForgeTerminalNext` "Start temper with
  temper box to set one up here.". Keeping the counts keeps this branch off the `PHRASES` line
  another draft's branch (`reference-m8/panel-steps`) changes.
- `temper-client-view/src/forge/connect.rs`, `here`: when `terminal`, the Details heading
  `ForgeTerminal`, then one `Role::Paragraph` holding `ForgeTerminalDetails`, a space and
  `ForgeTerminalNext` as plain runs; the `FLOW_CODE` block and its loop go. Nothing else of
  `here` changes.
- `temper-client-tools/src/forges.rs`: `setup_text` puts `words::UNDER_THE_BOX` and a newline
  after the refusal line; `setup_json` puts it as `next`; both still only when `forge != 0`, as
  today. `forge setup`'s `notes` (line ~59): its last sentence becomes ``"`refused: not under the
  box` (exit 2): the deployment is no box; start temper with `temper box` to set one up."`` (wrap
  the notes' lines as the rest are wrapped).
- `temper/docs/help/reference/ctl-forge.md`: regenerated, never hand-written, by
  `SKEIN_CODEGEN_WRITE=1 heavy cargo nextest run -p temper-client-tools --lib -E
  'test(=tests::help::the_forge_reference_is_what_the_table_renders)'` (find its exact path with
  `--list` first); the regenerated diff is the notes' sentence alone. This departs from "Not
  yours: `docs/help/`" because the page is the table's rendering, which the test checks. If any
  other help golden (`temper-client-tools/src/tests/golden/`) holds the notes, regenerate it the
  same way.
- Tests, each asserting the new words where it asserted the commands (rename a test whose name
  says "commands"):
  - `temper-client-domain/src/tests/forges.rs:310` (`forge_connection`'s words);
  - `temper-client-view/src/tests/connect_forge.rs:168`
    (`forgejo_here_without_the_box_lists_its_four_commands`): heading, the one paragraph's words,
    no button, no `FLOW_CODE` node;
  - `temper-client-tools/src/tests/forges.rs:173` (the row) and `:286` (the refusal's text);
  - `temper/tests/client/tests/suite/forges.rs:362`
    (`without_the_box_the_row_and_setup_refusal_name_the_manual_commands`), `forges_edges.rs:397`
    (`not_under_box_explains_all_four_commands_without_action`) and `first_repository.rs:179`
    (`without_the_box_the_flow_names_the_commands_and_offers_no_setup`): each asserts the new
    words and that `temper forge fetch` appears nowhere.
- Status, as `notes/common.md` says:
  - `temper/docs/design/client/tools.status.md`, **Departs**: remove the sentence that begins
    `5.5: \`forge setup\` refused "not under the box" prints the four` through `which the box's
    group files leave without a way.`; the rest word for word.
  - `temper/docs/design/client/forge.status.md`, **Departs**: remove the sentence that begins
    `5.4: without the box, Forgejo on this machine's Details say "Set it up from a terminal"`
    through its end, `leave without a way.`; the `?forge=elsewhere` entry before it stays word for
    word.

Pre-approved: if `rg -n 'set_up_commands|temper forge fetch|Set it up from a terminal|ForgeTerminal'
temper/crates temper/tests` finds a reader not listed here, change it the same way (its assertion
or call site only) and list it in the report. Leave `temper/docs/help/forge.md` and `box.md` as
they are (they say `forge setup` is refused without the box, still true). Leave the box's own
`temper forge` commands and their help untouched: they stay for the box, which runs them.

## Steps

1. The domain's words and their test; `heavy cargo nextest run -p temper-client-domain --lib -E
   'test(forges)'`.
2. The view's words and `here`, and `connect_forge.rs`; `heavy cargo nextest run -p
   temper-client-view --lib -E 'test(connect_forge)'` (the phrasebook's own tests in
   `words.rs` run with it: check they pass with the counts unchanged).
3. ctl: `forges.rs`, its tests, the notes, then the regenerated reference page; `heavy cargo
   nextest run -p temper-client-tools --lib`.
4. `temper/tests/client`: the three tests; `heavy cargo nextest run -p temper-client-world
   --test suite -E 'test(without_the_box) | test(not_under_box)'` (three selected).
5. `heavy cargo nextest run -p temper-docs` (the help pages' rules read the regenerated page).
6. The status edits; the citations check (`notes/common.md`).
7. `cargo fmt`; old forms for temper; `heavy cargo clippy -p temper-client-domain -p
   temper-client-view -p temper-client-tools -p temper-client-world --all-targets -- -D
   warnings`; the focused tests of those packages. Commit the code and tests, then the status
   text; push.

## Report

As the implementer's text says, with the regenerated page's `git diff --stat` and any reader
found beyond the list.
