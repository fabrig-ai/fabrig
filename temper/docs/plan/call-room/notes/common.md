# call-room: the rules every task names

Read this note, then your spec. The request is `temper/docs/plan/call-room/inputs/request.md`; the draft is `temper/docs/design/draft/goal-as-designer.md`, section 6, and it wins where another design differs.

## What holds in every task

- **Failing tests first.** No test reaches a real host.
- **Only focused and fuzzy run.** A live story is written and compiled (`--no-run`), not run; after-landing runs it. Say so in the report.
- **Tests stay cheap.** The focused and fuzzy limits are being tightened, and the gate rejects a plan that adds cost. A new case of one behaviour is a row of an existing table test, a new story goes in a world below the real loop, and a test holding a large figure says in a comment what it costs (CPU, measured).
- **A figure that moves a heap ceiling:** measure before and after and say why it moved in the report (`skein/docs/foundation/testing-strategy.md`, 6). Budgets and timeouts of test tiers are the user's (8.1): never raise one.
- **A stored format or a protocol's shape** (a generated codec's fields or its ceilings, `jig/crates/jig-protocol-store/src/generated/*`, `smith/crates/smith-channel/src/generated/*`, `smith/crates/smith-charter/src/generated/*`) is a `Question:` first, so its owner can be told. Moving a limit below its ceiling isn't one.
- **The dependency rule:** jig never names temper, and smith names neither.

## What stays as it is

- **The stock's figures.** `temper init`'s agent keeps `tool_payload` 10,662 and an `output` of 8,192, and `temper/crates/temper-service/src/stock.rs` keeps every figure of its models: that file's `output` field (20,000 for Opus) is a price per unit, and the stock's token output is the literal 8,192 beside it. If a task finds it must move one of these, that's a `Question:`.
- **`temper init`'s largest turn.** T stays 16,803,810 unless a spec says it moves. If it moves, that's a finding for the report, with why.
- **The words of the stock and reference charters,** their wakes and their move to files.
- **Budgets:** no spend, period or time moves. The half a `change` carves stays.
- **`temper/docs/design/draft/reference-code-production.md`** stays unedited: tests and a judge read it.
- **Any `docs/design/`:** this plan's cut writes it; what the code taught goes in the report. `docs/help/` is closed but for what the implementer's text allows.
