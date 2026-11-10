# brainstorm: status

2026-11-05. Where the design lags or departs from [brainstorm.md](brainstorm.md).

Section 13's settled questions are carried by the design: a layer includes `/var`,
and prepare splits off the slot's writable copy, with persistent data declared
separately (`hearth/docs/design/domain/store/README.md`, section 4;
`hearth/docs/design/domain/slots/prepare.md`, 3); hearth's state is the filesystem alone
(`hearth/docs/design/domain/store/README.md`, section 6;
`hearth/docs/design/domain/activator/README.md`, section 6). EFI uses direct stub entries,
BIOS uses GRUB with its environment pair on FAT, and tiny-initramfs stays
(`hearth/docs/design/domain/machine/boot.md`, sections 1, 2 and 4). Secure Boot and
firmware that forgets its variables remain that document's open questions
(`brainstorm.md`, section 12), rather than alternatives to the ordinary boot paths.

The template subset is `hearth/docs/design/protocol/format/templates.md`; a
task's schema is its `task.json`'s `settings`, with types in `brainstorm.md`, section 6 of that
document. Exports are declared and read by task and never cross an environment
(`brainstorm.md`, section 8); clusters declare a `members` map and name each target
`<base>-<member>-<env>` (3.6). These settle `brainstorm.md`, section 13's template, schema, exports
and cluster questions.

Section 13's artifact recipe, sandbox, compile caches and output storage are
`hearth/docs/design/domain/build/artifacts.md`, sections 2 to 6. Peers' change sets,
verification and HTTP sources, including pool caches, are
`hearth/docs/design/domain/store/README.md`, section 8 and
`hearth/docs/design/protocol/wire.md`, section 7. The activator's operations and
verification that an output is in its builder's store are
`hearth/docs/design/domain/activator/operations.md` and `hearth/docs/design/domain/activator/README.md`, section 5. The choices those
documents still leave open are recorded below where they leave the draft's
question unsettled.

- **Design lags:**
  - 5.2: the pool's engine placing containers, rather than repository placement,
    has no complete design yet. `hearth/docs/design/protocol/format/README.md`, section
    11 leaves how placement and parameters reach the host to M5's pool design;
    `hearth/docs/design/domain/slots/containers.md`, 4 gives the shared-output shape but
    `../domain/slots/README.md`, section 7 leaves naming several containers of one target undecided.
- **Design departs:**
  - 1, 4.1, 4.2, 4.4, 5.3 and 10.2: three processes replace the draft's pair.
    The persistent steward owns convergence, the client and engine faces,
    checks and telemetry; its builder owns git, render, builds and the store.
    A builder crash loses a job, retried once, while the steward can still
    report it; the bless also requires rendering the slot's own revision.
    `hearth/docs/design/domain/steward.md`, sections 2 to 6 carry the split.
    Decided with the user's input, 2026-10-14
    (`hearth/docs/design/decisions.md`).
  - 4.1: the activator starts its steward and hears only the pipe pair it
    creates, rather than listening to its builder on a local socket
    (`hearth/docs/design/domain/activator/README.md`, sections 3 and 4). This confines
    the privileged face to its own child. The design's own decision
    (`hearth/docs/design/decisions.md`).
  - 4.1: the privilege boundary confines operations and protects secrets; it
    does not make an unreviewed output safe to activate. A compromised builder
    or steward can place an output whose activator is root at the next trial,
    so review and the lab must keep bad outputs out
    (`hearth/docs/design/domain/activator/README.md`, 2.6). This qualifies the draft's
    claim that the activator runs nothing from IaC. The design's own decision
    (`hearth/docs/design/decisions.md`).
  - 4.1 and 4.6: layers belong on disk to the builder's subordinate ids;
    digests use namespace ids. Prepare shifts the snapshot's owners, modes,
    capabilities and ACLs to real ids, so digests agree across machines while
    the builder stays unprivileged (`hearth/docs/design/domain/slots/prepare.md`, 2).
    Decided with the user's input, 2026-10-09
    (`hearth/docs/design/decisions.md`). The tree helper, as namespace
    root, snapshots, freezes, walks, hashes, applies and empties layers because
    the builder cannot read all their files or change a subvolume owned by
    namespace root (`hearth/docs/design/domain/store/tree-helper.md`, 3 and 8). The
    helper and the default range, 262144 with 65536 ids, clear of containers'
    ranges (9.2), are the design's own decisions.
  - 4.1 and 11.4: each build step has its own sandbox; mmdebstrap's chroot
    alone does not contain unreviewed code. All build steps are offline, with
    packages taken from a local repository made from the resolution's kept
    indices and fetched packages. Only the separate, digest-checked fetch
    through `apt-helper` reaches the network
    (`hearth/docs/design/domain/build/packages.md`, 1 to 3, and `hearth/docs/design/domain/build/sandboxes.md`, 1 to 5;
    `hearth/docs/design/domain/build/reproducible.md`, 5.2). This also replaces the
    draft's later exception for the packages layer to use an apt proxy.
    The design's own decisions (`hearth/docs/design/decisions.md`).
  - 4.3: root tests run on Incus VMs with their own disk and OVMF until the lab
    exists, outside the gate (`hearth/docs/design/testing/README.md`, 2.8 and `hearth/docs/design/testing/vm.md`
    ). The draft reserves these tests to the lab; the VM tier lets
    slots and boots be tested before it exists. The design's own decision
    (`hearth/docs/design/decisions.md`).
  - 4.4 and 10.3: agents may explore throwaway development machines by hand,
    capture their changes and apply working trees as revisions. Mutable roots
    need explicit bootstrap acceptance; the run side never takes them. Every
    apply prepares a fresh slot, discarding changes not encoded in IaC
    (`hearth/docs/design/domain/development.md`, sections 2 to 7). This gives
    agents a development loop despite the draft's blanket rule of reads only
    and no hand on a machine. The operational shell stays local and read-only
    (`hearth/docs/design/client/README.md`, 3.7). Decided with the user's input,
    2026-10-09 (`hearth/docs/design/decisions.md`).
  - 4.6: btrfs and subordinate ids are prerequisites of layers alone. A store
    that only renders can be a directory on any filesystem, so M1's render and
    diff need neither (`hearth/docs/design/domain/store/README.md`, 2.4). The
    milestone split is the design's own (`hearth/docs/design/README.md`,
    section 3).
  - 5.1 and 13: standalone mode follows a desired commit or a polled branch,
    set locally, with automatic or manual trials; a manual machine waits for
    its user before any trial. A machine follows an engine or converges alone,
    never both (`hearth/docs/design/protocol/standalone.md`, 1;
    `hearth/docs/design/client/README.md`, 3.1 and 3.8). This replaces a revision
    pinned only in configuration and lets a workstation choose when it changes.
    The design's own decisions (`hearth/docs/design/decisions.md`).
  - 5.1: prepare's durable key is the output, slot parts and current identity
    and secret generations, with the trial window from M5; refusals name that
    key, not an output alone. A corrupt layer is deleted and rebuilt, and a
    client can clear a refusal (`hearth/docs/design/protocol/standalone.md`, 2 and
    4). Target and revision alone cannot distinguish secret rotation or a
    slot-only change. Configuration errors wait for a changed input,
    operational failures back off, and failed outputs are refused, so a
    deterministic failure cannot start a retry or boot loop. The design's own decisions
    (`hearth/docs/design/decisions.md`).
  - 5.2 and 9: M5's containers run no hearth. Their host builds, prepares,
    tries and blesses them, with a trial of their own that never fails the
    host's bless. The host reads their minimal criterion from outside and
    keeps their identity, secrets and per-slot `/var`; container parameters
    are placement fields in their `boot` part, so changing them prepares a
    new slot of the same output (`hearth/docs/design/domain/slots/bless.md`, 1.1 and 1.2 and `hearth/docs/design/domain/slots/containers.md`
    ; `hearth/docs/design/protocol/format/README.md`, 3.5, and `hearth/docs/design/protocol/format/tasks.md`, 12).
    This keeps privileged ownership on the host and avoids an activator with
    nothing to own inside a container. The design's own decisions
    (`hearth/docs/design/decisions.md`); whether a container should
    ever run hearth remains open (`hearth/docs/design/domain/slots/containers.md`, 5).
  - 6.1, 7.9 and 9: no template is completed at activation. The activator
    writes fstab, slot records and machine-id from typed data, and supplies the
    filesystem UUID to fstab and `root=` at prepare. This keeps device identity
    out of shared outputs and templates out of the privileged process
    (`hearth/docs/design/domain/slots/prepare.md`, 5;
    `hearth/docs/design/domain/machine/boot.md`, section 3). The design's own
    decisions (`hearth/docs/design/decisions.md`).
  - 6.1 and 7.9: task declarations, including `requires` and `files`, are in
    `task.json`, replacing `etc.json`'s `require` and file rules. `tree/`
    mirrors the root; only `.j2` files are templates, and trailing newlines
    stay. Static files can therefore hold arbitrary bytes and Jinja-like
    text without evaluation (`hearth/docs/design/protocol/format/README.md`, 2.1 and 2.3, and `hearth/docs/design/protocol/format/tasks.md`, 1, and `hearth/docs/design/protocol/format/templates.md`, 2, and `hearth/docs/design/protocol/format/README.md`, 10
    ). These are the design's own choices in
    carrying the draft's real directory trees and inert templates further.
  - 6.2: the last layer is `config`, because it also writes users,
    `/usr/local`, unit links and other paths, beyond the draft's `etc`
    (`hearth/docs/design/domain/build/README.md`, section 3). The design's own
    decision (`hearth/docs/design/decisions.md`).
  - 6.2 and 13: a layer includes `/var` throughout building; prepare copies
    it into the slot's own writable subvolume and seals the root, so dpkg's
    database travels with its slot while declared data outlives slots
    (`hearth/docs/design/domain/store/README.md`, section 4;
    `hearth/docs/design/domain/slots/prepare.md`, 3). This settles the draft's
    root-and-`/var` pairing rather than splitting at the packages phase.
    Decided with the user's input, 2026-10-09
    (`hearth/docs/design/decisions.md`).
  - 6.2: secrets and identity are files written at prepare from the
    activator's root-only store, with paths, owners and modes declared by
    tasks, rather than delivered by a separate mount or systemd credentials.
    Rotation prepares another slot of the same output
    (`hearth/docs/design/protocol/format/README.md`, section 7;
    `hearth/docs/design/domain/slots/prepare.md`, 4). This keeps outputs shareable
    while the privileged side alone supplies unique and secret bytes.
    Decided with the user's input, 2026-10-09
    (`hearth/docs/design/decisions.md`). M3 generates SSH host keys
    with `ssh-keygen`; WireGuard identity waits for in-process generation to
    avoid requiring a tool available only in the slot being prepared. That
    restriction is the design's own decision.
  - 6.2 and 12: configurations are written again and machines installed
    fresh by bootstrap, with new identity; there is no converter or migration
    of os's data or machines. os's golden cases inspire rewritten fixtures,
    whose expected output is hearth's, rather than an os oracle: os omits
    modes and strips final newlines (`hearth/docs/design/domain/render/README.md`,
    6.7; `hearth/docs/design/domain/machine/bootstrap.md`). The design's own
    decisions (`hearth/docs/design/decisions.md`).
  - 7.7: a container names its host in placement; a host's container map is
    derived, with no second list in its configuration. This gives placement
    one source of truth (`hearth/docs/design/protocol/format/README.md`, 3.5).
    The design's own decision (`hearth/docs/design/decisions.md`).
  - 7.8: "built from this head" names a recipe in the revision being
    rendered. The spec holds that recipe and its sources' listing digest;
    the build plan resolves the artifact key before the software layer,
    rather than the lab supplying a digest before render. This preserves
    render as a function of revision and target
    (`hearth/docs/design/domain/build/artifacts.md`, 7.2). This is the design's own
    choice; a product head with IaC in another repository needs a pin change.
  - 7.9: above packages, `/usr` may be written only under `/usr/local`;
    hearth's binary is `/usr/local/bin/hearth`, its units and generated files
    are in `/etc`. Render refuses other task paths in `/usr`, and build checks
    scripts too, keeping Debian's `/usr` apart from live overlays
    (`hearth/docs/design/protocol/format/tasks.md`, 5 and 12;
    `hearth/docs/design/domain/build/README.md`, 7.3). Decided with the user's input,
    2026-10-16 (`hearth/docs/design/decisions.md`).
  - 7.9: package-created users and groups are declared too, and every owner
    of data, secrets or identity has a fixed id; build seeds package accounts
    before installation, rather than leaving all account work to the config
    layer. This prevents a rebuild renumbering persistent owners
    (`hearth/docs/design/protocol/format/tasks.md`, 6;
    `hearth/docs/design/domain/build/packages.md`, 5). The design's own decisions
    (`hearth/docs/design/decisions.md`).
  - 7.9: render reserves the activator's files, command-line keys and hearth
    units, and adds the `hearth` task implicitly except on containers.
    Prepare resolves paths beneath its snapshot without following links.
    The format therefore has a reserved task and paths despite replacing
    os's magic task names with settings: every host output must carry hearth,
    and IaC cannot replace the privileged side's files or redirect its writes
    (`hearth/docs/design/protocol/format/tasks.md`, 11 and 12;
    `hearth/docs/design/domain/activator/README.md`, 2.3 and section 5). The design's
    own decisions (`hearth/docs/design/decisions.md`).
  - 8 and 9: eligible config and software changes run live, through the
    activator's atomic overlay swap over `/etc`, `/usr/local` and `/opt`, with
    declared consumers. A live bless arms the prepared slot's next native
    trial; it never moves the boot default. This avoids reboots for changes
    that need none while retaining a native boot before commitment
    (`hearth/docs/design/domain/slots/trial.md`, 6 to 9, and `hearth/docs/design/domain/slots/bless.md`, 1.6).
    Decided with the user's input, 2026-10-09
    (`hearth/docs/design/decisions.md`). Accordingly the facts are
    running, default, trial and live; running is read by the root subvolume's
    UUID, cross-checked with its slot record, rather than parsed from the
    mount table (`hearth/docs/design/domain/slots/README.md`, section 3). That
    observation method is the design's own decision, for an exact identity.
  - 8 and 9: soft reboot eligibility compares the kernel image, modules,
    initramfs inputs and command line without slot tokens, rather than the
    kernel and initramfs bytes alone. Non-reproducible initramfs bytes would
    force unnecessary full boots, while changed modules or command-line
    options cannot safely keep the old kernel
    (`hearth/docs/design/domain/slots/trial.md`, 1). The design's own decision
    (`hearth/docs/design/decisions.md`).
  - 9 and 13: EFI uses direct kernel-stub entries, not systemd-boot or UKIs;
    BIOS uses GRUB with a FAT boot partition and two environment files, so a
    one-shot can be consumed and a torn write cannot lose the default
    (`hearth/docs/design/domain/machine/boot.md`, sections 1 and 2). Decided with
    the user's input, 2026-10-09 (`hearth/docs/design/decisions.md`).
    Secure Boot is off because a Debian kernel cannot boot by the direct
    stub path under it; tiny-initramfs stays, explicitly configured by
    placement (`hearth/docs/design/domain/machine/boot.md`, sections 4 and 5).
    Those are the design's own decisions.
  - 9: trials have `panic=N`, watchdog coverage where available and a
    dead-man timer that survives emergency isolation, armed from `/run` or
    the trial command-line token. EFI has normal and trial entries. These
    make a trial end even when hearth cannot ask to fall back
    (`hearth/docs/design/domain/slots/prepare.md` and `hearth/docs/design/domain/slots/trial.md`). The minimal
    bless before task checks exist accepts only expected failed units and
    requires the steward and its builder's render; bless records the slot as
    blessed before moving the default, so a cut can be finished by re-blessing
    (`brainstorm.md`, section 7). The design's own decisions
    (`hearth/docs/design/decisions.md`).
  - 9: an engine's held bless has a ceiling ended by a timer independent of
    hearth, so a canary hold cannot make a trial permanent
    (`hearth/docs/design/domain/slots/bless.md`, 1.4;
    `hearth/docs/design/domain/activator/operations.md`, 13). The design's own decision
    (`hearth/docs/design/decisions.md`).
  - 10.2 and 10.4: data entries are directories in the target's `core`,
    `data` and `cache` class subvolumes, bound at declared paths; snapshots
    and backups apply by class, never to `cache`. A class snapshot is atomic
    across its services; an entry restore copies its directory, a class
    restore swaps its subvolume (`hearth/docs/design/protocol/format/tasks.md`,
    7 and 12; `hearth/docs/design/domain/operating.md`, section 7).
    This retains the fleet's layout and gives policies a class as their unit,
    rather than one subvolume per entry. Decided with the user's input,
    2026-10-09 (`hearth/docs/design/decisions.md`). An entry is seeded
    once from the output, so its bind hides no initial package data
    (`hearth/docs/design/domain/slots/prepare.md`, 3); that is the design's own
    decision.
  - 10.3, 10.4 and 12: local snapshots and restores are M5, before M6's
    off-machine backups, so restore is already the third operational effect
    rather than waiting for backups (`hearth/docs/design/domain/operating.md`,
    sections 6 and 7). M1 has the encoding and a store without layers; M2
    adds layers and builds; M3's standalone EFI machine is the MVP, before
    M3b's BIOS and Incus driver, M4's wire, M5's operating and containers,
    and M6's artifacts, peers, backups and faults. Tests grow in every
    milestone; os parity is not required for the MVP
    (`hearth/docs/design/README.md`, section 3). The milestone split is the
    design's own, making each step usable without waiting for the whole vision.
  - 13: an artifact's build environment is a container target with no host,
    used as a build sandbox's root; it is never an nspawn container or a slot.
    This shares the rootfs build machinery without requiring a host to run
    the environment (`hearth/docs/design/domain/build/artifacts.md`, 2.3 and
    section 5). The design's own decision
    (`hearth/docs/design/decisions.md`).
- **Open:**
  - 4.3 and 13: the fake machine starts in hearth; its generic half is to
    move to a skein kit when temper needs it, while hearth's half stays
    (`hearth/docs/design/testing/fake-machine.md`, 11). The move's timing and the
    kit's name remain open (`brainstorm.md`, section 11).
  - 4.3 and 13: tier ceilings are specified provisionally
    (`hearth/docs/design/testing/README.md`, section 4): the focused suite's
    30 CPU seconds are the user's, and the fuzzy suite's CPU budget and the
    conformance tier's budget and place after landing remain open
    (section 11). The mirror has a pin, requests and a fetch path (6.2);
    whether those requests are right, its size and retention between runs
    still need measurements and a decision (section 11).
  - 5.3 and 13: machine identity toward the engine and the plane's secret
    store, where its credential lives and how secrets reach the activator
    still await the factory's registration and secret-store design
    (`hearth/docs/design/protocol/wire.md`, 4.3 and section 10). The builder's
    repository read credential is separately imported at bootstrap and kept
    in its store, never the root-only secret store
    (`hearth/docs/design/domain/render/README.md`, 2.2); whether an engine issues it
    later is also open (`hearth/docs/design/protocol/wire.md`, section 10).
  - 7.1 and 13: who posts the rendered diff and how a many-target change is
    presented remain open (`hearth/docs/design/domain/render/README.md`, 4.4 and
    section 7).
  - 10.1 and 13: checks have a format and remote probes select their source
    targets by task role (`hearth/docs/design/domain/operating.md`, 2.1, 2.2
    and 2.6). What production checks may write or send still awaits policy;
    until then they only read, and no probe sends a message for delivery
    (2.8 and section 10).
  - 10.3 and 13: operational effects are roll back, restart a unit and restore
    data (`hearth/docs/design/domain/operating.md`, section 6). Any further
    effect waits for a need and a policy (section 10).
  - 10.4 and 13: backup periods and bucket retention are task data, and
    streams are encrypted to the peer before reaching a steward
    (`hearth/docs/design/domain/operating.md`, 7.1 to 7.3). Where peers live,
    how many a target has and whether they keep plaintext subvolumes or
    ciphertext chains remain open (section 10;
    `hearth/docs/design/protocol/format/README.md`, section 11).
  - 13: disk provisioning as an effect is M6, conditional on a blank disk and
    keyed by its supplied identifiers (`hearth/docs/design/domain/machine/bootstrap.md`,
    5). How the lab names the disks it orders still awaits the pool design
    (`brainstorm.md`, section 12).
  - 13: the activator checks an output's store path, record, read-only flag,
    namespace owners and snapshot, then hashes its contents again in M3
    (`hearth/docs/design/domain/activator/README.md`, 5.1 to 5.3). Whether it may
    trust recorded content digests when only landed code runs remains open,
    as does restricting placement to known hearth binaries (`brainstorm.md`, section 10).
