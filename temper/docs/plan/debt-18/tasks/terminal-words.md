+++
id = "terminal-words"
part = "smith-review"
kind = "feature"
needs = ["local-residue"]
files = ["smith/crates/smith-local-protocol/src/terminal.rs", "smith/crates/smith-local-protocol/src/lib.rs", "smith/crates/smith-local-service/src/service.rs", "smith/crates/smith-local-shell/src/front/headless.rs", "smith/crates/smith-local-domain/src/domain.rs", "smith/crates/smith-local-domain/src/lib.rs", "smith/crates/smith-local-domain/src/agent.rs", "smith/crates/smith-local-domain/src/person.rs"]
effort = "high"
minutes = 50
+++

# smith: the person's words in the service and headless front go to `terminal.rs`, and its loops become one

## Goal

`temper/docs/plan/debt-18/inputs/smith-debt14-review.md`, findings 5 and 6. The words a person reads, left in
`smith-local-service/src/service.rs` and `smith-local-shell/src/front/headless.rs`, move to
`smith-local-protocol/src/terminal.rs`; its two UTF-8 loops become one helper, `said_text` stops cloning, the domain's
two forwarders go, and three stale comments are put right. Every byte a person sees stays the same.

Acceptance:

- `service.rs` holds no person's words (`> ` and `Unknown or incomplete command` come from `smith_local_protocol`), and
  `front/headless.rs` has no `format!`, `write!` or `writeln!` of words outside its tests.
- One helper holds the UTF-8 walk: `append_text` and `append` both call it, and no second copy of the width test stays in
  `terminal.rs`. Their tests (`turn_caps_keep_whole_characters_and_allow_later_segments` and the headless ones) pass
  unchanged.
- `said_text` returns `&[u8]`.
- `Domain::said` and `Domain::tool_verdict` are gone; `smith_local_domain::said` and `smith_local_domain::tool_verdict` are
  re-exports of the functions they forwarded to.
- The doc comments on `Terminal::show`, `Terminal::notice` and `choice_text` say what's true (below).
- The branch's net lines fall by at least 10 (the review's estimates: -14, -8, a little for `said_text`; the words only
  move). Every test passes unchanged, its bytes included.

## Inputs

- `temper/docs/plan/debt-18/notes/common.md`
- `temper/docs/plan/debt-18/inputs/smith-debt14-review.md`, findings 5 and 6
- `smith/docs/design/shell.md`, 4.2

## Owns and may change

- **Owns:** the files in `files`, in these places only.
- **The status counts in finding 6 need nothing.** `smith/docs/design/testing.status.md` and `STATUS.md` already say three
  domain worlds call protocol crates (`tests/agent`, `tests/run`, `tests/host`); no docs change in this task.
- **Other words stay.** `service/accounts.rs` (`Sign in: `), `service/choices.rs` (`Unknown model `), and
  `front/interactive.rs` and `front/remote.rs` aren't in the request. Name them in your report as left.
- **Public items.** Removing the two `Domain` forwarders is asked for by the request, and only `front/headless.rs` reads
  them. `said_text` has one reader outside its crate, the same file. Nothing outside smith reads any of these.

## Steps

1. The helper, as `terminal.rs` is a strict-subset step module:

   ```rust
   /// The longest prefix of `text` made of whole characters that fits in `room` bytes.
   fn whole_prefix(text: &[u8], room: u32) -> &[u8]
   ```

   `append_text` pushes `whole_prefix(text, shown.room())`. `append` keeps its one byte for the newline
   (`text.bytes.room().saturating_sub(1)`), and sets `text.cut` when the prefix is shorter than `bytes`.

2. `said_text(said: &local::Said) -> &[u8]`; its callers (`.into()` in `headless.rs`).

3. `pub fn said` in `agent.rs`, `pub fn tool_verdict` in `person.rs`, with their docs and `#[must_use]`, re-exported from
   `lib.rs`; `domain.rs`'s two forwarders go; `headless.rs` calls them.

4. In `terminal.rs`, re-exported from `lib.rs`, with one line of doc each:

   ```rust
   pub const PROMPT: &[u8] = b"> ";
   pub const REFUSED_COMMAND: &[u8] = b"Unknown or incomplete command";
   /// `smith: tool NAME\n`
   pub fn told_tool_line(name: &[u8]) -> Box<[u8]>
   /// `smith: Tool finished\n`, or failed
   pub fn told_verdict_line(verdict: local::ToolVerdict) -> Box<[u8]>
   /// `accepted in T turns, S notional, M ms\n`
   pub fn accepted_line(turns: u32, spent: u64, millis: u64) -> Box<[u8]>
   /// `needs input: answer with `smith exec --chat CHAT`\n`
   pub fn needs_input_line(chat: &[u8]) -> Box<[u8]>
   ```

   `headless.rs` keeps `smith_events::clip` and `String::from_utf8_lossy` on its side, passing `.as_bytes()`: a name or a
   chat that isn't UTF-8 prints as it does today. It writes the returned bytes with `errors.write_all`, dropping the result
   as it drops `write!`'s. A field's real type wins over the sketch's.

5. Comments: `show`, "Forward bounded text to the terminal."; `notice`, "Render a typed local notice as bounded UTF-8
   text: a line ends in a newline, a choice doesn't."; `choice_text`, "Render local choice feedback under the display bound."
   Check them against the code as you write them.

6. `cargo fmt`; `python3 -I -B smith/scripts/old_forms.py`; `heavy cargo clippy -p smith-local-protocol -p smith-local-service -p
   smith-local-shell -p smith-local-domain --all-targets -- -D warnings`; `heavy cargo nextest run` on those, with
   `smith-local-world`; `heavy cargo check --tests -p temper` only if `rg` finds a temper reader; the citations check. Commit
   and push.

## Report

The common note's items, each move, and the words left. A rewrite with no new rule: nothing to break by hand, unless you
add a test.
