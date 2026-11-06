+++
id = "focus-target"
part = "focus-target"
kind = "feature"
needs = []
files = [
  "skein/crates/skein-ui-bundle/skein-ui.css",
  "skein/crates/skein-ui-bundle/src/tests.rs",
  "skein/crates/skein-ui-web/src/page.rs",
  "skein/crates/skein-ui-web/src/page/web.rs",
  "skein/crates/skein-ui-web/src/tests/page.rs",
  "skein/examples/ui/shell-web/src/folds_fixture.rs",
  "skein/tests/browser/tests/suite/folds_away.rs",
  "skein/docs/design/ui-web.status.md",
  "skein/docs/design/STATUS.md",
  "temper/docs/design/client/journeys/skein-ui.css",
]
effort = "high"
minutes = 50
+++

# A closing form leaves the focus on its disclosure's summary; a focus target draws no ring

## Goal

Two faults in skein's web face, the second half of `inputs/request.md`. A removal that takes the focus passes over an enclosing disclosure and lands on the main, so closing a form in a menu loses the person's place; and Chromium draws `:focus-visible` on a main or heading given `tabindex="-1"` only to take the focus, seen as a dark line beside the rail.

Acceptance:

- A removal inside a disclosure that takes the focus leaves it on that disclosure's `<summary>`, in real Chromium, with the main not focused.
- `Op::Focus` on a disclosure, from a removal or a focus request, focuses the `<summary>` and sets no `tabindex` on `<details>`.
- A `tabindex="-1"` focus target (a main, a heading, a region) computes `outline-style: none` under `:focus-visible` in real Chromium. A button, a link, a field, a `<summary>` and a menu's item reached by arrow keys keep their ring.
- Each fix had its test red first, and the report says what it showed.
- `ui-web.status.md` is back to `No gap.` and the line in `skein/docs/design/STATUS.md` names `ui-web.md` among those with no gap.

## Inputs

- `skein/docs/design/ui-web.md`, 7 (*Never replaced while it can be updated*, *A focus request*) and 12 (its list of what the sheet does).
- `temper/docs/plan/ui-rename-focus/notes/common.md`, its departures and *What crosses the two tasks*.
- `skein/docs/foundation/testing-strategy.md`, 8.1, for the browser tier's budget.

## Owns and may change

Owns, as `files` lists them.

May also change: nothing in temper but the generated `temper/docs/design/client/journeys/skein-ui.css`. No page of the journeys should change: this task changes neither the mapping nor a class. A page that does is a red to explain, not to reprint past. `temper.css` is `rename-row`'s.

Options, each pre-approved:

- **A real control you find carrying `tabindex="-1"`** beyond `[role=menuitem]` and `[role=treeitem]` (`rg tabindex` over `skein/`, `jig/` and `temper/` sources): add it to the selector's `:not(...)` and name it in your report.
- **The computed-style check:** the rig's steps read selectors, not computed styles, so the test that stays is the sheet's rule in `skein-ui-bundle`'s tests, in the idiom of `the_stylesheet_names_no_class`, and the computed values are proved once with a throwaway CDP script. Add no step to the harness for this.
- **The browser story's fixture:** extend `folds_fixture.rs` as you judge; if its form and its Escape press already give the removal, reuse them.

## Steps

1. **Reproduce.** The removal's fault by the page test below, red. The ring with a throwaway page holding the real `skein-ui.css`: a `main` and an `h1` focused by script after a Tab key, read `getComputedStyle(...).outlineStyle`.
2. **The removal,** `skein/crates/skein-ui-web/src/page.rs`: `focusable` gains `Role::Disclosure`. A page test beside `a_removal_taking_the_focus_moves_it_to_an_ancestor` in `skein/crates/skein-ui-web/src/tests/page.rs`:

   ```rust
   #[test]
   fn a_removal_inside_a_disclosure_moves_the_focus_to_the_disclosure() {
       // main, a named disclosure, a form holding a button; page.focused(the button);
       // the form is removed: ops are [Remove { node: FORM }, Focus { node: DISCLOSURE }].
   }
   ```

   `a_removal_taking_the_focus_falls_back_to_the_main` stands, and an enclosing dialog nearer than a disclosure still takes it.
3. **The element,** `skein/crates/skein-ui-web/src/page/web.rs`, `Op::Focus`: a `<details>` group focuses `:scope > summary`, the selector `fold_away` already uses; `SUMMARY` joins the tags that take no `tabindex`.
4. **The story in real Chromium,** `skein/tests/browser/tests/suite/folds_away.rs` and its fixture, in that file's steps: open a disclosure that does not fold away, focus a field in a form inside it, `Step::Key(Key::Escape)` removes the form, `Step::Focused(Find::exactly_named(Role::Disclosure, NAME))`, and a `check` of `main:not(:focus)`. Run it with `heavy tools/tiers/run skein browser -- -E 'test(=NAME)'`, which prepares the bundle.
5. **The sheet,** `skein/crates/skein-ui-bundle/skein-ui.css`, after the `:focus-visible` rule, and no class (`the_stylesheet_names_no_class`):

   ```css
   /* A programmatic focus target: tabindex=-1 only to take the focus (ui-web, 7). */
   [tabindex="-1"]:not([role=menuitem], [role=treeitem]):focus-visible {
     outline: none;
   }
   ```

   Its test in `skein/crates/skein-ui-bundle/src/tests.rs` reads the sheet for that rule after the general one. Prove the acceptance's computed values with the throwaway script: the targets `none`; `button`, `a`, `input`, `summary` and a `[role=menuitem]` `solid`.
6. **The print's copy:** `heavy cargo run -p temper-client-journeys` rewrites `temper/docs/design/client/journeys/skein-ui.css`. Commit that file alone.
7. **Check,** the implementer's text, and because a removal's focus changes meaning for every shell: `heavy cargo nextest run -p skein-ui-web -p skein-ui-bundle -p skein-ui-web-tests -p skein-browser-world -p jig-client-shell-web -p temper-client-shell-web -p temper-client-journeys -p temper-client-gallery`; `heavy tools/tiers/run skein browser` once, whole, since the page part changed, reporting its time against its budget. `python3 -I -B tools/citations/citations.py --base origin/main`. Push.

## Report

The implementer's report, with: the red each test showed; the computed values from the script; any selector you added, and why; the browser tier's time.
