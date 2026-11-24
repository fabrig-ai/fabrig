# The IaC format: targets

## 1. Targets

### 1.1 Bases, environments and targets

- **A target is a base in an environment.** For each `hosts/<base>/` and
  each `<base>-<env>.json` beside it whose `envs/<env>.json` exists, there
  is one target `<base>-<env>`, or one per member of a cluster (1.6). The
  file may be `{}`. A base alone is not a target.
- **Names are made, never parsed.** Render knows each target's base,
  member and environment from the files that made it, and never splits a
  name on `-` (os inferred a target's kind and host from a hyphen). Two
  targets with the same name are an error: base `stoa-1` in `lab` and
  member `1` of base `stoa` in `lab` both make `stoa-1-lab`.

| Target | Base | Environment | Placement |
|---|---|---|---|
| `x1-prod` | x1 | prod | metal, efi, 7th-gen |
| `east-prod` | east | prod | vm, bios, provider-a |
| `west-prod` | west | prod | vm, bios, provider-b |
| `mail-prod` | mail | prod | container on `east-prod` |
| `mail-lab` | mail | lab | vm, efi, incus; a container of a lab pool from M5 |
| `chat-prod` | chat | prod | container on `east-prod` |
| `stoa-1-lab` | stoa, member 1 | lab | the lab's |

### 1.2 Composition order

```
task defaults                      the defaults of every task the target has (section 6)
fleet.json                         defaults for every target
hosts/<base>/<base>.json           the base, without its members key
  members.<member>                   the member, in a cluster (3.6)
envs/<env>.json                    defaults for every target in <env>
hosts/<base>/<base>-<env>.json     the base in <env>, without its members key
  members.<member>                   the member in <env>
```

- **Each layer is merged onto the result of those above it** (`README.md`, section 4),
  so a later layer wins.
- **The environment comes after the base,** so its defaults (the lab's
  certificate authority, its DNS overrides) apply to every base in it;
  the base in the environment comes last, for what belongs to one target
  alone. This is hearth/docs/design/draft/brainstorm.md's order (`templates.md`, 3), with task defaults
  and members added.

### 1.3 Which tasks a target has

- **A base, a member and a base-env layer decide presence.** A key naming
  a task adds it: `"ssh": {}` adds ssh with all its defaults, and
  `"ssh": {"allow": ["alice"]}` adds it with a setting. `"wifi": null`
  removes it.
- **`fleet.json` and `envs/` never add or remove a task.** A task's key
  there sets settings for the targets that have the task; `null` for a
  task there is an error. This is what "defaults" means, and it lets
  `fleet.json` hold the SSH port without putting an SSH server on every
  target.
- **A target's tasks** are the keys left after merging its base, member
  and base-env layers alone, and `hearth`, which every target but a
  container has (`tasks-hearth.md`, 1). **Its settings** are the merge of every layer,
  restricted to those tasks.
- **Every task named anywhere must exist** under `tasks/`, and its
  settings are validated even where no target has it, so a typo in a
  default fails. A layer's value of a task that none of the targets it
  composes has (`envs/lab.json` setting wifi, which only prod targets
  have; a base with no target) is checked alone, as a patch: each name
  declared, and each value but `null` of its type and form. What only a
  composed value shows is not checked there: a required setting, and
  whether a `target` names a target of the environment. Such an error
  names its task and no target (hearth/docs/design/domain/render/errors.md, 1).
- **Reserved keys** of a document name no task: `debian` (`tasks.md`, 3),
  `placement` (1.4), `members` (1.6) and `_note`.

### 1.4 Placement

`hosts/west/west-prod.json`:

```json
{ "placement": { "kind": "vm", "firmware": "bios", "hardware": "provider-b" } }
```

| Field | Type | Meaning |
|---|---|---|
| `kind` | enum `metal`, `vm`, `container` | where the target runs |
| `firmware` | enum `efi`, `bios` | how it boots (hearth/docs/design/domain/machine/boot.md, sections 1 and 2); never for a container |
| `hardware` | string | the profile tasks key hardware data by: `7th-gen`, `incus`, `provider-a`, `provider-b`, `provider-c`; never for a container |
| `on` | target | for a container only: its host, a target of the same environment that is not a container; absent for a build environment, which no host runs (hearth/docs/design/domain/build/artifacts.md, 2.3) [M6] |
| `ids` | int | for a container on a host [M5]: the base of its 64 Ki id range, nspawn's `PrivateUsers=` (1.5) |
| `binds` | map | for a container on a host [M5]: host paths bound into it, keyed by the path inside, each `{ "source": path, "read_only": bool }`, `read_only` true by default |
| `capabilities` | set of names | for a container on a host [M5]: capabilities added to nspawn's default set, such as `CAP_NET_ADMIN`; none by default |
| `network` | enum `host`, `veth` | for a container on a host [M5]: `host` by default, the host's own network; `veth`, a veth pair whose host end joins `bridge` [M6] |
| `bridge` | string | for `network: veth` only: the host's bridge [M6] |
| `mutable` | bool | `false` by default; `true` leaves the target's slots unsealed and their root read-write, for changes by hand on a development machine (hearth/docs/design/domain/development.md, section 4); a container's too [M5]; never a build environment's |

- **Placement is data of the environment** (hearth/docs/design/draft/brainstorm.md, 7.7). It
  may appear in `envs/<env>.json` and in a base-env layer or its members,
  never in `fleet.json` or a base: a base describes a system apart from
  where it runs. Every target must have a placement once composed.
- **A placement is whole where it is written:** `kind` always; `firmware`
  and `hardware` for metal and a VM, which boot by one and are keyed by
  the other (`tasks.md`, 4; hearth/docs/design/domain/machine/README.md, 2.7); `on` for a container until M6's
  build environments. One missing is an error at the placement's `{`,
  and a field the kind never takes is one at its key.
- **`placement` and `members` are objects:** `null` for either is an
  error at the value, never a removal. A placement is replaced, never
  cleared, and a base-env removes a member by its own key (1.6).
- **Placement is replaced whole,** the one exception to RFC 7386 (`README.md`, 4.1):
  a layer that sets `placement` replaces it entirely, so `envs/lab.json`'s
  `firmware` and `hardware` never leak into a lab target that its own
  file places in a container.
- **What placement decides by itself:** the bootstrap variant
  (hearth/docs/design/domain/render/spec.md, 1.2), and that a container has no kernel, so its initramfs
  modules and boot parameters must be empty: its packages part has no
  initramfs, its boot part holds its container parameters instead of a
  command line (1.5), and render refuses a module or a parameter on it
  (hearth/docs/design/domain/render/errors.md, 2). The `boot` task, which installs the kernel, refuses a
  container by a guard of its own (`tasks.md`, 4).
- **`mutable` is a slot's, never an output's.** Render puts it in the
  `boot` part (hearth/docs/design/domain/render/spec.md, 1.3), so it enters the prepare key and no
  layer key: a mutable target builds the output its sealed twin would.
  Render accepts it in every layer placement may be in, on metal and a
  VM, and on a container from M5 (`README.md`, 2.2): before M5 a container's
  `mutable` is refused at its key, as its `ids` is, since a container in
  development is M5's (hearth/docs/design/domain/development.md, 3.4). Whether a machine takes
  it is the machine's configuration (hearth/docs/design/domain/machine/bootstrap.md, 1), and the run
  side's engine never orders one [M4] (hearth/docs/design/domain/development.md, 4.3). An
  environment such as `dev` sets it for all its targets.
- **Everything else that differs by placement is task data** that reads
  `target.placement`: the gettys masked in containers, the initramfs
  modules per hardware profile, EFI's tools (`tasks.md`, 10).

### 1.5 Containers on a host

- **A container names its host** in `placement.on`. A host's containers
  are the targets of its environment placed on it; render hands them to
  the host's templates as `target.containers` (`templates.md`, 7), a map from target
  name to placement. A container reads no more of its host than that
  placement, so what else fails on its host leaves it rendering
  (hearth/docs/design/domain/render/README.md, 3.4).
- **One source of truth.** The host keeps no list of its containers,
  where hearth/docs/design/draft/brainstorm.md (`templates.md`, 7) kept one in the host's configuration.
  The bash fleet's `runner:600` user, declared once for carbon and again
  for its container, shows what a second copy invites.
- **The same environment only.** `chat-prod` runs on `east-prod`; a lab
  container runs on a lab host. So the bash fleet's error, a container
  file that cannot vary by environment and so carries production's forge
  host key in the lab, cannot be written. The container is the target
  `chat-prod`, with `hosts/chat/chat-prod.json`:

  ```json
  { "placement": { "kind": "container", "on": "east-prod" } }
  ```

- **What a container's target holds** [M5]: tasks like any target's,
  but no `hearth` task (`tasks-hearth.md`, 1) and nothing that needs a kernel (1.4); the
  gettys masked by the `core` task (`tasks.md`, 10). Its `data` entries, identity
  and secrets are its own target's, kept by its host's activator: its
  class subvolumes sit on the host's top level, and its machine-id is no
  longer its host's (hearth/docs/design/domain/slots/containers.md, 1).
- **Users are not mirrored** [M5]. The bash fleet declared
  a container's `runner:600` on its host too, with a home inside the
  container's `/srv`. A container's entries are bound into it alone,
  from its own class subvolumes, and never mounted on its host
  (hearth/docs/design/domain/slots/containers.md, 1), so its host needs no such user; a host task that
  must know a container's user reads it from the container's exports
  (`exports.md`, section 1).
- **A container's parameters are placement fields** [M5] (1.4): its id
  base, its binds, its capabilities and its network. A target's file
  sets them with the rest of its placement:

  ```json
  { "placement": { "kind": "container", "on": "east-prod", "ids": 327680 } }
  ```

  - **Render puts them in the container's `boot` part,** a slot part,
    which for a container holds them instead of a command line
    (hearth/docs/design/domain/render/spec.md, 1.3). So a change to one changes the slot's prepare key
    and prepares a new slot of the same output, which its host tries by
    a restart, since nspawn reads them only when it starts
    (hearth/docs/design/domain/slots/containers.md, 1 and 2).
  - **Render checks their form:** a base aligned to 64 Ki and clear of
    the builder's range (hearth/docs/design/domain/store/README.md, 7.2) and of the other containers'
    placed on the same host; binds whose two paths are absolute and
    normal; capability names from the kernel's list. The host's
    activator checks them again (hearth/docs/design/domain/activator/README.md, 2.2).
  - **They are refused on anything but a container on a host:** a host
    and a build environment have none.
- **How a host runs a container** (its slots, the shift into its ids,
  its nspawn settings) is hearth/docs/design/domain/slots/containers.md [M5].
- **A container's journal is its host's** [M5]: nspawn links it into
  the host's journal entry, `/srv/core/journal`
  (hearth/docs/design/domain/operating.md, section 5), so the `core` task declares no `journal`
  entry on a container (`tasks.md`, 7).

### 1.6 Clusters

- **A base declares its members** under `members`, a map keyed by member
  name. Each member's value is a layer of its own (1.2): tasks and their
  settings, and `debian`; `placement` in a base-env's member alone, since
  a base's member is the base's (1.4); never `members`.
  `hosts/stoa/stoa.json`:

  ```json
  {
    "core": { "hostname": "stoa" },
    "stoa": { "port": 7000 },
    "members": {
      "1": { "stoa": { "index": 1 } },
      "2": { "stoa": { "index": 2 } },
      "3": { "stoa": { "index": 3 } }
    }
  }
  ```

- **Members are named `<base>-<member>-<env>`:** `stoa-1-lab`,
  `stoa-2-lab`, `stoa-3-lab`. The environment stays last, so every lab
  target ends in `-lab`. A base with members has no target `<base>-<env>`.
- **A base-env file may change the members** in its environment, under
  its own `members`: add one, remove one (`"3": null`), or set one's data
  there, such as its placement on a lab pool host. `"members": null` is
  an error (1.4).
- **A base is a cluster in an environment** when its document or its
  base-env's has `members`. It then makes one target per member left
  once the two are merged, and none when none is left. A `null` member
  in a base removes nothing. `members` in `fleet.json` or `envs/` is a
  `composition` error at its key, as `placement` in a base is.
- **Members see each other through exports** (`exports.md`, section 1), like any
  targets of the environment: stoa exports its address and index, and
  each member's templates loop over `exports.stoa`. `target.member` holds
  the member's name.

### 1.7 Example: x1-prod

`fleet.json`:

```json
{
  "_note": "Defaults for every target of the fleet.",
  "debian": {
    "suite": "trixie",
    "architecture": "amd64",
    "snapshot": { "archive": "20261001T030405Z", "security": "20261001T031522Z" }
  },
  "core": { "domain": "example.org" },
  "hearth": {
    "snapshots": {
      "core": { "every": "1d", "keep": { "daily": 7 } },
      "data": { "every": "1h", "keep": { "hourly": 24, "daily": 14, "weekly": 8 } }
    }
  },
  "locale": { "timezone": { "area": "Etc", "zone": "UTC" }, "locale": "en_US.UTF-8" },
  "ssh": {
    "port": 2222,
    "authorized": { "alice": { "x1": "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAA… alice@x1" } }
  }
}
```

`hosts/x1/x1.json`:

```json
{
  "_note": "x1, the laptop: Debian testing and a desktop.",
  "debian": {
    "suite": "forky",
    "pockets": { "updates": false, "security": false },
    "components": { "contrib": true }
  },
  "core": { "hostname": "x1" },
  "host": {},
  "locale": {},
  "boot": { "params": { "audit": { "param": "audit=0" } } },
  "users": { "accounts": { "alice": { "uid": 1000, "groups": ["sudo"], "password": "users/alice.passwd" } } },
  "wifi": {},
  "desktop": { "user": "alice" }
}
```

`hosts/x1/x1-prod.json`:

```json
{
  "placement": { "kind": "metal", "firmware": "efi", "hardware": "7th-gen" },
  "debian": { "components": { "non-free-firmware": true } },
  "firmware": {}
}
```

- **x1-prod's tasks:** core, host, locale, boot, users, wifi, desktop
  and firmware, and `hearth`, which render adds (`tasks-hearth.md`, 1). Not ssh:
  `fleet.json` set its port and keys, but defaults add no task. No
  `snapper` task, as os had: `fleet.json`'s `hearth` settings snapshot
  its `core` and `data` classes (`tasks-hearth.md`, 1).
- **Its sources:** forky, main, contrib and non-free-firmware, with no
  updates or security pocket, from the archive snapshot of `fleet.json`.
