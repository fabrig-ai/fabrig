# ui-box-slots: the plan

The request is `inputs/request.md`. It allows two tasks at most: focused and fuzzy alone, with new cases as rows of existing tables.

## Parts and tasks

| Part | Task | Needs | What it does |
|---|---|---|---|
| `box-words` | `box-words` | none | The request's sections 2 and 3: a box's wait, and a failed settings read |
| `box-slots` | `box-slots` | none | The request's section 1: 4 slots, and what rises with them |

Critical path: none. The two tasks share no file and neither needs the other, so each is its own 40 minutes.

## What 4 costs

**4 fits, so the number stays 4.** This comes from reading the code and from a test already on the trunk. No build ran for this cut, because a cut runs where nothing builds. `box-slots`' focused run is the first to start these documents at 4.

| What was checked | Found |
|---|---|
| The start's relations (`temper/docs/design/domain/worker.md`, 6) | 4 keeps slots ≤ `hub.slots`, `fleet.slots` and `checkout.workspaces`, and workspaces ≤ `git_component.operations`. `serve_prepares_eight_slots_and_ten_workspaces_on_the_owned_example` already starts this document at 8, 8, 8, 10, 10. |
| The start's socket fit (`temper-service/src/startup/checks.rs`, `socket_count`) | It needs 4 sockets a slot and 4 an operation, of 256. Going from 1 and 3 to 4 and 4 adds 16. That same test passes with 40 more than 4 and 4 take. |
| Whether a stock limit assumes one run | None does. jig's `standard` is sized for this: `hub.slots` and `fleet.slots` 4, `fleet.calls` 256 against 4 runs' 192 at `hub.run_calls` 48, `fleet.turns` 128, `fleet.attempts` and `fleet.workstreams` 64. The box's 2s were overrides below the profile. Past a shared bound, the fleet answers busy. |
| The engine's byte budget, `engine.memory_bytes`, 1 GiB | A run's ceilings are `hub.transcript_bytes` plus `hub.turn_queue_bytes`, 201.6 MB. Four come to 806.6 MB, 75% of the budget; five 94%; six 113%. Past it, new runs wait; nothing fails. |
| The worker's watermark, 10,198 entries | Its comment counts one replay window, but it's a limit on pacing, not memory. Four windows add about 51 entries to the 5,099 it was doubled from. It doesn't bind. |
| Memory on a laptop: box and serve | Measured 2026-11-04 (`setup.status.md`): anonymous memory 12.76 MB idle and 13.23 MB with one goal running. So about 0.5 MB a run. |
| Memory on a laptop: the agent processes | **Unknown.** That measure leaves them out, and no figure for one's resident size was found in the design. `declared.memory` is 2 GiB each, 8 GiB at 4, but it's carried unenforced and no start compares it. A run on a program (Claude Code) adds that program's process, also unmeasured. |
| Disk | One more cached workspace, 4 over 3, each up to `checkout.repositories` repositories. The size is the person's repositories'. The least recently used is evicted as before. |
| CI | `runner.capacity` stays 1, so CI runs one job at a time beside 4 runs. The request doesn't ask to change this; `setup.md`, 8's *Runner slots* says what it takes to raise it. |

**The one open risk is the agents' resident memory on a small laptop.** No start refuses it, and nothing in this plan measures it, by the user's rule that no walk runs in a plan. A check after landing would settle it, and 4 is the user's to reverse.

## What was decided

### The slots

`hub.slots` and `fleet.slots` lose their overrides, rather than rising to 4. `standard` already holds them at 4, and `configuration.md`, 13.4 says a document's overrides say only what differs. This is the request's "move them together" reached by deletion.

`checkout.workspaces` and `git_component.operations` gain overrides at 4, the least the relations take. temper's `standard` stays at 3, so hand-written documents don't change.

`bare.json` and the owned example move with `temper.json`. `the_default_keeps_the_owned_structure_with_stock_models` holds the template to the example, and `the_bare_document_is_the_box_less_its_parts` holds bare to the box.

`temper/docs/design/domain/root.md`, 5.2 said "a box hosts one run at a time" of `fleet.calls`' 4 MB. The bound is shared, so it now says so.

### A box's words

The view can already tell a box from `temper serve`. The deployment's read, `DeploymentAnswer.owned`, is true only where the document has `application.machine.owned` and the engine runs under the box (`temper-service/src/system.rs`, `deployment_owned`). So no wire field is added.

`temper box` on a document with no `owned` part reads false and keeps today's words. `temper init`'s box document has the part. That's written down as an open question in `temper/docs/design/client/README.md`, with its default.

### A failed call's answer

A new `Problem::Needs { kind, member }` follows the commit identifier's precedent, in that the answer says what to do. Both members a read's kind can need get it: `number` and `project`.

A kind no setting has, with no `number`, used to answer "member missing: number". It now answers out of range for `kind`, which is what it was already answered when it had a `number`.

The `propose_settings` form reader isn't changed: it has its own words, and `model-catalogue-2/configuring-learns` is on that crate.

No shape crosses between the two tasks, so there is no `notes/contracts.md`. No stored format, protocol or public interface changes.

## Overlaps that could not be avoided

- `temper-init/template/temper.json` and `bare.json`, `docs/examples/owned.json`, and `temper-init/src/tests/**`: `model-catalogue-2/stock-models`. The request expects this: whichever lands second catches up.
- `temper/crates/temper/tests/support/**`: `model-catalogue-2/process-fixtures`. `box-slots` doesn't edit it, but that task moves the same documents.
- `temper/docs/help/deployment.md`: `model-catalogue-2/models-help`, one line.
- `temper-engine-smith/src/settings.rs` and `calls.rs`: `model-catalogue-2/configuring-learns`.
- `temper-engine-smith/src/answers.rs`: `ui-proposal-2/chat-revises`, one compiler-forced arm.
- `temper-client-view/src/words.rs`: `limits-fixtures/view-words`.
- Status files beside this plan's design lines: `temper/docs/design/client/README.status.md` and `protocol/configuration.status.md` with `model-catalogue-2/models-proof`, and the first also with `look-round-13/look-proof` and `ui-proposal-2`.

## After landing

- The process, browser and live tiers run then, by the user's rule. A story that relied on one run at a time without saying so comes back as a repair. None was found by word or value but `process.rs`'s fixture, which `box-slots` edits and compiles.
- The agents' resident memory at 4 is the check worth filing after landing.
