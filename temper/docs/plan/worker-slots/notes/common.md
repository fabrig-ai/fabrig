# worker-slots: what every task holds to

- **Off is today, byte for byte.** With the variable unset no default path changes: no argument, no operation, no fact and no order. The existing tests of `temper-worker-domain-checkout`, `temper-worker-checkout-world` and `temper-worker-protocol-git` stay as they are but for compiler-forced arms. A test that needs its expected value changed is a red, not a catch-up.
- **The variable is the shell's alone.** `temper/crates/temper` reads `TEMPER_WORKSPACE_REUSE`, once, at the worker's start. Below it, it is a call on a value (`notes/contracts.md`), never a field of `Limits`, the configuration, a stored record or a wire. A stored format, a protocol or a public interface change is a `Question:`.
- **No help page.** `temper/docs/help/**` stays as it is, and no configuration reference gains a line.
- **fabrig names nothing of the deployment** (`AGENTS.md`, 1; `tests/boundary/tests/outward.rs`): no tool, machine, target directory or cache. A build's target and limits are the deployment's own.
- **`Command::Clean` and the saved start's empty directory stay as they are.**
- **fabrig/ui has priority.** Its drafts are changing `temper-client-*`, `temper/tools/gallery`, `temper/tools/journeys`, `temper/tests/client*`, `temper/tests/one-process` and `temper-service/src/tests/**`. Leave those files alone. New test code goes in a new file, with its one `mod` line. Expect catch-ups from the trunk; take them as they come.
- **No skein, jig or smith change.** `skein_fake_checkout` keeps its `replace_tree`, which issues every file a new version, so a story measures what a check-out touched by the world's own comparison of the tree before and after it (`notes/contracts.md`), not by `Checkout::version`.
- **No test reaches a real host.** The worlds' fakes alone.
- **Budgets and timeouts are the user's** (`skein/docs/foundation/testing-strategy.md`, 8.1): over budget is red, and your report gives the measurements.
- **Design is the lead's.** `temper/docs/design/domain/worker.status.md` and `temper/docs/design/protocol/git.status.md` already say what is built here: if the code teaches otherwise, say so in your report.
