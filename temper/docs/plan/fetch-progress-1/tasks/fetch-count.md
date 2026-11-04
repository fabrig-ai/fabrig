+++
id = "fetch-count"
part = "fetch"
kind = "feature"
needs = ["fetched-line"]
files = ["temper/crates/temper-forge-setup/src/fetch.rs", "temper/crates/temper-forge-setup/src/tests/fetch.rs", "temper/crates/temper-forge-setup-shell/src/fetch.rs", "temper/crates/temper-forge-setup-shell/tests/suite/fetch.rs", "temper/crates/temper/src/forge.rs", "temper/crates/temper/src/commands.rs", "temper/crates/temper/tests/suite/forge.rs", "temper/docs/help/reference/commands.md"]
effort = "high"
minutes = 65
+++

# The fetch counts its bytes and says so

## Goal

`temper forge fetch` keeps each binary's size from its answer, refuses a body past it, and says how far it has gone on stdout. Today it is silent while it streams and keeps no `Content-Length` bound.

Acceptance:

- the machine's lines, in order, for an answer with a size: 0, each whole percent passed at most once per chunk, and 100 once; with no size: 0, each MiB, and the end;
- a body past its size removes the part file and is refused, naming the URL;
- a zero `Content-Length` is no size;
- a file already there says nothing;
- the size is that of the `200` carrying the body, past a macOS redirect;
- `temper forge fetch` prints the lines, and a stdout that takes no bytes fails no fetch.

## Inputs

- `temper/docs/design/protocol/setup.md`, 6.6, and 13.1's fetch bullet.
- `temper/docs/plan/fetch-progress-1/notes/contracts.md`, section 2, and `notes/common.md`.

## Owns and may change

Owns the files in the header. Call sites of `Pin` and `fetch::run` in their tests are yours.

May also change: a departure from the implementer's text, `temper/docs/help/reference/commands.md`'s section on `temper forge fetch` and the matching text in `temper/crates/temper/src/commands.rs`: one sentence, "It says how far each binary has gone on its stdout, a line at each percent." Tests read that Markdown: run `heavy cargo nextest run -p temper --test suite` on what reads it, found with `rg 'help/reference|commands.md'`.

Pre-approved:

- skein-http delivers no byte past `Content-Length`, so the refusal cannot be seen through the host. It is a machine test alone; say so in the report and add no fault to the host.
- If the host test's local file server cannot answer without a length, give it a chunked answer in that test file.
- The macOS path counts the archive's bytes; `process::extract` stays silent.

## Steps

1. The machine, `temper-forge-setup/src/fetch.rs`. The sketches bind the names; the phases are yours.

   ```rust
   pub struct Pin { pub binary: Binary, /* its fields */ }   // from `pins()`'s loop
   pub enum Failure { /* its variants */ Size { url: Box<[u8]> } }   // `URL: body past its Content-Length`
   pub enum Request { /* its variants */ Say(Fetched) }   // answered by Event::Done
   ```

   One request per event stays the rule: a line is a phase between a write's `Done` and the next `Read`, after `Open`'s `Done` for the start, and at `Ended` before the digest's verdict for the end. `head` keeps `length`, a zero as none, on the `200`, before `Open`, whose reset of `total` leaves it alone. A line is due when `done * 100 / total` (in `u128`) rose past the last said, or with no size when `done / 1048576` did, and at the end when `done` differs from the last said.
2. Machine tests in `src/tests/fetch.rs`: the traces (`fetch_machine`, `bottle_machine`, `bottle_redirects`) take the new requests. New: `an_answer_with_a_size_says_each_percent_in_order`, `an_answer_with_no_size_says_each_mib_and_its_end`, `a_body_past_its_size_is_refused`, `a_zero_length_is_no_size`, `a_binary_already_there_says_nothing`, `a_redirect_takes_the_size_of_the_answer_with_the_body`.
3. The host, `temper-forge-setup-shell/src/fetch.rs`: `pub fn run(limits, trust, machine, out: &mut dyn std::io::Write)`. `Request::Say(fetched)` writes `fetched.line()` and answers `Event::Done`, whatever the write did. Rust's `stdout()` flushes at a line feed.
4. `temper/src/forge.rs`, `fetch`: passes `&mut std::io::stdout()`.
5. Host tests in `tests/suite/fetch.rs`: `the_lines_reach_the_writer_in_order`, with a size and without; `a_writer_that_refuses_fails_no_fetch`; a present file writes nothing.
6. `temper/tests/suite/forge.rs`, `forge_fetch_command`: asserts `stdout.is_empty()` at two places for every row. The `digest` row now prints its lines before it refuses: assert them. Every other row stays empty. Run it: `heavy cargo nextest run -p temper --test suite -E 'test(=forge_fetch_command)'`, and check it selected one test.
7. `cargo fmt`; the old-forms check; `heavy cargo clippy -p temper-forge-setup -p temper-forge-setup-shell -p temper --all-targets -- -D warnings`; `heavy cargo nextest run -p temper-forge-setup -p temper-forge-setup-shell`. Commit and push.

## Report

As the implementer's text has it, with: the lines a real run of the `digest` row printed; each rule broken for a new test and the failure seen.
