# Containers and pool hosts

## 1. Container slots

- **On the host's top level:** `@<container>.a`, `@<container>.a.var`,
  `@<container>.b`, `@<container>.b.var`, and the classes its entries
  use, `@<container>.core`, `@<container>.data` and
  `@<container>.cache`, where the container's name is its target's
  unless a pool runs several of one target.
- **Its `/var` is per slot,** as a host's. The bash fleet bound a
  persistent `.var` into each container; what a container keeps across
  slots is a `data` entry of its tasks, a directory in one of its
  classes, as on a host. A container has no `journal` entry: its
  journal is its host's (hearth/docs/design/domain/operating.md, section 5).
- **Its classes follow its host's policies** [M5]: a container has no
  `hearth` task (hearth/docs/design/protocol/format/tasks.md, 12), so the host's activator snapshots its
  classes by the host's running slot's data policies, into the host's
  `@hearth.snapshots` under the container's target, and sends them to
  the backup peer of the container's environment [M6]
  (hearth/docs/design/domain/operating.md, section 7).
- **Prepared by the host's activator,** `prepare.md`'s steps, with these
  differences:
  - the shift writes ids from the container's own range, its
    `PrivateUsers=` base, 64 Ki aligned and clear of the builder's
    range (hearth/docs/design/domain/store/README.md, 7.2), instead of from 0 (or from 0, with idmapped
    mounts: 4); its class subvolumes are owned in the same ids,
    their root the container's root, and so are its entries'
    directories, seeded from the shifted slot;
  - `prepare.md`, 1 takes nothing out of a firmware's fallbacks, and `prepare.md`, 3 makes no
    mount point for hearth's state and store, which are never bound into
    a container (hearth/docs/design/domain/store/README.md, 7.4);
  - `prepare.md`, 5 writes its `/etc/fstab` with no line, and writes no dead-man
    unit and no watchdog drop-in: nspawn makes its root and binds the
    rest (hearth/docs/design/domain/machine/README.md, 3.3), and the host judges the trial (3);
  - `prepare.md`, 6 writes the host's nspawn settings instead of boot files (below),
    and `prepare.md`, 7 reads no boot file back;
  - the identity and secrets are the container's target's, made and
    kept by the host's activator, so a container no longer shares its
    host's machine-id.
- **Its parameters arrive in its `boot` part** (hearth/docs/design/protocol/format/README.md, 3.5;
  hearth/docs/design/domain/render/spec.md, 1.3): the id base, the binds, the capabilities and the
  network. Its slot record (`README.md`, 2.2) holds them where a host's holds a
  kernel and a command line, and its nspawn settings are written from
  them. They enter the prepare key as any slot part does, so a change to
  one prepares a new slot of the same output, which the host tries by a
  restart (2), since nspawn reads its settings only when it starts.
- **A new id base needs fresh data.** The shift writes the new slot in
  the new range, but the container's class subvolumes stay owned in the
  old one, which the old slot needs to fall back to. So the activator
  refuses a place whose base is not the owner of the roots of the
  container's existing class subvolumes, each made owned by the
  container's root (hearth/docs/design/domain/machine/README.md, 3.4), and names them. A person
  deletes them, and the container starts again from scratch, as every
  machine does (README, `bless.md`, section 1).
- **The host's nspawn settings,** written by the host's activator from
  the slot's parameters into `/run/systemd/nspawn/<container>.nspawn`:

  ```
  # /run/systemd/nspawn/chat-prod.nspawn, written by east-prod's activator for slot b (containers.md, 1)
  [Exec]
  PrivateUsers=327680
  LinkJournal=try-host
  Capability=<name>                                             one per capability
  [Files]
  ReadOnly=yes
  Bind=/run/hearth-activator/top/@chat-prod.b.var:/var
  Bind=/run/hearth-activator/top/@chat-prod.<class>/<name>:<path>  one per data entry, from its class
  BindReadOnly=<source>:<path>                                  one per bind; Bind= if not read-only
  [Network]
  VirtualEthernet=no                                            or Bridge=<bridge> for network veth [M6]
  ```

  A settings file in `/run/systemd/nspawn/` takes full effect, and
  `systemd-nspawn@.service` runs nspawn with `--settings=override`, so
  these lines win over the unit's own `--link-journal=try-guest` and
  `--network-veth`. `LinkJournal=try-host` keeps the container's journal
  in its host's (hearth/docs/design/domain/operating.md, section 5). Each entry's directory is
  bound at its path straight from its class, with no `/srv/<class>`
  mount inside the container; the ids are the container's own, so no
  `:idmap` is needed, but for a pool's idmapped roots (4). The root
  is found at `/run/machines/<container>`, a link to the slot's root
  (that nspawn's image search includes `/run/machines` is to verify). That
  top-level mount is `nodev,nosuid,noexec` (hearth/docs/design/domain/machine/README.md, 3.2); whether
  nspawn's bind of a root and of `/var` from it keeps those flags is to
  verify (`README.md`, section 7).

## 2. Activation: a restart, or live

A container's trial is live when the change is eligible (8), and
otherwise a restart: write the slot's settings and repoint the link,
each by rename over the old, and run `systemctl restart
systemd-nspawn@<container>.service`. There is no kernel, no soft reboot
and no one-shot. A change to the container's parameters is never live
(`trial.md`, 7, rule 2): nspawn reads them only when it starts.

- **The host's activator starts its containers.** At every start it
  makes each container's settings and link that `/run` lacks, from the
  default it recorded (6), then runs `systemctl start` on its unit,
  which leaves one already active alone. So a host reboot starts every
  container on its default, and an activator restart within a boot
  leaves them as they are.
- **No output enables them:** a host's output enables no
  `systemd-nspawn@` unit of its containers, and render refuses one that
  does (hearth/docs/design/protocol/format/tasks.md, 11). Started at boot before its link exists, it
  would fail.

## 3. Trial

- **The unit must be stably active:** two consecutive
  `systemctl is-active` answers of `active`, polled within 30 seconds;
  `failed` fails at once. This is the bash fleet's `south-host-control
  switch-container`, its only working trial and fall back.
- **Then the minimal criterion, read from the host:** the container's
  boot reached its default target with no failed unit but those its
  target's `bless.ignore_failed` lists (`bless.md`, 1.1; hearth/docs/design/protocol/format/tasks.md, 13), through
  `systemctl --machine=<container> is-system-running` and
  `list-units --state=failed`, as hearth/docs/design/domain/operating.md (2.9) reads a container's
  units. `bless.md`, 1.1's second condition, a steward of the slot's own and its
  bless's render, has no counterpart (5; hearth/docs/design/domain/steward.md, section 6).
- **The host's steward asks to bless** once the criterion holds, and the
  container's blessing checks too (hearth/docs/design/domain/operating.md, section 3), or to fall
  back on a failed unit that counts. The bless records the link's target
  as the container's default in the host's state (6).
- **The window is the host's activator's:** no dead-man runs in a
  container, so the activator falls the container back itself when the
  trial's window ends with no bless.
- **A held bless** is a host's (`bless.md`, 1.4), ended at its ceiling by the
  host's activator, as the window is.
- **Its trial and facts are its own,** never its host's: the host's
  minimal bless leaves out its containers' nspawn units (`bless.md`, 1.1), and a
  container that fails to start when its host boots is reported in its
  own facts, running none on its default, while the host's trial goes
  on.
- **On failure,** the host's activator points the link back and restarts
  the old slot.
- **A container's first slot** is tried with no other slot running, the
  one exception to `try`'s guard (`README.md`, section 4): a failed first trial
  leaves the container stopped, its slot `fell back`.
- **A host reboot mid-trial** falls back by itself: the links live in
  `/run`, the host's activator makes them again at start from the
  defaults it recorded, and records `fell back` from the marker of an
  older boot. An activator restart within the boot keeps the trial, its
  window counted from the marker's `armed at` (`trial.md`, 4).

## 4. Pool hosts

- **One build, then a snapshot per container** (`hearth/docs/design/draft/archive/factory.md`,
  6.5): a lab pool runs many containers of one output.
- **The lean:** shift once, into a base subvolume the host's activator
  keeps owned from 0; snapshot that base for each container; and let
  nspawn map ids with idmapped mounts (`PrivateUsersOwnership=map`). One
  shift then serves every container. The alternative shifts each
  container's snapshot into its own range, one walk each.
- **VMs with a virtiofs root [later]** have the same shape: their slots
  live on the pool host, and a trial stops the VM, repoints its virtiofs
  devices and starts it, judged from the host (hearth/docs/design/domain/machine/README.md, 2.6).

## 5. A hearth in the container

- **The lean, open:** none in M5. The host's hearth prepares, tries and
  judges its containers from outside, runs their checks through the host
  (hearth/docs/design/domain/operating.md, section 2), and reports their facts as its own.
- **What one would bring:** the container's own bless, as hosts have, and
  checks run inside it. What it would lack: an activator with anything
  to own, since the container has no boot and its root is the host's to
  place. This depends on how hearth/docs/design/domain/operating.md's checks reach a container,
  and on whether a pool's containers need to report to the lab's engine
  directly.

## 6. The facts and the state machine

A container's facts are its host's activator's, read from outside:

| Fact | Read from |
|---|---|
| **running** | the subvolume of the container's root, by uuid, while its unit is active (through its leader's root, to verify how); `none` otherwise |
| **default** | the slot the host's state records as the container's default (hearth/docs/design/domain/activator/README.md, 6.1), which its link names at every host start |
| **trial** | the trial marker, `trial/<container>.<label>` (`trial.md`, 4) |
| **live** | the container's leader's mount table, `/proc/<leader>/mountinfo`, against the container's live record, `live/<container>` (8) |

- **`README.md`, section 4's states, table and guards hold,** with a container's
  events: `try` is 2's restart; `try live` and `bless live` are
  8's; the unit stably active on the slot is `start here`, and on
  the other slot `start elsewhere`; a host start, or a restart of the
  container, is a new boot; `bless` comes from the host's steward
  (3), not from one running in the slot; `first bless` never happens
  (3's first slot). A `live-blessed` slot's one-shot is the host's
  record that the container's next start is its trial (8).
- **The invariant reads:** the slot the host links at start, and the one
  nspawn runs outside a trial, is blessed; a link to a slot on trial
  lasts until its bless, its fall back or the host's next start.
- **One mutating operation at a time** (hearth/docs/design/domain/activator/README.md, 4.2) holds across
  the host and its containers, so a host prepares and tries one target
  at a time (hearth/docs/design/protocol/standalone.md, 3).
- **What does not apply:** boot files and entries (`prepare.md`, 6), the dead-man
  and the watchdog (`prepare.md`, 5; the window is the activator's, 3), the
  kernel's rule (`trial.md`, 1), and the BIOS boot counter (`bless.md`, 2.4).

## 7. A container removed from its host's spec

A container leaves its host's spec when its target is deleted from the
repository or placed on another host. The host's steward then finds,
in the activator's facts, a container whose target its spec no longer
names (`target.containers`, hearth/docs/design/protocol/format/README.md, 3.5), and asks its activator to
remove it (hearth/docs/design/domain/activator/operations.md, 16):

1. **Stop it:** `systemctl stop systemd-nspawn@<container>.service`;
   remove its link in `/run/machines` and its settings in
   `/run/systemd/nspawn/`.
2. **Delete its slots:** each slot's subvolumes, nested ones first
   (`prepare.md`, 1), then their records, manifests and any marker, then the
   container's own record, so no host start links it again.
3. **Keep its data:** its class subvolumes and their snapshots stay,
   unmounted and reported as orphaned as any target's data is
   (hearth/docs/design/domain/machine/README.md, 3.4), until a person deletes them; its identity and
   secrets stay with them.

- **Its outputs are then unpinned:** no slot holds them, so the store
  may collect them (hearth/docs/design/domain/store/README.md, section 5).
- **Each step is idempotent,** and a removal cut anywhere is finished
  by asking again; a container already gone is done.
- **A container placed on another host starts there from scratch:**
  its new host prepares a first slot with no data and makes its
  identity anew; nothing moves between hosts.

## 8. Live changes in a container

A container gets live changes with the same mechanics (`trial.md`, 6 to 9), run
by its host's activator from outside, since no hearth runs inside:

- **Eligibility** is `trial.md`, 7's, read inside the container: the never-live
  list (hearth/docs/design/protocol/format/tasks.md, 14) applies to its units, the same as a host's;
  rule 10 reads the host's kernel and the container's own facts (6);
  rule 11 reads the mount namespaces of the container's processes.
- **The delta** is built from the container's new slot, in the
  container's ids, as `@<container>.live.<n>` on the host's top level.
- **The swap enters the container's mount namespace.** A short-lived
  child of the host's activator builds each overlay in the host's
  namespace, where the layers' paths resolve, then `setns` into the
  mount namespace of the container's leader (`/proc/<leader>/ns/mnt`),
  moves the overlay beneath `/etc`, `/usr/local` or `/opt` there as L5
  does, and reads the container's mount table back. The activator itself
  never leaves the host's namespace.
- **The units' actions** go through `systemctl --machine=<container>`
  (`stop`, `daemon-reload`, `reload`, `restart`, `reload-or-restart`,
  `start`, `show`), and tmpfiles through `systemd-run
  --machine=<container> --wait`; there is no steward inside to spawn
  anew. The host's steward has a builder plan the change
  (hearth/docs/design/domain/steward.md, 3.3), waits out the settle and asks the live bless
  (3).
- **The dead-man is the host's:** the window is the activator's deadline
  in the marker, as for any container trial (3). At its end, or when
  a step of the live fall back fails, the activator restarts the
  container on its default slot: the container's `reboot -ff`. A restart
  drops the overlays, which live only in the container's mount
  namespace.
- **The live bless records "the next start of this container is `b`'s
  trial"** in the host's state (hearth/docs/design/domain/activator/README.md, 6.1): the container's
  one-shot. The next start the host's activator makes, at a host boot or
  a restart it asks for, links `b` and tries it as 3 does; that
  bless records `b` as the default and deletes the delta.
- **A restart nobody asked for** (the unit restarting on its own, a
  person's `systemctl restart`) starts the linked slot, the base, and
  drops the live state; the activator finds it at its next reading of
  the facts and makes the `live-blessed` slot `prepared` again, as for a
  one-shot a firmware ignored (`README.md`, section 4).
- **A host reboot** ends every container's live state with the
  container, as it ends their trials.
- **Pool hosts** (4) whose containers' roots are idmapped mounts
  would need idmapped layers under the overlay: to verify.
