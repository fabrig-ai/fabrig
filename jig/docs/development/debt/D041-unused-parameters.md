# D041: Production functions keep parameters they never read

- **Kind:** dead code
- **Where:** `jig/crates/jig-client-view`
- **Seen:** 2026-10-31 at `c30cf6a03`, in the jig code-quality review (T1 "vestigial machinery": 98, 96 and 11 unused `_limits`, `_env` and `_out` parameters)
- **Checked:** 2026-11-06
- **Value:** low (signatures say what a function reads)
- **Size:** S
- **Status:** open
- **Paid when:** no production function in jig takes an underscore-named `limits`, `env` or `out` it does not read, except where a shape shared by every child requires it, said in a comment.
- **Measure:** `rg -o '\b_(limits|env|out): &' jig/crates -g '!**/tests/**' -g '!**/tests.rs' -g '!**/generated/**' | wc -l` (27)

## What

27 such parameters remain, all in `jig/crates/jig-client-view`
(about 205 across jig at the review). This plan left the crate alone for fabrig/ui.

## Why it matters

A parameter that is never read makes every caller build it, and hides that the function
does not depend on it.

## What pays it back

Delete them, caller by caller.
