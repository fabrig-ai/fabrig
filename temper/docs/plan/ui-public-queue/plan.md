# ui-public-queue

The public story reads a queued change on Landings. A red's repair, `red-56`. The request: `inputs/request.md`.

## Parts and tasks

- **`public-queue`:** one task, `landings-queue`. It needs nothing.

The critical path is that task.

## Decided here

- **The page is right; the check moves and widens.** No product change. The forms the page takes become `Tier::public_landings_reads` in the tier's library, and the story calls it.
- **A table in the focused suite, because the story can't choose its read.** Which form a run reads depends on whether the change is still queued, so a run that passes may never see a queued member, and none can show that another line still fails. The red's own text is a row. The cost is one table test of string checks in a library whose unit tests already run in focused.
- **The forms are the ones the page draws, read from `temper-client-view/src/public/forge.rs` and `words.rs`:**
  - The request's "a phase in words" is the literal `Ready`, the only one a queue's line draws (`Word::PublicReadySince`).
  - The time is `just now`, `a minute ago` or `N minutes ago` (`Times::public_time`). There is no `1 minute ago`.
  - Hours and days are refused: the tier's `global-timeout` is 120 s, so no change in this story has waited that long.
  - A bare `Queue` is refused too: the engine holds a public queue to be nonempty (`queue_before`'s `expect` in `temper-engine-domain-forge/src/public/landings.rs`). That comes from reading that line, not from following `queue_ready` through.
- **The one browser run.** The lead's rules have a spec name no slow-tier run, and the request's standing rules say no tier's story runs. The request's own task asks for this story by name, alone, through the wrapper, and `AGENTS.md`, 5, lets a red's repair do that. So the spec names it, once or twice, and nothing else in a browser. It needs the runner: the story takes the bundles and prerequisite binaries only `tools/tiers/run` builds, and `TIERS_WRAPPER=heavy` puts `heavy` before each build and nextest run.
- **A failure elsewhere in the story isn't widened.** `browser.status.md`, 8.4, already records an intermittent missing rectangle in this story. The spec reports such a failure under `## Left` and changes no other check.
- **No notes.** One task has nothing to share or contract across: its spec cites the request's standing rules.

## Overlaps

- **`temper/tests/client-browser/src/lib.rs`, avoided:** `browser-cap` adds lines beside its `pub use` list, and a new export there would meet them. So the check is an associated function in `public.rs`'s `impl Tier`, and `lib.rs` is not touched.
- **`temper/docs/design/client/browser.md`, unavoidable:** this plan changes one sentence of 8.4. `browser-cap` and `model-catalogue-2/models-proof` edit the file too. `browser-cap`'s hunks end at its line 1363, well above 8.4. `models-proof` hasn't pushed, so its hunks are unknown.
- **`temper/docs/design/client/browser.status.md`, net nothing:** the 8.4 entry this plan adds, the task removes. `browser-cap` edits lines further down.
- **`temper/tests/client-browser/src/public.rs`, possible:** `model-catalogue-2/process-fixtures` owns `temper/tests/client-browser/src/**` and hasn't pushed. This task only adds to the file.
- **`browser_public.rs`:** no other draft or trunk changes it.
