# landing-contract: plan

Request (`inputs/request.md`, step 7 of 12 of a user-approved plan): fabrig states the checks it
asks of a landing as its own contract, naming no one who runs them. Wording and documentation
only; no check, list, profile or script changes what it does.

## Parts and tasks

- **The design, written by the lead** in the first push (no task): `tools/tiers/design.md` gains
  section 10, a caller's interface (the commands; `cpu.py`'s `budgets`, `prepare`, `execute`,
  `summary` and `print_summary` with today's arguments, results and exceptions; `summary`'s
  result keys, `unobserved` among them; the files a caller reads after a run); 7.1, 7.5 and 8
  say "a caller" for the queue, ops and the after-landing runner; 8's paragraph on another
  repository's reader of the summary is gone, its contract kept as 10.2's `unobserved` keys.
  `smith/docs/design/testing.md` (1, 2.3, 9) and `temper/docs/design/domain/testing.md` (2.7)
  say "the after-landing tier" and its named sign-ins for the pipeline and its shelf;
  `skein/docs/design/testing.status.md`, `smith/docs/design/shell.status.md` say "the gate's
  wasm32 list", and `temper/docs/design/domain/testing.status.md` drops its runner's name.
- **landing-contract** (slice 1): `tasks/gate-readme.md`, no needs, released in the first and only
  push. `.config/gate/README.md`, the page of request section 1; `.config/gate/wasm32.txt:1` and
  the comment above `[profile.live-temper]` reworded; `tests/boundary/outward.txt` lowered for
  the six files this plan cleans (five dropped, `tools/tiers/design.md` 8 to 3).

Critical path: the one task, ~50 implementer-minutes, then fab's merge and trunk check.

## Decisions

- **The lead writes the design itself.** It is small, and implementers never edit `docs/design/`;
  the new section documents `cpu.py` as it stands (read whole at the cut), so the status of
  `tools/tiers/design.md` stays "No gap". Section 10 follows 9, the numbering kept.
- **One task, not two.** With the design written, what is left is one page, two comment lines
  and one inventory, which a reviewer reads together.
- **The after-landing tier names two profiles**, `live-smith` and `live-temper`: the only ones
  fabrig's design places after every landing (smith's `testing.md`, 2.3; temper's
  `domain/testing.md`, 2.7). The other slow tiers are listed as run through `tools/tiers/run`,
  outside the gate, without a schedule no design gives them.
- **Commands are plain `cargo` and `python3 -I -B`;** the page says a caller may wrap them in its
  build command. Its citations are not read by `tools/citations/` (`.config/` is outside its
  scope), so the task checks each cited heading by `grep`.
- **No reader to update.** No test reads the design text this plan changes: `smith-bench` reads
  only the headings its guards cite, and none cites `testing.md` or `shell.status.md`;
  `.config/nextest.toml` and `budgets.toml` are read as parsed TOML, so a comment changes nothing
  (the task proves the parse equal). The gate's run on a `smith/docs/design/**` change
  (`read-by-tests.txt`) is the trunk check's.
- **Left as they are:** `tools/citations/README.md` and `citations.py` name the plans' owner and
  another repository as examples, not a runner of a check, and the outward-citations draft owns
  them; `tools/tiers/budgets.toml`'s first line cites a decision record outside fabrig, not a
  runner. Both are for a later step of the same plan. The `heavy` names `tools/tiers/` keeps are
  the build wrapper, which the request keeps until its later step.

## Overlaps with the drafts beside this one

- **playground-browser** (integration, as its remote ref held it at 2026-11-03 23:25Z) rewrites
  the lines of `temper/docs/design/client/browser.status.md` and
  `temper/docs/design/playground.status.md` that say "fab's gate list"; its `status-nine` task,
  open, owns the second. This plan leaves both files alone to avoid a conflict: the browser one
  loses its line there; `playground.status.md` keeps "fab's gate list" in playground-browser's
  version too, for the next step to reword. Its `tools/tiers/design.md` hunk (section 2) and its
  `design.status.md` change do not meet this plan's hunks (7.1, 7.5, 8, 10; the status file
  untouched); its `wasm32.txt` line 7 does not meet line 1.
- **public-reading-build** adds lines to `browser.status.md`, which this plan does not touch.
- **outward-citations** owns `tools/citations/`, which this plan does not touch.
- **`tests/boundary/outward.txt`** is lowered by every step of the outward plan. A parallel step
  lowering `AGENTS.md` or `DESIGN.md` (lines 6 and 7) meets this task's dropped line 5
  (`.config/gate/wasm32.txt`) in adjacent hunks; the merge takes both sides' entries at the lower
  count.
- **v0.1.0-ladder, sign-in-wait, reference-walks** had no trunk; any of them may touch
  `temper/docs/design/domain/testing.md`, whose 2.7 paragraph this plan changes.
