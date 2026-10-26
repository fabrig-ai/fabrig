# Supplement jig-15.s1: a decision's facts in `facts`, their words in `words`, the check in the application's words

Added by ruling 394 (the designer's answer of 2026-10-18T19:47Z,
point 3). It also carries ruling 392's transport change. Read first:
jig's `AGENTS.md`; `docs/design/client/domain/client.md`, 19;
`testing.md`, 6.2 ("the referee says each fact as `words` says it in the
phrasebook the application's world gives it … The referee holds no words
of the reference UI's"); `view.md`, 7.

Scope:
- `jig_client::facts` gives a decision's facts as data, from a document
  and an act: what the referee reads today, moved there.
- `jig_client::words` says them.
- jig-client-view says them from `words` and drops its own.
- The referee, and the joint transport, lose `jig-client-view`.

Needs: nothing. It runs beside jig 20.5, off its chain.

Files:
- `crates/jig-client/src/{facts.rs, words.rs, words/}`;
- the phrase tables of `jig-client-view` that hold the 26 phrases, and their callers there:
  - `owner_proposal_words.rs`;
  - `proposed_task_words.rs`;
  - `phrases/jig.rs`, with `phrases/english.rs`;
- `tests/client/src/harness/referee.rs`:
  - `:17-18`, the `jig_client_view` imports;
  - `:453-533`, the checks;
  - `effect_words` `:535`, `funder_words` `:654`;
- `tests/client/src/harness/world.rs` (`Settings`, `:23`);
- the 28 lines naming `jig_client_view::Limits` in `tests/client/src/joint/`, 26 files: `packets.rs`, `read_packets.rs`, `requests.rs` and `wire/*.rs`;
- `tests/client/src/test.rs`.

Users found by `git grep -n -e jig_client_view -- tests/client/src`.

Shape:
- **The facts:** `facts` gives the facts `referee.rs:453-533` reads.
- **The words:** `words` says them: the 26 phrases the referee says, their English unchanged, counted in `PHRASES` (58 at `2bd1a5cbb`) and `ALL`.
- **The referee:** it says them in the phrasebook a new `Settings` field gives (default `words::english()`).
- **The transport:** `document`, `stream`, `people_reply`, `read_with_codec` and the rest take `&jig_client::DomainLimits` (`jig_client_domain::Limits`, `lib.rs:13`) in place of `jig_client_view::Limits`. If a bound they read is in the view's `Limits` alone, block with its field.

Test shape: a world given an override holds a tree in it, and faults on a
tree in jig's English. Each test under 20 ms (unit) or 0.5 s (world).

Proof: jig's gate.

Size: measure it first, by file, in the first report. Aim at most 400
production lines added; past 600, block with the count.

Don't: change an English text, a tree or a golden of jig's; move any
other phrase.
