# help-server: what every task holds to

- **fabrig/ui has priority.** Its drafts are changing `temper-client-view`, `temper-client-domain`, `temper-client-shell-*`, `temper/tools/gallery`, `temper/tools/journeys` and `temper/docs/design/client/**`. No task here needs them: leave them alone. Expect the trunk to catch up with main beneath you.
- **Your first step confirms your finding on the trunk.** If it no longer holds, say so in your report's first line and drop it.
- **No test reaches a real host.** The worlds' fakes, and the process tier's own Forgejo, on loopback.
- **Public files name nothing of the deployment the findings came from** (`AGENTS.md`, 1 and 2; `tests/boundary/tests/outward.rs`): `temper.example.com` and `git.example.com`, as the page has them.
- **`temper/docs/help/**` is compiled into its tests** (`.config/gate/read-by-tests.txt`): a page's change runs `heavy cargo nextest run -p temper-docs` and `heavy cargo nextest run -p temper --lib -E 'test(~help)'`.
- **Budgets and timeouts are the user's** (`skein/docs/foundation/testing-strategy.md`, 8.1): over budget is red, and your report gives the measurements. Nothing moves a test to escape one.
