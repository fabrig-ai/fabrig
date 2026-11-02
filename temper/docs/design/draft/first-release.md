# The first release

Draft, 2026-10-23. What it takes for temper to go public: its public
faces, the site, the forge, temper's own deployment and the download,
from a brainstorm with the user on 2026-10-23. It crosses skein, smith,
jig and temper on purpose, and a website outside them. Upgrades are
`upgrades.md`'s, configuring temper by talking to it `configuring.md`'s,
the playground in the browser `../playground.md`'s and the download's
file `../release.md`'s. Nothing here is adopted.

## 1. In one page

- **Five public faces,** each on its own name (section 2): the site,
  fabrig.ai; the forge, git.fabrig.ai; temper's own deployment,
  dev.fabrig.ai; the playground, playground.fabrig.ai; and a single
  binary to download.
- **The site** says what fabrig makes in plain words and presents
  temper as its first deliverable, with an honest status. It is static,
  and private until there is something to watch or try (section 3).
- **The forge is our own Forgejo,** a systemd service; GitHub is its
  mirror (section 4).
- **dev.fabrig.ai is the real thing:** a standalone engine and several
  workers under systemd, where temper's own development runs. Anyone may
  read it without signing in, as an anonymous party whose load is capped
  and who sees nothing an agent wrote (section 5).
- **Until upgrades' baseline,** dev.fabrig.ai's history resets when a
  stored format changes (5.4; `upgrades.md`).
- **The playground runs in the visitor's browser,** fakes only
  (`../playground.md`).
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
- **The licence is MIT everywhere:** skein, smith, jig, temper and
  hearth.

## 3. fabrig.ai, the site

- **The front page** says what fabrig makes in plain words. temper is
  the part that exists; the rest is direction, and the page says so.
- **Its strongest proof is that temper is built by temper,** shown live
  on dev.fabrig.ai, as `reference-code-production.md` and
  `reference-landing.md` describe it.
- **Static,** in a repository of its own, on a host that serves TLS. Its
  look may share its tokens with temper's client.
- **Built now, published at launch.** The copy and the look are taste:
  drafted by Claude, chosen by the user through option pages.
- **It hosts** the download page (section 9), the documentation's public
  side (whether it renders `docs/help/` is `agent-docs.md`'s open
  question), and links to the forge, dev.fabrig.ai and the playground.

## 4. git.fabrig.ai, the forge

- **Our own Forgejo, a systemd service,** owned or managed from temper's
  point of view. As designed, an owned Forgejo is one temper fetches and
  starts itself (`../protocol/setup.md`); a Forgejo that systemd runs and
  temper administers may be closer to managed. Which one it is is open
  (section 12).
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

Until upgrades' baseline (`upgrades.md`, 4.1), dev.fabrig.ai starts
afresh whenever a stored format changes, and a banner says so.
git.fabrig.ai keeps the lasting record (code, changes, releases)
throughout.

## 9. The download

- **One binary, `temper box`,** is how a person tries temper for real,
  with their own model sign-in and forge (`../release.md`).
- **Releases** on git.fabrig.ai, mirrored to GitHub: builds for Linux,
  and macOS as `macos.md` designs; the download page on fabrig.ai.
- **What the box expects on the machine** is checked and stated
  (`../release.md`, section 4), and the playground needs no git.

## 12. Open questions

- **Names:** fabrig, temper, hearth, and which the public sees first.
  They stick once public. Default: settled before the site launches.
- **The launch bar.** Default: the site, and either dev.fabrig.ai
  readable or a downloadable v0.1.0.
- **The forge's mode:** owned, which as designed temper starts itself,
  or managed, run by systemd and administered by temper. Default:
  whichever needs less new design.
- **Configuration in a repository:** `configuring.md`, section 10.
  Default: the store alone.
- **People's names on public pages.** Default: hidden.
- **When the upgrade policy starts.** At the first release, the baseline
  (skein's `codec.md`, section 5), proposed as v0.1.0; its date is open.
