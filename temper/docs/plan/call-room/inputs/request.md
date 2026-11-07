# call-room: a call and a report get room for a whole spec

`temper/docs/design/draft/goal-as-designer.md` puts a task's whole spec in `change`'s words, and a coding agent's report in its result. Today neither fits: a call's payload follows a model's declared output, a `change` of five changes is refused as too large whatever its bytes, and a report stops at 3,584 bytes. This plan builds that draft's section 6. The draft wins where other designs differ, and its status file names the sections that lag.

## What's wanted, ranked. Drop from the bottom if it passes six tasks, and say what you dropped

1. **The failing test first: a large argument.** An argument of 64 to 128 KB through a program into smith's MCP server has never run; the largest seen is about 10 KB. In `smith/tests/agent-process`, with its fake program, in the focused suite, as rows of a table if a test already takes sizes. Also a story of the same size through real Claude Code in smith's live tier, **written and compiled, not run**: after-landing runs it. Say what limit each size meets first.

2. **A call's room.** The reference takes `output` about 98,500 tokens, 128 KB, and 48,000 tokens, 64 KB, if the start's checks refuse it: the agent's largest turn, 16.8 MB at init's figures, has a ceiling of 33.5 MB and hasn't been measured there. Measure it, by a focused test, and pick by the result. `tool_payload` follows `output` (`temper/crates/temper-service/src/startup.rs`, near 254), and the start's checks must name each limit that moves with it. `model-catalogue.md` keeps each stock model's own `output` in `temper-service/src/stock.rs`, 20,000 for Opus. **Those stay as they are.** The reference's models, or the settings a deployment gives, carry the larger figure. If the mechanism needs a stock figure to move, that's a question first.

3. **A batch.** `authority.batch` goes from 4 to about 32 (`jig/crates/jig-protocol-config/src/profile/standard.rs`, `AUTHORITY`; the refusal is `jig-domain-authority/src/check.rs`, `delegates_within`). Say what that costs in bounds, and if a figure belongs in the reference rather than jig's profile, put it there.

4. **A brief.** `hub.charter_bytes` goes to 262,144, to hold a whole spec beside the charter and its tools (`jig/crates/jig-protocol-config/src/profile/small.rs`, inherited). Past it, a brief is cut, as today.

5. **A report.** A change's `body` follows the call's payload rather than `CHANGE_BODY_MAX` (`jig/crates/jig-protocol-charter/src/assignment.rs`), with smith's outcome (`smith/crates/smith-agent-service/src/profile.rs`, `outcome_bytes`) and `hub.outcome_bytes`, so it is a setting from then on. A rendering of a report may cut what it shows; the store keeps it whole. Keep a test holding a report past 3,584 bytes through to the requester.

6. **The design says so:** `temper/docs/design/protocol/configuration.md` 13.8, `domain/agent.md` section 10 and its open question, and the draft's status entries this settles. In the same status file, section 8 is settled: a survey of the pipeline's answers read tight, so a goal's later runs try high, by an amendment after the first run, and the choice is looked at again after a first comparison.

## Not in this plan

- The words of the stock and reference charters, their wakes and their move to files.
- Budgets: a deployment starts goals with `--as-allowed`, or too high to hit. The half a `change` carves stays.
- `temper/docs/design/draft/reference-code-production.md` stays unedited: tests and a judge read it.

**Tests stay cheap.** The focused and fuzzy limits are being tightened, and the gate will reject a plan that adds cost: a new case of one behaviour is a row of an existing table test, a new story goes in a world below the real loop, and a test holding a large figure says what it costs. A figure that moves a heap ceiling: measure before and after, and say why it moved (skein's `docs/foundation/testing-strategy.md`, section 6). Budgets and timeouts of test tiers are the user's (8.1).

Failing tests first; no test reaches a real host. **A plan runs the focused and fuzzy suites alone and is complete when it lands (`AGENTS.md`, 5).** A stored format or protocol change is a question first, so its owner can be told; both main areas write `temper-engine-domain` and `forge_route.rs`, and skein's interface changes get a heads-up. Formats change in place before v0.1.0. jig never names temper, and smith names neither. `walk = false`.
