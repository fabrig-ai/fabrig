# The command and its check

- **One command keeps a crate's patches,** `skein-codegen patch
  <crate>`, ordinary Rust in skein-codegen's `patch` module, which each
  repository may alias (`cargo xtask patch`, when needed). It
  diffs each family's current text against its latest landed kept text
  (`diff.md`), and per family refreshes the kept text, keeps renames
  waiting, or writes the branch's patch: its lines, the values it asks,
  a skeleton for its written sites, the next kept text, `patches/mod.rs`
  and the patch's goldens. It ends by calling the check.
- **The base is the branch's own,** so that a branch not yet rebased
  sees main as it was when it branched, never main's newer patches
  without their schemas. Once rebased, its base is main's tip, so a
  renumbering (6) and a gate after a rebase see what main holds.
- **Landed is what the base holds.** The files under the crate's
  `patches/` and `schema/kept/` as they are at the commit the branch is
  built on, `git merge-base HEAD origin/main` unless `--base` names
  another, are landed, read by the binary and passed to the library; whatever
  else the tree holds there is the branch's. A landed patch, its
  goldens and a kept text a landed patch names never change; the
  branch's patch is derived again on every run, so that one command
  serves a first write, a schema edited again, and a renumbering.
- **One patch per landing.** A landing moves each family's version at
  most once (temper's `draft/upgrades.md`, 4.2), so a branch holds at
  most one patch of its own, numbered after the last landed one, with a
  block per family it moves.
- **The check** (`check`) finds, without writing, what the command would
  change or refuse, each finding naming the command or the fix. The
  command calls it; a gate calls it from the baseline on, none before.

## 1. The crate's files

| Path | What | Whose |
|---|---|---|
| `schema/<family>-v<N>.schema` | the current texts: of each family, the greatest version there | the author's |
| `schema/kept/<family>-v<N>.schema` | every kept version's text from the family's first (11) | the command's |
| `schema/kept/<family>.renames` | the renames waiting for the family's next patch (4), `rename` lines | the command's |
| `patches/NNNN.patch` | the patch's text (`patches.md`, 1) | the command's lines, the author's values, sites and refusals |
| `patches/pNNNN.rs` | its written functions, for a patch with sites (`patches.md`, 5) | the command's skeleton, then the author's |
| `patches/context.rs` | the `Context`, written once, empty, deriving `Default` | the author's |
| `patches/mod.rs` | generated from the kept texts and the patches alone (3) | the command's |
| `patches/golden/NNNN/<family>/` | each old golden through the patch (3) | the command's or the golden test's, reviewed |

- **The owner's crate** includes `patches/mod.rs` as a module of its
  library (`#[path = "../patches/mod.rs"] pub mod patches;`), and
  depends on skein-codec, and on skein-codegen in its tests.
- **A family's baseline is its owner's** (`README.md`, section 5). A crate with no
  kept text, landed or its own, is before its baseline, and a run there
  is refused unless given `--baseline`, which its owner passes once, at
  its baseline: the guard is the crate's files, and the command knows no
  release. That first run keeps each family's current text as its first
  kept version and writes no patch. A family new to a crate past its
  baseline, or whose kept texts are all the branch's, is kept afresh in
  the same way, with no flag.
- **Fixtures stand for families past their baseline:** `tests/codec`'s
  `every` texts and `tests/codec-patches`' families belong to no build
  that stores, sends or embeds them, and move their versions as their
  stories need. `README.md`, section 5's rule binds the families a build stores,
  sends or embeds.

## 2. A run

For each family, in name order, with `K` its latest landed kept text at
version `k` (none for a family new to the base) and `C` its current
text at version `c`:

1. **Matching** by `diff.md`, section 1, the renames confirmed being those
   waiting (4), those given (`--rename`), and the `rename` lines of
   the branch's patch for the family.
2. **Proposals in rounds:** the diff is run again with its proposals
   taken as confirmed until none is new, at most once per declaration,
   so that renames a level of references down are proposed in the same
   run. Proposals are reported for the author to confirm; only
   confirmed renames reach a file.
3. **A patch is needed** when a change is derived or written (`diff.md`, 3), or
   when `K` is frozen and a change other than a rename or a declaration
   dropped is none needed: a frozen text cannot take a declaration, a
   variant or a bound that records of its version may now hold. `K` is
   frozen when a landed patch names its version, from the family's
   baseline on, since no version moves before it. This is the user's rule, and
   temper's `draft/upgrades.md`, 4.3, follows it. The alternative, a
   frozen text refreshed where every landed
   patch naming it still plans and keeps its goldens, would serve a
   variant appended and a bound widened alone, at the cost of landed
   kept texts that never change.
4. **The verdict,** `diff.md`, 4's, from whether a patch is needed:
   - `Kept`: the version kept, no patch needed. A kept text not frozen
     is refreshed (made a copy of `C`); a frozen one keeps renames
     waiting (4);
   - `Moved`: the version moved by one, a patch needed. The family's
     block is written in the branch's patch, and `C` kept as
     `kept/<family>-v<c>.schema`;
   - `Unmoved`, `Needless` and `Wrong`: refused for the family, which
     keeps its files as they are, with the fix: move the version to
     `k + 1`, keep it at `k`, or say which (5).
5. **The branch's patch:** with at least one block, it is written (3)
   under the number after the last landed patch's; with none, its text
   and goldens go, and its `pNNNN.rs`, if any, is reported for the
   author to remove.
6. **Landed files** the tree lacks or holds otherwise, a renumbering's
   among them (6), are written back from the base, and the report
   names each: an author's edit of a landed file is undone by name, as
   the check's `Changed` names it.
7. **`mod.rs`** is generated again; `context.rs` written if absent.
8. **The check** runs, and its findings are reported (7).

## 3. The patch it writes

- **The text:** `patch 1`; `owner <owner> <n>`, the owner given once by
  `--owner` and then read from the latest patch, with `after` and each
  upstream's level as given by `--after`; the branch's patch's marks
  kept; then a block per family, in name order, `family <family> <k> to
  <c>`, holding the diff's lines in their order (`diff.md`, 2) without their
  kind, an added field's value `?` or `none` as the diff asks, and a
  `written D` line after the last line of each declaration the diff
  names as a site (`diff.md`, 3). An added field whose type takes no value
  (`patches.md`, 2: a record, an enumeration whose every variant holds one, a
  secret, a `holds` field) is `written`, with its own `written D.f`
  site unless its declaration is one (`patches.md`, 3), so its author is asked
  for the site's function, not a value;
  the diff's line still reads `?`, as a retype's field site is the
  patch's. No comment is written.
- **Derived again, the author's part kept:** when the branch's patch
  exists, each added field's value is kept where the new diff adds the
  same field with the same type; each site line, the author's and the
  diff's alike, and the `refuses` lines after it, kept where its
  declaration or field is still in the new text; the marks kept. A
  text whose lines come out the same is left as it is, comments and
  all; otherwise it is written whole, without them.
- **The skeleton,** `patches/pNNNN.rs`, written for a patch with sites,
  and for a site the file lacks a function appended: one function per
  site in `patches.md`, 5's signature, named by its target in snake case, each
  capital starting a word as in the generated code's names (`Shape` is
  `shape`, `Every.score` is `every_score`, `HTTP` is `h_t_t_p`), its
  doc comment naming
  the site, its family and versions, and the block's lines for its
  declaration, its body `Err(Halt::NoSite(Site::new(n, i)))`, never a
  panic, until its author writes it. Two sites that spell one name are
  refused.
- **`patches/mod.rs`,** generated from the kept texts and the patches
  alone, its first line saying so, in rustfmt's layout under skein's
  `rustfmt.toml` (edition 2024, max_width 120, use_small_heuristics
  Max), which jig's and temper's match:
  - `mod context; pub use context::Context;` and `mod pNNNN;` per
    skeleton;
  - `PATCHES: &[&str]`, each patch's text by `include_str!` in number
    order, and per family `kept::<family>::TEXTS: &[&str]`, its kept
    texts in version order;
  - `kept::<family>::v<N>::<declaration>::<PART>`, the `FieldIx` and
    `VariantIx` of every part of every kept version, by its index in
    that version's schema; `refusal::pNNNN::<NAME>`, each refusal's
    `RefusalIx`; names in the skeleton's snake case, parts in upper
    case, a keyword raw, two names spelling one constant refused;
  - `kept::<family>::pNNNN::{old, new}`, aliases of each block's from
    and to versions, so written code names no version;
  - `written(site, old, new, at, cx)`, the dispatch of `patches.md`, 5; with no
    site, `Err(Halt::NoSite(site))` alone;
  - its tests: `current`, which fails when `mod.rs` is not as generated,
    naming the command, and a golden test per patch.
- **The golden test** of patch `n` (`patch::goldens(crate, n,
  written)`), for each family it moves: each golden of the old version,
  made from the kept text by `README.md`, section 6's rule, its declaration's
  counterpart its root (a declaration dropped has none, and is passed
  over), run through the patch, each stop answered by the dispatch with
  a `Context::default()` and each held field carried; the result decodes
  as a generic value of the new kept text, and equals its file
  `patches/golden/NNNN/<family>/<golden>`, or, refused with a declared
  reason, holds `refuses <name>` in `<golden>.refused`. A file missing,
  different or stale fails it, naming every golden that does;
  `SKEIN_PATCH_WRITE=1` writes them for review: `goldens`, which
  `mod.rs` calls, picks `patch::golden`'s mode from it, `Mode::Write`
  or `Mode::Check`, which never writes. The command writes them
  itself for a patch with no site once it plans; a patch with sites
  waits for its author's functions and the test.

## 4. Renames

- **Confirmed by its author,** `--rename <family> <old> <new>`, a
  declaration's `Old New` or a part's `Decl.old new` with its
  declaration's current name, as the patch's `rename` line spells it.
- **Where a confirmed rename goes:** into the branch's patch, as its
  `rename` line, when the family moves; else into the kept text by its
  refresh, when not frozen; else into `schema/kept/<family>.renames`, one
  `rename` line each, where it waits for the family's next patch, its
  lines the diff's confirmed renames from then on. That patch takes them
  and the file goes.
- **Unconfirmed,** a rename is a drop and an add: a refresh takes it
  alike, and under a frozen text it asks for a patch (2, 3), whose
  report shows the proposal.

## 5. Versions

- **`Kept`, `Moved`:** acted on (2).
- **`Unmoved`:** a patch needed and the version kept: the author moves
  it, the current schema's header and file, then runs the command.
- **`Needless`:** the version moved with no patch needed, against `diff.md`, 4;
  the author keeps it. Before the baseline no version moves (`README.md`, section 5),
  and no gate runs the check.
- **`Wrong`:** the current version neither `k` nor `k + 1`. A version
  without its kept text, such as a current `v9` over a latest kept `v7`,
  is `Wrong`: kept versions run without a gap from a family's first
  (7).

## 6. Renumbering

- **A taken number shows at the rebase:** two branches that each write
  the owner's next patch add the same `patches/NNNN.patch` and change
  `patches/mod.rs`, so the second's rebase on main stops on their
  conflict; a change to a landed file that rebases cleanly fails the
  check.
- **The author rebases on main,** and resolves: the branch's side of the files the
  command owns (`git checkout --theirs -- <crate>/patches
  <crate>/schema/kept`), the current schemas merged as any conflict, a
  family both branches moved taking the version after the landed one;
  then runs the command. A file holding conflict markers is refused,
  naming this step.
- **The command moves the branch's patch** to the number after the last
  landed one, and writes the taken number's landed files back from the
  base, now main's tip. It derives each block again against the latest
  landed kept text (3), so a family that moved meanwhile has its
  block from the new version, keeping its author's values, sites and
  refusals; its `pNNNN.rs` and `patches/golden/NNNN/` move with it, the
  file's own names rewritten: the module `pNNNN`, `Site::new(n,` and,
  for a family moved meanwhile, its versions in bare `<family>::v<N>`
  paths; code written through the aliases needs only its number.
  A family moved meanwhile has its goldens dropped instead of moved,
  each named in the report: their records are of the version it no
  longer moves from, so the author writes and reviews them again.
  Then the rebase goes on, and the branch lands.

## 7. The check

`check(crate, landed)` reads the crate's files and the landed ones, and
returns every finding, each displayed with its fix; a crate it cannot
read, such as a text that does not parse, is its error, as the run's:

| `Finding` | Found |
|---|---|
| `Unkept { family }` | a current text with no kept text |
| `KeptGap { family, missing }` | a kept version missing between a family's first and its latest |
| `Unpatched { family, version }` | a kept version after a family's first that no patch moves it to, or a current text that needs a patch (2, 3) none moves it to |
| `Lagging { family, version }` | a latest kept text not frozen that differs from the current with no patch needed: the command refreshes it |
| `Version { family, verdict, kept, current }` | `Unmoved`, `Needless` or `Wrong` (5) |
| `Changed { path }` | a landed patch's text or goldens, or a landed kept text, changed or gone; but a kept text no landed patch names, refreshed to its family's current text |
| `PatchGap { missing }` | a patch number missing between 1 and the last |
| `Misnumbered { patch, level }` | a patch whose header's level is not its number |
| `Unplanned { patch, family, error }` | a block that does not plan against its kept texts (`patches.md`, 3): a `?` left, a site missing, a line wrong |

A frozen kept text's differences from the current that need no patch,
renames waiting or not, declarations dropped, comments and order, are
no finding. The golden tests and `current` (3) are the owner's own
tests beside it.

## 8. The interface

```rust
pub mod patch;  // skein_codegen::patch

pub struct Options {
    pub owner: Option<String>,             // --owner, for a crate's first patch
    pub after: Vec<(String, u32)>,         // --after <upstream> <level>
    pub renames: Vec<(String, Rename)>,    // --rename <family> <old> <new>
    pub baseline: bool,                    // --baseline, the crate's first run (12.1)
}
pub struct Landed { /* the base's files under patches/ and schema/kept/, by path */ }
impl Landed {
    pub fn new(files: BTreeMap<PathBuf, Vec<u8>>) -> Landed;  // paths relative to the crate
    pub fn read(owner: &Path) -> Result<Landed, Error>;       // the tree's own, as if landed
    pub fn none() -> Landed;                                  // nothing landed
}
pub fn run(owner: &Path, landed: &Landed, options: &Options) -> Result<Report, Error>;
pub fn check(owner: &Path, landed: &Landed) -> Result<Vec<Finding>, Error>;
pub fn module(owner: &Path) -> Result<String, Error>;  // patches/mod.rs's text
pub fn current(owner: &Path) -> Result<(), Error>;     // mod.rs as generated
pub fn goldens<C: Default>(owner: &Path, patch: u32, written: Written<C>) -> Result<(), Error>;
pub type Written<C> = fn(Site, &Old, &mut Build, Node, &mut C) -> Result<(), Halt>;

pub struct Report {
    pub outcomes: Vec<Outcome>,            // a family's each, in name order
    pub patch: Option<u32>,                // the branch's patch
    pub moved: Option<u32>,                // the number it was moved from
    pub proposed: Vec<(String, Rename)>,   // proposals not confirmed
    pub asked: Vec<String>,                // what its author must give: `?` lines, sites
    pub restored: Vec<PathBuf>,            // landed files written back (12.2, 6)
    pub findings: Vec<Finding>,            // the check's, after the run
}
pub enum Outcome {
    Kept { family: String, version: u16 },        // its first kept text
    Current { family: String },                   // nothing to do
    Refreshed { family: String, version: u16 },
    Waiting { family: String, renames: Vec<Rename> },
    Patched { family: String, from: u16, to: u16 },
    Refused { family: String, verdict: Version },
}
pub enum Finding { /* 12.7's table */ }
pub enum Error {
    Io { path: PathBuf, error: String },
    Schema { path: PathBuf, error: String },  // a text that does not parse
    Diff { family: String, error: DiffError },
    Baseline,                                 // before the baseline, without --baseline
    Owner,                                    // a first patch without --owner
    Unlanded { patches: Vec<u32> },           // two patches of the branch's own
    Conflict { path: PathBuf },               // conflict markers (12.6)
    Spelled { first: String, second: String },// two names, one constant or function
    Golden(Vec<String>),                      // each golden that fails
    Stale,                                    // mod.rs not as generated
}
```

- **The library reads no git:** what is landed is its input, so that
  its tests and the story pass it in memory.
- **The binary:** `skein-codegen patch <crate> [--baseline]
  [--owner <name>] [--after <upstream> <level>]… [--rename <family>
  <old> <new>]… [--base <ref>] [--check]`. It reads the landed files with `git
  ls-tree` and `git show <base>:<path>`, the base `git merge-base HEAD
  origin/main` unless `--base` names another, prints the report, its proposals as the
  `--rename` that confirms each, and exits 1 while a finding is left.
  `--check` runs the check alone. `skein-codegen <schema> <out-dir>`
  is as before; a schema is never named `patch`.
- `Report`, `Outcome`, `Finding` and `Error` derive `Clone`, `Debug` and
  equality, and display as the command prints them.

## 9. Tests

Step tests in skein-codegen, the story in `tests/codec-patches`; no
world, no cargo inside a test.

- **Step tests,** each over a crate in a scratch directory, its files
  compared as whole text: each outcome's files (a run before the
  baseline refused, the baseline, a refresh, a rename waiting, a
  derived patch with its `?`, a written one with its skeleton), the
  author's values and sites kept when derived again, each verdict of
  `diff.md`, 4, the gap, renumbering with and without the family moved
  meanwhile, `mod.rs` for a crate of two families and patches, and each
  finding of 7 and its absence.
- **The story,** `tests/codec-patches`, an owner crate whose history the
  command makes: the baseline, by `--baseline`; a field added, its patch
  asking `?`, refused when planned, then filled, planned and its goldens
  written; a rename refreshed in a family no patch names, and one
  waiting in a frozen one; a written change, its skeleton's golden test
  failing with `NoSite` naming the site, then its function written;
  another branch's patch landing first, and the story's renumbered. The
  check at each step. Each step is checked as text against its expected
  files; the crate's final files are the last step's, built and tested
  by the gate as any member: `current` and the golden tests.
