# github-owner: a neutral owner in temper's GitHub tests and probes

The user decided that the private domain and personal names leave current files, with no history rewrite. Plan `fail-stop-names` (fabrig/hearth's) handles the docs; it leaves temper's live-probe code to this plan. The personal account's name is in:

- `temper/crates/temper/tests/support/relay_managed.rs` (20 mentions);
- `temper/crates/temper/tests/support/managed_client.rs`;
- `temper/crates/temper/tests/suite/startup.rs`;
- `temper/crates/temper/tests/support/live_github.rs`'s allowlist (`"freeekanayaka" | "fabrig-ai"`);
- `temper/scripts/proof-live-github-managed.py`, `OWNER`, its `temper-probe-2` marks and its steps' words;
- `temper/scripts/proof-live-github.sh`.

Run `git grep` for the name across every tree first, and take whatever else carries it.

## The task

1. **Tests on the fake forge** take a neutral owner, not a real account. Where a fake says the owner's type is `User`, keep that type under a neutral name.
2. **The live probes run under `fabrig-ai`:** `live_github.rs`'s allowlist drops the personal account, and so does `proof-live-github.sh`. `proof-live-github-managed.py`'s `OWNER` becomes `fabrig-ai`.
3. **Check whether any story needs a personal owner, of GitHub's type User, rather than an org.** `relay_managed.rs`'s fake says "User". If a live story truly needs a User owner, keep the guard as an input rather than a name in the file, and say so in the report. Never widen the guard: it stops a probe from reaching any other account (a safety guard, `AGENTS.md` 3).

**Known on the real system (agents/creds, 11-04):** `fabrig-ai/temper-probe`, `-2` and `-3` exist (private, branch `main`, `ci.yml`), App installation 177453345 covers all three, and the shelf's `github-1` to `-3` are current. No new repository, App or login is needed. A task never reaches a real host: no live probe runs in this plan, and fabrig/qa's next GitHub walk proves it. GitHub rungs are not due, so editing `kit-github` isn't this plan's.

No design change, unless a design section names the account; then neutral wording, as `fail-stop-names` does. Beside it: `fail-stop-names` touches `testing.md` and `draft/github.md`. Leave those to it.

## Size

One task.
