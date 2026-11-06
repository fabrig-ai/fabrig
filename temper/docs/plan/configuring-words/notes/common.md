# configuring-words: what every task holds to

- **The request** is `temper/docs/plan/configuring-words/inputs/request.md`; `plan.md` beside this file says what was decided.
- **Words are binding where a spec quotes them.** Quoted answers, refusals and `configuring.txt` go in byte for byte, unless the spec gives an option.
- **Fakes and fixtures:** `temper-fake` and the `temper-client-*` crates are the `ui-*` plans' and stay untouched. A scripted run lives in the engine's domain world, `temper/tests/engine/domain/tests/suite/configuring.rs`, through its `host_tool` and `World`.
- **Not touched:** `docs/help/` but for what its own generator writes, every `docs/design/` (this plan's cut wrote the sections), drafts, jig. A task that finds it needs jig stops with a `Question:`.
- **A person's identity:** no email, no display name, no credential enters a tool's answer. `notes/contracts.md` lists what does.
- **Tests:** through `heavy`, as your text says. The packages are `temper-service`, `temper-engine-domain`, `temper-engine-smith`, `temper-engine-domain-world` and `temper-engine-forge-world`.
