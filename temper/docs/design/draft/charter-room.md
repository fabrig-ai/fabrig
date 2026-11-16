# A charter that doesn't fit is named, and the room follows the profile

Draft, 2026-11-16, on the user's word. On 11-16, no goal on one deployment's charter 17 could start. Every first run was refused "assignment cannot encode", spent nothing, and the goal was held "out of tries: permanent failures", which a release didn't change. The refusal named neither the field nor the bound. A read of the code and the charter's texts found the likely cause:
- The fixed part of a first run's charter is everything but the brief: 42 tool declarations, about 36.5 KB; the instructions, which are the shared text and the first parts joined, about 26.2 KB; the models, the conventions and the delivery fields.
- That came to an estimated 65,618 bytes, against an allowance of 65,536.
- The charter's earlier revision, at 65,489 bytes, and the stock-based charter 5, at 64,774, both fit.
- The deployment's standard profile sets the charter's bytes to 262,144, but a fallback of 65,536 in the agent's allowance applies regardless.

## 1. The refusal says what didn't fit

- **Today:** temper's `assignment::down` and `fitted_charter` (`temper/crates/temper-engine-service-vocabulary/src/assignment.rs`), and jig's `translate::down` and `assignment_record` (`jig/crates/jig-protocol-engine-workers/src/translate.rs`), return no reason. `Workers::assignment` (`temper/crates/temper-engine-service-component/src/workers.rs`) and jig's `Component::deliver` (`component.rs`) say only "assignment cannot encode". The hub link's `payload::ending` (`jig/crates/jig-domain-hub-link/src/payload.rs`) turns an invalid start into `Failed(Permanent)` and drops its detail, so the held task's reason says "permanent failures".
- **Wanted:**
  - Each refusal on these paths returns a typed reason: the field (for example `charter.fixed_owned`), its actual figure, the configured bound and the wire's ceiling. Where the codec's `Problem.path` exists, it's kept.
  - The engine's log line prints the reason whole.
  - The held task's reason carries it too: the invalid start becomes `FailedWithReason`, within its 128-byte bound, cut with a marker.

## 2. The fixed charter's room follows the configured charter bytes

- **Today:** the effective allowance for a charter's fixed part is `min(hub.charter_bytes, the agent's allowance or 65,536)` (`assignment.rs`). The agent's run bytes derive as `max(2 × tool_payload, 65,536)` (`smith/crates/smith-domain-run/src/limits.rs`). So raising `hub.charter_bytes`, which the standard profile already sets to 262,144, leaves a hidden 65,536 cap in force.
- **Wanted:**
  - The fixed charter's allowance follows the configured `hub.charter_bytes` consistently on the engine's side and the receiving agent's, with no hidden fallback below it. The standard profile's 262,144 is what a standard deployment gets.
  - The small profile keeps its own figure. This draft sets no new figure: it makes the configured one hold.
  - Wherever the agent's allowance crosses a wire between engine and workers, the wire moves its version and bridges, by the upgrade rules.

## 3. A charter that can't fit is refused when it's applied

- **Today:** `settings apply` checks a charter's rendered room (`temper/crates/temper-engine-domain/src/room.rs`), but not its fixed part's ownership or encoded size. So a charter too large to start is accepted, and every goal on it fails at its first run.
- **Wanted:** applying settings refuses a charter whose first run's fixed part, with its tools and delivery fields, can't fit the allowance. The refusal names the charter, the part and the figures, and tells the person what to trim.

## 4. Out

- Trimming the deployment's charter, a separate settings change.
- Any other bound.

## 5. Tests

Cheap: focused and fuzzy suites only; walk = false.
- Beside the workers' assignment tests (`temper/crates/temper-engine-service-component/src/tests/workers.rs`), `a_first_assignment_over_its_fixed_charter_room_names_the_field_and_figures`. It builds a synthetic charter with these lengths: shared text 9,910 bytes, parts 11,031, 3,655, 1,597 and 649, the first three joined for a first run, 42 tools, a task's models, goal delivery, and a 2,342-byte task section.
  - At an allowance of 65,536, the refusal names `charter.fixed_owned` and its figures.
  - The same charter at 10,845 bytes in its first part fits.
  - At the standard profile's 262,144, it fits.
- The hub link's tests: a refused start's reason reaches the held task.
- The settings tests: applying a charter too large to start is refused, naming it.

Run only the new and affected tests, by name.
