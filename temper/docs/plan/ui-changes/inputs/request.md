# ui-changes: the Changes page, as the greenfield mock draws it

At the user's request (2026-11-05, through fabrig/ui): temper's client is to look and work exactly like the greenfield mock, modulo the look requests already landed. There is no supported deployment: formats, protocols and public interfaces may change freely, with the fewest moving parts. This plan starts once `look-round-4` has landed. The first of three plans of the area: `ui-changes`, `ui-settings`, `ui-settings-tabs`.

## The mock

The mock is on main at `temper/docs/design/client/greenfield/` (its `README.md` says what's what). Open its pages from the checkout; nothing needs extracting.

Render with `chromium --headless=new --no-sandbox --disable-gpu --allow-file-access-from-files --screenshot=OUT.png --window-size=W,H --virtual-time-budget=3000 file://...`. The mock's state is in localStorage; `hello` is reached with `changes.html?project=hello`.

## 1. The page

`Changes { project }`, at `/projects/<project>/changes`, a project page in the rail's frame. A column as Home's: `.col`, `max-width: 720px; margin: 0 auto; display: flex; flex-direction: column; gap: 60px` (32 on a phone). Reuse Home's intro, kicker, heading, lede and section head styles, moved to shared names if need be; `temper.css` has the mock's tokens as `--gf-*`.

In order:

- **Kicker:** `.kicker`, "PROJECT · landing".
- **Heading:** `.headline`, level 1:
  - none queued: "Nothing is on the way to main."
  - one: "One change is on the way to main."
  - two to nine, the number in words: "Three changes are on the way to main."
  - ten or more, in digits: "12 changes are on the way to main."
  - "main" is the landing branch's name. With queues into more than one branch, "on the way to landing" in its place.
- **Lede:** `.lede`, the line then ". Each change is checked on top of BRANCH before it lands.", BRANCH as above, "its branch" for several. The line is Home's "Landing:" line, without its "Landing: " and its link, and one function serves both, whichever of this plan and `ui-home-status` lands second moving to it. Joined by " · ":
  - something queued: "3 queued", "last landed 6 min ago";
  - nothing queued, something landed: "main last moved 1 h ago", "nothing queued";
  - nothing ever landed and nothing queued: "nothing has landed yet";
  - then "main red" while the forge's red record holds the branch, `--gf-signal`. The mock's "main green at 07:36" is dropped, as in `ui-home-status`: no green time is kept.
- **Queue:** `.section`, gap 14; `h2.section-title`, "Queue" (13 px, 600, `.06em`, capitals, muted); then `.lines`:
  - `.lines { display: flex; flex-direction: column; border-top: 1px solid var(--line); }`
  - `.lines > a, .lines > div { display: flex; justify-content: space-between; align-items: baseline; gap: 16px; padding: 14px 4px; border-bottom: 1px solid var(--line); text-decoration: none; }`
  - `.lines > a:hover { background: #f1f1ef; }`
  - `.lines .when { flex: none; font-size: 13px; color: var(--faint); }`
  - `.mono { font-family: var(--mono); font-size: 12px; color: var(--faint); }`
  - each row: at its left the change's title, a space, then `<span class="mono">` holding where it came from: the goal's title, or "chat T6"; at its right `.when`:
    - the first: `<span class="state-running">checking</span> · checks started 6 min ago`, `.state-running { color: var(--good); }`;
    - the others: "queued 2nd", "queued 3rd", "queued 4th".
  - empty: `<p class="empty">Empty.</p>`, muted, padding 4px 0.
  - more rows than the document's bound: a last row "and N more".
  - several landing branches: a section each, titled "Queue · owner/name BRANCH".
- **Landed recently:** the same section, titled "Landed recently": title, mono `from`, `.when` the time by the client's Times (5.12). Newest first, across the project's repositories, at most eight. Empty: the `.lines` alone, a rule, as the mock draws it.

Every row is a link to the change's `Task { task }` (the mock's rows are `div`s and it has no change's page; `.lines > a` carries the hover). A task page's "Changes" breadcrumb, owed since 5.10, is drawn: "Changes", to this page.

**The rail:** "Changes", after "Chats", current on this page, as the mock's `navItems()`. Shown for every project in this plan; `ui-settings` gates it. It is not in the phone's tab bar. The switcher keeps this page across projects, as on `Home` and `Settings` (5.3). The terminal prints the page; the view is shared.

## 2. The engine

**A watch, `Changes { project }`,** of the forge's family, in the project's scope, for a party with a role there. Model it on `Landings`: `temper-engine-domain-forge/src/views.rs` (`landings_subject`, `landings_view`, a third `Document` arm), `temper-engine-domain/src/route/panels.rs`, the wire's source in `temper-client-protocol-wire`, and the client's `temper-client-domain/src/forge/dismissed.rs` for a watch's life. Its document:

- `queues`: per landing branch with ready changes: the repository's `owner/name`, the branch, whether it is red, and the changes in order, each its task, its title (`ChangeRow.title`), its time ready, and, for the first, when its checks began. Reuse the order and the windowing of `crate::domain::queue_ready`, which `public/landings.rs::public_queues` and `panel.rs::queue_place` use. A queue is derived, never stored (`domain/forge.md`, section 9).
- `landed`: the last landings, below.
- marked as a change enters, leaves or moves in a queue, and at each merge.

Bounded by limits of its own, 32 queue rows and 8 landed in `small`; the panel names how many it cut.

`from`: the client reads each listed task, as `pages/task.rs` reads a chip: `Task.goal` gives the goal's title; else the root of `lineage_chips`, "chat T6". Reading is bounded by the rows shown; until it arrives the row has no mono part.

**`LandingLog`, family 30, a small version.** Designed in `domain/after-landing.md`, section 10, and not built (`protocol/store.status.md`). Build it as a ring per adopted repository, `landings_kept` rows (16 in `small`), written at each merge of temper's, in the merge's decision: the commit, the engine's wall time, and its changes, each its task, its pull's number and **its title**, new, since a change's close takes its row. Moves from outside, trailers and tier verdicts are left out until after-landing needs them. Its rule "a main no tier or subscription follows has no log" goes: every adopted repository logs. Erased with its repository, as the design has it.

## 3. Decisions already taken

Settled; do not ask.

- Held, not-ready and paused changes are not drawn (`client/forge.md`, section 3 asks for them). Held work is in Needs you. Cut section 3 to what is drawn, keeping the rest as later.
- No page of a change's own: `Task { task }`.
- Home's "Landing: … · Changes" line is `ui-home-status`', which draws its link once this page exists.
- A stored format, a protocol or a public interface may change: decide, and note it in `plan.md` so that it reaches the user.

## How the cut shapes it

- Two to four tasks, chained with `needs`: the design first (below); the engine; the page. `temper.css`, `temper/tools/gallery/src/tests.rs` and `names.txt`, and the terminal's golden pages conflict across plans: `needs` holds them in one line.
- **Design documents updated:** `temper/docs/design/client/README.md` (grep, never read whole: 3.5's table and its text, 5.3, 5.10's breadcrumb, section 11's two entries on it); `client/forge.md`, sections 3 and 9; `protocol/store.md`; `domain/after-landing.md`, section 10; their `.status.md`.
- **Tests:** the view's step tests; gallery samples `changes`, `changes-empty`, `changes-one`, `changes-landed-only`; a world test that lands a change and reads it in the document, with its title, after its close.
- **The proof:** the page against the mock's at 1440, 1000 and 420 wide: `changes.html` (fabrig, three queued), `changes.html?project=hello` (nothing queued), and a project with nothing landed. Screenshots of both, side by side by name, in `~/src/fabrig/ui-changes/` on the builder, outside any worktree, the report saying which builder. Never committed. Throwaway render scripts stay untracked.
- No `Look-request:` trailer: this lands no look request.

**Beside it:** other `ui-*` plans work on Home, goals and chats. The rail is shared: expect catch-ups.

## Size

One cut is at most 90 minutes. Release early. Budgets and timeouts are the user's (skein's `docs/foundation/testing-strategy.md`, 8.1).

## Render proofs

Chromium 147's new headless won't go under 500 px with `--window-size`, so a 420 px proof sets device metrics through the DevTools protocol (`Emulation.setDeviceMetricsOverride`). Any task may use a throwaway render script, untracked and gone before the push. Fonts: the mock's Google Fonts link needs the network; the client's own fonts are what's compared.

## Sources

- `temper/docs/design/client/forge.md`
- `temper/docs/design/domain/after-landing.md`
- `temper/docs/design/protocol/store.md`

## For every task of this plan

- **Decide, don't ask.** The user is asleep and takes fabrig/ui's judgement. A stored format, a protocol or a public interface may change freely: no compatibility, no `Question:`; note it in `plan.md`. A small edit a task needs just outside its file list (a re-export, a `mod` line, a design line, a golden) is made and named in the report, not parked. A `Question:` is only for what truly blocks the task.
- **`temper.css`:** put new rules under this plan's marker comment, `/* ==== ui: AREA ==== */` (chats, needs, goals, changes, settings, welcome, phone), never at the end of the file, so that parallel plans merge cleanly. Remove replaced rules where they stand.
- **Goldens, prints and the gallery's `names.txt`/`tests.rs`:** they conflict across plans; on a catch-up, regenerate rather than hand-merge, and keep `names.txt` sorted as it is.
- **Render proofs:** Chromium 147's new headless won't go below 500 px by `--window-size`; a 420 px proof uses CDP's `Emulation.setDeviceMetricsOverride`. Any task may use a throwaway render script, untracked and gone before the push. Handle `Page.javascriptDialogOpening`, or a pending request's `onbeforeunload` hangs a CDP driver.
- **Reach for the mock, not the old page:** `temper/docs/design/client/greenfield/`; where old tests walk a part the mock removes, change them with it.
- **Generated files** (`docs/help/reference/*`, prints, goldens, counted baselines) that your change makes stale are yours to regenerate in the same task; don't park on them.
