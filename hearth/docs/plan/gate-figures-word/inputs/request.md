# gate-figures-word: every tree's gate figures carry the user's word

The user confirmed on 11-07 that a tree's suite figures, 30 for focused and 120 for fuzzy, are CPU seconds per tree. `tools/tiers/budgets.toml` and hearth's testing design still say that this "stays open". This plan says what was decided, in those places alone. No figure, entry or test changes.

## `tools/tiers/budgets.toml`

Smith's comment line already carries the word, from plan `smith-gate-comment`:

```toml
# Smith's focused and fuzzy suites: 30 and 120 CPU seconds, the strategy's section 8, by the user's word of 11-07.
```

Bring the other trees' lines to the same form:

```toml
# Hearth's fuzzy suite: 120 CPU seconds, the strategy's section 8, by the user's word of 11-07.
# Skein's focused and fuzzy suites: 30 and 120 CPU seconds, the strategy's section 8, by the user's word of 11-07.
# Jig's fuzzy suite: 120 CPU seconds, the strategy's section 8, by the user's word of 11-07.
```

Any other tree's line that says "stays open" by the time this is cut gets the same form, naming its own suites. `Hearth's focused suite: the user's 11-04 ruling` stays as it is.

## `hearth/docs/design/domain/testing.md`

- **1, "In one page":** the fuzzy suite takes 120 CPU seconds, not "120 seconds". Keep the parenthesis on the doubling.
- **9.1:** the fuzzy suite takes at most 120 CPU seconds, by the user's word of 11-07, kept in `tools/tiers/budgets.toml` and measured as the gate does, like the focused suite's. It is no longer "wall time on the development machine, idle". nextest's global timeout stays, as the wall backstop that 8.1 of `skein/docs/foundation/testing-strategy.md` defines.
- **11, "The suites' figures":** drop what is now decided, which is whether the 120 seconds stay wall time or become a CPU budget, and the line that the fuzzy figure stands as an idle-machine figure until the user sets one. Keep what's still open: what the workspace's global timeouts hold, and whether `hearth-render-world` stays on every change. If nothing is left open, the item goes.
- Nothing else in that file. Leave `testing.status.md` alone unless it repeats the open question, and then bring it in line in the same words.

## Limits

- Only those two files, and only those passages. No other tree's design, and no strategy text: 8.1's "until a tier's CPU budget is set" stays.
- One direct task, low effort, no parts. Its checks: `tools/tiers/test_cpu.py` for the file's shape, and the citation check for the doc. Nothing is measured in this plan.
- `walk = false`.

## Sources

Too large for a source, so read by section: `hearth/docs/design/domain/testing.md`, 1, 9.1 and 11.

- `tools/tiers/budgets.toml`

## For every task of this plan

- **Only focused and fuzzy run in a plan.** Never `tools/tiers/run`, a browser, process or live tier, one of their stories, or a tier's CPU figure. This overrides anything below or in a spec.
- **Decide, don't ask.** A `Question:` is only for what truly blocks the task.
- **Before calling a focused or fuzzy failure "outside the change",** run it once on the pristine base. If main passes and your branch fails, it's yours.
