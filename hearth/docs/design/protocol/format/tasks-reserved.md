# The format's tasks: what hearth reserves

## 1. What hearth reserves

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
