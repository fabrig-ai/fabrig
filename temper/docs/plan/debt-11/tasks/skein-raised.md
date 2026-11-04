+++
id = "skein-raised"
part = "skein"
kind = "migration"
needs = ["lint-check"]
files = ["skein/tests/browser/**", "skein/tests/ui-web/**", "skein/tests/kv/**", "skein/tests/link/**", "skein/tests/llm-connection/**", "skein/crates/skein-fake-peers/**", "skein/testing/skein-fake-channel/**"]
effort = "high"
minutes = 90
+++

# skein: the crates `harness` raises

## Goal

Seven skein test crates hold a thinner block than the others, the gaps the review found. `skein/tests/browser`
and `skein/tests/ui-web` have no pedantic at all; the rest lack `allow_attributes`, `unwrap_used` or
`elided_lifetimes_in_paths`. Move them onto `harness` and meet it, with no behaviour changed.

Acceptance:

- For each of the seven, `python3 -I -B tools/lints/lints.py PATH | rg Cargo.toml` finds nothing.
- `heavy cargo clippy -p PKG ... --all-targets -- -D warnings` passes for the seven.
- Their tests pass as before: `heavy cargo nextest run -p PKG` for each, same count, none retimed.

## Inputs

- `temper/docs/plan/debt-11/notes/common.md`
- `temper/docs/plan/debt-11/notes/contracts.md`, 2 and 3
- `tools/lints/design.md`, 2.2, 5 and 6
- `skein/docs/foundation/testing-strategy.md`, 4

## Owns and may change

- **Owns:** `skein/tests/browser` (`skein-browser-world`), `skein/tests/ui-web` (`skein-ui-web-tests`),
  `skein/tests/kv` (`skein-kv-tests`), `skein/tests/link`, `skein/tests/llm-connection`,
  `skein/crates/skein-fake-peers`, `skein/testing/skein-fake-channel`. Not their `clippy.toml`: `skein-clippy`'s.
- **Overlap:** `kv-chunk-target` is changing `skein/tests/kv/src/world.rs` and `tests/suite/`. Keep your edits there
  to what a lint asks, line by line, and don't reformat.
- **Options:**
  - Where pedantic's `must_use_candidate` or `doc_markdown` findings run to many sites in
    `skein/tests/browser` or `skein/tests/ui-web`, take clippy's own fix with
    `heavy cargo clippy --fix --allow-dirty --allow-staged -p PKG --all-targets`, then read the diff.
  - `skein-fake-peers` loses `clippy::panic`: its two `#[expect(clippy::panic, ...)]` in `src/transport/tests.rs` go. It
    also loses `single_match_else = "allow"`: rewrite what fires if the `match` reads no worse, else a scoped exception.
  - `unwrap()` becomes `expect("what it expects")`, in a harness's `src/` and in its tests alike: `allow-unwrap-in-tests`
    comes later, with `ordinary`, in `skein-clippy`.
  - If one crate needs more than 40 scoped exceptions, stop and ask, as `notes/common.md` says.

## Steps

1. One crate at a time, smallest first (`skein-fake-channel`, `skein/tests/link`, `skein/tests/kv`,
   `skein/tests/ui-web`, `skein/tests/browser`, `skein/tests/llm-connection`, `skein-fake-peers`), one commit each:

   ```sh
   python3 -I -B tools/lints/lints.py --set harness skein/tests/link/Cargo.toml
   heavy cargo clippy -p PKG --all-targets -- -D warnings
   ```

2. Fix by `notes/common.md`'s order. `unsafe_code` goes from `deny` to `forbid` in `skein/tests/browser`: no
   `#[expect(unsafe_code)]` sits there at the base.
3. Fmt, old forms, the check, citations; the tests above; commit and push.

## Report

The report of your implementer text, and: for each crate, the levels that moved and the fixes they took, by lint;
the exceptions you added, each with its reason; the branch's net lines.
