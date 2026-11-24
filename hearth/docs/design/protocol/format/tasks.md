# The format's tasks

## 1. The declaration

`tasks/<task>/task.json` is one object. Every key is optional and an
unknown key is an error: `{"doc": "…"}` is a valid task that does
nothing.

| Key | Declares | Part (hearth/docs/design/domain/render/spec.md, section 1) | Section |
|---|---|---|---|
| `doc` | one line about the task | | |
| `settings` | the schema of its settings | | 6 |
| `reads` | the settings and exports of other tasks it reads | | `schema.md`, 1.4, `exports.md`, 1.2 |
| `requires` | guards on its settings | | 9 |
| `packages` | packages to install, with their component | packages | 3 |
| `preseeds` | debconf answers | packages | 3 |
| `initramfs` | initramfs modules | packages | 4 |
| `software` | downloads and artifacts | software | 10 |
| `files` | entries of the file tree, with modes and owners | config | 5 |
| `users`, `groups` | accounts, as sysusers | config | 6 |
| `presets` | systemd presets | config | 6 |
| `tmpfiles` | tmpfiles.d lines | config | 6 |
| `scripts` | `customize.sh` and `software.sh`, and their order | config, software | 8 |
| `data` | persistent data: directories in class subvolumes, bound at their paths | data | 7 |
| `secrets` | secret files | secrets | `README.md`, 7.1 |
| `identity` | identity items | secrets | `README.md`, 7.3 |
| `cmdline` | the kernel's command-line parameters | boot | 4 |
| `exports` | facts for the environment's other targets | exports | 8 |
| `checks` [M5] | what healthy means | checks | 11 |
| `bless` | units whose failure does not block a bless | checks | 13 |
| `live` | the units that consume its paths, and what each does when they change with no reboot | live | 14 |

- **`doc` is one line,** a string with no line feed, bounded only by its
  document (`README.md`, 2.2). It enters no part, so editing it changes no spec, as
  editing a `_note` does not.

## 2. Entries

- **Every declaration but `doc`, `settings`, `reads`, `requires`,
  `scripts`, `exports` and `bless` is a map of entries** keyed by a name or a
  path. Any such entry may carry:
  - `if`: an expression (`templates.md`, 4). The entry exists only when it is true.
  - `for_each`: an expression giving a map or a list. The entry stands
    for one entry per element, with `key` and `value` bound; for a list,
    `key` is the index from 0.
- **Keys and string fields are interpolated,** and a list's strings one
  by one: each `{{ expr }}` is replaced by its value, a string or an
  integer in decimal. Any other value, a boolean included, is an
  evaluation error. Statements (`{% %}`), comments (`{# #}`) and the `-`
  and `+` markers (`templates.md`, 2) are not allowed there: `${{ '{#' }}a[@]}` writes
  a shell's `${#a[@]}`. Strings outside entries are taken as written (a
  default, a `doc`, `bless`), but a guard's `message` holds no `{{`, `{%`
  or `{#`, so that a later milestone may interpolate it.
- **A field of another type** (an integer, a boolean, a list) may be
  written as one `{{ expr }}`, whose value must have the field's type.
- **A field that is one `{{ expr }}` alone,** a string field too, is
  absent when its value is undefined (`templates.md`, 6), so the field's default
  applies: `shell` below. Beside text, in a key and in a list's string,
  undefined is an evaluation error.
- **Keys are unique after expansion, within their declaration:** two
  elements of one declaration that make the same key are an error. One
  key in two declarations makes an entry in each, as `hearth` does in
  12's `software` and `users`. What two such entries claim together,
  a `groups` entry of a user's own name (6) or one path, is compared as
  hearth/docs/design/domain/render/README.md 3.7 says. The bound on entries counts every declaration of
  the task (hearth/docs/design/domain/render/README.md, 2.3).
- **Across tasks,** a file path, a data name and a secret's or identity's
  path belong to one task. Users, groups, presets, tmpfiles lines,
  preseeds and boot parameters may be declared by several tasks only
  identically. Packages and modules are a union. Anything else is a
  collision, an error that names both tasks (hearth/docs/design/domain/render/README.md, 3.7).
- Accounts from a setting, in `tasks/users/task.json`:

  ```json
  {
    "users": {
      "{{ key }}": {
        "for_each": "settings.accounts",
        "uid": "{{ value.uid }}",
        "home": "/home/{{ key }}",
        "shell": "{{ value.shell }}",
        "groups": "{{ value.groups }}",
        "password": "{{ value.password }}"
      }
    }
  }
  ```

## 3. Packages, components and preseeds

- **`packages`** is a map keyed by package name, with `if`, `for_each`
  and `component` (enum `main`, `contrib`, `non-free`,
  `non-free-firmware`; default `main`). The packages part requests the
  union over the target's tasks.
- **`component` is a check, never a switch.** A package whose component
  the target's `debian.components` does not enable is an error that names
  both, instead of apt's failure later. Components are a setting; no task
  name and no hardware value turns one on (os's `desktop` task and
  `7th-gen` did).
- **`debian`** is the one schema hearth fixes. It is reserved in every
  document and merges like any object:

  | Field | Type | Default | Meaning |
  |---|---|---|---|
  | `suite` | string | required | `trixie`, `forky` |
  | `architecture` | string | required | `amd64` |
  | `snapshot.archive` | timestamp | required | snapshot.debian.org's timestamp for `debian`, as `20261001T030405Z` |
  | `snapshot.security` | timestamp | required with the security pocket | the timestamp for `debian-security` |
  | `components` | map of component to bool | `{"main": true}` | `main` must stay on |
  | `pockets.updates`, `pockets.security` | bool | true | the `-updates` and `-security` suites |
  | `install_recommends` | bool | false | apt's `Install-Recommends` |

  The archives' URLs are hearth's, fixed:
  `http://snapshot.debian.org/archive/debian/<timestamp>/` and
  `…/debian-security/<timestamp>/`. Where the bytes come from (a proxy,
  the test tier's mirror) is the builder's own configuration, outside
  every spec. os's rule that `forky` has no updates or security lines
  becomes two booleans. Timestamps should be the snapshot's own, since
  snapshot.debian.org redirects any other to the nearest earlier one.
- **`debian`'s default is the bottom layer,** as a task's (`schema.md`, 1.3), so
  `{"contrib": true}` keeps `main` on; `main` written `false` or `null`
  is an error. A target whose layers set no `debian` is an error at its
  base's document. A `snapshot.security` beside a security pocket that
  is off is dropped, as the pocket's line is.
- **Names are Debian Policy's (5.6.1):** a package's name, `suite` and
  `architecture` are lowercase letters, digits, `+`, `-` and `.`, two
  bytes at least, starting with a letter or a digit, and at most 256
  bytes. A timestamp is `YYYYMMDDTHHMMSSZ`, its digits checked, not its
  date. `components`' keys are the four above. Render refuses any other
  where it is written, so no tool reads one.
- **`preseeds`** is a map keyed by debconf question, with `type` (enum
  `select`, `multiselect`, `string`, `boolean`) and `value`, both
  required, and `owner` (default: the question's first segment). There
  is no `password` type: a preseed is in the spec, and a spec holds no
  secret. They are answered before any package is configured (hearth/docs/design/domain/build/packages.md
  ). The timezone
  and locale, which os and the bash fleet set in code, are the `locale`
  task's:

  ```json
  {
    "doc": "The timezone and the locale, answered to debconf before packages are configured.",
    "settings": {
      "timezone": {
        "type": { "object": {
          "area": { "type": "string", "doc": "tzdata's area, such as Etc." },
          "zone": { "type": "string", "doc": "The zone in the area, such as UTC." }
        } },
        "doc": "The system's timezone."
      },
      "locale": { "type": "string", "default": "C.UTF-8", "doc": "The default locale, generated and set." }
    },
    "packages": { "locales": {}, "tzdata": {} },
    "preseeds": {
      "tzdata/Areas": { "type": "select", "value": "{{ settings.timezone.area }}" },
      "tzdata/Zones/{{ settings.timezone.area }}": { "type": "select", "value": "{{ settings.timezone.zone }}" },
      "locales/default_environment_locale": { "type": "select", "value": "{{ settings.locale }}" },
      "locales/locales_to_be_generated": { "type": "multiselect", "value": "{{ settings.locale }} UTF-8" }
    }
  }
  ```

- **A preseed is one line of `debconf-set-selections`** (hearth/docs/design/domain/build/packages.md, 5),
  so render refuses what would break it: a question is two or more
  segments joined by `/`, and an owner one, each of letters, digits,
  `.`, `_`, `+` and `-`; a value holds no control character and does not
  end in `\`, which joins the next line; a `boolean`'s value is `true`
  or `false`. Other values are debconf's to read.

## 4. The initramfs and the command line

- **`initramfs`** is a map keyed by kernel module name, with `if` and
  `for_each`: letters, digits, `_` and `-`, at most 55 bytes, the
  kernel's `MODULE_NAME_LEN` less its NUL. Their union is the
  configuration's `INCLUDE_MODULES`. The
  rest of the configuration is no task's: the packages part carries
  `microcode`, `none` on a VM and `generic` on metal, which render
  derives from placement (hearth/docs/design/domain/machine/boot.md, 4.1), and render writes
  `tiny-initramfs.conf` from the modules and that choice, with
  `AUTO_MODULES=no` always (hearth/docs/design/domain/render/spec.md, 1.2). So neither `yes`, which
  would read the build host, can reach an output (11).
- **`cmdline`** is a map of entries with `param` (the parameter as the
  kernel reads it: printable ASCII but a space, `"` and `'`, and `\`,
  `$`, `;`, `&`, `|`, `<`, `>`, `{`, `}` and `#`, which GRUB's script
  would read in a BIOS slot's fragment, hearth/docs/design/domain/machine/boot.md, 2.4) and `order`. The
  boot part is the parameters in order (`README.md`, 4.3). Ten keys are the
  activator's or would change how the root is found (11). A line that
  would pass 2,048 bytes with the activator's tokens is refused at the
  `param` of the first entry, in order, that takes it past. Boot
  parameters are a slot part, so changing one builds nothing: it is a
  new slot of the same output.
- **Modules by hardware profile** are data, not branches. The bash
  fleet's table, from its `tiny-initramfs.conf` template, becomes the
  default of a setting of the `boot` task:

  ```json
  {
    "doc": "The kernel, its initramfs and its command line.",
    "settings": {
      "kernel": { "type": "string", "default": "linux-image-amd64", "doc": "The kernel package." },
      "params": {
        "type": { "map": { "object": {
          "param": { "type": "string", "doc": "One parameter, as the kernel reads it." },
          "order": { "type": "int", "min": 0, "max": 999, "default": 500, "doc": "Its place on the line." }
        } } },
        "default": {}, "doc": "Command-line parameters, by name."
      },
      "modules": {
        "type": { "map": { "list": "string" } },
        "default": {
          "7th-gen": ["btrfs", "nvme"],
          "provider-b": ["btrfs", "virtio_blk", "virtio", "virtio_pci", "virtio_rng", "scsi_mod", "vmw_pvscsi"],
          "provider-a": ["btrfs", "scsi_mod", "sd_mod", "virtio_scsi", "virtio_pci", "virtio_mmio"],
          "provider-c": ["btrfs", "virtio_blk", "virtio_pci"]
        },
        "doc": "Initramfs modules, by hardware profile."
      }
    },
    "requires": {
      "kernel": { "if": "target.placement.kind != 'container'", "message": "a container has no kernel: its host runs it" },
      "modules": { "if": "target.placement.kind == 'container' or target.placement.hardware in settings.modules", "message": "no initramfs modules for this hardware" }
    },
    "packages": {
      "{{ settings.kernel }}": {},
      "tiny-initramfs": {},
      "efibootmgr": { "if": "target.placement.firmware == 'efi'" }
    },
    "initramfs": { "{{ value }}": { "for_each": "settings.modules[target.placement.hardware]" } },
    "cmdline": {
      "{{ key }}": { "for_each": "settings.params", "param": "{{ value.param }}", "order": "{{ value.order }}" }
    }
  }
  ```

  The lists are the bash fleet's and hearth/docs/design/domain/machine/README.md's (`templates.md`, 2). The `incus`
  profile is not a default: the bash fleet's list served a virtiofs root,
  and hearth's test VMs boot from a disk of their own. `envs/lab.json`
  adds it, with the lab's placement and console:

  ```json
  {
    "_note": "Every lab target: an Incus VM with its own disk, trusting the lab's CA.",
    "placement": { "kind": "vm", "firmware": "efi", "hardware": "incus" },
    "boot": {
      "modules": { "incus": ["btrfs", "virtio_pci", "virtio_scsi", "sd_mod"] },
      "params": { "console": { "param": "console=ttyS0,115200n8", "order": 900 } }
    },
    "certificates": { "extra_ca": { "lab": "lab-ca/lab.crt" } }
  }
  ```

  The `incus` list is to verify on the VM tier (hearth/docs/design/domain/machine/boot.md, 4.2;
  hearth/docs/design/testing/vm.md).

## 5. The file tree

- **`tree/` mirrors `/`.** A file at `tree/<path>` installs at `/<path>`,
  less a `.j2` suffix. Its mode is 0755 if git records it executable, and
  0644 otherwise; a symbolic link in git is a symbolic link, its target
  as written. Owner and group are root. A link's target, in `tree/` or a
  `link` entry, is 1 to 4,095 bytes with no NUL: Linux's `PATH_MAX`
  less its NUL.
- **`files` is a map keyed by absolute destination path.** An entry is
  one of:

  | Entry | Means |
  |---|---|
  | no source field | the file of `tree/` at this path, or with a key ending in `/`, the directory of `tree/` and its subtree |
  | `"template": "templates/NAME.j2"` | that template, rendered to this path |
  | `"copy": PATH` | a repository file, of type `file` (`schema.md`, 1.2), copied byte for byte |
  | `"link": TARGET` | a symbolic link |
  | `"directory": true` | an empty directory, which git cannot hold |

  with the fields `mode` (an octal string, `"0640"`), `owner` and `group`
  (names, default `root`), `if` and `for_each`.
- **Forms:** a key ends in `/` exactly when it names a directory of
  `tree/`, and then takes no source field, `directory` included.
  `directory` is `true` or absent. A `template` or `copy` entry's mode
  is 0644 unless it gives one, whatever git records of its source. A
  `template` is written as is, never interpolated, so that `README.md`, 2.1's check
  that every template is used stays exact.
- **A directory's entry governs its subtree:** its `if` covers every file
  beneath it; its `owner` and `group` apply, each alone, to entries
  beneath that do not give that field; and its `mode` is the
  directory's. A `tree/` path that an entry names is installed only
  where the entry is: a false `if` leaves it out, a file or a subtree.
- **An entry for a path that `tree/` lacks is an error,** as is a path
  that both `tree/` and a `template`, `copy` or `link` entry give.
- **Owners are names.** Build resolves them in the layer's own
  `/etc/passwd` and `/etc/group`, once the users are applied
  (hearth/docs/design/domain/build/README.md, section 6); a name it cannot resolve fails the build.
  `root` is always id 0.
- **A file replaces whatever the packages put at its path,** mode and
  owner included. os kept a packaged file's mode, so a task could not
  tighten a conffile.
- **Directories a path implies** are created 0755 and root-owned, only
  where the layers beneath lack them; a declared directory sets its own.
- **Under `/usr`, only `/usr/local`.** The rest of `/usr` is the
  packages layer's, as it was under os, whose tasks' helpers went to
  `/usr/local/bin` and their shared files to `/usr/local/share`. A unit
  a task ships goes in `/etc/systemd/system/`. So nothing above the
  packages changes what every process maps from `/usr`, and a live
  change overlays `/usr/local` alone of it (hearth/docs/design/domain/slots/trial.md, 6). The build
  holds scripts to the same rule (hearth/docs/design/domain/build/README.md, 7.3).
- **Forbidden destinations:** anything under a data path or a class's
  mount point (7), which a mount would hide, and the paths hearth
  reserves (11).
- One file per user, from a map setting, in `tasks/ssh/task.json`:

  ```json
  {
    "files": {
      "/etc/ssh/sshd_config.d/local.conf": { "if": "settings.allow" },
      "/etc/ssh/authorized_keys_{{ key }}": {
        "for_each": "settings.authorized",
        "if": "key in settings.allow",
        "template": "templates/authorized_keys.j2"
      }
    }
  }
  ```

  with `templates/authorized_keys.j2`:

  ```jinja
  {% for name, line in value | items %}
  {{ line }}
  {% endfor %}
  ```

## 6. Users, groups, presets and tmpfiles

- **`users`** is a map keyed by user name. Build writes them as sysusers
  records and applies them in the config layer (hearth/docs/design/domain/build/README.md, section 6).

  | Field | Type | Default |
  |---|---|---|
  | `uid` | int | a system id that sysusers picks |
  | `gid` | int | none; a package user's alone: its primary group's id, in the line build seeds (a sysusers user's group is `group`) |
  | `group` | string | the user's own group, made with the same id |
  | `gecos` | string | empty |
  | `home` | path | `/` |
  | `shell` | path | `/usr/sbin/nologin` |
  | `groups` | list of strings | none: supplementary groups, as sysusers `m` lines |
  | `password` | secret | none: the account stays locked (`README.md`, 7.2) |
  | `subordinate` | `{ "base": int, "count": int }` | none: a subordinate uid and gid range |
  | `from` | enum `sysusers`, `package` | `sysusers`: made by build; `package`: made by a package's maintainer script |

- **A user its packages create** is declared with `"from": "package"`,
  with no other field but `uid`, `gid`, `if` and `for_each`. Render knows
  the name and checks it as it checks a declared user, but never writes
  it: build writes no sysusers line for it, and the activator confirms it
  exists in the slot's `/etc/passwd` at prepare (hearth/docs/design/domain/slots/prepare.md, 7). A name
  declared by two tasks, once from a package and once for sysusers, is a
  collision.
- **`groups`** is a map keyed by group name, with `gid` and `from`, as
  for users: a group a package creates is declared with
  `"from": "package"`. Like a package user, build writes no sysusers
  line for it, and the activator confirms it exists in the slot's
  `/etc/group` at prepare (hearth/docs/design/domain/slots/prepare.md, 7).
- **Persistent owners have fixed ids.** A package's postinst, and
  sysusers for a user with no `uid`, take the next free id when they
  install, and the packages layer is rebuilt from scratch: a snapshot
  bump could renumber `postfix` while its data keeps the old ids. So
  every user or group that owns a data path (7), a secret file or an
  identity file (`README.md`, section 7) has a fixed id, package users included, and
  render refuses one that has none. Build seeds the package users' and
  groups' lines into `/etc/passwd` and `/etc/group` before any package
  installs, so adduser finds them (hearth/docs/design/domain/build/packages.md). In the mail
  task:

  ```json
  {
    "users": { "postfix": { "from": "package", "uid": 107, "gid": 113 } },
    "groups": { "opendkim": { "from": "package", "gid": 120 } }
  }
  ```

  The ids are the task's, fixed once and the same on every machine.
  Every machine is installed fresh (README, `templates.md`), so they need
  match no os machine's.
- **`subordinate` ranges** are the one thing sysusers cannot make. Render
  writes `/etc/subuid` and `/etc/subgid` from them into the file tree,
  and refuses two ranges that overlap; no task ships either file.
- **Ids are one user's:** two users or two groups with one id are an
  error.
- **A sysusers user's own group,** what an unset `group` names, has the
  user's name and the user's `uid` as its id. A `groups` entry of that
  name is that group, so its `gid` is the user's `uid`, and another is a
  collision (hearth/docs/design/domain/render/README.md, 3.7): user `alice` with `uid` 1000 and group
  `alice` with `gid` 1001 differ. A user whose primary group has another
  id names a group of another name in `group`.
- **A package user gives both `uid` and `gid`, or neither:** build seeds
  its passwd line from both (hearth/docs/design/domain/build/packages.md, 5), and one alone is an `ids`
  error (hearth/docs/design/domain/render/errors.md, 2). With neither, nothing is seeded, and the
  user may own nothing that needs a fixed id (above).
- **Ids and strings are bounded:** a `uid` or `gid` is 1 to 65533: 0 is
  root's, 65534 nobody's and 65535 no id, and the builder's namespace
  maps no more (hearth/docs/design/domain/store/digests.md, 1.4). A `subordinate` range has a count of 1
  at least and a base of 65,536 at least, so that no id a target's
  files own is subordinate, and base plus count is at most
  4,294,967,295. `gecos`, `home` and `shell` hold no `:`, which splits a
  passwd line, no `"` or `\`, which sysusers reads as quoting, and no
  control character. Render refuses any other where it is written.
- **`presets`** is a map keyed by unit name, with `action`, `enable` or
  `disable`. Build applies them with the target's own systemctl
  (hearth/docs/design/domain/build/README.md, section 6). A unit is masked by a `link` to `/dev/null` in
  `files`.
- **`tmpfiles`** is a map keyed by path, with `type` (enum `d`, `D`, `e`,
  `f`, `f+`, `w`, `w+`, `L`, `L+`, `C`, `z`, `Z`, `r`, `R`), `mode`,
  `owner`, `group`, `age` and `argument`, as tmpfiles.d(5) reads them.
  `v`, `q` and `Q` are reserved, so a line of one is refused as such
  (11), not as an unknown type; any other letter is a schema error.
  Build applies them to the image and writes them for every boot
  (hearth/docs/design/domain/build/README.md, 6.4), which is what directories inside `/var` and the
  data entries need. Lines that make subvolumes, lines that write over
  what is there outside `/var`, and lines for a path under `/usr` but
  `/usr/local`, are refused (11).
- **Homes live on persistent data:** the users task declares each
  account's home as a `data` entry of its own (`tasks.md`, 7, `schema.md`, 1.6).

## 7. Data

- **Persistent data lives in classes.** A target has up to three class
  subvolumes at the btrfs top level, `@<target>.core`, `@<target>.data`
  and `@<target>.cache`, each made only when some entry uses it, and
  mounted at `/srv/core`, `/srv/data` and `/srv/cache`
  (hearth/docs/design/domain/machine/README.md, section 3). They outlive every slot. The classes are os's
  and the bash fleet's, whose reason was a snapshot policy per class:

  | Class | Holds | Snapshots and backups (12) |
  |---|---|---|
  | `core` | the machine's own state: the journal, iwd's networks | its own policy, typically daily |
  | `data` | services' data and people's homes: a mail spool, a database, `/home` | its own policy, typically hourly; the class backed up [M6] |
  | `cache` | what can be made again: downloads, build caches | never snapshotted, never backed up |

- **`data` is a map keyed by name; each entry is a directory** in its
  class's subvolume, named by the entry's name (`/srv/core/journal`,
  `/srv/data/postfix`), and bind-mounted at the path the task declares,
  from the fstab the activator writes (hearth/docs/design/domain/machine/README.md, 3.3). An entry never
  names a subvolume.

  | Field | Type | Default |
  |---|---|---|
  | `path` | path | required |
  | `class` | enum `core`, `data`, `cache` | `data` |
  | `mode` | mode | `0755` |
  | `owner`, `group` | name, with a fixed id (6) | `root` |

- **An entry whose path is its class's directory,** `/srv/<class>/<name>`,
  needs no bind: the class's mount shows it. That is its own class and
  name only: `/srv/core/journal` for an entry `journal` of class `core`.
  Any other path at or beneath `/srv/core`, `/srv/data` or `/srv/cache`,
  another entry's directory or a mount itself, is refused.
- **The activator makes the class subvolume** when the top level lacks
  it, with a root of `0:0 0755`, and **the entry's directory** when the
  class lacks it, seeded once from the output's content at the entry's
  path (hearth/docs/design/domain/slots/prepare.md, 3), so what packages put there (a database's
  cluster, a mail spool) is not hidden by an empty bind. The data part
  holds names, classes and paths, a slot part that enters no layer key.
- **Mode and owner are applied at every boot:** render turns them into a
  `tmpfiles` line of type `d` at the entry's path, which systemd applies
  in the running slot, through the bind, where names resolve. So they,
  unlike the path and the class, enter the config key.
- **Paths** are beneath `/srv`, `/var` or `/home`, never one of the three
  itself, whose bind would hide the rest of it, never `/var/lib/hearth`
  or beneath it, and never beneath another entry's path. A target has at
  most 64 entries (hearth/docs/design/domain/machine/README.md, 3.4): the 65th, in name order (`README.md`, 4.3), is a
  schema error at its key. The rest of `/` stays immutable.
- **Names are per target, across classes:** two entries of one target
  with one name are a collision (hearth/docs/design/domain/render/README.md, 3.7), even in two classes.
  Moving an entry to another class moves no data, so the activator
  refuses it while the old directory exists (hearth/docs/design/domain/machine/README.md, 3.4).
- **Snapshots and backups are by class, never by entry:** their policies
  are the `hearth` task's settings (12). A snapshot is of a whole
  class subvolume, so it is atomic across every service in the class,
  postfix's spool and its mailboxes together; one entry is restored by
  copying its directory out of a snapshot (hearth/docs/design/domain/operating.md, 7.4).
- **Data keeps a program's default path.** An entry is bound where the
  program already looks, as Debian ships it, so no configuration changes
  for it: journald writes `/var/log/journal`, iwd keeps its networks in
  `/var/lib/iwd`. os did the same, binding the directories
  `/srv/core/journal` and `/srv/core/iwd` of its `@<target>.srv.core`
  subvolume onto those paths; hearth keeps the classes and the binds,
  and declares them per entry.
- The journal, which the bash fleet and os bound from a hard-coded
  `@<target>.srv.core/journal`, is the `core` task's:

  ```json
  {
    "data": {
      "journal": {
        "if": "target.placement.kind != 'container'",
        "class": "core", "path": "/var/log/journal", "mode": "2755", "group": "systemd-journal"
      }
    },
    "groups": { "systemd-journal": { "from": "package", "gid": 999 } }
  }
  ```

  The group owns persistent data, so its id is fixed (6); 999 is
  what systemd's sysusers gives it on the fleet's machines today, to
  verify. A container has no `journal` entry: its journal lives in its
  host's (`targets.md`, 1.5) [M5].

- iwd's state, its known networks and their passphrases, which os bound
  from `/srv/core/iwd` only when the Wi-Fi task was present, is the
  `wifi` task's, at iwd's own default path:

  ```json
  {
    "doc": "Wi-Fi through iwd, its known networks kept across slots.",
    "packages": { "iwd": {} },
    "data": { "iwd": { "class": "core", "path": "/var/lib/iwd", "mode": "0700" } }
  }
  ```

  iwd runs as root, so the root-owned default needs no fixed id; the
  mode keeps the passphrases from other users.
- A mail server's spool and mailboxes are the `mail` task's, in the
  `data` class, so one snapshot holds both as they were at one instant:

  ```json
  {
    "data": {
      "postfix": { "path": "/var/spool/postfix" },
      "mail": { "path": "/var/mail", "mode": "2775", "group": "mail" }
    },
    "groups": { "mail": { "from": "package", "gid": 8 } }
  }
  ```

  The spool's directory is root's, as Debian ships it, and the queues
  beneath keep the owners postfix gave them; `mail` is base-passwd's
  static group.
- **On x1-prod** the entries make `@x1-prod.core`, with `journal` and
  `iwd`, and `@x1-prod.data`, with `home-alice` (`schema.md`, 1.6), bound at
  `/var/log/journal`, `/var/lib/iwd` and `/home/alice`; neither journald
  nor iwd is configured to find them. Its fstab is hearth/docs/design/domain/machine/README.md, 3.3's
  example.

## 8. Escape hatches

- **`scripts` names the task's shell fragments and their order:**

  ```json
  { "scripts": { "customize.sh": { "order": 300 }, "software.sh": { "order": 500 } } }
  ```

  A declared script must exist in the task directory as `NAME` or
  `NAME.j2`, and a script file that is not declared is an error. Its
  keys are those two alone (`README.md`, 2.1), each an object of an optional `order`
  (`README.md`, 4.3). `scripts` is not a map of entries (2): it is decoded as
  written, so a third key is a decode error at the key, and an `order`
  that is not an integer from 0 to 999 one at its value (hearth/docs/design/domain/render/errors.md, 2).
- **`customize.sh`** runs in the config layer, after its files and users
  are in place (hearth/docs/design/domain/build/README.md, section 6). **`software.sh`** runs in the
  software layer, after every download is placed at its declared path
  (hearth/docs/design/domain/build/README.md, section 5), so it names a download by that path instead of
  guessing a cache's layout, as os's examples did and got wrong.
- **Fragments run in order** (`README.md`, 4.3), each as its own `sh -eu` script, so
  an error names its task and no `cd` or `set` leaks into the next one.
  os concatenated them in alphabetical order of task.
- **They run in the step's sandbox, offline** (hearth/docs/design/domain/build/sandboxes.md), and
  their output is part of the layer like any file.
- **They are the exception.** What the bash fleet's fragments did (users,
  modes, symbolic links, software unpacked into `/opt`) is declarative
  here. What remains for them: `postmap`, `update-ca-certificates`, and
  the like.

## 9. Requires

- **`requires`** is a map of guards, each named as `README.md`, 2.4 says and with `if`
  and `message`. Render evaluates them after validation and before any
  entry; a false guard is an error carrying its message and its line in
  `task.json`.
- Most of os's guards are types now (`"type": {"enum": [...]}`).
  `requires` is for what spans settings or reads placement: the boot
  task refuses a container, and its module table must hold the target's
  hardware profile (4).

## 10. From the fleet's code to task data

| Detail in code | As data |
|---|---|
| `non-free` for `host.hardware == "7th-gen"` (os, bash `manifest`) | `debian.components` in `hosts/x1/x1-prod.json`, and the `firmware` task checks it (below) |
| Etc/UTC and en_US preseeds (os's `install.rs`, bash's `deploy`) | the `locale` task's preseeds (3); the values in `fleet.json` (`targets.md`, 1.7) |
| the persistent `journal` and `iwd` directories (os's `provision`) | `data` entries of `core` and `wifi` (7) |
| gettys masked in containers (os's and bash's `deploy`) | `files` links in `core`, conditional on placement (below) |
| `domain=example.org` (bash's `manifest`) | the `core` task's `domain` setting, set in `fleet.json` (`targets.md`, 1.7) |
| SSH port 2222 (bash's ssh templates) | the `ssh` task's `port` setting, default 22, set in `fleet.json` (`targets.md`, 1.7), read as `Port {{ settings.port }}` in its sshd drop-in |

- **Firmware for x1's hardware.** `tasks/firmware/task.json`:

  ```json
  {
    "doc": "Firmware for Intel 7th-generation laptops.",
    "packages": {
      "firmware-intel-sound": { "component": "non-free-firmware" },
      "firmware-iwlwifi": { "component": "non-free-firmware" },
      "firmware-linux-free": {},
      "firmware-sof-signed": { "component": "non-free-firmware" }
    }
  }
  ```

  x1-prod has it, with `non-free-firmware` on, in its base-env file
  (`targets.md`, 1.7). The bash fleet also switched on `non-free`, which none of these
  packages needs.
- **Gettys in containers,** masked by links in `tasks/core/task.json`:

  ```json
  {
    "files": {
      "/etc/systemd/system/{{ value }}": {
        "for_each": "['getty@.service', 'serial-getty@.service', 'console-getty.service']",
        "if": "target.placement.kind == 'container'",
        "link": "/dev/null"
      }
    }
  }
  ```

## 11. What hearth reserves

Moved to [`tasks-reserved.md`](tasks-reserved.md), section 1.

## 12. The hearth task

Moved to [`tasks-hearth.md`](tasks-hearth.md), section 1.

## 13. Units that may fail

- **A task may name units whose failure does not block a bless,** such as
  a desktop's flaky units:

  ```json
  { "bless": { "ignore_failed": ["foo.service"] } }
  ```

- **The target's list is the union over its tasks,** in the checks part
  (hearth/docs/design/domain/render/spec.md, 1.4), which holds it from M3. The minimal bless counts only
  the failed units that are not on it (hearth/docs/design/domain/slots/bless.md, 1.1).
- **Before M3, which brings the bless,** render checks each list and
  places it in no part. Unlike a check (`README.md`, section 9), the list only
  relaxes: with no bless, no failure is ignored, so nothing it declares
  is dropped silently.
- **A failed unit on the list is still reported** as a fact
  (hearth/docs/design/domain/operating.md, section 4); it only stops counting against a bless.
- **A container's list holds as a host's [M5]:** its host reads the
  container's failed units from outside during its trial
  (hearth/docs/design/domain/slots/containers.md, 3).
- **A container never counts against its host [M5]:** the host's
  minimal bless leaves out its containers' nspawn units without any
  list, since each container's slot has its own trial (hearth/docs/design/domain/slots/bless.md, 1.1).

## 14. Live changes

Moved to [`tasks-live.md`](tasks-live.md), section 1.
