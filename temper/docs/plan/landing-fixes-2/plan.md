# landing-fixes-2: the plan

From `inputs/request.md`, what `landing-fixes` dropped. Ranked, with a ceiling of three or four tasks. Everything below was judged from reading the code on 2026-11-06. No test ran for this cut, so each task reproduces first and says if its cause is wrong.

## Parts and tasks

| Request's item | Part | Task | Needs | Minutes |
|---|---|---|---|---|
| 1, a refused worker | `hello-refusal` | `refusal-reason` (jig) | | 35 |
| 1, a refused worker | `hello-refusal` | `refusal-words` (temper) | `refusal-reason` | 70 |
| 2, a branch left after its goal | `saved-close` | `saved-close` | | 90 |
| 3, a repair's brief | `repair-cause` | `repair-cause` | | 45 |
| 4, a flake | `port-reservation` | `port-reservation` | | 45 |

**Nothing is dropped, and this is one task past the ceiling.** Item 1 needs a small jig step, since the fleet forgets a hello's figures and a branch stays in its tree. That showed only after `port-reservation`, the bottom item, had gone out under the rule to release early, so the bottom couldn't be dropped. Dropping item 3 to get back to four would put the list out of rank, and leave a small fix whose cause is already found. If four is firm, `repair-cause` is the one to stop.

**Critical path:** `refusal-reason` then `refusal-words`, about 105 implementer-minutes. `saved-close` alone is about 90.

## What was decided

- **1. Typed through jig, in words in temper.** The fleet records why (`HelloRefusalRecord`) but its `Request::Refuse` names only the channel, and the figures go with the hello.
  - **Why not read the record:** `hello_refusals()` is public, and temper could read it with no jig change, in one task. But that history is lossy and has no figures, so the words couldn't say what's needed and what's offered, which is what a start's words say. `notes/contracts.md`, section 1 has the shape.
  - **Not a format change:** the channel's refusal already has its text, and the figures cross no wire and aren't stored. So this isn't a question for the protocol's owner.
  - **The worker's own line is unchanged.** The request asks for the words sent, and logged in the engine. `protocol/README.md`, 15.3 says the worker's note doesn't carry the engine's text, because jig's link doesn't keep it. So a remote worker's log still says only `not-admitted`, and the words are on the engine's stderr. Carrying them to the worker would be a change through `jig-protocol-worker-link`, and it isn't in this plan.
- **2. Saved work goes through the holds a release already deletes.** Item 2's cause looks right from reading. `ReleaseRow` is stored, so a new field there would be a format change. Instead, as a release begins, each branch of the task's saved set becomes a hold of the closing task. `continue_release` then deletes it, retains it or moves on as it does for any hold, and no stored form changes.
  - **A failed task's saved work is kept with its root,** as a failed change's branch is, and deleted as the root closes. The request doesn't say. This destroys less and is easy to reverse, so `forge.md`, section 11 now says it.
  - **What it can't see:** the saved set is what a next attempt would resume from, not every branch ever pushed. A save whose terminal was never heard stays on the forge.
  - **`resume-told/saved-kept` matters here.** Until it lands, a producer's merge start whose save lands nothing drops its tag, by that task's own reading, and such a branch stays. `s10` and `s14` may have been that case, which can't be told from here. So temper/qa's next landing plan proves this only once both have landed.
- **3. The path is `Repair::Semantic`, from reading.** The change machine picks it for any CI red with a clean update behind it. Its words are `Repair the base's break at HEAD: BASE` and `Repair the base update's semantic conflict.`, naming no check. Nothing else found gives conflict words with no cause: a resolution's are about a conflict, and rightly. It now opens as a CI repair does and says no merge conflict waits.
  - **Not shown from here:** whether naming the check would have saved that `read_job`. The run's log isn't in this repository.
  - **Left as they are:** the state's name and its stored form; `the base broke it` in a change's result; and temper/ui's panel line, `Repairing: a semantic conflict with ...`. That line can still mislead on a red the base had no part in, and it's temper/ui's to change.
- **4. The flake is a lost reservation, from reading.** The test takes no port, but `prepare` takes kernel-chosen ones in two places. `reserve`'s `?` returns on the first bind error, so its 32 tries only cover excluded ports. `Io::listened` retries only a preferred port. Under `SO_REUSEADDR`, another process can bind a port explicitly and listen first, between this socket's bind and its listen. Both now take another port, staged in one thread. The evidence's log isn't in this repository, so which of the two was seen is unknown, and the task fixes both.

## Overlaps not avoided

- `port-reservation` changes `temper/crates/temper/src/init.rs`, which `ui-repo-link/box-sign-in` also changes. Its hunk stays inside `Io`.
- `refusal-words`, `saved-close` and `repair-cause` all change `temper-engine-domain`, in different files or functions. `saved-close` and `repair-cause` both change `route/forge_route.rs`, each adds a `mod` line to `temper/tests/engine/domain/tests/suite/forge_routes.rs`, and each deletes its own lines from `temper/docs/design/domain/forge.status.md`, as `resume-told/clean-merge` does too.
- `repair-cause` changes one line of `temper/docs/help/change.md`, which `resume-told/answer-words` also changes.
- `refusal-reason` changes `jig-domain-core/src/vocabulary.rs` and `routing/children.rs`, and a pattern in `temper-engine-domain/src/route/held.rs`, which `refusal-words` then uses.
- `saved-close` stays out of `temper-worker-domain/src/workspace.rs` and `temper-service/src/translate.rs`, where `resume-told/saved-kept` is working.
- No task touches `temper-client-*`.
