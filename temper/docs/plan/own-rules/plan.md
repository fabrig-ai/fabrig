# own-rules: plan

2026-11-03. From `inputs/request.md` (survey/agents-refs step 6, approved by the user): fabrig's
root `AGENTS.md` and `DESIGN.md` keep fabrig's own rules and drop the process of the lab that
runs its development. Wording only, two direct tasks at once.

## Parts and tasks

| Part | Task | Needs | Files | Minutes |
| --- | --- | --- | --- | --- |
| `agents` | `agents-rules` | none | root `AGENTS.md` (whole), `DESIGN.md`, `skein/AGENTS.md`, `temper/AGENTS.md`, the 7 citing lines, `outward.rs`'s doc, `outward.txt` | 30 |
| `old-forms` | `old-forms-wording` | none | the four `old-forms.txt` and `old_forms.py`, three `test_old_forms.py`, `tools/old_forms/README.md`, two design paragraphs | 20 |

Both are `docs`, effort medium, released in the first push. Slice 1 holds both.

## What the cut decided

- **No survey agent:** the work is a fixed list of lines; the cut read them itself. Citations of
  `AGENTS.md, N` found by `git grep`: 7 lines move or drop (sections 6 and 9 become 5 and 6;
  citations of 3 (the login rule, which goes), 7 and 8 are dropped), the rest cite 1 or 2,
  which keep their numbers, or sit in a dated review the checker leaves out. No test reads the
  root `AGENTS.md` but the outward check (counts) and the citation checker.
- **The new `AGENTS.md` is given whole in the spec,** so its implementer copies rather than
  composes. Two kept lines needed a touch the request leaves implicit: section 1's "On a doubt,
  stop and ask (section 7)" loses its citation (section 7 goes), and "Writing or reviewing
  design follows section 9" becomes 6. The new outward line sits in section 1 beside the
  dependency rule, the other boundary `tests/boundary` checks.
- **`TEMPER_BUNDLE=none` stays as a fabrig fact** in the new section 5 (Tests), citing
  `temper/docs/design/client/browser.md`, 2.6: only `temper` links `temper-client-bundle`,
  whose build script otherwise builds the browser's bundle for wasm32.
- **`temper/AGENTS.md`, 6** loses "leads write help pages, never implementers": it is the
  lab's division of roles (the request's "any other line there naming the lab's ... pipeline").
  `smith/AGENTS.md` and `jig/AGENTS.md` name nothing of the lab.
- **The outward inventory** loses three entries (`AGENTS.md` 24, `DESIGN.md` 1,
  `skein/AGENTS.md` 1): every name in them sits on a line this plan deletes or rewords. No
  other listed file is cleaned here (the old-forms files and the two design texts were never
  listed: "director" and "lead" are not names the check counts), so `outward.txt` has one
  writer, `agents-rules`.
- **`DESIGN.md`, 4's "never renumbered"** binds design documents; `AGENTS.md` is not one, and
  the request renumbers it. `notes/common.md` says so.
- **No design section changes meaning,** so no status entry changes; status files are a
  parallel step's anyway.

## Overlaps with other drafts

- **`outward-citations`** (integration, not yet on main): its `outward-wording` edits
  `AGENTS.md` section 8's sentence, which `agents-rules` deletes whole; its `strict-checker`
  rewrites `tools/citations/README.md` from line 4, and `agents-rules` changes line 3
  (`(AGENTS.md, 9)` to `6`). Adjacent lines: whichever merges second meets a conflict. The
  resolution is mechanical: `AGENTS.md` takes this plan's text (section 8 gone); the README
  takes `outward-citations`' text with line 3's citation set to `6`. Unavoidable: the
  renumbering must fix that citation.
- **`landing-contract`** (cut in flight): its request lowers `tests/boundary/outward.txt`
  entries, among them `.config/gate/wasm32.txt` on the line above `AGENTS.md`'s; a conflict
  there resolves as the union of both plans' removals. It edits
  `temper/docs/design/domain/testing.md` near line 1368; `agents-rules` edits line 2160 only.
  Its request leaves `AGENTS.md`, `DESIGN.md`, `tools/old_forms/README.md` and the old-forms
  baselines to this plan.
- The other drafts beside this one name none of these files.

## Critical path

`agents-rules` (~30 minutes) beside `old-forms-wording` (~20), then the trunk's end check:
`citations.py --all` at 0, the outward test and the old-forms suites green.

## For the lab

fab's implementer text cites `AGENTS.md`, 6 for the narrowest test; once `agents-rules` lands
that rule is `AGENTS.md`, 5 (section 6 becomes Design documents). The lab's own lines (heavy,
slots, how to ask, roles) now live only in the lab's briefs.
