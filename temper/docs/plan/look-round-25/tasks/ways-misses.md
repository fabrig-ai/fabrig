+++
id = "ways-misses"
part = "look"
kind = "feature"
needs = []
files = [
  "temper/crates/temper-client-view/src/paths/layout.rs",
  "temper/crates/temper-client-view/src/pages/ways.rs",
  "temper/crates/temper-client-view/src/pages/ask_page.rs",
  "temper/crates/temper-client-view/src/tests/paths.rs",
  "temper/crates/temper-client-shell-web/temper.css",
  "temper/docs/design/client/README.status.md",
  "temper/docs/design/client/browser.status.md",
]
effort = "high"
minutes = 80
+++

# The ways page's card positions and four tabs on a phone, and its drawing shared

## Goal

The request's item 3, first and fourth bullets (`temper/docs/plan/look-round-25/inputs/request.md`): two small card positions, at 1000 and on a phone, and, with a fourth "Yours" tab, the tabs fitting under 760 px ("Agents review" doesn't fit a quarter of a phone today). And the ground How work runs needs to draw its path as Ask temper's ways do: the drawing and its legend become the shared ways module's, their rules holding on both pages.

Acceptance, by `temper/docs/design/client/README.md`, 5.16, *Its layout* and *Pressed*, and `temper/docs/design/client/browser.md`, 2.5, *Ask temper's ways*, as this cut wrote them, and `temper/docs/plan/look-round-25/notes/contracts.md`, *The client's drawing*:

1. **On a phone** (`paths/layout.rs`): down the page in a box under 360, the scene is drawn 360 wide, says its width (`set_width(360)`) and is scaled to the box, as across under 720 is; the column's offset is 0 there; an open stop's room is `scaled(14, 360, W)` scene pixels. At 360 and over nothing changes.
2. **At 1000 and on a phone, by arithmetic, no browser:** from the look request's own page (`git show 98322b3565:temper/docs/design/client/journeys/ask-temper-ways/02.html` and `04.html`, and `overlay.css` at that commit: the figure's `viewBox`es, `.temper-ways-zoom`'s `margin: 14px 0 0`, the column's widths) and `temper.css`'s column widths for Ask temper's page, work out where the open card's top edge stands under the path's top, in the request and in the client, at a 1000 px window (a 658 box, today 141.65 against the request's 141.94) and at 390 (a 358 box). Repair every difference past half a pixel in the layout or in a rule under the ways' rules; one inside half a pixel that whole scene pixels cause stays, said in the account with both figures.
3. **Four tabs on a phone** (`temper.css`, the ways' rules under `ui: settings`, beside the existing `@media (max-width: 760px)` block, about line 2858): with a fourth tab, at 760 px and under, `grid-template-columns: repeat(2, minmax(0, 1fr))`, as the whole story's `whole-story.css` has `.ways-tabs:has(> :nth-child(4))`, for `#app > .temper-session` and `#app > .temper-settings` alike.
4. **The drawing shared** (`pages/ways.rs`, `pages/ask_page.rs`): `ask_page::path` moves, unchanged in what it draws, to `pages::ways::drawing(b, path, open, w, bindings, project)`; the legend paragraph of `ask_page::foot` moves to `pages::ways::legend(b, w)`; Ask temper's page calls both, its tree unchanged. The path's drawing rules (the `ask-temper-ways` block under `ui: chats`, about lines 1490 to 1510) and the legend's (about lines 2839 to 2845) select under `#app > .temper-settings` as well as `#app > .temper-session`, as the tabs' rules do, so that How work runs' frame draws them alike; Ask temper's page's computed styles unchanged.
5. **Rows, no new test function** (`src/tests/paths.rs`): `the_three_look_paths_keep_judgement_above_code_and_each_stop_in_words` gains the width 358 (down the page, scene width `Some(360)`, its height, the open room scaled); `path_bounds_refuse_bad_data_and_draw_any_keeps_flows`' table a row at 358 with `Some(360)`. The Ask page's own tests (`src/tests/ask.rs`) pass unchanged: the move changes no tree.
6. **Status:** delete the **Not yet** entries naming `ways-misses` in `temper/docs/design/client/README.status.md` (5.16) and `temper/docs/design/client/browser.status.md` (2.5). No other design line.
7. Runs through `heavy`, counts in the report: `cargo nextest run -p temper-client-view`, `-p temper-client-gallery`, `-p temper-client-shell-web`, `-p temper-client-journeys`; clippy on `temper-client-view`. No journey shows Ask temper's opening, so none is reprinted unless the journeys' test says one is stale.

## Inputs

- `temper/docs/plan/look-round-25/notes/common.md`, `temper/docs/plan/look-round-25/notes/contracts.md`.
- The request's item 3, its first and fourth bullets.
- `temper/docs/design/client/README.md`, 5.16, *Its layout* (down the page) and *Pressed* (the open stop's room).
- `temper/docs/design/client/browser.md`, 2.5, *Ask temper's ways*.
- The look request `ask-temper-ways` (`notes/common.md`, *The target*); the whole story's `temper/docs/design/client/web/whole-story/whole-story.css`, its `@media (max-width: 760px)` block.

## Owns and may change

Owns the files above. Not `paths.rs` (the drawing's data, `path-marks`'), `pages/how.rs` or `pages/settings.rs` (`how-drawing`'s), nor any engine or wire file (`path-stops`').

Options, each decided:

- **The phone's difference is not the column's scale alone** (a margin, the column's padding): repair it where it lies, by the layout or a rule, and say which.
- **At 1000 the difference is past half a pixel and not the room's rounding:** repair it where it lies; if only a viewport rule could, that rule goes under the ways' rules with `.temper-ways-path[data-scene-width]` (the scaled drawing), never a rule of the window's width alone, and the report says why.
- **The legend's words or tokens:** unchanged here; "when needed" is `path-marks`'.

## Steps

1. Read the Inputs, `paths/layout.rs`, `ask_page.rs`'s `path` and `foot`, `pages/ways.rs`, the rules named.
2. The arithmetic (2), written into the report first; the phone's layout (1); any repair at 1000; the rows (5); bite: drop the `set_width(360)`, see the 358 row fail, restore.
3. The tabs' rule (3), the move and the rules' scope (4), the status (6), the runs (7); `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; commit; `python3 -I -B tools/citations/citations.py --base origin/main`.
4. Commit with the account; push with `git push origin HEAD:refs/heads/<your branch>`.

The account's rows, at least:

| Item | Result |
|---|---|
| The open card at 1000 | landed, or differs by the figure and why |
| The open card on a phone | landed (`layout.rs`: the 360 column scaled under 360), with the figures |
| Four tabs under 760 px | landed (`temper.css`: two by two) |
| The drawing and legend shared with How work runs | landed (`pages/ways.rs`; the rules under both frames) |
| Render check | follows landing, temper/ui's (`temper/docs/design/draft/look-requests.md`, 4) |

## Report

The implementer's report: the arithmetic and its figures at both widths, the repairs, the rows and the bite with its failure, the runs' counts, and the account as committed.

## Rebase (2026-11-08, step merge, foreman look-round-25/ways-misses/foreman-1)

resolve README.status.md by retaining roles-view and removing only ways-misses
Rebase onto the landing’s current integration tip.
The conflict is in `temper/docs/design/client/README.status.md`, under **Not yet**.
Commit `dab3e93ee5` (merged by `438a819fec`) adds the `roles-view` entry immediately after the `ways-misses` entry that this branch deletes.
Resolve by deleting the five-line 5.16 `ways-misses` entry while retaining both the 4.6 `path-stops` entry and the two-line 4.6 and 5.16 `roles-view` entry.
Preserve all other status text and the branch’s implementation changes.
