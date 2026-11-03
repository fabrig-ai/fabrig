# Request: fabrig names no outside lab (a falling inventory)

From survey/agents-refs, 2026-11-03; the user approved the plan (step 1 of 12). fabrig (skein,
smith, jig, temper, tools) is to name nothing of whoever runs its development: no repository,
tool, machine or session of that lab. A survey found ~3,220 such names, most to be removed by later
steps. This step stops new ones arriving while the old ones go. One direct part, one task.

## The check
A new test `tests/boundary/tests/outward.rs` in the `fabrig-boundary` package, beside
`direction.rs`, written as `direction.rs` is (module doc citing what it enforces; a line it cannot
read is a failure, never a skip). The package is already in every gate (`.config/gate/always.txt`).

- **What it reads:** every text file of the repository from the workspace root, skipping `target/`,
  `.git/` and other untracked build output (say how in the module doc), binary files, the test's own
  file and its inventory. **`docs/plan/` directories are left out for now** (a comment says: until
  the plans move out; a later step removes the exclusion).
- **What it fails on** (matched in context, so product words pass):
  - `fabrig-ai/agents`; "the agents repository", "agents repo", "agents' " followed by a path into
    that repository is covered by the first two where phrased so;
  - `fab` as a word (`\bfab\b`, `fab's`), `bin/fab`, `FAB_HOME`;
  - `heavy` used as a command: `heavy cargo`, `heavy env`, `heavy --…`, and `"heavy"` as an argv
    element in scripts; never the bare word (it has product meanings);
  - lab hosts and paths: `hel8`, `hetzner` followed by digits, `debian-16gb`, `status.fabrig.ai`,
    `~/src/fabrig/`, `~/src/rust/tmp/`, `/home/free/`;
  - lab session names: `fabrig/qa`, `fabrig/dogfood`, `fabrig/debt`, `agents/loop`,
    `agents/status`, `agents/creds`, `agents/ops`.
  Not on the list on purpose: a bare `director`, a bare `heavy`, a generic `/home/<user>/` (product
  meanings in temper's, jig's and smith's tests).
- **The inventory:** `tests/boundary/outward.txt`, one line per file holding any match today:
  `PATH COUNT`, sorted, `#` comments allowed. A file not listed must have none; a listed file may
  hold at most its count (fewer is green, the failure message never asks for it). Raising a count
  or adding a file needs a `# Raised: <who, why>` comment line above the entry, as the old-forms
  baselines do. On failure the message lists each offending file, line and match, and prints the
  inventory the tree would need, so a task can paste it.
- **Its own tests:** unit tests of the matcher on short strings (each pattern, and the product words
  that must pass: "tool-heavy", "the director task", "/home/<user>/", "fabric").

## Bounds
- The full scan stays well under the gate's CPU budget for the package (say the measured time in
  the commit); no new dependency.
- The generated `outward.txt` is the inventory at the task's base; `tools/citations/citations.py
  --all` stays at 0.
- Nothing else changes: no file is reworded in this step.
