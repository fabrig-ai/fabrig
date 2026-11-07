# ui-folds-away: skein's menu stories stop flaking

red-65's repair, as `inputs/request.md` asks: `skein-browser-world::browser_real`, `folds_away::a_press_outside_closes_a_menu_and_goes_on`, failed on its run and its rerun at `2a158edd47`, and `folds_away::*` has flaked in about 4 of every 10 after-landing runs since 2026-11-06, in all three stories.

## Parts and tasks

| Part | Task | Tree | What it makes true |
|---|---|---|---|
| `stories` | `settle` | skein | the three stories run alone before the edit until they fail and are read; every story starts by seeing the fixture's first apply; any other read the runs show running ahead of its act waits for what a person would see; fifteen runs alone pass after |

**Critical path:** `settle` alone, about 90 minutes, most of it the thirty to forty-five runs of single stories and the example's wasm build and bundle.

## What this cut decided

- **Reading the code says the stories act before the page is there.** A `Go` ends at the page's load event (`skein-browser`'s `browser.rs`). Since `e8600f44be` (2026-11-06 14:05Z, the day the flakes began) the example loads behind the loader (`skein/docs/design/ui-web.md`, 4.3), which fetches, compiles and starts the module after that load. A person's press whose find comes back absent fails at once with `Missing` (`skein-fake-person`'s `lib.rs`; `ui-web.md`, 14.2: only `Gone` is looked for again). Two stories press `Panel` straight after their `Go`. red-57 (`1d8ba3e0d2`) fixed the same race in the third story alone, and the sibling `log_end` sees before it presses. So the shared fix is one arrival every story starts with: the `Go`, then red-57's `See` of `Editor`, which one apply draws with every other node. The runs confirm or refute it; the spec has an option for each other cause the request names (a read once ahead of its act, a press landing elsewhere, a skein-ui-web fault) and for no failure.
- **No design is written.** `ui-web.md`, 14.2 already has a story await what it must see, never for a fixed time, and says the page's load comes before its first read is answered. The arrival makes the stories follow it. A skein-ui-web fault (option D) would close or change the status's existing **Departs** entry under 7, which the spec pre-approves.
- **What else the reading ruled out,** so the runs are read against it: the fixture's inbox reports in the act's own turn, and the page applies its fold, its menu and its focus inside the click's listener, before the kit's mouse input is answered. The loader's placeholder takes `out`, `inert` and `pointer-events: none` in the same task as the fixture's first apply, so it can't cover a press. The example's stylesheet has no motion, and its menus sit in the page's flow: what an open menu moves, it moves within the press that opens it. No harness, kit or product change is pre-approved but option D's.
- **The repair runs the three stories by name, alone,** in the `browser-skein` profile, five times each before the edit (ten if none fails) and five after, as a red's repair may (`AGENTS.md`, 5). That is the one departure from "a spec names no slow-tier run", and the request asks for it. The spec builds the example's module, bundle and server by hand through heavy, as `tools/tiers/design.md`, 2 lists them, and never runs `tools/tiers/run`, `scripts/browser.sh` or the tier.
- **Checks and deadlines stay.** Every `check`, `Focused`, `see` and `Keep` stays in its order; the arrival's `See` waits 2 s, as red-57's and the siblings' do; skein's poll and answer limits are untouched; no browser test is added.
- **One task,** since one file holds all three stories and the request asks for one.

## Overlaps with the drafts beside this one

None for the expected fix: no draft beside this one names `skein/tests/browser/`. Under option D alone, `settle` would edit `skein/crates/skein-ui-web/src/page.rs`, `src/page/web.rs`, `src/tests/page/events.rs` and `skein/docs/design/ui-web.status.md`, which `look-round-16/field-options` (not yet merged) is changing. The spec keeps those hunks small, and whichever lands second catches up.
