# The fakes of tools

## 1. One face

```
in    an argument vector, an environment, a working directory;
      the process's ids, namespaces and view; bytes on stdin, where the tool reads any
out   bytes on stdout and stderr, in chunks over the world's time: progress;
      an exit status, or the signal that ended it;
      effects on the fake machine, made as that process
```

- **A domain and a face,** as temper's programs have: the domain is
  what the tool does, in the fake's own vocabulary; the face parses the
  argument vector and prints the outcome. At the domain face a world
  calls the domain directly; from the io face down, every call goes
  through the argument vector. Only the invocations hearth makes need a
  face.
- **It checks its client.** It refuses an option the real tool refuses,
  answers each argument vector hearth makes as the real tool does, and
  asserts that it is given no other, since a real tool's usage errors
  differ between its commands (`git diff` exits 129, `git rev-parse
  --verify` 1); and it works only where the real one would: efibootmgr writes variables only as root; mmdebstrap
  in unshare mode makes no user namespace when it runs as root, and as
  anyone else needs `newuidmap`, which `no_new_privs` disables.
- **Output follows recordings.** Every line it prints has the shape of a
  recorded real run (5), with its varying fields (names, versions,
  counts) filled from the fake's state. Recordings are of real tools,
  run as hearth runs them, not on a terminal, each under the version
  that made it (5), and a parser holds to every version recorded. M1's
  come from the machine its plan runs on, whatever Debian it runs;
  trixie's and forky's join when a machine that has them records, as a
  tool that moves does (`README.md`, 3.5). Which versions must be recorded is open
  (`README.md`, section 6).
- **Faults come from the seed,** each one that real runs meet, and a
  sweep asserts that each fell (skein/docs/foundation/testing-strategy.md, section 3).
- **Time:** each takes the time its work takes, on the world's clock,
  so a stall meets hearth's progress deadline.
- **A step machine,** since every tier from domain worlds up needs it,
  in `testing/hearth-fake-<tool>`, acting on the fake machine.

## 2. The catalogue

| Tool | Machine | Its fake | Faults | From |
|---|---|---|---|---|
| mmdebstrap | `hearth-protocol-mmdebstrap` | resolution's `--simulate --mode=chrootless`: `Inst` lines and `Packages` indices; hearth/docs/design/domain/build/README.md's bootstrap: the closure unpacked and configured into `/target`, chrooted, from the local repository | a stall; a maintainer script that fails; a full disk; a package missing from the repository; killed | M1 (simulate), M2 |
| `apt-helper download-file` | `hearth-protocol-dpkg` | each file named, by URI, into its target, checked by its SHA-256 | a stall; a 404; a hash or size mismatch; a reset; a full disk | M2 |
| dpkg-query | `hearth-protocol-dpkg` | hearth/docs/design/domain/build/packages.md 7's four fields per package, from the layer's dpkg database | a package left half-configured by an earlier failure | M2 |
| the target's tools: `systemd-sysusers`, `systemd-tmpfiles`, `systemctl preset-all` with `SYSTEMD_OFFLINE=1`, `tar` | `hearth-protocol-builder`'s generic machine | users, files and preset links made in the layer, as its root, inside it | a tool missing from the target; a failure with a tail | M2 |
| git | `hearth-protocol-git` | hearth/docs/design/domain/render/README.md 2.2's `rev-parse`, `ls-tree` and `cat-file --batch` on a repository, and its `diff --no-index` of two files, by a line diff of the fake's own; the fetch of hearth/docs/design/protocol/standalone.md | a remote unreachable; a revision missing; a branch rewritten; a stall | M1 |
| efibootmgr | `hearth-protocol-efi` | writes only: creates at a fixed number, deletes, `--bootnext`, `--delete-bootnext`, `--bootorder`, on the variables of efivarfs | the variable store full; efivarfs not writable; a number already taken | M3 |
| systemctl, at run time | `hearth-protocol-systemd` | `soft-reboot`, `reboot`, `reboot -ff`, `is-system-running`, `show`, `stop`; for a live trial, `daemon-reload`, `start`, `reload`, `restart` and `reload-or-restart` on PID 1's jobs (`fake-machine.md`, 13); `restart` [M5]; a container's nspawn unit and `--machine=` (`fake-machine.md`, 12) [M5] | a unit that fails; a job that times out; a reload that fails; a container that never comes up | M3 |
| systemd-tmpfiles and nscd, at run time | `hearth-protocol-activator` | a live trial's `--create` of `hearth.conf` in the running `/var`; nscd's `--invalidate` | a line that fails; nscd absent | M3 |
| systemd-run | `hearth-protocol-systemd` | the hold's timer [M4]; a check's transient unit (hearth/docs/design/domain/operating.md, 2.4), and with `--machine=` in a container [M5] | the unit refused; a timeout; an out-of-memory kill | M4 |
| ssh-keygen | `hearth-protocol-keygen` | hearth/docs/design/domain/activator/operations.md 7's host key: a private and a `.pub` file, their bytes from the seed | a full disk | M3 |
| the activator's others: mount, umount, cp, and bootstrap's disk tools | `hearth-protocol-activator` | their effects on the fake machine's mounts, files and disks (hearth/docs/design/domain/activator/README.md, section 7) | a failure with a tail | M3 |
| wg | `hearth-protocol-activator` | a WireGuard key as identity | a failure with a tail | later: M3's identity is SSH host keys only |
| grub-editenv, grub-install | `hearth-protocol-grub` | `set` and `unset` on one environment file, truncated and rewritten in place; GRUB installed on the FAT partition | a full partition; a block that is not GRUB's | M3b |

## 3. mmdebstrap

- **Two invocations:** render's resolution (hearth/docs/design/domain/render/spec.md, 2.2), with
  `--simulate --mode=chrootless`, which needs no namespace and, with
  `--skip=cleanup/apt/lists`, leaves the `Packages` indices it read
  under its scratch root; and hearth/docs/design/domain/build/README.md's
  bootstrap (`README.md`, 3.4).
- **The bootstrap runs as hearth/docs/design/domain/build/README.md runs it** (`worlds.md`, 3): as the root of the
  sandbox's user namespace, in the tool root, `--mode=unshare`, so it
  unshares mount, pid, uts and ipc and no user namespace, and never
  reads `/etc/subuid`. It chroots into `/target`, the staging layer, and
  runs maintainer scripts there with the target's binaries. The fake
  gives one tree for `--mode=unshare` and `--mode=root`; whether the
  real ones do is conformance's question (`README.md`, 3.2; hearth/docs/design/domain/build/README.md, section 13).
- **It checks what the real one checks:** an empty target; one
  `--include` joined from several; the hooks run as the namespace's
  root, with the chroot as `$1`; the local repository mounted into the
  chroot by the `file-mirror-automount` hook.
- **It does what the real one does that hearth must undo:** it copies
  the tool root's `/etc/hostname` and `/etc/resolv.conf` into the
  target, the postinsts of `fake-machine.md`, 6 make keys and a machine-id, and its own
  cleanup removes the apt lists and logs and empties
  `/etc/machine-id`. The customize hook and the scan must catch the
  rest (hearth/docs/design/domain/build/packages.md, 6, and hearth/docs/design/domain/build/README.md, 7.3).
- **Its output** off a terminal is `I:`, `W:` and `E:` lines; with
  `--verbose`, apt's and dpkg's lines; on failure, the tail of
  hearth/docs/design/domain/build/README.md, 8.1.

## 4. The other tools

- **`apt-helper download-file`** takes `<uri> <target> SHA256:<hex>`
  triples (hearth/docs/design/domain/build/packages.md, 1), from the machine's network namespace with no
  sandbox, and prints `Get:`, `Err:` and `E:` lines, exiting 0 or 100;
  on a mismatch it leaves nothing at the target. Given a `file:` URI it
  links the file instead of fetching it, as the real one does, which is
  why the builder passes only `http` and `https`.
- **git** takes its mechanics from `skein-fake-checkout` (commits, trees,
  a remote the world owns); its face is hearth/docs/design/domain/render/README.md's commands and the
  fetch standalone mode makes. Its remote holds the test repositories at
  their revisions.
- **The target's tools** run inside the layer, as its root, with
  `SYSTEMD_OFFLINE=1`: `systemd-sysusers` adds the part's users to the
  layer's `/etc/passwd` and `/etc/group` with the next free ids;
  `systemd-tmpfiles` makes what the part's lines say; `systemctl
  preset-all` makes the enablement links. Only their exit and tail are
  parsed (hearth/docs/design/domain/build/README.md, 8.4).
- **efibootmgr only writes.** The activator reads every variable from
  efivarfs (hearth/docs/design/domain/machine/boot.md, 1.5); with `--quiet`, only the fake's exit and
  error lines matter. Its `-v` listing exists for conformance only
  (`README.md`, 3.2). `--create-only --bootnum N` makes an entry outside
  `BootOrder`, and exits 40 if `N` exists, as the real one does, so the
  activator deletes before it creates. A slot's trial entry in
  `BootOrder` at any time, or its normal entry before its bless, fails a
  world. `--unicode` joins several positional arguments with NULs, the
  mistake hearth/docs/design/domain/machine/README.md (2) avoids.
- **systemctl at run time** acts on the fake PID 1 (`fake-machine.md`, 7, 8).
- **ssh-keygen** writes the files and modes the real one writes, the
  key's bytes from the seed; the world records each key, so no private
  half hides in an output (`worlds.md`, 3). It is only ever asked into a `.new`
  directory (hearth/docs/design/domain/activator/operations.md, 7), and asserts it.
- **grub-editenv** keeps GRUB's 1024-byte block, byte for byte the real
  one's (`README.md`, 3.2). `set` truncates the file and rewrites it in place, so a
  cut mid-write leaves it torn or empty (`fake-machine.md`, 9).

## 5. Recordings

- **Where:** `testing/recordings/<tool>/<version>/`, each with its
  argument vector, its environment (no terminal), its working directory,
  its stdin where the tool reads one (`git cat-file --batch`), its
  stdout, its stderr and its exit. A tool whose lines another prints is
  recorded under both versions: mmdebstrap's runs under
  `mmdebstrap/<its version>-apt<apt's>/`, as a resolution's record
  keeps both (hearth/docs/design/domain/render/spec.md, 2.4).
- **From the store's root:** every conformance case runs there, as the
  builder runs its tools, so a recorded path (`./cache/…`) is the one a
  run prints.
- **One recording, two uses:** it is a machine world's transcript, with
  what it must decode to beside it (skein/docs/foundation/testing-strategy.md, 4.1), and the
  shape the fake's face prints.
- **Captured, never written by hand.** The conformance profile records
  them from real runs (`README.md`, section 3); firmware recordings come from the VM
  tier.
- **Faults are recorded too:** a 404 and a hash mismatch from the local
  mirror, a failing postinst from the extras (`real-builds.md`, 2), a full disk and a
  full variable store in a VM.
- **A new version is re-recorded.** A recording that changes is a diff
  to review, and a likely change for its parser.
- **Firmware is recorded as bytes:** efivarfs's variables after each
  step on OVMF, and, on SeaBIOS, the environment pair, are the
  transcripts of `hearth-protocol-efi` and `hearth-protocol-grub`.

## 6. The engine's fake [M4]

`testing/hearth-fake-engine` is the engine at the wire's far end
(hearth/docs/design/protocol/wire.md, section 9), a step machine like the tools' fakes:

- **Its domain** speaks the wire's records: it accepts or refuses dials,
  checks that the hello comes first, gives orders with keys and
  attempts, resends them, delays and loses acknowledgements, drops the
  channel, and opens a second channel under the same credential.
- **Its faces:** at the domain face, the wire's domain records; in a
  protocol world, bytes through skein's channel, with
  `skein-fake-channel` beneath.
- **It is shaped for the factory's infrastructure connector,** whose
  world (`jig/docs/design/domain/connectors.md`, section 15) can put it, and the fake
  machine, behind a fake hearth.
