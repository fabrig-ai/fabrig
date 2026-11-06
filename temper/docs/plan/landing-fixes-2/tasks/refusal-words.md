+++
id = "refusal-words"
part = "hello-refusal"
kind = "feature"
needs = ["refusal-reason"]
files = [
  "temper/crates/temper-service/src/route.rs",
  "temper/crates/temper-service/src/route/refusal.rs",
  "temper/crates/temper-service/src/engine.rs",
  "temper/crates/temper-service/src/lib.rs",
  "temper/crates/temper-service/src/startup/problems.rs",
  "temper/crates/temper-service/src/startup/checks.rs",
  "temper/crates/temper-service/src/tests/hello_refused.rs",
  "temper/crates/temper-engine-domain/src/decision.rs",
  "temper/crates/temper-engine-domain/src/route/held.rs",
  "temper/crates/temper/src/host.rs",
  "temper/tests/engine/domain/tests/suite/hello_refused.rs",
  "temper/docs/design/protocol/README.status.md",
]
effort = "high"
minutes = 70
+++

# A refused worker hears why, and the engine says it

## Goal

The second half of the request's item 1. `temper-service/src/route.rs` sends the fixed text `not admitted`, the engine's stderr says nothing, and the worker logs `refused w1 ... not-admitted` and redials for good. `startup/problems.rs` has the words for an agent's shortfall at start, which a charter set later never reaches. Send those words as the refusal's text, and write them on the engine's stderr.

Acceptance:

- The words and the line in `temper/docs/plan/landing-fixes-2/notes/contracts.md`, section 2, as written.
- A worker refused for a charter set after the start gets that shortfall's words as the channel's text, and the engine's stderr has one line with them.
- Two refusals with the same words in a row print one line. Different words print again, and so do the first words if they come back after them.
- Every refusal's text fits the channel's 256 bytes at the largest figures and numbers, since `skein-channel` asserts it. A test holds this for every row of the table.
- A refusal with no reason sends `not admitted`, and is logged too.
- A start's own problem lines read as they did: the tests of `admission` in `temper-service/src/tests/startup.rs` pass unchanged.
- The worker's line is unchanged, by `temper/docs/design/protocol/README.md`, 15.3. No wire or stored form changes: the channel's refusal already has its text field.

## Inputs

- `temper/docs/design/protocol/README.md`, 15.3, the paragraph beginning `The engine says why a hello is refused`.
- `jig/docs/design/domain/engine.md`, section 8; `jig/docs/design/protocol/channel.md`, 4.3.
- `temper/docs/plan/landing-fixes-2/notes/contracts.md`, sections 1 and 2.
- `temper/docs/plan/landing-fixes-2/notes/common.md`.

## Owns and may change

Owns the files in the header. `route/refusal.rs` and both `hello_refused.rs` are new.

Reader and neighbour files:

- `temper/crates/temper-service/src/tests.rs` and `temper/tests/engine/domain/tests/suite/main.rs`: the `mod hello_refused;` line.
- `temper/crates/temper-engine-domain/src/boundary.rs`, `domain.rs`, `route/making.rs`, `temper-service/src/clients/translation.rs`, `temper/tests/system/src/composed.rs`, `temper/tests/engine/domain/src/walking.rs`: these read `Delivery::Refuse { .. }` or `core::Held::Refuse { .. }` and need no edit. `temper/tests/engine/domain/tests/suite/root_routes.rs`, lines 1234 and 4622, count them; a pattern naming only `channel` takes `..`.
- `temper_service::Note` has exhaustive readers: `temper/crates/temper/src/host.rs`, `note_line`, and any other the compiler names. Each gets its arm, and only that.

Other drafts: `ui-repo-link/box-sign-in` changes `temper-service/src/clients.rs`, `route/box_link.rs` and `startup/application.rs`, and `model-catalogue-1/stock-seven` changes `startup/members.rs`. None of those are yours.

Departs from the implementer's text: delete from `temper/docs/design/protocol/README.status.md` the sentence beginning `15.3: the engine's refusal of a hello`, leaving the rest of its line.

Options, each pre-approved:

- `problems.rs`'s `admission` writes through `(writer, length, measuring)`. Default: add `pub(crate) fn admission_words(why, offered, required) -> Box<[u8]>` that measures, then writes through it, so there's one copy of the words.
- `startup/checks.rs`, `admission`: take its amounts from `fleet::figures`, passing the `HostKind::Worker` its `shortfall` call passes, leaving only its path. Default: do this, so one decision has one projection. Where it can't express a row (`What::Agent` for a missing model stays), say so in your report.
- The last words live in a field beside `notes` on `Engine` (`temper-service/src/engine.rs`), `Option<Box<[u8]>>`, at most 256 bytes. It starts empty on each start, so a restart prints the line again.
- If `Delivery`'s size is checked and `hello` trips it, box it, and say so.

## Steps

1. **Reproduce.** In `temper/tests/engine/domain/tests/suite/hello_refused.rs`, modelled on `root_routes.rs`, `hello_with` at line 55: raise a charter's `turns` by the settings path after the start, then say hello with the old bound. Assert `driver.delivered` has `Delivery::Refuse { hello: Some(..) }` with `AgentShortfall::Turns` and the new figures. It fails until `Delivery::Refuse` carries the field.
2. **Words.** In `temper-service/src/tests/hello_refused.rs`, add `each_refusal_says_its_words` (a row per table row of the note, exact bytes) and `no_refusal_passes_the_channels_bound` (each row at `u64::MAX` figures, `u32::MAX` numbers, at most 256 bytes).
3. **The delivery.** In the same file, drive `root::Delivery::Refuse` through the route as `temper-service/src/tests/workers.rs` drives a worker's request (near its line 516). Add `a_refused_hello_sends_its_words_and_notes_them` and `a_repeat_of_the_same_words_notes_nothing`. If no fixture reaches the arm within this crate's tests, make `route::refusal` and a small `refuse` function the seam, and test those. Say which you did.
4. **The line.** In `temper/crates/temper/src/host.rs`, beside the test at line 700, assert the exact line.
5. Fix: `Delivery::Refuse` with `hello`, passed by `route/held.rs` (it reads `channel, ..` on the trunk). Add `route/refusal.rs`, the words and the note in `route.rs`, and the line in `host.rs`.
6. **Break each rule once:** send the fixed text and see `a_refused_hello_sends_its_words_and_notes_them` fail; remove the repeat check and see `a_repeat_of_the_same_words_notes_nothing` fail. Restore.
7. Words an operator reads change, so read `rg 'not admitted|not-admitted' temper` and confirm that no test or help page pinned the old text. A reader you find is an edit you may make, and say so.
8. `cargo fmt`, `python3 -I -B temper/scripts/old_forms.py`, `heavy cargo clippy -p temper-service -p temper-engine-domain -p temper --all-targets -- -D warnings`.
9. Tests: `heavy cargo nextest run -p temper-service -p temper-engine-domain -p temper-engine-domain-world`, and `heavy cargo nextest run -p temper --lib -E 'test(host::)'`. Commit and push.

## Report

The implementer's report, plus the line a real shortfall prints, as an example, and whether `checks.rs` could take all its amounts from `fleet::figures`.
