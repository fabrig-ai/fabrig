# replay-repairs: what the first real goal on a fresh box tripped over

A replay of a three-task documents plan on a fresh `temper box`, at `77544586ee`, ran the goal as the designer of its own tree (`temper/docs/design/draft/goal-as-designer.md`). It cut the tree, and one change landed into g. These are the faults it met, each seen once. Its evidence is outside this repository, so the words here are what you have. **Run, unless marked read.**

## What's wanted, ranked. Drop from the bottom if it passes six tasks, and say what you dropped

1. **A restart after a cancelled tree panics.** The goal's tree was cancelled with grace 0 while a review task was held; live tasks read 0; the box stopped and restarted on the same data. `serve` then panicked at `temper/crates/temper-engine-domain/src/loads.rs:383`, `startup continuation advances in walk order`, six times, and the deployment gave up. A deployment that can't start after a cancel loses everything after it. **The failing story first,** in the engine's domain world, where crash and restart stories already live (`temper/tests/engine/domain/tests/suite/loads.rs`), as a row of a table if one fits. Find why a startup continuation's cursor can fall behind its `after` once a tree is cancelled, and fix that cause. The assertion stays unless the design says the order may repeat. Say what state the cancel left.

2. **A small documents pull is too large to review.** A reviewer task (stock review charter 4) was held before it ran: `pull: the answer is past the client's limit`, logged as `section=pull stage=files error=TooLarge`. The goal's own `read_pull_files` on the same pull failed with "too large at forge.client.rows or forge.client.answer_bytes (measured size unknown); host answer limit 63740 bytes". The deployment had the reference's room (`temper/docs/examples/reference/room.json`). A review of a small change must start: page the files where a page is needed, or size the bound from the room, whichever the design holds (`temper/docs/design/domain/forge.md`). And a refusal names the bound it met and the size it measured. The failing test first, below the real loop.

3. **`temper ctl task turn` loses the end of a call's input.** The goal's call carrying three specs, about 34 KB, read back as two pages, both `cut=true`. Together they made 32,549 bytes ending inside the third spec, while the second page said `bytes_left=0` and `read_on=null`. Every byte of an accepted call's input must be readable, by continuation if not at once, and the continuation must say truthfully what's left. A step test in `temper-client-tools` or `temper-client-shell-cli`, wherever the cut is made.

4. **A delivery's checks time out saying nothing.** Three `deliver` calls returned `checks-failed ... exit=timed-out cut=0 tail=""` on document trees whose explicit `pre-pr` check passed; one passed on retry. **Read, not run:** find what times out and why the tail is empty. Then a refusal names the check that timed out and carries its output's tail, with a step test. If the cause is a figure, say which and leave figures as they are.

5. **The reference's room says what a box needs.** `room.json` and its `README.md`, "A call's room", gain `root.journal.run_bytes` 65,536: it bounds a charter's stored policy, and 20,480 refuses a charter of 31 KB, as `temper-service`'s `tests/room.rs` already sets. Also whatever figure item 2 settles. And a sentence: a charter's `turns`, `time` and the run spend must fit a worker's advertised bounds (`jig-domain-fleet/src/needs.rs`), or no host takes it.

6. **A settings refusal names the wrong item.** `jig/crates/jig-domain-core/src/routing/settings/dispatch.rs`, `malformed_refusal`, hardcodes `at: 0`, so a refusal of the third item, a charter, read "item 0". Name the failing item's position. A row in an existing refusal test.

## Not in this plan

- How the goal's charter reads, or when it surveys and writes design: the next plans, from a draft being written.
- Raising any figure beyond the reference's room, and test budgets.

**Tests stay cheap:** the focused and fuzzy limits are being tightened, and the gate will reject a plan that adds cost. A new case of one behaviour is a row of an existing table test, and a new story goes in a world below the real loop. Failing tests first; no test reaches a real host. **A plan runs the focused and fuzzy suites alone and is complete when it lands (`AGENTS.md`, 5).** A stored format or protocol change is a question first, so its owner can be told; both main areas write `temper-engine-domain` and `forge_route.rs`, and skein's interface changes get a heads-up. Formats change in place before v0.1.0. jig never names temper. `walk = false`.
