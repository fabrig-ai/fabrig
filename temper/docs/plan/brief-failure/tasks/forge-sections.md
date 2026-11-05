+++
id = "forge-sections"
part = "forge-sections"
kind = "feature"
needs = ["brief-words", "brief-counts"]
files = [
  "temper/crates/temper-engine-domain-forge/src/brief.rs",
  "temper/crates/temper-engine-domain-forge/src/brief_document.rs",
  "temper/crates/temper-engine-domain-forge/src/held.rs",
  "temper/crates/temper-engine-domain-forge/src/brief_words.rs",
  "temper/crates/temper-engine-domain-forge/src/tests/brief.rs",
  "temper/tests/engine/domain/tests/suite/forge_routes/brief_failure.rs",
  "temper/tests/engine/forge/tests/suite/brief.rs",
]
effort = "high"
minutes = 210
+++

# A forge section's read that fails says why, and one doesn't sink a repair

## Goal

Any forge read error at a section's stages falls to `BriefRead::Failed`, with no reason and no way on. Build `temper/docs/design/domain/forge.md`, 8.4 (*A brief section's read that fails*): each failure names its REASON and how it counts, a repair's two sections stand for each other, and each failing read is a line in the engine's log. This ends the story of the request: a CI repair whose pull read answers `InvalidAnswer` starts, and one with neither ends held, the reason in its document.

Acceptance:

- A failing test first for each of stories 1 and 2 below, in the `temper/tests/engine` forge world; the report quotes each failure on the code before the fix.
- All stories pass. No test reaches a real host.
- Every row of 8.4's table is the fate and the words the code gives: a table test of the function that maps an error, one row each, cut before the end to the rows that can still fail on a rule.
- `temper serve: brief read refused ...` and `temper serve: brief failed ...` are the lines 8.4 shows, each pinned by a `note_line` test in `temper/crates/temper/src/host.rs`, beside `read refused`'s.
- `TooLarge` at `Compare` still says `[comparison omitted: forge client limit exceeded]`; a job's log still says `[job log could not be read]`.
- **This departs from the implementer's text,** for two entries alone: delete `temper/docs/design/domain/forge.status.md`'s "8.4: a brief section's read failure" entry and `jig/docs/design/domain/connectors.status.md`'s "9: a connector saying why" **Not yet** entry.

## Inputs

- `temper/docs/plan/brief-failure/notes/common.md`; `temper/docs/plan/brief-failure/notes/contracts.md`, 1, 3 and 5.
- `temper/docs/design/domain/forge.md`, 8.4; `jig/docs/design/domain/engine.md`, 9.1; `jig/docs/design/domain/connectors.md`, 9.
- `temper/docs/design/domain/testing.md`. `temper/AGENTS.md`.

## Owns and may change

Owns the files in `files`; `brief_words.rs` is new, holding `BriefFate`'s and `BriefUnread`'s rows of 8.4.

May also change, the lines this needs and no others. **Other drafts are changing each of these files,** so keep each hunk small:

- `temper/crates/temper-engine-domain-forge/src/boundary.rs`: the types of `notes/contracts.md`, 5. `src/domain.rs`: `PlanBrief`, `GatherPlanned`, the planned map's value, and the `BriefSized` it answers. `src/lib.rs`: a `mod` line and exports. `src/tests.rs`: arms.
- `temper/crates/temper-engine-domain/src/assemble.rs`: `finish_brief_plan` and `plan_forge_section` name the partner, the `!gatherable` `Missing`. `src/route/forge_route.rs`: the `BriefSized` arm and a `BriefReadRefused` arm. `src/route/now.rs`: `BriefFailed`'s arm. `src/boundary.rs`: two `Fact`s. `src/route/diagnostics.rs`: their emit. `src/admission.rs`: an arm if it needs one. `src/route/forge_route/tests.rs`, `src/tests/documents.rs`: literals.
- `temper/crates/temper-service/src/engine.rs`, `src/lib.rs`: two `Note`s. `temper/crates/temper/src/host.rs`: their lines and tests.
- `temper/tests/engine/domain/src/forge_routes.rs`: a test-only fault beside `fail_job_reads`. `temper/tests/engine/domain/tests/suite/forge_routes.rs`: a `mod` line.

Options, pre-approved:

- The world's fault names which reads fail. If the connector's keep-up makes the same read, tell a brief's call by its owner token and say how.
- The change's panel or requester says something about a held repair: leave it, and report what it says. A held repair escalates as any `Hold::Failures` does; this task adds no route for it.
- A stall or a test around a repair moves because its brief now counts: read why first. If 8.4 or 9.1 says so, change the expectation and name it; if not, it's a `Question:`.
- `brief_ci::Section` needs nothing for a failed `Checks` read, where `fetch.ci` is `None`: the words go through `brief_part`. If you find otherwise, say so.

## Steps

1. The types. The root's translation is one-to-one: `Err(unread)` is `GatherEvent::Missing { why: GatherFailure { fate, section: forge_brief_name(&row.kind), words: unread.words } }`. This replaces `brief-words`' two stand-ins.
2. `brief_words.rs`: `pub(crate) fn unread_of(result: &Result<Answer, Error>) -> BriefUnread`, every `Error` arm listed, no `_`. The answer that isn't an error but isn't what the stage takes: a `Pull` with another head is transient, `the change's head moved`; any other shape is permanent, `the forge answered another read`. Each `BriefRead::Failed` before a read (`brief_pull`, `brief_reviews`, `brief_ci`, `gather`, `held`, `domain.rs`, `brief_document`) names its row of the table; a `Busy` kept to the deadline in `expire` is capacity.
3. The pair. `PlanBrief { partner }` on the reason's plan names the pull's section, and the connector knows it both ways, through `Pending` into `Held`. In `brief_answer`'s catch-all, by `unread_of`'s fate:
   - capacity: `BriefRead::Failed`, whatever the partner;
   - no partner, or a partner already unread: `BriefRead::Failed`;
   - otherwise: mark this section unread, `append` its marker to what it gathered, finish as a read that ended.

   The markers are 8.4's. `Ok(_) | Err(_)` at `BriefStage::Job` stays the job's own marker.
4. Each failed read pushes `Request::BriefReadRefused { section, stage, error }`, the stage by `BriefStage`'s name. The root makes `Fact::BriefReadRefused { task, section: forge_brief_name, stage, error }` from the row; `Now::BriefFailed` makes `Fact::BriefFailed { task, counts, reason }`. The service and the host carry them as `ReadRefused` is carried.
5. Stories, new `temper/tests/engine/domain/tests/suite/forge_routes/brief_failure.rs`, from `failed_ci_repair`, each a CI repair on the world's fake forge:
   1. `a_repair_whose_pull_read_answers_wrongly_starts_with_words`: the pull section's first read answers `InvalidAnswer`. The repair is assigned; its pull section still names `Pull request: #`, and says `[the pull request could not be read: the forge's answer did not match its read]`; the CI section is whole; one `Fact::BriefReadRefused`, no `Fact::BriefFailed`.
   2. `a_repair_with_neither_section_ends_held_naming_the_reason`: both sections' reads answer `InvalidAnswer`. No assignment; the notice says `ci: the forge's answer did not match its read` with count and allowance; past it `Hold::Failures(Permanent)` and those words on the task's document; the change stays `Repairing`, and its stall isn't what held it; `Fact::BriefFailed` each time. Clear the fault, release: the repair is assigned.
   3. `a_repair_whose_checks_cannot_be_read_starts_on_its_pull`: only the `Checks` read fails, `[the failed checks could not be read: ...]`, and the task's words still name the check.
   4. `a_repair_with_the_forge_not_connected_pauses`: `NotConnected`; the notice is `Refused`, its reason saying so, count past the allowance, never held; the fault cleared, it starts.
6. A review's pull section alone: in `temper/tests/engine/forge/tests/suite/brief.rs`, its read's failure is `BriefSized { size: Err(..) }`, no words.
7. `cargo fmt`; old forms for temper; `heavy cargo clippy -p temper-engine-domain-forge -p temper-engine-domain -p temper-service -p temper --all-targets -- -D warnings`.
8. This changes what a shared item means: run its readers. `heavy cargo nextest run -p temper-engine-domain-forge -p temper-engine-domain -p temper-engine-domain-world -p temper-engine-forge-world -p temper-service`, and `-p temper --bin temper` for `host.rs`. Name the worlds as their manifests have them. Check each count.
9. Commit, push.

## Report

As the implementer's text has it, with each story's first failure, the rule each test bit on, and what the change's panel says of a held repair.
