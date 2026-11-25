# The tools' waits

This document holds the tools' waits, split from [`tools.md`](tools.md).

## 1. Waiting

```
READ [--since DIGEST] [--until CONDITION] [--timeout DURATION]
```

- **A watched read** is one whose arguments are a watch's subject (jig's
  `client/domain/contract/watches.md`, 1.1): `me`; `inbox` with no flag, the
  inbox's watch being the party's whole inbox; `task show TASK` with no
  flag; `task tree TASK`; `task run TASK`; `task list` with what its
  watch takes alone, without `--after` or `--ended`, `tasks{query}`
  following live tasks; `account list` (area 5 of `areas.md`); `forge list`
  and `change show TASK` (area 6). Given `--since` or `--until`, it
  waits; given neither, it reads at once and prints its digest.
  `--timeout` alone, with neither, and a wait's flag on any other read,
  `task show TASK --tree` among them, are usage errors, exit 1. There is
  no `wait` tool.
- **A digest** is the first 8 bytes of the SHA-256 (skein's `runtime/lib-bytes.md`,
  1.2) of the subject's document as `--json` prints it (`results.md`, 2), without
  its `digest` field, printed as 16 hex digits. A read and a wait digest
  one form, the watch's document: for `inbox` and `task list`, whose
  read is a page and whose watch a window, the rows the watch's first
  window holds, in its order, and whether more follow, within the tools'
  page of rows (`bounds.md`, section 1). `task list`'s read makes that form from its
  own rows; an unfiltered `inbox` read opens the inbox's watch once for
  it, since a needs page leaves unfetched the updates the watch's window
  holds. Each prints its cursor beside the digest, outside it. A read's digest is
  therefore a wait's first delivery's while nothing moved between them,
  so a wait `--since` a read's digest never answers at once. Two calls
  of the same tools, under the same limits, print the same digest for
  the same document. jig's revisions move only with what a party decides
  about (jig's `client/domain/contract/documents.md`, 2), so they cannot say
  that a document changed; the digest can, and needs nothing of jig. It
  is not the mark the engine keeps per watch (jig's
  `client/domain/contract/watches.md`, 1.2), which no tool prints.
- **Which reads print one:** every watched read, and every wait's
  answer. `task show TASK --tree --run` prints one for each of its
  parts, `task show`, `task tree` and `task run`, each over its own part
  of the result. `--activity`'s lines, `chat show`, `task activity` and
  `task turn` are pages, not subjects, and print none.
- **`--since DIGEST`:** answered at the first delivery whose digest
  differs. **`--until changed`,** a condition of every watched read:
  answered at the first change after the watch's first delivery.
- **`--until CONDITION`:** answered at the first delivery that meets it,
  the first delivery included; with `--since`, at the first delivery
  whose digest differs and that meets it. Conditions are named facts
  over the document:

  | Read | Conditions, beside `changed` |
  |---|---|
  | `me` | `waiting` (something waits for the party) |
  | `inbox` | `waiting`; `updates` (an unread update) |
  | `task show TASK` | `ended`; `waiting-on-me`; `held`; `replied` (a chat's run done for now); `phase:<phase>` |
  | `task tree TASK` | `ended` |
  | `task run TASK` | `ended` |
  | `task list` | none |
  | `account list` | `usable` (an account the party's work may use has a source usable on a host); `settled` (no sign-in or check the party asked is in progress) |
  | `forge list` | `settled` (no forge setting up or connecting: no set up, connect or install the deployment asked is in progress); `connected:FORGE` (forge FORGE is connected); `made:FORGE` (forge FORGE's first repository is made, `../../protocol/setup/owned-repositories.md`, 1) |
  | `change show TASK` | `pull` (its pull request opened: the panel names its number, which it keeps after the pull request closes or lands); `ci:pending`, `ci:passed`, `ci:failed` (CI at its head, as the panel says it); `held` (it is held); `landed` (its step is landed, or one after it: verifying, red, cleared); `cleared` (its step is cleared, or it ended landed and uncleared, as the panel says) |

- **A change's wait outlives its panel's watch,** as the change's page
  does (`../forge/served.md`, 2, **Its subject**): when the panel's
  watch opens `Unknown` or ends, the wait reads task N; while the task
  is live, it watches the panel again at the task's next document (a
  change the forge has not registered yet); once the task has ended
  with a change result, it opens the panel once more and judges the
  condition on that last panel, its digest the last panel's. Any other
  end answers not met, exit 3, with the last panel and its digest, and
  a task with no panel ever delivered prints `no change panel for task N`,
  exit 2. A watch the engine let go as it stopped is no end
  (below): the wait goes on through jig's client's reopening, and its
  read of task N, like every read a wait makes, waits out a link down
  within the link's deadline (`bounds.md`, section 1) and fails only past it, as
  the link down's line (`results.md`, 3), never as a task it could not read.
- **A change's wait follows its task,** since the panel of a change
  that ended without landing keeps its last step (`../../domain/forge/change/panel.md`,
  1): the wait watches task N beside the panel, and when the task
  ends with no change result (successful no-change completion, cancelled,
  failed) it answers at once, not met, exit 3, naming how the task ended:
  `ended`, `ended: cancelled` or `ended: failed`, with the panel's last
  delivery and its digest. A successful report result establishes no-change
  completion (`../../domain/forge/change/README.md`, 2): both a read and a wait print
  `result: completed, no change` and `next: temper ctl DIR task show N`
  to reach its explanation directly, even when its parent chat's tree no
  longer lists N. JSON includes `"result":"completed, no change"`; the
  digest includes this completion detail. A panel whose step is ended
  also points to `task show N`. One with no panel ever delivered prints
  `no change panel for task N`, exit 2.
- **A task wait fills its text document before printing** through the same
  bounded specification reads as plain `task show` (`results.md`, 4). These supplemental
  words leave its completion, terminal result and digest unchanged.
- **`replied`** holds while a chat's status (jig's
  `client/domain/client/helpers.md`, its facts) is parked, waiting
  for you, waiting for them, held, closing or ended. A chat wakes on its
  person's words (jig's `domain/tasks/messages.md`, 3), so a wait begun once
  `chat start` or `task message` has answered is not met by the reply
  before them: `task message 41 TEXT --key k`, then `task show 41
  --until replied`.
- **`waiting-on-me`** covers task N's tree: what a task waits on is
  computed over its tree (jig's `domain/tasks/README.md`, section 3), so a
  question or a proposal below for the party meets it on `task show
  TASK`, and `task tree TASK` needs no condition of its own.
- **`phase:<phase>`** names the client's phase in lower case, hyphenated:
  `waiting`, `idle`, `due`, `preparing`, `claimed`, `running`,
  `backing-off`, `active`, `closing`, `held`. The word parses in any
  ASCII case (`Running`, `RUNNING`), and `backing-off` also as
  `BackingOff` and as `--json`'s `backing_off`; refusals and echoes print
  the lower-case word, and `help task show` lists the words. The ended
  phases, `ended`, `done`, `failed` and `cancelled`, are never conditions: the engine
  ends a task's watch as the task ends (jig's
  `client/domain/contract/watches.md`, 1.2), so a delivery may never show one.
  `--until phase:done` and its kin are usage errors, exit 1, naming
  `--until ended`.
- **`settled`** follows a sign-in or a check, or a forge's connect or
  install, to its end. A sign-in is in
  progress from its request's answer until it ends: made, renewed,
  refused, or past the sign-in's deadline (skein's `oauth/README.md`, section
  3); a check, until each host it asks has reported (jig's
  `domain/accounts-sources.md`, 1). So a wait begun once `account connect` or
  `account verify` has answered is not met before them: `account
  connect 1 --key chatgpt`, then `account list --until settled`, then
  the account's state in its answer. On `forge list`, a connect or an
  install is in progress from its request's answer until its run ends:
  connected, refused, lacking, or failed, or its link's state past its
  life (`setup.state_life_ms`, `../../protocol/configuration/README.md`); `forge
  connect 1 github --key gh`, then `forge list --until settled`, then
  the forge's row in its answer. A set up is in progress from its
  request's answer, setting up, until the forge's row leaves setting up
  and connecting: set up failed, connected, stopped, or not connected;
  `forge setup 3`, then `forge list --until settled`. `connected:FORGE` is
  met by a delivery where forge F is connected, whoever connected it.
- **`usable`** counts the party's own accounts, those whose uses name
  the party, and the deployment's (jig's `domain/accounts-uses.md`, 1.3), on
  any host: the box's harness waits on it once it has added the
  deployment's key (`README.md`, section 8).
- **`ended`** is met by a delivery that shows the subject ended, or by
  its watch's end because it ended. For `task show`, completion is checked
  by the task's number and terminal phase as below; its answer says
  how it ended.
- **A task wait whose condition a terminal task can hold watches first.**
  These conditions are `ended` and `replied`; terminal `phase:<phase>`
  conditions are usage errors as above. A watch on a subject
  that has ended is refused `Unknown` (jig's
  `client/domain/contract/reads.md`, section 1, and `client/domain/contract/watches.md`, section 1.2). On that opening
  refusal or `SubjectEnded`, the tool reads N through the existing
  authorized task read. On both paths, a `Historical` record or a
  live-form task establishes completion only when its returned number
  is N and its phase is terminal. `Unknown` alone never means success.
  A nonterminal or wrong-number result fails visibly; it cannot succeed or
  leave the wait hanging. A valid terminal uses the ordinary result
  renderer and meets the wait when its condition holds on that task;
  otherwise it answers not met, `ended`, exit 3, with the final document
  and its digest. Unknown, hidden or forgotten history keeps status 2;
  Busy, sign-out and read failures keep their ordinary statuses (`results.md`, 3).
  The read follows the watch, so the engine's retained-load durability
  barrier leaves no gap at live retirement. Conditions no terminal task
  holds (`held`, `waiting-on-me` and live phases) take no terminal read:
  `SubjectEnded` answers not met with the last document and its digest,
  and an opening `Unknown` keeps status 2. Run/tree waits retain their
  watch-based completion; there is no additional retry or timer policy.
  A tree completing from `SubjectEnded` prints `tree ended (watch)`
  and labels its retained rows `last observed snapshot` before the tree.
  Their chip phases remain as delivered, even when live: the watch
  establishes tree completion, not terminal chip facts or a task result.
  JSON retains the tree and its snapshot digest unchanged and adds
  `"completion":{"source":"watch","ended":true}` and
  `"snapshot":"last_observed"` outside it. The digest describes the
  last observed snapshot, never a reconstructed final tree. A terminal
  delivery uses the ordinary tree form. Without any delivery, a
  run/tree watch end refuses `gone`, status 2.
- **`--timeout`:** 300 seconds by default, at most 3600; answered with
  the current document, its digest, and that the condition is not met
  (`results.md`, 3).
- **A met text inbox wait** prints the plain read's inbox (`results.md`):
  the question's words, its `read whole` call when cut, its answer line
  and the read-on call, then `met`.
- **Wait timers** are bounded by `Deadlines::with_capacity(rows)` in
  `temper-client-tools-jig`'s `jig/wait.rs`, one per admitted call. Arming a
  new timer past that capacity refuses; admitted calls keep within it.
  The tools do not price the timer table.
- **A watch that ends under a wait** (jig's `client/domain/contract/watches.md`,
  1.2) answers it: met where the condition is `ended` and the subject
  ended. For `task show`, `ended` and `replied` first require the matching
  terminal read above and answer according to the condition on that task;
  refused where the party may no longer see the subject; failed
  where the sign-in ended; otherwise, not met, with the last document
  and its digest (`results.md`, 3). An engine that stops ends no watch: jig's
  client opens again a watch the engine let go as it stopped (jig's
  `client/domain/client/watches.md`, 1), so the wait goes on while the
  engine comes back, and fails as a link down past the link's deadline
  (`results.md`, 3, `bounds.md`, section 1).
- **Level-triggered, as the watch is** (jig's
  `client/domain/contract/watches.md`, 1.2): deliveries between are coalesced,
  and the answer is the document as it is when answered. A change before
  the wait began is in its first delivery, which `--since` compares.
- **The loop:** `me --until waiting`; `inbox`; act; again. An agent
  following one chat: `task show TASK --until waiting-on-me`, or `task
  show TASK --until ended`.
- **On a box,** the forge's webhooks make the engine's documents move
  soon after the forge does; with polling alone, the same waits answer
  later (`../../protocol/setup/webhooks.md`, section 1).

## 2. Open questions

- **A busy watch's precise cause** (`results.md`, 3): the deadline's line uses jig's
  `watches_full`, whose refusal carries no count and does not distinguish
  watch capacity from an opening's load slot. Distinct lines wait for jig
  to distinguish the causes (`jig/docs/design/client/domain/client/README.md`, 15).
  Until then the line names watches in use, as the `watch-room` request asks.

- **`--until phase:` in the words `task show` prints** (`tools-waiting.md`, 1, `results.md`, 1): the
  header prints jig's phrase-book word, and several are no phase word
  (`Waiting for news`, `Due to run`, `Preparing a host`, `Claimed by a
  host`, `Retrying`, `ChipExecutor active`); a wait takes the phase
  word, in any case, until the header and the condition share one
  spelling.
