+++
id = "gate-more"
part = "gate-more"
kind = "feature"
needs = []
files = [
  ".config/gate/wasm32.txt",
  "skein/docs/design/testing.status.md",
  "skein/docs/design/world.status.md",
  "skein/docs/design/STATUS.md",
  "jig/docs/design/protocol/browser.status.md",
  "jig/docs/design/protocol/README.status.md",
  "jig/docs/design/STATUS.md",
  "temper/docs/design/client/browser.status.md",
  "temper/docs/design/STATUS.md",
  "smith/docs/design/shell.status.md",
  "smith/docs/design/STATUS.md",
  "temper/docs/plan/wasm32-list/plan.toml",
]
effort = "high"
minutes = 90
+++

# The design's other wasm32 crates measured; the green join fab's gate list

## Goal

fab's gate step `clippy-wasm32` reads `.config/gate/wasm32.txt` and runs `cargo clippy --target
wasm32-unknown-unknown -p <each listed package the selection holds> -- -D warnings`. The list
holds six packages; the designs name more crates for that check. Clippy each named crate alone
for wasm32 at your base, append every green one to the list, run the whole list once, record
each red one in its tree's status as **Not yet** with its first error, close the status entries
the green ones settle, and answer plan wasm32-list's question `wasm32-more`. No Rust changes:
a red crate's code stays as it is, for its tree to repair.

Acceptance:

- Each package of the table below was clippied alone, one `heavy` command each, in the table's
  order (but for the one skip the table states), and the report gives each one's outcome.
- `.config/gate/wasm32.txt` holds its base's lines unchanged and in order, its comment line
  first, then every green package of the table, appended after the last line in the table's
  order, one per line, the file ending with one newline. No red package is in it.
- `heavy cargo clippy --target wasm32-unknown-unknown -p <each package line of the list, in
  order> -- -D warnings` passes, run once over the final list (see Pre-approved for a red), and
  the report gives its wall time and host.
- The status files below read as `## Status edits` says, their date lines today's (`date -u
  +%F`), every other word unchanged.
- `temper/docs/plan/wasm32-list/plan.toml` carries the answer (`## Steps`, 5).
- `python3 -I -B tools/citations/citations.py --base origin/main` prints no finding this branch
  adds.

## Inputs

- `temper/docs/plan/wasm32-more/inputs/request.md`: the request.
- `DESIGN.md`, 2: a status file's form. It holds only the gaps as they stand after your
  change, never history; a status with no gap left says `No gap.` under its date line (as
  `skein/docs/design/ui-web.status.md` does).
- Read only, for the crates each names: `skein/docs/design/testing.md`, 6 and 10;
  `skein/docs/design/world.md`, 8; `jig/docs/design/protocol/browser.md`, 15 and 18;
  `jig/docs/design/protocol/README.md`, 11; `temper/docs/design/client/browser.md`, 4.3;
  `smith/docs/design/shell.md`, 2.5. Their crates are resolved below; you need not re-derive
  them.

**Departures from the implementer's text,** each because the request asks it:

- You edit the status files and `STATUS.md` summaries listed in `## Owns and may change`
  (`docs/design/`), in the way `## Status edits` says, and no design document (`<name>.md`),
  draft or `docs/help/`.
- You edit another plan's `plan.toml` by one line (`## Steps`, 5).
- No `cargo fmt`, `old_forms.py`, `--all-targets` clippy or nextest run: no Rust and nothing
  they read changes. The builds are the table's commands and the combined one, each once
  through `heavy` (once more only after exit 137).
- `smith/docs/design/**` is in `.config/gate/read-by-tests.txt`; the one test that reads that
  tree, `smith/crates/smith-local-domain/src/parent_tests.rs`, includes `shell.md` alone,
  which you do not touch, so no test runs for the smith status edits.

## The crates

Resolved from `cargo metadata --no-deps --format-version 1` and the design sections above at
`0db5c4f348`. Before the first build, check each name against `cargo metadata` (it builds
nothing): a name that is no package is reported under `## Left` and skipped, never replaced
by a guess.

Already in the list at `0db5c4f348`, not measured, in the combined run: `skein-ui`,
`skein-ui-web`, `skein-ui-example-view`, `skein-ui-example-shell-web`, `jig-client-shell-web`,
`skein-playground`.

| Order | Package | A red one's status file, label |
|---|---|---|
| 1 | `skein-lib` | `skein/docs/design/testing.status.md`, `6, 10` |
| 2 | `skein-codec` | same |
| 3 | `skein-io` | same |
| 4 | `skein-sim` | same, and `skein/docs/design/world.status.md`, `8` |
| 5 | `skein-shell` | same, and `skein/docs/design/world.status.md`, `8` |
| 6 | `skein-http` | `skein/docs/design/testing.status.md`, `6, 10` |
| 7 | `skein-json` | same |
| 8 | `skein-channel` | same |
| 9 | `skein-kv` | same |
| 10 | `skein-tls` | same |
| 11 | `skein-fake-machine` | same, and `skein/docs/design/world.status.md`, `8` |
| 12 | `skein-sim-world` | same, and `skein/docs/design/world.status.md`, `8` |
| 13 | `skein-fake-llm-domain` | `skein/docs/design/testing.status.md`, `6, 10` |
| 14 | `skein-fake-llm-protocol` | same |
| 15 | `skein-fake-forge-domain` | same |
| 16 | `skein-fake-forge-protocol` | same |
| 17 | `skein-fake-forge-github` | same |
| 18 | `skein-fake-peers` | same |
| 19 | `jig-protocol-transcript` | `jig/docs/design/protocol/README.status.md`, `11` |
| 20 | `jig-client-protocol-wire` | `jig/docs/design/protocol/browser.status.md`, `15` |
| 21 | `jig-client-domain` | same |
| 22 | `jig-client-protocol-link` | same |
| 23 | `jig-client-protocol-http` | same |
| 24 | `jig-client-view` | same |
| 25 | `jig-client` | same |
| 26 | `jig-client-test-shell-web` | same |
| 27 | `temper-client-protocol-wire` | `temper/docs/design/client/browser.status.md`, `4.3` |
| 28 | `temper-client-domain` | same |
| 29 | `temper-client-view` | same |
| 30 | `temper-client-shell-web` | same; **skipped** when your base's list already holds it (plan playground-browser adds it with its fix of `turn`): then it is in the combined run, and neither measured nor named |
| 31 | `smith-events` | `smith/docs/design/shell.status.md`, `2.5` |
| 32 | `smith-transcript` | same |
| 33 | `smith-channel` | same |
| 34 | `smith-charter` | same |
| 35 | `smith-domain` | same |
| 36 | `smith-domain-tools` | same |
| 37 | `smith-domain-session` | same |
| 38 | `smith-domain-run` | same |
| 39 | `smith-protocol-events` | same |
| 40 | `smith-protocol-channel` | same |
| 41 | `smith-protocol-translate` | same |
| 42 | `smith-protocol-llm` | same |
| 43 | `smith-protocol-machine` | same |
| 44 | `smith-protocol-program` | same |
| 45 | `smith-agent-service` | same |
| 46 | `smith-agent-shell` | same |

Left out, by the design: `jig-client-shell-native`, the native shell over skein's HTTP client
and io (`jig/docs/design/protocol/README.md`, 5), which builds for the host alone; skein's
harness, browser kit, conformance suite and scratch directory, which testing.md 10 keeps
native.

Each measurement is `heavy cargo clippy --target wasm32-unknown-unknown -p <package> -- -D
warnings`. Clippy lints the named package and the workspace crates it depends on, so a red
one's first error may sit in a dependency's file; record it where it sits.

**A red one's first error** is the first diagnostic in heavy's log that starts with `error`
(not `error: could not compile`, which is cargo's summary): its lint as `clippy::<name>` or
rustc's lint name (from the `-D ... implied by -D warnings` note or the lint's help link), or
its code (`E0433`) for a compile error; and its `-->` location, as a path from the repository
root and a line. A file of a crates.io dependency is named `<crate> <version>` and its path
inside that crate, never an absolute path. An error with no location (a build script, a link)
is named by its message's first line, cut to about 80 characters.

## Status edits

The entries below are main's at `0db5c4f348`. "Red" means measured red by this task.

**A red line** is one line per red package, inside its entry:
`` `<package>`: `<lint or code>` at `<file>:<line>` ``, separated by `;`, the last ending with
`.`. An entry that keeps red packages reads, under its label (indented as the entry's file
indents its continuation lines, wrapped at about 72 columns but for the red lines, which stay
one line each):

```text
<label>: these crates fail the gate's wasm32 clippy, so fab's gate list
(`.config/gate/wasm32.txt`) leaves them out; each at its first error:
`<package>`: `<lint>` at `<file>:<line>`;
`<package>`: `<lint>` at `<file>:<line>`.
```

### skein

- `skein/docs/design/testing.status.md`: the entry `6, 10: the gate's wasm32 clippy lints the
  packages of fab's gate list ... the channel and TLS.` (between `...the programs for trees.`
  and `7:`). No skein package red: remove those words, so that `...the programs for trees.` is
  followed by `7:`. Else: the red form with label `6, 10`, naming every red skein package.
- `skein/docs/design/world.status.md`: `- **Not yet:** 8: the fake machine is not in the gate's
  wasm32 clippy (...).` None of `skein-sim`, `skein-shell`, `skein-fake-machine` and
  `skein-sim-world` red: the body becomes `No gap.`. Else: `- **Not yet:** ` then the red form
  with label `8`, naming those of the four that are red.
- `skein/docs/design/STATUS.md`, the summary paragraph: the sentence `Of the portable crates,
  the fake machine, the fakes a world hosts, the key-value store, JSON, the channel and TLS are
  not in the gate's wasm32 clippy (`testing.md`, `world.md`).` No skein package red: remove it,
  and add `` `world.md`, `` after `` `kv.md`, `` in the list of documents with no gap. Else:
  `Of the portable crates, <the red packages, backticked> fail the gate's wasm32 clippy
  (`testing.md`)` with `, `world.md`` inside the parentheses when `world.status.md` keeps its
  entry. Rewrap the paragraph at about 72 columns.

### jig

- `jig/docs/design/protocol/browser.status.md`: `- **Not yet:** 15: `jig-client-test-shell-web`
  is not in the gate's wasm32 clippy (...).` None of packages 20 to 26 red: the body becomes
  `No gap.`. Else: `- **Not yet:** ` then the red form with label `15`.
- `jig/docs/design/protocol/README.status.md`: unchanged unless `jig-protocol-transcript` is
  red; then append to its one-line **Not yet**, after `...contained process trees for workers'
  agents.`, ` 11: ` and the red form's words, on the same line as that entry is written.
- `jig/docs/design/STATUS.md`, the summary paragraph: the sentence `Of jig's wasm32 builds,
  `jig-client-test-shell-web` is not in the gate's clippy (`protocol/browser.md`).` No jig
  package red: remove it, and `` `domain/core.md`, `domain/providers.md` and
  `client/domain/README.md` have no gap of their own. `` becomes `` `domain/core.md`,
  `domain/providers.md`, `client/domain/README.md` and `protocol/browser.md` have no gap of
  their own. ``. Else: `Of jig's wasm32 builds, <the red packages> fail the gate's clippy
  (`protocol/browser.md`)`, with `, `protocol/README.md`` when `jig-protocol-transcript` is red.

### temper

- `temper/docs/design/client/browser.status.md`, in **Not yet**, the `4.3:` entry (main's names
  four crates and `turn`'s `clippy::too_many_lines`; plan playground-browser's names three).
  None of the measured temper packages red: remove the 4.3 entry's words, so the 2.1 entry is
  followed by `4.6:`. Else: the red form with label `4.3`, naming the red ones.
- `temper/docs/design/STATUS.md`, the summary paragraph: the line `the client's crates in the
  gate's wasm32 clippy (`client/browser.md`),`. None red: remove it, so `` `client/tools.md`),
  `` is followed by `the playground in the browser,`. Else: unchanged.

### smith

- `smith/docs/design/shell.status.md`, in **Not yet**, the item `- 2.5: `smith-agent-shell` and
  what it depends on in smith are not in the gate's wasm32 clippy (...), but for
  `smith-transcript`, which `jig-client-shell-web` depends on.` None of packages 31 to 46 red:
  remove the item; the `4, 5.2` and `5.2` items stay. Else: the item becomes `- 2.5: ` and the
  red form, its continuation lines indented four spaces as the file's items are.
- `smith/docs/design/STATUS.md`, the summary paragraph: `` the `--account` flag (`shell.md`,
  `testing.md`) and `smith-agent-shell` in the gate's wasm32 clippy (`shell.md`), and the
  benchmarks' `` becomes `` the `--account` flag (`shell.md`, `testing.md`), and the
  benchmarks' `` when none is red; else unchanged. Rewrap at about 72 columns.

Every status file and `STATUS.md` you edit takes today's date line. A file you do not edit
keeps its date.

## Owns and may change

- `.config/gate/wasm32.txt`: green packages appended, as above. Its comment line and its base's
  lines stay.
- The eleven status files and summaries of `## Status edits`, as it says, and no other.
- `temper/docs/plan/wasm32-list/plan.toml`: the one `answer` line (`## Steps`, 5), nothing
  else.

Pre-approved:

- **The text differs from the quotes above** (another plan landed first: plan
  playground-browser rewrites temper's 4.3, plan public-reading-build edits the `STATUS.md`
  summaries): apply the same rule to the text as found (an entry or sentence that says which
  crates are not in the gate's wasm32 clippy names, after your change, only the red ones in the
  red form, or goes when none is red), and say so in the report.
- **A package your base's list already holds** (other than `temper-client-shell-web`, which
  the table covers): skip it, keep its line where it is, and say so.
- **A package red alone that the list's own packages already lint through a dependency** (one
  of 1 to 6, 12, 19 to 25 or 32, whose build alone differs by features): record it as red like any
  other, with ` (alone)` after its name in its red line, and say so in the report.
- **The combined command fails:**
  - at a file of a crate that only packages this task added bring in: take out of the list each
    added package whose wasm32 tree holds that crate (`cargo tree --target
    wasm32-unknown-unknown -e normal -p <package>`, which builds nothing), record each as red
    with that error, and run the combined command once more. If that fails too, stop: commit
    and push what you have, and report `Question:` with both runs' first errors and the
    options (take out more packages, or keep the list at its base).
  - at a file of a crate the base's list already brings in: the trunk's red, not this task's.
    Keep the list, finish, and report `Done:` with that error under `## Left`.
- **heavy prints no wall time** for the combined command: take `date -u +%s` just before and
  after it, and give the difference.
- **The wasm32 target is missing** on the builder (`can't find crate for `core``, "the target
  may not be installed"): stop and report `Failed: wasm32 target not installed`, with the host
  from heavy's output. Never `rustup target add`: a builder's toolchain is the user's.
- **heavy exits 75:** stop and report `Infra: disk`, as fabrig's `AGENTS.md`, 5 says.

## Steps

1. Check the table's names against `cargo metadata --no-deps --format-version 1`; read your
   base's `.config/gate/wasm32.txt` and decide the skip for package 30.
2. Measure each package in the table's order, one `heavy` command at a time. Note for each:
   green, or red with its first error and heavy's log path; and the host.
3. Append the green packages to `.config/gate/wasm32.txt` in the table's order; commit (the
   design's other wasm32 crates measured green join fab's gate list).
4. Run the combined command once; note its outcome, wall time and host. Follow Pre-approved on
   a failure.
5. In `temper/docs/plan/wasm32-list/plan.toml`, in the `[[question]]` table whose `id =
   "wasm32-more"`, add after its `parts = ["wasm32-status"]` line:

   ```toml
   answer = "Measured by plan wasm32-more: <n> added, <m> red and recorded as Not yet in their trees."
   ```

   `<n>` is the number of lines this task added to the list, `<m>` the number of red packages
   (one each, though a skein one may be recorded in two files). Commit it with the status
   edits or alone.
6. Make the status edits; commit (the status names the red crates at their first error, and the
   green ones leave it).
7. `python3 -I -B tools/citations/citations.py --base origin/main`; `git log --stat` of your
   commits shows only the files of `## Owns and may change`.
8. Push your branch.

## Report

As the implementer's text says, and also:

- a table of the 46 packages: green, red (lint or code, file, line, heavy's log path), skipped
  or not a package; the host of each run;
- the list as it stands, `<n>` and `<m>`;
- the combined command's outcome, its wall time and host, and whether the target was warm from
  the per-package runs (the gate pays that time on each batch the list's packages touch);
- the citations check's output, and any pre-approved option taken.
