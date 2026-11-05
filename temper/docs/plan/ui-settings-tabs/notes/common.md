# ui-settings-tabs: the rules every task names

temper's client is to look and work exactly like the greenfield mock (`temper/docs/design/client/greenfield/`, its `README.md` says what's what). The mock draws none of these tabs: this plan is its language applied to what the client has. Every fact, act, form and state stays as built, and only its look changes; nothing is new behaviour but Builders. Read this note, then `temper/docs/plan/ui-settings-tabs/notes/contracts.md`, then your spec. Where this note and the implementer's text differ, this note wins.

## Departures from the implementer's text

- **Decide, don't ask.** The user is asleep and takes the plan's judgement. A stored format, a protocol or a public interface may change freely: no compatibility, no `Question:`; say it in your report. A small edit you need just outside your file list (a re-export, a `mod` line, a design line, a golden) is made and named in the report, not parked. A `Question:` is only for what truly blocks the task.
- **Design text:** the cut wrote the design. A task may edit only the `docs/design/` files its `Owns and may change` lists: the status entries it closes, and a line the code taught, named in the report.
- **Generated files** your change makes stale (terminal goldens, gallery names, counted baselines) are yours to regenerate in the same task, by their own tool.
- **No `Look-request:` trailer:** this plan lands no look request.

## The look

- **The mock's rules are taken, values and all,** from `temper/docs/design/client/greenfield/mock.css`, as `README.md`, 3.5, *The rows*, and `browser.md`, 2.5, say. Reach for the mock, not the old page.
- **The tokens are `--gf-*`** in `temper.css`. A rule under the old `--temper-*` tokens that reaches these pages goes where it stands, rather than being overridden.
- **Beside it:** the welcome's forge and model steps share `forge/connect.rs` and `pages/connect_model.rs`. They keep their look as built. A control a row shares with them takes the tab's look by a rule scoped under `.temper-settings-row`, never by a change to what they draw.
- **The dots stay:** the tabs' marks, as `settings::open` draws them.
- **The names stay:** most tests of these pages look for names and roles, and stay. Where a heading moved or its words changed, follow it. No test loses a state.

## Across plans

`ui-settings`' `how-work-runs`, `settings-proposal` and `settings-proof` are still to land, on `pages/settings.rs`, `look.rs`, `words.rs`, `temper.css` and the gallery, and `ui-chats`, `ui-needs`, `ui-welcome` and `ui-goal-page` on `temper.css`, `look.rs`, `words.rs` and the gallery. fab merges them in as they land. So:

- **Read the trunk, not this plan's memory of it.** A name a spec gives is on the trunk unless the spec or `notes/contracts.md` says a task makes it.
- **`temper.css`:** new rules go under `/* ==== ui: settings ==== */`, never at the end of the file. Replaced rules are removed where they stand.
- **`look.rs`:** this plan's tokens are `SETTINGS_*`, at the next free number. A number another plan took on a catch-up is renumbered, the higher one moving.
- **Goldens, prints and the gallery's `names.txt`/`tests.rs`** conflict across plans: on a catch-up, regenerate rather than hand-merge, and keep `names.txt` sorted as it is.
- **Keep to your functions in `pages/settings.rs`:** `how-work-runs` is rewriting `how_work_runs` there.

## The terminal

The terminal's prints keep working and keep their meaning, `adopt_*` golden pages among them. These pages' rows, facts and acts print as they did: a change here is to styles, slots and words, and a golden that moves for more than the changed words is a red.

## The proof

Each task proves its pages against the mock's How work runs tab, for its frame and rows (`temper/docs/design/client/greenfield/settings.html`): its tab or page at 1440, 1000 and 420 wide, every gallery sample its spec names, with a row open in each state. Screenshots go in `~/src/fabrig/ui-settings-tabs/<task>/` on the builder, outside any worktree, the report saying which builder. Never committed.

- Render with `chromium --headless=new --no-sandbox --disable-gpu --allow-file-access-from-files --screenshot=OUT.png --window-size=W,H --virtual-time-budget=3000 file://...`. The mock's Google Fonts link needs the network; the client's own fonts are what's compared.
- Chromium 147's new headless won't go below 500 px by `--window-size`: a 420 px proof uses CDP's `Emulation.setDeviceMetricsOverride`. Handle `Page.javascriptDialogOpening`, or a pending request's `onbeforeunload` hangs a CDP driver.
- A row opens through CDP (`Runtime.evaluate`, `details.open = true`).
- `temper-client-gallery`'s binary prints the samples to a directory it is given; its `temper/tools/gallery/src/main.rs` says how.
- A throwaway render script stays untracked and is gone before the push.
- A state no gallery world reaches: add a sample if a few lines reach it (`names.txt` sorted, `tests.rs`' `Holds` beside it), otherwise name in the report the view test that holds the state.
