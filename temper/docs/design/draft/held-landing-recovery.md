# A held landing recovers, and its remake starts clean

Draft, 2026-11-15, on the user's word. temper/dev builds it. On 11-14, a goal's landing change spent about three hours stuck. A resolution's merge failed its delivery checks and held the change. The goal branch then gained the fix, but each release of the hold brought back the same refusal from the failed resolution's saved head (P50). The goal cancelled the landing and made it again. The new landing took over the cancelled one's closed pull request and was held. After a person reopened it, the new landing opened a second pull request from the same branch and was held again, this time for drift (P46). Before any of this, one merge job ran 17 attempts in an hour, and its history named 16 of them only as "lost".

## 1. A release decides afresh, end to end

- **Today:**
  - `repairs.md`, *A release*, says a change released from a failed resolution or merge job reads its branch, base and pull afresh, never the ended delegate's status again. `repairs.status.md` still lists that as **Not yet**: the change keeps `Resolving { requested: true }` and holds again on the ended delegate's status (`temper/crates/temper-engine-domain-forge-change/src/domain.rs`, `released_state`).
  - The pure policy has a test, `a_released_failed_resolution_decides_from_fresh_facts_despite_the_stale_failure` (`temper/crates/temper-engine-domain-forge-change/src/tests.rs`). On a deployment's engine, change 3031 still held again at hold revisions 3 and 4, with the failed resolution's refusal from saved head eae13cae0a, after its branch had moved to 3de74e92d9, which contains eae13cae0a and the fix.
  - Main has release work that the deployment's build lacked: release revisions, archived delegate recovery, abandoned gathers and fresh-read hints, in `temper/crates/temper-engine-domain/src/route/forge_route/delegate_record.rs` and forge's `release_change`. Nothing pins the whole path through the route.
- **Wanted:**
  - A release of a change held after a failed resolution or merge job takes effect once. The release fences the gathers and reads made before it. The change reads its branch, base and pull afresh, and never takes the ended delegate's status or result again, a restart included.
  - The change keeps the head it reads as its last head. With no conflict left, it checks that head. With a conflict left, it asks exactly one fresh resolution, pinned to that head and the current base.
  - Fix whatever on main still replays the old refusal. Don't fix it by accepting a failed delivery unchecked.
  - Make `repairs.status.md` say what the code does. Clarify `repairs.md`, *A release*, and `keeping-up.md`, 3, where they leave the order of the release and the fresh read open.

## 2. A remade change owns its own pull request

- **Today:**
  - A new change has no pull request number, so its opening looks one up by branch pair (`PullFor { head, base }`). That lookup returns the open pull request, else the oldest, closed ones included (`skein/docs/design/forge/README.md`, `PullFor`).
  - temper accepts a closed match when its commit equals the change's last head. A change remade at the same head as a cancelled one therefore takes the closed pull request its predecessor's cancel closed (`temper/docs/design/domain/forge/change/README.md`, 4), and holds for `pull_closed` (`temper/crates/temper-engine-domain-forge/src/domain.rs`, the opening's pull answer and `pull_changed`).
  - After that pull request was reopened by hand and the hold released, the change opened a second pull request from the same branch. Nothing on record explains that opening.
- **Wanted:**
  - A change owns only a pull request it opened, or one whose number it stored. A branch-pair lookup never adopts a closed pull request: a closed match at opening belongs to an ended change, and the new change opens its own.
  - Once a change has a pull request, it stores the number, and reads and writes only that one, a restart included. It never opens a second pull request while it has one.
  - Find what opened the second pull request on 11-14 (#1346 beside #1292, change 3199), and cover it in the fix.
  - Revise `temper/docs/design/domain/forge/change/README.md`, 2 (opening), 3 (drift) and 4 (cancelled), and `keeping-up.md`, 3, to say so.

## 3. An ended change stays frozen

- **Today:**
  - `pull_changed` also scans ended changes' rows. It turns an ended change's pull request closing into drift on the shared branch, and that drift fans out to the goal and its live landing change. Its guard suppresses a cancel's own close only while the release row exists (`temper/crates/temper-engine-domain-forge/src/domain.rs`, `pull_changed` and `drift`).
  - `change_history::pushed` records a push on every retained row for that branch, ended ones included (`temper/crates/temper-engine-domain-forge/src/change_history.rs`). Change 3031 gained four pushes after it was cancelled.
  - `keeping-up.md`, 3, says an ended change's facts are frozen.
- **Wanted:**
  - An ended change's pull request and branch raise no drift on any live task.
  - An ended change's history gains no event after its end, apart from its verification's reads (`temper/docs/design/domain/after-landing.md`, 6).
  - Update `temper/docs/design/domain/landing-history.md`, 2.2, if it says otherwise.

## 4. A job's attempt says why it ended

- **Today:** when a new claim arrives, `job_history::heard` closes any earlier attempt that left no terminal result as `lost`, and a cancel also maps to `lost` (`temper/crates/temper-engine-domain-forge/src/job_history.rs`; `temper/crates/temper-engine-domain/src/route/forge_route/job_history.rs`). Change 3031's merge job 3107 shows 16 `lost` attempts, and nobody can tell from them whether a push timed out, a worker was lost or a run failed.
- **Wanted:** an attempt's end in the landing history names its class where the engine knows it: failed with its failure class, timed out, cancelled, or lost only when the worker was lost. The words follow `temper/docs/design/domain/landing-history.md`, 2.2. Wherever a stored or wire format carries the outcome, it moves its version by the upgrade rules.

## 5. Out

- Why merge job 3107's pushes timed out (forge push lifetimes against large merges). Section 4 makes the next case readable.
- Whether a cancel should leave its pull request open for a successor. Cancelling closes it, by design.
- Relaying answers and words to producers: `kept-change-answers.md`, and a separate draft.

## 6. Tests

Cheap: focused and fuzzy suites only; walk = false.
- A focused story in temper's engine domain tests (`temper/tests/engine/domain/tests/suite/forge_routes/`, near `resolution_head.rs`):
  1. A landing change's resolution delivers head H1, which fails its checks, and the change holds.
  2. Another change lands on the branch, giving H2, which contains H1 and passes.
  3. A party releases the hold. The change checks H2 and goes on, with no repeated refusal.
  4. Repeat with a restart between the release and the next step.
  5. Repeat with H2 conflicting with the base: exactly one fresh resolution, pinned to H2.
- A focused story:
  1. A goal's landing change A has an open pull request P, and is cancelled, which closes P.
  2. Landing change B is made on the same branch at the same head.
  3. B opens its own pull request and stores its number, with no hold.
  4. Then reopen P by hand, release, restart and push. B still reads only its own pull request, raises no drift for P, and opens no third one.
  5. A's history gains nothing after its end.
- A focused test of the job history: an attempt that failed or timed out is named so, and `lost` is used only for a lost worker.

Run only the new and affected tests, by name.
