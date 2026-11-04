# fake-signin-model: the rules every task names

Read this note, then your spec. Where this note and the implementer's text differ, this note wins.

## The fakes

- **No test reaches a real host,** and no walk does. A fake box's peers are its own: the issuer
  (`temper fake oauth`), the model (`temper fake llm`), and the forge fakes
  (`temper/docs/design/protocol/setup.md`, 7.5).
- **Forgejo is real on a fake box:** a walk or a test that runs it on this machine needs the
  builder's `TEMPER_FORGEJO` and `TEMPER_FORGEJO_RUNNER`, and the callback port 1455 free. Never a
  second box on it at once.
- **A box's secrets stay unread** (`AGENTS.md`, 3): never open its key files, or print a recording's
  tokens into a report. The printed sign-in link and the fakes' keys are dummies, and a report still
  leaves them out.

## What not to touch

- **Budgets and timeouts are the user's** (`skein/docs/foundation/testing-strategy.md`, 8.1). Over
  budget is red; the spec of your task doesn't change that.
- **A wire schema or a stored record** is the user's: a fix that would change one is a `Question:`,
  with options and your recommendation.
- `temper/docs/design/client/journeys/overlay.css` stays empty, and a draft's own text and
  `docs/help/` stay as they are.
- `look-requests` is the person's branch: never push it, or merge or cherry-pick from it.

## Beside this plan

`look-round-4` starts after this draft lands and re-records the same journey on its own Home, so
land quickly: no refactor, no wider repair. What you see that is outside your task goes in your
report's `## Left`.
