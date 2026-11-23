# Tools by role: a run is offered the tools its role uses, each in a small declaration

Draft, 2026-11-16, on the user's word. A run's first request carries three things: the charter's text, the brief, and the host tools' declarations. The engine fits all three within one allowance, 65,536 bytes, and gives the brief what the other two leave (temper's `fitted_charter`, `temper/crates/temper-engine-service-vocabulary/src/assignment.rs`). The user, 11-16, on that cap: "a reasonable 'size of the whole blob that we send right away as context' cap that we should keep". This draft makes room inside it. Each run is offered only the tools its role uses, and each tool is declared in a schema that states what a call usually takes. It also makes a charter that doesn't fit visible, at apply and at a run's start (section 6). On 11-16 one deployment's charter didn't fit, and no goal on it could start.

## 1. What a first request carries today

Read on one deployment, 2026-11-16. The declarations' figures are names, descriptions and schemas in UTF-8 bytes, measured from `temper/crates/temper-engine-smith/src/calls.rs` and the specs beside it (`finding.rs`, `red.rs`, `follow.rs`) at main `a285a8079e`.

| | A goal's first run (its designer) | A change's coder |
|---|---|---|
| Charter text | about 26 KB: the shared text and the three parts a first run takes | about 16.6 KB, one text with no parts |
| Host tools declared | 42, 36,512 bytes | 36, 25,570 bytes |
| The first request's input | 33,974 tokens (goal 4426) | 14,684 tokens (task 4299) |

- **The brief gets what's left.** A goal's fixed part, its charter text and tools, comes to about 64.8 KB by an estimate from the source. That leaves its brief under 1 KB.
  - Goal 4426's request was 2,608 bytes. Its designer read about 640 of them, then `[cut 1971 bytes; whole: the task's activity]`, and spent a turn on `read_task` to read the rest.
  - A charter 82 bytes larger couldn't start at all. Every first run was refused "assignment cannot encode" and spent nothing, and the goal was held "out of tries: permanent failures", naming neither the field nor the bound.
- **The bytes are in the schemas.** Descriptions are already held to 160 bytes (`../protocol/documentation.md`, 8). A goal's 42 declarations take 406 bytes of names, 4,746 of descriptions and 31,360 of schemas.
  - Six are over 1,024 bytes: `delegate` 6,124, `amend` 5,211, `change` in a goal's form 2,412, `propose` 1,377, `note` 1,351 and `decided` 1,105. `delegate` and `amend` each spell out jig's whole shapes: an executor, a contract, an authority with its grants, delegation, budget and notes, a wake with seven members, and an LLM choice.
  - The forge's 18 reads (`read_items` to `read_permission`) take 9,854, one tool per endpoint. Fifteen of them carry the same 80-byte description of `repository`.
- **A run is offered tools its charter doesn't name.** `../domain/agent/tools.md` declares most host tools to every charter but an asking one, whatever its families. Only the delegate family's tools (`delegate`, `goal`, `change`, `finding`, `red`, `follow`) and the settings family's follow the charter, while `message` and `read_task` follow the run's authority alone.
  - A coder whose charter names only the message and read families is still offered 36 tools: `amend`, `propose`, `decide`, `decide_escalation`, `release`, `cancel`, `introduce`, `subscribe`, `subscribe_forge`, `effect_forge`, the 18 forge reads and more. Every request carries them.
  - `tools.md`'s own reason for trimming the asking charters, "A tool a run never calls costs every request and blurs what it does", holds for every role.
- **What runs call.** In the turns the engine kept for four goals (4272, 4275, 4276, 4426), the host tools called were `note`, `recall`, `read_task`, `message`, `release`, `amend`, `finding`, `red` and `decided`. Task 4299, a coder, called none: its shell, edits and `deliver` are smith's own. The engine keeps a window of each run's turns, not all of them, so this is a sample.

## 2. A run is offered its charter's families

- **The families are the ones the engine already decides each call by** (`temper/crates/temper-engine-domain/src/translate.rs`, `tool_kind`; `jig/docs/design/domain/authority.md`, 3). No family is added:

  | Family | Its host tools |
  |---|---|
  | delegate | `delegate`, `goal`, `change`, `finding`, `red`, `follow` |
  | message | `message`, `introduce` |
  | control | `decided`, `amend`, `cancel`, `release` |
  | decide | `decide`, `decide_escalation`, `withdraw` |
  | propose | `propose` |
  | subscribe | `subscribe`, `unsubscribe`, `subscribe_forge` |
  | effect | `effect_forge` |
  | note | `note` |
  | recall | `recall` |
  | read | `read_task` and the forge's reads |
  | settings | `settings` and `propose_settings`'s configuring form |
  | ask | `see`, `document`, `remember` and `propose_settings`'s asking form |

- **An asking charter keeps today's offer,** exactly. An asking charter is one whose `tools` name the asking family and no family but it and settings (`temper/crates/temper-engine-domain/src/boundary.rs`; a mask of every family is not one). It is offered its own tools alone, where its authority holds their families: `see`, `document` and `remember`, and `propose_settings` in its asking form where its charter and authority hold settings.
- **Any other charter is declared a tool where its `tools` names the tool's family and the run's authority holds it.** That replaces the offer to every run, and the offers of `message` and `read_task` by authority alone. The ask family's tools are never declared to a charter that isn't asking.
- **`read_docs` stays offered to every run but an asking one,** as today: it is how a run reads the rest of a tool's form (section 3), and it costs 465 bytes.
- **The stock charters name the families their roles use** (`../protocol/configuration/stock.md`):
  - A coder reads changes and CI, so its charter names message and read. It is declared `message` (its question to whoever cut its change), `read_task`, `read_docs`, `read_forge` and `read_ci`, about 3 KB, beside smith's own tools.
  - A reviewer's names read.
  - A goal's charter names every family but settings and asking. Its authority leaves out decide, propose and effect, so it is declared 21 tools once the forge's reads are two.
- **Deployments' charters move by an inventory, not on trust.** A charter of a deployment's own already names its families in `tools`, but today that names only what delegate and settings offer, so a charter may hold a tool through the run's authority alone.
  - T2's design reads every charter, and every revision of one that a live task holds (`jig/docs/design/domain/engine/executors.md`, 1), and lists the tools each would lose and whether its text names them.
  - The charter's keeper then adds the families it relies on, by a settings change, before the redeploy.
  - The narrower offer applies to runs that start under the new build. A run in flight keeps its contract (section 5).
- **The `host-tools` page's offer lines** follow the one rule (`../protocol/documentation.md`, 8), so a page and a run's list never differ.

## 3. A declaration states what a call usually takes

- **Today** a schema spells every member at every depth, and `amend` repeats `delegate`'s authority, wake and LLM shapes.
- **Wanted:**
  - A schema types the members a call usually sets. The rest are declared as an object, or as any value, with a member description (120 bytes at most, as today) that names where the whole form is: "as `read_docs host-tools` gives it". Every dialect takes that: a root must be an object without alternatives (smith's `object_schema` and the program's MCP `tool_list`), and both GPT request writers send `strict: false`.
  - The engine's protocol layer already decodes every call whole and answers a malformed one with what was wrong (`../domain/routing.md`, 2). So a run that needs a rare member reads its page once, and a wrong call costs one turn, where a schema costs every request.
  - The `host-tools` page gives each tool's whole input, members at every depth, rendered from the table its decoder reads, so the page and the decoder never differ.
  - **Bounds,** held by `temper-engine-smith`'s step tests as the 160-byte description is today:
    - a tool's name, description and schema together come to 1,024 bytes at most;
    - the tools a role is declared come to 14,336 bytes at most for a stock goal's charter and 4,096 for a coder's. These are acceptance targets, measured on each stock charter's own offer, its families and its authority together. The tests also measure the offer for every family and every authority, which has no bound beyond the per-tool one.
  - Words that repeat across tools are said once, in their shortest form ("owner/name; omit for the project's one").

## 4. The forge's reads, as two tools

- **`read_forge`:** items, an item, a pull, the pull for a head and base, reviews, remarks, a pull's files, a branch, branches, a comparison, a file, protection, settings, collaborators and a permission.
- **`read_ci`:** checks, a job's log and statuses, which a goal reads for its reds.
- **Each takes a `what` member and the union of its reads' members, each member described once.**
  - A member keeps one meaning. Today two reads give one name different meanings: `after` is an item's comment id in one and a comparison's later commit in the other, and `head` is a branch in one and a commit in another. The union gives each meaning a member of its own, and section 5's adapters map the old names.
  - The decoder checks each `what`'s required members and answers a missing one by naming what that `what` needs.
- **Each `what` is answered exactly as its tool is today:** the same renderers, bounds and pages. What each reads from the forge is `archive/forge-reads-targeted.md`'s, unchanged here.
- **Each stays within section 3's bound.** A terse union with every member described is about 1.4 KB, so `read_forge` likely splits in two, for example the repository's reads and a pull's. The split is part of T2's acceptance, and the design records it.

## 5. Upgrades

- **The host tools' contract moves from version 2 to 3, and its floor stays 1** (`temper/crates/temper-engine-smith/src/wire.rs`, carried as smith's `Grants.contract`, `smith/docs/design/protocol/charter.md`, 2). Versions 1 and 2 keep their kept texts. The upgrade rules want every version from the floor answered, not just the previous one (`../protocol/wires.md`, 7; `redeploys.md`, 4.2).
- **Every declaration that differs from a kept version gets an adapter** for both 1 and 2: an identity adapter for a schema-only change, and a mapping for each renamed read and member. `contract.rs` compares each kept declaration with the current one and requires an adapter wherever they differ, and its coverage test holds that.
- **The old reads' names are also decoded under 3,** though not declared, as the `what` they map to. So a charter whose text names `read_checks` keeps working until the floor passes 2.
- **A run started under 1 or 2 keeps its contract** across a redeploy. A resumed new attempt gets the current contract (`temper/crates/temper-engine-smith/src/start.rs`), and its transcript's old calls are read through their mapping.

## 6. The room, held

- **The cap stays 65,536** (the user, 11-16): the allowance `fitted_charter` fits a charter to, `min(hub.charter_bytes, the agent's allowance or 65,536)` (`assignment.rs`), is unchanged.
- **A refusal says what didn't fit.**
  - **Today:** temper's `assignment::down` and `fitted_charter` (`assignment.rs`), and jig's `translate::down` and `assignment_record` (`jig/crates/jig-protocol-engine-workers/src/translate.rs`), return no reason. `Workers::assignment` (`temper/crates/temper-engine-service-component/src/workers.rs`) and jig's `Component::deliver` (`jig/crates/jig-protocol-engine-workers/src/component.rs`) say only "assignment cannot encode". The hub link's `payload::ending` (`jig/crates/jig-domain-hub-link/src/payload.rs`) turns an invalid start into `Failed(Permanent)` and drops its detail.
  - **Wanted:** each refusal on these paths returns a typed reason: the field (for example `charter.fixed_owned`), its figure, the configured bound and the wire's ceiling, keeping the codec's `Problem.path` where there is one. The engine's log prints it whole. The held task's reason carries it: the invalid start becomes `FailedWithReason`, within its 128-byte bound, cut with a marker.
- **A charter that can't fit is refused when it's applied.**
  - **Today:** `settings apply` checks a charter's rendered room (`temper/crates/temper-engine-domain/src/room.rs`), but not its fixed part's ownership or encoded size, so a charter too large to start is accepted.
    - `room::HostPart` counts one ordinary total of declarations, plus the settings and asking forms, so it can't count a family's subset.
    - Without a local agent's declarations at startup the check is skipped (`temper/crates/temper-engine-domain/src/route.rs`, `room_refusal`).
  - **Wanted:** applying settings refuses a charter whose fixed part, for any wake reason, with the tools its families declare and its delivery fields, can't fit the allowance. From T2 on, it is also refused where the fixed part leaves the brief less than its floor.
    - The floor is the budgets a goal's brief fills: its task, transcript tail, dependencies, attempts, plan and notes. That is 13,312 bytes of body in the shipped profiles, whose standard profile takes the small one's budgets (`jig/crates/jig-protocol-config/src/profile/small.rs`). The sections' ownership and framing count beside it.
    - The refusal names the charter, the wake reason and the figures, and says what to trim.
- **The costs reach the check as facts.** The declarations live in `temper-engine-smith`, which depends on `temper-engine-domain`, so the domain can't measure them itself without a cycle (`skein/docs/foundation/programming-model.md`, 4; `crate-graph.md`).
  - The service layer, which holds both, gives the domain typed costs: each family's declarations, each wake reason's parts, each delivery form, and both the owned and encoded bounds.
  - With no local agent, the check uses the 65,536 allowance rather than skipping.
- **Why the floor waits for T2:** today a stock designer's fixed part is about 56,177 bytes: 19,665 of joined texts (`temper/crates/temper-charter-set/src/lib.rs`) and 36,512 of declarations. That is over the 52,224 the floor leaves, so only T2's smaller declarations let it pass.

## 7. Out

- Claude Code's own six tools (Bash, Read, Write, Edit, Glob, Grep) and their descriptions, which the program adds outside the cap.
- smith's own tools: `wait`, `deliver`, `save`, `finish` and sub-agents.
- The charters' texts, about 26 KB of a goal's first run. They belong to whoever keeps each charter, and the inventory in section 2 tells them where a family is needed.
- The cap's figure.

## 8. After landing

- A designer's and a coder's first request on a deployment, from `task turn --json`'s usage, against 4426's 33,974 and 4299's 14,684 input tokens.
- A goal's request of up to 4 KB arrives whole in its brief.

## 9. Tests

Cheap: focused and fuzzy suites only; walk = false.

- **`temper-engine-smith`'s step tests:**
  - every declaration within 1,024 bytes;
  - each stock charter's offer, with its families and its authority: the coder's exactly `message`, `read_task`, `read_docs`, `read_forge` and `read_ci`, within 4,096 bytes; the goal's within 14,336; the reviewer's its reads and `read_docs`;
  - the asking cases: a mask of asking and settings (3,072), asking without settings, a mask of every family, and an asking task whose authority lacks the asking family;
  - the `host-tools` page renders each offer line from the one rule, and each tool's whole input;
  - the contract's coverage: an adapter for every declaration that differs from versions 1 and 2.
- **The engine:**
  - calls adopted live across a restart under versions 1, 2 and 3, and an old read's name under 3, each answered as its `what`;
  - a `what` missing its required members, answered by naming them;
  - beside the workers' assignment tests (`temper/crates/temper-engine-service-component/src/tests/workers.rs`), a synthetic first assignment over its room is refused naming `charter.fixed_owned` and its figures: shared text 9,910 bytes, parts 11,031, 3,655, 1,597 and 649, the first three joined, 42 tools, a task's models, goal delivery, and a 2,342-byte task section;
  - the hub link's: a refused start's reason reaches the held task.
- **Settings:**
  - a charter that can't fit is refused, naming the charter, the wake reason and the figures, with and without a local agent;
  - from T2, one that leaves the brief less than its floor is refused the same way.
- **Each stock charter:** its first run's fixed part stays within 65,536 bytes less the brief's floor.

Run only the new and affected tests, by name.

## 10. Open questions

- None. The figures are the user's of 11-16: the brief's floor as drafted, and 1,024 bytes per tool to start with, tighter where no risk shows. The role bounds (3) are the draft's reading of that.
