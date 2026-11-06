# journeys: status

2026-11-06. Where temper's code lags or departs from [journeys.md](journeys.md).

- **Not yet:** 8.1: `again` opens a fresh box and walks the recording from its first act, with no way to prepare the box (`temper/tools/journeys/src/again.rs`), so a journey that begins past the welcome can't be recorded again by it: `again chat-draft` stopped at step 2 on `NoControl` on 2026-11-06, for want of a project and an account. `chat-draft` and `project-landing` are walked anew and added, as section 8 has it for a walk that changes (section 10). Whether `again project-landing` holds on its present recording is unknown.
- **Departs:** 3.6: the API-key field carries a pasted trailing newline rather
  than trimming it. Redaction preserves that newline and replaces both the
  whole-value and trimmed-value digests, matching what the host carries so the
  replay holds.
