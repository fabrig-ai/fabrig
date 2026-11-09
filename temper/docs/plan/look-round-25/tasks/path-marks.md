+++
id = "path-marks"
part = "marks"
kind = "feature"
needs = ["ways-misses", "fade"]
files = [
  "temper/crates/temper-client-view/src/paths.rs",
  "temper/crates/temper-client-view/src/paths/**",
  "temper/crates/temper-client-view/src/pages/ways.rs",
  "temper/crates/temper-client-view/src/views.rs",
  "temper/crates/temper-client-view/src/look.rs",
  "temper/crates/temper-client-view/src/words.rs",
  "temper/crates/temper-client-view/src/tests/paths.rs",
  "temper/crates/temper-client-view/src/tests/ask.rs",
  "temper/crates/temper-client-shell-web/temper.css",
  "temper/tools/gallery/src/tests.rs",
  "temper/docs/design/client/README.status.md",
  "temper/docs/design/client/browser.status.md",
]
effort = "high"
minutes = 170
+++

# The path's marks: outlined when needed, the words over a stop, the agent in ink, the line's fades

## Goal

The request's item 1, "who acts by mark: a person's ring orange, an agent's star ink, code grey, and no blue" and "a repair and a conflict told apart", and item 3, "the line painted as the request's continuous gradient" (`temper/docs/plan/look-round-25/inputs/request.md`). temper's path drawing (Ask temper's ways, How work runs once `how-drawing` lands, a reply's view) draws a stop made only when needed outlined and a stop's small words over its own, the agent's star and the rise to it in ink, code's square grey, and each rise as a fade from its mark's colour to the line's, through skein's fade (`fade`). The legend gains an agent when needed. Its callers pass `needed: false` and no words until `how-drawing` passes the wire's.

Acceptance, by `temper/docs/design/client/README.md`, 5.16, *Drawn* and *Its layout*, and `temper/docs/design/client/browser.md`, 2.5, *Ask temper's ways* (who acts' marks, the line's fades), as this cut wrote them; `skein/docs/design/ui.md`, 11.2, and `skein/docs/design/ui-web.md`, section 8, on a fade, as `fade` wrote them; and `temper/docs/plan/look-round-25/notes/contracts.md`, *The client's drawing* and *skein's fade*:

1. **The data** (`paths.rs`): `paths::Stop<'a> { mark, words, needed: bool, over: &'a [u8] }`, `pub const OVER: u32 = 16`; `encode` and `decode` carry both (a needed flag beside the mark's tag, `over`'s length and bytes after the words, refused past `OVER` or on a bar), `Problem::Over` for an `over` too long; `DATA`, `MARKS` and `SCENE_BYTES` raised to the worst path's new figures, each figure's sum in its comment as today, `SCENE_BYTES` counting each fade as `skein_ui::scene::fade_bytes(3)` (20) and the second rise path. The fallback's line: "WORDS, ACTOR", then ", when needed" and " (OVER)" where they hold ("repair, an agent, when needed (tests, review)").
2. **The marks** (`paths/layout/marks.rs`): a needed stop's star or square also carries a new token (`WAYS_MAYBE`), which the stylesheet outlines; a stop's `over` a text mark with `WAYS_WHY`: across at `y` 9 centred on the stop, whichever side of the line; down the page under its words, 12 lower, anchored as its words are.
3. **The line** (`paths/layout/line.rs`): three paths, the base (`WAYS_BASE`), the rises to a person (`WAYS_RISE_PERSON` alone) and those to an agent (`WAYS_RISE_AGENT` alone), each rise path followed by `scene.fade(...)`, its own token and its three stops' within `styles_per_node` (5; skein's `ui.md`, 15, *A fade's share*); `WAYS_RISE`'s slot is retired, `temper-reserved-620`: across from `(0, 40)` to `(0, 106)`, down the page from `(offset + 206, 0)` to `(offset + 150, 0)`, stops at 0, 500 and 1000 with their tokens (`WAYS_FADE_PERSON`, `WAYS_FADE_PERSON_FAINT`, `WAYS_FADE_AGENT`, `WAYS_FADE_AGENT_FAINT`, `WAYS_FADE_BASE`, or names of your choosing). A path with no rise of one kind draws no path for it. The split at the dotted line that two tones needed goes: each curve whole in its own path.
4. **The look** (`look.rs`, appended at the end of `NAMES`, `notes/common.md`): the new tokens' names and constants. `temper.css`, the path's rules where they stand (the `ask-temper-ways` block under `ui: chats`, both frames' selectors as `ways-misses` left them): the colours of `browser.md`, 2.5 (an agent's star `--gf-ink`, outlined `--gf-paper` stroked `--gf-ink` at 1.3 with round joins; code's square `--gf-muted`, outlined `--gf-paper` stroked `--gf-muted` at 1.3; a person's ring as today; a stop's small words 10 px `--gf-faint`; each fade stop's `stop-color` and `stop-opacity`: a person's `--gf-signal` at .8 and .28, an agent's `--gf-ink` at .3 and .1, then `--gf-line-2` at 1); the two rises' rules keep `fill: none` and `stroke-width: 1.5` and set no `stroke` and no `stroke-opacity` (today's `.temper-ways-rise` rule, its signal stroke and its `.28`, goes: an opacity there would multiply the stops' own); the path's `stroke: none` rule and any rule setting `stroke` on a faded mark take `:not([data-fade])`, as `ui-web.md`, 8, *A fade's paint and the stylesheets*, says. The legend's agent "✦" in `--gf-ink`; its agent when needed the outlined star.
5. **The legend** (`pages/ways.rs`, `legend`): a fifth item after the agent, "when needed" (`words.rs` key 5100) on `WAYS_KIND_MAYBE`; `WaysLegend`'s words (key 4310) "An agent, an agent when needed, a person, code; no merge tool above the dotted line".
6. **Its callers** (`pages/ways.rs`, `drawing`; `views.rs`, `path`): `needed: false, over: b""`; `how-drawing` passes the wire's.
7. **Rows, no new test function:**
   - `tests/paths.rs`, `the_three_look_paths_keep_judgement_above_code_and_each_stop_in_words`: its fixtures' repair needed with "tests, review", a conflict, a resolution and a catch-up needed, a goal's merge with "into goal"; at 720, 658, 388 and 358 the outlined marks carry `WAYS_MAYBE`, each `over` a `WAYS_WHY` text where 5.16 places it, the two rise paths with their tokens and each its fade's points and stops, no rise path for a kind the path lacks; the fallback's lines with ", when needed" and the brackets. `path_bounds_refuse_bad_data_and_draw_any_keeps_flows`: an `over` past `OVER`, an `over` on a bar, a needed bar, each refused; the worst path within `DATA`, `MARKS` and `SCENE_BYTES`.
   - `tests/ask.rs`, `terminal_prints_tabs_stops_card_legend_and_act_in_order`: the legend's five items.
   - The gallery's prints (scratch) take the new marks; its tests' assertions, where they name the rise's token or the legend's words, follow them.
8. **Status:** delete the **Not yet** entries naming `path-marks` in `temper/docs/design/client/README.status.md` (5.16) and `temper/docs/design/client/browser.status.md` (2.5). No other design line.
9. Runs through `heavy`, counts in the report: `cargo nextest run -p temper-client-view`, `-p temper-client-gallery`, `-p temper-client-shell-web`, `-p temper-client-shell-terminal`, `-p temper-client-journeys`; clippy on `temper-client-view` and `temper-client-gallery`. How work runs still draws its lanes until `how-drawing`, so no journey is reprinted unless the journeys' test says one is stale.

## Inputs

- `temper/docs/plan/look-round-25/notes/common.md`, `temper/docs/plan/look-round-25/notes/contracts.md`.
- The request's items 1 and 3 (the line's gradient).
- `temper/docs/design/client/README.md`, 5.16, *Drawn*, *Its layout* and *Pressed*.
- `temper/docs/design/client/browser.md`, 2.5, *Ask temper's ways*.
- `skein/docs/design/ui.md`, 11.2 (a fade); `skein/docs/design/ui-web.md`, section 8 (its paint, `data-fade`, the rules that must skip it).
- The targets: `temper/docs/design/client/web/whole-story/how.html`'s paths (`.ways-why`, `.maybe`, `.ways-rise.p`, `.ways-rise.a`) and `whole-story.css`, lines 143 to 169; the look request's fade (`git show 98322b3565:temper/docs/design/client/journeys/ask-temper-ways/02.html`, its `linearGradient`s).

## Owns and may change

Owns the files above. Not `pages/how.rs` (`how-drawing`'s), `pages/ask_page.rs` (but a call `ways-misses` left, unchanged), nor any engine or wire file.

Options, each decided:

- **Night:** the same tokens' night values; no rule of its own unless a token has none.
- **A halo's or the caret's colours:** as today.
- **The fade's stop tokens' names:** yours, appended as item 4 says; the report lists them.
- **A test elsewhere asserting the old legend words or the rise's single path:** its expectation follows, named in the report.

## Steps

1. Read the Inputs, `paths.rs`, `paths/layout/*.rs`, `pages/ways.rs`' `drawing` and `legend`, `views.rs`' `path`, the rules named and the tests named.
2. The data (1), the marks (2), the line (3); the rows of `tests/paths.rs`; bite: drop the needed flag in `encode`, see the outlined row fail, restore; give both rises one path, see the fade row fail, restore.
3. The look (4), the legend (5), the callers (6), the other rows (7), the status (8), the runs (9); `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; commit; `python3 -I -B tools/citations/citations.py --base origin/main`.
4. Commit with the account; push with `git push origin HEAD:refs/heads/<your branch>`. Every commit carries `Look-request: ask-temper-ways` directly above its other trailers, the line's fade being a miss of that request (`temper/docs/design/draft/look-requests.md`, 3).

The account's rows, at least:

| Item | Result |
|---|---|
| A person's ring orange, an agent's star ink, code grey | landed (`temper.css`) |
| A stop made only when needed, outlined | landed (`WAYS_MAYBE`), drawn from the wire once `how-drawing` lands |
| A stop's small words over its own | landed (`WAYS_WHY`), likewise |
| The line as the request's continuous gradient | landed: each rise a fade, a person's from the signal and an agent's from ink |
| The legend's agent when needed | landed |
| Render check | follows landing, temper/ui's (`temper/docs/design/draft/look-requests.md`, 4) |

## Report

The implementer's report: the data's layout and the new bounds with their sums, the marks and the fades as drawn, the tokens appended, the rows and the bites with their failures, the runs' counts, and the account as committed.
