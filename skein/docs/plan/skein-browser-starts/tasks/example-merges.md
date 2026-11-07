+++
id = "example-merges"
part = "example-tier"
kind = "feature"
needs = ["in-turn"]
files = [
  "skein/tests/ui-web/tests/browser_example.rs",
  "skein/tests/ui-web/tests/browser_stories.rs",
  "skein/tests/ui-web/tests/support/live_tree.rs",
  "skein/tests/ui-web/tests/support/tab.rs",
  "skein/docs/design/ui-web.status.md",
]
effort = "medium"
minutes = 75
+++

# The example's stories share a start, and the tab's press looks first

## Goal

`browser_example` and `browser_stories` start 29 Chromiums, one per test, most on one server and setup. They become 11 tests. `a_tab_opens_only_from_a_press` reads 26.8 s of wall, a quarter of the tier's, and its presses go to a page its tab has hidden; a person looks at a page before pressing it. **Nothing here runs in Chromium:** it's written and compiled, and after-landing runs it.

Acceptance:

- `browser_example` holds the 3 tests, and `browser_stories` the 8, of `plan.md`'s rows 39 to 67, by the names there. Every assertion of those rows is where its row says: the table is the checklist.
- `support/tab.rs` holds no `Play`: its person is a `Script`, every press a `Step::Press` by name.
- `heavy cargo nextest run -p skein-ui-web-tests --no-run` and `heavy cargo check -p skein-ui-web-tests --tests` pass. **No `browser_*` test runs,** not one, nor by name.
- The status edits below are made.

## Inputs

- `skein/docs/design/ui-web.md`, 14.1, 14.2 (*Stories in turn*, and the step that checks a closing warning) and 14.3.
- `skein/docs/design/browser.md`, 6: *A look brings its page to the front*.
- `skein/docs/plan/skein-browser-starts/notes/common.md`, and `notes/contracts.md`, *Stories in turn*.
- `skein/docs/plan/skein-browser-starts/plan.md`: the table's `browser_stories` and `browser_example` parts, *What this plan decided*, and *What the two tab tests wait for*.

## Owns and may change

Owns the files in the header, and in the status file only the lines below. `stories::*` and `Setup` don't change: rows 40 to 43 call them as jig's and temper's tiers do, and stay.

If `harness-world`, or a plan beside this one, has landed in `browser_stories.rs` or `lib.rs`, catch up. A story the table doesn't name stays working, and the report names it.

Options, each decided:

- **The example's server keeps its notes for the run.** That sets the order below, and each story but the first opens with `Go /`, which also drops a bar's filter.
- **Only the example's store is shared:** the `/test/...` fixtures hold their state in the page, and each `Go` is a new document.
- **Stories that fail to compile as a merge** stay as they are, and the report says why.

## Steps

1. **`browser_example.rs`,** three tests, each step's `within` as it is:
   - **`a_person_goes_through_the_example_in_turn`:** `example_harness().run_stories(..)`, `Outcome::Done`, these stories in this order:

     | Story | Row | Changes |
     |---|---|---|
     | `opens` | 56, 66, 44 | See the heading, `Gone` `Starting notes` (row 44's, moved from `browser_stories`), see `No notes yet.`, then row 66's four. Needs the empty store, so first |
     | `bar` | 65 | Needs only its own two notes, so second |
     | `link` | 59 | `Address` `/notes/3`: two notes come before it |
     | `fills` | 62 | |
     | `chooses` | 63 | |
     | `escape` | 64 | The title `Kept words`, not `Groceries`: its last find must not pass on `link`'s note |
     | `grows` | 67 | |

   - **`a_reload_meets_the_closing_warning_and_keeps_a_pending_add_made_once`:** rows 57, 60 and 61, one person, row 57's `Setup`:

     ```
     See Title; StopServer; Type Groceries; Press Add; Pause 100 ms; StartServer;
     Reload; Warned(true);                 // row 60
     See link Groceries;                   // row 57
     Type Milk; Press Add; See link Milk;  // row 61's acts, on a live page
     Reload; Warned(false);                // row 61
     See link Groceries                    // row 57: made once
     ```

     **Row 61 needs its own acts here.** The browser warns only on a page acted on since it loaded (`skein/docs/design/ui-web.md`, 14.2), so a reload straight after a reload would pass `Warned(false)` having shown nothing.
   - **`a_note_added_in_one_page_shows_in_another`** stays as it is.

   `harness::Setup` is built once now, where rows 57 and 60 had one each.
2. **`browser_stories.rs`:** rows 39, 40, 41, 42, 43 and 45 stay as they are. `a_placeholder_gives_way_to_the_client` goes (step 1).
3. **`support/live_tree.rs`:** `the_live_tree_keeps_its_structure_and_nodes`, one `Harness::new(super::setup())` and two people:
   - **Person 0, a `TreeStory`, plays rows 46 to 53 in turn.** `TreeStory::new()` takes no case. `story.case(name)` adds a new `Part::Case`, which clears `nodes` (a new document has new ones) and records `(name, at)`, then `Go` and `revision(0)` as `new` did. Each test's body becomes a function that takes the story.
   - **A step's number:** a nested `Script`'s `Next::Failed` comes back with `step: self.at`, so that `harness::name_failure(outcome, &starts)` can name the case. `Identity`'s `assert_eq!` names the case too.
   - **Person 1 plays row 54's `Script`.**
   - After `Outcome::Done`, by `person`: person 0's entries hold `value == name`, and aren't empty; person 1's are two, not empty, and equal.
   - `run` goes, with whatever else is left unused.
4. **`support/tab.rs`:** `TabStory`, `Part`, and the `impl Play` go. One `Script`:

   ```rust
   fn press(name: &[u8]) -> harness::Step {
       harness::Step::Person(Step::Press { find: Find::exactly_named(Role::Button, name) })
   }
   ```

   Go, status `Ready`, press `Open`, the `Keep`, tabs `[opened()]`, press `Point`, tabs `[landed()]`, then for each of `Point outside` and `Close`: press `Open`, tabs of two, press it, tabs `[landed()]`. `without_press`, `harness.run` of both, and the `kept` assertions stay as they are. Every `tabs(..)` keeps `within: SECOND`. The comment on Chromium's order stays; the one on the four button nodes goes. The four `Part::Find`s go: each name is pressed, so a missing one still fails.
5. **Status,** a stated departure (`notes/common.md`), in `skein/docs/design/ui-web.status.md`:
   - Remove the lines beginning `- **Not yet:** 14.1: \`browser_example\` and \`browser_stories\`` and `- **Not yet:** 14.3: \`tests/ui-web/tests/support/tab.rs\``, each whole.
   - In the line beginning `- **Not yet:** 4.3:`, "`browser_stories`'s `a_placeholder_gives_way_to_the_client` and `a_module_not_found_says_so_and_offers_to_try_again`" becomes "`browser_example`'s `a_person_goes_through_the_example_in_turn` and `browser_stories`'s `a_module_not_found_says_so_and_offers_to_try_again`".
6. Your run's checks: `cargo fmt`, `python3 -I -B skein/scripts/old_forms.py`, `heavy cargo clippy -p skein-ui-web-tests --all-targets -- -D warnings`, `heavy cargo nextest run -p skein-ui-web-tests`, which selects no `browser_*` binary, and the two compile checks above. Commit and push.

## Report

Your run's text, and: that no `browser_*` test ran; any row that stayed, and why; each assertion whose words changed; any story of another plan that you kept.
