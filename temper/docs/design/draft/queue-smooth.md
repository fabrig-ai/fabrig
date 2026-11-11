# queue-smooth: a landing by queue never waits silently, keeps its turn, and runs again what the runner broke

A project that lands by queue (`temper/docs/design/domain/forge/queues.md`) lands one change at a time: the first is brought up to date with main, CI runs on its new head, and it merges. Three faults make that rough at ten or more goals in flight, several times a day. A landing's own update, its catch-up merge of main, or its goal's close-out merged into the goal branch just before, is read as someone else moving the branch: the landing either sits at "updating" with no hold, for up to two hours, or is held as "moved". The next change takes the queue while the first is still updating, so their CI runs go stale and run again. And a CI failure the runner caused starts a repair, so a goal's agents work on a failure that isn't theirs. **Run, unless marked read.**

## What's wanted, ranked. Drop from the bottom if it passes four tasks, and say what you dropped

1. **A landing's own update is its own.** When the change's update merges main into its head, and when its goal's close-out has just merged into the branch, the change records the head it made as its own and goes on to CI on it. Only a head moved by someone else holds the change, with a hold that names the head it expected and the one it found, as today. And no landing waits silently: a step that makes no progress past a bound raises a hold that names the step and what it waits on. Say which bound, from the existing limits if one fits. Stories in the forge world: an update that lands goes on to CI and merges; a push from outside during the update holds with its reason; a stalled step holds after its bound.
2. **The first keeps the queue** through its update, CI on its new head and its merge, as `queues.md` says ("It keeps its turn through a clean update, CI on its new head and its merge"). Today the code admits the next change as first while the first is updating, checking or landing, and `changes_document_keeps_the_first_while_it_updates` asserts that. The code and that test follow `queues.md`.
3. **A runner's fault runs again:** `temper/docs/design/draft/ci-on-forgejo-17.md`, section 2, for a change's CI: which failures are the runner's (2.1) and the bounded re-run (2.2), with its `reruns` bound in the configuration. A failure inside a workflow's own step stays the change's, by 2.1.
4. **The design says so:** `temper/docs/design/domain/forge/README.md` and `queues.md`, with their status files, and `ci-on-forgejo-17.status.md`'s section 2 lags close.

## Not in this plan

Landing by turn, stacks or batches. What the project's CI runs. The forge tables' bounds.

**Tests stay cheap:** a new case of one behaviour is a row of an existing table test. Failing tests first. **A plan runs the focused and fuzzy suites alone and is complete when it lands (`AGENTS.md`, 5).** A stored format or protocol change is a question first. `walk = false`.
