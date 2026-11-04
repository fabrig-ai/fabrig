+++
id = "skein-worlds"
part = "skein"
kind = "migration"
needs = ["lint-check"]
files = ["skein/tests/**/Cargo.toml", "skein/tests/**/*.rs", "skein/testing/**/Cargo.toml", "skein/testing/skein-fake-person*/**", "skein/testing/skein-forge-conformance/**", "skein/testing/skein-throwaway-forgejo/**", "skein/crates/skein-fake-checkout/**"]
effort = "medium"
minutes = 60
+++

# skein: the worlds and kits that take `harness` as they are

## Goal

Move skein's test crates whose block is already near `harness` onto it. These packages lose
`wildcard_enum_match_arm` and little else, so this is the mechanical half of skein's move. Those that rise are
`skein-raised`'s, the shells are `skein-shells`', and every `clippy.toml` is `skein-clippy`'s.

Acceptance:

- For each package below, `python3 -I -B tools/lints/lints.py PATH | rg Cargo.toml` finds nothing.
- `heavy cargo clippy -p PKG ... --all-targets -- -D warnings` passes for each package you changed.

## Inputs

- `temper/docs/plan/debt-11/notes/common.md`
- `temper/docs/plan/debt-11/notes/contracts.md`, 2 and 3
- `tools/lints/design.md`, 2.2, 3 and 5

## Owns and may change

- **Owns:** the manifests of the list in `## Steps`, and their code as far as `notes/common.md`'s list says.
- **Not yours:** `skein/tests/browser`, `skein/tests/ui-web`, `skein/tests/kv`, `skein/tests/link`,
  `skein/tests/llm-connection`, `skein-fake-peers`, `skein-fake-channel` and every `clippy.toml`.
- **Options:**
  - `skein-fake-checkout` loses `missing_docs = "deny"`: its docs stay, and nothing else follows.
  - If a package below turns out to need more than a dozen fixes, finish the others, keep it for last, and say
    so in the report.
  - If `--set` deleted a comment that spoke of something other than lints, put it back: `skein/tests/tls`'s
    says something of rustls.

## Steps

1. ```sh
   python3 -I -B tools/lints/lints.py --set harness PATH...
   ```

   `PATH...` is each `Cargo.toml` of the contracts' section 2, skein, `harness`, but for the seven of **Not yours**.
2. Clippy, then what it turns red, by `notes/common.md`: 9 `#[expect(clippy::wildcard_enum_match_arm, ...)]` sit in
   skein's harness crates at the base (`skein/tests/llm`, `skein/tests/io`, `skein/tests/channel`,
   `skein-throwaway-forgejo`), and go. Those in step crates stay. `skein/tests/codec` and `skein/tests/codec-patches`
   are on `step` and stay there.
3. Fmt, old forms, the check, citations.
4. Tests: `heavy cargo nextest run -p PKG` for each package whose code changed beyond a deleted or added attribute.
   A lint block changes no behaviour.
5. Commit and push.

## Report

The report of your implementer text, and: each level you saw move, against the contracts' section 3; the
exceptions you added, by crate; the branch's net lines.
