+++
id = "rig-cases"
part = "kit-facts"
kind = "feature"
needs = ["in-turn"]
files = [
  "skein/tests/browser/src/tests.rs",
  "skein/tests/browser/tests/support/real.rs",
  "skein/tests/browser/tests/support/real/tabs.rs",
  "skein/tests/browser/tests/support/real/downloads.rs",
  "skein/tests/browser/tests/browser_real.rs",
  "skein/tests/browser/tests/browser_facts.rs",
  "skein/tests/browser/tests/suite/folds_away.rs",
  "skein/tests/browser/tests/suite/log_end.rs",
  "skein/tests/browser/tests/suite/log_reading.rs",
  "skein/docs/design/browser.status.md",
  "skein/docs/design/ui-web.status.md",
]
effort = "medium"
minutes = 90
+++

# One Chromium holds the kit's facts

## Goal

`browser_real` and `browser_facts` start 40 Chromiums for 38 tests. Almost all of what they check needs a page and a person, and not a browser of its own. `Rig` learns to play cases in one Chromium, the suite's stories share a run, and one test's press looks first. **38 tests become 10. Nothing here runs in Chromium:** it's written and compiled, and after-landing runs it.

Acceptance:

- `Case` and `Rig::cases` exist as `skein/docs/plan/skein-browser-starts/notes/contracts.md` has them.
- `a_person_closed_opens_again_with_its_tokens` passes in focused, and bites.
- `browser_real` holds the 9 tests of `plan.md`'s rows 1 to 33, by the names there, and `browser_facts` the 1 of rows 34 to 38. Every assertion of those rows is where its row says: the table is the checklist.
- `heavy cargo check -p skein-browser-world --tests` and `heavy cargo nextest run -p skein-browser-world --no-run` pass. **No `browser_*` test runs,** not one, nor by name.
- The status edits below are made.

## Inputs

- `skein/docs/design/browser.md`, 6 (*A look brings its page to the front*) and 9 (*One Chromium holds many facts*).
- `skein/docs/design/ui-web.md`, 14.1, and 14.2, *Stories in turn*.
- `skein/docs/plan/skein-browser-starts/notes/common.md`, and `notes/contracts.md`.
- `skein/docs/plan/skein-browser-starts/plan.md`: the table's first two parts, *What this plan decided*, and *What the two tab tests wait for*.

## Owns and may change

Owns the files in the header, and in the status files only the lines below. `real/beside.rs`, `real/dialogs.rs`, `real/scope.rs` and `real/viewport.rs` take call-site edits only. Their arms call `close_browser`, which is where a case ends, so they should need none. Nothing outside `skein-browser-world` reads this rig: `real.rs` is `#[path]`-included by `browser_real` and `browser_facts` alone.

Options, each decided:

- **A case gets a new person and page with the same tokens,** `PERSON` and `PAGE`, so that no arm changes. The kit reuses them once the last are closed: the focused test below holds that.
- **The rig's one minute stays one `Deadline` over the whole `run`.** The loop's `50_000` turns are a guard against a run that doesn't settle, not a time: they count per case, as each test had them.
- **`io_limits`' `sockets` rises from 8 to 24** (child, three pipes, listener, `ORIGIN_CONNECTIONS` 16 and spare): a closed person's connections close some time after it, and a new page's open meanwhile. It's the rig's capacity, not a budget or a deadline.
- **A denied download ends on its trouble.** With a check at a case's end, `a_person_without_a_directory_downloads_nothing`'s `download_trouble` would race the case's close, where today it's read after the whole run. So a `Download { allow: false }` case ends once its heading is met and its `Trouble::Download` has come, in either order, as `allow: true` does with `Downloaded`. If Chromium stopped reporting it, the deadline fails that case, which still fails the fact.
- **`Scenario::Attribute`'s `expected`** becomes a `Copy` enum of yours, with `Value`, `NoElement` and `NoAttribute`, since row 82's real fact needs the third.
- **If a merge won't compile for a reason this spec missed,** that row's test stays as it is and the report says why. Don't reach into `skein-browser`.

## Steps

1. **The kit's half, first,** in `skein/tests/browser/src/tests.rs`:

   ```rust
   #[test]
   fn a_person_closed_opens_again_with_its_tokens() {
       let mut world = World::new();
       world.start();
       world.open();
       world.ask(Request::Close { entity: Token::new(2) });
       // both `Closed`s arrive, the page's then the person's; drain them
       world.browser.reclaim();
       world.open(); // asserts `Opened` for the same page token
       world.assert_commands_settled();
   }
   ```

   Run it by name in `--lib`. **Bite:** make `Browser::person` find a closed person, and see `Request::Person` refused. Restore.
2. **`Rig::cases`,** in `real.rs`. `Rig::new` is one case with no name. What changes:
   - `close_browser` is where a case ends. It runs the case's `check`. With no case left, it closes the browser, as now. With one left, it sets a new `Stage::Changing` and asks `Request::Close { entity: PERSON }`.
   - `browser_event` starts: in `Stage::Changing`, `Event::Closed { owner: PERSON }` calls `self.browser.reclaim()` and begins the next case; every other event is what the closed case's page still said, and is dropped.
   - Beginning a case resets all of a case's state: `scenario`, `dialog`, `scope`, `choice`, `choice_failure`, `downloaded`, `download_trouble`, `form_node`, `tab_due`, `context_ready`, `exception_seen`, `csp_log_seen`, `await_met`, `snapshot`. Sets `download_directory` to `profile.join(format!("downloads-{n}"))`, so that no case reads another's. Sets `pages.pages = pages::pages()` and runs the case's `prepare`. Sets `Stage::Context`, and asks for the person: the `Event::Ready` arm's asking goes into a function they share.
   - `eprintln!("case: {name}")` as each begins, and the timed-out and did-not-settle panics name it: nextest shows a failed test's output, so the last such line names what failed.
3. **`tabs.rs`, the press that looks first** (*What the two tab tests wait for*). The flow becomes: `Opened`; find `Blocked`; tabs, none; find `Open`; press; tabs, one, opener `true`; **find `Cut opener`, fresh, in `Stage::Keying`; press it;** tabs, one, opener `false`; close. `tabs_cut` goes, with the `Stage::Capturing` arm that filled it. Every assertion stays. `eprintln!` stays.
4. **`browser_real.rs`:**

   | Test | Its cases, in order |
   |---|---|
   | `chromium_opens_finds_and_presses_a_button` | as it is: `Rig::new`, with its version and exit |
   | `chromium_acts_on_a_page` | the two `MouseDown` rows, each its page; `Form`; `Covered`; `Scroll`; `Figure`; `Letter`; `Viewport`, last, with its page |
   | `chromium_reports_what_a_page_holds` | `Csp`; `Throw`; `Words`; on `ATTRIBUTE_PAGE`, each of rows 14, 15 and 16, and `input`'s `absent` as `NoAttribute` |
   | `chromium_keeps_a_page_through_reloads_and_dialogs` | the two `RELOAD_SOURCES`, named by source, each its page; `ScopeReload` with its page; `Leave`; `Confirm` |
   | `chromium_saves_a_download_only_for_a_person_with_a_directory` | `Download { allow: true }` and `{ allow: false }`, each with its row's assertions as its `check` |
   | `chromium_holds_tabs_beside_a_page` | `TabBeside`, `prepare`d by `beside_page`; then `Tabs`, its page and `tab_script`; after `run`, row 20's `root_closed && exited && child_closed` |

   The `catch_unwind` in row 10 goes, since the case's name does its work. So do `attribute_rig` and whatever else is left unused.
5. **`browser_facts.rs`:** `chromium_holds_the_faces_facts`, five cases: `Facts`, `Choose { name: b"Second" }` and `Dates` on `fixed_page()`, each with its row's assertions and `choice_failure == None` as its `check`; `Choose { name: b"task" }` and `Select` on their own pages. `snapshot()` goes. A `check` reads `rig.snapshot` through a borrow.
6. **The page-part stories,** `suite/`:
   - `folds_away.rs`'s `page_parts_fold_away_and_a_log_keeps_following`: one `Harness::new(setup())`, four people: rows 23, 24, 25 and, from `log_end.rs`, 26. Each file's story becomes a function returning its `Script`; `log_end.rs`'s is `pub(super)`.
   - `log_reading.rs`'s `a_log_reads_on_and_holds_its_words`: `Harness::run_stories`, in rows 27, 28, 30, 31, 32, 33's order, each story as its row has it, with its `See Inspect` inserted as `run` does. Kept names repeat between them (`reads`, `top`, `viewport`), so each story's carry its name, and `value`, `number` and `held` take it. The assertions follow `run_stories`.
   - `reading_without_browser_anchoring_follows_requests_and_page_keys` stays its own run, as it is but for what those helpers now take.
7. **Status,** a stated departure (`notes/common.md`):
   - `skein/docs/design/browser.status.md`: remove the lines beginning `- **Not yet:** 6: \`tests/browser/tests/support/real/tabs.rs\`` and `- **Not yet:** 9: \`browser_real\` and \`browser_facts\``, each whole. The viewport's line stays.
   - `skein/docs/design/ui-web.status.md`: remove the line beginning `- **Not yet:** 14.1: \`browser_real\`'s page-part stories`, whole. In the line beginning `- **Not yet:** 7:`, "`log_reading`'s seven stories" becomes "`log_reading`'s stories".
8. Your run's checks: `cargo fmt`, `python3 -I -B skein/scripts/old_forms.py`, `heavy cargo clippy -p skein-browser-world --all-targets -- -D warnings`, `heavy cargo nextest run -p skein-browser-world`, which selects no `browser_*` binary, and the two compile checks above. Commit and push.

## Report

Your run's text, and: that no `browser_*` test ran; what the bite showed; any row that stayed, and why; each assertion whose words changed.
