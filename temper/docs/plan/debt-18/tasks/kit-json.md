+++
id = "kit-json"
part = "protocol-kit"
kind = "feature"
needs = ["protocol-kit"]
files = ["skein/tests/json/Cargo.toml", "skein/tests/json/src/world.rs", "skein/tests/json/src/transcript.rs", "skein/tests/json/tests/suite/fuzzy/documents.rs", "Cargo.lock"]
effort = "medium"
minutes = 20
+++

# skein: the JSON world on the protocol-world kit

## Goal

`skein/tests/json` takes `fault` and `draw`, the tally and the transcript loader from `skein-protocol-kit`
(`notes/contracts.md`, 1), and its copies go.

Acceptance:

- `rg -n 'fn (fault|draw)\b' skein/tests/json` and `rg -n 'struct Seen\(BTreeMap' skein/tests/json` find nothing.
- `src/transcript.rs`'s scan is `kept`, and `quote` and `unquote` are the kit's.
- Every test in `skein-json-world` passes unchanged, focused and fuzzy.
- The branch's net lines fall by at least 60.

## Inputs

- `temper/docs/plan/debt-18/notes/common.md`
- `temper/docs/plan/debt-18/notes/contracts.md`, 1

## Owns and may change

- **Owns:** the files in `files`. `skein/tests/json/Cargo.toml` gets a `[dependencies]` line; `Cargo.lock` follows.
- **`unquote` is the one trap.** json's strips both ends and accepts a raw `"` inside; the kit's returns `None` for it, and reads hex
  otherwise. A transcript with a raw inner quote then panics when parsed; it can't pass silently. If one does, keep json's own
  `unquote` for json, delete only the rest, and report it. Don't change a transcript.
- **`read_len`** in `json/src/world.rs` isn't `meets`: it stays, and so does `below_acts`.
- Its call sites want boxes (`src/transcript.rs`, about lines 130-131): `.map(Vec::into_boxed_slice)`.

## Steps

1. The dependency, then `fault` and `draw`, then `fuzzy/documents.rs`'s tally (`record` a free function), then the loader.

2. `cargo fmt`; `python3 -I -B skein/scripts/old_forms.py`; `heavy cargo clippy -p skein-json-world --all-targets -- -D warnings`;
   `heavy cargo nextest run -p skein-json-world`, then with `--profile fuzzy`; the citations check. Commit and push.

## Report

The common note's items, and whether `unquote` moved. A move: nothing to break by hand.
