+++
id = "configuring-told"
part = "configuring-told"
kind = "feature"
needs = ["offer-engine", "stock-seven"]
files = [
  "temper/crates/temper-engine-domain/src/route/host_route.rs",
  "temper/crates/temper-engine-smith/src/settings.rs",
  "temper/crates/temper-engine-smith/src/tests/settings.rs",
  "temper/crates/temper-service/src/stock/configuring.txt",
  "temper/tests/engine/domain/tests/suite/configuring.rs",
]
effort = "high"
minutes = 70
+++

# The configuring agent is told the offer alone

## Goal

On a Claude-only box the configuring agent could propose a ChatGPT model, because nothing tells it which models
an account can run, and the charters it reads name them all. Add the offer to the inventory it reads first, and
tell it that this is the list it speaks from.

Acceptance, each a story in `temper/tests/engine/domain/tests/suite/configuring.rs`, beside
`an_inventory_names_its_own_projects_people`:

1. An inventory read by a configuring run whose payer has a usable Claude login names `models you can offer:` and
   a line per Anthropic model that its authority allows, each with its efforts and default, and names no
   OpenAI model.
2. With no account: the line `models you can offer: none; no account can run a model yet`.
3. A read of one setting, a revision or a revisions page carries no such line, as `project people:` isn't carried.
4. `stock/configuring.txt` holds the rule, and `temper-service`'s `stock_brief.rs` still fits charter 3's host part.

## Inputs

- `temper/docs/plan/model-catalogue-1/notes/common.md`, `temper/docs/plan/model-catalogue-1/notes/contracts.md`
  (*The configuring agent*, *The core*).
- `temper/docs/design/domain/agent.md`, 8.1.

## Owns and may change

- Owns `route/host_route.rs`'s settings enrichment, `temper-engine-smith/src/settings.rs`'s `repository_text`,
  `repository_rows` and its call chain's parameters, its test, `stock/configuring.txt` and the world test above.
- May change, call sites only: whatever carries `people` and `forge_people` from `host_route.rs` to
  `temper-engine-smith`'s `settings_page`, adding `offer` beside them; `temper/crates/temper-engine-smith/src/lib.rs`'s
  re-export line.
- May change the pinned byte counts in `temper/crates/temper-service/src/tests/stock_brief.rs` and
  `temper/crates/temper-service/src/tests/stock.rs` that charter 3's longer instructions move, each only in its figure.
  The reference's `temper/docs/examples/reference/**` holds its own instructions: leave it.
- Removes `temper/docs/design/domain/agent.status.md`'s **Not yet** 8.1 entry about the offer.
- Options, pre-approved:
  - The charters the agent reads keep naming every model: don't filter a setting's read, since its value goes
    back whole in a proposal (`agent.md`, 8.1). The instructions cover it.
  - A proposal naming a model outside the offer isn't refused: a charter's "the rest" names them. Don't add a check.

## Steps

1. `host_route.rs`: `OfferedModel` as the contracts note has it; `settings_offer(domain, flight, answer)` beside
   `settings_people`, `Inventory` alone: `domain.core.tasks.payer(flight.task)`, its project from `delegation`,
   its authority's models from the task, then `Core::offer`. A provider's name from `settings.llm_providers`.
2. `settings.rs`: the rows after `forge people:`, measured and written as the others are.
3. `configuring.txt`, in the paragraph beginning "First: settings without a member": "Inventory: models you can
   offer: all that accounts can run now. Propose and mention only those, even where a charter names others; where
   it names none, say no account can run a model yet, and to connect one."
4. The world stories, using the fixtures `a_box_inventory_distinguishes_the_requester_from_the_forge_approver` uses.
   Break each once to see it bite.
5. Narrow tests, then the implementer's text's closing checks over `temper-engine-domain`, `temper-engine-smith`,
   `temper-service` and the world's package. Commit and push.

## Report

As the implementer's text says; the host part's figure for charter 3 before and after.
