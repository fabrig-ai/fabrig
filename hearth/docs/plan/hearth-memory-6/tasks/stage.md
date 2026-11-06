+++
id = "stage"
part = "stage"
kind = "migration"
needs = []
files = ["hearth/crates/hearth-service-builder/src/lib.rs", "hearth/crates/hearth-service-builder/src/tests.rs", "hearth/scripts/old-forms.txt"]
effort = "high"
minutes = 40
+++

# The service calls the shared stage

## Goal

D004's **Paid when**: `hearth-service-builder` routes completions with `file_layer::owns`, calls `skein_io::stage::{up, down}`, and has no `tag` and no `io::resume`. `io_up` and `io_down` end in their final form, and the queues only they push to are uncapped, so `Io`'s constants leave the service. `protocol_up`, `domain_stage` and `protocol_down` stay as they are, room checks included: `uncap` takes those.

Acceptance:

- `git grep -nE '\bfn tag\b|io::resume|io_room|io_within|io::MAX_OUT' -- hearth/crates/hearth-service-builder/src/lib.rs` prints nothing.
- `git grep -n 'file_layer::owns' -- hearth/crates/hearth-service-builder/src/lib.rs` shows the split in `io_up`.
- `Service` holds what `notes/contracts.md` gives, `io_up`, `io_down` and the end of `iterate` have that form, and `file_told.room()` and `file_submissions.room()` are gone.
- A new test, `a_file_operation_reaches_the_kernel_under_its_own_token`, passes and bites (below).
- `hearth/scripts/old-forms.txt`: `io_resume` falls to 0, with `--write`.
- Every test of `hearth-service-builder` and `hearth-builder-world`'s focused suite passes, with `fixtures::CUTS` unchanged, and `hearth`, `hearth-test-repository` and `hearth-conformance-world` build their tests.
- A change of meaning: `hearth-builder-world`'s `fuzzy` profile runs too (`--profile fuzzy`), as routing and its order moved.

## Inputs

- `hearth/docs/plan/hearth-memory-6/notes/common.md`
- `hearth/docs/plan/hearth-memory-6/notes/contracts.md`, "`hearth-service-builder/src/lib.rs` after `stage`"
- `hearth/docs/development/debt/D004-service-drives-io-by-hand.md`, whole
- `hearth/docs/development/debt/D001-room-checks-and-max-out.md`, "The service", the rows on `io_room`, `file_told.room()` and `file_submissions.room()`
- `skein/crates/skein-io/src/stage.rs`, `skein/testing/skein-world/src/files.rs`, and `skein/docs/design/io.md`, 5: the lanes
- `smith/crates/smith-agent-service/src/service/up.rs`, `io_up`: a root that splits completions and then calls the stage

## Owns and may change

- **Owns:** the files under `files`.
- **`limits.queue >= 3` stays** in `Service::new`, for `uncap` to take. So do `calls`, `asked`, `answers` and their room checks, and `queue_limit`.
- **`protocol_up`, one line:** `0..service.file_told.capacity()` becomes `0..service.file_told.len()`, as `file_told` is now uncapped. `protocol_room` stays beside it.
- **`hearth/tests/` and tokens:** the request says these know about `tag` where they read tokens. A survey of `hearth/tests/` and `hearth/testing/` found nothing reading a token's parity or shifting one. `fixtures::STARTUP_CLOSE` is `Token::new(u64::MAX)`, which `file_layer::owns` reads as the file lane, but `hearth/tests/builder/src/simulated/client.rs` takes it before the service sees it, so it stays. `fixtures::tool_spans` pairs a `Submit` and a `Complete` by `(pid, op)`, whatever the token. If a test does read a token's value or parity, follow it to the token its owner minted, in that test alone, and name it in the report.
- **`hearth/tests/builder/` otherwise:** no change is expected. A test that fails is a finding (`notes/common.md`).
- **An order you think moved:** a file completion is no longer held back while `Io` has ready work, and `expire` no longer waits for room. Both are what `skein/testing/skein-world/src/files.rs` and smith's root do, and neither is an order `hearth/docs/design/domain/render.md`, 8.5 states. Say so in the commit.

## Steps

1. Add the two queues, uncap the three, and rewrite `io_up`, `io_down` and the end of `iterate` to `notes/contracts.md`. `work_pending` and `is_empty` read the new queues.
2. Take out `tag`, `io_room`, `io_within` and the token shift in `io_up`.
3. In `src/tests.rs`:

   ```rust
   #[test]
   fn a_file_operation_reaches_the_kernel_under_its_own_token() {
       // A beneath request, through `io_down` and the end of `iterate`, as
       // `a_beneath_request_reaches_io_with_its_roots_descriptor` does it.
       // The `Submit` in `service.submissions()` has `file_layer::owns(submit.op)`.
   }
   ```

4. **Bite:** put a doubling of the token back at the end of `iterate`, see this test fail because the operation left the file lane, and restore it.
5. `python3 -I -B scripts/old_forms.py --write` from `hearth/`.
6. `heavy cargo clippy -p hearth-service-builder --all-targets -- -D warnings`, the tests named above, the `fuzzy` run, and the `check`s.
7. Commit and push. The commit says why: one routing rule, skein's, and skein's tested order.

## Report

The rule broken and the failure seen; anything `hearth/tests/` needed; `fixtures::CUTS` and any ceiling, moved or not; what ran, how many tests, and the CPU seconds of the new test.
