# long-text-pages: a long text read in pages is read whole

On a real deployment, a change's agent read its 35 KB specification through `task text`. It got the first 16 KiB page, which ends `18800 bytes left; read on with: task text 240 --field specification --from 16384`. It stopped there, held itself, and asked for the "missing" steps. The page bound is `temper-client-tools`' `limits::PART`, 16 KiB. With a result's text, `limits::TEXT`, at 32 KiB, it decides how much of a task's text, a turn's part or a document one call prints (`temper/docs/design/client/tools.md`, section 12). A specification may be up to 128 KiB (`tasks.spec_bytes`, standard), so up to eight reads, and each page is a chance to stop early. The owner has approved both parts below. **Run, unless marked read.**

## What's wanted, ranked. Drop from the bottom if it passes three tasks, and say what you dropped

1. **A cut page says so first.** Every paged read prints its position in its first line as well as its last: which page of how many, the bytes it holds, and the bytes left with the call that reads on. This covers `task text`, `task turn --part` and any other read that continues with `--from`. An uncut text says nothing new. These are the tools' words: agents and people read the same output.
2. **One page holds 64 KiB.** `PART` becomes 65,536 source bytes, with `TEXT` and the result's bytes raised so that a page that size still fits. JSON's six bytes for one source byte stay reserved, as `task_source::capacity` reserves them. The step test of 11.1 (the tools' window within the client's limit) still holds, raising the client's bound if it must. Say what it took.
3. **The design says so:** `temper/docs/design/client/tools.md`, section 12's table, and 6.4 where it describes continuing a read, with their status files. Regenerate the help pages that print these words or bounds with their own tools.

## Not in this plan

Any other figure. Charter wording: the charters are generated elsewhere, and the owner adds their line separately.

**Tests stay cheap:** a new case of one behaviour is a row of an existing table test. Failing tests first. **A plan runs the focused and fuzzy suites alone and is complete when it lands (`AGENTS.md`, 5).** A stored format or protocol change is a question first. `walk = false`.
