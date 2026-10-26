# The job answer's users, the store and the bounds, at the base

The code audit of temper at `118f7fb7` (`origin/main`), 2026-10-16, by a
read-only sub-agent. Paths are temper's.

## 1. What a new field of `Answer::Job` reaches

`crates/temper-engine-domain-forge-client/src/api.rs:156`, `Answer::Job
{ attempt, log, truncated }`; its derives follow `Answer`'s (`:129`).

- **Does not compile until changed** (constructs it, or binds every
  field):
  - `crates/temper-engine-protocol-forge/src/actions.rs:362`, the one
    production constructor;
  - `crates/temper-engine-smith/src/answers.rs:527`, the agent's
    rendering, `Job <job> run <run>:\n<log>` and `\n[log truncated]`,
    within `HostAnswer::CAPACITY` (`:32`);
  - `crates/temper-engine-domain-forge/src/brief.rs:308`, the brief;
  - `crates/temper-protocol-store/src/records/answers.rs:115-121`
    (encode) and `:223-226` (decode);
  - tests: `crates/temper-engine-domain-forge-client/src/tests.rs:405,416,424`;
    `crates/temper-engine-protocol-forge/src/tests/contents.rs:482`;
    `crates/temper-protocol-store/src/tests/fixtures.rs:464-468`;
    `tests/engine/forge/client/src/translate.rs:504-508`
    (`byte_answer`); `tests/engine/forge/client/tests/simulation.rs:505`.
- **Compiles, but must count or check the step:**
  `crates/temper-engine-domain-forge-client/src/bounds.rs:152` counts the
  log only (checked at `calls.rs:281`; it also feeds `answer_bytes`,
  temper-engine-domain-forge's `stored_bytes`, and
  `crates/temper-engine-domain/src/store.rs:655`'s `call_answer_bytes`);
  `crates/temper-engine-protocol-forge/src/reads/size.rs:113`, the
  component's copy of that measure; `api.rs:511-514`, `accepts`, binds
  `{ attempt, log, .. }`; `tools/forgejo-conformance/src/component/reads.rs:105`.
- **No change** (`{ .. }`, or `Read::Job` only): `api.rs:453,588,636,
  673,713,741,769,799,832`, `outbox.rs:494`, `locate.rs:22`,
  `keep.rs:794,892`; `crates/temper-engine-domain-forge/src/domain.rs:2993`;
  `crates/temper-engine-protocol-forge/src/reads.rs:446,586`; the forge
  client world's `src/world.rs:733`, `src/referee.rs:56`,
  `tests/simulation.rs:542,607,662,774`;
  `crates/temper-engine-domain/src/route/forge_route.rs:246`,
  `crates/temper-engine-smith/src/forge.rs:89`,
  `tests/engine/domain/tests/forge_routes.rs:414,437,1384`.
- The fake forge has no `Answer::Job`: its `Read::Job` answers bytes
  (`testing/temper-fake-forge-domain/src/api.rs:104-110`, `next.rs:150`).

## 2. The store

- skein's `skein_codegen` generates `src/generated/v1.rs` and
  `golden/v1/*.bin` from `schema/store-v1.schema`; the drift test,
  `store_schema_matches_generated_code_and_goldens`
  (`tests/codecs/tests/drift.rs:54`, `temper-codecs-world`), writes them
  with `SKEIN_CODEGEN_WRITE=1 cargo nextest run -p
  temper-codecs-world --test drift` (`:61`). Write mode never deletes a
  golden. A changed `JobAnswer` changes `record_job_answer_{full,smallest}.bin`
  and `enum_answer_job_{full,smallest}.bin`.
- `job_answer_log` (`generated/v1.rs:46`, ceiling 1_048_576 at `:143`,
  checked at `:1332`) is set to `forge.client.answer_bytes` at
  `crates/temper-protocol-store/src/records.rs:142`. `records.rs:146`
  ends `..w::CEILINGS`, and `lib.rs:89` `..records::limits(forge)?`: a
  limit left out still compiles, at the 1 MiB ceiling.
- `STORE_VERSION` is 1 (`src/lib.rs:15`), asserted by
  `src/tests/sizes.rs:46-49`. `docs/design/store/README.md` moves it
  "once per plan, at the first landing that changes an encoding";
  client-api's temper 04 moves it to 2 (decisions.md, 3).

## 3. Bounds

- The client's `answer_bytes` ships 65_536
  (`crates/temper-service/src/limits.rs:51`); it caps `Read::Job`'s
  `max_bytes` (`bounds.rs:28-31`) and the whole answer. **`max_bytes`
  may equal `answer_bytes`,** so a full log plus a step's name would be
  refused `TooLarge` (`calls.rs:281-283`); a file's path plus its bytes
  has the same gap.
- The brief asks at most `min(answer_bytes, ci_budget / 5)`
  (`crates/temper-engine-domain-forge/src/domain.rs:701`), then half its
  share (`brief.rs:401`); the agent chooses `max_bytes`
  (`crates/temper-engine-smith/src/forge.rs:96`).
- `crates/temper-engine-protocol-forge/src/limits.rs` has no job bound;
  the raw log is bounded by twice `answer_bytes` (`:321`).
- **A step's name:** the component's `string_bytes`, "the longest name,
  title, context, description, link or path kept"
  (`crates/temper-engine-protocol-forge/src/limits.rs:26-27`), caps
  every text Forgejo answers (`crates/temper-forge-forgejo/src/decode/walk.rs:362`),
  512 shipped (`crates/temper-service/src/forge.rs:120`). The client
  has no bound per field: a status's context and description count
  against the answer (`bounds.rs:142-150`); in the store they are
  `forge.client.answer_bytes` (`records.rs:113-114`) under `bytes
  1048576`.

## 4. The agent's tool

`read_job` is written by hand in `crates/temper-engine-smith/src/calls.rs:35`:
"Read one bounded CI job log.", with the shared `READ_SCHEMA` (`:19`);
neither names the answer's fields. No test checks its rendered text.
