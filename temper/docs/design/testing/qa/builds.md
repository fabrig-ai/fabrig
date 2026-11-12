# Kept builds

Provisional, 2026-11-12, from `../../draft/landed-qa.md`, section 3,
milestones Q1 and Q3 of `../../draft/landed-qa.execution.md`. The builds
a pass judges: made by the deployment at landed tips, published to
landed QA with a manifest, held by the passes that judge them, and
deleted once no pass can need them.

## 1. In one page

- **The deployment builds** (section 2), as part of its after-landing
  work: at a landed tip, the box, the fake coding agent beside it, the
  TUI's recording client, the drive tool, and the web bundle with its
  recording build, staged in a directory. temper needs nothing for it
  but the command that keeps it.
- **`temper-qa build keep COMMIT DIR`** (section 3) publishes it: every
  temper executable must print that commit as its version, the files
  are moved into the builds directory under the commit, a manifest names
  the commit and each file's checksum (section 4), and the ledger
  records the build.
- **Held by a pass** while it runs or waits on it, and installed at its
  place from the manifest (`places.md`, 3).
- **Deleted by the loop** (section 5), by the ledger's rule alone: at
  once when triage finds nothing to check, once a newer triaged build
  is kept, and once the pass holding it ends or resumes on another.

## 2. The deployment's build

- **When:** at a tip of main its after-landing work reaches, a tier of
  its own or a part of one (`../../domain/after-landing.md`, 2). Not
  every tip is kept, since one run covers several landings and a tip a
  receipt judges has no run; a case waits for the next kept build that
  holds its landing (`ledger.md`, 4.7).
- **What,** staged in a fresh directory STAGE, by its absolute path, on
  the builds directory's filesystem, from a clean checkout of the tip:
  - `bin/temper-box`: the box, its client and its embedded bundle;
  - `bin/fake-claude`: the fake coding agent the box finds beside itself
    (`../../protocol/setup/box.md`, 5);
  - `bin/temper-client`: the TUI's recording build, whose saves the
    drive session's TUI lines read (`../../client/drive.md`, 2.1;
    `../../client/tui/journeys.md`, 2);
  - `bin/temper-client-journeys`: the drive session and the recipe walk
    (`../../client/drive.md`, 2; `../../client/recipe.md`, 5), so that a
    pass drives each build with that build's own grammar;
  - those four from one debug build, `cargo build -p temper-box -p
    smith-fake-claude -p temper-client-journeys -p temper-client
    --features temper-client-shell-tui/record`, as the journeys' `again`
    builds its set: the client's `record` feature gives the box no
    recording, which only `temper-box-shell`'s own feature does
    (`../../client/tui/journeys.md`, 2); any other executable a later
    milestone names goes directly in `bin/` too;
  - `bundle/`: the web bundle, `temper/scripts/bundle.sh --loopback
    STAGE/bundle`, which the drive session gives the box
    (`../../client/drive.md`, 2.1);
  - `bundle-record/`: its recording build, `temper/scripts/bundle.sh
    --loopback --record STAGE/bundle-record`, as the journeys' walk
    takes it (`../../client/web/journeys.md`, 8.1).

  A debug build's set is about 400 MB.
- **Then** `temper-qa build keep COMMIT DIR`, with the home's
  `TEMPER_QA_HOME`. A refusal fails the deployment's step, and its words
  say why.

## 3. Keeping one

`temper-qa build keep COMMIT DIR`, in this order, refusing at the first
check that fails, with nothing moved or recorded:

1. **COMMIT** is 40 or 64 lowercase hexadecimal digits: `COMMIT is no
   commit id`.
2. **Not kept already:** a build of COMMIT in state `kept` is refused,
   `build COMMIT12 is kept already`; a deleted one may be kept again,
   its record kept anew.
3. **DIR holds the build:** `bin/temper-box`, `bin/fake-claude`,
   `bin/temper-client`, `bin/temper-client-journeys`,
   `bundle/index.html` and `bundle-record/index.html`, each a regular
   file: `DIR lacks PATH`, each missing path named. Every entry under
   DIR is a directory or a regular file: `DIR holds PATH, neither a file
   nor a directory`.
4. **Every temper executable prints COMMIT:** each regular file directly
   in `bin/` must be executable: `bin/NAME is not executable`. Each but
   `bin/fake-claude` is temper's, and its `--version`, run with the
   captured environment, stdin closed and 30 seconds to answer, must
   print a first line ending in ` (C12)`, C12 COMMIT's first twelve
   hexadecimal digits, as every binary of temper prints it
   (`../tools.md`, 6): `bin/NAME prints LINE, not COMMIT12`. A `-dirty`
   build fails here. `bin/fake-claude` is smith's fake of the coding
   agent's command line, which answers `--version` as that command
   does; built in the same build, it is held by the manifest alone.
5. **Same filesystem:** DIR is moved by a rename, so it must be on the
   builds directory's filesystem: `DIR is not on the filesystem of
   BUILDS`.
6. **Published:** DIR renamed to `BUILDS/.COMMIT.partial`, its manifest
   written there (section 4), and that renamed to `BUILDS/COMMIT`; then
   the ledger records the build `kept`, its `holds` absent until the
   watcher reads a head that holds it (`landings.md`, 5). A
   `.COMMIT.partial` left by a failed keep is removed by the next keep of
   that commit.

It prints `build COMMIT12 kept: N files`.

## 4. The manifest

`BUILDS/COMMIT/manifest`, text: its first line `commit COMMIT`, then one
line per file of the build but the manifest, sorted by path,
`SHA256  PATH` (the lowercase hexadecimal SHA-256 of its bytes, two
spaces, its path from the build's directory), as `sha256sum` prints and
checks it. So `grep -v '^commit ' manifest | sha256sum -c` checks a kept
build from its directory, and a place's install checks each file it
copies against it (`places.md`, 3).

## 5. Deleting

- **The rule is the ledger's** (`ledger.md`, 4.6), and the loop's sixth
  step applies it (`README.md`, 4): for each build it lets go,
  `BUILDS/COMMIT` is removed, then the build recorded `deleted` with the
  time.
- **A pass's hold** keeps a build: from the pass's start or resume on it
  until the pass ends or resumes on another (`ledger.md`, 4.8, 4.12 and
  4.13). Released, it falls under the rule as any build does, so a pass's
  end or resume deletes it at the next tick unless queued cases still
  need it.
- **A directory already gone** counts as removed. One that will not go
  stays `kept` and is tried again at the next tick; failing for an hour,
  it is a line (`README.md`, 4).
- **Nothing else deletes a build:** not a restart, not `build keep`.

## 6. Later

- **A release build** where debug builds' time or size tells against
  them; none is asked yet.
