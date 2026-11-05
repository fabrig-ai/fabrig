# debt-21: the twenty-first debt round

Six independent parts (`inputs/request.md`): smith's shell-cwd grader; jig's D004 record; skein D009, D020; jig D017; hearth D009. No temper client view, no `temper.css`, no `jig-client-view`. This file grows with each push.

## Parts and tasks

| Source | Part | Task | Needs | Effort | Min | What it does |
|---|---|---|---|---|---:|---|
| request | `shell-cwd-grader` | `grader-worst-case` | | high | 60 | `worst_case` back; a bundle's proof, or "not validated" in the status |
| request | `d004-record` | `d004-record` | | medium | 5 | one line of a debt record |
| skein D009 | `oauth-secrets` | `oauth-tokens` | | high | 90 | records and token answers on `Secret`, readers in every tree; **Measure** 19 |
| skein D009 | `oauth-secrets` | `oauth-exchange` | `oauth-tokens` | high | 60 | a sign-in's values on `Secret`; carriers print lengths; **Measure** 13 |
| skein D009 | `oauth-secrets` | `accounts-secrets` | `oauth-exchange` | high | 60 | accounts' credentials and its `Debug`s |
| skein D009 | `oauth-secrets` | `fake-oauth-secrets` | `oauth-exchange` | medium | 45 | the fake's credentials and its `Debug`s |
| skein D009 | `oauth-secrets` | `oauth-canary` | `accounts-secrets`, `fake-oauth-secrets` | medium | 30 | the canary's table; **Measure** 0; the item goes |

Still to come: skein D020, jig D017, hearth D009.

## What was decided

- **`shell-cwd` stays at version 3.** `smith/docs/design/benchmarks.md`, 5.2 changes a task's version with its grader, so that results of different versions are never compared. No result exists at 3: version 3 came on 2026-11-04 and no bundle has run it. A bump would say something changed since a run that never happened.
- **The rebuild is timed.** The frozen source is in neither this repository nor the old mirror, so a digest match is a search among rewritten commits and may not exist. `grader-worst-case` has 40 minutes, then records "not validated".

- **D009's order is forced.** Public fields change type in one commit with all their readers, so `skein-oauth` goes first and alone. `state` and `nonce` are `Secret` too: `skein/docs/design/oauth.md`, 4 names them, and it lets `Redirect` and `Event` derive. Carriers stay byte boxes with a hand `Debug` (`skein/docs/design/lib.md`, 6). `Secret` gains no `From`: 54 test sites spell `Secret::new`, and lib's interface stays.

## Overlaps

- D009 edits lines in `temper/crates/temper-service/src/clients/google.rs`, `temper/crates/temper-service/src/tests/accounts_host.rs` and `temper/crates/temper-fake/src/oauth.rs`; `ui-welcome/setup-intent` holds `temper/crates/temper-service/src/tests/**`. No other draft names the first or the last. They are one-line edits, and fab's merges carry them.

## Critical path

`oauth-tokens`, `oauth-exchange`, `accounts-secrets`, `oauth-canary`, 240 minutes.
