# journey-walks: a journey is its walk, printed in a world, and main's pages have one writer

Draft, 2026-11-18, from a session with the user. They asked for "an assessment of the situation and how we could turn it into a nice clean story", having seen the journeys conflict between unrelated goals, check too much in, be recorded again without need and run slowly. Their direction, the same day: the printed pages may stay on main "with the constraint that only the writer writes it"; "recording / replay should be as cheap as possible according to the context", so that an agent answering "does what I write match the diff/html in the request" does it "without a full blown process (box or whatever)", through "a world-level client with a renderer"; and a check after landing is enough. The figures are the survey's, on main at `7307b6172b` (`temper/docs/development/journeys-survey-2026-11-18.md`).

It is the journeys' part of what `codec-shape.md` does for generated code, which leaves recordings aside (`codec-shape.md`, 5.3). Where they differ, this draft wins over the browser's journeys (`../client/web/journeys.md`, sections 3 to 8), the TUI's (`../client/tui/journeys.md`, sections 3 to 10), `../client/journey-clock.md`, `../client/journey-placeholders.md`, the look requests' steps (`../client/web/look-requests.md`, 2 to 5) and the drafts `journey-clock.md`, `journey-placeholders.md`, `real-journeys.md` and `record-anywhere.md`, until its goals fold it into them.

## 1. In one page

**The story:**

- **A journey is its walk:** a short script of what a person does and where a page is printed, with a README. It is all of a journey that anyone writes, and all that a goal changes (section 3).
- **One walk, three places to run it,** by what each costs (section 4):
  - **in a world,** in one process, with no box, browser, terminal, recording or network: for an agent, and for main's pages;
  - **on a box,** with a real browser and a real terminal: after landing, for what only they show;
  - **by hand, on a deployment,** with the recorder: for a person, and the one place a recording exists.
- **The pages are printed in the world,** the same bytes from the same tree, every time (section 5).
- **Main's pages have one writer,** a command run after a landing. No goal writes a page (section 6).
- **A goal checks its work by printing into scratch,** and compares what it prints with the request's pages (section 7).
- **No recording is committed,** and no test at the gate depends on a journey's page.

**Today** (section 2): a journey on main is a recording of every byte its client exchanged with the engine, and the pages printed from it. Any goal that changes the client's wire, or what a page asks and when, stales every recording, fails the default suite, and walks every journey again on a builder. Since 11-02, 101 commits on main rewrote a recording, and journey files are 6.7% of every line changed there.

**What goes** (section 8): the committed recordings, `again`, the placeholders and `normalise`, the fixed start, the `setup` file, and the test that holds main's pages to their recordings.

**Unchanged:** a page's printed form, the directories the pages live in, the look requests' branches, the recorder in both clients, the drive session and the recipes, and the gallery.

**What it costs:** a walk names controls by their words, so a goal that renames one changes the walks that hold it, where a recording held (3.2). And most of 4.1 is new work: the pieces exist, apart, inside test crates (survey, 2.1).

**On the way there** (8.3): look requests are drawn and landed all the while. The one writer comes first, walking boxes as today, and each journey moves to the world under it.

## 2. Today

From the survey, section 1:

- **What main holds:** four recordings and fourteen pages, 235 KB, all of it made by walking a fake box with headless Chromium or a pseudo-terminal. The sources, every `walk` and `setup` file together, are 99 lines.
- **A recording is the wire's bytes:** each turn's clocks and deadline, each ask, each answer. An ask names the wire's version, and jig's client wire went from version 19 to 34 between 11-10 and 11-18.
- **So every change to the wire stales every journey,** and so does a change to what a page asks or when. The print's test then fails (`../client/web/journeys.md`, 6), and the goal runs `again` for each journey.
- **A walk rewrites its files whole.** In "Name tasks waiting after a program starts without its tools", 6,015 of the landing's 6,630 changed lines are journey files, and the pages differ by a drawn time. Two goals that walked the same journey cannot merge their recordings: one walks again.
- **A walk needs what a plan may not have:** a built box, a browser, and Forgejo fetched from its host. On 11-18 that host's name did not resolve, and a goal removed six journeys in place of walking them. Main holds four of eight today.
- **The fixes so far remove noise, not the cause.** Placeholders (`../client/journey-placeholders.md`) and the fixed start (`../client/journey-clock.md`) make two walks of the same screens give the same bytes. A wire change still rewrites everything.
- **One walk, three languages:** the browser's `setup`, the TUI's `walk`, and the recipes' lines, which wrap both (survey, 1.6).

## 3. A journey is its walk

### 3.1 Its directory

`NAME/` under `temper/docs/design/client/web/journeys/` or `temper/docs/design/client/tui/journeys/`, as today:

- **`walk`:** 3.2's text. A source.
- **`README.md`:** what the journey walks and what each page shows. A source.
- **`scripts.json`,** where it has its own: the fake model's scripts for its steps, in the box's `--scripts` form, as a recipe's (`../client/recipe.md`, 2). A source.
- **Its pages,** `01.html` onwards, and for the TUI `01.txt` onwards: written by the writer alone (section 6).
- **No `recording` and no `setup`.**

A journey belongs to the client whose pages it prints. Its walk may act in the other one too: a TUI's sign-in is approved in the browser.

### 3.2 The file

Text, one record per line, in the drive session's form (`../client/drive.md`, 3.1):

| Line | What it is |
|---|---|
| `walk 2` | the first line: the format's version |
| `after NAME` | the second line, or absent: the journey NAME's lines come first (3.3) |
| `web press`, `web submit`, `web toggle`, `web input`, `web see`, `web reload` | the drive session's browser lines (`../client/drive.md`, 3.2) |
| `tui key`, `tui see`, `tui stop` | its TUI lines (`../client/drive.md`, 3.3) |
| `tui approve` | the browser opens the address the TUI last asked to browse, presses its "Approve", and loads again the address it was on |
| `web mark`, `tui mark` | prints that client's page as it is drawn then (3.4) |

`project-landing`, whose `setup` holds the first six lines today:

```text
walk 2
web press link "Set up temper"
web press link "Run on this machine"
web press button "Run on this machine"
web press button "Continue with ChatGPT"
web press button "Create your project"
web see "work/hello is in Hello"
web reload "/projects/1"
web see "Nothing needs you."
web mark
```

`sign-in-approved`:

```text
walk 2
tui see "then press Approve."
tui mark
tui approve
tui see "Setting temper up makes your first project"
tui mark
```

- **A control is found as a person finds it:** by its role, its exact accessible name and its order among those, never by the event it gives. A `textbox` is a text field or a text area, in a world as in a browser.
- **An act waits for its control** as a `see` waits for its text, up to the line's bound (`../client/drive.md`, 3.2).
- **Nothing is implied in a journey.** The TUI starts at the first `tui` line, not signed in; a walk that needs it signed in says `tui approve`. The browser starts on the page the printed link opens. A recipe's walk keeps its session's own sign-in (`../client/drive.md`, 3.3): the file's first line, `walk 2` or `recipe 1`, says which holds, and `tui approve` and the marks are a journey's lines alone.
- **No `ctl`, `forge` or `look` line.** A journey shows what a person does in a client. A state reached another way is a recipe's (`../client/recipe.md`).
- **A walk changes with what a person does:** a control renamed, a step added or gone. Whoever changes those words changes the walks that hold them, in the same change, as it changes a test, and prints them (section 7). A recording held through a rename (`../client/web/journeys.md`, 3.3); a walk does not, and that is the price of a source a person can read.
- **No walk changes for the wire, the exchange, the view's shape or the look.** Two goals that rename different controls of one journey change different lines of its walk, and merge as text. Two that change the same line meet a judgement, as in any source. No conflict in a walk is settled by printing.

### 3.3 `after`

- **`after NAME`** names a journey of either client. Its lines run first, its own `after` before them, and its marks print nothing.
- **A chain** holds at most eight, with no cycle, as a recipe's (`../client/recipe.md`, 3.2).
- **`scripts.json`** files join along the chain, as a recipe's parents' do.

### 3.4 Marks

- **A mark prints one page:** the client's page as drawn when the line is reached, every line before it having held. A walk says what it waits for with a `see` before its `mark`.
- **Pages are numbered** in the marks' order, per journey, from `01`.
- **Typing marks nothing,** and a walk prints exactly its marks: there is no last page without one.
- **Its link:** the first press, submit, toggle or key of the same client after a mark and before the next one links the marked page to the next page, where its control is on the marked page. A control that only typing or another act revealed is not on it: that page leads on by its neighbours, as the last page does.

## 4. Three places to run it

### 4.1 In a world

- **One process and nothing else:** no child, socket, clock, environment or network, and no file but the journeys it reads and the directory it writes.
- **What runs** is the playground's composition, which the one-process world and the playground's own page already run (`../testing/README.md`, 2.5): `temper-service` with its engine and one worker, its hosted agents over the fake model, the fake forge and the fake git program, on skein's simulator.
- **The browser's client** is the web shell's own host and its page. `temper/tests/playground` runs it against that service today, inside a test crate, with no reload, no tab and no HTML handed out (survey, 2.1). The print is what that host draws, never a view built aside.
- **The TUI's client** is `temper-client-shell-tui`'s host, given keys and read as frames with no terminal. Today it runs only joined to the engine's root, in the client's world (`../client/testing.md`), where typed requests and events pass with no service between. Carrying its requests, its stream, what it keeps, its deadlines and its browse to the simulated service is new work, and a goal of its own.
- **A walker gives the lines.** It finds a control through skein's fake person's faces (`../testing/fakes.md`, 4), by role, exact name and order, and adds what a walk needs beyond that person's steps: the wait for a control, a key, an approval, a mark. What is generic among those is skein's.
- **The world plays what a box would do,** each as a person would meet it, with no step taken through ctl:
  - **the box,** a peer of the world's that speaks the box's protocol to the service (`../protocol/setup/box.md`, 1) and reports an owned forge's set-up step by step, as the box reports it, on the simulator's time. The one-process world's peer answers a start alone today (survey, 2.1);
  - **a tab the page opens,** carried to the fake issuer and back, as a browser carries a sign-in's (`sign-in-new-tab.md`), so that an account is connected through the client;
  - **a reload,** the page started again at the address, with the sign-in ways its document names and what its small store kept;
  - **the address a TUI asks to browse,** given to the walk's browser.
- **Its facts are fixed,** so nothing a page draws varies: the seed 7, the wall 2026-01-06T09:00:00Z, the service at `127.0.0.1:10001` and the forge at `127.0.0.1:10002`, a TUI of 80 columns by 24 rows whose build reads `0.0.0 (000000000000)`. They are the values main's pages hold now (`../client/journey-placeholders.md`, 2; `../client/journey-clock.md`, 2).
- **The same bytes every time.** The simulator owns the schedule, so a walk takes the same turns on any host, on any day.
- **A walk that cannot go on stops,** naming its journey, its line and what the line sought. Its bounds are the drive session's figures, in the simulator's time (`../client/drive.md`, 3.2 and 3.3).

### 4.2 On a box

- **The drive session takes a journey's lines** on a fake box or a deployment, as it takes a recipe's (`../client/drive.md`, 1), with a journey's rule that nothing is implied (3.2). A `mark` there is a `look`: the page's accessible text, and a screenshot at the widths asked.
- **A recipe's walk is unchanged:** its sign-in, its `tui stop` between steps and checks, and what `recipe walk` does (`../client/recipe.md`, 5.1).
- **What it is for:** what only a browser or a terminal shows, the rendered look and a real program's set-up, after landing and outside every plan (`../client/web/look-requests.md`, 4).
- **It writes no page of main's.**

### 4.3 By hand, on a deployment

- **The recorder stays** in both clients, switched on as today (`../client/web/journeys.md`, 3.1; `../client/tui/journeys.md`, 2). A person records on a deployment or on a box of their own, and saves a file.
- **`temper-client-journeys print FILE OUT`** takes that file: it redacts it, proves the replay against the saved pages, scans it and prints its pages into OUT, as `add` does today (`../client/web/journey-redaction.md`; `../client/web/journeys.md`, 3.7 and 4.3). OUT is laid out as 5.2 has it, and is never under the repository's journeys.
- **The file is the person's,** never committed. Its pages may be drawn on as a hand-made page's are (`../client/web/look-requests.md`, 2).
- **Printed by the build that recorded it.** A recording holds one build's wire, so the tool that prints it is built at the deployment's commit. `print` says so, by the layout's name, when it meets another.

## 5. The print

### 5.1 A page

- **Its form is unchanged:** the browser's document, one tag per line, its stylesheets linked by their paths in the checkout, its ids numbered afresh (`../client/web/journeys.md`, 5.2 and 5.3); the TUI's text and themed page (`../client/tui/journeys.md`, 6).
- **Linked by the act** (3.4), where today every control that gives the recorded event is.
- **A journey's pages are replaced whole** by a walk that reached its last line: fewer marks leave fewer files. A walk that stopped leaves every page of its journey as it was.

### 5.2 The program

- **A small binary of its own,** `temper-client-walk`:
  - `temper-client-walk print [--out DIR] [NAME]...` prints the named journeys, or all, into DIR, a scratch directory it names when none is given;
  - `temper-client-walk pages` writes main's pages in place (section 6).
- **DIR is laid out as the tree is,** from `temper/` down: each journey's directory at its path, and the stylesheets its pages link, with their files, copied to theirs. So a page opens from DIR as it opens from the checkout, and its bytes are the ones `pages` would write.
- **Its exit:** 0; 1 when a walk stopped, once the others are printed; 2 for its usage.
- **It is a print, not a walk of a box:** it starts no process, as the gallery's print starts none (`../client/web/gallery.md`, 1), so a plan may run it.

### 5.3 Its code

What the binary needs sits today where it cannot be reached, so the first goal that prints moves it:

- **The lines' grammar and the two printers** leave `temper-client-journeys-tool`, which depends on the browser kit, for a library that does not. The drive session and the new binary both read lines through it.
- **The world's driver,** the web host on the simulated service with its stepping, leaves the playground's test crate for a library its tests and the binary share. The box's peer and the walker join it there.
- **The binary links** those two, the playground's composition and the two shells, and nothing of the box, the browser kit or the drive session.
- **Its tests' layers:** the grammar and the printers, step; a walk in a world, simulated (`../testing/README.md`, 2). Each within its layer's budget as it stands, with the cost of a journey's walk measured by the goal that first prints one, beside the one-process stories' half a CPU second each (survey, 2.4).

## 6. Main's pages have one writer

- **What it writes:** each journey's pages, whole (5.1), and the generated files beside them (`skein-ui.css`, `journey.js`, `theme.css`, `tui-journey.js`). It removes the pages of a directory that holds no `walk`.
- **One run at a time, on main's tip.** The writer prints the tip. While a change of its own is open it starts no other. When that change lands, or any other landing does, it runs again on the new tip. So it settles: once it has run on a tip and found nothing to write, main's pages are that tip's print.
- **Its change** touches pages alone and is opened from a pushed branch (`mechanical-changes.md`). The last paragraph of its message holds `Printed: COMMIT`, the tip it printed.
- **Nothing else writes its files, so its change meets no conflict.** Were one to arise all the same, from a branch made by hand, fabrig's merge job settles it with no model, taking the landing branch's copy (`tools/merge/README.md`), and the next run writes the pages again.
- **It rules by writing, not by guarding.** Whatever lands in a page, the next run replaces it with the tip's print. So no check has to know who wrote a page.
- **Goals are kept off all the same,** against the habit: the delivery check (`tools/pre-pr/design.md`) refuses an agent's delivery whose branch changes a page, or a generated file beside one, together with any other file, naming `temper-client-walk print`. `AGENTS.md` says so.
- **By code alone:** no model, no judgement. Its runner is the after-landing check's owner, by a script, at each landing (section 12).
- **Its engine** is `temper-client-walk pages` for a journey that prints in a world, and `again` with the reprint for one that has not moved yet (8.3).
- **A page holds no stamp:** no commit, no date. A landing that changes no page changes no file.
- **How far behind.** Between a landing and the writer's change, main's pages are an earlier tip's, and the last `Printed:` on main says which. Nothing reads them in between but a person.
- **A walk that stops** leaves its pages as they were, and the run fails, naming the journey and the line, on that tip and on every later one until a goal repairs the walk or the client: one after-landing red, its owner the after-landing check's. A new journey whose walk stops has no page until then.
- **The overlays stay empty on main:** the delivery check refuses a branch whose `overlay.css` holds anything, as the print's test does today (`../client/web/journeys.md`, 6).
- **A new journey** lands as its `walk` and README. Its pages follow, from the writer.

## 7. Checking work against a request

- **A request is drawn as today:** a commit on the person's `look-requests` or `tui-requests` branch, its diff the pages' changes and the overlay's rules, against main's pages (`../client/web/look-requests.md`, 2).
- **`look-refresh.sh` goes by the print.** It says which commit main's pages were printed at, and how many landings follow it. It drops a request only once its pages are on main: when the commit carrying its trailer is the last `Printed:` commit, or an ancestor of it. Until then the request stays on its branch, on the pages main still holds, and a later request stays drawn on it.
- **Whoever lands it prints into scratch,** `temper-client-walk print --out DIR`, on its own tree, and compares DIR's pages with the request's, read from the request's commit. They are equal where their facts agree. It needs no box, and nothing it compares can be stale.
- **It commits no page.** After its landing the writer writes them.
- **For a journey not yet moved,** the print into scratch is the reprint in the goal's own tree (8.3).
- **The render check follows the writer's change:** the request's pages under their overlay against main's under the landed stylesheets, in a browser (`../client/web/look-requests.md`, 4).
- **A state no journey reaches** is the gallery's (`../client/web/gallery.md`), a hand-made page's, or a page printed from a person's own recording (4.3).

## 8. What goes, what stays

### 8.1 Goes

- **Every committed `recording` and `setup`,** and `add` as the way a journey reaches main.
- **`again`,** the browser's and the TUI's, with its walk's check against the old one, its prepared inputs and its scratch (`../client/web/journeys.md`, 8; `../client/tui/journeys.md`, 8).
- **Placeholders and `normalise`** (`../client/journey-placeholders.md`): nothing varies in a world, and nothing recorded is compared.
- **The fixed start** in the box, the page and the TUI (`../client/journey-clock.md`): `again` was its only user.
- **The test that reprints main's journeys** and holds each recording to its normalisation, with the gate's entries that name a committed recording.
- **Stale,** as a state of main: no committed file depends on the wire.
- **A real journey on main** (`real-journeys.md`, 5): a person's recording prints where they choose (4.3).

### 8.2 Stays

- **The recorder, the replay and the print of a recorded file,** for 4.3, with redaction and the last scan. The drive session reads the TUI's frames through them (`../client/drive.md`, 3.3).
- **The drive session and the recipes,** QA's, with a recipe's walk as it is.
- **The gallery,** for the states no journey reaches.
- **The look requests,** but for their refresh and step 6 of their landing (section 7).
- **The scans that read every tracked file,** the outward check among them: they read a page as any text.

### 8.3 Until the last journey has moved

Look requests, the browser's and the TUI's, are drawn and landed all the while (the user, 2026-11-18). So the writer comes first, by the means there are, and the journeys move under it one at a time.

- **First, one writer, with `again` as its engine.** The gate's test goes, and with it every goal's part in a journey's files. From that landing the writer of section 6 runs on main's tip: it walks again, on a fake box, each journey whose recording no longer replays, reprints them all, and offers what differs, recordings and pages, as its change. Sections 6 and 7 hold from then: one run at a time, `Printed:`, the refresh going by the print, and the delivery check keeping goals off every journey file.
- **A request is drawn on main's pages, as today,** and the writer keeps them current after each landing.
- **Whoever lands a request** reprints in its own tree and compares with the request's pages. A change to the look leaves a recording replaying (`../client/web/journeys.md`, 8), so the reprint holds wherever main's recording does. Where the writer is still behind a change to the wire, that journey's comparison is the render check's. It commits no journey file.
- **A journey moves alone:** in the change that gives it a `walk` of version 2 that prints in a world, its `recording` and `setup` go. From that landing the writer prints it in the world, and whoever lands a request prints it into scratch (section 7).
- **The writer knows a moved journey** by its `walk` of version 2, and takes the other engine for every other directory.
- **What a box cannot walk today** stays as it is until its journey moves, or is repaired sooner on its own: the TUI's `design-session`, `launchpad-chat` and `launchpad-drawn`, and `chat-draft`, have no pages on main. A request for a state they would show is drawn on a hand-made page or the gallery's, as the look requests allow (`../client/web/look-requests.md`, 2).
- **A new journey** is written as a walk of version 2 once its client walks in a world. Before that it is walked and added as today, by the writer's runner alone.

## 9. Upgrades

Nothing stored and no wire changes. A `walk` file's version is 2 from the change that moves its journey (8.3); `again` reads version 1 alone and refuses another by name, until it goes. A recorded file stays `journey 2`.

## 10. Its tests

Focused, and native: no box, browser or terminal.

- **The walk's reader:** each line read back; `after` joining a chain, and a cycle, a ninth and a missing name refused; a `ctl` line refused; `tui approve` and a mark refused in a recipe.
- **A world's walk,** on small journeys of the tests' own: walked and printed; printed twice, the same bytes; a control that appears late found within its bound, and one that never does stopping the walk by its journey and line; a text area found as a `textbox`; a reload keeping the small store and the sign-in's ways; a tab carried to the issuer and back; an owned set-up's steps drawn as the box's peer reports them; `tui approve` approving what the TUI browsed, and a TUI not approved staying on its approval.
- **The print:** the acted control linked, by act; a control that typing revealed not linked; fewer marks leaving fewer files; a walk stopped at its last line leaving every page.
- **`pages`:** on a tree whose pages are current, nothing written; a word of the view changed, exactly the pages that draw it written; a directory with no `walk`, its pages removed.
- **The delivery check:** a delivery changing a page and another file refused, with and without a base to compare; a nonempty overlay refused.
- **The refresh:** a request whose trailer is on main and not yet printed kept, and dropped once a `Printed:` commit covers it.
- **The recorded file's tests keep their own data:** redaction, the scan and the proof build their recordings in the host's world.
- **No test's verdict depends on a page or a recording of main's:** none replays, reprints or compares one. The box's walks are written and compiled, never run in a plan.

## 11. Later

- **temper runs the writer:** a command the project names, run on a landing branch's new tip as `close` is run on a goal's (`../protocol/tooling.md`, 4.2), its staged files landing as a change of their own, one at a time. The script of section 6 then goes.
- **A journey after a recipe,** so that a start QA already keeps is not walked again through the client.
- **The gallery's states as walks,** once a world reaches them, their answers then the engine's own.
- **The playground's `story-a.log`** (`temper/tests/playground/tests/fixtures/`), another recording on main that the merge job makes again (`tools/merge/README.md`): the same rule would have a test make it and no file keep it.
- **A recorded file printed across layouts,** by the plans of `layouts-both-ways.md`, 4, so that 4.3 needs no build of the deployment's commit.
- **The box's `mark` compared with the world's page,** by their accessible text, as a check that a world draws what a box does.

## 12. Open questions

- **The writer inside the landing.** It could run on the candidate a landing gates, so that main's pages are never behind. Default: after landing, as the user asked for the check, since a walk that stops must not hold a landing.
- **Who runs the writer.** Default: the after-landing check's owner, by a script that holds section 6's rule of one run at a time. temper running it is section 11's.
- **A test that walks every journey to its end,** in the simulated layer. It would tell a goal that renamed a control before it lands, where section 6 tells it after. Default: none, as the user asked; its cost is known once a journey's walk is measured (5.3).
- **The fixed start, landed on 11-18.** Default: it goes with `again` (8.1), once the last journey has moved; until then it keeps the writer's walks of a box the same from run to run (8.3). Kept past that, it would do the same for the box's walks of 4.2.
- **A person's recording on main.** Default: never; its pages are drawn on from wherever they were printed (4.3).
- **The binary's name.** Default: `temper-client-walk`, beside `temper-client-gallery` and `temper-client-journeys`.
