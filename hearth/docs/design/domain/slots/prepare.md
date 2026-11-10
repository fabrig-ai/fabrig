# Preparing a slot

Inputs, from the place request (hearth/docs/design/domain/activator/operations.md, 1): the slot, the
output's key and tree digest, its kernel version and digests, the
`data`, `secrets` and `boot` parts, the package users, the trial's
window, the units allowed to fail, and the target's kind and firmware
from `meta`, or a container's host, with its parameters in its `boot`
part [M5] (`containers.md`, 1).

- **Identity first:** before anything else the activator makes every
  identity item the request declares and the store lacks
  (hearth/docs/design/domain/activator/operations.md, 7), then computes the prepare key (`README.md`, 2.2), whose
  identity generation is then final. A request whose key the slot
  already holds is answered at once, as done.
- **Every path in the slot is resolved from the snapshot's root
  descriptor** with `openat2(RESOLVE_BENEATH | RESOLVE_NO_SYMLINKS |
  RESOLVE_NO_XDEV)` (an ask of skein, README, `bless.md`, 2.1), never by a path name
  in the activator's own namespace: a link the output plants at `/var`,
  `/etc` or a kernel's path cannot lead the activator to the running
  machine's files. An output whose `/var`, `/etc`, `/boot`,
  `usr/lib/modules/<version>` or a mount point's parents are not
  directories, or whose kernel, initramfs, `/etc/passwd`, `/etc/group`
  or `/etc/shadow` are not regular files, is refused; no error carries
  a file's content.
- **Each step starts by writing the activator's journal**
  (hearth/docs/design/domain/activator/README.md, 6.3), so a crash finds the slot `preparing` and deletes
  it (`crashes.md`).

## 1. The snapshot

1. **Take the slot out of the firmware's fallbacks:** on EFI its normal
   entry leaves `BootOrder` if it is there (it is, when the slot holds
   the previous blessed output); on BIOS `previous` is cleared if it
   names the slot (`bless.md`, 2.4) [M3b]. A one-shot that names it, a
   `live-blessed` slot's (`bless.md`, 1.6), is cleared too: `BootNext` on EFI,
   `next_entry` on BIOS; then its marker. So nothing falls through to,
   or boots, a slot being written.
2. **Delete the slot's old subvolumes,** nested ones first: the children
   of its `.var` (systemd's `/var/lib/machines` and
   `/var/lib/portables`), found with `BTRFS_IOC_GET_SUBVOL_ROOTREF`; then
   the `.var`; then the root, read-only or not (the activator is root).
3. **Open the output by its key,** beneath
   `/run/hearth-activator/top/@hearth.store/` without following links
   (hearth/docs/design/domain/store/README.md, section 2), and check it (hearth/docs/design/domain/activator/README.md, 5.1).
4. **Snapshot it, writable,** with `BTRFS_IOC_SNAP_CREATE_V2` from the
   descriptor opened in step 3, into the top level as `@x1-prod.b`.
5. **Check the snapshot** (hearth/docs/design/domain/activator/README.md, 5.2): its parent uuid is the
   output's; the output is still read-only, with the same change
   generation. Record the new subvolume's id and uuid.

A snapshot is a metadata operation: milliseconds, and no data copied.

## 2. The shift

The output is owned on disk by the builder's subordinate range, base
`B` (hearth/docs/design/domain/store/README.md, section 7), and its extended attributes are in that
range's terms. The shift makes it owned by real ids, metadata only, in
one depth-first walk of the snapshot with a bounded stack
(`hearth-domain-tree`, a child of `hearth-domain-slots`). For each
inode, once (hard links are found by inode number):

1. **stat it without following links** (`statx`, `AT_SYMLINK_NOFOLLOW`);
   refuse any owner or group outside `[B, B + 65536)`, and any block or
   character device (none can be made in a user namespace);
2. **read `security.capability`** (regular files), and
   **`system.posix_acl_access`** and, for directories,
   **`system.posix_acl_default`**, before anything else: `chown(2)`
   clears capabilities;
3. **`lchown` to `uid - B`, `gid - B`;**
4. **restore setuid and setgid** with `chmod` if the mode had them (not
   on links): `chown(2)` clears them, for root too;
5. **write the capability back as v2:** a v3 capability with root id `B`
   (what a namespace's root writes) becomes v2 with the same sets; a v2
   one is written back as it was; a v3 with any other root id is refused;
6. **rewrite each ACL's `ACL_USER` and `ACL_GROUP` qualifiers** from
   `q` to `q - B`, refusing any outside the range;
7. **hash its contents** if it is a regular file (hearth/docs/design/domain/activator/README.md, 5.3),
   and **write its manifest entry,** the same entry the builder's tree
   helper wrote inside its namespace.

- **The ids land where the namespace saw them.** With `B` aligned to
  64 Ki, `uid - B` is the id inside the builder's namespace, and a
  capability or ACL read by root now reads as the namespace read it. So
  the digest of the shifted tree, computed as the walk goes, must equal
  the output's tree digest: that is the shift's own verification
  (hearth/docs/design/domain/activator/README.md, 5.3).
- **What it costs:** about 10 µs an inode for the metadata, so half a
  second to two seconds for a root of 50k to 100k inodes, plus hashing
  contents when the activator hashes again. Data extents stay shared
  with the store; btrfs copies the metadata leaves it touches, tens of
  MB per slot. Each step's time is recorded as a cost.
- **A cut leaves a half-shifted tree,** which nothing can tell from a
  whole one by looking; the slot is still `preparing`, so it is deleted
  (`crashes.md`).
- **Precedents:** systemd's `shift-uid.c` shifts ACLs and restores modes
  but leaves capabilities alone; Incus's `shift_linux.go` rewrites
  capabilities with a new root id.

## 3. /var

A layer is one subvolume, `/var` included (hearth/docs/design/domain/store/README.md, section 4); a slot
splits it off, so that dpkg's database travels with its slot and the
root can stay read-only.

1. **Make `@x1-prod.b.var`** with `BTRFS_IOC_SUBVOL_CREATE`; its root
   inode takes the owner and mode of the output's `/var`.
2. **Copy the output's `/var` into it, sharing extents,** in one walk
   through descriptors: directories made, regular files cloned with
   `FICLONE` (an ask of skein, README, `bless.md`, 2.1), links and hard links made
   again, and owners, modes, times and extended attributes copied as
   the shift left them. Anything but a directory, a regular file or a
   link beneath `/var` is refused. No `cp`: it resolves paths in the
   activator's own namespace.
3. **Empty the root's `/var`,** depth first, by `unlinkat` on directory
   descriptors; `/var` itself stays, as the mount point.
4. **Make the classes and the entries' directories** (hearth/docs/design/domain/machine/README.md, 3.4).
   For each class an entry of the `data` part uses, make
   `@x1-prod.<class>` with `BTRFS_IOC_SUBVOL_CREATE` if the top level
   lacks it, its root `0:0 0755`. For each entry whose directory its
   class lacks, refuse the place if a directory of its name is orphaned
   in another of the target's classes; otherwise make it as
   `<name>.new`, deleting one a cut left, seed it from the output's
   content at the entry's path, cloned as in step 2, so what packages
   put there (a database's cluster, a mail spool) is not hidden by an
   empty bind, and rename it to `<name>`, so a cut never leaves a
   half-seeded entry under its name. With nothing at that path in the
   output, it is made `0:0 0755`. An existing directory is never
   touched. The slot's own copy at the entry's path stays, hidden by the
   bind.
5. **Make the mount points** that a sealed root cannot make later:
   `lib/hearth/state` and `lib/hearth/store` in the slot's `/var`;
   `srv/<class>` in the root, for each class used; each bind's path, in
   the slot's `/var` if it is beneath `/var`, in the root otherwise,
   where the output lacks it. Missing parents are made `0:0 0755`.
6. **Amend the manifest:** the entries beneath `/var` leave the root's
   manifest for the `/var` manifest, whose digest is the record's `var
   tree`; the mount points made join.

A copy, rather than a second snapshot of the output trimmed down to its
`/var`: trimming would unlink every other file of the output, about
15 µs each, while `/var` holds a few thousand files.

## 4. Identity and secrets

Written from the secret store in `@hearth.state` (hearth/docs/design/domain/activator/README.md, 6.1)
into the slot, through descriptors (`prepare.md`): every parent must
already be a directory in the slot; the file is created anew, replacing
a file at its path and refusing a link there. A path beneath `/var` is
written into the slot's `.var` subvolume, which 3 has made, and its
entry joins the `/var` manifest.

- **The identity** (hearth/docs/design/protocol/format/README.md, 7.3), made before the prepare key
  (`prepare.md`):
  - the machine-id, every target's and never declared, at
    `/etc/machine-id`, `0:0 0444`: 32 hexadecimal digits and a newline.
    The journal's directory is named by it, so it must be the same for
    every slot of the target, and it must be written: on a read-only
    root an empty file means a new id at every boot;
  - each item of the `secrets` part's `identity` map (hearth/docs/design/domain/render/spec.md, 1.3),
    at its `path`, with its `public_path`, mode and owners: an SSH host
    key in M3, such as `/etc/ssh/ssh_host_ed25519_key`. An output holds
    none (hearth/docs/design/domain/build/README.md, section 7).
- **Secret files** from the `secrets` part: name, path, mode, owner,
  group. Owner and group are names, resolved against the slot's own
  `/etc/passwd` and `/etc/group` after the shift, through bounded parsers
  (hearth/docs/design/domain/activator/README.md, 2.3); an unknown name, or a missing secret, fails the
  prepare and names them all.
- **Never readable by the steward or its builder:** a secret or
  identity file owned by `hearth`, or by a group `hearth` belongs to,
  and a secret whose mode lets others read it, are refused (hearth/docs/design/protocol/format/tasks.md,
  11 refuses them first).
- **Password hashes,** secrets too, for the users the `secrets` part
  names: the activator replaces the second field of those users' lines
  in the slot's `/etc/shadow`, keeping the file's owner, group and mode.
  It is the one edit hearth makes to a file the output holds, and it is
  a fixed operation: no other field and no other file.
- **What leaves the activator:** the identity's public halves, the
  secrets' names with present or missing, and the store's generation.
  Never a value, nor a digest of one (7).
- **Rotation:** an import bumps the secret store's generation, which
  changes the prepare key, so converge prepares the same output again
  into the other slot (hearth/docs/design/protocol/standalone.md, 6), and the new secret arrives by trial
  and bless like any change.

## 5. The slot's files

Written from typed data into the root before it is sealed, each into the
manifest, by the format crates of hearth/docs/design/domain/activator/README.md, 7.1:

| Path | Owner, mode | Content |
|---|---|---|
| `/etc/fstab` | `0:0 0644` | hearth/docs/design/domain/machine/README.md, 3.3 |
| `/.hearth/slot` | `0:0 0444` | the record inside the root (`README.md`, 2.2) |
| `/.hearth/grub.cfg` [M3b] | `0:0 0444` | the slot's `linux` and `initrd` lines (hearth/docs/design/domain/machine/boot.md, 2.4) |
| `/etc/systemd/system/hearth-trial.timer` | `0:0 0644` | the dead-man's timer |
| `/etc/systemd/system/hearth-trial.service` | `0:0 0644` | the dead-man's reboot |
| `/etc/systemd/system/hearth-trial-arm.service` | `0:0 0644` | starts the trial's target when the slot is on trial |
| `/etc/systemd/system/hearth-trial.target` | `0:0 0644` | the trial boot's job timeout |
| `/etc/systemd/system/sysinit.target.wants/hearth-trial.timer`, `…/hearth-trial-arm.service` | links | evaluated at every boot |
| `/etc/systemd/system.conf.d/zz-hearth-watchdog.conf` | `0:0 0644` | `RuntimeWatchdogSec=`, when the machine has a watchdog device; named to sort last |

An output that holds `/.hearth`, any of these unit names, or a drop-in
sorting after `zz-hearth-watchdog.conf` is refused: they are hearth's
(hearth/docs/design/protocol/format/tasks.md, 11). The units, for slot `x1-prod.b` with a 10-minute
window (their exact behaviour is to verify on the VM tier):

```
# hearth-trial.timer
[Unit]
Description=hearth: end the trial of x1-prod.b unless it is blessed
DefaultDependencies=no
IgnoreOnIsolate=yes
ConditionKernelCommandLine=|hearth.trial=x1-prod.b
ConditionPathExists=|/run/hearth-trial/x1-prod.b

[Timer]
OnActiveSec=10min
AccuracySec=1s

# hearth-trial.service
[Unit]
Description=hearth: the trial of x1-prod.b was not blessed in time
DefaultDependencies=no
IgnoreOnIsolate=yes

[Service]
Type=oneshot
ExecStart=/usr/bin/systemctl reboot -ff

# hearth-trial-arm.service
[Unit]
Description=hearth: arm the trial boot of x1-prod.b
DefaultDependencies=no
IgnoreOnIsolate=yes
ConditionKernelCommandLine=|hearth.trial=x1-prod.b
ConditionPathExists=|/run/hearth-trial/x1-prod.b

[Service]
Type=oneshot
ExecStart=/usr/bin/systemctl start --no-block hearth-trial.target

# hearth-trial.target
[Unit]
Description=hearth: the trial boot of x1-prod.b
DefaultDependencies=no
IgnoreOnIsolate=yes
After=multi-user.target
JobTimeoutSec=10min
JobTimeoutAction=reboot-force

# zz-hearth-watchdog.conf, when /sys/class/watchdog/ lists a device
[Manager]
RuntimeWatchdogSec=30s
RebootWatchdogSec=10min
```

- **Armed by what exists before any mount:** the kernel command line's
  `hearth.trial=<target>.<label>`, which a one-shot boots with (`trial.md`, 3), or
  `/run/hearth-trial/<target>.<label>`, which the activator writes before
  a soft reboot (`trial.md`, 2) and which `/run` carries across it. No trial unit
  depends on a mount, so a failed `/var` cannot leave a trial unarmed.
- **Inert elsewhere:** a blessed slot's own boot has neither, and a
  fallback boot of the old slot names another slot.
- **Emergency mode does not disarm it:** `IgnoreOnIsolate=yes` keeps the
  timer and the target when systemd isolates `emergency.target`, and the
  timer alone still reboots (to verify that the target's pending job
  survives an isolate too).
- **Only the arming starts the target:** were it enabled for every boot,
  a blessed slot that took longer than the window to boot would be
  rebooted by its timeout, again and again. Queued early in the boot,
  its job times out unless `multi-user.target` is reached within the
  window.
- **It depends on nothing of the new release:** the units are written by
  the activator that prepares the slot, the old blessed one, and they
  use only systemd. A slot whose hearth never starts still reboots.
- **`OnActiveSec=` counts from the timer's start,** which a soft reboot
  starts afresh; `OnBootSec=` would count from the kernel's boot, which
  a soft reboot keeps.
- **After a soft reboot the command line keeps its token,** so a person's
  soft reboot of a slot that was booted on trial arms it once more, and
  ends in one full reboot into the default; every soft reboot hearth
  makes is a trial anyway.

## 6. Boot files and entries

- **EFI:** mount the ESP at `/run/hearth-activator/boot`; write
  `vmlinuz.x1-prod.b` and `initrd.img.x1-prod.b` from the slot's
  `boot/vmlinuz-<version>` and `boot/initrd.img-<version>`, read
  through descriptors (`prepare.md`), each through a temporary file, synced
  and renamed; unmount. Then read the slot's two entries, normal and
  trial, from efivarfs and make again each one that is missing or
  differs, at its number (hearth/docs/design/domain/machine/boot.md, 1.2 and 1.3). Both stay out of
  `BootOrder`.
- **BIOS [M3b]:** nothing on the boot partition changes at prepare. The
  slot's fragment is one of its files (5), and `grub.cfg` is fixed.
- **Containers [M5]:** no boot files; the host's nspawn settings instead
  (`containers.md`).

## 7. Verifying

1. **Seal the root:** `BTRFS_IOC_SUBVOL_SETFLAGS` with
   `BTRFS_SUBVOL_RDONLY`; left out for a mutable slot, whose fstab
   mounts `/` `rw` (hearth/docs/design/domain/development.md, 4.2). Its manifest is written all
   the same: capture compares against it (hearth/docs/design/domain/development.md, section 5).
2. **Check the package users and groups:** every user and group a task
   declares as made by a package (hearth/docs/design/protocol/format/tasks.md, 6), such as `postfix`,
   must be in the slot's `/etc/passwd` or `/etc/group`, read through the
   same bounded parsers as 4's; a missing one refuses the slot, naming
   every one missing.
3. **Walk the sealed root's metadata** against the manifest, without
   contents, and record each entry's inode and change time as the seal
   left them. Anything else that differs (nothing should) fails the
   prepare. `H("tree", …)` over the manifest's entries (hearth/docs/design/domain/store/README.md, 3.4)
   is the slot's own tree digest; it stays in the state.
4. **Compute the reported tree digest,** the same formula with each
   secret and identity file's content digest replaced by
   `H("secret", name, generation)`: the slot's digest the facts carry to
   the steward and to an engine, which says when a secret changed but
   not what it holds.
5. **On EFI, read the boot files back** from the ESP and check their
   digests against the kernel's, and **read both entries back** and
   check them field by field.
6. **Write the slot record,** state `prepared`, then clear the journal.
   The answer carries the prepare key and the reported tree digest.
