# The format's tasks

## 1. The declaration

`tasks/<task>/task.json` is one object. Every key is optional and an
unknown key is an error: `{"doc": "…"}` is a valid task that does
nothing.

| Key | Declares | Part (hearth/docs/design/domain/render/spec.md, section 1) | Section |
|---|---|---|---|
| `doc` | one line about the task | | |
| `settings` | the schema of its settings | | 6 |
| `reads` | the settings and exports of other tasks it reads | | `README.md`, 5.4, 6.2 |
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
- **`debian`'s default is the bottom layer,** as a task's (`README.md`, 5.3), so
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
  | `"copy": PATH` | a repository file, of type `file` (`README.md`, 5.2), copied byte for byte |
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
  maps no more (hearth/docs/design/domain/store/README.md, 3.4). A `subordinate` range has a count of 1
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
  account's home as a `data` entry of its own (`tasks.md`, 7, `README.md`, 5.6).

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
  host's (`README.md`, 3.5) [M5].

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
  `iwd`, and `@x1-prod.data`, with `home-alice` (`README.md`, 5.6), bound at
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
| Etc/UTC and en_US preseeds (os's `install.rs`, bash's `deploy`) | the `locale` task's preseeds (3); the values in `fleet.json` (`README.md`, 3.7) |
| the persistent `journal` and `iwd` directories (os's `provision`) | `data` entries of `core` and `wifi` (7) |
| gettys masked in containers (os's and bash's `deploy`) | `files` links in `core`, conditional on placement (below) |
| `domain=example.org` (bash's `manifest`) | the `core` task's `domain` setting, set in `fleet.json` (`README.md`, 3.7) |
| SSH port 2222 (bash's ssh templates) | the `ssh` task's `port` setting, default 22, set in `fleet.json` (`README.md`, 3.7), read as `Port {{ settings.port }}` in its sshd drop-in |

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
  (`README.md`, 3.7). The bash fleet also switched on `non-free`, which none of these
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

What a later step would refuse, render refuses first, at the line that
declares it (hearth/docs/design/domain/render/errors.md, 2):

| Reserved | Why | Defined in |
|---|---|---|
| command-line keys `root`, `rootflags`, `rootfstype`, `rootwait`, `rootdelay`, `ro`, `rw`, `init`, `initrd`, `panic`, and every `hearth.*` key (the trial's token, hearth/docs/design/domain/slots/trial.md, 3); a line past 2048 bytes with the activator's tokens | the activator's, or they change how tiny-initramfs finds and starts the root | hearth/docs/design/domain/machine/boot.md, 3.2 |
| task files at `/etc/fstab`, anything under `/.hearth/`; in `/etc/systemd/system.conf.d/`, a name beginning `zz-hearth` or sorting after `zz-hearth-watchdog.conf` | the activator writes them at prepare, and the watchdog's drop-in must sort last | hearth/docs/design/domain/slots/prepare.md, 5 |
| units, links and presets named `hearth-trial*`; `hearth-check-*` | the dead-man's; checks' transient units | hearth/docs/design/domain/slots/prepare.md, 5; hearth/docs/design/domain/operating.md, 2.4 |
| [M5] presets and links that enable `systemd-nspawn@<c>.service` for a container `c` placed on the target | the host's activator starts each container once its link and settings exist; one started before fails | hearth/docs/design/domain/slots/containers.md, 2 |
| task files at `/etc/passwd`, `/etc/group`, `/etc/shadow`, `/etc/gshadow`, `/etc/subuid`, `/etc/subgid`, `/etc/machine-id`, `/var/lib/dbus/machine-id` | users are declared (6); identity is written at prepare | hearth/docs/design/domain/build/README.md, 6.3 and 7.2 |
| task files at `/etc/ssh/ssh_host_*_key` and their `.pub`, `/etc/dropbear/dropbear_*_host_key`, anything under `/etc/ssl/private/`, `/var/lib/systemd/random-seed`, `/var/lib/systemd/credential.secret` | identity and secrets: `identity` and `secrets` entries name these paths, files never do | hearth/docs/design/domain/build/README.md, 7.2 |
| anything under `/var/lib/hearth/` | the mount points of hearth's state and store | hearth/docs/design/domain/machine/README.md, 3.4 |
| task files, links, downloads and tmpfiles lines under `/usr` but `/usr/local` | the rest of `/usr` is the packages layer's: nothing above it may change what every process maps, and live changes overlay `/usr/local` alone | 5; `README.md`, 8.1; hearth/docs/design/domain/build/README.md, 7.3; hearth/docs/design/domain/slots/trial.md, 6 |
| anything under `/srv/core`, `/srv/data` or `/srv/cache` but a data entry at its own `/srv/<class>/<name>` (7) | the class subvolumes' mounts hide what the slot holds there | 7; hearth/docs/design/domain/machine/README.md, 3.4 |
| anything under `/dev`, `/proc`, `/sys`; anything but a tmpfiles line under `/run`, `/tmp`, `/var/tmp`; a file under `/var/log` that is not empty; `/etc/.git`, `/etc/.etckeeper` | runtime, noise, history; a tmpfiles line there waits for boot (hearth/docs/design/domain/build/README.md, 6.4) | hearth/docs/design/domain/build/README.md, 7.2 |
| a task file at `/etc/tiny-initramfs/tiny-initramfs.conf`, so `AUTO_MODULES=yes` and `MICROCODE=yes` cannot reach an output | the packages part's, written before the kernel installs | hearth/docs/design/domain/build/packages.md, 5; hearth/docs/design/domain/machine/boot.md, 4.1 |
| tmpfiles `v`, `q`, `Q`; `w`, `w+`, `f+`, `L+` outside `/var`, `/run` and `/tmp` | a nested subvolume breaks a layer; a write on the read-only root fails at boot | hearth/docs/design/domain/build/README.md, 6.4 |
| preseeds of type `password` | they leave the password in `/var/cache/debconf/passwords.dat` | hearth/docs/design/domain/build/README.md, 7.2 |
| a secret or identity file owned by user `hearth`, or by a group `hearth` belongs to (`systemd-journal`); a secret's or identity item's `mode` with a bit outside `0770` | neither the steward nor its builder may ever read a secret or a private key | hearth/docs/design/domain/activator/README.md, section 2; `README.md`, 7.1; `README.md`, 7.3 |
| an owner of a data path, a secret or an identity file without a fixed id; a download's owner other than `root` or a package user with a fixed id | ids that move between builds; no sysusers user exists when software is placed | 6; `README.md`, 8.1 |
| `live` entries that name a never-live unit or path (14); a `unit` with action `none`; a path not beneath `/etc`, `/usr/local` or `/opt` | a never-live unit must not be acted on live, and a change elsewhere is never live | 14; hearth/docs/design/domain/slots/trial.md, 7 |
| presets and links that enable `systemd-sysext.service` or `systemd-confext.service`; anything under `/etc/extensions`, `/usr/lib/confexts`, `/usr/local/lib/confexts`, `/var/lib/extensions` or `/var/lib/confexts` | nothing may merge extensions over hearth's live overlays, at boot or after a soft reboot, a container's included, which has no `hearth` task to disable them | 12; hearth/docs/design/domain/slots/trial.md, 6 |

- **A row's paths hold for whatever is placed at a path:** a `tree/`
  file, a `files` entry or link, a download, a secret or identity file,
  a tmpfiles line and a data path, whatever the row's wording, "task
  files at" or "anything under". "Under" a path is that path and every
  path beneath it. Two rows leave a kind out: secrets and identity are
  what the identity and secrets row's paths are for, and a tmpfiles line
  may stand under `/run`, `/tmp` and `/var/tmp`. So a data path at
  `/var/lib/dbus/machine-id`, or a tmpfiles `f` line at
  `/var/lib/systemd/random-seed`, is refused.
- **Above a reserved path, only a directory may stand:** a `tree/`
  directory, a `directory` entry, or a tmpfiles line of type `d`, `e` or
  `z` without an `age`, which act on the directory alone. Anything else
  at a strict ancestor of a path a row names, or at the directory a
  row's pattern matches in (`/etc/ssh`, for `ssh_host_*_key`), is
  refused under that row: a data path, whose bind would hide it; a
  file, a link, a secret or an identity file, which would put something
  other than a directory there; a download, whose archive may hold it;
  and any other
  tmpfiles line, which may act beneath its path (`D`, `R`, `Z`, `C`, an
  `age`).
- **A tmpfiles line names one path:** its key holds no `*`, `?` or `[`,
  which tmpfiles reads as a glob, and no `%`, which it reads as a
  specifier, so that each row checks it as written.

## 12. The hearth task

- **Every target has the `hearth` task,** but a container (below),
  without naming it: render adds it, and `"hearth": null` is an error.
  In any layer, `fleet.json` and an environment's included, it is a
  `reserved` error at the key (hearth/docs/design/domain/render/errors.md, 2), and no other kind
  reports it.
  Every such output then carries hearth's binary, its activator's unit
  (which starts the steward, which starts builders) and its user, which
  hearth/docs/design/domain/machine/README.md (`README.md`, 6.3) and hearth/docs/design/domain/slots/README.md assume. Its files are in the repository
  like any task's, `tasks/hearth/`, so the release a fleet runs is
  pinned by a change. A revision without them fails each target that
  has the task, as any task that does not exist does (`README.md`, 3.3; hearth/docs/design/domain/render/errors.md,
  2), and its containers render (hearth/docs/design/domain/render/README.md, 3.4).
- **But a container's:** render adds no `hearth` task to a
  container's target, and naming it there, in its base, member or
  base-env layer (`README.md`, 3.3), is a `reserved` error at the key. The `hearth`
  settings of `fleet.json` and of an environment reach no container,
  since it lacks the task (`README.md`, 3.3). No hearth runs in
  a container [M5] (hearth/docs/design/domain/slots/containers.md, 5): its host's builder builds its output and
  its host's activator prepares and starts it, so an activator enabled
  inside would only fail, and the tools, the user and the range would
  serve nothing. Should a container run a hearth of its own, this
  exception goes.
- `tasks/hearth/task.json`:

  ```json
  {
    "doc": "hearth itself: its binary, its activator's unit, its user, its tools and its alarms.",
    "settings": {
      "alarms": {
        "type": { "map": { "object": {
          "warning": { "type": "int", "optional": true, "doc": "The threshold of a warning." },
          "critical": { "type": "int", "optional": true, "doc": "The threshold of a critical alarm." }
        } } },
        "default": {
          "disk_free_percent": { "warning": 10, "critical": 5 },
          "disk_free_gib": { "warning": 5, "critical": 1 },
          "memory_available_percent": { "warning": 10, "critical": 5 },
          "pressure_some_percent": { "warning": 20 },
          "pressure_full_percent": { "critical": 10 }
        },
        "doc": "Resource alarms' thresholds (../../domain/operating.md, section 4)."
      },
      "snapshots": {
        "type": { "map": { "object": {
          "every": { "type": "duration", "doc": "How often the class is snapshotted." },
          "keep": { "type": { "map": "int" }, "doc": "How many to keep per bucket: hourly, daily, weekly, monthly." }
        } } },
        "default": {},
        "doc": "Local snapshots of each class's subvolume, by class: core or data [M5] (../../domain/operating.md, 7.1)."
      },
      "backups": {
        "type": { "map": { "object": {
          "every": { "type": "duration", "doc": "How often the class is sent to the backup peer." },
          "keep": { "type": { "map": "int" }, "optional": true, "doc": "What the peer keeps, per bucket; the class's local keep if absent." }
        } } },
        "default": {},
        "doc": "Backups of each class to the environment's backup peer, by class [M6] (../../domain/operating.md, 7.2)."
      }
    },
    "packages": { "git": {}, "mmdebstrap": {}, "uidmap": {}, "openssh-client": {} },
    "software": {
      "hearth": {
        "url": "https://git.example.org/ai/hearth/releases/download/v0.1.0/hearth-x86_64-linux",
        "sha256": "…", "unpack": "file", "path": "/usr/local/bin/hearth", "mode": "0755"
      }
    },
    "users": {
      "hearth": {
        "uid": 59999, "home": "/nonexistent", "groups": ["systemd-journal"],
        "subordinate": { "base": 262144, "count": 65536 }
      }
    },
    "presets": {
      "hearth-activator.service": { "action": "enable" },
      "systemd-sysext.service": { "action": "disable" },
      "systemd-confext.service": { "action": "disable" }
    }
  }
  ```

  with `tree/etc/systemd/system/hearth-activator.service`, the unit of
  hearth/docs/design/domain/activator/README.md (`README.md`, 3.1); `tree/usr/local/lib/hearth/askpass`, which gives
  git the fetch's credential (hearth/docs/design/domain/render/README.md, 2.4); and
  `tree/etc/hearth/alarms.json.j2`, which is
  `{{ settings.alarms | tojson }}`.
- **Where hearth's files live:** under `/usr/local` and `/etc`, as
  every download and every task file does (`tasks.md`, 5, `README.md`, 8.1), since `/usr/lib`
  and the rest of `/usr` are Debian packages' alone. Should hearth
  become a Debian package, the task installs it as one, and it moves to
  `/usr` with the packages layer, its unit in `/usr/lib/systemd/system/`
  as any package's.
- **The uid, gid and range are constants of the task,** not settings:
  every slot of a machine mounts the same `@hearth.store`, owned by them
  (hearth/docs/design/domain/machine/bootstrap.md, 3; hearth/docs/design/domain/store/README.md, 7.2). The range is 64 Ki at base 262144.
  The uid 59999, with its own group of the same id, sits at the top of
  the ids adduser hands to people from 1000 up, and away from the system
  ids packages take below 1000 (to verify against the fleet's packages).
- **`systemd-journal`** lets the steward read the journal for telemetry
  (hearth/docs/design/protocol/wire.md, section 6); its builder runs with no supplementary group
  (hearth/docs/design/domain/steward.md, 2.3).
- **Snapshots and backups are declared once, by class** (7), normally
  in `fleet.json` or an environment file:

  ```json
  {
    "hearth": {
      "snapshots": {
        "core": { "every": "1d", "keep": { "daily": 7 } },
        "data": { "every": "1h", "keep": { "hourly": 24, "daily": 14, "weekly": 8 } },
        "cache": null
      },
      "backups": { "data": { "every": "1h" } }
    }
  }
  ```

  - **`snapshots`** are local, snapper's role [M5]: the activator takes
    read-only snapshots of the class's subvolume on the period and
    prunes them by retention; they stay on the machine, readable by a
    person and restorable (hearth/docs/design/domain/operating.md, 7.1 and 7.4).
  - **`backups`** are what is sent to the backup peer, encrypted with
    `age` [M6] (hearth/docs/design/domain/operating.md, 7.2).
  - **`cache` is never snapshotted or backed up:** a key other than
    `core` and `data` is an error, and `"cache": null` only says so,
    since null deletes (`README.md`, 4.1). Buckets are `hourly`, `daily`, `weekly`
    and `monthly`, each at least 1.
  - **Render puts them in the data part** (hearth/docs/design/domain/render/spec.md, 1.3), so a change
    to one prepares a new slot of the same output, which the activator
    then follows (hearth/docs/design/domain/slots/README.md, 2.2).
  - **A container has no `hearth` task:** its classes follow its host's
    policies, which the host's activator applies to them under the
    container's target (hearth/docs/design/domain/slots/containers.md, 1) [M5].
- **`systemd-sysext` and `systemd-confext` are disabled.** Debian's
  `90-systemd.preset` enables both; hearth makes its live overlays
  itself (hearth/docs/design/domain/slots/trial.md, 6), and a boot service that merged an extension
  would stack it over them, or over a root it was not made for after a
  soft reboot.
- **The tools** hearth runs as child processes come from Debian: git
  for render, mmdebstrap and `newuidmap` for builds (hearth/docs/design/domain/store/README.md, 7.3), and
  `ssh-keygen` for the activator's host keys (`README.md`, 7.3).

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

A change applies live, with no reboot, only if every path it changes
has a consumer: a unit to act on, or a declaration that none is needed
(hearth/docs/design/domain/slots/trial.md, 7, rule 7). A task declares its own. Without one the
change takes an ordinary trial, a soft reboot or a reboot; that is never
an error.

- **`live` is a map of entries keyed by name,** with `if` and
  `for_each` (2):

  | Field | Type | Default | Meaning |
  |---|---|---|---|
  | `unit` | unit name | required, but with action `none`, which names none | the systemd unit that reads the paths |
  | `action` | enum `reload`, `restart`, `reload-or-restart`, `start`, `none` | required | what happens to the unit once the change is mounted |
  | `paths` | list of paths (`README.md`, 5.2) beneath `/etc`, `/usr/local` or `/opt`, never one of the three itself; at least one, each once | required | what it consumes; one trailing `/`, set aside before `README.md`, 5.2's check, makes a path cover its subtree |

  | Action | For | Done |
  |---|---|---|
  | `reload` | a daemon that reads its configuration again on `ExecReload=`: postfix, nginx, sshd | `systemctl reload` after the swap; a reload that fails, as sshd's on a bad `sshd_config`, fails the live trial |
  | `restart` | a daemon whose binary, libraries or data change, or that reads its configuration only at start | for a changed path under `/usr/local` or `/opt`, stopped before the swap and started after, so the old binary never reads new files; for one under `/etc` only, restarted after |
  | `reload-or-restart` | a unit that may lack a reload | `systemctl reload-or-restart` after the swap |
  | `start` | a unit whose run applies the change: a oneshot, a timer | `systemctl start` after the swap |
  | `none` | inert files, read afresh at every use: a command-line tool, its manual, data a short-lived command reads | nothing |

- **Postfix,** whose configuration a reload applies, in the `mail` task:

  ```json
  {
    "live": {
      "postfix": { "unit": "postfix.service", "action": "reload", "paths": ["/etc/postfix/", "/etc/mailname"] }
    }
  }
  ```

- **An in-house service shipped as software under `/opt`,** whose new
  binary needs a restart, in a `stoa` task:

  ```json
  {
    "software": {
      "stoa": {
        "url": "https://git.example.org/ai/stoa/releases/download/v0.4.0/stoa-x86_64-linux.tar.gz",
        "sha256": "…", "unpack": "tar", "strip": 1, "path": "/opt/stoa"
      }
    },
    "files": {
      "/etc/systemd/system/stoa.service": {},
      "/etc/stoa/stoa.json": { "template": "templates/stoa.json.j2" }
    },
    "presets": { "stoa.service": { "action": "enable" } },
    "live": {
      "stoa": { "unit": "stoa.service", "action": "restart", "paths": ["/opt/stoa/", "/etc/stoa/"] }
    }
  }
  ```

  A new release under `/opt/stoa` stops the service before the swap and
  starts it after; a change to `stoa.json` alone restarts it after. Its
  unit file needs no entry (below).
- **A command-line tool,** ripgrep from `README.md`, 8.1, which no unit runs:

  ```json
  { "live": { "ripgrep": { "action": "none", "paths": ["/opt/ripgrep/"] } } }
  ```

- **Several consumers may cover one path,** in one task or several: a
  certificate under `/etc/ssl/certs/` read by postfix and nginx runs
  both reloads. Render takes their union (hearth/docs/design/domain/render/README.md, 3.7).
- **A deleted path's consumer** is declared by the running slot's spec,
  where the path still exists; every other path's by the new spec.
- **Consumers hearth knows itself,** which no task declares:

  | Path | Consumer |
  |---|---|
  | a unit file or drop-in in `etc/systemd/system/` or `usr/local/lib/systemd/system/` | its unit: `daemon-reload`, then restarted if it runs, started if new and wanted, stopped before the swap if its file goes |
  | a `.wants/` or `.requires/` link | the unit it names: started when the link comes, stopped before the swap when it goes |
  | `etc/passwd`, `etc/group`, `etc/shadow`, `etc/gshadow`, `etc/subuid`, `etc/subgid` | the accounts, entries only added (hearth/docs/design/domain/slots/trial.md, 7, rule 5); `nscd --invalidate` if nscd runs |
  | `etc/tmpfiles.d/hearth.conf` | `systemd-tmpfiles --create` on that file (rule 6) |
  | D-Bus policy in `etc/dbus-1/system.d/` | `systemctl reload dbus` |
  | `etc/sysusers.d/hearth.conf`, `etc/systemd/system-preset/00-hearth.preset` | none: their effects, account lines and unit links, are in the change already (hearth/docs/design/domain/build/README.md, 6.2 and 6.5) |

- **Never live.** A change to any of these takes an ordinary trial,
  whatever a task declares, and render refuses a `live` entry that names
  one (11). Boot-time and PID 1 configuration, read only at boot or
  at PID 1's re-exec:

  ```
  etc/systemd/system.conf  etc/systemd/system.conf.d/  usr/local/lib/systemd/system.conf.d/
  etc/systemd/system-generators/  usr/local/lib/systemd/system-generators/
  etc/systemd/system-environment-generators/  usr/local/lib/systemd/system-environment-generators/
  etc/udev/  usr/local/lib/udev/  etc/modprobe.d/  usr/local/lib/modprobe.d/  etc/modules
  etc/modules-load.d/  usr/local/lib/modules-load.d/  etc/sysctl.conf  etc/sysctl.d/  usr/local/lib/sysctl.d/
  etc/binfmt.d/  usr/local/lib/binfmt.d/  etc/kernel/  usr/local/lib/kernel/
  etc/initramfs-tools/  etc/tiny-initramfs/  etc/crypttab  etc/default/grub  etc/fstab
  etc/hostname  etc/localtime  etc/locale.conf  etc/vconsole.conf  etc/machine-id
  etc/os-release
  ```

  Their counterparts under `usr/lib/` are the packages' alone (5),
  which never change live (hearth/docs/design/domain/slots/trial.md, 7, rule 1).

  and the services that must not restart live:

  | What | Units and paths | Why |
  |---|---|---|
  | systemd's own | every unit the `systemd` packages ship; `etc/systemd/*.conf`, their `.d/` directories and `usr/local/lib/systemd/*.conf.d/` (journald, logind, networkd, resolved, timesyncd and the rest) | PID 1's and its daemons' state does not survive a careless restart |
  | the core services | `dbus.service` and `dbus.socket` (their policy files are allowed, with a reload of dbus); `systemd-logind`, `systemd-journald`, `systemd-udevd`, `systemd-networkd`, `NetworkManager`; `getty@`, `serial-getty@`, `console-getty`; `apparmor.service` and `etc/apparmor.d/` | a restart ends sessions, loses logs, devices or the network; a profile needs `apparmor_parser -r` (hearth/docs/design/domain/slots/README.md, section 7) |
  | hearth itself, for now | every file of the `hearth` task (12) and every path 11 reserves | until live changes are proven (hearth/docs/design/domain/slots/README.md, section 7) |

- **Render refuses what it can tell is never live** (11): a `unit`
  among the core services, or named `systemd-*` or `hearth-*`, and a
  path that is one of the never-live paths above, a file of the
  `hearth` task or a path 11 reserves, or lies beneath one. A path that covers one, such
  as `/etc/systemd/`, is accepted, and a change to the never-live path
  beneath it still takes a trial. Render reads no package, so the
  builder and the activator find the units the `systemd` packages ship
  under other names (hearth/docs/design/domain/slots/trial.md, 7, rule 8).
- **What the declarations cost:** nothing in any key. The `live` part
  enters no layer key and no prepare key (hearth/docs/design/domain/render/spec.md, 1.4), so declaring
  consumers builds nothing and prepares nothing.
