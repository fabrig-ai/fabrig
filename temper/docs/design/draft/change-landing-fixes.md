# Three faults in a change's way to landing

Draft, 2026-11-13. Three faults seen on a deployment the same day, each in
the change procedure's way from a producer's result to a landed pull
request. Each wants its cause found, then fixed, with a focused test that
reproduces it first.

## 1. A delivered change that never moves on

- **Seen:** a change into a goal branch sat at `producing` after its
  producer ended with its result (a pushed head, "Done: pushed …"). The
  change opened no pull request, waited on news, and was never held: the
  stall bound (`../domain/forge/change/repairs.md`, *Stalls*) passed
  without a hold. A party's stop and release of its task changed nothing.
  Six sibling changes that depended on it waited until the goal cancelled
  it and cut a replacement from its branch's head.
- **What was particular:** its producer had asked a question earlier, which
  the change passed up and its goal answered (`../domain/forge/change/questions.md`); and
  a sibling change into the same goal branch landed while the producer
  worked, so the delivered head did not contain the goal branch's tip.
- **Wanted, ranked:**
  1. Find why the result didn't move the change on, and fix it: a
     producer's result with a head always takes the change to its next
     step (opening its pull request, or updating it with its base).
  2. Whatever a change waits on at a step, either its stall counts or a
     named exemption applies (*Stalls*): a change that stops moving with
     no exemption is held, never left silent.
  3. A release re-reads what the change was waiting for, so a release
     moves a change whose result had already arrived.

## 2. A head already in its base is retried as a merge

- **Seen:** a goal's catch-up pull request (main into the goal branch, about
  7,000 files) failed its forge merge three times. Its merge was then made
  by hand: the goal branch was fast-forwarded to a merge commit holding the
  pull request's head. The forge left the pull request open, since it
  marks no manual merge, and the change kept calling the forge's merge
  until held again.
- **Wanted:** at landing, and before any merge call, a change reads
  whether its head is already contained in its base. If it is, the change
  has landed: the base's commit holding it is its merge, and no merge call
  is made. The same reading ends a retry loop.

## 3. A drift hold with the same commit on both sides

- **Seen:** a change was held with the words `its branch moved: expected
  290a9d834ca0, found 290a9d834ca0` (`temper/crates/temper-engine-domain/src/route/forge_route/holds/change.rs`,
  `account`, the branch-moved case). The words name the same commit twice,
  so the drift that held it was false: perhaps a full id compared with a
  short one, or a stale read compared with a fresh one.
- **Wanted:** find the comparison that held and fix it, so a branch whose
  head equals the expected head never holds; the words keep naming both
  heads.

## 4. Out

- The stall bound and every other step's waits.
- How the forge marks merges; manual-merge detection on the forge.

## 5. Tests

Cheap, focused and fuzzy suites only; walk = false. For each fault, a
focused test that fails before the fix, in temper's engine domain stories
for forge routes, beside the stall, question and landing stories. Run only
the new and affected tests by name.

## 6. Formats

If a fix needs a field no store keeps today, the store's version moves
and its patch comes with it: upgrades are supported (`redeploys.md`).
