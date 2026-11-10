# forge-polls-quiet: temper polls the forge only when its events can't tell it

The owner's expectation (2026-11-09): temper puts no needless load on Forgejo by reading too much. A survey of `origin/main` against a real deployment (Forgejo 17.0.1, its push, pull, comment, review and workflow-job events subscribed and handled) found these recurring reads of unchanged facts. The evidence is outside this repository. **Run, unless marked read.**

- **Any watch installs a repository scan** of issues and pulls every 5 s (`poll_ms`), a branch-only watch included, though a scan can't nudge a branch.
- **Pull watchers** reread the item, its whole comment list, the pull and its status pages every 5 to 30 s. Issues and missing resources are reread every 40 s. A branch watch rereads the branch about every 40 s. Each of these continues while the forge's events arrive and say nothing changed.
- **Cached facts suppress duplicate news, not reads.** A fresh read never consults the keep's cache first. No request sends `If-None-Match` or `If-Modified-Since`, and no validator is kept. Forgejo returns an ETag on raw file reads, and answered 304 with 0 bytes to one. Branch JSON returned no ETag.
- **Failures retry as if transient.** A repository scan retries any error at the same cursor after 1 s. A resource read retries a non-missing failure after 1 s, a comparison refused `TooLarge` included, which can never succeed.
- **A queued merge** polls its pull every 5 s. **A landing or tier flight** polls its run, with its whole job list, every 5 s.

## What's wanted, ranked. Drop from the bottom if it passes four tasks, and say what you dropped

1. **Scans only where they can matter.** A repository's issue and pull scan runs only while an issue or pull watch needs it.
2. **Events first, polls as a fallback.**
   - While the forge's events are arriving for a repository, its fallback polls stretch to a long interval. Choose it, and say why it's safe.
   - A missed event is still caught. A watch with no event for longer than the fallback polls as today.
   - A hint still rereads at once.
   - Say how temper knows events are arriving: delivery acknowledgements, a heartbeat, or a periodic reconciliation.
3. **Permanent failures stop.**
   - A refusal that can't change for the same inputs (`TooLarge` for an immutable range, a bound past the answer) is recorded and not retried until its inputs change.
   - A transient one backs off with growing waits, capped.
4. **Conditional reads where the forge supports them.**
   - Keep each answer's ETag and Last-Modified, and send them on the next read of the same resource.
   - A 304 reuses the kept answer and counts as a read.
   - Where Forgejo sends no validator, nothing changes.
5. **The design says so:** `temper/docs/design/domain/forge/README.md` (watches, polls, retries) and its status file.

## Not in this plan

Any figure but the new fallback interval this plan chooses, which it names and justifies. The shapes of reads, which another plan takes (comparisons, CI fan-out, pages). Admission's slots.

**Tests stay cheap:** a new case of one behaviour is a row of an existing table test, and no test reaches a real host. Failing tests first. **A plan runs the focused and fuzzy suites alone and is complete when it lands (`AGENTS.md`, 5).** A stored format or protocol change is a question first. `walk = false`.
