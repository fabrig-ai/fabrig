# Request: the outward check becomes absolute

From survey/agents-refs, 2026-11-04; the user approved the plan (step 12 of 12, the last). fabrig now
names nothing of whoever runs its development outside plan directories: the inventory
`tests/boundary/outward.txt` is empty. This step removes the inventory, so the rule is absolute.
One direct task.

## Changes
1. `tests/boundary/tests/outward.rs`: any match fails. Remove the inventory, its counts, the
   `# Raised:` handling and the "fewer is green" logic, and delete `tests/boundary/outward.txt`.
   The failure message lists each file, line and match. Keep the matcher and its unit tests.
2. The exclusion stays for one thing only: directories `*/docs/plan/<draft>/`, described in the
   module doc as the transient run files of whoever runs a plan; each leaves main when its plan
   retires. Nothing else is excluded except the test's own file and binary files.
3. `.config/gate/README.md`: the outward check's entry says the rule is absolute and has no inventory.
4. `smith/docs/design/shell.md` ~l.1091: "the dogfood's polls" says the same without the word.
5. If anything else in the repository cites `outward.txt` or the inventory, fix it.

## Bounds
- `tools/citations/citations.py --all` at 0. The gate's outward test green at the base.
- If a file holds a match at your base, clean that file in this task rather than excepting it.
