# ui-welcome: the first run, whole, as the mock draws it

At the user's request (2026-11-05, through fabrig/ui): temper's client is to look and work exactly like the greenfield mock, modulo look requests already made. There is no supported deployment: formats, protocols and public interfaces may change freely, with the fewest moving parts. Budgets are low priority. This plan starts after `look-round-4` and `claude-always` (section 1 of it at least: the Model step offers Claude on a fresh box).

The look of what the client draws today is done: Welcome, Forge's cards, the sheet and Ready's form match the mock to the pixel. What is missing is **states and acts**. Don't restyle what matches. The survey is `coord/ui/gaps/first-run.md`.

**The mock is not on main.** It is on `origin/ux-ideas/greenfield`, under `temper/docs/design/client/greenfield/`: read it with `git show origin/ux-ideas/greenfield:temper/docs/design/client/greenfield/app.js`, `welcomePage()` above all. Render it from a throwaway worktree of that branch, outside this tree. Its first run is a mode in localStorage: set `temper-greenfield-mode` to `first-run` and `temper-greenfield-first-run` to `{"setup":{"forge":…,"model":…}}` in an inline script, or press "Start the first run" on `welcome.html`. Its first project is "Default": ignore that (see Decided).

## 1. First: the sign-in

Plan `ui-signin` fixes the fake ChatGPT sign-in in a browser and lands first; this plan records with it (section 10).

## 2. The wizard's frame

`temper-client-view/src/pages/welcome.rs`, `steps`; mock's `steps` and last line of `welcomePage()`.

- **The steps are links,** every one, to its own step, in every state: `.steps a`. They stop using `flow::steps` (`ConnectForge` and `ConnectModel` keep it). Keep their look (`.temper-flow-step`): the number or tick in the ring, the label; below 760 only the current label shows.
- **A step's tick is a fact:** `done = { welcome: at != welcome or a forge exists, forge: a forge exists (even under way), model: a usable model, ready: never }`. A step is ticked when done and not current. The current step is always current: **Ready stays current**, black "4", when both are done. Today it shows all ticks.
- **`/welcome?at=welcome`** opens the Welcome step whatever the store's state, unless that state is whole, when it moves on as `/welcome` does. Bare `/welcome` keeps its decision (`pages/welcome.rs`, `decision`). The step's link goes to `?at=welcome`.
- **The Welcome step with a forge begun:** the two cards draw as ever, but "Set up temper"'s button reads **"Go on setting up"** and goes to Ready if a model is usable, else Model. The playground's card is left out while the playground isn't offered, and the note under them with it.
- **The brand** links to `/welcome`, not `/chats`.
- **The tab's title** on every step: "Welcome · first run · temper".

## 3. The Forge step with a forge

`forge/connect.rs`, `welcome`; mock's `at === 'forge'`, branch `else if (f)`. Today: one line and "View", or a card "Setting up Forgejo on this machine".

For a personal forge under way or connected (GitHub with an accepted installation counts):

- The h1 "Connect a forge". The lead: **"One forge is enough to begin; add others later, from Settings."**
- `.forge-status`, in the column (not the Model step's pane): under way a spinner, the forge's name, a muted line, and the disclosure "Each step"; connected a tick, the name, and "connected as temper · work/hello made". Reuse `welcome_line`'s words. On GitHub: "temper’s App on ACCOUNT", ACCOUNT the first accepted installation's. No repository count.
- `.acts`: the primary **"Continue"**, to Ready if a model is usable, else Model.
- The foot.

A failed set up, a forge cut short, and GitHub's install and Forgejo's admin steps stay as they are. "View" goes: it had no place in the mock.

## 4. The Model step and its ways

`pages/connect_model.rs`, the `welcome(root)` branches; mock's `at === 'model'`. Standalone `ConnectModel` keeps everything. The forge's pane beside the column stays (look request `welcome-model-forge-pane`): where the mock has `status` above the head, the client has the pane.

- **A model usable, at any way or none:** the h1 "Connect a model", no lead; `.forge-status.done` "✓ ChatGPT plan · owner@example.com", in the same words as Ready's Model line; the primary "Continue" to Ready; the foot. This replaces both the choices and the "Signed in to ChatGPT as…" card with its facts, "Continue with ChatGPT", "Manage usage" and "…is on the Accounts page", on the welcome alone.
- **A way's page has its own heading,** with no lead, then one `.panel`. "‹ All ways" above the card goes; "All ways", quiet, stands after the primary in `.acts`:
  - **Key,** h1 "An OpenAI API key": "From OpenAI’s platform, under API keys. What temper’s agents use is billed to the key’s organization."; field "Key, pasted once", the panel's width; "Add key", disabled until typed; "All ways"; `.facts` "Nothing checks a key when it is added: a wrong one shows once OpenAI refuses a run."
  - **Paste,** the `elsewhere` state, h1 "Signing in to ChatGPT from another computer": `.numbered`: "Open ChatGPT’s sign-in › and approve temper." (the link), "When ChatGPT is done, its tab shows a page that can’t be reached. Copy that page’s whole address from the address bar.", "Paste it here."; field "Address of the page that couldn’t be reached", secret entry; "Finish connecting", disabled until typed; "All ways". Its refusals and endings (`welcome-paste-*`) keep their words, in the panel.
- **The sign-in under way, the tab not gone yet,** instead of the card, "LLM account sign-in started" and the bare "Pending": `.away-note`: a spinner, **"ChatGPT’s sign-in, in this tab"**, muted "You sign in and approve temper; ChatGPT sends this tab back to temper’s listener on this machine, and on to Ready." Keep its existing disclosure, "Signed in, but the page couldn't be reached?", under it. No `Mockup:` line.
- **Finishing a paste:** `.away-note`, spinner, "Finishing your sign-in…".

## 5. The Claude way

Mock's `way === 'claude'`, `data-claude-check`. Today the client declares a name and stops at "Declared": the account isn't usable until `temper ctl account verify`, so the wizard can't reach Create. **This is the happy path's first walk.**

- h1 **"Your Claude subscription"**, `.panel`: "temper sets Claude Code up on this machine in an environment of its own: its settings, sessions and sign-in stay apart from your own Claude Code. In a terminal here:"; `.cmd` `temper program-login claude-owner`, NAME from `program_default`, no Name field on the welcome; muted "It makes that environment, then Claude Code asks you to sign in to your subscription there. temper never reads its login."; primary **"I’ve signed in"**; "All ways".
- **"I’ve signed in"** sends `DeclareProgram` under that name if no account of the person's has that source, then the accounts page's `Operation::CheckProgram { source }`, as `temper ctl account verify` does (`walk/program_here.rs` shows the order). While out: `.away-note`, spinner, **"Checking Claude Code’s sign-in on this machine…"**. Once `model_usable` holds it moves to Ready, its address replacing the step's.
- **Not signed in** (the check answers and the source isn't usable; the mock doesn't draw it): the page stays, with an alert "Claude Code isn’t signed in here yet. Run the command above, then press I’ve signed in." A second press rechecks.
- **While Claude Code is being fetched** (`claude-always`, section 3) the `.cmd`'s command does that itself; nothing to add.

## 6. Ready

`welcome.rs`, `ready`; mock's last branch.

- **Forge, then Model, always.** Today Model comes first until both are done.
- **Forge done:** `Forge Forgejo on this machine · connected as temper · work/hello made`, the last part while a first repository was made. "Open Forgejo ↗" stays at the row's end.
- **The line's size:** 15px, what follows the strong name muted (`.checklist`): today 14px.
- **Under way, a line's ring spins:** `.checklist .current .mark { border-top-color: var(--ink); animation: spin .9s linear infinite; }`. Under way: Forge while it sets up, Model while signing in.
- **`Forge none yet · Connect a forge`:** one line, the link after " · ", as Model's is. Today the link sits at the row's right.
- **Model's words:** signing in, `Model ChatGPT plan · signing in…` (today `ChatGPT · signing in`). A program, `Claude subscription · claude-owner, signed in on this machine`. A key, `OpenAI API key · billed to the key’s organization`.
- **Under the form,** while a forge is under way and not ready: `.facts` **"Forgejo is still setting up; this page moves on by itself."**
- **The foot:** "‹ Back" to Model, as on Forge and Model, and "Try the playground instead" while offered.
- **"Set up again"** keeps its place, at its natural width.

## 7. The sheet's other two choices

Mock's `repoSheet('welcome')`, `repoChosen()`, `parseCopies()`; `temper/docs/design/draft/repository-copies.md`, 5 and 7. `WelcomeForgejoNotYet` and `not_yet` go.

**The engine.** `SetUp`'s first repository becomes `Empty { org, name }`, `Copies { addresses }` or `Nothing`, kept in the forge's record as today's org and name are (`protocol/setup.md`, 6.7), changed in place. Copies and Nothing make no repository at set up. The row tells clients the intent. `temper ctl forge setup` keeps `Empty`.

**The client.**
- **Nothing for now:** Run sends it. Ready offers "My project" and the plain lead.
- **Copies:** before sending, each non-empty line parsed as `parseCopies` does: strip `http(s)://`, `.git` and trailing `/`; `HOST/OWNER/NAME`, or `OWNER/NAME` meaning `github.com`; then `source::parse`. Words, in the sheet's alert: `Not an address: LINE. Write it as github.com/owner/name.`, `At least one address, as github.com/owner/name.`
- **Empty:** either field blank: one alert, "Both an organization and a name.", in place of "Required." under each.
- **Ready:** offered name from the first copy, capitalised ("Widget"). Lead: `Everything is connected. Create your first project, with acme/widget in it, and start working.`, or `with 2 repositories in it`. The forge's line: `connected as temper` alone.
- **The act:** `MakeProject`, then one `MakeRepository` (`forge/making.rs`) per address in order, `into { project, home: first }`, each under a key of its own, `welcome-copy-N`. All are sent at once: the engine's rows queue them. A taken-name project takes them the same way.
- **After a restart in between,** the row's intent still names addresses no making row holds: Home's second step offers them, "Copy", filled. One act.

## 8. Home after Ready

`pages/home.rs`, `to_begin`, the second step only. The rest of Home is `to-begin.md`'s and other plans'. Keep the row's look.

- **Copies running** (`repository-copies.md`, 7): a line per copy: "Copying acme/widget from github.com…", the next "Waiting to copy", then "acme/widget is in Widget", ticked. Failed: the forge's words and "Copy again". The rows come with the `forges` watch.
- **Copies in:** title `acme/widget is in Widget`, or `acme/widget, acme/other are in Widget`; line "Chats can change its code now, on your Forgejo; nothing goes back where it was copied from.", or "…their code… where they were copied from."
- **An owned or managed Forgejo with no first repository to offer** (Nothing for now): title **"Add a repository"**; line "Your Forgejo has none yet. Until one is in, chats talk and plan but change no code."; acts "Make an empty one" and "Copy a public one", small, and quiet "Adopt another…", to `Adopt { project }`.
- **Their sheet:** mock's `repoSheet('home')`, the welcome's sheet's look: "Add a repository to PROJECT"; muted "Made or copied on your Forgejo, then in this project at once."; legend "A repository"; "An empty repository" with its two fields, "Copies of public repositories" with its addresses (opened at the one pressed); Cancel, Escape; "Add it". It is the page's state, not an address. It sends `MakeRepository` `into { project, home: true }`, the first of several the home. The step's lines above say the rest: no toast. This sheet is for this state alone; `repository-copies.md`, 9's "Add repositories" stays Settings'.

## 9. The box's terminal

`temper-box/src/terminal.rs`, `layout::label`. Mock's `boxPage()` prints:

```
01:40:56Z Forge 1 (Forgejo): starting
01:41:09Z Forge 1 (Forgejo): started
```

The label becomes `Forge N (Forgejo)`, N the forge's number, for every `Group*` fact. The `api` and `person` lines (`Step::set_started_notice`) go to the box's log, not the terminal. Everything else already matches. `box.html`'s page around it is the mock's story, not a product page.

## 10. The journeys

- **`project-landing`,** recorded at last, to Home, from a walk of `ui-signin`'s kind, as `journeys.md`, 8. Update its README, the table in 9 and `journeys.status.md`.
- **`welcome-set-up`:** add nothing past Model; it stays within `again`'s 40 seconds. Refresh its prints only as these changes reach them.
- **Gallery states,** added to `names.txt`: `welcome-go-on`, `welcome-forge-under-way`, `welcome-forge-connected`, `welcome-forge-github`, `welcome-model-connected`, `welcome-model-claude`, `welcome-model-claude-checking`, `welcome-model-claude-not-yet`, `welcome-model-away`, `welcome-ready-none`, `welcome-ready-signing`, `welcome-ready-copies`, `welcome-sheet-copies`, `welcome-sheet-copies-refused`, `welcome-sheet-nothing`, `home-copying`, `home-copied`, `home-add-repository`, `home-add-repository-sheet`. `welcome-model-signing-in` and `welcome-paste-*` redraw. Check the fixture's providers: `welcome-model-*` show one card where a box shows two.

## Decided (the mock departs, or says nothing)

- **"Default" is superseded:** Ready offers "Hello" or "My project" (`first-run-projects.md`, 4). Don't copy the mock's name.
- **Copies run at Ready,** as `repository-copies.md`, 7 and the user's ruling say, not in the forge's steps as the mock does. So no "Copying…" on the Model step's pane and no "N repositories copied" on the forge's line.
- **"Open Forgejo ↗" stays** on Ready's forge line: Home lost Connected, so it's the one door to the forge.
- **The forge's pane on Model stays** (look request). The pill is for Forge's step alone.
- **`to-begin.md` overrides `freshHome()`:** no count headline, no Connected, "Skip" stays.
- **Skip:** every `Mockup:` line and `.mock-inline`, GitHub's `.away-note` (a real tab leaves at once), `notFirstRun`, `mockCorner`, `phone.html`, GitHub's repository count on the forge's line and Home, the playground (its pages are landed).
- **Kept though the mock doesn't draw them:** failed set up, a forge cut short, declined, held, refused, every paste ending, the port taken. Their words stay.
- **The email:** "ChatGPT plan · owner@example.com" needs jig's `account-facts`, section 6, step 2. Until that lands the line says "ChatGPT plan" alone. Not this plan's.
- **README's older words give way:** "Model then Forge while either is under way", "Ready is done, not current", "the brand's link to Chats", and the sheet's "Copies and nothing are not made yet".

## How the cut shapes it

- **One order, each `needs` the last:** the design; the frame and Welcome (2); Forge (3); Model (4); Claude (5); Ready (6); the engine's intent and the sheet (7); Home (8); the terminal (9); the journey and proof (10). `temper.css`, `temper/tools/gallery/src/tests.rs`, `names.txt` and the terminal's goldens conflict across plans: chain them, and expect catch-ups against `ui-home-status`, which also touches Home.
- **Design documents first:** `client/README.md` 3.5 (The welcome, `ConnectModel`), 4.6, 5.6; `client/forge.md` 5.4; `client/browser.md` 2.3 (`?at=welcome`) and 2.5; `protocol/setup.md` 6.7 and 7.1; `protocol/forge.md`; `draft/repository-copies.md` and `first-run-projects.md` with their status files. Keep section numbers: the queue's citation gate rejects a broken citation (`tools/citations --all`).
- **The terminal keeps its meaning:** the view is shared. Ready's order and the new lines print; the step list prints as text, its links as acts the terminal already offers. Goldens move with reasons.
- **Commits carry no `Look-request:` trailer:** nothing here lands a look request by name.
- **The proof, for every page and state above:** a render beside the mock's page at 1440, 1000 and 420 wide, plus its open states (the sheet with each choice, "Each step" open, the steps hovered), screenshots saved on the builder in `~/src/fabrig/ui-welcome/`, outside any worktree, the report saying which builder. Never committed. **Headless Chromium won't go below 500 wide by `--window-size`:** render 420 in an iframe of that width, or by CDP's `Emulation.setDeviceMetricsOverride`, or the media queries lie. Fonts: compare with the client's bundled Plex, loading the mock's from a local copy.
- **Two box walks in headless Chromium through CDP, on `temper box --fake-backends`, each a fresh box:** Forgejo with an empty repository, then Claude (`temper program-login` run by the walk, as `Act::ProgramLogin`), then Create, to Home with "work/hello is in Hello"; then the same with ChatGPT. Then a third: Nothing for now, then "Make an empty one" from Home. Copies stay on the fake forge's catalogue (`repository-copies.md`, 6): an owned Forgejo reaches no `.invalid` host, so prove the act in a process walk on a connected fake forge, and say what a real box's copy left unproved. Answer `Page.javascriptDialogOpening` in the driver.
- **Tests:** the address round trip for `?at=welcome`; `parseCopies`' table; the tick rule's table; view tests beside `tests/welcome.rs` for each state above, in browser and terminal; `SetUp`'s intent through store and wire; `CheckProgram` from the welcome in `tests/connect_model.rs`.
- **Throwaway render scripts** stay untracked and are gone before the push.

## Size

One cut is at most 90 minutes. Budgets and timeouts are the user's (skein's `docs/foundation/testing-strategy.md`, 8.1).

## Sources

- `temper/docs/design/draft/first-run-projects.md`
- `temper/docs/design/draft/first-run-projects.status.md`
- `temper/docs/design/draft/to-begin.md`
- `temper/docs/design/draft/repository-copies.md`
- `temper/docs/design/draft/repository-copies.status.md`
- `temper/docs/design/draft/claude-provider.md`
- `temper/docs/design/draft/serve-without-box.md`
- `temper/docs/design/draft/fetch-progress.md`
- `temper/docs/design/client/journeys.md`
- `temper/crates/temper-client-view/src/pages/welcome.rs`
- `temper/crates/temper-client-view/src/pages/connect_model.rs`
- `temper/crates/temper-client-domain/src/pages/welcome.rs`
- `temper/crates/temper-fake/src/oauth.rs`
- `temper/crates/temper/tests/fake_backends/walk/sign_in.rs`
- `temper/crates/temper/tests/fake_backends/walk/program_here.rs`

## For every task of this plan

- **Decide, don't ask.** The user is asleep and takes fabrig/ui's judgement. A stored format, a protocol or a public interface may change freely: no compatibility, no `Question:`; note it in `plan.md`. A small edit a task needs just outside its file list (a re-export, a `mod` line, a design line, a golden) is made and named in the report, not parked. A `Question:` is only for what truly blocks the task.
- **`temper.css`:** put new rules under this plan's marker comment, `/* ==== ui: AREA ==== */` (chats, needs, goals, changes, settings, welcome, phone), never at the end of the file, so that parallel plans merge cleanly. Remove replaced rules where they stand.
- **Goldens, prints and the gallery's `names.txt`/`tests.rs`:** they conflict across plans; on a catch-up, regenerate rather than hand-merge, and keep `names.txt` sorted as it is.
- **Render proofs:** Chromium 147's new headless won't go below 500 px by `--window-size`; a 420 px proof uses CDP's `Emulation.setDeviceMetricsOverride`. Any task may use a throwaway render script, untracked and gone before the push. Handle `Page.javascriptDialogOpening`, or a pending request's `onbeforeunload` hangs a CDP driver.
- **Reach for the mock, not the old page:** `temper/docs/design/client/greenfield/`; where old tests walk a part the mock removes, change them with it.
- **Generated files** (`docs/help/reference/*`, prints, goldens, counted baselines) that your change makes stale are yours to regenerate in the same task; don't park on them.
