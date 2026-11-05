# ui-home-status: what changed since you last looked, written on Home

At the user's request (2026-11-05, through fabrig/ui): Home's everyday page is to read as the greenfield mock's. This is the second of three plans for the area. It `needs` `ui-needs` and `ui-goals`. The frame and "To begin" have landed; this plan adds only what Home's intro and foot lack. It holds for Home on any project, not while "To begin" shows.

## The mock

The mock is on main at `temper/docs/design/client/greenfield/` (its `README.md` says what's what). Open its pages from the checkout; nothing needs extracting.

```
Tuesday, 08:04 · you last looked 9 hours ago
41 changes landed overnight. Three things need you.
first-run-projects is six of nine tasks in. Its store task hit the wasm size limit twice and was split in two.
Written by the status agent at 07:42
...
Landing: 3 queued · last landed 6 min ago · main green at 07:36 · Changes
```

## 1. The engine's facts

All in the forge, as `Dismissed` is (`temper/docs/design/draft/to-begin.md`, section 5; `domain/forge.md`, 5.5; `protocol/store.md`, 133-172):

- **`ProjectFacts`,** a family at the store's next free byte, keyed by project: `landed` (a count of changes landed, every landing branch of the project), `last_landed` (the wall time of the latest, its change's ended time), and `brief`, none or `{ sentence, lede, at, landed }`. Saved in the decision that lands a change.
- **`Looked`,** the next byte, keyed by party and project: `previous`, `latest` (walls) and `seen` (the `landed` count at `previous`).
- **A FORGE request `Looked { project }`,** as `Dismiss` goes: the client sends it once on entering Home, not while "To begin" shows. With `latest` under 30 minutes old it moves `latest` alone; otherwise `previous` becomes `latest`, `seen` becomes the current `landed`, `latest` the time. So a sitting keeps one "last looked". No row: `previous` none.
- **A subject `status`,** per project and per asking party: the `ProjectFacts` row, the party's `Looked`, and `queued` (the project's changes in temper's landing queues and the forge's merge queue) and `red` (the forge's red record holds for the home repository's landing branch), computed at the read. A document, marked as `dismissed` is, moved by a landing, a queue change, a brief, a look.

Erased with its project. There is no supported deployment: formats change freely. Update `protocol/store.md`, `domain/forge.md` and `client/README.md`, 4.6.

## 2. The computed Home (first, and alone worth landing)

`temper/crates/temper-client-domain/src/pages/home.rs` watches `status`; `pages/home.rs` in the view draws:

- **Kicker:** the day and time, then, with a `previous` over a minute old, " · you last looked SPAN ago": "2 minutes", "an hour", "9 hours", "3 days". New word; the existing span's short form isn't this.
- **Headline:** the landed sentence, then the needs sentence `ui-needs` left. Landed sentence, with `landed - seen` as N: the brief's `sentence` if the brief's `landed` count is above `seen`; else "One change landed since you last looked." or "N changes landed since you last looked."; nothing for 0. Until `status` is delivered, as now.
- **Lede and byline:** the brief's, if as above. Otherwise none: no computed prose. `temper-home-lede`, and a byline "Written by the status agent at 07:42" (`temper-home-byline`, 12px `--gf-faint`).
- **The foot,** `temper-home-facts`, after the last section, 13.5px muted:
  - queued above 0: "Landing: 3 queued · last landed 6 min ago"
  - none queued: "Landing: main last moved 1 h ago · nothing queued"
  - nothing landed: "Landing: nothing has landed yet"
  - then " · main red" in `--gf-signal` while `red`. The mock's "main green at 07:36" is dropped: no green time is kept. "main" is the home repository's landing branch's name.
  - then " · Changes", a link, once the Changes page exists; without it, not drawn. `client/README.md`'s section 11 entry is settled by this.
- **Chats' section,** with no goal in the project (the `Domain` fact `ui-goals` keeps for its rail): after its rows, `.facts`: "A chat suits one change. For bigger work, a chat can write a draft beside it and start a goal from it: a plan, its tasks, landing." "Turn on drafts and goals", to Settings, only once Settings has that switch.
- **Goals' section:** `ui-goals` makes `goals::row` public for this. With a goal in the project, Goals takes Chats' place: `section-head` "Goals", "All N" to `Address::Goals { project, filter: All }`, then rows by `home`'s rule: live first, then ended, `Math.max(5, live)` rows. Its reads are that page's.

## 3. The status agent

Simplest real version. Decided:

- **A stock charter 7,** the status agent's, in `temper-service`'s `stock` (`stock/status.txt`, `stock.rs`, as charter 6 is; `protocol/configuration.md`, 13.2 and 13.4), models as charter 1's, a run of at most 4 turns and 5 minutes, no tools but finish, no workspace, no notes. Words: "You write the few lines at the top of a project's home page, from the facts in your task. Write one sentence on what landed, starting with the count. Then a blank line, then at most three plain sentences on what stands out: what finished, what is stuck and why. Say only what the facts say. No greetings, no lists, no advice."
- **When:** at a `Looked` that moves `previous` (a new sitting), with `landed` above the brief's `landed` count, the brief over an hour old or none, and none running for the project. No timers: no spend when nobody looks, or when nothing landed.
- **Made by the forge** as a tier run's task is (`forge::Request::TierRun`, the root's `tier_route.rs`, `TierTask`): a new `Request::StatusRun { project, words }`, under the project's pool, unlisted, forgotten once read, no person asked. Spec words: the span, the landed count, titles of the changes landed since the last brief (at most 20), the project's live goals with their landed and total changes, and failures the forge holds since. Only what the forge keeps.
- **Heard as a tier's result is:** the report's first paragraph is `sentence`, the rest `lede`, each bounded (200 and 600 bytes). Saved to `ProjectFacts.brief` with `at` and the `landed` count. A failed, cancelled or empty run saves nothing and asks nobody; Home keeps the computed words. The next chance is the next sitting.
- **Settings' "The brief on Home"** isn't this plan's. The agent always runs for now.

## 4. The terminal

The kicker, headline and lede keep reading as one run of words; the foot is a paragraph. `each_fixture_page_prints_as_its_golden_file`: reprint where meaning holds.

## Decisions already taken

- "overnight" is the agent's word; the computed sentence says "since you last looked".
- The computed Home has no lede and no byline.
- Green's time is dropped; red is shown.
- The landed count covers all the project's landing branches. A merge to a goal's branch counts; say so in the design.
- A change to a stored format, a protocol or a public interface needs no `Question:` in this plan: decide, and note it in `plan.md`, so that it reaches the user. Note the new families' bytes: another plan may take the next byte first. Catch up.

## How the cut shapes it

Tasks, chained by `needs`:

1. Design: `domain/forge.md`, `protocol/store.md`, `client/README.md` 3.5, 4.6 and section 11's "What changed since the person last looked" and "The landing's line on Home" settled, `protocol/configuration.md` 13.4.
2. The families, `Looked`, `status`, with tests.
3. Home's kicker, headline, foot, and Chats' sentence.
4. Charter 7, `StatusRun`, the result heard; a test with a scripted fake run; the host-part figures.
5. Home's lede and byline.

**Gallery states** (from fixtures): `home-morning` (with everything), `home-computed` (no brief), `home-last-looked-none`, `home-landing-none`, `home-landing-red`, `home-chats-sentence`.

**The proof:** Home rendered against the mock's `home.html` at 1440, 1000 and 420 wide, full page, and its phone's order, with screenshots saved on the builder under `~/src/fabrig/ui-home-status/`, outside any worktree, the report saying which builder. A real walk with `temper-fake`: land a change, come back past 30 minutes on a fake clock if the tests have one, see the count, the agent's words and the byline.

**Beside it:** `ui-away` chains after this and adds the Away words to the same intro. Keep the intro's composition in one function.

## Size

One cut is at most 90 minutes. Budgets and timeouts are the user's (skein's `docs/foundation/testing-strategy.md`, 8.1).

## Render proofs

Chromium 147's new headless won't go under 500 px with `--window-size`, so a 420 px proof sets device metrics through the DevTools protocol (`Emulation.setDeviceMetricsOverride`). Any task may use a throwaway render script, untracked and gone before the push. Fonts: the mock's Google Fonts link needs the network; the client's own fonts are what's compared.

## Sources

- temper/docs/design/draft/to-begin.md
- temper/crates/temper-client-view/src/pages/home.rs
- temper/crates/temper-client-domain/src/pages/home.rs
- temper/crates/temper-client-domain/src/forge/dismissed.rs
- temper/crates/temper-engine-domain-forge/src/dismissed.rs
- temper/crates/temper-engine-domain/src/route/forge_route/tier_route.rs
- temper/crates/temper-service/src/stock.rs
- temper/crates/temper-service/src/stock/foreman.txt

## For every task of this plan

- **Decide, don't ask.** The user is asleep and takes fabrig/ui's judgement. A stored format, a protocol or a public interface may change freely: no compatibility, no `Question:`; note it in `plan.md`. A small edit a task needs just outside its file list (a re-export, a `mod` line, a design line, a golden) is made and named in the report, not parked. A `Question:` is only for what truly blocks the task.
- **`temper.css`:** put new rules under this plan's marker comment, `/* ==== ui: AREA ==== */` (chats, needs, goals, changes, settings, welcome, phone), never at the end of the file, so that parallel plans merge cleanly. Remove replaced rules where they stand.
- **Goldens, prints and the gallery's `names.txt`/`tests.rs`:** they conflict across plans; on a catch-up, regenerate rather than hand-merge, and keep `names.txt` sorted as it is.
- **Render proofs:** Chromium 147's new headless won't go below 500 px by `--window-size`; a 420 px proof uses CDP's `Emulation.setDeviceMetricsOverride`. Any task may use a throwaway render script, untracked and gone before the push. Handle `Page.javascriptDialogOpening`, or a pending request's `onbeforeunload` hangs a CDP driver.
- **Reach for the mock, not the old page:** `temper/docs/design/client/greenfield/`; where old tests walk a part the mock removes, change them with it.
- **Generated files** (`docs/help/reference/*`, prints, goldens, counted baselines) that your change makes stale are yours to regenerate in the same task; don't park on them.
