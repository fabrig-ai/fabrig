# fetch-progress-1: what every task holds

- **Changed in place:** until v0.1.0 there is no supported deployment, so the pipe, the client protocol, its codecs and its goldens change in place, with no version kept for an older side (`temper/docs/design/draft/fetch-progress.md`, 1).
- **Nothing here is stored:** a fetch's progress lives in serve's memory and on the wire to a client, never in the store (`temper/docs/design/protocol/forge.md`, 2.8). A task that finds itself in `temper-protocol-store` stops and asks.
- **Beside this plan** `look-round-3`, `to-begin-store` and `repository-copies-1` are building on the same files: `temper-client-view`, `temper.css`, `temper-client-protocol-wire`, `temper-service/src/setup*`. Keep each edit small and on lines of its own; merges meet them. A rebase only when a spec asks.
- **The design's Later stays later:** one percentage for both binaries, progress on the other steps, a fake fetch for the fake box (`temper/docs/design/draft/fetch-progress.md`, 8). A journey records no percentage.
- **Budgets and timeouts are the user's.** `.config/nextest.toml` is not touched.
- **`docs/design/` stays out of a task,** but where its spec says otherwise: a spec may close its own status entries, by exactly the lines it names. What the code taught goes in the report.
