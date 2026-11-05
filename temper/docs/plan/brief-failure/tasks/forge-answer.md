+++
id = "forge-answer"
part = "forge-answer"
kind = "feature"
needs = []
files = [
  "skein/crates/skein-forge-client/src/api.rs",
  "skein/crates/skein-forge-client/src/calls.rs",
  "skein/crates/skein-forge-client/src/tests/**",
  "skein/crates/skein-forge/src/forgejo/**",
]
effort = "high"
minutes = 60
+++

# The real Forgejo answer the client refuses

## Goal

On a deployment, real reads of one repository are refused `InvalidAnswer`, from an agent's `read_pull_for` and `read_compare` tools, and very likely from a repair brief's own reads. The deployment's answers are being captured; they reach this plan as the answer to this task's question. Find the parser or check that refuses a real Forgejo's answer and fix it, with that answer as a fixture. finding-323 (`git log --grep=finding-323`), a real compare listing each file twice, is a landed cousin.

**This task departs from the implementer's text:** its first run ends in a `Question:` by design, unless the capture is already in `temper/docs/plan/brief-failure/inputs/`. Nothing else in this plan needs it or waits for it.

Acceptance, one of:

- With a capture: a fixture of the real answer, its addresses and names replaced by the fixtures' own, a test that fails on today's code with `InvalidAnswer` and passes after the fix, in the lowest tier that shows it, and the design's words on what a Forgejo answers, if the fix needs them, as a report item.
- The answer says no capture will come: `Done:`, no code change, your reading of the candidates in the report, and `## Left` naming the cause as open.

## Inputs

- `temper/docs/plan/brief-failure/notes/common.md`.
- `temper/docs/plan/brief-failure/inputs/request.md`, the route and item 4.
- `skein/docs/design/forge-client.md`; `skein/docs/design/forge.md`, 3.3 and 3.4.
- `skein/AGENTS.md`.

## Owns and may change

Owns the files in `files`.

May also change: a fixture file beside the test; `skein/crates/skein-forge-connection/src/connection/forgejo/tests.rs` and `skein/crates/skein-forge-connection/src/tests/reads.rs`, adding tests only, where the fault shows at the connection. `debt-21` changes `skein-forge-connection`'s production files: a fix needed there is a `Question:`.

Options, pre-approved:

- The fault is the check being too strict on a value a real forge may lawfully send: loosen that check, keeping the rule that an answer for another object is refused, with a test that a wrong answer stays refused.
- The fault is the dialect's decoding: fix it there, with the fixture at the dialect's tests.
- The fault is not in these crates: report it, with the evidence, as a `Question:`.

No probe of a real host, ever (`AGENTS.md`, 3): the capture is what you have.

## Steps

1. Read the candidates, and rule out what you can. From the lead's reading, none confirmed: `api.rs`, `compared` (the observed `before` and `after` must be the asked ones: a dialect giving back a merge base, a short or a resolved commit would fail it); `read_accepts`, `Read::PullFor` (`pull.head` and `pull.base` must be the asked ones, byte for byte); `calls.rs`, `Err(Error::Busy) => Err(Error::InvalidAnswer)` (a connection's `Busy` reads as a bad answer); `skein-forge/src/forgejo/decode*` for pull and compare, what a real answer may hold `null` or omit.
2. If no capture is in `inputs/`: write the report with first line `Question: what did the deployment's API answer to the refused reads? I need the raw answers to a pull-by-branch read and a compare read of that repository, with their request paths.` Give your ranked candidates under it. Recommend sending them; no code pushed. Stop.
3. With the capture: the failing test first, quoting its failure in the report; the fix; the bite check.
4. `cargo fmt`; old forms for skein; `heavy cargo clippy -p skein-forge-client -p skein-forge --all-targets -- -D warnings`; `heavy cargo nextest run -p skein-forge-client -p skein-forge`. Check each count. Commit, push.

## Report

As the implementer's text has it, and what the real answer held that the code didn't take.

## Answers

2026-11-05 (fabrig/temper). The capture is here, so don't stop for it: temper/docs/plan/brief-failure/notes/pr7/ on the trunk (pull.json, status.json, compare-base-sha.json, base-ref-tip.json, index.txt). Read at 15:02Z, anonymously, the repository being public, after the refusals began at 14:51Z. So a field that varies with the caller may differ, and the head's status had by then gone green after a re-run. Base and head are the ones the failing brief read.

First: decode each through skein-forge's real decoder and the client's checks (compared, the pull's checks) as a test, and report plainly which pass. By eye they look ordinary: compare has 13 commits and total_commits 13, 50 file rows over 16 paths (finding-323's repeats, whose repair is in the deployed build), and merge commits in the list. No cause is proved, and the request's route is a guess: if all of them pass, say so, and don't invent a fix.

Candidates to read, none proved:
- the compare plan's total_commits != commits.len(), in skein-forge/src/forgejo/plans/commits.rs: Forgejo may list fewer commits than total_commits on a long range, as the journal's refusals of read_compare were on other tasks' ranges. A real refusal of a long range would be a true bug, whichever task it hit;
- a tool's refusal from the agent's own arguments (a before that isn't the answer's), which is not a parser fault, and is words at most;
- what this capture lacks: the red state. The brief's Ci section reads checks and jobs at a red head. A later repair of the same change read 'Repair CI at <head>: ' with no check named, which first-goal-fixes owns, but it may be the same answer's shape. If the cause isn't in this capture, name exactly which answer you'd want, in the red state, under ## Left: a request can be made of the deployment by a person, not by a test.

A fixture's names and addresses go neutral, as your task says. No test reaches a real host. Done with no code change and the candidates read is an accepted end.
