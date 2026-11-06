# box-small: what every task holds to

- **No test reaches a real host.** A rule is a pure function with fakes for `PATH`, `HOME` and the file system; a `sh -c` that checks what a shell makes of a word has no network and runs in a scratch directory.
- **Failing tests first, in the lowest tier.** Budgets and timeouts are the user's (`skein/docs/foundation/testing-strategy.md`, 8.1).
- **Other drafts are on these files.** `ui-walk-3`, `sign-in-new-tab` and `first-usable-model` are changing `temper-client-view`, `temper-client-domain` and `temper-client-shell-web`. `ui-walk-3` has priority: keep hunks small, put new code in new files where a task says so, reorder nothing, reformat nothing beyond `cargo fmt` of what you changed. If your base already has another draft's change to a line you edit, take its shape and keep its tests passing. Say in your report which of their files you touched.
- **Status files:** a task removes the **Not yet** line it closes, as its spec names, and no other. That one edit is allowed against the implementer's text.
- **Citations:** a task that edits design or a status file runs `python3 -I -B tools/citations/citations.py --base origin/main` before it pushes.
