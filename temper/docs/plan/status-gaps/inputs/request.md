# Request: six code gaps the normalized status files record

From the director, 2026-11-02, after docs-normalization landed (fabrig main 2a122ba7e5): the status
files now name these as "Not yet" or "Departs". **The status files are the spec**: read each
item's status entry and the design section it cites, and close the gap in code, design text that
the change makes true updated beside it (the status entry removed or reworded). If an item turns
out to be design work rather than code (a rule the design leaves open, a choice it does not make),
ask: a `[[question]]` or the task's `Question:`; never guess.

1. **skein's readiness backend and an interrupted `Socket`, `Accept` or `Connect`**
   (`skein/docs/design/shell.status.md`, the 7.4 entry). The status entry says the rule is open
   (`shell.md` 10) and no reason is recorded, so this is likely the design's call first: ask
   before building.
2. **Remove `testing/skein-lab-browser`** and `skein/docs/design/browser.md` 5.1's look reads,
   since the lab is retired (`skein/docs/design/lab-browser.status.md`, `ui-web.status.md` 14.2,
   `testing.md`'s crate list, `skein/docs/design/README.md`'s table). Remove the crate from the
   workspace and its tests from the tiers' package map.
3. **A connected Forgejo can be configured:** `Connect` with its pasted tokens, the `Check` plan
   and a token's `Replace` (`temper/docs/design/protocol/forge.status.md` 2.2 and 2.6,
   `temper/docs/design/protocol/setup.status.md` 4, `temper/docs/design/protocol/README.status.md`
   4).
4. **temper's `STORE_VERSION` and `CLIENT_VERSION` against skein's `codec.md` 5** (the versions'
   entries in `temper/docs/design/protocol/README.status.md` and `store.status.md`). A stored
   format is hard to reverse, so ask if the design does not settle the values.
5. **Remove the withdrawn compatibility forms** (`smith/docs/design/STATUS.md`, the agent,
   channel, hosts and llm status files; `skein/docs/design/llm.status.md` 4.6 and 4.7: the Codex
   route and Anthropic's bearer route with its identity profile).
6. **Remove `TaskWaitRefused::Program` and the retained `Seen` compatibility** in jig
   (`jig/docs/design/STATUS.md`, `jig/docs/design/domain/hosts.status.md`,
   `jig/docs/design/protocol/client.status.md` 3).

One task per item at most, each in its own tree. They share nothing, so they may run in parallel
and items that ask can wait without holding the rest. Low priority: the ladder's repairs come
first.
