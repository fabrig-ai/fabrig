# hearth-names-2: what every task holds

1. **No old value is spelled.** The plan's files are on main while it runs, and fabrig is public. So no spec, commit message, comment or report spells the old port, the old zone or either half of it, the forge's old port, or a machine's name from the dropped sentence. Read each from the repository into an untracked throwaway (a task's first step says how), add files by name so it can't be committed, and delete it before the push. The new values may be spelled.
2. **Nothing changes but what `hearth/docs/plan/hearth-names-2/inputs/request.md` names,** and what the task's table adds with its reason. No step code changes, no behaviour changes and no test is added, so a test's bite isn't asked for here.
3. **A test failing for any reason other than these values is a finding, not something to update:** leave it, push what you have, and stop with `Question:`, naming the test and its log.
4. **Before the push:** `python3 -I -B tools/citations/citations.py --all` reads 0, and after your commit `heavy cargo nextest run -p fabrig-boundary --test outward` passes.
