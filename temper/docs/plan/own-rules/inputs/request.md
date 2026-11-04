# Request: AGENTS.md and DESIGN.md hold fabrig's own rules only

From survey/agents-refs, 2026-11-03; the user approved the plan (step 6 of 12). fabrig is to name
nothing of whoever runs its development. The root `AGENTS.md` mixes fabrig's own rules (code,
security, tests, design documents) with the process of the lab that runs fabrig's development
(its pipeline, its build tool and machines, its slots, how to ask). The lab now gives its agents
those lines itself, so fabrig drops them. Wording only. One or two direct tasks.

## 1. Root AGENTS.md (line numbers at main ffdebce7de; re-check)
- **Delete:** l.3's last sentence (role instructions come from the pipeline); §1 l.11, l.12, l.13;
  §3 l.30, l.32, l.33; §4 l.37, l.38, l.39; §5 whole (l.45-51); §7 whole (l.65-67); §8 whole
  (l.71); §9 l.75's sentence about the pipeline's charters.
- **Keep, reworded:** §6 l.55 as the same rule with plain `cargo nextest run -p PKG --test BIN -E
  'test(=NAME)'`; l.57 without "(heavy prints its path)"; l.61 "ask the user, with measurements"
  in place of "through the director"; §2 l.24 "no `qa` path or address" → "no path or address of a
  machine outside the repository". `TEMPER_BUNDLE=none` (l.50) stays as a fabrig fact if a build
  needs it (say where it applies), without "the machines' cargo config".
- **Keep as they are:** the rest of §1, §2, §3 l.28/29/31, §4 l.40-41, the rest of §6, §9.
- **Add to §1** one line: fabrig names no tool, machine, session or repository of whoever runs its
  development; `tests/boundary/tests/outward.rs` checks it.
- Renumber the sections that remain, and fix every citation of `AGENTS.md, N` in the repository
  (the citation checker reads them); `CLAUDE.md` stays `@AGENTS.md`.
- The trees' `AGENTS.md`: `skein/AGENTS.md:9` keeps its standalone TLS check as plain `cargo
  check`; any other line there naming the lab's tool or pipeline is reworded the same way.

## 2. DESIGN.md
- l.5: drop the sentence that the pipeline's charters carry this file; say instead that whoever
  writes or reviews design reads this file.
- l.42: "the queue rejects ..." → "a branch with a broken citation does not land".

## 3. The old-forms counts
- "only the director raises one" in `skein/`, `smith/`, `jig/` `scripts/old-forms.txt` (l.2), the
  headers `old_forms.py` writes (`skein/scripts/old_forms.py` ~l.77, `smith/...` ~l.30, `jig/...`
  ~l.26) and their tests (`jig/scripts/test_old_forms.py` ~l.16, 147, 149;
  `smith/scripts/test_old_forms.py` ~l.65): a count rises only with a `# Raised:` line saying who
  and why. Check temper's copy too.
- `tools/old_forms/README.md`: l.4 "the queue" → "a landing check"; l.64-67 the commands without
  the `heavy` prefix.

## 4. Design text naming the lab's people
- `skein/docs/foundation/testing-strategy.md` ~l.619 ("a lead's run"), ~l.650 ("the user through
  the director"); `jig/docs/design/domain/testing.md` ~l.377: say it without the lab's roles
  ("the user").

## Bounds
- No behaviour change; tests pass (the old-forms tests change only their expected header text).
- `tools/citations/citations.py --all` at 0.
- `tests/boundary/outward.txt`: lower the entry of each file this cleans.
- Not here: `.config/`, `tools/tiers/`, status files (a parallel step owns them).
