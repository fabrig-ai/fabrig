# Shapes: what smith's worlds reached, in words

Provisional, 2026-11-28. The vocabulary in which smith's worlds, and the jig and temper worlds that run smith's fake model, count what their completions reached, `smith/3`: what a shape is, its words, how a sweep counts them, and how the vocabulary is versioned. A deployment's runs are read in the same words by the shapes reader (`tools/shapes/design.md`), so that the shapes a deployment hits and the worlds' samples missed can be named (`temper/docs/design/draft/misbehaving-agents.md`, 8). Skein's sweeps count and write them (`skein/docs/design/testing/sweeps.md`, 4.6). Written from the draft's 8, with the draft's 5.2 for jig's and temper's tools.

## 1. In one page

- **A shape is one completion's,** in allowlisted words: its calls in order, each with its tool, its answer's kind and each reference's class, and its run's end, begin reason and wake (section 2).
- **Its words are a closed list,** `unknown` for anything else (section 3).
- **Each sweep that plays a fake model counts,** per case, the shapes its cases reached, beside its sample's size, and writes them (section 4).
- **The vocabulary is versioned,** and the repository holds it in one file, never a deployment's data (section 5).

## 2. A shape

- **One completion:** the calls of one answer of the model, in the order it made them, with what came of each, in the run whose completion it is. A completion with no call is a shape too.
- **Each call:** its tool; its answer's kind; and, for each reference it carries, in the order path, offset, job, task, question, message, the reference and its class ([drawn-agent.md](drawn-agent.md), 5).
- **Its run:** the run's end class, its next begin reason, and the kind of message that woke it, in the words of jig's activity (section 3): the end of the attempt whose completion it is, the reason the task's next attempt began, and, where that reason is a message, the kind of that message. Every world writes `unknown` for each, since no world counts them yet (section 7); the shapes reader fills them from a deployment's activity (`tools/shapes/design.md`, 3).
- **Its text,** as a sweep writes it: the calls joined by `,`, each `TOOL:ANSWER` followed by `:REFERENCE=CLASS` for each reference, or `text` for a completion with no call; then ` end=END begin=BEGIN wake=WAKE`. So `job:unknown_job:job=consumed,write:not_run_after_failure:path=live,read:ok:path=live:offset=unissued end=unknown begin=unknown wake=unknown`; and a drawn `message` to a task never issued, refused for its reference, `message:no_reference:task=unissued end=unknown begin=unknown wake=unknown`.

## 3. The words

- **Tools:** `read`, `list`, `search`, `write`, `edit`, `remove`, `shell`, `job`; jig's and temper's reading and messaging tools, `read_task`, `message`, `subscribe`, `mark_read` (the draft's 5.2); `delegated` for any other tool its host serves; `unknown` for a name the session offers no tool for.
- **Answers,** by what came of the call (`domain/tools.md`, 3 to 5; `domain/session.md`, 5):
  - **done:** `ok`, a file tool's or a search's result; `search_timed_out`, hits cut by the search's deadline; for `shell` and `job`, `running`, `exited` (exit 0), `exit_nonzero`, `signalled`, `command_timed_out`, `stopped`, `cancelled`, `interrupted`;
  - **refused by the tools:** `not_granted`, `too_long`, `outside`, `read_only`, `protected`, `not_found`, `not_file`, `not_directory`, `linked`, `not_empty`, `nul_byte`, `too_large`, `not_read`, `stale`, `edit_no_match`, `edit_ambiguous`, `edit_unchanged`, `failed`, `timed_out`, `busy`, and `cancelled` for an operation cancelled before its end;
  - **invalid:** `unknown_tool`; `malformed`, a call that isn't an object, misses a field or holds one of the wrong type or value, or nests a sub-agent; `call_too_large`; `cut_off`; `unknown_job`; `jobs_full`;
  - **by the session:** `not_run_outside_choice`, `not_run_after_failure`, `not_run_stopped`, `withdrawn`, `delegated` for the answer of a delegated call whose tool is `delegated`, `unanswered` for a call its completion's end left with none.
  - **by jig's and temper's tools,** for a call whose tool is one of their four, by what the product answered; a world that sees only the answer's text classes it by the product's own texts and keeps the word alone:
    - **done:** `read`, a task read; `sent`, a message sent; `subscribed`; `marked`, words recorded read;
    - **refused:** `no_reference`, a task the caller holds no reference to, issued or not (a task read's `no reference`, a message's missing reference, a subscription's target it neither references nor may read); `recipient_ended`, a message to a task that has ended; `question_answered`, an answer to a question already answered; `not_unread`, a `mark_read` naming no unrecorded words; `refused`, any other refusal of these tools.
- **References:** `path`, `offset`, `job`; `task`, a task number (a task read's, a message's target, a subscription's); `question`, an answer's question; `message`, a message number `mark_read` names.
- **Classes:** `live`, `held`, `consumed`, `before_opening`, `foreign`, `unissued`, `stale`, `unknown` ([drawn-agent.md](drawn-agent.md), 5; `domain/session.md`, 10); and `ended`, a task that had ended when the call was made. For a number, `live` is one the run was shown whose task the world knows is live, `foreign` a question drawn from another task's labels by its foreign weight, `unissued` one never issued (drawn-agent.md's never-issued number), and `unknown` one the world can't tell.
- **Run fields,** as `temper ctl task activity --json` prints a task's rows (`jig/docs/design/domain/tasks/lifecycle.md`, 12.1, and `jig/docs/design/domain/tasks/messages.md`, 1 and 6, say what each means):
  - **ends,** the attempt's `run_ended` end, by its tag: `parked`, `finished`, `cancelled`, `refused`, `failed`, `suspended`;
  - **begins,** the next attempt's `run_began` why, by its tag: `made`, `ready`, `message`, `released`, `retry`, `narrowed`, `changed`; and `none` where the task ended with no attempt after;
  - **wakes,** the kind of the message a `message` begin names, by its tag: `question_owed`, `nothing_awaited`, `settings_decision`, `escalation`, `proposal`, `person_stalled`, `proposal_decision`, `question_answered`, `question_ended`, `words`, `effect_not_made`, `amendment`, `question`, `answer`, `notice`, `timer`, `news`, `result`, `met`; and `none` where the next begin is not a message;
  - and `unknown` in each, for anything else and wherever the field isn't known.

## 4. Counted per case

- **Each sweep that plays a fake model** in the session, agent, host and local worlds, and in jig's one-process world's drawn chat (`jig/docs/design/protocol/service-testing.md`, 1) and temper's composed workload (`temper/docs/design/testing/workloads.md`, 5), folds each case's shapes into a count of its own (`skein/docs/design/testing/sweeps.md`, 4.6), named for the sweep, in vocabulary `smith/3`: each shape counts the cases that reached it, once a case however often, beside the cases run.
- **Every completion of the case,** each session's and each sub-agent's, drawn or scripted.
- **Written** after the last case, as `skein/docs/design/testing/sweeps.md`, 4.6 writes them, in every mode.
- **Read by its reach checks too.** The cases in which each answer and each class fell come from the same case's shapes, so a sweep's reach check over answers and classes reads the count it writes ([drawn-agent.md](drawn-agent.md), 8).

## 5. Versioned

- **`smith/3`** is this document's list. A word added, removed or given another meaning, or a shape's text written otherwise, moves the version, and the shapes a sweep writes name the version they are in. `smith/2` was the list whose run fields held `unknown` alone; every world writes `smith/3` now, and no code keeps `smith/2`.
- **One list, in one file:** `smith/testing/smith-shapes/vocabulary.txt`. Its first line is the version; then one line per field, in the order `tool`, `answer`, `reference`, `class`, `end`, `begin`, `wake`: the field's name and its words, each separated by one space, in section 3's order for the run fields and in the order the kit declares them for the rest. It ends with a newline, and holds nothing else.
- **Its readers:** the `smith-shapes` test kit (`smith/testing/smith-shapes`) holds the words as its enumerations, which smith's, jig's and temper's worlds use, never a list of their own; a focused test holds the kit's version and its tool, answer, reference and class words equal to the file's, line by line. The activity's answer words (`jig/docs/design/client/domain/client/conversations.md`, 2) are tested to be among the file's answers. The shapes reader reads the file itself (`tools/shapes/design.md`, 2). No list is taken from a deployment.
- **Nothing raw:** a shape holds only these words, never a path, a value or an id the world drew or a deployment's run named.

## 6. Why

- **One completion, not one run.** A run's whole sequence of completions almost never repeats, so a count of run-long tuples would be ones, and nothing read against a deployment would match. A completion's calls, with what came of them, repeat, and the run's fields keep the context the draft asks for.
- **Answers by kind, not by text.** The rendered text names paths and numbers. A kind is what the product decided, and what a deployment's archived turn holds typed.
- **The counts' words, the checks' words.** A reach check that counted answers on its own would be a second tally of the same thing; reading the shapes keeps one.
- **Run fields in jig's activity words,** not smith's own words for a session's end: a deployment's activity names jig's end classes, begin reasons and wakes, and a smith word for them would be a second vocabulary for one thing. The tags alone, never an end's failure class or a suspension's why, so that each word is one the activity always prints.
- **A file, not only enumerations.** The shapes reader is a script that links no Rust, and the activity's answer words are written by jig's decoder, which links no test kit. One file all three are checked against keeps one list; the kit's enumerations stay, since its worlds name the words as types.

## 7. Later

- **The run's end, begin reason and wake, counted by the worlds** that run smith under jig's wakes, jig's one-process world's drawn chat and temper's composed workload, from their runs' activity: the draft's parks against wakes (`temper/docs/design/draft/misbehaving-agents.md`, 6). Until then a deployment's run fields are compared against a world's `unknown` (`tools/shapes/design.md`, 4).
