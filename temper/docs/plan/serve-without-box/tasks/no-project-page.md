+++
id = "no-project-page"
part = "pages"
kind = "feature"
needs = ["client-reads-deployment"]
files = ["temper/crates/temper-client-view/src/pages/chats.rs", "temper/crates/temper-client-view/src/words.rs", "temper/crates/temper-client-view/src/tests/chats.rs", "temper/crates/temper-client-shell-terminal/src/tests/render.rs", "temper/docs/design/client/README.status.md"]
effort = "medium"
minutes = 30
+++

# With no welcome and no project, the chats page says so

## Goal

On a deployment with no welcome, with no project yet, the chats page's main is the heading "No
project yet" and the line "Make one from the project switcher.", in both shells, and offers neither
"Set up temper" nor "Try the playground". "+ New project" in the switcher stays the way to make one.

Acceptance:

- `pages/chats.rs`, `main`: with `me` known, `me.projects` empty and `root.forge().deployment()`
  `Some` with `welcome` false, the pane's bar and list give way to the heading (level 1,
  `Word::NoProjectYet`) and the line, a new word `Word::NoProjectLine`.
- With `welcome: true`, or the fact unknown, the page draws what it draws today: the box's lag in
  `README.status.md` stays.
- View test `no_welcome_and_no_project_says_so`, and `the_chats_page_offers_nothing_of_the_welcome`:
  neither "Set up temper" nor "Try the playground" is anywhere in the view's tree.
- A terminal render test, `no_project_reads_the_same_in_the_terminal`, that the words reach it.
- `heavy cargo nextest run -p temper-client-view -p temper-client-shell-terminal` pass.
- `README.status.md` loses "3.5, With no project yet, with no welcome: its heading and line."

## Inputs

- `temper/docs/plan/serve-without-box/notes/common.md`; `notes/contracts.md`, K5.
- `temper/docs/design/client/README.md`, 3.5 (With no project yet).
- `temper/docs/design/draft/first-run-projects.md`, 7.

## Owns and may change

- The files of the header; a test file other than `tests/chats.rs` if chats' tests live elsewhere.
- `temper-client-view/src/words.rs`: the new word in each of its tables, as `NoProjectYet` is.
- The word's test (`words`' every-word checks): whatever a new word needs.
- `frame.rs`'s `switcher` already draws the form with no project: no edit.

## Steps

1. The word; the branch at the top of `main`'s pane, closing `Main` as the page does.
2. The tests; the implementer's checks; commit and push.

## Report

As the implementer's text asks.

## Rebase (2026-11-04, step merge, foreman serve-without-box/no-project-page/foreman-1)

combine both branches’ word-table counts in words.rs during rebase
The merge conflicts in `temper/crates/temper-client-view/src/words.rs`, around `PHRASES`, `SEGMENTS`, and `ALL`.
Main’s `d0b5f0fe52` adds Forgejo-sheet words; `f2af118844` adds Ready words and removes two obsolete words.
Preserve those changes and this task’s `NoProjectLine` enum entry, `ALL` entry, ID `2197`, and English text.
Against the refused integration tip, the combined declarations are `PHRASES = 1266`, `SEGMENTS = 2163`, and `ALL: [Word; 671]`.
Keep both sides’ explanatory comments.
Retain the chats-page behavior, tests, and task-specific status-line removal; preserve other status edits.
Validate with `TEMPER_BUNDLE=none heavy cargo nextest run -p temper-client-view -p temper-client-shell-terminal`.
