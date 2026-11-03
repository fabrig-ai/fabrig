# playground-decisions: bring the code to the playground reset's design

The playground-reset close-out (on main since 2026-11-02 19:19Z) ported the adopted draft into the design. It left five questions for the user, each recorded in the status files where the code departs from the design. The user decided them on 2026-11-02. This plan brings the code to the design where it departs, and the status files to the decisions.

## The decisions

1. **A box restarted while the playground runs: reset it, as the design says.** Today the code keeps it running. See temper's `docs/design/protocol/setup.md` and its status.
2. **A removed forge's repository names: keep them while any kept task names them, as the design says.** Today the code keeps them only while a live task does: `temper-engine-domain-forge`'s `erase_removed_repositories` and `unnamed`. See `domain/forge.status.md`.
3. **A deleted project's conversations: cancel them with their close's grace, as a person's close does.** Today `RemoveProject` cancels with no grace: `jig-domain-core/src/admission/deletion.rs`, `cancel`, `grace: None`. See jig's `domain/people.status.md` and `domain/tasks.status.md`, which call this the user's decision.
   - The grace is the chat template's, as `client/domain/objects.md`'s Close gives it.
   - Where the engine reads no template, the design decides how the grace reaches the deletion, within the tasks child's bound.
4. **The playground's start acts stay under the sign-in of the person who sent the start,** as `protocol/setup.md`, 8.4 has it. No code change. Remove any status wording that leaves this open.
5. **The words for setting a forge up without the box are accepted for now:** "Set it up under `temper box`" and "Start temper with `temper box` to set one up here". The user will revise them when reviewing the web UI. Bring the code and the mockup to them: today they show the four `temper forge` commands and a restart. See `client/tools.status.md`, `client/forge.status.md` and the mockup's `pages.js`.

## How

- Through the loop, as any plan. The design already says what the code is to do. Where a design document still leaves a decision open, its designer states the decision there, in a batch of its own before the specs.
- As each piece lands, its Departs and open entries leave the status files (`DESIGN.md`).
- A budget or timeout is the user's (skein's `docs/foundation/testing-strategy.md`, 8.1); none should move here.

## Walk

None of its own. The playground's stories in the worlds carry it. fabrig/qa may fold a restart-during-the-playground check into the playground's rung if it sees fit.
