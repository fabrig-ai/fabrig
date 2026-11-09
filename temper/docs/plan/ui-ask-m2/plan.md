# ui-ask-m2: plan

Milestone M2 of `temper/docs/design/draft/ask-temper.execution.md`,
*Reading*: `see`'s addresses `me`, `project`, `goals`, `goals ended`,
`tasks`, `task N` with its tree, activity and turns, `change N`, `changes`
and `inbox`, read as the chat's person, of the chat's project alone, and
rendered by `temper/docs/design/draft/ask-temper.md`, 5.1's rules
(`inputs/request.md`).

The cut wrote the design first:

- jig's `jig/docs/design/domain/engine.md`, 7.3, *A run's read as its
  person*: the named action `PersonRead`, a party's `Read` read as the
  run's task's requester, admitted only where it names the task's project
  or a task of it; section 16's question on it answered, and a narrower one
  left (a frame narrowed to one project).
- temper's `temper/docs/design/domain/agent.md`, 8.4.1: the rows `me` to
  `inbox`, *Read as the person*, *How an answer reads*, the person's
  refusals, *What it never reads*, *On its page* and *Its tests*; 8.4 and
  section 10 brought along.
- `temper/docs/design/domain/root.md`, 5.1: a run's read as its person and
  the forge's document answering it.
- `temper/docs/design/protocol/documentation.md`, section 8: the reference
  page `see`; section 7's question on it answered.
- Their statuses' **Not yet** entries, and the draft's status lag on 5.1
  narrowed to what M2 leaves (the settings' addresses' text, kinds 4 and 5).

## Decided

- **`me` reads no frame.** jig's frame spans the person's projects; the
  run's read refuses it, and `me` is made of the project read (the person's
  role and pool there) and the inbox's needs filtered by the project,
  counted by kind (jig's section 16 keeps a narrowed frame for later).
- **What is held** (`project`) is the project's held tasks, a task query of
  the held phase: jig has no read of a project's placed holds
  (`jig/docs/design/client/domain/contract.status.md`), and a placed hold's
  tasks are held.
- **A goal's headline** comes from the project's notes read in a goal's
  scope (`report: ` and the headline, `agent.md`, 8.3).
- **`task N tree`** reads the task's delegates, a task query by parent, a
  level at a time; jig has no one-shot read of the tree watch's document,
  and a delegate's own tree is its own address.
- **`change N` and `changes`** are jig's application reads of the forge's
  subjects, which the core admits as the person's; temper's root answers
  them with the forge's documents, made as their watches' are, where it
  answered every application read unknown.
- **The settings' addresses keep their text.** 5.1's rules for how an
  answer reads hold M2's addresses; the settings, revisions and proposals
  read as their renderers write them (`agent.md`, section 10).
- **"As tables"** is read as the draft's tests say it: each answer a table
  of lines, a heading and a line a row, tested as table rows.
- **The reference page `see`** joins, from a table of addresses in
  `temper-engine-smith` that also gives the unknown address's line and each
  `see:` line, as `documentation.md`'s question deferred it to M2.

## Parts and tasks

- **person** (jig): `person-surface`, high, 90 minutes, no needs: the named
  action, its answer, its routing and first checks, every admitted read
  unknown.
- **answers** (temper's root): `see-surface`, medium, 60, no needs: the
  address grammar and `SeeAnswer`'s records (`notes/contracts.md`,
  section 2), every M2 address still not yet.

Slice 1 holds both. The rest follows in the next batch: jig's admitted
reads, the root's reads and the forge's answer, the renderer, the reference
page, and the stories with the fuzzy test of addresses.

## Critical path

So far: `person-surface` (90) and `see-surface` (60) in parallel.

## Who hears first

For jig's owner, shared work:

- **`NamedAction::PersonRead(Read)` and `Now::PersonAnswer { to, key,
  answer }`** in `jig-domain-core` (`notes/contracts.md`, section 1): every
  root that matches `Now` exhaustively gains an arm, jig's test kits, its
  ops example and temper's root among them.

For the owner of coding runs: no change to `temper-engine-domain-forge`'s
or `forge_route.rs`'s behaviour so far; `route/asks.rs`'s `Ask::Read` is
the next batch's, named there.

## Overlaps

The drafts beside this one, `merge-job-runs-2` (a remote worker's merge
job) and `ui-cli-c1` (`skein-shell`'s conventions), touch none of this
batch's files.
