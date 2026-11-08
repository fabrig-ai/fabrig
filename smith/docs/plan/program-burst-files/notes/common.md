# program-burst-files: common rules

Every task of this plan follows these, beside fab's implementer text, `AGENTS.md` and `smith/AGENTS.md`.

- **The request:** `smith/docs/plan/program-burst-files/inputs/request.md`. Its tests stay cheap: a new case of one behaviour is a row of an existing table test, on a scripted program (`smith-fake-claude`) and a fake model; no test reaches a real host or a real LLM.
- **The design:** `smith/docs/design/protocol/programs.md`, 5.1, the bullets *Each program's files are its own* and *An answer not asked for* (written by this cut). Read only.
- **Failing test first:** each task writes its rows, runs them on its base and sees them fail for the reason its spec names, before its fix; the report quotes that failure. This is the rows' bite: no separate break and restore is needed for them.
- **Runs:** focused only (the `default` profile), through `heavy`, as the task names them. No slow tier, no walk, no throwaway driver.
- **Status:** each task deletes its own **Not yet** entry, named in its spec, from `smith/docs/design/protocol/programs.status.md`: a stated departure from the implementer text's "Not yours: any `docs/design/`", for that entry alone. `smith/docs/design/**` is read by tests (`.config/gate/read-by-tests.txt`): no other design line changes.
- **Not to touch:** any other `docs/design/` file; `smith/docs/design/STATUS.md`; the other trees; the program backend's stored formats and the agent's wire (a change there is a question).
- **Before the push:** `cargo fmt`; `python3 -I -B smith/scripts/old_forms.py`; `python3 -I -B tools/citations/citations.py --base origin/main`.
