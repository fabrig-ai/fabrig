# The IaC format

Provisional, 2026-10-09, revised 2026-10-17. The format of an IaC
repository as hearth reads it: the repository's files, how targets are
made from bases and environments, how layers merge, what a task declares
and the schema of its settings, the template language, exports between
targets, and how secrets, identity and software are named. It takes os's
configuration and task formats further, for agents and digests
(hearth/docs/design/draft/brainstorm.md, section 7), and is written fresh: there is no
converter from os's configurations. Milestone M1 builds it, but for
checks (M5) and artifacts (M6). How a revision becomes a spec is
hearth/docs/design/domain/render/README.md. Open questions are in section 11.

## 1. In one page

- **The repository is the whole input.** `fleet.json`, `envs/`, `hosts/`
  and `tasks/` at a revision decide every spec. Nothing on a command
  line, on the machine or in the process's environment changes one.
- **A target is a base in an environment.** `hosts/mail/mail-lab.json`
  makes the target `mail-lab`; a cluster's base makes one target per
  member, such as `stoa-1-lab`. Names are made from files and never
  parsed back.
- **Layers compose in a fixed order and merge by RFC 7386.** Task
  defaults, fleet, base, member, environment, base in the environment.
  Objects merge key by key, `null` removes, anything else replaces, lists
  included. Collections are maps keyed by name.
- **Defaults never add a task.** `fleet.json` and `envs/` set settings for
  the targets that have a task. Only a base and its environment files say
  which tasks a target has: `{}` adds a task with all its defaults, `null`
  removes it.
- **Placement is data of the environment.** Bare metal, a VM or a
  container on a named target, with its firmware and its hardware
  profile. A host's containers are the targets placed on it; the host
  keeps no list.
- **A task declares everything about one concern** in `task.json`: its
  settings and their schema, what it reads from other tasks, packages
  and preseeds, initramfs modules, software, files with modes and owners,
  users, presets, tmpfiles, persistent data, secret files, identity, boot
  parameters, exports, the units that consume its files when they change
  live, and its escape hatches with their order.
- **Every setting is declared:** a type, a default or `optional`, and one
  line of documentation. An undeclared setting, a wrong type, a missing
  required setting and an undeclared read are errors, with their file and
  line.
- **Templates are inert.** A file named `*.j2` is a template in a small
  Jinja subset: no macros, no includes, no calls, nothing that runs code.
  Every other file is copied as it is. An undefined name is an error,
  except an optional setting that is absent.
- **Exports are the only data that crosses targets.** A task declares
  them, computed from its settings alone; a reader names the task whose
  exports it reads; only targets of the same environment see them. Public
  halves of secrets and identity are recorded in the repository as data.
- **Secrets and identity are named, never held.** A task names each
  secret file by its name in the secret store, with a path, mode and
  owner; identity items (host keys, WireGuard keys) by kind; password
  hashes by secret. The activator writes them into a slot at prepare.
- **Software is fetched by digest.** A download is `{url, sha256}`,
  placed as one file or unpacked from a tar archive at a declared path;
  an artifact is pinned by its key and placed as a tree [M6].
- **hearth is a task too.** Every target but a container has it:
  hearth's binary, its activator's unit, its user and its subordinate
  range. No hearth runs in a container in M5; its host runs it.

## 2. The repository

### 2.1 Layout

```
<repository>/
├── fleet.json                          defaults for every target: the sources, fleet-wide constants
├── envs/
│   ├── lab.json                        defaults for every target in lab
│   └── prod.json                       defaults for every target in prod
├── hosts/
│   ├── x1/
│   │   ├── x1.json                     the base: a system described once
│   │   └── x1-prod.json                the base in prod: makes the target x1-prod
│   ├── chat/
│   │   └── chat-prod.json …
│   └── stoa/
│       ├── stoa.json                   a cluster's base, with its members (3.6)
│       └── stoa-lab.json               makes stoa-1-lab, stoa-2-lab, stoa-3-lab
└── tasks/
    └── ssh/
        ├── task.json                   the declaration (section 5)
        ├── tree/                       files installed at the same path under /
        │   └── etc/ssh/sshd_config.d/local.conf.j2
        ├── templates/                  templates rendered to computed paths (5.5)
        │   └── authorized_keys.j2
        ├── customize.sh                an escape hatch in the config layer (5.8)
        └── software.sh.j2              an escape hatch in the software layer (5.8)
```

- **Render reads `fleet.json`, `envs/`, `hosts/` and `tasks/`.** Any other
  file (a `../../README.md`, `lab-ca/lab.crt`) is read only when a setting of
  type `file` names it (5.2). Every path in the revision is UTF-8; one
  that is not is a `repository` error (hearth/docs/design/domain/render/errors.md, 2).
- **Under `hosts/<base>/`** the documents are `<base>.json` and
  `<base>-<env>.json` for environments that exist in `envs/`. Any other
  `.json` directly in it is an error; deeper, a `.json` is another
  file. Other files (a base's own nginx site) may sit beside them, named
  by `file` settings.
- **A task directory holds `task.json` and only the entries above,** plus
  an optional `../../README.md`. Anything else is an error, so a misspelt
  `tre/` fails instead of being ignored. Every file under `templates/`
  must be used by a `files` entry.

### 2.2 Documents

- **Strict JSON (RFC 8259),** one object per file, in UTF-8. Duplicate
  keys are an error at every depth. Numbers are integers within i64: a
  fraction or an exponent is an error. No string may hold U+0000.
- **`_note` is the one annotation.** A string allowed in any object, it is
  dropped when the document is decoded: it never reaches merging, a
  schema, a template or a digest, so editing a note changes no spec.
- **No document is a template.** Values in `fleet.json`, `envs/` and
  `hosts/` are data, taken literally. Only the declarations of a
  `task.json` (`tasks.md`, 2) and `*.j2` files are evaluated.
- **Bounds** are render's limits (hearth/docs/design/domain/render/README.md, 2.3): 256 KiB a document,
  depth 32, 4,096 keys an object. They count the document as written: a
  `_note` is one of its object's keys.
- **What a later milestone adds is refused before it.** A key or value
  this document marks [M5] or [M6] is an error at its key until that
  milestone builds it, as an undeclared field is: a placement's `ids`,
  `binds`, `capabilities`, `network` and `bridge`, and a container's
  `mutable`, whatever its value (3.4), a download's `artifact` (8.2), a
  task's `checks` (section 9). So nothing in a
  repository is silently ignored.

### 2.3 Templates and static files

- **A file whose name ends in `.j2` is a template; every other file is
  static.** `tree/etc/ssh/sshd_config.d/local.conf.j2` renders to
  `/etc/ssh/sshd_config.d/local.conf`; `tree/etc/issue.net` is copied byte
  for byte. The same rule holds for `customize.sh(.j2)`,
  `software.sh(.j2)` and everything under `templates/`, which must all be
  templates.
- **Why a suffix:**
  - *Inert by default.* A static file is never parsed, so it may hold any
    bytes: a certificate, a binary, a shell script with `${#a[@]}` (which
    a Jinja lexer reads as the start of a comment), a Prometheus rule
    with `{{ $labels }}`. os rendered every file, so a file that was not
    UTF-8 failed and a stray `{#` broke a script.
  - *Visible.* An agent, a diff and an editor see from the name alone
    which files are evaluated, and the path is still the real one, less
    a suffix.
  - *Cheaper.* Render hashes a static file and parses only templates.
  - *The alternatives are worse.* Two trees (`static/` and `templates/`)
    split one destination directory across two places; a list of
    templates in `task.json` drifts from the files.
- **A destination that itself ends in `.j2`** is written as
  `NAME.j2.j2`: the suffix is stripped once.
- **One source a destination:** a task that holds both `NAME` and
  `NAME.j2` (in `tree/`, or `customize.sh` or `software.sh` beside its
  `.j2`) is a `repository` error at the second in byte order, the first
  its `also` (hearth/docs/design/domain/render/errors.md, 1). Across tasks, one path from two is a
  collision (`tasks.md`, 2).
- **A template is a file with a name.** A symbolic link whose name ends
  in `.j2`, and a `.j2` alone, which would install at its directory's
  path, are `repository` errors.
- **Templates are UTF-8 with `\n` line ends;** a `\r` in a template is an
  error. Static files are bytes.

### 2.4 Names

| Name | Form | Notes |
|---|---|---|
| base, environment, member | `[a-z0-9]`, then `[a-z0-9-]`, not ending in `-` | no dot |
| target | made from them (3.1), at most 63 bytes | no dot; never `hearth` |
| task, setting, object field, export | `[a-z]`, then `[a-z0-9_]` | identifiers in expressions |
| guard (`tasks.md`, 9) | `[a-z]`, then `[a-z0-9_]` | printed in its error, never in an expression |
| data | `[a-z]`, then `[a-z0-9-]`, at most 32 bytes | names a directory in its class (`tasks.md`, 7); no dot |
| secret | segments of `[a-z0-9._-]` joined by `/`, at most 255 bytes | no `.` or `..` segment |
| key of a map setting | any string of 1 to 255 bytes | |

- **No dots in target names.** The btrfs top level names a target's
  subvolumes `@<target>.<label>` and `@<target>.<class>`, and the ESP its
  files `vmlinuz.<target>.<label>` (hearth/docs/design/domain/machine/README.md, section 3, and hearth/docs/design/domain/machine/boot.md, section 1); a dot
  in a target would make them ambiguous. Data names have no dot either,
  so a restore's `<name>.restoring` beside an entry's directory is never
  an entry (hearth/docs/design/domain/machine/README.md, 3.4).
- **`hearth` is never a target,** since `@hearth.store` and `@hearth.state`
  share the top level.
- **Tasks and settings are identifiers,** so `tasks.core.domain` and
  `settings.public_key` parse; a hyphen would read as a minus.
- **A guard's name has their form,** though no expression reads it: a
  false guard prints it in its error, `boot: requires modules: no
  initramfs modules for this hardware` (hearth/docs/design/domain/render/errors.md, 3), and a key of any
  bytes, a `: ` or a line feed in it, would break that line. A name out
  of form is a decode error at its key.

## 3. Targets

### 3.1 Bases, environments and targets

- **A target is a base in an environment.** For each `hosts/<base>/` and
  each `<base>-<env>.json` beside it whose `envs/<env>.json` exists, there
  is one target `<base>-<env>`, or one per member of a cluster (3.6). The
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

### 3.2 Composition order

```
task defaults                      the defaults of every task the target has (section 6)
fleet.json                         defaults for every target
hosts/<base>/<base>.json           the base, without its members key
  members.<member>                   the member, in a cluster (3.6)
envs/<env>.json                    defaults for every target in <env>
hosts/<base>/<base>-<env>.json     the base in <env>, without its members key
  members.<member>                   the member in <env>
```

- **Each layer is merged onto the result of those above it** (section 4),
  so a later layer wins.
- **The environment comes after the base,** so its defaults (the lab's
  certificate authority, its DNS overrides) apply to every base in it;
  the base in the environment comes last, for what belongs to one target
  alone. This is hearth/docs/design/draft/brainstorm.md's order (`templates.md`, 3), with task defaults
  and members added.

### 3.3 Which tasks a target has

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
  container has (`tasks.md`, 12). **Its settings** are the merge of every layer,
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
  `placement` (3.4), `members` (3.6) and `_note`.

### 3.4 Placement

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
| `ids` | int | for a container on a host [M5]: the base of its 64 Ki id range, nspawn's `PrivateUsers=` (3.5) |
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
  cleared, and a base-env removes a member by its own key (3.6).
- **Placement is replaced whole,** the one exception to RFC 7386 (4.1):
  a layer that sets `placement` replaces it entirely, so `envs/lab.json`'s
  `firmware` and `hardware` never leak into a lab target that its own
  file places in a container.
- **What placement decides by itself:** the bootstrap variant
  (hearth/docs/design/domain/render/spec.md, 1.2), and that a container has no kernel, so its initramfs
  modules and boot parameters must be empty: its packages part has no
  initramfs, its boot part holds its container parameters instead of a
  command line (3.5), and render refuses a module or a parameter on it
  (hearth/docs/design/domain/render/errors.md, 2). The `boot` task, which installs the kernel, refuses a
  container by a guard of its own (`tasks.md`, 4).
- **`mutable` is a slot's, never an output's.** Render puts it in the
  `boot` part (hearth/docs/design/domain/render/spec.md, 1.3), so it enters the prepare key and no
  layer key: a mutable target builds the output its sealed twin would.
  Render accepts it in every layer placement may be in, on metal and a
  VM, and on a container from M5 (2.2): before M5 a container's
  `mutable` is refused at its key, as its `ids` is, since a container in
  development is M5's (hearth/docs/design/domain/development.md, 3.4). Whether a machine takes
  it is the machine's configuration (hearth/docs/design/domain/machine/bootstrap.md, 1), and the run
  side's engine never orders one [M4] (hearth/docs/design/domain/development.md, 4.3). An
  environment such as `dev` sets it for all its targets.
- **Everything else that differs by placement is task data** that reads
  `target.placement`: the gettys masked in containers, the initramfs
  modules per hardware profile, EFI's tools (`tasks.md`, 10).

### 3.5 Containers on a host

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
  but no `hearth` task (`tasks.md`, 12) and nothing that needs a kernel (3.4); the
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
  (`README.md`, section 6).
- **A container's parameters are placement fields** [M5] (3.4): its id
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

### 3.6 Clusters

- **A base declares its members** under `members`, a map keyed by member
  name. Each member's value is a layer of its own (3.2): tasks and their
  settings, and `debian`; `placement` in a base-env's member alone, since
  a base's member is the base's (3.4); never `members`.
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
  an error (3.4).
- **A base is a cluster in an environment** when its document or its
  base-env's has `members`. It then makes one target per member left
  once the two are merged, and none when none is left. A `null` member
  in a base removes nothing. `members` in `fleet.json` or `envs/` is a
  `composition` error at its key, as `placement` in a base is.
- **Members see each other through exports** (section 6), like any
  targets of the environment: stoa exports its address and index, and
  each member's templates loop over `exports.stoa`. `target.member` holds
  the member's name.

### 3.7 Example: x1-prod

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
  and firmware, and `hearth`, which render adds (`tasks.md`, 12). Not ssh:
  `fleet.json` set its port and keys, but defaults add no task. No
  `snapper` task, as os had: `fleet.json`'s `hearth` settings snapshot
  its `core` and `data` classes (`tasks.md`, 12).
- **Its sources:** forky, main, contrib and non-free-firmware, with no
  updates or security pocket, from the archive snapshot of `fleet.json`.

## 4. Merging

### 4.1 RFC 7386

```
merge(target, patch):
    if patch is not an object:      return patch                 scalars and lists replace
    result = target if target is an object, else {}
    for key, value in patch:
        if value is null:           remove key from result
        else:                       result[key] = merge(result[key], value)    a missing key is not an object
    return result
```

- **Each layer is a patch** over the result of the layers above it
  (3.2), but for `placement`, which a layer replaces whole (3.4). Objects merge key by key; `null` deletes; scalars and lists
  replace. The result never holds `null`, and no setting can: the schema
  has no null type, and an absent optional setting is simply absent.
- **A `null` a merge would carry is an error where it is written:** one
  inside a list (`[1, null]`, `[{"a": null}]`), which replaces whole, is
  a `decode` error in a document (hearth/docs/design/domain/render/errors.md, 2) and a bad default in a
  `task.json` (5.5); one in a placement is a wrong type (3.4).
- **Lists replace.** os concatenated them, so a variant could add to a
  base's list but never drop or replace an entry, and thin variants grew
  into copies.
- **Values keep where they came from.** Each composed value carries the
  file, line and column of the layer that set it, so an error names the
  layer at fault (hearth/docs/design/domain/render/errors.md).

### 4.2 Collections are maps keyed by name

- **A collection is a map keyed by name:** accounts, keys, peers, domains,
  modules, parameters. A later layer then adds, changes or removes one
  entry. `hosts/mail/mail.json`:

  ```json
  { "users": { "accounts": { "alice": { "uid": 1000 }, "bob": { "uid": 1001 } } } }
  ```

  and `hosts/mail/mail-lab.json`, which removes bob and adds a tester:

  ```json
  { "users": { "accounts": { "bob": null, "tester": { "uid": 1100 } } } }
  ```

- **A list is for values whose order is their meaning,** which a layer
  replaces whole: DNS servers in order of preference, a command's
  arguments. The bash fleet's positional `user:uid:home` strings and os's
  lists of objects become maps.

### 4.3 Order

- **Maps iterate in key order:** the bytes of the UTF-8 keys, compared
  lexicographically. Templates, `for_each` and the spec's parts see the
  same order on every machine. The order of members in a JSON source
  never matters.
- **Where order matters and keys do not give it, an `order` field says
  it:** an integer from 0 to 999, default 500. Entries sort by order,
  then key, then task name. It is used by `customize.sh` and
  `software.sh` fragments (`tasks.md`, 8) and by boot parameters (`tasks.md`, 4), where the
  last `console=` names the kernel's primary console. Out of that range,
  an `order` is a decode error in `scripts`, which is decoded as written
  (`tasks.md`, 8), and a schema error in a boot parameter, an entry whose `order`
  may be an expression (`tasks.md`, 2).

### 4.4 Presence, defaults and removal

- **A task present with all its defaults:** `"locale": {}` in
  `hosts/x1/x1.json`.
- **A task removed in one environment:** carbon is a laptop whose lab
  variant is an Incus VM, which has no Wi-Fi. `hosts/carbon/carbon-lab.json`:

  ```json
  { "_note": "An Incus VM has no Wi-Fi.", "wifi": null }
  ```

  The bash fleet could not express this: task presence was the union of
  both files, so carbon-lab inherited the Wi-Fi task and its iwd mount.
- **One entry removed, one replaced:**
  `{"ssh": {"authorized": {"alice": {"old-laptop": null}}}}` drops one key
  of one user; `{"dns": {"servers": ["10.0.0.1"]}}` replaces the whole
  list of servers.

## 5. The schema

### 5.1 Form

`settings` in `task.json` is a map from setting name to its spec:

```
spec  = { "type": type, "doc": string,
          "default"?: value, "optional"?: true,
          "min"?: int, "max"?: int, "max_length"?: int }
type  = "bool" | "int" | "string" | "path" | "mode" | "duration" | "secret" | "target" | "file"
      | { "enum": [string, …] }
      | { "list": type }
      | { "map": type }                               keyed by name
      | { "object": { field: spec, … } }              fixed fields, each a spec
```

- **`doc` is required:** one line, at most 200 bytes, for every setting
  and every object field. It is what an agent reads to edit the setting.
- **`min` and `max`** bound an int; **`max_length`** bounds a string's
  bytes, a list's elements or a map's entries. Each is refused on any
  other type: `min` and `max` on all but `int`, `max_length`, 0 or more,
  on all but `string`, `list` and `map`, so on a `path`, a `secret` or an
  `enum` too.
- **An `enum` lists one value at least,** each once.
- A schema is checked when its `task.json` is decoded: a default that does
  not have its type, `default` with `optional`, or `min` above `max` is an
  error in the task, wherever it is used. Whether a `target` in a default
  exists depends on the environment, so it is checked on each target
  whose settings hold it (5.3).

### 5.2 Types

| Type | JSON | Notes |
|---|---|---|
| `bool` | `true`, `false` | |
| `int` | integer | i64 |
| `string` | string | UTF-8, no U+0000 |
| `enum` | string | one of the listed values |
| `list` | array | ordered; replaced whole on merge |
| `map` | object | keyed by name; merged per key; iterated in key order |
| `object` | object | only its declared fields; merged per field |
| `path` | string | absolute; no empty, `.` or `..` segment; no trailing `/`; `/` alone is one |
| `mode` | string | three or four octal digits, at most `"7777"` |
| `duration` | string | a positive integer with no leading zero and one unit, `s`, `m`, `h` or `d`: `5s`, `1m`, `14d`; a number of seconds once decoded, which fits an i64 |
| `secret` | string | a name in the secret store (2.4), never a value |
| `target` | string | a target of the same environment, which must exist |
| `file` | string | a path relative to the repository's root, with no empty, `.` or `..` segment and no leading or trailing `/`, of a file git records at the revision, mode `100644` or `100755`: never a link, a submodule or a directory |

- **No floats and no null.** Nothing in the fleet needs a fraction, and
  null is RFC 7386's deletion.
- **`secret` keeps values out of the repository.** A template that prints
  one prints a name.
- **`file` brings a repository file into a target,** such as the lab's
  certificate authority: `"extra_ca": {"lab": "lab-ca/lab.crt"}` with a
  `copy` entry in `files`. It replaces os's `copy` from a workspace
  outside the repository.

### 5.3 Defaults, optional and required

- **Defaults are the bottom layer** (3.2): the schema's defaults form a
  document that the composition merges first, so a layer can extend a
  default map or remove one of its entries with `null`.
- **Inside maps and lists,** whose entries the schema cannot name in
  advance, each object entry has its absent fields filled from the
  object type's defaults during validation: `"alice": {"uid": 1000}` gets
  `/bin/bash` from the users task's `shell` default.
- **`optional`:** the setting may be absent, and reads as undefined
  (`templates.md`, 6). A setting with neither `default` nor `optional` is required.
- **Validation fills what is absent,** after the composition: a setting
  from its `default`, and an object's field, at any depth, from its
  field's. So `null` for a setting that has a default restores the
  default, and `null` for an entry of a default map removes the entry.
  A default that is a whole value, an object or a map or list of them, has
  its fields checked and filled like any value, and a `target` in it is
  checked on each target whose settings hold it.

### 5.4 Reads

```json
{
  "reads": {
    "core": { "settings": ["domain"] },
    "wifi": { "settings": ["ssid"], "optional": true },
    "mail": { "exports": true }
  }
}
```

- **A task reads another task's settings only if it declares them:**
  `tasks.core.domain` in its templates and expressions. The read value is
  the other task's setting with its default applied; its type is the
  other task's.
- **The other task must be on the target,** unless the read is
  `optional`; then `tasks.wifi` is undefined where wifi is absent, and
  `tasks.wifi is defined` tells. That is a settings read: a read of
  `exports` alone needs the task on no target, and `exports.<task>` is
  an empty map where no target of the environment has it (6.2).
- **`exports: true`** reads that task's exports from the environment's
  targets (6.2).
- **References are checked statically:** every `settings.<name>`,
  `tasks.<task>.<setting>` and `exports.<task>` in a task's templates and
  declarations must be declared. `tasks` and `exports` take only literal
  attribute names after them. So the typo `settings.alow` fails at
  render, where os dropped a file.
- **What may stand alone:** `tasks` is read only as
  `tasks.<task>.<setting>`, both names literal, so bare `tasks`,
  `tasks[x]`, `tasks.<task>[x]` and `tasks.<task>` used whole are
  errors, but `tasks.<task> is defined` for an optional read.
  `exports.<task>` may be used whole, the map of 8.2. `settings` may be
  used whole or looked into with `settings[x]`, which is checked when
  evaluated: a name the task does not declare is an error there, and an
  absent optional one is undefined (`templates.md`, 6).

### 5.5 What is an error

Each with its file and line (hearth/docs/design/domain/render/errors.md):

- a setting the task does not declare, or an undeclared field at any
  depth of an object;
- a value of the wrong type, an enum value not listed, an int out of
  range, a malformed path, mode or secret name, a `target` that does not
  exist in the environment, a `file` absent from the revision;
- a required setting missing after composition;
- a read of a task or setting not declared in `reads`, or of a setting
  the other task does not declare;
- `null` for a task in `fleet.json` or `envs/`, `placement` in a base or
  in `fleet.json`, `members` in `fleet.json` or `envs/` (3.6), a task
  name that `tasks/` lacks.

### 5.6 Example: the users task

`tasks/users/task.json`, whose `users` declaration is `tasks.md`, 2's example:

```json
{
  "doc": "People's accounts: their ids, shells, groups and password hashes.",
  "settings": {
    "accounts": {
      "type": { "map": { "object": {
        "uid": { "type": "int", "min": 1000, "max": 59999, "doc": "The account's id." },
        "shell": { "type": "path", "default": "/bin/bash", "doc": "The login shell." },
        "groups": { "type": { "list": "string" }, "default": [], "doc": "Supplementary groups, such as sudo." },
        "password": { "type": "secret", "optional": true, "doc": "The secret holding the password's hash." }
      } } },
      "default": {},
      "doc": "Accounts, by user name."
    }
  },
  "data": {
    "home-{{ key }}": {
      "for_each": "settings.accounts",
      "path": "/home/{{ key }}", "mode": "0700", "owner": "{{ key }}", "group": "{{ key }}"
    }
  }
}
```

- `hosts/x1/x1.json`'s `"alice": {"uid": 1000, "groups": ["sudo"], "password": "users/alice.passwd"}`
  validates, and gets `/bin/bash`; its home is the directory
  `home-alice` in `@x1-prod.data`, bound at `/home/alice`.
- `"alice": {"uid": "1000"}` is a wrong type; `"alice": {"shel": "/bin/sh"}`
  an undeclared field; `"alice": {}` misses the required `uid`. Each error
  names `hosts/x1/x1.json` and the line.

## 6. Exports

### 6.1 Declaring

- **`exports` is a map from export name to a spec** like a setting's
  (`type`, `doc`, `optional`) and a `value`, written as one
  `{{ expr }}`.
- **The value is checked as a setting's is** (5.5), on each target that
  evaluates it: its type and form, an object's fields declared and its
  required ones present, a `target` that exists in the environment, a
  `file` the revision holds.
- **Computed from settings only:** the task's own settings and `target`.
  Never from another task's reads, never from another target's exports,
  so exports cannot form a cycle; never of type `secret`, at any depth of
  the type (6.4). `debian`, `tasks` and `exports` are refused in a
  value, and a `settings.<name>` the task does not declare is an error,
  as anywhere (5.4).
- **An optional export** whose expression is undefined is absent.
- The exports part holds them (hearth/docs/design/domain/render/spec.md, 1.4); a change to an export
  changes exactly the specs that read it.

### 6.2 Reading

- **Exports are read by task, which is to say by role.** A reader
  declares `"reads": {"vpn": {"exports": true}}`, and its templates see
  `exports.vpn`: a map from target name to that target's vpn exports, for
  every target of the environment that has the task, itself included.
- **Heterogeneous pairs read the same way.** The bash fleet's pairs are
  roles, not identical members: mail's primary and backup, the VPN's hub
  and spokes. Each side exports its role, and the reader picks by it
  (`templates.md`, 2's `mynetworks`).

### 6.3 Scope

- **Exports never cross an environment.** A lab target reads only lab
  targets' exports, and production only production's. The bash fleet's
  recurring error, lab variants carrying production's public keys, and a
  container carrying production's forge host key in the lab, cannot be
  written.
- **Render evaluates the exports of the environment's targets** to render
  any one of them (hearth/docs/design/domain/render/README.md, 3.4): rendering stays a function of the
  revision and the target.

### 6.4 Public halves as data

- **Secrets are never exported, but their public halves often must be:**
  a WireGuard public key, an SSH host key for `known_hosts`, a Syncthing
  device id. Render never sees a secret, so it cannot derive them.
- **So the public half is recorded in the repository** as a setting of
  the target that owns it, in its base-env file. For identity, the
  activator generates the item and reports its public half as a fact
  (hearth/docs/design/protocol/wire.md, section 5); a person or an agent records it, and the change's
  rendered diff shows every peer that picks it up. Until then the
  setting is absent, the optional export with it, and peers skip that
  target.

### 6.5 Fleet-wide constants

- **A value every target shares is a default in `fleet.json`, not an
  export:** the domain, the operator's SSH keys. Exports are for what
  differs by target and must agree between targets.

### 6.6 Example: the VPN

`tasks/vpn/task.json`:

```json
{
  "doc": "A WireGuard network between the targets of an environment, through a hub.",
  "settings": {
    "address": { "type": "string", "doc": "This target's address on the network, such as 10.0.0.2." },
    "hub": { "type": "bool", "default": false, "doc": "Whether this target relays for the others." },
    "endpoint": { "type": "string", "optional": true, "doc": "The public address peers dial, for a hub." },
    "port": { "type": "int", "min": 1, "max": 65535, "default": 51820, "doc": "The UDP port." },
    "public_key": { "type": "string", "optional": true, "doc": "This target's public key, as its activator reported it." }
  },
  "reads": { "vpn": { "exports": true } },
  "identity": {
    "wireguard": {
      "kind": "wireguard-key", "path": "/etc/wireguard/wg0.key",
      "mode": "0640", "owner": "root", "group": "systemd-network"
    }
  },
  "exports": {
    "address": { "type": "string", "value": "{{ settings.address }}", "doc": "Its address on the network." },
    "hub": { "type": "bool", "value": "{{ settings.hub }}", "doc": "Whether it relays." },
    "endpoint": { "type": "string", "optional": true, "value": "{{ settings.endpoint }}", "doc": "Where to dial it." },
    "public_key": { "type": "string", "optional": true, "value": "{{ settings.public_key }}", "doc": "Its public key." }
  }
}
```

`hosts/west/west.json` holds `"vpn": {"address": "10.0.0.1", "hub": true}`,
and `hosts/west/west-prod.json` what belongs to west in production,
`"vpn": {"endpoint": "203.0.113.157", "public_key": "AAAA…="}`, the
key as west-prod's activator reported it.

`tasks/vpn/tree/etc/systemd/network/wg0.netdev.j2`:

```jinja
[NetDev]
Name=wg0
Kind=wireguard

[WireGuard]
PrivateKeyFile=/etc/wireguard/wg0.key
ListenPort={{ settings.port }}
{% for name, peer in exports.vpn | items %}
{% if name != target.name and peer.public_key is defined and (settings.hub or peer.hub) %}

[WireGuardPeer]
PublicKey={{ peer.public_key }}
AllowedIPs={{ peer.address }}/32
{% if peer.endpoint is defined %}
Endpoint={{ peer.endpoint }}:{{ settings.port }}
PersistentKeepalive=25
{% endif %}
{% endif %}
{% endfor %}
```

- carbon-prod, a spoke, gets west-prod's key, address and endpoint from
  west-prod's file; west-prod gets carbon-prod's key from carbon-prod's.
  Nobody copies a key, and carbon-lab sees only lab peers, with lab keys.
- **`wireguard-key` is [later]** (7.3). Until it exists, the vpn task
  names the private key as a secret file at the same path, an operator
  imports it, and records its public half in the target's file just as
  above. The group `systemd-network` owns the key, so the task declares
  it from its package with a fixed id (`tasks.md`, 6).

### 6.7 Example: the backup peer [M6]

- **Backups are encrypted to the peer's key** with `age` before any byte
  reaches a steward (hearth/docs/design/domain/operating.md, section 7). The peer's `backup_peer`
  task holds that key as identity and exports its public half, recorded
  as data like any other (6.4):

  ```json
  {
    "doc": "Receives the environment's backups, encrypted to its own key.",
    "settings": {
      "recipient": { "type": "string", "optional": true, "doc": "This peer's age recipient, as its activator reported it." }
    },
    "identity": { "backup": { "kind": "backup-key" } },
    "exports": {
      "recipient": { "type": "string", "optional": true, "value": "{{ settings.recipient }}", "doc": "Where backups are encrypted to." }
    }
  }
  ```

- **The backed-up target's render reads it.** A class that the `hearth`
  task's `backups` names (`tasks.md`, 12) makes render read `backup_peer`'s
  exports in the target's environment and put the peer and its
  recipient into that class's policy in the data part (hearth/docs/design/domain/render/spec.md, 1.3),
  where the activator takes it at prepare. A target with backups and no
  peer that exports a recipient is an error.

## 7. Secrets and identity

### 7.1 Secret files

- **`secrets` is a map keyed by the secret's name in the secret store**
  (2.4), with `path`, `mode` (default `0600`), `owner` and `group`
  (default `root`, names with fixed ids, `tasks.md`, 6, that the activator resolves
  in the slot), `if` and `for_each`:

  ```json
  {
    "secrets": {
      "mail/dkim/{{ key }}.key": {
        "for_each": "settings.domains",
        "path": "/etc/dkimkeys/{{ key }}.key",
        "mode": "0640", "group": "opendkim"
      }
    }
  }
  ```

- **The activator writes each into the slot at prepare,** from the
  root-only secret store (hearth/docs/design/domain/slots/prepare.md, 4). A missing secret fails the
  prepare, not the render: render never sees the store.
- **Paths:** a secret's path may not be one the file tree, another secret
  or identity gives, nor lie under a data path; a collision is an error
  (hearth/docs/design/domain/render/README.md, 3.7). A path beneath `/var` is allowed, and is written into
  the slot's own `/var` subvolume (hearth/docs/design/domain/slots/prepare.md, 4).
- **Never `hearth`'s to read,** the steward's nor its builder's: a
  secret's mode gives others nothing (at most `0770`, no setuid, setgid
  or sticky bit), and its owner is neither user `hearth` nor a group
  `hearth` belongs to, such as `systemd-journal` (`tasks.md`, 11).
- **Rotating a secret changes no part.** The activator prepares a new
  slot of the same output (README, `templates.md`).
- **TLS keys are secrets.** Build removes the snakeoil pair that
  ssl-cert's postinst makes (hearth/docs/design/domain/build/packages.md, 6), so a task that serves TLS
  declares its private key as a secret file, such as
  `"mail/tls/mx1.key": {"path": "/etc/ssl/private/mx1.example.org.key",
  "mode": "0640", "group": "ssl-cert"}`, with `ssl-cert` declared from
  its package with a fixed id (`tasks.md`, 6). A certificate a machine obtains
  itself (ACME) lives on persistent data instead.

### 7.2 Password hashes

- **An account's `password` names a secret** that holds its hash, as
  crypt(3) writes it. The activator applies it to the slot's shadow file
  at prepare, the one edit it makes to a packaged file (hearth/docs/design/domain/slots/prepare.md, 4).
  An account without one stays locked.
- The bash fleet ran `chpasswd --encrypted` in `customize.sh`, so hashes
  ended up in every slot's `/etc/shadow` and in etckeeper's history.

### 7.3 Identity

- **Identity belongs to a target on its machine.** The activator
  generates each item once, when it is missing, keeps it in the secret
  store, writes it into every slot at prepare, and reports its public
  half as a fact (6.4).
- **`identity` is a map keyed by item name,** each with `kind`, its own
  fields, `path`, an optional `public_path`, `mode` (default `0600`),
  `owner` and `group` (fixed ids, `tasks.md`, 6; never `hearth`'s, `tasks.md`, 11):

  | Kind | Fields | Public half | Milestone |
  |---|---|---|---|
  | `ssh-host-key` | `algorithm`: `ed25519`, `ecdsa` or `rsa` | the `.pub` line | M3: made with the running slot's `ssh-keygen`, which the hearth task installs (`tasks.md`, 12) |
  | `wireguard-key` | none | the base64 public key | [later]: a target's first slot cannot be prepared with a `wg` only that slot installs, so it waits for keys made in-process, an ask of skein |
  | `backup-key` | none; no `path`: the activator keeps it and writes it nowhere | the `age1…` recipient | [M6]: an X25519 key made with `age-keygen` (hearth/docs/design/domain/activator/operations.md, 7; 6.7) |

  ```json
  {
    "identity": {
      "ssh-ed25519": {
        "if": "settings.allow", "kind": "ssh-host-key", "algorithm": "ed25519",
        "path": "/etc/ssh/ssh_host_ed25519_key", "public_path": "/etc/ssh/ssh_host_ed25519_key.pub"
      }
    }
  }
  ```

- **Never `hearth`'s to read,** as a secret (7.1): an item's `mode`, its
  private half's, has no bit outside `0770`, so it gives others nothing
  and has no setuid, setgid or sticky bit.
- **`wireguard-key` and `backup-key` wait for their milestones:** until
  then render refuses each as a kind outside the list. So M1 takes
  `ssh-host-key` alone, which the spec's identity part carries for M3's
  activator.
- **The machine-id is every target's,** never declared. The bash fleet
  fixed it in configuration so the journal had a stable path, and lab,
  production and a container shared one; here each target on each
  machine has its own, kept across its slots (hearth/docs/design/domain/slots/prepare.md, 4); a
  container's is kept by its host's activator (hearth/docs/design/domain/slots/containers.md, 1) [M5].
- **The kinds are a closed list** that grows with the activator's
  operations (hearth/docs/design/domain/activator/operations.md).

### 7.4 What the spec holds

- **Names and places, never values:** the secrets part holds secret
  names with their paths, modes and owners; accounts' password secret
  names; identity items' names, kinds and paths (hearth/docs/design/domain/render/spec.md, 1.3). No
  byte of a secret or of identity is in a repository, a spec or an
  output.

## 8. Software

### 8.1 Downloads

This section defines how software is placed; hearth/docs/design/domain/build/README.md (section 5) and
hearth/docs/design/domain/build/artifacts.md (`templates.md`, 3) cite it.

- **`software` is a map keyed by name;** a download is fetched by its
  SHA-256 into the builder's content-addressed cache and placed in the
  software layer (hearth/docs/design/domain/build/README.md, section 5), in one of two ways:

  | Field | `file` | `tar` | Meaning |
  |---|---|---|---|
  | `url` | required | required | where to fetch it; outside the part's digest |
  | `sha256` | required | required | 64 lowercase hex digits: what it is |
  | `unpack` | `"file"` | `"tar"` | a single file, or an archive the target's `tar` unpacks, detecting its compression |
  | `path` | required | required | the file's path; the directory the archive unpacks into |
  | `strip` | | default 0; 0 to 255 | leading path segments dropped, as `tar --strip-components` |
  | `mode` | default `0755` | | the file's mode, at most `0777` |
  | `owner`, `group` | default `root` | default `root` | the owners of the file, or of every file and directory the archive gives |

  ```json
  {
    "software": {
      "forgejo": {
        "url": "https://codeberg.org/forgejo/forgejo/releases/download/v15.0.0/forgejo-15.0.0-linux-amd64",
        "sha256": "…", "unpack": "file", "path": "/opt/forgejo/bin/forgejo"
      },
      "ripgrep": {
        "url": "https://github.com/BurntSushi/ripgrep/releases/download/15.1.0/ripgrep-15.1.0-x86_64-unknown-linux-musl.tar.gz",
        "sha256": "…", "unpack": "tar", "strip": 1, "path": "/opt/ripgrep"
      }
    }
  }
  ```

- **Modes:** a `file` takes its `mode`; an archive's files keep the modes
  it records. `path`, and any parent missing, is made `0755`.
- **Owners:** software is placed in the software layer, before any
  sysusers user exists (hearth/docs/design/domain/build/README.md, 6.1). So an owner or group is `root`,
  or a package user or group with a fixed id (`tasks.md`, 6), which build seeds
  before any package installs; render refuses any other. An archive is
  unpacked with `--no-same-owner`, whatever owners it records, and its
  files then take the entry's owners (hearth/docs/design/domain/build/README.md, 5.2).
- **No setuid, setgid or file capability:** a `file`'s mode has no such
  bit, and an archive whose members carry setuid or setgid fails the
  build. A program that needs privilege comes from a package.
- **Paths are beneath `/opt` or `/usr/local`,** never either itself,
  the bash fleet's and os's places for software (`tasks.md`, 5), and outside every
  path of the file tree and every other download's: an overlap is an
  error. hearth's own binary is `/usr/local/bin/hearth` (`tasks.md`, 12;
  hearth/docs/design/domain/activator/README.md, 3.1).
- **A version moves with its digest.** A task pins both in `task.json`; a
  bump edits both, in one change whose rendered diff shows it. os's
  `%VERSION%` in a URL goes.

### 8.2 Artifacts [M6]

- **An artifact is pinned by its key,** with how to make it, and its tree
  is placed at `path` (hearth/docs/design/domain/build/artifacts.md, 7.1):

  ```json
  {
    "software": {
      "stoa": {
        "artifact": {
          "repository": "https://git.example.org/ai/stoa.git",
          "commit": "4e1d0c3a…",
          "recipe": "stoa",
          "key": "9f2c71e0…"
        },
        "path": "/opt/stoa"
      }
    }
  }
  ```

- **Built from this head,** a lab variant writes `"artifact": {"recipe":
  "stoa"}`: the recipe at the revision being rendered (hearth/docs/design/domain/build/artifacts.md, 7.2).
  The software part holds the recipe and the digest of its sources'
  listing at the revision, which render computes from its own listing and
  the repository's `artifacts.json`; never a resolved key. The key,
  resolved and remembered as packages are, enters the software layer's
  key instead (hearth/docs/design/domain/render/spec.md, 1.5), so render stays a function of the
  revision.
- **An artifact is a tree, not an archive:** it has no `unpack`, `strip`
  or `mode`; its tree is placed under `path` from the artifact's
  subvolume, with the owners and modes 8.1 allows (hearth/docs/design/domain/build/artifacts.md, 7.3).
  An entry has either `url` and `sha256`, or `artifact`.

## 9. Checks [M5]

- **A task declares its checks** under `checks`, a map keyed by check
  name: what healthy means for what it installs. Their kinds, fields and
  semantics are hearth/docs/design/domain/operating.md, section 2; render places them in the
  checks part.
- **Checks are declarations like the others** (`tasks.md`, 2): `if` and
  `for_each`, keys and string fields interpolated, typed fields written
  as one `{{ expr }}`. So a check reads the task's settings:

  ```json
  { "checks": { "ssh": { "kind": "probe", "probe": "tcp", "host": "127.0.0.1", "port": "{{ settings.port }}", "class": "blessing" } } }
  ```

- **A role is a task.** A probe's `from` names a task, and the targets of
  the environment that have it run the probe (hearth/docs/design/domain/operating.md, 2.6),
  reading it as they read exports (section 6).
- **Render refuses** a `blessing` check with `from`, a `from` that no
  target of the environment has, and a command check whose `user` is
  `root`, or is neither `nobody`, a sysusers user nor a package user some
  task declares (`tasks.md`, 6; hearth/docs/design/domain/operating.md, 2.2; hearth/docs/design/domain/render/README.md, 3.9). The
  postfix task's `queue` check, as `postfix`, needs the postfix task to
  declare `"postfix": {"from": "package"}`.
- **Until M5, render refuses a task that declares checks,** so nothing in
  a repository is silently ignored. A declaration without effect would
  be ignored silently, which the format never does.

## 10. What changes from os

| os | hearth |
|---|---|
| `hosts/<host>/<host>-<env>.json` over a base | fleet, base, member, environment, base in the environment (3.2) |
| containers in `hosts/<host>/machines/`, alone, with no environment | containers are targets; placement names their host (3.5) |
| `--overlay` | gone: every input is in the repository |
| lists concatenate; nothing can be removed | RFC 7386; collections are maps; `null` removes (section 4) |
| a top-level key is a task, case-insensitively, in any file | presence from base and base-env layers only, exact names (3.3) |
| `_`-prefixed keys skipped for tasks, kept in the context | `_note` dropped when decoded (2.2) |
| reserved `suite`, `arch`, `snapshot` | `debian` (`tasks.md`, 3) |
| `container`, `desktop`, `7th-gen` steer the build | placement, and `debian.components` (`README.md`, 3.4, `tasks.md`, 3) |
| kind, host and env inferred from a hyphen | names made from files, never parsed (3.1) |
| `etc/` flat, with `___` for `/`; `usr-local-share/` | `tree/` mirrors `/` (`tasks.md`, 5) |
| every file rendered; final newline stripped | `*.j2` only; newlines kept (`README.md`, 2.3, `templates.md`, 2) |
| `etc.json`: `require`, `tree`, `render`, `templates` | `requires`, `files` with `if` and `for_each` (`tasks.md`, 2, 5) |
| templates completed at deploy or activation | none: the activator writes fstab from the data part (README, `templates.md`) |
| settings undeclared; undefined is falsy | schemas; undefined only where optional (6, `templates.md`, 6) |
| modes from the source file; owners root | modes and owners declared (`tasks.md`, 5) |
| collisions overwrite in task order | collisions are errors (hearth/docs/design/domain/render/README.md, 3.7) |
| `customize.sh` concatenated in task order | ordered, each run alone (`tasks.md`, 8) |
| `packages` and `secrets` as templates of lines | maps of entries (`tasks.md`, 3, `README.md`, 7.1) |
| secrets copied into the output | written into the slot at prepare, with modes and owners (7.1) |
| machine-id from configuration | identity, per target and machine (7.3) |
| software in config as `{name, version, url, sha256}`, cached by name | in the task, by digest, placed as a file or unpacked from tar (8.1) |
| `copy` from a workspace | `file` settings, from the repository (5.2) |
| variant maps (never implemented) | maps of objects |
| hard-coded preseeds, journal and iwd, gettys, snapshot host | task data (`tasks.md`, 10) |
| `@T.srv`, `@T.srv.core`, `@T.srv.data`, `@T.srv.cache` made at format time, with `journal` and `iwd` bound by fstab's template; snapper for snapshots | the classes `@<target>.core`, `.data` and `.cache`, made when an entry uses one; each entry a directory bound at its path; snapshots by class, the `hearth` task's (`tasks.md`, 7, 12) |
| `deploy patch`, rewriting a running root's `/etc` | live changes: a prepared slot's delta mounted over the running one, its consumers declared (`tasks.md`, 14; hearth/docs/design/domain/slots/trial.md, 6) |

## 11. Open questions

- **Identity kinds beyond SSH host keys:** the Syncthing
  certificate and DKIM keys could be identity or imported secrets; it
  depends on hearth/docs/design/domain/activator/README.md's operations and on whether DNS records follow
  from facts.
- **The bootstrap variant for containers:** os's `custom` with an
  essential list in a task, or mmdebstrap's `essential` variant; settled
  with containers in M5 (hearth/docs/design/domain/slots/containers.md). It depends too on where
  preseeds load: mmdebstrap runs no essential hook for `custom`, and
  runs it for `essential` (hearth/docs/design/domain/build/packages.md, 5).
- **A lab pool's containers** [M5], which its engine places rather than
  a repository: how their placement, parameters included (3.5), reaches
  the pool host; it depends on M5's pool design.
- **The `incus` module list** for a lab VM booting from its own disk, to
  verify on the VM tier (hearth/docs/design/domain/machine/boot.md, section 4; hearth/docs/design/testing/vm.md).
- **Whether interpolation and `for_each` in declarations are enough,** or
  some declaration needs statements; it depends on converting the
  fleet's 27 tasks, which will show it.
- **Archive formats for software:** whether `zip` is needed beside
  `tar`; it depends on the downloads the fleet's tasks pin.
- **A download over a packaged file:** render cannot see what the
  packages layer holds, so a software path that replaces a packaged
  file is caught, if at all, by the build; it depends on hearth/docs/design/domain/build/README.md's
  scan.
- **A subvolume per entry,** as `"subvolume": true` on an entry that
  needs its own snapshot policy, quota or restore unit: not in the
  design, a possible later addition if a service needs one; it depends
  on what restoring a single entry and qgroups cost in practice (`tasks.md`, 7).
- **Owners in the file tree:** render checks the owners of data,
  secrets, identity and software against declared users and groups with
  fixed ids (`tasks.md`, 6, 11), but a file's owner in the config layer is still
  resolved only by build, which fails on an unknown name. Whether render
  should check those too depends on how often a wrong owner reaches a
  build.
- **`wireguard-key`** waits for keys made in-process, an ask of skein
  (7.3).
- **Several backup peers in one environment** [M6]: which one a target
  sends to, by a setting or by placement; until it is settled, more than
  one target with `backup_peer` is an error (6.7).
- **hearth's fixed uid, 59999,** to verify against the ids the fleet's
  packages and sysusers files take.
- **A static file whose name ends in `.j2`** must be written as a
  template wrapped in `raw`; whether that is enough depends on what
  tasks ship.
- **Live consumers by inference:** whether a unit's `ExecStart=` path
  or its `ConfigurationDirectory=` could name its consumers without a
  `live` entry (`tasks.md`, 14); it depends on how many entries the fleet's tasks
  need once written.
