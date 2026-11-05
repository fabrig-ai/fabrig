# debt-20: the widening migration (skein D002)

Every `widen-*` task follows this note. The item is `skein/docs/development/debt/D002-index-and-length-widening.md`; the
helpers are `skein_lib::usize_of` and `skein_lib::len_u64` (`skein/docs/design/lib.md`, 6, its **Widening** line).

## The rule

| At a site | Becomes |
|---|---|
| `usize::try_from(X).expect("..")`, `X: u32` | `usize_of(X)` |
| `u64::try_from(Y.len())` with any ending (`.expect(..)`, `.ok()?`, `.unwrap_or(..)`), `Y.len(): usize` | `len_u64(Y.len())` |

The compiler decides the type: `usize_of` takes only a `u32` and `len_u64` only a `usize`. So replace, then read what fails to compile:

- `X: u64` (or anything that can fail): **it stays as it is.** These are the item's floor; your report lists each with its file and line.
- `X: u8` or `u16`: `usize::from(X)`.
- `Y.len(): u32` (a `List`, a `Queue`): `u64::from(Y.len())`.

Inside `skein-lib`, the same through `crate::`; the two helpers' own bodies in `length.rs` stay.

## What stays

- **What the schema compiler prints stays:** every file under a `generated/` directory, and any string literal that prints such code. Changing them would regenerate every tree's codecs (27 sites at base: `skein-oauth/src/generated/v3.rs`, 9; `examples/ui/view/src/generated/v1.rs`, 18).
- **Test files stay:** a crate's `src/tests/**`, `tests.rs`, `*_tests.rs` and its `tests/`, and everything under `skein/tests/` and `skein/testing/`. A site in an inline `#[cfg(test)]` module is in scope, since the item's measure counts it.
- **Other shapes stay:** `usize::try_from(X)` with `.ok()`, `?` or `.unwrap_or(..)` is not the form, even for a `u32`, and may stay; the item names only `.expect(..)` for it.
- **A crate without a `skein-lib` dependency keeps its sites,** and your report lists them. Don't add the dependency for this.

## Keeping behaviour

A `u32` fits a `usize` and a native length a `u64` on every supported target, so no site's result changes. A signature doesn't change either: where `len_u64` removes the only `?` of a function returning `Option`, keep the signature, with a scoped `#[expect(clippy::unnecessary_wraps, reason = "..")]` if clippy asks. A message that goes with the `expect` goes.

Import the way the file does: a `use skein_lib::{..}` where it has one, or the path at the call where it names `skein_lib::` there. `rustfmt` reflows the lines.

## The searches

Where `P` is one of the paths your spec names (`rg` skips nothing else, so run it on each):

```
rg -n 'usize::try_from\(.*\)\.expect\(|u64::try_from\([^)]*\.len\(\)\)' P -g '!**/tests/**' -g '!**/tests.rs' -g '!*_tests.rs' -g '!**/generated/**'
rg -nU 'usize::try_from\([^;]*?\)\s*\.expect\(|u64::try_from\(\s*[^;]*?\.len\(\)\s*,?\s*\)' P -g '!**/tests/**' -g '!**/tests.rs' -g '!*_tests.rs' -g '!**/generated/**'
```

Both list only what your report names as staying. The second also finds what `rustfmt` wrapped and a nested call the first misses. A line it lists whose conversion is not the form (a `.ok()` chain ending in another value's `.expect`) is exempt; name it.

## The run

A widening changes nothing a dependent can see, so your run names the crates you own and not their dependents. This departs from the implementer's text for that reason.

`cargo fmt`; `python3 -I -B skein/scripts/old_forms.py`; `heavy cargo clippy -p PKG [-p PKG ...] --all-targets -- -D warnings`; `heavy cargo nextest run -p PKG [-p PKG ...]`; the citations check.

No new test: `widen-close` pins the rule with an old form. Your report gives the two searches' counts before and after, each site that stays with its type, the net lines, and any test you could not run.
