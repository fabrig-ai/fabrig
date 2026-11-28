# The tools' areas

The table grows area by area, each with its stories (`testing.md`).

## 1. A chat to a landed change
1. **A chat to a landed change** (the first milestone's story,
   `../../README.md`, section 2): the host's tools of `host.md`; every read
   of `tools-reads.md`, 1 other than the general proposal read of area 2; the watched
   reads' waits (`tools-waiting.md`, 1); every act of 5.3.
## 3. Several parties
3. **Several parties:** `party make --project PROJECT --role ROLE --out
   FILE` (jig's `MakeService`, keyed; `README.md`, section 7), `sign-in --service
   FILE`, `party reissue PARTY --out FILE` (jig's `ReissueService`, keyed;
   `README.md`, section 7), `deployment show`, `deployment census` (`tools-reads.md`, 1), `task move TASK REASON`, a person task's
   `task take`, `task hand-back` and `task answer TASK --rev REV`, and
   the project's own operations, `policy`, `role`, `pool` (jig's
   `domain/people/acts.md`, 2), past the spend per period that `project
   fund` sets (`tools-acts.md`, 1).

   `task show TASK` joins a person task's document to its generic task:
   its question, result contract, choice codes and bounds, person revision,
   holder, and each offered act with its admission. Its holder prints as
   `taken by: PARTY`, or `taken by: none`. Once an answer has ended the
   task, `answered by: PARTY` takes that line's place, from the retained
   result's `chosen_by`. A closing person task with no holder prints
   neither: its live document does not say who answered. `--json` keeps
   `taken_by` as jig gives it. The inbox's person-task
   line prints `read: task show TASK`. Role-addressed work offers
   `task take TASK` and `task hand-back TASK`; it must be taken before an
   answer. These commands and person answers are optionally keyed.

   A person answer keeps the shown revision: `task answer TASK --rev REV
   CHOICE TEXT` for a choice and its reason, `task answer TASK --rev REV
   TEXT` for words, or `task answer TASK --rev REV --approve STATE [TEXT]`
   for the exact approval state, STATE as lowercase hexadecimal. Every
   contract also accepts `task answer TASK --rev REV --cannot REASON`.
   A stale revision reaches the engine and is refused; a retained keyed
   answer reaches it for replay even after the task ends. The numbered
   question form of `tools-acts.md`, 1 keeps its separate question read and answer.

   An answer its contract does not take reaches the engine and is
   refused as past its contract (exit 2): the client checks nothing
   first, since a keyed answer must reach the engine for its replay.
   The act's read gave the contract, so after jig's words the refusal
   says what the task takes:

   - **Past its bound,** a report's words or the reason of the choice it
     names: `task answer: Limit reached: task 44's reason for choice 1
     is at most 128 bytes`, or `task 44's words are at most 128 bytes`;
     JSON's `bound`.
   - **A choice for a task that takes words:** `task answer: Limit
     reached: task 44 takes words, at most 4096 bytes, not a choice`,
     then the line `answer instead: task answer 44 --rev 3 TEXT`, REV
     the call's own; JSON's `takes` (`"words"`), `bound` and
     `answer_instead`, that call as a string.
   - **A choice for a task that takes an approval:** `task answer: Limit
     reached: task 44 takes an approval, not a choice`, then `answer
     instead: task answer 44 --rev 3 --approve STATE [TEXT]`; JSON's
     `takes` (`"approval"`) and `answer_instead`.
   - **A choice the contract lacks,** its choices read whole (not cut)
     and at least one: `task answer: Limit reached: task 44 has no
     choice 2; its choices are 1, 3`, the codes in the read's order,
     comma-separated; JSON's `choices`, an array of those codes.

   The `answer instead:` call names no key: the refused answer's key
   replays its refusal. JSON's keys sit beside the refusal, as
   `read_again` does. The same refusal for any other cause has nothing
   the task asks for, and prints as it is: an answer within its bounds,
   words or an approval for a task with choices, a choice its read's cut
   choices lack, a choice for a change's or a job's contract, a
   `--cannot` or an `--approve` past the deployment's result bound; so
   does a restored request, which read nothing. TEXT's bound in the
   tool's line is the most any call takes: the tool's notes say a person
   task may ask for less, and that CHOICE answers only a task with
   choices, a task that takes words being answered with TEXT alone.

## 4. jig’s plumbing whole
4. **jig's plumbing whole:** `create`, `propose`, `compound --project
   PROJECT FILE`, a batch, a proposal or a compound request as JSON
   (keyed).
## 7. Landed drafts
7. **Landed drafts** (`../../domain/forge/drafts.md`): a writer saying
   that a draft landed in one of a project's repositories is ready to
   start a goal, or no longer is, and the project's landed drafts listed,
   as Work draws the ready ones (`../pages/work.md`, 1). The group
   `draft`. Each act is the FORGE request `SetDraft`
   (`../forge/records.md`), made through `jig-client` as `repository
   adopt`'s is; OWNER and NAME name a repository adopted into PROJECT,
   both or neither, the project's home with neither.

   | Tool | What it does | temper's | K |
   |---|---|---|---|
   | `draft ready --project PROJECT [--owner OWNER --name NAME] [--chat TASK] PATH` | says the draft at PATH is ready to start a goal: temper reads it at the head of the repository's landing branch and keeps it, with TASK the chat it was written in; prints `ready: OWNER/NAME PATH at COMMIT on BRANCH`, COMMIT its first 14 digits, then its title where it has one, exit 0 | `SetDraft { change: ready { chat } }` | yes |
   | `draft not-ready --project PROJECT [--owner OWNER --name NAME] PATH` | says it no longer is; Work stops listing it; prints `not ready: OWNER/NAME PATH`, exit 0 | `SetDraft { change: not_ready }` | yes |
   | `draft list --project PROJECT` | the project's landed drafts, a watched read of the subject `drafts`: a line each, newest first, `OWNER/NAME PATH`, then `ready`, `not ready` or `goal G`, COMMIT's first 14 digits, BRANCH, when it was said ready in the clock form, `chat T` where it names one, and its title; "No landed drafts." with none, exit 0 | the subject `drafts` | |

   - **The refusals,** exit 2, each saying what to do: `no repository`,
     "no repository OWNER/NAME in PROJECT", or "PROJECT has no home
     repository: name one with --owner and --name"; `path`, "PATH is no
     draft's path: a relative path ending .md"; `missing`, "no PATH at
     the head of the landing branch: land it first"; `not a file`, "PATH
     is a directory"; `unreachable`, "the forge did not answer: try
     again"; `unknown`, "no landed draft at PATH"; `full`, "PROJECT keeps
     64 landed drafts, every one listed: say one not ready first". A
     refusal at admission is jig's, as for `repository adopt`.
   - **`--json`** prints the outcome's row, or the panel's rows, as the
     wire has them, a commit whole.

## 8. Milestones
8. **Milestones** (`../../domain/milestones.md`): the group
   `milestone`, `milestone new`, `whole`, `list` and `show`, and `goal
   start`'s `--milestone`, `--after` and `--no-deadline`, designed in
   full in `milestones.md`.

## 9. Growing the table
- **Areas 1, 5, 6 and 7 are designed here in full,** and area 8 in
  `milestones.md`. Areas 2 to 4 name their
  tools and jig's operations; each area's arguments are checked against
  jig's operations as they stand when the area is built.
- **A tool no area names** is a design question, and joins the table as
  a row of an area.

## 10. Open questions

None.
