+++
id = "probe-account"
part = "names"
kind = "docs"
needs = []
files = [
  "temper/docs/design/domain/testing.md",
  "temper/docs/design/draft/github.md",
]
effort = "medium"
minutes = 10
+++

# temper's design says "the probe account"

## Goal

Two passages of temper's design name a person's GitHub account as a probe owner. They say "the probe
account" instead, keeping their section numbers and meaning.

Acceptance:

- Neither file holds `freeekanayaka`.
- No heading changes, and each passage says what it said: the owners a run may take, and where the
  probes ran.
- The temper design's other text is untouched.

**A departure from the implementer's text, which says `docs/design/` and drafts aren't yours:** the
request names these two passages, a draft's among them, and the edit is a word's. These two passages
are all you may touch under `docs/design/`.

## Inputs

- `temper/docs/design/domain/testing.md`, the bullet at line 1236; `temper/docs/design/draft/github.md`,
  section 11, lines 619 to 633.
- `hearth/docs/plan/fail-stop-names/notes/common.md`: `freeekanayaka` stays in temper's code and
  scripts, `temper/scripts/proof-live-github.sh` among them.

## Owns and may change

- Owns those passages alone. Rewrap their paragraph as needed.
- `fabrig-ai` and the App's names stay: the request names only the person's account.
- The script still takes that literal owner, so the design says what it means and not what to type.
  That's the request's choice: leave it.

## Steps

1. `temper/docs/design/domain/testing.md`:

   ```markdown
   - **A chat's change lands on github.com, through a restart,** a test
     of its own, `crates/temper/tests/live_github.rs`, run by
     `scripts/proof-live-github.sh OWNER`, `OWNER` the probe account or
     the org `fabrig-ai`, one repository a run: the process tier's story of 2.6
   ```

2. `temper/docs/design/draft/github.md`, section 11:

   ```markdown
   Run on 2026-10-20 with a throwaway public App, `temper-probe-d0fc06`,
   installed on the probe account (repository `temper-probe`) and the org
   `fabrig-ai`, then named
   ```

   and, further down:

   ```markdown
   its fixtures, with a spare repository on the probe account,
   `temper-probe-2`, not granted to the App.
   ```

3. No test reads these files. The common note's checks, then commit and push.

## Report

The commit and the citations count.
