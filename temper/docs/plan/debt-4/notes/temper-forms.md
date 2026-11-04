# debt-4: the temper forms sweep

The rules every `temper-sweep-*` task shares, for `temper/docs/development/debt/D003-widening-try-from.md`
and `temper/docs/development/debt/D004-env-with.md`. Generated code (`**/generated/**`) stays.
Each spec names its packages, as `$OWN` below. No behaviour moves, and this note is why no test
assertion changes.

## The three searches

Each reads nothing over a task's `$OWN` when it ends (`rg -c` prints no line):

```sh
rg -c --type rust -g '!**/generated/**' 'u64::try_from\([^)]*\.len\(\)\)' $OWN
rg -c --type rust -g '!**/generated/**' '"u32 fits usize"' $OWN
rg -U -c --type rust 'Env \{\s*now: (\w+\.)+now,\s*wall: (\w+\.)+wall' $OWN
```

## The forms

Spell the helpers by path, `skein_lib::len_u64(..)` and `skein_lib::usize_of(..)`, as
`temper-engine-domain/src/room.rs` does; add no `use` line for them. Every package of this sweep
has `skein-lib` in `[dependencies]`. `x` below is any expression whose last call is `.len()`,
including a nested one (`u64::try_from(page.text.bytes().len())`), which the first search does not
see and which is swept too.

| Form | Becomes |
|---|---|
| `u64::try_from(x.len()).expect("..")`, `.unwrap()` | `skein_lib::len_u64(x.len())` |
| `u64::try_from(x.len()).ok()?`, `.map_err(..)?`, `?` | `skein_lib::len_u64(x.len())`, the `?` gone |
| `u64::try_from(x.len()).unwrap_or(u64::MAX)`, any `.unwrap_or(..)` | `skein_lib::len_u64(x.len())` |
| `u64::try_from(x.len()).ok()` where an `Option<u64>` is wanted (a match arm, a tail) | `Some(skein_lib::len_u64(x.len()))` |
| `let Ok(n) = u64::try_from(x.len()) else { .. }`, `if let Ok(n) = ..`, a `match` on it | a plain `let n = skein_lib::len_u64(x.len());`; the branch that can never run goes |
| `usize::try_from(n).expect("u32 fits usize")`, over several lines too | `skein_lib::usize_of(n)` |
| `Env { now: env.now, wall: env.wall, limits: l }`, whatever the path to `env` | `env.with(l)` |
| `Env { now: env.now, wall: env.wall, limits }` | `env.with(limits)` |

`&Env { .. }` stays borrowed: `&env.with(l)`.

## What stays

- **`size_of`:** `u64::try_from(size_of::<T>())` is no length of a value; the measure does not
  count it, and it stays on the same line as a swept `len_u64`.
- **A sum or difference of lengths:** `u64::try_from(a.len() - b)`,
  `u64::try_from(size_of::<T>() + x.len())` stay. At base those are in `temper-worker-domain-checkout/src/tests.rs`,
  `temper-engine-domain/src/tests.rs`, `temper-engine-domain-forge/src/tests/brief.rs` and
  `temper-engine-domain-forge/src/tests/tiers/goals.rs`, and one in `tests/real-loop/src/case/oracle.rs`.
- **Other `usize::try_from` texts:** a `usize::try_from(..).expect(<another text>)` is outside
  the item's **Paid when**; leave it.
- **A real narrowing:** `u32::try_from(x.len())`, `u16::try_from(..)`, `usize::try_from(<u64>)`.
- **An `Env` built from other clocks** (`Env { now: self.now, wall: Wall::EPOCH, .. }`,
  `Env { now, wall, limits }`): no rewrap, and the third search does not see it. Leave it.

## Pre-approved fallout

- **The argument of `usize_of` is no `u32`** (the compiler says so): a `u8` or `u16` takes
  `usize::from(n)`; a `u64` is a narrowing under a wrong text, so keep the `try_from`, correct
  the text to what is true, and list it in your report.
- **`clippy::unnecessary_wraps`** on a function the sweep left unable to fail: a private
  function drops its `Option` or `Result` and its callers in the crate change; a `pub` function
  keeps its signature with `Some(..)` or `Ok(..)` and a scoped
  `#[expect(clippy::unnecessary_wraps, reason = "...")]`.
- **A private refusal variant or helper left unused** goes; a `pub` one stays, and your report
  names it.
- **Formatting:** `cargo fmt` reflows what the sweep shortens; let it.

## What runs

This departs from the implementer's text: no meaning and no signature moves, so a task runs its
own packages only, and no direct dependent. `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`
from the root has no form here, and no count moves; `heavy cargo clippy -p PKG ... --all-targets
-- -D warnings`; `heavy cargo nextest run -p PKG ...`, naming every package of `$OWN`. If a `pub`
signature changes after all, stop and say so.

A throwaway `sed`, kept untracked and gone before the push, is the way through most sites; read
the diff by `git diff --stat`, then the sites the table's first two rows did not take.
