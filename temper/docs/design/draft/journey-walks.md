# journey-walks: a journey is its walk, printed in a world, and main's pages have one writer

Draft, 2026-11-18, from a session with the user. They asked for "an assessment of the situation and how we could turn it into a nice clean story", having seen the journeys conflict between unrelated goals, check too much in, be recorded again without need and run slowly. Their direction, the same day: the printed pages may stay on main "with the constraint that only the writer writes it"; "recording / replay should be as cheap as possible according to the context", so that an agent answering "does what I write match the diff/html in the request" does it "without a full blown process (box or whatever)", through "a world-level client with a renderer"; and a check after landing is enough. The figures are the survey's, on main at `7307b6172b` (`temper/docs/development/journeys-survey-2026-11-18.md`).

It is the journeys' part of what `codec-shape.md` does for generated code, which leaves recordings aside (`codec-shape.md`, 5.3). Where they differ, this draft wins over the browser's journeys (`../client/web/journeys.md`, sections 3 to 8), the TUI's (`../client/tui/journeys.md`, sections 3 to 8), `../client/journey-clock.md`, `../client/journey-placeholders.md`, the look requests' steps (`../client/web/look-requests.md`, 2 and 3) and the drafts `journey-clock.md`, `journey-placeholders.md`, `real-journeys.md` and `record-anywhere.md`, until its goals fold it into them.

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
- **No recording is committed,** and no test at the gate reads a journey's page.

**Today** (section 2): a journey on main is a recording of every byte its client exchanged with the engine, and the pages printed from it. Any goal that changes the client's wire, or what a page asks and when, stales every recording, fails the default suite, and walks every journey again on a builder. Since 11-02, 101 commits on main rewrote a recording, and journey files are 6.7% of every line changed there.

**What goes** (section 8): the committed recordings, `again`, the placeholders and `normalise`, the fixed start, the `setup` file, and the test that holds main's pages to their recordings.

**Unchanged:** a page's printed form, the directories the pages live in, the look requests' branches, the recorder in both clients, the drive session and the recipes, and the gallery.

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
| `tui approve` | the browser opens the address the TUI last asked to browse and presses its "Approve" |
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

- **A control is found as a person finds it:** by its role, its accessible name and its order among those, never by the event it gives. A `textbox` is a text field or a text area, in a world as in a browser.
- **Nothing is implied.** The TUI starts at the first `tui` line, not signed in; a walk that needs it signed in says `tui approve`. The browser starts on the page the printed link opens.
- **No `ctl`, `forge` or `look` line.** A journey shows what a person does in a client. A state reached another way is a recipe's (`../client/recipe.md`).
- **A walk changes with what a person does:** a control renamed, a step added or gone. Whoever changes those words changes the walks that hold them, in the same change, as it changes a test. No walk changes for the wire, the exchange, the view's shape or the look.

### 3.3 `after`

- **`after NAME`** names a journey of either client. Its lines run first, its own `after` before them, and its marks print nothing.
- **A chain** holds at most eight, with no cycle, as a recipe's (`../client/recipe.md`, 3).
- **`scripts.json`** files join along the chain, as a recipe's parents' do.

### 3.4 Marks

- **A mark prints one page:** the client's page as drawn when the line is reached, every line before it having held. A walk says what it waits for with a `see` before its `mark`.
- **Pages are numbered** in the marks' order, per journey, from `01`.
- **Typing marks nothing,** and a walk prints exactly its marks: there is no last page without one.

## 4. Three places to run it

### 4.1 In a world

- **One process and nothing else:** no child, socket, clock, environment or network, and no file but the journeys it reads and the directory it writes.
- **What runs** is the playground's composition, which the one-process world and the playground's own page already run (`../testing/README.md`, 2.5): `temper-service` with its engine and one worker, its hosted agents over the fake model, the fake forge and the fake git program, on skein's simulator.
- **The browser's client** is the web shell's own host and its page, as `temper/tests/playground` runs it against that service today (survey, 2.1). **The TUI's** is `temper-client-shell-tui`'s host, given keys and read as frames with no terminal, as the client's joint world runs it (`../client/testing.md`). The print is what those hosts draw, never a view built aside.
- **A person** is skein's fake person in front of each (`../testing/fakes.md`, 4), given the walk's lines.
- **The world plays what a box would do:**
  - an owned forge's set-up, its steps reported as the box reports them (`../protocol/setup/box.md`, 1);
  - a tab the page opens, carried to the fake issuer and back, as a browser carries a sign-in's (`sign-in-new-tab.md`);
  - a reload, the page started again at the address with what its small store kept;
  - the address a TUI asks to browse, given to the walk's browser.
- **Its facts are fixed,** so nothing a page draws varies: the seed 7, the wall 2026-01-06T09:00:00Z, the service at `127.0.0.1:10001` and the forge at `127.0.0.1:10002`, a TUI of 80 columns by 24 rows whose build reads `0.0.0 (000000000000)`. They are the values main's pages hold now (`../client/journey-placeholders.md`, 2; `../client/journey-clock.md`, 2).
- **The same bytes every time.** The simulator owns the schedule, so a walk takes the same turns on any host, on any day.
- **A walk that cannot go on stops,** naming its journey, its line and what the line sought. Its bounds are the simulator's time, the drive session's figures (`../client/drive.md`, 3.2 and 3.3).

### 4.2 On a box

- **The drive session takes the same lines** on a fake box or a deployment, as it takes a recipe's today (`../client/drive.md`, 1). A `mark` there is a `look`: the page's accessible text, and a screenshot at the widths asked.
- **What it is for:** what only a browser or a terminal shows, the rendered look and a real program's set-up, after landing and outside every plan (`../client/web/look-requests.md`, 4).
- **It writes no page of main's.**

### 4.3 By hand, on a deployment

- **The recorder stays** in both clients, switched on as today (`../client/web/journeys.md`, 3.1; `../client/tui/journeys.md`, 2). A person records on a deployment or on a box of their own, and saves a file.
- **`print FILE OUT`** takes that file: it redacts it, proves the replay against the saved pages, scans it and prints its pages into OUT, as `add` does today (`../client/web/journey-redaction.md`; `../client/web/journeys.md`, 3.7 and 4.3). OUT is any directory but a journeys' one.
- **The file is the person's,** never committed. Its pages may be drawn on as a hand-made page's are (`../client/web/look-requests.md`, 2).
- **Printed by the build that recorded it.** A recording holds one build's wire, so the tool that prints it is built at the deployment's commit. `print` says so, by the layout's name, when it meets another.

## 5. The print

- **A page's form is unchanged:** the browser's document, one tag per line, its stylesheets linked from the checkout, its ids numbered afresh (`../client/web/journeys.md`, 5.2 and 5.3); the TUI's text and themed page (`../client/tui/journeys.md`, 6).
- **Linked by the act:** the control the next line acted on carries the next page's name. A page whose next line is no act leads on by its neighbours.
- **The program** is a small binary of its own, `temper-client-walk`, which links the world and the two hosts and nothing of the box, the browser kit or the drive session:
  - `temper-client-walk print [--out DIR] [NAME]...` prints the named journeys, or all, into DIR, a scratch directory it names when none is given;
  - `temper-client-walk pages` writes main's pages in place (section 6).
- **Its exit:** 0; 1 when a walk stopped, once the others are printed, a stopped journey's pages left as they were; 2 for its usage.
- **Its cost is measured** by the first goal that prints a journey, beside the one-process stories' half a CPU second each (survey, 2.4). Its tests keep within their layers' budgets as they stand.

## 6. Main's pages have one writer

- **What it writes:** each journey's pages, and the generated files beside them (`skein-ui.css`, `journey.js`, `theme.css`, `tui-journey.js`). It removes the pages of a directory that holds no `walk`.
- **When:** after a landing on main, on main's tip. Where a file differs, the difference lands as a change of its own that touches nothing else, opened from a pushed branch (`mechanical-changes.md`), its title naming the commit printed. Where none differs, nothing happens, so the writer's own landing ends the round.
- **By code alone:** no model, no judgement. Who runs it on a deployment is that deployment's.
- **A page holds no stamp:** no commit, no date. A landing that changes no page changes no file.
- **Behind by one run at most.** Between a landing and the writer's, main's pages are the last print's. Nothing reads them in between but a person.
- **A walk that stops** leaves its pages as they were. The writer names the journey and the line: an after-landing finding, repaired by a goal as any red.
- **No goal writes a page.** The delivery check (`tools/pre-pr/design.md`) refuses a branch that changes a page, or a generated file beside one, together with any other file, naming `temper-client-walk print`. `AGENTS.md` says so.
- **The overlays stay empty on main:** the same check refuses a branch whose `overlay.css` holds anything, as the print's test does today (`../client/web/journeys.md`, 6).
- **A new journey** lands as its `walk` and README. Its pages follow, from the writer.

## 7. Checking work against a request

- **A request is drawn as today:** a commit on the person's `look-requests` or `tui-requests` branch, its diff the pages' changes and the overlay's rules, against main's pages (`../client/web/look-requests.md`, 2). `look-refresh.sh` keeps the branch on main as it does.
- **Whoever lands it prints into scratch,** `temper-client-walk print --out DIR`, on its own tree, and compares DIR's pages with the request's, read from the request's commit. They are equal where their facts agree. It needs no box, and nothing it compares can be stale.
- **It commits no page.** After its landing the writer writes them, and the request leaves its branch as today.
- **The render check is unchanged:** after landing, the request's pages under their overlay against main's under the landed stylesheets, in a browser (`../client/web/look-requests.md`, 4).
- **A state no journey reaches** is the gallery's (`../client/web/gallery.md`), a hand-made page's, or a page printed from a person's own recording (4.3).

## 8. What goes, what stays

**Goes:**

- **Every committed `recording` and `setup`,** and `add` as the way a journey reaches main.
- **`again`,** the browser's and the TUI's, with its walk's check against the old one, its prepared inputs and its scratch (`../client/web/journeys.md`, 8; `../client/tui/journeys.md`, 8).
- **Placeholders and `normalise`** (`../client/journey-placeholders.md`): nothing varies in a world, and nothing recorded is compared.
- **The fixed start** in the box, the page and the TUI (`../client/journey-clock.md`): `again` was its only user.
- **The test that reprints main's journeys** and holds each recording to its normalisation, with the gate's entries that name a committed recording.
- **Stale,** as a state of main: no committed file depends on the wire.
- **A real journey on main** (`real-journeys.md`, 5): a person's recording prints where they choose (4.3).

**Stays:**

- **The recorder, the replay and the print of a recorded file,** for 4.3, with redaction and the last scan. The drive session reads the TUI's frames through them (`../client/drive.md`, 3.3).
- **The drive session and the recipes,** QA's, which now take a journey's lines too.
- **The gallery,** for the states no journey reaches.
- **The look requests,** but for step 6 of their landing (section 7).

## 9. Upgrades

Nothing stored and no wire changes. The `walk` file's version moves from the TUI's 1 to 2, and each walk on main is rewritten in the change that moves it. A recorded file stays `journey 2`.

## 10. Its tests

Focused, and native: no box, browser or terminal.

- **The walk's reader:** each line read back; `after` joining a chain, and a cycle, a ninth and a missing name refused; a `ctl` line refused.
- **A world's walk:** a small journey of the test's own walked and printed; printed twice, the same bytes; a line that finds no control stopping the walk by its journey and line; a reload keeping the small store; a tab carried to the issuer and back; `tui approve` approving what the TUI browsed.
- **The print:** the acted control linked, by act; a mark with no act leading on by its neighbours.
- **`pages`:** on a tree whose pages are current, nothing written; a word of the view changed, exactly the pages that draw it written; a directory with no `walk`, its pages removed.
- **The delivery check:** a branch changing a page and another file refused; one changing pages alone passed; a nonempty overlay refused.
- **The recorded file's tests keep their own data:** redaction, the scan and the proof build their recordings in the host's world, and read none from the repository.
- **No test in either suite reads a page of main's.** The box's walks are written and compiled, never run in a plan.

## 11. Later

- **A journey after a recipe,** so that a start QA already keeps is not walked again through the client.
- **The gallery's states as walks,** once a world reaches them, their answers then the engine's own.
- **The playground's `story-a.log`** (`temper/tests/playground/tests/fixtures/`), another recording on main that the merge job makes again (`tools/merge/README.md`): the same rule would have a test make it and no file keep it.
- **A recorded file printed across layouts,** by the plans of `layouts-both-ways.md`, 4, so that 4.3 needs no build of the deployment's commit.
- **The box's `mark` compared with the world's page,** by their accessible text, as a check that a world draws what a box does.

## 12. Open questions

- **The writer inside the landing.** It could run on the candidate a landing gates, so that main's pages are never behind. Default: after landing, as the user asked for the check, since a walk that stops must not hold a landing.
- **The fixed start, landed on 11-18.** Default: it goes with `again` (section 8). Kept, it would give the box's walks of 4.2 the same times on every run.
- **A person's recording on main.** Default: never; its pages are drawn on from wherever they were printed (4.3).
- **The binary's name.** Default: `temper-client-walk`, beside `temper-client-gallery` and `temper-client-journeys`.
