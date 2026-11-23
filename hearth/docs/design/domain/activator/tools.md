# The activator: the tools it drives

## 1. The tools it drives

### 1.1 Formats, each its own machine

Every format the activator reads or writes is a lib-only protocol
machine crate, fuzzed alone (`skein/docs/foundation/programming-model.md`, section 4); the
domain sees only typed values.

| Crate | Format | Way |
|---|---|---|
| `hearth-protocol-link` | the link's records (`README.md`, 4.1), `TryLive`'s plan included (`operations.md`, 17) | both |
| `hearth-protocol-state` | the records of `@hearth.state` (`state.md`, 1.1), the trial marker, the live record and its manifest (hearth/docs/design/domain/slots/README.md, 2.4), the classes' snapshot records [M5], the slot record inside the root (hearth/docs/design/domain/slots/README.md, 2.2) | both |
| `hearth-protocol-accounts` | `/etc/passwd`, `/etc/group`, `/etc/shadow` lines | read; `/etc/shadow` written |
| `hearth-protocol-units` | fstab (hearth/docs/design/domain/machine/README.md, 3.3), the trial's units and the watchdog's drop-in (hearth/docs/design/domain/slots/prepare.md, 5); a container's nspawn settings [M5] (hearth/docs/design/domain/slots/containers.md, 1) | written |
| `hearth-protocol-xattr` | `security.capability` v2 and v3, POSIX ACL values (hearth/docs/design/domain/slots/prepare.md, 2) | both |
| `hearth-protocol-tree` | the store's layer records and tree files, read; the slot manifest, a tree file (hearth/docs/design/domain/store/records.md, 2 and 3) | both |
| `hearth-protocol-efi` | efivarfs variables and `EFI_LOAD_OPTION` (hearth/docs/design/domain/machine/boot.md, 1.5); efibootmgr's argument vectors | both |
| `hearth-protocol-grub` [M3b] | `grub.cfg`, a slot's fragment, the environment block (hearth/docs/design/domain/machine/boot.md, section 2); grub-editenv's and grub-install's argument vectors | both |
| `hearth-protocol-systemd` | systemctl's and systemd-run's lines: states, counts, ids, failed units, `Result`; a job's outcome, and `ActiveState`, `InvocationID`, `NRestarts` and `MainPID` per unit (hearth/docs/design/domain/slots/trial.md, 8) | read |
| `hearth-protocol-proc` | `/proc/<pid>/mountinfo` lines: mount ids, parents, mount points, propagation, fstype, source and options, an overlay's layers included; `/proc/<pid>/cgroup`, a process's unit | read |
| `hearth-protocol-keygen` | ssh-keygen's argument vector and public line | both |
| `hearth-protocol-dpkg` | dpkg's status, for capturing a mutable machine's packages (hearth/docs/design/domain/development.md, section 5) | read |

`hearth-protocol-activator` is the protocol layer: it routes these
machines, and holds only the argument vectors of tools whose answer is
an exit status (mount, parted, the mkfs tools and the rest of 1.2).

### 1.2 Tools

| Tool | For | Protocol | What the domain gets |
|---|---|---|---|
| btrfs ioctls: `SNAP_CREATE_V2`, `SUBVOL_CREATE`, `SNAP_DESTROY_V2`, `SUBVOL_SETFLAGS`, `GET_SUBVOL_INFO`, `GET_SUBVOL_ROOTREF`; `FICLONE` | snapshots, slots, sealing, the running slot; cloning `/var`, seeding entries and restoring one; classes, their snapshots and pruning [M5] | skein's ring adapter: a closed list of synchronous ioctls (an ask of skein, README, 1.1) | typed results |
| files: `openat2` with `RESOLVE_BENEATH`, `RESOLVE_NO_SYMLINKS` and `RESOLVE_NO_XDEV`; stat without following, readlink, extended attributes, chown, chmod, mkdir, symlink, link, unlink, rmdir, `Store` | the shift, `/var`, the slot's files, the state | skein's io, with the records hearth asks for (README, 1.1) | typed results |
| mounts: `fsopen`, `fsconfig`, `fsmount`, `move_mount` with `MOVE_MOUNT_BENEATH`, `open_tree`, `mount_setattr`, `umount2` with `MNT_DETACH`; `mknodat` of a 0/0 character device; `setns` into a container's mount namespace [M5], in a child | live overlays: the layers' source, the swap and the swap back, the whiteouts (hearth/docs/design/domain/slots/trial.md, 8, and hearth/docs/design/domain/slots/containers.md, 8) | skein's io, asks of skein (README, 1.1) | typed results; the mount table through `hearth-protocol-proc` |
| efivarfs reads | entries, `BootOrder`, `BootNext`, `BootCurrent`, `SecureBoot` | `hearth-protocol-efi` | typed variables |
| `/usr/bin/efibootmgr` | create, delete, `--bootnext`, `--delete-bootnext`, `--bootorder` | `hearth-protocol-efi` | success, or exit and tail |
| `/usr/bin/grub-editenv` [M3b] | `set` and `unset` in `env.0`, then `env.1` | `hearth-protocol-grub` | success, or exit and tail |
| `/usr/sbin/grub-install` [M3b] | bootstrap, once | `hearth-protocol-grub` | success, or exit and tail |
| `/usr/bin/mount`, `/usr/bin/umount` | the top level, the boot partition, `/run/nextroot` | `hearth-protocol-activator` | success, or exit and tail |
| `/usr/bin/systemctl` | `soft-reboot`, `reboot`, `is-system-running`, `list-units --state=failed`, `show`, `stop`; for a live trial, `start` and `stop` of the base's dead-man, `daemon-reload`, `reload`, `restart`, `reload-or-restart`, `start` and `stop` of the plan's units (hearth/docs/design/domain/slots/trial.md, 8); `restart` [M5]; for a container [M5], `start`, `restart` and `is-active` of `systemd-nspawn@<container>.service`, and `is-system-running` and `list-units` with `--machine=<container>`; `--machine=<container> kill` [M6] | `hearth-protocol-systemd` | a state, a count, an id, a list of units |
| `/usr/bin/systemd-run` | the hold's timer [M4]; checks as transient units, `--machine=<container>` for a container's [M5] | `hearth-protocol-systemd` | the unit's exit status and `Result`, nothing of its output |
| `/usr/bin/systemd-tmpfiles` | `--create /etc/tmpfiles.d/hearth.conf` in a live trial (hearth/docs/design/domain/slots/trial.md, 8) | `hearth-protocol-activator` | success, or exit and tail |
| `/usr/sbin/nscd` | `--invalidate passwd` and `group`, in a live trial, when nscd runs | `hearth-protocol-activator` | success, or exit and tail |
| `/usr/bin/ssh-keygen` | identity's host keys | `hearth-protocol-keygen` | success; the keys as files |
| `/usr/sbin/wipefs`, `/usr/sbin/parted`, `/usr/sbin/partprobe`, `/usr/bin/udevadm`, `/usr/sbin/mkfs.vfat`, `/usr/sbin/mkfs.btrfs`, `/usr/sbin/useradd`, `/usr/sbin/usermod` | bootstrap; disks as an effect [M6] | `hearth-protocol-activator` | success, or exit and tail |
| `/usr/bin/btrfs` `send`, `receive`, `qgroup limit` [M6] | backups, and a restore from the peer; a full-disk fault | `hearth-protocol-activator` | a stream, and success |
| `/usr/bin/age`, `/usr/bin/age-keygen` [M6] | encrypting a backup stream, decrypting one received; the backup key | `hearth-protocol-activator`; the recipient line, `hearth-protocol-keygen` | a stream, and success; the `age1…` recipient |
| `/usr/sbin/nft`, `/usr/sbin/tc` [M6] | partition, latency and loss faults on a pool host | `hearth-protocol-activator` | success, or exit and tail |

- **Spawned with an absolute path,** an argument vector and an
  environment given whole (skein spawns with no `PATH` search):
  `LANG=C.UTF-8`, `PATH=/usr/bin:/usr/sbin` for tools that run helpers,
  and `SYSTEMCTL_SKIP_AUTO_SOFT_REBOOT=1` for every reboot it asks for
  (hearth/docs/design/domain/slots/bless.md, 2.1). stdin is `/dev/null`; stdout and stderr are kept to
  their last 4 KiB, which never carry a slot's file.
- **Each with a deadline,** after which it is killed: 30 seconds for
  efibootmgr, grub-editenv, mount and systemctl; minutes for mkfs and
  grub-install.
- **No tool walks a slot.** `cp` and `rm` would resolve paths in the
  activator's own namespace; the activator copies and empties through
  descriptors itself (hearth/docs/design/domain/slots/prepare.md, 3).
- **Not the `btrfs` tool** for subvolumes: its `list` and `show` need a
  tree search, and its text would be one more protocol for what the
  ioctls give typed.
- **Each tool has a fake** on the fake machine with the same face
  (hearth/docs/design/testing/tool-fakes.md).
