# Request: fabrig's landing contract, in its own words

From survey/agents-refs, 2026-11-03; the user approved the plan (step 7 of 12). fabrig is to name
nothing of whoever runs its development. Today the checks fabrig asks of a landing are described as
another system's steps ("the queue", "the pipeline", "the after-landing runner", "ops", named
gate steps), and the tiers' design has a paragraph about a consumer's code. This step states them
as fabrig's own contract, with no knowledge of who runs them. Wording and documentation only: no
check, list, profile or script changes behaviour. One or two direct tasks.

## 1. One page: what a branch passes to land
`.config/gate/README.md` (new). For each check, what it is, its command, and where its rule lives
(cite the design section; do not copy it):
- the format check; clippy, warnings denied, and the wasm32 clippy of the packages in
  `.config/gate/wasm32.txt`;
- the gate's nextest profiles and what `always.txt` and `read-by-tests.txt` mean (format, who edits
  them, what adding a line does);
- the old-forms counts (`<tree>/scripts/old_forms.py`, counts only fall);
- `tools/citations/citations.py --all` at 0;
- the outward check (`tests/boundary/tests/outward.rs`, its inventory);
- the slow tiers (`tools/tiers/run`; `tools/tiers/design.md`) and the after-landing tier: which
  profiles run after each landing, what they need as inputs (sign-ins as named inputs of the tier,
  not someone's shelf), what red means;
- the `.temper/` hooks (`merge`, `pre-pr`) and `.config/fixture-readers.toml`.
It says "a landing check", "the gate", "a caller", never who runs them.

## 2. tools/tiers/design.md: the caller's interface
- A section for a caller of `tools/tiers/cpu.py`: `summary(directory, ..., profile)`, `prepare`,
  `execute`, their arguments and results, the summary's `unobserved` entries, and the files a
  caller may read after a run. Keep today's names and behaviour exactly (a later step changes the
  build wrapper); this section is the contract that step will keep.
- Rephrase ~l.174-183, 460-474, 494 ("the queue", "Ops builds the wrapper", "Gate integration ...
  belong to ops", "the after-landing runner") as "a caller".
- Drop ~l.497-507, the paragraph describing another repository's reader of the summary; keep the
  `unobserved` keys as the contract.

## 3. Reword where a check is named after its runner
- `.config/gate/wasm32.txt:1` ("fab's clippy-wasm32 step") → the gate's wasm32 clippy (section 1).
- `.config/nextest.toml` ~l.159; `smith/docs/design/testing.md` ~l.45, 241, 709 ("after every
  landing, the pipeline runs ...", "the logins it lends from its shelf") → the after-landing tier
  as fabrig defines it.
- `temper/docs/design/domain/testing.md` ~l.1368 ("fab's shelf of logins ...").
- status files saying "fab's gate list" or similar: `skein/docs/design/testing.status.md` ~l.8,
  `smith/docs/design/shell.status.md` ~l.6, `temper/docs/design/client/playground.status.md` ~l.29,
  `temper/docs/design/client/browser.status.md` ~l.9 → "the gate's wasm32 list".
- Find any other place in `tools/` or `.config/` that names a runner of a check and reword it the
  same way. Not here: root `AGENTS.md`, `DESIGN.md`, `tools/old_forms/README.md` and the old-forms
  baselines (a parallel step owns them).

## Bounds
- No behaviour change; tests pass unchanged.
- `tools/citations/citations.py --all` at 0.
- `tests/boundary/outward.txt`: lower the entry of each file this cleans (counts only fall).
