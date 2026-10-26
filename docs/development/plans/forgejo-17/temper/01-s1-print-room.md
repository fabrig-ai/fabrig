# temper-01.s1: temper on skein's print room, and the full process tier

A supplement step of forgejo-17 (ruling 104), from the designer's answer
of 2026-10-16T14:46:00Z (temper `551b1d2d`). In temper. Needs skein-00.s1
landed. It ends ruling 101.

Read first: temper's `docs/design/client.md`, 5.4 and 10 (the unit test
from `limits::native()`); decisions.md, ruling 101.

Files: `Cargo.lock`; `crates/temper-client-shell-terminal/src/tests.rs`.

Shape:
1. temper's skein moves to skein-00.s1's landing, `c=$(git -C
   ~/src/rust/skein log origin/main --all-match --grep='^Plan:
   forgejo-17$' --grep='^Step: skein-' --format=%H -1)`: `cargo
   update -p skein-lib --precise $c`, which moves the one skein source
   (29 crates) and touches `Cargo.lock` alone. No move when temper's pin
   already holds it (`git -C ~/src/rust/skein merge-base --is-ancestor $c
   <the pin>` exits 0).
2. The unit test `the_native_limits_make_the_terminal`: `Loop::new(
   &limits::terminal(), &view.tree, ...)` with `view` from
   `limits::view(&limits::native())` (as `crates/temper/src/open.rs:91-101`
   makes it) returns, and its `worst_case()` is `Some`.

No other temper code.

Proof: the gate; then the full process tier, once:
`TEMPER_FORGEJO=$HOME/src/rust/tmp/temper/forgejo/v17.0.0/forgejo-17.0.0-linux-amd64
scripts/proof-one-process.sh` exits 0, and its log's nextest
`Summary` line reads `6 tests run: 6 passed`. The landing report names
its log.

Size: at most 10 production lines (the test is not counted).

For this step only, README's "This plan changes no dependency" and
decisions.md 12 give way: its `Cargo.lock` moves skein. A `Cargo.lock`
conflict takes `origin/main`'s and moves again, as step 1 says.

Don't: change the terminal, `open.rs` or the view limits; move jig or
smith.
