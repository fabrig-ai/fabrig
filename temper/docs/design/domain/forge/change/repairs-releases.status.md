# repairs-releases: status

2026-11-26. Where the tree's code lags or departs from [repairs-releases.md](repairs-releases.md).

- **Not yet:** 1, *Bounded apart*: CI whose runner's faults outlast `reruns` asks a repair; no `Runner` hold [in flight: change 18912].

- **Design departs:** 1, *A refusal where nothing it judged has moved*: `../../../draft/whose-red.md`, 4.2 checks a released refusal again when the goal's branch's tip, main's tip or the published counts it judged against have changed, as the request of 2026-11-24 did. The engine reads no counts, since `refs/temper/counts` reaches only a job's checkout (`../../../protocol/git-operations.md`, 1.3), so main's tip stands for its published counts, which follow its landings: counts published for main's tip after a close refused leave the release holding until the goal's branch or main moves. The refusal's counts are kept and named in its words. A close whose counts aren't there yet breaks instead (`tools/pre-pr/design.md`, 4.1). Decided by the goal's designer, 2026-11-24, and kept by the requester of whose-red's first goal, 2026-11-25, over a forge read of the ref.
- **Not yet:** "A release", under `turn`: `waiting_producer` prepares the released head against the saved main tip. Its fresh snapshot carries the pull's base tip, which can be a goal branch, not main; a fresh main tip is not at hand at this decision (`temper-engine-domain-forge-change/src/domain.rs`, `waiting_producer`; `temper-engine-domain-forge/src/domain.rs`, `step_read`).

- **Not yet:**
  - the whole document: a no-change on a change under another hold stays on the row, but the
    hold that stands does not say its words: `hold_change_decision`
    (`temper-engine-domain/src/route/forge_route.rs`) reads them for
    `Hold::Failed` alone, and they are in no panel. A release reads the result
    as a failed producer's, so its words first reach a person in the `Failed`
    hold that follows, and the change does not end. Seen by reading: the step
    test stops at the row
    (`a_questions_hold_keeps_the_result_until_release_and_says_no_change_words`).

- **Not yet:**
  - the whole document: a result ending a stalled hold has no world: a live delegate does not
    stall, so `temper-engine-domain-forge/src/tests/change.rs` gives the held
    row by `Event::Change`, and only a green head's worlds run the release its
    `EndHold` asks for
    (`temper/tests/engine/domain/tests/suite/forge_routes/repair_green.rs`).

- **Not yet:**
  - the whole document: the request `parallel-faults` of 2026-11-06 has its proof in a rerun of
    its parallel replay, and no record shows one, so no real run has shown a
    producer working past the stall or a stall's hold reaching anyone. It also
    asks why the replay's hold came about 57 minutes in, where the stall is 30,
    and that stays unknown: in a world the clock starts at the producer's
    request, after any wait for a dependency, and a hold shows only when a step
    runs, but no deployment's times were read. So is whether that row held with
    its task not, by the path `change_decision` now closes: a world shows that
    drop only by delivering the decision out of step, though the empty inbox,
    and `task show`'s "Waiting for news", which jig answers of a task not held
    (`jig/crates/jig-domain-core/src/runs/waits.rs`), fit it.

- **Not yet:**
  - the whole document: `change show`'s `held:` line for a cancelled delegate reads `a
    delegate or a write failed for good`: the panel's hold is the client
    wire's `failed` (`temper-client-protocol-wire/schema/client-wire-v1.schema`,
    `Hold`), and its words are the client's
    (`temper-client-domain/src/forge/words.rs`, `FailedHold`), and the plan
    `first-goal-fixes` edits neither, the ui drafts holding them, so a cancel
    is said by `task show` and not by this line.
