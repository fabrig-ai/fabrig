# D028: The scripted engine re-implements the core's rules

- **Kind:** tests
- **Where:** `testing/jig-client-scripted-engine/src`, `tests/client/src/stories`
- **Seen:** 2026-10-18 at `86971be`, in jig's tech-debt list ("Integration came last": the scripted engine is checked against nothing)
- **Seen:** 2026-10-31 at `c30cf6a03`, in the jig code-quality review (T6, A10 F2, F3, F5; order of work, items 16 and 17)
- **Seen:** 2026-10-31 at `1b5c98834`, in the quality run's debt list ("Test kits and worlds": the scripted engine still reimplements core's inbox order, phase classification and requester choice)
- **Checked:** 2026-11-03 at `a6a8155d35`
- **Value:** medium (the fake cannot agree with the client where the engine does not)
- **Size:** M
- **Status:** open
- **Paid when:** every rule the scripted engine copies from the core (inbox order, phase classification, requester choice) is either taken from jig's own code or held to the engine by a shared story on the scripted and joint fronts.
- **Measure:** `find jig/testing/jig-client-scripted-engine -name '*.rs' | xargs cat | wc -l` (15,363 when checked)

## What

Much is paid. The scripted engine encodes through the generated codec (`testing/jig-client-scripted-engine/src/codec`), sends
only owner documents, and 48 stories are written once over `Front` (`tests/client/src/stories/shared`)
and run on the scripted, joint and protocol fronts (`impl Front<Test>` in
`tests/client/src/scripted.rs`, `tests/client/src/joint/front.rs` and `tests/client-protocol/src/front.rs`).
What remains:

- the fake is 15,363 lines and still decides what the core decides: `queries.rs`'s `order` sorts task lines by the query's order (line 81), and it classifies phases and picks requesters itself;
- 985 lines of stories outside `tests/client/src/stories/shared` run on the scripted front alone, beside 2,081 lines of joint-only stories in `tests/client/src/joint`.

## Why it matters

`jig/docs/design/client/domain/testing.md`, 5 makes the joint world the check. A rule
copied into a fake is right only where a shared story exercises it.

## What pays it back

Move front-only stories into `tests/client/src/stories/shared` where both fronts can run them; for each copied
rule, call jig's code or add the shared story that pins it. Then shrink the fake.
