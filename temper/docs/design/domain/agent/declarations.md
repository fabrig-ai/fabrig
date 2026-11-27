# How a run's host tools are declared

2026-11-16. A run's first request carries its host tools' declarations
beside its charter's text and its brief, and the three share one
allowance (`room.md`, 2). This document says which host tools a run is
declared (1), in what form (2), how large a declaration and a role's
offer may be (3), how the declarations' contract moves (4), and what a
charter loses where its `tools` doesn't name a family its runs used (5).
What each tool does is `tools.md`'s. Written from
`../../draft/archive/tools-by-role.md`, sections 2 to 5.

## 1. The offer

A run is offered the tools its role uses, by the families its charter's
`tools` names (jig's `domain/engine/executors.md`, 1), the families the
engine already decides each call by (`temper/crates/temper-engine-domain-translate/src/shape.rs`,
`tool_kind`; jig's `domain/authority.md`, 3). No family is added.

| Family | Its host tools |
|---|---|
| delegate | `delegate`, `goal`, `change`, `finding`, `red`, `follow` |
| message | `message`, `introduce`, `mark_read` |
| control | `decided`, `amend`, `cancel`, `release`, `measure` |
| decide | `decide`, `decide_escalation`, `withdraw` |
| propose | `propose` |
| subscribe | `subscribe`, `unsubscribe`, `subscribe_forge` |
| effect | `effect_forge` |
| note | `note` |
| recall | `recall` |
| read | `read_task`, `read_forge`, `read_issues`, `read_ci` |
| settings | `settings`, and `propose_settings` in its configuring form |
| ask | `see`, `document`, `remember`, and `propose_settings` in its asking form |

- **`measure`** takes `target` and `revision`, as `release` does, and asks jig's `Measure` (jig's `domain/people/acts.md`, 1): a release of a held task below the run that asks its procedure for one fresh measurement of what its hold judged (`../forge/change/repairs-releases.md`, 4). It is no option of `release`, whose `again` stays for an effect held uncertain. Declaring it moves the contract's version by one, the version before kept as 4 says (`../../protocol/wires.md`, 7).

- **An asking charter keeps its own tools alone,** exactly as `tools.md`
  has it: one whose `tools` name the asking family and no family but it and
  settings (`temper/crates/temper-engine-domain-room/src/lib.rs`, `asking`;
  a mask of every family is not one). It is declared `see`, `document` and
  `remember` where its task's authority holds the asking family, and
  `propose_settings` in its asking form where its charter names settings
  and its authority holds it; nothing else.
- **Any other charter is declared a tool where its `tools` names the
  tool's family and its run's authority holds that family.** Every family
  follows this one rule, the delegate family's included, which until now
  followed the charter alone, and the message and read families', which
  followed the authority alone. The ask family's tools are never declared
  to a charter that isn't asking.
- **`read_docs` is declared to every run but an asking one,** whatever its
  families: it is how a run reads the whole form of a tool (2), and it
  costs 465 bytes. It is the only such tool.
- **`change` in its plan form** is declared where the run's authority
  also holds the control family (`tools-change.md`, 1); otherwise its short form.
- **One rule, in one place:** each declaration in `temper-engine-smith`
  carries its offer (`calls.rs`, `Offer`): the family whose tools it is,
  `read_docs`' every run but an asking one, or an asking charter's own.
  The run's declarations (`tools_for`), the `host-tools` page's offer line
  (`../../protocol/documentation.md`, 8) and the costs by family that the
  check at apply counts (`room.md`, 4.4) all read it, so a page, a run's
  list and its measured room never differ.
- **The offer lines** follow the rule's words: `Offered to every run but
  an asking one.` for `read_docs`; `Offered where the run's charter names
  the F family and its authority holds it.` for a family's tool, F the
  family's name as the table above has it (`settings` among them); and
  `Offered to an asking run, where its authority holds the asking family.`
- **The stock charters' offers** (`../../protocol/configuration/stock.md`,
  whose `tools` already name their roles' families):
  - the coder's, charter 2 (read and message, 514), where its change's
    questions go up, so that its authority holds message
    (`../forge/change/questions.md`): `message`, `introduce`, `mark_read`,
    `read_task`, `read_docs`, `read_forge`, `read_issues` and `read_ci`,
    beside smith's own tools; with questions `none`, the same without
    `message`, `introduce` and `mark_read`;
  - the reviewer's, charter 4 (read, 512): `read_task`, `read_forge`,
    `read_issues`, `read_ci` and `read_docs`;
  - the goal's, charter 5 (every family but settings and asking, 1,023),
    under policy 1's authority, which leaves out decide, propose and
    effect: the delegate family's six, the message family's three, the
    control family's four, the subscribe family's three, `note`, `recall`,
    the read family's four and `read_docs`, 23 tools;
  - the chat's, charter 1, as the goal's; the drafter's, charter 12, as
    the chat's but the delegate family's; a standing role's, charters 13
    to 16, as the chat's and the settings family's; Ask temper's, charter
    11, its own tools; the status agent's and a trial's, charters 9 and 10,
    which name no family, `read_docs` alone.
- **A run in flight keeps what it was declared:** the offer is decided
  when a run starts, so the narrower offer reaches runs that start under
  the build that brings it (4).

## 2. The declared form and the whole form

Each host tool has two schemas: the declared form, which a run's tool
list carries and every request pays for, and the whole form, every member
at every depth as its decoder takes it.

- **The declared form types the members a call usually sets.** The rest
  are declared as an object, or as any value, with a member description
  that says where the whole form is: the `host-tools` page, which
  `read_docs` searches by the tool's name and the member's. A member left
  out of a level is one whose parent is declared an object open to any
  member, so that the declared form refuses nothing its decoder takes.
- **Every dialect takes it:** a schema's root is an object without
  alternatives (smith's `object_schema` and the program's MCP `tool_list`),
  and both GPT request writers send `strict: false`.
- **Descriptions keep their bounds:** a tool's 160 bytes and a member's
  120 (`../../protocol/documentation.md`, 8). Words that repeat across
  tools are said once, in their shortest form: "owner/name; omit for the
  project's one".
- **The forge's reads declare what each `what` needs:** each of
  `read_forge`, `read_issues` and `read_ci` carries one member
  description, its `what`'s, listing each `what`'s required members
  (`tools.md`, *The forge's reads are three tools*), since a run reads a
  tool's declared form before its page, and a call missing a member costs
  a turn and a refused step a person sees. They pay for it in their own
  declared forms, which leave out what their decoders check and their
  whole forms keep: the `type` beside `what`'s `enum`, as `read_task`'s
  `field` is declared, and every integer member's `minimum`. Every member
  of their tables keeps its `type`. So the stock goal's offer and the
  coder's stay within their bounds (3), each about 110 bytes larger.
- **The whole form** is the schema the declared form was cut from, every
  member at every depth: `delegate`'s and `amend`'s executor, contract,
  authority with its grants, delegation, budget and notes, wake and LLM
  choice among them. Where a tool's declared form is already whole, the
  two are one. The `host-tools` page renders it
  (`../../protocol/documentation.md`, 8).
- **The decoders don't change.** Each decodes a call's input whole and
  answers a malformed one with what was wrong, deciding nothing
  (`../routing.md`, 2), so a run that needs a rare member reads its page
  once, and a wrong call costs one turn, where a schema costs every
  request.
- **The contract renders the declared form** (4): a declared form's change
  moves the contract's version; a description's doesn't.

## 3. The bounds

Held by `temper-engine-smith`'s step tests, as a description's 160 bytes
are.

- **A declaration** (its name, description and declared form, in UTF-8
  bytes) comes to 1,024 bytes at most: every tool, `change`'s plan form
  and `propose_settings`' asking form among them.
- **A role's offer,** each stock charter's own, at its families and its
  run's authority:
  - the stock goal's, charter 5 at policy 1's authority, 14,336 bytes at
    most;
  - the stock coder's, charter 2 at the authority of a change's producer
    whose questions go up, 4,096 bytes at most, and exactly the seven tools
    section 1 lists;
  - the reviewer's, the read family's tools and `read_docs`.
- **Every other offer,** each family's alone, every family's, and each
  authority's, is measured by the same tests, with no bound but the
  per-tool one.

## 4. The contract's versions

The host tools' contract (`../../protocol/wires.md`, 7) is the declared
forms, rendered by `temper-engine-smith`'s `wire.rs` and carried as
smith's `Grants.contract` (smith's `protocol/charter.md`, 2).

- **It moves from version 9 to 10, its floor staying 1.** Versions 1 to 9
  keep their kept texts; every version from the floor is answered, not
  just the previous one (`../../protocol/wires.md`, 7;
  `../../draft/archive/redeploys.md`, 4.2). Version 4 refuses
  zero-word reports to people or roles; version 5 adds the welcome
  setting; version 6 makes the forge's reads three tools, each taking a
  `what`; versions 7 to 9 move `settings` and `propose_settings`; version
  10 declares what each forge read's `what` needs, and takes
  `read_forge`'s branch left out and `file` at a branch (2; `tools.md`,
  *The forge's reads are three tools*), a change of the three
  declared forms and of what the decoders take, none of what they took
  before.
- **Every kept declaration that differs from 10 has an adapter** for each
  version it differs in: an identity adapter where only its declared form
  changed, or what its decoder takes grew, since the decoders take the
  whole form (2), so that `read_forge`, `read_issues` and `read_ci` take
  an identity adapter at each of 6 to 9; and, for each of the forge's
  former reads, the read its `what` gives (`tools.md`, *The forge's
  reads*), decoded and answered exactly as it was. A run under 6 to 9 is
  answered under the current decoders, the new forms included.
  `contract.rs` compares each kept declaration with the current one and
  requires an adapter wherever they differ, and its coverage test holds it.
- **The former reads' names are decoded under 6 and later too,** though not
  declared, each as its `what`. A run under 6 that calls one is refused by
  smith before the engine sees it, since smith offers a host tool exactly
  when it is declared (smith's `domain/run.md`, 5.1); so a charter's text names
  the reads as `tools.md` does, and the inventory (5) names the texts that
  don't.
- **A run keeps its contract across a redeploy:** a run started under 1 to
  5 is answered under it. A new attempt, resumed or fresh, is started
  under 6 (`temper/crates/temper-engine-smith/src/start.rs`), and its
  transcript's earlier calls stay as they were recorded.

## 5. What a charter loses: the inventory

Until this build, `tools` decided only the delegate and settings families'
declarations, so a charter could rely on a tool its `tools` doesn't name.
Under section 1's rule, a charter that doesn't name a family is no longer
declared its tools, where its task's authority held them:

| Not named | No longer declared |
|---|---|
| message | `message`, `introduce` |
| control | `decided`, `amend`, `cancel`, `release` |
| decide | `decide`, `decide_escalation`, `withdraw` |
| propose | `propose` |
| subscribe | `subscribe`, `unsubscribe`, `subscribe_forge` |
| effect | `effect_forge` |
| note | `note` |
| recall | `recall` |
| read | `read_task` and the forge's reads |

- **The stock charters lose nothing their texts use.** Charters 1, 5, 12
  and 13 to 16 name every family their texts call on. The coder's, 514,
  loses the control, decide, propose, subscribe, effect, note and recall
  families' tools, and the reviewer's, 512, those and `message`, none of
  which their texts name. The goal's and the coordinator's texts name
  `decide_escalation`, of the decide family, which policy 1's authority
  never held, so its calls were refused before and are undeclared now.
- **The stock texts name the forge's reads as `tools.md` does:** the
  chat's, the drafter's and the goal's name `read_forge` and its `what`
  where they named `read_branch` and `read_file`.
- **A deployment's own charters move by its inventory, not on trust.**
  Before the redeploy that brings this build, its keeper reads every
  charter, and every revision of one that a live task holds, and adds by a
  settings change each family whose tools its runs use; a text that names
  a former read moves to the new form.
  - `temper ctl DIR settings show --json` lists the charters and the
    settings revisions live tasks hold; `temper ctl DIR settings show
    charter N --json` gives a charter's `tools`, text and parts, with
    `--revision R` at a held revision (`../../client/ctl/tools-reads.md`, 1);
    `temper ctl DIR task show TASK --json` names the revision a task's run
    holds.
  - A charter's loss is the table's rows for the families its `tools`
    doesn't name; whether it matters is whether its text, or its runs,
    call those tools.

## 6. Open questions

- **A tighter per-tool bound.** 1,024 bytes is the user's figure to start
  with, "tighter where no risk shows" (`../../draft/archive/tools-by-role.md`, 10);
  the forge's merged reads are its tight spot. Default: 1,024.
- **A page per tool.** A run reaches a tool's whole form by searching
  `read_docs` for the tool's name, then reading the page from the offset
  the search gives: two calls. A topic per tool would make it one. Default:
  the search, until runs are seen to miss it.
