# temper-program-child: common rules

Every task of this plan follows these, beside fab's implementer text, `AGENTS.md` and its tree's `AGENTS.md`.

- **The request:** `temper/docs/plan/temper-program-child/inputs/request.md`. Its tests stay cheap: no test reaches a real host or a real LLM; the scripted program is a shell script in the test's scratch directory (`notes/contracts.md`, 2).
- **Failing test first:** write the table test, run it on the task's base and see the rows `notes/contracts.md`, 2 names fail for the reason it gives (the binary's own parser refusing `--program-child`); then the dispatch; then the same run passes. The report quotes the failure seen. This is the new test's bite: no separate break and restore is needed.
- **Runs:** focused only, through `heavy`, as the task names them. No slow tier, no walk, no throwaway driver.
- **Not to touch:** any smith crate or smith design file (`smith/docs/design/**` is read by tests); `temper/docs/design/STATUS.md` and `jig/docs/design/STATUS.md` (their indexes already list the status files this plan changes); the other tree's files.
- **Status:** each task deletes its own **Not yet** entry, named in its spec, from the status file of the design section it builds: a stated departure from the implementer text's "Not yours: any `docs/design/`", for that entry alone.
- **Before the push:** `cargo fmt`; `python3 -I -B <tree>/scripts/old_forms.py` for the tree touched; `python3 -I -B tools/citations/citations.py --base origin/main`.
