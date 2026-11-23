# What the tiers check: stage 1's golden, its check and its judge

This holds stage 1's golden, its check and its judge, split from [what the tiers check](checks.md).

## 1. Stage 1: the golden, its check and its judge

Stage 1 of configuring temper by talking to it
(`../draft/configuring.md`, 8.1) is a walk, never in the gate: a pass
of QA, playing a person on a box with the stock settings, has Ask
temper (`../domain/agent/agents.md`, 1 and `../domain/agent/ask-temper.md`) propose the reference's
configuration from the person's documents alone, accepts it, and checks
what was accepted twice, by code and by a judge. What it is checked
against is the golden: `../draft/reference-code-production.md`,
section 14, cut into obligations, with the values of the golden's own
profile, `golden/profile.json` in the tool's crate: the reference's
settings file as it stood with whole charters 5, 7, 2 and 8, before
the reference became overlays of its fields over the stock's charters 5
and 2 (`../protocol/configuration/reference.md`). Ask temper proposes
whole charters, as the person's documents describe them, an overlay of
its own being later (`../draft/stock-charters.md`, 11), so the golden
keeps the whole charters it checks a proposal against; the reference's
landing rules, template 4, limits and room are the same in both.

- **Held by the acceptance, never by the binary.** The golden, the check
  and the judge's brief are one tool crate, `crates/temper-stage-one-tool`
  (`bin/temper-stage-one`, `README.md`, section 5), which may read temper's crates and
  which no crate of the binary reads. Neither the stock nor the
  documentation compiled into the binary carries any of it, so the agent
  works from the person's description alone.
- **The profile is the golden's values; the obligations say which
  count.** An obligation names a role or a setting, a member of its
  value and a test; its expected value is the profile's item for that
  role, at that member, unless the obligation states its own. A member
  no obligation names is free: a charter's spend, its call timeout, a
  run's reserves stay the box's, as the profile leaves them
  (`../protocol/configuration/reference.md`: no budget or time moves).
- **The profile's placeholders** for the forge and the repository
  (`127.0.0.1:3000`, `acme/widget`) stand for the box's: the check
  replaces them in every expected value, a landing rule's pattern
  compared as its segments' text.

### 1.1 The obligations

Each obligation is one field of one row of the reference's table, and
of one kind:

- *a setting*, which the accepted settings hold, by one of nine
  tests: `equal` (the value, whole), `at-least` (a number no smaller),
  `bits` (each expected bit of a family mask set, others allowed),
  `lacks` (no expected bit of a mask set),
  `contains` (each expected element of a list present, a member it does
  not name free and one absent counting as null, others allowed),
  `set` (present and not null, whatever its value), `judged` (text the
  judge reads, 1.4; the check only finds it set),
  `unchanged` (the stock's value, which the accepted revision leaves
  alone) and `only` (every current setting of a kind binds to one of
  the golden's roles);
- *the box's*, a prerequisite the box brings (1.2), which the agent
  proposes nothing for, and names where the box lacks it;
- *a gap*, a field of the table no design carries
  (`../draft/reference-code-production.status.md`), which the agent
  must say temper cannot model, offering the nearest thing
  (`../domain/agent/agents.md`, 1): the golden gives the words, any of whose
  alternatives in the proposal's or the chat's text names it.

**The roles.** Charters, templates and the deployment's landing rules
are numbered by the proposal, so the golden names them by role and the
check binds each role to a number:

| Role | The profile's | Bound by |
|---|---|---|
| `coordinator` | charter 5 | the charter `goal-template` executes |
| `designer` | charter 7 | assignment |
| `coding` | charter 2 | the charter forge charter 1 names |
| `triage` | charter 8 | assignment |
| `goal-template` | template 4 | the template whose `resources` hold *its own goal branch* |
| `small-goal-template` | the stock's goal template | the stock's number |
| `main-rule` | landing rule 1 | the deployment's rule whose pattern ends exactly `main` |
| `goal-rule` | landing rule 2 | the deployment's or the project's rule whose pattern ends open on `temper`, or open on nothing beneath a segment `temper`: each covers a goal branch |

A role bound by assignment takes one of the charters the accepted
revision adds that no anchor took: of every assignment, the one that
meets the most obligations, ties going to the lower numbers. The check
prints the binding; a role left without a charter makes each of its
obligations missing.

**The rows, cut:**

| Section 14's row | Obligations |
|---|---|
| the coordinator's charter | setting: its instructions (`judged`); its model, `model.name`; its workspace and sub-agents, `inspect`, `shell`, `agents`, its `modify` free, since no word of the reference says what a goal's runs deliver, where it lets them (`../domain/agent/tools.md`); starting fresh, `resume` false; `waiting` 0; its wake policy, the members of `wake` `../draft/reference-code-production.md`, section 13 names (`words`, `notices`, `news`, `results`, `questions`), `answers` and `timers` free; its documents, `documents` (`contains`: `AGENTS.md` and the code's rules, by path, a heading free). Box: Claude Code, the program the document's model runs on. The tools it names are the authority's families, the rules' and policy 1's rows below |
| the designer's charter | setting: its instructions (`judged`); its model, `model.name`; the workspace, delivery and sub-agents, `inspect`, `modify` and `agents` true; `documents` (`contains`: `DESIGN.md`, the design rules, whole). Box: Claude Code, as the coordinator's |
| the goal template for drafts | setting: `executor.charter`, `{"role": "coordinator"}`; `llm`, whole (`fakes.md`'s choice); `tracked` (`at-least` 1); `contract.report` (`set`); `resources` (`contains`: *its own goal branch*) |
| a small goal's template | setting: the stock's goal template, `unchanged` |
| the coding agent's charter | setting: its instructions (`judged`); `shell` and `modify` true; the stock's run, which the table retains, `turns` (`at-least` 800) and `time` (`at-least` 28,800 s) |
| the forge's charters (kind 3) | setting: 1 names the coding agent's, its anchor; 2, the reviews', `unchanged` (unused, since neither landing rule asks a gate); no number 3, the foreman retired (`../draft/stock-charters.md`, 3) |
| each change's LLM choices | setting: the coding agent's model, `model.name`, gpt-6.1-sol; the coordinator's instructions (`judged`: a coding change at gpt-6.1-sol at the spec's effort, a designer's at `fakes.md`'s choice); box: gpt-6.1-sol on smith's direct backend with an OpenAI account's key |
| the project's policy and rules | setting: the rules' ceiling's and the project's own policy's role 0's tool families, its own ceiling holding them by jig's check, (`bits`: 1,983, the stock's with the families the coordinator's row names, 13.9), their note scopes (`bits`: 3, the goal's and the project's), and for role 0 no effect family (`lacks`: 64); their delegation's kinds (`contains`: each charter role's charter, the stock's procedures 2 to 4, the merge job, job 3, and role 0) and its tasks and depth (`at-least`: 1,024 tasks, 4 deep, N21); policy 1 free, a kind no obligation asks for: it reaches no project already made (jig's `domain/people/acts.md`, 2), and the walk's project is |
| the project's ways | setting: its `way`, the goal template for drafts, by role; its switches free. The project's questions are free too, a kind no obligation asks for: the coordinator names each change's route |
| the landing rule for goal branches | setting: `goal-rule`'s `pattern`; `ci` false, `up_to_date` false; `gates` and `approvals` empty |
| the landing rule for main | setting: `main-rule`'s `pattern`; `enforce` true, `ci` true, `up_to_date` true; `gates` and `approvals` empty; the deployment's rules `only` the two, so the stock's rule on every branch, which asks CI and a review, is replaced or removed |
| the repository's merge style | box: a merge commit, the repository's |
| the project's checks | box: `.temper/pre-pr` in the repository |
| g's tier | box: the document's tier `goal`, following goal branches, `span_ms` 1,200,000 |
| main's tiers | box: the document's tier `slow`, following main |
| questions | setting: the coordinator's instructions (`judged`: `up` for each change, a coding change's and a designer's alike, the foreman retired); box: four per change, the document's `forge.change_policy.questions`, the reference's explicit override of 4 |
| the merge job | box: the document's `merge_job`, `.temper/merge` in the repository |
| the profile's limits | box: the limits 1.2 names |

The golden's file holds each obligation with its id, its row, its kind,
and for a setting its subject, member, test and any value of its own;
its rows above are the design, the file their only encoding. The
triage's charter's row (`../draft/archive/reference-landing.md`, section 6;
`../protocol/configuration/reference.md`) cuts into its
instructions (`judged`), its model, and `inspect` true with `modify`,
`shell` and `agents` false. No field of the table is a gap: the design
carries each, as a setting or the box's. The kind
stays for a field a later edition of the table adds before a design
carries it.

**Beyond the obligations,** the check fails on:

- *a setting no obligation asks for:* an item of the accepted revision
  whose setting binds to no role and is no obligation's subject, of a
  kind the golden does not name as free, and no landing rule removed,
  since the stock's on every branch must go or be replaced;
- *a rule made structure without the person's choice*
  (`../draft/configuring.md`, section 2), as far as settings show it: a
  landing rule's gate or approval, or a charter's family the golden
  does not give, fails its `equal` obligation. The pass makes no such
  choice (1.3), so any is the agent's;
- *what needs a restart, proposed rather than said:* settings cannot
  carry it, so the judge reads the proposal and the chat for it
  (1.4).

### 1.2 The box

`temper box` with its owned forge (`../protocol/setup/box.md`),
its settings the stock's. What the box brings, and the pass confirms
before the chat starts (temper's through ctl, the repository's through
the forge's API as an observer, the limit settings through ctl):

- **the models:** Opus 5.5 on Claude Code for the coordinator and
  designers; gpt-6.1-sol on smith's direct backend, on an OpenAI
  account's key, for producers and the triage; a real model
  for Ask temper, charter 11's;
- **the profile's limit settings,** `limits` and `temper-limits`, over the
  stock, confirmed through ctl (`../protocol/configuration/limit-settings.md`, 2);
- **the document's members the profile sets:** a change's question bound,
  `forge.change_policy.questions`, at 4; and `landing.json`, the project's
  merge job and its two tiers (13.9);
- **the project,** with its repository adopted, holding `AGENTS.md`,
  `DESIGN.md`, its code rules, `.temper/pre-pr`, `.temper/merge`, the
  tiers' workflows and a merge-commit merge style; and, at the path the
  walk's brief names, `../draft/reference-code-production.md` and
  `../draft/archive/reference-landing.md`, the person's process documents. It holds no
  recipe of the reference either: a copy of fabrig keeps neither the
  profile, this crate nor the plans that describe them, and an agent
  found reading a description of the reference's settings elsewhere
  is a finding against the walk's premise.

Each is a box obligation, recorded present or lacking with what the
pass saw. A lacking one ends the walk before the chat, a finding against
the box, never the agent.

The walk's bounds, the user's (2026-11-04): the box's limits are the
reference's profile whole, as above, its spends and times the box's; the
pass takes at most 3 hours of wall time from the box's start to the
stage's verdict, the Ask temper chat at most 40 of the person's
messages, and the judge one session at Opus 5.5, high, within 30
minutes.

### 1.3 The evidence

After the acceptance, the pass collects what the check reads into one
directory, with `temper-stage-one collect`, through `temper ctl`'s
reads alone (`../client/ctl/tools-acts.md`, 1): the revision's record; its
items, each value whole, as a settings file; every setting's current
value, the project's policy and temper's kinds among them, as a
settings file; the proposal's review and the chat, as text. The pass
adds what only it holds: the box's confirmations, the agent's
description of the accepted settings back as a workflow (asked once
the proposal is accepted, `../draft/configuring.md`, section 6), each
document an accepted charter names, read at its branch's head through
the forge's API, and the person's two documents. The pass answers the
agent's questions from the documents and its brief, chooses no rule to
become structure, and accepts with running work staying as it is.

The revision's edits alone form `items.json`; `current.json` is collected
separately. The collector never unions earlier revisions or substitutes current
values for unrelated accepted edits. The recorded-ctl regression carries the
complete note proposal's accepted set and retains designer and triage bindings;
the eleven-proposal workaround's incomplete final revision leaves them unbound.
The engine domain world proves bounded note writes, including a line past one
call, exact whole-value reconstruction, no application before acceptance and
one complete accepted revision, with refusal and decided-call replay cases (`checks.md`, section 4).

### 1.4 The check and the judge

- **The check,** `temper-stage-one check EVIDENCE`: the binding, then a
  line per obligation (met; wrong, with what it found and expected;
  missing; judged; unchanged; named, and where; unnamed; confirmed;
  lacking), then what fails beyond them. It passes when every setting is
  met, judged or unchanged, every gap named, every box obligation
  confirmed, and nothing fails beyond them. Exit 0 passed, 1 failed, 2
  for evidence it cannot read.
- **The judge,** an LLM, reads what code cannot: `temper-stage-one judge
  EVIDENCE` prints its brief, whole, for a session of its own (never the
  pass's), at Opus 5.5:
  - *the rules:* the golden's rules, the reference's text that stays
    text (`../draft/configuring.md`, section 2's table, and the
    instructions `../draft/reference-code-production.md`, section 14
    gives each charter), each with the role it
    concerns; for each, whether it reaches that role's runs, from where
    the person wrote it, through the bound charter's instructions or a
    document it names: `reaches`, `missing`, or `elsewhere` (another
    role's), with the quote;
  - *the description* the agent gave back, against the reference's
    section 1: `matches` or `differs`, with the differences. It differs
    where it says what the section contradicts, or leaves out what the
    flow turns on (the goal holding its branch, every change landing
    there, designers before their changes, how questions go, the whole
    landing into main at the end); a number or other detail it leaves
    out is no difference;
  - *what needs a restart:* whether the proposal or the chat claims to
    set what the box brings rather than saying it needs the box:
    `said` or `proposed`, with the quote.

  Its verdict is a JSON file in the brief's form, which the pass saves
  beside the evidence.
- **The stage's verdict,** `temper-stage-one verdict EVIDENCE`: passed
  when the check passes, every rule reaches its role, the description
  matches, and what needs a restart is said.
- **It moves with the reference.** A design that comes to carry a gap
  makes it a setting: the golden's file changes with the status entry
  that closes, in the same branch.
- **Tested in the gate as a tool is:** the crate's own step tests, in
  milliseconds: the golden decodes, and each setting obligation's member
  resolves in the settings form's `FIELDS` (`../client/ctl/tools-acts.md`, 1)
  and in the profile; the check over evidence made from the profile
  itself, as if the agent had proposed it, passes with every setting met,
  and with the profile's charters renumbered still passes; each failure
  by one mutation of that evidence (a value changed, a charter added,
  a role's charter removed, a gap's words absent, a box obligation
  lacking); the judge's brief for that evidence, as golden text; the
  verdict's decoder refusing an unknown rule, a rule twice or missing;
  `collect`'s assembly from recorded ctl outputs.
- **Walked,** never in the gate: a pass of QA, run again whenever the
  agent's instructions, the documentation in the binary or a reference
  milestone's settings change (`../draft/configuring.md`, 8.1).

## 2. Open questions

None.
