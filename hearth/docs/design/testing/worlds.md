# Worlds and their invariants

## 1. The worlds

| World | Real | From |
|---|---|---|
| `hearth-render-world` | render's domain, under a scripted parent that plays the protocol layer and the store's records (hearth/docs/design/domain/render/README.md, 6.1) | M1 |
| `hearth-builder-world` | the builder's root and its children; tree helper and steward scripted | M1 |
| `hearth-store-world` | the store, on fake btrfs | M2 |
| `hearth-build-world` | plans and steps, on the fakes of `apt-helper`, mmdebstrap, dpkg-query and the target's tools, in sandboxes | M2 |
| `hearth-tree-world` | `hearth-protocol-tree`'s records, cut and mutated, its machine world (`README.md`, 2.4); from M2, also the tree helper's domain, on files with namespace ids | M1 |
| `hearth-system-world` | the system worlds (`README.md`, 2.3) | M2 |
| `hearth-steward-world` | the steward's root and its children; builder and activator scripted (hearth/docs/design/domain/steward.md, section 8) | M3 |
| `hearth-converge-world` | the desired revision and the orders | M3 |
| `hearth-slots-world`, `hearth-boot-world`, `hearth-disk-world` | the activator's children, on firmware, btrfs and disks | M3 |
| `hearth-activator-world` | the activator's root and its children; the steward scripted | M3 |
| `hearth-checks-world` | checks, on units and signals | M5 |
| `hearth-backups-world` | class snapshots, their retention and restores; from M6, send streams and a backup peer | M5 |
| `hearth-artifacts-world` | the second recipe | M6 |
| `hearth-client-world` | the client's protocol layer, against the steward's domain (hearth/docs/design/client/README.md, section 4) | M3 |
| `hearth-client-dev-world` | the development driver's domain, on `hearth-client-fake-incus` (hearth/docs/design/domain/development.md, section 10) | M3b |

Each document's own world section names its scenarios:
hearth/docs/design/domain/render/README.md, section 6; hearth/docs/design/domain/store/README.md, section 9; hearth/docs/design/domain/build/README.md, section 11;
hearth/docs/design/domain/machine/README.md, section 4; hearth/docs/design/domain/slots/README.md, section 5; hearth/docs/design/domain/activator/README.md, section 8;
hearth/docs/design/domain/steward.md, section 8; hearth/docs/design/protocol/wire.md, section 9; hearth/docs/design/client/README.md, section
4; hearth/docs/design/domain/operating.md, section 9; hearth/docs/design/domain/build/artifacts.md, section 8; hearth/docs/design/domain/development.md,
section 10.

## 2. Scenarios, cues and the referee

- **A scenario is set up in the fakes' terms:** the test repository at
  its revisions on git's remote; the universe and its mirror's
  timestamps; the machine's kind and disks; the secret store and the
  identities; the faults; the power cuts.
- **Boots follow content.** The test repositories have tasks that
  install real units: `test-panic` crashes the kernel through
  `/proc/sysrq-trigger`, at once or when a flag file exists in a data
  entry; `test-hang` blocks the boot's target;
  `test-mount-fails` adds a mount that cannot succeed; `test-unit-fails`
  fails; for live changes, `test-reload-fails` fails its reload when
  its configuration says so, `test-dies-later` dies a while after it
  starts, and `test-service` is an in-house service under `/opt` that
  answers with its version. The VM tier meets the real effect; the
  fake PID 1 knows these units by name and plays the same one. A
  scenario is then one repository at every tier, as temper's CI follows
  content.
- **The referee** (skein/docs/foundation/testing-strategy.md, section 7) watches the machine's
  ledger and hearth's facts, events and answers, never hearth's state.
  Its stimuli: a power cut, a reboot, a builder killed at any job, the
  steward killed, the client gone, the engine's channel dropping [M4].
  Its deadlines are simulated time, so a trial's window costs nothing.

## 3. hearth's invariants

- **Every crash leaves the default booting a blessed slot.** After every
  power cut, reboot, panic and watchdog reset, the world asks the fake
  firmware what it would boot (`BootOrder`'s first option that loads,
  after OVMF's pruning; on BIOS, `default` of the environment file GRUB
  would read), and checks that it names a
  slot whose record says blessed and whose tree equals that record.
  This holds from the first bless; what a cut during bootstrap leaves is
  hearth/docs/design/domain/machine/README.md's (`../domain/machine/bootstrap.md`). **Its container form [M5]:** after every
  host start, each container's link names a slot whose record says
  blessed, or none before its first bless; a link to a slot on trial
  never outlives its trial (hearth/docs/design/domain/slots/containers.md, 6).
- **The four facts stay consistent** (hearth/docs/design/domain/slots/README.md, section 3). At every
  settle, and with every fact hearth reports, the world compares them
  with the machine's ground truth: running is the subvolume the fake
  booted; default is what its firmware would boot; trial is the
  activator's marker; live is the fake's mount table. For a container
  [M5], running is the subvolume of its root, and default the record
  its host keeps.
- **No secret and no identity in an output.** Secrets, machine-ids and
  keys in worlds are drawn from the seed, and the world knows each. It
  searches every layer, record, fact, event, trace and build step's
  argument vector for any of them; only slots and the secret store may
  hold them. And two machines with different identities and secrets
  build one spec, with one tool seed, to equal tree digests.
- **A sealed layer never changes.** Once the tree helper freezes a
  layer and the builder publishes it (hearth/docs/design/domain/store/README.md, 4.2), the machine's
  change counter for it stays put until it is retired as `<key>.trash`
  (hearth/docs/design/domain/store/README.md, 5.3), and its tree digest equals its record.
- **The activator never places an output it has not verified.** The
  world injects a builder that lies: a layer changed after its freeze,
  changed between the activator's check and its snapshot, a record that
  does not match, an output of a hostile shape (5). No slot goes on trial whose tree is not the verified
  output shifted (hearth/docs/design/domain/activator/README.md, section 5), which the world checks
  against its own naive shift (`fake-machine.md`, 4).
- **Every order gets exactly one answer:** from the client [M3] and from
  the engine [M4], across restarts of either side, with keyed orders
  answered once by key (hearth/docs/design/protocol/wire.md, 4.4). The same holds for each
  operation the steward asks of its activator over the link, and each
  job it gives its builder ends in one answer or one loss
  (hearth/docs/design/domain/steward.md, 3.2).
- **Trials always end.** When a trial starts, the referee arms a
  deadline: the bless, or the fallback, by the trial's bound. A command
  line without `panic=` meets the fake's endless panic and fails here.
  A container's trial ends by its host's activator alone, so a host's
  steward that never asks still meets the deadline [M5].
- **A soft reboot exactly when the kernel is unchanged.** The world
  computes hearth/docs/design/domain/slots/trial.md 1's rule on its own, from the fake trees: the
  digests of the kernel image, of the modules subtree and of the
  initramfs's inputs (never its bytes), and the command line without
  the slot's tokens. Each trial's kind must follow it. The named
  scenario is a snapshot bump that keeps the kernel: the packages layer
  is rebuilt, the initramfs's bytes differ, and the trial is a soft
  reboot; a bump that changes a module takes a full one.
- **Build steps stay in their sandbox.** A step reaches only its view,
  and no network (`fake-machine.md`, 5).
- **Live exactly when allowed.** The world computes hearth/docs/design/domain/slots/trial.md, 7's
  rules on its own from the fake trees and specs, and each trial is live
  exactly when they hold; the merged view after every swap equals the
  live record's manifest, which equals the new slot's files under the
  three trees.
- **The steward stays in view** (hearth/docs/design/domain/steward.md, section 5). Whatever its
  builder does, dying at any job, hanging, or killed for memory, the
  steward keeps answering the client and, from M4, its channel; a job
  lost twice fails as `crashed`, and the pass follows hearth/docs/design/domain/steward.md, 5.1's
  table; nothing outside `steward/builder/` is killed for the builder's
  memory; and the ledger shows the steward opening nothing beneath
  `layers/` or `repo/` and spawning no build tool (`fake-machine.md`, 1; hearth/docs/design/domain/steward.md,
  2.2).
- **No native bless before the bless's render.** No slot is blessed by
  its own steward before a builder of that steward has rendered the
  slot's revision; a render that fails ends the trial in a fall back
  (hearth/docs/design/domain/steward.md, section 6).
- **A live state never moves the default.** No live bless writes
  `BootOrder` or `default`; after every reboot or power cut no overlay
  remains, and the firmware boots a blessed slot or the trial entry a
  live bless armed; and every live trial ends within the base's window,
  by the live bless, the live fall back or a reboot.

## 4. The strategy's checks

- **Contracts** as the world goes; **settled invariants:** no live
  entity, nothing in flight, every child gone and its pipes read to the
  end.
- **Memory:** `skein-heap` measures each fixed scenario, in a test binary
  of its own, and holds its peak under a ceiling named beside its test
  (skein/docs/foundation/testing-strategy.md, section 6). A scenario that
  must reach a limit shows it by what it holds, and a formula left at an
  edge keeps a test of the edge's value
  (skein/docs/foundation/programming-model.md, 6.3). The large render, a
  revision of 4,096 entries, 512 of them documents, declarations and
  templates, with 64 targets, runs through the builder at the shipped
  limits, in the fuzzy suite, under a ceiling of its own: it is the
  measure that the defaults fit a modest machine
  (hearth/docs/design/domain/render/README.md, 5.4). A client that stops reading
  has a ceiling too, in the focused suite: it is the measure that the
  watermark holds render, and removing the watermark passes it.
- **Replay:** a seed replays to the same trace, the same fake machine,
  whose state the fake digests itself, and the same digest of hearth's
  own state: a digest over the state types' derived `Hash`, which
  hearth asks of skein (hearth/docs/design/README.md, 7.1; `skein/docs/design/lib.md`, 5.1). Each
  process's `state_digest` and the fake machine's are read at the end of
  a run, and after every iteration when a world asks (
  `skein/docs/design/testing.md`, section 5).
- **Transition coverage:** `cargo llvm-cov` over both suites lists the
  handlers never reached; each is a missing scenario.
- **Fuzzing:** each machine and decoder alone; step functions fed
  recorded events.
- **Functions against naive ones:** the shift against a naive shift;
  resolution against a naive closure; the tree digest against a naive
  walk.

## 5. Named scenarios

Each is a focused test in the world named, and the referee's checks of
3 hold in every one; where the VM tier can cause the same, it runs the
same repository (`vm.md`, 3).

| Scenario | World | What happens | What must hold |
|---|---|---|---|
| a mount fails during a trial | slots, system | `test-mount-fails`: the trial's boot isolates `emergency.target` | the trial's units, armed from the `/run` marker or `hearth.trial=` and kept by `IgnoreOnIsolate=yes`, still end the trial; fell back |
| a failed unit | slots, system | `test-unit-fails`: the trial boots `degraded` | the slot's steward asks to fall back at once; the dead-man plays no part. With the unit on `bless.ignore_failed`, the bless passes |
| a cut in the bless | activator, system | a crash of the activator, and a power cut, between each step of the bless (hearth/docs/design/domain/slots/bless.md, section 1): journal, `blessed`, marker removed, default moved, dead-man stopped | a power cut boots the default, a blessed slot; after a crash the running slot is blessed but not the default, and the steward blesses it again, which finishes the order |
| both GRUB saves fail [M3b] | boot, system | GRUB cannot save `env.0` nor `env.1`, so `next_entry` stays set | the activator in the trial's slot clears it and reports it (hearth/docs/design/domain/machine/boot.md, 2.6); a trial that panics meanwhile is bounded by the boot counter; the default stays a blessed slot |
| a blessed default that panics [M3b] | boot, system | the default panics at every boot | the boot counter moves GRUB to the other blessed slot (hearth/docs/design/domain/machine/boot.md, section 2) |
| hostile tree shapes | activator, slots | an output whose `/var`, `/etc` or kernel is a symbolic link, or with a FIFO where `/etc/passwd` should be | prepare refuses each, since every slot path is resolved with `RESOLVE_BENEATH`, `RESOLVE_NO_SYMLINKS` and `RESOLVE_NO_XDEV` (`fake-machine.md`, 2); nothing outside the slot is read or written |
| a cut between walk and record | store, system | a power cut after a layer's walk and before its record; then, later, a published layer found corrupt (`fake-machine.md`, 10) | `syncfs` precedes the record (hearth/docs/design/domain/store/README.md, 4.2), so a recorded layer is whole; the startup scan deletes the staging layer (hearth/docs/design/domain/store/README.md, 4.4); a corrupt layer found later is deleted and rebuilt, not refused |
| `systemd-container` | build | the closure holds `systemd-container`, whose tmpfiles make `Q /var/lib/machines` | with the setup hook's directories, the layer holds a directory; without them, a nested subvolume, which the walk refuses (hearth/docs/design/domain/store/README.md, 4.1) |
| a snapshot bump that keeps the kernel | system | 3's soft-reboot rule | a soft reboot, then the bless |
| a reload that fails | slots, system | `test-reload-fails` in a live trial | the live fall back: files back, units reloaded again, `fell back (live)`, no reboot |
| a cut at every live phase | activator, system | a crash of the activator, and a power cut, at each step from L1 to L9 (hearth/docs/design/domain/slots/trial.md, 8) | before arming, `prepared`; after, `reboot -ff` and `fell back`; after the live bless, its trial at the next boot; the default always a blessed slot |
| a later live change that falls back | slots, system | `c` while `b` is live-blessed, its reload failing | the delta against `a`; the swap back to `b`'s delta, which no slot holds any more; the next boot runs `a` |
| a person's soft reboot while live | slots, system | a soft reboot with no next root, the seed drawing which overlays survive | the start runs `reboot -ff`; nothing runs a half-overlaid tree past it |
| an entry that changes class | slots, system | a revision moves `mail` from `data` to `core` | the place is refused, naming the orphaned directory in `data`; no empty `mail` is seeded in `core` |
| a cut while seeding | slots, system | a power cut while an entry's `<name>.new` is filled | the next prepare deletes the `.new` and seeds again; no half-seeded entry is ever bound |
| a snapshot across two services [M5] | backups, system | two writers in one class, snapshots and prunes cut at every step | each snapshot is both writers' state at one instant; retention keeps each bucket's newest |
| restoring one entry [M5] | backups, system | a restore of `mail` from a snapshot, cut between its renames | finished or undone at start; `postfix`, in the same class, untouched |
| a revision that crashes the builder | steward, system | every builder dies rendering one revision: scripted in the steward's world, killed by the referee at that job in the system world | the job lost, tried by a new builder, lost again: `crashed`; the pass waits for another commit, the crash a fact and an event naming the revision; the client's `status` answered throughout; the next commit converges (hearth/docs/design/domain/steward.md, 5.1) |
| a builder killed for memory | steward, system | `steward/builder/` reaches its ceiling in a packages step (`fake-machine.md`, 10): the step's leaf killed in one seed, the builder's own in another | the step killed: that step's failure (hearth/docs/design/domain/build/README.md, 4.3); the builder killed: a lost job, tried once more; nothing outside `steward/builder/` killed; the steward answers throughout |
| a builder that hangs | steward | the scripted builder never answers a build | at the job's deadline the steward stops the builder's cgroup; the job lost and tried again (hearth/docs/design/domain/steward.md, 5.2) |
| the bless's render fails | steward, system | in the new slot, the render of the slot's revision fails: a configuration error in one seed, the builder lost twice in another | the steward asks to fall back, never to bless; the default never moved; `a` boots, and `b`'s prepare key is refused (hearth/docs/design/domain/steward.md, section 6) |
| a release that renders another output | steward, system | the new slot's builder renders the slot's revision to another output key | the bless is asked and given; a `render changed` fact; the next pass builds that output and tries it |
| a client that stops reading | builder | many targets, with the client's output held until render has stopped | render waits at the watermark (hearth/docs/design/domain/render/README.md, 5.4): the peak stays under its ceiling, and once the output moves every record prints, in order, with the call's exit |
| a steward that dies with a builder running | activator, system | the steward killed in the middle of a build | the activator empties `steward/` with `cgroup.kill` before it spawns a new steward, so no builder outlives its steward; the next builder's startup scan settles the staging layer; the pass goes on (hearth/docs/design/domain/steward.md, 5.4) |
