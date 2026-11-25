# The worker tools

2026-11-16, revised 2026-11-20. `temper ctl DIR worker VERB`: four
shorthands for the workers as settings (jig's
`domain/engine/workers.md`; `../../draft/config.md`, 6.7), each one read
or one settings change, and three asks of a linked worker, to take no
new work, to drain, or to take work again (jig's
`domain/engine/workers.md`, 5; `../../draft/archive/worker-restarts.md`,
section 2). They are rows of the one table (`tools.md`, 1), group `worker`, of
area 3 as `deployment`'s are (`areas.md`), and the map lists the group
under the deployment's noun (`../../protocol/documentation.md`, 2.4).

## 1. The rows

| Tool | Does | Keyed |
|---|---|---|
| `worker list` | reads jig's deployment document; prints each worker not withdrawn, in number order, a line each: `WORKER · N places · up since TIME · R runs`, `down since TIME` or `down` in place of its link's part, `1 place` and `1 run` for one, `no runs` for none, then `· paused`, `· draining` or `· drained` for a worker not taking work (`WorkerState.pause`), nothing for one taking; a watched read (2) | no |
| `worker add WORKER --places PLACES --digest SHA256` | adds a worker, or brings a withdrawn one back at its number | yes |
| `worker places WORKER PLACES` | changes a worker's places | yes |
| `worker remove WORKER` | removes a worker, withdrawn at its number | yes |
| `worker pause WORKER` | asks a linked worker to take no new run or job; its runs go on | no |
| `worker drain WORKER` | asks a linked worker to take no new work and drain: each run stops at its turn's end, what is left cut at the worker's drain bound; then `worker list --until drained:WORKER` | no |
| `worker resume WORKER` | asks a linked worker to take work again | no |

- **Their placeholders** join `tools.md`, 1's vocabulary: `WORKER`, a
  worker's name, in jig's form (jig's `domain/engine/workers.md`, 2);
  `PLACES`, a count from 1; `SHA256`, a digest as 64 lowercase
  hexadecimal digits, as `sha256sum` prints it. A malformed one is
  refused before anything is sent, as any argument is (exit 2).
- **Each act prints** `worker WORKER: revision R`, R the revision its
  change made, and with `--json` the change's answer as `settings
  apply`'s.
- **Their notes** say what the tools alone do: `worker add`'s, that the
  digest is the SHA-256 of the worker's secret, made on its own machine
  as `temper help server` says, and that a worker started first redials
  until it is added; `worker places`', that a linked worker takes its new
  places at its next start; `worker remove`'s, that a linked worker is
  stopped first, its runs ending interrupted.
- **The three asks print** `worker WORKER: asked to pause`, `asked to
  drain` or `asked to take work again` once the engine has the ask, and
  `worker drain` then the call that follows it, `worker list --until
  drained:WORKER`. Their notes say: `worker pause`'s, that a worker's
  restart ends its pause; `worker drain`'s, that a drained worker stays
  linked and takes nothing until `worker resume` or its stop, which then
  finds nothing to drain, and that a worker of an older build is kept
  from new work but not drained; `worker resume`'s, that a run already
  asked to stop still stops at its turn's end.

## 2. What each does

- **An act reads, then changes.** It reads the settings' newest
  revision, as `settings show` does, and the deployment document's
  `worker_settings` (jig's `domain/engine/workers.md`, 6), then sends
  one `ChangeSettings` against that revision, one item, `Worker(number)`,
  running work staying. Which number is `worker_number`'s
  (jig's `domain/engine/workers.md`, 2):
  - `worker add`: a free number or a withdrawn one's, set to the name,
    places and digest given, not withdrawn; a live worker of that name is
    refused before sending, "worker WORKER exists; `worker places`
    changes its places".
  - `worker places`: a live worker's number, set to its value with the
    places given.
  - `worker remove`: a live worker's number, *removed*.
  - A name with no live worker, for `places` and `remove`: "no worker
    WORKER", before sending.
- **A refusal is the change's,** worded as `settings apply`'s are, and
  exits as a refusal: *linked*, "WORKER is linked: stop it first; its
  runs end interrupted"; *count*, the configured workers' bound reached;
  *stale*, the settings changed between the read and the change, "run it
  again"; *not theirs*, an owner's act. No act retries.
- **An ask is no settings change.** `worker pause`, `drain` and `resume`
  read the deployment document's `worker_settings`, take the live
  worker of that name's number, as `worker places` does, and send one
  `PauseWorker { worker, mode }` (jig's `domain/engine/workers.md`, 5),
  with no key and no revision: asked again, it is asked again. A name
  with no live worker is "no worker WORKER", before sending. Its
  refusals exit as refusals: *unlinked*, "WORKER is not linked";
  *not theirs*, an owner's act. No ask retries.
- **`worker list`** reads the deployment document alone. A worker's runs
  are the runs of the connected host of its name (`Deployment.hosts`);
  `since` is printed as `settings show` prints a time. Its JSON is the
  rows not withdrawn, each with its runs' count and its pause, as
  `taking`, `paused`, `draining` or `drained`.
- **`worker list` is a watched read too,** beside those `tools-waiting.md`, 1
  lists, and waits as they do, on the deployment's watch (jig's
  `client/domain/contract/watches.md`, 1.1): `--since DIGEST`, `--until
  changed`, `--until drained:WORKER` (worker WORKER says it has
  drained) and `--until paused:WORKER` (it is paused, draining or
  drained), with `--timeout`; its digest is over the rows it prints, as
  `--json` prints them. A WORKER that names no live worker is a usage
  error, exit 1, before the watch opens. So a redeploy asks `worker
  drain w1`, waits on `worker list --until drained:w1`, then stops the
  worker.
- **Ask temper proposes, never acts:** these tools are ctl's alone, never
  a built-in agent's. Ask temper proposes a worker change through its
  proposal of settings, with the kind `worker` (jig's
  `domain/engine/workers.md`, 1), for an owner to accept.

## 3. Its pages and tests

- **Rendered:** the group's reference page, `ctl-worker`, and
  `temper ctl help worker`, from the rows, as every group's are
  (`tools.md`, 1; `testing.md`).
- **Focused tests,** in the tools' crate against its fake engine, as the
  settings tools' are: an add over a free number, over a withdrawn name
  and over a live one; places and remove over a live worker and an
  unknown name; a *linked* refusal's words; `worker list`'s lines for a
  worker up, one down since a loss, one never linked, and one withdrawn,
  which it omits; each ask sent with its mode for a live worker, "no
  worker WORKER" for an unknown name, and an *unlinked* refusal's
  words; `worker list`'s line for a paused, a draining and a drained
  worker; and its wait answered at the first delivery whose row for
  WORKER says drained, and not met past its timeout.
