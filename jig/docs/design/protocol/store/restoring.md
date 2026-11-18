# Restoring a checkpoint

2026-11-17. Bringing a checkpoint back over a stopped deployment,
crash-safe (temper's `draft/checkpoints.md`, section 5.3; milestone C2
of `draft/checkpoints.execution.md`). The component first takes a
safety checkpoint of the deployment as it stands. It then stages the
next generation and a transcript tree from the checkpoint, writes a
durable intent, and rolls forward: the transcript trees are swapped by
two renames, then `active` is replaced. Every open resolves an
intent that is still outstanding before it reads `active`. What a
checkpoint is and how it is taken is `checkpoints.md`. temper's command
is temper's `protocol/checkpoints.md`, 2.5.

## 1. In one page

- **Asked for** (3) by `Adoption::Restore { name, draw }` over a
  stopped deployment. It is answered `Restored`, naming the restored
  generation and the safety checkpoint, or `RestoreRefused`.
- **Locked** (4.1): the deployment's lock from the first step to the
  answer, the source checkpoint's from before the safety checkpoint,
  and the safety checkpoint's once it is in place.
- **A safety checkpoint first** (4.3), taken as `checkpoints.md`, 5
  takes one over a stopped deployment. Undoing a restore is restoring
  the safety checkpoint.
- **Staged** (4.4, 4.5): the next generation, made from the
  checkpoint's files, with a fresh manifest that names no previous
  generation, so `--back` refuses after a restore. Beside it,
  `transcripts.staged/`. Nothing live changes while they are made.
- **The intent** (4.6), `restore`, written once both are staged and
  synced; then **rolled forward** (4.7), each step idempotent.
- **At every open** (5), an outstanding intent is resolved from
  whatever stands: the restored maps never open with the old
  transcripts, nor the old maps with the restored ones.

## 2. On disk

### 2.1 Names

In the deployment's directory, beside `active` (`README.md`, section
2):

| Path | What it is | Who removes it |
|---|---|---|
| `generations/<N + 1>/` | the staged generation, N the number `active` named when the restore read it | `active` names it at 4.7, step 3; with no intent standing it is a stray (`opening.md`, 1.5) |
| `transcripts.staged/` | the staged transcript tree, laid out as `transcripts/` (`transcripts.md`, 1) | renamed into place at 4.7, step 2; a stray with no intent |
| `transcripts.aside/` | the live tree, set aside at 4.7, step 1 | 4.7, step 5; a stray with no intent |
| `restore` | the intent (2.2) | 4.7, step 4 |
| `restore.skein-<16 hex>` | the temporary of the intent's durable replace (skein's `runtime/io/files.md`, 2) | swept with `active`'s temporaries (`opening.md`, 1.5) |

- **The tree's names are fixed,** not drawn. At most one restore runs,
  under the deployment's lock. An open removes whatever an unfinished
  restore left before the next restore can begin.
- **`checkpoints/`** holds the source and the safety checkpoint as
  ordinary checkpoints (`checkpoints.md`, 2). The restore removes
  neither, and retention keeps or removes them as any others once
  their locks are let go (`checkpoints.md`, 4.7).

### 2.2 The intent

`restore` is written by io's durable replace expecting no file. It
holds a versioned record of the component's own schema, added to
`component-v3.schema` in place beside `Checkpoint`, as `Checkpoint`
was (`checkpoints.md`, 2.3):

```
versioned record RestoreIntent {
    generation: u64        # the staged generation, N + 1
    active: fixed 32       # the digest of `active` as the restore read it (skein's runtime/io/files.md, 2)
    source: text 37        # the checkpoint restored, <20 digits>-<16 hex>
    safety: text 37        # the safety checkpoint
    made: u64              # env.wall as the intent is written, in Manifest.written's unit
}
```

The staged tree's name is fixed (2.1), so the intent names it by being
there. Both names decode through `CheckpointName`'s parse. An intent
that doesn't decode refuses the open as 5 says.

## 3. The request and its answer

```rust
// jig_protocol_store, beside Request::Back and Event::Back (boundary.rs)
Request::Restore { owner: Token, root: Root, name: CheckpointName, draw: [u8; 16] }
Event::Restored { owner: Token, restored: Restored }
Event::RestoreRefused { owner: Token, why: RestoreRefusal }

pub struct Restored {
    pub name: CheckpointName,     // the checkpoint restored
    pub safety: CheckpointName,   // the safety checkpoint taken first
    pub generation: u64,          // the generation `active` now names, N + 1
    pub linked: u32,              // files linked into the staged generation and tree
    pub copied: u32,              // files copied there, a refused link counted as a copy
    pub bytes: u64,               // bytes copied there
    pub took: u64,                // nanoseconds from the request to the answer, the safety checkpoint included
}

pub enum RestoreRefusal {
    Busy,                                                // the deployment's lock is held (opening.md, 1.1)
    Source { why: CheckpointRefusal },                   // the source's lock: InUse or Missing (checkpoints.md, 6)
    Unreadable { path: Box<str> },                       // the source's record or manifest absent or undecodable
    Deployment,                                          // the source's record names another deployment
    Open { why: OpenRefusal },                           // the safety checkpoint's open refused (checkpoints.md, 5)
    Safety { why: CheckpointRefusal },                   // the safety checkpoint refused (checkpoints.md, 3)
    Staging { safety: CheckpointName, path: Box<str>, error: Option<file::Error> }, // a staging step failed; None: a file shorter than its record says
    Interrupted { safety: CheckpointName, path: Box<str>, error: file::Error },     // a step after the intent failed (4.8)
}

// host.rs, beside Back
Adoption::Restore { name: CheckpointName, draw: [u8; 16] }
```

- **One terminal per request,** tagged with `owner`, as every request's
  (`README.md`, section 3). The request needs no open: it makes the
  safety checkpoint's open itself, under its own lock (4.3).
  `Adoption::Restore` makes it as `Back` is made, and gives the shell
  that terminal alone. How the component and its host split the steps
  of 4 between them is the code's choice. The adoption, the two events
  and the two types above are the contract.
- **`draw`** is 16 bytes its asker draws from its randomness, as
  `Request::Checkpoint`'s is (`checkpoints.md`, 3). It is the safety
  checkpoint's `draw`.
- **A path** in a refusal is relative to the deployment's directory,
  at most 128 bytes, as a checkpoint refusal's is.
- **Its words,** `RestoreRefusal::words`, for whoever prints a refusal:
  - `Busy`: "the deployment is in use by another process";
  - `Source`: the checkpoint refusal's words ("in use: its lock is
    held", "no such checkpoint");
  - `Unreadable`: "the checkpoint's record can't be read, PATH";
  - `Deployment`: "the checkpoint is another deployment's";
  - `Open`: "the deployment does not open". A shell prints the open
    refusal's own words in its place, as it prints a refused start;
  - `Safety`: "the safety checkpoint was refused: " and its words;
  - `Staging`: "staging failed, PATH: ERROR, and nothing live changed;
    the safety checkpoint is SAFETY". With no error, "PATH is shorter
    than its record says" stands for "PATH: ERROR";
  - `Interrupted`: "interrupted after its intent, PATH: ERROR; the
    next start finishes the restore; the safety checkpoint is SAFETY".

## 4. Restoring

### 4.1 Locked

- **The deployment's lock,** io's `Lock` on its root, as an open takes
  it (`opening.md`, 1.1), from the first step to the answer. `Busy`
  refuses `Busy`. The safety checkpoint's open runs under that lock and
  doesn't take it again. Its stores close at the end of 4.3, but the
  root stays open, and so does the lock with it.
- **The source's lock:** `checkpoints/<name>` opened and locked as
  `HoldCheckpoint` does (`checkpoints.md`, 6), before the safety
  checkpoint, so that its retention skips the source (4.7 there).
  `Busy` refuses `Source { why: InUse }`, and an absent name
  `Source { why: Missing }`.
- **The safety checkpoint's lock,** once it is in place, until the
  answer.
- **Let go** on every terminal, in reverse order: the safety
  checkpoint's, the source's, then the deployment's with its root.

### 4.2 The source read

- **Its record,** `checkpoints/<name>/checkpoint`, loaded whole and
  decoded as `Checkpoint` (`checkpoints.md`, 2.3). Its manifest,
  `generations/1/manifest`, is decoded as `Manifest` (2.4 there). Either
  one absent or undecodable refuses `Unreadable`, naming it.
- **Its deployment:** once the safety checkpoint's open has read the
  format row (`records.md`, 3), the record's `deployment` is compared
  with the row's. Another deployment's refuses `Deployment`. The open
  then closes, and nothing has changed.
- **Its files** are those the record lists, at the paths 2.3 there
  gives. Nothing else under `checkpoints/<name>` is read.

### 4.3 The safety checkpoint

- **Taken** as `checkpoints.md`, 5 takes one over a stopped
  deployment, with the request's `draw`. First comes an open for a
  checkpoint, which resolves an outstanding intent (5) as every open
  does and refuses as that open refuses. Then `Request::Checkpoint`.
  Then the stores close.
- **Refused:** the open's refusal is `Open { why }`, and the
  checkpoint's is `Safety { why }`. Neither has changed anything but
  what a checkpoint's open and its failure leave (`checkpoints.md`,
  4.8 and 5).
- **`active` as read:** the open has read `active` and its digest
  (`opening.md`, 1.2). The deployment's lock has been held since, and a
  checkpoint's open switches nothing, so N and that digest still stand
  at 4.6.
- **With which build:** the open refuses a store ahead of the build
  (`opening.md`, 1.8, `Ahead`) and one it must upgrade first
  (`checkpoints.md`, 5, `Unready::Upgrade`), so a restore runs with a
  build at the deployment's levels as it stands. The generation it
  stages holds the source's state at the source's levels, so an older
  build whose levels those are opens it as it was. A redeploy's going
  back after the upgraded generation's first commit restores with the
  new build, then starts the previous one (temper's `redeploy.md`, 4.6).

### 4.4 The staged generation

`generations/<N + 1>/` is made as an upgrade makes `A + 1` from `A`
(`opening.md`, 2, step 2). Any entry already there is first removed,
as a stray is (`opening.md`, 1.5). Then `store/`, `store/logs/` and a
private `secrets/` are made in it, each directory durable as io makes
it. Then the source's files, in the record's order, on the store's
`checkpoints` file lane, one file request in flight
(`checkpoints.md`, 4.5):

| The record's file | Staged how |
|---|---|
| the snapshot, each segment | linked: skein-kv never writes a named snapshot or a sealed segment again, and the restored store's recovery starts a new segment (skein's `data/kv.md`, 3 and 5) |
| a log the checkpoint linked (`way` 1) whose file's length, by io's `Stat`, is its record's `length` | linked: an ended task's turns log, which nothing appends to (`checkpoints.md`, 4.3) |
| every other log | copied up to its record's `length`, its extent at C. The restored store appends to it past that extent (a live task's turns, every history log), and a linked file would change the checkpoint. A linked log is never cut to end at its extent |
| the secrets' snapshot and segments | copied, each file `0o600` with one link, as an upgrade copies `secrets/` |

- **A link the file system refuses** is that file copied instead, and
  counted as a copy (`checkpoints.md`, 4.5, step 3).
- **A file the record names** that is absent, or shorter than its
  `length`, refuses `Staging`, naming it.
- **Its manifest,** written afresh by io's durable replace expecting no
  file: `generation` N + 1, `from` none, `before` and `after` both the
  source manifest's `after`, `counts` empty, `store` the record's
  `store`, `secrets` its `secrets`, `written` env.wall as staging ends.
  With no `from`, `--back` refuses the restored generation
  `NoPrevious` (`opening.md`, 5). Going back assumes the transcripts
  are as the previous generation left them, and a restore has changed
  them.
- **Synced** deepest first: `store/logs/` and each directory in it,
  `store/`, `secrets/`, the generation's directory, then
  `generations/`.

### 4.5 The staged tree

`transcripts.staged/` is made `0o700`, durable as io makes it, after
any entry of that name is removed as a stray. Each transcript the
record lists is then placed at `transcripts.md`, 1's path, each
directory made where it is missing:

| The record's transcript | Staged how |
|---|---|
| linked (`way` 1) | linked: an ended task's file is not touched again (`transcripts.md`, 4) |
| copied (`way` 2) | copied up to its `length`: the restored open cuts a live task's file to its committed length and writes past it (`transcripts.md`, 2 and 4) |
| missing (`way` 3) | left missing: reads answer it as dropped (`transcripts.md`, 5) |

Each directory is then synced, deepest first, then the deployment's
top.

### 4.6 The intent

Once both are staged and synced, `restore` (2.2) is written by io's
durable replace expecting no file, which syncs the top. An existing
one can't stand there: the safety checkpoint's open resolved it (4.3).
Until this replace is durable, the restore has changed nothing live.

### 4.7 Rolled forward

Each step is made durable before the next begins. Each one can be
taken again from what stands after a cut (5):

1. `transcripts/` renamed to `transcripts.aside/` by io's `Rename`,
   then the top synced.
2. `transcripts.staged/` renamed to `transcripts/`, then the top
   synced.
3. `active` replaced by io's durable replace, expecting the intent's
   digest, to name N + 1.
4. `restore` removed, then the top synced.
5. `transcripts.aside/` removed, its tree deepest first as a stray's
   is, then the top synced. The safety checkpoint holds the old
   transcripts.

Then the locks are let go (4.1), and the request is answered
`Restored`. The next start opens N + 1 as any start opens a
generation. A build at later levels upgrades it into N + 2, and
`--back` may then return to N + 1 (`opening.md`, 2 and 5). Generation
N stays as N + 1's previous by number until an upgrade drops it
(`opening.md`, 2, step 1). Nothing names it again: `--back` follows a
manifest's `from`, and N + 1's names none.

### 4.8 When it fails

- **Before the safety checkpoint** (`Busy`, `Source`, `Unreadable`,
  `Deployment`, `Open`, `Safety`): nothing has changed beyond what the
  open and the checkpoint's own failure leave.
- **While staging** (`Staging`): the staged generation and tree are
  removed where that can be done. What remains is a stray to the next
  open. The safety checkpoint stays, a checkpoint as any other. Nothing
  live has changed.
- **After the intent** (`Interrupted`): the intent stays. The next open
  rolls forward from what stands (5): a start, a checkpoint's open,
  another restore's safety checkpoint, or `--back`.

## 5. At open

**Every open of either purpose (`Serve`, `Checkpoint`), and `Back`,**
resolves an outstanding intent once it holds the deployment's lock
and before it reads `active` (`opening.md`, 1.1 and 1.2). So it comes
before recovery and before the strays are removed. With `restore` standing, the
open loads it, decodes it, and reads what stands: `transcripts.staged/`
(s), `transcripts/` (t), `transcripts.aside/` (a), and `active`, whose
digest is either the intent's (d) or not, in which case `active` may
name the intent's generation (g).

| s | t | a | `active` | It goes on from |
|---|---|---|---|---|
| yes | yes | no | d | 4.7, step 1 |
| yes | no | yes | d | step 2 |
| no | yes | yes | d | step 3, once the intent's generation's manifest is read and names it |
| no | yes | yes | g | step 4 |

- **Then the open goes on** from step 5's end, reading `active`, now
  N + 1, as `opening.md`, 1.2 does.
- **Anything else,** or an intent that doesn't decode, refuses
  `Generations::Unmatched` and changes nothing. Something outside the
  engine changed the directory during a restore. In `opening.md`,
  1.8's words: "`<deployment>` holds an unfinished restore whose files
  no longer match its intent, `restore`, and this start has changed
  nothing in it. Something outside the engine changed them: restore
  `<deployment>` from a backup, or copy one of its checkpoints into
  place, and start again."
- **The strays** (`opening.md`, 1.5): with no intent standing,
  `transcripts.staged/`, `transcripts.aside/` and the intent's
  temporary are removed, each tree deepest first as a stray
  generation's is (a transcript's depth allowed), then the top synced.
  A staged generation that no intent named is a stray generation
  already.
- **A rehearsal** (`opening.md`, 5) writes nothing in the directory it
  is given, so it can't roll forward. A directory holding `restore`
  refuses `Generations::Unfinished` before it copies anything:
  "`<deployment>` holds a restore that has not finished, and this
  rehearsal has changed nothing in it. Start the deployment, or take a
  checkpoint of it, which finishes the restore, then rehearse again."
- **`HoldCheckpoint` and `RemoveCheckpoint`** need no open and read
  only `checkpoints/`, so an intent doesn't concern them. An operator
  may remove the safety checkpoint an outstanding intent names. The
  roll forward needs none of its files.

## 6. Power loss

Every write, sync, link and rename of 4 is a cut point. After each
one, the next open finds one matching pair, the old maps with the old
transcripts or the restored maps with the restored transcripts, and
nothing else:

| Cut during | What the next open finds | What it does |
|---|---|---|
| the safety checkpoint | its temporary, or it whole | sweeps the temporary (`checkpoints.md`, 7); a whole one stays, a checkpoint as any |
| staging the generation or the tree | no intent; part of `generations/<N + 1>/` or of `transcripts.staged/` | removes both as strays; opens N with the live tree |
| the intent's replace, before its rename | no intent; its temporary | as above, and the temporary swept |
| after the intent, before step 1's rename is durable | the intent; s, t, d | rolls forward from step 1 |
| step 1 made, before step 2's is durable | the intent; s, a, d | from step 2 |
| step 2 made, before step 3's | the intent; t, a, d | from step 3 |
| step 3 made, before step 4's removal is durable | the intent; t, a, g | from step 4 |
| step 4 made, during step 5 | no intent; part of `transcripts.aside/` | removes the rest as a stray; opens N + 1 |

- **A cut per write, sync, link and rename,** each run from its own
  start in the store's world (`README.md`, section 9), not their
  product, as `opening.md`, 4 runs its own.
- **A failed file request** at any step is answered as 4.8 says, and
  the next open finds what the cut at that step leaves.

## 7. Bounds

- **Memory:** the source's record, at most `FILES` files
  (`checkpoints.md`, 2.3); one directory page; one copy chunk of at
  most `CHUNK`; the safety checkpoint's own, as `checkpoints.md`, 8
  bounds it, before staging begins.
- **Steps:** each makes one file request or reads one page.
- **io:** one request in flight on the `checkpoints` lane. Nothing
  serves beside a restore.
- **Disk:** the safety checkpoint's copies (`checkpoints.md`, 8), and
  the staged copies: the secrets, every log but an ended task's turns
  log, and the transcripts of tasks live at C. Generation N and the
  old transcripts are held by the safety checkpoint's links until it
  is removed.

## 8. Tests

- **Step tests,** in `jig-protocol-store`: the intent's bytes, with
  golden bytes; `RestoreRefusal::words`; the resolution from each row
  of 5's table, and `Unmatched` from a state outside it and from an
  intent that doesn't decode; the strays with no intent standing; a
  rehearsal refusing `Unfinished`; the staging table of 4.4 over a
  record with each kind of file, and a log whose file runs past its
  extent copied; and 4.3's levels: a checkpoint taken at levels L,
  restored by a build at L + 1 over the store it has upgraded to L + 1,
  leaves the format row (`records.md`, 3) reading L, and a build at L
  then opens the restored store without a refusal, so that a
  redeploy's going back by restore starts the previous build (temper's
  `redeploy.md`, 4.6).
- **The store's world,** `jig-store-world`, on its simulated disk,
  focused. A checkpoint is taken, more is committed, and the checkpoint
  is restored. The next open finds the state at C by the referee: both
  maps, each log to its extent, each transcript to its length. The
  transcripts are the checkpoint's. Restoring the safety checkpoint
  then brings the later state back. Power loss at each cut of 6, and a
  failed file request at each rename, at each link and copy, and at
  the intent: each reopen finds one matching pair, the old or the
  restored, never one map with the other's transcripts. Retention
  during the safety checkpoint skips the locked source. A second
  restore beside a held lock refuses. `--back` after a restore refuses
  `NoPrevious`. A source of another deployment refuses `Deployment`.
- **Fuzzy:** commits, drops and snapshots before a checkpoint and after
  it, then a restore cut at a random file request, over seeds. Each
  reopen recovers exactly the referee's state at C or the state before
  the restore, with that state's own transcripts.

## 9. Open questions

- **A deployment that does not open** (no `active`, `Unreadable`,
  `Missing`) has no state to save, and a restore refuses it at the
  safety checkpoint's open. A checkpoint is laid out as a deployment's
  directory, so it is restored there by copying its files into place,
  as a backup is.
- **The copies again:** every log the checkpoint copied is copied once
  more at staging (4.4). If that proves slow on a store with many
  history logs, a log whose task has ended at C and whose drop
  retention has already scheduled could be linked.
