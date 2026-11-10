# Booting a machine

## 1. EFI boot

### 1.1 The ESP

```
ESP: GPT partition 1, 512 MiB, FAT32, flag esp
├── vmlinuz.x1-prod.a        slot a's kernel
├── initrd.img.x1-prod.a     slot a's initramfs
├── vmlinuz.x1-prod.b        slot b's kernel
├── initrd.img.x1-prod.b     slot b's initramfs
└── EFI/…                    other loaders, if any: never touched by hearth
```

- **Flat files, one pair per slot,** as os. The EFI stub loads `initrd=`
  from the volume the kernel came from, so both live on the ESP.
- **Written at prepare, for that slot only** (hearth/docs/design/domain/slots/prepare.md, 6): copied
  from the slot's `boot/vmlinuz-<version>` and
  `boot/initrd.img-<version>`, synced, read back and checked against the
  slot record's digests. The slot's entry left `BootOrder` first (1.4),
  so a cut leaves torn files that nothing boots.

### 1.2 Entries

Each slot has two direct EFI-stub entries on the same files, normal and
trial:

| Field | Normal | Trial |
|---|---|---|
| number | `4800` for `a`, `4801` for `b` (`0x48` is `H`) | `4810` for `a`, `4811` for `b` |
| label | `<target>.<label>`, such as `x1-prod.b` | `<target>.<label>.trial` |
| device | `HD()` of the ESP: partition 1 of the disk the btrfs filesystem is on | the same |
| loader | `\vmlinuz.<target>.<label>` | the same |
| optional data | the whole command line, UCS-2 (section 3) | the same, then `hearth.trial=<target>.<label>` |
| attributes | active | active |
| `BootOrder` | from the slot's bless until its next prepare | never: only `BootNext` names it |

The numbers are chosen at bootstrap, or the next free ones above if the
firmware uses these, and kept in the machine record. Made with
`efibootmgr`, one element per argument, never through a shell:

```
/usr/bin/efibootmgr --quiet --create-only --bootnum 4811
    --disk /dev/nvme0n1 --part 1 --label x1-prod.b.trial
    --loader \vmlinuz.x1-prod.b --unicode
    initrd=\initrd.img.x1-prod.b root=UUID=0f3c9a6e-5b2d-4c1e-9a7f-2d8e6b1c4a90 rootflags=subvol=@x1-prod.b ro panic=10 hearth.trial=x1-prod.b
```

- **Why a trial entry:** the dead-man must be armed before any mount
  (hearth/docs/design/domain/slots/prepare.md, 5), and the command line is what a one-shot boot carries
  that nothing on disk can lose.
- **`--create-only`** makes the entry without adding it to `BootOrder`;
  `--create` would add it.
- **The command line is one argument.** `--unicode` makes the
  positional arguments UCS-2 and joins them with a UCS-2 NUL, and the
  stub stops at the first NUL, so a command line split into arguments
  loses all but its first word.
- **`initrd=` is written with a backslash.** efibootmgr converts `/` in
  `--loader`, not in the command line.
- **The disk is found afresh** at each start, from
  `/sys/fs/btrfs/<fs uuid>/devices/` (hearth/docs/design/domain/activator/README.md, 6.4).
- **Other calls,** each an argument vector of `hearth-protocol-efi`:

  ```
  efibootmgr --quiet --bootnum 4801 --delete-bootnum      remove an entry, before re-creating it
  efibootmgr --quiet --bootnext 4811                      trial: the trial entry (4.4)
  efibootmgr --quiet --delete-bootnext                    clear a one-shot left behind
  efibootmgr --quiet --bootorder 4801,4800,0001,0002      bless (4.4)
  ```

### 1.3 Writing NVRAM

- **An entry is written only when it is missing or differs** in label,
  loader, partition or command line: its files change at every prepare,
  its variable does not. To change it, the activator deletes it, then
  creates it at the same number: `--create-only --bootnum` exits 40 when
  the number exists.
- **Only hearth's own numbers** are ever deleted or created; foreign
  entries keep their place in `BootOrder`, behind hearth's.
- **Writes per rollout:** one `BootOrder` write when prepare takes the
  slot out of it, if it was in; one `BootNext`, which the firmware
  deletes; one `BootOrder` write at bless; the two entries only if the
  command line changed. NVRAM is flash, and OVMF's store is 256 KiB.
- **The activator never writes variables itself;** efibootmgr clears
  efivarfs's immutable flag through libefivar.

### 1.4 Default, trial and bless

- **The default** is the first of the target's normal entries in
  `BootOrder`. With both slots blessed, `BootOrder` reads: the default,
  then the other slot, then every foreign entry in the order it had. A
  firmware that fails to load the default falls through to the other
  blessed slot.
- **A slot about to be prepared leaves `BootOrder` first,** so the
  firmware never falls through to a slot being written.
- **A trial is `BootNext` to the trial entry.** The boot manager deletes
  it before it transfers control to the entry (UEFI 2.10, 3.1.2; edk2's
  `BdsEntry()`), so any later reset boots `BootOrder`, and an entry that
  fails to load falls back to `BootOrder` at once. The trial's boot
  checks `BootCurrent` and `BootNext` (hearth/docs/design/domain/slots/trial.md, 3).
- **A bless rewrites `BootOrder`** with the slot first, and reads it back
  (hearth/docs/design/domain/slots/bless.md, 1.3).

### 1.5 Reading the firmware

- **The activator reads variables, not efibootmgr's text.** os parsed
  three shapes of `efibootmgr -v` output, the UTF-16 options glued to
  the path as hex among them. The variables are sized and stable:

  ```
  /sys/firmware/efi/efivars/<Name>-8be4df61-93ca-11d2-aa0d-00e098032b8c
  BootCurrent, BootNext      4 attribute bytes, then a u16
  BootOrder                  4 attribute bytes, then u16 entries
  Boot4801                   4 attribute bytes, then an EFI_LOAD_OPTION: attributes u32, path list length u16,
                             a NUL-terminated UCS-2 description, device path nodes, optional data (the command line)
  SecureBoot                 4 attribute bytes, then a u8
  ```

- **`hearth-protocol-efi` decodes them** into typed values: the entry's
  label, its `HD()` node (partition number, start, size, GPT GUID), its
  file path and its command line. It also builds efibootmgr's argument
  vectors. The domain sees neither bytes nor text.
- **The ESP's GPT GUID** is read back from the first entry bootstrap
  makes and kept in the machine record, so drift can tell an entry for
  another disk.

## 2. BIOS boot [M3b]

### 2.1 The disk

```
MBR: GRUB's boot.img
post-MBR gap, sectors 1 to 2047: GRUB's core.img
partition 1: 1 MiB to 65 MiB, FAT, flag boot      the boot partition: GRUB, its configuration, its environment
partition 2: 65 MiB to the end, btrfs             the top level (section 3)
```

- **An msdos table with the core image in the post-MBR gap,** as os and
  the bash fleet. A GPT `bios_grub` partition would also work; one
  layout per firmware is enough, and msdos's limit of 2 TiB is far above
  any VPS here.
- **The boot flag is on partition 1,** for BIOSes that boot only a disk
  with an active partition; GRUB's own code ignores it.
- **Partition numbers match EFI's:** the boot partition is 1 and btrfs
  is 2 on both.

### 2.2 The boot partition

```
/grub/i386-pc/…     GRUB's modules and core image source, from grub-install at bootstrap
/grub/grub.cfg      fixed: written once, at bootstrap, from typed data (5.3)
/hearth/env.0       a GRUB environment block, 1024 bytes
/hearth/env.1       the same variables, written second (5.5)
```

- **All of GRUB is on FAT,** its prefix included, so GRUB can write its
  environment there and the btrfs top level holds only subvolumes. os
  kept `grub.cfg` and `grubenv` on btrfs, where GRUB's read-only btrfs
  driver makes `save_env` fail.
- **GRUB is installed once, at bootstrap,** by the live system's
  `grub-install` (`bootstrap.md`, 2). hearth does not reinstall it in M3b: GRUB is
  shared by both slots, so updating it cannot be tried in a slot
  (`README.md`, section 6).

### 2.3 grub.cfg

Written at bootstrap from typed data: the target, both labels, the
btrfs UUID and the serial console the `boot` part names. It never
changes after, so a power cut cannot tear it. The exact script is to
verify on GRUB 2.12, trixie's:

```
# written by hearth at bootstrap (boot.md, 2.3)
insmod part_msdos
insmod fat
insmod btrfs
insmod loadenv
insmod test
serial --speed=115200 --unit=0 --word=8 --parity=no --stop=1
terminal_input serial console
terminal_output serial console

set env=($root)/hearth
set loaded=
if load_env -f ${env}/env.0 default previous tries next_entry; then
    set loaded=env.0
    set other=env.1
elif load_env -f ${env}/env.1 default previous tries next_entry; then
    set loaded=env.1
    set other=env.0
fi
set fallback="${default}"
set hearth_trial=
export hearth_trial
if [ -n "${loaded}" -a -n "${next_entry}" ]; then
    set trial="${next_entry}"
    set next_entry=
    if save_env -f ${env}/${loaded} next_entry; then
        set default="${trial}"
        set hearth_trial="hearth.trial=${trial}"
        save_env -f ${env}/${other} next_entry
    fi
elif [ -n "${loaded}" -a -n "${previous}" ]; then
    if [ "${tries}" = 0 ]; then
        set default="${previous}"
    else
        if [ "${tries}" = 3 ]; then set tries=2
        elif [ "${tries}" = 2 ]; then set tries=1
        else set tries=0
        fi
        if save_env -f ${env}/${loaded} tries; then
            save_env -f ${env}/${other} tries
        fi
    fi
fi
set timeout=3

search --no-floppy --fs-uuid --set=hearth_fs 0f3c9a6e-5b2d-4c1e-9a7f-2d8e6b1c4a90
menuentry 'east-prod.a' --id east-prod.a {
    set root=${hearth_fs}
    source /@east-prod.a/.hearth/grub.cfg
}
menuentry 'east-prod.b' --id east-prod.b {
    set root=${hearth_fs}
    source /@east-prod.b/.hearth/grub.cfg
}
```

- **A trial is taken only once its one-shot is spent.** GRUB clears
  `next_entry` in the file it loaded first, and boots the trial only if
  that save succeeded, then clears it in the other. A failed save boots
  the default instead: a trial is never sticky, so a trial that panics
  cannot loop. On btrfs every save fails, which is why GRUB lives on FAT
  and `grub-reboot` is never used.
- **The trial boots with `hearth.trial=<target>.<label>`,** appended by
  the slot's fragment (2.4), which arms its dead-man (hearth/docs/design/domain/slots/prepare.md, 5).
- **A boot counter falls back to the previous blessed slot,** RAUC's
  pattern: each boot of the default spends one of `tries`, and at 0 GRUB
  boots `previous` (hearth/docs/design/domain/slots/bless.md, 2.4). GRUB's script has no arithmetic,
  hence the chain of comparisons.
- **`fallback` is the default,** so a trial whose kernel fails to load
  returns to the default within the same boot. It covers only entries
  whose loading returns, not a kernel that panics: that is `panic=10`'s
  and the counter's job.
- **The serial lines** are written only when the `boot` part names a
  serial console (`console=ttyS0,115200n8` gives unit 0 at 115200).
  `timeout=3` leaves a person at the provider's console time to pick a
  slot by hand.

### 2.4 The slot's fragment

Each slot carries its own `linux` and `initrd` lines, in
`/.hearth/grub.cfg` inside its root, written at prepare
(hearth/docs/design/domain/slots/prepare.md, 5) and sealed with the slot:

```
# slot east-prod.b, written by hearth's activator at prepare (boot.md, 2.4)
linux /@east-prod.b/boot/vmlinuz-6.12.48+deb13-amd64 root=UUID=0f3c9a6e-5b2d-4c1e-9a7f-2d8e6b1c4a90 rootflags=subvol=@east-prod.b ro panic=10 console=tty0 console=ttyS0,115200n8 ${hearth_trial}
initrd /@east-prod.b/boot/initrd.img-6.12.48+deb13-amd64
```

- **Inside the slot,** so the command line changes with the slot it
  belongs to, a torn write harms only the slot being prepared, and
  `grub.cfg` stays fixed.
- **`${hearth_trial}` is empty** but on a trial's boot (2.3).
- **Paths are resolved by the activator,** to the kernel's own files,
  rather than through Debian's `/vmlinuz` link, so GRUB never follows a
  link on btrfs.
- **GRUB reads paths from the default subvolume,** the top level, which
  is why `/@east-prod.b/` leads every path (`README.md`, 3.1).

### 2.5 The environment pair

| Variable | Meaning | Written by |
|---|---|---|
| `default` | the default slot's entry, such as `east-prod.a` | the activator: bootstrap, bless, re-bless |
| `previous` | the previous blessed slot, or empty | the activator: set at bless; cleared at re-bless and before that slot is prepared again |
| `tries` | boots of the default left before `previous`: 3 to 0 | the activator: 3 at bless and at every start that meets the minimal criterion; GRUB counts it down |
| `next_entry` | the slot on a one-shot trial, or empty | the activator at trial; GRUB clears it at boot |

- **Two files hold the same variables.** The activator writes `env.0`,
  then `env.1`, each with `grub-editenv` (`hearth-protocol-grub`):

  ```
  /usr/bin/grub-editenv /run/hearth-activator/boot/hearth/env.0 set next_entry=east-prod.b
  /usr/bin/grub-editenv /run/hearth-activator/boot/hearth/env.1 set next_entry=east-prod.b
  ```

  GRUB reads `env.0`, and `env.1` only if `env.0` is not a valid block.
  A cut while one file is written leaves the other holding a state the
  activator meant.
- **Why a pair:** `grub-editenv set` truncates the file and rewrites it
  in place (only `create` writes a new file and renames it), FAT has no
  journal, and GRUB's `save_env` rewrites the whole 1024-byte block in
  place, two sectors. Any of these writes can be torn by a power cut,
  which leaves that file invalid; one file alone could leave no default
  at all.
- **The activator reads both files directly** (a 1024-byte sized format
  in `hearth-protocol-grub`) and, at each start, makes a mismatch whole:
  the other file from a valid `env.0`, or `env.0` from a valid `env.1`.
- **Writes per boot:** GRUB's count, and the activator's reset, two
  small writes to each file at every boot; a FAT partition on a VPS's
  disk bears them.

### 2.6 Default, trial and bless

The default is `default` in the valid file GRUB would read; a trial sets
`next_entry` in both files; a bless sets `previous`, `default` and
`tries` in both (hearth/docs/design/domain/slots/bless.md, 1.3). A `next_entry` found still set on the
trial's boot means GRUB did not save, and the activator clears it at
once (hearth/docs/design/domain/slots/trial.md, 3).

## 3. The kernel's command line

### 3.1 Tokens

| Token | Value | Why |
|---|---|---|
| `initrd=` | `\initrd.img.<target>.<label>` | EFI only: the stub loads it from the ESP. GRUB's fragment has its own `initrd` line |
| `root=` | `UUID=<fs uuid>` | tiny-initramfs resolves `UUID=` for btrfs; the device never appears in an output |
| `rootflags=` | `subvol=@<target>.<label>` | passed as mount data by tiny-initramfs |
| `ro` | | tiny-initramfs mounts the root read-only, as fstab does after it |
| `panic=` | `10` | Debian builds with `CONFIG_PANIC_TIMEOUT=0`, which hangs on a panic; a trial must end in a reboot |
| the `boot` part | its parameters, in their order, such as `console=ttyS0,115200n8` or `audit=0` | the target's own, from task data keyed by hardware (hearth/docs/design/protocol/format/tasks.md, 4; hearth/docs/design/domain/render/spec.md, 1.3); the last `console=` is `/dev/console` |
| `hearth.trial=` | `<target>.<label>` | the trial entry's line only (1.2), or GRUB's on a trial boot (2.3): it arms the dead-man before any mount (hearth/docs/design/domain/slots/prepare.md, 5) |

An example, for `x1-prod`'s slot `b` on EFI, whose `boot` part is
`audit=0`, with no console:

```
initrd=\initrd.img.x1-prod.b root=UUID=0f3c9a6e-5b2d-4c1e-9a7f-2d8e6b1c4a90 rootflags=subvol=@x1-prod.b ro panic=10 audit=0
```

### 3.2 Rules

- **Reserved keys:** the `boot` part may not set `root`, `rootflags`,
  `ro`, `rw`, `initrd`, `panic` or anything beginning `hearth.`, which
  are the activator's, nor `rootfstype`, `rootwait`, `rootdelay` or
  `init`, which would change how tiny-initramfs finds and starts the
  root; `rootwait` in particular would make a missing root wait for ever
  instead of panicking. Render refuses them (hearth/docs/design/protocol/format/tasks.md, 11), and the
  activator refuses them again.
- **Tokens** are printable ASCII without spaces, quotes or what GRUB's
  script reads (`\`, `$`, `;`, `&`, `|`, `<`, `>`, `{`, `}`, `#`;
  hearth/docs/design/protocol/format/tasks.md, 4); a value with a space cannot be expressed. Parameters are compared and written as
  tokens, never as one string.
- **Length:** the whole line, the trial's token included, is at most
  2048 bytes with its terminating NUL, x86's `COMMAND_LINE_SIZE`. The
  `boot` part is refused when the line would be longer, counted as an
  EFI entry carries it, its `initrd=` token included, on BIOS too: one
  rule for every target, and a target that moves to EFI hardware keeps
  a line that fits.
- **Nothing in hearth reads `/proc/cmdline`** to learn which slot runs:
  after a soft reboot it still names the slot the kernel was booted for
  (hearth/docs/design/domain/slots/README.md, section 3).

### 3.3 Where it is written, and when it changes

- **Written at prepare only,** into the EFI entry or the BIOS fragment
  (2.4); a trial and a bless never touch it.
- **A changed line** costs one entry write on EFI (1.3) and forbids a
  soft reboot (hearth/docs/design/domain/slots/trial.md, 1).

## 4. The initramfs

### 4.1 Configuration

tiny-initramfs's configuration, `/etc/tiny-initramfs/tiny-initramfs.conf`,
is part of the `packages` part (hearth/docs/design/domain/render/spec.md, section 1), because the
kernel's postinst hook builds the image while packages install, and the
configuration must exist before that (hearth/docs/design/domain/build/packages.md):

```
DEBUG=no
AUTO_MODULES=no
MICROCODE=no
INCLUDE_MODULES=btrfs,virtio_pci,virtio_scsi,sd_mod
```

- **`AUTO_MODULES=no`** always: `yes` inspects what mounts `/` where the
  hook runs, which inside a build is the builder's own root.
- **`MICROCODE=no`** on a VM (placement's kind), whose hypervisor loads
  microcode; `generic`, every installed microcode package, on bare
  metal. `yes` means the build host's processor.
- **`INCLUDE_MODULES`** is the packages part's `initramfs` set, which the
  `boot` task keys by hardware profile (4.2; hearth/docs/design/protocol/format/tasks.md, 4), in the
  set's order, bytewise (hearth/docs/design/domain/store/README.md, 3.1), comma-joined: the file follows
  the part's digest, whatever order a list in 4.2 or a task gives.
  `mktirfs` resolves their dependencies when it makes the image, and
  nothing is resolved at boot.
- **The image is in the output,** at `boot/initrd.img-<version>`,
  independent of the build host. Its bytes differ between two builds of
  the same inputs, since mktirfs's cpio keeps inode numbers and mtimes
  (hearth/docs/design/domain/build/reproducible.md, 2.2), so the soft-reboot rule compares its inputs,
  never its bytes (hearth/docs/design/domain/slots/trial.md, 1).

### 4.2 Modules per hardware profile

| Profile | Example | `INCLUDE_MODULES` | `MICROCODE` | Consoles, in the `boot` part |
|---|---|---|---|---|
| `7th-gen` | `x1-prod` | `btrfs,nvme` | `generic` | none |
| `incus`, own disk | the VM tier | `btrfs,virtio_pci,virtio_scsi,sd_mod` (to verify, by M2's boot test) | `no` | `console=ttyS0,115200n8` |
| `provider-a` | `east-prod` | `btrfs,scsi_mod,sd_mod,virtio_scsi,virtio_pci,virtio_mmio` | `no` | `console=tty0 console=ttyS0,115200n8` |
| `provider-b` | `west-prod` | `btrfs,virtio_blk,virtio,virtio_pci,virtio_rng,scsi_mod`, and `vmw_pvscsi` (needed in 2022) | `no` | `console=tty0 console=ttyS0,115200n8` |
| `provider-c` [later] | south | `btrfs,virtio_blk,virtio_pci` | `no` | `console=tty0 console=ttyS0,115200n8` |

These are the bash fleet's lists, learned by trial and error; `virtio_blk`
needs `virtio_pci` to work. They move from a template keyed by hardware
into the `boot` task's settings (hearth/docs/design/protocol/format/tasks.md, 4); the `incus` list for a
VM with its own disk is new, and `envs/lab.json` adds it.

### 4.3 What it does, and what it cannot

- **It finds the root** by `root=` as a kernel device name, `UUID=` (for
  ext2, ext3, ext4, xfs and btrfs) or `0xMAJMIN`. It does not resolve
  `PARTUUID=`, `LABEL=` or any `/dev/disk/by-*` name: it has no udev.
  os accepted those and could not boot them; hearth writes `UUID=` only.
- **It mounts with `rootflags=`, `rootfstype=` and `ro`,** reads `/etc/fstab`
  only for a separate `/usr`, and runs `init=` or `/sbin/init`.
- **It cannot do** LVM, LUKS, FUSE, fsck, overlays, idmapped mounts, a
  virtiofs or 9p root, or a multi-device btrfs (it scans no devices). A
  machine's btrfs is one device.
- **Every failure panics.** On an error it prints, sleeps 10 seconds and
  exits, and init exiting panics the kernel. A missing root device
  panics after 180 seconds, a constant compiled in; `rootdelay=` only
  adds to it, and `rootwait` waits for ever, which is why both are
  reserved (3.2). With `panic=10`, a trial whose root is missing reboots
  into the default after about three minutes.

### 4.4 After a soft reboot

A soft reboot skips the initramfs, so the activator mounts
`/run/nextroot` as tiny-initramfs would have mounted the root
(hearth/docs/design/domain/slots/trial.md, 2).

### 4.5 Debian's package

- **hearth uses Debian's `tiny-initramfs` as it is:** trixie has
  0.1-3.1 and forky 0.1-8. It has been stable for years, and it boots
  the fleet today with the lists above.
- **Its maintenance:** upstream's last commit is from 2016, and Debian
  orphaned the package on 2026-08-02 (#1143475); 0.1-8 is a QA upload.
  Its stability is why it is used, not a reason to replace it. Should
  Debian drop it, it is about 2.3k lines of C, and hearth could carry
  it.

## 5. Firmware quirks and Secure Boot

| Quirk | Seen on | What it breaks | hearth's answer | Reported as |
|---|---|---|---|---|
| prunes the options `BootOrder` lists whose file cannot be loaded, or that sit on a disk without a bootindex | OVMF, every boot | a blessed slot's normal entry; never a slot being written, nor a trial entry, which are outside `BootOrder` | the entries are checked, and made again, before every trial and at every start | fact: entry missing |
| reorders the options `BootOrder` lists by device order | OVMF, every boot | the default | the relative order of two entries on one disk is kept (to verify); the activator restores it at start | fact: order changed |
| ignores `BootNext` | some vendors' firmware | the trial never runs | `BootNext` still set on the next boot: cleared, the slot stays prepared | fact: one-shot ignored |
| drops entries not in `BootOrder` | some vendors' firmware | the trial's entry | made again before each trial; if it is gone by boot, as above | fact: entry missing |
| discards all variables | provider C | every entry, and the default | open (5.4) | fact: entries missing |
| no boot manager | old OVMF without NVRAM | entries | none: such firmware is out of scope | |

### 5.1 OVMF

At every boot OVMF refreshes the options `BootOrder` lists, removes
those it cannot load, and calls `SetBootOrderFromQemu()`, which reorders
them by QEMU's device order (Incus always gives disks a bootindex).
Options outside `BootOrder` are left alone. Hence both slots on one
disk, entries outside `BootOrder` until blessed and while written, and
fixed numbers reused (1.2 to 1.4).

### 5.2 Other firmware

Nothing public describes GCP's, Azure's or Hetzner's UEFI; AWS
publishes its own as edk2 patches. hearth assumes nothing beyond the
UEFI specification and checks every boot, so a quirk shows up as a
fact, never as a slot silently left on trial.

### 5.3 Hangs and resets

The UEFI watchdog (5 minutes) is disabled at `ExitBootServices()`, so it
covers the boot loader only; after that a hang needs the machine's own
watchdog (hearth/docs/design/domain/slots/trial.md, 5).

### 5.4 Firmware that discards its variables [later]

- **A stub-only machine has nothing to boot** when its variables are
  gone. The removable path, `\EFI\BOOT\BOOTX64.EFI`, is what keeps such
  a machine bootable, and a bare Debian kernel there has no command line
  (trixie builds no built-in one).
- **The lean:** GRUB for EFI on the removable path, with the BIOS
  design's fixed `grub.cfg`, slot fragments and environment pair on the
  ESP (section 2). The trial then lives on the ESP and needs no NVRAM,
  and most of M3b's work is reused. systemd-boot with boot counting is
  the alternative, and needs unified kernel images.

### 5.5 Secure Boot

- **Off,** on every kind of machine. A Debian kernel loaded directly by
  the firmware fails with `EFI_SECURITY_VIOLATION` (Debian signs it for
  its own chain, shim then GRUB), and shim cannot load it directly
  because Debian kernels lack the `.sbat` section shim requires.
- **A failed load is safe** (the firmware moves on to `BootOrder`) but
  no trial ever runs, so bootstrap refuses a machine whose `SecureBoot`
  variable is 1, and the activator reports it as a fact.
- **Incus test VMs set `security.secureboot=false`** (`README.md`, 2.2). Bare metal is
  switched off in the firmware's setup by the person bootstrapping it.
- **Later:** enrolling hearth's own key in `db` and signing kernels or
  unified kernel images (Incus 4.4 and later take
  `initial.secureboot.db`), or shim, then a signed loader, then the
  kernel (`README.md`, section 6; README, `boot.md`, section 5). Lockdown does not block
  efivarfs, so hearth's writes would keep working.
