# hearth's design

Provisional, 2026-10-09, revised 2026-10-19. The design of hearth, the
factory's build daemon (`hearth/docs/design/draft/archive/factory.md`, section 7): what it
makes, how it makes it, how a machine runs what it made, and how all of
it is tested. It takes `hearth/docs/design/draft/brainstorm.md` further and holds where the
two differ (`decisions.md`). The mechanics are those of
`skein/docs/foundation/programming-model.md`; the tests follow `skein/docs/foundation/testing-strategy.md`. This
document is the overview: the reading order, the milestones, the
structure, the names, the conventions, the decisions taken, what hearth
asks of others and what is open. Each part has a document of its own
beside this one.

## 1. In one page

- **hearth makes outputs from inputs, keyed by digest, and runs them in
  slots.** One daemon on every machine: hosts, the lab's pool hosts and
  its builders. It replaces os's `manifest`, `deploy`, `provision` and
  `ek`.
- **The IaC format is os's, rebuilt for agents and digests**
  (hearth/docs/design/protocol/format/README.md): strict JSON, inert Jinja templates, tasks that declare
  their settings, merging that can remove, exports between targets.
  Every input is in the repository.
- **Render is a pure function of a revision and a target** (hearth/docs/design/domain/render/README.md).
  It runs in full in milliseconds and gives a spec: parts, each with a
  digest. Any hearth renders any target, and the rendered diff of a
  change is what its reviewers read.
- **Outputs are layers** (hearth/docs/design/domain/store/README.md, hearth/docs/design/domain/build/README.md): packages, then software,
  then config, each a read-only btrfs subvolume named by its cumulative
  key. A change rebuilds from the lowest layer whose key it changed, on
  a snapshot of the layer beneath.
- **Three processes: two split by privilege, two by failure**
  (hearth/docs/design/domain/activator/README.md, hearth/docs/design/domain/steward.md). The activator, root and small, owns the
  machine's slots, boot state and secrets, and does a closed list of
  operations. It starts the steward and listens to nothing else. The
  steward, unprivileged and persistent, decides and faces: converge,
  the client, the engine, checks. It spawns the builder, unprivileged
  too, for work: render, builds, the store. A builder that crashes ends
  a job, never the process that reports it.
- **Immutable roots.** A slot's root (`/usr`, `/etc` and the rest) is
  mounted read-only. Only the slot's own `/var` and the data its tasks
  declare are writable: a directory per entry in one of the target's
  three class subvolumes, `core`, `data` and `cache`, bound at the
  entry's path and snapshotted by class. Nothing on a running machine
  is changed in place: a fix is a new slot, prepared beside the running
  one. Its configuration and in-house software may still reach the
  running machine with no reboot, as a delta of the new slot mounted
  over the old (the live changes below; hearth/docs/design/domain/slots/trial.md, 6). A development
  machine may be mutable instead (hearth/docs/design/domain/development.md, section 4).
- **Slots try before they commit** (hearth/docs/design/domain/slots/README.md): prepare, trial, bless,
  fall back. A new slot runs once; the boot default moves only when the
  slot's own hearth blesses it (a container's, its host's), and any
  failure during the trial ends in a reboot into the old default.
- **Configuration and in-house software change live** (hearth/docs/design/domain/slots/trial.md, 6):
  when a change touches only `/etc`, `/usr/local` and `/opt` over the
  same packages, and every changed path has a declared consumer, the
  activator mounts a delta of the new slot over the running one with an
  atomic swap and reloads or restarts the declared units, with no
  reboot; a failure puts the old files back. Its bless arms the new
  slot's trial for the next boot, so the default still moves only after
  a native boot.
- **Nothing unique and nothing secret in an output,** so outputs can be
  shared between machines. Identity and secrets are written into a slot
  when it is prepared, from a root-only store on the machine.
- **The engine decides, hearth carries out** (hearth/docs/design/protocol/wire.md). With an engine,
  hearth prepares and activates on orders and reports facts. With none,
  it converges to a desired revision on its own: standalone mode, which
  is the MVP's.
- **Built on skein,** following `skein/docs/foundation/programming-model.md`: one loop, sans-io
  step functions, a domain complete enough to run whole on a fake
  machine. The heavy work runs in child processes: the builder, git,
  mmdebstrap, efibootmgr, and hearth's own tree helper.
- **Tested from day 0** (hearth/docs/design/testing/README.md): on a fake machine, with a realistic
  fake for every tool, a conformance tier that holds the fakes to the
  real tools, real builds as a regular user, and VMs for what needs root
  until the lab exists.
- **Agents develop on throwaway machines** (hearth/docs/design/domain/development.md): Incus VMs
  cloned in seconds, roots they may change by hand, a capture of what
  they changed, and their working tree applied as a revision. Builds
  from scratch stay the lab's.
- **Built in milestones** (section 3): render, then builds, then slots
  on one EFI machine in standalone mode (the MVP), then BIOS, the wire,
  operating, and the rest of the vision.

## 2. Reading order

1. **hearth/docs/design/README.md:** this overview.
2. **hearth/docs/design/protocol/format/README.md:** the IaC format: the repository, targets, merging,
   tasks and their schemas, templates, exports, and secrets and data as
   the format names them.
3. **hearth/docs/design/domain/render/README.md:** from a revision and a target to a spec: the pipeline,
   parts and digests, package resolution, the rendered diff.
4. **hearth/docs/design/domain/store/README.md:** the store on btrfs: digests and keys, layers, the tree
   helper, records, pins and collection, hearth's own state.
5. **hearth/docs/design/domain/build/README.md:** how an output is made: the plan, the three layers,
   sandboxes, the tools as protocols, caches, and what an output must not
   hold.
6. **hearth/docs/design/domain/machine/README.md:** what a machine looks like: disks and partitions, the
   btrfs top level, how each kind of machine boots, the initramfs,
   bootstrap.
7. **hearth/docs/design/domain/slots/README.md:** a slot's life: the four facts, prepare, trial (live
   changes included), bless, fall back, roll back, drift, containers.
8. **hearth/docs/design/domain/activator/README.md:** the privilege boundary: the activator and its
   steward, the link, the activator's operations and what it verifies.
9. **hearth/docs/design/domain/steward.md:** the unprivileged side: the steward and the builder it
   spawns for work, the jobs between them, failures, the bless's render.
10. **hearth/docs/design/protocol/wire.md:** hearth's faces: standalone mode, the wire to an
    engine, bytes by digest; and **hearth/docs/design/client/README.md**, the command-line
    client, with the client's other designs under `client/`.
11. **hearth/docs/design/domain/operating.md:** after a rollout: checks, facts, telemetry,
    operational effects, snapshots and backups.
12. **hearth/docs/design/testing/README.md:** how `skein/docs/foundation/testing-strategy.md` applies to hearth: the fake
    machine, the fakes of tools, the tiers and their budgets.
13. **hearth/docs/design/domain/development.md:** how an agent develops a machine: development
    machines on Incus, mutable roots, capture, applying a working tree.
14. **hearth/docs/design/domain/build/artifacts.md:** the second recipe: an artifact from a build
    environment, sources and a command.
15. **hearth/docs/design/domain/build/reproducible.md:** reproducible builds, deferred: what is kept from
    day one and how they are retrofitted.

`hearth/docs/design/draft/brainstorm.md` is the record of the first discussion. These
documents take it further, and hold where the two differ.

## 3. Milestones

Each milestone leaves something usable on its own. Each document says
in its first paragraph which milestone builds it, and marks the sections
a later one builds (`[M5]` after the heading). The tests grow within
every milestone, from the first (hearth/docs/design/testing/README.md, section 5).

| | Milestone | Gives | Builds |
|---|---|---|---|
| M1 | Render | specs with digests; the rendered diff of any change | hearth/docs/design/protocol/format/README.md, but for checks and artifacts; hearth/docs/design/domain/render/README.md; the client's render and diff (hearth/docs/design/client/README.md); hearth/docs/design/domain/store/README.md's encoding and a store without layers (2.4) |
| M2 | Build | outputs keyed by digest, built unprivileged, layer by layer; a built root boots in a VM | hearth/docs/design/domain/store/README.md, but for peers; hearth/docs/design/domain/build/README.md; the tree helper; the boot test (hearth/docs/design/testing/real-builds.md, 6) |
| M3 | Slots, standalone, on EFI: **the MVP** | one real EFI machine runs on hearth, installed from scratch | hearth/docs/design/domain/machine/README.md's EFI path and bootstrap; hearth/docs/design/domain/slots/README.md, but for containers; hearth/docs/design/domain/activator/README.md; hearth/docs/design/domain/steward.md, but for peers and artifacts; hearth/docs/design/client/README.md's client and hearth/docs/design/protocol/wire.md's standalone mode; a minimal bless; hearth/docs/design/domain/development.md's mutable machines, capture and working trees |
| M3b | BIOS; development machines | BIOS machines, which most IaaS providers offer; development machines on Incus | hearth/docs/design/domain/machine/README.md's BIOS path; hearth/docs/design/domain/slots/README.md's trial through GRUB; hearth/docs/design/domain/development.md's Incus driver |
| M4 | The wire | hearth as a system behind an engine | hearth/docs/design/protocol/wire.md's wire, facts and events |
| M5 | Operating | the machine's end of the run side | hearth/docs/design/domain/operating.md, but for backups; checks as task data; local snapshots of each class, and restores from them (hearth/docs/design/domain/operating.md, section 7); containers and pool hosts (hearth/docs/design/domain/slots/containers.md); clusters and containers in development (hearth/docs/design/domain/development.md) |
| M6 | The rest | the full vision | hearth/docs/design/domain/build/artifacts.md; peers; backups; faults; disks as an effect; development VMs from the host's store (hearth/docs/design/domain/development.md, section 9) |
| later | Reproducible builds | a level of trust | hearth/docs/design/domain/build/reproducible.md |

**The MVP needs no parity with os.** M3 leaves out on purpose: BIOS,
containers, the wire, task checks (a minimal bless stands in,
hearth/docs/design/domain/slots/bless.md, section 1), peers, artifacts and backups.

**What it must get right anyway,** because changing it later is costly:

- outputs named by key, each recording its tree digest, in the ids seen
  inside the builder's namespace (hearth/docs/design/domain/store/README.md);
- nothing unique and nothing secret in an output (hearth/docs/design/domain/build/README.md, section 7);
- render a pure function of the revision and the target, exports
  included (hearth/docs/design/domain/render/README.md);
- the slots' four facts and their state machine (hearth/docs/design/domain/slots/README.md);
- every effect keyed or conditional, even while the client is the only
  one to ask (hearth/docs/design/protocol/wire.md);
- the privilege split, from the first slot (hearth/docs/design/domain/activator/README.md), and the
  steward apart from its builder (hearth/docs/design/domain/steward.md).

## 4. Structure

### 4.1 Processes

```
a machine
├── hearth activator          root, started by systemd: the machine's slots, boot state, secrets
│   └── hearth steward        user hearth; persistent; started by the activator: converge, the faces, checks
│       └── hearth builder    user hearth, with a subordinate id range; started by the steward for work
│           ├── tools         git, apt-helper, mmdebstrap, dpkg-query: each build step in a sandbox of its own
│           └── tree helper   in the builder's user namespace: snapshots, freezes, walks, hashes, applies, empties
└── hearth (the client)       any user: talks to the steward over a local socket, or runs in-process
```

- **The link** joins the activator and its steward: a pipe pair made
  when the activator spawns the steward, so the activator hears no one
  else (hearth/docs/design/domain/activator/README.md, section 3).
- **The jobs** join the steward and its builder: the builder's standard
  input and output. The steward spawns a builder when it has work and
  lets it exit once idle, so a crash in render or a build loses a job,
  not the machine's face (hearth/docs/design/domain/steward.md).
- **The steward never reads the repository, runs a build tool or opens
  a layer;** whatever IaC steers runs in the builder or beneath it
  (hearth/docs/design/domain/steward.md, 2.2).
- **cgroups are delegated:** the activator's unit has `Delegate=yes`; the
  activator moves itself into a leaf and gives the steward a subtree of
  its own, in which the builder gets one with its own memory ceiling,
  for its sandboxes (hearth/docs/design/domain/steward.md, section 7).
- **A builder alone** serves tests, the lab's builders and a developer's
  machine: rendering, diffing and building need neither a steward, an
  activator nor root.
- **The client runs the builder's domain in-process** when no daemon
  runs, on a developer's machine (M1, M2), and whenever it renders a
  local repository: the daemon renders only its own clone
  (hearth/docs/design/client/README.md, section 3).
- **Bootstrap runs the activator's code in-process,** as root from a
  live system, and it spawns a steward over a link as the daemon does
  (hearth/docs/design/domain/machine/bootstrap.md).

### 4.2 Crates

Crates are named by their layer (`skein/docs/foundation/programming-model.md`, section 4):
`hearth-domain-*`, `hearth-protocol-*`, `hearth-service-*`; the
client's are `hearth-client-*` (below). The crate graph is the role
graph.

```
lib                           no_std, skein-lib only
hearth-lib                    the canonical encoding and H (domain/store/README.md, 3.1 and 3.2)  M1

domain                        no_std, skein-lib, hearth-lib and their own children only
hearth-domain-builder         the builder's root: routes between its children; jobs (domain/steward.md)
├── hearth-domain-render      configuration to spec (protocol/format.md, domain/render.md)  M1
├── hearth-domain-store       layers, keys, records, pins, collection (domain/store.md)   M1, M2
├── hearth-domain-build       plans and steps (domain/build.md)                           M2
└── hearth-domain-artifacts   the second recipe (domain/artifacts.md)                     M6
hearth-domain-steward         the steward's root: routes between its children; orders, facts, jobs  M3
├── hearth-domain-converge    the desired revision, orders, a slot's decisions (protocol/wire.md)  M3
└── hearth-domain-checks      checks, signals, windows (domain/operating.md)              M5
hearth-domain-activator       the activator's root (domain/activator.md)                  M3
├── hearth-domain-slots       slot state, prepare, trial, bless, fall back, drift         M3
├── hearth-domain-boot        boot entries and files: EFI, then GRUB (domain/machine.md)  M3
├── hearth-domain-disk        partitions, filesystems, the top level (domain/machine.md)  M3
├── hearth-domain-backups     class snapshots, restores; from M6, streams (domain/operating.md)  M5
└── hearth-domain-faults      a pool host's faults (domain/operating.md)                  M6
hearth-domain-tree            walks, digests, change sets: the tree helper's, and the shift's (domain/store.md)  M2

protocol
hearth-protocol-config        strict JSON to typed configuration and schemas              M1
hearth-protocol-template      the template language's parser (protocol/format/templates.md)  M1
hearth-protocol-git           git's argument vectors and output                           M1
hearth-protocol-mmdebstrap    mmdebstrap: simulate, bootstrap, progress, failure tails    M1
hearth-protocol-dpkg          dpkg-query and apt                                          M2
hearth-protocol-tree          the tree helper's records; the store's records and tree files  M1, M2
hearth-protocol-accounts      passwd, group and shadow lines                              M2
hearth-protocol-tail          a tool's output tail, kept for a failure                    M2
hearth-protocol-xattr         file capabilities and POSIX ACLs                            M2
hearth-protocol-link          the steward–activator records, both ways                    M3
hearth-protocol-jobs          the steward–builder records, both ways (domain/steward.md, 3.2)  M3
hearth-protocol-state         the activator's records, the trial marker, the slot record  M3
hearth-protocol-units         fstab, the trial's units, the watchdog's drop-in           M3
hearth-protocol-efi           EFI variables from efivarfs; efibootmgr's argument vectors  M3
hearth-protocol-systemd       systemctl, systemd-run and the journal's queries            M3
hearth-protocol-proc          /proc/<pid>/mountinfo and cgroup: mounts, overlays, units   M3
hearth-protocol-keygen        ssh-keygen and the other identity tools                     M3
hearth-protocol-grub          grub-editenv, GRUB's configuration and env                  M3b
hearth-protocol-wire          the wire to an engine, over skein's channel                 M4
hearth-protocol-probe         probes of a service as its users see it (domain/operating.md, 2.5) M5
hearth-protocol-http          bytes by digest                                             M6
hearth-protocol-builder       the builder's protocol layer: routes its machines           M1
hearth-protocol-steward       the steward's protocol layer: routes its machines           M3
hearth-protocol-activator     the activator's protocol layer: routes its machines         M3
hearth-protocol-helper        the tree helper's protocol layer: routes its machines       M2

client                        what exists for the client, under one prefix, each crate held to its layer's rules (client/README.md, section 2)
hearth-client-protocol-records  the client's records and their printer                    M1, M3
hearth-client-domain-dev      the client's development loop; the Incus driver from M3b (domain/development.md)  M3
hearth-client-protocol-incus  incus's argument vectors and JSON output (domain/development.md, 8.3)  M3b

service and shell
hearth-service-builder        the builder's iterate                                       M1
hearth-service-steward        the steward's iterate                                       M3
hearth-service-activator      the activator's iterate                                     M3
hearth-service-tree           the tree helper's iterate                                   M2
hearth                        the binary: each process, and the client

testing
hearth-fake-machine           the fake machine (testing/fake-machine.md)             M1
hearth-fake-<tool>            one fake per tool, acting on the fake machine               M1
hearth-fake-engine            an engine at the wire's far end (domain/testing.md)         M4
hearth-client-fake-incus      incus's fake, for the client's development loop             M3b
hearth-system-world           the builder's and the tree helper's domains; the steward's and the activator's from M3     M2
hearth-<name>-world           the other worlds, under tests/
hearth-client-world           the client's world, under tests/client (client/README.md, section 4)  M3
hearth-client-dev-world       the development loop's world, under tests/dev (domain/development.md, section 10)  M3b
```

- **Three crates start before their milestone.** M1 builds what render
  needs of them, and the later milestone grows each: `hearth-domain-store`
  holds resolutions and cache files' records, over a store without layers
  (hearth/docs/design/domain/store/README.md, 2.4); `hearth-protocol-tree` decodes the frame and records of
  kinds 1, 4 and 5 (hearth/docs/design/domain/store/records.md, 1); `hearth-client-protocol-records`
  holds the records of `render` and `diff` and prints them (hearth/docs/design/domain/render/README.md,
  5.1; hearth/docs/design/client/README.md, 3.2).
- **Client crates live under one prefix,** as jig's do (
  `jig/docs/design/README.md`, section 15). A crate that exists for the client is
  `hearth-client-{domain,protocol,<kind>}-<name>`: after the prefix
  comes its layer, as for the daemon's crates, or its kind (a test kit's
  `-fake-`, a world's name ending in `-world`), so that everything the
  client alone needs lives under `hearth-client`. It is the client's
  domain, its protocol (the records it speaks, even where the steward
  reads them too), its tools' protocols, its fakes and its worlds. A
  crate the client hosts but the daemon runs too keeps the daemon's
  name: the builder's domain and protocol layer, `hearth-protocol-git`,
  the activator's crates for the root-only commands. The binary keeps
  the project's name alone, `hearth`, and composes the client; the
  prefix's base, `hearth-client`, is kept for a crate that composes it
  apart from the binary, should another program need one. The client's
  designs live under `client/` (hearth/docs/design/client/README.md).
- **`hearth-lib` is hearth's one crate in the lib role:** the canonical
  encoding and `H` are needed by domains in three processes, which may
  depend only on lib and their own children (`skein/docs/foundation/programming-model.md`,
  4.5). It moves into skein's lib if another service needs it.
- **The builder's children,** and the steward's, are siblings that
  share no types; each root translates between them
  (`skein/docs/foundation/programming-model.md`, 4.5).
- **Every format is its own machine crate,** lib-only and fuzzed alone;
  a process's protocol layer only routes between its machines
  (`skein/docs/foundation/programming-model.md`, section 4; hearth/docs/design/domain/activator/README.md, 7.1). JSON's machine
  is skein's tokenizer, which `hearth-protocol-config` composes
  (hearth/docs/design/domain/render/README.md, 5.1).
- **The activator's domain never depends on the steward's, nor the
  steward's on the builder's.** The link's records are the only
  vocabulary the first two share, through `hearth-protocol-link`; the
  jobs' are the only one the last two share, through
  `hearth-protocol-jobs`. The client hosts the builder's domain
  in-process (hearth/docs/design/client/README.md, 3.5), and needs no steward for it.
- **Drift is the activator's** (hearth/docs/design/domain/slots/bless.md, section 4): it wrote the slot
  record and alone can read every file of a slot.
- **`hearth-domain-tree` has two parents in two processes:** the tree
  helper's service, and the activator's slots domain, whose shift walks
  every inode and computes the slot's tree digest as it goes.

## 5. Names

| Name | What it is |
|---|---|
| IaC repository | a system's declarative inputs: fleet defaults, environments, bases, tasks (hearth/docs/design/protocol/format/README.md) |
| revision | a commit of it |
| base | a system described once, `hosts/<base>/<base>.json`: the host's own data |
| environment | `envs/<env>.json`: defaults for every target in it, such as `lab` or `prod` |
| target | a base in an environment, such as `mail-prod`: what is rendered and built |
| placement | where a target runs: bare metal, a VM, a container on a named target; data of its environment |
| machine | where a target runs: a disk with slots, or a container's root on a host |
| development machine | a throwaway standalone machine an agent changes and applies its working tree to: an Incus VM cloned from a base image [M3b], which may be mutable (hearth/docs/design/domain/development.md) |
| mutable | a placement whose slots are not sealed and whose root is mounted read-write, for changes by hand; taken only by a machine bootstrapped to accept it, never by the run side (hearth/docs/design/domain/development.md, section 4) |
| task | everything about one concern: settings, files, packages, software, data, secrets, checks, exports |
| setting, schema | a value a task reads; the task's declaration of its settings: names, types, defaults, a line each |
| export | a fact a task publishes to the other targets of its environment |
| spec | what a target is at a revision, rendered: parts with digests |
| part | a piece of a spec with its own digest: meta, packages, software, config, data, secrets, boot, checks, exports, live |
| resolution | the pinned package closure a packages part resolves to, remembered by its key |
| digest | SHA-256 over a canonical encoding |
| key | what a layer or a resolution is named by: the digest of its inputs; a layer's is cumulative |
| layer | a read-only btrfs subvolume, named by its key: packages, software or config |
| output | a target's top layer: a build of its spec |
| tree digest | the digest of a tree as seen inside the builder's namespace: paths, types, modes, owners, extended attributes, contents, link targets |
| change set | what a layer changed from its parent: changed files and deletions |
| store | the builder's layers, records, resolutions and downloads (hearth/docs/design/domain/store/README.md) |
| pin | what keeps a layer from collection |
| tree helper | hearth's child process inside the builder's user namespace that walks, hashes, applies and empties trees |
| sandbox | a build step's own namespaces, pivoted into its layer (hearth/docs/design/domain/build/sandboxes.md) |
| slot | where a machine runs an output: a root and its own `/var`, labelled `a` or `b` |
| slot record | what a slot holds and how it was prepared: its output, its digests, its own tree digest after the shift |
| shift | rewriting a placed tree's owners from the builder's id range to real ids |
| class | a kind of persistent data, `core`, `data` or `cache`, with one subvolume per target, `@<target>.<class>`, mounted at `/srv/<class>`, and a snapshot and backup policy of its own (hearth/docs/design/protocol/format/tasks.md, 7) |
| data entry | a target's data that outlives its slots: a directory in its class's subvolume, bound at the path a task declares |
| snapshot, backup | a read-only snapshot of a class, kept on the machine by its policy [M5]; one sent to the backup peer, encrypted [M6] (hearth/docs/design/domain/operating.md, section 7) |
| secret store | root-only, on the machine: the target's secrets and the machine's identity |
| identity | what makes a machine itself: its machine-id, its host keys |
| running, default, trial, live | the four facts: the slot running, the boot default, the slot on trial, and the live state over the running slot (hearth/docs/design/domain/slots/README.md, section 3); a container's, read by its host (hearth/docs/design/domain/slots/containers.md, 6) |
| live change, delta, live bless | a trial with no reboot: a delta of the new slot (`@<target>.live.<n>`) mounted over the running slot's `/etc`, `/usr/local` and `/opt`, the declared units acted on; its bless arms the new slot's native trial (hearth/docs/design/domain/slots/trial.md, 6, and hearth/docs/design/domain/slots/bless.md, 1.6) |
| prepare, trial, bless, fall back, roll back | a slot's transitions (hearth/docs/design/domain/slots/README.md, section 4) |
| boot partition | the FAT partition the firmware or GRUB writes: the ESP on EFI, GRUB's env on BIOS (hearth/docs/design/domain/machine/README.md) |
| activator, steward, builder | the three processes: root and closed; persistent, deciding and facing; spawned for work (hearth/docs/design/domain/activator/README.md, hearth/docs/design/domain/steward.md) |
| link, jobs | the pipe pairs between the activator and its steward, and between the steward and its builder |
| client | the command-line face (hearth/docs/design/client/README.md, section 3) |
| engine | what orders a machine: the run side's or the lab's (`hearth/docs/design/draft/archive/factory.md`) |
| plane | the lab or the run side: outputs and credentials never cross between them |
| standalone | hearth with no engine, converging to its desired revision |
| desired revision | what a machine converges to: ordered by its engine, or set locally |
| order | an engine's request: prepare, activate, bless (releasing a held one), roll back, restart a unit; later, build an artifact |
| fact, event | what hearth reports: state, read afresh; what happened, as a hint |
| check | what healthy means for what a task installs, as data (hearth/docs/design/domain/operating.md) |
| drift | a slot that differs from its record |
| costs | hearth's own, as facts: time per step, bytes fetched, cache hits |

## 6. Conventions

- **A document is cited by its path from the repository root:**
  `hearth/docs/design/domain/store/README.md`; with a section,
  `hearth/docs/design/domain/store/README.md, section 4` or
  `(hearth/docs/design/protocol/standalone.md, 2)`. Every document names the
  others this way, whatever directory it is in.
- **Other trees' documents** are cited the same way:
  `skein/docs/foundation/programming-model.md`,
  `skein/docs/design/net/channel.md`, `jig/docs/design/domain/connectors.md`.
- **Each document** says in one page what it is, then its parts, what it
  owes and is owed below the domain, its world, what it keeps or changes
  from os, and what is open.
- **Milestones:** a document's first paragraph names the milestone that
  builds it; a section a later milestone builds says so after its
  heading, as `[M5]`.
- **Code cites a section** at module or type level, as
  `hearth/docs/design/domain/slots/prepare.md, 2`, never on every field.
- **Examples** use the fleet's targets (`x1-prod`, `mail-prod`,
  `east-prod`) and the factory's (`mail-lab`, `stoa-1-lab`).

## 7. What hearth asks of others

### 7.1 skein

Each of these is generic, so it belongs in skein with hearth as its first
user (`skein/docs/foundation/testing-strategy.md`, section 9). From the earliest:

| Need | For | Milestone |
|---|---|---|
| a public, streaming SHA-256 in lib, usable by the domain | digests | M1 |
| JSON: each of the tokenizer's tokens and errors with its position in the document, its byte offset, line and column (the column in bytes, both from 1). `hearth-protocol-config` reads every document through skein-json's tokenizer, and refuses duplicate keys itself, at every depth, as it decodes each object's keys | strict configuration (hearth/docs/design/protocol/format/README.md, 2.2); the file, line and column of every render error (hearth/docs/design/domain/render/README.md, 3.1 and hearth/docs/design/domain/render/errors.md; hearth/docs/design/protocol/format/README.md, 5.5) | M1 |
| a full stat by name, without following links; readlink; extended attributes (`skein/docs/design/runtime/kernel.md`, 6.1 and 6.4; `skein/docs/design/runtime/io/files.md`) | the tree helper's walk | M2 |
| writing a tree's metadata: symlink, link, chmod, chown, extended attributes, mkdir, truncate (`skein/docs/design/runtime/kernel.md`, 6.4) | change sets | M2 |
| setting an entry's times, without following links (`utimensat`) | a slot's `/var`, copied (hearth/docs/design/domain/slots/prepare.md, 3); later, the walk's clamp (hearth/docs/design/domain/build/reproducible.md, 5.1) | M3 |
| spawning as another user, with supplementary groups; signalling a process group | the steward under the activator, the builder under the steward; stopping a tool | M2 |
| contained process trees, with id maps through `newuidmap`, a composed root and `pivot_root`, network and uts namespaces, fresh `/proc`, `/sys` and `/dev`, `no_new_privs` and limits (hearth/docs/design/domain/build/sandboxes.md, 8) | sandboxes | M2 |
| a closed list of synchronous ioctls in the ring adapter: `GET_SUBVOL_INFO`, `GET_SUBVOL_ROOTREF`, `INO_LOOKUP_USER`, `SUBVOL_CREATE`, `SNAP_CREATE_V2`, `SNAP_DESTROY_V2`, `SUBVOL_SETFLAGS` (hearth/docs/design/domain/store/tree-helper.md, 8; `skein/docs/design/runtime/kernel.md`, 6.5) | the store and the slots, unprivileged where they can be | M2 |
| a digest of a world's state, for replay (hearth/docs/design/testing/worlds.md, 4) | replay checks in hearth's worlds | M1 |
| an exclusive lock on a directory, taken without waiting, which the owner asks for again on its own timer (`skein/docs/design/runtime/kernel.md`, 6.1; `skein/docs/design/runtime/io/files.md`) | the store's lock (hearth/docs/design/domain/store/README.md, 6.2, step 1) | M1 |
| a rename from one directory to another (`skein/docs/design/runtime/io/files.md`) | a resolution's indices moved from its scratch directory into the cache (hearth/docs/design/domain/render/spec.md, 2.3; hearth/docs/design/domain/store/records.md, 4) | M1 |
| a directory listed page by page (`skein/docs/design/runtime/io/files.md`) | the startup scan of `cache/` and `resolutions/`, whatever their size (hearth/docs/design/domain/store/README.md, 6.2) | M1 |
| `statfs`'s filesystem type (`skein/docs/design/runtime/kernel.md`, 6.5) | the scan's subvolume steps, on btrfs alone (hearth/docs/design/domain/store/README.md, 2.4) | M2 |
| Unix sockets with peer credentials | the client | M3 |
| redialling and identity on the channel's TLS client | the wire | M4 |
| `syncfs` (`skein/docs/design/runtime/kernel.md`, 6.5) | a layer durable at its freeze, before its record (hearth/docs/design/domain/store/README.md, 4.2) | M2 |
| `openat2` with `RESOLVE_BENEATH`, `RESOLVE_NO_SYMLINKS` and `RESOLVE_NO_XDEV`; `FICLONE` | prepare resolving and copying a slot's paths safely | M3 |
| the new mount API: `fsopen`, `fsconfig`, `fsmount`, `move_mount` with `MOVE_MOUNT_BENEATH`, `open_tree`, `mount_setattr`; `umount2` with `MNT_DETACH`; walking a detached mount through its descriptor | live changes: overlays built aside, verified, swapped beneath and back (hearth/docs/design/domain/slots/trial.md, 8) | M3 |
| `mknodat` of a 0/0 character device | a live delta's whiteouts | M3 |
| `setns` into another process's mount namespace, from a child | a container's live swap (hearth/docs/design/domain/slots/containers.md, 8) | M5 |
| cgroup v2: making, delegating and moving into subtrees, and spawning into one; `cgroup.kill` | sandboxes' limits; the steward's and the builder's subtrees | M2 |
| generating Ed25519 and X25519 keys in-process | identity without tools from a slot | later |
| HTTP ranges and file-backed bodies | bytes by digest | M6 |

### 7.2 temper

- **The fake machine is temper's too** (`temper/docs/design/domain/testing.md`, 3.2).
  hearth builds it first, under `testing/`. Its generic half (files,
  processes, programs on an argument-vector face) moves into skein as a
  kit when temper needs it; hearth's half (btrfs, namespaces, firmware,
  the package universe) stays as `hearth-fake-machine`
  (hearth/docs/design/testing/fake-machine.md, 11).

### 7.3 The factory

- **An infrastructure connector** speaking hearth's wire, in jig's terms
  (`jig/docs/design/domain/connectors.md`), for M4.
- **A plane's secret store, and machine identity** toward it and toward
  the engine (hearth/docs/design/protocol/wire.md, 4.3), for M4.

## 8. Open questions

Each document lists its own. The ones that reach across:

- machine identity, toward the engine and the plane's secret store
  (hearth/docs/design/protocol/wire.md);
- who posts the rendered diff at a change's head, and how (hearth/docs/design/domain/render/README.md);
- the checks' format, and what a check may do on a production machine
  (hearth/docs/design/domain/operating.md);
- signing kernels for Secure Boot (hearth/docs/design/domain/machine/README.md);
- firmware that discards its variables, as provider C's does (hearth/docs/design/domain/machine/README.md);
- whether the activator hashes every output it places again, or trusts
  the builder's records where the builder runs only landed code
  (hearth/docs/design/domain/activator/README.md);
- one generation for the whole secret store, so any import prepares
  every target's slot again, or one per target (hearth/docs/design/domain/activator/README.md);
- whether a container runs a hearth of its own (hearth/docs/design/domain/slots/containers.md, 5);
- live changes on a real machine: real ids, PID 1, D-Bus and AppArmor
  around a swap, overlays across soft reboots, `BootNext` kept for days
  (hearth/docs/design/domain/slots/README.md, section 7; hearth/docs/design/testing/vm.md, 3);
- hearth's own release applied live, once live changes are proven
  (hearth/docs/design/domain/slots/README.md, section 7);
- whether a manual machine applies live changes at once and waits only
  for reboots (hearth/docs/design/protocol/wire.md);
- a blessed EFI default that later stops booting panic-loops, since the
  firmware falls through only when a load fails: keeping the previous
  slot first in `BootOrder` and arming `BootNext` at each healthy start
  is one answer (hearth/docs/design/domain/slots/README.md); BIOS has a boot counter (hearth/docs/design/domain/machine/boot.md, 2);
- whether the activator should place only hearth binaries whose digests
  it holds itself (hearth/docs/design/domain/activator/README.md);
- the fake machine's final home (hearth/docs/design/testing/README.md);
- the local mirror for the real tier: which packages, pinned how,
  fetched by whom (hearth/docs/design/testing/README.md).
