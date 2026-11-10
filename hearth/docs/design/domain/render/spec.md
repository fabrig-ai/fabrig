# The spec and its resolution

## 1. The spec: parts and digests

### 1.1 The parts

| Part | Kind | Holds | Enters |
|---|---|---|---|
| `packages` | output | sources, the request, the variant, preseeds, the initramfs, apt's options | the packages key, and the resolution key |
| `software` | output | downloads by digest and where each is placed; artifacts by key [M6]; `software.sh` | the software key |
| `config` | output | the file tree; users; presets; tmpfiles; `customize.sh` | the config key |
| `data` | slot | data entries, each a directory in a class, bound at its path; each class's snapshot and backup policies | no key |
| `secrets` | slot | secret files, password secrets and identity items, by name | no key |
| `boot` | slot | the kernel's command-line parameters; a container's parameters instead [M5] | no key |
| `checks` [M5] | other | the target's checks (hearth/docs/design/domain/operating.md, section 2) | no key |
| `live` | other | the consumers of the target's paths, for live changes (hearth/docs/design/protocol/format/tasks.md, 14) | no key; read by the builder to plan a live trial |
| `exports` | other | what this target publishes to its environment | no key; read by other targets' renders |
| `meta` | | target, base, member, environment, placement | no key |

- **Every part M1 builds is in every spec,** an empty one too: the keys
  and the spec digest need each part's digest (1.5). A part with nothing
  in it is its record with each field empty, a field left out only when
  it is optional and absent (hearth/docs/design/domain/store/README.md, 3.1), so no part is ever lacking.
  A part a later milestone adds is left out until then. Parts are
  printed in this table's order.

### 1.2 Output parts

```
packages
  sources
    suite              string                          forky
    architecture       string                          amd64
    components         set of component, in fixed order main contrib non-free non-free-firmware
    archive            timestamp                       20261001T030405Z
    updates            bool
    security           timestamp; absent when the pocket is off
  request              set of package names
  variant              apt for metal and vm; container for a container, until M5 settles it (../../protocol/format/README.md, section 11)
  preseeds             map question → { owner, type, value }
  initramfs            { modules: set of names, microcode: none | generic }; absent for a container
  apt                  { install_recommends }
  seeds                users: map name → { uid, gid }; groups: map name → { gid }

software
  downloads            map path → { sha256, owner, group, file { mode } | tar { strip } }      ../../protocol/format/README.md, 8.1
  artifacts [M6]       map path → { pinned { key } | head { recipe, listing digest } }
  scripts              list of { task, order, content digest }        software.sh, in order

config
  tree                 map path → one of
                         file       { content digest, mode, owner, group }
                         link       { target, owner, group }
                         directory  { mode, owner, group }
  groups               map name → { gid? } | package
  users                map name → { uid?, group, gecos, home, shell } | package
  members              set of (user, group)
  presets              map unit → enable | disable
  tmpfiles             map path → { type, mode?, owner?, group?, age?, argument? }
  scripts              list of { task, order, content digest }        customize.sh, in order
```

- **The variant is a function of placement,** never a task
  (hearth/docs/design/protocol/format/README.md, 3.4): os chose it from a `container` task in render and
  from the target's kind in deploy, which disagreed. A container's
  variant changes its resolution key (2.1), so a host and its container
  never share a packages layer. Until M5 settles what mmdebstrap is
  given, `container` is a placeholder: it enters the key, but render asks
  no resolution for it, so its packages key stays needed, with
  `--resolve` too, and that is no error (2.7).
- **A container's packages part has no initramfs,** and render refuses
  a task's module on it, as it refuses a boot parameter (hearth/docs/design/protocol/format/README.md, 3.4;
  `errors.md`, 2): with no kernel there is nothing to configure.
- **The initramfs configuration is render's,** written as
  `tiny-initramfs.conf` from the part's two fields: `AUTO_MODULES=no`
  always, `INCLUDE_MODULES` from `modules`, and `MICROCODE=no` or
  `generic` from `microcode` (hearth/docs/design/domain/machine/boot.md, 4.1; hearth/docs/design/domain/build/packages.md, 5). Render
  derives `microcode` from placement's kind, `none` on a VM and `generic`
  on metal; placement enters no key, so the part must carry the choice
  for the build to know it. Nothing a task writes can set `yes` for
  either. Render writes the text into the cache, as a rendered file
  (`README.md`, 3.6), and the spec names its content digest beside the part, outside
  the part's digest, which the two fields already make; build copies it
  from there into its inputs (hearth/docs/design/domain/build/packages.md, 4).
- **`seeds` are the fixed ids build writes into `/etc/passwd` and
  `/etc/group`** in mmdebstrap's extract hook, before any package is
  configured (hearth/docs/design/domain/build/packages.md, 5): every package user and group, and every
  owner of a data path, a secret or an identity file, with its uid and
  gid (`README.md`, 3.3). They change what the packages layer builds (adduser keeps a
  user it finds), so they belong to its key; they do not change what apt
  resolves, so not to the resolution's.
- **`install_recommends` is the part's one option.** hearth's fixed apt
  and dpkg options (`Check-Valid-Until "false"`, `force-confold`, and the
  rest hearth/docs/design/domain/build/packages.md, 4 passes) are constants of the encoding's version
  (hearth/docs/design/domain/store/README.md, 3.2): changing one changes every key, and no part carries
  them.
- **A download's owners** are `root`, or a package user or group with a
  fixed id: software is placed before any sysusers user exists, and
  render refuses any other owner (hearth/docs/design/protocol/format/README.md, 8.1).
- **A pinned artifact is its key** (hearth/docs/design/domain/build/artifacts.md, 7.1); its repository,
  commit and recipe ride beside the part, outside its digest, as how to
  make it. **A head artifact is its recipe and its sources' listing**
  at the revision: the SHA-256 of the `ls-tree` records of the paths the
  recipe selects in `artifacts.json` (hearth/docs/design/domain/build/artifacts.md, 3.2), from render's
  own listing. Its resolved key never enters the part (1.5).
- **A download is what it is, not where it comes from.** Its URL stays
  outside the digest, as a fetch hint beside the part (map from SHA-256
  to URL), so moving a release to a mirror rebuilds nothing. Its name
  stays out too; the path keys it. Two downloads of one SHA-256 give the
  hint the URL of the first by path; a URL is text, kept as written.
- **Owners are names** in every part; build resolves them in the layer
  (hearth/docs/design/protocol/format/tasks.md, 5). Those of data, secrets, identity and software have
  fixed ids (`README.md`, 3.3).
- **A package user or group** is a name only (hearth/docs/design/protocol/format/tasks.md, 6): build
  writes no sysusers line for it, and the activator confirms it in the
  slot's `/etc/passwd` or `/etc/group` at prepare (hearth/docs/design/domain/slots/prepare.md, 7).
- **Each data entry adds a `tmpfiles` line** of type `d` with its mode
  and owners (hearth/docs/design/protocol/format/tasks.md, 7), so they are applied at boot in the running
  slot (hearth/docs/design/domain/machine/README.md, 3.4).

### 1.3 Slot parts

```
data
  entries              map name → { class: core | data | cache, path }
  policies             map class (core, data) → { snapshots? { every, keep } [M5],
                         backup? { every, keep, peer, recipient } [M6] }
secrets
  files                map secret name → { path, mode, owner, group }
  passwords            map user → secret name
  identity             map item → { kind, algorithm?, path, public_path?, mode, owner, group }
boot                   { mutable, params }: mutable from placement (../development.md, section 4);
                         params the kernel's command-line parameters, a list in order; for a container,
                         empty in M1, and from M5 its own parameters instead:
                         { ids, binds: map path → { source, read_only }, capabilities, network, bridge? }
```

- **Slot parts never enter a layer key:** changing one makes a new slot of
  the same output (hearth/docs/design/domain/slots/prepare.md).
- **A container's slot parts are its own,** placed by its host's
  activator into the container's slot on the host [M5] (hearth/docs/design/domain/slots/containers.md, 1).
  Its `boot` part holds no command line, since it has no kernel, but
  the parameters its placement gives nspawn (hearth/docs/design/protocol/format/README.md, 3.5): what
  starts the slot, as a host's command line is, so a change to them
  prepares a new slot of the same output.
- **The data part's entries are the tasks' `data` entries,** a name, a
  class and a path each (hearth/docs/design/protocol/format/tasks.md, 7); mode and owners go to the
  config part's tmpfiles lines instead (1.2). **Its policies are the
  `hearth` task's `snapshots` and `backups` settings** (hearth/docs/design/protocol/format/tasks.md, 12),
  by class, a backup's peer and recipient read from the `backup_peer`
  task's exports (hearth/docs/design/protocol/format/README.md, 6.7). A host carries every class's policy
  its settings give, used or not, since its containers' classes follow
  them; a container's data part carries none (hearth/docs/design/domain/slots/containers.md, 1).
- **The secrets part holds names, never values.** Beside secret files it
  carries password secrets and identity items, the two other things the
  activator writes into a slot from the secret store
  (hearth/docs/design/protocol/format/README.md, section 7; hearth/docs/design/domain/slots/prepare.md, 4).

### 1.4 Other parts and meta

```
checks                 from M3, the units whose failure does not block a bless (../../protocol/format/tasks.md, 13);
                       [M5] the target's checks, and the probes it runs for others (3.9; ../operating.md, section 2)
live                   map path → list of { task, unit?, action }, in path order, each path's consumers
                       by task, then unit (none first), then action; a path ending in `/` covers its
                       subtree (../../protocol/format/tasks.md, 14)
exports                map task → map name → value, empty when no task exports, every integer in a value
                       at any depth as its decimal text (4.6); and [M5] the task's probes with `from`, rendered
meta                   { target, base, member?, environment, placement { kind, firmware?, hardware?, on?, and a container's parameters [M5] } }
```

- **Exports have a digest** so a target's own diff shows what it
  publishes; a change to one reaches its readers through their own parts,
  whose content changes.
- **Meta enters no key:** two targets whose output parts agree share an
  output, whatever their names.
- **The live part enters no key, not even the prepare key:** how a
  change is applied is not what a slot holds, so a change to `live`
  entries alone builds and prepares nothing.

### 1.5 Keys

```
part digest        = H("part:<name>", the part's record)
resolution key     = H("resolution", sources, request, variant, apt)                 5.1
resolution digest  = H("closure", list [ list [ name, architecture, version, sha256 ] … ])  5.4
packages key       = H("packages", resolution digest, packages part digest)
software key       = H("software", packages key, software part digest
                         [, M6: the resolved key of each head artifact, by path])
config key         = H("config", software key, config part digest)                   the output's name
spec digest        = H("spec", map part name → part digest, meta's among them)
```

- **The packages key takes the closure's digest,** not the resolution's
  key. Two builders that resolved one key to different closures (another
  apt, say) then name their layers differently, and a key never names two
  sets of installed packages.
- **A head artifact's resolved key enters the software key,** as the
  closure's digest enters the packages key: the part holds only what the
  revision says, so render stays a function of it, and the key is
  remembered by the listing's digest (hearth/docs/design/domain/build/artifacts.md, 7.2) [M6].
- **The output is the config layer,** and its key is the output's name
  (hearth/docs/design/domain/store/README.md, section 4).
- **The spec digest** answers "did anything change for this target";
  the revision's commit id rides beside it, outside every digest, so two
  commits that render the same give the same digests.

### 1.6 What render needs from the canonical encoding

hearth/docs/design/domain/store/README.md, section 3 defines the encoding and H. Render relies on it for:

- **one injective encoding of typed records:** integers at fixed width,
  byte strings and lists with their lengths, booleans as kinds of their
  own, enumerations as their variants' names, optional fields left out
  when absent, sets as sorted lists, maps sorted by key bytes, and a
  record's fields as a map of its field names (hearth/docs/design/domain/store/README.md, 3.1);
- **a tag per kind of record,** distinct between part digests and keys,
  with a version, so equal bytes of two kinds never share a digest and a
  change of encoding changes every key;
- **H as SHA-256 over the tag and the encoding,** computable in the
  domain (`no_std`), with a digest a fixed 32 bytes inside other records.

The encoding has no signed integers. An integer setting may be negative,
so every integer in an export's value, at any depth, enters as its
decimal text; every other integer in a part (ids, modes, orders, sizes)
is unsigned.

A field's name in the encoding is its JSON name: a two-word field is
snake case in both (`content_digest`, `install_recommends`,
`public_path`), where the blocks above may write it as two words.

## 2. Resolution

### 2.1 The key

```
resolution key = H("resolution", sources, request, variant, apt)
```

- **It covers everything that changes what apt resolves:** the sources
  (suite, architecture, components, snapshot timestamps, pockets), the
  requested packages, the bootstrap variant, and `install_recommends`;
  hearth's fixed options change it through the encoding's version (1.2).
- **It leaves out what does not:** preseeds, the initramfs, the seeds,
  the proxy and where the bytes come from.

### 2.2 The invocation

```
mmdebstrap --verbose --simulate --mode=chrootless
  --variant=<variant>               the part's: apt for metal and vm; never a container's in M1 (5.7)
  --architectures=<architecture>    the sources'
  --include=<names, comma-joined>   the request, sorted; split, and none when it is empty
  --aptopt=Acquire::Check-Valid-Until "false"
  --aptopt=APT::Install-Recommends "<true | false>"    the part's install_recommends
  [--aptopt=Acquire::http::Proxy "<the builder's proxy>"]
  --skip=cleanup/apt/lists
  <suite> <scratch>/root <scratch>/sources.list
```

with `<scratch>/sources.list` encoded from the sources:

```
deb http://snapshot.debian.org/archive/debian/20261001T030405Z/ forky main contrib non-free-firmware
```

one line for each pocket on, each with the sources' components in their
fixed order (1.2): the suite's own; then `<suite>-updates` from the same
archive when `updates` is on; then `<suite>-security` from
`http://snapshot.debian.org/archive/debian-security/<security>/` when
`security` is set.

- **The request is split** into `--include` arguments of at most 64 KiB
  each, as hearth/docs/design/domain/build/packages.md, 4 splits the closure: Linux caps one argument at
  128 KiB, and mmdebstrap joins its `--include`s in order. An empty
  request passes none.

- **Unprivileged, with no namespace:** chrootless simulate needs no
  subordinate ids, where mmdebstrap's automatic mode would pick unshare
  and need them.
- **`--verbose`** makes apt print its `Inst` lines; **`--skip=cleanup/apt/lists`**
  keeps the indices apt fetched, which 2.3 reads.
- **The scratch directory** is new and empty for each run,
  `staging/<resolution key>/` in the store (hearth/docs/design/domain/store/README.md, 2.1), so that its
  indices move into the cache by a rename; it is removed after. One
  resolution runs at a time (2.5), and the store's lock keeps other
  builders out (hearth/docs/design/domain/store/README.md, 6.2).
- **mmdebstrap is `/usr/bin/mmdebstrap`,** spawned by its path, as git
  is (`README.md`, 2.2).
- **apt checks each `InRelease` with the host's keys,** those
  mmdebstrap gives it by default (`/etc/apt/trusted.gpg.d/`, the host's
  `debian-archive-keyring`), with no `--keyring`: a builder resolves
  what its host trusts, and the keys are in no key (2.1). A host that
  lacks them fails the run, an error of kind `tool` (`errors.md`, 2).
- **The environment is given whole:** `LC_ALL=C`,
  `PATH=/usr/sbin:/usr/bin:/sbin:/bin` for the tools mmdebstrap and apt
  run, and `TMPDIR` the scratch directory.
- **The proxy is the builder's configuration,** never in a key (2.1):
  none by default, and none in M1 (hearth/docs/design/client/README.md, 3.5); an
  apt-cacher-ng where the builder has one (hearth/docs/design/domain/build/packages.md, 2); the local
  mirror in the test tiers, which serves the fixed URLs above
  (hearth/docs/design/testing/real-builds.md, 2).

### 2.3 What it yields

- **`Inst` lines give the closure's names:** `Inst <name> [<old>]
  (<version> <origins> [<architecture>])` gives the name, the version and
  the architecture of each package apt would install. Under `--verbose`
  they come on standard error (mmdebstrap 1.5.7, apt 3.0.3), and
  `hearth-protocol-mmdebstrap` reads both pipes as they fill. A line may
  go on after `])` with apt's list of broken packages, ` [<names>]` or
  ` []`; the reader takes the triple and ignores the rest. Each triple
  comes twice, as mmdebstrap simulates the download and then the
  install: equal triples are one package. The closure's bound counts
  packages (hearth/docs/design/domain/store/records.md, 1: 8,192), so the reader takes at most twice as
  many `Inst` lines.
- **The `Packages` indices give the rest.** `hearth-protocol-mmdebstrap`
  reads every `*_Packages` file under `<scratch>/root/var/lib/apt/lists/`
  in bounded chunks, stanza by stanza (at most 1 MiB a stanza: trixie's
  `librust-winapi-dev` has 76,338 bytes), and sends up each stanza's
  `Package`, `Version`, `Architecture`, `Filename`, `Size` and `SHA256`,
  holding no other field. The indices are what apt verified against the
  signed `InRelease`, so their digests can be trusted. apt keeps them
  decompressed (mmdebstrap 1.5.7, apt 3.0.3), each named by its URI
  without the scheme, `_` for `/`; the reader takes the files ending
  `_InRelease` and `_Packages` and skips every other name (`lock`,
  `partial/`).
- **Output the tool should not give is a fault of the tool:** a malformed
  `Inst` line, a pipe that breaks, a stanza past its bound, or a
  `_Packages` or `_InRelease` list that no source line names, that is
  compressed, or that a source line names and is missing, is a `tool`
  error, operational (`errors.md`, 2), as git's malformed output is (`README.md`, 2.2).
- **The domain matches them:** each `Inst` triple must match a stanza;
  several stanzas may match (one per index that holds the package) and
  must agree on size and digest. A triple with no stanza, or stanzas
  that disagree, is a `resolution` error, operational: the indices read
  are not those apt resolved from. The package's source is the first
  matching index's line, in the order of the sources; its URI is that
  line's archive followed by `Filename`.
- **The indices are kept.** Debian, and so snapshot.debian.org,
  publishes no uncompressed `Packages` file; `InRelease` lists the
  uncompressed file's size and SHA-256 so that apt can check it after
  decompressing. apt's lists hold the decompressed, verified files.
  Before the scratch directory is removed, each source line's
  `InRelease` and each `Packages` file moves into the store's cache by
  SHA-256 (hearth/docs/design/domain/store/records.md, 4): the protocol layer hashes each in the pass
  that reads its stanzas, and the domain then has it synced and renamed
  from the scratch directory to `cache/<sha256>`, with no copy, and
  recorded (hearth/docs/design/domain/store/records.md, 5 and 8). The record names them by those
  digests. hearth/docs/design/domain/build/README.md's local repository serves them at plain paths
  (hearth/docs/design/domain/build/packages.md, 3), so the build reads the indices apt verified, offline.
- **The closure** is, per package: name, architecture, version, source,
  archive filename, size and SHA-256, sorted by name, then architecture.
  It is what hearth/docs/design/domain/build/README.md's fetch step needs to fetch each `.deb` by digest
  into its cache.
- **Every requested package must be in the closure;** one that is not
  (apt installs another that provides it, or replaces it) is a
  `resolution` error, configuration: the request names a package apt
  does not install.

### 2.4 The record

```
resolution record                                                            ../store/records.md, 4
  key           the resolution key                                           5.1
  inputs        sources, request, variant, apt: the key's values (5.1), as the packages part holds them
  sources       per source line: uri, suite, components;
                  its indices in the cache, each [ path, size, sha256 ]: InRelease first, then each
                  Packages index, at the path, size and sha256 of its InRelease entry
  closure       per package: name, architecture, version, source, filename, size, sha256
  digest        H("closure", list [ list [ name, architecture, version, sha256 ] … ]),
                  in the closure's order (5.3)                               the resolution digest, 4.5
  made          { at, apt, mmdebstrap, costs: { fetched, took } }            outside the digest
```

- **A record of the store,** found by its key (hearth/docs/design/domain/store/records.md, 4), and
  pinned by the layers whose keys use its digest (hearth/docs/design/domain/store/README.md, section 5).
  Before the store holds layers, in M1, it holds these records alone.
- **The digest covers what is installed,** not where it came from: a
  package's archive and filename stay outside it.
- **An index's InRelease entry** is the line that lists its path in the
  suite's directory, `dists/<suite>/` (`main/binary-amd64/Packages`),
  with the size and SHA-256 of the decompressed file, which apt checked
  and the record repeats; so a cached index is tied to the signed
  `InRelease` beside it, whose own path there is `InRelease`. render
  reads no field of `InRelease`: the size and SHA-256 recorded are those
  of the protocol layer's pass over the file (2.3), which equal the
  entry's because apt checked the file against it.
- **`made` is outside the digest** and says how the record came to be:
  `at`, the wall time when it is recorded; `took`, the run's duration
  from the spawn to the last index read; `fetched`, the bytes apt
  reports on its `Fetched <n> <unit> in <time>` line, in its units (`kB`
  is 1,000 bytes) and as apt rounds them, or 0 when it prints none. A
  simulate run prints neither tool's version, so `apt` and `mmdebstrap`
  are empty in M1; M2 records them from each tool's `--version`.
- **Once recorded, a key's closure does not change** on that builder:
  render reuses it until the record is collected. Re-resolving the same
  key later, with another apt, may give another closure, and so another
  packages key (1.5).

### 2.5 Failures and costs

- **A failed run gives a tail** of at most 20 lines, each cut at 160
  bytes with no marker, chosen by hearth/docs/design/domain/build/README.md 8.1's lists, deny before
  allow, with each indented line that follows a kept line kept with it.
  Resolution adds to them: the tail keeps apt's verdict on the request
  whole, `The following packages have unmet dependencies:` and `However
  the following packages replace it:` with their indented lines, and
  denies mmdebstrap 1.5.7's own framing of apt's failure, `E: setup
  failed: `, `E: apt-get update … failed: process exited with` and `W:
  hooklistener errored out`. apt 3.0.3 under mmdebstrap prints `E:` and
  `W:`, not `Error:`. These lists are the contract, held to recordings
  of real runs (`README.md`, 6.6).
- **The tail gives the cause,** taken over every kept line with its
  indented lines, before the cut to 20:

  | Cause | Lines | Kind, class |
  |---|---|---|
  | a package apt does not know | `Unable to locate package <name>`; `Package '<name>' has no installation candidate` | `resolution`, configuration, naming the package |
  | dependencies apt cannot satisfy | `Unable to correct problems`, or the unmet-dependencies line, named by its first indented line's `<name> :` | `resolution`, configuration, naming the package |
  | a suite, component or architecture the archive does not have | an `Err:` or `Failed to fetch` the archive answered `404  Not Found` (snapshot.debian.org prints `NOT FOUND`), or apt's `does not have a Release file` | `resolution`, configuration, naming the source line: snapshot.debian.org never changes a timestamp's archive, so the same revision fails again |
  | a fetch with no answer, or a broken one | any other `Err:` or `Failed to fetch`: `Could not connect`, `Unable to connect`, `Could not resolve`, `Temporary failure resolving`, a timeout, a hash or size mismatch, a 5xx answer | `tool`, operational |
  | no space | `No space left on device` | `tool`, operational (hearth/docs/design/protocol/standalone.md, 4) |
  | none of these | | `tool`, configuration |

  An operational cause on any kept line outranks every configuration
  one: a fetch that failed leaves apt without the lists it would have
  found the package in. Then the first configuration cause decides.
- **Timeouts:** 60 s without output ends a run, as in os; the domain
  sets an overall deadline (10 minutes by default). Both, and a spawn
  that fails, are `tool`, operational.
- **render never retries.** An operational error tells its caller that
  the same revision may resolve later: the client exits 75
  (hearth/docs/design/client/README.md, 3.3) and the steward backs off (hearth/docs/design/protocol/standalone.md, 4).
- **No marker within a run goes up:** render reports a resolution as it
  starts and as it ends (hearth/docs/design/client/README.md, 3.2), and none of
  mmdebstrap's stages.
- **One resolution at a time per builder:** snapshot.debian.org is fetched
  serially. A run took 9 to 15 s, mostly `apt-get update`.
- **Its costs are facts:** each run's `took` and `fetched` (2.4) go up
  as costs (README, section 2).

### 2.6 What changes from os

- **The key covers the architecture, the variant and apt's options.**
  os's covered the sources text and the names alone, so changing the
  architecture or adding a `container` task reused a stale resolution.
- **`--architectures` is passed;** os resolved for the build host's.
- **It yields digests and URIs,** not only versions, so build fetches by
  digest.
- **The cache is a record in the store,** not the previous spec's
  `versions.json`, and there is no `--resolved` injection.

### 2.7 When render resolves

- **Render never resolves of its own accord.** It gives the resolution
  key and asks the store for its record (`README.md`, 5.2); a key the store does not
  hold is needed, and the spec says so (`README.md`, 3.8, 6.2). No mmdebstrap runs,
  and nothing reaches the network.
- **A container's packages key stays needed in M1:** its variant is a
  placeholder until M5 (1.2), so render asks no resolution for it,
  `--resolve` resolves none, and the spec says needed, with no error.
  Its other parts render, and its host reads its placement (`README.md`, 3.4).
- **The client resolves on request.** `hearth render --resolve` and
  `hearth diff --resolve` (hearth/docs/design/client/README.md, 3.1) resolve each needed key, one at
  a time (2.5), and record each closure before they print: render's
  targets' keys, and for a diff the keys of the targets whose
  resolution key changed, on both sides, so that its packages change
  lists what was added, removed and moved (`README.md`, 4.1). A target that one side
  alone has resolves nothing.
- **A resolution that fails stops no other.** The keys go in the order of
  their first holder, the targets in name order, and every key is tried.
  A key that fails is the error of each target that holds it (`errors.md`, 2), with
  no place (`errors.md`, 1), operational or configuration as 2.5 classifies it;
  each keeps its spec, with that key needed. In a diff the target's
  change stands, its resolution `new key`, and the error prints after
  the changes (`README.md`, 4.1).
- **A build resolves what its plan needs** (hearth/docs/design/domain/build/README.md, 4.1) [M2], and the
  daemon's builder resolves for its jobs as they need it (hearth/docs/design/domain/steward.md,
  3.3) [M3].
