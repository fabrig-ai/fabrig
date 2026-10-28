# The first release

Draft, 2026-10-23. What it takes for temper to go public, beyond the
v0.1.0 that roadmaps 1 and 2 make (the reference case on fakes, then
real backends). It records a brainstorm with the user on 2026-10-23 and
the two investigations run for it, and it is roadmap 3's north star:
its work is broken into missions progressively, each mission taking a
piece of it. Roadmap 3 has a lower priority than roadmaps 1 and 2. It
crosses skein, smith, jig and temper on purpose, and a website outside
them. Nothing here is adopted.

## 1. In one page

- **Five public faces,** each on its own name (section 2): the site,
  fabrig.ai; the forge, git.fabrig.ai; temper's own deployment,
  dev.fabrig.ai; the playground, playground.fabrig.ai; and a single
  binary to download.
- **The site** tells the factory's vision (`factory.md`) in plain words
  and presents temper as its first deliverable, with an honest status.
  It is static, and private until there is something to watch or try
  (section 3).
- **The forge is our own Forgejo,** a systemd service; GitHub is its
  mirror (section 4).
- **dev.fabrig.ai is the real thing:** a standalone engine and several
  workers under systemd, where temper's own development moves lane by
  lane. Anyone may read it without signing in, as an anonymous party
  whose load is capped and who sees nothing an agent wrote (section 5).
- **Upgrades are patches,** landed with each stored change from a
  baseline at the first release, proposed as v0.1.0, and applied at
  start, as `upgrades.md` designs. Workers may keep their runs while the
  engine alone restarts. Until the
  baseline, dev.fabrig.ai's history resets when a format changes
  (section 6).
- **A person configures temper by talking to it:** they describe their
  workflow, as agents' `roles/owners.md` and `roles/director.md` describe ours, and
  an agent proposes how temper will run it. Settings live in the store,
  never in the configuration document; new work follows a change while
  running work keeps what it started under, as `configuring.md` designs
  (section 7).
- **The playground runs in the visitor's browser,** fakes only: the
  one-process world built for wasm32, served as static files. An
  investigation found it feasible (section 8).
- **Trying temper for real is a download:** one binary, `temper box`
  (section 9).

## 2. The public faces

| Name | What | Serves |
|---|---|---|
| fabrig.ai | the site | the vision, temper, the download, links to the others |
| git.fabrig.ai | Forgejo | the code, its changes and releases; mirrored to GitHub |
| dev.fabrig.ai | temper's own deployment | temper's development, readable by anyone |
| playground.fabrig.ai | static files | temper on fakes, in the browser |
| the download | releases on git.fabrig.ai | `temper box` on the visitor's machine |

- **TLS ends at a reverse proxy** in front of the forge and temper, which
  serve plain HTTP behind it. temper grows no TLS server, as the user's
  rule for v0.1.0 asks.
- **The licence is MIT everywhere:** skein, smith and jig were already;
  temper (5a913979b) and hearth (463fa1700) followed on 2026-10-23.

## 3. fabrig.ai, the site

- **The front page** is the vision in plain words: temper writes, the
  lab proves, production runs, meeting through the forge (`factory.md`,
  section 1). temper is the part that exists; the rest is direction, and
  the page says so. `factory.md`'s vocabulary is internal, so the page
  is written fresh, not excerpted.
- **Its strongest proof is that temper is built by temper,** shown live
  on dev.fabrig.ai. Before temper-on-temper runs, the current process (a
  director running owners, agents landing most of the code) is the
  story.
- **Static,** in a repository of its own, on a host that serves TLS. Its
  look may share its tokens with temper's client.
- **Built now, published at launch.** The copy and the look are taste:
  drafted by Claude, chosen by the user through option pages.
- **It hosts** the download page (section 9), the documentation's public
  side (`configuring.md`, section 7), and links to the forge,
  dev.fabrig.ai and the playground.

## 4. git.fabrig.ai, the forge

- **Our own Forgejo, a systemd service,** owned or managed from temper's
  point of view. As designed, an owned Forgejo is one temper fetches and
  starts itself (`first-run.md`, 3.7 and 3.8); a Forgejo that systemd
  runs and temper administers may be closer to managed. Which one it is
  is open (section 12).
- **GitHub is a push mirror.** Its issues and pull requests are off, or
  point to git.fabrig.ai.
- **It is where code is read.** Casual visitors browse commits, diffs and
  pull requests there; dev.fabrig.ai links to them and shows no code.
- **Its owned Forgejo's upgrades** already have a path: a dump before the
  migration, pins that refuse a downgrade (`../protocol/setup.md`). Its
  backups cover the whole forge directory.
- **Public issues are untrusted text.** Once they can reach agents, an
  agent acts on an issue only after a maintainer has accepted it.

## 5. dev.fabrig.ai, the deployment

### 5.1 Its shape

- **No box:** a standalone engine and several workers, each a systemd
  unit, possibly on several machines; the Forgejo of section 4 beside
  them. The cluster's engine-only and worker-only processes (designed
  for one machine) are its base.
- **Workers on other machines reach the engine over WireGuard,** by
  default. jig's TLS between hosts is not planned, the worker channel
  already authenticates each worker by its credential, and a private
  network keeps temper free of TLS. Each machine's git, toolchains and
  secrets are the real work of a worker elsewhere.
- **A dedicated host,** not a development machine, with backups.
- **Private first.** It runs early, to find what operating a long-lived
  deployment needs (upgrades, backups, restarts, the proxy), and opens
  to the public once section 5.3 is built.

### 5.2 temper-on-temper, lane by lane

- **The owners process moves in pieces,** as roadmap 1 reaches the
  reference case incrementally: one low-risk lane first, such as landings
  of documentation or the triage of findings, then more as the gaps of
  `temper-on-temper.md`, 6.8, close.
- **Its backends** are roadmap 2's: an owned Forgejo and a ChatGPT plan
  first, which is exactly this forge. Whether temper-on-temper needs
  Claude before it starts (the current process uses Claude for taste) is
  open.

### 5.3 Reading without signing in

The user's three asks: an anonymous person or role; a cap on the load
anonymous requests put on temper; and nothing shown that an agent wrote,
so that what may appear is predictable. jig has an observer role, "read
and watch" (jig's `domain/people.md`, section 4), and leaves who may see
what, and guests, open (section 12 there). Today every client signs in,
holds a stream and its watches (jig's `protocol/client.md`, section 17),
and counts against the deployment's room for sign-ins (jig's
`domain/people.md`), so one sign-in per visitor is the wrong shape. The
direction:

- **One anonymous party,** fixed in code, for every visitor. Its role
  grants public reads and nothing else, and no policy widens it: a start
  whose policy tries is refused. A request with no credential is served
  as that party; visitors never sign in.
- **Reads, not streams.** The anonymous party has a few `GET` routes for
  public documents: a project's board, a task's public card, the
  landings. Their answers are the same for every visitor, so they carry a
  short `max-age`, and the reverse proxy caches them: a thousand visitors
  cost one engine read per document per few seconds. Pages refresh
  themselves at first; one shared stream per document, fanned out by the
  server, may come later, never one per visitor.
- **A load cap of its own.** The engine admits anonymous reads within a
  budget (reads per second, bytes) behind every signed-in request, and
  answers busy first. The proxy limits each address. The crowd is one
  party, bounded as jig bounds any party.
- **A typed allowlist, private by default.** Each public document is a
  type of its own, built by a projection; a field reaches it only when
  it is added there, and a golden of the public types makes every
  addition visible in review. The test for adding one is the user's: no
  agent wrote it.
  - *Public:* structure and mechanics: states, timings, the shape of a
    task tree, charter names, gate results, the landing queue, CI
    statuses, commit hashes, spend and throughput.
  - *Private unless marked:* what people wrote. A goal's title may be
    made public by a flag its author sets.
  - *Never:* what agents wrote: transcripts, results, notes, the labels
    and words of delegated tasks, reviews.
  - *Linked, not shown:* commit messages and pull requests, which agents
    write but the forge publishes once they land.

  A visitor sees, for example, a goal "Public reading", under it task
  1234, a coder, running for 14 minutes, and its change on git.fabrig.ai.
- **Visibility is per project.** The deployment also holds private
  projects (its operations, budgets and accounts); only a project marked
  public has projections. Whether people's names appear is open.
- **jig holds the generic part:** the anonymous party, its admission and
  bounds, the public routes and per-project visibility, which answer
  `domain/people.md`'s open questions. temper holds its projections (its
  forge's documents, its landings) and its pages' anonymous mode.
- **The fallback is a mirror:** a separate process holding one observer
  credential and serving cached pages, if a security review finds the
  direct way too risky.

### 5.4 Until upgrades

Until the v0.1.0 baseline (section 6), dev.fabrig.ai starts afresh
whenever a stored format changes, and a banner says so. git.fabrig.ai
keeps the lasting record (code, changes, releases) throughout.

## 6. Upgrades

Designed in `upgrades.md`, from an investigation of 2026-10-23 (Codex,
read-only; `~/src/rust/tmp/roadmap-3/store-upgrades.md`). In short:

- **Patches, from a baseline,** as an application migrates its
  database: every landing that changes what is stored brings a patch,
  and a start applies the pending ones to a copy of the store and
  switches to it. Any store from the baseline on upgrades to any later
  `main`, so dev.fabrig.ai may follow `main`. The baseline is the first
  release, proposed as v0.1.0 (skein's `codec.md`, section 5). Until
  then versions keep their current numbers, schemas and their one set of
  golden bytes change in place, and a store from an older build is made
  again when its families change (section 5.4).
- **What survives:** every record, live and ended. An agent resumes its
  conversation, its old turns patched as they are read, unless a patch
  says its window starts fresh.
- **dev.fabrig.ai's workers may keep their runs** while the engine alone
  is upgraded, when their wire is unchanged and the window fits their
  grace (`upgrades.md`, section 9). An upgrade that changes the workers'
  wire upgrades them too, and their runs end.
- **The policy from the baseline** costs a patch per stored change,
  most of them derived by a tool from two schemas that skein-codec reads
  at run time, and the gate holds it (`upgrades.md`, sections 4, 5 and
  12).
- **Retention.** A long-lived deployment keeps history on purpose. Tasks
  forgotten once read and transcripts past their horizon are erased
  (jig's `protocol/store.md`), and temper's table of landings is bounded
  (`../protocol/store.md`). Its retention policy is roadmap 1's gap 8 (a
  retention policy), designed before dev.fabrig.ai opens; an upgrade
  cannot bring back what retention erased.

## 7. Configuring temper by talking to it

Designed in `configuring.md`, from a session with the user on
2026-10-23. In short:

- **A person describes how they work,** as agents' `roles/owners.md` and
  `roles/director.md` describe ours, and an agent proposes how temper will
  run it; an owner accepts, on a page in plain words.
- **Structure becomes settings; rules stay text** that runs read, so
  most changes to a process need no configuring.
- **Settings live in the store,** never in `temper.json`: charters,
  templates, the policies projects are made from, the rules and the
  landing rules. A setting the store does not hold takes the binary's
  default, and a change is one decision, with no restart.
- **New work follows a change; running work keeps the charter it
  started under,** in revisions side by side. Conversations move at
  their next run, and what a task may do is never pinned.
- **The documentation is built into the binary,** for the site, the
  command line and the agent; the agent's test is the encoding
  `temper-on-temper.md` made by hand.

## 8. The playground

*Superseded by `browser-playground.md` (2026-10-26), which takes this
section over.*

Fakes only: the demo with richer stories, in the visitor's browser.

### 8.1 In the browser

An investigation of 2026-10-23 (Codex, with builds;
`~/src/rust/tmp/roadmap-3/playground-wasm.md`, its patches beside it).

- **The base is the one-process world** (`../domain/testing.md`, 2.5):
  the engine and a worker, jig's store on skein-kv over the simulator's
  disk, smith's agents as hosted programs, the fake LLM, forge and git,
  all on skein's simulator. No operating system's process or socket is
  needed.
- **It builds and runs, measured.** With throwaway patches, the world
  and the web client build for wasm32, and the chat story and the
  adoption-and-change story finish, calm, in Node's wasm engine. The
  patches show what is needed:
  - skein's shell separating a portable host from the native ring,
    clock, seed and terminal (medium);
  - smith's agent shell fencing off its native launch, sign-in and path
    handling (medium);
  - browser features for `getrandom`, `ring` and rustls, and skein's
    world gating its real-ring helpers (small).
- **Size, measured** (`gzip -9`): the world 14.7 MB raw, 5.2 MB
  compressed, with the harness still in it; the web client 2.5 MB and
  0.9 MB. Brotli and leaving the harness out shrink both.
- **Two modules:** the client on the page, the world in a Web Worker, a
  small bridge between them carrying requests and streamed answers.
- **What a page adds:** a world that ticks instead of running to its
  end, its clock paced to the browser's with pause and speed (medium); an
  in-page link from the web client to the world, as a virtual HTTP
  adapter with its own cookie jar or a direct transport of jig's
  (medium); the page's start, reset, speed and errors.
- **Its estimate:** about 10 milestones, 24 to 36 hours with
  contingency, for one guided story with no faults. The riskiest part is
  turning a finite test world into a long-lived interactive one without
  breaking flow control, sign-in or the store's acknowledgements.
- **After macOS's split:** the macOS port splits skein's shell and the
  simulator's harness by platform (`macos.md`, 3.5 and section 6), the
  same seams the playground needs; the portable host builds on that
  split.

### 8.2 Memory

The investigation's world grew the wasm memory to 548 MB at its first
step, and to 627 MB and 671 MB through its stories. That is reserved
memory, not profiled. The test fixtures leave memory unbounded on
purpose (a `memory_bytes` of 2^60), and the harness keeps the
simulator's whole trace. The user's lever: the playground has a
configuration of its own, with capacities and a memory bound just big
enough for its stories, and the harness's trace and captures stay out.
The memory milestone's first question is how much of the 548 MB that
configuration controls, and what else reserves it; its bar is a phone's
browser.

### 8.3 Stories

- **The fake LLM's scripts are the playground's content:** cues in the
  text select ordered turns, tool calls and finishes. The demo's stories
  (`first-run.md`) carry over; richer ones come from
  `reference-case.md`, each with its forge, files, CI and the visitor's
  expected actions.
- **Off-script,** a visitor gets a useful answer and the suggested next
  actions, never a real model.

### 8.4 Later

- **Pull the plug:** a button that kills the service or cuts the
  power at a chosen moment, then shows temper recover on the fake disk.
  The deterministic simulation, shown rather than claimed.
- **Saving across a reload,** in IndexedDB: medium for snapshots, large
  for a faithful disk.

## 9. The download

- **One binary, `temper box`,** is how a person tries temper for real,
  with their own model sign-in and forge; roadmap 2 makes that work.
- **Releases** on git.fabrig.ai, mirrored to GitHub: builds for Linux,
  and macOS as `macos.md` designs; the download page on fabrig.ai.
- **What the box expects on the machine** (git, a browser) is checked and
  stated, and the demo needs no git (`first-run.md`, section 8).

## 10. Already done

- MIT in all five repositories (section 2).
- The two investigations of sections 6 and 8.

## 11. Order

Roadmap 3 takes what roadmaps 1 and 2 leave free. Its tracks, each
broken into missions as it starts:

1. **The user's calls:** the names, the launch bar, the forge's mode
   (section 12).
2. **The site,** now, private until launch.
3. **The playground in the browser,** after the macOS split: the
   portable host and the hosted agent, the ticking world, the in-page
   link, memory, one story, the static page.
4. **Public reading,** designed in jig first (the anonymous party,
   visibility, public routes), then temper's projections and pages.
5. **dev.fabrig.ai,** private first: the forge, the engine and workers
   under systemd, WireGuard, backups; resets until the baseline; then
   public reading; then lanes moved onto it.
6. **Configuring by talking,** in `configuring.md`'s order: a survey,
   settings in the store and their revisions, in jig then temper; the
   pages; documentation in the binary; the configuring agent.
7. **The download,** from roadmap 2's usable point.
8. **Upgrades,** designed in `upgrades.md`, built as the baseline
   approaches, with the policy turned on by the user at the baseline.
   Retention before dev.fabrig.ai opens.

## 12. Open questions

- **Names:** fabrig, the factory, temper, the lab, hearth, and which the
  public sees first (`factory.md`, section 14). They stick once public.
  Default: settled before the site launches.
- **The launch bar.** Default: the site, and either dev.fabrig.ai
  readable or a downloadable v0.1.0.
- **The forge's mode:** owned, which as designed temper starts itself,
  or managed, run by systemd and administered by temper. Default:
  whichever needs less new design, decided with roadmap 2's owned
  Forgejo.
- **Configuration in a repository:** `configuring.md`, section 10.
  Default: the store alone.
- **People's names on public pages.** Default: hidden.
- **Claude for temper-on-temper's first lane,** or ChatGPT alone.
  Default: ChatGPT, roadmap 2's first backend.
- **When the upgrade policy starts.** At the first release, the baseline
  (skein's `codec.md`, section 5), proposed as v0.1.0; its date is open.
- **The playground's memory bar.** Default: a phone's browser.
