+++
id = "fresh-box"
part = "gate"
kind = "feature"
needs = ["stock-scale"]
files = ["temper/tests/engine/domain/tests/suite/fabrig_scale.rs", "temper/tests/engine/domain/tests/suite/main.rs", "temper/crates/temper-service/src/tests/welcome_policy.rs"]
effort = "high"
minutes = 150
+++

# A fresh box from `temper init` runs fabrig-scale work with no override

## Goal

The draft's test (`inputs/request.md`, "A test"): a fresh box from `temper init`, with no
override, starts; its `me` lists the stock charters; its stock budget funds 80 chats at once;
one goal's tree takes 384 tasks. Each part in the lowest world that shows it
(`temper/docs/design/domain/testing.md`, section 1): the start in the binary's suite, `me` in
the service's, the counts in the engine domain world over init's own document and the stock.

Acceptance:

- **It starts:** `temper/crates/temper/tests/suite/startup.rs`'s
  `a_box_on_a_fresh_init_reaches_serves_link_with_forge_3_dormant` (near 348), which runs the
  real `temper init` and `temper box` overriding only ports, passes as it is on this trunk (run
  it; it is not this task's file); and in `temper-service/src/tests/welcome_policy.rs`, a test
  `a_fresh_init_names_the_standard_profiles` reads init's generated document (the module's
  `super::stock::generated()` or its neighbours' helper) and asserts its three `"profile"` lines
  are `"standard"` and that its engine overrides hold none of jig's tasks', people's or
  authority's members but `tasks.inbox_bytes`, `tasks.message_bytes` and `tasks.result_bytes`
  (configuration.md, 13.4: the overrides say only what differs; the inbox's bytes follow the
  box's message bound, `init-standard`'s answers).
- **`me` lists the stock charters:** `temper-service/src/tests/welcome_policy.rs` (near 107-157)
  drives init's documents through the engine; extend it, or add
  `a_fresh_box_me_lists_the_stock_charters_at_fabrig_scale` beside it: after its project is
  made, `me`'s charters are the stock's in order (1, 2, 4, 6, and 3 where the stock holds it),
  each with its run as 13.4's table says (64 turns and 3,600 s for 1; 800 and 28,800 s for
  2, 4, 6).
- **Eighty chats at once:** a new suite file of the engine domain world,
  `temper/tests/engine/domain/tests/suite/fabrig_scale.rs` (`mod fabrig_scale;` in `main.rs`),
  over init's document's limits (`forge_routes`' `box_limits()`, which `init-standard` made read
  init's profiles) and the stock (`temper_service::stock`, as `b2_tree.rs`'s `stock_world`
  builds it): `eighty_chats_are_funded_at_once`: one project, its owner starts 80 chats from
  template 1, each admitted and funded its 24K from the owner's pool, all 80 live together; an
  81st is refused for the pool's funds, naming it (assert the refusal's kind, not its words).
- **A goal's tree of 384:** in the same file, `a_goal_tree_takes_384_tasks`: the owner starts
  a goal (template 2); the goal's runs delegate in batches (at most `tasks.batch` each, keeping
  live delegates within `tasks.delegates`, ending each batch's tasks before the next, nesting
  where the world's helpers make that simpler) until its tree holds 384 tasks, the goal among
  them; one more is refused naming `tasks.tree_tasks` (or the delegation's tasks, whichever the
  engine names first: assert which, and say so in the report).
- Each test selects and passes alone:
  `heavy cargo nextest run -p temper-engine-domain-world -E 'test(/fabrig_scale::/)'`,
  `heavy cargo nextest run -p temper-service --lib -E 'test(/welcome_policy/)'`,
  `heavy cargo nextest run -p temper --test suite -E 'test(/a_box_on_a_fresh_init/)'`.
  Each within its tier's CPU budget (`skein/docs/foundation/testing-strategy.md`, 8.1): note
  each test's time in the report; if one is over budget, it is red: ask, with the figure (never
  move it to another tier).

## Inputs

- `temper/docs/plan/fabrig-scale/notes/common.md`; `notes/contracts.md`, K2, K5 and K6.
- `temper/docs/design/protocol/configuration.md`, 13.4 (the table of the box's limits; the
  stock's charters, policy 1, the rules; "Budgets in runs of the default model").
- `temper/docs/design/domain/testing.md`, section 1 (where a test lives).

## Owns and may change

- The files of the header. Helpers the engine domain world already has (`temper/tests/engine/domain/src/`)
  are used, not changed; a small helper this file needs lives in it.
- If the world cannot make 80 live chats or 384 tasks for a limit the stock or init's document
  sets lower than 13.4's table, that is a defect of an earlier task: report it with the limit
  and stop (a question), never override it in the test.

## Steps

1. The profiles' test and the `me` test; run each; run the binary suite's fresh-box test.
2. The engine domain world's two stories; run them alone, then the world's suite.
3. `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `heavy cargo clippy -p
   temper-engine-domain-world -p temper-service --all-targets -- -D warnings`; commit and push.

## Report

As the implementer's text asks, with each test's time.

## Answers

### 2026-11-04: may this test admit init's `tasks.inbox_bytes` 874,120 override?

Yes. The acceptance above now names it; assert exactly those three task overrides, and that
`tasks.inbox_bytes` is `tasks.inbox_messages` × `tasks.message_bytes` (82 × 10,660). It differs
from the profile for a reason (`init-standard.md`'s answers: `standard`'s 21,495,808-byte inbox
passes the journal carrier, and the inbox's bytes follow the box's message bound; 13.4's table
leaves coupled limits sized the same way), so it is what "only what differs" allows. init's
document and the journal bound stay as they are.

Carry on with the remaining acceptance. Two things the rerun should expect:

- `startup/problems.rs`'s `clippy::semicolon_if_nothing_returned` is `stock-scale`'s file. Leave
  it under `## Left` unless the merged `stock-scale` has mended it; lint this task's own files.
- A worker's hello is checked against every configured charter of its kind
  (`jig_domain_fleet::needs::shortfall`), so your worlds' hello offers the stock's largest:
  `turns` 800, `time` 28,800 s, and `spend` what the refusal names. Set that in
  `fabrig_scale.rs` if the world's own bounds offer less. That is what an agent offers, not an
  override of a limit, so it does not fall under "never override it in the test".
