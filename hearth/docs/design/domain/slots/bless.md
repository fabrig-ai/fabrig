# Bless, fall back, roll back and drift

## 1. Bless

### 1.1 The minimal criterion (M3)

- **The boot reached its default target with no failed unit that
  counts:** `systemctl is-system-running` answers `running`, or
  `degraded` with every failed unit (`systemctl list-units --state=failed`,
  parsed by `hearth-protocol-systemd`) on the target's
  `bless.ignore_failed` list (hearth/docs/design/protocol/format/tasks.md, 13), which the place request
  carries. `starting` means not yet; a failed unit that counts means the
  steward asks to fall back rather than wait out the window. A desktop
  such as `x1-prod` often carries a routinely failed unit; comparing
  against a baseline of the previous slot is later, once measured there.
- **The slot's own steward is up and has answered its activator, and
  a builder it spawned has answered the bless's render** of the slot's
  own revision (hearth/docs/design/domain/steward.md, section 6): the bless request arrives over
  the link from the steward the slot's activator spawned, after its
  hello and after that render. The activator checks the rest of the
  criterion itself (1.3); the render it cannot see, and takes the
  request as its proof. A render that fails makes the steward ask to
  fall back instead (hearth/docs/design/domain/steward.md, 5.1).
- **A host's containers never count [M5]:** the failed units counted
  leave out the nspawn unit, `systemd-nspawn@<container>.service`, of
  every container the host's activator keeps records for
  (hearth/docs/design/domain/activator/README.md, 6.1), with no list to write. Each container's slot has
  its own trial and its own facts (`containers.md`, 3); one bad container must not
  roll its host back.
- **Task checks join in M5** (hearth/docs/design/domain/operating.md, section 3).

### 1.2 Who asks

- **The new slot's hearth blesses itself:** its steward's converge asks
  its own activator, once the link is up and the criterion holds. The
  old slot's hearth never blesses the new one: it is not running.
- **A broken hearth release cannot bless itself.** If the new steward
  never starts, never connects or never asks, the dead-man ends the
  trial, and the default never moved. If its builder cannot render the
  slot's own revision, an error or a second loss, the steward asks to
  fall back at once (hearth/docs/design/domain/steward.md, 5.1's table). No outside probe judges
  the slot: the bash fleet once lost a good boot to an SSH probe that
  never reconnected.
- **A container is the exception [M5]:** no hearth runs in it, so its
  host's steward asks its host's activator, which reads the container
  from outside (`containers.md`, 3), with no bless's render: the host's hearth did not
  change (hearth/docs/design/domain/steward.md, section 6).
- **A live trial is blessed by the base's steward,** spawned anew after
  the swap so that it starts in the new files, and whose builder runs
  the bless's render (1.6); the new slot blesses itself only at its
  native boot (`trial.md`, 8, L9).

### 1.3 What it does

1. **Check:** the marker names this slot and attempt; the slot runs; the
   criterion holds (the activator checks it itself).
2. **Journal** the bless (hearth/docs/design/domain/activator/README.md, 6.3).
3. **Write the slot record** as `blessed`.
4. **Remove the marker,** and `/run/hearth-trial/x1-prod.b` if a soft
   reboot left one; after a live trial, delete the delta it names
   (`trial.md`, 8, L9).
5. **Move the default:** `efibootmgr --bootorder 4801,4800,…`, read back
   (EFI); or, on BIOS, `previous` set to the old default, `default` to
   `x1-prod.b` and the boot counter reset, in `env.0`, then `env.1`
   (2.4).
6. **Stop the dead-man:** `systemctl stop hearth-trial.timer
   hearth-trial.target`.

- **The default moves last but one,** so a cut never leaves the default
  on a slot still `on trial`, which would re-arm at every boot. A cut
  before step 5 leaves a blessed slot that is not the default, which the
  re-bless finishes (1.5); a cut before step 6 leaves a dead-man that
  reboots once into a blessed default.
- **The other slot stays `blessed`,** second in `BootOrder` on EFI and
  `previous` on BIOS: the roll back's target, and the firmware's or
  GRUB's fallback, until the next prepare takes it.

### 1.4 Holding a bless [M4]

An engine may hold a bless while a canary bakes (hearth/docs/design/protocol/wire.md, 4.4: an
`activate` with `bless: held for at most d`):

- **The slot's steward asks to hold** once the minimal criterion holds
  (hearth/docs/design/domain/operating.md's checks too, from M5). The activator moves the marker to
  phase `held`, with the hold's end, and replaces the dead-man's timer by
  a transient one ending at the hold's end, `systemd-run
  --on-active=<d> /usr/bin/systemctl reboot -ff`, so the end does not
  depend on hearth either. The default does not move.
- **The engine's `bless` releases it:** the bless of 1.3, conditional on
  the slot on trial, healthy and held. Its `roll back`, or the hold's
  end, falls back (section 2).
- **While held, the slot runs on trial,** not the default: any reboot or
  power cut lands on the old default, which is what an unbaked canary
  should do, and the next start records `fell back`.
- **The table gains one state,** `held`, between `on trial` and
  `blessed`, with `on trial`'s cells but for `bless` (released: `blessed`)
  and `hold` (same attempt: done).
- **A container's bless is held the same way [M5]:** the engine may hold
  it, and the `bless` order releases it. Its end is the host's
  activator's deadline, kept in the marker as the window is (`containers.md`, 3): at
  the hold's ceiling the activator falls the container back, a restart
  on its old slot.
- **A live trial's bless is held the same way:** the marker's phase
  `held`, the base's dead-man replaced by the hold's transient timer;
  the `bless` order releases it into the live bless (1.6), and the
  hold's end reboots into the default, which drops the overlays.

### 1.5 Re-bless

- **The running slot is blessed but not the default** after a bless cut
  before it moved the default, after a firmware fell through to it, or
  after GRUB's counter booted it (2.4). The facts say so.
- **Its steward asks to bless it again,** a stage of converge's pass
  (hearth/docs/design/protocol/standalone.md), with no bless's render: the slot's hearth was
  proven when it was first blessed (hearth/docs/design/domain/steward.md, section 6). The
  activator checks the criterion as for a trial, but needs no marker:
  the slot is already blessed, and it runs.
  Then it moves the default to it (1.3, step 5), and writes nothing else.
- **Until then, prepare is refused** (`README.md`, 2.3), so no slot is written while
  the firmware would boot another.
- **A live state never calls for one:** the default never leaves the
  base, which runs, so "running and blessed but not the default" does
  not arise. Had the live bless moved the default, this rule would move
  it back to the base at the next start.

### 1.6 The live bless

The bless of a live trial (`trial.md`, 8, L8), asked by the steward on the link,
the base's, spawned after the swap, once its builder has answered the
bless's render (hearth/docs/design/domain/steward.md, section 6):

1. **Check:** the marker names this slot and attempt, in phase
   `running`; the live fact agrees with the live record; every unit L4
   to L6 touched is active with the `InvocationID` and `NRestarts` L6
   read; the settle has passed; the criterion holds (1.1); from M5, the
   blessing checks pass (hearth/docs/design/domain/operating.md, section 3).
2. **Journal** the live bless (hearth/docs/design/domain/activator/README.md, 6.3).
3. **Write the slot record** as `live-blessed`.
4. **Set the one-shot to the slot's trial entry:** `efibootmgr
   --bootnext 4811`, read back (EFI); or `next_entry=x1-prod.b` in
   `env.0`, then `env.1`, from which GRUB adds the trial's token (BIOS,
   hearth/docs/design/domain/machine/boot.md, 2.3) [M3b].
5. **Set the marker's phase to `live-blessed`,** and delete the previous
   delta, if any (`trial.md`, 9).
6. **Stop the base's dead-man:** `systemctl stop hearth-trial.timer`,
   and remove `/run/hearth-trial/x1-prod.a`.

- **The default does not move.** `a` stays first in `BootOrder`, or
  `default` on BIOS, until `b` has booted natively and blessed itself
  (`trial.md`, 8, L9). A live bless that moved it would make `b` the default with
  no native boot ever tried: should `b` then fail in user space, no
  token and no `/run` marker would arm its dead-man, and the firmware
  falls back only when a kernel fails to load.
- **Cuts:** a crash after step 3 is finished at start from the journal.
  A power cut before step 4 leaves `b` `live-blessed` with no one-shot,
  after a boot that dropped the overlays: the start makes it `prepared`
  again (`README.md`, section 4). After step 4 the power-on boots `b`'s trial. A
  crash before step 6 leaves the base's dead-man armed, which the
  journal's recovery stops.
- **The one-shot may wait for days** before a boot consumes it: that a
  firmware keeps `BootNext` across a long uptime is to verify.

## 2. Fall back

### 2.1 The reboot

- **A fall back is a full reboot into the default,** never a soft reboot
  back into the trial. A live trial's is 2.5's instead, which reboots
  only when one of its steps fails. While `/run/nextroot` is a mount
  point, `systemctl reboot` turns into a soft reboot (systemd 255 and
  later, trixie's 257 included; soft reboot itself came in 254), unless
  `SYSTEMCTL_SKIP_AUTO_SOFT_REBOOT=1` is set.
- **So every reboot hearth makes avoids it:** the activator unmounts
  `/run/nextroot` and runs `systemctl reboot` with that variable set; the
  dead-man runs `systemctl reboot -ff`, which calls `reboot(2)` without
  a clean shutdown; `reboot-force` is PID 1's own.
- **In the trial slot `/run/nextroot` is not a mount point,** since the
  switch consumed it. The trap matters in the old slot, while a soft
  reboot is armed, which is why aborting one unmounts it first.
- **Asked for in two cases:** the trial slot's steward finds a
  definite failure (1.1); or the old slot's steward, before the reboot,
  gives up on a trial it armed. In the second case nothing reboots: the
  one-shot is cleared, `/run/nextroot` unmounted, `/run/hearth-trial/`
  emptied, and the slot is `prepared` again.

### 2.2 Recording it

At its start in the default slot, the activator finds a marker naming
another slot, in phase `armed` or `running`, and a boot id or soft
reboot count that changed:

1. **write that slot's record** as `fell back`, with the phase it reached
   (`never reached hearth`, or `not blessed in time`) and the failed
   boot's id;
2. **clear what the trial left:** a `BootNext` or `next_entry` still set,
   and `/run/hearth-trial/`;
3. **remove the marker;**
4. **report** the facts and a `fell back` event.

Each step is idempotent, so a cut between them is finished at the next
start.

### 2.3 What a fell-back slot keeps

- **Its subvolumes, until the next prepare,** including its `/var` with
  the failed boot's state, for triage [M5].
- **Its entries, outside `BootOrder`.**
- **No second trial without a new prepare:** its `/var` has run, so it is
  no longer what prepare made.

### 2.4 A blessed default that stops booting

A slot that passed its bless can later fail at every boot: a disk, a
firmware or a kernel that turns bad. A panic then reboots into the same
default, for ever: EFI falls through `BootOrder` only when an image
fails to load, and GRUB's `fallback` is the default itself.

- **BIOS [M3b]: a boot counter in the environment pair,** RAUC's pattern
  (hearth/docs/design/domain/machine/boot.md, 2.3). At bless the activator writes `previous`, the old
  default, and `tries=3`. GRUB counts `tries` down at each boot of the
  default; at 0 it boots `previous` instead. The activator resets
  `tries` at every start that meets the minimal criterion. The previous
  slot, booted so, is blessed and not the default: its steward blesses it
  again (1.5), and the slot that stopped booting is recorded `fell back`.
  Prepare clears `previous` before it writes over that slot (`prepare.md`, 1).
- **EFI (M3): no such fallback.** A blessed default that panics at every
  boot loops until a person or, with an engine, a power cycle and an
  order intervene. One option: keep the previous blessed slot first in
  `BootOrder` and set `BootNext` to the default at each start that meets
  the criterion. It costs one NVRAM write per boot, and a power cut
  during a boot rolls back to the previous slot. It is open
  (`README.md`, section 7).

### 2.5 The live fall back

How a live trial falls back with no reboot (`trial.md`, 8). The activator runs it
itself when an action of L6 fails; the steward asks for it when a
touched unit dies or the criterion fails during the settle, or, a margin
before the base's window ends (60 seconds to start), when the bless has
not come, so the dead-man stays the backstop; and a `fall back` or a
roll back of a `live-blessed` slot is the same, its one-shot cleared and
the base's dead-man armed again first. The dead-man stays armed
throughout:

1. **stop the units L6 started new;**
2. **swap back,** per hierarchy: beneath, onto the previous delta's
   overlay (`trial.md`, 9), or `umount2(MNT_DETACH)` down to the base; read the
   mount table back; write the live record, or remove it;
3. **`systemctl daemon-reload`,** if L6 ran it;
4. **replay the actions under the old files:** reload and restart the
   same units again, and start what L4 stopped;
5. **reload D-Bus and invalidate nscd again,** if L6 did;
6. **check every job,** as L6 does;
7. **only then** stop the dead-man and remove
   `/run/hearth-trial/x1-prod.a`, write the slot record `fell back` with
   kind `live`, the step and the reason, remove the marker, and delete
   the failed delta.

- **Any failure in steps 1 to 6 is `reboot -ff`:** the default boots
  natively, and a fall back that hangs meets the dead-man.
- **When it succeeds nothing reboots:** the files that ran before run
  again, and the facts report `fell back (live)` (hearth/docs/design/protocol/wire.md, section 5).
- **It restores files, not state.** An open file or a working directory
  keeps the new files until closed, a process keeps what it loaded,
  users added keep their files in `/var`, tmpfiles' paths stay, and what
  was written to `/var` or a data entry stays: a data migration is not
  undone, as with any fall back (2.3). Only a reboot restores process
  state; only a restore from a snapshot brings data back
  (hearth/docs/design/domain/operating.md, 7.4).
- **A fell-back slot is refused by its prepare key** (hearth/docs/design/protocol/standalone.md, 4), as
  after any trial, though its `/var` never ran.

## 3. Roll back

- **The request names the other slot,** conditional on the facts it was
  decided from: the current slot runs and is the default, and the other
  slot is `blessed` with the record the decision saw. Anything else is
  refused as stale.
- **It is a trial of the other slot,** soft or one-shot by `trial.md`, 1's rule,
  followed by that slot's own bless when the criterion holds. Its `/var`
  is as it was when it last ran.
- **It never moves the default blind.** If the old slot no longer boots,
  its trial falls back and the machine stays where it was, with both
  facts reported.
- **While a live state runs,** a roll back is its live fall back (2.5):
  back to the files that ran before it, with the one-shot cleared and no
  reboot. A
  `live-blessed` slot is not blessed, so no other slot is a roll back's
  target until it has booted natively.
- **Who asks:** an engine's order, an operational effect granted by
  policy [M5] (hearth/docs/design/domain/operating.md, section 6); standalone, a person through
  the client (hearth/docs/design/client/README.md, section 3).

## 4. Drift

The activator compares the running slot with its record, and the
machine with its layout, and reports each check under a stable id. The
ids are facts; drift is never repaired in place.

| Check id | Passes when |
|---|---|
| `slot.root.subvolume` | `/` is the running slot's root subvolume, by uuid, and its read-only flag is set |
| `slot.root.record` | `/.hearth/slot` equals the slot record's fields |
| `slot.root.tree` | a walk of the running slot's root, through `/run/hearth-activator/top/@<target>.<label>` and never through `/`, matches the manifest; the first paths that differ are listed, up to a bound |
| `slot.var.subvolume` | `/var` is the slot's var subvolume, by uuid |
| `slot.mounts` | every fstab line is mounted, from its subvolume, with its options |
| `slot.live.mounts` | with a live record: each hierarchy it names carries one overlay with source `hearth-live`, its layers the record's delta over the base, the delta read-only; the others none. Without one: no `hearth-live` overlay anywhere |
| `slot.live.tree` | with a live record: a walk of the merged `/etc`, `/usr/local` and `/opt` matches the record's manifest (`README.md`, 2.4), comparing type, mode, owners, size, extended attributes and link targets |
| `slot.boot.files` | EFI: the slot's ESP files match their digests |
| `slot.boot.entry` | EFI: the slot's two entries match; BIOS: covered by `slot.root.tree` |
| `machine.default` | the default is the slot the records say, and blessed |
| `machine.subvolumes` | every subvolume the layout names exists, and no other `@<target>.*` but a restore's or the deltas the live record and the marker name (hearth/docs/design/domain/machine/README.md, 3.1) |
| `machine.class.<class>` | the class subvolume exists and is mounted at `/srv/<class>`, for each class an entry uses |
| `machine.data.<name>` | the entry's directory exists in its class's subvolume, and its path shows it: bound from `/srv/<class>/<name>`, or that path itself |
| `machine.data.orphaned` | no directory in a class subvolume but the entries' and a restore's (hearth/docs/design/domain/machine/README.md, 3.4); each orphan is listed, and reported, never removed |

- **On a mutable slot** (hearth/docs/design/domain/development.md, 5.3) the same checks are the
  experiment's diff: what `slot.root.tree` finds was changed by hand, is
  reported as `explored` rather than as a fault, and is what `hearth
  diff --running` presents with contents (hearth/docs/design/domain/development.md, section 5);
  `slot.root.subvolume` expects the read-only flag clear.
- **Left out:** the contents of `/var` and of the class subvolumes,
  which are meant to change, and anything mounted beneath `/`. The root's
  walk goes through the top level, so it stays on the root's subvolume
  and never sees a live state's overlays, which `slot.live.tree` walks.
- **Two depths:** a metadata walk compares type, mode, owners, size,
  change time and inode with the manifest; a sealed root changes none of
  them unless someone cleared its read-only flag. Contents are hashed
  again only on demand.
- **When:** on demand from the client, and at every start; on a period
  with operating's signals [M5] (hearth/docs/design/domain/operating.md, section 4).
- **Drift is the activator's:** it wrote the record, and only root can
  read every file of a slot (`hearth/docs/design/README.md`, 4.2).
- **A host's containers [M5]** are checked by the host's activator under
  the same ids, reported for the container's target: the running slot's
  root from the top level, against its record and manifest;
  `slot.mounts` against the nspawn settings it wrote (`containers.md`, 1), not fstab;
  no `slot.boot.*`; `machine.default` against the default it recorded.
  `machine.subvolumes` counts the containers' subvolumes as the host's
  layout.
