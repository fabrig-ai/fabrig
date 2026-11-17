# Gaps: what QA can't judge yet

Provisional, 2026-11-16, from `../../draft/landed-qa.md`, 8, on the user's
word: "file the gap but don't act". A gap is a landing, or one of its
cases, that QA should judge and can't yet: temper ctl offers no way to
do or see what a person would, the fakes show no such behaviour, or QA's
own lines and setups can't reach the state. Triage's or a pass's agent
judges one; the tool keeps it, lists it and files it, and starts no goal.
A landing that says it closes a gap puts its case back in the queue.

## 1. In one page

- **Recorded** by the agent that meets it: triage, for a landing that
  needs QA no case can judge (section 2); a pass, for a case no line or
  setup reaches (section 3).
- **Its kind and owner:** `ctl`, `fake` or `qa`, each owned by an area
  from `qa.json` (section 4).
- **Kept and listed:** the ledger's gaps, open until a landing closes
  them; `temper-qa gaps` (section 5).
- **Filed, never acted on:** open gaps go out once, as a request with no
  goal, its document on main for people to read (section 6). Only a
  person starts work on one.
- **Closed by a landing** whose commits carry `QA-gap: N`: the gap's case
  queued again, or its landing triaged again (section 7).
- **Not a finding:** a finding is temper's fault on the green path, and
  a gap is what keeps QA from judging that path; `blocked` stays a pass's
  passing trouble, and a note what is off the path.

## 2. In triage

`temper-qa triage gap KIND WHY`, KIND `ctl`, `fake` or `qa`, WHY one
nonempty argument: what a person would do or see, and what is missing to
judge it. It answers the session's landing as `gap` (`ledger.md`, 4.2),
in place of cases or none, records the gap, and prints `gap N`. A
landing answered with cases may also take gaps, for the parts no case
can judge: each recorded, the landing's answer staying `cases`.

The brief, after its commands: "A landing a person would meet that no
case can judge yet, because ctl offers no way to do or see it, the fakes
show no such behaviour, or QA's lines and setups can't reach it, is a
gap: `temper-qa triage gap KIND WHY`, never none."

## 3. In a pass

`temper-qa pass gap CASE KIND WHY`: the case reaches no state its steps
need by any line or setup of this build. Its outcome `gap`, with the gap's
number; out of the queue at once, as no outcome of `blocked` takes it,
and counted neither failed nor blocked. The pass's report lists its gaps
beside its findings (`passes.md`, 8). A pass's brief names the command
where it names `blocked`: "A case no line or setup can bring to its
state is a gap, not blocked: `temper-qa pass gap CASE KIND WHY`."

## 4. Kinds and owners

| Kind | When | Owner |
|---|---|---|
| `ctl` | a person's act or what they would see has no ctl tool, or ctl's words don't say how | `areas.ctl`, else `owner` |
| `fake` | the green path needs a behaviour no fake shows (a CI run queued for a runner, a forge's own account) | `gaps.fake`, else `owner` |
| `qa` | QA's lines, setups or tool can't reach the state (a branch the person pushed) | `owner` |

`qa.json`'s `gaps`, optional: `{"fake": "AREA"}` names the fakes' area;
absent, the owner takes `fake` gaps too.

## 5. Kept and listed

The ledger's `gaps` (`ledger.md`, 2.11), each: `number`; `kind`; `why`;
`owner`; `landing`; `case`, a pass's gap's; `session` and `time` of its
record; `state`, `open`, `filed` or `closed`; `filed`, the time its
request was filed; `closed`, the landing that closed it and its time.

`temper-qa gaps` prints each gap one line, newest first:
`N KIND OWNER STATE landing L[ case C]: WHY`, cut at 200 bytes, with
`--open` for those not closed. `temper-qa show` counts them.

## 6. Filed

The loop's fourth step (`README.md`, 4), with the requests
(`findings.md`, 5): open gaps not yet filed, every one at once, as one
request with no `goal`, whatever the findings' requests did. A findings
request's failure never stops the gaps' request from being tried:

- **Its files:** `temper/docs/development/qa/gaps/gap-<N>.md` for each,
  its kind, owner, landing (number, goal, title, commit), case and steps
  where a pass's, the session's agent's why, and the line closing it
  takes, `QA-gap: N`; its message `QA gaps N, M...: K open`, then a
  blank line and a `QA-gaps: N, M...` trailer.
- **`request.command`** as for any request with no `goal` (a recipes'
  alone, `findings.md`, 5): it puts the files on main and starts nothing.
  Its success marks each gap `filed`; a failure is tried again at the
  next tick, as a request's is, and included with the step's other
  failures in its result and its line after an hour (`README.md`, 4).
  It does not stop the waiting goals' state checks.
- **No goal, ever, from a gap:** the user, 2026-11-16. A person reads the
  gap and starts what they choose, whose landing then closes it.

## 7. Closed

- **By a landing:** the watcher (`landings.md`, 4) reads each landing's
  commits' `QA-gap: N` trailers, as it reads `QA:` ones
  (`triage.md`, 3): each names gap N, which is `closed` with that
  landing, on a kept build holding it. A pass's gap's case goes back to
  the queue, `queued` with its outcome cleared, so the next pass on a
  build holding the landing takes it; a triage gap's landing is triaged
  again on such a build, its earlier answer kept beside the new one.
- **Never by hand alone:** a gap stays open until a landing says so; one
  the person decides to leave is closed by a landing removing its file
  with the trailer.

## 8. Its code and tests

In `temper-qa-tool`: the two commands, the ledger's gaps and their rules
(`ledger.md`, 2.11 and 4.18), the gaps' request in the loop's fourth
step, the trailer in the watcher, `temper-qa gaps`, both briefs' words.
Focused tests in its existing idiom: each command's record and
refusals; a pass's gap out of the queue and in its report; the request
written once with every open gap, no `goal`, the gaps `filed` on its
success and retried on its failure; a `QA-gap` trailer closing a gap and
queuing its case again, or its landing's triage; `gaps` and `show`.
