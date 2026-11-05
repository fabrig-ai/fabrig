# hearth-docs: how a status task works

The rules of `DESIGN.md`, 1 and 2 hold; the forms are `hearth/docs/plan/hearth-docs/notes/contracts.md`, 3. A status task owns its
documents, and does these four things to each, in this order. Writing `docs/design/` departs from the implementer's text, as
`notes/common.md` says.

1. **The closing section.** Rename `## <n>. What is open` to `## <n>. Open questions`, number unchanged. The sentence in the opening
   paragraph that points at it stays. `draft/brainstorm.md`'s is named so already.
2. **The seed.** `hearth/docs/design/STATUS.md` says, by document, what's built and what isn't. Your spec quotes what carries over.
   It's a starting point and may be wrong or stale. The "Built" column carries over to nothing, and `STATUS.md` isn't yours to edit
   (`status-index` rewrites it at the end).
3. **The gaps, checked against the code.** M1 and part of M2 are built (`hearth/docs/design/README.md`, 3 says what each builds).
   - Walk the document's numbered sections that an already started milestone builds. For each, find the code with `rg` for the type, record
     or function the section names, in the crates your spec lists, and read enough to know.
   - Designed and not built is **Not yet**, by section. Built otherwise is **Departs**, by section, with the reason; the reason comes
     from the code, a commit or `hearth/docs/development/debt/`, and where none shows, write "the reason is unknown".
   - What the document defers itself, in its `Open questions` or a later milestone's `[M5]` mark, is no gap: a section marked for a
     milestone nobody has started isn't listed one by one.
   - A document nothing of which is built says so in one line: `- **Not yet:** the whole document, which M6 builds.`
   - A document with no gap says `No gap.`, as `smith/docs/design/domain/session.status.md` does.
   - **The design disagreeing with itself or another document** isn't settled: a **Departs** entry names both places and starts
     "unsettled:".
   - The status file records only gaps as they stand, never history, and is dated from `date -u +%F`.
4. **Build state leaves the design text.** A sentence that says what the code has or hasn't got yet goes, and its gap, if it still
   stands, goes to the status file. List the candidates with:

   ```sh
   rg -n -i '\b(not yet|already built|built today|is built today|currently|at present|for now|so far|for the moment|unimplemented|implemented|on main|today)\b' <the documents>
   ```

   Most hits are design and stay: a decision ("Secure Boot off for now"), a behaviour ("what is built twice"), a machine's state, a
   precedent. Milestone words are design and stay, by `hearth/docs/design/README.md`, 3: the `[M5]` marks and each document's first
   paragraph naming what builds it. Remove the fewest words, so that the paragraph reads on and its meaning as a design stays.
   Datelines ("Provisional, 2026-10-09, revised 2026-10-19") stay.

A sentence that still describes hearth as its own repository, a gate of its own, `scripts/check.sh` or a git dependency on skein goes
too, in the fewest words (request, 4). None is known in these documents. A profile or a budget the design states stays as design, and its
status says where the root differs.

## Acceptance, in every status task

- Each owned document has its `<name>.status.md`, in the form above, and closes with `Open questions` unless your spec says otherwise.
- `git diff` of each owned document shows only the heading, the build-state sentences and any stale repository sentence, with no section
  number changed.
- `python3 -I -B tools/citations/citations.py --all` reads 0 (it holds a status file's labels against its document) and `outward` passes.

## Report

Per document: the gaps written, by label; each entry of the seed dropped, and why; each build-state sentence removed; each "unsettled:"
entry; each "the reason is unknown".
