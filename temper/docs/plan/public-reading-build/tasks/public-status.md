+++
id = "public-status"
part = "public-checks"
kind = "docs"
needs = ["public-limits", "visibility-surface", "public-wire", "public-vocabulary", "forge-public-wire", "fetch-headers", "visibility-wire", "visibility-core", "public-index", "public-reads-core", "public-component", "public-http", "public-story", "public-fuzzy-jig", "forge-public", "public-ctl", "public-web", "public-walk", "public-fuzzy-temper", "public-negative"]
files = ["jig/docs/design/domain/people.status.md", "jig/docs/design/protocol/client.status.md", "jig/docs/design/client/domain/contract.status.md", "jig/docs/design/domain/engine.status.md", "jig/docs/design/domain/root.status.md", "jig/docs/design/protocol/configuration.status.md", "jig/docs/design/protocol/store.status.md", "jig/docs/design/domain/authority.status.md", "jig/docs/design/STATUS.md", "temper/docs/design/client/forge.status.md", "temper/docs/design/client/browser.status.md", "temper/docs/design/protocol/sign-in.status.md", "temper/docs/design/client/tools.status.md", "temper/docs/design/draft/first-release.status.md", "temper/docs/design/STATUS.md", "skein/docs/design/ui-web.status.md", "skein/docs/design/STATUS.md"]
effort = "medium"
minutes = 30
+++

# The statuses: public reading built, and what is left

## Goal

Once every task of the plan has merged, the status files say what the code now lags in
public reading, and nothing more (`DESIGN.md`, section 2: a status records only the gaps as
they stand, never history): every **Not yet** entry this plan built leaves, what is still not
built stays, worded as a gap, and each tree's `STATUS.md` summary agrees.

Acceptance:

- **Removed where built,** each checked against the merged code (below), entry by entry:
  - jig: `domain/people.status.md`, 13 (the public party, `SetVisibility`, visibility in the
    people family's rows, the public tasks' rule, public identifiers, the public index, the
    public documents, admission, the recheck, races and world); `protocol/client.status.md`,
    13 (the public reads' limits) and every 17.5 and 19 entry of public reading;
    `client/domain/contract.status.md`, 3.2 (`SetVisibility`), 5.1 (`me`'s public
    identifier) and 12; `domain/engine.status.md`, 11; `domain/root.status.md`, 3 and 10;
    `protocol/configuration.status.md`, 6's public figures and relations;
    `protocol/store.status.md`, 4.4 (a project's visibility) and 9 (the public identifiers'
    key); `domain/authority.status.md`, 6 (the public party's role);
  - temper: `client/forge.status.md`, 11; `client/browser.status.md`, 8;
    `protocol/sign-in.status.md`, 2.5; `client/tools.status.md`, 5.2 (`project show`'s
    visibility and public page) and 5.3 (`project amend --visibility`);
  - skein: `ui-web.status.md`, 10 (an exchange's cache and an answer's kept headers).
  Each other item in those entries (a different section, a different gap) stays as it is.
- **Kept where not built:** `protocol/sign-in.status.md`, 2.4: no operator's proxy
  configuration caches `/jig/v1/public/` by deployment, `Host` and path, keeps no stale copy
  or limits an address, and no deployment sets jig's public figures (the world's proxy of
  2.5 is a test's, not a deployment's); `draft/first-release.status.md`, 5.3's figures stay
  the user's (Open). Any part of section 13, 17.5, 19, 11 or 8 that a merged task reported
  under `## Left` (not built, or built otherwise) stays or becomes a **Not yet** or
  **Departs** entry in its document's status, in the status's own words, with the code's path
  for a departure.
- **`draft/first-release.status.md`:** only what the build changed: its sentence saying the
  code lags the documents of 5.3 says instead what still lags (the proxy's configuration,
  the figures); its other entries unchanged.
- **The indexes:** `jig/docs/design/STATUS.md`'s paragraph ("Public reading is designed and
  not built ...") and `temper/docs/design/STATUS.md`'s ("public reading's pages, the forge's
  public documents, a deployment's proxy and ctl's visibility are designed and not built")
  say what is left (temper: a deployment's proxy configuration), or drop the clause where
  nothing is; `skein/docs/design/STATUS.md`'s clause on `ui-web.md` (10) goes. Each status's
  date line becomes the day of this task's commit (`date -u`).
- `python3 -I -B tools/citations/citations.py --base origin/main` passes: no citation of a
  status label or section broken by an entry's removal.

## Inputs

- `DESIGN.md`, sections 2 (status files) and the `STATUS.md` rule; `AGENTS.md`, 9.
- `temper/docs/plan/public-reading-build/notes/common.md`; `plan.md` (the revision, and what
  each task built).
- Each merged task's report as fab keeps it, where it is readable to you; otherwise the code
  on your base: for each entry you remove, name in your report the file and item that builds
  it (for example `jig-domain-core/src/public.rs`, `PublicRead`; `temper-client-tools`'
  `rows::PROJECT_AMEND`), and keep the entry if you cannot find one.
- The status files in the header, and the sections they cite, as they stand on your base.

## Owns and may change

- Owns the status files and indexes in the header; this task alone of the plan edits design
  status files. No design document, draft (but its status), note, help page or code changes:
  where the code departs from a design section, the status says so; the design is not
  edited to match.
- Pre-approved: if a merged task added a status entry of its own, keep its form and merge
  it into the entry for its section; if a status ends with no gap, it says so in one line, as
  `DESIGN.md` asks.

## Steps

1. For each entry above, find the code that builds it on your base (and the task's report),
   remove what is built, word what is left.
2. The indexes and dates.
3. `python3 -I -B tools/citations/citations.py --base origin/main`; no build or test is
   needed for status files (none is read by a test: `.config/gate/read-by-tests.txt`).
4. Commit (why: the statuses say what public reading still lacks once built); push your
   branch.

## Report

As the implementer's text says, and also: each entry removed with the code that builds it,
each entry kept or added with why, and the indexes' new sentences.

## Answers

2026-11-03. **The user's decision on public titles.** User, 2026-11-03 15:40Z: public reading shows titles. A goal's and a task's title (the first line of its spec, as jig's client contract names a task's title) crosses into its public line, and a public project's name crosses into its public record. There is no author flag and no human publication capability for these. Everything else in people.md 13.3's "No text at all" list still never crosses: spec beyond its first line, instructions, inputs, results, words, questions and answers, transcripts, notes, reasons, labels' meanings, proposals, escalations, people's and parties' names, charters', procedures' and roles' names. Review finding 1 is settled by the user's decision.

For this task: `temper/docs/design/draft/first-release.status.md` records the user's decision and closes the names entry for titles; people's names stay hidden.
