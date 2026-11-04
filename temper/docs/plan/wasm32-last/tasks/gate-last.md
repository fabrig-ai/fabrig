+++
id = "gate-last"
part = "gate-last"
kind = "feature"
needs = []
files = [
  ".config/gate/wasm32.txt",
  "skein/docs/design/testing.status.md",
  "skein/docs/design/STATUS.md",
  "smith/docs/design/shell.status.md",
  "smith/docs/design/STATUS.md",
  "temper/docs/design/playground.status.md",
  "temper/docs/design/STATUS.md",
]
effort = "high"
minutes = 50
+++

# The last wasm32 re-measure: the green join the gate's list, the red keep a refreshed status

## Goal

The gate's step `clippy-wasm32` reads `.config/gate/wasm32.txt` and runs `cargo clippy --target
wasm32-unknown-unknown -p <each listed package the selection holds> -- -D warnings`. The list
holds 44 packages. Measure the nine the status files still name as left out, alone, at your
base; append every green one; run the whole list once; close the status entries the green ones
settle; and leave each red one's entry with its first error's file, line and lint, refreshed.
No Rust changes: a crate needing code is its tree's.

Acceptance:

- Each package of the table below was clippied alone, one `heavy` command each, in the table's
  order, and the report gives each one's outcome.
- `.config/gate/wasm32.txt` holds its base's lines unchanged and in order, its comment line
  first, then every green package of the table appended after the last line in the table's
  order, one per line, the file ending with one newline. No red package is in it.
- `heavy cargo clippy --target wasm32-unknown-unknown -p <each package line of the list, in
  order> -- -D warnings` passes, run once over the final list (see Pre-approved for a red), and
  the report gives its wall time and host.
- The status files below read as `## Status edits` says, their date lines today's (`date -u
  +%F`), every other word unchanged.
- `python3 -I -B tools/citations/citations.py --base origin/main` prints no finding this branch
  adds.

## Inputs

- `temper/docs/plan/wasm32-last/inputs/wasm32-last.md`: the request.
- `.config/gate/README.md`, `The gate's lists`: who adds a line to the list.
- `DESIGN.md`, 2: a status file's form. It holds only the gaps as they stand after your change,
  never history.
- Read only, for what each says the gate checks: `skein/docs/design/testing.md`, 10;
  `smith/docs/design/shell.md`, 2.5; `temper/docs/design/playground.md`, 9. This cut wrote no
  design: those sections already ask for these checks, and only the status follows the code.

**Departures from the implementer's text,** each because the request asks it:

- You edit the status files and `STATUS.md` summaries listed in `## Owns and may change`
  (`docs/design/`), in the way `## Status edits` says, and no design document (`<name>.md`),
  draft or `docs/help/`.
- No `cargo fmt`, `old_forms.py`, `--all-targets` clippy or nextest run: no Rust and nothing they
  read changes. The builds are the table's commands, the one `--keep-going` run of step 3 and the
  combined run, each once through `heavy` (once more only after exit 137).
- `smith/docs/design/**` is in `.config/gate/read-by-tests.txt`; the one test that compiles a
  document of that tree, `smith/crates/smith-local-domain/src/parent_tests.rs`, includes
  `shell.md` alone, and the benchmarks' tasks cite no status file, so no test runs for the smith
  status edits.
- **Status files are not plan files.** `tests/boundary/tests/outward.rs` reads them: what you
  write there says "the gate", never the name of the tool that runs it, and never a `heavy`
  command. The quoted forms below keep to that.

## The crates

These are every package the status files' Not yet lines that cite wasm32 name:
`skein/docs/design/testing.status.md`, `6, 10`; `smith/docs/design/shell.status.md`, `2.5`;
`temper/docs/design/playground.status.md`, `9`. Each name was checked against `cargo metadata
--no-deps --format-version 1` at `6730de2602`; a name that is no package at your base is reported
under `## Left` and skipped, never replaced by a guess.

| Order | Package | Its entry |
|---|---|---|
| 1 | `skein-tls` | `skein/docs/design/testing.status.md`, `6, 10` |
| 2 | `skein-fake-forge-github` | same |
| 3 | `skein-fake-peers` | same |
| 4 | `smith-protocol-events` | `smith/docs/design/shell.status.md`, `2.5` |
| 5 | `smith-protocol-llm` | same |
| 6 | `smith-protocol-program` | same |
| 7 | `smith-agent-service` | same |
| 8 | `smith-agent-shell` | same |
| 9 | `temper-playground` | `temper/docs/design/playground.status.md`, `9` |

Each measurement is `heavy cargo clippy --target wasm32-unknown-unknown -p <package> -- -D
warnings`. Clippy lints the named package and the workspace crates it depends on, so a red one's
first error may sit in a dependency's file; record it where it sits.

What `cargo tree --target wasm32-unknown-unknown -e normal` gives at `6730de2602` (it builds
nothing): no package of the list reaches any of the nine; packages 1 to 7 reach neither
`smith-agent-shell` nor `smith-local-service`; `temper-playground` reaches all eight others and
`smith-local-service` and `temper-engine-domain-forge`. So `temper-playground` is green only if
`smith-agent-shell` is. No build ran for this cut, and every place the entries name still stands
in the source at `6730de2602`, so expect packages 8 and 9 red, and record what the runs say.

**A red one's first error** is the first diagnostic in heavy's log that starts with `error` (not
`error: could not compile`, which is cargo's summary): its lint as `clippy::<name>` or rustc's
lint name (from the `-D ... implied by -D warnings` note or the lint's help link), or its code
(`E0433`) for a compile error; and its `-->` location, as a path from the repository root and a
line. A file of a crates.io dependency is named `<crate> <version>` and its path inside that
crate, never an absolute path. An error with no location (a build script, a link) is named by its
message's first line, cut to about 80 characters.

**Why one `--keep-going` run:** the `temper-playground` entry names places in three crates, and a
plain run stops scheduling crates at the first that fails, so a place it does not report may
still fail. The extra run, for that one package alone, says which still do; what depends on a
failed crate stays unlinted in it too, which the entry then says.

## Status edits

The entries below are the text at `6730de2602`. "Green" and "red" mean as measured by this task.

`<red clause>` is `` `<package>` fails that clippy, warnings denied, first with `<lint>` at
`<file>:<line>` ``. Files are from the repository root. An entry keeps its file's indent and wraps
at about 72 columns, but for a path, which stays whole, as the entries do now. One red package
reads in the singular (`reaches it`, `does not lint it`).

### skein

- `skein/docs/design/testing.status.md`, the entry between `...the programs for trees.` and `7:`:

  ```text
  6, 10: the gate's wasm32 list (`.config/gate/wasm32.txt`) leaves out
  `skein-tls`, `skein-fake-forge-github` and `skein-fake-peers`, and no
  listed package reaches them on wasm32, so the gate's wasm32 clippy
  lints none of them; run outside the gate, their wasm32 clippy passes,
  warnings denied.
  ```

  All three green: remove those words, so that `...the programs for trees.` is followed by `7:`.
  Else it names the red ones alone, and ends `...lints none of them; run outside the gate,
  <red clause>; <red clause>.`, one clause per red package.
- `skein/docs/design/STATUS.md`, the summary paragraph, the sentence ``Of the portable crates,
  `skein-tls`, `skein-fake-forge-github` and `skein-fake-peers` are not in the gate's wasm32
  clippy, though they pass it (`testing.md`).``. All three green: remove it. Else: ``Of the
  portable crates, <the red packages, backticked> fail the gate's wasm32 clippy (`testing.md`).``
  Rewrap the paragraph at about 72 columns.

### smith

- `smith/docs/design/shell.status.md`, in **Not yet**, the item `- 2.5: the gate's wasm32 list
  ...` (ending `...those of `EndpointTransport::Tls` (`endpoints.rs:39`).`). All five green: remove
  the item; the `4, 5.2` and `5.2` items stay. Else it names the red ones alone. With
  `smith-agent-shell` the only red one it reads:

  ```text
    - 2.5: the gate's wasm32 list (`.config/gate/wasm32.txt`) leaves out
      `smith-agent-shell`, and no listed package reaches it on wasm32,
      so the gate's wasm32 clippy does not lint it. Outside the gate,
      <red clause>: in `src/config/`, what only its `cfg(unix)` code
      reads is unused on wasm32: the import `skein_tls as tls`
      (`native.rs:10`), the parameter `errors` (`native.rs:31`), the
      constant `CERTIFICATES` (`native.rs:74`), `TrustDocument`'s fields
      (`document.rs:185`) and those of `EndpointTransport::Tls`
      (`endpoints.rs:39`).
  ```

  The places after `<red clause>` are the ones the run reports for that crate, each with the line
  it gives: a place it no longer reports leaves, one it newly reports joins in the same form. Any
  other red package of 4 to 7 gets `<red clause>` of its own, before `smith-agent-shell`'s.
- `smith/docs/design/STATUS.md`, the summary paragraph, the words `` the `--account` flag
  (`shell.md`, `testing.md`) and `smith-agent-shell` in the gate's wasm32 clippy, which it fails
  (`shell.md`), and the benchmarks' ``. `smith-agent-shell` the only red one: unchanged. All five
  green: `` the `--account` flag (`shell.md`, `testing.md`), and the benchmarks' ``. Else: the red
  packages in `smith-agent-shell`'s place. Rewrap at about 72 columns if the words change.

### temper

- `temper/docs/design/playground.status.md`, in **Not yet**, the `9:` item, the words from `The
  wasm32 builds in the gate:` to `(smith's `shell.md`, 2.5, and its status).`.
  - Green: remove those words, so the item ends `...lifts that is unknown.`.
  - Red: the first sentence and the words `` `temper-playground`'s wasm32 clippy fails, with
    warnings denied, `` stay. The rest names what the `--keep-going` run reports, in the same
    forms (a package's unused items by the pointer to its status file; an unmet expectation by its
    type, its lint and its `file:line`), each line as that run gives it: a place it no longer
    reports leaves, one it newly reports joins, `in smith's and temper's crates alone` names the
    trees it reports. The plain run's first error comes first, with its lint named. Then one
    sentence: ``What depends on those crates goes unlinted, `temper-playground` among them.``
- `temper/docs/design/STATUS.md`, the summary paragraph, the words ``whose story A walks in the
  browser tier but not yet by hand, whose download is past its bar and which the gate's wasm32
  clippy leaves out (`playground.md`)``. Red: unchanged. Green: ``whose story A walks in the
  browser tier but not yet by hand and whose download is past its bar (`playground.md`)``. Rewrap
  at about 72 columns if the words change.

`temper/docs/design/playground.status.md`'s **Departs** entry `6, 9` stays as it is, whichever way
`temper-playground` goes: it is about the shell's feature, not the list.

Every status file and `STATUS.md` you edit takes today's date line. A file you do not edit keeps
its date; a red-only outcome may leave a `STATUS.md` untouched.

## Owns and may change

- `.config/gate/wasm32.txt`: green packages appended, as above. Its comment line and its base's
  lines stay.
- The six status files and summaries of `## Status edits`, as it says, and no other.

Pre-approved:

- **The text differs from the quotes above** (another plan landed first, or a `STATUS.md` summary
  was rewritten): apply the same rule to the text as found (an entry or sentence that says which
  crates the gate's wasm32 clippy leaves out names, after your change, only the red ones, or goes
  when none is red), and say so in the report.
- **A package your base's list already holds:** skip it, keep its line where it is, and say so.
- **The combined command fails:**
  - at a file of a crate that only packages this task added bring in: take out of the list each
    added package whose wasm32 tree holds that crate (`cargo tree --target
    wasm32-unknown-unknown -e normal -p <package>`, which builds nothing), record each as red with
    that error, and run the combined command once more. If that fails too, stop: commit and push
    what you have, and report `Question:` with both runs' first errors and the options (take out
    more packages, or keep the list at its base).
  - at a file of a crate your base's list already brings in: run the same command once over your
    base's package lines alone. Red there too: the trunk's red, not this task's: keep your list,
    finish, and report `Done:` with that error under `## Left`. Green there: an added package
    changes that crate's build by its features, and the list must not land red: stop, commit and
    push what you have, and report `Question:` with both runs' first errors and the options.
- **heavy prints no wall time** for the combined command: take `date -u +%s` just before and
  after it, and give the difference.
- **The wasm32 target is missing** on the builder (`can't find crate for `core``, "the target may
  not be installed"): stop and report `Failed: wasm32 target not installed`, with the host from
  heavy's output. Never `rustup target add`: a builder's toolchain is the user's.
- **heavy exits 75:** stop and report `Infra: disk`.

## Steps

1. Check the table's names against `cargo metadata --no-deps --format-version 1`, and read your
   base's `.config/gate/wasm32.txt`.
2. Measure each package in the table's order, one `heavy` command at a time. Note for each: green,
   or red with its first error and heavy's log path; and the host.
3. If `temper-playground` is red, once: `heavy cargo clippy --target wasm32-unknown-unknown -p
   temper-playground --keep-going -- -D warnings`. Note each place it reports, and which crates
   it failed in.
4. Append the green packages to `.config/gate/wasm32.txt` in the table's order; commit (the
   status files' green wasm32 crates join the gate's list).
5. Run the combined command once; note its outcome, wall time and host. Follow Pre-approved on a
   failure.
6. Make the status edits; commit (the statuses close what the green crates settle, and keep each
   red one's first error, refreshed).
7. `python3 -I -B tools/citations/citations.py --base origin/main`; `git log --stat` of your
   commits shows only the files of `## Owns and may change`.
8. Push your branch.

## Report

As the implementer's text says, and also:

- a table of the nine packages: green, red (lint or code, file, line, heavy's log path), skipped or
  not a package; the host of each run;
- for a red `temper-playground`, what the `--keep-going` run reported, by crate, and which crates
  it left unlinted;
- the list as it stands, and how many lines this task added;
- the combined command's outcome, its wall time and host, and whether the target was warm from the
  per-package runs (the gate pays that time on each batch the list's packages touch);
- the citations check's output, and any pre-approved option taken.
