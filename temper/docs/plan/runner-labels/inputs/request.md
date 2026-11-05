# runner-labels: a re-prepare doesn't strand a runner on another host

Found on a real deployment on 2026-11-05 at `f9955bf`. On a Forgejo whose runner runs elsewhere (`application.machine.owned.runner` set, the runner on another host), `temper forge prepare` at each start rewrites the `action_run` row: `agent_labels` to the raw `["NAME:host"]`, with an empty version. The runner, already declared, doesn't declare again. So jobs asking `runs-on: NAME` wait forever with no error: status waiting, the runner's `last_online` current. Restarting the runner declares `["NAME"]` and its version, and it picks up at once.

## The task

1. **A failing test first,** at the cheapest tier that shows it: prepare's row over one a runner already declared.
2. **Prepare leaves a declared runner's labels and version alone** when its uuid matches, if that's sound by `temper/docs/design/protocol/setup.md`, 6 and 7. Read why prepare writes the row at all, and what a first preparation needs. If it can't, `temper help server`, 5 says to restart the runner after Forgejo, and the status says why.
3. A help change comes with its test (`temper/docs/help` is read by tests).

No test reaches a real host. Leave fabrig/ui's files alone. A stored format, a protocol or a public interface change is a `Question:`.

## Size

One task.
