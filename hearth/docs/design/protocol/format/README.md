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

- [`targets.md`](targets.md): composition, placement, containers and clusters.
- [`schema.md`](schema.md): settings types, defaults, reads and errors.
- [`exports.md`](exports.md): how targets declare and read shared data.
- [`tasks.md`](tasks.md): task declarations and their entries.
- [`tasks-reserved.md`](tasks-reserved.md): what hearth reserves.
- [`tasks-hearth.md`](tasks-hearth.md): what the built-in `hearth` task declares.
- [`tasks-live.md`](tasks-live.md): live changes and their consumers.
- [`templates.md`](templates.md): the template language.

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
  type `file` names it (`schema.md`, 1.2). Every path in the revision is UTF-8; one
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
  `mutable`, whatever its value (`targets.md`, 1.4), a download's `artifact` (8.2), a
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
| target | made from them (`targets.md`, 1.1), at most 63 bytes | no dot; never `hearth` |
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

Moved to [`targets.md`](targets.md), section 1.

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
  (`targets.md`, 1.2), but for `placement`, which a layer replaces whole (`targets.md`, 1.4). Objects merge key by key; `null` deletes; scalars and lists
  replace. The result never holds `null`, and no setting can: the schema
  has no null type, and an absent optional setting is simply absent.
- **A `null` a merge would carry is an error where it is written:** one
  inside a list (`[1, null]`, `[{"a": null}]`), which replaces whole, is
  a `decode` error in a document (hearth/docs/design/domain/render/errors.md, 2) and a bad default in a
  `task.json` (`schema.md`, 1.5); one in a placement is a wrong type (`targets.md`, 1.4).
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

Moved to [`schema.md`](schema.md), section 1.

## 6. Exports

Moved to [`exports.md`](exports.md), section 1.

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
  `hearth` belongs to, such as `systemd-journal` (`tasks-reserved.md`, 1).
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
  half as a fact (`exports.md`, 1.4).
- **`identity` is a map keyed by item name,** each with `kind`, its own
  fields, `path`, an optional `public_path`, `mode` (default `0600`),
  `owner` and `group` (fixed ids, `tasks.md`, 6; never `hearth`'s, `tasks-reserved.md`, 1):

  | Kind | Fields | Public half | Milestone |
  |---|---|---|---|
  | `ssh-host-key` | `algorithm`: `ed25519`, `ecdsa` or `rsa` | the `.pub` line | M3: made with the running slot's `ssh-keygen`, which the hearth task installs (`tasks-hearth.md`, 1) |
  | `wireguard-key` | none | the base64 public key | [later]: a target's first slot cannot be prepared with a `wg` only that slot installs, so it waits for keys made in-process, an ask of skein |
  | `backup-key` | none; no `path`: the activator keeps it and writes it nowhere | the `age1…` recipient | [M6]: an X25519 key made with `age-keygen` (hearth/docs/design/domain/activator/operations.md, 7; `exports.md`, 1.7) |

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
  error. hearth's own binary is `/usr/local/bin/hearth` (`tasks-hearth.md`, 1;
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
  reading it as they read exports (`exports.md`, section 1).
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
| `hosts/<host>/<host>-<env>.json` over a base | fleet, base, member, environment, base in the environment (`targets.md`, 1.2) |
| containers in `hosts/<host>/machines/`, alone, with no environment | containers are targets; placement names their host (`targets.md`, 1.5) |
| `--overlay` | gone: every input is in the repository |
| lists concatenate; nothing can be removed | RFC 7386; collections are maps; `null` removes (section 4) |
| a top-level key is a task, case-insensitively, in any file | presence from base and base-env layers only, exact names (`targets.md`, 1.3) |
| `_`-prefixed keys skipped for tasks, kept in the context | `_note` dropped when decoded (2.2) |
| reserved `suite`, `arch`, `snapshot` | `debian` (`tasks.md`, 3) |
| `container`, `desktop`, `7th-gen` steer the build | placement, and `debian.components` (`targets.md`, 1.4, `tasks.md`, 3) |
| kind, host and env inferred from a hyphen | names made from files, never parsed (`targets.md`, 1.1) |
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
| `copy` from a workspace | `file` settings, from the repository (`schema.md`, 1.2) |
| variant maps (never implemented) | maps of objects |
| hard-coded preseeds, journal and iwd, gettys, snapshot host | task data (`tasks.md`, 10) |
| `@T.srv`, `@T.srv.core`, `@T.srv.data`, `@T.srv.cache` made at format time, with `journal` and `iwd` bound by fstab's template; snapper for snapshots | the classes `@<target>.core`, `.data` and `.cache`, made when an entry uses one; each entry a directory bound at its path; snapshots by class, the `hearth` task's (`tasks.md`, 7, `tasks-hearth.md`, 1) |
| `deploy patch`, rewriting a running root's `/etc` | live changes: a prepared slot's delta mounted over the running one, its consumers declared (`tasks-live.md`, 1; hearth/docs/design/domain/slots/trial.md, 6) |

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
  a repository: how their placement, parameters included (`targets.md`, 1.5), reaches
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
  fixed ids (`tasks.md`, 6, `tasks-reserved.md`, 1), but a file's owner in the config layer is still
  resolved only by build, which fails on an unknown name. Whether render
  should check those too depends on how often a wrong owner reaches a
  build.
- **`wireguard-key`** waits for keys made in-process, an ask of skein
  (7.3).
- **Several backup peers in one environment** [M6]: which one a target
  sends to, by a setting or by placement; until it is settled, more than
  one target with `backup_peer` is an error (`exports.md`, 1.7).
- **hearth's fixed uid, 59999,** to verify against the ids the fleet's
  packages and sysusers files take.
- **A static file whose name ends in `.j2`** must be written as a
  template wrapped in `raw`; whether that is enough depends on what
  tasks ship.
- **Live consumers by inference:** whether a unit's `ExecStart=` path
  or its `ConfigurationDirectory=` could name its consumers without a
  `live` entry (`tasks-live.md`, 1); it depends on how many entries the fleet's tasks
  need once written.
