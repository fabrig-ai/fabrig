# Sandboxes

## 1. Namespaces

Each build step runs in namespaces of its own (hearth/docs/design/decisions.md):

- **user:** uids and gids `0 <base> 65536`, written by `newuidmap` and
  `newgidmap`; the builder's own ids unmapped, so what it owns reads as
  `nobody` and through its "other" bits. The step runs as uid 0, with
  the namespace's capabilities.
- **mount:** a root of its own (2), `pivot_root`, propagation private.
- **pid:** every process dies when the namespace's first process ends.
- **ipc:** its own.
- **uts:** hostname `localhost`. Maintainer scripts write the hostname
  into configuration (postfix's `myhostname`), and a packages layer is
  shared by targets, so the name is fixed and none of theirs, and never
  the builder's.
- **network:** its own, with loopback only (5).
- **`no_new_privs`,** and no core dumps.

## 2. Two views

```
the packages step: a tool root                    every other step: the layer as root
/                  tmpfs, read-only once made     /                     the staging layer, writable
/usr               the builder's, read-only       /run, /tmp, /var/tmp  tmpfs: nothing there is kept
/bin /sbin /lib*   links into /usr                /run/hearth/…         the step's inputs, read-only
/etc               composed by hearth, below      /proc /sys /dev       8.4
/target            the staging layer, writable
/run/hearth/repo   the local repository, read-only
/run/hearth/input  sources, seeds, preseeds, initramfs configuration, read-only
/tmp               tmpfs
/proc /sys /dev    8.4
```

- **The packages step runs in a tool root and chroots into the layer.**
  Its target starts empty, and mmdebstrap runs the builder's tools
  (mmdebstrap, perl, apt, `dpkg-deb`) outside it. So that sandbox's
  root is a tmpfs holding the builder's `/usr` read-only and an `/etc`
  of hearth's own, with the layer at `/target`, and mmdebstrap chroots
  from it into the layer. Every other step is pivoted into its layer
  itself (hearth/docs/design/decisions.md).
- **The tool root's `/etc` is composed, never the machine's.** On a
  machine, `/etc` holds the slot's machine-id and host keys and the
  secret files written at prepare; a maintainer script that left its
  chroot could copy any it can read into the layer. So `/etc` is a
  tmpfs, read-only once made, with only what mmdebstrap, apt and perl
  need:
  - `passwd` and `group`, written by hearth: `root`, `_apt` and
    `nogroup`;
  - `nsswitch.conf` (`files`); `hostname` (`localhost`), which
    mmdebstrap copies and the customize hook removes (`packages.md`, 6); no
    `resolv.conf`;
  - `mtab`, a link to `/proc/self/mounts`;
  - the builder's `ld.so.cache`, `ld.so.conf`, `ld.so.conf.d/` and
    `alternatives/`, bound read-only, which its binaries need to load;
  - no apt configuration and no keyring: mmdebstrap points apt at the
    target with a configuration of its own, and the sources are
    `trusted=yes` (`packages.md`, 3).

  Whether mmdebstrap 1.5.7 reads anything more is to verify on the
  conformance tier.
- **Nothing else is visible:** no other layer, no store, no home.
- **A maintainer script that leaves the chroot** lands in the tool
  root: the builder's `/usr` and hearth's `/etc`, read-only, with
  nothing of the machine's own.

## 3. mmdebstrap inside the sandbox

**Decided: hearth's sandbox around, mmdebstrap inside as its root,
`--mode=unshare`.**

- **As root, unshare mode makes no user namespace.** It unshares mount,
  pid, uts and ipc, nested in the sandbox's, and runs no `newuidmap`:
  1.5.7's `get_unshare_cmd` adds `CLONE_NEWUSER` only when not root. Its
  own pid namespace reaps what maintainer scripts start. Unable to
  `mknod`, it binds the sandbox's devices into the chroot.
- **Not mmdebstrap around, as the builder:**
  - its isolation is a chroot, which a process holding `CAP_SYS_CHROOT`
    in its namespace leaves, to see the builder's filesystem as the
    range's root sees it;
  - it never unshares the network (`if (0) { … CLONE_NEWNET }` in
    1.5.7), so maintainer scripts would reach it;
  - its limits, stop and proof of emptiness would differ from every
    other step's.
- **Not nested as a non-root user:** a second user namespace needs
  `newuidmap`, which is setuid and so disabled by `no_new_privs`, and
  could map only ids the sandbox maps already.
- **`--mode=root` is the alternative.** mmdebstrap's code allows for
  root mode in a user namespace ("we don't know whether we run in root
  mode inside an unshared user namespace or as real root"); the
  conformance tier runs both (hearth/docs/design/testing/README.md, section 3).

## 4. `/proc`, `/sys` and `/dev`

- **`/proc`:** a new procfs for the sandbox's pid namespace, `nosuid`,
  `nodev`, `noexec`. The kernel allows it only where a fully visible
  `/proc` exists; in an Incus container that takes
  `security.nesting=true`.
- **`/sys`:** a new sysfs, read-only; the sandbox owns its network
  namespace, which the kernel requires for it.
- **`/dev`:** a tmpfs (`nosuid`, `noexec`) with the builder's `null`,
  `zero`, `full`, `random`, `urandom` and `tty` bound in; a new
  `devpts` instance with `ptmx` linked to it; a tmpfs at `shm`; `fd`,
  `stdin`, `stdout` and `stderr` linked into `/proc/self/fd`.
  bubblewrap's set.
- **os ran its chroots with none of them,** and wrote 8 KiB of the
  host's randomness into the rootfs as a fake `/dev/urandom`, never
  removed. Here the device is real and only in the sandbox; a layer's
  `/dev` stays empty, and the scan refuses anything in it.

## 5. Offline

- **A build step's network namespace has loopback only:** no route, no
  DNS, no proxy.
- **Only the fetch reaches the network** (`packages.md`, 1, `README.md`, 5.1), and it runs no
  code from the repository.
- **A maintainer script or `customize.sh` that downloads fails,** and
  the failure tail says so. What it wanted becomes a download of the
  software part.

## 6. The environment

- **Given whole, fixed, and covered by the key:** `PATH`,
  `HOME=/root`, `LC_ALL=C.UTF-8`, `DEBIAN_FRONTEND=noninteractive`,
  `SYSTEMD_OFFLINE=1`; then the step's own, as above.
- **Never:** the target's name, the revision, the builder's hostname,
  its variables, its proxy. Inputs are files at fixed paths. A step
  can still read the clock and count processors; that is
  hearth/docs/design/domain/build/reproducible.md's.

## 7. Limits and stopping

| Limit | Default | By |
|---|---|---|
| memory | 4 GiB for packages, 2 GiB otherwise | cgroup `memory.max` |
| processes | 4096 | `pids.max` |
| wall time | 60 min for packages, 15 min otherwise | the domain's deadline |
| progress | 10 min with no output | the domain's deadline on the pipes |
| disk | the store's free-space floor (hearth/docs/design/domain/store/README.md, 5.2) | `statfs` while the step runs |

- **Deadlines are the domain's;** the protocol layer runs the timers.
  The steward bounds the whole job besides, and stops a builder that
  outlives it (hearth/docs/design/domain/steward.md, 5.2).
- **The builder's cgroups** (hearth/docs/design/domain/steward.md, section 7). On a machine the
  activator's unit has `Delegate=yes`; the activator moves itself into a
  leaf, `activator/`, and makes `steward/`, owned by `hearth`. The
  steward moves itself into `steward/self/` and spawns each builder into
  `steward/builder/`, with `memory` and `pids` enabled and a memory
  ceiling of its own (6 GiB to start), so a build the kernel must kill
  for memory is killed within the builder's subtree, never the
  steward. On a developer's machine the client runs under `systemd-run
  --user --scope -p Delegate=yes`. cgroup v2 lets no cgroup with
  controllers enabled for its children hold processes itself (the
  no-internal-processes rule), so the builder moves itself into a leaf,
  `builder/self/`, enables both controllers in `builder/`'s
  `cgroup.subtree_control`, and gives each sandbox, fetch and tree
  helper a leaf of its own beside it, named by a token and removed once
  empty. The limits above are written to those leaves.
- **Stopping is skein's three steps:** the termination signal to the
  first process, then to every process, then a kill of the cgroup. A
  step is over when its cgroup is empty (`cgroup.events`), and only
  then is its layer walked or discarded.

## 8. What it asks of skein

hearth/docs/design/README.md (1) asks for contained process trees with id maps through
`newuidmap`. Beyond contained process trees, sandboxes need:

- **a view that is only what is given:** a root the owner composes and
  `pivot_root`, rather than the host read-only with given directories
  writable;
- **user namespaces mapped by `newuidmap` and `newgidmap`;** network,
  pid, ipc and uts namespaces, and a hostname;
- **a new `/proc` and `/sys`, a minimal `/dev`;**
- **`no_new_privs` and rlimits.**

The domain and its worlds need none of this
(hearth/docs/design/testing/real-builds.md).
