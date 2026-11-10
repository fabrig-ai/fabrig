# forge-reads-targeted: temper reads from the forge only what each reader needs

The owner's expectations (2026-11-09):
1. temper uses the forge efficiently. It reads only what it needs, with targeted queries, and its branch naming scheme helps where needed.
2. temper puts no needless load on Forgejo by reading too much.

A survey of `origin/main` against a real deployment, Forgejo 17.0.1, found reads well past what their readers use. Its findings are in the words below, cited by path. The evidence is outside this repository. **Run, unless marked read.**

## What the survey found

- **A comparison** (the agent's `read_compare`; forgejo `plans/contents.rs`) fetches every commit, every changed file and both whole sides of each file, unpaged. For eight files that's up to 18 requests. A comparison past the client's bounds fails `TooLarge`, so a goal branch far behind main can't be compared at all. 16 such refusals came in one day.
- **Readers that use far less than a comparison:**
  - landing news and workflow discovery use only changed file names;
  - goal overlap collects only paths, through pull file pages that fetch both sides;
  - landing members, lost-writer reconciliation and update recovery need only "does it contain X?" or "is the range empty?";
  - a brief's `pull` section fetches sides through pull files, then again through a comparison that its compare stage discards;
  - opening a pull fetches a whole commit range to learn whether it's empty.
- **A pull read** (`read_pull`) always fetches the head's status pages. A change decision then reads the head's CI and the base's CI again. Each status link's run is resolved with its whole job list, with no deduplication per run.
- **Finding a pull** (`read_pull_for`): when it falls back to scanning open pulls, it pages every open pull without the `head` and `base` filters Forgejo offers.
- **Comments are paged on temper's side only:** an issue's comments are read whole on every call (the `after` cursor is applied locally), even with nothing new, and a review's remarks are reread for each logical page.
- **Run discovery** sends `workflow=`, which Forgejo ignores. Its filter is `workflow_id`.
- **Branch lists** (`read_branches`, `GET git/refs/heads/{prefix}`) can't page and are capped at `forge.client.rows` = 8 names. One goal's saved branches already pass that.
- **Page sizes** come from the worst permitted row (about 255 KB for a pull file with both sides), so items, reviews and pull-file pages hold one row each.
- **A refusal is a bare `TooLarge`** that names no bound. `temper/docs/design/domain/agent/README.md` (about lines 374-378) says "nine files, nine commits" pass, against the bound of 8.

## What's wanted, ranked. Drop from the bottom if it passes six tasks, and say what you dropped

1. **Comparisons split by what a reader needs.**
   - Names alone: compare with `files` metadata and no raw sides.
   - Containment and emptiness: compare with `files=false`, reading `total_commits` or the pull's `merge_base` where a pull exists. No file sides.
   - Contents only when the reader asks for a file, by path, through the existing file read.

   The agent's `read_compare` returns names, counts and containment, with sides on request per path. Every engine reader takes the smallest form it needs. A goal branch far behind main can be compared for names and containment.
2. **CI read once per gather.**
   - Forgejo's combined status (`state`, `total_count`) is used where a reader needs only the verdict.
   - A pull read doesn't fetch statuses unless asked.
   - Runs are resolved once per run.
   - Run discovery filters by `workflow_id`.
3. **Narrow lists.**
   - Finding a pull uses the `head` and `base` filters.
   - Issue comments are read with `since` from the last seen update, keeping the ID cursor as the decoder's check.
   - A branch list is bounded by its bytes, not 8 rows, since names are small. Its refusal names the bound.
4. **Pages sized for real rows.** Item, review and pull-file pages hold as many rows as fit the answer's bytes, not one for a worst case that never comes, with the per-row bound still checked.
5. **Refusals name their bound and size, and the design says what passes:**
   - `TooLarge` says which bound and by how much;
   - `../domain/agent/README.md` and `../domain/forge/README.md` (about line 4078) agree with the configured figures;
   - the agents' tool descriptions say how to read on: by page, by path, or by the smaller form.
6. **The design says so,** in `temper/docs/design/domain/forge/README.md` and `../domain/agent/README.md` and the forgejo dialect's design, with their status files.

## Not in this plan

Any figure. Polling and its intervals, which another plan takes. Admission's slots. The worker's clone.

**Tests stay cheap:** a new case of one behaviour is a row of an existing table test, and no test reaches a real host. Failing tests first. **A plan runs the focused and fuzzy suites alone and is complete when it lands (`AGENTS.md`, 5).** A stored format or protocol change is a question first. `walk = false`.
