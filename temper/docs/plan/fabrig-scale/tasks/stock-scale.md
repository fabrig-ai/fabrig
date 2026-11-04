+++
id = "stock-scale"
part = "stock"
kind = "feature"
needs = ["brief-room"]
files = ["temper/crates/temper-service/src/stock.rs", "temper/crates/temper-service/src/startup.rs", "temper/crates/temper-service/src/startup/checks.rs", "temper/crates/temper-service/src/startup/problems.rs", "temper/crates/temper-service/src/tests/stock.rs", "temper/crates/temper-service/src/tests/startup.rs", "temper/docs/help/budget.md", "temper/docs/help/configure.md", "temper/crates/temper/tests/support/forge_land/rung.rs", "temper/crates/temper/tests/suite/startup.rs", "temper/crates/temper/tests/proof_live.rs", "jig/crates/jig-domain-core/src/tests/funding/settings.rs", "temper/docs/design/protocol/configuration.status.md", "temper/crates/temper-service/src/tests/fixtures/agent.rs", "temper/crates/temper-service/src/tests/fixtures/startup.rs", "temper/crates/temper-service/src/tests/fixtures.rs", "temper/crates/temper-playground/src/documents.rs", "temper/tests/engine/domain/src/protocol.rs"]
effort = "high"
minutes = 150
+++

# The stock runs fabrig-scale work: its charters' runs, its delegation and its pool

## Goal

temper's stock settings hold a project of fabrig's scale (contracts K6;
`temper/docs/design/protocol/configuration.md`, 13.4): a chat runs 64 turns in an hour, the
coding agent, its reviewers and its foremen 800 turns in eight hours; the rules, policy 1's
ceiling and role 0 delegate 384 tasks 32 deep; the owner's pool funds 80 chats or goals at once
(the ceilings and periods 80 × 24K); and the chat and goal templates' report is the
deployment's result bound, read at each start as C is. QA's walks then need no settings change
to fund their chats.

Acceptance:

- `temper-service/src/stock.rs`, `checked`:
  - charter 1 `charter(1, 64, …)` with a run time of 3,600 s; charter 2 800 turns; the
    reviewer's (4) and the foreman's (6) 800 turns; each of charters 2, 4 and 6 a run time of
    28,800 s; every reserve 1 turn and 60 s as today. If the stock holds charter 3 when you
    start (another plan's configuring agent), its run is charter 1's. Name the figures as
    constants beside the charter builders (`CHAT_TURNS`, `CHAT_TIME`, `WORK_TURNS`,
    `WORK_TIME`), each commented with the draft's row it holds (13.4's table).
  - K's floor stays four units a chat's run turn (now 256), the comment beside it saying so.
  - `chat_spend` stays 24K; `ceiling_spend` becomes 80 × 24K (the rules' ceiling and period,
    policy 1's ceiling and period, role 0's period); role 0's authority spend stays the chat's.
    Comment: eighty chats or goals funded at once from one pool.
  - `ceiling()`'s delegation: 384 tasks, 32 deep (was 18 and 4).
  - Templates 1 and 2: `Contract::Report { words }` with `words` the deployment's
    `tasks.result_bytes`: `checked` (and `Given::defaults`, `stock::defaults` if public) take it
    as an argument (`checked(llm, result_bytes: u32)`); `startup.rs`'s call passes
    `engine.limits.core.tasks.result_bytes`. The floor on `tasks.result_bytes` then holds by
    construction; keep it.
- `temper-service/src/tests/stock.rs`: the tests that pin the old figures (near 83-160,
  550-587: turns `[8, 16, 16, 16]`, 120 s, delegation 18 and 4, 48K, the K rows with C 7 giving
  K 32, the templates' report 32) name the new ones (turns `[64, 800, 800, 800]` and their
  times; 384 and 32; 80 × 24K; C 7 giving K 256; the report the limits' `result_bytes`); and a
  new test, `the_stock_pool_funds_eighty_chats_at_once`: at init's models (C 1,057, K 4,228),
  the rules' and policy 1's period spend and role 0's period spend are each at least 80 times
  the chat template's spend (role 0's authority spend), and less than 81 times.
- **QA's raised funding:** `temper/crates/temper/tests/support/forge_land/rung.rs` (near 560-576)
  builds a settings change raising the old stock's 48K ceiling (207,552 at a C of 1,081) so a
  walk's chats fit: it is no longer needed; remove that settings change and the step that
  applies it, keeping every other step (if a caller needs the step to exist, make it apply no
  change and say so). `temper/crates/temper/tests/suite/startup.rs:48`'s `"amount":207_552` is
  a parser fixture: leave it. `temper/crates/temper/tests/proof_live.rs` (ignored, offline
  proof, near 13-56): where it raises the stock's funding the same way, remove the raise as in
  `rung.rs`; check it builds (`heavy cargo check -p temper --tests`). jig's
  `jig/crates/jig-domain-core/src/tests/funding/settings.rs` (near 46-52, 109-118) tests jig's
  own raised settings over jig's own defaults: leave its figures unless its comments say they
  stand for temper's stock (then report it and change nothing).
- `temper/docs/help/budget.md`: the bullet that starts `- **A run** draws what its task has
  left` becomes exactly:

  > - **A run** draws what its task has left, up to the deployment's run
  >   cap; its charter (temper help charter) sets its turns, its time and
  >   a reserve to finish in. The stock sizes its spends at each start
  >   from C, one costliest completion of the first configured model at
  >   its prices: its rules' floor is 2C and their run cap K, 4C or four
  >   units for each of the chat's run turns (256 for its 64), whichever
  >   is more; a person's chat or goal may spend 24K, and the ceilings
  >   and a project's period 80 × 24K, so eighty chats or goals run at
  >   once from one pool. On `temper init`'s box C is 1,057 units, so a
  >   run draws 2,114 to 4,228. The stock chat runs 64 turns and an hour;
  >   the coding agent, its reviewers and its foremen 800 turns and eight
  >   hours, each with a reserve of 1 turn and 60 seconds; a run is held
  >   at its spend long before its turns are spent, and its person funds
  >   more. `settings show` reads the current ones.

  (without the `> ` quoting, its lines as given). Nothing else of the page. Run temper's help
  tests (`rg -l 'docs/help' temper/crates --type rust` finds them; `heavy cargo nextest run -p
  temper-docs` at least).
- **The task row's words** (jig's `protocol/store.md`, 5.4: since `tasks-pieces` the largest
  task row grows with a batch's bytes and the inbox, not with `tasks.batch`):
  - `temper-service/src/startup/checks.rs` (near 1143-1156) attributes a tasks row's value
    refusal to `tasks.batch_bytes` in place of `tasks.batch` (`What::StoreTask`'s field
    renamed `batch_bytes`), and `startup/problems.rs`'s `store_task` (near 346) reads
    `; fixed profile ceiling engine.limits.store.value; reduce tasks.batch_bytes,
    tasks.inbox_messages or the task record's spec/message/result/authority capacities; this
    ceiling has no scalar override`; `src/tests/startup.rs`'s row that asserts those words
    names the new ones.
  - `temper/docs/help/configure.md`: the sentence ``\`tasks.batch\` grows the stored task
    record; reduce the named capacity or its spec/result/authority sizes when it exceeds that
    ceiling.`` becomes ``A batch's bytes, `tasks.batch_bytes`, and the inbox's messages grow
    the stored task record; reduce the named capacity or its spec, message, result and
    authority sizes when it exceeds that ceiling.`` Nothing else of the page (another plan,
    `configuring-p3`, edits it too: keep the hunk to that sentence).
  - `temper/docs/help/budget.md`: in the overrides table, after the `tasks.batch` row, a row
    `| \`tasks.batch_bytes\` | A batch's bytes in all |`; the sentence ``At a schema ceiling or
    profile store ceiling, reduce `tasks.batch` or spec/result/authority sizes.`` becomes ``At a
    profile store ceiling, reduce `tasks.batch_bytes`, `tasks.inbox_messages` or the spec,
    message, result and authority sizes.``
- `configuration.status.md`: from its first **Not yet** entry, the clause "13.4: the stock's
  runs are 8 and 16 turns and 120 s, ... its delegation is 18 tasks 4 deep, its ceilings and
  periods 48K, and its templates' report 32 bytes (`temper-service/src/stock.rs`)." removed
  (what `brief-room` removed of it stays removed).
- `heavy cargo nextest run -p temper-service --lib -E 'test(/tests::stock::/)'` passes; then
  `-p temper-service`; the engine domain world's tests that start from the stock
  (`heavy cargo nextest run -p temper-engine-domain-world -E 'test(/forge_routes::/)'`,
  `b2_tree` among them); `heavy cargo nextest run -p temper --test suite -E 'test(/startup::|forge_land/)'`;
  `heavy cargo check -p temper --tests`.

## Inputs

- `temper/docs/plan/fabrig-scale/notes/common.md`; `notes/contracts.md`, K6.
- `temper/docs/design/protocol/configuration.md`, 13.4 (the table's charter, template, policy 1
  and rules rows; "Budgets in runs of the default model"; "The box's limits for that work").

## Owns and may change

- The files of the header, for what acceptance names.
- **A change of meaning:** every world started from the stock gains longer runs, a deeper
  delegation and a larger pool. Run the tests of each package that starts from it (above); a
  test whose figure was the old stock's (a held spend, a run's 8 turns) changes its expected
  figure only, listed in the report; a test that fails otherwise is a question.
- The stock's instructions (`stock/*.txt`) are not yours.
- If another plan's charter 3 or template 3 is in the stock when you start, keep them, with
  charter 3's run as charter 1's.

## Steps

1. The charters' runs, K's floor comment, the pool, the delegation; the templates' report and
   `startup.rs`'s call; the stock tests and the new one.
2. `rung.rs`'s funding step and `proof_live.rs`'s; the help bullet; the task row's words.
3. The packages of acceptance, one at a time.
4. The status line; `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `heavy cargo clippy
   -p temper-service -p temper --all-targets -- -D warnings`; the citations check; commit and
   push.

## Report

As the implementer's text asks, with every test figure you changed.

## Answers

### 2026-11-04: may this task raise the test agents' advertised bounds so the stock's runs are admitted?

Yes, option A. A fixture's `jig_domain_host::Bounds` is what its agent offers, not a stock
figure; `admission` and the fleet's `fits` (`jig/crates/jig-domain-fleet/src/needs.rs`) compare
it with the stock's charters, so acceptance cannot hold while it offers less. The task's
acceptance needs it, and no other task owns it. This widens "Owns and may change" as follows:

- **What moves:** only what an agent offers, and only in a fixture that fails on the stock
  (a startup refusal `What::Admission`, or a stock run no worker fits): `turns` 800; `time`
  28,800 s; `spend` the required figure the refusal names, at least the K of that world's
  models. Nothing else of a fixture: not its `waiting`, `checks`, `message_bytes`, `inbox`,
  carriers or model bounds, and no stock figure.
- **Known sites:** `temper-service/src/tests/fixtures/agent.rs`, the agent `startup_agent` and
  `worker_agent` derive from (turns 16, 300 s, spend 100); `tests/fixtures/startup.rs`'s
  `owned_worker_agent`; `tests/fixtures.rs`'s linked worker (near 950); and
  `temper-playground/src/documents.rs`, whose `documents::agent()` starts stock worlds:
  `agent_profile`'s `policy.max_turns` 800 (no carrier derives from `max_turns`) and its
  `Bounds`' `spend` and `time`. Further sites are found by running the packages of acceptance,
  and `temper-playground`'s tests; `temper/tests/engine/domain/src/protocol.rs`'s `bounds()` is
  the likely next one, for the worlds that start from the stock.
- **What stays:** a fixture whose world brings its own charters (`temper/tests/engine/domain/src/walking.rs`'s,
  for one) and passes as it is; a test that sets a bound to assert a refusal
  (`jig/crates/jig-domain-fleet/src/tests/needs.rs`). jig is not touched: it names no stock.
- **A test that asserts an offered figure** from a raised fixture changes that figure only; list
  every raised bound and changed figure in the report.
- **Budgets:** a test that now drives a stock run to its 64 or 800 turns, or past its CPU budget
  or timeout, gets no raised budget and no lowered stock. It sets its own charter through a
  settings change where it means a short run; otherwise, stop and ask with the measurements.
- **Still a question:** a failure that no offered bound repairs (a carrier, a store row, a hub
  limit refusing the stock's figures). The production agent needs no change:
  `temper/crates/temper/src/agent.rs` offers the profile's policy (`max_turns` 800, spend and
  time at smith's defaults of 1,000,000,000 and 24 h).
