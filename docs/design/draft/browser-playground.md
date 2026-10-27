# The playground in the browser

Draft, 2026-10-26, from a session with the user. What the playground in
the browser is, what it is built from, its bar for v0.1.0, and the order
of its work. It takes over `first-release.md`, section 8, which retires
with it, and the investigation of 2026-10-23 behind that section (Codex,
with builds; `~/src/rust/tmp/roadmap-3/playground-wasm.md`), checked
against the repositories on 10-26. It is one of `v0.1.0.md`'s parts
(its section 5): built beside the ladder, its first story rung A's. It
crosses skein, smith and temper on purpose; an owner splits it. Nothing
here is adopted.

## 1. In one page

- **The box's playground, with no box.** The playground a person meets
  in `temper box`, "Try the playground", with its Scripted model, its
  Playground forge and `playground/widget` (`first-run.md`, 2.3), is
  served as static files and runs whole in the visitor's browser:
  nothing to install, no account, no server. temper's engine and
  worker, jig's store, smith's agents and the fake LLM, forge and git
  run on skein's simulator in a Web Worker, and temper's web client
  runs on the page.
- **Product code on fakes, with no fake branch** (`fake-backends.md`,
  section 1). The engine, the worker, the store, the agents and the
  client are the shipped crates. Only the composition differs: the
  simulator's machine for the operating system, an in-page link for
  the network, and documents of its own for the configuration.
- **Two modules** (section 3): the client on the page and the world in
  a Web Worker. A bridge carries the client's HTTP exchanges and its
  stream to the world's clients' listener, through jig's mapping,
  cookie and all.
- **A world that lives while the page does** (section 4): stepped a
  slice at a time, its clock paced to the browser's with pause and
  speed, waiting for the visitor when nothing is due. The page shows
  the world's time.
- **Reproducible:** the world's draws are its seed's, and the visitor's
  inputs enter between slices, stamped with the simulator's time. The
  seed, the build and the inputs replay a session exactly, natively, as
  a test. A failure offers them as a file (4.5).
- **Stories are data** (section 5): the box's three requests first,
  with the box's scripts, which rung A walks on real backends; then
  rungs B0 to B2 as their passing live transcripts become scripts.
  Off-script, the scripted model says what it is and what it answers.
  There is never a real model.
- **Bounded for a phone** (section 6): capacities just big enough for
  its stories, and the harness's trace and captures left out. Its bar
  is a phone's browser.
- **Nothing leaves the browser.** Once loaded, the page makes no
  request: no model, no analytics, no telemetry. A failure's log goes
  where the visitor chooses to send it (section 7).
- **Its seams are shared** (section 8): skein's shell and harness split
  by platform, which macOS's port needs too (`macos.md`, 3.5); smith's
  agent apart from its native start; the fake machine and the
  simulated runner shipped as product.
- **Its bar for v0.1.0** (section 10): rung A's story walked in the
  browser, on desktop browsers and on a phone, within its budgets,
  published with the release.

## 2. What the visitor sees

- **A loading line,** then the client as the box's person sees it after
  following the printed link: signed in as the deployment's owner, on
  the welcome. The visitor chooses "Try the playground", and the demo's
  start makes the project Playground, adds the Scripted model and
  adopts `playground/widget` (`../protocol/setup.md`, 8.4). From there,
  the box's playground: the banner, the three suggested requests, a
  change followed from a branch to `main`.
- **Only what the deployment can do is offered.** The playground's
  document configures the demo alone, with no other forge and no other
  model provider, so the client offers no Set up temper, no Connect a
  forge and no Connect a model. Where the client still offers them, the
  fix is the client's, for every deployment that configures them out,
  never a branch for the playground (13).
- **A frame around the client,** which is the page's own and not the
  client's: what this is ("temper's playground: a scripted model and
  forge, running in your browser"), the world's controls (pause, speed,
  reset, the story), temper's version, and the way to the real thing,
  the download. The frame has temper's look and its own mockup, chosen
  by the user as every page is (`v0.1.0.md`, section 5, the UI).
- **A pace a person can follow.** A story's model replies and its CI
  take time, set in the story's documents (5.1), so its delegates'
  phases show at 1×; the speed control shortens them.
- **A reload starts over** (4.4).

## 3. The two modules

```
the page                                          a Web Worker
  the frame: pause, speed, reset, story             the world, on skein's simulator
  temper's web client                                 temper-service: the engine and one worker
    jig's adapter: exchanges, stream                  jig's store on skein-kv, the fake disk
    the playground's platform ── the bridge ──► the visitor: HTTP on a simulated socket,
                                                      its cookie jar, to the clients' listener
                                                    smith's agents, hosted programs
                                                    the fake LLM, forge and git
```

- **The client is the shipped one:** temper's host and pages, and jig's
  adapter, which already hands a step's exchanges and stream to skein's
  transport as plain data (jig's `protocol/browser.md`, 10). The
  playground's platform carries them over the bridge where the shipped
  bundle's carries them by fetch (skein's `ui-web.md`, section 10). The
  write-before-send rule stays the transport's (jig's
  `protocol/browser.md`, 5.3). The client's small store is the page's
  memory, not session storage, since a reload starts a new world.
- **The bridge carries plain data,** one message per item: an exchange
  and its answer; the stream's opening, its pieces, read on as the
  client asks and not before, and its end; an abandon; the frame's
  controls; and the world's clock with each message (4.3).
- **In the world, the visitor** is a program of the simulated machine.
  For each exchange, and for the stream, it opens a simulated
  connection to the clients' listener and speaks HTTP with skein's
  client. It sends the Host the deployment names, and it keeps the
  cookie the sign-in sets in a jar of one, adding it as a browser does.
  The page never holds the credential, which is the HTTP-only promise
  kept.
- **HTTP through the listener, not a direct transport:** the playground
  runs the mapping, the guards, the cookie and the sign-in a deployment
  runs, rather than going round them. This costs a jar and an HTTP
  client in the world, and skein's HTTP is already there.
- **Signed in by the printed link.** The world's `temper-service`
  prints its link on its stdout, as `temper serve` does, and the
  bridge brings it to the page. The page's starter puts it in the
  address and starts the client, whose sign-in part spends it as a
  person's browser does (jig's `protocol/browser.md`, section 11). No
  credential is made by hand.
- **Why two modules:** the world steps without holding the page's
  turns, and a reset is the worker ended and started again. One module
  would share more code under LTO but would budget the world's slices
  on the page's thread. No threads and no shared memory are needed,
  so the page needs no cross-origin isolation.

## 4. The world

### 4.1 Composed from the one-process world

- **What runs** is the one-process world's composition
  (`../domain/testing.md`, 2.5) without its harness: `temper-service`'s
  engine and one worker in one `iterate`, started from configuration
  bytes; jig's store on skein-kv over the fake disk; smith's agents as
  hosted programs; the fake LLM and fake forge as services on
  simulated sockets; the fake git program; and the visitor (section 3).
  There is no referee, no fake person, no terminal client, no captures
  and no fault schedule.
- **One composition, two users.** It moves out of `tests/one-process`
  into a crate of temper's, `temper-playground` (provisional), built
  natively and for wasm32. The one-process world composes it with its
  harness, so that the gate runs the world the browser runs.
- **No box.** The fakes are the runner's, as for a deployment without
  the box (`../protocol/setup.md`, 7.4), and the demo's start asks
  nothing of a pipe.
- **Calm,** at every layer: no fault in the simulator, the fake LLM or
  the fake forge. Every seed derives from the world's one seed; today
  the fixtures' seeds are separate.

### 4.2 Stepped, paced, waiting

- **A slice at a time.** skein's world gains a runner that steps to the
  next due event, or to a slice's budget of work, and yields to the
  worker's loop. Today's run consumes the world and runs it to
  settlement, failing after a million steps or when it is idle with
  nothing due (skein-world's `World::run`). Those are a test's verdicts
  and not a page's.
- **Paced by the browser's clock.** The simulator advances to its next
  deadline only as far as browser time allows, times the speed (0
  while paused). skein's simulator has the parts (`next_due`,
  `advance_to`).
- **Idle is waiting:** with nothing due, the worker sleeps until the
  bridge brings input or the next deadline comes.
- **A hidden tab pauses the world;** shown, it resumes where the
  visitor left it.
- **The trace is bounded,** or off. The simulator keeps every entry
  today, until the world is dropped.

### 4.3 One clock

- **The page's clock is the world's.** The client's host takes its
  `now` and its wall time from the playground's platform. The platform
  gives the world's clock, as the bridge last brought it, advanced at
  the current speed between messages, so the client's times and
  deadlines are the world's at any speed.
- **The world starts at the browser's time,** which is recorded with
  the session's inputs (4.5).

### 4.4 One life per page

- **In memory only.** A reload, a reset or a new story ends the worker
  and starts a fresh world from its documents, and nothing is saved.
  Saving across a reload is later (section 12).
- **A panic in either module is fail-stop.** The page says the
  playground failed, offers the session's log (4.5) and a start over,
  and does not reload by itself as skein's restart part would.

### 4.5 Reproducible

- **The world is a function of its inputs:** the build, the seed, its
  start time, and each bridge input with the simulated time at which
  it entered, always between slices. The frame's pace changes only how
  fast simulated time passes, never what happens in it.
- **The session's log** keeps those inputs, bounded, in the page's
  memory. On a failure, or by a key in the frame, the page offers it as
  a file.
- **Replayed natively** by `temper-playground`, a log is a test: the
  world's referee runs over it, and the bug is in the lowest world from
  the start. This is the simulator's promise shown to whoever finds a
  bug, and the playground's only way of hearing of its failures
  (section 7).

## 5. Stories

### 5.1 What a story is

- **Data, no code:** the deployment's document, which is the demo's
  part with the forge, the provider and the project; the fake LLM's and
  the fake forge's documents (`../protocol/setup.md`, 8.4, "From a
  document of its own"), which hold the scripts and the seeded
  repositories with their files, CI and cues; its pace, the latencies
  of the model's replies and of CI, which those documents gain, since
  the box's fakes have none; and a description of what the visitor is
  expected to do.
- **One world per story.** A story's scripts rely on its repository's
  state, so that the CSV edit, for one, applies once. Choosing a story
  in the frame starts a fresh world on its documents, and reset starts
  the same story again.
- **Checked as data,** in the gate, as `temper-init`'s scripts are:
  each cue matched apart, within the fakes' bounds. Each story is also
  walked whole in the playground's world (section 9).

### 5.2 The first

- **The box's playground** (`../protocol/setup.md`, 8.4): three
  suggested requests. "Add CSV export to widget" is delegated, edited,
  pushed, passed by CI and landed. The dark theme and the empty search
  are answered in text.
- **One source for its scripts:** `temper-init`'s template, which the
  box writes. The playground's story is derived from it, its
  `otherwise` worded for the browser (5.4), so the box and the browser
  tell one story. Rung A walks the same requests on real
  backends (`v0.1.0.md`, 2.1).

### 5.3 From the ladder

- **B0 to B2 become stories as their rungs pass** (`v0.1.0.md`, 2.1):
  a question the delegate asks and the visitor answers (B0), a first
  push that fails CI and its repair (B1), and two delegates, one
  depending on the other, with an advisor's report (B2).
- **One transcript, three uses.** A rung's passing live transcript
  becomes a script of the fake LLM (`qa-live.md`, section 8). It serves
  as the rung's fake twin in the worlds, as the playground's story, and
  as the help's example of that story.
- **A story is added with no code,** as data. None is in the bar but
  A's.
- **The visitor's answer** in B0 is free text. The delegate's script
  goes on whatever it says, or branches on cue words by skein's turn
  rules (`inputs`), which the fakes' own documents allow.

### 5.4 Off-script

- **The scripted model says what it is:** the playground's, answering
  only the story's requests, which it names again. It says a real model
  comes with the download. This is the box's `otherwise`, worded for
  the browser.
- **Agents never go off-script:** each is cued by the brief a script
  gave it.

## 6. Bounded for a phone

- **Measured on 10-23:** the world's linear memory grew to 548 MB at
  its first step and to 627 MB and 671 MB through its stories. That is
  reserved memory, not a profile, with the fixtures' memory bound at
  2^60 on purpose and the simulator's trace kept whole.
- **The playground's documents bound it:** the engine's memory bound,
  jig's capacities, smith's windows and the fakes' limits, each just
  big enough for the stories. The trace is bounded and no captures are
  kept.
- **First,** how much of the 548 MB those control, and what else
  reserves it. Then the bar, held for every story at its high water.
  Default: 256 MiB of linear memory (13).
- **The download,** measured on 10-23 with `gzip -9`: the world 5.2 MB
  with the harness in it, the client 0.9 MB. Brotli and leaving the
  harness out shrink both. Default bar: 6 MB compressed for a first
  load, then cached (13).
- **The first paint does not wait for the world:** the frame and the
  loading line come first, and the world's module compiles as it
  streams, in the worker.

## 7. Served

- **Static files:** the page, its starter, the two modules and their
  glue, the stylesheets and the fonts, each but the page under its
  digest's name and cached for good. The page itself has a short
  lifetime.
- **At the site's playground address,** `playground.fabrig.ai` in the
  names `first-release.md` proposed, which are open until the site
  launches. Any path answers with the page, which always starts at the
  printed link.
- **The page's policy** is temper's strict one (`../client/browser.md`,
  2.2), with the worker allowed from the origin and no `connect-src`
  beyond it. After the load, nothing is fetched.
- **Built by `scripts/bundle.sh --playground OUT`:** temper's web shell
  with its `playground` feature (the bridge's platform, the frame and
  the starter), as `--record` builds the recorder, and
  `temper-playground` for wasm32, in the `web` profile.
- **Released with temper:** built at each release tag from the commit
  the download is built from, and linked from the site and the help.
  Between releases, a preview of `main` is kept private, for the user
  and QA.
- **Nothing is collected.** No analytics, no error reporting, no
  request after the load. A log reaches us only as a file a visitor
  attaches to an issue on the forge.

## 8. The seams, and who owns them

skein ships the primitives and temper wires them (`fake-backends.md`,
section 1). Each seam with where it stands on 10-26:

| Seam | Owner | Stands, 10-26 | Shared with |
|---|---|---|---|
| The shell split: a portable `Host` apart from the native ring, clock, seed, terminal and settings | skein | the readiness backend landed (skein's `shell.md`, section 7); no split, and a compile guard now refuses every target but Linux and Apple Silicon | macOS (`macos.md`, 3.5) |
| The harness split: the simulated runner and its program registry apart from the real loop, the ring and the tree keeper | skein | unchanged: all four in `skein-world`, unconditionally | macOS (3.5) |
| Shipped as product: the fake machine, the simulated runner, and what `skein-fake-peers` takes from `testing/` | skein | the fake LLM, forge and peers are product crates; the fake machine and the world are test kits, and `skein-fake-peers` depends on the GitHub fake in `testing/` | the box's fakes (`first-run.md`, 3.5) |
| The stepped world: a slice, the next deadline, input between slices, no step cap, idle waiting, a bounded trace | skein | `World::run` consumes the world; the simulator has `next_due` and `advance_to`; its trace is unbounded | a world debugged by hand |
| The worker bridge: a page's platform carrying skein's transport over messages to a Web Worker, and the worker's loop with its timer | skein (`skein-ui-web`) | none; the transport already carries a step through a platform of plain data, which lacks reading on and abandoning | any application's playground |
| Browser features for `getrandom`, `ring` and rustls in the compile closure | skein, and `temper-playground`'s features | throwaway patches only | — |
| The hosted agent: smith's `Agent` apart from its native run, program sign-in and relay; roots and configuration supplied, with no current directory and no DNS | smith | unchanged | any host embedding smith |
| The playground's composition, its documents and stories, the fakes' documents' latencies, the web shell generic over its platform, the frame, the build, the stories' tests | temper | the web shell holds skein's browser platform by its type | — |

- **jig changes nothing:** its adapter is pure, and the shell chooses
  the carrier.

## 9. Testing

- **Natively, in the gate:** each story walked whole in the
  playground's world, through the web client's host built natively, as
  the journeys' replay drives it (`../client/journeys.md`, section 2),
  within a world's budget. The memory bound is the configuration's,
  checked as any document's is.
- **The wasm32 builds in the gate:** `temper-playground` and the web
  shell with `playground` checked for wasm32 with clippy's lints, as
  jig's gate checks its own (jig's `protocol/browser.md`, section 15).
- **A log replayed:** a session log of story A, kept as a fixture,
  replays natively to the same end. A change that makes it differ is
  either a new log, recorded again, or a finding.
- **In a browser, outside the gate:** the browser tier walks story A on
  the built playground in headless Chromium (`../client/browser.md`,
  section 4), and measures the download, the memory's high water and a
  slice's latency against their bars, as its budget is held
  (`../client/browser.md`, 4.3).
- **By hand, before the release:** Firefox and Safari on a desktop, and
  a phone of each kind, walking story A.

## 10. Its bar for v0.1.0

- **Story A walks in the browser,** from the welcome to the CSV change
  landed and the other two requests answered: in the browser tier, and
  by hand on Firefox, Safari and a phone of each kind.
- **Within its bars:** the download and the memory (section 6), and a
  page that answers while the world steps.
- **A failure offers its log,** and a log replays natively (4.5).
- **Published** at the site's playground address with the release,
  from the release's commit, and linked from the site and the help.
- **B0 to B2 are not in the bar:** each is added once its rung passes.

## 11. Order

1. **skein's splits,** the shell's and the harness's, shared with
   macOS. Whichever needs them first builds them, and the other takes
   them.
2. **smith's hosted agent,** apart from its native start.
3. **skein's stepped world and its worker bridge,** with the fake
   machine and the simulated runner moved to product.
4. **`temper-playground`,** extracted from `tests/one-process`, which
   then composes it, with the gate unchanged.
5. **The memory pass:** measured, then bounded by the playground's
   documents and the trace.
6. **The page:** the platform, the starter and the printed link, the
   frame from the user's mockup, the lifecycle, the failure and its
   log.
7. **Story A,** with its pace, walked in the gate and by the browser
   tier.
8. **Delivery:** the static build, the policy, the bars measured in
   browsers and on phones, then publication with v0.1.0.
9. **B0 to B2,** as their rungs pass.

The investigation estimated steps 1 to 8, for one story with no
faults, at 24 to 36 hours with contingency. The log and its replay are
beyond that estimate.

## 12. Later

- **Pull the plug:** a button that kills the service or cuts the power
  at a chosen moment, then shows temper recover on the fake disk. The
  deterministic simulation, shown rather than claimed. The mechanism is
  the one-process world's kill and power loss.
- **Saving across a reload,** in IndexedDB: medium for snapshots, large
  for a faithful disk.
- **The playground forge's pages:** a pull request's diff in the page,
  shared with the box's playground (`first-run.md`, section 7).
- **The reference case's acts as stories,** once rung C passes.
- **The box's playground taking every story,** since a story is
  documents both can read.
- **Journeys recorded on the browser playground,** with no box behind
  it (`../client/journeys.md`).

## 13. Open questions

- **The welcome, or straight in.** The visitor chooses "Try the
  playground" as the box's person does, or the world starts the demo
  before the page signs in. Default: the welcome. It is the product's
  path, and it says what the playground is.
- **What the client offers on a demo-only deployment:** the welcome's
  Set up temper, Connect a forge, Connect a model. Default: nothing the
  document cannot serve, as a rule of the client's for every
  deployment.
- **The playground forge's links:** a pull request's link leads off the
  page to an address that answers nothing. Default: shown as text for a
  forge that serves no pages, until its pages (section 12).
- **The memory bar.** Default: 256 MiB of linear memory at the high
  water of every story, revised once measured on phones.
- **The download bar.** Default: 6 MB compressed for a first load.
- **A hidden tab:** paused, or running with its catch-up capped.
  Default: paused.
- **The stories in one world or one each.** Default: one world each,
  chosen in the frame (5.1).
- **Names:** the address, and `temper-playground`. Default: the site's
  draft decides the address; the crate's name is provisional.
