# Findings and their goal

Provisional, 2026-11-12, from `../../draft/landed-qa.md`, sections 7, 8
and 9, milestone Q3 of `../../draft/landed-qa.execution.md`. What a pass
finds against temper, and how it goes back: each finding with its setup
and the lines that reproduce it, linked to an open one it matches, and a
pass's open findings filed as one request, put on main by the
deployment's command, with one goal started on it.

## 1. In one page

- **A finding** (section 2) is recorded by the pass's agent as it
  finds it: its case, title, severity, area, setup, lines, what was
  expected and seen, and its evidence, kept beside the ledger.
- **Never filed twice** (section 3): one that matches an open finding,
  by setup and lines or by page and expectation, is linked to it.
- **One request per pass session that leaves findings open**
  (section 4): a document listing them, with the pass's new and edited
  recipes, in `temper/docs/development/qa/requests/`.
- **Filed by the deployment's command** (section 5), which puts the
  request on main and starts its goal under the configured area; the
  goal's number is recorded.
- **Its goal's end** (section 6): its landing, seen by the watcher, or
  an end without landing the deployment reports; either resumes the
  pass (`passes.md`, 7). Two goals at most, then handed on.

## 2. A finding

`temper-qa pass finding FILE`, by a pass session's agent, FILE a path or
`-`, one JSON object:

```json
{
  "case": 7,
  "title": "Start on a ready draft starts no goal",
  "severity": "medium",
  "area": "AREA",
  "setup": "project-landing",
  "lines": ["web reload \"/projects/1\"", "web press button \"Start\""],
  "expected": "a goal starts on the draft at the project's way",
  "seen": "the button is pressed and the page lists no goal",
  "evidence": ["/PATH/places/1/box/session/looks/0004-1280.png"]
}
```

- **`case`,** a case assigned to the session's pass; **`title`,
  `expected`, `seen`,** nonempty; **`severity`,** `low`, `medium` or
  `high`; **`setup`,** `fresh` or a recipe of the pass's setups
  (`passes.md`, 2), the state the lines start from; **`lines`,** at
  least one, each a nonempty drive line on one line
  (`../../client/drive.md`, 3), the driver's kept lines from that setup
  to the fault, without looks (`places.md`, 5).
- **`area`,** optional: the area the finding is for; absent, the case's
  surface's in `qa.json`'s `areas`, else `owner` (`README.md`, 3.2).
- **`evidence`,** optional, at most 16 paths, each a regular file of at
  most 16 MiB, relative to the command's working directory: a look's
  screenshot, the screen's text or the box's log where it matters
  (`../../draft/landed-qa.md`, 7). Each is copied into `evidence/N/` in
  the home under its file name, N the finding's number, before the
  ledger records it; two of one name are refused.
- **Recorded** as `ledger.md`, 4.11 has it, the case's surface, page and
  landings with it, and answered `finding N recorded` or `finding N
  linked to open finding M of pass P`; refused as its section 5 words
  it.
- **A recipe's step that broke** with no landing meaning it to is a
  finding like any other, its setup the recipe's parent or the nearest
  that restores, its lines the walk's to the step, its seen what the
  session answered, and its expected that the step walks as it did on
  the build its copy was made at (`places.md`, 7).

## 3. Matching

- **The same fault twice** is one finding: a finding with the same
  setup and the same lines as an open one, or on the same surface and
  page with the same expected, is recorded `linked` to it
  (`ledger.md`, 4.11), never carried by a request itself.
- **A failed case whose findings are all linked** to another pass's
  open findings moves to that pass at its session's end, to run at that
  pass's resume (`ledger.md`, 4.12), so one goal answers both.
- **A waiting pass keeps its page:** a case triage queues later for a
  page and setup that one of its open findings is on joins it
  (`ledger.md`, 4.15).

## 4. The request

Made at a pass session's end with the pass's open findings and the
setups it changed (`ledger.md`, 4.12 and 4.16); written when filed
(section 5) as files at their paths in the repository:

- **`temper/docs/development/qa/requests/pass-<P>-<R>.md`,** `<P>` the pass's number and
  `<R>` the request's, where it carries findings: a report, as
  `DESIGN.md`, section 4 places reports, written by code from the
  ledger:
  - its title, `# QA pass P, request R: N findings at COMMIT12`, COMMIT
    the pass's build;
  - a paragraph saying what it asks: each finding fixed, each fix with a
    test in the lowest world that reproduces it, starting from the
    finding's lines (`../../draft/v0.1.0.md`, 4); what the cases
    expected and nothing past it (`../../draft/landed-qa.md`, 8);
  - each finding, `## F. TITLE`, then its severity, surface and page,
    area, landings (each landing's goal and title), setup (its recipe's
    path), its reproduction as a fenced `text` block of its lines, what
    was expected, what was seen, its evidence's file names, kept beside
    the QA ledger, and the earlier goals that carried it;
  - where it carries recipes, a closing section naming each.
- **Each recipe it carries,** `temper/docs/development/qa/setups/NAME/`,
  the pass's directory for it whole (`passes.md`, 2), so the pass's new
  recipes and edits land with its request as documentation
  (`../../draft/landed-qa.md`, 8).
- **A request with no finding** carries recipes alone, lands as
  documentation and starts no goal.

## 5. Filing

The loop's fourth step (`README.md`, 4), for each request not yet
filed, oldest first:

- **Its directory,** `passes/P/requests/R/`, written afresh: `tree/`,
  the request's files at their repository paths (section 4); `message`,
  the commit message, `QA pass P, request R: N findings at COMMIT12`, or
  `QA pass P: recipes` for recipes alone, then a blank line and a
  `QA-pass: P` trailer; and, where it carries findings, `goal`, the
  goal's words: `Read the QA request at
  temper/docs/development/qa/requests/pass-<P>-<R>.md and fix each finding
  it lists, each with a test in the lowest world that reproduces it,
  starting from its lines.`
- **`request.command`** run with that directory as its working
  directory, its stdin closed, the captured environment with
  `TEMPER_INTERNAL_QA_HOME` and `TEMPER_INTERNAL_QA_AREA` set, the
  latter `request.area`, both internal entries of the tool's table
  (`README.md`, 3.2), 600 seconds to exit; its stdout and stderr to
  `output` there.
- **The deployment's command,** its contract: it puts `tree/`'s files on
  main at their paths, with `message` as their commit's message, as the
  deployment puts any goal's request there; then, where `goal` exists,
  starts one goal with those words under `TEMPER_INTERNAL_QA_AREA` and prints
  `goal N` as its stdout's last nonempty line, N the goal's task number.
  Run again for the same request after a failure, it is the
  deployment's to make it safe to repeat.
- **Its answer:** exit 0, with `goal N` where `goal` exists: the request
  `filed`, with its `goal` (`ledger.md`, 4.16). Anything else is the
  step's failure, tried again at the next tick; failing for an hour, a
  line (`README.md`, 4).

## 6. Its goal's end

- **Landed:** the watcher records the goal's landing
  (`landings.md`, 4) and the pass resumes on a reached kept build
  holding it (`ledger.md`, 4.13).
- **Ended without landing:** where `request.state` is configured, the
  loop's fourth step runs it, with the goal's number as one more
  argument, for each waiting pass whose goal has no landing recorded
  and no `ended`: its stdin closed, 60 seconds to exit; its stdout's last
  nonempty line `live` changes nothing, `ended` records the request's
  `ended`, and anything else is the step's failure. Without
  `request.state`, a pass waits until its goal lands.
- **Then,** the pass resumes on the newest reached kept build and files
  its findings again in its next request (`../../draft/landed-qa.md`,
  9), each goal counting towards the two that hand a finding on
  (`ledger.md`, 4.14).

## 7. Its tests

Focused, through the binary and over the loop's fake world
(`README.md`, 6):

- **A finding** recorded with its case's surface, page and landings and
  its evidence copied; each refusal of `ledger.md`, 5's for findings.
- **Matching:** a second finding with the same setup and lines linked
  to the first; one on the same page with the same expected linked; a
  failed case whose findings are all linked moved to the other pass.
- **The request's files:** the document's title, each finding's
  reproduction block and the recipes it carries, written under the
  request's directory; recipes alone with no `goal` file.
- **Filing** through a fake command: `goal N` recorded and each
  finding's `goals` grown; a failing command retried at the next tick
  and a line after an hour; a goal reported `ended` resuming the pass on
  the newest reached build.

## 8. Open questions

- **Evidence in the request:** screenshots stay beside the ledger and
  the request names them, since the goal's coding agent needs the lines,
  not the picture. Default: so, until a finding is a look's.
- **A request's goal settings:** the deployment's command chooses the
  goal's template, budget and deadline. Default: the deployment's, until
  the ledger shows what findings' goals need.
