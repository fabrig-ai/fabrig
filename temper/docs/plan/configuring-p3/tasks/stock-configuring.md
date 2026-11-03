+++
id = "stock-configuring"
part = "configuring-agent"
kind = "feature"
needs = ["tools-offered", "tools-store"]
files = ["temper/crates/temper-service/src/stock.rs", "temper/crates/temper-service/src/stock/configuring.txt", "temper/crates/temper-service/src/tests/stock.rs", "temper/crates/temper-service/src/tests/stock_brief.rs", "temper/docs/design/domain/agent.status.md", "temper/docs/design/protocol/configuration.status.md"]
effort = "high"
minutes = 180
+++

# stock-configuring: charter 3 and template 3 in the stock

## Goal

The binary's stock holds the configuring agent (temper's `protocol/configuration.md`, 13.4
and 13.6's step 3; `domain/agent.md`, 8.1): charter 3 with its instructions, template 3 that
starts it, the settings family in policy 1 and the rules, their delegation to charter 3, and
each stock charter's own `tools` (contract C1), so that charter 3's runs alone are offered
`settings` and `propose_settings`. This changes what the stock means: every world started
from it gains a charter and a template, a project's first web chat starts template 3
(`temper-client-domain/src/pages/chats.rs`, `first_chat`, already written for it), and a
chat's runs lose the settings tools their authority now holds; so this task runs the tests of
every package that starts from the stock, and fixes their limits and expectations as below.

Acceptance:

- `stock.rs`: the charters, in order, 1, 2, 3, 4, 6; charter 3 is `charter(3, 8,
  include_bytes!("stock/configuring.txt"), llm)` with `modify` and `agents` false (as
  `reviewer()` sets them), every other member charter 1's; each charter's `policy.tools`:
  charter 1 `1023`, charter 2 `514`, charter 3 `1536`, charters 4 and 6 `512` (the numbers of
  13.4's "Each charter's host families"), as named constants beside `CHAT_TOOLS`.
- Template 3, `configuring()`: `number` 3, `name` `configuring`, `executor`
  `TemplateExecutor::Charter(3)`, every other member `chat()`'s (contract, shallower, budget,
  deadline, tracked, flags, label, grace, llm). Templates in order 1, 2, 3.
- `CHAT_TOOLS` becomes `1 | 2 | 512 | 1024` (1,539), its doc comment naming Settings (jig's
  settings family: the configuring chat reads the settings and proposes); the ceiling's
  delegation kinds gain `Executor::Charter(3)` after `Charter(2)`.
- `stock/configuring.txt` holds exactly the text below (3,058 bytes), ending with its line end.
- Step tests in `temper-service/src/tests/stock.rs`, updated and added: the stock's
  numbers, tools, delegation, floors (`tasks.charters` 5, `core.templates` 3,
  `authority.executors` 8, `core.policy_bytes` re-measured) as the survey below lists; a new
  `the_configuring_agent_is_charter_3_and_template_3` holding charter 3's instructions equal
  to `configuring.txt`, its `tools` 1,536, `modify` and `agents` false, and template 3's
  executor and flags; `stock_brief.rs`' charter list gains `(3, false)` and every case reads
  the charter's own `policy.tools` where it faked the authority's families for charters 4 and
  6 (`assignment()`), each charter admitted whole, its figures printed as now.
- In the engine's domain world, `suite/settings/b2.rs`' `changes_without_settings_or_message`
  holds the new meaning: with settings in the ceiling, roles and rules, a stock chat (charter
  1) is offered no settings tool and a charter-3 run is; `suite/settings.rs`'
  `a_run_reads_the_settings_and_proposes_a_change_an_owner_accepts` runs its run under charter
  3 (template 3), since a chat's charter no longer names the family.
- temper's `docs/design/domain/agent.status.md`: the two sentences "4.3: the `settings` family
  offered only where the charter's `tools` names it too; the task's authority alone decides
  (`temper-engine-smith/src/calls.rs`)." and "4.3, 8.1: the configuring agent: the stock's
  template and charter 3." removed; `docs/design/protocol/configuration.status.md`: the
  sentence "13.4, 13.6 step 3: charter 3, template 3, the rules' delegation to charter 3, and
  the settings family in policy 1 and the rules (`temper-service/src/stock.rs`'s `CHAT_TOOLS`
  is 515, not 1,539)." removed. Each entry's other text stays.
- The focused tests of `temper-service`, `temper`, `temper-init`, `temper-engine-smith`,
  `temper-engine-domain`, the engine's domain world, the system world, the real-loop world,
  the one-process world, `temper-playground` and `temper-client-domain` pass.

The instructions, `temper/crates/temper-service/src/stock/configuring.txt`, verbatim:

  You configure temper for the person you talk with: you turn how they say they work into one proposal of settings, which an owner of the deployment accepts. You change nothing yourself.
  
  Read first. Call settings with no member for the inventory and the newest revision, then read whole each setting their description touches. When they point to a document, read it with read_file at the path they give, in the repository they name; when they paste a text, work from that.
  
  Sort each sentence of their description with one test: would temper's engine act differently without it? A sentence that changes what temper wakes, schedules, permits, pays for, lands or holds is structure: it becomes settings (a charter, a template, a policy, the rules, a landing rule, a forge charter, a project's policy). A sentence that only changes how an agent behaves is a rule: it stays text that runs read. A rule written in a document of an adopted repository stays there: name that document in the charters of the roles it concerns, so that a change landed there reaches their next runs. A rule typed in this chat goes into those charters' instructions, in the person's words.
  
  Ask until you can propose, one question at a time. Where temper's settings can hold a rule as structure, offer it and say what each choice guarantees: "I can make each family's changes wait for its designer, as dependencies temper holds; 'push early' stays a rule your producers follow." The person chooses; never make a rule structure without their choice.
  
  Say what temper's settings cannot say: name it, and offer the nearest setting there is, never a stand-in as if it were what they asked. Say what needs a restart, and never propose it: a provider, a forge, a listener, a worker or a limit is a member of the configuration document, which whoever runs the deployment sets.
  
  Propose once, with propose_settings, against the newest revision you read, each item a setting whole as settings showed it. The reason's first line is the proposal's title; then, under "temper enforces", each change of structure in the person's words; under "your agents follow", each rule and where it comes from; then what the change does to running work. Tell them the proposal's number, and that it waits for an owner.
  
  You hear its outcome in your messages. Accepted: say so in one line, with the revision it made. Rejected: take the reason as the person's words, and revise. Withdrawn as stale: read the settings again and propose afresh, saying what changed.
  
  Explain from the settings and from temper's pages, with read_docs, naming the page and never its sections or offsets: settings, revision, proposal, charter, template, policy and landing-rule for the concepts, each with what it cannot do and what needs a restart; settings-fields for each setting's fields, bounds and defaults; configuration and configuration-engine for the document's members; limits and limits-engine for the limits. Asked, describe the settings back as a workflow: who does what, under which role, and what temper holds them to.

## Inputs

- `temper/docs/design/protocol/configuration.md`, 13.4 (the stock's rows; "Each charter's
  host families"; the configuring agent as a setting) and 13.6, step 3.
- `temper/docs/design/domain/agent.md`, 4.3 (the settings family by the charter's `tools`)
  and 8.1 (its instructions, tools and kinds).
- `temper/docs/design/client/README.md`, 3.5 (the first chat; the client code is written).
- `temper/docs/plan/configuring-p3/notes/contracts.md`, C1.
- `temper/docs/plan/configuring-p3/notes/common.md`.

## Owns and may change

- Owned: the files in this spec's `files`.
- **Readers: a stock of five charters and three templates.** The cut's survey found these;
  edit each for the new count or member alone, and list every file you change:
  - Limits of 4 charters (and 2 templates) where a start takes the stock: raise
    `tasks.charters` to 5 and `core.templates` to 3 (never below what the document had):
    `temper-service/src/tests/fixtures.rs` (`ENGINE`) and `root_checks.rs`'
    `ENGINE_OVERRIDES` if it sets them; `temper/crates/temper/tests/support/configuration.rs`,
    `github_land.rs`, `forgejo.rs`, `case.rs` (its `max(held, 4)`);
    `temper/crates/temper/tests/fixtures/cluster/engine.json` and
    `temper/crates/temper/tests/support/github/engine.json`; the engine's domain world's
    `src/forge_routes.rs` (`STOCK_CHARTERS` 5, `STOCK_EXECUTORS` 8) and
    `tests/suite/settings.rs`' `stock_limits`.
  - Lists of the stock's numbers or counts: `temper-service/src/tests/checks.rs` (the
    "charter N.model" lines gain charter 3), `temper/crates/temper/src/tests/settings_fields.rs`
    (`STOCK_VALUES` 10, its comment), `temper/crates/temper/src/tests/reference_settings.rs`
    (the stock count 5), `temper-init/src/tests/documents.rs` (the stock charters `[1, 2, 3,
    4]`, the assertion's list with 3).
  - The real loop's helper charter is numbered 3 (`temper/tests/real-loop/src/case.rs`
    `HELPER_CHARTER`, `case/defaults.rs`, `fixtures.rs`' assertion): renumber it 9, its
    constant and the assertion alone.
  - **The playground (playground-browser's crate):** `temper-playground/src/documents.rs`'
    `given_with` takes the stock as its own defaults. Leave charter 3, template 3 and the
    rules' and policy 1's `Executor::Charter(3)` out of the defaults it builds there, in
    that one function, with the line `// The playground's scripted model plays the stock
    chat; the configuring agent needs a real one (configuration.md, 13.4).`, so its settings
    and its first chat stay as they are; touch no other playground file. If the playground
    also starts from `Given::Stock` somewhere, raise that document's `tasks.charters` to 5
    instead, and say so.
  - A test that walks the web client's first chat on the stock and now meets template 3:
    if it is about the first chat itself, expect template 3; otherwise stop and report it.
- Not yours: the help pages that describe the stock in prose (`temper/docs/help/charter.md`,
  `template.md`, `settings.md`, `chat.md`): list what they now say wrongly in the report; a
  later task of this plan changes them.
- Options, pre-approved:
  - `core.policy_bytes`' pinned 3,266 moves by one executor: re-measure and pin the new figure.
  - If the stock's floors refuse a document this task did not list because it holds 4
    charters, raise it as above and list it.
  - If `stock_brief.rs` shows charter 3 cannot take its whole maximal brief, leave the
    assertion as `Admitted` with fitting, and report its figures.

## Steps

1. `stock.rs`: the masks, charter 3, template 3, `CHAT_TOOLS`, the delegation;
   `stock/configuring.txt`.
2. `heavy cargo nextest run -p temper-service --lib -E 'test(stock)'`, then the readers above,
   one package at a time, each fixed as listed.
3. The status sentences.
4. `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `heavy cargo clippy` naming every
   package you changed, `--all-targets -- -D warnings`; the focused tests of the acceptance's
   packages, one command per package group; the citations check.
5. Commit ("temper: the configuring agent in the stock, charter 3 and template 3
   (configuration.md, 13.4)") and push.

## Report

As the implementer's text says, and: every reader file changed, by group; the help pages'
wrong lines; `stock_brief.rs`' printed figures for each charter.

## Answers

2026-11-03. **Q** (`stock-configuring-1`): widen the readers to `welcome_policy.rs` and
`startup.rs`, and take `tools-store` as a prerequisite? **A:** yes to both (option A); the
acceptance cannot hold without either.

- `needs` now names `tools-store`. Until it lands, `jig-protocol-store`'s `decode_run_policy`
  fills `RunPolicy.tools` with `EVERY_FAMILY`, so a restart turns every stock charter's new mask
  into a release revision: that is the cause of the restart-oracle failures in the engine's
  domain world and of `fixtures.rs`' `Refused(Stale { newest: 2 })`, and of the `engine.rs`,
  `tools.rs` and `workers.rs` failures that reach it. Start from the pushed branch, merge the
  trunk holding `tools-store` (and `tools-offered`), and rerun those first; weaken no revision
  assertion. If any still fails with `tools-store` merged, stop and report it with its log.
- Readers added to the survey, for the new count or member alone, as the others:
  `temper-service/src/tests/welcome_policy.rs` (the stock charter list `[1, 2, 3, 4, 6]`; the
  pinned policy bytes re-measured, 3,282 as `stock.rs`' test pins it) and
  `temper-service/src/tests/startup.rs` (the template labels and numbers with template 3, as
  the stock now orders them). List them in the report.
- `configuration.status.md`'s other 13.4 host-family sentence stays, as this spec says; report
  it under `## Left` for `help-stock` or the trunk's end check.
- The rest of the acceptance is unchanged: every package it names runs its focused tests, one
  command per group, and the report gives each run's counts.

2026-11-03. **Q** (`stock-configuring-2`): widen the readers to
`temper/crates/temper/tests/suite/creating.rs`? **A:** yes (option A); temper's focused tests,
in the acceptance, cannot pass without it.

- `creating.rs`, for the new count alone, as the other readers: in
  `the_cluster_offers_every_stock_charter`, the count 5 and its message naming the
  configuring agent ("stock chat, producer, configuring and reviewers"); its overflow line
  `documents.lines = 4`, one below the stock's five charters, so the refusal stays at the
  boundary; in `the_browser_receives_a_complete_stock_me_offer`, the isolating
  `documents.lines = 5`. Every other assertion stays. List it in the report.
- Run each of the two tests alone first, then the rest of the acceptance's focused suites, one
  command per package group, each excluding the live and proof binaries
  (`-E 'not binary(/^live/) & not binary(=proof_live)'`), and give each run's counts.
- Another reader the merged trunk brings that fails only on the count of four charters or two
  templates is fixed the same way and listed; any other failure, stop and report it with its
  log.

2026-11-03. **Q** (`stock-configuring-3`): widen the readers to
`temper/crates/temper/tests/suite/startup.rs`, its expected policy bytes 3,266 to 3,282? **A:**
yes (option A); temper's focused tests, in the acceptance, cannot pass without it, and the
figure is the one this task re-measured and pinned.

- `startup.rs`, in `a_stock_policy_past_its_limit_refuses_before_listening`, the message's
  "needs 3266 bytes" becomes "needs 3282 bytes"; every other assertion stays. List it in the
  report. `temper-service/src/tests/fixtures.rs`' comment naming 3266 says 3282 too.
- Readers widened once more, so that the rest of the acceptance runs through: a test or
  fixture that fails only on the stock's re-measured policy bytes (3,266 now 3,282), or only
  on the count of four charters or two templates, is fixed for that figure alone and listed.
  Any other failure, stop and report it with its log.
- Start from the pushed branch (`d385dea91e`); run `startup.rs`' test alone first, then the
  rest of `temper`'s focused tests and every package the acceptance names that has not yet
  run in this branch's last state, one command per package group, excluding the live and
  proof binaries, and give each run's counts.

2026-11-03. **Q** (`stock-configuring-4`): in `temper/crates/temper/src/tests/reference_settings.rs`,
let `reference_delegations()` keep the stock's `Executor::Charter(3)`? **A:** yes (option A);
temper's focused tests, in the acceptance, cannot pass without it, and the reader is already in
this spec's survey.

- `reference_delegations()`: `Executor::Charter(3)` after `Executor::Charter(2)` in the base
  list, which the derived cases keep; rename the bindings for their new lengths (ten, nine,
  twelve) and nothing else. The assertion stays `Ok(())`; its `count <= executor_kinds` check
  stays. List it in the report.
- Readers widened once more, by the kind of change rather than one figure at a time: a test or
  fixture that fails only because the stock now holds charter 3 and template 3 (the counts
  of five charters and three templates, the policy bytes 3,282, the rules' and policy 1's
  delegation to charter 3, the settings tools offered under charter 3 and no longer under
  a chat's charter 1) is fixed in that reader for that alone and listed. Weaken no
  assertion, and change no stock, design or engine code for it. Any other failure, stop and
  report it with its log.
- The flake `fake_backends::tests::source_adapter_is_a_bounded_fact_without_path_discovery`
  stays under `## Left` with both logs, as reported.
- Start from the pushed branch (`588d49e697`); run
  `tests::reference_settings::init_limits_hold_reference_delegation` alone first (`--lib`),
  then `temper`'s focused tests and every package the acceptance names, one command per
  package group, excluding the live and proof binaries, and give each run's counts.
