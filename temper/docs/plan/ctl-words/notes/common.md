# ctl-words: what every task keeps

- **Reproduce first, in the lowest tier.** The request comes from reading walks, not from a run. Write the failing test before the fix and run it. If a cause is wrong, say so in your report's first lines, and what you found instead.
- **Focused and fuzzy alone** (`AGENTS.md`, 5). No test reaches a real host: fakes and scripted worlds only.
- **Stay out of the other `temper-client-*` crates.** `temper-client-tools` and `temper-client-shell-cli` are this plan's; what a person meets in a browser is temper/ui's. If a type you change has a reader there, stop at the compiler-forced arm and report it under `## Left`.
- **`ctl-waits` is beside this plan,** on `temper-client-shell-cli`'s `host.rs`, `waited.rs`, its `tests.rs`, and `temper-client-tools`' `root.rs` and `limits.rs`. Stay out of them. `conditions.rs`'s list for `--until` stays as it is.
- **A stored format or a wire form is a question first.** No task here adds a stored field or a wire field, unless its spec says so. If yours turns out to need one, stop and ask.
- **Budgets and timeouts are the user's** (`skein/docs/foundation/testing-strategy.md`, 8.1).
- **`docs/design/` and `docs/help/` depart from your text in one way:** your task names the status lines it closes and the help lines it changes. Change those, nothing else there. The design sections are already written and on the trunk.
- **`walk = false`:** nothing here adds a walk or a QA rung.
- **Keep hunks small.** Other drafts are changing files beside yours. Put a new story in its own file or at a file's end, so a merge finds them apart.
