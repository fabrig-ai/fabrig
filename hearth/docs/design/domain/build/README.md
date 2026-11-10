# Building outputs

Provisional, 2026-10-09, revised 2026-10-19. How an output is made from a spec: the recipe,
the three layers, the plan, each layer's steps, the sandboxes they run
in, what an output must not hold, the tools as protocols, caches and
costs. M2 builds it. Where layers live, how they are named and how a
layer is walked and published is hearth/docs/design/domain/store/README.md. What is open is in section
13.

## 1. In one page

- **An output is three layers:** packages, software and config, each a
  writable snapshot of the one beneath, changed by steps in sandboxes,
  then walked, frozen and published (hearth/docs/design/domain/store/README.md, 4.2).
- **The plan starts from the highest layer whose key is published.** A
  config change is a snapshot of the software layer and the config
  steps: seconds. The packages layer is reconciled from the target's
  previous one by apt, and built from scratch instead only when there is
  no base, when a reconcile fails, and now and then to bound history
  (4.2).
- **Packages are fetched by digest, then installed offline.** A fetch
  step runs `apt-helper` for every `.deb` the resolution names by
  SHA-256, into the store's cache, where resolution kept the indices apt
  verified. mmdebstrap then installs from a local repository of hard
  links to both, with no network.
- **mmdebstrap runs inside hearth's sandbox, as its root.** It makes no
  user namespace of its own, and chroots from a root where only the
  layer is writable and the builder's tools are read-only.
- **The target's own tools make the config layer:** its
  `systemd-sysusers`, `systemd-tmpfiles` and `systemctl preset-all`,
  then each task's `customize.sh`. The tree helper applies the file
  tree with exact modes and owners.
- **Every build step runs in a sandbox of its own:** user, mount, pid,
  ipc, uts and network namespaces, pivoted into its root, inputs
  read-only, a minimal `/proc`, `/sys` and `/dev`, no network, cgroup
  limits. The fetch alone has the network, in a cgroup with no sandbox.
- **Nothing unique and nothing secret leaves a step.** mmdebstrap's
  cleanup stays on; the packages step removes what packages make (host
  keys, the snakeoil key, the sandbox's hostname, logs); a scan after
  every layer refuses an output that holds any offender.
- **Tools are protocols:** an argument vector down, typed events up.
  mmdebstrap's `I:` stage lines and failure tail, `apt-helper`'s
  `Get:` and `Err:` lines and `dpkg-query`'s fields are parsed below the
  domain, against recordings of real runs.
- **Everything slow is keyed:** layers, resolutions, downloads. What
  each step cost is a fact.
- **On a developer's machine the client builds in-process** (M2), as a
  regular user with a subordinate range and a btrfs store of its own
  (hearth/docs/design/client/README.md, 3.5; hearth/docs/design/domain/store/README.md, section 7).

## 2. Recipes

A recipe turns inputs into a chain of layers; the top one is the
output.

```
recipe rootfs                                                this document
  inputs   the spec's output parts: packages, software, config (../render/spec.md, section 1)
           the resolution the packages part names (../store/records.md, 4)
  layers   packages on the empty layer; software on packages; config on software
  output   the config layer, named by its key (../store/README.md, 3.3)

recipe artifact [M6]                                         domain/artifacts.md
  inputs   an environment's output, sources, declared inputs, a command
  layers   one, collected from the command's outputs
```

- **One machinery for both:** keys (hearth/docs/design/domain/store/README.md, section 3), the fetch
  (`packages.md`, 1), sandboxes (`sandboxes.md`), the tree helper, records, pins.
- **Recipes are a closed set in `hearth-domain-build`,** each layer's
  steps fixed by its recipe; the parts give the data.
- **Slot parts never reach a recipe:** `data`, `secrets` and `boot` are
  the activator's at prepare (hearth/docs/design/domain/render/spec.md, section 1; hearth/docs/design/domain/slots/prepare.md).
- **A host builds its containers' outputs [M5]** with the rootfs recipe,
  in its own store: no hearth runs in a container (hearth/docs/design/domain/slots/containers.md, 5). A
  container's output differs only by its parts: the containers' variant
  (`packages.md`, 4), no initramfs configuration and no kernel, so its tree file's
  `boot` is empty (hearth/docs/design/domain/store/records.md, 3); nothing unique or secret enters it
  either (`README.md`, section 7), since its identity and secrets are written at
  prepare as a host's are.
- **The artifact recipe's sandbox is not a container target [M6]:** a
  build environment's output is only a sandbox's root, never run by
  nspawn nor placed in a slot (hearth/docs/design/domain/build/artifacts.md, 2.3).

## 3. The layers

| Layer | Parent | Key (hearth/docs/design/domain/store/README.md, 3.3) | Holds | Steps |
|---|---|---|---|---|
| packages | empty | `H("packages", resolution digest, packages part)` | the bootstrapped system: every package's files and what its maintainer scripts made, dpkg's database, the kernel and initramfs in `/boot`, `/var` as installed | fetch; mmdebstrap; verify (`packages.md`) |
| software | packages | `H("software", packages key, software part)` | downloads placed; what each `software.sh` made (`/usr/local`, `/opt`) | fetch; place; `software.sh` (section 5) |
| config | software | `H("config", software key, config part)` | users; the file tree (`/etc`, `/usr/local`, `/opt` and more, but never the rest of `/usr`); tmpfiles' paths in the image; unit links from presets; what each `customize.sh` made | users; file tree; tmpfiles; presets; `customize.sh` (section 6) |

- **A layer is one staging subvolume;** its steps run on it in order,
  and it is walked once, after the last (hearth/docs/design/domain/store/README.md, 4.2).
- **An empty part still makes its layer:** a snapshot and a walk, about
  a second, so every output has three layers and keys stay uniform.
- **"config", not "etc":** the layer writes users, `/usr/local`, unit
  links and more (hearth/docs/design/decisions.md).

## 4. The plan

### 4.1 From the highest published layer

```
plan(spec)
  resolution  ← the record named by the packages part's resolution key,
                or a resolution (../render/spec.md, section 2)
  keys        ← packages, software, config (../store/README.md, 3.3)
  start       ← the highest of config, software, packages that is published
  build       ← each layer above start, bottom-up; from the empty layer if none is
```

- **A published config key is the output:** nothing is built.
- **A config change** builds the config layer on a snapshot of the
  published software layer. A software change builds software and
  config. A new closure or packages part builds all three.
- **The plan is data** in `hearth-domain-build`: layers, each with its
  steps. The builder's root routes its requests to the store (snapshots,
  walks, records) and to the protocol layer (tools).
- **A plan pins its chain** while it runs (hearth/docs/design/domain/store/README.md, 5.1).

### 4.2 The packages layer: reconcile, and scratch now and then

- **By default, a reconcile from a base.** The base is the target's
  previous packages layer, or else the published packages layer of the
  same suite, architecture and variant whose closure shares the most
  packages with the new one. The store answers the candidates from its
  layers' records: a packages layer's `inputs` name its resolution,
  whose record holds the suite, the architecture, the variant and the
  closure (hearth/docs/design/domain/store/records.md, 2 and 4); a layer whose resolution is gone is
  no candidate. With no candidate of the same suite, the build is from
  scratch. The step snapshots the base and, in the
  sandbox, offline from the local repository (`packages.md`, 3), with the seeds and
  preseeds of the new part applied first (`packages.md`, 5):
  `apt-get install --allow-downgrades <name>=<version>…` for the whole
  closure (a pin with no architecture is the target's), then `apt-get purge` of every name the closure no longer
  holds, never `autoremove`, then `dpkg --configure -a`. The installed
  set is then checked against the closure, and the cleanups and the
  scan run, exactly as after a scratch build (`packages.md`, 6, 7, `README.md`, 7.3).
- **The same key either way.** The key names the inputs (hearth/docs/design/domain/store/README.md,
  3.3); the record says how the layer was made: `how` scratch or
  reconcile, its `base`, and its `depth`, the reconciles since the last
  scratch build in its line (hearth/docs/design/domain/store/records.md, 2).
- **From scratch instead, never in addition,** when there is no base;
  when the suite, the architecture or the variant changes; when a
  reconcile fails, which a scratch build then retries at once; and when
  the line's depth reaches a bound (8 by default) or its last scratch
  build is older than a period (30 days by default), both settings of
  the hearth task.
- **Why reconcile is the default:** a snapshot bump changes tens of a
  thousand packages, and only those are unpacked and configured, with
  triggers (the initramfs, `ldconfig`, man-db) run once; the walk hashes
  only the files that changed (hearth/docs/design/domain/store/README.md, 4.3); the layer shares its
  unchanged extents with its base; and a peer's change set [M6] is a
  few megabytes, not a system.
- **What a reconciled layer may carry:** a purged package's leftovers,
  old conffiles, debconf answers and maintainer scripts' state, which a
  scratch build would not hold (`hearth/docs/design/draft/archive/factory.md`, 6.4). The
  periodic scratch build bounds that history. Comparing a reconciled
  layer with a scratch build of the same key is the lab's experiment,
  run in the background, never a step of a build.
- **Above packages, a patch and a scratch build agree by construction:**
  a config layer is always built on the same software layer, from that
  layer's tree, never on an older config layer.

### 4.3 Steps and failure

- **A step** is a tool or a tree helper request, with its inputs, its
  sandbox (`sandboxes.md`) and its limits, and ends in exactly one terminal
  event.
- **Failures are typed,** in hearth/docs/design/protocol/wire.md's classes (hearth/docs/design/protocol/standalone.md, 4): a fetch
  is transient (the network, a 404, a digest mismatch); a step's exit,
  a limit, a failed verification or a scan finding fails the key.
- **A failed layer publishes nothing.** Its staging subvolume is
  deleted; the layers beneath stay published.

### 4.4 Concurrency

- **One build per key** (hearth/docs/design/domain/store/README.md, 4.6), different keys at once within
  the builder's limits: one packages step at a time, other steps 4 at
  a time and builds in flight 8 by default (both configured), one
  fetcher per source host.
- **On a machine, its own builds one at a time:** the steward gives its
  builder the pass's jobs one by one; from M6, artifacts and peers'
  change sets run beside them within the limits above
  (hearth/docs/design/domain/steward.md, 3.4).

## 5. The software layer

### 5.1 Downloads

- **What a download is, and who owns what it gives, is defined once:**
  hearth/docs/design/protocol/format/README.md, 8.1 and hearth/docs/design/domain/render/spec.md, 1.2. The part maps each path to a
  SHA-256 and either a `file` (mode and owners) or a `tar` (leading
  segments to strip); software is root's, or a package user's with a
  fixed id, and render refuses any other owner. Its URL is a fetch hint
  beside the part, outside the digest.
- **Fetched as in `packages.md`, 1,** into the same cache, unless the cache has it.
  An artifact by key [M6] comes from the store instead
  (hearth/docs/design/domain/build/artifacts.md, 7.3).
- **Recorded:** the software layer's record lists the SHA-256 of each
  download it placed, and the tree digest of each artifact [M6], so
  collection keeps them while the layer is kept (hearth/docs/design/domain/store/records.md, 2, and hearth/docs/design/domain/store/README.md, 5.5).

### 5.2 Placing them

- **A `file`** is a put of its path, mode, owners and content digest,
  applied by the tree helper (hearth/docs/design/domain/store/tree-helper.md, 5), owners resolved by name in
  the layer; a package user is there with its fixed id (`packages.md`, 5). No tool
  runs.
- **A `tar`** is unpacked by the target's own `tar`, in a sandbox
  pivoted into the layer, the archive bound read-only at
  `/run/hearth/download`, its directory made first by a put:

  ```
  tar --extract --file=/run/hearth/download --directory=<path> --no-same-owner [--strip-components=<strip>]
  ```

  `--no-same-owner` makes every file the namespace root's, whatever the
  archive says, and modes are kept, as hearth/docs/design/protocol/format/README.md (8.1) says. `tar`
  detects the compression, with the target's own decompressors.

### 5.3 `software.sh`

- **Each task's script, after every download is placed,** in the order
  the software part lists them (hearth/docs/design/domain/render/spec.md, 1.2), each in a sandbox of
  its own: pivoted into the layer, offline, as the namespace's root,
  `/bin/sh -eu /run/hearth/script`, with the target's shell and tools.
- **It names a download by the path it was placed at** (hearth/docs/design/protocol/format/tasks.md, 8);
  nothing of the builder's is visible (`sandboxes.md`, 6).
- **os ran it outside the chroot,** with the host's tools against
  `$ROOTDIR` and a writable host cache.

## 6. The config layer

### 6.1 The order

```
1  users          the part's users, through the target's systemd-sysusers
2  file tree      the part's files, applied by the tree helper with modes and owners
3  tmpfiles       the part's lines for paths in the image, through the target's systemd-tmpfiles
4  presets        the part's presets, through the target's systemctl preset-all
5  customize.sh   each task's, in the part's order
```

- **Users first,** so the file tree's owners resolve. **`customize.sh`
  last,** so it sees everything else in place.
- **Each runs in a sandbox of its own,** but the file tree, which the
  tree helper applies.

### 6.2 Users

- **The part's groups, users and memberships** (hearth/docs/design/domain/render/spec.md, 1.2) become
  sysusers lines (`g`, `u`, `m`), written to
  `/etc/sysusers.d/hearth.conf` (a put), then:

  ```
  systemd-sysusers /etc/sysusers.d/hearth.conf
  ```

  Given a path, it runs that file alone (systemd-sysusers(8)). A
  field with a space, a `gecos` most often, is written in double quotes,
  which hearth/docs/design/protocol/format/tasks.md, 6, keeps out of the fields.
- **Ids are fixed where hearth/docs/design/protocol/format/README.md requires them:** a user or group that
  owns a data path, a secret or an identity file has its uid and gid in
  its line, the same a package user was seeded with (`packages.md`, 5), so a snapshot
  bump renumbers nothing a data entry holds. A package user
  gets no line: the package made it (hearth/docs/design/domain/render/spec.md, 1.2).
- **Passwords are locked** (`!*`). A password hash is a secret: the
  activator applies it to the slot's shadow file at prepare
  (hearth/docs/design/domain/slots/prepare.md, 4).
- **At boot** `systemd-sysusers.service` finds the users made, and with
  `/etc` read-only it writes nothing.

### 6.3 The file tree

- **Path to content digest, mode and owner** (hearth/docs/design/domain/render/spec.md, 1.2). Each
  content is in the cache by digest: a static blob streamed there by
  render's protocol layer (hearth/docs/design/domain/render/README.md, 2.3), a rendered file written by
  the builder. The tree helper then applies the tree as a change set of
  puts (hearth/docs/design/domain/store/tree-helper.md, 5), with owner and group names resolved against the
  layer's own `/etc/passwd` and `/etc/group`, after 7.2.
- **Modes and owners are exactly the part's,** a packaged conffile's
  included. os's `rsync -rltD` kept a packaged file's mode and its
  owner root, and tasks fixed them afterwards with tmpfiles `z` lines.
- **A missing parent directory** is made 0755, owned by root, unless the
  tree declares it.
- **A path through a link is refused:** `/lib` is `usr/lib` since the
  merged `/usr`, so tasks name canonical paths.
- **What hearth reserves is refused at render,** paths and kinds as
  hearth/docs/design/protocol/format/README.md (5.11) lists them.

### 6.4 tmpfiles

- **The part's lines go to `/etc/tmpfiles.d/hearth.conf`** for every
  boot, and at build are applied to paths in the image:

  ```
  systemd-tmpfiles --create -E --exclude-prefix=/tmp --exclude-prefix=/var/tmp /etc/tmpfiles.d/hearth.conf
  ```

  `-E` excludes `/dev`, `/proc`, `/run` and `/sys`. A line for `/etc` or
  `/usr/local` takes effect in the root, read-only at boot, where the line
  then finds its work done; one for `/var` lands in the output's
  `/var`, which every slot copies; one for a data entry's path is
  applied again at boot, through the entry's bind, on its directory in
  its class; lines for `/run` and `/tmp` wait for boot.
- **The types are hearth/docs/design/protocol/format/README.md's (`packages.md`, 6),** less what it reserves (5.11):
  lines that would make a nested subvolume in a layer (hearth/docs/design/domain/store/README.md, 4.1),
  or write into the read-only root at every boot and fail its bless
  (hearth/docs/design/domain/slots/bless.md, section 1).
- **Only hearth's file runs at build;** the packages' own `tmpfiles.d`
  run at boot, as Debian intends.

### 6.5 Presets

- **The part's presets go to
  `/etc/systemd/system-preset/00-hearth.preset`,** which sorts before
  every other preset file, in whichever directory, so its lines win;
  then:

  ```
  SYSTEMD_OFFLINE=1 systemctl preset-all
  ```

- **No PID 1 answers in a sandbox;** `SYSTEMD_OFFLINE=1` makes
  `systemctl` act on files (systemd's environment variables), with the layer
  as `/`, so no `--root`.

### 6.6 `customize.sh`

- **Each task's, last, in the part's order** (hearth/docs/design/domain/render/spec.md, section 1),
  each in a sandbox of its own: `/bin/sh -eu /run/hearth/script`, with
  the target's shell, offline, as the namespace's root.
- **It sees the layer and nothing of the builder:** no secret (they are
  written at prepare), no target name, no network. What a script did
  with a secret (os's users task ran `chpasswd` from `/secrets`) moves
  to the activator or to boot.

### 6.7 The target's own binaries

- **Every step above packages is pivoted into its layer,** so
  `/usr/bin/systemd-sysusers` is the target's, at the target's systemd
  version, with its configuration. os ran the host's with `--root`.
- **A target without the tool** (no systemd, yet users declared) fails
  the step with "not found"; render may refuse it earlier
  (hearth/docs/design/domain/render/errors.md).
- **The packages step is the one exception:** the builder's mmdebstrap,
  apt and `dpkg-deb` unpack the essential set, and every maintainer
  script then runs chrooted with the target's binaries. The
  `dpkg-query` that verifies is the target's.

## 7. Nothing unique, nothing secret

### 7.1 Classes

- **Secret:** whoever reads it can act as the machine or a user.
- **Identity:** unique to a machine, so shared by every machine that
  runs the output.
- **The builder:** what tells which builder made it: its hostname,
  resolver, paths, proxy.
- **Noise:** the build's logs and caches, which carry its time.

All four are kept out of outputs (hearth/docs/design/README.md, section 1), so outputs can
be shared between machines and peers.

### 7.2 The offenders

| Path | Class | Made by | Treatment |
|---|---|---|---|
| `/etc/machine-id` | identity | systemd's postinst | emptied by mmdebstrap (`packages.md`, 6); refused unless empty; written at prepare |
| `/var/lib/dbus/machine-id` | identity | dbus | removed by mmdebstrap; refused unless a link to `/etc/machine-id` |
| `/etc/ssh/ssh_host_*_key{,.pub}` | identity, secret | openssh-server's postinst | removed (`packages.md`, 6); refused; written at prepare |
| `/etc/dropbear/dropbear_*_host_key` | identity, secret | dropbear's postinst | the same |
| `/etc/ssl/private/*`, not empty | secret | ssl-cert's postinst (snakeoil) | snakeoil removed (`packages.md`, 6); refused |
| `/var/lib/systemd/random-seed`, `/var/lib/urandom/random-seed` | identity | a seed service run at build | refused; made at first boot |
| `/var/lib/systemd/credential.secret` | secret | `systemd-creds` run at build | refused |
| `/etc/shadow`, `/etc/gshadow` and their `-` backups: a password field but `*`, `!`, `!!`, `!*` | secret | `chpasswd`, `useradd -p` in a script | refused; hashes applied at prepare |
| `/etc/passwd`: a password field but `x` or `*` | secret | the same | refused |
| `/var/cache/debconf/passwords.dat`, not empty | secret | a password preseed | refused; render refuses `password` preseeds (hearth/docs/design/protocol/format/tasks.md, 11) |
| `/etc/resolv.conf`, `/etc/hostname` | the builder | mmdebstrap copies the host's | removed (`packages.md`, 6); a task may write its own |
| `/etc/apt/sources.list` | the builder | mmdebstrap: the local repository | replaced by the part's sources (`packages.md`, 6) |
| an apt proxy setting | the builder | os's `--aptopt` | never passed: the build is offline |
| `/var/lib/apt/lists/*` but `lock`, `partial/` | the builder, noise | apt | removed by mmdebstrap; refused |
| `/var/cache/apt/archives/*.deb`, `/var/cache/apt/*.bin` | noise | apt | removed by mmdebstrap; refused |
| `/var/log/**`, a file not empty | noise | dpkg, apt, fontconfig | removed or emptied (`packages.md`, 6); refused |
| `/var/cache/ldconfig/aux-cache` | noise | ldconfig | removed by mmdebstrap |
| `/etc/.git`, `/etc/.etckeeper` | noise; secret (`/etc/shadow`'s history) | etckeeper | refused: no use on a read-only root |
| anything in `/dev` | the builder | os's fake `/dev/urandom` | never made (`sandboxes.md`, 4); refused |
| anything in `/proc`, `/sys`, `/run`, `/tmp`, `/var/tmp` | noise | a step | emptied by mmdebstrap; tmpfs later (`sandboxes.md`, 2); refused |
| a socket | noise | a daemon a script started | refused |
| an initramfs with the builder's microcode or modules | the builder | `MICROCODE=yes`, `AUTO_MODULES=yes` | never written: render writes the configuration (`packages.md`, 5) |
| an owner outside the range | the builder | a file the builder wrote | refused by the walk (hearth/docs/design/domain/store/README.md, 3.4) |

### 7.3 The scan

- **After every layer, in the tree helper's walk,** over what the layer
  changed (hearth/docs/design/domain/store/tree-helper.md, 4). A layer is its parent and its change set,
  and the parent passed, so checking the changes covers the output.
- **Rules are data** in `hearth-domain-build`: a path (exact, a prefix,
  or a pattern in its last component, such as `ssh_host_*_key`), a
  predicate (exists; not empty; not a link to a given path; password
  fields locked; a socket; anything beneath), the class, and the reason
  with its Debian bug where there is one.
- **The helper judges contents** (password fields, emptiness), since
  only it can read every file. A password field is read by a protocol
  machine, `hearth-protocol-accounts`, never by the helper's domain
  (hearth/docs/design/domain/store/tree-helper.md, 3). A finding names the path and the rule, never
  contents.
- **A finding fails the layer,** naming the step's layer, the path and
  the rule. Nothing is removed silently above the packages step: a
  `customize.sh` that makes a host key fails its build.
- **Above packages, `/usr` is fixed but `/usr/local`** (hearth/docs/design/protocol/format/tasks.md,
  5): a change in the software or config layer under `usr/` outside
  `usr/local/` is a finding, a `software.sh` or a `customize.sh` that
  writes there included. The rest of `/usr` is the packages', and a
  live change overlays `/usr/local` alone of it (hearth/docs/design/domain/slots/trial.md, 6).
- **Above packages, the boot files are fixed.** A change in the
  software or config layer to `boot/`, `usr/lib/modules/` or
  `etc/tiny-initramfs/` is a finding: nothing above packages makes the
  initramfs again, and the boot digests are carried up unchanged (`packages.md`, 8).
- **The rules' digest is in the layer's record** (hearth/docs/design/domain/store/records.md, 2).

## 8. The tools as protocols

Each tool is a protocol machine: an argument vector goes down; stage,
progress, a result or a typed failure with a bounded tail comes up. The
domain never sees a tool's text.

### 8.1 mmdebstrap

`hearth-protocol-mmdebstrap`. Its argument vector is `packages.md`, 4's, its hooks
`packages.md`, 5's and `packages.md`, 6's.

- **On a pipe there is no progress bar:** mmdebstrap prints it only when
  stderr is a terminal. It prints `I:`, `W:` and `E:` lines and, with
  `--verbose`, apt's and dpkg's own lines between them.
- **Stages,** from 1.5.7's `I:` lines:

  ```
  I: running apt-get update...                              update
  I: downloading packages with apt...                       download
  I: extracting archives...                                 extract
  I: installing essential packages...                       essential
  I: installing remaining packages inside the chroot...     remaining
  I: running --<stage>-hook …                               a hook
  I: cleaning package lists and apt cache...                cleanup
  I: success in <seconds> seconds                           done
  ```

  An `I:` line it does not know is reported as other, not refused;
  apt's `Ign:` and `Fetched` lines are ignored.
- **Progress** within essential and remaining counts dpkg's `Setting up
  <package> (<version>) ...` lines against the closure's size.
- **The failure tail** keeps the last 20 lines that match os's list
  (`dpkg: error`, `E: `, `Err:`, `No space left on device`, `Hash Sum
  mismatch`, `Failed to fetch`, `Unable to `, `Could not `) and none of
  mmdebstrap's own framing (`I: PID n exited with exit code`,
  `E: mmdebstrap failed to run`, and 1.5.7's framing of apt's failure,
  `E: setup failed: `, `E: apt-get update … failed: process exited
  with` and `W: hooklistener errored out`), each cut at 160 bytes. It is
  classified: a maintainer script failed (and whose), no space, a
  package missing from the repository, other.
- **Bounds:** a line at most 4096 bytes, the rest cut; the tail 20
  lines.

### 8.2 `apt-helper`

`hearth-protocol-dpkg`, which holds dpkg's and apt's machines.

- **Argument vector:** `/usr/lib/apt/apt-helper download-file <uri>
  <target> SHA256:<hex> …`, at most 256 triples; `APT_CONFIG` the
  builder's.
- **Output** (seen with apt 3.3.3; trixie's apt 3 is to record):
  `Get:<n> <uri> [<size>]` as each file starts; on a failure `Err:<n>
  <uri>`, indented reason lines (`Hash Sum mismatch`, the HTTP status),
  `E: Failed to fetch <uri>  <reason>`, then `E: Download Failed`; exit
  0 or 100.
- **Events:** each file started; each failure with its kind (not found,
  mismatch, network, other). After the exit the builder looks for each
  target: those present are kept.

### 8.3 `dpkg-query`

`hearth-protocol-dpkg`: `packages.md`, 7's argument vector; lines of four
tab-separated fields, decoded to name, architecture, version and a
three-letter status; at most the closure's size and a margin of lines,
each at most 1 KiB.

### 8.4 The target's tools and the scripts

`systemd-sysusers`, `systemd-tmpfiles`, `systemctl`, `tar`,
`software.sh` and `customize.sh`: an exit status and the last 20 lines
of their output. The tail is kept by `hearth-protocol-tail`, a machine
crate of its own (lib-only, as `skein/docs/foundation/programming-model.md`, section 4 has
every machine), which `hearth-protocol-builder` routes to. Nothing more
is parsed. A status other than 0, a signal or a deadline fails the step
with its tail.

### 8.5 Recordings

- **Every machine is tested against real runs** (hearth/docs/design/testing/tool-fakes.md):
  the argument vector, the environment, the bytes of stdout and stderr
  as they came, the exit status, recorded by the conformance tier on the
  local mirror (hearth/docs/design/testing/README.md, section 3).
- **They replace os's hand-written fixtures** (a 17-line `Inst` sample,
  a 22-line failure log), and are recorded again when a tool moves.
- **What a recording settles, the design does not restate:** a
  maintainer script's failure line, the fetch's environment,
  `dpkg-query`'s margin and the `--version` formats behind `tools`
  (hearth/docs/design/domain/store/records.md, 2) are taken from the recordings, with the rules above:
  `Ign:` and `Fetched` ignored, a pin with no architecture the
  target's.

## 9. Caches

- **Layers by key** (hearth/docs/design/domain/store/README.md, section 4): a config change reuses the
  software and packages layers.
- **Resolutions by key** (hearth/docs/design/domain/render/spec.md, section 2): unchanged sources and
  requests resolve with no network.
- **Downloads by SHA-256** (hearth/docs/design/domain/store/README.md, 2.1): `.deb`s, indices and
  software, shared by every layer and target; a `.deb` is fetched once
  per store.
- **An apt proxy is optional,** the fetcher's from the builder's
  configuration (apt-cacher-ng on a pool's host, `hearth/docs/design/draft/archive/factory.md`,
  6.5), never in a key. snapshot.debian.org already redirects to
  content-addressed URLs.
- **Peers [M6]:** change sets and blobs by digest, within a plane
  (hearth/docs/design/domain/store/README.md, section 8).
- **Not cached in M2:** work inside a packages build (an essential base
  shared by closures); compile caches (hearth/docs/design/domain/build/artifacts.md, section 6).

## 10. Costs

- **Facts, per step and per build** (hearth/docs/design/protocol/wire.md, section 5): wall and CPU
  time (cgroup `cpu.stat`), peak memory (`memory.peak`), bytes fetched,
  bytes written (from the walk), entries walked, files and bytes
  hashed, hits and misses per cache. The layer's record keeps them
  (hearth/docs/design/domain/store/records.md, 2).
- **What to expect:**

  | Work | Cost | Source |
  |---|---|---|
  | essential bootstrap, local mirror on tmpfs | 9.5 s; the `apt` variant 11 s | mmdebstrap's README |
  | essential trixie from deb.debian.org | 28.6 s, 3,940 paths | measured, unprivileged |
  | a host's packages layer, about 1,000 packages | minutes | an estimate, to measure |
  | snapshot, freeze, publish | constant | btrfs |
  | walking an unchanged root of 100,000 entries | under a second | an estimate (hearth/docs/design/domain/store/tree-helper.md, 7) |
  | hashing a packages layer | its bytes at the SHA-256 rate | to measure (hearth/docs/design/domain/store/tree-helper.md, 7) |
  | a config change, end to end | seconds | an estimate |
  | deleting a layer without the mount option | about 15 µs a path | measured |

- **hearth's own memory is a cost fact too:** the steward's, idle,
  since it always runs, and a steward that grows is a finding; the
  builder's at its peak, and its start, the store's scan (hearth/docs/design/domain/store/README.md,
  6.2), which every pass pays. The heavy work runs in child processes
  that exit, and an idle machine runs no builder at all
  (hearth/docs/design/domain/steward.md, 4.1).
- **A regression is a finding:** the lab benchmarks hearth as it does
  any product, and the real tier's second run builds nothing, which its
  costs show (hearth/docs/design/testing/README.md, section 4).

## 11. Its world

- **`hearth-domain-build`'s world** runs the build's domain with the
  store's (or a scripted store) on the fake machine
  (hearth/docs/design/testing/fake-machine.md). Its fakes: mmdebstrap, installing a seeded
  universe from a local repository, with users made by maintainer
  scripts, conffiles, a kernel, and output from recordings; the fetcher;
  `dpkg-query`; the target's systemd tools and `tar`; scripts in
  temper's shell subset; the tree helper.
- **Faults from the seed:** a 404, a mismatch or a stall in a fetch; a
  maintainer script that fails; a full disk; a step past its deadline;
  a step that leaves an offender; a power cut mid-step.
- **Invariants** (hearth/docs/design/testing/worlds.md):
  - no published output holds an offender of 7.2, as the world checks
    on the fake tree on its own;
  - every step's processes saw only their view, and no network;
  - a failed step publishes nothing, and the layers beneath stay;
  - a packages layer's installed set is its closure;
  - keys follow hearth/docs/design/domain/store/README.md, 3.3: a second build of a spec builds nothing,
    a config change builds only config;
  - every step reports its costs.
- **Real builds** run as a regular user against the local mirror
  (hearth/docs/design/testing/real-builds.md).

## 12. What changes from os

- **Unprivileged:** os's deploy ran as root through sudo in practice;
  the builder runs as itself, and the activator shifts at prepare.
- **Layers by key, shared between targets,** replace `@T`, `@T.var` and
  the pristine `@T.install` snapshots named by target; `/var` stays in
  the layer until prepare.
- **Reconcile kept, by key:** as os ran apt install and purge on its
  pristine snapshot, hearth reconciles from the target's previous
  packages layer, but bounds the history with periodic scratch builds
  and leaves the comparison of the two to the lab (4.2).
- **Offline, from a local repository,** where os fetched through
  apt-cacher-ng with the proxy as an `--aptopt` left in the output.
- **mmdebstrap's cleanup kept,** where os passed `--skip=cleanup`.
- **Sandboxes with `/proc`, `/sys` and `/dev`,** where os used
  `mmdebstrap --unshare-helper` and `chroot` with none, and left a fake
  `/dev/urandom` in the rootfs.
- **The target's own systemd tools,** where os ran the host's with
  `--root`; **`software.sh` with the target's tools,** offline.
- **Modes and owners as data, applied exactly,** where `rsync -rltD`
  kept packaged modes and tmpfiles `z` lines fixed them later.
- **No etckeeper and no secrets in the build:** the `pristine-debian`
  tag and `Manifest config` commit go, and so does `/secrets`, which os
  copied into the output for `customize.sh`.
- **The fleet leaves the code:** os's Etc/UTC and en_US preseeds become
  part data; the variant comes from placement; the initramfs
  configuration enters the packages part with `MICROCODE` explicit.
- **Verification adds architecture and status;** no `/.manifest/` in the
  output, whose records live beside the layer (hearth/docs/design/domain/slots/prepare.md, 5 for the
  slot's).
- **Recordings replace hand-written fixtures;** pipes and deadlines
  replace `os-proc`'s one-second polling of files.

## 13. Open questions

- **The bounds on a reconciled line:** a depth of 8 and a period of 30
  days are guesses; they depend on what the lab's comparison of
  reconcile and scratch finds (`hearth/docs/design/draft/archive/factory.md`, 6.4), and on how
  often snapshots are bumped.
- **Preseeds where no essential hook runs:** the `custom` variant, if
  containers take it (hearth/docs/design/protocol/format/README.md, section 11), and preseeds for
  `Essential: yes` packages; depends on whether the extract hook can
  load them with an unconfigured debconf.
- **`--mode=unshare` or `--mode=root`** inside the sandbox; depends on
  the conformance tier's comparison.
- **The local repository's details:** whether apt asks a `file:` source
  for a by-hash path or a compressed index before the plain `Packages`,
  and whether `trusted=yes` is needed; depends on recordings of apt on a
  `file:` source.
- **The builder's tool versions** (mmdebstrap, apt, `dpkg-deb`) are
  recorded but not in the packages key; depends on whether two versions
  ever build different trees from one closure.
- **The snakeoil pair:** removed, or made per machine by the activator
  as identity; depends on hearth/docs/design/domain/slots/README.md's identity (`packages.md`, 4).
- **setuid, setgid and capabilities added above packages,** by an
  archive's modes or a script: allowed, or refused unless declared;
  depends on hearth/docs/design/protocol/format/README.md (`packages.md`, 8, `README.md`, 8.1).
- **Nested subvolumes made at build** beyond `/var/lib/machines` and
  `/var/lib/portables` (`packages.md`, 5), which trixie's systemd 257 is to confirm;
  depends on recordings of a bootstrap of the fleet's closures.
- **The sandboxes' hostname,** `localhost`, and whether it should be
  something a scan can find in files; depends on recordings of the
  packages that write it.
- **seccomp in sandboxes;** depends on skein's contained trees.
- **The limits' defaults;** depends on the real tier's costs.
- **`SOURCE_DATE_EPOCH` per layer;** depends on hearth/docs/design/domain/build/reproducible.md.
- **`packages.md`, 4's hooks and `packages.md`, 5's:** `packages.md`, 4's argument vector lists setup, essential
  and customize hooks, while `packages.md`, 5 also needs an extract hook; one of the
  two is to change.
