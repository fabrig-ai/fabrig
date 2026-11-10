# Slots

Provisional, 2026-10-09, revised 2026-10-19. A slot's life on a
machine: what a slot is and what its record holds, the four facts, the
state machine, prepare step by step, trial (by a reboot, a soft reboot, or live, with no reboot),
bless, fall back, roll back, drift, what crashes and power cuts do at
every step, and containers. M3 builds it on EFI, with a minimal bless
and live changes; M3b adds trials through GRUB; containers and pool
hosts are M5, live changes in a container included. The layout slots
live on is hearth/docs/design/domain/machine/README.md's; the process that carries out every transition
is hearth/docs/design/domain/activator/README.md's. What is open is in section 7.

## 1. In one page

- **A slot is a read-only root and its own `/var`,** two subvolumes
  labelled `a` or `b`, holding one output prepared for this machine:
  shifted to real ids, with this machine's identity and secrets, fstab
  and units written in, then sealed; a mutable target's slot is left
  unsealed and read-write, for changes by hand (hearth/docs/design/domain/development.md).
- **Four facts, kept apart:** the slot running (the root subvolume's
  identity, never `/proc/cmdline`), the default (what the firmware or
  GRUB boots), the slot on trial (the activator's marker), and the live
  state (hearth's overlays on `/etc`, `/usr/local` and `/opt`, read from
  the mount table).
- **One state machine per slot,** written as a total table: empty,
  preparing, prepared, trying, on trial, on live trial, live-blessed,
  blessed, fell back. Its invariant: **the default always boots a
  blessed slot,** whatever crashes and power cuts happen, at any step.
- **Prepare is seven steps,** each in the activator: snapshot the
  output, shift it while hashing it, split `/var` off as a reflink copy,
  write identity and secrets, write the slot's files, write its boot
  files and entries, verify and seal. Every path in the slot is resolved
  from the snapshot's own descriptor, never through a link the output
  planted. A slot left preparing is deleted and made again.
- **A trial runs the new slot once.** A soft reboot into `/run/nextroot`
  when the kernel and its command line are unchanged; otherwise a
  one-shot reboot, `BootNext` or GRUB's `next_entry`. The default does
  not move.
- **Configuration and in-house software change live, with no reboot**
  (`trial.md`, 6 to 9). When a change touches only `/etc`, `/usr/local` and `/opt`
  over the same packages, and every changed path has a declared
  consumer, the activator builds a delta of the new slot, mounts it over
  the running slot's hierarchies with an atomic swap, and reloads,
  restarts or starts the declared units. Its bless sets the one-shot to
  the new slot's trial entry instead of moving the default, so the slot
  still boots natively on trial before it becomes the default.
- **Every trial ends:** in a bless, or in a reboot into the default,
  from `panic=10`, a hardware watchdog, a job timeout on the trial's
  target, or a dead-man timer the old activator wrote into the slot. The
  dead-man is armed by a kernel command-line token or a marker in `/run`,
  so no failed mount and no emergency mode can disarm it.
- **The new slot blesses itself:** its own steward asks its own
  activator, once the boot has reached its target with no failed unit
  and a builder has rendered the slot's own revision (hearth/docs/design/domain/steward.md,
  section 6). A broken hearth release cannot bless itself, so it cannot
  strand a machine.
- **Fall back is recorded, not decided:** the default slot's activator
  finds the marker naming a slot that is not running, and records that
  it fell back.
- **Roll back is a trial of the other blessed slot,** conditional on
  the facts it was decided from, never a blind move of the default.
- **Drift compares the live slot with its record,** under stable check
  ids, leaving out `/var`, the class subvolumes and other mounts.
  Drift is reported, never repaired: a fix is a new slot.
- **Containers have slots on their host [M5]:** prepared the same way,
  activated by a restart or live with the same mechanics, tried until
  their unit is stably active and their boot has no failed unit, and
  blessed by the host's hearth, since none runs inside.

## 2. Slots and targets

### 2.1 A slot

- **Two subvolumes at the top level,** `@<target>.<label>` (the root)
  and `@<target>.<label>.var` (its `/var`), with the label `a` or `b`
  (hearth/docs/design/domain/machine/README.md, section 3).
- **One output at a time,** prepared for one machine: the output's tree
  is the same on every machine that builds its key; the slot adds this
  machine's identity, secrets, fstab and units, and splits off `/var`.
  So a slot is not the output, and has a tree digest of its own.
- **Read-only once prepared.** Its root's subvolume is sealed with the
  read-only flag, and mounted `ro`; only its `/var` and the target's
  class subvolumes are written while it runs. Nothing is changed in
  place: a fix, a new secret or a new command line is a new prepare.
- **But a mutable slot,** whose `boot` part says `mutable`, on a machine
  that accepts it: its root is not sealed and is mounted `rw`
  (hearth/docs/design/domain/development.md, 4.2). What is changed in it by hand is never
  carried into another slot: the next prepare starts from the output.
- **One machine runs one target,** so its slots are `@<target>.a` and
  `@<target>.b`. A host's containers have slots of their own on the
  host's top level [M5] (`containers.md`).

### 2.2 The slot record

The activator keeps one record per slot in `@hearth.state`
(hearth/docs/design/domain/activator/README.md, section 6), written atomically:

```
slot record    slots/<target>.<label>
  state              empty | preparing | prepared | trying | on trial | on live trial | live-blessed
                     | blessed | fell back
  output             the output's key: its name in the store
  prepare key        H("prepare", output, data, secrets and boot digests, window, units allowed to fail,
                     the identity's generation, the secret store's generation)
  window, allowed    the trial's window and the units whose failure does not block a bless (7.1)
  data policies      from the data part, per class: the snapshot policy [M5], and the backup's
                     period, peer and age recipient [M6], the only recipient a send may use
                     (../activator/operations.md, 11; ../operating.md, section 7)
  output tree        the output's tree digest, as its store record gives it and the shift found it
  slot tree          the slot's own tree digest: the sealed root after prepare; root-only
  reported tree      the same, each secret and identity file's content digest replaced by
                     H(name, generation): the one the facts carry (5.7)
  var tree           the slot's /var as copied, secrets beneath it included
  kernel             version; the four digests of 6.1; the initramfs file's digest, to check its ESP copy
  command line       as written for this slot (../machine/boot.md, section 3)
  subvolumes         root and var: id and uuid
  boot               EFI: its two entry numbers, normal and trial, and its ESP files' digests;
                     BIOS: its fragment's digest
  parts              digests of the data, secrets and boot parts it was prepared with
  generations        the identity's; the secret store's
  trial              last attempt: kind (soft, one-shot, live), phase reached, outcome, the failed
                     boot's id; for a live one, the step that failed and the units acted on
  times              prepared, tried, blessed, fell back (wall clock)
  costs              time per prepare step
```

- **The manifest** beside it, `slots/<target>.<label>.manifest`, is a
  tree file (hearth/docs/design/domain/store/records.md, 3) of the sealed root: one entry per path in
  tree order, with its local inode and change time as the seal left
  them. Drift compares against it (`bless.md`, section 4).
- **The record inside the root,** `/.hearth/slot`, holds the target, the
  label, the output, the prepare key and the root subvolume's uuid. It
  lets a running slot confirm which slot it is (section 3), and it is
  covered by the slot's tree digest like any file.
- **Nothing secret leaves the state:** secrets appear by name, and the
  store by its generation. The slot tree digest covers each secret's
  content digest, so only the reported tree digest is ever sent.

### 2.3 Which slot

- **Prepare takes the slot that is neither running, nor the default,
  nor on trial.** On a machine at rest, running and default are the
  same slot, so the other one is free. When they differ (the firmware
  fell through to the other slot, or a bless was cut), prepare is refused
  until the facts agree again: the running slot's steward blesses it
  again, which moves the default to the slot that runs (`bless.md`, 1.5).
- **A `live-blessed` slot is neither running nor the default,** so a
  later change prepares over it while its delta, a subvolume of its own,
  keeps serving (`trial.md`, 9).
- **The steward chooses nothing:** it asks to place an output, and the
  activator answers with the slot it used. converge decides whether a
  new prepare is needed by comparing the desired output and parts with
  the slots' records in the facts (hearth/docs/design/protocol/standalone.md).

### 2.4 The live record

While a live state exists (`trial.md`, 6), the activator keeps one more record in
`@hearth.state`, written atomically, beside the slot records:

```
live record    live/<target>
  base               the running slot: its label and its root subvolume's uuid
  delta              @<target>.live.<n>: n, the subvolume's id and uuid
  previous           the delta it replaced, kept until the next live bless (6.9); or none
  carries            the output key and prepare key the delta was built from, and the slot that
                     holds them, while one does
  hierarchies        which of /etc, /usr/local and /opt it overlays
  swapped at         wall clock; the boot id and soft reboot count at the swap
```

- **The manifest beside it,** `live/<target>.manifest`, is the merged
  view's expected entries, made at L2 (`trial.md`, 8): the base's manifest under
  `etc/`, `usr/local/` and `opt/`, with the change's paths replaced or
  removed. Drift compares the live view against it (`bless.md`, section 4), and it
  survives the slot being prepared over (`trial.md`, 9).
- **Written at every swap** (`trial.md`, 8, L5) and every swap back (`bless.md`, 2.5);
  removed when no overlay remains: by a live fall back to the bare
  base, or by the start that finds the overlays gone, after a reboot or
  a soft reboot into another root.
- **The live fact is read from the mount table** and checked against it
  (section 3); the record says what hearth made, the mount table what
  runs.

## 3. The four facts

| Fact | Read from | When it can be wrong |
|---|---|---|
| **running** | `BTRFS_IOC_GET_SUBVOL_INFO` on `/`: the root subvolume's uuid, matched with the slot records; cross-checked with `/.hearth/slot` | never; `none` on a live system or a root hearth did not prepare |
| **default** | EFI: the first of the target's entries in `BootOrder`; BIOS: `default` in the valid environment file (hearth/docs/design/domain/machine/boot.md, 1.4 and 2.6) | a firmware that reorders or forgets entries: reported, and put right at start |
| **trial** | the trial marker in `@hearth.state` (`trial.md`, 4) | never: it is the activator's own record |
| **live** | `/proc/self/mountinfo`: an overlay on `/etc`, `/usr/local` or `/opt` with source `hearth-live`, whose layers are a delta `@<target>.live.<n>` over the running slot's directory; checked against the live record (2.4) and the marker | after a person's soft reboot, which keeps the overlay on `/etc` and drops `/usr/local`'s and `/opt`'s (to verify): the disagreement is found at start, and ends in a full reboot (`crashes.md`) |

- **Running stays the root subvolume.** A live state changes what
  `/etc`, `/usr/local` and `/opt` show, never which subvolume `/` is:
  prepare, the guards and `slot.root.*` still name the base.
- **What runs, derived:** the prepare key the machine's files come from,
  the live record's while a verified live state is in place, else the
  running slot's; and the effective slot, the slot that holds it, if
  any. converge compares the desired prepare key with it (hearth/docs/design/protocol/standalone.md, 2).
  After a later live change falls back, no slot may hold it (`trial.md`, 9).
- **Running is never read from `/proc/cmdline`.** After a soft reboot the
  kernel's command line still names the slot the kernel was booted for.
  The mount table would do on btrfs, but the subvolume's uuid is exact,
  unprivileged, and needs no parsing.
- **Observations beside the facts,** read at every start: `BootNext` and
  `BootCurrent` (EFI) or `next_entry` (BIOS), the boot id
  (`/proc/sys/kernel/random/boot_id`), the manager's `SoftRebootsCount`
  (`systemctl show --property=SoftRebootsCount`), and the booted kernel:
  the four digests of `trial.md`, 1 of the slot running at the first start of
  each boot id, since a full boot always runs the kernel of the slot it
  boots.
- **Facts are read afresh** at every start of the activator and after
  every operation, and reported to the steward when they change
  (hearth/docs/design/domain/activator/README.md, section 4) and to an engine [M4] (hearth/docs/design/protocol/wire.md, section 5).
  An event such as `fell back` is a hint; the facts are the truth.

## 4. The state machine

Each slot has one state; the machine adds guards across both slots.
States, as in `skein/docs/foundation/programming-model.md`, 5.4, each holding exactly what it
needs:

| State | Holds | Meaning |
|---|---|---|
| `empty` | nothing | no subvolumes, or none worth keeping |
| `preparing` | prepare key, step | prepare in flight; never booted, never bootable |
| `prepared` | record | complete, verified, sealed; not the default |
| `trying` | record, attempt, kind, phase | a trial armed; the machine still runs the default |
| `on trial` | record, attempt | the slot runs on trial |
| `on live trial` | record, attempt, phase, live: base, delta, previous delta, unit plan | armed, then its files run over the running slot (`trial.md`, 8); nothing boots it yet |
| `live-blessed` | record, live: base, delta | passed a live trial; the one-shot names its trial entry; never booted natively (`bless.md`, 1.6) |
| `blessed` | record | passed a bless; the default, or the previous default |
| `fell back` | record, attempt | tried and not blessed; must be prepared again |

Events:

| Event | What it is |
|---|---|
| `prepare K` | a place request for this slot, whose prepare key is K |
| `prepared` | prepare's last step passed (`prepare.md`, 7) |
| `prepare failed` | a prepare step failed, typed |
| `try` | a trial request for this slot, or a roll back to it |
| `try live` | a live trial request for this slot (`trial.md`, 8) |
| `start here` | the activator starts (after a boot, a soft reboot or its own restart) and this slot runs |
| `start elsewhere, same boot` | the activator starts, another slot runs, and the boot id and soft reboot count equal the trial marker's |
| `start elsewhere, new boot` | the activator starts, another slot runs (or none), and they differ; with the marker's boot id and a higher soft reboot count, while a live state was recorded, it is a person's soft reboot (`trial.md`, 8) |
| `bless` | a bless request from the steward running in this slot |
| `bless live` | a live bless request from the steward running over this slot's live state (`bless.md`, 1.6) |
| `fall back` | a request to end this slot's trial now; for a live one, the live fall back (`bless.md`, 2.5) |
| `first bless` | bootstrap's bless: no slot is blessed and none runs |

The table, total: every cell a transition, `refuse` (a typed answer that
changes nothing), `ignore`, a `fault` (reported; every mutating
operation refused until a start finds a blessed slot running and the
facts agreeing), or `impossible` with the reason, only where the
activator's own order of steps excludes the cell.

| state ↓ event → | `prepare K` | `prepared` | `prepare failed` | `try` | `try live` | `start here` | `start elsewhere, same boot` | `start elsewhere, new boot` | `bless` | `bless live` | `fall back` | `first bless` |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| `empty` | `preparing` | impossible: no prepare runs | impossible: no prepare runs | refuse: not prepared | refuse: not prepared | fault: state lost; every operation refused | ignore | ignore | refuse | refuse | refuse | refuse |
| `preparing` | same K: answered when done; else refuse: busy | `prepared` | `empty`: subvolumes deleted | refuse | refuse | fault: a person booted its entry from the firmware's menu after a cut; it is never the default, in `BootOrder` or named by a one-shot | `empty`: deleted, the activator crashed | `empty`: deleted, the power was cut | refuse | refuse | refuse | refuse |
| `prepared` | same K: done; else `preparing`, old subvolumes deleted first | impossible | impossible | `trying` | eligible (`trial.md`, 7): the delta built and verified (`trial.md`, 8, L1 and L2), then `on live trial`; L1 or L2 failed: the delta deleted, failed; not eligible: refuse, naming the rules | fault: booted with no trial, by hand; reported | ignore | ignore | refuse: not on trial | refuse: not on trial | refuse | `blessed`, and the default |
| `trying` | refuse: busy | impossible | impossible | same attempt: done; else refuse | refuse: busy | `on trial`: marker set `running`; one-shot checked consumed | `prepared`: the reboot never came; one-shot cleared, `/run/nextroot` unmounted, marker removed | phase `arming`, or one-shot still set: `prepared`, one-shot cleared, reported; else `fell back` | refuse: not running | refuse | `prepared`: as `start elsewhere, same boot` | refuse |
| `on trial` | refuse: running | impossible | impossible | refuse | refuse | `on trial`: the activator restarted within the trial | fault: a person switched roots by hand | `fell back` | criterion holds: `blessed`; else refuse, saying which part failed | refuse: not a live trial | reboot into the default (`bless.md`, 2.1); the next start decides | refuse |
| `on live trial` | refuse: on trial | impossible: no prepare runs | impossible: no prepare runs | refuse: busy | same attempt: done; else refuse: busy | fault: booted natively with no trial, by hand; it was never armed | phase `arming`: `prepared`, disarmed, the delta deleted; later phases: `reboot -ff`, and the next start decides (`crashes.md`) | a person's soft reboot: `reboot -ff`; else phase `arming`: `prepared`; else `fell back`; the delta deleted | refuse: a live trial is blessed live | phase `running`, settled, criterion holds: `live-blessed` (`bless.md`, 1.6); else refuse, saying which part failed | the live fall back (`bless.md`, 2.5): `fell back`; a step that fails: `reboot -ff` | refuse |
| `live-blessed` | neither running nor default: `preparing`, its one-shot and marker cleared and old subvolumes deleted first; the delta keeps serving (`trial.md`, 9) | impossible | impossible | `trying`: an ordinary trial of it now, soft or one-shot (`trial.md`, 1) | refuse: already live | the one-shot booted it: `on trial`, marker set `running` (`trial.md`, 8, L9) | the live fact agrees with the live record: ignore; else `reboot -ff`, into its trial | a person's soft reboot: `reboot -ff`; the one-shot never set, or still set: `prepared`, one-shot cleared, reported; else `fell back`, never reached hearth | refuse: not running | same attempt: done; else refuse | the live fall back (`bless.md`, 2.5), one-shot cleared first: `fell back` | refuse |
| `blessed` | running or default: refuse; else `preparing`, old subvolumes deleted first | impossible | impossible | not the default: `trying` (a roll back); else refuse | refuse: a roll back is an ordinary trial | ignore; not the default: reported, and the steward blesses it again (`bless.md`, 1.5) | ignore | ignore | running and not the default, criterion holds: the default moves here (`bless.md`, 1.5); same attempt: done; else refuse | refuse | refuse | refuse |
| `fell back` | `preparing`, old subvolumes deleted first | impossible | impossible | refuse: prepare again | refuse: prepare again | fault: booted with no trial; reported | ignore | ignore | refuse | refuse | refuse | refuse |

Guards across both slots, checked before a cell is taken:

- **One mutating operation at a time** (hearth/docs/design/domain/activator/README.md, section 4), so at
  most one slot is `preparing`, `trying`, `on trial` or `on live trial`.
- **`prepare`** names a slot that is neither running nor the default.
- **`try`** names a slot whose other slot runs and is the default.
- **`try live`** names a `prepared` slot whose other slot runs, is the
  default and is blessed, with no trial; a live state, if one exists,
  has the running slot as its base; and the change is eligible (`trial.md`, 7).
- **`bless`** names the running slot, which is on trial, or blessed but
  not the default; it can only come from the steward of the running
  slot, the only one on the link.
- **`bless live`** names the slot on live trial; it comes from the
  steward on the link, the base's, spawned anew after the swap (`trial.md`, 8,
  L6).
- **`first bless`** only while running is `none` and neither slot is
  blessed: bootstrap (hearth/docs/design/domain/machine/bootstrap.md, 2).

The invariant, checked by every world (section 5):

- **the default is blessed;**
- **a slot that is `preparing` is never the default,** never in
  `BootOrder`, and never named by a one-shot or `/run/nextroot`;
- **a one-shot exists only while a slot is `trying` or
  `live-blessed`,** and names that slot (a `live-blessed` slot's trial
  entry);
- **a live state sits on the running slot,** which is blessed and the
  default, and the default never moves while one runs, its bless
  included;
- **a live state is what its record says:** the overlays in the mount
  table name the record's delta over its base, and the merged view
  equals the record's manifest;
- **a trial ends:** every `trying`, `on trial` or `on live trial` slot
  leaves that state within its window, or at the next start.

## 5. Its world

`hearth-domain-slots` runs in domain worlds on the fake machine
(hearth/docs/design/testing/fake-machine.md and hearth/docs/design/testing/worlds.md), under the activator's root or alone,
with:

- **the fake machine's top level, firmware and GRUB**
  (hearth/docs/design/domain/machine/README.md, section 4), and its models of tiny-initramfs and
  systemd: a boot reaches its target or not, units fail,
  `is-system-running` answers, a soft reboot switches roots and keeps
  the boot id;
- **a scripted steward** on the link, which asks to place, try, bless,
  fall back and roll back, at awkward moments and with stale conditions;
- **data by class:** entries added, removed and moved between classes
  across prepares, so classes are made once, each entry seeded once and
  never touched again, a moved entry refused, an orphan reported, and
  every bind mounted after its class (hearth/docs/design/testing/fake-machine.md, 7);
- **faults from the seed** at every step: a crash of the activator, a
  power cut, a kernel that panics or hangs, a unit that fails, a hearth
  that never blesses, a firmware quirk, a torn FAT or NVRAM write;
- **a host's containers [M5]** on the fake machine's nspawn
  (hearth/docs/design/testing/fake-machine.md, 12): units that start or fail, failed units inside, a
  host reboot or an activator crash mid-trial, a container removed from
  the spec (`containers.md`, 7), checked against `containers.md`, 6's invariant;
- **live changes** on the fake machine's mounts, overlays and PID 1's
  jobs (hearth/docs/design/testing/fake-machine.md, 13): eligible and ineligible changes drawn from
  the seed, a reload that fails, a unit that dies during the settle, a
  crash or a power cut at every phase from L1 to L9, a person's soft
  reboot, a later change while live and its fall back to the previous
  delta, the native boot after a live bless; and [M5] the same in a
  container.

The referee checks section 4's invariant after every event, every cell
of the table is reached (transition coverage), and every request gets
exactly one answer. It computes `trial.md`, 7's rules on its own from the fake
trees and specs, and each trial's kind must follow them. The VM tier
runs the trial's ending paths for real: a panic, a dead-man reboot, a
soft reboot and its fall back, and every live scenario
(hearth/docs/design/testing/vm.md).

## 6. What changes from os

- **Trial, bless and fall back are new.** os had two states, prepared and
  default, and an activation that was permanent at once; nothing ended a
  bad slot's boot.
- **Prepare shifts and verifies.** os built as root and snapshotted;
  hearth snapshots an unprivileged output and shifts it, and the shift
  checks the output's digest.
- **`/var` splits at prepare,** not after the packages phase of a build:
  a layer is one subvolume.
- **No template is completed at activation.** os rendered `fstab` and any
  declared file with MiniJinja inside the root-owned slot; the activator
  writes fixed files from typed data.
- **Secrets and identity enter at prepare,** from a root-only store. os
  copied `/secrets` into every root filesystem and rendered the
  machine-id from configuration.
- **Which slot runs** comes from the root subvolume's uuid, not from
  `/proc/cmdline`.
- **Slots are sealed and drift is checked against the slot's record,**
  so fstab and secrets no longer show as drift; `ek shell`, which made
  the running slot writable, is gone.
- **Nested subvolumes are deleted first,** where os deleted the `.var`
  first and looked for them in the root.
- **Containers' slots** move from a symlink under `/var/lib/machines`,
  switched by a script on one host, to records and links the host's
  activator keeps [M5].
- **Live changes replace patching in place.** os's and the bash fleet's
  `deploy patch` rewrote a running root's `/etc` (reset to a tag,
  replayed, committed again), with no trial; the bash fleet's failed
  live cutover (its #603) came from raw copies into a live `/etc`.
  hearth writes nothing in a running root: it mounts a verified delta
  of a prepared slot, swaps it atomically, tries it under a dead-man,
  and still boots the slot natively on trial before it becomes the
  default.

## 7. Open questions

- **The trial window:** 10 minutes in M3, a place input sealed into the
  units; from M5 the steward computes it from the blessing checks its
  builder's render answered (hearth/docs/design/domain/operating.md, section 3). Its right length
  depends on measuring the slowest target.
- **A blessed EFI default that stops booting:** keeping the previous
  slot first in `BootOrder` and arming `BootNext` to the default at each
  healthy start (`bless.md`, 2.4); depends on what one NVRAM write per boot costs,
  and on whether a roll back on a power cut during boot is acceptable.
  It would compete with a live bless's `BootNext` (`bless.md`, 1.6).
- **Failed units against a baseline:** blessing when the trial fails no
  unit the previous slot did not; depends on measuring `x1-prod` (`bless.md`, 1.1).
- **A dead-man that survives the soft reboot:** a unit of the old slot,
  `SurviveFinalKillSignal=yes`, that reboots unless the new slot's hearth
  cancels it through `/run`, would not depend on the new slot's systemd
  at all; untested, and depends on the VM tier.
- **Hardware watchdogs across a soft reboot:** whether systemd keeps
  `RuntimeWatchdogSec=` armed through the switch (to verify).
- **`systemctl reboot -ff` and `/run/nextroot`:** that `-ff` calls
  `reboot(2)` without the automatic soft reboot (to verify); if not, the
  dead-man's service uses `SuccessAction=reboot-immediate`.
- **The units' behaviour:** the triggering conditions on the command
  line and `/run`; the arming service's `systemctl start --no-block`
  early in the boot and the target's job timeout counted from then; and
  whether `IgnoreOnIsolate=yes` keeps the target's pending job through
  an isolate into `emergency.target` (to verify on the VM tier).
- **Remounting `/` after a soft reboot** with fstab's `subvol=` (to
  verify).
- **Holding a bless [M4]:** the `held` state's exact cells, and whether a
  hold survives the activator's restart by its marker alone; depends on
  hearth/docs/design/protocol/wire.md, 4.4.
- **What a fell-back slot leaves for triage:** its journal is in the
  `core` class only if a task declares the `journal` entry; depends on
  hearth/docs/design/domain/operating.md's telemetry.
- **Data policies in the prepare key:** a change to a class's snapshot
  or backup policy is a change to the `data` part, so it prepares a new
  slot of the same output, never live (`trial.md`, 7); whether policies should
  reach the activator by a request of their own depends on how often
  they change.
- **Hashing contents again at prepare,** or trusting the builder's
  records (README, `bless.md`, section 3; hearth/docs/design/domain/activator/README.md, 5.3).
- **Drift's period and its full depth;** depends on hearth/docs/design/domain/operating.md's
  signals and on the cost of hashing a root, once measured.
- **Containers:** whether a container runs a hearth of its own; shifting
  to each container's range, or once with idmapped mounts; how a pool
  names several containers of one target; whether nspawn finds images in
  `/run/machines`. Depends on M5's pool design and hearth/docs/design/domain/operating.md's checks.
- **nspawn's binds from the activator's top-level mount [M5]:** whether
  a root and a `/var` bound from `/run/hearth-activator/top`, mounted
  `nodev,nosuid,noexec`, keep those flags (to verify); if they do,
  containers need a mount of the top level without them.
- **Reading a container from its host [M5]:** whether
  `systemctl --machine=` needs a system bus inside the container, and
  how the running slot's subvolume is read through the container's
  leader (to verify on the VM tier, hearth/docs/design/testing/vm.md, 3).
- **Live changes on a real machine,** to verify on the VM tier
  (hearth/docs/design/testing/vm.md, 3) on trixie's systemd 257 and forky's 262, Incus with
  OVMF:
  - real root and real ids: `trusted.overlay.*` whiteouts and opaque
    directories honoured as the `user.*` forms were; owners, v2
    capabilities and ACLs
    through the overlay; a delta built from a shifted slot exactly what
    L2 expects;
  - `MOVE_MOUNT_BENEATH` from the host's namespace, PID 1's shared `/`,
    with real services: `ProtectSystem=strict` and `full`,
    `ReadWritePaths=/etc`, `InaccessiblePaths=` and `BindPaths=` beneath
    `/etc` and `/usr/local`, `PrivateTmp=`, `DynamicUser=` and
    `RootDirectory=` units;
  - PID 1 around a swap: `daemon-reload` with new, changed and removed
    fragments of running units, new `.wants` links and masks;
    `NeedDaemonReload=`; whether dynamic `etc.mount`, `usr-local.mount`
    and `opt.mount` units appear, whether `daemon-reload` disturbs them, and
    what shutdown does with them;
  - soft reboots: the overlays gone with the old root when
    `/run/nextroot` is set; `/etc`'s kept and `/usr/local`'s and
    `/opt`'s dropped without it (`trial.md`, 8);
  - the base's dead-man started after boot from
    `/run/hearth-trial/<target>.a`; `reboot -ff` landing on the default;
    `BootNext` to a trial entry surviving days of uptime and consumed by
    a power cut, on OVMF and real firmware;
  - timing on a real tree: L1's build, L2's walk of `/etc`,
    `/usr/local` and `/opt`, the swap under a load of opens;
  - Debian's specifics: `/opt` present in outputs; trixie's kernel
    configuration without overlayfs's `XINO_AUTO`, `INDEX` or `METACOPY`
    defaults; AppArmor profiles on overlay paths, exec transitions for
    `/usr/local/bin` and `/opt` programs; dbus-daemon's `ReloadConfig` after a
    policy change; glibc reading `nsswitch.conf`, `hosts` and
    `resolv.conf` afresh after a swap; `sshd -t` failing a reload
    reported as a failed job;
  - the unit actions end to end: stop, swap, start for a changed binary;
    a reload for a configuration change; newly wanted units started; a
    unit stopped whose file and binary the delta removes; every job's
    result reaching the steward;
  - the fall backs: `bless.md`, 2.5 end to end, including a unit that fails only
    after the settle; an activator killed in each phase; the power
    pulled in each phase; a person's soft reboot;
  - whether overlayfs applies a lower mount's `noexec` or `nosuid` to
    the merged files, which decides whether the layers need their own
    source mount (`trial.md`, 8).
- **hearth itself, live:** its own binary and units are never live
  until live changes are proven; then the activator would restart into
  the new release and recover. Depends on the VM tier's results.
- **`/var` paths live:** whether a change may make paths beneath `/var`
  that hearth's own tmpfiles lines create, run live (`trial.md`, 7, rule 3);
  depends on the tasks that would use it.
- **AppArmor and networkd live:** a profile change with `apparmor_parser
  -r`, a `.network` change with `networkctl reload`, which may cut the
  machine off; depends on the fleet needing them.
- **The settle's length:** 60 seconds to start (`trial.md`, 8, L7); depends on
  measuring the slowest services' restarts.
- **A live roll back to the other blessed slot,** its files over the
  running one's, refused for now (section 4); depends on the need for
  undoing a change faster than a trial.
