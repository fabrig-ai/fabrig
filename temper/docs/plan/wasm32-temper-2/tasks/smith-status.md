+++
id = "smith-status"
part = "smith-status"
kind = "docs"
needs = ["playground-list"]
files = [
  "smith/docs/design/shell.status.md",
  "smith/docs/design/STATUS.md",
]
effort = "medium"
minutes = 10
+++

# smith-agent-shell in the gate's wasm32 clippy leaves smith's status

## Goal

Task playground-list puts `temper-playground` in fab's gate list `.config/gate/wasm32.txt`
when its wasm32 clippy is green. The gate's step lints the listed packages and the workspace
crates they depend on (the note), and `temper-playground` links `smith-agent-shell` and what it
depends on in smith for wasm32, the check `smith/docs/design/shell.md`, 2.5 asks for and smith's
status records missing. When the list holds the line, close that entry; when it does not,
change nothing.

Acceptance, when step 1 finds the line and step 2 the crate:

- `smith/docs/design/shell.status.md`: its date line is today's (`date -u +%F`), and its
  lines 6-8, now

  ```text
    - 2.5: `smith-agent-shell` and what it depends on in smith are not in the gate's
      wasm32 clippy (fab's gate list, `.config/gate/wasm32.txt`), but for
      `smith-transcript`, which `jig-client-shell-web` depends on.
  ```

  are gone, so the **Not yet** list starts at its entry ``- 4, 5.2: the `--account` flag``.
  Nothing else in the file changes.
- `smith/docs/design/STATUS.md`: its date line is today's; its lines 16-17, now

  ```text
  flag (`shell.md`, `testing.md`) and `smith-agent-shell` in the gate's
  wasm32 clippy (`shell.md`), and the benchmarks' calibrations and
  ```

  read, as one line:

  ```text
  flag (`shell.md`, `testing.md`), and the benchmarks' calibrations and
  ```

  `shell.md` keeps its other gaps, so it stays out of the documents with no gap. Nothing else
  in the file changes.
- `python3 -I -B tools/citations/citations.py --base origin/main` prints no finding this branch
  adds.

Acceptance, otherwise: no file changes; your branch is pushed at its base; the report's first
line is `Done: smith's status is unchanged`, and says why: the list lacks `temper-playground`,
or `temper-playground`'s wasm32 tree lacks `smith-agent-shell`.

## Inputs

- `temper/docs/plan/wasm32-temper-2/notes/common.md`: the standing rules, the gate's step and
  how the status files count its coverage, and this plan's departure from the implementer's
  text for status files.
- `smith/docs/design/shell.md`, 2.5 ("The hosted face", its item "Checked"). Read only.
- The two status files above.

## Owns and may change

- The two files above, in the edits above alone.
- Nothing else: no `<name>.md`, no other status file, no file outside `smith/`.
  `.config/gate/read-by-tests.txt` lists `smith/docs/design/**`; its one reader in code,
  `smith-local-domain`'s `src/parent_tests.rs`, includes `shell.md` alone, which this task
  leaves as it is, so no test runs (the note).

Pre-approved:

- If a line quoted above differs at your base (another plan edited it), make the same change
  in its words: remove only the entry, or the clause, that names `smith-agent-shell` missing
  from the gate's wasm32 clippy, and say what differed in the report.

## Steps

1. From the repository root, at your base: `grep -x temper-playground
   .config/gate/wasm32.txt`. No line: push your branch at its base, report as under
   "Acceptance, otherwise" and stop.
2. Run directly (it builds nothing):
   `cargo tree --target wasm32-unknown-unknown -e normal -p temper-playground --prefix none
   --format '{p}' | awk '{print $1}' | sort -u | grep -x smith-agent-shell`. No line: push at
   base, report it under `## Left` and stop.
3. Edit the two files as above; commit (temper-playground in the gate's wasm32 list brings
   smith-agent-shell and its smith dependencies into its clippy).
4. `python3 -I -B tools/citations/citations.py --base origin/main`; `git log --stat` of your
   commit shows the two files alone.
5. Push your branch.

## Report

As the implementer's text says, and also: step 1's and step 2's outputs, and the citations
check's output.
