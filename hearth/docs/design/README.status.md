# hearth's design: status

2026-11-11. Where hearth's code lags or departs from [README.md](README.md).

- **Not yet:** 4.1: the activator, steward and their link and jobs; the
  delegated cgroup subtrees; bootstrap. The binary hosts the builder
  in-process, without the daemon's other processes.
- **Not yet:** 4.2: the remaining domain crates: `hearth-domain-steward`,
  `hearth-domain-converge`, `hearth-domain-checks`, `hearth-domain-activator`,
  `hearth-domain-slots`, `hearth-domain-boot`, `hearth-domain-disk`,
  `hearth-domain-backups` and `hearth-domain-faults`.
- **Not yet:** 4.2: the remaining protocol crates: `hearth-protocol-dpkg`,
  `hearth-protocol-tail`, `hearth-protocol-xattr`, `hearth-protocol-link`,
  `hearth-protocol-jobs`, `hearth-protocol-state`, `hearth-protocol-units`,
  `hearth-protocol-efi`, `hearth-protocol-systemd`, `hearth-protocol-proc`,
  `hearth-protocol-keygen`, `hearth-protocol-grub`, `hearth-protocol-wire`,
  `hearth-protocol-probe`, `hearth-protocol-http`, `hearth-protocol-steward`,
  `hearth-protocol-activator` and `hearth-protocol-helper`.
- **Not yet:** 4.2: the client crates and worlds remain `client/README.status.md`, 2; `hearth-service-steward`, `hearth-service-activator`, `hearth-service-tree`, `hearth-fake-engine` and `hearth-system-world` remain unbuilt. The builder and tree helper have separate worlds, but no system world joins their domains.
- **Departs:** 6: the 32 printed errors in
  `hearth/crates/hearth-client-protocol-records/src/errors/reserved.rs` and
  `hearth/crates/hearth-client-protocol-records/src/errors/repository.rs`,
  and the tests and design quotations that expect them, name documents
  without the repository-root prefix, as in `protocol/format/README.md, 7.1`.
  Task JSON's `doc` strings and their copies in design examples keep the
  same form. A printed error and a digest's input are hearth's behaviour,
  so these strings keep their bytes rather than follow the citation
  convention.
- **Departs:** 6: the `reason` strings of hearth's `clippy.toml` files cite
  `skein/docs/design/runtime/io/processes.md`` and "skein's programming-model.md, 10.2" without the
  repository-root prefix, because each file is a copy of
  `tools/lints/clippy/*.toml`, which `tools/lints/lints.py` holds equal.
- **Not yet:** 7.1: a full stat by name without following links, readlink
  and extended attributes; metadata writes for symlinks, hard links,
  chmod, chown and extended attributes. Skein's stat reports only kind,
  size, permission bits, owner and link count; mkdir and truncate exist.
- **Not yet:** 7.1: spawning as another user with supplementary groups;
  contained process trees with id maps, a composed root and namespaces;
  the closed list of btrfs ioctls; `statfs`'s filesystem type; `syncfs`.
  Process-group signalling exists. Cgroup v2 subtree creation, delegation,
  movement and `cgroup.kill` are absent; the shell can spawn a binary into
  an already-open delegated cgroup, but the step boundary cannot.
- **Not yet:** 7.1: Unix sockets with peer credentials; redialling and
  identity on the channel's TLS client; `utimensat`; `RESOLVE_NO_XDEV` and
  `FICLONE` (the no-follow opens already use `RESOLVE_BENEATH` and
  `RESOLVE_NO_SYMLINKS`); the new mount API and detached-mount walks,
  `umount2` with `MNT_DETACH`, whiteouts through `mknodat`, and `setns`.
- **Not yet:** 7.1: in-process Ed25519 and X25519 key generation; HTTP
  ranges and file-backed bodies.
