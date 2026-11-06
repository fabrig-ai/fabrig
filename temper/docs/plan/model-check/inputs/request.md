# model-check: a background check of an account, shown in the side pane

The user reviewed the look and said "go for it", 2026-11-06. The spec is `temper/docs/design/draft/model-check.md`, as revised: read it whole. The look request `model-check-pane` lands with this build: it's fabrig/ui's by the look rule, and this plan takes it by catch-up, or builds what it needs, as you and that plan's overlaps settle.

In short: once an account is usable, one real turn goes through the path a chat takes. **The welcome doesn't wait:** "I've signed in", a sign-in's end or a key added goes on to Ready at once, and the host's check and the trial go on behind it. So the check's state belongs to the account, and leaving a page doesn't stop it.

- It shows in the side pane, beside the forge's line and drawn as it is. The status box in the panel's place that `look-round-9` landed goes, on the welcome and on `ConnectModel`, where the account's row holds the same lines.
- Ready's checklist goes; its lines move to the pane.
- Ready counts an account just declared as usable, for the heading and Create, until its host's check says otherwise, so the page doesn't flip from "Almost ready" to "Ready" after 300 ms.
- "Continue" and "Continue anyway" go. If only the turn failed, the page stays Ready, with "Try again" in the pane. If the account isn't usable, it's Almost ready, with the reason in the pane. If the person has already left, Home says "Claude didn't answer when tried".
- **The trial deliberately names no model,** so that it selects as a new chat does. It rests on `first-usable-model`: read how that landed, and don't work around it.

## What the user met

At `20d7950e4e`, a flicker after "I've signed in": *The try*'s box was on screen for 305 ms, with a different layout before and after. Ready came from the host's `claude auth status` alone, and no model answered: "validate that claude is actually working via smith" isn't there yet. A test holds that no such flash can come back: the welcome's layout doesn't change for the check's sake.

## What the plan must hold

- **Spend:** one real turn costs a little of the person's plan: say what, and when it runs again. It runs once per account's first being usable and on "Try again", unless the draft says otherwise. The draft's limits win.
- **A hang isn't a step:** the trial has a time bound, and a run that can't start, for any wait `first-usable-model` names, says so at once in that wait's words rather than running out its bound.
- **Tests:** failing first, lowest tier: success; each step's failure, naming the step; no model named; a person who's left sees it on Home; a restart doesn't spend a second turn without cause. A real turn is only for a walk; a test uses the fake LLM, and no test reaches a real host. `walk = true`: on a real box, once with a good account.

## Size

As it needs. fabrig/ui has priority on `temper-client-*`: keep hunks small and name each overlap. Budgets and timeouts are the user's (skein's `docs/foundation/testing-strategy.md`, 8.1). Formats change in place before v0.1.0.
