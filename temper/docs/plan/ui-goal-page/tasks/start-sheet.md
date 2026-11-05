+++
id = "start-sheet"
part = "start-sheet"
kind = "feature"
needs = ["start-domain", "goal-composer"]
files = [
  "temper/crates/temper-client-view/src/pages/start_goal.rs",
  "temper/crates/temper-client-view/src/pages/goals.rs",
  "temper/crates/temper-client-view/src/pages.rs",
  "temper/crates/temper-client-view/src/root.rs",
  "temper/crates/temper-client-view/src/words.rs",
  "temper/crates/temper-client-view/src/tests/start_goal.rs",
  "temper/crates/temper-client-view/src/tests/goals.rs",
  "temper/crates/temper-client-view/src/tests.rs",
  "temper/tools/gallery/names.txt",
  "temper/tools/gallery/src/samples.rs",
  "temper/tools/gallery/src/worlds.rs",
  "temper/tools/gallery/src/tests.rs",
  "temper/crates/temper-client-shell-terminal/src/tests/golden/goals.txt",
]
effort = "high"
minutes = 90
+++

# The Start goal sheet

## Goal

The sheet, and where it opens, as `temper/docs/design/client/README.md`, 5.14, says.

Acceptance:

- `pages/start_goal.rs`, `Role::Dialog` over the page in `look::SHEET`, drawn by `root.rs` after the page while `Pages::start()` is `Some`, on any page: `h2` and `.muted`, or `h2` "Start a goal" and its Draft `.field`; `.field` "Name";
  `fieldset` "How it runs", a `ChoiceGroup` of `label.option` per `goals::ways`, the default adding " PROJECT’s default." and chosen; `.link` "The project’s ways of working" to `Settings { project }`; `.foot` "Cancel", with `Shortcut::ESCAPE`, and
  "Start goal", disabled while Name or Draft is empty or a request is out; `.sheet-error` in jig's words on `StartGoal.error`.
- **No "Budget", no "When it needs you".**
- **On Goals:** the `.link` "Start one from a draft" at the section head, in `.temper-goals-section-head`, while the project has a tracked template, `Act::OpenStart { draft: None, chat: None }`.
- `goals::Way` gains `number: u32`, the template's.
- **Gallery:** `start-goal`, `start-goal-draft`, `start-goal-refused`.
- `temper/docs/design/client/browser.status.md`'s 7.2 line is gone, and its 2.5's.
- The tests below pass, each having bitten; the crates green whole.

## Inputs

- `temper/docs/plan/ui-goal-page/notes/common.md`, `notes/contracts.md`.
- `temper/docs/design/client/README.md`, 5.14; `browser.md`, 2.5.
- The mock's `startSheet`, `startGoal`.
- `temper-client-view/src/forge/connect.rs`: the welcome's sheet, the same kind of thing.

## Owns and may change

Owns, as `files` lists them.

Options, each pre-approved:

- **`ui-settings`' default way, or the Chats area's Drafts filter or `data-start-goal`, on your base:** if `ui-settings` has landed and gives a default way, read it where `default_way` does, with its one line; if the Chats area's Drafts filter has landed,
  the link goes to `Chats` under Drafts; if its "Start goal…" has landed, wire it to `Act::OpenStart` with its draft and chat. If not, leave each and say so in the report. `ui-away`, likewise, with its fieldset (the request's section 5).
- **`Address::Settings`' fields:** as your base has them.
- **The terminal:** the dialog prints as `confirm.rs`' does; `goals.txt` gains the link, regenerated.

## Steps

1. The view, by the note and 5.14.
2. **Tests,** `tests/start_goal.rs`: `no_draft_asks_for_one`, `a_draft_names_the_sheet_and_its_name`, `one_option_per_way_in_order`, `the_default_is_chosen_and_says_so`, `no_budget_and_no_asks`, `start_is_disabled_while_empty_or_out`, `cancel_carries_escape`, `the_error_is_jigs_words`, `its_settings_link`; `tests/goals.rs`: `the_link_shows_with_a_tracked_template`, `none_with_none`.
3. `heavy cargo nextest run -p temper-client-view -p temper-client-gallery -p temper-client-shell-terminal`.
4. **Check,** the implementer's text, then push.

## Report

As `notes/common.md`, Your report: which of the Options' other plans were on your base.

## Answers

2026-11-05 (fabrig/ui). Ruling (rulings.md, 3, 7): see start-domain's answer. Goals' "Start one from a draft" is a link to Chats' Drafts filter, no draft field; ui-chat-draft adds opening the sheet from a chat, so export what it needs. ui-away adds "When it needs you" later.

## Rebase (2026-11-05, step merge, foreman ui-goal-page/start-sheet/foreman-1)

merge start-sheet additions with the landed Chats, NewChat and session changes
The merge conflicts are in `temper/crates/temper-client-view/src/pages.rs`, `temper/crates/temper-client-view/src/words.rs`, and `temper/tools/gallery/src/samples.rs`.
Rebase onto the landing trunk; integration commit `a7654118c9` introduced the overlapping catch-up, retained by `7b7ce7cdcf` and `8f392c1df8`.
In `pages.rs`, retain all three module declarations: `new_chat`, `session`, and `start_goal`.
In `words.rs`, preserve the trunk’s vocabulary, phrase IDs and sizing changes, then add the branch’s eight start-sheet words, IDs 3450–3457, and English entries.
Against integration `8f392c1df8`, the combined counts are `PHRASES = 1521`, `SEGMENTS = 2663`, and `ALL: [Word; 925]`.
In gallery `samples.rs`, preserve all landed samples, `Then` variants, disclosure support and domain setup; add the three start-goal samples and `start_domain` dispatch.
The combined gallery count is `N = 169`. Keep the trunk’s `Address::Chats { filter: ChatsFilter::All }`; its removed `start` field must not return.
Preserve the automatically merged NewChat dispatch and the branch’s sheet rendering after the page in `root.rs`.
Update the new `start-goal` gallery assertion in `temper/tools/gallery/src/tests.rs`: its underlying Chats heading is now “Talk to an agent”, replacing “Chats”.
Regenerate any combined gallery names, prints or terminal goldens made stale using their owning tools, preserving both plans’ entries.
Run the focused start-goal tests, then `heavy cargo nextest run -p temper-client-view -p temper-client-gallery -p temper-client-shell-terminal`.
