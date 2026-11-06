# watch-room: what every task keeps

- **Failing tests first, in the lowest tier.** The evidence is outside this repository, so the words in `temper/docs/plan/watch-room/inputs/request.md` are all you have of it. No test ran for this cut: each figure below comes from reading the code at `bc3996bb40`.
- **Focused and fuzzy suites alone** (`AGENTS.md`, 5). No browser, process or live run. A slow-tier file you edit is compiled with `--no-run`, not run; say so in your report.
- **No test reaches a real host.** No HTTP, browser or kernel in a story.
- **The boundaries.** jig never names temper, and skein names neither. A branch stays in its tree.
- **Stay out of `temper-client-*`,** except `temper-client-tools` and `temper-client-shell-cli`: temper/ui owns them, and is fixing a refused page's words (`No chats yet`, `Behind`) apart. Report what you find there under `## Left`. A story in `temper/tests/client` is fine.
- **A stored format or a protocol is a question first,** so its owner can be told. No task here adds a stored field or a wire field. A busy refusal carries no count, and stays that way.
- **Budgets and timeouts are the user's** (`skein/docs/foundation/testing-strategy.md`, 8.1). The 30 s link deadline stays, and so do `temper_client_world::fixtures::WITHIN` and every tier's budget. No existing heap ceiling rises.
- **`docs/design/` and `docs/help/` depart from your text only where your task says.** It names the status lines it closes: delete those lines, nothing else there. The design sections are already written and on the trunk.
- **`walk = false`:** temper/forge re-runs the check by hand after landing. Nothing here adds a walk or a QA rung.
- **Keep hunks small.** Other drafts are changing files beside yours, and each task names the ones it meets. New stories go in their own file, so a merge finds them apart.
