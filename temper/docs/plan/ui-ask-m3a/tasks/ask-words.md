+++
id = "ask-words"
part = "instructions"
kind = "feature"
needs = ["ways-surface"]
files = ["temper/crates/temper-service/src/stock/ask.txt", "temper/crates/temper-service/src/tests/stock.rs", "temper/crates/temper-service/src/tests/ask_bound.rs", "temper/crates/temper-engine-smith/src/see.rs", "temper/crates/temper-engine-domain/src/route/see.rs", "temper/crates/temper-engine-domain/src/tests/see.rs", "temper/tests/client/src/front.rs", "temper/docs/design/domain/agent.status.md"]
effort = "medium"
minutes = 50
+++

## Goal

Give Ask temper's instructions M3a's part of Showing and `ways` on their
`see` line, as `temper/docs/design/draft/ask-temper.md`, section 8 has
them, taken as written, with `see`'s address line agreeing and `see ways`
answering that it is not there yet until task `ways-read` reads it.

Acceptance:

- `ask.txt` holds, between the `## Configuring` part and `## Your voice`,
  exactly these lines (a blank line before `## Your voice`, as between the
  file's other parts):

  ````text
  ## Showing

  Show a picture when it explains better than words. Draw what exists from a view, a fenced block naming it:
  ```view
  path OWNER/NAME main @revision N
  ```
  Views: path OWNER/NAME [BRANCH] · way NAME · charter N · proposal tN/K; a path or a charter takes @revision N, the newest when left out. Never draw by hand what a view shows.
  A way is drawn, never described: way NAME draws one of temper's stock ways over this project, as see ways lists them. A person who wants a way of their own gets a proposal, which draws its path before and after.
  ````

  and its `see` line has `ways · ` after `inbox · `. Nothing else of the
  file changes.
- `temper-engine-smith`'s `ADDRESSES` is the new `see` line; `see ways`
  answers `see ways: not there yet.`, an error, as the other addresses not
  built yet do.
- `ask_temper_is_charter_11_and_template_5` requires `## Showing` and
  still refuses `## Remembering`, and refuses in the Showing part the
  views M3b adds (`queue`, `task N tree`, `task N timeline`) and a
  `mermaid` block; the instructions stay within its 8,192 bytes
  (about 7,880 with this part).
- The first-request bound (`asks_first_request_fits_both_shapes`) holds
  16,384 on both shapes, and every test that pins charter 11's
  instructions or their room passes, `temper/tests/client/src/front.rs`'s
  `run_bytes` raised to what the instructions now take.
- `temper/docs/design/domain/agent.status.md`: the sentence "8.4: `ask.txt`
  has no Showing part." leaves its **Not yet** entry, the rest of it
  staying for `ways-read`.

## Inputs

- `temper/docs/plan/ui-ask-m3a/notes/common.md`.
- `temper/docs/design/draft/ask-temper.md`, section 8 (the whole text,
  whose Showing part this takes the first step of) and its 5.1;
  `temper/docs/design/domain/agent.md`, 8.4 (*Its instructions*) and
  8.4.1 (*Not yet*, *Unknown*).
- The tests that read `ask.txt` and `ADDRESSES`:
  `temper/crates/temper-service/src/tests/stock.rs`
  (`ask_temper_is_charter_11_and_template_5`,
  `stock_words_hold_no_placeholder_a_run_cannot_fill`),
  `temper/crates/temper-service/src/tests/ask_bound.rs`,
  `temper/tests/engine/domain/tests/suite/configuring.rs`
  (`an_asking_chat_reads_each_address`, `the_asking_chat_reads_each_kind`,
  `an_owners_asking_chat_proposes_and_hears_it_accepted`),
  `temper/crates/temper-engine-smith/src/tests/see.rs`,
  `temper/crates/temper-engine-domain/src/tests/see.rs`.

## Owns and may change

- **Owns:** `ask.txt`; the `ADDRESSES` constant in
  `temper-engine-smith/src/see.rs` (nothing else of that file); the parser
  in `temper-engine-domain/src/route/see.rs`, where `ways` joins the
  addresses answered `Address::NotYet`.
- **Tests:** the rows and asserts above; a parser row for `ways` in
  `temper-engine-domain/src/tests/see.rs`; `front.rs`'s `run_bytes` and
  its comment; any other test whose pinned room for charter 11's
  instructions the new part passes, raised to the measured figure, named
  in the report.
- **Options, pre-approved:** if the instructions pass 8,192 bytes, stop:
  that is a `Question:` with the measured size. If a journey recording
  goes stale (none holds `ask.txt` today), regenerate it.

## Steps

1. Edit `ask.txt` and `ADDRESSES`; add `ways` to the parser's not-yet
   addresses.
2. Edit the stock test as the acceptance says; run it; break the Showing
   part once (remove its Views line) to see the new assert fail, and
   restore.
3. Run the bound test and the engine domain world's configuring tests
   named above; raise `front.rs`'s `run_bytes` to the figure the client
   world's asking stories need (`heavy cargo nextest run -p
   temper-client-world --test suite -E 'test(/asking_chat/)'`).
4. The status sentence.
5. `cargo fmt`; `heavy cargo clippy -p temper-service -p
   temper-engine-smith -p temper-engine-domain -p temper-client-world
   --all-targets -- -D warnings`; the focused tests of `temper-service`
   (`--lib`), `temper-engine-smith` (`--lib`), `temper-engine-domain`
   (`--lib`) and `temper-engine-domain-world` (`--test suite -E
   'test(/configuring/)'`). Commit and push.

## Report

As the implementer's text asks; give the instructions' bytes and both
shapes' first-request figures as the bound test prints them.
