+++
id = "playground-cookie-debug"
part = "skein-debug"
kind = "feature"
needs = []
files = [
  "skein/crates/skein-playground/src/visitor/jar.rs",
  "skein/crates/skein-playground/src/visitor/jar/tests.rs",
]
effort = "medium"
minutes = 25
+++

# skein-playground's cookie jar prints a cookie's value as its length

## Goal

`skein/crates/skein-playground/src/visitor/jar.rs`'s `Cookie` (~5) derives `Debug` over its
`value`, a session's cookie among them, and `Jar` and the public `Visitor`
(`src/visitor.rs` ~37) derive `Debug` over it, so a debug print of a visitor shows every cookie
its jar keeps. Give `Cookie` a hand-written `Debug` that prints the value as its length
(`skein/docs/design/playground.md`, 4: the jar keeps the visit's cookies and the page never sees
one; `skein/docs/design/lib.md`, 6, `Secret`).

Acceptance:

- `Cookie` prints `Cookie { name: .., value_len: N, path: .., expires: .. }`; no byte of the value.
- `Jar` and `Visitor` keep their derives, and a `Jar` holding a cookie of the canary formats
  without it.
- The new test fails on the old code and passes on the new.
- clippy clean; the crate's lib tests pass.

## Inputs

- `temper/docs/plan/bugs-security/notes/common.md` and `notes/debug.md`.
- `skein/AGENTS.md`.
- `temper/docs/plan/bugs-security/inputs/request.md`, item 4.
- `skein/docs/design/playground.md`, 4 (the jar); `skein/docs/design/lib.md`, 6.
- Readers, surveyed: none assert these types' `Debug` text.

## Owns and may change

- `skein/crates/skein-playground/src/visitor/jar.rs`: `Cookie`'s derive and its impl.
- `skein/crates/skein-playground/src/visitor/jar/tests.rs`: the test below, appended.

Nothing else.

Pre-approved options:

- If clippy's `missing_fields_in_debug` fires, a scoped `#[expect(clippy::missing_fields_in_debug,
  reason = "lib.md, 6: a cookie's value prints as its length")]`.

## Steps

1. The test first, appended to `src/visitor/jar/tests.rs`, with a made-up canary in both forms
   (`notes/debug.md`), formatting under a scoped `#[expect(clippy::disallowed_macros, reason =
   "...")]` if the crate's lints ban `format!`:

   ```rust
   #[test]
   fn debug_of_a_jar_shows_cookie_lengths_never_values() {
       // Jar::new(3, 64); jar.set(b"session=canary-7f3a9c", b"/", Wall::EPOCH): formatted,
       // "value_len: 13" and the name present, the canary absent in both forms
   }
   ```

   Run `heavy cargo nextest run -p skein-playground --lib -E 'test(/debug_of_a_jar/)'` on the
   old code: it fails. Keep the output.
2. The impl, a line above it citing playground.md, 4.
3. Narrow tests, one at a time:
   - `heavy cargo nextest run -p skein-playground --lib -E 'test(/debug_of_a_jar/)'`: 1 passed.
   - `heavy cargo nextest run -p skein-playground --lib`.
4. `cargo fmt`; `python3 -I -B scripts/old_forms.py` from `skein/`;
   `heavy cargo clippy -p skein-playground --all-targets -- -D warnings`.
5. One commit, the impl and its test, saying why; push.

## Report

As the implementer's text says, and: the failing run on the old code, the passing runs and their
counts.
