# A resolution sees what a merge deleted

Draft, 2026-11-15, on the user's word. On 11-14, a goal deleted `.temper/known-reds.md` from main. Another goal's landing then merged main into its own branch, which had only appended entries to that file. The merge stopped on a modify/delete conflict and left the file to a resolution. The resolution's words said to preserve both sides' content and never remove content to pass a check, so the agent kept all 54 lines, and the landing put the deleted file back on main (P53).

## 1. The merge job names each left file's conflict

- **Today:** the reference merge helper reads each unmerged path's index stages, modes and blobs (`tools/merge/merge.py`, `unmerged`), then reports paths alone: exit 3 with `{"settled":[PATH],"left":[PATH]}`, at most 512 bytes, with `more` for what was cut (`result_bytes`; `temper/docs/design/protocol/tooling.md`, the merge job; `temper/docs/design/domain/worker/jobs.md`). Whether a file was modified on both sides, deleted on one, added on both or renamed is lost before the resolution starts.
- **Wanted:**
  - Each left file's record carries its path, its kind (both modified, deleted on the base, deleted on the head, added on both, a rename on either side), and, for a deletion, the deleting side and the commit that deleted it. The head is the change's branch, and the base is what's merged into it. The deleting commit is the commit that removed the file, which is not always the base's tip.
  - A rename is told apart from a deletion by the sides' history, not by a missing stage alone. Where the helper can't tell, the record says unknown rather than guessing.
  - The result stays bounded. Records are whole or omitted, never cut mid-record, and what was omitted is named, with how to read it from the checkout (`git status`, `git log --diff-filter=D`, for example).
  - The worker still carries the result opaquely, so jig gains no Git policy. Revise `tooling.md` and `jobs.md` for the new result shape, moving its version by the upgrade rules if any reader parses it.

## 2. The resolution's words cover a deleted file

- **Today:** `resolution_words` (`temper/crates/temper-engine-domain/src/route/forge_route.rs`) tells the agent to preserve the requested change and the base's unrelated content, and never to remove content to make a check pass. Its compact form says "Keep both sides and blank lines". The merge result follows as optional text, and may be cut or left out. The stock implementer charter repeats the preservation rule (`temper/docs/charters/stock/implementer.md`). Nothing in either says what to do with a file one side deleted, and `temper/docs/design/domain/forge/change/repairs.md` (the resolution's instructions) doesn't either.
- **Wanted:**
  - In the required words, in both the full and the compact form, and with or without a merge job's result: a file deleted on the base stays deleted unless the requested change needs it. A deletion on the head, or a rename on either side, is an intended change too. The agent reads the deletion's commit before it restores anything.
  - The resolution's result says, for each such file, whether it kept the deletion or restored the file, and why.
  - Keeping a deliberate deletion doesn't break the preservation rule, so reword the rule so the two agree. Make the same change to the implementer charter's line.
  - The judgement stays with the agent. The merge job settles no modify/delete conflict itself, since whether the change still needs the file is a judgement.
  - Revise `repairs.md` (the resolution's instructions) and the update path in `temper/docs/design/domain/forge/README.md`, 6, to say so.

## 3. Out

- Deleting `.temper/known-reds.md` from main again: a docs branch does that, on the user's word.
- Any other merge strategy.

## 4. Tests

Cheap: focused and fuzzy suites only; walk = false.
- `tools/merge/test_merge.py`:
  - each kind of left file named, with both deleting sides and the deleting commit;
  - a deletion older than the base's tip;
  - a rename told from a deletion;
  - a path that needs escaping;
  - a result over its bound, which omits whole records and names them.
  - Each leaves the unresolved files and the index as they were.
- Focused tests of `resolution_words` (`temper/crates/temper-engine-domain/src/route/forge_route/tests.rs`): the deletion rule appears in the full and compact forms, with and without a merge result, and survives truncation.
- A focused story (`temper/tests/engine/domain/tests/suite/forge_routes/merge_job.rs`): a merge job leaving a file deleted on the base reaches the resolver with its kind and deleting commit.
- The stock charter tests (`temper/crates/temper-engine-service-stock/src/tests/stock.rs`) pass with the revised implementer line.

Run only the new and affected tests, by name.
