+++
id = "gallery-print"
part = "gallery"
kind = "feature"
needs = []
files = [
  "temper/tools/gallery/src/tests.rs",
  "temper/tools/gallery/src/lib.rs",
  "temper/docs/plan/limits-heavy/figures/gallery-print.md",
]
effort = "high"
minutes = 90
+++

# The gallery's tests: does a check render every page because it needs them?

## Goal

`temper-client-gallery`'s `tests` is 30 tests and 12.1 CPU s: `print_writes_every_page_and_the_bundle` 4.2, and four others at 1.8 each. They belong where they are (`plan.md`, *Belonging*). Find what each pays for, whether it needs every page it renders, and fix the cause.

Acceptance:

- Every check still runs against every state it reads today, including states other drafts add: nothing is limited to a list of today's names that wasn't one already. A test that goes is a row of a named test, by the rule in the common note, and is listed under *Tests that went*.
- `print` still writes every page, the bundle and the index, and a second print still replaces the whole directory; a test still holds each.
- `temper/docs/plan/limits-heavy/figures/gallery-print.md` holds the tests you profiled or changed, before and after in instructions, with the cause of each.
- The next cause's gain is under about 0.2 CPU s, or what's left is under *Found and not fixed*.

## Inputs

- `temper/docs/plan/limits-heavy/inputs/request.md`.
- `temper/docs/plan/limits-heavy/notes/common.md`: how to measure, the figures' form and the rules, *Rows* above all.
- `temper/docs/plan/limits-heavy/plan.md`, *Belonging*.
- `temper/docs/design/client/browser.md`, 7: what the gallery promises.
- `skein/docs/foundation/testing-strategy.md`, 8, *A test is a process*, *What a test costs the suite* and *A slow test is understood, never hidden*.

## Owns and may change

Owns `temper/tools/gallery/src/tests.rs`, `src/lib.rs` and its figures.

Readers: `temper/tools/gallery/src/main.rs` reads `print` and `samples::samples()`, and `tests.rs` reads those and `view`. Keep the public signatures of `print`, `samples::samples()`, `view` and `page`; add beside them if a cost needs it. No document cites these tests by name: `rg` over the repository, hidden files included, finds them in `tests.rs` alone. `temper/tools/gallery/names.txt` stays.

May change, at the size the cause needs:

- `temper/tools/gallery/src/samples.rs`, `worlds.rs` and `worlds/**`, only where a profile puts a cause, with small hunks. **`look-round-13` has several tasks open in `temper/tools/gallery/**`,** adding samples, worlds and tests' rows. Don't reorganize these files or re-indent what they may touch. If a merge meets them, keep their change.
- `temper-client-view` (`Words::english`, `Limits::of`, `View::with_sign_in`, `build`) and `skein-ui-bundle` for a cause a profile puts there. `limits-fixtures` is looking at `temper-client-view`'s floors, so keep such a fix small.

Options, each decided:

- **Several tests build every state's view and their own check costs little beside that:** they become rows of one test that makes one pass over the views, each failure naming its state and rule, with every assertion kept. Name it for what it holds. `no_state_reads_of_zero`, `no_named_control_draws_a_text_child_in_any_state`, `each_chat_reads_its_allotment` and `navigation_links_have_addresses_and_install_action_links_keep_their_binding` are the likely ones. Find the request's four at 1.8 from your own runs, as it doesn't name them. Rows only where the profile shows that; a check that costs what it costs keeps its test.
- **`built` makes `Words::english` and `Limits::of` for every state:** if that's a real share, make them once per pass and hand them in, keeping `view`'s and `page`'s signatures.
- **A state's world is the cost (`samples::domain`):** states sharing one world may share its building, where the code makes that small. If it would mean reorganizing `worlds.rs`, it goes under *Found and not fixed*, for after `look-round-13`.
- **`print_writes_every_page_and_the_bundle` prints twice:** its second print proves that a reprint replaces the whole directory. If the print is the cost, split the replacing out of `print` into a function a test can hold on a small directory, keep `print` calling it, and keep the first print's checks whole. A test still calls `print` on an existing gallery if that costs little; if it doesn't, the split's test holds it and the figures say so.
- **`skein_ui_bundle::bundle` is the cost:** fix it in skein if small and clearly theirs. Otherwise it goes under *Found and not fixed*.

## Steps

1. Run the package's tests once with their CPU seen (`heavy cargo nextest run -p temper-client-gallery --lib`, reading each test's time) to name the four. Profile `print_writes_every_page_and_the_bundle`, those four, and a test that reads one sample by name, as the floor. The filters read `tests::NAME`.
2. Starting points, to check and not findings: `samples::samples()` builds the whole array even for a test that wants one sample by name, and 27 places call it. `built` builds a domain, the words, the bounds and a view for each state. What share of a state is the domain, what share the view's build and what share the HTML?
3. Fix the causes, largest first, re-measuring after each.
4. Write the figures.
5. The implementer's text's end of a run: `cargo fmt`, old forms, `heavy cargo clippy -p temper-client-gallery --all-targets -- -D warnings` with any other package you changed, and the focused tests of those packages and their direct dependents. Commit and push.

## Report

What each of the five paid for, before and after, which tests became rows and of what, the causes found and not fixed, and anything met from `look-round-13`. Your implementation and check minutes apart.
