+++
id = "status-draft"
part = "status"
kind = "docs"
needs = ["citations-docs"]
files = ["hearth/docs/design/draft/brainstorm.status.md"]
effort = "high"
minutes = 50
+++

# Status for the brainstorm

## Goal

`hearth/docs/design/draft/brainstorm.md` is hearth's one live draft, and a draft wins over the design unless its status says otherwise
(`DESIGN.md`, 1). `hearth/docs/design/README.md` says the design "holds where the two differ", but nothing lists where. This status does.
`draft/archive/factory.md` is archived as temper's `draft/archive/` is: the file alone, with no status and no entry in `STATUS.md`, so it needs
nothing here.

Acceptance:

- `draft/brainstorm.status.md` is in `DESIGN.md`, 2's form for a draft, dated: **Design lags**, **Design departs**, **Open**, each by the
  draft's section; a label with nothing under it is left out.
- Every difference `hearth/docs/design/README.md`, 7 names that touches the draft has its **Design departs** entry, saying what the design does,
  why, and who decided, from what section 7 records ("Decided with the user's input", dated, or "the design's own" where it names no one).
- Each question of the draft's section 13 is settled by a section of the design, which the entry cites, or is an **Open** entry.
- `git diff --stat` shows the status file alone: the draft's text doesn't change.
- `python3 -I -B tools/citations/citations.py --all` reads 0 and `outward` passes.

## Inputs

- `hearth/docs/plan/hearth-docs/notes/common.md`
- `hearth/docs/plan/hearth-docs/notes/contracts.md`, 3
- `DESIGN.md`, 1 and 2
- `temper/docs/design/draft/first-release.status.md`: a draft's status, for its form
- `hearth/docs/design/draft/brainstorm.md`
- `hearth/docs/design/README.md`, 2, 3, 7 and 9, then the sections each entry needs

## Owns and may change

- **Owns:** `hearth/docs/design/draft/brainstorm.status.md`, new. Writing `docs/design/` departs from the implementer's text, as
  `notes/common.md` says.
- **Not to touch:** the draft, the design documents, `hearth/docs/design/STATUS.md`, `draft/archive/factory.md`.
- **Options:**
  - A disagreement among design documents that you meet goes in your report; the draft's status records only draft against design.
  - The draft's section 12, its order of building, is carried by `hearth/docs/design/README.md`, 3: a difference there is **Design departs**.
  - A sentence of the draft no design section carries is **Design lags** when a later milestone may yet carry it, and **Open** when it waits on
    the user.

## Steps

1. Read the draft by section, and `hearth/docs/design/README.md`, 7.
2. For each section, find where the design carries it, using `hearth/docs/design/README.md`, 2's reading order, and write only the differences.
3. Check section 13's questions against each design document's closing section.
4. Commit (`hearth: the brainstorm's status`), run the checks in `notes/common.md`, push.

## Report

The entries by label and draft section; each of section 13's questions and where it went; any disagreement among design documents you met.
