+++
id = "keep-decisions"
part = "keep-decisions"
kind = "docs"
needs = ["audit-goal", "audit-landing", "audit-configuring", "audit-playground", "citations-out"]
files = [
  "skein/docs/design/**",
  "smith/docs/design/**",
  "jig/docs/design/**",
  "temper/docs/design/**",
  "tools/tiers/design.md",
  "tools/tiers/design.status.md",
]
effort = "high"
minutes = 100
+++

# What only the retired plans record, written into the design

## Goal

Four readers have checked the 49 retired plans against the design and written what it lacks
as findings in four kept notes. This task writes each finding into the design document or
status file it concerns, dated, in that document's words, with no reference to the plan or to
who ran it, and lists in the commit what was added and where (the request's Task 1). After it,
the directories can go.

Acceptance:

- Every finding of the four notes (`temper/docs/plan/retired-plans-out/notes/kept-goal.md`,
  `kept-landing.md`, `kept-configuring.md`, `kept-playground.md`) ends **written**, **merged**,
  **skipped** or **left** (`notes/contracts.md`, K3), and the report's table gives each, by
  number, with its file and section.
- The commit messages list each addition as `` `<file>`, <section>: <a few words> `` with its
  finding numbers, one line each.
- What you add follows `notes/common.md`, "What text added outside `docs/plan/` may say", and
  "Where the design may change, and how": `## Steps`, 4's check prints nothing but product
  words, which the report names.
- `notes/common.md`'s checks: citations at 0; the outward test passes; the reader tests of any
  file `.config/gate/read-by-tests.txt` lists that you changed pass.
- No file changes outside `files` above. Pushed.

## Inputs

- `temper/docs/plan/retired-plans-out/notes/common.md`: the standing rules, the allowed
  targets, and the files other drafts are changing.
- `temper/docs/plan/retired-plans-out/notes/contracts.md`: K1 (the notes' form), K2 (what each
  finding is), K3 (what you do with it).
- The four kept notes, whole: your input. Their **Source** lines point into the 49 directories,
  which are still there: read a source where a finding's **Text** leaves its fact unclear.
- `DESIGN.md`, whole: a design document holds the contract and closes with its "Later" or
  "Open questions"; its status holds **Not yet** and **Departs**; a draft's status holds
  **Design lags**, **Design departs** (with who decided) and **Open**; each tree's `STATUS.md`
  indexes its status files.
- `temper/docs/plan/retired-plans-out/inputs/request.md`, "Task 1".

## Owns and may change

- **`files` above:** the design documents and status files the findings target, the drafts'
  status files, `tools/tiers/design.md` and its status. Each edit is a finding's text at its
  place, the edited status files' date lines (`notes/common.md`), and a tree's `STATUS.md`
  line where its summary of a status file stops being true (a file that said it had no gap
  gains one).
- **Never:** a draft's own text, `draft/notes.md`, `docs/help/`, `AGENTS.md`, `DESIGN.md`,
  `skein/docs/foundation/`, any code, any plan file (the kept notes included).
- **A finding's wording** is the reader's proposal: fit it to the target section's words and
  form, shorten it, or place it in a nearer section of the same document, and say so in the
  table. A finding whose **Text** names a plan, a role or the lab is rewritten without them.
- **The same fact in two findings** (two plans, or two notes): written once, **merged**, both
  numbers in the table and the commit line.
- **Held or superseded at your base** (another draft may have landed it since the reader
  read): **skipped**, naming the section that holds it now.
- **A finding that contradicts the design** with nothing saying which came later: the design
  stands; the finding is **left**, under `## Left` with both texts, for the user.
- **Target "none: ..., the user's":** **left**, under `## Left` with its text.
- **A target in a file another draft is changing** (`notes/common.md`'s list): write it as a
  line or bullet of its own, at the section's end; never rewrite an existing line there.
- **An open choice** goes to the design document's "Open questions" (or "Later", where the
  document has only that), or to a draft's status as **Open**, with each option; it stays
  open: no default is chosen here.
- **smith's design** is read by tests (`smith-bench`): if you change a file under
  `smith/docs/design/`, run `heavy cargo nextest run -p smith-bench`; a red there that your
  text caused is yours to fix in the text.

## Steps

1. **Read the four notes** and list the findings by target file. Check each against its
   target at your base: the section is there, and it does not already say the fact.

2. **Write, file by file.** For each target, read the section, then add the fact in the
   document's words, dated: a design sentence carries "(decided YYYY-MM-DD)" or "(YYYY-MM-DD)"
   where the date is the decision's, not today's; a status entry is
   `<section>: <the gap>, because <the reason>` under its label, the file's date line today's.
   One commit per tree, its message listing that tree's additions (Goal, second bullet).

3. **The table.** In the report, one row per finding: number, outcome, file and section (or the
   other number it merged with, or where it was held, or why it is left).

4. **The check.** From the repository root, with `<base>` your branch's base:

   ```sh
   git diff <base>..HEAD -- . ':!*/docs/plan/*' | grep '^+' \
     | grep -niE 'docs/plan|reference-m[0-9]|configuring-p[0-9]|wasm32-(jig|skein|list|more|rustls|temper)|live-smith-|live-temper-stories|playground-(skein|temper|decisions)|\bfab\b|heavy (cargo|env|--)|agents (repo|repository)|fabrig/qa|agents/loop|hel8|director|implementer|answer call|the cut\b|\blead\b'
   ```

   It prints nothing, or only product words (a forge's "lead", a role a person holds), which
   the report names.

5. **The checks of `notes/common.md`,** after committing: citations at 0; the outward test; the
   reader tests of what you changed. Push.

## Report

`Done:` with the counts written, merged, skipped and left. Then the table of `## Steps`, 3; the
commit hashes and their messages' lists; the checks' results (the outward test's count,
citations' line, reader tests run with their counts); `## Left`; your minutes.
