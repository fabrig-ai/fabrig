# look-round-22: the outside pages' render-check misses

The request: `inputs/request.md`. The render check after `look-outside-pages` found six misses of the look request `outside-pages` (`43a18a5042` on `origin/look-requests`), landed by `temper/docs/design/draft/look-requests.md`, 3 and 5. `notes/common.md` holds the rules every task follows.

## Parts and tasks

| Part | Task | Request | Needs | What it makes run | Minutes |
|---|---|---|---|---|---|
| `endings` | `tasks/google-endings.md` | 6 | | the Google endings' words in ’; one assertion in the endings' table | 25 |
| `public` | `tasks/public-sign-in.md` | 4 | | "Sign in" on the public pages' top bar wherever the page's origin is known; the table's rows | 30 |
| `ground` | `tasks/outside-ground.md` | 3, 5 | | the column's field-name rule over skein's small label; the root's day paper gives way at night | 30 |
| `welcome` | `tasks/welcome-end.md` | 1 | | the welcome's top bar draws the outside pages' end after its steps; its row packed to its end, the steps taking the free space; the welcome's journeys reprinted | 60 |

Request 2 (the top bar alone while `me` is read) comes in the next batch, after a survey of the tests and tools that draw a page before `me` arrives; it builds on `welcome-end`, since both change the outside frame's tests and reprint the same journeys.

**Critical path, so far:** `welcome-end`, about 60 implementer-minutes; the first three run beside it.

## What was decided

- **The label's cause is the cascade (task 3):** the column's `.temper-outside form > label` (0,1,2) loses to skein's `label[for^="skein-ui-label-"]:has(+ input:not(…))` (0,2,2), `font-size: small`, 13 px; the rule is restated as `.temper-outside form > label[for]:has(+ input)` (0,2,3), its declarations unchanged, where it stands.
- **The night strip's cause is the placeholder's root ground (task 5):** `loading.css` gives `html` the day's `#f7f7f5`, so the body's night ground stops at the body's box instead of propagating to the canvas; the welcome at 420×900 is 847 px high. At night the root drops its background (`:root[data-night]:has(body #app) { background: none; }`, in `ui: night`), so the body's ground, glow and grain fill the window on every page however short; `loading.css` and the boot stay day-only.
- **No test for the two rules:** both are rendered facts the render check after landing measures; a focused test of the stylesheet's text would pin a selector's spelling, not the cascade. No browser row either: no existing browser table reaches the approval's or the no-project column's form, or the welcome at night at 420.
- **"Sign in" on public pages needs the origin alone (task 4):** the link goes to `ORIGIN/` as before, and `/` signed out is the sign-in page with Google and the "Signed out" column with the printed link's words without. `Ways` keeps its shape. The table's rows gain the no-way and unknown-way origins.
- **The Google endings (task 6):** every apostrophe of the seven endings' words, headings and paragraphs, `escape`'s arm kept for the addresses and returns it escapes. The GitHub App's setup pages are not the request's and keep theirs.
- **The welcome's end (task 1)** is the outside pages' end, drawn after the steps by dropping `top`'s early return, so the welcome's frame is the top bar the request named. The design said otherwise (`README.md`, 3.5, *Its frame*: "no person's name or Sign out"), against the request `outside-pages` ("Signed in, the person's name and Sign out at the right"; "the welcome's top bar becomes this one"); this cut changes it. Its row in `temper.css`: `space-between` would leave a wrapped end alone at the left (likely at 1440 too, temper box's line being about 640 px), so the row packs to its end and the steps, not temper box's line, take the free space; an end straight after the brand (before the deployment answers) takes its own.
- **Tasks 3 and 5 share one task,** `outside-ground`: two rules of `temper.css` in two areas, one file, about thirty minutes together.
- **Design written here:** `temper/docs/design/client/README.md`, 3.5, The welcome, *Its frame* and *Its steps*, and 5.1, *the top bar* (the welcome's end); `browser.md`, 2.5, *The welcome's look* (its row); `temper/docs/design/protocol/sign-in.md`, 4.3 (the table's words in ’, and the sentence saying so); `temper/docs/design/client/browser.md`, 2.5, *Outside a project* (the field name at 14 px on 22.4 px, over skein's small label) and *Day and night*, *The glow* (the root's ground gives way at night), and 8.2 (Sign in with or without Google). A **Not yet** entry per gap in `sign-in.status.md`, `browser.status.md` and `README.status.md`, each deleted by its task; `browser.status.md`'s entries are placed so that an unchanged line stands between any two that different tasks delete (the 7.3 entry, then a blank line before `welcome-end`'s), so the tasks' deletions never meet in a merge. No draft status changes: the design doesn't lag `look-requests.md`.

No stored format, protocol or public interface changes.

## Overlaps with drafts beside this one

- **`browser.status.md`** is also `look-ask-ways/account`'s: this cut adds three entries at its top, and two tasks delete them. Unavoidable: the status is where the gaps go.
- **`temper.css`** was changed by `look-ask-ways/opening` and `path-look` (merged on their trunk): `outside-ground` changes one rule in `ui: outside` and adds one in `ui: night`, and `welcome-end` three declarations and one rule among the welcome's top bar rules (about line 1108), nowhere near theirs or each other's.
- **`README.status.md`** is also `look-ask-ways/account`'s and several `tui-box-front` tasks': this cut adds entries at its top, which this plan's own tasks delete in turn.
- **`temper-client-view`:** `look-ask-ways/starters-out` changes `pages.rs`, `pages/reference.rs`, `tests.rs` and `tests/reference.rs`; `public-sign-in` changes `public.rs` and `tests/public.rs` alone, and `welcome-end` `outside.rs` and `tests/welcome.rs` (and `assert_top_end`'s visibility): two of this plan's tasks in the crate at once, on files neither shares.
- **`temper-service`:** `tui-box-front/serve-sign-in` changes `settings.rs`, `clients.rs`, `route.rs`, `family.rs`, `system.rs`, `engine.rs` and their tests; `google-endings` changes `clients/google/pages.rs`, `clients/google/tests.rs` and one line of `tests/sign_in.rs`, none of theirs.
