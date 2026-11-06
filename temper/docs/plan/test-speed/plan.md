# test-speed: the slow tiers' bottlenecks that need no cache

The request is `inputs/request.md`. Its survey read code and logs and ran nothing, so each task measures first and its figures go in `figures/<task>.md`.

## Parts and tasks

| Part | Task | Needs | What it does |
|---|---|---|---|
| `stopped-box` | `ctl-stopped` | none | `temper ctl` says at once that nothing listens (`client/tools.md`, 6.3), ending the panicking story's 30 s |
| `walk-waits` | `walk-waits` | none | timelines from `box.log`, then each walk wakes on its event or the short paces |
| `tier-threads` | `tier-threads` | none | a fake box on the kernel's ports (`protocol/setup.md`, 7.5), `process-temper` at three threads |
| `again-setup` | `harness-horizon` | none | skein's harness may have a longer horizon, for a program that is no test (skein's `ui-web.md`, 14.2) |
| `again-setup` | `again-setup` | `harness-horizon` | a journey's `setup` (`client/journeys.md`, 8.2), and `again chat-draft` proven with no hand driver |

## Critical path

`harness-horizon` then `again-setup`: about 205 implementer-minutes. The other three stand alone, `tier-threads` the longest at about 150, most of it waiting on tier runs.

## What this cut decided

- **A person's interfaces changed, with no compatibility:**
  - `temper ctl`'s new line and exit at a refused connection (`client/tools.md`, 6.3).
  - `temper box --fake-backends` on a fresh sandbox takes the kernel's ports, where it preferred 8367 to 8371. The request says "give those walks a free port as the other tests have". A walk's first run is a person's, on empty directories, so the free port belongs in the product: a fake box then leaves the fixed ports to the real box a person starts after it. `temper init` and a person's `temper box` are unchanged.
- **`again`'s horizon is no budget:** the request forbids changing a budget, timeout or limit, and skein's harness ends a run at forty seconds, which `chat-draft`'s walk outlasts. `again` is a person's command in no tier, with its own bound on each step, so it alone sets ten minutes; every test keeps forty, and the method's comment says no test calls it. `client/journeys.md` section 10 already named this as the way.
- **A setup holds marks too:** the recording shows quiet gaps of about 10 s inside a fake model's turn, where the page reads as still at half a second (8.1), so a key mark would fall before the reply. The request's "controls by role and name, then the reload" needed one more record, `mark N TEXT`, which says what the person waited for. `chat-draft`'s pages 06, 08, 11 and 19 are the ones at risk.
- **The setup is `after`-shared:** `chat-draft/setup` names `project-landing`, and `welcome-set-up`, which has none, is tied to them by a native test against its recording, by event and not by its recorded words (its first act is named "Forge", the first control of that event, where the setup presses "Set up temper"). That test catches a renamed control at the gate, which the setup's own walk, run by hand, wouldn't.
- **Grouped, expected:** walks signing in to ChatGPT, whose redirect comes back to the fixed port 1455.

## Overlaps not avoided

- `box-small/link-hours`: `tests/process.rs`, `tests/fake_backends/box_link.rs` and `walk.rs`, small hunks on each side.
- `sign-in-new-tab/sign-in-holds`: `tests/fake_backends/walk/sign_in.rs`, `tier-threads` touching only lines that bind.
- `chat-reading-c/reading-close`: `temper/docs/design/client/journeys.md` and `journeys/**`, this plan's edits being its 5.2, 6, 8, 8.2, 9 and 10; `again-setup`'s prints are regenerated on a catch-up. `again-setup` comes last of this plan's in any case, as it needs `harness-horizon`.
- `sign-in-new-tab`: moves the sign-in `project-landing`'s setup presses, into a new tab, when it lands; `again-setup` says what to do either way.
- `walk-waits` and `tier-threads` share `tests/fake_backends/walk.rs`, each with its own hunk.
