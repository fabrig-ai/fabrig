+++
id = "wasm32-status"
part = "wasm32-status"
kind = "docs"
needs = ["gate-list"]
files = [
  "skein/docs/design/testing.status.md",
  "skein/docs/design/lib.status.md",
  "skein/docs/design/shell.status.md",
  "skein/docs/design/world.status.md",
  "skein/docs/design/STATUS.md",
  "jig/docs/design/protocol/browser.status.md",
  "jig/docs/design/protocol/README.status.md",
  "jig/docs/design/STATUS.md",
  "temper/docs/design/client/browser.status.md",
  "temper/docs/design/STATUS.md",
  "smith/docs/design/shell.status.md",
  "smith/docs/design/STATUS.md",
]
effort = "medium"
minutes = 20
+++

# The wasm32 gate list's gaps, recorded in the status files

## Goal

fab's gate clippies for `wasm32-unknown-unknown` exactly the packages of
`.config/gate/wasm32.txt` (task gate-list): `skein-ui`, `skein-ui-web`,
`skein-ui-example-view`, `skein-ui-example-shell-web`, `jig-client-shell-web` and
`skein-playground`. The design names more crates for that check, in every tree, and their
status files record no gap. Record each as **Not yet**, in the words below, so the status
says what the gate checks. Status text only: no design document, list or code changes.

Acceptance: the twelve files below read as given, their date lines today's
(`date -u +%F`), every other word of them unchanged; the citations check passes.

### skein

- `skein/docs/design/testing.status.md`: in **Not yet**, between the entry `4: ...programs for
  trees.` and `7:`, insert:

  ```text
  6, 10: of the portable crates, fab's gate list (`.config/gate/wasm32.txt`) holds
  `skein-playground` alone, so lib, io, the simulator, the shell's portable face, the
  simulated world, the fake machine, the fakes a world hosts, the key-value store, HTTP,
  JSON, the channel, the codec and TLS are not in the gate's wasm32 clippy.
  ```

- `skein/docs/design/lib.status.md`: its **Not yet** becomes:

  ```text
  - **Not yet:** 2: lib, `skein-codec` and `skein-http` are not in the gate's wasm32
    clippy (fab's gate list, `.config/gate/wasm32.txt`), where `skein-ui` is. 3:
    `Slab::get2_mut`.
  ```

- `skein/docs/design/shell.status.md`: at the end of **Not yet**, after `...runs Darwin's
  code.`, append:

  ```text
  14: the shell and the simulated world are not in the gate's wasm32 clippy (fab's gate
  list, `.config/gate/wasm32.txt`).
  ```

- `skein/docs/design/world.status.md`: `No gap.` becomes:

  ```text
  - **Not yet:** 8: `skein-sim-world`, the simulator, the fake machine and the shell's
    portable face are not in the gate's wasm32 clippy (fab's gate list,
    `.config/gate/wasm32.txt`).
  ```

- `skein/docs/design/STATUS.md`, the summary paragraph: after the sentence that ends
  `` the fake GitHub refuses no push as `GH013` (`fake-forge.md`). ``, insert the sentence:

  ```text
  Of the portable crates, the gate's wasm32 clippy (fab's gate list) holds
  `skein-playground` alone (`testing.md`, `lib.md`, `shell.md`, `world.md`).
  ```

  and in the list of documents with no gap, drop `` `world.md`, ``. Rewrap at about 72
  columns as the paragraph is.

### jig

- `jig/docs/design/protocol/browser.status.md`: `No gap.` becomes:

  ```text
  - **Not yet:** 15: of jig's wasm32 builds, fab's gate list (`.config/gate/wasm32.txt`)
    holds `jig-client-shell-web` alone: `jig-client-test-shell-web` and the client crates
    are not in the gate's wasm32 clippy. 18: nor is `jig-protocol-transcript`, so no gate
    checks `smith-transcript`'s build for wasm32.
  ```

- `jig/docs/design/protocol/README.status.md`: at the end of **Not yet**, after `...contained
  process trees for workers' agents.`, append on the same line, as that entry is written:

  ```text
   11: `jig-protocol-transcript` is not in the gate's wasm32 clippy (fab's gate list, `.config/gate/wasm32.txt`).
  ```

- `jig/docs/design/STATUS.md`, the summary paragraph: after the sentence
  `` `RemoveProject` is offered in no project's document (`client/domain/contract.md`). ``,
  insert:

  ```text
  Of jig's wasm32 builds, the gate's clippy (fab's gate list) holds `jig-client-shell-web`
  alone (`protocol/browser.md`, `protocol/README.md`).
  ```

  and `` `domain/core.md`,
  `domain/providers.md`, `client/domain/README.md` and `protocol/browser.md` have no gap of
  their own. `` becomes `` `domain/core.md`, `domain/providers.md` and
  `client/domain/README.md` have no gap of their own. ``

### temper

- `temper/docs/design/client/browser.status.md`: in **Not yet**, between the 2.1 entry (ending
  `` `record` and `loopback-navigation` alone. ``) and `4.6:`, insert:

  ```text
  4.3: none of `temper-client-domain`, `temper-client-view`,
  `temper-client-protocol-wire` and `temper-client-shell-web` is in the gate's wasm32
  clippy (fab's gate list, `.config/gate/wasm32.txt`); `temper-client-shell-web` fails
  it at `clippy::too_many_lines` (`src/lib.rs`, `turn`).
  ```

- `temper/docs/design/STATUS.md`, the summary paragraph: after
  `` many of ctl's tools (`client/README.md`, `client/forge.md`, `client/tools.md`), ``
  insert:

  ```text
   the client's crates in the gate's wasm32 clippy (`client/browser.md`),
  ```


### smith

- `smith/docs/design/shell.status.md`: **Not yet** gains a first item, before `- 4, 5.2:`:

  ```text
    - 2.5: `smith-agent-shell` and what it depends on in smith are not in the gate's
      wasm32 clippy (fab's gate list, `.config/gate/wasm32.txt`).
  ```

- `smith/docs/design/STATUS.md`, the summary paragraph: `` the `--account`
  flag (`shell.md`, `testing.md`), `` becomes `` the `--account` flag (`shell.md`,
  `testing.md`) and `smith-agent-shell` in the gate's wasm32 clippy (`shell.md`), ``.

## Inputs

- `temper/docs/plan/wasm32-list/notes/common.md`: the standing rules, and this plan's
  departure from the implementer's text for status files.
- The sections each entry cites, read only: `skein/docs/design/testing.md`, 6 and 10;
  `skein/docs/design/lib.md`, 2; `skein/docs/design/shell.md`, 14;
  `skein/docs/design/world.md`, 8; `jig/docs/design/protocol/browser.md`, 15 and 18;
  `jig/docs/design/protocol/README.md`, 11; `temper/docs/design/client/browser.md`, 4.3;
  `smith/docs/design/shell.md`, 2.5.
- `.config/gate/wasm32.txt` on your base (gate-list's).

## Owns and may change

- The twelve files above, as given; nothing else. No design document, `.config/gate/` file,
  draft or code changes.
- No test reads a status file (`rg -n 'status\.md|STATUS\.md' --glob '*.rs'` finds only
  `#[ignore]` reasons); smith's design tree is read by tests, but only `shell.md` among the
  files near these (`smith-local-domain/src/parent_tests.rs`), which this task leaves alone.

Pre-approved:

- If an anchor sentence above is not found word for word (another branch rewrote it), place
  the entry where the section order puts it (status labels ascend by section) and say so in
  the report.
- If the base's `.config/gate/wasm32.txt` holds a package an entry above says it lacks, drop
  that package's words from the entry, and drop the entry if none is left; say so.
- If the citations check reports a broken label or section in an entry above, fix that label
  alone, in the entry's words, and say so.

## Steps

1. Make the edits above, set the twelve date lines, commit (the gate's wasm32 clippy holds six
   packages; the status files record where the design names more).
2. `python3 -I -B tools/citations/citations.py --base origin/main`: no finding this branch
   adds.
3. `git log --stat` of your commits shows the twelve files alone. No build and no test: nothing
   compiled or read by a test changes.
4. Push your branch.

## Report

As the implementer's text says, and also the citations check's output and any anchor that
needed the pre-approved placement.
