# fake-signin-model: ChatGPT's fake sign-in leaves Ready with no model

`look-round-3` could not re-record the journey `project-landing`. Its task `record-landing` walked a fresh `temper box --fake-backends` on 2026-11-04:

1. Welcome → "Set up temper" → "Run on this machine": the Forgejo sheet, `work`, `hello`; Forgejo 17.0.1 started.
2. Model read "Forgejo on this machine", "connected as temper", "work/hello made".
3. "Continue with ChatGPT", once. After the fake sign-in's departure and return, the browser reached `/welcome?at=ready`.
4. **Ready read "Almost ready", "Model none yet · Connect a model", "Forge Forgejo on this machine · connected as temper". The name field offered `Hello`; "Create your project" was disabled. No refusal or failure text explained the absent model.**
5. It persisted. "Connect a model" again offered "Continue with ChatGPT", with no failure message.

The whole report: `/home/free/src/fabrig/fab/runs/look-round-3/record-landing-1/report.md`. Its walk logs: `~/src/fabrig/look-round-3/project-home/walk/record-landing-1/logs` on hetzner6, if the task's builder can read them; otherwise reproduce afresh.

The user recorded this path earlier on 2026-11-04 (`project-landing`, ChatGPT plan), so it may be a regression. Check that first against the journey's commit `68cddad304`, or it may be the fake's: a sign-in that returned no served model.

## The task

1. **A failing test first,** in the lowest tier that shows it: a fake box's ChatGPT sign-in leaves an account with a usable model, and Ready enables "Create your project".
2. **Find the fault and fix it** in the lowest tier (`AGENTS.md`, 5).
3. **If a sign-in really ends with no usable model,** Ready must say why, not stay silent: name the words as a `Question:` if the design has none.
4. **Then record `project-landing` again** (`temper/docs/design/client/journeys.md`, 8), walking to Home `/projects/1`, with project "Hello" and `work/hello` adopted, and reprint it. Commit it carrying `Look-request: project-home`.

## Beside it

`look-round-4` starts after this and will re-record the same journey on its own Home; so land quickly. No test reaches a real host.

## Size

One or two tasks. Budgets and timeouts are the user's (skein's `docs/foundation/testing-strategy.md`, 8.1).
