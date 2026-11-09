# red-87-replay: Story A's session log replays the same as the browser ran it

`browser-temper`'s `browser_playground::story_a_walks_on_the_built_playground` (`temper/tests/client-browser/tests/browser_all/browser_playground.rs`) walks Story A in the built playground. It downloads the session log, then replays the log natively over the same world (`skein/docs/design/playground.md`, 6.2, `skein/crates/skein-playground/src/replay.rs`). Now and then the replay differs from the log:

```
browser_playground.rs:160 panicked: Differs { line: 524, build: Some("618a121ca75da4f3"), why: Said }
```

What the native world said by line 524 differs from the digest the browser recorded there. It happened twice on 11-07 and 4 times on 11-08, out of about 90 runs a day. It also happened in both the run and the rerun at tip `23261749d4` (red-87), whose changes since the last pass were documents only. So something the world says depends on something the log doesn't carry: a time read outside the world's clock, an order that isn't fixed, a fetch's result or timing, or a value from the browser's environment.

## What's wanted

1. **Find what the browser's world said that the native replay doesn't reproduce, and make it deterministic at its cause.**
   - Either what the world says depends only on what the log carries,
   - or the log carries what it depends on, by the design's 6.2.
   - The cause may be in the playground's world, in `skein-ui-web`'s bridge or log, or in temper's playground application.
2. **A `Differs` says what differed.** It names the log line's input and, for `Said`, both digests and the last things the world said before them. The next failure, if any, then shows its cause without another run. Keep it small: a few lines.
3. **A deterministic test in the lowest world** that fails before the fix. It replays a log whose world reads the thing that varied.

## Limits

- No budget, tier or timeout changes. The story keeps its replay check.
- This repairs a red, so it may run `story_a_walks_on_the_built_playground` by name and alone, a few times, to see it pass. It reads no CPU figure.
- `walk = false`. Effort medium. One or two tasks.

## Sources

- `skein/crates/skein-playground/src/replay.rs`
- `temper/tests/client-browser/tests/browser_all/browser_playground.rs`

Read by section: `skein/docs/design/playground.md`, 6.

## For every task of this plan

- **Only focused and fuzzy run in a plan, and a plan is complete when it lands.** The exception is the story named above, by name and alone. Never `tools/tiers/run`, nor a process or live tier. No walk, render check or throwaway driver.
- **Decide, don't ask.** A small edit just outside a task's file list is made, and named in the report. A `Question:` is only for what truly blocks the task.
- **Before calling a focused or fuzzy failure "outside the change",** run it once on the pristine base. If main passes and your branch fails, it's yours.
