# Goals and plans through ctl

This document holds goals and plans through ctl, split from [`areas.md`](areas.md).

## 1. Goals and plans
2. **Goals and plans:** `goal start --project PROJECT [--template TEMPLATE]
   [--charter CHARTER] [--priority PRIORITY] [--budget AMOUNT | --as-allowed]
   [--deadline TIME | --no-deadline] [--milestone TASK] [--after TASK]... TEXT`
   (`shapes::set_goal`, keyed; its last three flags area 8's), `proposal show PROPOSAL`, `task amend
   TASK [--rev REV] CHANGE... REASON`, `proposal withdraw PROPOSAL`,
   `seen`.

   `goal start`'s TEXT is the goal's words, and their first line its
   title, as Home and Work show it: `help goal start` says so, in
   TEXT's line ("The goal's words; the first line, at most 60
   characters, is its title") and in its notes, at most 60 characters
   naming the work for a person, with an example, the rest of TEXT on
   the lines below (`../../domain/agent/charter.md`, 4, *Words a person
   reads first*). A longer first line is a warning, never a refusal:
   where TEXT's first line, everything before its first line feed as
   jig takes the title, runs past 60 characters, counted as Unicode
   scalar values, `goal start` still sends the goal, and its answer,
   when it prints with status 0 (done or proposed), adds one line on
   stderr: `warning: TEXT's first line, the title the pages show, is N
   characters, past 60; temper ctl help goal start`, N the line's
   characters. Under `--json` the line is one JSON object, `{"warning":
   WORDS}`, WORDS the text line's after `warning: `, as the `kept:`
   lines are (`results.md`, 2). Its stdout and its exit status are the
   answer's; an answer of another status, and a TEXT whose first line
   is at most 60, print no warning. TEXT read from stdin (`-`) is
   checked once read; `--call` alike.

   `goal start` defaults to stock template 2; `--template` names an
   offered template, including its charter, contract, tracking, label,
   flags, LLM choice and symbolic resources. The chosen executor needs
   `--charter CHARTER`. Missing controls are collected before returning usage,
   exit 1, nothing sent: `template 2 needs: --charter CHARTER; --budget AMOUNT or --as-allowed; --deadline TIME; help goal start`.
   Only controls required by the selected template and absent from the call
   appear; fixed budget and deadline defaults are kept.
   A fixed executor takes its template's charter; also giving `--charter`
   fails as usage: `template 4's executor is charter 5; leave out --charter`
   (the numbers name that template and charter). A charter not offered
   in the project's creating bounds sends nothing, naming the template,
   the required charter and the offered charters. A budget or deadline
   past the offered ceiling has a separate diagnostic; a refusal beyond
   the direct bounds retains jig's reason. If the project has no creating
   offer, the diagnostic points to the role and the receiving bound
   `documents.lines` for charter and procedure lists. The fill and authority/proposal
   judgement are jig's (`jig/docs/design/client/domain/pages.md`, 6.2).
   Absent controls keep template defaults. A given budget requires
   `--budget` or `--as-allowed`; a given deadline requires `--deadline`,
   UTC to the minute as `2026-10-24T17:00Z`. Explicit budget or deadline
   controls replace the template's corresponding default before filling;
   `--priority` replaces a tracked template's priority. Model controls
   are area 5's; absent sub-agent flags preserve the template's list.

   **A budget is ctl's amount** (`results.md`, 1), as `task show`
   prints it: `goal start --budget 130` gives the goal 13,000 spend
   units, which `task show` prints `of 130.00`; `milestone new --budget
   130` makes a milestone that `task show` and `milestone show` both
   print `of 130.00`; `--budget 1.005` is usage, nothing sent. **`--as-allowed`
   takes all the party may give from its pool,** what the pool has
   left up to its role's spend (jig's `domain/authority.md`, section
   7), or, with `--milestone`, all the milestone has left
   (`milestones.md`, 3). Its reservation holds that while the work is
   live, so the party's next start finds none left. Its line in `help`
   says so, `goal start`'s: `--as-allowed: a switch. Take all you may
   give from your pool, or with --milestone all the milestone has
   left, instead of a named budget: none is left for the next start
   while this goal is live`; `milestone new`'s: `--as-allowed: a
   switch. Take all you may give from your pool instead of a named
   budget: none is left for your next start while this milestone is
   live`. A start then refused for its pool says why and names
   `--budget`, in the `reserved:` line of `requests.md`, section 1.

   The general `proposal show PROPOSAL` reads every proposal kind. For
   settings it prints the engine's current/origin/would-be review and its
   newest revision; retained ended and withdrawn records keep their state.
   An automatically replaced task proposal is unknown (7). It
   uses the same read vocabulary independently of the first area's
   settings proposal act. An amendment's, a task's or a party's, prints
   its target task and the authority before and after, as the engine's
   document gives them (jig's `client/domain/contract/documents.md`, 1).

   **`proposal withdraw PROPOSAL --project PROJECT`,** keyed, withdraws
   the party's own pending proposal: PROPOSAL is `pN/K`, as `propose`,
   `task amend` and the inbox print it, and PROJECT the project they
   name. It sends `Withdraw` at once, reading nothing first, so a
   proposal no read can show is still withdrawn. A `tN/K` is usage,
   exit 1, nothing sent: a task's proposal is its run's to withdraw; so
   is a `pN/K` naming a party other than the signed-in one. It prints
   `done: withdraw`, and as `proposal decide` does otherwise: asked again
   under its key, its first answer; a retained decision, its withdrawal
   included, `decided first:`, exit 0, changing nothing; an unknown
   proposal, or one of another project, exit 2. A proposal's `acts:`
   line copies it, the engine offering `Withdraw` to its maker alone.

   - **`task amend`'s changes,** one or more, each a control of the task,
     the rest kept as the task's document shows them (jig's
     `domain/tasks/changes.md`), drafted by the client domain's
     amendment editor (`../root.md`, 1.7):

     | Change | Sets |
     |---|---|
     | `--priority PRIORITY` \| `--untracked` | its tracking: tracked at PRIORITY, or untracked |
     | `--deadline TIME` \| `--no-deadline` | its authority's deadline, TIME as `2026-10-24T17:00Z` (UTC, to the minute), the rest of its authority restated as shown |
     | `--instructions FILE` | its instructions, whole: FILE's bytes, less a final newline; empty is none, said explicitly |
     | `--spec FILE` | its spec's words, whole; its parameters and inputs kept |
     | `--dependency TASK`... \| `--no-dependencies` | its dependencies, whole: those named, in order; what it names anew is added, what it leaves out removed |
     | `--model MODEL [--effort EFFORT]` | its main model and effort (area 5) |
     | `--budget AMOUNT` | its authority's spend, AMOUNT in ctl's amount (`results.md`, 1), the rest of its authority restated as shown |
     | `--tasks N` | the tasks its subtree may make over its life, the rest of its authority restated as shown |
     | `--to-party PARTY` \| `--to-role ROLE` | a person task's address: party PARTY, or the project's role ROLE, at the person revision the tool's read shows |

     FILE is a path, or `-` for stdin, used once, as TEXT is. The form's
     positions keep their numbers: `--to-party` and `--to-role` follow
     `--tasks`, and `--budget` is its last, after `--to-role`; the help's
     synopsis names `[--budget AMOUNT]` before `[--tasks N]`, as the
     table does. Its line in `help`: `--budget AMOUNT: an amount with at
     most two decimals, as 130 or 1.25; optional. Its budget, as task
     show prints it; the rest of its authority kept`.
   - **A budget or tasks raised is a widening,** funded by the party who
     gives it, who becomes the task's funder (jig's `domain/authority.md`,
     section 7, "Whoever widens a task funds it"): from their role's pool,
     within what the policy lets them give alone, or proposed past it, as
     the next bullets say. It is the way out a hold for a task's own
     budget or tasks names (jig's `domain/tasks/lifecycle.md`, 6; `../../domain/forge/change/repairs.md`
     ): `task amend 41 --rev 8 --budget 20 "room for its review"`,
     then `task release 41 --hold 3`. `--budget 20` restates the task's
     authority with spend 2,000 units, which `task show` then prints `of
     20.00`; JSON's `budget` names spend units, as `goal start`'s does;
     `--budget 1.005` is usage, nothing sent. A budget lower than the
     task's is a narrowing, sent as any narrowing is. For tasks, the hold prints
     `task amend 41 --rev 8 --tasks N REASON`, then the release. N is
     a replacement lifetime allowance with room for a repair and review;
     release never erases tasks already made. A hold whose lack the task's
     authority does not hold (a project's period, a pool) names `project
     fund` instead, and a hold no amendment lifts names the cancel.
   - **`--rev REV` is the spec's revision** that `task show TASK`
     prints, as the task's offered `amend` act carries it: `task show
     TASK`'s acts print it as `task amend 41 --rev 8 CHANGE... REASON
     --key NAME`. A change of a tracked task's priority alone names none
     (jig's `domain/tasks/changes.md`; `../../domain/agent/tools.md`);
     every other amendment does, and one without it exits 1, `temper
     ctl: task amend needs --rev REV`, nothing sent. The tool reads the
     task's document, drafts the change on it, and sends nothing when
     the document's revision is no longer REV: `temper ctl: task 41
     moved: its spec revision is 9, not 7; read again: task show 41`,
     exit 1.
   - **An address passes a person task up** to whoever can settle it,
     such as the project's owner: `task amend 41 --to-party 1 "over to
     the owner"`, or `--to-role ROLE` for a role of the task's project.
     It is jig's `Amendment.addressed`, an `AddressChange` (jig's
     `domain/tasks/changes.md`, *What an amendment may change*):
     - **Its revision is the person task's,** `Task::person_revision` of
       the document the tool reads, never `--rev`'s. A document with no
       person revision is no person task: the tool sends nothing and
       exits 1, `temper ctl: task 41 is not a person task: only a person
       task takes --to-party or --to-role`, before any other check of
       the call's changes.
     - **It needs no `--rev`.** jig compares the spec revision of every
       amendment but a priority alone, so the tool names the spec
       revision of the same read; `--rev`, given, is compared with it
       as for any change. A priority beside an address needs none
       either, the read's spec revision named for both; the call's
       other changes need it, as the bullet above says, refused as
       usage without it. `--rev REV`'s line says it is not needed for
       a priority or an address alone.
     - **It is a change,** even naming the address the task has now: jig
       then lets go of a party who took it. An amendment naming an
       address alone is sent, never `amend changes nothing`, and the
       amendment editor never compares an address away (`../root.md`,
       1.7).
     - **A person revision moved** since the read is jig's refusal,
       `Moved`, printed as any moved refusal is: `task amend: ` and its
       words, then `read again: task show 41`, exit 2 (`results.md`, 3).
     - **Both paths send it:** the drafted one, and the keyed one
       (`--key KEY`), which names its changes whole. A keyed retry made
       after the change took effect reads the moved person revision, so
       it is no longer the request its key holds, and jig answers it as
       that key's conflict, not as done; `task show TASK` says where the
       task waits now.
     - **Who may** is jig's admission of the task's `amend` act (jig's
       `domain/tasks/changes.md`, *Who may*): its requester and those
       above it in the tree, and a party whose role in the project
       allows it, the project's owners and maintainers among them. The
       party a person task waits on holds no `amend` act for that alone,
       and the tool says `amendment not offered`, as for any act it is
       not offered.
   - **What the document must hold whole:** a deadline, a budget or
     tasks restates the task's authority, a spec's words its spec. Where the document cut
     it to the client's bounds, the tool sends nothing and exits 1,
     `temper ctl: task 41's authority is cut in this read; amend its
     deadline, budget or tasks from a client with wider bounds` (or `spec`, `words`).
   - **Beyond the party's bounds,** an amendment follows `tools-acts.md`, 1, compared
     by jig's admission for the task's offered `amend` act: within,
     sent; past what the party holds but proposable, a deadline later
     than its funder allows among them, sent as `Propose`, and
     `proposed: t41/3, to …`; past the bounds, refused, nothing sent.
     Each change is proposable (jig's `domain/tasks/README.md`, 4 and `jig/docs/design/domain/tasks/changes.md`); what
     fails another check than authority (a dependency out of reach, a
     circle, a goal past the project's goals) is the engine's refusal,
     exit 2.
   - **The same flag twice, or both of a pair,** exits 1 as usage;
     `--dependency` alone may repeat. `--to-party` and `--to-role` are a
     pair, as `--priority` and `--untracked` are.

## 2. Open questions

None.
