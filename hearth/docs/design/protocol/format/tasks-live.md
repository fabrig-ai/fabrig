# The format's tasks: live changes

## 1. Live changes

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
  | `paths` | list of paths (`schema.md`, 1.2) beneath `/etc`, `/usr/local` or `/opt`, never one of the three itself; at least one, each once | required | what it consumes; one trailing `/`, set aside before `schema.md`, 1.2's check, makes a path cover its subtree |

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
