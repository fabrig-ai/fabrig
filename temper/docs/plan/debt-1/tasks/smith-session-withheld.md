+++
id = "smith-session-withheld"
part = "smith-withheld"
kind = "interface"
needs = []
files = [
  "smith/crates/smith-domain-session/src/record.rs",
  "smith/crates/smith-domain-session/src/boundary.rs",
  "smith/crates/smith-domain-session/src/lib.rs",
  "smith/crates/smith-domain-session/src/domain.rs",
  "smith/crates/smith-domain-session/src/session.rs",
  "smith/crates/smith-domain-session/src/session/restore.rs",
  "smith/crates/smith-domain-session/src/tests.rs",
  "smith/crates/smith-domain/src/route.rs",
  "smith/crates/smith-domain/src/peer.rs",
  "smith/tests/session/src/recorded.rs",
  "smith/tests/session/src/world.rs",
  "smith/tests/session/tests/suite/**",
  "smith/crates/smith-agent-service/src/tests/line.rs",
  "jig/tests/channel/tests/suite/source_carry_over.rs",
  "smith/docs/design/domain/session.status.md",
  "smith/docs/design/STATUS.md",
  "smith/docs/development/debt/D004-session-knows-wait.md",
  "smith/docs/development/debt/README.md",
]
effort = "high"
minutes = 120
+++

# smith D004: the opening names the withheld call, and one answered event

## Goal

The session stops knowing the run's `wait` tool: the opener names in its `Opening` the one
call it may have withheld, and the session's restore matches that name, never `b"wait"`
(`smith/docs/design/domain/session.md`, 1 and 3; 3's sentence written by this cut). Beside
it, `Event::Answered` and `Event::AnsweredAndYield` become one event with a field for what
follows. Behaviour is unchanged. This pays
`smith/docs/development/debt/D004-session-knows-wait.md`.

Neither type is stored or crosses a wire (the cut's survey): `record::Opening` and `Event`
derive only `PartialEq, Eq, Debug` and are built in-process by the root, `smith-domain`
(`route.rs`); `smith-transcript`'s generated `Opening` is another type and does not change.

**How the cut reads Paid when:** "no code in `smith-domain-session` matches a tool name" is
read as no name of an opener's tool. `session/batching.rs`'s `b"job"` (in `other_call`)
names the session's own tool, which `smith-protocol-llm` decodes, and stays.

Acceptance:

- `rg -n 'b"wait"' smith/crates/smith-domain-session/src -g '!tests.rs' -g '!tests/**'`
  finds nothing.
- `rg -n 'AnsweredAndYield' skein smith jig temper -g '!**/docs/plan/**'` finds nothing.
- The tests in Steps pass, each count above zero, with no assertion changed: test edits are
  the new field in `Opening` literals and `Answered`'s `after`.
- `session.status.md` says `No gap.`; smith's `STATUS.md` lists `domain/session.md` among
  the documents with no gap again; D004's file and its README row are gone;
  `tools/citations/citations.py --all` reads 0; clippy is clean.

## Inputs

- `temper/docs/plan/debt-1/notes/common.md`.
- `smith/AGENTS.md` (its module-doc rules and `clippy.toml`'s bans).
- `smith/docs/development/debt/D004-session-knows-wait.md`, whole.
- `smith/docs/design/domain/session.md`, 1 (a session knows nothing of runs) and 3 ("Told as
  it ends", "Opening from a transcript", whose last sentence this cut wrote: the opener names
  the one call it may have withheld, and the session restores that call alone as withheld).
- `smith/docs/design/domain/run.md`, 6 (a run parks with its `wait` withheld and answers it
  when it opens the conversation again), read only.

## Owns and may change

`smith-domain-session` (the files in the header): `record.rs`, `boundary.rs`, `lib.rs`,
`domain.rs`, `session.rs`, `session/restore.rs` and its unit tests (`src/tests.rs`).

Reader and neighbour files, each edited only as said:

- `smith/crates/smith-domain/src/route.rs`: the `Opening` literal (~1038) gains `withheld`
  (below); the `Due::Answered` and `Due::Yielding` arms (~274-279) build one
  `Event::Answered` with `after: After::Complete` and `After::Yield`. `Due` stays.
- `smith/crates/smith-domain/src/peer.rs`: if the run's `wait` name is two literals there
  (~199, ~401), one private `const` beside them that `route.rs` also uses; no new public item.
- `smith/tests/session/src/recorded.rs`, `src/world.rs` and `tests/suite/**` (smith's session
  world): `Opening` literals gain `withheld`; every `Event::Answered` literal gains
  `after: After::Complete`; each `Event::AnsweredAndYield { .. }` becomes
  `Event::Answered { .., after: After::Yield }`; readers that match both (`world.rs` ~1720
  and ~1755, `recorded.rs` ~171) match the one event, and the trace keeps its two texts
  ("answered …", "answered and yield …") by `after`, so no trace changes.
- `smith/crates/smith-agent-service/src/tests/line.rs` (~312) and
  `jig/tests/channel/tests/suite/source_carry_over.rs` (~47): their `Opening` literals gain
  `withheld: None`.
- `smith/docs/design/domain/session.status.md`: its **Not yet** entry goes; it says
  `No gap.` under today's date line.
- `smith/docs/design/STATUS.md`: delete the sentence "A restored session knows the run's
  `wait` by its name, which its opening does not give (`domain/session.md`)." and put
  `` `domain/session.md`, `` back at the head of the list of documents that have no gap,
  before `` `protocol/charter.md` ``.
- `smith/docs/development/debt/D004-session-knows-wait.md` (delete) and its row in
  `smith/docs/development/debt/README.md`.

Pre-approved:

- **Which literals name `wait`.** The run's opening names it always, whether or not its
  charter grants `wait` now, so that a transcript parked under an earlier charter restores as
  today. The session world's helpers that stand in for the run's opening
  (`recorded.rs`'s `opening()`, `world.rs` ~1543) name `wait` too; every other literal
  (unit tests, `line.rs`, jig's) gives `None`. If a test then fails only for want of the name,
  name `wait` in that literal and list it.
- **The name's life.** The session uses `withheld` while it opens (restore) and keeps none of
  it afterwards. No new byte bound: the root gives the name. If an existing admission measure
  of the opening counts its heap bytes and a test pins that measure, count the name as that
  measure counts the transcript's call names, and say so.
- **`unrun`** may compare the withheld call's id (`Resumed::WaitWithheld { id }` carries it)
  rather than its name; naming the variant afresh (it says `Wait`) is yours, private.
- If any other crate in any tree constructs or matches `record::Opening` or the two events
  (the survey found none beyond the list), edit it the same way and list it.

## Steps

1. Sketch of the outer layer:

   ```rust
   // record.rs
   pub struct Opening {
       // ... the fields as today, then:
       /// The name of the one call this opener may have withheld when the conversation
       /// last closed (session.md, 3), restored as withheld; `None` withholds none.
       pub withheld: Option<Box<[u8]>>,
   }

   // boundary.rs, re-exported from lib.rs beside `Event`
   /// What the session does once a delegated call's answer is in.
   #[derive(Clone, Copy, PartialEq, Eq, Debug)]
   pub enum After {
       /// Ask for the next completion.
       Complete,
       /// Yield to the opener first, once the tool turn settles, so that it can append work.
       Yield,
   }

   pub enum Event {
       // ...
       /// Parent terminal for Delegate, possibly winning a Withdraw race (today's docs of
       /// `text`, `error` and `spent` kept).
       Answered { owner: Token, text: Box<[u8]>, error: bool, spent: u64, after: After },
       // `AnsweredAndYield` is gone.
   }
   ```

   `session.rs`'s `pub(crate) enum After` moves to `boundary.rs` as above (one type, not two);
   `domain.rs` has one arm, passing `after` into `Bill`. `open` hands `withheld` to `restore`,
   which hands it to the checks that today compare `b"wait"` (`validate_results`,
   `resumed_tail`); a call in the tail is withheld when its name equals `withheld`, and none
   is when `withheld` is `None`. The module doc's list of a `Delegate`'s terminal events
   (`boundary.rs` ~19) names three.
2. Narrow tests, one at a time, each count above zero:
   - `heavy cargo nextest run -p smith-domain-session --lib`
   - `heavy cargo nextest run -p smith-session-world --test suite`
   - `heavy cargo nextest run --profile fuzzy -p smith-session-world --test suite`
   - `heavy cargo nextest run -p smith-domain --lib`
   - `heavy cargo nextest run -p smith-agent-service --lib -E 'test(/line::/)'`
   - `heavy cargo nextest run -p jig-channel-world --test suite -E 'test(/source_carry_over/)'`
   The session world's restore stories are the ones that pin `wait`
   (`a_parked_wait_preserves_other_results_and_resumes_only_the_missing_result`,
   `a_generic_yield_with_two_unstarted_wait_calls_restores_both_as_not_run`,
   `partial_wait_mutations_are_refused`) and the yield story
   (`a_delegated_result_can_yield_after_its_seen_turn_before_the_next_provider_request`):
   check they ran.
3. `cargo fmt`; `python3 -I -B scripts/old_forms.py` from `smith/` and from `jig/`;
   `heavy cargo clippy -p smith-domain-session -p smith-domain -p smith-session-world -p smith-agent-service -p jig-channel-world --all-targets -- -D warnings`.
4. smith's design Markdown is read by tests (`.config/gate/read-by-tests.txt`:
   `smith/docs/design/**`): the benchmarks check their tasks' section guards against it
   (`smith/benchmarks/src/check.rs`, `check_guard`). This branch edits only two status files,
   which no guard names; if you find a test that reads them, run it.
5. Status and debt text as in "Owns"; `python3 -I -B tools/citations/citations.py --base
   origin/main` and `--all` from the repository root.
6. Two commits ("the opening names the withheld call, and one answered event: the session
   knows no tool of its opener's (session.md, 3)"; "pay smith D004: session's status and the
   item"), pushed.

## Report

As the implementer's text says, and also: every literal that names `wait` and why; any
reader beyond the list; the four named stories' results.
