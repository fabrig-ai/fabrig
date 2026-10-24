# The audit's brief

The brief for Codex's audit of a group of pages, the first of the three
moves that bring the group under the README's rules (`README.md`,
section 5). Fill in the group, its pages and its states (each step's
`look` lists them), and the documents that design them; run it with
`codex exec`, its stdin the brief; read its report with the user.

---

# Audit: GROUP, in temper's client mockups

Read-only everywhere: edit, commit or build nothing. Write your report
only to REPORT, and screenshots beside it, in `shots/`. Use `rg`.

## Context

temper's client mockups (`~/src/rust/temper`, at `main`:
`docs/design/client/mockups/`, its `README.md` first) are the design of
the client's pages. Under their rules, a page's main drawing shows only
what the engine already has (temper, and jig and skein at the versions
temper's `Cargo.lock` pins: `~/src/rust/jig`, `~/src/rust/skein`), so
that building the page is the client's work alone; anything that needs
work below the client is a future piece, drawn only under View →
Future. This audit is the input for bringing GROUP under those rules.

The client's code: `crates/temper-client-domain` (pages, routes,
addresses), `crates/temper-client-view` (what each page draws, its
words), `crates/temper-client-shell-web`, and the engine's side that
the pages read. Its design: DOCUMENTS, and `docs/design/STATUS.md` (what
is built). Render a state with `chromium --headless=new --no-sandbox
--window-size=1300,2400 --screenshot=FILE 'file://…/PAGE.html?QUERY'`.

## Scope

PAGES, at these states (`page?query`, each with its `world` and
`step`): STATES.

A page that is not the client's (GitHub's own, in `github.html`):
audit only its slate notes' claims about what temper does.

## The report

At the top, a summary: counts per class, and the five findings that
matter most. Then, for each state, a table of what it draws: every
value, every word that isn't generic chrome, every act and where it
leads or what it does, each classed:

- **BUILT:** the client draws it today. Its code (`path:line`), and
  both wordings where they differ.
- **LAG:** the client doesn't draw it, but the engine has the data or
  the operation it needs: the record, field or operation
  (`path:line`) and its design section.
- **FUTURE:** it needs work below the client: exactly what is missing,
  and the design, draft or plan that covers it with its state in
  `docs/design/STATUS.md`, or "not designed".
- **UNCLEAR:** what you checked, and why it didn't decide.

Then four lists:

1. **Disagreements** between the mockup, the documents and the code,
   about words, parts, order or behaviour: each side quoted, and the
   side the code takes. Include `STATUS.md`'s notes for these pages and
   the documents' passages that compare a page with its mockup.
2. **Stale marks:** each mark or note in scope whose claim is no longer
   true, with the evidence.
3. **The code ahead of the mockup:** what the client draws or does on
   these pages that no state shows.
4. **Citations:** for each source tag in scope (`fact`, `conf`), its
   prose, the citation that should replace it (the design section, and
   the record and field), the code that holds it (`path:line`), and
   whether its claim is wrong.

Be factual and complete, and prefer evidence to judgement.
