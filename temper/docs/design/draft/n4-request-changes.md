# N4: a person's "request changes" on the forge starts a repair

Milestone N4 of `temper/docs/design/draft/whole-story.execution.md`, section 3, under "N4 to N6, P3e, P3f, P7 and P8". Today a person who asks for changes on the forge's review page only holds the change (`temper/docs/design/domain/forge.md`, 8.3). Their remarks should become a repair's brief, as a blocking review's remarks do, so that You approve's loop closes on the forge. The milestone writes its design as it runs, in `forge.md` 8.3 and wherever a blocking review's repair is written, then the code and its focused and fuzzy tests. **Run, unless marked read.**

## What's wanted, ranked

1. **The failing story first,** at the lowest tier holding the forge's review reads and a change's repairs: a person's review that requests changes, with remarks on the change's head, starts a repair whose brief carries those remarks, and the change goes on as it does after a blocking review's repair. Rows of the existing review table:
   - a person's request with remarks;
   - one with no remarks;
   - one on a head the change has moved past;
   - one past the change's repairs bound, which holds as today.
2. **The fix,** in the forge's route, as a blocking review's is today. A person's remarks are read as words, never as instructions to the repair beyond the brief's framing.
3. **The design says so:** `forge.md` 8.3, and `what-reaches-you.md` wherever You approve's loop is drawn.

## Not in this plan

A charter's words, and any figure.

**Tests stay cheap:** a new case of one behaviour is a row of an existing table test; no test reaches a real host or a real LLM. Failing tests first. **A plan runs the focused and fuzzy suites alone and is complete when it lands (`AGENTS.md`, 5).** A stored format or protocol change is a question first. `walk = false`.

The close-out leaves this request removed.
