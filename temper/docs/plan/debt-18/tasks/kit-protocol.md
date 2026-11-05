+++
id = "kit-protocol"
part = "protocol-kit"
kind = "feature"
needs = ["protocol-kit", "kit-json", "kit-tls"]
files = ["skein/tests/protocol/Cargo.toml", "skein/tests/protocol/tests/suite/fuzzy/worlds.rs", "Cargo.lock", "skein/docs/development/debt/D032-world-harness-copies.md"]
effort = "medium"
minutes = 15
+++

# skein: the protocol world's tally on the kit, and D032 rewritten to what's left

## Goal

`skein/tests/protocol`'s sweep takes `Seen` from `skein-protocol-kit`. That finishes the kit clause of
`skein/docs/development/debt/D032-world-harness-copies.md`. Its tests/io clause stays open, by this plan's decision
(`temper/docs/plan/debt-18/plan.md`), so the item is rewritten to it, not deleted.

Acceptance:

- From the repository root, `rg -c 'struct Seen\((pub )?BTreeMap' skein/tests skein/testing | awk -F: '{s+=$2} END{print s}'` reads 1 (7 at
  this plan's base), the kit's, or 2 if `protocol-kit` left a tally it reports.
- The item's own measure, `rg -c 'struct Seen\b' skein/tests skein/testing | awk -F: '{s+=$2} END{print s}'`, reads 10 (16 at base):
  the kit's and the nine other records `notes/contracts.md`, 1 names. Your report gives the reading.
- `rg -c 'fn (meets|delivers)\b' skein/tests skein/testing | awk -F: '{s+=$2} END{print s}'` reads 2.
- D032 reads as below, and `python3 -I -B tools/debt/debt.py` still prints it.
- Every test in `skein-protocol-world` passes unchanged, focused and fuzzy. The branch's net lines fall, the item's text aside.

## Inputs

- `temper/docs/plan/debt-18/notes/common.md`
- `temper/docs/plan/debt-18/notes/contracts.md`, 1
- `skein/docs/development/debt/D032-world-harness-copies.md`

## Owns and may change

- **Owns:** the files in `files`. `skein/tests/protocol/Cargo.toml` gets a dev-dependency (only `tests/suite` reads the kit), with
  the section if it has none; `Cargo.lock` follows.
- **`src/client_end.rs` and `src/server_end.rs`** hold other records, and stay. The inline `between` calls in `src/world.rs` stay.

## Steps

1. `fuzzy/worlds.rs`: `Seen` from the kit, `record` a free function. Its `tally.0.contains_key` line can stay as it is.

2. Rewrite the item. Keep its `Seen:` lines, `Value` and `Status`; set `Checked` to your base; the file's name stays:

   ```markdown
   # D032: tests/io runs its worlds on hand loops, beside `skein_world` and io's stage

   - **Kind:** tests
   - **Where:** `skein/tests/io/src/world.rs`, `skein/tests/io/src/output.rs`, `skein/tests/io/src/joined.rs`, `skein/testing/skein-world/src/world.rs`, `skein-io/src/stage.rs`
   - **Size:** M
   - **Paid when:** tests/io's socket, output and either worlds run on `skein_world` and io's stage, keeping the per-call `MAX_OUT` check and the ledger.
   - **Measure:** `rg -c 'advance_to' skein/tests/io/src | awk -F: '{s+=$2} END{print s}'`: 3 at base

   ## What

   `world.rs` over `Sim`, `output.rs`'s conversation and `joined.rs`'s `either` each run their own loop; nine `impl Host` modules
   already run on `skein_world`. Of those, only `append.rs` and `usage.rs` use `skein_io::stage`.

   ## Why it matters

   A service runs on the stage, so these worlds hold its machines to a drive no service uses.

   ## What pays it back

   Two steps, and the second is a decision, not a refactor.

   - **The loop:** `skein_world` needs a `Referee::idle` that can change the referee at idle, and the worlds a shared directory
     that doesn't delay a dial by an iteration. By reading it keeps each schedule; a per-seed digest of the trace and logs, before and after,
     proves it. About 30 lines out.
   - **The stage:** not the same schedule by construction. These worlds fire deadlines with completions waiting, and `stage.rs`
     returns first; it has no per-stage resume bound here; and it has no place for the per-call `MAX_OUT` check or for the ledger's
     `request` on each request taken, since `Request` isn't `Clone`. It needs a per-entry witness in `skein_io::stage`, a step crate,
     or a panic path in shipped code.
   ```

3. `cargo fmt`; `python3 -I -B skein/scripts/old_forms.py`; `heavy cargo clippy -p skein-protocol-world --all-targets -- -D warnings`; `heavy
   cargo nextest run -p skein-protocol-world`, then with `--profile fuzzy`; the citations check. Commit and push.

## Report

The common note's items, and all three searches' readings. A move: nothing to break by hand.
