# Real builds

## 1. What they show

What only real tools on real btrfs show: mmdebstrap as the root of a
sandbox's namespace, chrooted into its layer; real maintainer scripts;
the target's systemd tools run offline inside a layer; the tree helper
snapshotting, freezing, walking and emptying layers the builder cannot
touch; layers removed with and without `user_subvol_rm_allowed`; the
real costs of each step.

## 2. The local mirror

- **The pin,** in `testing/mirror`: a snapshot timestamp (the canonical
  one snapshot.debian.org gives), the suite, components and
  architecture, and the requests of the test repository's targets:
  `test-min` (the `apt` variant with no package of its own: the test
  repository's `tasks/hearth/task.json` declares none, hearth/docs/design/protocol/format/tasks.md,
  12), and `test-boot` for the boot
  test and the VM tier (a kernel, tiny-initramfs, systemd, systemd-container,
  openssh-server, incus-agent, and the tools hearth runs: mmdebstrap,
  uidmap, efibootmgr, git). The
  mirror digest is the SHA-256 of the pinned `InRelease`, kept in the
  pin: it names every index the mirror serves. A bump is a commit, as a
  snapshot bump in IaC is, and carries the keys and closures the tests
  hold with it.
- **The extras:** a few test packages built from sources in the
  repository with `dpkg-deb`, as a regular user: a postinst that fails,
  a conffile, a user made by a postinst, a setuid file, a capability.
- **At M1, indexes only:** a snapshot of forky, `main`, amd64, without
  the updates and security pockets; the suite's `InRelease` and its
  `Packages` index, whole and signed, so apt verifies them as it would
  snapshot.debian.org's, with the host's keys (hearth/docs/design/domain/render/spec.md, 2.2), and any
  request resolves. `test-min` and
  `test-boot` are targets of the test repository (`testing/repos`) with
  the requests above, and the resolve scenario compares their closures
  (`README.md`, 3.2). The pin's first timestamp is the canonical one
  snapshot.debian.org gives for the time the pin is made; a bump is a
  commit.
- **Fetched once, online,** by a script: indexes only at M1, then the
  closure's .debs through hearth's own fetch step [M2], into a cache
  directory named by the mirror digest, outside the repository, each
  file checked as any fetch is (`InRelease`, then SHA-256). A run whose
  digest is present fetches nothing, as mmdebstrap's `make_mirror.sh` is
  a no-op when nothing changed.
- **Served on loopback as an HTTP proxy,** by the harness, at a port it
  chooses: it answers each request for
  `http://snapshot.debian.org/archive/…` with the file the mirror holds
  at that path (`by-hash` paths included); a timestamp it does not hold
  as the latest earlier one it does (`fake-machine.md`, 6), and one before them all, or
  any other request, with 404, so no test reaches beyond loopback. The builder under test takes
  it as its proxy (hearth/docs/design/domain/render/spec.md, 2.2's `Acquire::http::Proxy`; the fetch's
  apt configuration [M2], hearth/docs/design/domain/build/packages.md, 1), as a builder takes an
  apt-cacher-ng: the proxy is the builder's configuration, made by the
  test (hearth/docs/design/client/README.md, 3.5), and never in a key (hearth/docs/design/domain/render/spec.md, 2.1). So
  the test targets' sources are any target's at the pinned timestamp,
  with hearth/docs/design/protocol/format/tasks.md, 3's fixed URLs, and their keys are those any
  builder computes for them. The build then makes its own local
  repository from its cache, as on any machine (hearth/docs/design/domain/build/packages.md, 3).

## 3. The base layer, once per mirror digest

- **The real tier's store is a hearth store** that lives across runs in
  the test directory.
- **The base layer is keyed** by the resolution's digest
  (hearth/docs/design/domain/store/README.md, 3.3), so it is built once per mirror digest: the first run
  builds it, later ones find it. An essential bootstrap from a local
  mirror takes about ten seconds; mmdebstrap's README quotes 9.5 on a
  tmpfs.
- **A setup step** builds or finds the shared layers before the tests
  start (nextest's setup scripts), pins them (hearth/docs/design/domain/store/README.md, section 5), and
  so tests in parallel never race for them. Each test's own layers are
  collected after it.
- **A second run builds nothing,** and its cost facts say so (`README.md`, 4.3): the
  digest rule, tested on hearth itself.

## 4. Prerequisites

- **The builder's own** (hearth/docs/design/domain/store/README.md, section 7): Linux with unprivileged
  user namespaces, Debian's default; uidmap's `newuidmap` and
  `newgidmap`; cgroup v2 with a subtree delegated to the test user; and
  skein's contained process trees, which sandboxes wait for
  (hearth/docs/design/domain/build/sandboxes.md, 8).
- **On a developer's machine** the profile runs under `systemd-run --user
  --scope -p Delegate=yes`, which delegates that subtree for the
  builder's cgroups; a runner's service needs `Delegate=yes` as well.
- **One subordinate uid range and one gid range for the test user,** as
  hearth/docs/design/domain/store/README.md (`README.md`, 4.2) demands of the `hearth` user: 65536 ids, one entry
  each, overlapping no other. A builder alone accepts an unaligned base,
  since only the activator's shift needs alignment, so the range
  `useradd` gives at 100000 serves as it is.
- **A btrfs filesystem** mounted `nosuid,nodev,noatime` and
  `user_subvol_rm_allowed`, made once by an administrator, in which the
  test user makes the store as a subvolume of its own (hearth/docs/design/domain/store/README.md, 7.1).
  The option covers the whole filesystem, so a dedicated one (a
  partition, or a loop file) is better than `/home`.
- **The mirror,** fetched (2).
- **No prerequisite, no run:** the profile refuses to start, naming what
  is missing. It never passes by skipping, as os's gated tests did.

## 5. Each test pays only for its own step

- **A config change:** a snapshot of the shared software layer, then the
  config step: a second or less.
- **A software change:** a snapshot of the packages layer, then the
  software step.
- **The tree helper's walk** of the layer just built against its
  parent's record: only changed files are hashed.
- **Removing layers:** the tree helper's `delete` where the mount option
  is set, and thaw, empty and `rmdir` where it is not (hearth/docs/design/domain/store/README.md, 5.3 and
  5.4): both paths run.
- **A packages layer of its own** only where a test is about packages:
  the extras on the base; and `test-boot`'s, whose `/var/lib/machines`
  must be a directory, not a nested subvolume (`worlds.md`, 5).
- **Both ways of making a packages layer** (hearth/docs/design/domain/build/README.md, 4.2): a closure
  that adds an extras package and drops another is reconciled from the
  base, and built from scratch when the test forces it. Each must pass
  the installed-set check, and the test records how the two trees
  differ, as the recordings the lab's comparison starts from.

Every test asserts its own cost facts: which steps ran, which layers
were reused, what was fetched.

## 6. The boot test

M2 ends with a machine booting what it built: `test-boot`'s root, its
kernel and its initramfs, from a btrfs subvolume, with no image built,
kept or shipped. The rest of booting, the ESP, the entries, fstab, the
shift and the activator, stays M3's (`vm.md`).

- **What it boots:** `test-boot`, built as any real build is, through
  hearth's store. Its packages layer holds the kernel and the initramfs
  the kernel's hook made with the packages part's `tiny-initramfs.conf`
  (hearth/docs/design/domain/build/packages.md, 5); its boot digests (hearth/docs/design/domain/build/packages.md, 8) name the
  version. The test repository's `boot` task carries the `incus`
  profile's modules (hearth/docs/design/domain/machine/boot.md, 4.2) and `console=ttyS0,115200n8`
  (hearth/docs/design/protocol/format/tasks.md, 4), and its tree the signal's units, below.
- **The disk:** a sparse file, the VM's only disk, holding a fresh
  btrfs filesystem, its uuid fixed by the test, made by `mkfs.btrfs
  --uuid <uuid> --rootdir <dir> --subvol rw:@test-boot.a`, where
  `<dir>/@test-boot.a` is the config layer, bind-mounted. Its top level
  is hearth/docs/design/domain/machine/README.md's (`fake-machine.md`, 1) with one slot, and nothing else: no ESP,
  no `/var` subvolume.
- **Ids by view, not by shift:** `mkfs.btrfs` runs unprivileged on the
  file, in a user and mount namespace mapping the test user's
  subordinate range to 0 to 65535, as the builder's sandboxes see a
  layer. It copies owners, capabilities and ACLs as that namespace reads
  them, which is what the activator's shift writes on disk
  (hearth/docs/design/domain/slots/prepare.md, 2), with neither root nor the shift. The same
  namespace copies `boot/vmlinuz-<version>` and
  `boot/initrd.img-<version>` out for QEMU. The test deletes all three
  files after it.
- **The boot:** QEMU's q35 with OVMF, Secure Boot off (hearth/docs/design/domain/machine/boot.md,
  5.5), and `-kernel`, `-initrd` and `-append`: OVMF starts the kernel
  by its EFI stub, as an M3 entry does from the ESP. The line is
  hearth/docs/design/domain/machine/README.md's (1) less `initrd=`, which QEMU passes itself:
  `root=UUID=<uuid> rootflags=subvol=@test-boot.a ro panic=10`, the
  `boot` part, then `systemd.unit=hearth-test-booted.target`. The disk is
  on virtio-scsi (`virtio-scsi-pci`, `scsi-hd`), as Incus attaches a
  VM's root disk, so the run verifies the `incus` module list. With
  `-no-reboot`, a panic ends QEMU. The serial console is logged.
- **The signal:** `hearth-test-booted.target`, in the test target's
  tree, requires `multi-user.target`, and a oneshot service after it
  writes `hearth-test-booted <subvol>` to `/dev/console`, the root's
  subvolume as `findmnt` reads it, then powers off
  (`SuccessAction=poweroff`). It runs only when the line asks for it, so
  the VM tier's boots of `test-boot` are unchanged. The test passes when
  that line, naming `@test-boot.a`, reaches the serial log and QEMU exits
  0 within the ceiling (`README.md`, 4.2); anything else fails, with the log kept.
- **KVM where `/dev/kvm` opens, TCG otherwise** (`-accel kvm -accel
  tcg`): a developer's machine has KVM, a shared runner may not. TCG
  takes minutes, which is why the test boots once, not per scenario.
- **Prerequisites:** 4's and the mirror, and `qemu-system-x86` and
  `ovmf`. No prerequisite, no run, as 4 says.
- **What it is not:** an image, a test of the activator, the shift,
  entries, trials or fstab, or a replacement for the VM tier, which
  boots from the disk's own ESP and runs as root inside.
