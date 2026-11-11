# Testing `temper ctl`

## 1. Step tests

- **The table:** each tool's argv parsed into its call and back, and its
  JSON; usage errors; the help, as golden text.
- **The rendering:** each result kind in text and JSON, as golden text,
  with jig's and the forge's words; a turn by its size; a cut and its
  call.
- **Digests and conditions:** a digest moves with a document's change
  and only then; each condition over documents that meet and miss it.
- **Task waits,** in the existing tools step fixture: `Unknown` and
  `SubjectEnded` each followed by real `Historical` and live-form
  terminal records, nonterminal and wrong-number records, unknown or
  hidden history, Busy, sign-out and read failure. Only matching
  terminal records meet `ended`; each path emits exactly one result
  and releases its handles and timer. No new harness (`tools.md`, 4).
- **Tree waits,** in the same tools step fixture: an Idle-root delivery
  remains pending until `SubjectEnded`; text and JSON then report
  watch completion and label the unchanged rows as the last observed
  snapshot, with its digest. Terminal deliveries, no-delivery `gone`,
  timeout, `--since`, and a single answer with watch/timer cleanup
  are covered here: no forge, workflow, model or kernel is needed (`tools.md`, 4).
- **No-change completion,** in the tools step fixture: a producing panel,
  a successful no-change report, the unmet landing wait and the retained
  ended panel on a later read. Completion on the first task read and
  arriving during a wait both name the result and the direct report
  command; `task show` keeps the empty-search explanation (`tools.md`, 4).
- **The boundary:** `src/jig/` names no temper item (`README.md`, section 4).
- **The bounds:** each of the tools' ceilings within the client's limit
  it reads from, and a picker's page within a result's text (`README.md`, section 9
  ): a result's text within one document's bytes, a page within a
  turn's bytes, the client's text within a result's text and equal to
  the words an act takes, and a `task text` page holding `README.md`, section 9's
  page whole, its lines within the 512 bytes beside it and its JSON
  within a result's bytes.
- **Paged reads,** as table rows beside the existing `task text` and
  `task turn` tests: a cut page's first line, with its page, of how
  many, the bytes it holds and the call its last line prints; a middle
  page, and the last with `no more`; a stopped retrieval's first line
  naming the call its last line names; a part found as the renderer
  counts parts; and an uncut text, or a whole read, with no such line
  (`results.md`, 4).
- **DIRECTORY's records,** in `temper-client-shell-cli`'s tests: a
  record of a store version the tools do not read counted by `pending`,
  and the next call that sends saying it dropped it.
- **An account's move,** in `temper-client-shell-cli`'s tests, the
  engine's answers scripted (`areas.md`, area 5; `results.md`, 3): `account export`'s FILE
  already there, a link among them, exit 1 with nothing sent; FILE made
  with mode 0600, the record written whole and synced, then its
  directory, before `written` is sent; each of the engine's refusals,
  exit 2, FILE removed; `written` refused "cancelled", exit 2, FILE
  removed; a write that fails, and an answer lost, exit 1, FILE removed
  and no `written` sent; `written`'s answer not come, FILE kept.
  `account import`'s FILE not private, a link, empty and past its bound,
  exit 1 with nothing sent; *moved in*, the account printed and FILE
  removed; each refusal, exit 2, and *lost*, exit 1, FILE kept. No
  record's byte on stdout, stderr or in DIRECTORY's store.

## 2. `tests/client`

`temper-client-world` runs the tools' root against temper's engine root,
beside the fake person, as a scripted agent making calls (`../testing.md`
; `../../testing/checks.md`, 3). Area 1's stories: a chat started, its
reply waited for, a message, a turn too large for a page read whole,
the chat closed; a repository adopted; a held task released. The
existing agent story also waits for a task whose end was observed before
`task show TASK --until ended` begins, checking the matching terminal
answer (`tools.md`, 4). A turn too large for a page exists only where the engine's
bound on a turn passes its page bound, so the world's profile sets it
so. The world's faults hold: a call cut by a link drop or an engine
restart, asked again under its named key, is made once. Each area adds
its stories. Area 5's, over the engine root's accounts child with each
source's mechanism scripted: a key added, then the deployment's,
`account list` showing both with no value in DIRECTORY's store or in
any document the tools print; an `account add-key` cut before its
answer and asked again with its key and file, one account made; a
sign-in started, its answer scripted made, refused and past its
deadline, each meeting `account list --until settled`; a limited
notice, the next task waiting for its account and `task show TASK`
saying why, then `account retry` and the run started; `account rank`
and `account forget`; a program source declared, and `account verify`
answered as each host reports; a program account forgotten, its name
declared again by its holder, its host-check dispatch observed, then a
declaration of that live name refused at exit 2 with its reason and next
step (`areas.md`). Area 6's, over the engine root's forge
connector and the setup's host with skein's fake forge
(`../../testing/real-loop.md`): a GitHub connect printing its link,
never opening it, the browser's steps driven by the world, then `forge
list --until settled` and the forge connected; a pasted connect cut
before its answer and asked again with its key and file, one connect;
an App lacking a permission and an event, each named on the forge's
row; a tokens file with a line broken, exit 1 naming the line and
never its value; a connected Forgejo connected by its tokens file, its
replace of one token, and a replace refused `not connected`; `forge repositories` by page number, a full last
page's next and a page past the last answering no rows, and one row
adopted with `--ids`; a second picker's read while one runs, busy; a
call by a party whose role does not hold the deployment's connectors,
exit 2; `forge forget` on a managed Forgejo; `forge setup` without the
box, refused naming the commands; `forge setup` on an asked owned
forge, answered `setting up`, then `forge list --until settled`
following the routing's told states, setting up at each step to not
connected, and to set up failed with its why; a `forge setup` of the
same forge while it sets up, not offered, and of another asked forge
while one sets up, busy; `playground start` answered `running`, run
again answered the same, adding and adopting nothing twice, and
refused "not offered" with a real forge kept; `playground end`
answered `reset`, and refused "not offered" with no playground; `forge setup` and `forge connect`
without FORGE each making a forge, numbered next; `forge remove`
answered `removed`, the forge gone from `forge list`, and refused for
the playground's forge. After each, no pasted value in
DIRECTORY's store or in any document the tools print.

## 3. The real loop

`tests/real-loop` hosts the tools' root over jig's native transport,
the standalone engine and worker, smith's agents on skein's fake LLM
peer's scripted turns, skein's forge peer hosting the fake forge, and
the fake git program over that forge, on one thread and one real ring
(`../../testing/real-loop.md`). Agents are hosted in that loop and
joined to their worker by real pipes. The store and client credentials
use a scratch directory on the real filesystem.

Its stories sign in, start a chat and wait for its reply, and satisfy one
client's wait by another client's act; each drains with everything
settled. Then a service restart retains the work. A call whose answer was
lost is made once: the tools' root is dropped once the engine holds its
request, before its answer, and a fresh one, on another DIRECTORY signed
in as the same party, asks under the same name and is answered the first
answer; the party has one chat. One signed in again as another party:
the first party's kept request stays unsent, and the call goes on. The
forge's stories adopt a repository, and clone, push and land through the
fake git program over the forge peer's git route, with CI by the fake
forge's content marker and its webhooks delivered to the engine
(`../../testing/real-loop.md`). Real git meets a forge in the process
tier (4). The forge's stories run on `skein-world`'s
`World::new_controlled`, unchecked, while the fake git program shares
the fake forge in memory: its memory there outlives the git program
that made it, which the checked world fails (skein's
`../foundation/testing-strategy.md`, section 6; skein's `testing/fakes/forge.md`, section 5).
Over the git route the program shares nothing, and they run checked.
Every other story here is checked.

Every story whose agents run first adds a key the fake LLM peer
accepts, `account add-key` from a scratch file, as `README.md`, section 8's harness
does, and waits for `usable`. Area 5's stories (`../../testing/real-loop.md`
):

- **A sign-in:** `account connect` against skein's fake issuer, the
  browser's visit and the redirect to the engine's loopback listener
  driven by the test, then `account list --until settled`, the account
  usable, and a run on it whose request the peer observes with its
  bearer.
- **A program's login:** `account declare`, with smith's fake `claude`
  as the agent's program, its login signed in in the source's directory
  under the agent's programs directory; `account verify`, then `account
  list --until settled`, the source usable on the worker's host, and a
  run on it through the program backend.
- **No value kept:** after each, neither DIRECTORY's store nor the
  engine's store holds a key's or a record's value in a request, an
  answer or a fact; the secrets' store alone does.

Area 6's stories reach the forge through the tools, on skein's fake
forge, whose browser pages are plain HTTP: its manifest post answers
`200` with a page holding one link, `Create GitHub App` (id `create`),
and its install page `302` (skein's `testing/fakes/github.md`, 1.1), so the test
drives a browser's steps as plain requests, following the link as a
person clicks it, and no story calls github.com:

- **GitHub through the manifest:** `forge connect FORGE github`, the
  printed link followed as plain requests on the clients' listener and
  the fake's pages, then `forge list --until settled`, and `forge list`
  showing the forge connected as the App's user.
- **A repository added later,** from the picker: `forge repositories
  FORGE`, then `repository adopt --ids` of one of its rows into a
  project holding another.
- **A repository granted on GitHub:** the fake grants one to an
  installation of selected repositories, and the next `forge
  repositories FORGE` lists it.
- **Forgejo connected, managed and owned** from the tools: `forge
  connect FORGE forgejo-admin FILE --key NAME`, with and without
  `--first work/hello --project PROJECT --label docker`: with the first
  repository, waits for creation and connection, makes a missing
  project, adopts once under `first-repository-F`, and prints its
  project and adoption; existing projects by name or number, reruns,
  stops and failed Create are covered. Without `--first`, prints `forge
  list --until connected:FORGE`. Also an owned forge's `forge
  repositories`, and a connected Forgejo: `forge connect forgejo
  FILE --api HOST:PORT --git URL` with a tokens file, connected as its
  user; `forge replace FORGE FILE` with a new git token alone,
  connected again as the same user; and a tokens file of another user,
  stopped, named on its row, the forge as it was.
- **No secret kept:** after each, no pasted value, key, token or secret
  in a root, a fact, a record but the secrets' store's, a request or an
  answer, nor in DIRECTORY's store.

Each call of `temper ctl` in these stories is its host,
`temper-client-shell-cli`'s, hosted as a program of the real world, one
per call, on its DIRECTORY: the lock, DIRECTORY's files and the call's
exit run on the real kernel. Two acts at once on one DIRECTORY are
serialized by the lock: skein's `Lock` is `flock`, held per open
descriptor (skein's `runtime/kernel.md`), so two programs of one process contend
as two processes do.

These are smoke, one end-to-end path per feature, at the gate
(`../../testing/real-loop.md`). A tool's behaviour and corner cases are
`tests/client`'s (2) and the lower worlds' (1): a sign-in refused
or past its deadline, a limit and `account retry`, a record the tools do not
read. These stories reuse lower worlds' scripts and outside expectations
where applicable, and add checks of the full transport, persistence and
teardown on the real kernel; the harness checks memory ownership,
outstanding operations and release between iterations.
A real-loop failure is kept with its trace and reproduced lower down
where possible; its seed does not replay the kernel's schedule.

One story reads at `temper-box init`'s template's bounds, its documents'
rows below the tools' 32-row window: a goal with more delegates than
those rows, one of them ended with an ended dependency, and `task show
TASK --tree` answering within a second, cut and saying so, and `task
activity TASK --json` of the ended one answering its rows, newest
first, and reading on to its first; the tree's `cut:` line's call
followed, page by page, to the last delegate. Its fixture sets that
bound itself, whatever the others' fixtures raise.

## 4. The process tier

Settings inventory, individual values, history, proposals, owner decisions
and placed holds use the CLI's scripted machine world, client tools' step
tests and jig's core worlds. Their rendering is 1's. Running work
keeps its revision in `tests/system/tests/suite/running_revision.rs`
(`../../testing/README.md`, 2.1): after an owner's instruction change makes
R+1, running work staying, a parked task that is no conversation runs
again on R, its run and smith's requests reading R's instructions, while
a parked chat moves at its next run and reads R+1's; the owner's advance
then moves the first, whose next run reads R+1's. Jig's worlds cover a restart with
R held, a removed charter and revision bounds.

The focused suite has one short path through the shipped binary on fakes
(`../../testing/process-tier.md`). Other process stories add only the OS's
boundary: a ctl act cut by a termination signal, its status 128 plus the
signal number and its keyed replay; process-tree supervision and a real
binary restart. An interrupted account export reuses a minimal imported
account on a fake issuer, then restarts and reads it usable. The CLI's
`moves` machine tests own private FILE checks, write synchronization,
lost replies and kept retries; jig's account worlds own move deadlines
and restore decisions.

The owned-Forgejo stories (`../../testing/process-tier.md`) share setup where
one adds a variant: a real turn gate cancelled and retried across restart;
a first CSV change whose failed CI output reaches its repair, followed by
a blocking review, landing and a restart that finds it. The lock,
DIRECTORY's files and a call's exit are the real loop's (3).

## 5. The agents' help

- **`temper ctl help`** prints the page `ctl` of `temper-docs`
  (`../../protocol/documentation.md`), then the tools from the table, a
  line each, its usage and its line. The page is the driver's first
  read (`../../protocol/documentation.md`, section 2.4): how to sign in, the loop
  of `tools.md`, 4 (read, act, wait), naming keys and their prefix, reading acts
  and the revisions they name, the exit statuses of `results.md`, 3, JSON, the forge
  read with whatever the agent chooses and what temper owns there
  changed only through temper (`tools.md`, 2), and a secret read from a private
  file, never typed or given as an argument, with a link printed as
  text, never opened, and followed by a wait (`areas.md`, areas 5 and 6).
- **`temper ctl help GROUP VERB`** prints the tool's record (`tools.md`, 1): its
  usage, its line, each argument's name, type, bound and line, its
  result's line and record's name, whether it is keyed, and its notes;
  for a watched read, its conditions as the table's kinds give them.
  Enough to make every call of a walk without the source.
- **What a goal takes across tools is a walk's,** `box` and `github`
  among them, never the page `ctl`'s.
- **Checked:** every `temper ctl` line of the page parses
  (`../../protocol/documentation.md`, section 6); a tool named in it that
  the table lacks fails the gate. Written pages use that document's
  placeholders (2.1) and voice (2.3).

The indexed walk `open-public-reading` joins the engine's public admission,
HTTP and index figures, their configuration location and restart, owner
visibility calls and public page paths. Project help links it; visibility
alone does not open admission whose limits remain zero.

## 6. Findings into stories

Each bug an agent finds through the tools ends as a story in
the lowest tier that shows it, before its fix lands: a domain world,
`tests/client`, the simulated world, the real loop or the process tier. A story's calls are the tools' calls, so an
agent's history of `temper ctl` commands transcribes into one.
