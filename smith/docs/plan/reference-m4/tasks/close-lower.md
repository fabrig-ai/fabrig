+++
id = "close-lower"
part = "close-lower"
kind = "interface"
needs = []
files = ["smith/crates/smith-domain/src/boundary.rs", "smith/crates/smith-domain/src/route.rs", "smith/crates/smith-domain/src/tests/program.rs", "smith/crates/smith-protocol-llm/src/boundary.rs", "smith/crates/smith-protocol-llm/src/component.rs"]
effort = "high"
minutes = 60
+++
# A child conversation on a program ends with `Close`

## Goal

A child's program must end when the child's conversation does, and nothing tells the protocol
layer today when a conversation ends. Add contract C1 of
`smith/docs/plan/reference-m4/notes/contracts.md`: `smith_domain::Lower::Close { owner }`, sent
once when a child conversation whose session ran on a program backend ends, and
`smith_protocol_llm::FromDomain::Close { owner }`, which the service maps it to and the direct
component ignores. Nothing acts on it yet: task `program-children` routes it to the child's
program.

Acceptance:

- `Lower::Close` and `FromDomain::Close` exist exactly as C1 writes them, with its doc lines.
- smith-domain sends `Lower::Close { owner }` once for a child conversation on a program
  endpoint when its session ends, after the terminal of its last `Complete`, and never for main,
  a compaction or a direct child; unit tests show each.
- The direct component ignores `FromDomain::Close`; the service maps `Lower::Close` to it and,
  with no backend keeping `owner` yet, drops it.
- No test outside the files you own changes its expectation; fmt, clippy and the focused tests
  below are green.

## Inputs

- `smith/docs/plan/reference-m4/notes/common.md` (read it first) and
  `smith/docs/plan/reference-m4/notes/contracts.md`, C1.
- `smith/docs/design/protocol/programs.md`, 5.10 ("Its end": the agent closes the child's program
  when its conversation ends).
- `smith/docs/design/domain/run.md`, 5.3 (a child ends with the call that asked for it).

## Owns and may change

- `smith/crates/smith-domain/src/boundary.rs`: the `Lower::Close` variant.
- `smith/crates/smith-domain/src/route.rs`: sending it where a session's `Ended` is handled
  (`from_session`, `ended_session`); the peer or the session's spec tells you whether the
  conversation is a child on a program backend (`translate::spec` sets
  `session::record::Backend::Program` from `domain.config.program(endpoint)`).
- `smith/crates/smith-domain/src/tests/program.rs`: new tests.
- `smith/crates/smith-protocol-llm/src/boundary.rs` and `component.rs`: the `FromDomain::Close`
  variant and its ignoring arm beside `Yield`.
- Neighbour edits allowed: the arms the compiler requires for the new variants, in any smith file
  (`smith-domain/src/tests.rs`'s exhaustive `Lower` matches at about lines 306 and 1090;
  `smith-agent-service/src/service/request.rs`, mapping `Lower::Close` to `FromDomain::Close`;
  `smith-agent-service/src/service/program.rs` `complete`, where `Close` goes to no backend yet,
  as `Cancel` for an owner no backend keeps does). Each is edited only for the new variant.
- Pre-approved:
  - If the conversation's backend is not reachable at `Ended` (the peer is freed first), keep
    what you need on the peer when the session opens (a `bool` such as `program`), and say so.
  - If smith-domain's test harness cannot open a child on a program endpoint under a direct main
    (the way below), give `program()` a second endpoint and a charter model on it; if that still
    cannot, ask.

## Steps

1. Read the inputs, `smith-domain/src/boundary.rs` around `Lower::Yield`, `route.rs`
   `from_session` and `ended_session`, and the test
   `a_sub_agent_opens_a_child_session_whose_last_message_answers_the_call`
   (`smith-domain/src/tests.rs`, about line 1000), the model for yours.
2. Add the variants and the compiler-forced arms; `heavy cargo check -p smith-domain -p
   smith-protocol-llm -p smith-agent-service --tests`.
3. Send `Lower::Close { owner }` where a child's session ends, when that session ran on a program
   backend. `owner` is the token its `Lower::Complete`s carried. It follows the terminal of the
   last `Complete` (a cancelled one's `Cancelled` included): check that `session::Request::Ended`
   comes after it, and say in the report how you know.
4. Tests in `smith-domain/src/tests/program.rs`, each with a direct main whose charter lists a
   sub-agent model on the program's endpoint (`program()` maps `Endpoint(1)` to `claude_code`;
   main on another configured endpoint), as the existing child test does:
   - `a_program_child_that_answers_sends_close_once`: the child opens on the program's endpoint
     (its `Complete` prompt's asker is `Child`), answers with text, the call returns; the requests
     hold exactly one `Lower::Close` with the child's owner, after its last `Complete`'s terminal,
     and the next steps hold none.
   - `a_cancelled_program_child_sends_close_after_its_cancel`: the run cancels while the child's
     `Complete` is out; `Lower::Cancel` for it, then, once `Below::Cancelled` comes, one
     `Lower::Close`.
   - `a_direct_child_and_main_send_no_close`: a child on a direct endpoint answers and the run
     finishes; no `Lower::Close` in any step.
   Run them: `heavy cargo nextest run -p smith-domain --lib -E 'test(/program::/)'`.
5. `cargo fmt`; `heavy cargo clippy -p smith-domain -p smith-protocol-llm -p smith-agent-service
   --all-targets -- -D warnings`.
6. Focused tests, one command at a time: `heavy cargo nextest run -p smith-domain`,
   `heavy cargo nextest run -p smith-protocol-llm`, `heavy cargo nextest run -p smith-agent-service`.
   `smith-local-domain` and the worlds read `Lower` through the service: check them with
   `heavy cargo check -p smith-local-domain -p smith-agent-world -p smith-agent-process-world -p smith-local-world --tests`
   (use the package names `cargo metadata` gives if these differ).
7. Commit, push.

## Report

The variants as written, where `Close` is sent and how you know it follows the last terminal,
the three tests (red first where they can be, then green), every neighbour arm you added, the test
runs (selected, passed), and implementation and check minutes apart.
