# The VM tier

## 1. The VMs

- **Incus VMs** (q35), each with its own disk: a block volume holding a
  partition table, an ESP and a btrfs filesystem
  (hearth/docs/design/domain/machine/README.md, section 3). Not the old lab's virtiofs boot: firmware,
  ESP and disk are what the MVP's machine has.
- **OVMF, with `security.secureboot=false`.** Incus turns Secure Boot
  on by default, and a Debian kernel cannot boot from a stub entry under
  it (hearth/docs/design/domain/machine/boot.md, section 5).
- **The iTCO watchdog** q35 provides; QEMU's default action for it is a
  reset (hearth/docs/design/domain/slots/trial.md, 5).
- **A serial console,** logged by the harness.
- **BIOS [M3b]:** `security.csm=true`, which runs SeaBIOS and requires
  Secure Boot off, on an msdos disk laid out as hearth/docs/design/domain/machine/README.md (`README.md`, 3.1) says.
- **Containers [M5] run inside a VM,** which is their host target: its
  hearth runs the container targets with systemd-nspawn, as `carbon`,
  `east` and `south` do (hearth/docs/design/domain/slots/containers.md). A VM needs no nesting
  settings for that, unlike an Incus container, and the real tier
  cannot run them: nspawn needs root.

## 2. Driving them

- **The harness runs as a regular user** who may use Incus. Root exists
  only inside the VMs.
- **It serves the mirror and the hearth binary under test** on the Incus
  bridge. The test repository's hearth task installs that binary as a
  software download pinned by digest (hearth/docs/design/protocol/format/README.md, section 8), which the
  harness writes into the revision it commits for the run.
- **It drives a VM** with `incus exec` (the test targets install
  `incus-agent`), runs hearth's client there as an operator would, and
  reads the serial log for what comes before userspace: the firmware, a
  panic.
- **It checks hearth's facts** against what it reads independently:
  efivarfs's variables, the environment pair [M3b], the root subvolume,
  the slot records; and records `efibootmgr -v` for conformance (`README.md`, 3.2).
- **A power cut** is `incus stop --force`, which kills QEMU.

## 3. Scenarios

| Scenario | What happens | What must hold |
|---|---|---|
| bootstrap | from a live system on a second disk, the client, as root, bootstraps the empty disk (hearth/docs/design/domain/machine/bootstrap.md); the VM boots it | slot a is blessed and the default; two entries per slot, normal and trial; only a's normal entry in `BootOrder` |
| prepare, soft reboot, bless | a change that cannot be live, such as a drop-in in `/etc/sysctl.d/` (hearth/docs/design/protocol/format/tasks.md, 14): b prepared, a soft reboot into it, the minimal bless (hearth/docs/design/domain/slots/bless.md, section 1) | b running, its normal entry first in `BootOrder`; no trial left |
| a packages change that keeps the kernel | a new version of an extras package: b's packages layer rebuilt, its initramfs rebuilt with the same inputs | hearth/docs/design/domain/slots/trial.md 1's rule holds, so a soft reboot, then the bless |
| a kernel change | a mirror bump that changes the kernel: b prepared, `BootNext` to b's trial entry, a reboot, the bless | before the bless, `BootNext` gone and `BootCurrent` b's trial entry; then b the default |
| fallback after a panic | the revision adds the `test-panic` task (`worlds.md`, 2) | the trial panics; `panic=N` reboots into a, which records that b fell back |
| fallback after a hang | `test-hang` blocks the trial's target | `JobTimeoutAction=` or the dead-man unit reboots with `-ff`; fell back |
| a mount fails during a trial | `test-mount-fails` adds a mount that fails, and the trial lands in emergency mode | the dead-man, armed from `/run` or `hearth.trial=` and kept by `IgnoreOnIsolate=yes`, still reboots with `-ff`; fell back |
| a failed unit | `test-unit-fails` fails a unit | the boot is `degraded`, and the slot's steward asks to fall back at once (hearth/docs/design/domain/slots/bless.md, section 2), with no dead-man; fell back |
| a failed unit, allowed | the same unit, named in `bless.ignore_failed` (hearth/docs/design/domain/slots/bless.md, 1.1) | the minimal bless passes; b blessed |
| a power cut during prepare | a kill at a drawn moment of prepare | a boots; the half-prepared slot is redone or discarded (hearth/docs/design/domain/slots/crashes.md) |
| a power cut in trial and bless | kills just after `BootNext` is set, and between the steps of the bless: journal, `blessed`, marker removed, default moved, dead-man stopped | the default boots a blessed slot |
| BIOS [M3b] | the bootstrap, a one-shot trial, a fallback after a panic, and a cut while the activator writes the environment pair, on SeaBIOS | `next_entry` consumed in both files before the kernel; the default boots a blessed slot |
| BIOS, a blessed default that panics [M3b] | after a bless, the harness arms `test-panic` through a flag in a data entry, so the default panics at every boot | GRUB's boot counter answers (hearth/docs/design/domain/machine/boot.md, section 2), and the machine reaches the other blessed slot without a person |

The steward and its builder, on the same VMs (hearth/docs/design/domain/steward.md, section 5):

| Scenario | What happens | What must hold |
|---|---|---|
| a builder killed for memory | during a packages step, the harness, root in the VM, writes a `memory.max` on `steward/builder/` below the step's peak | the kernel kills within `steward/builder/` only; a step killed fails as that step, the builder killed loses its job, tried once more by a new builder (hearth/docs/design/domain/steward.md, 5.1 and 5.3); `hearth status` answers throughout, and the loss is a fact |
| a builder killed mid-pass | `kill -9` of the builder during a build | the job lost and tried again by a new builder, whose startup scan settles the staging layer (hearth/docs/design/domain/store/README.md, 4.4); converged; the steward never restarted |

Data by class, on the same VMs (hearth/docs/design/protocol/format/tasks.md, 7; hearth/docs/design/domain/machine/README.md, section 3;
hearth/docs/design/domain/operating.md, section 7):

| Scenario | What happens | What must hold |
|---|---|---|
| binds after their class | a target with `journal` and `iwd` in `core` and `home-alice` in `data` | `/srv/core` and `/srv/data` mounted from `@<target>.core` and `@<target>.data`; each bind up after its class's mount and after `/var`; journald writes `/srv/core/journal/<machine-id>` |
| seeding | an entry whose path the output fills, `/var/spool/postfix` | its directory made once and seeded by reflink, owners and modes as the output had them; the next prepare leaves it alone; the slot's own copy hidden under the bind |
| a class that will not mount | the harness renames `@<target>.data` before a boot | the class's mount and every bind of it fail, the boot isolates `emergency.target`, and no service starts on the slot's copy of a path |
| a snapshot across two services [M5] | two services in `data` append numbered records to their files, without pause, while snapshots are taken | each snapshot holds both files as of one instant: no record in one past what the other had at that instant, beyond what was in flight |
| retention [M5] | a day of hourly snapshots, with time sped up | each bucket's newest kept, every other snapshot pruned; `cache` never snapshotted |
| restoring one entry [M5] | `hearth restore mail --from <seq>` | after the reboot, `/var/mail` holds the snapshot's files and `/var/spool/postfix` today's; `mail.before.<seq>` kept beside it |
| restoring a class [M5] | `hearth restore --class data --from <seq>` | after the reboot, every entry of the class as of the snapshot; `@<target>.data.before.<seq>` kept |
| a power cut in a restore [M5] | `incus stop --force` between the two renames | the start finishes or undoes the swap from its record, then reboots |
| reading a snapshot [M5] | `hearth shell --snapshot data@<seq>` | the snapshot's files, read-only, as `nobody` with `CAP_DAC_READ_SEARCH` |

Live changes, on the same VMs (hearth/docs/design/domain/slots/trial.md, 6 to 9):

| Scenario | What happens | What must hold |
|---|---|---|
| a config change applied live, with a reload | a new line in the `ssh` task's drop-in, its `live` entry `reload` on `ssh.service` | no reboot; `/etc` an overlay with source `hearth-live`; `ssh` reloaded, its main process the same; `b` live-blessed; `BootNext` `b`'s trial entry and `BootOrder` unchanged; drift's `slot.live.*` clean |
| an in-house binary under `/opt`, with a restart | a `test-service` task whose download under `/opt/test-service` is a new release, its `live` entry `restart` | the unit stopped before the swap and started after; the new binary answers with its version; the old one never ran with the new files |
| an added file and a deleted file | the revision adds a file under `/etc` and deletes one under `/opt`, both covered | the merged view equals `b`'s, field by field; the whiteout hides the deleted file; L2's walk and `slot.live.tree` pass |
| a failed reload | an `sshd_config` drop-in that `sshd -t` rejects | the reload job fails while the unit stays active; the live fall back: files back, `ssh` reloaded again, `fell back (live)`; no reboot; `b`'s prepare key refused |
| a crash at each live phase | `kill -9` of the activator at L1, L2, L3, L4, L5, L6, L7 and L8 in turn | before arming: `b` `prepared`, the delta deleted; from `armed` to `running`: `reboot -ff` into `a`, then `fell back`; after the live bless: `live-blessed` kept |
| a power cut at each live phase | `incus stop --force` at the same points, and after the live bless | `a` boots natively and `b` fell back, or `prepared` if it was not armed; after the live bless, `b`'s trial boots |
| a person's soft reboot while live | `systemctl soft-reboot` with no next root, after a live bless | the start finds the overlays disagreeing with the live record and reboots with `-ff`; recorded: which overlays survived the soft reboot |
| the next native boot as the trial | after a live bless, the harness reboots the VM | `BootCurrent` `b`'s trial entry; `b` blessed and the default; the delta deleted; no overlay |
| a later live change, and its fall back | `c` while `b` is live-blessed, then `c`'s reload fails | `c` prepared into `b`; its delta against `a`; the fall back swaps back to `b`'s delta; the next boot runs `a` |
| a sandboxed service | a unit with `InaccessiblePaths=` beneath `/etc`, unchanged by the change | restarted by hearth/docs/design/domain/slots/trial.md, 7's rule 11, and seeing its sandbox's view after |
| not live | a change to `/etc/sysctl.d/`, a new package, a renumbered user | an ordinary trial; the `activation` fact names the rule that failed |

The VM tier also settles what hearth/docs/design/domain/slots/README.md, section 7 lists as to verify
for live changes, on trixie's systemd 257 and forky's 262: real ids,
capabilities and ACLs through the overlay; `MOVE_MOUNT_BENEATH` from
PID 1's shared root, with real sandboxed services; `daemon-reload` and
the dynamic `etc.mount`, `usr-local.mount` and `opt.mount` units around
a swap; overlays across soft reboots; the base's dead-man started after
boot; `BootNext` kept over days and consumed by a power cut; timing on
a real tree; Debian's
specifics (AppArmor, dbus's `ReloadConfig`, glibc rereading
`nsswitch.conf`, `hosts` and `resolv.conf`); and whether a lower mount's
`noexec` reaches the merged files.

Containers [M5], in a host VM:

| Scenario | What happens | What must hold |
|---|---|---|
| a container's slot | a container target placed on the VM's host target: its slot prepared on the host, then its nspawn unit restarted on it (hearth/docs/design/domain/slots/containers.md, 1 and 2) | the unit stably active, the slot blessed as the container's default in the host's state |
| a container that fails | `test-unit-fails` inside the container | its unit stays active, but the host reads a failed unit through `systemctl --machine=`, asks to fall back, and the host's activator restarts the container on its old slot (hearth/docs/design/domain/slots/containers.md, 3); with the unit on `bless.ignore_failed`, it is blessed |
| a container that never comes up | a slot whose nspawn unit fails | the trial fails at the first `failed`; the container runs its old slot |
| a host reboot during a container's trial | `incus stop --force` mid-trial | the links in `/run` are gone; the host's activator restarts the container on its default, and records the trial `fell back` |
| reading a container from its host | the facts and the criterion of a running container | recorded: whether `systemctl --machine=` needs a bus inside (hearth/docs/design/domain/slots/README.md, section 7); running reads as the subvolume of the container's root |
| a container's checks | a check of a unit and a probe, run by the host's hearth (hearth/docs/design/domain/operating.md, 2.9) | reported as the host's facts, naming the container's target |
| a pool host | one output, several containers (hearth/docs/design/domain/slots/containers.md, 4's lean) | one build and one shift; a snapshot per container; each container's files owned as nspawn maps them |
| a host and its container in one revision | both specs change | the container's trial first, the host's last, since the host's may reboot the machine (hearth/docs/design/protocol/standalone.md, 7) |
| a container's parameters change | a bind added to its placement | a new slot of the same output, its settings with the bind, tried by a restart (hearth/docs/design/domain/slots/containers.md, 1) |
| a container that fails on its host's trial | the host's trial boots and the container's unit fails | the host is blessed all the same; the container's facts report it (hearth/docs/design/domain/slots/bless.md, 1.1) |
| a container's journal | entries written inside the container, read by the host's steward while it runs and after it stops | read from `/var/log/journal/<its machine-id>` in the host's journal entry, in its `core` class; recorded: whether journald inside writes there under its id range (hearth/docs/design/domain/operating.md, section 5) |
| a container removed | the revision drops the container's target | its unit stopped, its slots deleted and their outputs unpinned; its class subvolumes reported orphaned (hearth/docs/design/domain/slots/containers.md, 7) |
| a container's change applied live | a config change with a `reload` entry, in a container target | the overlays in the container's mount namespace, none in the host's; the unit reloaded through `systemctl --machine=`; the host's state records the container's next start as `b`'s trial (hearth/docs/design/domain/slots/containers.md, 8) |
| a container's live fall back | the container's reload fails, then a step of the fall back is made to fail | the files back and the units acted on again; then a restart of the container on its default |
| a container's next start after a live bless | the harness restarts the host VM | the container starts on `b`'s trial, which its host blesses; the delta deleted |

A hang that only the watchdog ends needs a kernel that stops; how to
cause one in a VM is open (`README.md`, section 6).

## 4. Paying once

- **A VM is bootstrapped once** per mirror digest and hearth binary,
  then snapshotted; each scenario starts from a copy. Whether a copy
  carries the VM's variables (`qemu.nvram`) is to verify; if not, a
  stub-only disk has nothing to boot (hearth/docs/design/domain/machine/boot.md, 5.4), and each
  scenario pays for a bootstrap of its own.
- **The builder's store inside the VM** already holds the base layers,
  so a scenario rebuilds only what its change touched.

## 5. Where it runs

- **On a host with KVM and Incus:** the development machine, or a
  runner labelled for it. `cargo nextest run --profile vm`, a VM per
  test, a few at a time.
- **Never at the gate.** It runs by choice, and before a hearth release
  reaches a real machine: x1-prod, the MVP's.
- **Until the lab exists.** The lab's throwaway machines then run the
  same scenarios (`hearth/docs/design/draft/archive/factory.md`).

## 6. The development loop

hearth/docs/design/domain/development.md's loop runs here too, on the VM tier's machines and, from
M3b, on a development host with the Incus driver. Its worlds live with
the others: capture and the bundle in the slots', activator's and
standalone worlds [M3], and `hearth-client-dev-world` under `tests/dev`, with
`hearth-client-fake-incus` under `testing/`, for the driver [M3b]
(hearth/docs/design/domain/development.md, section 10).

| Scenario | What happens | What must hold |
|---|---|---|
| a mutable slot [M3] | a VM bootstrapped with `--accept-mutable`, its target placed `mutable`; and one bootstrapped without | the root writable, its read-only flag clear; the second refuses the place, naming the field |
| capture [M3] | by hand: `apt install`, an edit in `/etc`, a program in `/opt`, `systemctl enable`, `useradd` | `hearth diff --running` lists each once, packages without their files; recorded: what systemd writes in `/etc` at a mutable root's first boot |
| the round trip [M3] | the suggestions, unchanged, committed and applied | a soft reboot; the next capture empty |
| a working tree [M3] | `hearth revision bundle`, then `hearth revision set --bundle` on the VM | the commit under `refs/hearth-dev/`; converged; the same tree again changes nothing |
| explored, then a config change [M3] | an edit by hand, then a change with a `reload` entry | not live (hearth/docs/design/domain/slots/trial.md, 7, rule 12): a soft reboot, the edit gone |
| up [M3b] | `hearth dev up` from the base image, timed | a copy in seconds; slot `a` of the target blessed, its identity not the seed's; whether the copy kept `qemu.nvram` recorded |
| several [M3b] | `hearth dev up --count 3` | one build; three identities |
| a cluster [M5] | three members of one base on one bridge | each resolves the others by `<instance>.incus` and reads their exports |
| a container in development [M5] | a host target with a mutable container placed on it | the container explored with `machinectl shell`, captured through its host |
