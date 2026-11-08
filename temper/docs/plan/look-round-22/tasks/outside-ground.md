+++
id = "outside-ground"
part = "ground"
kind = "feature"
needs = []
files = [
  "temper/crates/temper-client-shell-web/temper.css",
  "temper/docs/design/client/browser.status.md",
]
effort = "medium"
minutes = 30
+++

# The column's field names at 14 px, and the night ground over the whole window

## Goal

The request's tasks 3 and 5 (`temper/docs/plan/look-round-22/inputs/request.md`), two rules of `temper/crates/temper-client-shell-web/temper.css`:

- **Task 3:** the column's field names, "Code" on the approval page (the request's `02.html`) and "Project name" with no project yet (`08.html`), render 13 px on a 20.8 px line; the request has 14 px on 22.4 px. The cause: `.temper-outside form > label` (about line 2900, `/* ==== ui: outside ==== */`), specificity (0,1,2), loses to skein's `label[for^="skein-ui-label-"]:has(+ input:not([type="radio"], [type="checkbox"])) { font-size: small; }` (`skein/crates/skein-ui-bundle/skein-ui.css`, about line 86), specificity (0,2,2): `small` is 13 px, and the body's 1.6 line makes 20.8.
- **Task 5:** the night welcome at 420×900 shows a 53 px strip of the day's `#f7f7f5` under the night ground. The cause: the placeholder's `loading.css` gives the root `html { background: #f7f7f5; }`, so the body's night ground (`:root[data-night]:has(body #app) body { … }`, about line 266, `/* ==== ui: night ==== */`) no longer propagates to the canvas, and the welcome's body at that size is 847 px high. With the root's ground gone at night, the body's ground is the canvas's, whatever the page's height.

Acceptance, by `temper/docs/design/client/browser.md`, 2.5, *Outside a project* (a form's field name) and *Day and night* (*The glow*):

1. **The field name's rule, restated where it stands** (`ui: outside`), its declarations unchanged:

   ```css
   .temper-outside form > label[for]:has(+ input) { flex-basis: 100%; color: var(--gf-ink-2); font-size: 14px; font-weight: 500; }
   ```

   Specificity (0,2,3), over skein's (0,2,2), so "Code" and "Project name" take 14 px on the inherited 1.6 line, 22.4 px. No other rule of `ui: outside` changes.
2. **The root gives way at night,** a new rule directly above the body's ground in `ui: night`, with its comment:

   ```css
   /* At night the root drops the placeholder's day paper, so that the body's ground fills the window however short the page. */
   :root[data-night]:has(body #app) { background: none; }
   ```

   Specificity (1,2,1) over `loading.css`'s `html` (0,0,1). A root with no background colour or image hands the body's background to the canvas (CSS Backgrounds 3, 2.11.2), so the night ground, its glow and grain, fill the window. The day keeps the placeholder's paper, which equals the day's `--gf-paper`. No other rule changes, and `loading.css` stays as it is (the boot has no night: `browser.md`, 2.5, *The first paint*).
3. **No new test:** both are rendered facts, which the render check after landing measures; a focused test reading the stylesheet's text would pin a selector's spelling, not the cascade's result. The suites that read `temper.css` pass: `temper-client-shell-web`'s `tests/suite` and the view's `tests::look::every_class_the_stylesheet_selects_is_a_look_name` (no new class).
4. **Status:** delete the two **Not yet** entries naming `outside-ground` in `temper/docs/design/client/browser.status.md` (section 2.5's). No other design line.
5. Runs through `heavy`, counts in the report: `cargo nextest run -p temper-client-shell-web`; `cargo nextest run -p temper-client-view --lib -E 'test(=tests::look::every_class_the_stylesheet_selects_is_a_look_name)'`.

One commit or more, each with the trailer `Look-request: outside-pages`; the last carries the account (below).

## Inputs

- `temper/docs/plan/look-round-22/notes/common.md`.
- `temper/docs/plan/look-round-22/inputs/request.md`, tasks 3 and 5.
- `temper/docs/design/client/browser.md`, 2.5: *Outside a project* (the column's form), *Day and night* (*The glow*), *The first paint, the placeholder's*.
- The request's overlay, from its own commit: `git show 43a18a5042:temper/docs/design/client/journeys/overlay.css` (its `.temper-outside form > label` rule, about line 101).
- `temper.css`: `ui: night` (about lines 264 to 283), `ui: outside` (about lines 2863 to 2912); `loading.css`, line 1; skein's label rule (above).

## Owns and may change

Owns the two rules of acceptance 1 and 2 in `temper.css` and the status entries. Not `loading.css`, `skein-ui.css`, the view, the night tokens' block (about line 143), or any other rule.

Options, each decided:

- **The outside form's name is not a `label[for]` directly before an `input`** (read the view's form in `temper/crates/temper-client-view/src/outside.rs`, `no_project_main`, and `approve.rs`; skein draws a `TextField`'s name as `label[for=…]` + `input`): use the narrowest selector that outweighs (0,2,2) and holds both forms, and say which.
- **A test reads the label rule's or the night rule's text** (`rg -n "temper-outside form > label|data-night\]:has\(body" temper`): change it with the rule, keeping what it checks, and name it.

## Steps

1. Read the request's tasks 3 and 5, the Inputs and both rules; restate the label's rule (acceptance 1); add the root's rule (acceptance 2).
2. The status entries (acceptance 4); the runs of acceptance 5; `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `python3 -I -B tools/citations/citations.py --base origin/main`. The implementer's bite step does not apply: no test is added (acceptance 3); the render check after landing is the rules' check, and the report says so.
3. Commit with the trailer and the account; push with `git push origin HEAD:refs/heads/<your branch>`.

The account's rows, at least:

| Item | Result |
|---|---|
| "Code" (02) and "Project name" (08): 14 px on a 22.4 px line, the field and its act 1.6 px lower | landed (`ui: outside`, the label's rule over skein's small label) |
| The night welcome at 420×900 fills the window: no day strip under the night ground | landed (`ui: night`, the root's ground gives way) |
| Render check | follows landing, temper/ui's (`temper/docs/design/draft/look-requests.md`, 4): focused tests establish no rendered equality |

## Report

The implementer's report: both rules as written, the form's markup read for the label's selector, the runs' counts, and the account as committed.
