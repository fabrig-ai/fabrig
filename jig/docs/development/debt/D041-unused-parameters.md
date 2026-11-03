# D041: Production functions keep parameters they never read

- **Kind:** dead code
- **Where:** `jig/crates`, `examples/ops/domain/src/translate.rs`
- **Seen:** 2026-10-31 at `c30cf6a03`, in the jig code-quality review (T1 "vestigial machinery": 98, 96 and 11 unused `_limits`, `_env` and `_out` parameters)
- **Seen:** 2026-10-31 at `1b5c98834`, in the quality run's debt list ("Ops": `health_words` keeps an unused `_service` parameter)
- **Checked:** 2026-11-03 at `41d63da8c3`
- **Value:** low (signatures say what a function reads)
- **Size:** S
- **Status:** open
- **Paid when:** no production function in jig takes an underscore-named `limits`, `env` or `out` it does not read, except where a shape shared by every child requires it, said in a comment.
- **Measure:** `rg -o '\b_(limits|env|out): &' jig/crates -g '!**/tests/**' -g '!**/tests.rs' -g '!**/generated/**' | wc -l` (31 at base)

## What

31 such parameters remain in `jig/crates`' production code (about 205 at the review), and
`ops`'s `health_words(_service: &observability::Service, healthy: bool)`
(`examples/ops/domain/src/translate.rs`, line 178).

## Why it matters

A parameter that is never read makes every caller build it, and hides that the function
does not depend on it.

## What pays it back

Delete them, caller by caller.
