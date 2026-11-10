# The packages layer

## 1. The fetch

- **What:** each `.deb` of the closure (hearth/docs/design/domain/store/records.md, 4) whose SHA-256 the
  cache lacks. Never an index: Debian publishes no uncompressed
  `Packages` file, so the digest an `InRelease` gives for one names
  nothing a mirror serves. Resolution kept the indices apt verified in
  the cache instead (hearth/docs/design/domain/store/records.md, 4); a resolution whose kept indices the
  cache lacks is made again (hearth/docs/design/domain/render/spec.md, section 2).
- **How:** `apt-helper download-file` with `<uri> <target> SHA256:<hex>`
  triples, at most 256 a run, into `staging/<key>/`; each file is then
  renamed into `cache/<sha256>` with its record (hearth/docs/design/domain/store/records.md, 5).
- **URIs** join a source's URI with a package's `Filename`. Only `http` and `https` are allowed: with a `file:` URI
  `apt-helper` links the local file instead of fetching.
- **Checked before kept:** `apt-helper` verifies the SHA-256 and leaves
  nothing at the target on a mismatch (verified with apt 3.3.3: `Err:`,
  `Hash Sum mismatch`, exit 100). The builder does not hash again: apt
  in the build checks every `.deb` against its `Packages` entry, and the
  walk hashes what lands in the layer.
- **Polite:** one fetcher per source host, serially; snapshot.debian.org
  publishes no limits and has changed them. apt follows its redirects
  to content-addressed `/file/<sha1>/` URLs.
- **The fetch runs in a cgroup of its own, with no sandbox.** It is the
  one step with the network, and it has no namespaces: it runs as the
  builder, writes only the builder's own staging, and runs no code from
  the repository, since its inputs are data checked by digest. Its
  cgroup, a leaf of the builder's subtree (`sandboxes.md`, 7), bounds its memory and
  processes, and stops it whole.
- **Its apt configuration is the builder's,** given whole with
  `APT_CONFIG`: retries, timeouts, a proxy where the builder has one
  (apt-cacher-ng on a pool's host). None enters a key or an output.

## 2. The fetcher, chosen

| | `apt-helper download-file` | apt in a fetch-only sandbox | skein's HTTP client |
|---|---|---|---|
| fetches | exactly the files named, each checked by SHA-256 | the closure again, from the sources | what the builder asks |
| needs | the list with digests, which the resolution holds | a scratch apt root; `apt-get update` every time; apt's text | DNS at run time, redirects, proxies, hashing gigabytes in step code |

**Chosen: `apt-helper`.**

- **The resolution already names every byte by digest,** so the fetch
  is a list, not a second resolution.
- **It is apt's acquire code:** HTTP and HTTPS, apt's proxy settings,
  snapshot's redirects, retries; it checks the digest before it keeps a
  file; it is on every Debian machine, in apt itself.
- **Its output is three line shapes** (`README.md`, 8.2).
- **apt in a sandbox** would resolve again what resolution resolved,
  fetch the indices on every build, and speak more of apt's text.
- **skein's HTTP client** resolves names only at startup, and SHA-256
  in step code is unmeasured. It arrives with peers [M6]
  (`hearth-protocol-http`, hearth/docs/design/protocol/wire.md, section 7), for bytes within a
  plane; the archive stays `apt-helper`'s.

## 3. The local repository

```
staging/<key>/repo/<a source URI's path>/                 one tree per source line
  dists/<suite>/InRelease                                 a hard link to cache/<sha256>
  dists/<suite>/<component>/binary-<arch>/Packages        the kept index, decompressed, at its plain path
  pool/…/<package>_<version>_<arch>.deb                   the closure's only
```

- **A sparse copy of the snapshot:** the genuine `InRelease`, the
  `Packages` files resolution kept (hearth/docs/design/domain/store/records.md, 4), and only the
  closure's packages. apt checks each index against `InRelease` and each
  `.deb` against its index, as it would online.
- **Indices at their plain paths:** `InRelease` lists the uncompressed
  `Packages` file with its SHA-256, so apt accepts it there. Debian's
  `InRelease` also announces `Acquire-By-Hash: yes`; whether apt asks a
  `file:` source for the by-hash path, or for a compressed file first,
  is to verify on recordings.
- **The sources mmdebstrap gets,** one line per source:

  ```
  deb [trusted=yes check-valid-until=no] file:///run/hearth/repo/<path> <suite> <components>
  ```

  `trusted=yes`: the signature was checked when resolution fetched the
  `InRelease` (hearth/docs/design/domain/render/spec.md, section 2), and every byte since is checked
  by SHA-256; checking it again would add nothing, and could fail an
  old snapshot whose signing key has expired (to verify).
- **Bound read-only at `/run/hearth/repo`** in the sandbox (`sandboxes.md`, 2), and
  into the chroot at the same path by mmdebstrap's
  `file-mirror-automount` hook, for apt inside. Directories are 0755,
  files 0644: their owner is unmapped there (hearth/docs/design/domain/store/README.md, 2.2).

## 4. mmdebstrap

```
mmdebstrap
  --mode=unshare                    run as the sandbox's root: no user namespace of its own (8.3)
  --format=directory
  --verbose                         apt's and dpkg's lines: progress and the failure tail (10.1)
  --variant=<variant>               the packages part: from placement
  --architectures=<arch>            the packages part
  --include=<name>=<version>,…      the closure, in arguments of at most 64 KiB
  --aptopt='APT::Install-Recommends "<install_recommends>"'     the packages part's one apt option
  --aptopt='Acquire::Check-Valid-Until "false"'                 hearth's, as at resolution (../render/spec.md, 2.2)
  --dpkgopt=force-confold                                       hearth's: keep what the setup hook placed
  --hook-dir=/usr/share/mmdebstrap/hooks/file-mirror-automount
  --setup-hook=<5.5>   --essential-hook=<5.5>   --customize-hook=<5.6>
  <suite> /target /run/hearth/input/sources.list
```

- **The whole closure is pinned** as `name=version`, so apt chooses
  nothing. Linux caps one argument at 128 KiB (`MAX_ARG_STRLEN`): a
  desktop's 3,000 names would pass it, so they are split, and
  mmdebstrap joins its `--include` options.
- **`--architectures` is given;** os's never was, so it built the
  host's.
- **apt's and dpkg's options stay in the output**
  (`/etc/apt/apt.conf.d/99mmdebstrap`, `/etc/dpkg/dpkg.cfg.d/99mmdebstrap`).
  `install_recommends` is the packages part's (hearth/docs/design/domain/render/spec.md, 1.2). The
  others are hearth's constants, part of the encoding's version
  `hearth/1` (hearth/docs/design/domain/store/README.md, 3.2), so changing one changes every key:
  `Check-Valid-Until`, which resolution passes too, and `force-confold`,
  which keeps the configuration file the setup hook places before its
  package (5). No proxy is passed: the source is local.
- **No `--skip`:** every cleanup runs (6), and the target is an empty
  snapshot, so `check/empty` passes.
- **The environment is given whole:** `PATH=/usr/sbin:/usr/bin`,
  `HOME=/root`, `LC_ALL=C.UTF-8`, `DEBIAN_FRONTEND=noninteractive`,
  `DEBCONF_NONINTERACTIVE_SEEN=true`. No `SOURCE_DATE_EPOCH`
  (hearth/docs/design/domain/build/reproducible.md).
- **Inputs** sit read-only in `/run/hearth/input`: the local sources,
  the part's sources for the output, the seeded `passwd` and `group`
  lines, the preseeds, the initramfs configuration.

## 5. Hooks: seeds, preseeds, the initramfs and apt

```
--setup-hook='mkdir -p "$1/etc/tiny-initramfs" "$1/run/hearth" "$1/var/lib/machines" "$1/var/lib/portables"
  && cp /run/hearth/input/tiny-initramfs.conf "$1/etc/tiny-initramfs/"
  && cp /run/hearth/input/preseed "$1/run/hearth/preseed"'
--extract-hook='for f in passwd group; do
    [ -e "$1/etc/$f" ] || cp "$1/usr/share/base-passwd/$f.master" "$1/etc/$f"
    cat "/run/hearth/input/$f" >> "$1/etc/$f"; done'
--essential-hook='chroot "$1" debconf-set-selections /run/hearth/preseed'
```

- **Fixed text,** in `hearth-protocol-mmdebstrap`. Data from the
  repository reach hooks only as files, never as shell. A part with no
  initramfs configuration (a container) has no copy of it.
- **The setup hook runs before any package is unpacked:**
  - tiny-initramfs builds the initramfs from its kernel hook when the
    kernel package is configured, so its configuration must already be
    there (os does the same);
  - `/var/lib/machines` and `/var/lib/portables` are made directories
    first: systemd-container's postinst runs `systemd-tmpfiles --create`,
    whose `Q` lines make both subvolumes on btrfs where they are absent,
    and the walk refuses a nested subvolume (hearth/docs/design/domain/store/README.md, 3.4). Seen with
    sid's systemd 261; trixie's 257 is to confirm.
- **The extract hook seeds fixed ids,** after the essential packages are
  unpacked and before any is configured, in every variant. hearth/docs/design/protocol/format/README.md
  requires a fixed uid and gid for each user and group that owns a data
  path, a secret or an identity file, package users included; the
  packages part carries the package users' lines (hearth/docs/design/domain/render/spec.md, 1.2),
  since they change what this layer builds. A snapshot bump then cannot
  renumber them under their data entries: `adduser --system`
  finds the user and keeps it, and the config layer's sysusers lines
  carry the same ids (`README.md`, 6.2).
  - **Why the extract hook, not the setup hook:** base-passwd's preinst
    writes `/etc/passwd` and `/etc/group` only where they are absent, so
    files seeded before it would lack root and every static account. The
    hook copies base-passwd's own master files first, as its postinst
    would (read in base-passwd's maintainer scripts on sid), then adds
    the seeded lines.
- **The essential hook runs once the `Essential: yes` packages are
  configured and before the rest,** so the target's own
  `debconf-set-selections` loads the preseeds before tzdata, locales or
  postfix ask. mmdebstrap runs no essential hook for the `custom`
  variant, os's for containers and one candidate for hearth's
  (hearth/docs/design/protocol/format/README.md, section 11): where a container's preseeds would load is
  open (`README.md`, section 13).
- **The initramfs configuration is the packages part's,** which render
  writes from its two fields, the modules and the microcode choice
  (hearth/docs/design/domain/render/spec.md, 1.2; hearth/docs/design/domain/machine/boot.md, 4.1). Nothing a task writes reaches it
  (hearth/docs/design/protocol/format/tasks.md, 11), so `AUTO_MODULES=yes` and `MICROCODE=yes`, which
  read the build host, cannot occur.
- **apt's configuration** enters as `--aptopt` (4), written by
  mmdebstrap during its setup, permanent by design.

## 6. Cleanup

**mmdebstrap's own cleanup, all of it** (os skipped it all):

| Cleanup | Removes | Why it stays on |
|---|---|---|
| `cleanup/apt/lists` | the package lists | they name the local repository; no layer above runs apt |
| `cleanup/apt/cache` | downloaded `.deb`s and apt's caches | the store's cache has them |
| `cleanup/mmdebstrap` | its setup files: `00mmdebstrap`, the temporary apt configuration | the builder's details |
| `cleanup/reproducible` | `dpkg.log`, apt's `history.log`, `term.log`, `eipp.log.xz`, `alternatives.log`, ldconfig's `aux-cache`, `/var/lib/dbus/machine-id`; writes `/etc/machine-id` empty if it exists | logs and identity |
| `cleanup/run`, `cleanup/tmp` | everything in `/run` and `/tmp` | runtime only |

**hearth's customize hook,** fixed text, before mmdebstrap's cleanup:

```
--customize-hook='cp /run/hearth/input/sources.final "$1/etc/apt/sources.list"
  && rm -f "$1/etc/resolv.conf" "$1/etc/hostname"
  && rm -f "$1"/etc/ssh/ssh_host_*_key "$1"/etc/ssh/ssh_host_*_key.pub "$1"/etc/dropbear/dropbear_*_host_key
  && rm -f "$1/etc/ssl/private/ssl-cert-snakeoil.key" "$1/etc/ssl/certs/ssl-cert-snakeoil.pem"
  && find "$1/var/log" -type f -size +0 -exec truncate --size=0 {} +'
```

- **The sources** become the packages part's (the snapshot's URLs), in
  place of the local repository's.
- **`resolv.conf` and `hostname`:** mmdebstrap copies the host's, here
  the tool root's: `hostname` alone, `localhost` (`sandboxes.md`, 2). Both are removed
  rather than given fixed bytes: an absent file is as reproducible
  (hearth/docs/design/domain/build/reproducible.md, 2.3), and holds nothing a target did not choose. A
  task that wants either writes it in the config layer.
- **Host keys:** openssh-server's postinst makes them, and dropbear's;
  the activator writes the machine's at prepare (hearth/docs/design/domain/slots/prepare.md, 4).
- **The snakeoil pair:** ssl-cert's postinst makes a private key, and
  dovecot-core and postgresql-common depend on ssl-cert. A task that
  serves TLS declares its key as a secret (hearth/docs/design/protocol/format/README.md, section 7).
- **Logs** that mmdebstrap's list misses (fontconfig's trigger writes
  `/var/log/fontconfig.log`) are emptied, keeping modes and owners.
- **The empty `/etc/machine-id`.** A maintainer script makes an id
  (systemd's, to verify); mmdebstrap empties it. An empty file means "not a first boot"
  (machine-id(5)); on a read-only root systemd would then use a new
  transient id at every boot. The activator writes the machine's id
  into the slot at prepare (hearth/docs/design/domain/slots/prepare.md, 4), so the output keeps the
  empty file and the scan refuses any other.

## 7. Verifying the installed set

After mmdebstrap exits 0, the target's own `dpkg-query`, in a sandbox
pivoted into the layer, read-only:

```
dpkg-query --show --showformat='${Package}\t${Architecture}\t${Version}\t${db:Status-Abbrev}\n'
```

- **It must equal the closure exactly:** each name, architecture and
  version once, each `ii ` (installed, no error flag), nothing more.
- **A difference fails the layer,** listing the missing, the extra and
  the wrong: a package half-configured (`iF`, `iU`), an essential one
  the resolution missed, a version apt chose otherwise.

## 8. Boot digests

After the packages layer's walk, the builder records, per kernel the
layer holds, what hearth/docs/design/domain/slots/README.md (`README.md`, 5.1) needs to decide whether a slot's kernel
is the running one. They go in the tree file's end item (hearth/docs/design/domain/store/records.md, 3):

```
version     the <version> of usr/lib/modules/<version> and boot/vmlinuz-<version>
kernel      the content digest of boot/vmlinuz-<version>, from the walk
modules     the subtree digest of usr/lib/modules/<version> (../store/README.md, 3.4)
initramfs   the initramfs's input digest, below
```

**The initramfs's input digest,** defined here alone (hearth/docs/design/domain/slots/trial.md, 1
cites it):

```
H("initramfs",
  list [ the kernel package's name, its version ],          the package that installs boot/vmlinuz-<version>
  bytes tiny-initramfs-core's version,
  digest the content digest of etc/tiny-initramfs/tiny-initramfs.conf,   which lists the modules and MICROCODE
  bytes <version>,                                          the kernel version
  list [ list [ name, version ] … ])                        with MICROCODE=generic, each installed microcode
                                                            package by name; empty otherwise
```

- **The initramfs's inputs, not its bytes:** mktirfs packs its staging
  tree with `cpio -H newc`, whose headers carry inode numbers and the
  build's mtimes (hearth/docs/design/domain/build/reproducible.md, 2.2), so the image differs on every
  build, and comparing bytes would force a full reboot on every snapshot
  bump.
- **From the resolution and the walk:** versions come from the closure
  (hearth/docs/design/domain/store/records.md, 4), digests from the tree file; nothing is read again.
- **Carried up unchanged:** the software and config layers copy them.
  A change above packages to `boot/`, `usr/lib/modules/` or
  `etc/tiny-initramfs/` fails the scan (`README.md`, 7.3), since nothing above
  packages makes the image again.
