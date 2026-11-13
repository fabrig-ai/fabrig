# known-reds-merge: the merge job settles `.temper/known-reds.md`

Every goal edits `.temper/known-reds.md` (`tools/pre-pr/design.md`, 5): it writes its own prose at the top and adds the entries its changes inherited, and each merge of main brings main's entries back. So the file conflicts at nearly every merge of main into a goal's branch, and the project's merge job (`.temper/merge`, `tools/merge/README.md`) leaves it to a resolution, an agent's task. In one eight-hour window it was in 10 of 11 conflicts with main, the only file in 5 of them, and one resolution stopped to ask what to do when the union of both sides passed the file's cap of 64 entries and 8,192 bytes. The answer was to take main's version. This request makes the merge job settle the file itself. A later draft replaces the file; this is the stopgap until then. **Run, unless marked read.**

## What's wanted, ranked. Drop from the bottom if it passes two tasks, and say what you dropped

1. **A rule in the merge job for `.temper/known-reds.md`,** beside its baselines and recordings. Entries are the file's `- test …` and `- lint …` lines as `tools/pre-pr/design.md`, 5 reads them (trailing whitespace stripped, matched exactly); every other line is prose. With the merge base, the head's side and the incoming side read from the index's stages:
   - the result is the incoming side's text, its prose and its entries in its order;
   - less each entry the head's side removed since the merge base (in the base and still on the incoming side, not on the head's);
   - plus each entry the head's side added since the merge base and the incoming side lacks, after the incoming side's last entry, in the head's order, each key once;
   - the head's prose is dropped: the delivery check ignores prose.

   A file missing on one side, or a result past the cap of `tools/pre-pr/design.md`, 5, is left unmerged, with the conflicted bytes restored, so the resolution takes it as today; the job's result says which and why. A settled file is staged like a recount. The README says so.
2. **Its tests,** in `tools/merge/test_merge.py`'s scratch repositories, failing first: a conflict settled with entries added on each side and one removed on each; the head's prose dropped; an entry both sides added kept once; a result past the cap left unmerged; a deletion on one side left unmerged; a merge with this file and another conflicted file settles this one and leaves the other to the resolution.

## Not in this plan

The delivery check, the cap, the file's format, the charters' words about it, and the draft that replaces the file. Any other conflicted file.

**Tests stay cheap:** a new case of one behaviour is a row of an existing table test. Failing tests first. **A plan runs the focused and fuzzy suites alone and is complete when it lands (`AGENTS.md`, 5).** A stored format or protocol change is a question first. `walk = false`.
