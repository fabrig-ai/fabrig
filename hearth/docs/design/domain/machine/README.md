# Machines

Provisional, 2026-10-09. What a machine looks like to hearth: the kinds
of machine it runs on, the btrfs top level and its mounts, how each kind
boots a slot, the kernel's command line, the initramfs, bootstrap from a
live system, and the firmware's quirks. M3 builds the EFI path and
bootstrap; M2's boot test (hearth/docs/design/testing/real-builds.md, 6) first boots a built
root with the command line's tokens (`boot.md`, 3.1) and the initramfs (`boot.md`, section 4);
M3b builds the BIOS path; containers and pool hosts are M5
(hearth/docs/design/domain/slots/containers.md). A slot's life on this layout is hearth/docs/design/domain/slots/README.md's, and
the process that writes the layout is hearth/docs/design/domain/activator/README.md's. What is open is
in `README.md`, section 6.

## 1. In one page

- **A machine is one disk: a FAT boot partition and one btrfs
  filesystem.** On EFI the boot partition is the ESP; on BIOS it holds
  GRUB and its environment. Everything else is a subvolume at the btrfs
  top level.
- **The top level holds only subvolumes, named by target:** two slots,
  each a root and a `/var`; up to three class subvolumes for persistent
  data, `core`, `data` and `cache`; the builder's store; the
  activator's state; local snapshots [M5]; received backups [M6]. A
  host's containers have their slots and classes there too, under their
  own targets' names [M5]. Slots and classes are siblings, never nested,
  and only the activator mounts the top level.
- **Persistent data is a directory per `data` entry in its class's
  subvolume,** bound at the path its task declares: one subvolume per
  class, as os and the bash fleet had, so a snapshot is a class's and
  atomic across its services.
- **The root is read-only twice over:** its subvolume is sealed
  read-only when prepare ends, and it is mounted `ro`. `/var`, the
  classes, the store and the state are writable mounts beside it. A
  mutable development machine is the one exception (2.8).
- **fstab is the activator's, written from typed data** at prepare: the
  filesystem by UUID, the slot's own subvolumes, the classes at
  `/srv/<class>`, and a bind per entry at the path its task declares.
  No template, and no device in an output.
- **EFI boots each slot's kernel directly.** The ESP holds
  `\vmlinuz.<target>.<label>` and `\initrd.img.<target>.<label>`; each
  slot has two `Boot####` entries at fixed numbers, normal and trial,
  rewritten only when their command line changes. The normal one enters
  `BootOrder` when the slot is blessed; the trial one never does, and a
  trial is `BootNext` to it, checked with `BootCurrent`.
- **BIOS boots GRUB from the post-MBR gap [M3b].** GRUB lives on the
  FAT partition with a fixed `grub.cfg` and a pair of environment files
  it can write, so `next_entry` is a real one-shot, cleared before the
  kernel loads, and a boot counter falls back to the previous blessed
  slot. It reads kernels from the slot subvolumes.
- **The command line is hearth's, then the target's:** `root=UUID=`,
  `rootflags=subvol=`, `ro`, `panic=10`, then the `boot` part's
  parameters, consoles among them, in at most 2048 bytes.
- **tiny-initramfs stays,** configured per hardware profile: no
  automatic modules, an explicit list, no build host's microcode. It finds the
  root by UUID and panics on any error, which `panic=10` turns into a
  reboot. hearth uses Debian's package, in trixie and in forky.
- **Bootstrap is one command, run as root from a live system:**
  partition, format, make the top level, build as the unprivileged
  `hearth` user, place the output into slot `a`, write its boot files,
  and make it the default.
- **Secure Boot is off:** a Debian kernel cannot boot from a direct
  entry under it. Firmware that prunes, reorders or ignores entries is
  met by keeping hearth's entries few, fixed and checked after every
  boot.

## 2. Kinds of machine

| Kind | Firmware | Disk, boot partition | How a slot is tried | Watchdog | Examples | Milestone |
|---|---|---|---|---|---|---|
| bare metal on EFI | the vendor's UEFI | NVMe; ESP | live, soft reboot, or `BootNext` | iTCO where the chipset has it (to verify per machine) | `x1-prod` | M3 |
| Incus VM with its own disk | OVMF | a virtio disk; ESP | live, soft reboot, or `BootNext` | iTCO on q35 | the VM tier's machines (hearth/docs/design/testing/vm.md) | M3 |
| IaaS VM on BIOS | the provider's BIOS | a virtio disk; FAT environment partition | live, soft reboot, or `next_entry` | none known | `east-prod`, `west-prod` | M3b |
| IaaS VM on EFI, NVRAM unreliable | the provider's UEFI | a virtio disk; ESP | open (`boot.md`, 5.4) | none known | south | later |
| container | none: the host's kernel | slots on the host's top level | live, or restart | the host's | `chat-prod` and `mail-prod` on `east-prod`; `mail-lab` in a lab pool | M5 |
| VM with a virtiofs root | Incus direct kernel boot | slots on the pool host's top level | stop, repoint, start | the host's | the bash fleet's lab | later |

### 2.1 Bare metal on EFI

The MVP's machine is `x1-prod`, a laptop with an NVMe disk.
hearth installs it afresh on a wiped disk, as every machine (`bootstrap.md`):
nothing of os's is kept. A person is at hand, so a hang no watchdog
covers is recoverable; the machine is held to the same invariants as
the others all the same.

### 2.2 Incus VMs with their own disk

The MVP's test machines (hearth/docs/design/testing/vm.md). Unlike the bash
fleet's lab VMs they have a real disk, a real ESP and real firmware
variables, so bootstrap, entries, `BootNext` and fall back run as on
bare metal.

- **`security.secureboot=false`:** Incus enables Secure Boot by default,
  and a direct entry for a Debian kernel then fails to load (`boot.md`, 5.5).
- **OVMF keeps its variables** in `qemu.nvram` beside the instance
  volume, regenerated only when missing or invalid, or when the Secure
  Boot or CSM settings change; `incus publish` drops them. At every
  boot it prunes and reorders the options `BootOrder` lists (`boot.md`, 5.1).
- **q35 has the ICH9 TCO watchdog,** whose default action is a reset,
  driven by `iTCO_wdt` and `RuntimeWatchdogSec=`.
- **The console is `ttyS0`;** the disk is Incus's default bus,
  virtio-scsi (to verify), which sets the initramfs's modules (`boot.md`, 4.2).
  BIOS is tested with `security.csm=true`, which boots SeaBIOS (to
  verify) [M3b].

### 2.3 IaaS VMs on BIOS [M3b]

- **Two of three production VPSes are BIOS:** `east-prod` on provider A
  and `west-prod` on provider B. No BIOS host has been provisioned by
  os's code, whose BIOS path has three mismatches and keeps GRUB's
  environment on btrfs.
- **Each provider has its own disk bus,** hence its own initramfs
  modules (`boot.md`, 4.2), and a serial console as its last recovery path, which
  needs a root password hash that is not locked: a secret applied to the
  slot's `/etc/shadow` (hearth/docs/design/domain/slots/prepare.md, 4).
- **Bootstrap runs from the provider's rescue system** (`bootstrap.md`).

### 2.4 IaaS VMs on EFI with unreliable NVRAM [later]

Provider C's firmware can discard its variables when a disk moves between
contexts; the bash fleet booted GRUB from the removable path
(`\EFI\BOOT\BOOTX64.EFI`) with no slot entries. A trial in NVRAM is not
reliable there; `boot.md`, 5.4 gives the lean.

### 2.5 Containers [M5]

No kernel, no boot partition, no entries: a container's slots and
class subvolumes are subvolumes on its host's top level, prepared
by the host's activator, and activating one restarts its nspawn unit
(hearth/docs/design/domain/slots/containers.md). `boot.md`, sections 1 to 5 do not apply to it; no
fstab mounts anything of it (3.3), and it is never bootstrapped (`bootstrap.md`, 4).

### 2.6 VMs with a virtiofs root [later]

The bash fleet's lab booted Incus VMs straight into a host subvolume
over virtiofs, with `root=/dev/null rootfstype=virtiofs` (tiny-initramfs
accepts only a `/dev/` name, so the root's tag was `/dev/null`) and
`io.cache=unsafe` on the root (`mmap(2)` otherwise fails with
`ENODEV`). virtiofs rejects `subvol=`, and the guest cannot choose its
slot. Such a VM is closer to a container: its slots live on the pool
host, whose activator tries one by stopping the VM, repointing its
devices and starting it. hearth's test VMs do not use it.

### 2.7 What placement says

Placement is data of the environment (hearth/docs/design/protocol/format/README.md, 3.4), in the spec's
`meta`. What this document takes from it:

| Field | Values | Decides here |
|---|---|---|
| `kind` | `metal`, `vm`, `container` [M5] | section 2; the initramfs's microcode (`boot.md`, 4.1) |
| `firmware` | `efi`, `bios` | `boot.md`, sections 1 and 2; checked against the machine record |
| `hardware` | `7th-gen`, `incus`, `provider-a`, `provider-b`, `provider-c` | nothing directly: the `boot` task keys its modules and parameters, consoles among them, by it (hearth/docs/design/protocol/format/tasks.md, 4) |
| `on` | the host's target [M5] | hearth/docs/design/domain/slots/containers.md |
| `mutable` | `true`, `false` | 3.3's `/` line `rw` and no seal at prepare, on a machine that accepts it (2.8; hearth/docs/design/domain/development.md, section 4) |

- **The watchdog is the machine's own fact:** the activator looks for a
  device under `/sys/class/watchdog/` when it prepares a slot, on the
  machine the slot will run on (hearth/docs/design/domain/slots/prepare.md, 5).
- **The trial window** is a place input, 10 minutes in M3, sealed into
  the slot's units (hearth/docs/design/domain/activator/operations.md, 1); a container's is kept by its
  host's activator instead (hearth/docs/design/domain/slots/containers.md, 3) [M5].

### 2.8 Development machines [M3b]

- **An Incus VM cloned from a base image** (hearth/docs/design/domain/development.md, section 3):
  2.2's kind of machine, with its own disk, ESP and OVMF variables,
  made by `incus copy` of a VM bootstrapped once from a small target,
  then given its own target by `hearth retarget`, which places and blesses
  its first slot as bootstrap does (`bootstrap.md`, 2).
- **It may be mutable:** bootstrapped with `--accept-mutable` (`bootstrap.md`, 1), it
  takes targets whose placement says `mutable`, whose slots are not
  sealed and whose root is mounted `rw` (3.3). Every other rule of this
  document holds: the same top level, boot entries, trials and fall
  backs. A machine that does not accept is never mutable.
- **Every clone shares the base's filesystem UUID** and partition
  identifiers; no machine sees two disks of one image, so nothing
  confuses them.

## 3. The btrfs top level

### 3.1 Subvolumes

```
btrfs filesystem, UUID 0f3c9a6e-…, label hearth     top level: subvolid 5, mounted only by the activator
├── @x1-prod.a              slot a's root           a snapshot of an output, shifted, sealed read-only
├── @x1-prod.a.var          slot a's /var           a reflink copy of that output's /var
├── @x1-prod.b              slot b's root
├── @x1-prod.b.var          slot b's /var
├── @x1-prod.live.3         a live delta            only while a live state exists (../slots/trial.md, 6)
├── @x1-prod.core           the core class          one directory per entry: journal, iwd
├── @x1-prod.data           the data class          one directory per entry: home-alice
├── @hearth.store           the builder's store     owned by hearth, 0700; its layers nested in it (../store/README.md, section 2)
├── @hearth.state           the activator's state   root, 0700 (../activator/README.md, section 6)
├── @hearth.snapshots [M5]  class snapshots         root, 0700: <target>.<class>/<seq>, read-only (../operating.md, 7.1)
└── @hearth.backups [M6]    received backups        root, 0700, on a backup peer: <from>.<class>/<seq> (../operating.md, 7.3)
```

| Subvolume | Its root inode | Made by | Removed by |
|---|---|---|---|
| `@<target>.<label>` | the output's `/`; read-only flag set at the end of prepare | prepare: a snapshot (hearth/docs/design/domain/slots/prepare.md, 1) | the next prepare of that slot |
| `@<target>.<label>.var` | the output's `/var`, `0:0 0755` | prepare (hearth/docs/design/domain/slots/prepare.md, 3) | the next prepare of that slot, nested subvolumes first |
| `@<target>.<class>`: `core`, `data`, `cache` | `0:0 0755`; one directory per `data` entry of the class, each seeded once from the output's content at the entry's path | the first prepare that declares an entry of the class (hearth/docs/design/domain/slots/prepare.md, 3) | never by hearth (3.4) |
| `@<target>.live.<n>` | holds `etc`, `usr` and `opt` with the new slot's metadata: its changed files as reflinks, and 0/0 whiteouts; read-only once verified | a live trial (hearth/docs/design/domain/slots/trial.md, 8, L1) | the native bless of its slot, a live fall back, or the start that finds no record naming it |
| `@hearth.store` | `hearth:hearth 0700` | bootstrap | never |
| `@hearth.state` | `0:0 0700` | bootstrap | never |
| `@hearth.snapshots` [M5] | `0:0 0700`; a directory `<target>.<class>` per class snapshotted, holding read-only snapshots `<seq>`, this machine's own, for its targets and its containers' | the first snapshot | each snapshot by its class's retention, or once sent and no longer needed (hearth/docs/design/domain/operating.md, 7.1) |
| `@hearth.backups` [M6] | `0:0 0700`; `<from>.<class>/<seq>`, read-only snapshots received from the targets that send to this backup peer | a backup peer's first receive | each by the sender's retention (hearth/docs/design/domain/operating.md, 7.3) |
| `@<target>.<class>.restoring` [M5] | a writable snapshot of one of the class's snapshots | a class's restore (hearth/docs/design/domain/operating.md, 7.4) | renamed to `@<target>.<class>` by the restore's swap |
| `@<target>.<class>.before.<seq>` [M5] | the class a restore replaced | the restore's swap | an operator or the engine |

- **One tree for both snapshots and backups:** `@hearth.snapshots`
  holds this machine's own snapshots of its classes, whatever took
  them, and each snapshot's record says whether the local policy keeps
  it, whether it was sent, and whether the peer confirmed it
  (hearth/docs/design/domain/activator/README.md, 6.1). When the local and the backup period meet, one
  snapshot serves both; an incremental send's parent is a local
  snapshot the backup keeps; and one walk prunes them all. What a
  backup peer receives from others goes to `@hearth.backups` instead:
  another target's data, kept by its sender's retention, which no local
  policy may prune and no local restore lists.
- **A host's containers [M5]** add the same names under their own
  targets: on `east-prod`, `@chat-prod.a`, `@chat-prod.a.var`,
  `@chat-prod.b`, `@chat-prod.b.var`, and `@chat-prod.data` and the
  other classes its `data` entries use, beside the host's own
  (hearth/docs/design/domain/slots/containers.md, 1), and their snapshots in the host's
  `@hearth.snapshots`. Its journal has no entry of its own: it lives in
  the host's (hearth/docs/design/domain/operating.md, section 5). The table above holds for them,
  the host's activator acting for the container; os's
  `@<target>.machines` under `/var/lib/machines` has no successor.
- **Slots exist once prepared:** os made empty placeholders at format
  time; hearth makes a slot by snapshot, so `@x1-prod.b` is absent until
  the first prepare into `b`. No build happens at the top level: os's
  `@T`, `@T.var` and `@T.install` are layers in the store now
  (hearth/docs/design/domain/store/README.md, section 4). Classes, too, exist once an entry uses them:
  os made `@T.srv`, `@T.srv.core`, `@T.srv.data` and `@T.srv.cache` at
  format time; hearth makes `@x1-prod.cache` only when an entry is
  `cache`, and has no `@T.srv`, since `/srv` is the root's.
- **In a slot, nested subvolumes appear only at runtime,** made by
  systemd in its `/var` (`/var/lib/machines`, `/var/lib/portables`), and
  are deleted before their `.var`, found with
  `BTRFS_IOC_GET_SUBVOL_ROOTREF`.
- **The default subvolume stays 5:** GRUB resolves its paths from it
  (`boot.md`, 2.4).
- **No filesystem-wide option is set.** Most btrfs options apply to the
  whole filesystem, from its first mount, the root's. hearth passes only
  `subvol=`, and never `user_subvol_rm_allowed`: the builder deletes a
  layer by emptying it through its tree helper and calling `rmdir(2)`
  (hearth/docs/design/domain/store/tree-helper.md).

### 3.2 Mounts

| Path | Source | Mounted by | Options |
|---|---|---|---|
| `/` | `@<target>.<label>` | tiny-initramfs, from `root=` and `rootflags=`; then systemd remounts it from fstab | `ro,subvol=/@x1-prod.b` |
| `/var` | `@<target>.<label>.var` | systemd, from fstab | `nodev,nosuid` |
| `/var/lib/hearth/state` | `@hearth.state` | systemd, from fstab | `nodev,nosuid,noexec` |
| `/var/lib/hearth/store` | `@hearth.store` | systemd, from fstab | `nodev,nosuid,noatime` (hearth/docs/design/domain/store/README.md, 7.1) |
| `/srv/core`, `/srv/data`, `/srv/cache`, each class some entry uses | `@<target>.<class>` | systemd, from fstab | `nodev,nosuid` |
| each `data` path but `/srv/<class>/<name>` | a bind of `/srv/<class>/<name>`, the entry's directory | systemd, from fstab, after its source's mount | `bind`; the class mount's `nodev,nosuid` carried over (to verify) |
| `/run/hearth-activator/top` | the top level, `subvolid=5` | the activator, when it starts | `nodev,nosuid,noexec`; `/run/hearth-activator` is `0700`, root's |
| `/run/hearth-activator/boot` | the boot partition | the activator, for each write, then unmounted | `vfat`, `umask=0077,nodev,nosuid,noexec` |
| `/run/nextroot` | a slot's root, for a soft reboot | the activator (hearth/docs/design/domain/slots/trial.md) | `ro,subvol=/@x1-prod.a` |
| `/run/hearth-activator/layers` | a clone of the top level: the live overlays' layers | the activator, when it starts | `ro`, private; without `noexec` and `nosuid` (hearth/docs/design/domain/slots/trial.md, 8) |
| `/etc`, `/usr/local`, `/opt`, while a live state runs | overlayfs, source `hearth-live`: a delta over the running slot's directory | the activator (hearth/docs/design/domain/slots/trial.md, 8) | `ro,nodev`; `nosuid` on `/etc` only |

- **The store and the state hang below `/var`,** which is per slot, so
  each slot's `/var` gets their mount points at prepare (hearth/docs/design/domain/slots/prepare.md, 3).
  systemd orders a mount after its parent path's (systemd.mount(5)), so
  fstab needs no `x-systemd.requires-mounts-for=`.
- **A bind waits for its source:** systemd orders a bind mount after the
  mount of the path it binds from, as well as after its own parent's,
  so `/var/log/journal` comes up after both `/srv/core` and `/var`. A
  class mount that fails fails its binds, and the boot isolates
  `emergency.target` rather than start a service on the slot's own copy
  of the path (hearth/docs/design/domain/slots/prepare.md, 3).
- **The snapshots are not mounted:** a person reads one through
  `hearth shell`, read-only (hearth/docs/design/client/README.md, 3.7), and `@hearth.snapshots`
  stays the activator's.
- **The store is `nosuid`:** it holds setuid binaries owned by the
  builder's range, behind a `0700` root. Builds run as root inside their
  namespaces and need no setuid binary (hearth/docs/design/domain/build/sandboxes.md).
- **The boot partition is not in fstab:** only the activator uses it,
  mounted for one write at a time, so a power cut rarely finds it
  dirty.

### 3.3 fstab

The activator writes `/etc/fstab` into each slot at prepare
(hearth/docs/design/domain/slots/prepare.md, 5), from typed data: the filesystem's UUID from its machine
record, the target and label, and the `data` part's entries. An output
holds no fstab of its own; the activator replaces whatever is there.

```
# /etc/fstab of slot x1-prod.b, written by hearth's activator at prepare (README.md, 3.3)
UUID=0f3c9a6e-5b2d-4c1e-9a7f-2d8e6b1c4a90  /                      btrfs  subvol=/@x1-prod.b,ro                   0  0
UUID=0f3c9a6e-5b2d-4c1e-9a7f-2d8e6b1c4a90  /srv/core              btrfs  subvol=/@x1-prod.core,nodev,nosuid     0  0
UUID=0f3c9a6e-5b2d-4c1e-9a7f-2d8e6b1c4a90  /srv/data              btrfs  subvol=/@x1-prod.data,nodev,nosuid     0  0
UUID=0f3c9a6e-5b2d-4c1e-9a7f-2d8e6b1c4a90  /var                   btrfs  subvol=/@x1-prod.b.var,nodev,nosuid     0  0
UUID=0f3c9a6e-5b2d-4c1e-9a7f-2d8e6b1c4a90  /var/lib/hearth/state  btrfs  subvol=/@hearth.state,nodev,nosuid,noexec 0  0
UUID=0f3c9a6e-5b2d-4c1e-9a7f-2d8e6b1c4a90  /var/lib/hearth/store  btrfs  subvol=/@hearth.store,nodev,nosuid,noatime 0  0
/srv/data/home-alice                        /home/alice            none   bind                                    0  0
/srv/core/iwd                               /var/lib/iwd           none   bind                                    0  0
/srv/core/journal                           /var/log/journal       none   bind                                    0  0
```

- **Lines in a fixed order:** the subvolumes' mounts in path order,
  then the binds in path order of their targets, so the file is a
  function of its inputs. The order of lines decides nothing: systemd
  orders the mounts by their paths and a bind by its source.
- **The filesystem by UUID,** never by device: the device name differs
  between a live system, a rescue system and the machine itself, and a
  provider may change it.
- **A bind by its source's path,** `/srv/<class>/<name>`, so a restore
  that swaps a directory's name or a class's subvolume is mounted at the
  next boot with no fstab to change (hearth/docs/design/domain/operating.md, 7.4).
- **Every mount point already exists** in the sealed root or in the
  slot's `/var`, because a read-only root cannot get one later, and
  systemd would otherwise try to create it (hearth/docs/design/domain/slots/prepare.md, 3 and 5):
  `/srv/<class>` for each class used, and each bind's path.
- **A container is in no fstab [M5].** Its host's fstab holds the
  host's own lines only; nspawn makes the container's root and binds its
  `/var` and its entries' directories from the activator's mount of the
  top level, as the host's nspawn settings say (hearth/docs/design/domain/slots/containers.md, 1). The
  container's own `/etc/fstab` is written with no line, so its systemd
  mounts nothing.

### 3.4 Names and data entries

- **Target names** are lowercase letters, digits and `-`, with no dot,
  since `.` separates the parts of a subvolume's name; `hearth` is
  reserved for `@hearth.*`. After a target, `a`, `b`, `core`, `data`,
  `cache` and `live` are the only names, and after a class `restoring`
  and `before` are reserved for restores [M5].
- **A `data` entry** (hearth/docs/design/protocol/format/tasks.md, 7) has a name, a class and a path.
  The name is one segment of lowercase letters, digits and `-`, at most
  32 bytes, unique within its target across classes, and names the
  entry's directory in its class's subvolume. The path is absolute and
  normal, beneath `/srv`, `/var` or `/home`, not `/var/lib/hearth` or
  beneath it, not beneath another entry's path, and beneath
  `/srv/core`, `/srv/data` or `/srv/cache` only as the entry's own
  `/srv/<class>/<name>`, which needs no bind. At most 64 entries a
  target, each a line of fstab at most. Render refuses an entry that
  breaks a rule, and the activator refuses it again (hearth/docs/design/domain/activator/README.md, 2.2).
- **A class's contents belong to its entries:** the subvolume is made
  with a root of `0:0 0755`, and each entry's directory when the class
  lacks it, seeded with what the output holds at the entry's path, so
  packages' files there are not hidden by an empty bind (hearth/docs/design/domain/slots/prepare.md, 3);
  the slot's own `tmpfiles.d` lines set owners and modes at boot, by
  name. A container's `0:0` is its own root, in the ids its slots are
  shifted to (hearth/docs/design/domain/slots/containers.md, 1) [M5].
- **Beside an entry's directory,** only a prepare's `<name>.new` while
  it seeds (hearth/docs/design/domain/slots/prepare.md, 3), and a restore's `<name>.restoring` and
  `<name>.before.<seq>` (hearth/docs/design/domain/operating.md, 7.4); entry names hold no dot,
  so none of these is ever an entry.
- **An entry that disappears keeps its directory,** unbound and
  reported as orphaned; a class left with no entry keeps its subvolume,
  unmounted, orphaned too, and its snapshots. Deleting persistent data
  is a person's decision (section 6).
- **An entry that changes class moves no data:** its directory stays in
  the old class, and the new one would get a fresh, seeded directory.
  So the activator refuses a place whose entry has no directory in its
  class but an orphaned one of its name in another of the target's
  classes, and names it; a person moves it, a reflink copy within one
  filesystem (`cp --reflink`), or deletes it.
- **Why a subvolume per class and not per entry:** a class is a policy's
  unit, so a snapshot is atomic across every service in it; there are
  three subvolumes to snapshot, prune and mount instead of one per
  entry; and fstab grows by one bind per entry. A subvolume per entry,
  for one that needs a policy, a quota or a restore of its own, is open
  (section 6).

## 4. Its world

`hearth-domain-disk` and `hearth-domain-boot` run in worlds of the fake
machine (hearth/docs/design/testing/fake-machine.md), which models for them:

- **disks:** gpt and msdos tables, partitions with types and flags, the
  post-MBR gap, FAT volumes, one btrfs filesystem and its subvolumes;
- **firmware:** a bounded variable store, a boot manager that deletes
  `BootNext` before it starts an entry, and `boot.md`, section 5's quirks as
  switches drawn from the seed;
- **GRUB:** a model that runs the fixed `grub.cfg` against the
  environment pair, its saves torn at any sector;
- **boots:** an entry picked as the firmware and GRUB would, models of
  tiny-initramfs and systemd, and reboots, soft reboots and power cuts
  at any instant, mid-write included.

The fakes of the tools in `boot.md`, sections 1 and 2 and `bootstrap.md` take their argument
vectors and act on that machine (hearth/docs/design/testing/tool-fakes.md); their
conformance cases run the real tools in the VM tier
(hearth/docs/design/testing/README.md, section 3 and hearth/docs/design/testing/vm.md): efibootmgr against OVMF, GRUB against
SeaBIOS [M3b].

The invariants every world checks (hearth/docs/design/testing/worlds.md), beside
hearth/docs/design/domain/slots/README.md, section 4's, judged by booting the firmware and GRUB models:

- **no write touches** the default's boot files, its entry, a foreign
  entry, or a file outside the boot partition and the top level;
- **NVRAM writes per rollout stay within `boot.md`, 1.3's count,** and an entry is
  written only when its command line changed;
- **a one-shot whose save fails is never taken,** and a default that
  panics at every boot reaches `previous` on BIOS (hearth/docs/design/domain/slots/bless.md, 2.4);
- **bootstrap ends with one blessed default slot,** or with a typed
  failure and a disk that a second run can finish.

## 5. What changes from os

- **The device leaves the output:** os rendered `root=/dev/nvme0n1p2`
  and fstab's devices from configuration; hearth writes `UUID=` at
  prepare, from the machine record, and fstab from typed data rather
  than a template completed with MiniJinja.
- **Entries at fixed numbers, written only on change:** os re-created an
  entry at every install, at a new number, with labels `a` or
  `x1-prod.a`.
- **`panic=10` always:** os set none, so a missing root hung the
  machine (trials are hearth/docs/design/domain/slots/README.md, section 6's).
- **BIOS works:** os's BIOS path booted a kernel nothing copied, edited
  a `grubenv` GRUB did not read, and kept it on btrfs, where no
  one-shot is possible.
- **tiny-initramfs takes no build host's microcode,** and its module
  lists are task settings instead of a template branching on hardware.
- **The top level holds only subvolumes,** flat; and the root is
  sealed, not only mounted `ro`.
- **Persistent data keeps os's classes, declared per entry.** os made
  `@T.srv` and `@T.srv.{core,data,cache}` at format time and bound
  `journal` and `iwd` from a template; hearth makes `@T.core`, `@T.data`
  and `@T.cache` when an entry uses them, binds each entry from the
  fstab it writes, and snapshots by class itself instead of snapper.
- **Firmware state is read from efivarfs,** not from efibootmgr's text.
- **Bootstrap is one command** with an unprivileged build, instead of
  `format`, `deploy install` under sudo, `prepare-rootfs` and `boot`.
- **Gone:** the `rescue` entry, the `efi-removable` and `efi:grub`
  loaders (until `boot.md`, 5.4), `startup.nsh`, and legacy `@a`/`@b` and labels.

## 6. Open questions

- **Secure Boot:** an enrolled key signing kernels or unified kernel
  images, or shim's chain; depends on which machines require it
  (README, `boot.md`, section 5).
- **Firmware that discards its variables (provider C):** GRUB on the
  removable path with the environment pair on the ESP, or systemd-boot
  with boot counting; depends on M3b and on south's priority (README,
  `boot.md`, section 5).
- **OVMF's reorder:** whether it keeps the relative order of two entries
  on one disk (to verify), which decides how often the activator
  rewrites `BootOrder`.
- **The GRUB script on 2.12:** `load_env -f` with a whitelist, the
  status `save_env -f` returns on FAT, `fallback`, `source` and an
  exported variable in a menu entry (to verify on SeaBIOS).
- **GRUB 2.14's environment block in the btrfs header** (forky, sid):
  one 512-byte sector `grub-editenv` and `save_env` can write, which
  would spare the FAT partition on machines bootstrapped later; depends
  on when the fleet's BIOS machines move past trixie's 2.12.
- **Updating GRUB after bootstrap:** never in M3b, since GRUB is shared
  by both slots and cannot be tried; depends on how often trixie's GRUB
  changes in ways that matter.
- **The `hearth` user's fixed uid and gid,** 59999 (hearth/docs/design/protocol/format/tasks.md, 12), to
  verify against the ids the fleet's packages take.
- **Owners of persistent data:** users made by maintainer scripts get
  dynamic uids, which may differ between outputs; depends on hearth/docs/design/domain/build/README.md
  and hearth/docs/design/protocol/format/README.md pinning them.
- **Deleting orphaned data:** an entry's directory, or a class with no
  entry left, by an operational effect or a person's command; depends
  on hearth/docs/design/domain/operating.md, section 6.
- **A subvolume per entry:** `"subvolume": true` on an entry that needs
  its own snapshot policy, quota or restore unit, made `@<target>.<name>`
  and mounted at its path as before classes; not in the design, a
  possible later addition if a service needs one (3.4).
- **Re-targeting a machine,** or two targets on one disk: the layout
  allows it, bootstrap and entries assume one; depends on the factory's
  need.
- **Watchdogs on bare metal:** whether `x1-prod`'s chipset has an iTCO
  (to verify on the machine).
- **The Incus test VMs:** virtio-scsi as the default disk bus, and
  `security.csm=true` for BIOS (to verify).
- **Disks as an effect [M6]:** how the lab names the disks it orders;
  depends on the pool's design.
