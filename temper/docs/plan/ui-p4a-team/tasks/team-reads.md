+++
id = "team-reads"
part = "model"
kind = "feature"
needs = ["team-surface"]
files = [
  "temper/crates/temper-client-domain/src/pages/team.rs",
  "temper/crates/temper-client-domain/src/pages.rs",
  "temper/crates/temper-client-domain/src/tests/pages/team.rs",
  "temper/docs/design/client/README.status.md",
]
effort = "high"
minutes = 160
+++

# Team's reads: the project's people and what waits on each

## Goal

Give `pages::team::shown` and `pages::team::people` their bodies: while
`Team { project }` is open, the domain reads the project's document,
counts what waits on the person reading from Needs you's standing inbox
and their approvals, and watches, for each other person, the live tasks
they requested, counting what waits on them. `team-surface` made the
model; `team-page` draws from it.

Acceptance, by `temper/docs/design/client/README.md`, 3.5, *Team*, *A
person's line* and *Its reads*, and `notes/contracts.md`, T3:

1. **The state:** `pub struct Team` in `pages/team.rs`, held by `Pages`
   beside `roles` (`pages.rs`: its field, its `Default`, its configure
   while the page is open and the link neither signed out nor out of
   date, its `told` in `page_told`, its leave on every path that leaves
   `roles`: a move away, `Left`, `Started`, a party's change, signed out,
   out of date). Model: `pages/roles.rs`.
2. **The project's document:** on `Team { project }`, one
   `Event::Read { handle, read: Readable::Jig(c::Read::Project { project
   }) }` on opening; another on each `me` delivery (`Told::Changed` for
   `pages::me_handle()`) while none is pending; its answer kept,
   `ReadValue::Project`, or the page unreadable on `ReadResult::None`, a
   value of another kind, or `Told::Ended`; the handle dropped and the
   outcome forgotten when the address leaves Team or its project.
3. **The reader's waits** (T3, *yours*): from the standing inbox watch's
   document (`needs::handle()`): each `needs` row that `shows_in(project)`
   and isn't answered here, counted by `group()`: `Questions` to
   `questions`, `Proposals` to `proposals`, `Escalations` to `held`,
   `PersonTasks` to `person_tasks`; each of the person's approvals' rows
   with `row.project == project` to `reviews`; `cut` as
   `needs::Session::count` sets it. `None` while that watch has delivered
   nothing or is ended. "Answered here" is `Session`'s own rule: add one
   `pub(crate) fn answered(&self, object: ObjectKey) -> bool` to
   `needs::Session`, beside `later`, returning `self.line(object).is_some()`.
4. **Others' waits** (T3, *Others' waits*): once the document is read, one
   `Event::Watch { handle, subject: Watchable::Jig(c::Subject::Tasks {
   query, window: limits.client.domain.rows }) }` for each person member
   but the reader, in `gather`'s order, at most `OTHERS`; `query`:
   `project: Some(project)`, `requester: Some(c::Requester::Person(party))`,
   the other keys `None`, the order Chats' query uses. A delivered page:
   each live line with `waits_on` `c::WaitsOn::Party { whom:
   c::Whom::Person(party), kind, .. }` counts one by `kind` (`Question`,
   `Proposal`, `Escalation` to held, `PersonTask`); `cut` while the page
   says more follow. A watch that hasn't delivered gives no pair; an ended
   one gives none either. Each closed (`Unwatch`) on leaving; when the
   document's people change, the watches of people gone close and new
   people's open, at most `OTHERS` in all.
5. **`shown` and `people`** (T2's signatures): `shown(root, project)` is
   `Shown::Read(gather(document, you, yours, &others))` once the document
   is read for `project`, `you` the `me` party's number (`Pages::me`);
   `Shown::Unreadable` after a refusal or a failed read; else
   `Shown::Reading`. `people(root, project)` is the number of `gather`'s
   people while the document is read for `project`, else `None`.
6. **Rows** in `tests/pages/team.rs` (no new test file), one table test
   `team_reads_the_project_and_what_waits_on_each`, a domain stepped to
   `Team { project: 3 }` and fed deliveries, as `tests/pages/project_pages.rs`
   feeds Role's read. Rows: opening asks `Read::Project { project: 3 }`
   once; a `me` delivery while pending asks nothing, after the answer asks
   again; an answer gives `Shown::Read` with its people and `people` its
   count; a `ReadNone` gives `Unreadable`; two other members open two
   `Tasks` watches with their requesters, nine open `OTHERS`; a page with
   a line waiting on that person counts it, one waiting on a role or on
   another person doesn't; the reader's question and approval rows of
   project 3 count, project 4's don't, an answered one doesn't; moving to
   `Home { project: 3 }` unwatches every Team handle and `shown` is
   `Reading`.
7. **Status:** delete, whole, the **Not yet** entry "3.5, *Team*, *Its
   reads*: Team reads nothing; `team::facts` and `team::people` answer
   `None`" in `temper/docs/design/client/README.status.md`. No other
   design line.

## Inputs

- `temper/docs/plan/ui-p4a-team/notes/common.md`;
  `temper/docs/plan/ui-p4a-team/notes/contracts.md`, T2 and T3.
- `temper/docs/design/client/README.md`, 3.5, *Team* (*A person's line*,
  *Its reads*); 3.5, *What the client keeps* (Needs you's count).
- jig's `jig/docs/design/client/domain/contract.md`, 6.1 (task queries),
  7.1 (subjects; membership moves `me`); `jig/docs/design/domain/people.md`,
  section 6.
- The code: `temper-client-domain/src/pages/roles.rs` (a page's read, its
  handle, `told`, `leave`); `pages/chats.rs` (a `Tasks` watch and its
  order); `pages/needs.rs`, `Session::count` (the rows and the cut);
  `forge/approvals.rs` (`Row`, `approvals()`); `pages.rs`, `page_told`,
  `went`, and the `roles` lifecycle's sites; jig's `contract/queries.rs`
  (`TaskQuery`, `WaitsOn`, `Whom`, `WaitedFor`) and `contract/project.rs`.

## Owns and may change

Owns the files in the header. Reader and neighbour files, and what may
change there: `temper-client-domain/src/pages/needs.rs`, the one
`pub(crate) fn answered` of acceptance 3 and nothing else (ui-p2-home's
file). No view file: `team-page` draws.

Options, each decided:

- **Chats' order isn't a fit for a query naming a requester:** take
  `QueryOrder` by last activity, latest first, and say so in the report.
- **The approvals' document isn't reachable from `Pages`:** compute
  `yours` inside `shown(root, ..)`, which has the whole `Domain`, and keep
  the state to the document and the watches.
- **A `me` delivery comes many times while a read is pending:** one read
  at a time; the next delivery after its answer asks again.
- **Others' order:** `gather`'s, the reader left out, the first `OTHERS`.

## Steps

1. Read the Inputs. The state and the document's read (1, 2) and their
   rows; bite: ask the read again on every `me` delivery, pending or not,
   see the pending row fail, restore.
2. The reader's waits (3) and their rows; bite: count project 4's rows,
   see that row fail, restore.
3. Others' watches and counts (4) and their rows; bite: count a line
   waiting on a role, see that row fail, restore.
4. `shown` and `people` (5); the status entry (7).
5. Runs through `heavy`, counts in the report: `cargo nextest run -p
   temper-client-domain`, `-p temper-client-view`, `-p
   temper-client-shell-terminal`, `-p temper-client-gallery`, `-p
   temper-client-world` (Team's new reads must leave every story as it
   was: Team opens in none of them yet); `cargo check -p temper-client-tui
   --tests`; clippy on every package changed; then as
   `notes/common.md` says before the push. Commit, saying that Team now
   reads its people and what waits on each, and push.

## Report

The implementer's report: the state and its lifecycle's sites in
`pages.rs`, each read and watch as written, the counting rules, the rows
and the bites with their failures, the runs' counts, the new test's CPU
time, and what is left.

## Rebase (2026-11-08, step merge, foreman ui-p4a-team/team-reads/foreman-1)

resolve the status-file conflict by preserving both adjacent entry deletions
The merge conflicts only in `temper/docs/design/client/README.status.md`.
Integration commit `808d70ef78` (“Draw Team's people and what waits on each”), merged by `53905ace8e`, removes the two-line “Team: the view draws no Team page” entry.
Task commit `8a173e0d3c` removes the adjacent three-line “Team, Its reads: Team reads nothing” entry.
Rebase onto the landing's current integration tip and resolve this hunk with both entries absent.
Preserve the following “In the rail” and “The project's triager” entries and all other status content.
Keep the task's Rust changes intact; this refusal requires only the status-file conflict resolution.
