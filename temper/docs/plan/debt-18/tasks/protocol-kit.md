+++
id = "protocol-kit"
part = "protocol-kit"
kind = "feature"
needs = []
files = ["skein/testing/skein-protocol-kit/**", "skein/tests/http/**", "Cargo.toml", "Cargo.lock", "skein/docs/design/testing.status.md"]
effort = "high"
minutes = 65
+++

# skein: a protocol-world kit, and the HTTP worlds on it

## Goal

The first part of `skein/docs/development/debt/D032-world-harness-copies.md`. Make `skein-protocol-kit`, and move every world in
`skein/tests/http` onto it: `meets`, `delivers`, `fault` and `draw`, the sweeps' tally, and the transcript loader. Every Rng call
stays what it is, in the order it is, so seeded runs replay as they did.

Acceptance:

- The crate exists as `notes/contracts.md`, 1 says, on `skein-lib` alone.
- `rg -n 'fn (meets|delivers|fault)\b' skein/tests/http` finds nothing. `rg -n 'fn draw32?\b' skein/tests/http` finds only
  `src/generate.rs`'s, a different draw. `rg -c 'struct Seen\(BTreeMap' skein/tests/http` finds nothing.
- `src/transcript.rs` and `src/request_transcript.rs` scan through `kept`, and take `quote` and its helpers from the kit.
- Every test in `skein-http-world` passes unchanged, focused and fuzzy: no assertion, seed or census line changes.
- The `6:` entry about this crate is gone from `skein/docs/design/testing.status.md`.
- The branch's net lines rise by at most 30, the kit's 240 (54 of them TOML) against what http loses. json, tls and protocol, the
  next tasks, are where the item falls.

## Inputs

- `temper/docs/plan/debt-18/notes/common.md`
- `temper/docs/plan/debt-18/notes/contracts.md`, 1
- `skein/docs/development/debt/D032-world-harness-copies.md`
- `skein/docs/design/testing.md`, 6
- `tools/lints/design.md`, 3 and 4

## Owns and may change

- **Owns:** the new crate, and `skein/tests/http/**`.
- **Neighbours:** the root `Cargo.toml` (one line in `members`, one in `[workspace.dependencies]`), `Cargo.lock`,
  `skein/tests/http/Cargo.toml` (a `[dependencies]` line), and that one status entry. This cut already wrote the row in `testing.md`.
- **The item stays:** a later task of this part rewrites it.
- **`fuzzy/gzip.rs`'s unnamed tally** (about line 130) is the same record. Move it if that's shorter, otherwise leave it.
- **`unquote` differs between http and json.** The kit takes http's form (over `quoted`). Moving json is the next task's.
- **`writer_world.rs`** has a usize `generate::draw` and its own `draw32`: import the kit's as `draw as draw32`, and its call
  sites stay as they are.

## Steps

1. The crate, modelled on `skein/testing/skein-fake-channel`: `[lib] test = false, doctest = false`, as the worlds have; the `harness`
   lints (`python3 -I -B tools/lints/lints.py --set harness skein/testing/skein-protocol-kit/Cargo.toml`, copying the block first if
   `--set` refuses a manifest without one); and a `clippy.toml` of the `ordinary` variant. `kept` takes its directory: the kit
   doesn't read `skein-scratch` or any world.

2. `client_world.rs`, `server_world.rs`, `sse_world.rs`, `writer_world.rs`: the four functions. Compare each copy against the
   kit's as you delete it. A difference you find is a question, not a merge. `below_acts` isn't touched.

3. `tests/suite/fuzzy/{client,server,events,writer}.rs`: the tally, with `record` as a free function (the contract says why).
   `server`'s `assert_fell` takes `&[String]`; writer's inline check can stay as it is, since the field is `pub`.

4. The loaders. `all()` keeps its panic text `"{name}.expect: {error}"`, its parsing and its defaults.

5. `cargo fmt`; `python3 -I -B skein/scripts/old_forms.py`; `python3 -I -B tools/lints/lints.py`; `heavy cargo clippy -p skein-protocol-kit -p
   skein-http-world --all-targets -- -D warnings`; `heavy cargo nextest run -p skein-http-world`, then with `--profile fuzzy`; the citations
   check. Commit and push.

## Report

The common note's items, any copy that differed, and whether `gzip.rs` moved. A move: nothing to break by hand.
