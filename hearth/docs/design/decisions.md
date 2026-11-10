# hearth's decisions

Questions the brainstorm left open (its section 13), and others met
while writing, with where each is answered.

**Decided with the user's input** (2026-10-09), each with consequences
that last:

- **Ownership across the user namespace.** The builder stays
  unprivileged. Its layers are owned on disk by its subordinate id range,
  aligned to 64 Ki; digests and change sets use the ids seen inside the
  namespace, so they agree across machines. When it prepares a slot, the
  activator snapshots the output and shifts it: owners, setuid and setgid
  bits, file capabilities and ACLs, at about 10 µs an inode, rewriting
  metadata only (hearth/docs/design/domain/slots/prepare.md, 2).
- **Secrets and identity are written into the slot** at prepare, by the
  activator, from a root-only secret store on the machine. Tasks declare
  each secret file's path, mode and owner. Rotating a secret is a new
  slot of the same output (hearth/docs/design/protocol/format/README.md, section 7; hearth/docs/design/domain/slots/prepare.md, 4).
- **A slot is a read-only root and its own `/var`,** copied from the
  output at prepare, so dpkg's database travels with its slot. State that
  outlives a slot lives in data entries, at paths the tasks declare
  (hearth/docs/design/protocol/format/tasks.md; hearth/docs/design/domain/machine/README.md, section 3).
- **Persistent data by class** (hearth/docs/design/protocol/format/tasks.md, 7 and 12;
  hearth/docs/design/domain/machine/README.md, section 3; hearth/docs/design/domain/operating.md, section 7). A target has one
  subvolume per class, `@<target>.core`, `@<target>.data` and
  `@<target>.cache`, made only when an entry uses it and mounted at
  `/srv/core`, `/srv/data` and `/srv/cache`, os's and the bash fleet's
  layout. Each `data` entry is a directory in its class, named by the
  entry, bound at the path its task declares from the fstab the
  activator writes; tasks declare a path and a class, never a subvolume.
  Snapshots and backups are policies of the `hearth` task, by class:
  local snapshots [M5] and sends to the backup peer [M6], never of
  `cache`. A snapshot of a class is atomic across its services; one
  entry is restored by copying its directory out of a snapshot, a class
  by swapping its subvolume. A subvolume per entry is not in the design
  (hearth/docs/design/domain/machine/README.md, section 6).
- **Live changes, with hearth's own overlays** (hearth/docs/design/domain/slots/trial.md, 6 to 9;
  hearth/docs/design/protocol/format/tasks.md, 14), built in M3, a container's in M5. A change confined
  to `/etc`, `/usr/local` and `/opt`, over the same packages layer, with a
  declared consumer for every changed path, is applied with no reboot:
  a delta of the prepared slot, mounted over the running slot, the
  declared units reloaded, restarted or started. The activator mounts
  the overlays itself, with the new mount API and `MOVE_MOUNT_BENEATH`,
  not through `systemd-sysext` and `systemd-confext`, whose refresh is
  not atomic, whose mismatches are silent and whose search paths
  survive a soft reboot. The live bless arms the new slot's trial
  (`BootNext`, or `next_entry`) rather than moving the default, so every
  slot still boots natively on trial before it becomes the default.
- **The development loop** (hearth/docs/design/domain/development.md): up, explore, capture,
  encode in IaC, apply from the working tree, land. A development
  machine is an Incus VM cloned from a base image and driven by `hearth
  dev`, a client outside the daemon [M3b]. `hearth dev` names machines,
  never containers: a machine comes up with the containers placed on
  it, all or a selection, and a container is reached through its
  machine [M5]. An environment may mark its
  placement `mutable`: slots not sealed, roots read-write, taken only by
  a machine bootstrapped with `--accept-mutable` and never by the run
  side. `hearth diff --running` captures what was done by hand and
  drafts task fragments; a working tree becomes a machine's desired
  revision as a commit on no branch, shipped as a git bundle. Builds
  from scratch stay out of the loop: every apply prepares a fresh slot
  from the IaC, so what was not captured is gone at once, and a first
  boot, the upgrade from production and a sealed root are the lab's, at
  QA or at the landing gate (hearth/docs/design/domain/development.md, section 7).
- **Boot follows os, and adds the trial.**
  - EFI: the ESP holds each slot's kernel and initramfs, booted by a
    direct EFI-stub entry per slot. A trial is a `BootNext`; a bless
    moves `BootOrder`.
  - BIOS: GRUB, its core image after the MBR, reading kernels from the
    slot subvolumes. One change from os: a small FAT partition holds
    GRUB's files and its env, which GRUB cannot write on btrfs, so that
    `next_entry` is a one-shot; two env files, so a torn write never
    loses the default (hearth/docs/design/domain/machine/boot.md, sections 1 and 2).
- **`/usr` is the packages', but `/usr/local`** (2026-10-16; hearth/docs/design/protocol/format/tasks.md,
  5; hearth/docs/design/domain/build/README.md, 7.3). Above the packages layer, tasks' files,
  downloads, scripts and hearth itself write `/usr` only under
  `/usr/local`, as os did (its helpers in `/usr/local/bin`, its shared
  tree in `/usr/local/share`): hearth's binary is
  `/usr/local/bin/hearth`, and its unit and its generated sysusers,
  tmpfiles and preset files are in `/etc`. `/usr/lib` is Debian
  packages' alone: should hearth become one, it moves there with the
  packages layer (hearth/docs/design/protocol/format/tasks.md, 12). Render refuses a task's path
  elsewhere in `/usr`, and the build fails a step that writes there. So
  a live change overlays `/etc`, `/usr/local` and `/opt`, never the
  `/usr` every process maps (hearth/docs/design/domain/slots/trial.md, 6).
- **The steward and its builder** (2026-10-14; hearth/docs/design/domain/steward.md). The
  unprivileged side is two processes, by failure rather than privilege:
  the steward, persistent, holds converge, the client's socket, the
  wire, checks and telemetry, and never reads the repository, runs a
  build tool or opens a layer; the builder, spawned by the steward for
  work and gone once idle, does git, render, builds and the store. A
  builder that crashes loses a job, which a second loss fails as
  `crashed`, so a revision that trips a hearth bug is reported by a
  machine that stays in view. The bless asks for one render more, of
  the slot's own revision, so a release that cannot render what built
  it never becomes the default.

**Others:**

- **SHA-256 for every digest:** skein has it, and Debian's archive and
  snapshot.debian.org speak it (hearth/docs/design/domain/store/README.md, section 3).
- **The filesystem is hearth's only state:** subvolumes and the records
  beside them, read back at startup; no kv store (hearth/docs/design/domain/store/README.md, section 6;
  hearth/docs/design/domain/activator/README.md, section 6).
- **The layers are packages, software and config.** The brainstorm's
  etc layer also writes users, `/usr/local`, unit links and more, so it
  is named for what it holds (hearth/docs/design/domain/build/README.md, section 3).
- **A layer is one subvolume, `/var` included;** a slot's `/var` is
  split from it at prepare (hearth/docs/design/domain/store/README.md, section 4; hearth/docs/design/domain/slots/prepare.md, 3).
- **No template is completed at activation.** What os completed there
  (fstab's slot label) the activator writes from typed slot data: fstab,
  the slot record, the machine-id. The privileged side renders no
  template (hearth/docs/design/domain/slots/prepare.md, 5; hearth/docs/design/domain/activator/README.md, section 2). This replaces the
  brainstorm's activation templates (its 7.9).
- **The device leaves the output.** `root=` and fstab name the
  filesystem by UUID, supplied at prepare (hearth/docs/design/domain/machine/boot.md, section 3).
- **A tree helper runs inside the builder's user namespace,** as the
  namespace's root. The builder cannot read every file its layers hold,
  and only the owner of a subvolume's root may snapshot it or make it
  read-only, so the helper snapshots, freezes, walks, hashes, applies,
  thaws and empties layers (hearth/docs/design/domain/store/tree-helper.md, 3 and 8).
- **Each build step runs in a sandbox** of its own namespaces:
  mmdebstrap's chroot does not contain code nobody reviewed. The
  packages step runs in a tool root and mmdebstrap chroots into the
  layer; the steps above it run in the layer itself; none has a network
  (hearth/docs/design/domain/build/sandboxes.md, 1 to 5). Only the fetch, by SHA-256 through
  `apt-helper`, reaches the network, in a cgroup of its own
  (hearth/docs/design/domain/build/packages.md, 1).
- **The builder's subordinate range starts at 262144,** 65536 ids long,
  clear of systemd-nspawn's automatic range and of the fleet's
  containers' ranges (hearth/docs/design/domain/store/README.md, 7.2). The `hearth` user is declared by
  the hearth task, which every target but a container has
  (hearth/docs/design/protocol/format/tasks.md, 12).
- **The kernel is unchanged** when the kernel image, the modules' tree,
  the initramfs's inputs (not its bytes, which are not reproducible) and
  the command line without the slot's own tokens all are; then a soft
  reboot is enough (hearth/docs/design/domain/slots/trial.md, 1; hearth/docs/design/domain/build/packages.md, 8).
- **Render refuses early** whatever would fail at prepare: reserved
  command-line keys, files hearth or the activator own, units named for
  hearth (hearth/docs/design/protocol/format/tasks.md, 11).
- **A host keeps no list of its containers:** a container's placement
  names its host (hearth/docs/design/protocol/format/README.md, 3.5). This replaces the brainstorm's list in
  the host's configuration (its 7.7).
- **The activator starts the steward** and talks to it over a pipe pair;
  it listens to nothing else (hearth/docs/design/domain/activator/README.md, section 3).
- **Which slot runs** is read from the root subvolume's identity
  (`BTRFS_IOC_GET_SUBVOL_INFO` on `/`), cross-checked with the slot
  record inside the root; never from `/proc/cmdline`
  (hearth/docs/design/domain/slots/README.md, section 3).
- **Trials always end:** `panic=N` on a trial's command line, a hardware
  watchdog where there is one, and a dead-man timer in the trial slot
  that forces a reboot unless the slot's hearth blesses it first
  (hearth/docs/design/domain/slots/trial.md); a container's, by its host's activator at its
  window's end (hearth/docs/design/domain/slots/containers.md, 3); a live trial's, by the running slot's
  own dead-man, started from `/run` (hearth/docs/design/domain/slots/trial.md, 8).
- **A minimal bless until checks exist:** the boot reaches its target
  with no failed unit, the slot's own steward is up and talking to its
  activator, and a builder it spawned has rendered the slot's own
  revision (hearth/docs/design/domain/slots/bless.md, section 1; hearth/docs/design/domain/steward.md, section 6).
- **Secure Boot off** for now: a Debian kernel cannot boot from a direct
  stub entry under it (hearth/docs/design/domain/machine/boot.md, section 5).
- **tiny-initramfs is the initramfs,** Debian's package, configured
  explicitly per placement: it is in trixie and in forky, and it has
  been stable for years (hearth/docs/design/domain/machine/boot.md, section 4).
- **No converter from os's format:** configurations are written again.
  os's golden files inform render's tests but are not its oracle
  (hearth/docs/design/domain/render/README.md, section 6).
- **Everything starts from scratch:** configurations are written again
  and machines are installed fresh, `x1-prod` included, by `hearth
  bootstrap` on a wiped disk, with identity generated anew; no part of
  hearth migrates os's data or machines (hearth/docs/design/domain/machine/bootstrap.md).
- **Root tests run on Incus VMs** with their own disk and OVMF, until the
  lab exists (hearth/docs/design/testing/vm.md).
- **Standalone mode follows a desired revision set locally:** a commit,
  or a branch to follow, with `trial: auto|manual`, so a laptop such as
  `x1-prod` reboots into a new slot only when its user chooses
  (hearth/docs/design/protocol/standalone.md, 1).
- **A machine follows an engine or converges alone, never both**
  (hearth/docs/design/client/README.md, 3.1 and 3.8).
- **A bless can be held** by the engine while a canary bakes; the
  `bless` order releases it, and a timer that does not depend on hearth
  ends a hold that outlives its ceiling (hearth/docs/design/domain/slots/bless.md, 1.4;
  hearth/docs/design/domain/activator/operations.md, 13; hearth/docs/design/protocol/wire.md, 4.4).
- **Render failures fall into three classes:** a configuration error
  waits for another commit; an operational failure (git, the
  resolution, a fetch) is retried with backoff; an output that failed
  to build, verify or hold is refused (hearth/docs/design/protocol/standalone.md, 4; hearth/docs/design/domain/render/errors.md, 1).
- **The steward reads the journal itself:** it runs with `hearth`'s
  supplementary groups, `systemd-journal` among them, and no
  capabilities; its builder runs with none (hearth/docs/design/domain/activator/README.md, 3.2;
  hearth/docs/design/domain/steward.md, 2.3).
- **Users a package makes** (`postfix`) are declared by the task that
  installs it, so render knows them and the activator checks they exist
  at prepare (hearth/docs/design/protocol/format/tasks.md, 6; hearth/docs/design/domain/slots/prepare.md, 7).
- **Owners of persistent data have fixed ids.** Every user or group that
  owns a data path, a secret or an identity file has a fixed uid or gid,
  package users included, seeded before any package is configured, so a
  rebuild from scratch never renumbers what a data entry holds
  (hearth/docs/design/protocol/format/tasks.md, 6; hearth/docs/design/domain/build/packages.md, 5).
- **A data entry is seeded once,** from the output's tree at its path,
  when its directory is first made in its class, so its bind never hides
  what packages put there (hearth/docs/design/domain/slots/prepare.md, 3).
- **Trials end even in emergency mode:** the trial units survive
  isolation, and arm from a `/run` marker or a `hearth.trial=` token on
  the command line, never from a mount; each EFI slot has a normal and a
  trial entry (hearth/docs/design/domain/slots/trial.md; hearth/docs/design/domain/machine/boot.md, section 1).
- **A bless marks the slot blessed before it moves the default,** so a
  cut leaves a blessed slot that is not the default, and the steward
  blesses it again (hearth/docs/design/domain/slots/bless.md, section 1; hearth/docs/design/protocol/standalone.md, 2).
- **Refusals are keyed by the prepare key** (the output, the slot parts
  and the generations), never by the output alone; a corrupt layer is
  deleted and built again, not refused; the client can clear a refusal
  (hearth/docs/design/protocol/standalone.md, 4).
- **Prepare never follows a link the output plants:** every slot path is
  resolved beneath the snapshot's root without symlinks, and an output
  of the wrong shape is refused (hearth/docs/design/domain/slots/prepare.md;
  hearth/docs/design/domain/activator/README.md, section 5).
- **The boundary, stated honestly:** the activator keeps secrets from
  the steward and its builder and confines them to closed operations,
  but a compromised builder or steward is root at the next trial, since
  it can place an output whose activator is root. The defence against a bad output is review and the
  lab (hearth/docs/design/domain/activator/README.md, section 2).
- **Identity in M3 is SSH host keys,** made with `ssh-keygen`; WireGuard
  keys wait for in-process generation (hearth/docs/design/protocol/format/README.md, 7.3).
- **The repository's read credential is the builder's own,** imported at
  bootstrap and kept in the store, never in the secret store
  (hearth/docs/design/domain/render/README.md, 2.2).
- **A layer is durable before it is recorded:** `syncfs` on the store
  between its walk and its record, as the last step of the tree helper's
  freeze (hearth/docs/design/domain/store/README.md, section 4.2).
- **Package indices come from the resolution:** apt's verified,
  decompressed `Packages` files are kept by digest and served from the
  local repository; the fetch gets only packages (hearth/docs/design/domain/build/packages.md).
- **The packages layer is reconciled by apt** from the target's previous
  one, and built from scratch instead, never in addition, when there is
  no base, when a reconcile fails, and now and then to bound the history
  a line carries; comparing the two is the lab's (hearth/docs/design/domain/build/README.md, 4.2).
- **Failed units a target expects** are listed as task data
  (`bless.ignore_failed`); the minimal bless counts only the others
  (hearth/docs/design/protocol/format/tasks.md; hearth/docs/design/domain/slots/bless.md, 1.1).
- **A host runs its containers [M5]:** its builders build their outputs,
  its activator prepares their slots on its top level, starts each at
  every start from the default it recorded, and tries a new slot live
  when the change is eligible, inside the container's mount namespace
  (hearth/docs/design/domain/slots/containers.md, 8), or else by a restart, judged from outside with the
  minimal bless read through `systemctl --machine=`; the host's steward
  blesses it. No hearth runs in a container, which has no `hearth`
  task, while whether one should stays open (hearth/docs/design/domain/slots/containers.md;
  hearth/docs/design/protocol/format/tasks.md, 12).
- **A container's `/var` is per slot,** as a host's: the bash fleet's
  persistent `.var` becomes data entries, and a container's identity and
  secrets are its own target's, kept by its host (hearth/docs/design/domain/slots/containers.md, 1).
- **A container's parameters are placement fields [M5]:** its id range,
  binds, capabilities and network, rendered into its `boot` part, so a
  change prepares a new slot of the same output (hearth/docs/design/protocol/format/README.md, 3.5;
  hearth/docs/design/domain/slots/containers.md, 1).
- **A container never fails its host's bless [M5]:** the host's minimal
  criterion leaves out its containers' nspawn units; each container's
  slot has its own trial and facts (hearth/docs/design/domain/slots/bless.md, 1.1).
- **A container's journal is in its host's [M5]:** `LinkJournal=try-host`,
  under the container's own machine-id, read by the host's steward as
  its own (hearth/docs/design/domain/operating.md, section 5).
- **A container removed from its host's spec [M5]** is stopped and its
  slots deleted; its data stays, orphaned, until a person deletes it
  (hearth/docs/design/domain/slots/containers.md, 7).
- **A container's orders are a host's [M5]:** a held bless ends at its
  ceiling in a fall back run by the host's activator, and a unit's
  restart goes through `systemctl --machine=` (hearth/docs/design/domain/slots/bless.md, 1.4;
  hearth/docs/design/domain/activator/operations.md, 9).
- **A build environment is a container with no host** [M6]: its output
  is only a sandbox's root, never an nspawn container
  (hearth/docs/design/domain/build/artifacts.md, 2.3; hearth/docs/design/protocol/format/README.md, 3.4).
