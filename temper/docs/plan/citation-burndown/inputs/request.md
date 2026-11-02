# Request: bring the citation count to zero

From the session "fabrig citation consistency gate" (with the director's agreement), 2026-11-02. The citation
gate is live: fab's queue runs `tools/citations/citations.py --base`, and after landing it counts `--all`
(67 at main 150a68f13b; the list is in hetzner8's after-landing run, `citations.txt`). The user's plan, decision
C1 a on https://status.fabrig.ai/user/view.html?f=citation-gate.md: burn the count down to zero. After that, the
rule becomes absolute: no finding at all, with `tools/citations/allow.txt` for what must stay.

Run `python3 -I -B tools/citations/citations.py --all` for the current list. Every finding is one of three things,
and the fix differs:

1. **A real broken citation: fix the text.** Cite the section that carries the content now, or the path as it is
   (AGENTS.md 9: whoever removes or moves a section fixes its citations). Known ones:
   - jig `protocol/store.md` 15's numbered items, cited as 15.1 and 15.7, and 10.1's items, cited as 10.1.2 and
     10.1.5 (`jig-domain-tasks` `positions.rs` and `retire.rs`, `domain/people.md`, `domain/tasks.md`,
     `tests/store/.../generations.rs`). The director's pick: give those items numbered headings in `store.md`.
     Surviving numbers never change, and `store.md` is read by tests (AGENTS.md 2): run them.
   - skein `forge-github.md` 927 cites `tests/forge/github/README.md`. The file is `skein/tests/forge/github/README.md`.
   - temper `draft/reference-code-production.status.md` 50 cites "section 6 and 8.2". The draft has no 8.2.
   - temper `client/mockups/STATUS.md` 76, 84, 119 and 134: links to mockup pages and sections that no longer exist.
   - temper `protocol/configuration.md` 81 and `protocol/credentials.md` 694 cite "README, section N". Name the file
     (`README.md`, section N) so the reader, and the checker, can tell which README.
   - The director's backlog picks: temper `docs/design/qa.md` is a pointer, so remove it once nothing cites it;
     `temper/docs/plan/me-retention/plan.md` 4 cites the old `docs/design/debt/` path (plans aren't checked, but it
     is one line).
2. **The checker misreads it: fix `tools/citations/citations.py`, with a test in `test_citations.py`.**
   - "`x.md`'s, section 15" and "`x.md` (section 10, …)" belong to x.md. Today only "x.md, 15" and "x.md 15"
     do (jig `client/domain/README.md` 8, 38, `contract.md` 1368, `testing.md` 96, 421).
   - "Its sections 2, 3" with a capital letter is the cited file's, like "its" (jig `docs/development/quality-review.md` 21).
   - "its `domain/testing.md`" after "jig's `domain/engine.md`" is jig's (temper `domain/testing.md` 2183).
   - "RFC 9500's `testRSA2048` (section 2.1)" is the RFC's (skein `oauth-issuer.md` 421).
   - "`store.md`, 21 places" is a count (temper `draft/upgrades.status.md` 7).
   - `reference-case.status.md`'s labels are the case's steps ("By step"), not sections. Read them as steps, or list
     them in allow.txt.
   - Keep each fix narrow. A finding the checker drops must be a misreading, never a real break.
3. **Not a citation, and it must stay: one `allow.txt` line, with its reason.** Examples, fixtures and file names
   quoted as data (`crates/x/README.md` in temper `domain/landing.md` and `docs/help/landing.md`, `changes/1.md`,
   `process.md`, `status-page.md` in `draft/dashboards.md`, `CODEX_HOME/AGENTS.md`); evidence outside the repository
   (`~/src/rust/tmp/...` in the drafts, `ladder-b01/...` and `qa-focused/...` in code comments, `agents/backlog.md`);
   and history kept on purpose (`draft/notes.md` 512's `landing-model.md` with its commit, 168's old file names).
   Rewording a comment so that it no longer names a dead file is also fine.

Out of scope: X5 (playground-reset), which another session owns, and skein foundation's missing status files.

Done when `citations.py --all` exits 0 on main. Then tell the session "fabrig citation consistency gate": the queue
and after-landing switch to the absolute rule. Tasks: one per tree for (1), one for the checker (2), and allow.txt
with whichever task lands last, or with the checker task. Docs-only tasks are cheap. Priority: after the
ladder's repairs, but ahead of other doc work, because design-doc work is waiting on this.
