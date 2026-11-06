# update-recovers: what every task holds to

- **The request** is `temper/docs/plan/update-recovers/inputs/request.md`; `plan.md` beside this file says what was found and decided, and why. `notes/contracts.md` holds the shapes that cross tasks: binding.
- **Failing tests first, in the lowest tier.** A test of this plan is seen failing, for its rule's reason, before the code that passes it.
- **The focused and fuzzy suites alone** (`AGENTS.md`, 5): no process, browser or live tier runs here. `temper/ui` re-runs its three-thread trial after landing, which is the proof. Do not write a process story.
- **Budgets and limits are the user's:** `forge.client.lifetime_ms`, `forge.client.write_attempts`, `forge.change_policy.updates` and `stall` do not move, on the stock or in a fixture a story shares.
- **Words:** `uncertain: it may or may not have been made` stays for what stays unrecoverable. No new words but the account's, which exist.
- **Not touched:** every `temper-client-*` (temper/ui's), `temper-protocol-store`, every schema, `docs/help/`, and any `docs/design/` but the one status entry a spec names. Closing that entry is a departure from your text, stated here so you may.
- **Status entries** carry their task's name in `temper/docs/design/domain/forge.status.md`: remove the bullet naming your task and no other line.
- **Beside this plan,** and merged by fab where they meet: `debt-29/fake-config` edits `skein/tests/forge/client/src/world.rs`, so new helpers of the client world go in `skein/tests/forge/client/src/world/recovery.rs`; `landing-fixes-2/saved-close` and `landing-fixes-2/repair-cause` edit `temper-engine-domain-forge/src/domain.rs` and `temper-engine-domain/src/route/forge_route.rs`: keep your hunks to the functions your spec names.
- **The fake's late copy reads at its landing:** a rebase's copy in `skein-fake-forge-domain` rebases the head as it then is, so it never drops a later push. Do not change the fake to show that; a story needing a dropped push moves the branch by an outside hand.
