# Sessions: context management

## 1. Context management

A session keeps its conversation within its model's context window by
compaction into a new window. Nothing is edited in place: within a
window, history only grows (`session.md`, section 3).

### 1.1 Admission against the window

- **The window comes with the model.** The opener gives the model's
  usable input tokens and the most output tokens a completion may have,
  as the charter's model entry says (../protocol/charter.md).
- **What the session tracks:** the last prompt's size, from its usage,
  the sum of its input, cache reads and cache writes, which do not
  overlap (`session.md`, section 6), each counted as reported; that completion's
  output, as reported, or the model's most output when none was; the
  bytes the session counts in that prompt; and the bytes it counts in
  what was appended since, the results and messages after that output.
  The session counts one measure throughout: the payload bytes of each
  block's text, call, result and replay as it holds them, never the
  cells that hold them, recounted from the history it loaded after a
  resume.
- **The appended count** turns those appended bytes into tokens at the
  window's measured ratio: the last prompt's counted bytes over its
  reported size, at least one byte a token and at most `TOKEN_BYTES`
  (../protocol/limits.md, rule 4 of `session.md`, section 3). It is an estimate: content
  that the provider counts at more tokens a byte than the window so far
  passes it, and the line holds what it misses (below).
  - **Without a ratio:** with no earlier prompt in the window (a
    conversation's first completion, a window's first, the first after
    a resume from a window turn), or with usage that reports no input
    count, the bound is the whole prompt's bytes plus the allowance the
    session's limits give for what the protocol layer and the provider
    add (budget.md), a byte a token. In a new window's first
    check, the summary that opens it counts as the output tokens its
    compaction reported, or the model's most output when none were
    reported, never as its bytes.
- **The check,** before each completion: the last prompt, plus its
  output, plus the appended count, is at most the window times its
  threshold, a fraction of the window that the session's limits give
  (../protocol/limits.md). The arithmetic is checked. A completion that
  fails the check is not made: the session compacts instead (1.2).
  - **No output is reserved below the threshold.** The window bounds a
    prompt, and a completion's output is beside it (../protocol/limits.md,
    2.1). What that output adds to the next prompt is held above the
    threshold.
  - **So the threshold is the prompt's.** A session compacts when its
    next prompt would pass the threshold's share of the window, as
    counted, not when that prompt and a completion's most output would.
- **The line holds a compaction.** The line is the window less one
  completion's most output, the summary instruction and a cut result's
  marker for each of `calls_per_response` calls; the spare is what lies
  between the threshold and the line (../protocol/limits-derivation.md, 1.9). No
  ordinary completion is sent whose prompt may pass the line:
  - **the guard.** With each ordinary completion that has an earlier
    prompt in its window, the session gives the protocol layer the line,
    which its opener gave it at open from the charter's model entry
    (../protocol/limits-derivation.md, 1.9), the last prompt's reported size, and
    where that completion's answer starts in the history. Before it
    sends the request, the protocol layer measures that answer and
    everything after it as the request encodes them, their framing
    included, a byte a token (../protocol/llm.md, section 6). When the last
    prompt's size plus that measure passes the line, the request is not
    sent, and the completion fails as *context too long*, unsent
    (`session.md`, section 4): the window's one compaction (1.2);
  - **why it holds.** A token stands for at least one byte of the text
    it is read from, and encoding only adds bytes to that text, so the
    encoded measure bounds what the new part adds in tokens, whatever
    its ratio; its framing counts as encoded, as the prompt's allowance
    counts a whole prompt's (../protocol/limits-derivation.md, 1.2). A prompt the guard lets through
    is within the line. Its output is at most the model's most output,
    so the compaction after it, carrying both, finds above them the
    summary instruction and a marker for each owed result.
- **The headroom holds a compaction.** Two relationships are checked at
  startup (../protocol/limits.md):
  - the window above its threshold holds a completion's most output,
    the summary instruction and a cut result's marker for each of
    `calls_per_response` calls, so the line is at or above the
    threshold, and a compaction the check or the guard calls for always
    fits the window, its owed results cut to what is left;
  - a new window's opening at its largest (the system text, the task,
    the summary preamble, a summary of a completion's most output, and
    the messages held back, at most one offer of its opener's, which the
    opener bounds for this in messages and bytes, run-messages.md, section 1) and
    one completion's most output fit within the window at its threshold,
    so a window just opened makes its first completion and holds that
    completion's output below its threshold. Where a declared window
    would not hold that material, the derivation lowers the material's
    bounds, never the window (../protocol/limits.md).
- **The history's other bounds** (its messages, blocks and bytes) are
  derived so that the window check is reached first (../protocol/limits.md).
  A history that would cross one of them anyway is compacted as a full
  window would be. A session never ends because its history grew.
- **A completion's results never cross them.** The derivation holds one
  batch's results whole: the calls that run together (`session.md`, section 5), each
  at its tool's caps (../protocol/limits-derivation.md, 1.2). A later batch of the same
  completion runs within what is left of the history, less room for an
  answer to each call after it, shared among its calls: each result is
  what its tool answers with that share as its cap, saying what it left
  out (tools.md, section 4), and a text result, a host's answer or a
  sub-agent's, is cut with its marker. Every call is run and answered,
  and the compaction is owed each result as it was kept. A history so
  filled has no room for another completion, so the next is the
  compaction, whose request cuts the owed results to its own room (1.2).

- **Nor do they cross the line.** Every batch, the first
  included, also runs within an estimate of what is left below the line in
  the guard's units: the line, less the last prompt's reported size, less
  the retained byte count beyond that prompt's payload-byte baseline and
  everything already reserved, at `ENCODED_BYTES` (5/4: an allowance for
  escaping and framing as a request encodes them), less room for an answer
  to each call after the batch. The retained count includes the completion's
  answer, appended results and owned overhead. The smaller of this room and
  the history's is shared as above. So a completion of many large reads is
  answered with what fits, each result saying what it left out, and the next
  completion can be sent where the guard would otherwise refuse it unsent
  and the window compact without the results the reads were for. In a window
  with no earlier prompt, or where this room cannot give each call its
  minimum answer, the batch runs as before: the guard stays what makes the
  line hold, and this rule only makes its refusals rare.

### 1.2 Compaction

- **Its trigger:** a failed window check (1.1), a provider's *context
  too long* or the guard's (`session.md`, section 4; 1.1), or a session opened from a window whose last
  turn's model is not its charter's main model, as when a person chose
  another (`../shell/commands.md`, 1). That compaction runs with the
  window's own model, when the agent's configuration declares it, and
  the new window opens with the charter's, which its window turn records
  (../protocol/transcript.md, section 2), so a later resume does not
  compact again; one the configuration no
  longer declares makes the transcript one the run cannot resume
  (`session.md`, section 3).
- **A window carried over** (../protocol/transcript.md, section 4): a
  session opened from a window that ran on another endpoint or dialect
  than its charter's main LLM, on another account or backend, or on a
  program whose session its host does not hold, compacts first. That
  compaction runs on the charter's main LLM, since the window's own
  account may not be granted, with the window's history less its
  opaque blocks and providers' replay, which go back only to the dialect
  that wrote them. A message that history leaves with no block keeps one
  text block in its place, `[reasoning not carried over]`. That covers
  an assistant message that held only reasoning, and a program's whose
  reasoning the program did not tell (../protocol/programs-claude-code.md, 1.5). A
  provider may refuse a message with no content, as Anthropic's does,
  unsent, on every attempt. The marker's bytes and block count against
  the session's limits as any block's (`session.md`, section 3). A carried-over
  history they would take past those limits is refused as too large. A
  gone session's history they would take past them ends the session
  with the window's limit, as a window that cannot be compacted does.
  It is otherwise this section's compaction, a model change's included: the new window opens on the charter's LLM, and its
  window turn records its endpoint, dialect, account and backend. On a
  program backend it runs as a program session of its own, with no
  tools (../protocol/programs-claude-code.md, 1.8).
- **The compaction completion** has the same model, affinity, system
  text and tools as the completion it stands in for, and is asked with
  `ToolChoice::None`, whatever the conversation's own choice; the new
  window asks with the conversation's choice again. It holds the
  window's history with every result owed to it, then one appended
  user message: the summary instruction, which the run gives at open
  (run-start.md, section 1.3). The results owed to it are rendered within the
  room the window leaves for them, each cut with its marker where they
  would pass it (../protocol/limits-derivation.md, section 1.9). Its summary is its
  output, beside its prompt as every completion's is, and takes no room
  in the window (../protocol/limits.md, 2.1). The history before them is
  the ordinary prompt's, unchanged:
  - **on the OpenAI dialect,** which keeps the tool choice outside the
    cached prefix, the request reads the cached prefix;
  - **on the Anthropic dialect,** a changed choice keeps the cache of the
    tools and the system text and loses that of the messages, so a
    compaction there writes its window's history to the cache once more
    (../protocol/llm.md, section 2.2). The design accepts that cost, once
    per window, so that the model is asked for a summary and offered no
    call.

  Messages the opener passed since the last completion are held back for
  the new window. Calls in the compaction's answer, which a provider may
  write anyway, are not run.
- **The new window** opens with one user message of separate text
  blocks, in this order: the conversation's task (its opening message,
  carried from window to window); the summary preamble, which the run
  gives at open with the summary instruction (run-start.md, section 1.3); the
  summary (the answer's text); then the held-back messages, one block
  each. The compaction's own blocks are not kept, and the old window's
  history is released.
- **The summary** is the text of the answer's text blocks, in order,
  separated by a blank line. A summary cut at the output limit opens the
  window as written, with no marker, as an ordinary answer cut there
  stays in history as written;
  the compaction's `response.completed` has `stop` `max_tokens`
  (../protocol/events.md, 3.5).
- **Told as a window turn,** like any other turn: its usage and spend
  are the compaction's, and its messages are the new window's opening
  (`session.md`, section 3). It records the conversation's affinity, so a transcript
  can start there: transcripts and resumes carry only the current window
  (../protocol/transcript.md, section 3).
- **It counts as a turn and as spend.** It is reserved, charged and
  counted against the session's budgets like any completion (`session.md`, section 6).
  A budget that leaves no room for it ends the session for that budget.
- **Failure on repeat.** A window is compacted once. *Context too long*
  on the compaction itself, or a failed check or *context too long* in a
  window that has made no ordinary completion yet, fails the session
  with a limit naming the window. A compaction that fails otherwise
  fails as any completion does, after its retries (`session.md`, section 4).
  - **The window's limit keeps its cause.** When a provider's *context
    too long* caused it, the limit carries that failure's evidence and
    HTTP status, and the session's last failure detail stays the
    provider's text (`session.md`, section 4): the limit is the end it led to, not a
    later failure that replaces it. A check that fails before any request
    is unsent, and its detail is the limit and its bound.
- **No summary, no window.** An answer that is a refusal (its completion
  stopped for refusal, or it holds a refusal block), or whose text
  blocks hold nothing but ASCII whitespace, opens no window. The session
  fails at once, with the fault of the model's answer:
  - *refused*, for a refusal, whatever text came with it;
  - *truncated*, for no text and a stop at the output limit;
  - *malformed*, for no text otherwise.

  The compaction is neither retried nor nudged: a window is compacted
  once, and the headroom above the threshold holds one compaction, not
  an exchange after it (1.1). It is charged and counted against the
  budgets like any completion (`session.md`, section 6), and told as no turn, so the
  transcript still ends at the window's last told turn, and a session
  opened from it compacts first (`session.md`, section 3). The messages held back for
  the new window are not read. The run takes the failure as any
  session's: main's fails the run with that fault (run.md, section 9),
  and a child's returns unanswered (run.md, 5.3).

### 1.3 What else keeps context small

- **Results bounded at their source** stay the first defence: a read's
  window, a command's head and tail, search hits and list entries, each
  within the bounds the tools domain keeps (tools.md, section 4).
- **Children are the main isolation tool.** A child's reading never
  enters its parent; only its answer does (`session.md`, section 7).
- **No elision in place.** Old results are never shortened or removed
  within a window: that would edit history, lose the provider's cache
  from the edit onwards, and break providers' preserved reasoning. Only
  a window boundary drops history.
