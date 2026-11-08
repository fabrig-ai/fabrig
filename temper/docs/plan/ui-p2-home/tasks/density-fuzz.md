+++
id = "density-fuzz"
part = "density"
kind = "feature"
needs = ["all-home-reads"]
files = ["temper/tests/client/tests/suite/fuzzy/density.rs"]
effort = "high"
minutes = 140
+++

## Goal

A fuzz test of `temper/docs/design/draft/surfaces.md`, section 12's
rules on Home's, Home on All projects' and Needs you's models, as
`temper/docs/design/client/README.md`, 3.5, *A page's parts* holds them:
over random facts and random moves of a project's settings, each part
shows exactly when its rule holds, none shows empty, every open card is
placed exactly once, and a settings move never hides an open card.

Acceptance:

- `fuzzy::density::…` in `temper-client-world`'s suite, run by the fuzzy
  profile (`heavy cargo nextest run -p temper-client-world --profile fuzzy
  -E 'test(/^fuzzy::density/)'`), seeds `1..=N` with N chosen so the test
  stays under 2 CPU seconds, a failing seed named in its message and
  replayed by it alone.
- Each seed builds a client root signed in with one to four projects and
  steps it by random deliveries and acts, the way `fuzzy::client` steps
  its world: statuses per project (landed, seen, previous, a brief new or
  old), inbox cards across projects (a question, a proposal, a held task, a
  funds card), `approvals` rows, decisions within and past their day,
  answers through the held send and its deadline, the address moved
  between `AllHome`, `Home { project }` and `Inbox` (filtered and not), and
  a settings revision of a project's way between You approve and Agents
  review. After each step it checks, on the open page's model
  (`all_home::facts`, `home::facts`, `parts::needs_facts`, the needs
  session's open cards):
  - each `parts::home` and `parts::needs` field equals its rule over
    those facts (contracts, section 3);
  - a part shown holds something: Needs you's with an open card, Decided
    for you's with a row, the brief's with a line, Projects' with two
    rows; a part hidden holds nothing the person would miss;
  - every open card's object is in the open set once, the set's count is
    the parts' `open`, and Home's three cards plus its "N more" count it;
  - a settings revision leaves the open set as it was, an approval asked
    before it still open after it.
- Breaking a rule by hand (a part shown on `>= 0`, a card counted twice)
  fails a seed; the report says which.

## Inputs

- `temper/docs/design/client/README.md`, 3.5, *A page's parts*, *Home on
  All projects*, *Needs you*, *What the client keeps*.
- `temper/docs/design/draft/surfaces.md`, 12 and 14 (the fuzz test).
- `skein/docs/foundation/testing-strategy.md`, section 1, and its
  sections on seeds and replays.
- `temper/docs/plan/ui-p2-home/notes/contracts.md`, sections 2 and 3.
- `temper/docs/plan/ui-p2-home/notes/common.md`.

## Owns and may change

- Owns `temper/tests/client/tests/suite/fuzzy/density.rs` (new) and its
  `mod density;` line inside `main.rs`'s `mod fuzzy { … }`.
- Its helpers stay in `density.rs`: `joint-across` changes the shared
  fixtures beside it, so this task changes none of `src/fixtures.rs`,
  `src/front.rs` or another story's file. A helper it needs from them it
  copies into `density.rs`, and says so.
- Options, each pre-approved: if a root stepped without the engine is
  the faster way to the models, use it (the deliveries as the domain's
  tests build them) and say so; if a settings revision needs the engine,
  deliver the `settings` document's new revision instead.

## Steps

1. Read `tests/suite/fuzzy/client.rs`, `fixtures::world`, and the
   domain's documents as `temper-client-domain`'s `src/tests/pages/home.rs`
   builds them.
2. The generator, the step loop and the four checks, one function each.
3. Break each check's rule once and see a seed fail; restore.
4. `cargo fmt`; `heavy cargo clippy -p temper-client-world --all-targets
   -- -D warnings`; the fuzzy run above, then `heavy cargo nextest run -p
   temper-client-world` (focused). Commit and push.

## Report

The generator's moves, the checks, N and the CPU seconds, each broken
rule and the seed that caught it, and your implementation and check
minutes apart.
