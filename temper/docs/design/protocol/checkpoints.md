# Checkpoints

2026-11-16. temper's part of taking a checkpoint (`draft/checkpoints.md`,
sections 3.4, 4 and 5.1; milestone C1 of
`draft/checkpoints.execution.md`): the deployment's binaries' commands,
the owner's act through `temper ctl`, and the setting. What a checkpoint
is and how the store takes it are jig's `protocol/store/checkpoints.md`.

## 1. In one page

- **Over a stopped deployment:** `temper-engine checkpoint
  [DIRECTORIES]`, and `temper-box checkpoint` of the same words (2.1).
- **While the engine serves:** `temper ctl DIR deployment checkpoint
  [--timeout DURATION]`, an act of a person who owns the deployment
  (3).
- **Readers:** `checkpoint --hold NAME -- COMMAND` runs a command under
  the checkpoint's lock (2.2); `checkpoint --remove NAME` removes one by
  name, refused while it is held (2.3).
- **The setting:** `store.checkpoints.keep`, default 2 (4).

## 2. The commands

On both deployment binaries, `binaries.md`'s rows, run by the same code,
`temper-engine-shell`'s, as `upgrade` is (`binaries.md`, 2), its lines
naming the binary that runs it. Each reads the configuration `serve`
would read (`DIRECTORIES`) for the deployment's directory and the
store's limits, and drives jig's store host alone, as `upgrade --check`
does: no worker, no listener, no root.

### 2.1 `checkpoint [DIRECTORIES]`

- **Takes one** of the stopped deployment: the store host's
  `Adoption::Checkpoint` (jig's `protocol/store/checkpoints.md`, 5),
  with 16 bytes drawn as the start draws its installation's identifier.
  It holds the deployment's lock throughout.
- **Prints one line and exits 0:**
  `temper-engine checkpoint: NAME at commit C, secrets at S: L files linked, N copied (B bytes) in T ms; H bytes held; R older removed, K skipped`,
  each figure `Checkpointed`'s (jig's `protocol/store/checkpoints.md`,
  3): `linked`, `copied`, `bytes`, `took` in whole milliseconds,
  `held`, `removed`, `skipped`.
- **Refused, exit 1, one line:**
  - the deployment held by another process, the engine serving it:
    `temper-engine checkpoint: DIRECTORY is in use by another process; while the engine serves, take one with temper ctl DIR deployment checkpoint`;
  - an open's refusal (`Unready`, or any other of the open's): its
    words, as `serve` prints a refused start;
  - a checkpoint's refusal: `temper-engine checkpoint: refused: WORDS`,
    `CheckpointRefusal::words`.

### 2.2 `checkpoint --hold NAME [DIRECTORIES] -- COMMAND [ARGUMENT...]`

- **Locks** the checkpoint NAME (`Adoption::HoldCheckpoint`; jig's
  `protocol/store/checkpoints.md`, 6), then runs COMMAND with its
  arguments as a child, in the working directory it was run from, with
  its environment and `TEMPER_CHECKPOINT`, the checkpoint's directory's
  absolute path; waits for it; lets the lock go; and exits as the child
  did: its status, or 128 and the signal's number when a signal ended
  it. A COMMAND that can't be run exits 127, naming it.
- **Refused, exit 1, before COMMAND runs:** `temper-engine checkpoint:
  NAME is in use` (its lock held) and `temper-engine checkpoint: no
  checkpoint NAME`.
- **While the engine serves** too: it takes no deployment lock, and
  retention skips the checkpoint while it holds it.

### 2.3 `checkpoint --remove NAME [DIRECTORIES]`

- **Removes** the checkpoint NAME (`Adoption::RemoveCheckpoint`), while
  the engine serves or not; prints `temper-engine checkpoint: removed
  NAME`, exit 0.
- **Refused, exit 1:** in use, or no checkpoint NAME, in 2.2's words.

### 2.4 Their arguments

- **NAME** is a checkpoint's name as its answer prints it, `<20
  digits>-<16 hex>`; any other is a usage error, exit 2.
- **`--hold` and `--remove`** exclude each other; COMMAND follows `--`
  and only with `--hold`.

## 3. The act

`temper ctl DIR deployment checkpoint [--timeout DURATION]`, a row of
`../client/ctl/tools.md`, 3.

- **Its request** is temper's client family's (`../client/forge/records.md`):
  `Checkpoint`, with no field, keyed as an act is, the key optional. It
  moves `CLIENT_VERSION` by one, unbridged, as a client's wire moves
  (`draft/redeploys.md`, 8).
- **Its outcome:**

```
record Checkpointed {
    name: text 37      # <20 digits>-<16 hex>
    store: u64
    secrets: u64
    linked: u32
    copied: u32
    bytes: u64
    paused: u64        # nanoseconds
    took: u64          # nanoseconds
    held: u64
    removed: u32
    skipped: u32
}
enum CheckpointWhy { NotOwner, InProgress, Unavailable, Files, Unrecognized { path: text 128 }, Vanished { path: text 128 }, Failed { path: text 128, error: text 128 } }
```

  The figures are the store's `Checkpointed`'s (jig's
  `protocol/store/checkpoints.md`, 3), and the refusals its
  `CheckpointRefusal`'s that a serving engine can meet, with
  `NotOwner`.
- **The root:** admitted as an application of the person's own scope
  (`ApplicationScope::Own`: the deployment's scope would refuse a
  person with no role over the deployment with jig's generic `Role`
  before the root sees it), then held by the root to the deployment's
  owners alone: a person whose role over
  the deployment is `Owner` (`people::DEPLOYMENT`), never a project's
  owner, and never a task's party, refused `NotOwner` with nothing
  asked of the store. Then it asks the store, `StoreRequest::Checkpoint`,
  keeps the admitted request until the store's terminal returns as
  `StoreInput::Checkpointed` or `StoreInput::CheckpointRefused`, and
  answers it then, finishing the application, as `Reachable`'s answer
  waits for its forge (`domain/root.md`).
- **`draw`** is drawn where the engine's service routes the request to
  the store, from the service's randomness, never from the commit or the
  clock.
- **Printed:** `checkpoint NAME at commit C, secrets at S: L files linked, N copied (B bytes); paused P ms, took T ms; H bytes held; R older removed, K skipped`, exit 0;
  `--json` gives the outcome's fields by their names. A refusal exits 2,
  `temper ctl: checkpoint refused: WORDS`, `NotOwner`'s words "only an
  owner of the deployment takes a checkpoint", the others the store's.
- **Its wait** is its own: `--timeout DURATION`, 10 minutes when none
  is given, in place of a call's ordinary deadline
  (`../client/ctl/README.md`), since a checkpoint links and copies for
  seconds to minutes. Past it, it exits 3, `temper ctl: no answer within
  DURATION; the checkpoint goes on in the engine`, and the engine goes
  on.

## 4. The setting

- **`store.checkpoints.keep`:** how many of the newest checkpoints the
  store keeps, default 2, an override of jig's limits
  (`engine.limits.overrides`, `configuration/limits.md`; jig's
  `protocol/limits.md`), from 1 to 64.
- **It reaches the store** with the rest of its limits: the engine's
  store limits carry it as each other field
  (`configuration/README.md`, section 2), and so do the commands of 2,
  which read the same configuration.

## 5. Tests

- **Step tests:** the commands' rows and arguments parsed, `--hold`
  and `--remove` excluding each other, a NAME of another form refused;
  their lines from given answers; the family's request and outcome with
  golden bytes; the root admitting an owner's act, asking the store and
  answering only at its terminal, refusing a project owner and a task
  `NotOwner`; the tool's text and JSON, and its timeout; the setting's
  default, an override of 3, and 0 and 65 refused.
- **The engine's world:** the act through the root over its store, an
  owner's answered from the store's terminal, a non-owner's refused.
- **The process tier,** written and compiled, run after landing: a
  checkpoint over a stopped deployment, then `--hold` with a command
  that lists it, then `--remove`; the act while the engine serves.

## 6. Open questions

- **Restore** and the redeploy with checkpoints are milestones C2 and
  C3 (`draft/checkpoints.execution.md`).
- **A list of checkpoints:** none of the commands lists them; `ls`
  of `checkpoints/` and each one's record serve until one does.
