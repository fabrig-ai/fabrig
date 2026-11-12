# setup: status

2026-11-11. Where temper's code lags or departs from [README.md](README.md).

`owned.md`, section 6, and `README.md`, section 7's shared fetch and launcher are present:
`temper-fetch` owns the machine and Claude pin, `temper-fetch-shell`
the TLS host and Claude Code's fetch under its lock, with no smith
crate, and `temper-launcher` the launcher, the placing of its links in
a private directory through `smith-agent-document`, and the
substitution. Forgejo's pins and bottle extraction stay with forge
setup, whose old fetch paths are re-exports. The pin/progress tests in
`temper-fetch`, the substitution's in `temper-launcher`, the retained
local-TLS fetch suite and the agent's launcher rows cover the split; the
process scan exempts `temper-launcher/src/lib.rs` for the launcher's
exec and process id. Neither forge-setup crate is in the agent's normal
closure, and the forge's setup links no launcher, so `temper-engine`'s
closure row holds (`../binaries.status.md`, 1). `owned.md`, section 6's `claude`
link names the worker's agent program, the box's `temper-box` or a
deployment's `temper-agent`, both at worker start and at program login
(`temper-worker-shell/src/worker.rs`, `sources`;
`temper-worker-shell/src/program_login.rs`, `command`). `README.md`, section 9's
real Claude download walk and other deferred work remain unchanged.

`init.md`, section 3 has no known implementation gap in the binary supplying
every project's rule 1 and no deployment rule, or supplying stock
charter 5 and template 4 with their delegation. The documents carry none
of those settings. Applying the reference's combined project settings
in a walk remains unobserved (`../configuration/reference.status.md`);
`README.md`, section 9 keeps that after-landing work.

`box.md`, section 2's sign-in block names the TUI command at the link's origin,
on stderr in the first block alone. Serve keeps the engine's private
link file at each `LocalLink`, never a printed link, and removes it when its loop
returns (`../../client/tui/README.md`, 2). No known implementation gap
remains in those additions; a forced kill can leave a stale file as
designed. `testing.md`, section 4's box stories
(`bin/temper-box/tests/suite/box_listeners.rs`) hold the file's link at stdout's address and apart from every printed
link, and left as it was when the first printed link is spent.

- **Not yet:** `box.md`, 2, *Its children*: the owned-forge Claude Code landing
  story (`bin/temper-box/tests/live.rs`,
  `a_person_asks_claude_code_to_land_a_change_on_the_owned_forge`) has no
  supplied live rerun establishing no surviving descendant after the
  orderly-stop fix. The held-turn test does not establish that live
  outcome. The live judge retains its no-survivor assertion and reports
  each survivor's command line, parent, group and session, with the
  parent's and group leader's rows at stop, through the log's filter.

- **Measured** (`init.md`, 3, the resource gate, 2026-11-04, on a development host, dev build): a fresh box at the one-slot init defaults of that date, excluding agent processes: anonymous memory (smaps Anonymous, box and serve summed) idle 12.75904 MB (serve 7.647232 MB, box 5.111808 MB), with one goal running 13.23008 MB, its sampled peak 13.23008 MB, after 80 chats and the playground's change 17.567744 MB, after a restart 33.988608 MB; beside it, summed, RSS idle 100.27008 MB, with one goal running 106.770432 MB, PSS idle 66.407424 MB, with one goal running 53.19168 MB; the store on disk fresh 0.014139 MB, after 80 chats and the playground's change, and a restart, 11.121464 MB (the data directory 11.122122 MB).
- **Kept** (`init.md`, 1, 3; the user's request of 2026-11-03 asks that it be said here): the rest of the machine's concurrency stays as `temper-box init` set it, sized by no figure of that request: `fleet.workers` 2, `fleet.configured_workers` 8, the clients' streams 8 with their 12 exchanges, and the owned runner's capacity 1, so a box's CI runs one job at a time beside its runs.
- **Design departs:** `box.md`, 2, from the request `ui-welcome` of 2026-11-05, decided by its plan's cut: an owned forge's hands go to its Forgejo step's own log, `forgejo-N.log`, where the request has the box's log: the box had no log of its own when that plan was cut. `box.log` now exists (`box.md`, 2), and the hands stay: no request asks to move them. `owned.md`, 3, from the request `runner-labels` of 2026-11-05, decided by its plan's cut: a preparation keeps the row when `runner/config.yaml` names a `uuid`, where the request has it keep a declared runner's labels and version when its uuid matches: no Forgejo runs under the preparation's lock and its command line lists no runners, so no row is read or compared, and a `runner/secret` replaced by hand beside a kept configuration does not reach the row.
- **Design departs:** `init.md`, 1, from the request `first-goal-fixes` of 2026-11-05, item 4, decided by its plan's cut: `documents.text_bytes` is 512 in `temper-box init`'s documents, where the request has jig standard's 8,192 unless that breaks a bound a start checks. A start takes 8,192 and compares it with none of the web and terminal clients' bounds (section 9), and they refuse longer document text, which `crates/temper-box-shell/src/tests/init_clients.rs` holds for init's documents. So a client reads a task's spec in pieces of 512 bytes (section 10). The playground keeps 128 (`temper-playground/src/documents.rs`, `stories/a/deployment.json`), where the request keeps it only if its size needs it, and whether it does is unknown: no document records it.

`init.md`, section 3's printed link and approval are enabled in both init
documents and the owned example. The service table checks that the
approval setting reaches an ask. The TUI's browser flow and recorded
approval journey are present. Its fresh-box terminal check remains
later work in `README.md`, section 9 and `../../client/README.md`, section 5.
- **Not yet:**  the first-box browser notice remains `box.status.md`, 2.  the owned-Forgejo sign-in command remains `owned.status.md`, 8. 2: the connected-user admin takeover remains `reconcile.status.md`, 5.  the new-repository action remains `owned.status.md`, 5.  the owned-forge header-file paths remain `box.status.md`, 2.  the repeated-fetch terminal output remains `box.status.md`, 2. `box.md`, 2, *On a terminal*: the real-terminal box proof remains `box.status.md`, 2, *On a terminal*.  the fake forge's smart HTTP remains `playground.status.md` (Not yet, the whole document).  the removed-forge playground refusal remains `playground.status.md` (Not yet, the whole document).  the playground's model restriction remains `playground.status.md` (Not yet, the whole document).  the playground/reset and restarted-GitHub stories remain `testing.status.md`, 4.  the App's status/action permissions remain `github.status.md`, 1.  the GitHub port-zero check remains `../forge/engine.status.md`, 1. 6: the managed-hook delivery proof remains `reconcile.status.md`, 5.  the polling-after-hook-failure proof remains `reconcile.status.md`, 5.  the fresh reference-case walk remains `init.status.md`, 3.
- **Departs:**  the group-file ownership departure remains `../forge/engine.status.md`, 2 and 8.  the scripted model's setup direction remains `playground.status.md` (Departs, the whole document).  the init fallback's forge number remains `init.status.md`, 2. 10: the shared managed-forge bot login remains `reconcile.status.md`, 5.  the idle-issuer socket timing departure remains `box.status.md`, 5.  the early-link backlog proof remains `testing.status.md`, 4.
