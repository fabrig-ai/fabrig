# Testing hearth

Provisional, 2026-10-09, revised 2026-10-19. How hearth is tested. The
strategy is `skein/docs/foundation/testing-strategy.md`: the tiers, faults, what every fake
shares, what every world checks, scenarios and the referee, the two
suites, and where a failure is fixed. This document keeps what is
hearth's: its tiers and the three it adds outside the gate (section 2),
the fake machine (`fake-machine.md`), the fakes of its tools (`tool-fakes.md`),
conformance (`README.md`, section 3), real builds (`real-builds.md`), the VM tier (`vm.md`
), its worlds and their invariants (`worlds.md`), budgets and costs
(`README.md`, section 4), and what each milestone adds (`README.md`, section 5). It applies from
M1 and grows with every milestone. What is open is in `README.md`, section 6.

## 1. In one page

- **Efficiency is a rule of the tests too.** Every test runs as a regular
  user, and the gate needs Linux and git, nothing else. A world settles in
  milliseconds; the focused suite takes 30 CPU seconds and the fuzzy one
  120 CPU seconds (60 seconds before the user's doubling of 10-22,
  provisional). Each crate lands with its world, and each tool's protocol
  machine with its fake and its recordings.
- **The domain runs whole on a fake machine** (`fake-machine.md`): files with
  owners, modes, extended attributes, inode numbers and change times;
  btrfs under the kernel's unprivileged rules; user namespaces and id
  maps; processes running the tools' fakes; a Debian-shaped package
  universe; firmware variables and GRUB's env; reboots, soft reboots,
  panics and a watchdog; power cuts with a crash model; full disks.
- **Every tool has a realistic fake with one face** (`tool-fakes.md`): an
  argument vector in; progress, output and an exit code out; effects on
  the fake machine. Output follows recordings of real runs, and the
  seed draws the faults real runs meet.
- **Conformance keeps the fakes honest** (section 3): one scenario runs
  against the real tool and its fake, and what each leaves behind is
  compared. The fake machine's own rules are held to the kernel too.
- **Real builds run as a regular user** (`real-builds.md`), against a small
  pinned local mirror fetched once, through hearth's own store: the base
  layer is built once per mirror digest, and each test pays only for its
  own step. M2 ends with one of its builds booting in QEMU (`real-builds.md`, 6).
- **What needs root runs on Incus VMs** (`vm.md`) with their own disk,
  OVMF, Secure Boot off and an iTCO watchdog: bootstrap, trials, every
  fallback, power cuts, and live changes with real overlays and real
  services. It stays outside the gate until the lab exists.
- **Every world checks hearth's promises** (`worlds.md`): a crash leaves
  the default booting a blessed slot; the four facts agree with the
  machine; no secret and no identity in an output; a sealed layer never
  changes; nothing unverified is placed; every order gets one answer.
- **Costs are behaviour** (section 4): worlds hold hearth's cost facts
  to what changed and to the fake machine's own count; the real tier
  measures them; a regression in efficiency is a finding.
- **The fake machine is temper's too** (`fake-machine.md`, 11). It starts in hearth's
  `testing/`, its generic half knowing nothing of Debian, boot or
  hearth, and that half moves into skein when temper builds its machine.

## 2. The tiers

Each tier makes one more layer real (skein/docs/foundation/testing-strategy.md, section 2).
io worlds are skein's: hearth does not retest io.

| Tier | Real | Faked | Replays | Budget | From |
|---|---|---|---|---|---|
| step tests | one step function, or one protocol machine's step | its events, by hand | yes | focused | M1 |
| domain worlds | one domain: a child with the world as its parent, or a root with its children | the machine and the tools at their domain faces; the other process, scripted | yes | focused; sweeps fuzzy | M1 |
| system worlds | the builder's, the tree helper's, the steward's and the activator's domains together | the machine and the tools at their domain faces; the world carries the link's, the jobs' and the tree helper's records | yes | focused; sweeps fuzzy | M2 |
| machine worlds | one protocol machine | the stream below; the user above; the tool's bytes: recordings, generated, mutated | yes | focused; fuzz targets | M1 |
| protocol worlds | the protocol layers, their machines and the domains above, at both ends | the tools' fakes at their io face; the bytes between two hearth processes | yes, in plaintext | focused | M1 |
| simulated worlds | every process's `iterate`: steward, builder, tree helper, activator, client | the kernel, by skein's simulator; the machine and the tools behind it | yes, in plaintext | focused, a few; fuzzy | M1 |
| real loop | hearth as it ships, in one loop over one ring; real git | nothing: it runs only what needs no prerequisite | no | focused | M1 |
| end to end | the `hearth` binary, as processes | as the real loop | no | focused, one per path through `main` | M1 |
| conformance | each real tool as a regular user; real btrfs; OVMF and SeaBIOS in a VM | nothing: the same scenario also runs on the fake | no | `conformance` profile | M1 |
| real builds | the builder, the tree helper and the real tools on real btrfs, as a regular user | the network: a pinned local mirror | no | `real` profile | M2 |
| boot test | a built root, its kernel and its initramfs, from the EFI stub up, in QEMU | the disk, made from the output; the ESP and the activator | no | `boot` profile | M2 |
| VM tier | everything, as root, from the firmware up | the hardware: an Incus VM | no | `vm` profile | M3 |

### 2.1 Step tests

Every step crate tests its step functions with events by hand: render's
merge of a variant that removes a key, a key's canonical encoding, a
slot's transition, the activator refusing an operation outside its
closed list. Each protocol machine's step meets one delivery, one
demand, one refusal.

### 2.2 Domain worlds

Most of hearth's tests. Each child domain has a world with the world as
its parent; each root (`hearth-domain-builder`, `hearth-domain-steward`,
`hearth-domain-activator`) has one with its children beneath it.

- **The machine and the tools sit at their domain faces** (`fake-machine.md`, 1, `tool-fakes.md`, 1):
  the world turns the domain's requests into the fakes' typed calls, and
  their outcomes into the domain's events, as the protocol layer would.
  The domain never sees a tool's text here or anywhere.
- **The other process is scripted:** the builder's world plays the tree
  helper and the steward; the steward's world plays the builder and the
  activator, including builders that die, hang or answer for a stale
  generation (hearth/docs/design/domain/steward.md, section 8); the activator's world plays the
  steward, including one whose store lies (`worlds.md`, 3).
- **A restart is cold.** A reboot or a power cut ends every domain; the
  world builds each again from what the fake machine holds, as its
  startup reads it back (hearth/docs/design/domain/store/README.md, section 6; hearth/docs/design/domain/activator/README.md, section 6).

### 2.3 System worlds [M2]

`hearth-system-world`, under `tests/system`, is the crate of hearth's
system worlds. It begins at M2 with the builder and the tree helper; the
steward and the activator join it at M3. Its two worlds are each joined
on one fake machine:

- **The builder and its tree helper** [M2]: the layers the helper
  snapshotted, froze and emptied are the ones the builder named; the
  change set it computed is the one the builder records.
- **The daemon and the tree helper** [M3]: the activator, the steward
  it starts, the builders the steward spawns, and the helper, across
  reboots, power cuts and builders lost at any job. The output the
  activator placed is the one a builder built; the slot the steward
  asked to bless is the one running, after the bless's render
  (hearth/docs/design/domain/steward.md, section 6).

The world carries the link's, the jobs' and the tree helper's records
between the domains, as their protocol layers would
(skein/docs/foundation/testing-strategy.md, 2.3).

### 2.4 Machine worlds

Each protocol machine (hearth/docs/design/README.md, 4.2) is fed recordings of real runs
(`tool-fakes.md`, 5), each with what it must decode to, output generated from the
seed, and both mutated: mmdebstrap's `I:`, `W:` and `E:` lines and
failure tails; apt's `Inst` lines and `Packages` stanzas; `apt-helper`'s
`Get:` and `Err:` lines; `dpkg-query`'s fields; efivarfs's variables as
OVMF and x1-prod's firmware write them; GRUB's environment blocks
[M3b]; strict JSON from the test repositories. Each machine is fuzzed
alone.

### 2.5 Protocol worlds

Both ends of each connection, joined by streams cut at random: the
builder and the tools' fakes at their io face (`fake-machine.md`, 1) [M1]; the builder
and its tree helper [M2]; the steward and the activator over the link,
the steward and its builder over the jobs, and the client and the
steward [M3]; the steward and the engine's fake over skein's channel
[M4]; two machines, bytes by digest [M6]. They run
the system worlds' scenarios through bytes: the argument vectors hearth
makes, a tool's output cut mid-line, a pipe that fills, a tool killed
mid-write.

### 2.6 Simulated worlds

Each process's `iterate` on skein's simulator, the fake machine behind
its machine seam (`skein/docs/design/testing/simulator.md`, section 3). A spawn starts a
tool's fake, or one of hearth's own processes in the same world: the
activator spawning the steward, the steward its builder, the builder
its tree helper. A reboot
ends every process; the fake firmware boots a slot; its PID 1 starts the
activator, which reads its records from the fake's files.

### 2.7 The real loop and end to end

In the gate, these tiers run only what needs no prerequisite: no btrfs,
no subordinate ids, no mirror, no root.

- **The real loop:** the builder rendering with real git on a scratch
  repository, its resolutions already in the store by key so no
  mmdebstrap runs [M1]; the tree helper walking, hashing, applying and
  emptying a scratch tree, in a user namespace that maps only the test
  user's own id, which needs no subordinate range [M2]; a real program
  that never exits, stopped by signals to its process group, since a
  cgroup's kill needs a delegated subtree and is the real tier's [M2];
  the client on a Unix socket [M3].
- **End to end:** `hearth` as processes, one test per path through
  `main`: the client's render and diff in-process [M1]; the steward as a
  daemon, the builders it spawns, and the client talking to it [M3]. The
  activator's `main` needs root, so the VM tier runs it.

### 2.8 Outside the gate

Four tiers are hearth's own: conformance (section 3), real builds
(`real-builds.md`), the boot test (`real-builds.md`, 6) and the VM tier (`vm.md`). They do not replay. A failure
found there is rerun lower down as a scenario, where it replays, and
fixed there (skein/docs/foundation/testing-strategy.md, section 9).

### 2.9 Layout

```
crates/*/src/tests.rs               step tests
testing/hearth-fake-machine         the fake machine (section 3)
testing/hearth-fake-<tool>          a tool's fake: its domain and its face (section 4)
testing/hearth-fake-engine          the engine's fake [M4] (4.6)
testing/hearth-client-fake-<tool>   a fake of a tool only the client runs: incus [M3b]
testing/recordings/<tool>/<version> recordings of real runs (4.5)
testing/hearth-recordings           the recordings' reader (4.5)
testing/repos                       the test IaC repositories, cue tasks included (8.2)
testing/mirror                      the local mirror's pin, its extras, its fetch script (6.2)
testing/hearth-test-mirror          the local mirror as the tiers read it, and its proxy (6.2)
testing/hearth-test-fixtures        what more than one test crate reads: the goldens as corpora, a hex literal, the hostile mutations
testing/hearth-test-render          the render goldens' readers, the generated repositories, the scripted parent
testing/hearth-test-repository      a scratch repository of the test IaC and a run of the built binary
testing/hearth-test-git             git's expected answers and their replay
tests/<name>                        hearth-<name>-world: one subject's domain and machine worlds (2.4)
tests/system                        hearth-system-world: the system worlds (2.3)
tests/client, tests/dev             the client's worlds, hearth-client-world and hearth-client-dev-world [M3, M3b]
tests/*/tests/suite/*.rs            focused tests
tests/*/tests/suite/fuzzy/*.rs      fuzzy tests
tests/conformance/tests/*.rs        conformance (section 5)
tests/real/tests/*.rs               real builds (section 6)
tests/real/tests/boot.rs            the boot test (6.6)
tests/vm/tests/*.rs                 the VM tier (section 7)
```

The world harness is skein's `skein-world`, and the counting allocator
`skein-heap` (`skein/docs/design/testing/README.md`, section 5).

### 2.10 What changes from os

- **The machine's behaviour is tested at the gate.** os's 622 default
  tests never ran mmdebstrap, apt, dpkg, btrfs or sudo; its real tests
  sit behind cargo features and `#[ignore]`, need passwordless sudo, and
  skip themselves when a tool is missing. hearth runs all of it on the
  fake machine, and its real tiers need no sudo and refuse to start,
  rather than skip, without their prerequisites (`real-builds.md`, 4).
- **Recordings replace hand-written logs:** os's parser fixtures were
  approximations; hearth's are captured from real runs.
- **Boots are tested.** os has none: mocks of efibootmgr on `PATH`, and
  loopback images under sudo. The bash fleet's lab booted a virtiofs
  root with no firmware or bootloader. hearth's VMs boot from their own
  disk through OVMF, and SeaBIOS at M3b.
- **The dir backend and `--resolved` go:** the fake machine and its
  universe take their place. os's golden files inform render's tests but
  are not their oracle (hearth/docs/design/decisions.md). The bash fleet's protocol
  fakes (efibootmgr, mmdebstrap, systemctl) become fakes on the fake
  machine; the three renderings of `efibootmgr -v` os parsed are no
  longer parsed, since the activator reads efivarfs (hearth/docs/design/domain/machine/boot.md, 1.5).

## 3. Conformance

### 3.1 The rule

The same scenario runs against the real tool and against its fake, and
what each leaves behind is compared, as skein holds its simulator to the
kernel (skein/docs/foundation/testing-strategy.md, section 5).

- **A difference is a conformance bug:** the scenario first, then the
  fake.
- **A behaviour found in a real tool** that the fake lacks becomes a
  conformance scenario first, then a fake's feature.
- **Where the real tool draws** among outcomes (the order dpkg
  configures in, so the ids users get), the fake shows each over its
  seeds, and the real one's is among them.

### 3.2 Scenarios

| Scenario | The real side | What is compared |
|---|---|---|
| resolve | `mmdebstrap --simulate --mode=chrootless` through the local mirror, as its proxy (`real-builds.md`, 2; hearth/docs/design/domain/render/spec.md, 2.2) | the closure: names, architectures, versions; the `Packages` stanzas it left, in the fields the universe keeps (`fake-machine.md`, 6), the others one allowed difference (3.3) |
| resolve, both modes [M2] | the same requests simulated with `--mode=chrootless` and with `--mode=unshare`, for every variant | the two closures, which must be equal: os and the bash fleet resolved chrootless and built unshare, and a reason once found for preferring unshare was lost (hearth/docs/design/domain/render/README.md, section 7). With M2's real builds, since unshare needs the subordinate range (`real-builds.md`, 4) |
| fetch | `apt-helper download-file` through the local mirror, as its proxy, over loopback | files by digest; a 404's and a mismatch's exit, lines and leftovers |
| bootstrap | mmdebstrap as hearth/docs/design/domain/build/README.md (`README.md`, 3.4, `worlds.md`, 3) runs it: as the root of a sandbox's user namespace, `--mode=unshare`, chrooted into a staging layer | the installed set; dpkg's status, stanza by stanza; the tree: paths, kinds, modes, owners as the namespace sees them, attributes, link targets, contents by digest |
| the same, `--mode=root` | the bootstrap again in root mode | the two trees, to settle hearth/docs/design/domain/build/README.md's open choice (`../domain/build/README.md`, section 13) |
| install the extras | the bootstrap with the extras repository | users made; conffiles recorded; setuid bits; version 3 capabilities and their root id; a failing postinst's status and tail |
| query | `dpkg-query` on the bootstrapped layer, in hearth/docs/design/domain/build/packages.md 7's format | stdout, byte for byte |
| the target's tools | `systemd-sysusers`, `systemd-tmpfiles` and `SYSTEMD_OFFLINE=1 systemctl preset-all`, inside a bootstrapped layer | users and groups made, files made, links made |
| git | real git on scratch repositories | revisions resolved, listings, blobs; hunks of `diff --no-index` on changes whose shortest diff is unique (a line changed, added or removed, lines appended), where any two line diffs agree |
| host keys | `ssh-keygen` as hearth/docs/design/domain/activator/README.md (5.7) runs it, into a scratch directory | file names, modes and formats; never the keys |
| the btrfs rules | the builder's and the tree helper's operations on the real tier's btrfs | each row of `fake-machine.md`, 3's table, as 3.4 says |
| env file [M3b] | `grub-editenv` on a scratch file | the file, byte for byte; `set` truncating it and rewriting it in place |
| firmware [M3] | efibootmgr's writes in an Incus VM on OVMF, for both of a slot's entries | efivarfs's variables after each write; exit 40 for `--create-only --bootnum N` over an existing `N`; after a reboot, `BootNext` gone and `BootCurrent`; what OVMF dropped and reordered, of `BootOrder`'s options only; `efibootmgr -v`, recorded |
| GRUB [M3b] | the fixed `grub.cfg` and the environment pair on a SeaBIOS VM | `next_entry` consumed in both files before the kernel, each save the whole block, two sectors; the boot counter; the default booted after a panic |

### 3.3 What is compared, and what is not

- **Trees are compared as namespace ids,** as tree digests are
  (hearth/docs/design/domain/store/README.md, 3.4), so the subordinate base on disk does not matter.
- **Left out:** times, inode numbers, the order of entries, keys' bytes,
  logs that carry a timestamp.
- **Allowed differences are data,** each with its reason, and kept
  short.

### 3.4 The fake machine against the kernel

The fake machine's own rules (`fake-machine.md`, 2 to 4) are held to the kernel, as a
regular user on the real tier's btrfs directory (`real-builds.md`, 4): each row of `fake-machine.md`, 3's
table, the builder refused a snapshot and a freeze of a layer its range
owns and the tree helper allowed them; a frozen root that cannot be
renamed into another directory; a snapshot keeping inode numbers and
ctimes; chown clearing setuid and capabilities; a capability's root id
set in a namespace; ACL ids; a map of two extents; and `fake-machine.md`, 13's mounts,
in a user and mount namespace of the regular user's own: a delta of
puts and 0/0 whiteouts merged over a base giving the new tree exactly,
for adds, changes, deletions, kind changes, links and modes; a
directory taking the topmost layer's attributes; a beneath-swap with
no window under readers in a loop; the propagation traps. Real owners,
setuid bits, capabilities and `trusted.*` attributes through an overlay
need root, and wait for the VM tier (`vm.md`, 3). Each case names the
rule it checks. No running kernel can check
the crash model, as `skein/docs/design/data/kv.md` says of its own: it rests on the
sources it cites, and the VM tier's power cuts (`vm.md`, 3) test hearth, not
the model.

### 3.5 When it runs

`cargo nextest run --profile conformance`, as a regular user, with its
scenarios' prerequisites and the mirror; firmware scenarios run in the
VM tier. The profile needs what its scenarios need: in M1 (resolve and
git), git, mmdebstrap and the mirror's indexes, and none of `real-builds.md`, 4's; from
M2, `real-builds.md`, 4's too. Without them it refuses to start, naming what is missing
(`real-builds.md`, 4). It runs when a fake or a recording changes, when a tool
moves (a trixie point release, a mirror bump), and before a milestone
lands. With recording on, it rewrites the recordings (`tool-fakes.md`, 5).

## 4. Budgets and costs

### 4.1 The two suites

The focused suite takes at most **30 CPU seconds**, the user's ruling of
11-04, kept in `tools/tiers/budgets.toml` and measured as the gate does
(`tools/tiers/design.md`, 7.3). The fuzzy suite takes at most **120 CPU
seconds** (60 seconds before the user's doubling of 10-22, provisional), the
user's word of 11-07, kept and measured as the focused suite's. nextest's
global timeouts stay, as the suites' wall backstop
(`skein/docs/foundation/testing-strategy.md`, 8.1). Raising either is an
explicit decision (skein/docs/foundation/testing-strategy.md, section 8).
temper's gate recorded 1,092 focused tests in 5.2 seconds; the budgets hold
when the design is cheap from the first step:

- **The fakes are small:** a universe of tens of packages, trees of
  hundreds of files, snapshots that copy no bytes, a fake bootstrap in
  milliseconds.
- **Time is simulated:** a trial's window, a stall's deadline, a panic's
  ten seconds cost nothing.
- **Real processes are few:** one test per path through `main`, and
  real git on tiny repositories.
- **A fake's call costs what it touches:** the fake machine keeps what a call
  changed and how many bytes it holds, and the tree world's filesystem keeps its
  names and link counts, so neither scans all its nodes per call.
- **A scratch repository is written:** `hearth-test-repository` writes its blobs,
  trees, commits and ref as loose objects, with the ids `git add -A` and
  `git commit` give, so real git runs for the code under test alone. One test
  holds the two equal. Such a repository has no index until git makes one, so
  whatever adds to it by hand adds with `-A`. The test that git's absence fails by
  name keeps a repository real git makes.
- **A world computes once what its seed doesn't change,** in the test's process:
  every seed still runs and still replays, and no state crosses seeds.
- **The fuzzy suite sweeps:** random worlds settled under every
  invariant, and a small prepare, trial and bless cut at every one of
  its operations in turn, as skein's kv world crashes at every
  operation.

### 4.2 Profiles outside the gate

| Profile | Runs | Needs | Ceiling |
|---|---|---|---|
| `conformance` | section 3 | its scenarios' (3.5): in M1 git, mmdebstrap and the mirror's indexes; from M2, `real-builds.md`, 4's too | 4 minutes (2 minutes before the user's doubling of 10-22, provisional) |
| `real` | `real-builds.md`, but `real-builds.md`, 6 | `real-builds.md`, 4 and the mirror | 10 minutes (5 minutes before the user's doubling of 10-22, provisional) |
| `boot` | `real-builds.md`, 6 | `real-builds.md`, 4, the mirror, QEMU and OVMF; KVM where it exists | 20 minutes (10 minutes before the user's doubling of 10-22, provisional) |
| `vm` | `vm.md` | KVM and Incus | 60 minutes (30 minutes before the user's doubling of 10-22, provisional) |
| `measure` | both suites with `-j 1`, timing each test | — | none |

Each is selected by its test binaries' names and kept out of the
default and fuzzy profiles, as `live` is in smith. The ceilings are
provisional, set from the first measured runs, and enforced as the
gate's budgets are: raising one is a decision. A cold run adds the base
layers once per mirror digest (`real-builds.md`, 3, `vm.md`, 4).

### 4.3 Costs as facts

hearth reports its costs as facts: time per step, bytes fetched, cache
hits (hearth/docs/design/README.md, section 5). Worlds check them two ways:

- **Against what changed.** A config change runs the config step and no
  other; a second build of one key runs nothing; an unchanged resolution
  key runs no `--simulate`; the walk hashes only files whose path, inode
  and ctime differ from the parent's record; a second fetch of one .deb
  is a hit.
- **Against the machine's own count** (`fake-machine.md`, 1): bytes read and written,
  programs run, subvolumes made. A cost fact that disagrees with the
  ledger fails, so hearth's costs can be trusted.
- **Ceilings on counts,** as temper holds its engine's requests to the
  forge: programs run per build, bytes read per walk.

The real tier keeps each test's cost facts beside the mirror digest. A
step slower than its last run on the same digest by more than a margin
is reported as a finding, with both measurements: timing on a shared
machine is too noisy to fail on.

### 4.4 The lab

Once the lab exists, it benchmarks hearth as it does any product
(`hearth/docs/design/draft/archive/factory.md`): real targets built and activated, their cost
facts compared run over run. A regression in hearth's efficiency is a
finding, as a failing check is.

## 5. By milestone

| | Tiers added | The fake machine grows | Fakes of tools | Outside the gate |
|---|---|---|---|---|
| M1 | step; domain (render, builder, with the store's resolutions and cache, hearth/docs/design/domain/store/README.md, 2.4); machine (config, template, git, mmdebstrap); protocol (builder and tools); simulated (builder); real loop (render, real git); end to end (render, diff) | files with owners and modes; processes; the universe's index at snapshot timestamps | git; mmdebstrap `--simulate` | conformance: resolve, git; the mirror's indexes |
| M2 | domain (store, build, tree); system (builder and tree helper); machine (dpkg, tree); protocol (tree helper); real loop (tree helper, process groups) | btrfs and its ownership rule; ids, maps of several extents, namespaces, cgroups, sandboxes; `openat2`'s flags; attributes, inodes, ctimes; the full universe, `systemd-container` included; full disks; power cuts on files and subvolumes, and `syncfs` | mmdebstrap's bootstrap; `apt-helper`; dpkg-query; the target's systemd tools and `tar` | conformance: bootstrap in both modes, extras, fetch, query, the target's tools, the btrfs rules; real builds; the boot test (`real-builds.md`, 6) |
| M3 | domain (steward, converge, slots, boot, disk, activator); system (the steward and the activator join); machine (link, jobs, efi, systemd, keygen, client, proc), each fuzzed; protocol (link, jobs, client); simulated (every process, across reboots); real loop (the client's socket); end to end (daemon and client); builders lost at every job, and the bless's render (hearth/docs/design/domain/steward.md, section 8); capture and the bundle (hearth/docs/design/domain/development.md, section 10) | GPT and the ESP; efivarfs, two entries per slot, and OVMF; the kernel, the initramfs, PID 1 with its isolation to emergency mode and its jobs; reboots, soft reboots, panics, the watchdog; power cuts on FAT and variables; mount namespaces, the new mount API and overlayfs (`fake-machine.md`, 13); out-of-memory kills within a cgroup (`fake-machine.md`, 10); cues; a mutable root changed by hand, apt and dpkg on a running slot | efibootmgr; systemctl at run time, its unit verbs included; systemd-tmpfiles; ssh-keygen, for SSH host keys only; mount, cp and bootstrap's disk tools; git's bundles | VM tier on OVMF, live changes included, builders killed for memory and mid-pass, and mutable slots, capture and working trees (`vm.md`, 6); conformance: firmware, host keys, overlays in a user namespace |
| M3b | machine (grub, incus); domain (dev) | SeaBIOS; msdos and the post-MBR gap; GRUB on FAT, its environment pair, its boot counter, saves that fail, and the slots' fragments; Incus's instances and pool | grub-editenv; grub-install; incus | VM tier on SeaBIOS, and the development loop on a development host (`vm.md`, 6); conformance: env file, GRUB, incus |
| M4 | protocol (the wire, on `skein-fake-channel`); machine (wire) | — | `hearth-fake-engine` (`tool-fakes.md`, 6); systemd-run, for a held bless's timer | — |
| M5 | domain (checks, backups); containers and pool hosts | units' states and signals; containers on a host (`fake-machine.md`, 12); `setns` into a container's mount namespace; read-only snapshots of class subvolumes and their pruning | systemctl restarting a unit, a container's nspawn unit and `--machine=`; systemd-run's checks | VM tier: bless with checks; containers in a host VM, live changes included (`vm.md`, 3); clusters and containers in development (`vm.md`, 6) |
| M6 | domain (artifacts; backups' sends); protocol (bytes by digest, between two machines) | send streams and a backup peer; faults as effects; virtual disks | — | real builds: artifacts; development VMs from the host's store (hearth/docs/design/domain/development.md, section 9) |
| later | identity beyond SSH host keys | — | wg, for WireGuard keys | — |

M1 sets the pattern: every tier up to end to end exists for render
before M2 begins. M2 is the largest step for the fakes, since a build
touches everything; M3, the MVP, adds the machine's lower half and the
VM tier its one real machine relies on.

## 6. Open questions

- **When the generic half moves, and its name in skein** (`fake-machine.md`, 11): it
  depends on when temper builds its machine.
- **The local mirror:** whether `test-min` and `test-boot` are the right
  requests, how large the kernel makes the mirror, and whether a runner
  keeps the fetched mirror between runs. It depends on the first
  measurements and on the runner's disk.
- **How much shell the machine interprets:** temper's subset, plus which
  file commands. It depends on temper's own open question, and on what
  the test repositories' escape hatches need.
- **The crash model's open guarantees:** whether btrfs's subvolume
  create, delete and read-only flag are final once the call returns;
  whether a rename on FAT is atomic; whether edk2's variable writes
  never tear. Until verified, the fake takes the weaker answer (`fake-machine.md`, 9).
- **The watchdog:** whether it stays armed across a soft reboot; and how
  to cause, in a VM, a hang only the watchdog ends (a kernel test
  module such as `test_lockup`, if Debian builds it). It depends on a
  VM experiment.
- **Incus as a regular user:** whether a restricted per-user project
  allows `security.secureboot=false`, `security.csm=true` and the
  snapshots `vm.md`, 4 relies on, and whether a copy carries `qemu.nvram`. It
  depends on Incus's configuration on the runner.
- **Firmware that forgets its variables:** how the VM tier causes it,
  since removing `qemu.nvram` needs the Incus host. It depends on
  hearth/docs/design/domain/machine/README.md's answer for such firmware.
- **`systemctl reboot -ff` and sync:** to verify before the fake
  settles (`fake-machine.md`, 8).
- **The version matrix:** trixie's tools only, or forky's too (GRUB
  2.14's env block, systemd 262). It depends on which suites hearth's
  targets run.
- **The ceilings outside the gate** (4.2): provisional until measured.
- **The suites' figures** (4.1): what the workspace's global timeouts
  hold, and whether `hearth-render-world` stays on every change. It
  depends on the user's ruling on the measurements.
  Bringing `tools/tiers/design.md`, 7.3 and 8, into line with the user's
  11-07 CPU ruling is separate work: those sections still call Smith's,
  Skein's and Jig's CPU figures open, although `tools/tiers/budgets.toml`
  records them as decided CPU budgets.
- **The focused suite's way to its figure** (4.1): whether what is left needs
  repair to hold the queue's figure under 30 CPU seconds. The shares below are a
  sampled profile's estimates of the suite before its repairs (the `hearth-speed-2`
  plan's figures, at `668f7cd3e5`), which overlap and do not add. What each would
  save is unknown.
  - What a world repeats: what 4.1 has it compute once, `rendered_keys` above all;
    config's `reference::decode` of one document's bytes per seed and its
    `reference::position`, which counts from byte zero (1.20% of the suite); the
    tree world's file read twice in `Helper::digests` (2.57%) and its parents
    built and decoded again per edit and per fault; the render world's expected
    directories read and templates compiled again.
  - The fake machine: a call's second path resolution, after `touched`'s.
  - Render's own code: the per-task listing scans, which `class` every entry; the
    exports cache's `prime`, which restarts after each pair, and its `needed`,
    which scans the targets; `placement`'s clones; a call's rescans in `needs`,
    `contains` and `copies`; `retain_spec`'s re-walk of what it holds; and
    `Collector::push`, which walks the retained errors twice (0.59%).
  - The git protocol's code: `hunks::append`, which pushes a chunk's bytes one at
    a time (1.30%).
  - skein's: through the tree family, `skein-heap`'s counting `alloc` (4.58%) and
    `dealloc` (4.06%) and `skein_lib::sha256::compress` (2.22%); through the
    smaller worlds, `skein_world::stream::Wire::answer` (1.57%) and
    `skein_lib::intake::Intake::take` (0.96%). Their current shares are unknown,
    and a repair there is a question first, not a task.

  Test builds stay unoptimized (`skein/docs/foundation/testing-strategy.md`, 8).
  Measurements of any remaining overrun go to the user for a ruling
  (`skein/docs/foundation/testing-strategy.md`, 8.1). The figure stands, and a run
  over it is recorded, never refused. It depends on the queue's record and on
  the user's ruling.
- **The large render's cost:** about 4 to 5 CPU seconds, which is why the fuzzy
  suite carries it (`worlds.md`, 4). No run has isolated that cost: it followed the
  work per target, not the listing, and how it divides between the harness and
  hearth's crates is unknown. It depends on a run that isolates it.
- **The conformance tier's budget and its runs** (3.5, 4.2):
  whether it takes a budget and a place after landing
  depends on the user, and on a runner holding its prerequisites (`real-builds.md`, 4).
- **Live changes on a real machine:** every item hearth/docs/design/domain/slots/README.md, section 7
  lists as to verify, run as `vm.md`, 3's scenarios. Until then the fake
  machine follows the kernel's and systemd's sources (257 to 262) as
  read for this design, and where a behaviour is unknown (overlays
  across a person's soft reboot) the seed draws each answer.
- **Differential checks:** whether a scenario must end the same at every
  tier it runs at, as temper also asks. It depends on how much the
  referee can observe in the real tiers.
