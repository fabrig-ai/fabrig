# Request: the lab's names leave fabrig's documents, scripts and tests

From survey/agents-refs, 2026-11-04; the user approved the plan (step 9 of 12). fabrig names nothing of
whoever runs its development. `tests/boundary/outward.txt` (the inventory of
`tests/boundary/tests/outward.rs`) lists the files that still do: a build tool's command form
(`heavy cargo ...`), machine names (hel8, hetzner*), a home directory (`/home/free/`), scratch paths
(`~/src/rust/tmp/...`, `~/src/fabrig/...`), session names, the pipeline's name. This step cleans
every listed file except those other steps own. One task per tree (skein, smith, jig, temper,
tools/citations), independent.

## Files (the inventory at main 444de312a3; take the inventory at your base)
Every file listed in `tests/boundary/outward.txt` **except**: `tools/tiers/**` (a parallel step);
`temper/docs/development/qa.md`, `v0.1.0-ladder.md`, `reference-walks.md` (a later step moves them);
`temper/docs/design/draft/notes.md`, `reference-case.execution.md`, `configuring.execution.md`,
`dashboards.md` (a later step); anything under `docs/plan/`.

## How
- A command shown as `heavy cargo ...` or `heavy env ...` (READMEs, test hint strings such as
  `drift.rs` "regenerate with ...", scripts' comments) becomes the plain command.
- Scripts that call the build tool (`temper/scripts/proof-live.sh`, `proof-live-github*.sh`,
  `jig/scripts/live-sign-in.sh`, `temper/tests/real-loop/tests/fixtures/{b0,b1}/regenerate.sh`) run
  the plain command; a wrapper, if a caller wants one, comes from the environment the same way as
  the tiers' `TIERS_WRAPPER` (`tools/tiers/design.md` 3), or not at all. `regenerate.sh` takes its
  data directory as a required argument, with no default path. Tests that stub the tool
  (`temper/crates/temper/tests/suite/forge_land.rs` ~l.478, 665) follow the script.
- Machine names, home directories and scratch paths in prose, messages and code comments are said
  generically ("a builder", "the scratch directory", `~/.local/state/...` → a relative or
  placeholder path). Hard-coded strings in code or tests (`startup.rs`, `cluster.rs`, the browser
  test shells' `lib.rs`) become neutral.
- Recorded fixtures (`smith/benchmarks/tests/recorded/codex/*.jsonl`, `manifest.json`): replace the
  home directory in recorded paths with a neutral one if the tests and the recording's hashes allow
  (say how you checked); otherwise keep them, each with a `# Raised:`-style reason line in the
  inventory explaining it is recorded data.
- Dated reviews and debt items (`*/docs/development/debt/reviews/*.md`, `D008-...md`) keep their
  findings; only the lab's names change (a machine is "the review machine", the pipeline is "the
  former pipeline").
- `tools/citations/README.md` ~l.27: "a runner's records of a change".

## Bounds
- Behaviour unchanged except the scripts' wrapper (plain by default); tests pass.
- Each cleaned file's entry leaves `tests/boundary/outward.txt` (or is lowered, with the reason).
- `tools/citations/citations.py --all` at 0.
