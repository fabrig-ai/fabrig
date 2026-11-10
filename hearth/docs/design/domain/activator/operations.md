# The activator's operations

| Operation | Asked by | Recovery class | Milestone |
|---|---|---|---|
| place (prepare) | the steward | keyed: the prepare key | M3 |
| trial | the steward | conditional | M3; through GRUB M3b |
| bless, and re-bless | the steward of the slot on trial, or of a blessed slot that is not the default | conditional | M3 |
| fall back | the steward | conditional | M3 |
| roll back | the steward, for a person or an engine | conditional | M3 |
| try live (17) | the steward | conditional, on the slot's prepare key, the running slot, the default and the live state; a repeat by attempt gets the first answer | M3; a container's M5 |
| bless live (18) | the steward running over the live state | conditional | M3 |
| the live fall back (19) | the steward; the activator itself when an action fails | conditional | M3 |
| report facts, and drift | the steward | read only | M3 |
| generate identity | the steward; prepare; bootstrap | idempotent | M3 |
| import secrets | the client as root, in-process | idempotent | M3 |
| hold, and release, a bless | the steward of the slot on trial, for an engine | conditional | M4 |
| run a check | the steward, for operating's checks | idempotent: a reading | M5 |
| restart a unit | the steward, for an engine | keyed by the order's key; conditional on the unit's `InvocationID` | M5 |
| place, trial, bless, hold, fall back and roll back of a container's slot, and removing a container (16) | the host's steward | as the machine's own: place keyed by the prepare key, the rest conditional; remove idempotent | M5 |
| partition and format | bootstrap in-process; the lab as an effect | conditional | M3; M6 as an effect |
| snapshot and prune a class (11) | the activator itself, on the class's period | keyed: the sequence; a prune is idempotent | M5 |
| send, receive (11) | the steward, on the backup's period, or for a peer | send idempotent; receive keyed by the snapshot | M6 |
| restore an entry or a class (12) | the steward, for a person or an engine | keyed by the order's key; conditional on the class subvolume's UUID | M5; from the peer M6 |
| apply, and remove, a fault | the steward of a lab pool host, for the lab's engine | apply keyed by the experiment; remove idempotent | M6 |

## 1. Place

```
Place {
  slot        a | b
  output      Key                          the output's name in the store
  tree        Digest                       its tree digest, as its record gives it
  kernel      { version, image, initramfs, modules, initramfs inputs: Digest }   ../slots/trial.md, 1
  data        entries    [ { name, class, path } ]                                   at most 64
              policies   per class, core or data: { snapshots? { every, keep } [M5],
                         backup? { every, keep, peer, recipient } [M6] }   ../operating.md, section 7
  users       [ name ]                                        package users the tasks declare (../../protocol/format/tasks.md, 6), at most 256
  groups      [ name ]                                        package groups the tasks declare (../../protocol/format/tasks.md, 6), at most 256
  secrets     files       [ { name, path, mode, owner, group } ]                     at most 256
              passwords   [ { user, name } ]                                         at most 64
              identity    [ { item, kind, algorithm, path, public path, mode, owner, group } ]   at most 16
  boot        [ token ]                                       at most 2048 bytes in all
  window      Duration                                        the trial's, sealed into the units (../slots/prepare.md, 5)
  allowed     [ unit ]                                        bless.ignore_failed (../../protocol/format/tasks.md, 13), at most 64
  meta        { kind, firmware }                              checked against the machine record
  parts       { data, secrets, boot: Digest }                 the spec's (../render/spec.md, 1.5)
  expect      { running, default, slot state }
}
```

For a container's slot [M5], `meta` is
`{ kind: container, target, on }`, `on` checked against the machine
record (`README.md`, 2.2); `kernel` is absent, and `boot` holds the container's
parameters instead of tokens:
`{ ids, binds, capabilities, network, bridge? }`, at most 32 binds
(hearth/docs/design/domain/slots/containers.md, 1).

- **Effects:** hearth/docs/design/domain/slots/prepare.md, steps 1 to 7: identity first,
  then the prepare key, which covers the window and the units allowed
  to fail.
- **`window`** is 10 minutes in M3; from M5 the steward computes it from
  the blessing checks (hearth/docs/design/domain/operating.md, section 3). It rides here, not in
  `Try`, because prepare seals it into the slot's units.
- **The part digests are the spec's,** not recomputed in M3: they name
  what the builder's render meant, for the prepare key and the facts; the
  shift's digest is the check (`README.md`, 5.3). Under an engine [M4], an order's
  expected digests are checked against the typed values (`README.md`, 2.6).
- **Done:** the slot, its prepare key, its reported tree digest
  (hearth/docs/design/domain/slots/prepare.md, 7).
- **Refused:** invalid, stale, busy, or the slot is running, the default
  or on trial.
- **Failed:** the output is missing, not read-only or not the builder's;
  the digest differs; an id is out of range; a path that must be a
  directory or a regular file is not; a secret is missing, or readable
  by `hearth`; an owner or a package user is unknown; a parent directory
  is missing; an entry's directory is orphaned in another of the
  target's classes, so the entry changed class (hearth/docs/design/domain/machine/README.md, 3.4); the
  disk is full; a tool failed (which, its exit, its tail). No error
  carries a file's content.
- **A crash** leaves the slot `preparing`; the next start deletes it,
  and the steward asks again.

## 2. Trial

```
Try { slot, attempt, how: auto | full, expect { prepare key, running, default } }
```

- **Effects:** hearth/docs/design/domain/slots/trial.md: the activator chooses a soft or a
  one-shot reboot, writes the marker, arms, answers, reboots. The window
  is the slot's, from its place (1).
- **Done:** the kind chosen. The facts after the reboot tell the rest.
- **A crash** is resolved from the marker's phase (hearth/docs/design/domain/slots/crashes.md).

## 3. Bless

```
Bless { slot, attempt }
```

- **Effects:** hearth/docs/design/domain/slots/bless.md, 1.3: the criterion, then the record says
  `blessed`, the marker goes, the default moves, the dead-man stops. For
  a blessed slot that runs but is not the default, the re-bless of
  hearth/docs/design/domain/slots/bless.md, 1.5: the criterion, then the default moves.
- **Refused** while the criterion does not hold, saying which part; the
  steward asks again, or asks to fall back.
- **Moving the default where it already is writes nothing,** so a bless
  cut anywhere is finished by asking again.

## 4. Fall back

```
FallBack { slot, attempt }
```

- **Effects:** hearth/docs/design/domain/slots/bless.md, section 2: before the reboot, an abort; on
  trial, a full reboot that is never a soft one.

## 5. Roll back

```
RollBack { to, attempt, expect { running, default, record of to } }
```

- **Effects:** hearth/docs/design/domain/slots/bless.md, section 3: a trial of the other blessed slot,
  then its own bless. While a live state runs, it is the live fall back
  (19).

## 6. Report facts, and drift

```
Facts { drift: none | metadata | contents }
```

- **Done:** the facts (`README.md`, 4.3), after running drift's checks at the depth
  asked (hearth/docs/design/domain/slots/bless.md, section 4). Read only: it changes nothing but the
  results it reports.
- **The capture** (hearth/docs/design/domain/development.md, section 5) is the same walk, asked by
  `hearth diff --running`: what differs, by kind, with the contents of
  changed files under `/etc`, `/usr/local` and `/opt` on a mutable slot
  only, and never a secret's or an identity file's.

## 7. Generate identity

```
Identity { target, items: [ { item, kind, algorithm } ] }
```

- **Each item once per target on this machine.** An item that exists is
  answered as it is; a missing one is made in `identity/<target>/.new/`,
  then renamed into place, and the identity's generation is bumped. The
  kinds are a closed list (hearth/docs/design/protocol/format/README.md, 7.3):

  | Kind | Made with | Public half |
  |---|---|---|
  | the machine-id, always | `/proc/sys/kernel/random/uuid`: a random v4 UUID's 32 hexadecimal digits | itself |
  | `ssh-host-key` | the running system's `ssh-keygen -q -t <algorithm> -N "" -C "" -f <.new>/<item>`: the running slot's, from the `hearth` task's `openssh-client` (hearth/docs/design/protocol/format/tasks.md, 12), or the live system's at bootstrap, which checks for it | the `.pub` line |
  | `wireguard-key` [later] | in-process, from the kernel's random source and X25519, an ask of skein: no running slot need carry `wg` | the base64 public key |
  | `backup-key` [M6] | `age-keygen -o <.new>/<item>`: an X25519 key, kept in the identity store and written into no slot (hearth/docs/design/protocol/format/README.md, 7.3 and 6.7) | the `age1…` recipient, reported as a fact |

- **Done:** the generation and the public halves, which the steward
  reports so that people and agents can record them as data in the
  repository (hearth/docs/design/protocol/format/README.md, section 6).
- **A crash** leaves a `.new` directory, removed at the next start.
- **Prepare asks for it first,** before it computes its key
  (hearth/docs/design/domain/slots/prepare.md), so a new item never causes a second prepare;
  bootstrap does the same.

## 8. Import secrets

Run by the client as root, in its own process, with the activator's
domain (hearth/docs/design/client/README.md, section 3), never over the link: no value ever passes
through the steward or its builder.

- **Input:** files named on the command line, or a directory of them,
  whose relative paths are secret names (hearth/docs/design/protocol/format/README.md, section 7), each
  value at most 1 MiB. Identity is never imported: 7 makes it, fresh
  on every machine.
- **Effects:** each value whose digest is new is written as
  `secrets/<target>/<name>@<generation>`, `0600`, through a temporary
  file, synced and renamed; then the index (the store's generation, and
  each name's value file, digest and size) is written with the
  generation bumped once for the command, conditional on the digest of
  the index it read (skein's `Store`), so two imports never lose each
  other's writes. The previous generation's files stay until the next
  import, since a running activator may still be reading through the
  previous index; that import removes what neither index names.
  Removing a secret is an index without its name.
- **Idempotent:** an import that changes nothing bumps nothing.
- **Readers read the index, then the files it names,** so they always
  see one consistent generation. The running activator reads the index
  before every prepare and every facts report; the client then asks the
  steward to refresh its facts (hearth/docs/design/client/README.md, section 3).
- **A crash** leaves value files no index names, removed by the next
  import.

## 9. Restart a unit [M5]

```
RestartUnit { unit, invocation }
```

- **Keyed by the order's key,** which the steward keeps with the unit's
  `InvocationID` before it asks (hearth/docs/design/protocol/wire.md, 4.4), and **conditional on
  that invocation** at the activator: `systemctl show
  --property=InvocationID --value <unit>` must still equal it, then
  `systemctl restart <unit>`. Done when the invocation has changed, so
  an uncertain restart is resolved by reading it again.
- **Never hearth's own units** (`hearth-activator.service`,
  `hearth-trial*`, `hearth-check-*`); that the unit is one the running
  slot's spec ships or enables is the steward's check
  (hearth/docs/design/domain/operating.md, section 6).
- **For a container's target [M5],** the same operation with the
  container's target named, `RestartUnit { target, unit, invocation }`:
  the unit is one inside the container, its `InvocationID` read with
  `systemctl --machine=<container> show --property=InvocationID --value
  <unit>`, then `systemctl --machine=<container> restart <unit>`. The
  container's own nspawn unit is never named: only the host's activator
  starts and stops it (16).

## 10. Partition and format

- **M3: bootstrap only,** run in-process by the client as root
  (hearth/docs/design/domain/machine/bootstrap.md, 2): wipe, partition, format, make the top level.
- **M6: an effect for the lab** (hearth/docs/design/domain/machine/bootstrap.md, 5):

  ```
  Disk { device id, firmware, fs uuid, boot volume id }
  ```

  conditional on a blank disk (no partition table, no signature), or on
  one that already holds exactly these identifiers, which is then done.

## 11. Snapshots [M5] and backups [M6]

```
Snapshot { target, class, seq }      the activator's own, on the class's period
Send { target, class, seq, parent }      Receive { from, class, seq }      [M6]
```

- **Snapshot,** by the activator itself: when a class's deadline comes
  (hearth/docs/design/domain/operating.md, 7.1), a read-only snapshot of `@<target>.<class>` into
  `@hearth.snapshots/<target>.<class>/<seq>` (`BTRFS_IOC_SNAP_CREATE_V2`
  with the read-only flag), atomic and crash-consistent. The policy is
  the running slot's record's (hearth/docs/design/domain/slots/README.md, 2.2), never a request's; a
  container's class follows its host's (hearth/docs/design/domain/slots/containers.md, 1). Keyed by
  `<seq>`, which only grows: a snapshot of that sequence means done.
  Then the class's record (`README.md`, 6.1) gains it, and the next deadline is set.
- **Prune,** after every snapshot: each snapshot that no bucket keeps
  and the backup does not need (hearth/docs/design/domain/operating.md, 7.3) is deleted with
  `SNAP_DESTROY_V2`, then dropped from the record. Idempotent: a
  snapshot already gone is done. No request can prune: the steward
  never decides what a machine's history loses.
- **Send** [M6]: `btrfs send -p <parent> <seq>`, or without `-p` the
  first time, piped into `age --encrypt --recipient <recipient>` before
  any byte reaches the steward, then handed to it over a pipe made for
  the transfer, never the link. The recipient is the peer's backup key,
  from the class's policy in the sending slot's record (hearth/docs/design/domain/slots/README.md, 2.2),
  never from the request, so the steward cannot redirect a stream to a
  key of its own. Idempotent: a send the peer has not confirmed is sent
  again. The semantics are hearth/docs/design/domain/operating.md, 7.2's.
- **Receive** [M6], on the backup peer: the ciphertext from its steward,
  piped into `age --decrypt -i <its backup key>` and then `btrfs
  receive` into `@hearth.backups/<from>.<class>/`, accepted only from a
  target the peer's spec names; what the peer keeps at rest is
  hearth/docs/design/domain/operating.md, 7.3's. Keyed by the received snapshot: one already
  there, read-only, is confirmed again; a partial one is deleted and
  received again.

## 12. Restore [M5]

```
Restore { target, class, entry?, seq, expect { uuid } }
```

- **Keyed by the order's key and conditional on the class subvolume's
  UUID** being the one decided from. With `entry`, one entry's
  directory is copied back out of the snapshot and swapped in by name;
  without, the class's subvolume is swapped for a writable snapshot of
  it. Both then reboot, or restart a container. The steps are
  hearth/docs/design/domain/operating.md, 7.4's; the names, hearth/docs/design/domain/machine/README.md, 3.1 and 3.4's.
- **The copy is the activator's own walk** through descriptors, with
  `FICLONE` (hearth/docs/design/domain/slots/prepare.md, 3): no `cp`, and nothing resolved outside the
  snapshot and the class.
- **A snapshot the machine lacks** [M6] arrives from the peer encrypted
  to this target's own backup key; the activator pipes `age --decrypt -i
  <its backup key>` into `btrfs receive`, into its own
  `@hearth.snapshots`, so plaintext exists only between two activators.
- **A cut between the renames** is finished or undone at the next start
  from the recorded swap (`README.md`, 6.3).

## 13. Hold, and release, a bless [M4]

```
HoldBless { slot, attempt, until }        Bless { slot, attempt }        releases it
```

- **Conditional** on the slot running on trial, with that attempt, and
  the minimal criterion holding. What a hold does (the marker `held`, a
  transient timer at `until`, the default unmoved) and how it ends are
  hearth/docs/design/domain/slots/bless.md, 1.4's; a `Bless` releases it as in 5.3.
- **A crash** finds the marker `held` and its transient timer still
  armed: nothing to redo.
- **A container's [M5]** is held the same way, with no timer: its end is
  the activator's own deadline, as a container's window is (16).

## 14. Run a check [M5]

```
RunCheck { task, name, user, capabilities, network, timeout, argv }
```

- **systemd runs it, not the activator:** a transient unit,
  `hearth-check-<task>-<name>`, through `systemd-run --wait --collect`
  with the fixed properties of hearth/docs/design/domain/operating.md, 2.4, as the declared user
  (`nobody` by default, never `root`), the file system read-only, no
  network unless declared, and memory, tasks and time bounded. The
  activator builds the argument vector from the typed check and renders
  nothing.
- **Secrets stay out of reach:** `InaccessiblePaths=` names
  `/var/lib/hearth/state`, `/run/hearth-activator` and every secret and
  identity path the running slot's record holds, so a check cannot read
  them even as a user their owners would allow.
- **The program is the running slot's,** a system the machine already
  runs, never a program of a slot being prepared or of the store.
- **In a container [M5],** the same unit is started inside it with
  `systemd-run --machine=<container>`, its `InaccessiblePaths=` the
  container slot's secret and identity paths (hearth/docs/design/domain/operating.md, 2.9).
- **Done:** `pass` (exit 0), `fail` (another status, or the timeout) or
  `error` (the unit could not start), with the unit's `Result`, and
  nothing else: no output reaches the steward, which could otherwise
  read a file through a check.
- **Idempotent:** checks only read until hearth/docs/design/domain/operating.md, 2.8 says
  otherwise, so an uncertain run is run again.

## 15. Faults [M6]

```
Fault { experiment, kind, targets, parameters, until }        Unfault { experiment }
```

- **Only on a lab pool host,** as the target's own spec says: a target
  in the lab's environment with containers placed on it, whatever an
  engine says; refused everywhere else (hearth/docs/design/domain/operating.md, section 8).
- **Kinds, each a fixed operation with typed parameters,** never a
  command line from the engine; their mechanisms (nftables, `tc netem`,
  `systemctl --machine kill`, a qgroup limit) are
  hearth/docs/design/domain/operating.md, section 8's.
- **Apply is keyed by the experiment:** a fault already applied under
  that key is found. **Remove is idempotent.**
- **Every fault ends at `until`,** which the activator keeps and
  enforces itself, after a restart too, so a fault ends even when the
  engine is gone.

## 16. A container's slots [M5]

Place, trial, bless, hold, fall back and roll back (1 to 5 and 13) on
a container's slot, and removing a container, asked by the host's
steward with the container's target (`README.md`, 2.2). The effects are
hearth/docs/design/domain/slots/containers.md's; none touches the boot partition or NVRAM.

| Operation | Effects | Recovery class |
|---|---|---|
| place | `operations.md`'s steps with a container's differences (hearth/docs/design/domain/slots/containers.md, 1) | keyed: the prepare key |
| trial | the marker; the slot's nspawn settings and link, each by rename; `systemctl restart systemd-nspawn@<container>.service`; the poll (hearth/docs/design/domain/slots/containers.md, 2 and 3) | conditional on the container's running slot, default and the slot's prepare key; a repeat by attempt gets the first answer |
| bless | the criterion, read from the host; the record `blessed`; the marker removed; the default recorded (`README.md`, 6.1) | conditional |
| hold, and release | the marker `held`, with the hold's end as the activator's deadline in place of the window's; a `Bless` releases it (hearth/docs/design/domain/slots/bless.md, 1.4) | conditional |
| fall back | the default's settings and link back, a restart, the record `fell back` | conditional |
| roll back | a trial of the other blessed slot; while live, the live fall back | conditional |
| try live, bless live, the live fall back | hearth/docs/design/domain/slots/containers.md, 8: the delta on the host's top level, the swap by a child that enters the container's mount namespace, the units through `systemctl --machine=`, the live bless recording the container's next start as `b`'s trial | as the machine's own |
| remove | `systemctl stop` on its unit; its link and settings removed; its slots' subvolumes, records, manifests and marker deleted, then its record; its class subvolumes, their snapshots, its identity and secrets kept (hearth/docs/design/domain/slots/containers.md, 7) | idempotent: a container with no record is done |

- **A trial is answered after its restart,** not before a reboot: done
  once the unit is stably active, the slot `on trial`; failed once it is
  not, the container back on its default.
- **The window is the activator's own deadline,** kept in the marker:
  at its end with no bless, the activator falls the container back
  itself, after its own restart too, since no dead-man runs in a
  container. A held bless's ceiling is kept and enforced the same way.
- **Remove names a container the host's spec no longer places on it;**
  the steward checks that against its builder's render, as it checks a
  unit against the spec (9), and the activator refuses the host's own
  target.

## 17. Try live

```
TryLive {
  slot, attempt
  change      Digest                         the change from the base's output to the new slot's (../slots/trial.md, 7), in ../store/README.md, 3.5's form
  plan        stop                [ unit ]   before the swap
              reload              [ unit ]
              restart             [ unit ]
              reload or restart   [ unit ]
              start               [ unit ]
              then                { daemon reload, tmpfiles, dbus, accounts: bool }
  expect      { prepare key, running, default, live: delta | none }
}
```

- **Effects:** hearth/docs/design/domain/slots/trial.md, 8, L1 to L6: the change recomputed and
  checked (`README.md`, 5.5), the delta built and verified, the base's dead-man
  armed, the units stopped, the hierarchies swapped, the units acted on.
  The rule-11 units are added to the restarts.
- **The plan is a builder's,** computed in a `plan live` job the steward
  asked (hearth/docs/design/domain/steward.md, 3.3); the steward sends it, and the activator
  checks again what it can read (`README.md`, 5.5).
- **Done,** after L6: the plan as acted on and each job's result; then
  the activator spawns a new steward (`README.md`, 3.3), whose `Welcome` shows the
  slot `on live trial`.
- **Refused:** invalid, stale, busy; not eligible, naming the rules of
  hearth/docs/design/domain/slots/trial.md, 7 that fail, and the steward then asks `Try`.
- **Failed:** at L1 or L2, the delta deleted and nothing on the machine
  changed; at L3 to L5, the stopped units started again and the trial
  disarmed, the slot `prepared`; at L6, a job failed and the live fall
  back ran (19), the answer saying whether it held or ended in
  `reboot -ff`.
- **A crash** is resolved from the journal and the marker's phase
  (hearth/docs/design/domain/slots/crashes.md): before arming, the delta deleted; after,
  `reboot -ff`.

## 18. Bless live

```
BlessLive { slot, attempt }
```

- **Effects:** hearth/docs/design/domain/slots/bless.md, 1.6: the checks, the record `live-blessed`, the
  one-shot to the slot's trial entry, the marker's phase, the base's
  dead-man stopped. The default does not move.
- **Refused** while the settle, a unit or the criterion does not hold,
  saying which part; the steward asks again, or asks to fall back.
- **Asking again** with the attempt finds it done; a cut is finished at
  start from the journal.

## 19. The live fall back

```
FallBack { slot, attempt }        on a slot on live trial, or live-blessed
```

- **Effects:** hearth/docs/design/domain/slots/bless.md, 2.5: the units L6 started stopped, the swap
  back, the actions replayed under the old files, every job checked,
  then the dead-man stopped and the slot `fell back`. For a
  `live-blessed` slot, its one-shot is cleared and the base's dead-man
  armed again first.
- **Run by the activator itself** inside `TryLive` when an action of L6
  fails, and asked by the steward otherwise; a roll back while live is
  this (5).
- **Any failed step is `reboot -ff`,** and the next start records the
  outcome. Done once the files that ran before run again and every job
  held.
