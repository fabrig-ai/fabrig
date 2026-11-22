# The tools' acts

This document holds the tools' acts, split from [`tools.md`](tools.md).

## 1. Acts

Each act is one keyed request through `jig-client`'s request table,
answered once durable (`requests.md`, section 1). `K` marks the tools whose key the
agent must name.

| Tool | What it does | jig's | K |
|---|---|---|---|
| `chat start --project PROJECT [--template TEMPLATE] TEXT` | starts a chat from TEMPLATE, the project's chat template when none is named: template 5 is Ask temper (`../../domain/agent/ask-temper.md`) and template 6 a design session (`../../domain/agent/agents.md`, 4); a template that is no chat's, or not offered, is refused naming the templates `me` offers | `shapes::chat`, a `Create` | yes |
| `role start --project PROJECT --budget AMOUNT [--charter CHARTER] TEXT` | starts a standing role from template 7 (`../../domain/agent/standing.md`, section 2): TEXT its spec's words, what it covers and how; AMOUNT its budget each period; run by CHARTER, the stock's charter 13 when none is named; prints the task it made, or the proposal a batch beyond the person's role became | `shapes`' fill of template 7, a `Create` | yes |
| `role talk TASK TEXT [--timeout DURATION]` | words to a standing role in a talk (`../../domain/agent/standing.md`, section 7): opens one, ending its open session, or carries on the party's own; then waits, as `task show TASK --until replied` does (`tools-waiting.md`, section 1), and prints the role's reply, the text of its runs' turns after these words, as `chat show` prints a conversation's; a refusal exits 2 naming it, `not a standing role`, or `talking with PARTY` for another party's open talk; past the timeout, or the task held or ended first, it prints what has come and exits 3 as a wait not met | `Talk`, then `task{n}`'s watch and `activity{n}` through `thread` | yes |
| `role talk TASK --leave` | leaves the party's talk with the role, ending its session; prints `left: task N`; with no talk of theirs open, refused, exit 2, `no talk to leave` | `Leave` | yes |
| `task message TASK TEXT` | words to a task | `Message` | yes |
| `chat close TASK [--move TASK]... [--reason TEXT]` | closes a chat: its live delegates cancelled, or moved to the party where named; the reason empty unless given, as jig's reference UI sends it | `shapes::close_chat`, a compound request | yes |
| `repository adopt --project PROJECT [--forge FORGE] --owner OWNER --name REPOSITORY [--ids IDS] --landing BRANCH [--role owned\|fork\|context] [--prefix PREFIX] [--no-home] [--no-ci] [--check GATE]...` | adopts a repository | the forge family's request (`../forge/family.md`, 4) | yes |
| `change open --project PROJECT [--repository OWNER/NAME] --branch BRANCH [--at COMMIT] [--into BRANCH] --title TEXT [--body TEXT] (--budget AMOUNT \| --as-allowed)` | opens a change from a branch the person pushed, as their own change task; its usage, answer and refusals are `areas.md`, area 6's | the forge family's `OpenChange` (`../forge/records.md`) | yes |
| `repository make --project PROJECT [--forge FORGE] ORG/NAME [--no-home] [--until adopted] [--timeout DURATION]` | makes an empty repository, its starter workflow in it, and has it adopted into the project: owned, landing on its default branch, with CI, the project's home unless `--no-home` (`../../domain/forge/adoption.md`, 5) | the forge family's `MakeRepository` (`../forge/records.md`) | yes |
| `repository copy --project PROJECT [--forge FORGE] ADDRESS [--as ORG/NAME] [--no-home] [--until adopted] [--timeout DURATION]` | copies a public repository, `HOST/OWNER/NAME`, into `OWNER/NAME` on the forge, or `--as` where given, and has it adopted as `repository make` does; the address refused, naming the form to write, before anything is sent | the forge family's `MakeRepository` | yes |
| `draft ready --project PROJECT [--owner OWNER --name NAME] [--chat TASK] PATH` | says the draft at PATH, landed on the repository's landing branch, is ready to start a goal (`areas.md`, area 7) | the forge family's `SetDraft` (`../forge/records.md`) | yes |
| `draft not-ready --project PROJECT [--owner OWNER --name NAME] PATH` | says the landed draft at PATH is not ready (`areas.md`, area 7) | the forge family's `SetDraft` | yes |
| `task answer TASK QUESTION TEXT` | answers question QUESTION of the task | `Answer` | |
| `proposal decide PROPOSAL accept\|reject\|pass-up [REASON] [--running stays\|moves]` | decides a proposal: PROPOSAL is `tN/K` for task N's, `pN/K` for party N's | `Decide` | |
| `task decide TASK --hold REV release\|leave\|pass-up [REASON]` | decides an escalation | `Decide` | |
| `settings propose FILE [REASON] --against REVISION` | proposes the whole settings set to the owners of its items, the deployment's and each project's it names; FILE or `-` shares the format and bounds of `settings apply`; the optional reason is forwarded unchanged | `Propose` settings | yes |
| `settings apply FILE... --against REVISION [--running stays\|moves]` | changes the settings in one decision: each FILE (or `-`, once) holds items, the files' overlays of one charter joined and every other setting named once (`overlays.md`, 3); REVISION is the newest revision the caller read; running work stays unless `--running moves`; answered the new revision's number | `ChangeSettings` | yes |
| `revision advance REVISION` | moves every live agent task on held REVISION that may move to the newest, each at its next run; prints jig's answer, `moved 3 tasks to revision 9; 1 could not, and finishes on 7` | `MoveToNewest` | yes |
| `revision restore REVISION [REASON]` | proposes the settings as REVISION left them, against the newest, to the owners of its items: jig's kinds, temper's (`../../protocol/configuration/settings-kinds.md`, 1) and each project's policy a later revision set; REASON follows jig's "restores revision R"; prints the proposal's number and what it leaves out, a deleted project's settings and policy, by kind, project and number; `proposal show` then shows its items, and an owner decides it with `proposal decide`. A revision forgotten past the history exits 2 naming the oldest kept, `temper ctl: revision R is forgotten; the oldest kept is revision O`, and one too large to restore says so, exit 2, a later revision restorable instead | `Restore` (jig's `domain/engine/settings.md`, 10) | yes |
| `deployment checkpoint [--timeout DURATION]` | an owner's checkpoint of the serving deployment; prints its name and figures, waiting 10 minutes unless `--timeout` says otherwise (`../../protocol/checkpoints.md`, 3) | the forge family's `Checkpoint` | |
| `task release TASK --hold REV [--again]` | lifts a hold | `Release` | |
| `task stop TASK` | cancels a task's run, the task held | `Stop` | |
| `task move TASK --under TASK` | makes the second task the first's requester (jig's `domain/tasks/changes.md`) | `Move` | yes |
| `task succeed TASK --by TASK` | names a live task the successor of an ended one: its open questions, subscriptions and kept news go to it | `Succeed` | yes |
| `finding file --project PROJECT --kind qa\|flake\|red\|review\|feedback [--severity low\|medium\|high] [--area AREA] TEXT FILE` | files a finding (`../../domain/findings.md`, section 4): TEXT its title, FILE, or `-` for stdin, its body, `medium` when no severity is named; prints `filed: P/N, held by HOLDER`; an area the project lacks exits 2 naming its areas (`../../domain/areas.md`, section 7) | `FileFinding` (`../forge/records.md`) | yes |
| `finding hand FINDING --area AREA TEXT` | hands a finding on to an area, TEXT why, in a line: its new owner hears it (`../../domain/areas.md`, section 5); prints `handed: P/N, held by HOLDER` | `HandFinding` | yes |
| `finding route FINDING --goal TASK`, `finding backlog FINDING`, `finding drop FINDING REASON`, `finding repair FINDING... TEXT`, `finding blame FINDING (--landing COMMIT\|--none) TEXT` | triage a finding: `routed` to a goal, news to it; `backlog`; `dropped`, the reason kept, its issue, once it has one, closed with it (`../../domain/findings.md`, section 6); a repair goal; its blame moved (`../../domain/red-repairs.md`, section 8) | `TriageFinding` | yes |
| `need ask --project PROJECT [--party PARTY\|--role ROLE] --choice TEXT --choice TEXT [--choice TEXT]... --why TEXT TEXT` | asks a person a question with its choices and why (`../../draft/standing-roles.md`, section 8): a person task addressed to PARTY or to ROLE, role 0, the project's owners, when neither is named; its spec TEXT, a blank line, then `Why: ` and the why; its contract a verdict among the 2 to 6 choices, in order, codes from 1; prints the task, whose answer `task show` reads; jig refuses, in its words, a person the asker's role may not address | `Create` with a person executor, made whole, as `shapes` offers none | yes |
| `follow start --project PROJECT FOLLOW COMMIT` | gives the owner's word to the follow change for the landing COMMIT, lifts its owner's hold, or makes it again after an end without landing (`../../domain/forge/follows.md`, section 5) | the forge's request | yes |
| `follow hold --project PROJECT FOLLOW COMMIT --reason TEXT` | keeps the follow change for COMMIT from producing, its reason in the lag | the forge's request | yes |
| `follow skip --project PROJECT FOLLOW COMMIT --reason TEXT` | counts the landing COMMIT followed with no change, cancelling its follow change and keeping the chain whole | the forge's request | yes |
| `follow merge --project PROJECT FOLLOW COMMIT COMMIT...` | makes the first follow change of consecutive landings follow them all, cancelling the others' and keeping the chain whole | the forge's request | yes |
| `tier hold --project PROJECT [--owner OWNER --name NAME] TIER --reason TEXT` | holds the repository's tier TIER, the project's home's when no repository is named: it starts no run until released, its landings still verifying (`../../domain/after-landing-runs.md`, section 2, *A person's hold*); prints `held: TIER of OWNER/NAME`, exit 0; refused, exit 2, `no repository OWNER/NAME`, `no tier TIER on OWNER/NAME`, `TIER of OWNER/NAME is held already by party N: REASON`, or the reason's bounds (1 to 1024 bytes of UTF-8) | the forge family's `TierActRequest` (`../forge/records.md`) | yes |
| `tier release --project PROJECT [--owner OWNER --name NAME] TIER` | releases the held tier: its next run starts at its branch's newest tip; prints `released: TIER of OWNER/NAME`, exit 0; refused, exit 2, `no repository OWNER/NAME`, `no tier TIER on OWNER/NAME`, or `TIER of OWNER/NAME is not held` | `TierActRequest` | yes |
| `qa choose LANDING` | chooses a landing's mark for the next QA pass (`../../domain/findings.md`, section 8) | the forge's request | yes |
| `task hold TASK --stop\|--drain REASON` | places a hold of the party's over TASK's subtree: stop cancels its runs at once, drain lets them finish their turns; a party's hold lifts by `hold lift` alone (jig's `domain/tasks/lifecycle.md`, 10) | `Hold` | yes |
| `project hold PROJECT --stop\|--drain REASON` | places a hold of the party's over the whole project, the party's chats exempt | `Hold` | yes |
| `project make NAME` | makes a project from a copy of policy 1, as the client switcher does; prints its number, kept on key replay | `MakeProject { name, policy: 1 }` | yes |
| `party make --project PROJECT --role ROLE --out FILE TEXT` | makes a service, a party with no person, in PROJECT holding role ROLE, a role's number, TEXT its name, by the project's owner (`README.md`, section 7; `../../draft/standing-roles.md`, section 8); writes what `sign-in --service` reads to FILE, `0o600`: the engine's address this ctl signed in at and the credential jig's clients' component answers once; prints `made: party N`. FILE existing refuses it before anything is sent, exit 1; the credential is never printed, never in argv, and a replay under its key prints the party and writes nothing | `MakeService { project, name, role }` | yes |
| `party reissue PARTY --out FILE` | reissues service PARTY's credential and writes its service link to FILE, or to stdout alone for `-` (`README.md`, section 7) | `ReissueService { party }` | yes |
| `project amend PROJECT --visibility public\|private` | makes the project readable by anyone without signing in, or private again (jig's `domain/people/public-reading.md`, 2), by its owner; prints `project 1: public` and, made public, the public page's path from `me`'s public identifier, `/public/projects/3f2a9c01d4e5b6a7`, and on stderr, as its help does, "Anything said in this project's chats may show on its public pages, in the titles of its goals and tasks. On a public repository, a change's pull request shows what its agent wrote too." (jig's `domain/people/public-reading.md`, 3); project 0 and a member are refused, exit 2 | `SetVisibility` | yes |
| `project delete PROJECT` | deletes the party's own project, by its owner (jig's `domain/people/acts.md`, 2), a fresh store's default project among them: closed at once, gone from `me`'s project list, and deleted whole after; `project show` then says it was deleted (2). Its ask is built from the call alone, `RemoveProject { project, grace: None }`, since jig refuses an owner's deletion while any task of the project is live, so no chat is left to close: PROJECT, a number, is sent as it is, in `me` or not, so that the same call again under its key, once the project has left `me`, reaches jig's kept answer; a name is found in `me`, and one not there sends nothing, exit 2, `unknown project`. Prints `deleted: project 3, fabrig`, the answer's number and the name `me` gave PROJECT at the call, the number alone where `me` gave none; the same call again under its key prints it again, after the deletion as while it lasts, within the engine's retention. Refused, exit 2, in jig's words after `project delete: `: a project with live tasks, naming its live roots, `project delete: This project has live tasks; end them first: 12, 15` and `, and 2 more` past jig's 32; a member who is not its owner; a project unknown, or already being deleted under another key. No confirmation, as no act asks one (below) | `RemoveProject { project, grace: None }` | yes |
| `project fund PROJECT AMOUNT [--role ROLE]` | sets what the project's period may spend, its spend per period, to AMOUNT, in ctl's amount (`results.md`, 1), or with `--role` that role's; either resizes what the open period gave (jig's `domain/authority.md`, section 7), so a raise funds the next chat at once, and each period the calendar opens takes it (`../../domain/routing.md`, 7). An owner's chats are funded from their pool, their role's, and pools reserve from the project's period, so a raise for them is two calls, the project's first, then the role's; a role's raise past the period's room is refused, exit 2, naming what the period has left. A cut goes the other way, the role's first, since a project's spend below a role's is refused; it keeps each budget at what it has used. A refusal past a ceiling (jig's typed ceiling, its holder, dimension and, for a scalar, its value) exits 2 with one line naming it and the next step: `project fund: past the rules' period spend, V; raise the rules first with temper ctl settings apply`; with `--role`, `past the project's period spend, V; fund the project first: temper ctl project fund P AMOUNT`; a cut, `below role R's period spend, V; lower the role first: temper ctl project fund P AMOUNT --role R`; an authority over its ceiling when rechecked, `the project's authority is past the rules' DIMENSION[, V]; raise the rules with temper ctl settings apply, or narrow the project's policy` (for a role, `role R's authority is past the project's DIMENSION[, V]`), a list's dimension (tools, executors, grants, notes) named without its contents, read whole with `settings show`. Its answer is the edit, `project 1: spend per period 4496.96` or `role 0: spend per period 4151.04`, the same under one key by construction (`requests.md`, section 1); then a read marked as such, `now: period 1: budget 4496.96, left 4495.90`, and with `--role` each of the role's pools now: a replay prints the edit as first answered and today's numbers after it | the project's policy edit, `ProjectSpend`, or with `--role` `RoleSpend` | yes |
| `hold lift HOLD --rev REV [--with-tasks]` | ends a placed hold; with `--with-tasks`, releases every task it held that nothing else holds | `Lift` | yes |
| `task cancel TASK [--grace DURATION] REASON` | cancels a task and its tree; a task that has ended refuses a cancel (exit 2), saying how it ended, and the same cancel under its key gets its first answer | `Cancel` | |
| `push add FILE [--label LABEL]` | turns push on for a device: FILE is a browser's push subscription as its `toJSON()` writes it, `{"endpoint": URL, "keys": {"p256dh": KEY, "auth": KEY}}`, each key base64url without padding (65 and 16 bytes decoded), other members ignored, at most 4,096 bytes; LABEL, 1 to 60 bytes, by default `temper ctl`. Prints `device 1: added`; refused, the refusal's words and exit 2: "This deployment doesn't push.", "That push service isn't one this deployment sends to.", "That subscription's keys aren't a push subscription's.", "You have 8 devices already: remove one first." or "Only a person is pushed to." (`../../domain/forge/attention.md`, 7.1) | the forge's `AddDevice` | |
| `push remove NUMBER` | turns push off for a device of the party's; prints `device 1: removed`, or "No device NUMBER.", exit 2 | the forge's `RemoveDevice` | |

- **Settings FILE is JSON:** `{"items": [ITEM, ...]}`, at most one item per
  setting, each `{"setting": SETTING, "set": VALUE}`,
  `{"setting": SETTING, "edit": "removed"}`,
  `{"setting": SETTING, "edit": "reset"}` or, for a charter, `{"setting":
  SETTING, "overlay": OVERLAY}`, an overlay over its default, its text
  given literally or by directories of charter files (`overlays.md`). SETTING is
  `{"kind": "charter"|"template"|"policy", "number": N}`,
  `{"kind": "rules"}`, `{"kind": "landing-rule", "number": N}` for the
  deployment's, `{"kind": "landing-rule", "project": P, "number": N}`
  for project P's, `{"kind": "forge-charter", "number": N}`, N 1
  the change's charter and 2 its reviews', `{"kind": "charter-words",
  "number": N}`, N a charter's number, `{"kind": "project-ways",
  "project": P}`, `{"kind": "project-questions", "project": P}`,
  `{"kind": "project-classes", "project": P}`,
  `{"kind": "project-words", "project": P}`, or `{"kind":
  "project-policy", "project": P}`, project P's own policy (jig's
  `domain/engine/settings.md`, 2), set alone and at most once in a file; VALUE is exactly the
  `value` member `settings show … --json` prints: for a landing rule, `README.md`, section 3
  's rule member but `connector` and `kind`, its `pattern` in `FIELDS`'
  `landing-pattern` form, its segments as hex
  (`../../protocol/configuration/settings-kinds.md`, 1), its `sets` a list of
  path sets' numbers, `[1]`, optional, `[]` left out, and printed only
  where it names one, so a rule by none reads as it always has
  (`../../protocol/configuration/engine.md`), for the forge's charter a charter's
  number, for a project's ways `{"drafts": BOOL, "goals": BOOL, "changes": BOOL,
  "way": N}`, for a project's questions `"none"` or `"up"`,
  for a project's classes `{"items": [ITEM, ...], "sets": [SET, ...]}`,
  each ITEM `{"kind": KIND, "words": TEXT, "class": CLASS}` or `{"set": N,
  "words": TEXT, "class": CLASS}`, `words` optional and empty by default,
  KIND one of the ten kinds' names and CLASS `"decide"`, `"ask"` or
  `"ask-away"`, each SET `{"number": N, "name": TEXT, "patterns": [TEXT,
  ...]}` (`../../domain/forge/attention.md`, 6), refused as `Classes::check` refuses,
  for a charter's words its line as a string, for a project's words
  `{"headline": STRING, "lede": STRING}`,
  and for a project's policy a policy in the form `settings show
  policy NUMBER --json` prints, the stock's policy 1 a project's starting
  point. Decoded bounded (`results.md`, 4): the file's size, the item count and every
  value's bounds are jig's (`ChangeSettings`' own, and the limits the engine's
  `me` gives where it gives them). Past a bound it is refused before anything
  is sent, naming the item and the bound. Unknown members are refused,
  naming them. A member `FIELDS` marks optional may be left out and
  takes the default its row names: a template's model choice is
  `{"main": {"model": M, "effort": E}}` alone, its `account` none and
  its `subagents` empty; a template's `resources`, the symbols a task
  filled from it holds, is a list of `{"connector": C, "symbol": S}` as
  jig's format has it (jig's `protocol/configuration.md`, 3.3), left
  out `[]`, so the reference's goal template names the forge's *its own
  goal branch* as `{"connector": F, "symbol": 1}`, F the forge's
  connector (`../../domain/forge/README.md`, section 2); a required
  member left out is refused naming its path, `llm.main`. Every refusal a person reads numbers items from 1, the
  decoder's and the engine's alike (jig's index plus one), and names a
  member by its path in the file, `charter.alternatives`, never by the
  wire's limit's name. `null` is refused where a member takes a value:
  a charter's `model` is a model and its `alternatives` a list, and an
  authority's models are `"all"` or a list of model names. A project's
  policy `removed` or `reset`, or a second one, is refused before
  anything is sent, naming the item. So is a template's or charter's effort
  that its model doesn't offer, where `me` shows that model in any of the
  party's projects: exit 1, nothing sent, `settings file item 1: model M offers
  no effort xhigh; it offers high, medium`, or `it offers none`. A model
  `me` doesn't show goes to the engine, which refuses it by name (jig's
  `client/domain/contract/README.md`, 3). jig's kinds are the deployment's; a landing rule names
  its scope, the deployment's or a project's, as a project's policy
  names its project, and a set that touches a project's needs an owner
  of it, one that touches the deployment's too one person who owns both
  (jig's `domain/people/acts.md`, 1); `settings propose` sends such a set
  to them, and `settings apply` from one who is not both is refused
  naming the item, exit 2.
  `proposal show` prints the values of temper's items from the forge's
  `ProposalItems` read, and a project's policy as it is now and as it
  would be, each whole, from jig's review; `settings show` lists
  temper's settings with jig's, by the names above, and no project's
  policy, which is no setting (`project show` prints its roles and
  spends).
- **A goal template's work** (`../../protocol/configuration/settings-kinds.md`, 1, kind
  11): its SETTING `{"kind": "template-goal", "number": N}`, N the
  template's, the deployment's as a template is, and its VALUE
  `{"design": BOOL, "close_out": BOOL}`, both members required, as
  `settings show template-goal N --json` prints it; a proposal card
  names it `template-goal N`.
- **A charter's parts and a later choice** (`../../draft/per-wake.md`,
  sections 3 and 5; jig's `protocol/configuration.md`, sections 3 and
  3.1). A charter's `parts`, which `FIELDS` marks optional, left out
  `[]`, is a list, in order, of `{"on": [REASON, ...], "instructions":
  TEXT}`: each `on` a nonempty list of `first` and the kinds that
  choose parts, by jig's store names (jig's
  `client/domain/contract/requests.md`, 2), each named once, and its
  instructions text, bounded as the charter's own are. A run reads the
  charter's `instructions`, then each part whose `on` meets why it runs.
  A template's model choice may add `"later": {"main": {"model": M,
  "effort": E}, "subagents": [...]}`, its `subagents` left out empty:
  the choice every run after the task's first activation's takes (jig's
  `domain/tasks/README.md`, section 3), its efforts checked against `me` as
  the first choice's are; left out, it has none. `settings show … --json`
  prints `parts` only for a charter that has some and `later` only for
  a choice that has one, so a value without them prints as before. The
  decoder refuses, before anything is sent, naming the item and the
  member's path, its indices from 0 as a document's are: a name that is
  no reason, or one every run is offered, `settings file item 1:
  charter.parts[1].on[0]: "answer" goes to every run and chooses no
  part`; a name twice; an empty `on`; and more
  parts than `tasks.charter_parts`. A charter whose runs resume may hold
  parts, as jig admits them (jig's `domain/tasks/standing.md`, 5.4). jig's
  refusal of a change names the charter and no part
  (`../../draft/per-wake.status.md`, 3), so a malformed part prints as
  `settings apply: out of shape; item 1; charter 5: shape`.
  A charter's stored policy, its parts with it, past
  `root.journal.run_bytes` also names its measured bytes and the bound:
  `settings apply: out of shape; item 1; charter 5: shape; stored policy: supplied 70112; maximum root.journal.run_bytes 65536`.
  The run's settings tool uses the same words; JSON names the member
  `stored_policy` (`../../domain/root.md`, section 9).
  In text, `settings show charter N` prints,
  after the value, a line per part, by its path, with its `on` and its
  instructions' bytes, `parts[0]: on first; 4210 bytes`,
  and `settings show template N` a line for a later choice, `later:
  claude-opus-5-5 high`.
- **The form's fields are one table,** `FIELDS`, beside the form's
  decoder in the crate ctl and the runs' `propose_settings` share
  (`temper-settings-form`): a row per member of a shape, by the shape's
  name and the member's (`authority.spend`, `model.name`), with its JSON
  type, its bound where the settings' limits give one, whether it may be
  left out and its default then, and a line saying what it does to
  work. A shape that recurs (an authority in a policy's
  ceiling, its roles and the rules; a model in a charter's model and its
  alternatives) has its rows once. `resolve` walks a path from a kind
  through the shapes (`policy.ceiling.spend`, `charter.model.name`) to
  its row. The reference page `settings-fields` is rendered from it, a
  section per shape (`../../protocol/documentation.md`, section 6), and a
  step test holds it to the form: every member that `settings show …
  --json` prints for each stock value, and every member the decoder takes,
  resolves to a row, and every row is reached. temper's own kinds'
  rows come with their decoder.
- **The arguments are jig's,** named as its operations name them, with
  the revisions they need (jig's `client/domain/contract/documents.md`, 2): a
  proposal by its proposer and number, `t41/3` for task 41's third,
  `p3/17` for party 3's seventeenth, as every result prints it, an
  escalation and a release by the hold's revision. An agent copies them
  from the acts a result printed (`results.md`, 1); it never has to know which an
  act needs.
- **`repository adopt` takes the adoption form's fields and defaults**
  (`../forge/family.md`, 4): forge 1, role `owned`, prefix `temper/`, the home
  and CI on. `--landing` names the landing branch. A context repository
  is never the home: `--role context` turns the home off. `--no-ci`
  needs at least one `--check`, a gate number, each number once. `--ids
  IDS` is a picker row's repository id and owner id, `R/U`, as `forge
  repositories` prints them (`areas.md`, area 6), carried with that row's
  `--owner` and `--name`; without it the request carries no ids
  (`../forge/records.md`). The form's checks are the parser's: a call that
  fails one exits 1 before anything is sent.
- **No confirmation.** The person's client says what an act will do
  before sending it (`../README.md`, 3.6); an agent reads it in the
  document's acts and their effects, and the revision is its promise.
- **A draft beyond the party's bounds** is compared with the bounds the
  document gives, as the person's client does (jig's
  `client/domain/contract/documents.md`, 3): within, sent; between the bounds,
  sent as `Propose` where it is proposable, and the result says so; past
  them, refused by the client, nothing sent. Where `me` gives no bound
  for a draft, it is sent and the engine's answer printed.

`repository make` and `repository copy` answer once the request is kept.
`--until adopted` then follows the `forges` watch's row: gone with its forge
still listed, it is adopted, status 0; failed, its step's words and status 1;
past `--timeout`, the row's state and status 2, as a read's timeout is, the
copy going on. These two are the only acts that take `--until`, and a
`--timeout` without it is refused, as on a read.

A charter's `tools` in a settings file is an integer mask in the same bits
as an authority's tools. Left out, it is 1,023, every jig family but
settings, so a settings file written without that member does not offer the
settings family to its runs (2026-11-04).

## 2. Open questions

- **The settings reference's help pointer** (`tools-acts.md`, 1): `settings-fields`'s
  `reserve_time` description says "See settings help". The field table
  that generates it needs to name `temper help settings`, with a check
  over that generated pointer. Default: leave this non-waiting reference
  to separate wording work; `results.md`, 1's waiting-line pointers are checked now.

- **An effort refusal for a model absent from `me`** (`tools-acts.md`, 1): the local
  check knows only the models `me` shows in the party's projects. An item
  naming another model goes to the engine, whose refusal names the model
  alone. Naming the effort too needs jig's refusal to carry it, a protocol
  decision. Default: the local check and the engine's model-only refusal.

- **A step before a project goes public** (`tools-acts.md`, 1): `project amend --visibility
  public` acts at once, so a person who has skipped the publication help
  meets its words only in the completed answer's stderr, after visibility
  has changed. A flag that names them could stand first. Default: none,
  the request `ui-public-3` asking the answer or the help to say so.
