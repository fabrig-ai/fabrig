# Bootstrap and disks

## 1. The command

`hearth bootstrap` runs on a live or rescue system, as root, with the
client running the activator's domain in-process (hearth/docs/design/client/README.md, 3.6). It is
the only way a machine gets its first slot. Its arguments name the disk,
the machine's configuration and a directory of secrets to import;
hearth/docs/design/client/README.md, 3.8 owns their syntax:

```
hearth bootstrap --disk /dev/nvme0n1 --wipe --target x1-prod
    --repository https://git.example.org/ai/fleet.git
    --revision main --every 10m --trial manual --operator 1000
    --secrets /root/x1-prod.secrets
```

The machine's configuration, which bootstrap writes once (2, step 4;
hearth/docs/design/client/README.md, 3.8):

| Field | Standalone | Following an engine [M4] |
|---|---|---|
| mode | `standalone` | `engine`: never both (hearth/docs/design/client/README.md, 3.1) |
| target | the machine's target, such as `x1-prod`; its containers come from that target's spec [M5] (hearth/docs/design/protocol/standalone.md, 7) | the same |
| repository | the IaC repository's URL (hearth/docs/design/protocol/standalone.md, 1) | the same |
| desired revision | a commit, or a branch to follow with its interval, and `trial: auto` or `manual` (hearth/docs/design/protocol/standalone.md, 1) | none: the engine orders revisions (hearth/docs/design/protocol/wire.md, 4.4) |
| operators | the uids that may run every socket command (hearth/docs/design/client/README.md, 3.4) | the same |
| mutable | `refuse`, or `accept` with `--accept-mutable`: whether the activator places mutable targets (hearth/docs/design/domain/development.md, 4.3); in its machine record | `refuse`: the run side never deploys one |
| engine | none | its address, what to trust it by, and the machine's credential if the identity chosen is one (hearth/docs/design/protocol/wire.md, 4.1 and 4.3) |

The target and `mutable` go into the activator's machine record; the
rest is the steward's, kept in its state (hearth/docs/design/domain/store/README.md, 6.3), from which the
client changes only the desired revision and its trial mode
(hearth/docs/design/protocol/standalone.md, 1).

**Before it touches the disk,** bootstrap renders the target: it makes
the `hearth` user (3) and spawns `/proc/self/exe render` as that user,
in-process render with no store (hearth/docs/design/client/README.md, 3.5), given the repository's
read credential (`--repo-credential`) through a `0600` file it owns. Then
it refuses to start, naming every reason at once, unless:

- it runs as root, on a kernel with btrfs and unprivileged user
  namespaces (Debian's default; Ubuntu's AppArmor restriction must be
  lifted first);
- the firmware is the rendered placement's `firmware`: on EFI, efivarfs
  writable and `SecureBoot` off (`boot.md`, 5.5);
- its own binary, `/proc/self/exe`, has the digest the `hearth` task's
  download pins in the spec (hearth/docs/design/protocol/format/tasks.md, 12): the slot it makes runs
  that release, which must read the records bootstrap writes
  (hearth/docs/design/domain/activator/README.md, 6.2);
- the disk is a whole block device, none of it mounted, not the live
  system's own, and blank unless `--wipe` is given (provider C's volumes come
  with an image installed, and `x1-prod`'s holds os);
- the tools of 2 are present, `ssh-keygen` (`openssh-client`) among
  them, and what the builder needs: `newuidmap` and `newgidmap`
  (Debian's `uidmap`), `mmdebstrap` and `git` (hearth/docs/design/domain/store/README.md, section 7).

## 2. The steps

Each step is an effect of `hearth-domain-disk`, `hearth-domain-slots` or
`hearth-domain-boot`, keyed so that running bootstrap again after a
failure finds what the first run made, up to the disk's wipe.

1. **Wipe**, with `--wipe` only: `wipefs --all /dev/nvme0n1`.
2. **Partition:**

   ```
   EFI   parted --script /dev/nvme0n1 mklabel gpt mkpart ESP fat32 1MiB 513MiB set 1 esp on mkpart hearth btrfs 513MiB 100%
   BIOS  parted --script /dev/vda mklabel msdos mkpart primary fat16 1MiB 65MiB set 1 boot on mkpart primary btrfs 65MiB 100%
   ```

   then `partprobe /dev/nvme0n1` and `udevadm settle`, and both
   partitions must exist under `/sys/class/block/`.
3. **Format,** with identifiers hearth draws from the kernel's random
   source and passes in, so it knows them without reading them back:

   ```
   mkfs.vfat -F 32 -n ESP -i 3e5a91c4 /dev/nvme0n1p1          EFI
   mkfs.vfat -n HEARTH -i 3e5a91c4 /dev/vda1                   BIOS [M3b]
   mkfs.btrfs --force --label hearth --uuid 0f3c9a6e-… /dev/nvme0n1p2
   ```

4. **Make the top level:** mount it at `/run/hearth-activator/top`; make
   `@hearth.state` (`0:0 0700`) and `@hearth.store` (`hearth`, `0700`);
   write the machine record (target, firmware, filesystem UUID, boot
   partition's volume id, entry numbers) into the state, and the
   machine's configuration (1) into the steward's state, owned by
   `hearth`, with the repository's read credential under the store's
   `credentials/` (hearth/docs/design/domain/render/README.md, 2.2), never in the secret store.
5. **Import the secrets** from `--secrets` (hearth/docs/design/domain/activator/operations.md, 8). A
   secret the spec names and the store lacks fails the place in step 7,
   naming every missing one. Identity is never imported: step 7 makes
   it.
6. **Build:** bootstrap enters a transient scope with `Delegate=yes`
   (`systemd-run --scope --property=Delegate=yes`) and lays out its
   cgroups as the daemon does (hearth/docs/design/domain/activator/README.md, 3.1; hearth/docs/design/domain/steward.md,
   section 7); its activator's domain spawns the steward from
   `/proc/self/exe` (the live system has no `/usr/local/bin/hearth`) over a
   link, as on a running machine (hearth/docs/design/domain/activator/README.md, 3.2), with the store
   mounted at `/var/lib/hearth/store`. The steward spawns a builder for
   its jobs (hearth/docs/design/domain/steward.md, section 3), which renders the target at its
   desired revision and builds its output; the steward then asks to
   place it, as converge does on any machine (hearth/docs/design/protocol/standalone.md);
   under an engine [M4], the revision given builds this first slot
   only.
7. **Place into slot `a`:** prepare, from snapshot to verify
   (hearth/docs/design/domain/slots/prepare.md), including the boot files and both entries
   (EFI) or the slot's fragment (BIOS). Prepare first makes every
   identity item the spec declares and the store lacks, and the
   machine-id (hearth/docs/design/domain/activator/operations.md, 7).
8. **Install the boot loader,** BIOS only [M3b]:

   ```
   grub-install --target=i386-pc --boot-directory=/run/hearth-activator/boot --locales= --fonts= /dev/vda
   ```

   then write `grub.cfg` (`boot.md`, 2.3) and both environment files with
   `default=<target>.a`, and `previous`, `tries` and `next_entry`
   empty.
9. **Make slot `a` the default:** `efibootmgr --bootorder 4800,…` on
   EFI; `default` is already `a` on BIOS. There is nothing to fall back
   to, so there is no trial: bootstrap blesses `a` with the one
   transition that needs no running slot (hearth/docs/design/domain/slots/README.md, section 4).
10. **Report and stop:** print the machine-id and each identity item's
    public half for the repository's exports (hearth/docs/design/protocol/format/README.md, section 6);
    stop the steward, which stops its builder; unmount everything. The
    operator reboots.

The import comes before the build and the identity is made by the
place, though the task listed both last: prepare writes them into the
slot, so they must exist when it runs.

## 3. The build on a live system

- **The steward and its builder run as `hearth`, unprivileged, even
  here.** A live system has no `hearth` user, so bootstrap makes one,
  with the fixed uid, gid and subordinate range that every output also
  declares (hearth/docs/design/protocol/format/tasks.md, 12; hearth/docs/design/domain/store/README.md, 7.2):

  ```
  useradd --system --uid 59999 --user-group --home-dir /nonexistent --shell /usr/sbin/nologin hearth
  usermod --add-subuids 262144-327679 --add-subgids 262144-327679 hearth
  ```

  System accounts get no subordinate range by default, so it is added
  explicitly, aligned to 64 Ki; a live system's `/etc` is writable.
- **The uid must be the outputs' own:** every slot mounts the same
  `@hearth.store`, owned by it, so a uid that differed between outputs
  would lose the store at the first slot switch.
- **The build writes to the target disk,** under `@hearth.store`; its
  layers are the first slot's store when it boots.
- **No program from the output runs as root:** the tools are the live
  system's, the system running the activator (hearth/docs/design/domain/activator/README.md, 2.1).

## 4. The first boot

The firmware or GRUB boots `a`. Its activator finds `a` running and
default, no trial marker and no journal; its steward finds the desired
revision built and placed, and converges from there.

**A host's containers are not bootstrapped [M5].** Bootstrap places the
host's own target only. From its first boot, the host's hearth builds
and prepares each container's first slot as any change (hearth/docs/design/protocol/standalone.md, 7),
and tries it with nothing to fall back to (hearth/docs/design/domain/slots/containers.md, 3). A
container's secrets are imported for its own target (hearth/docs/design/client/README.md, 3.6),
before or after: a missing secret only makes its prepare wait
(hearth/docs/design/protocol/standalone.md, 4).

## 5. Disks as an effect [M6]

The lab's throwaway machines need disks made on order, not by a person
at a live system: the same partitioning and formatting as an effect of
the activator (hearth/docs/design/domain/activator/operations.md, 10), on a disk named by its stable id
(`/dev/disk/by-id/…`), conditional on the disk being blank, and keyed
by the identifiers the order gives. The pool host's activator makes the
disk and bootstraps the machine onto it.
