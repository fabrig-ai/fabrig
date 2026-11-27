# Shapes: what smith's worlds reached, in words

Provisional, 2026-11-27. The vocabulary in which smith's worlds count what their completions reached, `smith/1`: what a shape is, its words, how a sweep counts them, and how the vocabulary is versioned. A deployment's runs are to be read in the same words, so that the shapes a deployment hits and the worlds' samples missed can be named (`temper/docs/design/draft/misbehaving-agents.md`, 8). Skein's sweeps count and write them (`skein/docs/design/testing/sweeps.md`, 4.6). Written from the draft's 8, its world side.

## 1. In one page

- **A shape is one completion's,** in allowlisted words: its calls in order, each with its tool, its answer's kind and each reference's class, and its run's end, begin reason and wake (section 2).
- **Its words are a closed list,** `unknown` for anything else (section 3).
- **Each sweep that plays a fake model counts,** per case, the shapes its cases reached, beside its sample's size, and writes them (section 4).
- **The vocabulary is versioned,** and the repository holds it, never a deployment's data (section 5).

## 2. A shape

- **One completion:** the calls of one answer of the model, in the order it made them, with what came of each, in the run whose completion it is. A completion with no call is a shape too.
- **Each call:** its tool; its answer's kind; and, for each reference it carries, in the order path, offset, job, the reference and its class ([drawn-agent.md](drawn-agent.md), 5).
- **Its run:** the run's end class, its next begin reason, and the kind of message that woke it. In `smith/1` each is `unknown` in every smith world: they are the words of jig's activity, which no smith world runs (section 7).
- **Its text,** as a sweep writes it: the calls joined by `,`, each `TOOL:ANSWER` followed by `:REFERENCE=CLASS` for each reference, or `text` for a completion with no call; then ` end=END begin=BEGIN wake=WAKE`. So `job:unknown_job:job=consumed,write:not_run_after_failure:path=live,read:ok:path=live:offset=unissued end=unknown begin=unknown wake=unknown`.

## 3. The words

- **Tools:** `read`, `list`, `search`, `write`, `edit`, `remove`, `shell`, `job`; `delegated` for a tool its host serves; `unknown` for a name the session offers no tool for.
- **Answers,** by what came of the call (`domain/tools.md`, 3 to 5; `domain/session.md`, 5):
  - **done:** `ok`, a file tool's or a search's result; `search_timed_out`, hits cut by the search's deadline; for `shell` and `job`, `running`, `exited` (exit 0), `exit_nonzero`, `signalled`, `command_timed_out`, `stopped`, `cancelled`, `interrupted`;
  - **refused by the tools:** `not_granted`, `too_long`, `outside`, `read_only`, `protected`, `not_found`, `not_file`, `not_directory`, `linked`, `not_empty`, `nul_byte`, `too_large`, `not_read`, `stale`, `edit_no_match`, `edit_ambiguous`, `edit_unchanged`, `failed`, `timed_out`, `busy`, and `cancelled` for an operation cancelled before its end;
  - **invalid:** `unknown_tool`; `malformed`, a call that isn't an object, misses a field or holds one of the wrong type or value, or nests a sub-agent; `call_too_large`; `cut_off`; `unknown_job`; `jobs_full`;
  - **by the session:** `not_run_outside_choice`, `not_run_after_failure`, `not_run_stopped`, `withdrawn`, `delegated` for a delegated call's answer, `unanswered` for a call its completion's end left with none.
- **References:** `path`, `offset`, `job`.
- **Classes:** `live`, `held`, `consumed`, `before_opening`, `foreign`, `unissued`, `stale`, `unknown` ([drawn-agent.md](drawn-agent.md), 5; `domain/session.md`, 10).
- **Run fields:** `unknown` alone, in `smith/1`.

## 4. Counted per case

- **Each sweep that plays a fake model** in the session, agent, host and local worlds folds each case's shapes into a count of its own (`skein/docs/design/testing/sweeps.md`, 4.6), named for the sweep, in vocabulary `smith/1`: each shape counts the cases that reached it, once a case however often, beside the cases run.
- **Every completion of the case,** each session's and each sub-agent's, drawn or scripted.
- **Written** after the last case, as `skein/docs/design/testing/sweeps.md`, 4.6 writes them, in every mode.
- **Read by its reach checks too.** The cases in which each answer and each class fell come from the same case's shapes, so a sweep's reach check over answers and classes reads the count it writes ([drawn-agent.md](drawn-agent.md), 8).

## 5. Versioned

- **`smith/1`** is this document's list. A word added, removed or given another meaning, or a shape's text written otherwise, moves the version, and the shapes a sweep writes name the version they are in.
- **The repository holds it:** the words are the code's, its enumerations written from section 3, and no list is taken from a deployment.
- **Nothing raw:** a shape holds only these words, never a path, a value or an id the world drew.

## 6. Why

- **One completion, not one run.** A run's whole sequence of completions almost never repeats, so a count of run-long tuples would be ones, and nothing read against a deployment would match. A completion's calls, with what came of them, repeat, and the run's fields keep the context the draft asks for.
- **Answers by kind, not by text.** The rendered text names paths and numbers. A kind is what the product decided, and what a deployment's archived turn holds typed.
- **The counts' words, the checks' words.** A reach check that counted answers on its own would be a second tally of the same thing; reading the shapes keeps one.
- **Run fields `unknown` for now,** not smith's own words for a session's end: a deployment's activity names jig's end classes, begin reasons and wakes, and a smith word for them would be a second vocabulary for one thing.

## 7. Later

- **The run's end, begin reason and wake,** in the words of jig's activity, where a world runs smith under jig's wakes: the draft's parks against wakes (`temper/docs/design/draft/misbehaving-agents.md`, 6), with jig's and temper's words for their own tools (the draft's 5.2), in the version after this one.
- **The reader** of a deployment's activity in the same words (the draft's 8).
