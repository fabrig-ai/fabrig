# fabrig

These lines add fabrig's rules to the ones above.

- **Its rules for agents:** the root `AGENTS.md` and each tree's `AGENTS.md`; whoever writes or reviews design reads `DESIGN.md` too.
- **Upgrades are supported as far as dev.fabrig.ai needs them:** each build reads the store and talks to the workers of the build dev.fabrig.ai runs, by `AGENTS.md`'s layouts, and keeps no code for an older one.
- **A goal's reds,** failing on g and owned by no live change, are temper's records, which the goal's coordinator keeps with the `red` tool. Each is keyed as the delivery check names a failure (`test <binary_id> <name>`, `lint <package> <code> <file>`) and recorded at a commit of g. A change's check passes it when its base contains that commit.
