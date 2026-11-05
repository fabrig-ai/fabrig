# help-server: `temper help server`'s walk works as written

From ops, who redeployed dev.fabrig.ai by `temper help server` at main `fadf954` on 2026-11-05 and reproduced each of these there. fabrig/ui has priority: leave its files alone, and expect catch-ups. No test reaches a real host. A task's first step is to confirm its finding on today's main; if one no longer holds, say so and drop it.

## The findings

1. **`temper serve` refuses `application.sign_in.forges`, with a misleading error.** With `"forges": [{"forge": 1, "client": "forgejo"}]`, a config that worked before, `temper serve` exits:

   ```text
   application.clients.bundle: embedded: page needs </head> and room within clients.bundle_bytes for sign-in metadata
   ```

   Raising `clients.bundle_bytes` to 256 MiB didn't help; removing `sign_in.forges` did. The effect: signing in to the deployment's forge through temper is off. **Believed, from reading, not proved:** `crates/temper/src/lib.rs` calls `fill_forge_sign_in(named.sign_in.as_ref(), &[], ...)`, so the forges list is empty at startup, `forge_metas` probably returns None, and that's reported as this message. Fix both the refusal and the wording: an error says what's actually wrong. A failing test first, in the lowest tier that shows it.

2. **`help server`, step 3 fails:** `temper forge fetch ... 1` answers `forge 1: /var/lib/fabrig/data/box/groups/1: absent`. ops wrote `owned\n` there by hand, from `tests/support/owned.rs`, and fetch, prepare and run then worked. Neither the page nor the redeploy's "remove DATA but secrets and forge-1" step mentions it. Decide the fix: fetch needing no group outside a box, or a command that writes it, or at the least the page saying so. Prefer that the walk needs no hand-written file.

3. **The page's engine doesn't connect the owned forge:** after the start, `forge list` is empty and `forge setup` is box-only. ops used `forge connect forgejo-admin DATA/forge-1/admin_token`. The page says so, or the walk does it.

4. **The page's worker unit:** `ProtectSystem=strict` needs `PrivateTmp=yes`, or Claude Code runs fail with `program claude_code exit 1: EROFS: read-only file system, mkdir '/tmp/claude-986'`. The page's hint gains it. And the failure itself: the task only shows `held: out of tries: run failures`, with the reason in the engine's journal alone. If it's cheap, the hold names the run's last failure; if it needs a protocol or stored-format change, end with a `Question:` instead.

## Context

- `temper/docs/help/` is compiled into its help tests, so a page's change needs its tests (`AGENTS.md`, 2).
- ops's working config and scripts: `~/src/fabrig/coord/ops/bin/dev-*`. Read them for what really worked; name nothing of them in public files (`AGENTS.md`, 1 and 2).
- The design: `temper/docs/design/protocol/sign-in.md`, 2, and `protocol/setup.md`. Where the page and the design differ, the design wins, or says why it changes.

## Size

Small: three or four tasks, no design of its own beyond what the fixes need. Budgets and timeouts are the user's (skein's `docs/foundation/testing-strategy.md`, 8.1).
