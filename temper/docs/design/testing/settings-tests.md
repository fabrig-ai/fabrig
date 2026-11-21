# The settings tests: the shipped settings checked against themselves

Provisional, 2026-11-21. How temper holds `../draft/failure-worlds.md`,
3.6: the stock profiles, the stock policy with its templates and
charters, and the provider presets' catalogues are checked against one
another, not figure by figure. Each of four parts states its relations
and their valid exceptions, and, over generated mutations of the stock,
the decision each mutation must get. A settings test runs over shipped
settings, outside every world (the draft, 3), as a step test in the
crate that holds what it reads.

## 1. In one page

- **Four parts:** the stock profiles (section 2), the policy's references
  (section 3), the provider capability fixtures (section 4) and amendment
  admission (section 5).
- **A relation** is a row of its part's table: its name, the design that
  states it, what holds over the stock, and its valid exceptions. Each
  part's test walks its table over the stock and names the first
  relation that fails, with its design section.
- **Mutations** (section 6) are generated from the same tables: for each
  relation, each figure or reference it reads, moved by the rule its
  kind gives. Each mutation gets the decision its design gives: followed,
  kept, accepted or refused naming what it names.
- **A relation the stock breaks** is a finding. Where it is an open
  problem of the draft's section 7, it is that row's ignored scenario
  (section 7), and nothing else is left out; any other is repaired with
  the test that finds it, as its design says.
- **Where:** the profiles in `temper-engine-service-vocabulary` and
  `temper-engine-service-startup`; the references, the capabilities and
  amendment admission in `temper-engine-service-stock`, which already
  builds the stock's whole `Defaults`; P81's scenario in
  `temper-client-tools`.

## 2. The stock profiles

Over both profiles: the stock, resolved with no override through the
start's one resolution (`startup::resolve_limits`; `../protocol/configuration/limit-settings.md`, 3),
and the worlds' `small` (`temper_limits_fixtures::small()`). A relation
holds in both. `small` is made without the stock agent's derivations,
so its fixture's own figures for the carriers of T and P (5.1) and the
counts that scale with `live_goals` (5.4) are explicit overrides, the
exception of their kind: over `small` those rows are judged by the
start's checks as they judge an override, and every other row holds as
over the stock. The formulas and their mutations are walked over the
stock in full.

Each relation has one of five kinds, which fixes what holds, its valid
exception, and how its mutations are drawn (section 6):

| Kind | What holds over the stock | Valid exception | Design |
|---|---|---|---|
| **follows** | the figure equals its formula over its sources; an override of a source moves it | an explicit override of the figure itself, kept, and judged by the start's checks | `../protocol/configuration/derived.md`, 1 and 5; `../protocol/configuration/limits.md`, the forge client's lifetime |
| **set by temper** | the figure equals its formula; nothing configures it | none | `../protocol/configuration/limits.md`, limits checked against each other |
| **floor** | the figure is at least its derivation | an override at or above its derivation, kept; one below is refused, naming the figure and its source | `../protocol/configuration/derived.md`, 1 and 6 |
| **checked together** | the two figures meet their relation | none: an override of either alone that breaks it is refused at start, naming both | `../protocol/configuration/limits.md`, limits checked against each other |
| **holds what it must** | a table holds all its design says it must, at the profile's other figures | a table whose design lets it refuse a batch, or keep only the newest, holds less: refusing is a valid answer, and no finding | each table's own design, named in its row |

The relations, at least (the change that writes the test adds every
derivation it finds in `limits::derive`, `limits::apply`, `forge::stock`
and the resolution's followers, each once, with its kind and design):

| Relation | Kind | Over |
|---|---|---|
| the forge component's deadline | follows | the forge client's lifetime, at most 270,000 with a GitHub forge (`forge::deadline`); an override past the lifetime refused, naming both |
| the forge component's `kept_bytes` | set by temper | the forge client's calls times its answer bytes |
| the forge component's connections | checked together | equal to the forge client's calls |
| the forge component's body bound | floor | twice the forge client's op bytes, and a key's marker (`derived::forge_body`) |
| the forge component's repositories | follows | the forge's repositories |
| the forge component's forges | follows | the forge's `forge.forges`, which sizes the component's forges (`../protocol/forge/README.md`, 6) |
| the forge component's string bytes | checked together | at least the forge's `forge.name_bytes`; the stock's 512 is its margin, not a floor, and an override of the names past it alone is refused at start, naming both (`forge::stock`, the component's check) |
| the carriers of T and P: the forge client's answer and op bytes, the checkout's message bytes, the journal's and the loads' transcript, reply and read bytes | floor | `../protocol/configuration/derived.md`, 5.1 |
| a worker's workspaces and git operations | follows | its `slots` (`../protocol/configuration/derived.md`, 5.2 and 5.3) |
| the counts that scale with `live_goals` | follows | ⌈stock × `live_goals` / 40⌉, each at most its cap (`../protocol/configuration/derived.md`, 5.4) |
| the ties of 5.4: landing members, landings, delegates, operations, inbox tasks, exemptions | follows | their scaled side (`../protocol/configuration/derived.md`, 5.4) |
| the brief's figures: the brief's read bytes against its task budget, the forge's brief bytes against the brief's read bytes, the journal's result bytes against the brief's bytes | floor, each refused at start below its source, naming both (`../protocol/configuration/limits.md`, limits checked against each other, the brief's): the stock sets the brief's bytes from the host part's room, not from the sum of the brief's budgets, and each is configurable. The brief's bytes within the host part's authorised brief is no row: it is the charter's room, checked per charter at start (`../domain/agent/room.md`, 1) |
| the stop bound and a unit's stop timeout | follows | the forge client's lifetime and clock margin (`../protocol/configuration/derived.md`, 5.6): the stop bound is `skein_forge_client::stop_bound`, twice their sum; a unit's stop timeout is the deployment's unit file, which no code derives, so its rule (the bound plus at least 30 s, to a whole minute) is checked by arithmetic over the same mutations, and its stock 660 s is the help's, which its own test pins |
| the forge's holds against its landings and their members | holds what it must | none: a landing past them is refused at its forming and its changes wait (`../domain/landing.md`, 12) |
| the forge client's repositories, entries and resources against the forge's repositories, entries and holds | checked together | none: an override of either side alone that breaks it is refused at start, naming both (`../protocol/configuration/limits.md`, limits checked against each other, the forge client's tables) |
| the forge client's resources against the forge's tasks times their resources | holds what it must | a batch past the bound is refused, and a full table refuses rather than stops (`../domain/forge/keeping-up.md`, its capacity) |
| the store's tier rows against `forge.changes`, and kept and pending landings with their members | holds what it must | the store's ceilings refuse the start, naming the setting (`../protocol/configuration/limits.md`) |
| the git component's output against its conflicts' paths | holds what it must | refused once, naming every short figure (`../protocol/configuration/limits.md`) |

## 3. The policy's references

Over the stock's `Defaults` (`temper_engine_service_stock::checked`)
and the forge's stock kinds, every reference a stock template makes,
directly or through what it names, resolves to something that exists
and is live:

| Relation | What holds | Valid exception |
|---|---|---|
| a template's executor | a template naming a charter (`TemplateExecutor::Charter`) names one the stock offers | a template whose executor is chosen names none |
| a charter's models | each model a stock charter names, first or among its alternatives, is in the stock catalogue and not withdrawn | a charter whose alternatives end with the rest (`rest`) still names at least one live model of its own |
| a producer | each producer the stock names (the forge's kind 3, a way's standing roles, kind 15's charter) is a charter the stock offers, and none is retired (`../protocol/configuration/stock.md`; `../protocol/configuration/settings.md`, 8) | none |
| a template's model | a template naming a model (`llm`) names one a provider preset keeps (section 4) | a template naming none |

**Settings apply refuses retiring one a template still names:** a change
of settings that removes a charter a template names is refused,
`Fault::Reference`, as jig's whole check does today
(`jig/docs/design/domain/engine/settings.md`, 1). The same for a producer a template's
work names is P59's open problem (section 7). A model is withdrawn,
never removed, and withdrawing one is never refused, so that an owner
can always clear what is out of range (the same section): a charter
naming it offers its next live model, among its alternatives or the
rest, and the relation holds while every stock charter still resolves
to a live model.

## 4. The provider capability fixtures

Over the provider presets (`temper-engine-service-stock`'s
`providers.json`) and the stock catalogue: each model a stock template
or charter names has, in the catalogue the preset that keeps it gives,
its efforts, its effort among them, its window and its output. A model
no charter or template names may have no efforts (the scripted
provider's `demo`). What a real account serves stays a live fact,
outside these tests (the draft, 3.6).

A model a preset discovers from a provider's list, and which a template
or charter can pick, having no efforts is P128's open problem
(section 7).

## 5. Amendment admission

Over the stock policy (policy 1, its roles and their pools): a party's
proposal or amendment, of each kind each stock role may propose, at its
role's bounds and one past them, is addressed to the role jig names as
its accepter (`jig/docs/design/domain/authority.md`, 9;
`jig/docs/design/domain/tasks/proposals.md`),
and that role's acceptance of it, by the check acceptance runs, is not
refused for what the address already knew: its kind, its authority and
its pool. No proposal is addressed to a role whose acceptance the same
check would refuse.

- **The two checks are jig's own:** the address
  (`acts::bounds::recipient`, through `Core::proposal_holder` where a
  task proposes) and the acceptance (`authority::check_request` with
  `PersonRequest::Accept`), called as the engine calls them, never
  restated in the test.
- **The kinds walked** are those a party proposes, a batch and an
  amendment (`authority::ProposalKind`'s `Batch` and `Amend`). A case
  one past a bound that the policy or the rules refuse before any
  address (routing's own check) is addressed to no role: it is counted
  and printed as refused before its address, never a finding. A bound
  the stock leaves open (no deadline) has no case one past it, and is
  reported unbounded.
- **Out:** judging an amendment by what it widens, the user's rule of
  2026-11-20, which is the fix of the draft's S15 row and no check here
  until it is built.
- **The owner's own widening** addressed to the owners' role is P77's
  open problem, whose scenario is jig's (the draft, 7,
  `an_owners_widening_is_never_proposed_to_itself`): the generated cases
  that reproduce it, a proposal the address sends to its proposer's own
  role (an owner's to the owners, a maintainer's to roles 0 and 1, both
  deciding every kind), judged by that role, are left out by that
  scenario's name and nothing broader: the other roles it is addressed
  to are judged.
- **A proposal its addressed role can't fund** is an open problem this
  walk found (section 7, `a_members_batch_is_addressed_to_a_role_that_can_fund_it`):
  the address weighs only the kinds a role decides, while acceptance
  caps a batch by the role's authority spend as well as its pool. In
  the stock policy the owners' and maintainers' authority spend is
  24K beside pools of 1,920K, so a member's batch one past 24K is
  addressed to the owners and refused there; a role's pool narrowed
  doesn't move the address either. The generated cases whose proposal
  is past the addressed role's authority spend or pool, while within
  the kinds it decides, are left out by that scenario's name and
  nothing broader. Its fix, the address weighing its accepter's funding
  or the stock roles' spend covering what they are addressed, is no
  work of these tests: the second moves a stock figure, the user's.

## 6. Generated mutations

The mutations are generated from the tables of sections 2 to 5, never
written case by case, and each names its relation:

| Kind | Mutations | The decision each must get |
|---|---|---|
| follows | each source overridden at half and twice its stock; the figure itself overridden | the figure moved to its formula; an override of the figure kept, or refused by the start's checks as they judge it |
| set by temper | each source overridden | the figure moved to its formula; it can't be configured |
| floor | the figure at its derivation, one below it, and above it | kept at and above; refused below, naming the figure and its source |
| checked together | each side overridden alone, past the other | refused at start, naming both |
| holds what it must | the table one below what it must hold; where the table is the store schema's fixed ceiling, which no setting lowers, the demand it must hold overridden one past it instead (`forge.changes`, `forge.landings_pending`, the landing members), read against the ceiling from the schema | refused, naming the table's setting, where it must hold all (the demand's, with `../protocol/configuration/limits.md`'s words, for a fixed ceiling); accepted where its design lets it refuse a batch |
| a reference | each charter or producer a template names removed, and each model it names withdrawn, one at a time | a charter's removal refused, `Fault::Reference`, naming the template; a producer's is P59's (section 7); a model's withdrawal accepted, the charter still resolving to a live model |
| a capability | each named model's efforts emptied with its effort kept, and its effort set outside its efforts | refused by the catalogue's check, naming the model; its efforts and its effort both emptied is P128's (section 7) |
| amendment admission | each role's pool and decided kinds narrowed by one | the address moves to a role whose acceptance passes, or the proposal is refused as nobody's (`jig/docs/design/domain/tasks/proposals.md`) |

The decision is judged through the code that makes it at a start or at
settings apply: the profiles through `startup::resolve_limits` and the
start's checks; references, capabilities and admission through jig's
whole check and catalogue check, as settings apply runs them. A case
whose decision differs from its row fails, naming the mutation, its
relation and the decision it got.

- **Where one mutation meets two relations:** every profile mutation
  goes through the resolution. A source overridden alone that breaks
  another relation of the tables (the forge client's calls, which the
  component's connections must equal; the forge's repositories past the
  forge client's) gets that relation's refusal, named in the case, and
  only where that relation is a row of its own. The same source is then
  overridden again with a companion override that keeps the other
  relation (the connections with the calls), and that case gets its own
  row's decision, the formula observed through the same resolution.
  Where no companion in range keeps the other relation, the case keeps
  that relation's refusal as its decision and the row's formula is
  observed by its other mutations.
- **Where a mutation passes a fixed ceiling:** a value a wire's or the
  store schema's ceiling refuses whatever else is set (the forge
  client's `answer_bytes`, `forge.holds` and `forge.landing_members`
  at twice their stock, or one past a floor that sits at the ceiling)
  gets the start's
  refusal naming the figure and its ceiling, as its decision, and the
  table holds a row for each such ceiling, citing what fixes it. No
  companion raises a wire or a schema, and no result is made up for the
  row the mutation was drawn from: its other mutations observe it.
- **Where one removal meets several references:** jig's whole check
  names the first reference it meets, the policy's rules before the
  templates. A charter's or producer's removal from the stock as it
  ships gets the refusal the whole check gives, `Fault::Reference`
  naming what it met first; the same removal over a copy of the stock
  with every other reference to it cleared gets the row's decision,
  naming the template.
- **Where a narrowing meets the policy's own check:** every amendment
  admission mutation goes through the policy's admission before any
  walk. A role's decided kinds narrowed so that another field of the
  policy no longer holds (the owners' role without `Escalation` while
  the policy's escalation role is still the owners') gets that
  admission's refusal (`InvalidRole`), named in the case, and is not
  walked; no second field is moved to make it walkable. The other
  narrowings, admitted, are walked for the row's decision.

## 7. The draft's open settings rows

Each is an ignored scenario, its `#[ignore]` naming the work that fixes
it, run by that work's goals until they land it (the draft, 6):

| Problem | Scenario | Where | What it asserts | Ignored for |
|---|---|---|---|---|
| P128 | `a_discovered_model_has_efforts` | `temper-engine-service-stock`'s step tests | a model a preset discovers from its provider's list (`gpt-6.2-sol`, as the OpenAI list gives it, with no efforts), which a template or charter can pick, has efforts in the catalogue | provider failures |
| P59 | `a_retired_producer_is_refused_while_named` | `temper-engine-service-stock`'s step tests, through settings apply's whole check | settings apply refuses retiring a producer a stock template's work names, naming the producer and the template | provider failures |
| P81 residual | `a_json_proposal_carries_hundredths` | `temper-client-tools`' step tests | the `--json` answer's proposed command carries a budget's amount in the unit ctl reads, hundredths as `5.00` | its own fix |
| found by section 5's walk | `a_members_batch_is_addressed_to_a_role_that_can_fund_it` | `temper-engine-service-stock`'s step tests | a member's batch one past the owners' authority spend, in the stock policy, is addressed to a role whose acceptance passes, or refused as nobody's | the address weighing its accepter's funding |

A scenario that passes when written is no open problem: it stays
enabled, and the draft's status says its row is fixed.

## 8. Testing

These tests are the step tests above, in the focused suite, each part's
walk over the stock and its mutations; the three scenarios compile and
are ignored. They run in milliseconds and need no world.

## 9. Later

- Amendment admission judged by what an amendment widens, once the S15
  row's fix builds it.
