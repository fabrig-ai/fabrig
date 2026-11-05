+++
id = "goals-rail"
part = "goals-rail"
kind = "feature"
needs = ["goals-gallery", "has-goals"]
files = [
  "temper/crates/temper-client-view/src/frame.rs",
  "temper/crates/temper-client-view/src/words.rs",
  "temper/crates/temper-client-view/src/tests/rail.rs",
  "temper/tools/gallery/names.txt",
  "temper/tools/gallery/src/samples.rs",
  "temper/tools/gallery/src/tests.rs",
]
effort = "high"
minutes = 75
+++

# Goals in the rail and the tab bar

## Goal

The rail and the tab bar offer Goals where the project has one, as `temper/docs/design/client/README.md`, 5.3 says.

Acceptance:

- "Goals" between "Needs you" and "Chats", to `Address::Goals { project, filter: All }`, shown where `root.has_goals(project)` or the page is `Goals`.
- Current on `Goals`; on a goal (`Shape::Tracked`, `root.pages().task()`); on a task whose document names a goal (`Task.goal`, which includes a goal itself, read by `.document(root.client())`). A chat stays Chats' or Settings'. A document not yet delivered is current on nothing.
- **The tab bar,** four: Home, Needs you, Goals, Chats with Goals shown; Home, Needs you, Chats, Settings without, as the rail's.
- The terminal prints the rail's entry only where it shows: goldens with no goal are unmoved; run them to show it.
- A gallery state with a project at work on Home with Goals in the rail, `home-with-goals`, in `names.txt`, `samples.rs` and `HOLDS`.
- `temper/docs/design/client/README.status.md`'s *Not yet* keeps only what isn't closed: 5.3's words go here.

## Inputs

- `temper/docs/plan/ui-goals/notes/common.md`, `notes/contracts.md`.
- `temper/docs/design/client/README.md`, 5.3.
- `temper/crates/temper-client-view/src/frame.rs`, `entries`, `tabbar`; `src/tests/rail.rs`; the mock's `tabbar`.

## Owns and may change

Owns, as `files` lists them. May also change the gallery's prints in tests and the terminal's goldens where a project has goals, by regenerating.

Options, each pre-approved:

- **`ui-settings`:** if it has landed, the rail's test is its kind 7's `goals`, as its request says, and `has_goals` stays.
- `entries` takes `goals: bool`, or `tabbar` builds its own list; your call. Duplicate no entry's words.
- Solo's and Reference's other states now show Goals in their rail: this is right, and their tests' names lists move with it.

## Steps

1. **`frame.rs`.**
2. **Tests,** `tests/rail.rs`: `goals_shows_only_with_a_goal_or_on_its_page`; `goals_is_current_on_its_page_a_goal_and_a_goals_task`; `a_chat_is_never_goals`; `the_tab_bar_is_four_either_way`, with hrefs; the loop at `:223` gains the address.
3. **Check,** the implementer's text, with `temper-client-shell-terminal` and `temper-client-gallery`, then push.

## Report

As `notes/common.md`, Your report, with which goldens moved.
