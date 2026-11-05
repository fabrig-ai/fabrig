# ui-settings-tabs: Forges, Accounts and Builders in Settings, in the greenfield mock's look

At the user's request (2026-11-05, through fabrig/ui): temper's client is to look and work exactly like the greenfield mock, modulo the look requests already landed. There is no supported deployment: formats, protocols and public interfaces may change freely, with the fewest moving parts. This plan starts once `ui-settings` has landed, which draws Settings' frame, its tabs and How work runs, and puts today's `Forges` and `Accounts` under them.

## The mock

The mock is on main at `temper/docs/design/client/greenfield/` (its `README.md` says what's what). Open its pages from the checkout; nothing needs extracting.

Render with `chromium --headless=new --no-sandbox --disable-gpu --allow-file-access-from-files --screenshot=OUT.png --window-size=W,H --virtual-time-budget=3000 file://...`.

**The mock draws none of these tabs:** its tabs say "not drawn in this experiment". So this plan is the mock's language applied to what the client has: every fact, act, form and state stays as built, and only its look changes. No state is dropped. Nothing here is new behaviour but Builders.

## 1. The language

- The rows: `.lines`-like, each a folded disclosure: a hairline under each, 14 px 4 px, no card, no shadow. The summary: its name in ink, 15 px; its second line `small`, 13.5 px, muted; a chevron `⌄` at the far right, `var(--faint)`, as `.switcher > summary::after`.
- Opened: its content under the summary, 14 px below, indented nothing, with its facts as `.setting` rows.
- `.section-title` for each heading; "Your forges · 2"; the section's act at the head's right, `.section-head { display: flex; align-items: baseline; justify-content: space-between; gap: 12px; }`, a `.btn.small`.
- Buttons: `.btn` (min-height 40, padding 8 16, border `var(--line-2)`, radius 8, `var(--surface)`, 14.5 px); `.btn.primary`, ink; `.btn.quiet`, no border, muted; `.btn.small`, min-height 34, padding 5 12, 13.5 px.
- Fields: `.text`, min-height 38, padding 6 12, border `var(--line-2)`, radius 8.
- Everything in the page's `.col`, 720 wide. `temper.css` has the mock's tokens as `--gf-*`: rules under the old `--temper-*` tokens for these pages go, rather than being overridden. The dots stay.
- No card chrome: `look::CARD`, `PAGEBAR`, `HEADING`, `SUBTLE_COUNT` go from these pages.

## 2. Forges

`forge/forges.rs` and `forge/connect.rs` under the frame. The frame holds the heading and the description; the tab holds:

- "Your forges · N" with "+ Add a forge", a `.btn.small`, in its head, not while the playground runs, as today. None: `.band.quiet-band`, the tag "No forge yet", "No forge of yours yet.", and "Connect a forge".
- Each forge a row: "Forge 1 · Forgejo", its connection, its repository line. Opened, today's facts and acts in every state: setting up with its steps, failed, stopped, connecting, replacing, removing with its confirmation, GitHub's App and installations, a first repository making or failed; "Open Forgejo ↗", the owned forge's password's words, `Adopt` as today. `home-drops-connected` promised the first here.
- The playground's forge, its section and row.

## 3. Accounts

`pages/accounts.rs` and `pages/accounts/rows.rs` under the frame:

- "Your accounts · N" with "+ Connect a model", in its head, as today's act and condition. None: `.band.quiet-band`, the tag "No account yet", "No model of yours yet.", and the primary "Connect a model".
- Each account a row, its summary today's line (5.5, *An account's line*). Opened, today's states, sources, hosts, ranks, models, "Pays for", sign-in, paste, key, program logins, sign-out, remove with its confirmation, in every state.
- "Deployment accounts", an owner's, the same rows. The playground's, as today.
- The foot: `ui-settings`' "Signed in as NAME" and "Sign out".

## 4. `ConnectForge`, `ConnectModel` and `Adopt`

They stay pages of their own, in the welcome's look as built, with "Settings" current in the rail, each adding at its head a path as `Configure`'s: PROJECT / Settings / Forges / Add a forge, PROJECT / Settings / Accounts / Connect an account, PROJECT / Settings / Repositories / Adopt a repository. Links: to the project's `Home`, `Settings`, and the tab. On a phone "‹", a link to the tab, alone. Each one's ending returns to its tab where it ends with no page of its own. `Adopt` takes the `.text`, `.btn` and `.col` of the language.

## 5. Builders

New: `Builders { project }`, at `/projects/<project>/settings/builders`, the fifth tab, after "Accounts".

There is no Workers page to fold: the old mockups' `workers.html` was left out (`client/README.md`, 5.3 and 11). This is a small read: jig's `deployment` document, watched for a deployment role as `pages/accounts.rs` already does; `Deployment.hosts`.

- A section, "Builders · N"; `.lines`: a row per host: its name; `small`: "2 slots · 1 in use"; in `.when`: "Connected", "Draining" or "Not connected".
- Under it, `.facts`: "Each agent’s role names which builders it runs on. Tell Configure temper to change it."
- Not a deployment role's: one line, "Builders are seen by the deployment’s owners.", no rows.
- None connected: `.empty`, "No builder is connected."
- No acts. The word on the page is "builder"; the code's stays host.

The configured limit of `deployment` documents' rows is enough; change nothing of jig's.

## 6. Decisions already taken

Settled; do not ask.

- `Forges`, `Accounts` and their flows keep their addresses.
- Repository roles, health, landing rules and Delete are not drawn.
- Builders is read-only, with no capacity graph.
- Client README section 11's "Workers" entry changes to say what this draws.
- A stored format, a protocol or a public interface may change: decide, and note it in `plan.md` so that it reaches the user.

## How the cut shapes it

- Three or four tasks, chained with `needs`: Forges; Accounts; the flows' paths; Builders. `temper.css`, `temper/tools/gallery/src/tests.rs` and `names.txt`, and the terminal's golden pages conflict across plans: `needs` holds them in one line.
- **Design documents updated:** `temper/docs/design/client/README.md` (grep, never read whole: 3.5's table, 5.3, 5.5's page, section 11); `client/forge.md`, 5.4; `client/browser.md`, 2.3; their `.status.md`.
- **The tests' words:** most tests of these pages look for names and roles, and stay. Where a heading moved, follow it; no test loses a state.
- **Gallery:** keep every sample of `forges-*`, `forge-*`, `accounts-*`, `connect-*`, `adoption*`; add `builders`, `builders-not-owner`, `builders-none`.
- **The proof:** no mock page exists, so against the mock's How work runs tab, for its frame and rows: each tab at 1440, 1000 and 420 wide, every gallery sample above, with a row open in each state. Screenshots in `~/src/fabrig/ui-settings-tabs/` on the builder, outside any worktree, the report saying which builder. Never committed. Throwaway render scripts stay untracked.
- **The terminal's prints** keep working and keep their meaning: `adopt_*` golden pages among them.
- No `Look-request:` trailer: this lands no look request.

**Beside it:** the welcome's forge and model steps share `forge/connect.rs` and `connect_model.rs`: keep their look as built, drawing this page's look only where the place is the tab.

## Size

One cut is at most 90 minutes. Release early. Budgets and timeouts are the user's (skein's `docs/foundation/testing-strategy.md`, 8.1).

## Render proofs

Chromium 147's new headless won't go under 500 px with `--window-size`, so a 420 px proof sets device metrics through the DevTools protocol (`Emulation.setDeviceMetricsOverride`). Any task may use a throwaway render script, untracked and gone before the push. Fonts: the mock's Google Fonts link needs the network; the client's own fonts are what's compared.

## Sources

- `temper/docs/design/client/forge.md`
- `temper/docs/design/client/journeys.md`

## For every task of this plan

- **Decide, don't ask.** The user is asleep and takes fabrig/ui's judgement. A stored format, a protocol or a public interface may change freely: no compatibility, no `Question:`; note it in `plan.md`. A small edit a task needs just outside its file list (a re-export, a `mod` line, a design line, a golden) is made and named in the report, not parked. A `Question:` is only for what truly blocks the task.
- **`temper.css`:** put new rules under this plan's marker comment, `/* ==== ui: AREA ==== */` (chats, needs, goals, changes, settings, welcome, phone), never at the end of the file, so that parallel plans merge cleanly. Remove replaced rules where they stand.
- **Goldens, prints and the gallery's `names.txt`/`tests.rs`:** they conflict across plans; on a catch-up, regenerate rather than hand-merge, and keep `names.txt` sorted as it is.
- **Render proofs:** Chromium 147's new headless won't go below 500 px by `--window-size`; a 420 px proof uses CDP's `Emulation.setDeviceMetricsOverride`. Any task may use a throwaway render script, untracked and gone before the push. Handle `Page.javascriptDialogOpening`, or a pending request's `onbeforeunload` hangs a CDP driver.
- **Reach for the mock, not the old page:** `temper/docs/design/client/greenfield/`; where old tests walk a part the mock removes, change them with it.
- **Generated files** (`docs/help/reference/*`, prints, goldens, counted baselines) that your change makes stale are yours to regenerate in the same task; don't park on them.
