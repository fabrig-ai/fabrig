# The fake machine

## 1. Its parts and its faces

The fake machine is one set of state machines with no protocol or io
layers of its own (skein/docs/foundation/testing-strategy.md, 4.3). It shows a face to
whichever layer of hearth sits just above it:

| hearth is real down to | The machine's face | It answers |
|---|---|---|
| the domain | domain face | typed operations: a subvolume made, snapshotted, frozen, thawed or removed; a tree read or written; a tool run with typed arguments; a variable set; a reboot |
| the protocol layer | io face | io's records: files by name, a full stat, extended attributes, the btrfs ioctls, efivarfs, spawns with argument vectors into namespaces and cgroups, bytes on pipes, exits |
| io | behind the simulator | the operations the simulator passes on (`skein/docs/design/simulator.md`, 3.1), and the machine's power |
| everything | none | the real kernel: a scratch directory, real btrfs, or a VM (`README.md`, section 3 and `real-builds.md` and `vm.md`) |

The io face answers the records hearth asks of skein (hearth/docs/design/README.md, 7.1) and
grows as they do.

It is `testing/hearth-fake-machine`, in two halves:

```
hearth-fake-machine
├── linux     files, btrfs, ids and namespaces, processes, disks, power: no program, no package, no firmware
└── hearth    the package universe and its mirror, firmware, bootloaders, the kernel, PID 1, cues
```

- **Ordinary Rust in a step machine's shape,** as skein's minimal fake
  machine is: one call in, one answer out, held to the step crates'
  lints with a `clippy.toml` of its own. A scenario owns its sizes, tiny
  by default.
- **Its own vocabulary.** It depends on no hearth crate, and on skein's
  io and simulator only in its face behind the simulator, which
  translates.
- **It checks its client.** It refuses what Linux refuses, with Linux's
  error, and asserts what its client guarantees: a handle it issued, a
  process it started.
- **It keeps a ledger:** every operation, who made it (process, ids,
  view; one process until the steward and its builders run apart [M3],
  since skein's calls name no process), what it changed and what it
  refused, in a bounded queue the
  referee drains; and counts a world reads at settle: bytes read and
  written per process, inodes changed, subvolumes made, programs run.
  The steward's rule is read from it (hearth/docs/design/domain/steward.md, 2.2): no open beneath
  the store's `layers/` or `repo/`, and no build tool spawned, by the
  steward's process.

## 2. Files

- **Kinds:** regular files, directories, symbolic links, hard links (a
  link count), FIFOs and device nodes. A user namespace cannot make a
  device node, which is why mmdebstrap's unshare mode leaves `/dev`
  empty.
- **Metadata:** owner and group as on-disk ids (4); the full mode,
  setuid, setgid and sticky included; extended attributes as names and
  bytes (`security.capability`, `system.posix_acl_access`,
  `system.posix_acl_default`, `user.*`); size; inode number; ctime and
  mtime.
- **ctime** moves with every change to an inode: contents, owner, mode,
  attributes, link count, a rename. Its granularity is the scenario's:
  coarse as on trixie's 6.12, where btrfs has no fine-grained ctime yet,
  or fine.
- **Inode numbers** are per subvolume, and a subvolume's root is 256.
  After a remount, the numbers of deleted highest inodes can come back,
  as btrfs's next free id is the highest existing one plus one; a
  reused number comes with a new ctime.
- **A change counter** per inode and per subvolume: temper's file
  version (`temper/docs/design/domain/testing.md`, 4.3), and the ground truth that a
  sealed layer never changes (`worlds.md`, 3).
- **Listings come back in an order drawn from the seed,** so nothing
  hearth computes may depend on it.
- **Contents are kept once, by handle,** and shared between snapshots,
  as btrfs shares extents: a snapshot copies metadata, never bytes.
- **Paths resolve beneath a root** as io resolves them, and access is
  checked by owner, group and other bits and by the capabilities a
  process holds over an inode in its namespace (4).
- **`openat2`'s flags are honoured as Linux answers them:**
  `RESOLVE_BENEATH` refuses an escape (EXDEV), `RESOLVE_NO_SYMLINKS` any
  link on the way (ELOOP), `RESOLVE_NO_XDEV` a mount crossed (EXDEV).
  The activator resolves every path of a slot so
  (hearth/docs/design/domain/activator/README.md, section 2), and the fake gives it the same refusals for
  a hostile tree (`worlds.md`, 5).
- **Renames follow the kernel's rule:** a directory moved into another
  directory needs write permission on itself, to rewrite its `..`; a
  rename within one directory needs only that directory's. A frozen
  subvolume's root refuses write permission (EROFS). This is why
  `<key>.staging` and `<key>.trash` are names inside `layers/`
  (hearth/docs/design/domain/store/README.md, 2.3), and the fake keeps no exception that would hide it.

## 3. btrfs [M2]

One btrfs filesystem per disk, with its top level and the layout
hearth/docs/design/domain/machine/README.md, section 3 fixes. Mount options belong to the filesystem, not
to a subvolume. What a caller may do follows the kernel, as reproduced
unprivileged for this design:

| Operation | Allowed when | Otherwise |
|---|---|---|
| create a subvolume | the caller may write and search the parent directory | EACCES |
| snapshot | the caller owns the source's root: its fsuid is the owner, or it holds `CAP_FOWNER` in a user namespace that maps the owner (`inode_owner_or_capable`) | EPERM |
| set or clear read-only | the same rule, on the subvolume's root | EPERM |
| delete by ioctl | `CAP_SYS_ADMIN` in the initial namespace; or `user_subvol_rm_allowed`, write and search on its root, and rmdir allowed in the parent | EPERM; EROFS on a read-only subvolume, even with the option |
| rmdir an empty subvolume | rmdir allowed in the parent, read-only or not | ENOTEMPTY |
| `GET_SUBVOL_INFO` | anyone: id, uuid, parent uuid, read-only flag, generation | — |
| `GET_SUBVOL_ROOTREF` | anyone: the child subvolumes of a subvolume, 255 a call | — |
| `INO_LOOKUP_USER` | read and search on each component of the path | EACCES |
| list or show (tree search) | `CAP_SYS_ADMIN` | EPERM |

- **Ownership decides who acts on a layer.** A layer's root is the
  namespace root's, the range's base on disk (hearth/docs/design/domain/store/README.md, 2.2). So the
  builder is refused a snapshot, a freeze and a thaw (EPERM), and its
  tree helper, root of a namespace that maps the range, is allowed: it
  snapshots, freezes, thaws, empties and deletes, while the builder
  creates the empty layer, renames within `layers/`, `rmdir`s an empty
  subvolume and reads the three ioctls (hearth/docs/design/domain/store/README.md, 4.2, and hearth/docs/design/domain/store/tree-helper.md, 5, and hearth/docs/design/domain/store/README.md, 5.3).
- **A snapshot keeps** every inode number, ctime and mtime, and its
  root's owner; only the device differs per subvolume. A nested
  subvolume shows in a snapshot as an empty directory with inode 2.
- **A write into a read-only subvolume** is EROFS.
- **The root subvolume's identity** is what `GET_SUBVOL_INFO` on `/`
  answers: the subvolume the machine booted (hearth/docs/design/domain/slots/README.md, section 3).

## 4. Ids, namespaces and the shift [M2]

- **Subordinate ranges** are `/etc/subuid` and `/etc/subgid` on the
  machine. `newuidmap` and `newgidmap` grant exactly the ranges listed,
  plus the caller's own id, and do nothing under `no_new_privs`, which
  disables their setuid bit. The `hearth` user's range is 65536 ids at
  262144 by default (hearth/docs/design/domain/store/README.md, 7.2).
- **A user namespace has an id map of one or more extents:** the tree
  helper's maps the range at 0 and the builder's own ids at 65536
  (hearth/docs/design/domain/store/tree-helper.md, 2); a sandbox's maps the range only (hearth/docs/design/domain/build/sandboxes.md, 1). A
  process sees on-disk ids through its map; an unmapped owner reads as
  65534, and a chown to an unmapped id is refused. The namespace's root
  holds capabilities only over inodes whose owner and group it maps.
- **chown clears setuid, and setgid with group execute,** even for
  root, and removes `security.capability`.
- **A capability set inside a namespace** is stored as version 3, with
  the namespace root's on-disk id as its root id. It reads as version 2
  from inside, and confers nothing outside.
- **ACL entries** are stored with on-disk ids.
- **The shift is the activator's** (hearth/docs/design/domain/slots/prepare.md, 2). The machine keeps
  only the kernel's rules, so a shift that forgets a setuid bit, a
  capability's root id or an ACL leaves the slot visibly wrong, and the
  world compares every prepared slot with a naive shift of its output
  that it computes on its own (`worlds.md`, 3).

## 5. Processes, sandboxes and programs

- **Processes:** trees and process groups; credentials and their
  capabilities in their namespace; `no_new_privs`; user, mount, pid,
  ipc, uts and network namespaces; a mount view (a root and its binds,
  each writable or read-only); a cgroup with its memory and process
  limits; a working directory, an environment, three pipes; an exit
  status or a signal; time taken on the world's clock.
- **A sandbox is a view,** one of hearth/docs/design/domain/build/README.md's two (`worlds.md`, 2): for the packages
  step, a tool root of the builder's `/usr` and `/etc`, read-only, with
  the staging layer at `/target`, the local repository at
  `/run/hearth/repo` and the inputs at `/run/hearth/input`; for every
  other step, the staging layer itself as root. A step reaches nothing
  else, and its network namespace has loopback only.
- **The fetch has no sandbox:** `apt-helper` runs as the builder in a
  cgroup of its own, in the machine's network namespace (hearth/docs/design/domain/build/packages.md, 1).
  The fake mirror and the forge answer only that namespace, so a
  maintainer script or a `customize.sh` that downloads fails as it
  would offline.
- **Stopping is by signal and cgroup:** TERM to the first process, then
  to every process, then a kill of the cgroup; a step is over when its
  cgroup is empty (hearth/docs/design/domain/build/sandboxes.md, 7).
- **Programs run only where they are installed.** A spawn names a path,
  which must exist in the process's view and be executable for its ids:
  the packages step's mmdebstrap, apt and `dpkg-deb` come from the
  builder's `/usr` in the tool root; every other step's tools are the
  layer's own (hearth/docs/design/domain/build/README.md, 6.7); the activator's are the running slot's,
  put there by their packages (`/usr/bin/efibootmgr` from `efibootmgr`).
  A file no program claims exits 127; a missing one is ENOENT.
- **The generic half runs no program.** It reports each spawn to its
  owner, the hearth half, which hands it to a tool's fake (`tool-fakes.md`) or,
  in a simulated world, to one of hearth's processes; as
  `skein-fake-checkout` leaves processes to its world.
- **Shell:** the escape hatches (`customize.sh`, `software.sh`) in the
  test repositories are written in temper's shell subset (
  `temper/docs/design/domain/testing.md`, 4.3), with the few file commands scenarios need.
  Maintainer scripts are not shell (6).

## 6. The package universe

Drawn from the seed, and shaped like Debian's and like the recording of
the local mirror (`real-builds.md`, 2). A fake mirror serves it at snapshot timestamps.
Its index serves resolution from M1; what installs it comes with builds
at M2.

```
package    name, version, architecture, priority, Essential, section
relations  Depends, Pre-Depends, Provides, Conflicts, with their version relations
files      path, kind, mode, owner and group by name, contents by the package's handle
           (one seed per package, so a version's files are the same at every install);
           conffiles marked
scripts    typed effects, never shell: make a system user or group; set a file capability;
           run /etc/kernel/postinst.d; apply the package's tmpfiles lines; make the
           kernel's /vmlinuz and /initrd.img links; register an alternative; fail (a fault)
archive    Installed-Size; the .deb's size and SHA-256, as Packages lists them
```

- **Snapshots:** the mirror holds the universe at several timestamps,
  and a bump changes some versions. A timestamp between two snapshots
  answers as the earlier one, as snapshot.debian.org redirects.
- **Variants** as mmdebstrap defines them: `essential`, the Essential
  packages; `apt`, those and apt; `required`, `important` and
  `standard`, each the packages of its priority and those above it, with
  apt.
- **No Breaks, Replaces or Recommends,** until a scenario shows that
  one changes a closure (`README.md`, 3.1). The fake mmdebstrap asserts `APT::Install-Recommends "false"` (hearth/docs/design/domain/render/spec.md, 2.2),
  naming the option where a world passes another.
- **base-files and base-passwd:** `/usr/lib/os-release`,
  `/etc/debian_version`, and the static users and groups.
- **dpkg's database, of the real shape:** `/var/lib/dpkg/status` with
  each package's stanza, in the order of installation (Package, Status, Priority, Section,
  Installed-Size, Maintainer, Architecture, Version, Depends, Conffiles
  with their md5sums, Description), and `info/<package>.list`,
  `.md5sums` and `.conffiles`.
- **Users made by maintainer scripts** take the next free system id, so
  their ids depend on the order of installation, as on Debian. Users
  made by hearth's sysusers files are the config layer's, with the ids
  it gives them (hearth/docs/design/domain/build/README.md, 6.2), never the universe's.
- **Conffiles** under `/etc`, recorded with their md5sums. One changed
  both locally and in the package is the question dpkg answers by its
  options (`--force-confold`).
- **Setuid files and capabilities,** as Debian ships them: `su` 4755,
  `unix_chkpwd` 2755, a capability set by a postinst as
  `iputils-arping`'s sets `cap_net_raw+ep`. The shift has something to
  get wrong.
- **A kernel package** installs `/boot/vmlinuz-<version>`, its config
  and `/usr/lib/modules/<version>`, and runs `/etc/kernel/postinst.d`.
  The initramfs package's hook there writes `/boot/initrd.img-<version>`
  from `/etc/tiny-initramfs/tiny-initramfs.conf`. Debian's `/vmlinuz`
  and `/initrd.img` links point at the pair; hearth reads the versioned
  files (hearth/docs/design/domain/machine/boot.md, 1.1 and 2.4).
- **The fake initramfs** records which modules it holds, and the fake
  kernel mounts its root only if the disk's driver and btrfs are among
  them (7). Its bytes differ at every build while its inputs (the
  configuration, the modules, the generator) do not, so only
  hearth/docs/design/domain/slots/trial.md, 1's rule, which compares the inputs' digest, allows a soft
  reboot after a rebuild (`worlds.md`, 3). With `AUTO_MODULES=yes` the hook
  inspects the running system, which in a sandbox is the wrong one:
  render refuses it and the build checks again (hearth/docs/design/domain/build/packages.md, 5), and the
  fake keeps the behaviour so that both checks are tested.
- **The offenders of hearth/docs/design/domain/build/README.md, 7.2:** openssh-server's postinst makes
  host keys, ssl-cert's a snakeoil pair, systemd's a machine-id, and a
  trigger writes a log, so the customize hook and the scan have work.
- **Subvolumes made at build:** `systemd-container` ships
  `Q /var/lib/machines` in its tmpfiles, and its postinst applies it.
  The fake's tmpfiles makes a subvolume for a `v`, `q` or `Q` line whose
  path is absent on btrfs, and leaves an existing directory alone, as
  systemd-tmpfiles does. The setup hook pre-makes the directories
  (hearth/docs/design/domain/build/packages.md, 5), so the layer holds a directory and no nested
  subvolume (`worlds.md`, 5).
- **hearth's own tools are packages too:** mmdebstrap, apt, uidmap,
  efibootmgr, grub, git, systemd.
- **Small:** tens of packages and hundreds of files, so a fake essential
  bootstrap takes milliseconds.

## 7. Firmware and boot [M3]

A scenario picks the machine's kind:

| Kind | Firmware | Disk and boot | Watchdog |
|---|---|---|---|
| `efi` | UEFI as the specification says | GPT; ESP; direct stub entries (hearth/docs/design/domain/machine/boot.md, section 1) | none |
| `efi-ovmf` | OVMF as Incus runs it | the same; prunes and reorders `BootOrder`'s options at every boot | iTCO |
| `efi-forgetful` | UEFI that loses its variables at a drawn boot | the same, and the removable path `\EFI\BOOT\BOOTX64.EFI` | none |
| `bios` [M3b] | SeaBIOS | msdos; GRUB's core in the post-MBR gap; all of GRUB on the FAT partition (hearth/docs/design/domain/machine/boot.md, section 2) | iTCO on q35 |

**EFI variables:**

- **efivarfs is a filesystem of the fake machine:**
  `/sys/firmware/efi/efivars/<Name>-<GUID>`, each file 4 attribute
  bytes and then the variable, in the shapes hearth/docs/design/domain/machine/README.md (`tool-fakes.md`, 5) decodes.
  The activator reads them there; efibootmgr is its only writer.
- **A `Boot####` option** is an `EFI_LOAD_OPTION`: attributes, a
  description, a device path (the partition and the file) and optional
  data, the command line in UCS-2. Each slot has two at fixed numbers,
  its normal entry and its trial entry, whose command line carries the
  `hearth.trial=` token (hearth/docs/design/domain/machine/boot.md, 1.2). The fake stub reads the command line
  up to its first NUL, so the arguments efibootmgr joins with NULs are
  lost, as on a real machine; x86 caps the line at 2048 bytes.
- **`BootNext` is deleted** by the boot manager before it starts the
  option it names; `BootCurrent` is set to the option booted. A load
  that fails, a missing file or Secure Boot, falls back to `BootOrder`.
- **OVMF,** at every boot and before it chooses, drops the options
  listed in `BootOrder` whose file cannot be loaded or whose disk has no
  bootindex, and reorders `BootOrder` by QEMU's device order; options
  outside `BootOrder`, such as a trial entry, it leaves alone. The other
  quirks of hearth/docs/design/domain/machine/boot.md's section 5 (`BootNext` ignored, entries outside
  `BootOrder` dropped) are switches drawn from the seed.
- **The variable store fills.** It has a capacity (256 KiB in OVMF's
  4 MB build), and a deleted variable's space comes back only at the
  next boot, so cycles of create and delete without a reboot fill it.
- **Secure Boot** is off unless a scenario turns it on; then a Debian
  kernel loaded directly fails to load (hearth/docs/design/domain/machine/boot.md, section 5).

**GRUB [M3b]:**

- **All of GRUB is on the FAT partition:** its modules, the fixed
  `grub.cfg` written at bootstrap, and the environment pair
  `hearth/env.0` and `hearth/env.1`, 1024-byte blocks (hearth/docs/design/domain/machine/boot.md, 2.2
  and 2.5). Each slot carries its `linux` and `initrd` lines in its own
  fragment, `/.hearth/grub.cfg`, which GRUB sources from the slot's
  subvolume (hearth/docs/design/domain/machine/boot.md, 2.4).
- **The fake GRUB runs that `grub.cfg`:** it loads `env.0`, or `env.1`
  if `env.0` is not a valid block; if `next_entry` is set, it becomes
  `default`, and is cleared and saved in both files before any kernel
  loads, each `save_env` writing the whole 1024-byte block, two sectors;
  `fallback` is the default, and covers only an entry whose loading
  returns. Its boot counter is the one hearth/docs/design/domain/machine/README.md's `grub.cfg` keeps
  (`../domain/machine/boot.md`, section 2).
- **A save can fail,** on a full or read-only partition: GRUB then boots
  the entry anyway, with `next_entry` still set.
- **On btrfs `save_env` fails** and GRUB boots the entry anyway, leaving
  `next_entry` set: the trap of trixie's GRUB 2.12. The fake keeps it,
  so a world shows that hearth never puts an environment there.

**The boot:**

- **The kernel** boots the image an option or a fragment names, with
  its command line. Its initramfs mounts `root=` with
  `rootflags=subvol=` only if its modules hold the disk's driver and
  btrfs, and the subvolume exists. Otherwise it prints, waits ten
  seconds and exits, and the kernel panics, as tiny-initramfs does.
- **PID 1** reads from the slot only the keys the fake models, from
  where systemd reads them: `RuntimeWatchdogSec=`,
  `hearth-trial.target`'s `JobTimeoutSec=` and `JobTimeoutAction=`, the
  units enabled, their conditions on a path or on the kernel's command
  line, `IgnoreOnIsolate=`, the default target, and fstab's mounts of
  `/var`, the class subvolumes at `/srv/<class>` and the entries' binds.
  A bind waits for the mount its source lies on and for its own
  parent's, as systemd orders it, and fails if either fails. A unit's
  outcome is its package's, or its cue's (`worlds.md`, 2). The boot reaches its
  target, or does not.
- **A mount that fails** isolates `emergency.target`, which stops every
  unit but those with `IgnoreOnIsolate=yes`, and leaves unmounted what
  the mount would have shown. So the trial's units, which arm from a
  marker in `/run` or the `hearth.trial=` token and never from a mount,
  keep running there (hearth/docs/design/domain/slots/trial.md).
- **`is-system-running`** answers `degraded` while a unit has failed.

## 8. Reboots, soft reboots, panics and the watchdog [M3]

- **A reboot** stops every process (TERM, then KILL), syncs, unmounts,
  and hands over to the firmware.
- **`systemctl reboot -ff`** is `reboot(2)` at once: no unit stops.
  Whether it syncs first is to verify, so the fake does not.
- **`systemctl reboot`** turns into a soft reboot while `/run/nextroot`
  is a mount point, as systemd 257 does, unless
  `SYSTEMCTL_SKIP_AUTO_SOFT_REBOOT=1` is in its environment. This is why
  the dead-man unit uses `-ff` and the activator sets the variable
  (hearth/docs/design/domain/slots/trial.md, 3 and 5).
- **A soft reboot** keeps the kernel, its command line and `/run`, runs
  no firmware, bootloader or initramfs, and starts PID 1 in
  `/run/nextroot`; units marked to survive do. The new root needs
  `/usr/lib/modules/<running version>`, or every unit that loads a
  module fails.
- **A panic** reboots after `panic=N` seconds. Without `panic=` on the
  command line it hangs for ever, as Debian's kernel does.
- **Hangs come in three kinds:** the kernel's, which only a hardware
  watchdog ends; PID 1's, which the watchdog ends if
  `RuntimeWatchdogSec=` armed it; and a boot that never reaches its
  target, which `JobTimeoutAction=` or the dead-man unit ends. A hang
  nothing ends lasts until the referee's deadline fails the test (`worlds.md`, 3).
- **The watchdog** resets the machine when PID 1 stops pinging it for
  its timeout. Whether it stays armed across a soft reboot is to verify,
  so the seed draws either.

## 9. Power cuts [M2]

A power cut falls between any two of the machine's operations, or inside
a write: the referee picks one, the seed draws one, or a sweep takes
each in turn. The model follows skein's for its fake disk (
`skein/docs/design/kv.md`, section 8), and where a guarantee is still to verify it takes
the weaker one, so hearth is never written against a promise the kernel
may not keep:

| What | After a power cut |
|---|---|
| file data, synced | kept |
| a filesystem after `syncfs` | everything written to it before, kept |
| file data, not synced | any of its writes, in any order, the last torn at any byte |
| a name made, renamed or removed | kept once its directory is synced; a rename whole or not at all |
| a subvolume made or snapshotted | whole or absent; kept once the call returned (to verify) |
| a subvolume deleted | gone, or back whole (to verify when it is final) |
| the read-only flag | old or new (to verify) |
| a file on FAT: the ESP's kernels, an environment block | synced, kept; otherwise old, torn or absent; a file rewritten in place, as `grub-editenv set` truncates and rewrites one, may be found empty; a rename on FAT leaves either name, both or neither (to verify) |
| GRUB's own save | the 1024-byte block's two sectors, each old or new |
| an EFI variable | old or new, never torn (edk2's fault-tolerant writes: to verify); of several, any prefix |
| `BootNext` the firmware has read | gone |

The machine then powers on, and the firmware boots what it would (7).

## 10. A full disk, and the other faults [M2]

- **Capacities** per filesystem, tiny in worlds: btrfs data and
  metadata; the ESP; GRUB's env partition; the variable store. A
  snapshot needs metadata, and a shift unshares the metadata it touches
  (tens of MB on a real root), so a disk that held the output can fill
  while preparing a slot.
- **ENOSPC** falls at a drawn write.
- **The simulator's own disk faults** (`skein/docs/design/simulator.md`, section
  4: an I/O error, a filesystem gone read-only, an operation that hangs)
  apply behind it. The machine adds its own: a variable store full,
  firmware that forgets, a disk missing at boot, a GRUB save that fails,
  a mount that fails at boot, and a published layer whose contents were
  lost (a file torn as an unsynced write would leave it).
- **Out of memory:** a cgroup past its `memory.max` loses a process of
  its own subtree, killed: a step's in its leaf (hearth/docs/design/domain/build/sandboxes.md, 7), or,
  drawn by the seed, the builder itself when `steward/builder/` reaches
  its ceiling, never a process outside it (hearth/docs/design/domain/steward.md, 5.3).

## 11. Shared with temper

temper plans the same machine (`temper/docs/design/domain/testing.md`, 3.2 and 4.3):
files with versions, per-tree views, processes, and programs whose
protocol is their command line. hearth builds it first.

- **It starts in hearth,** as `testing/hearth-fake-machine` at M1.
- **Its generic half names nothing of hearth's:** no program, package,
  firmware, slot or store. It depends on nothing in the hearth half, so
  it can move unchanged.
- **No shared types** (skein/docs/foundation/testing-strategy.md, section 4). Its vocabulary is
  its own; hearth's worlds translate to and from it, and temper's will.
  It reuses no io record outside its face behind the simulator.
- **temper's needs are in it from the start:** the change counter is
  temper's version; mount views are temper's per-tree views. temper's
  programs stay temper's, and git's working trees stay
  `skein-fake-checkout`'s.
- **Only the generic half moves to skein,** when temper builds its
  machine (`temper/docs/design/domain/testing.md`, 4.3): as a kit beside
  `skein-fake-checkout`, with its tests and hearth as its first user
  (skein/docs/foundation/testing-strategy.md, sections 7 and 9). The hearth half (the
  universe, firmware, bootloaders, the kernel, PID 1, the tools' fakes)
  stays hearth's, as `hearth-fake-machine`, on the kit, since a
  service's fake machine is the service's (skein/docs/foundation/testing-strategy.md, 4.3).
- **skein's minimal fake machine stays minimal,** as
  `skein/docs/design/testing.md`, section 4 says; the kit does not grow it.

## 12. Containers [M5]

A host's containers are part of the hearth half, on the host's fake
kernel (hearth/docs/design/domain/slots/containers.md):

- **nspawn as a unit:** `systemd-nspawn@<c>.service` starts a container
  only if `/run/machines/<c>` links to a directory and
  `/run/systemd/nspawn/<c>.nspawn` exists; it reads `PrivateUsers=`,
  `ReadOnly=`, `Bind=`, `BindReadOnly=`, `Capability=`,
  `VirtualEthernet=` and `LinkJournal=` from it, and fails otherwise,
  as one started at boot before its link would.
- **A container is a view and a PID 1:** the slot's root, read-only,
  with its binds, seen through an id map of the container's range (4);
  the fake PID 1 of 7 runs inside it, with its units, failed units and
  `is-system-running`, and no kernel, firmware or initramfs of its own.
- **Reached from the host** with `systemctl --machine=` and
  `systemd-run --machine=`, which the fake answers from that PID 1;
  whether the real ones need a bus inside is the VM tier's question
  (`vm.md`, 3).
- **A host reboot** empties `/run`, so no container starts until the
  host's activator makes its link again; a power cut stops every
  container with its host.

## 13. Mounts, overlays and live changes [M3]

What a live change (hearth/docs/design/domain/slots/trial.md, 6 to 9) needs of the machine. The
mounts and overlayfs are the generic half's; PID 1's jobs are hearth's
half's.

- **Mount namespaces and propagation:** each namespace a tree of
  mounts, each `shared`, `private` or `slave`; a mount made in a shared
  one appears in its peers and its slaves. systemd's services hold slave
  copies with binds of their own (`ProtectSystem=`, `ReadOnlyPaths=`,
  `InaccessiblePaths=`), as the units in a scenario declare.
- **The new mount API:** `fsopen`, `fsconfig` and `fsmount` give a
  detached mount that no namespace shows, walkable through its
  descriptor; `move_mount` attaches it, with `MOVE_MOUNT_BENEATH` under
  the current top, refused on a scenario's kernel older than `real-builds.md`, 5, as
  `lowerdir+` is before 6.8; `umount2(MNT_DETACH)` removes a mount at
  once while open files and working directories keep it; `open_tree`
  clones and `mount_setattr` changes a clone's flags; `setns` moves a
  process into another namespace [M5]. `/proc/<pid>/mountinfo` is
  rendered from the tree, as the kernel renders it.
- **overlayfs, read-only:** lower layers stacked, topmost first; a 0/0
  character device hides a path, a whole directory included; a
  directory takes the topmost layer's attributes; a kind change needs no
  marker; a file reports its layer's device and inode, a directory the
  overlay's. A write to a layer while it is merged is a finding the
  referee reports, since the kernel leaves it undefined.
- **Propagation's two traps,** as measured on the kernel: a lazy unmount
  of a shared peer of a layer's source tears the overlay down; an
  overlay propagated into a service's namespace shadows that service's
  own mount beneath it until the service restarts.
- **PID 1's jobs:** `daemon-reload`, `stop`, `start`, `restart`,
  `reload` and `reload-or-restart`, each job with a result; a unit's
  `ActiveState`, `InvocationID` and `NRestarts`. A reload whose
  `ExecReload=` fails leaves the unit active and the job failed; a new
  `.wants` link starts nothing until started; a unit whose file is gone
  stays loaded while active. dbus and nscd are units with a reload.
- **Soft reboots and overlays:** with `/run/nextroot`, the old root's
  overlays go with it; a person's soft reboot without it keeps the
  overlay on `/etc` and drops `/usr/local`'s and `/opt`'s. Both are to
  verify (`vm.md`, 3), so until then the seed draws either.
- **Cues** (`worlds.md`, 2) that only a live trial meets: `test-reload-fails`,
  whose `ExecReload=` fails when its configuration says so, and
  `test-dies-later`, which dies a drawn time after it starts.
