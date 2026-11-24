# The format's tasks: the hearth task

## 1. The hearth task

- **Every target has the `hearth` task,** but a container (below),
  without naming it: render adds it, and `"hearth": null` is an error.
  In any layer, `fleet.json` and an environment's included, it is a
  `reserved` error at the key (hearth/docs/design/domain/render/errors.md, 2), and no other kind
  reports it.
  Every such output then carries hearth's binary, its activator's unit
  (which starts the steward, which starts builders) and its user, which
  hearth/docs/design/domain/machine/README.md (`exports.md`, 1.3) and hearth/docs/design/domain/slots/README.md assume. Its files are in the repository
  like any task's, `tasks/hearth/`, so the release a fleet runs is
  pinned by a change. A revision without them fails each target that
  has the task, as any task that does not exist does (`targets.md`, 1.3; hearth/docs/design/domain/render/errors.md,
  2), and its containers render (hearth/docs/design/domain/render/README.md, 3.4).
- **But a container's:** render adds no `hearth` task to a
  container's target, and naming it there, in its base, member or
  base-env layer (`targets.md`, 1.3), is a `reserved` error at the key. The `hearth`
  settings of `fleet.json` and of an environment reach no container,
  since it lacks the task (`targets.md`, 1.3). No hearth runs in
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
  hearth/docs/design/domain/activator/README.md (`targets.md`, 1.1); `tree/usr/local/lib/hearth/askpass`, which gives
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
