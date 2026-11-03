+++
id = "skein-status"
part = "skein-status"
kind = "docs"
needs = ["playground-list"]
files = [
  "skein/docs/design/testing.status.md",
  "skein/docs/design/world.status.md",
  "skein/docs/design/STATUS.md",
]
effort = "medium"
minutes = 15
+++

# skein's portable crates in the gate's wasm32 clippy leave skein's status

## Goal

Task playground-list puts `temper-playground` in fab's gate list `.config/gate/wasm32.txt`
when its wasm32 clippy is green. The gate's step lints the listed packages and the workspace
crates they depend on (the note), and `temper-playground` links, for wasm32, the portable
crates skein's status records missing from that clippy: the fake machine, the fakes a world
hosts, the key-value store, JSON, the channel and TLS (`skein/docs/design/testing.md`, 10).
When the list holds the line, close those entries; when it does not, change nothing.

Acceptance, when step 1 finds the line and every crate of step 2:

- `skein/docs/design/testing.status.md`: its date line is today's (`date -u +%F`), and its
  lines 7-13, now

  ```text
    and the programs for trees.
    6, 10: the gate's wasm32 clippy lints the packages of fab's gate list
    (`.config/gate/wasm32.txt`) and the workspace crates they depend on,
    which leaves out, of the portable crates, the fake machine, the fakes
    a world hosts, the key-value store, JSON, the channel and TLS. 7:
    `live-skein` has no CPU budget or wall hard limit in
    `tools/tiers/budgets.toml`.
  ```

  read:

  ```text
    and the programs for trees.
    7: `live-skein` has no CPU budget or wall hard limit in
    `tools/tiers/budgets.toml`.
  ```

  Nothing else in the file changes.
- `skein/docs/design/world.status.md` reads exactly, ending with one newline, as
  `channel.status.md` does for a document with no gap:

  ```text
  # world: status

  <today>. Where skein's code lags or departs from [world.md](world.md).

  No gap.
  ```

  with `<today>` from `date -u +%F`.
- `skein/docs/design/STATUS.md`: its date line is today's; its lines 12-14, now

  ```text
  (`fake-forge.md`). Of the portable crates, the fake machine, the fakes
  a world hosts, the key-value store, JSON, the channel and TLS are not in
  the gate's wasm32 clippy (`testing.md`, `world.md`). json, tls and
  ```

  read, as one line:

  ```text
  (`fake-forge.md`). json, tls and
  ```

  and its line 21, now

  ```text
  `channel.md`, `codec.md`, `kv.md`, `fake-checkout.md`, `fake-llm.md`,
  ```

  reads, as two lines:

  ```text
  `channel.md`, `codec.md`, `kv.md`, `world.md`, `fake-checkout.md`,
  `fake-llm.md`,
  ```

  so `world.md` joins the documents with no gap. Nothing else in the file changes; its index of
  status files keeps its `world.md` line.
- `python3 -I -B tools/citations/citations.py --base origin/main` prints no finding this branch
  adds.

Acceptance, otherwise: no file changes; your branch is pushed at its base; the report's first
line is `Done: skein's status is unchanged`, and says why: the list lacks `temper-playground`,
or the crates step 2 found missing.

## Inputs

- `temper/docs/plan/wasm32-temper-2/notes/common.md`: the standing rules, the gate's step and
  how the status files count its coverage, and this plan's departure from the implementer's
  text for status files.
- `skein/docs/design/testing.md`, 6 (the gate's wasm32 clippy, in "Layout") and 10 ("The
  portable target"); `skein/docs/design/world.md`, 8 ("Testing", its item "For the portable
  target"). Read only.
- The three status files above.

## Owns and may change

- The three files above, in the edits above alone.
- Nothing else: no `<name>.md`, no other status file, no file outside `skein/`. skein's
  `STATUS.md` is not read by tests (`.config/gate/read-by-tests.txt`).

Pre-approved:

- If a line quoted above differs at your base (another plan edited it), make the same change
  in its words: remove only the words that name the six crate groups as missing from the
  gate's wasm32 clippy, add `world.md` to `STATUS.md`'s documents with no gap when
  `world.status.md` says `No gap.`, and say what differed in the report.
- If `world.status.md` holds an entry besides 8 at your base, remove 8 alone and keep the file's
  **Not yet** with the rest; then `STATUS.md` does not list `world.md` as having no gap, and
  its sentence names `world.md` for the rest.

## Steps

1. From the repository root, at your base: `grep -x temper-playground
   .config/gate/wasm32.txt`. No line: push your branch at its base, report as under
   "Acceptance, otherwise" and stop.
2. Run directly (it builds nothing):
   `cargo tree --target wasm32-unknown-unknown -e normal -p temper-playground --prefix none
   --format '{p}' | awk '{print $1}' | sort -u`, and check that it lists each of
   `skein-fake-machine`, `skein-fake-llm-domain`, `skein-fake-llm-protocol`,
   `skein-fake-forge-domain`, `skein-fake-forge-protocol`, `skein-fake-forge-github`,
   `skein-fake-peers`, `skein-kv`, `skein-json`, `skein-channel` and `skein-tls`. One missing:
   push at base, report each missing crate under `## Left` and stop.
3. Edit the three files as above; commit (temper-playground in the gate's wasm32 list brings
   skein's remaining portable crates into its clippy).
4. `python3 -I -B tools/citations/citations.py --base origin/main`; `git log --stat` of your
   commit shows the three files alone.
5. Push your branch.

## Report

As the implementer's text says, and also: step 1's and step 2's outputs (the crates found, not
the whole tree), and the citations check's output.
