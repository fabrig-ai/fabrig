# A slot's trial

## 1. Which kind

Three kinds, the first that applies:

1. **Live,** with no reboot, when the change is eligible (7): the new
   slot's files are mounted over the running slot's (6 to 9).
2. **A soft reboot,** when the kernel is unchanged (below).
3. **A one-shot reboot** otherwise.

A container's trial is live when eligible, and otherwise a restart,
with no kernel to compare (`containers.md`, 2, 8) [M5]. For a machine's own slots:

**The kernel is unchanged,** and a soft reboot enough, when the new slot
and the booted kernel (`README.md`, section 3) agree on all four of:

| Digest | Of | From |
|---|---|---|
| image | the content of `boot/vmlinuz-<version>` | the output's tree file (hearth/docs/design/domain/store/records.md, 3); the shift's walk finds it again |
| modules | the subtree `usr/lib/modules/<version>` (hearth/docs/design/domain/store/README.md, 3.4) | the tree file; the walk finds it again |
| initramfs inputs | `H("initramfs", …)` over the versions of `tiny-initramfs-core` and of the kernel's package, its configuration (which lists the modules), the kernel version, and with `MICROCODE=generic` the microcode packages' versions | the resolution's closure (hearth/docs/design/domain/render/spec.md, 2.4), which hearth/docs/design/domain/build/packages.md, 7 checks is what was installed, and the `packages` part's initramfs configuration (hearth/docs/design/domain/render/spec.md, 1.2); the builder computes it with the output (hearth/docs/design/domain/build/packages.md, 8; hearth/docs/design/domain/store/records.md, 3) and the place request carries it (hearth/docs/design/domain/activator/operations.md, 1) |
| command line | the line without the slot's own tokens, `initrd=` and `rootflags=` | the slot record (hearth/docs/design/domain/machine/boot.md, section 3) |

- **The initramfs is compared by its inputs, never its bytes.** mktirfs's
  cpio carries inode numbers and mtimes, so two builds of the same inputs
  differ (hearth/docs/design/domain/build/reproducible.md, 2.2, row 4); comparing bytes would force a full
  reboot at every snapshot bump.
- **It counts at all because a soft reboot never runs it.** A changed
  initramfs would first run at the next full boot, after the bless, with
  no trial to catch it; a change to its inputs takes a one-shot reboot,
  which runs it.
- **The modules must match** because the running kernel loads its
  modules from the new root. Whether depmod's indexes beneath them
  reproduce across builds is to verify (hearth/docs/design/domain/build/reproducible.md, section 4);
  until then a snapshot bump that keeps the kernel may still cost a full
  reboot.
- **The command line must match** because the running kernel keeps its
  own, and systemd in the new root reads its `systemd.*` options.
- **A one-shot reboot** otherwise: `BootNext` (EFI), or `next_entry`
  (BIOS) [M3b].
- **The steward may ask for a full reboot** (`how: full`), to try the
  firmware's path as well; it may not ask for a soft reboot the rule
  forbids.
- **`how: auto`** takes the first kind that applies; **`how: live`**
  asks for live only, and is refused, with the rules that fail, when the
  change is not eligible; **`how: full`** a one-shot reboot
  (hearth/docs/design/protocol/wire.md, 4.4).
- **The steward asks for live,** with `try live`, since half of 7's
  rules read the specs and the store, which a builder checks for it in
  its `plan live` job (hearth/docs/design/domain/steward.md, 3.3); the activator checks the rest
  and refuses otherwise, and the steward then asks an ordinary `try`.
- **The activator chooses between soft and one-shot,** since that rule
  reads only the records and facts it owns, and it answers with the
  kind it chose.

## 2. By soft reboot

1. **Write the marker,** phase `arming` (4).
2. **Mount the slot's root** as a real btrfs mount, read-only, so that
   the new root's mount table shows its own subvolume:

   ```
   /usr/bin/mount -t btrfs -o ro,subvol=/@x1-prod.b /dev/nvme0n1p2 /run/nextroot
   ```

   It is the mount tiny-initramfs would have made, since the initramfs
   does not run again (hearth/docs/design/domain/machine/boot.md, 4.4).
3. **Write `/run/hearth-trial/x1-prod.b`,** which `/run` carries across
   the soft reboot and which arms the new root's dead-man (`prepare.md`, 5).
4. **Set the marker's phase to `armed`.**
5. **Answer the request, then `systemctl soft-reboot`.**

What the new root finds:

- **the same kernel, the same boot id, and `/proc/cmdline` unchanged:**
  it still names the old slot, which is why nothing reads it;
- **`SoftRebootsCount` one higher,** which tells the new activator that
  the root switched;
- **its own fstab mounted afresh:** its `/var`, the classes and their
  binds, the store and the state; `/` remounted by systemd with fstab's
  `subvol=` (to verify);
- **`/run` carried over;** the activator mounts `/run/hearth-activator/top` again
  if the switch dropped it.
- **No live overlay:** a live state's overlays sit on the old root's
  directories and go with it (to verify); a `live-blessed` slot tried by
  a soft reboot comes up natively, its own files on its own root.

## 3. By one-shot reboot

1. **Check the slot's trial entry** (EFI): present and as written, or
   made again (hearth/docs/design/domain/machine/boot.md, 1.2). Its command line is the normal entry's
   with `hearth.trial=x1-prod.b`, which arms the dead-man (`prepare.md`, 5).
2. **Write the marker,** phase `arming`.
3. **Set the one-shot:** `efibootmgr --bootnext 4811`, the trial entry;
   or, on BIOS, `next_entry=x1-prod.b` in `env.0`, then `env.1`, from
   which GRUB adds the same token (hearth/docs/design/domain/machine/boot.md, 2.3).
4. **Set the marker's phase to `armed`.**
5. **Make sure `/run/nextroot` is not a mount point,** unmounting a
   leftover, and answer the request.
6. **Reboot:** `systemctl reboot`, with
   `SYSTEMCTL_SKIP_AUTO_SOFT_REBOOT=1` in its environment.

On the trial's boot, the new slot's activator checks that the one-shot
was consumed: `BootCurrent` is the slot's trial entry and `BootNext` is
gone (EFI), or `next_entry` is empty in both files (BIOS); a
`next_entry` still set is cleared at once (hearth/docs/design/domain/machine/boot.md, 2.6).

## 4. The trial marker

```
/var/lib/hearth/state/trial/<target>.<label>
  slot, attempt          what is tried, and the steward's attempt id
  kind                   soft | one-shot | live
  phase                  arming | armed | running | held [M4]
                         live: arming | armed | swapped | running | live-blessed | held [M4]
  boot id, soft reboots  at arming: how a later start tells "same boot" from "new boot"
  armed at               wall clock
  window                 the dead-man's, from the place request, sealed into the units (5.5);
                         a live trial's is the base's (6.8, L3)
  live                   base, delta, previous delta; the unit plan; after L6, each unit's
                         InvocationID and NRestarts, which the settle compares
```

- **At most one marker exists.** It is written before the one-shot and
  removed only by a bless, an abort or the start that records the
  outcome.
- **`running`** is set by the trial slot's own activator when it starts
  there. It is the difference between a slot that never reached hearth
  (failed to load, panicked, hung early) and one that reached it and was
  not blessed in time; both fall back, and the record keeps which.
- **It does not arm the dead-man:** it lives on a mount, which may fail.
  The command line's token or `/run/hearth-trial/` does (`prepare.md`, 5); the
  marker is the activator's record of the trial.
- **A live trial's marker** is written at L3 (8) and kept through the
  live bless, in phase `live-blessed`, until the slot's native boot sets
  `running` (kind `one-shot` from then) and its bless removes it.

## 5. How every trial ends

| What happens in the trial | What ends it | Where it lands |
|---|---|---|
| the minimal criterion holds, and the slot's steward asks | bless (`bless.md`, section 1) | the new slot, now the default |
| the trial's kernel fails to load | the firmware moves on to `BootOrder`; GRUB's `fallback` | the default, in the same boot |
| the kernel or the initramfs panics (a missing root after 180 s) | `panic=10` | the default: the one-shot was consumed |
| the kernel or PID 1 hangs | the hardware watchdog (`RuntimeWatchdogSec=`), where the machine has one | the default |
| userspace never finishes booting | `JobTimeoutAction=reboot-force` on `hearth-trial.target` | the default |
| a mount fails and systemd isolates `emergency.target` | the dead-man's timer, kept by `IgnoreOnIsolate=yes`, armed without any mount | the default |
| it boots, but no bless comes: a failed unit, a broken hearth | the dead-man: `systemctl reboot -ff` after the window | the default |
| the slot's steward finds a definite failure: a failed unit that counts (`degraded`), or its builder cannot render the slot's revision (hearth/docs/design/domain/steward.md, section 6) | it asks `fall back` (`bless.md`, section 2) | the default |
| the power is cut | the next power-on | the default |
| live: a unit action fails, a touched unit dies in the settle, the criterion fails | the live fall back (`bless.md`, 2.5) | the base's files, with no reboot |
| live: the live fall back itself fails or hangs | `reboot -ff`, or the base's dead-man | the default |
| live: no live bless within the base's window | the base's dead-man, `reboot -ff`; no one-shot is set before the live bless | the default |
| live: the activator crashes after arming | `reboot -ff` at its restart (`crashes.md`) | the default |
| live: a power cut, a panic, a watchdog reset | the next power-on; the overlays were kernel state | the default |
| live: the live bless | the one-shot set to the slot's trial entry (`bless.md`, 1.6) | the base, still running; the next boot is the slot's trial |

Two gaps remain: a kernel that hangs with no hardware watchdog, which
needs a person or, with an engine, a power cycle from outside [M4]; and
a firmware that ignores `BootNext`, where the trial never runs and is
reported (hearth/docs/design/domain/machine/boot.md, section 5). A hardware watchdog's behaviour across
a soft reboot is to verify.

## 6. Live

A live trial runs the new slot's files over the running slot, which
stays the root: no reboot and no soft reboot. With `x1-prod` running
`a` and `b` prepared:

- **The base, `a`,** stays the root subvolume, running, blessed and the
  default throughout. Its `/var` keeps running; `b`'s `/var` is not used
  until `b` boots.
- **The new slot, `b`,** is prepared in full first, exactly as for any
  trial (`prepare.md`): shifted, verified, sealed, with its record. The
  live trial reads it; nothing runs from it.
- **The delta, `@x1-prod.live.<n>`,** a read-only subvolume at the top
  level, holds only what differs between `a` and `b` under `etc/`,
  `usr/local/` and `opt/`: reflink clones of `b`'s changed and added files,
  with `b`'s owners, modes, extended attributes and capabilities, and an
  overlayfs whiteout, a 0/0 character device, at each deleted path. `n`
  only grows, from the live record (`README.md`, 2.4).
- **The overlays:** one read-only overlayfs per hierarchy the delta
  changes, the delta over the base's directory, which the activator
  mounts on `/etc`, `/usr/local` and `/opt` itself. A process that opens a
  path afterwards gets `b`'s file; one already open stays `a`'s, and
  nothing on disk changes, so there is no `ETXTBSY` and no torn file.
- **The units** that read the changed files are stopped, reloaded,
  restarted or started, as the tasks declare (hearth/docs/design/protocol/format/tasks.md, 14).
- **The live bless arms `b`'s trial, and does not move the default**
  (`bless.md`, 1.6). `a + delta` is not `b`: `b`'s early boot, generators, udev,
  sysctl and every unit run `b`'s configuration only at `b`'s first
  native boot, and the units acted on live are a small part of what a
  boot starts. So the live bless sets the one-shot to `b`'s trial entry,
  and whatever ends the uptime next (a person's reboot, a power cut, a
  panic) boots `b` on an ordinary trial, with its dead-man and its own
  bless. Only that bless moves the default.

**hearth mounts the overlays itself, not through `systemd-sysext` and
`systemd-confext`.** Their `refresh` is not atomic: it unmounts the old
overlay before it mounts the new one, so for about half a millisecond on
a small tree (measured at systemd 257 and 261) `/etc` shows the bare
base and files only the extension had are missing, and a refresh that
fails after the unmount leaves no overlay at all; the fix (systemd PR
#43680) is in no release. A mismatched extension makes them silent
no-ops: an `extension-release` that does not match is ignored with exit
status 0, `merge` mounting nothing and `refresh` unmerging.
`systemd-confext` mounts `/etc` `noexec` by default, and Debian runs
scripts from `/etc`. Their behaviour differs across the versions hearth
meets: 260 and later skip a refresh they judge unchanged, 257 follows no
symlink under `--root`, 262 adds unit restarts of its own. And their
search paths in `/run` survive a soft reboot, where the new root's boot
services would merge the delta over whatever root came up. hearth's own
mount, `fsopen("overlay")` then `move_mount` with `MOVE_MOUNT_BENEATH`,
is the kernel mechanism upstream's fix and NixOS's `/etc` use: it swaps
with no window, fails without touching the old overlay, and costs about
as much code as driving the tools and checking their output. The
`hearth` task disables both services (hearth/docs/design/protocol/format/tasks.md, 12), so nothing
merges extensions behind hearth's back.

## 7. What may be live

An allowlist. A builder decides it when the steward has it plan the
trial (`plan live`, hearth/docs/design/domain/steward.md, 3.3), from the two outputs' tree files
(a sorted diff, hearth/docs/design/domain/store/records.md, 3) and the specs' `live` parts
(hearth/docs/design/domain/render/spec.md, 1.4); the activator checks again what its own records and
the machine show. A change that fails any rule takes an
ordinary trial (1). Not being live is never an error: the facts say
why (hearth/docs/design/protocol/wire.md, section 5).

| # | Rule | Why | Checked by |
|---|---|---|---|
| 1 | **The same packages layer:** both outputs have one packages key | packages, the kernel, its modules, the initramfs's inputs, systemd, dbus and libc stay; a packages change is never live | builder; activator, from the layer records' parents (hearth/docs/design/domain/activator/README.md, 5.1) |
| 2 | **The prepare key equal but for the output,** the window and the units allowed to fail: the same `data`, `secrets` and `boot` digests, the same identity and secret store generations | a new data entry, a class changed or a policy changed alters the `data` part: a new bind changes fstab and the class's directory must be made and seeded at prepare; a new command line changes the boot; a rotated secret is not in the output's diff | both, from the slot records |
| 3 | **Changes only under `etc/`, `usr/local/` and `opt/`** | `/var` content reaches a slot only at prepare; `/boot` and the rest of `/usr` are the packages', which rule 1 keeps and the build refuses above them (hearth/docs/design/domain/build/README.md, 7.3) | both |
| 4 | **No boot-time or PID 1 configuration** (hearth/docs/design/protocol/format/tasks.md, 14) | read only at boot or at PID 1's re-exec; applied live it is not what a boot does (a removed sysctl is not reverted), so the trial would test the wrong thing | both |
| 5 | **Users and groups only added:** no line of `passwd`, `group`, `shadow`, `gshadow`, `subuid` or `subgid` removed, no id changed | a renumbering touches files in `/var` and data; a removal strands running processes | builder; activator, with its account parsers (hearth/docs/design/domain/activator/README.md, 2.3) |
| 6 | **tmpfiles only creatable:** the changed lines of `etc/tmpfiles.d/hearth.conf` are of types `systemd-tmpfiles --create` applies whole; none is `r`, `R` or `D` | the output's `/var` got them at build; the running `/var` needs them now, and nothing removes live | builder |
| 7 | **Every changed path has a consumer:** a task's `live` entry, or one hearth knows (hearth/docs/design/protocol/format/tasks.md, 14); a deleted path's from the running slot's spec, every other path's from the new one | hearth must know what to reload or restart; guessing from paths is how this fails | builder |
| 8 | **Nothing never-live** (hearth/docs/design/protocol/format/tasks.md, 14): systemd's own units and configuration, dbus (its policy files are allowed, with a reload of dbus), logind, journald, udevd, networkd, NetworkManager, the gettys, AppArmor, and hearth itself for now | restarting them live is risky or wrong | builder; activator, for the plan's units |
| 9 | **No file dpkg owns** under `usr/local/` or `opt/` changed by the software layer, by the packages layer's dpkg database; conffiles in `etc/` are fine | a download over a packaged file is a packages change in disguise | builder |
| 10 | **The machine fits:** Linux 6.8 or later (`MOVE_MOUNT_BENEATH` since 6.5, overlayfs's `lowerdir+` since 6.8; trixie runs 6.12); `etc`, `usr` and `opt` are directories in the base; no mount on or beneath them but hearth's; running = default = blessed; no trial; at most one live state, whose base is the running slot; [M5] the new slot's window no longer than the base's | the swap's mechanisms; a base proven by a native boot; the base's dead-man ends the trial | activator |
| 11 | **Sandboxes beneath the hierarchies:** units whose mount namespace has a mount beneath `/etc`, `/usr/local` or `/opt` join the restarts, and if one is never-live the change is not eligible | the overlay, propagated into the unit's namespace, shadows that mount until the unit restarts | activator, as root, from `/proc/<pid>/mountinfo` of every process and its cgroup |
| 12 | **An unexplored base, on a mutable machine:** a metadata walk of the base's `etc/`, `usr/local/` and `opt/` matches its manifest (hearth/docs/design/domain/development.md, 4.4) | what was changed by hand beneath the overlay would stay visible and fail L2; a soft reboot discards it instead | activator |

- **What the steward sends:** `try live`, with the digest of the change
  from the base's output to the new one, which the delta will hold, and
  the unit plan (hearth/docs/design/domain/activator/operations.md, 17): the units to stop before the swap,
  to reload, restart or start after it, and whether `daemon-reload`,
  tmpfiles, D-Bus or the accounts need acting on: all as its builder's
  plan answered them.
- **What the activator recomputes:** the change between the two slots'
  manifests, leaving out each slot's own files (`prepare.md`, 4, 5), whose digest
  (in hearth/docs/design/domain/store/README.md, 3.5's form) must equal the plan's; then rules 1 to 5,
  8 for the plan's units, 10, 11 and 12. The delta is built from its own
  diff, never from paths the steward sent. A difference refuses the
  trial, naming the rule.
- **A later live change** is planned against what runs, the live
  state's files, and its delta built against the base (9).

## 8. The steps

Every step that leaves the state starts by writing the journal
(hearth/docs/design/domain/activator/README.md, 6.3); the marker carries the phase from L3.

| Step | What the activator does | Phase after |
|---|---|---|
| **L0. Prepare `b`** | `prepare.md`, unchanged: `b` sealed and verified, with its record | none: `b` is `prepared` |
| **L1. Build the delta** | make `@x1-prod.live.<n>` and fill it from the change (below) | none |
| **L2. Verify, then seal** | walk the merged view in no mount namespace anyone sees (below); then `BTRFS_SUBVOL_RDONLY` on the delta | none |
| **L3. Arm** | write the marker: kind `live`, phase `arming`, base `a`, the delta, the previous one, the boot id and soft reboot count. Write `/run/hearth-trial/x1-prod.a`, then `systemctl start hearth-trial.timer`: `a`'s own dead-man (`prepare.md`, 5), whose condition the file meets and whose `OnActiveSec=` counts from now; check it is active | `armed`; `b` is `on live trial` |
| **L4. Stop** | `systemctl stop`, one call: the units being removed or disabled, and each `restart` unit with a changed path under `usr/local/` or `opt/`; every job's result checked | `armed` |
| **L5. Swap** | for `/etc`, then `/usr/local`, then `/opt`: the overlay built and moved beneath the current top (below); the mount table read back; the live record written | `swapped` |
| **L6. Act** | `systemctl daemon-reload` if a unit file, drop-in or `.wants` link changed; `systemd-tmpfiles --create /etc/tmpfiles.d/hearth.conf` if it changed; `systemctl reload dbus` if D-Bus policy changed; `nscd --invalidate` for `passwd` and `group` if nscd runs and the accounts changed. Then the declared reloads, restarts and starts as one batch (below), every job checked. Then answer the request and spawn the steward anew (hearth/docs/design/domain/activator/README.md, 3.3), so the live bless comes from a steward started in the new files, whose builder renders the slot's revision (hearth/docs/design/domain/steward.md, section 6) | `running` |
| **L7. Settle** | 60 seconds to start, within the base's window: every unit L4 to L6 touched stays `active` with the `InvocationID` and `NRestarts` L6 read; the minimal criterion holds (`bless.md`, 1.1); from M5, the blessing checks pass (hearth/docs/design/domain/operating.md, section 3) | `running` |
| **L8. Live bless** | asked by the steward: 7.6. The default does not move | `live-blessed`; `b` is `live-blessed` |
| **L9. The native boot** | the next boot consumes the one-shot and boots `b`'s trial entry: `b` runs on trial (3), its dead-man armed by `hearth.trial=`; its activator sets the marker `running`; its steward blesses it (`bless.md`, 1.3), the default moves to `b`, and the delta is deleted | the marker removed by the bless |

**L1, the delta.** `BTRFS_IOC_SUBVOL_CREATE` makes it, with `etc`,
`usr` and `opt` carrying `b`'s metadata. The change is walked in tree
order through descriptors, `b`'s paths resolved beneath `b`'s root as
prepare resolves them (`prepare.md`), the delta's beneath the delta's:

- **A put:** missing parents made with `b`'s owner, mode, ACLs and
  extended attributes, since a merged directory takes the topmost
  layer's attributes and a default `0755` parent would loosen
  `/etc/sudoers.d`; a regular file cloned with `FICLONE`, then its
  owner, mode, extended attributes and v2 capability, in the shift's
  order (`prepare.md`, 2), since `chown` clears setuid bits and capabilities; a
  link made again; two changed paths that share an inode in `b` share
  one in the delta. A changed path whose other link is unchanged
  becomes an inode of its own, with another link count: harmless, and
  neither L2's walk nor drift compares link counts.
- **A delete:** `mknodat(…, S_IFCHR, makedev(0, 0))` at each topmost
  deleted path, a whole directory included. A path whose kind changes
  needs no whiteout and no opaque directory.
- **Built from the prepared slot,** the delta holds real ids, setuid
  bits and capabilities as the shift left them; the overlay presents
  them unchanged.

**L2, the merged view.** The three overlays are built exactly as L5 will
build them, with `fsopen` and `fsmount`, and never attached: a detached
mount is in an anonymous mount namespace no process sees, and the
activator walks it through its descriptor (hearth/docs/design/domain/activator/README.md, 5.5). The walk
must give the expected manifest: `a`'s manifest under `etc/`,
`usr/local/` and `opt/`, with each path of the change replaced by `b`'s
entry or removed. It compares type, mode, owners, size, extended
attributes and link targets for every entry, and contents for the
delta's files; not inode numbers or change times, which an overlay
reports from its layers or itself. Any difference fails the trial, and
the delta is deleted. The expected manifest becomes the live record's
(`README.md`, 2.4). It is hearth/docs/design/domain/store/README.md, 3.5's rule for a change set, kept for a delta:
applied and walked, it must give the child's tree. Then the delta is
sealed: nothing writes a layer once it is merged, which overlayfs calls
undefined.

**L4 and L6, the units.** Each changed path's consumers
(hearth/docs/design/protocol/format/tasks.md, 14) decide:

- **Before the swap** stop what must not straddle it: a `restart` unit
  whose binary, library or tree under `usr/local/` or `opt/` changes, so it
  never runs new code with old files or the reverse (NixOS stops changed
  units first for the same reason), and a unit whose file is removed or
  whose `.wants` link goes, since `ExecStop=` may name a file the delta
  removes.
- **After the swap,** in this order: reloads, then restarts (a
  `restart` unit with changes under `etc/` only, a unit whose file or
  drop-in changed, and rule 11's units), then `reload-or-restart`, then
  starts (the units L4 stopped, new units and newly wanted ones, and
  `start` units). One `systemctl` call per verb, each enqueuing its
  units' jobs in one transaction and waiting for them. A new `.wants`
  link starts nothing by itself, and a swap fires no inotify event on
  the base's inodes, so every action is explicit.
- **Every job's result counts, not the unit's state:** each is read back
  (`ActiveState`, `Result`, `InvocationID`, `NRestarts`). A reload of
  `ssh` that fails `sshd -t` leaves the unit active and the job failed,
  which is the live fall back (`bless.md`, 2.5).
- **What is not acted on:** shells, cron jobs and timers already
  running, and people's processes, which see a mixed world until they
  exit.

**L5, the swap,** for each hierarchy the delta changes:

```
fs  = fsopen("overlay", FSOPEN_CLOEXEC)
fsconfig(fs, FSCONFIG_SET_STRING, "lowerdir+", "/run/hearth-activator/layers/@x1-prod.live.<n>/etc")
fsconfig(fs, FSCONFIG_SET_STRING, "lowerdir+", "/run/hearth-activator/layers/@x1-prod.a/etc")
fsconfig(fs, FSCONFIG_SET_STRING, "source", "hearth-live")
fsconfig(fs, FSCONFIG_CMD_CREATE)
mnt = fsmount(fs, FSMOUNT_CLOEXEC, MOUNT_ATTR_RDONLY | MOUNT_ATTR_NODEV | MOUNT_ATTR_NOSUID)   NOSUID on /etc only
a hearth overlay on /etc:  move_mount(mnt, "", AT_FDCWD, "/etc", MOVE_MOUNT_F_EMPTY_PATH | MOVE_MOUNT_BENEATH)
                           umount2("/etc", MNT_DETACH | UMOUNT_NOFOLLOW)                        the old top
otherwise:                 move_mount(mnt, "", AT_FDCWD, "/etc", MOVE_MOUNT_F_EMPTY_PATH)
```

- **No window:** readers go from the old view straight to the new, and
  never see the base nor a missing file, as measured with readers
  opening a file in a tight loop. A `move_mount` that fails leaves the
  old top in place, so a failed hierarchy changes nothing.
- **Exec is allowed** on all three; `nosuid` is set on `/etc` only,
  until setuid files above the packages are settled (hearth/docs/design/domain/build/README.md,
  section 13): if they are refused, `/usr/local` and `/opt` take it
  too.
- **A hierarchy the new delta does not change** loses a previous overlay
  by `umount2(MNT_DETACH)` alone, and shows the base.
- **The hierarchies are not atomic with each other:** `/etc`, then
  `/usr/local`, then `/opt`, which is why software stops first (L4).
- **Read back** `/proc/self/mountinfo`: each hierarchy an overlay with
  source `hearth-live`, its layers this delta over the base.
- **The layers' source,** `/run/hearth-activator/layers`, is a clone of
  the top level (`open_tree` with `OPEN_TREE_CLONE`), read-only,
  without the top level's `noexec` and `nosuid` (`mount_setattr`), and
  private, made at the activator's start: a lazy unmount of a shared
  peer of a layer's source propagates and tears the overlay down (as
  measured), and whether overlayfs carries a lower mount's `noexec` to
  the merged files is to verify.
- **The activator stays in the host's mount namespace,** with no
  sandboxing in its unit (hearth/docs/design/domain/activator/README.md, 3.1), or its overlays would land
  in a namespace of its own.

**A person's soft reboot while live** (`systemctl soft-reboot` with no
`/run/nextroot`) keeps the overlay on `/etc`, which systemd may treat as
extrinsic, and unmounts `/usr/local`'s and `/opt`'s (to verify). The
next start finds the base still running, the boot id unchanged, the soft
reboot count higher and the mount table disagreeing with the live
record, and runs `reboot -ff`: the default, or the trial a one-shot
names, boots natively. hearth's own soft reboot into a `live-blessed`
slot (a `try`, 2) is not this: the new root is that slot's, and the
start removes the live record whose base no longer runs.

## 9. A later live change

While `b` is `live-blessed` (or a live state outlives its slot, below):

- **Prepare `c` into `b`:** allowed, since `b` is neither running nor
  the default; `prepare.md`, 1 clears the one-shot that names `b`'s trial entry.
  The delta keeps serving: it is a subvolume of its own, not `b`.
- **The delta is always against the base:** `delta(a → c)`, built and
  verified as L1 and L2, never stacked on the previous one. The unit
  plan and rules 3 to 9 compare `c` with what runs, the live state's
  files, so only what changed since the last live change acts; rules 1
  and 2 compare `c` with `a`.
- **L5's beneath-swap replaces the previous overlay** atomically.
- **The previous delta is kept until `c` is live-blessed,** and the live
  fall back swaps back to it, not to the bare base (`bless.md`, 2.5). The live bless
  deletes it.
- **After such a fall back no slot holds what runs:** `b` holds `c`,
  fell back, and the previous delta's slot was prepared over. The live
  record keeps naming the previous delta, with no slot, and no one-shot
  is set: the next boot runs `a`, older than what ran, and the next live
  change replaces the delta.
- **A dead-man reboot or a power cut** in any of these states lands on
  `a`, natively: the invariant ("the default boots a blessed slot")
  holding, older than the live state. The facts say so.
