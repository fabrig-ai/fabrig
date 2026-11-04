# Request: the citation checker has no outside-repository exception

From survey/agents-refs, 2026-11-03; the user approved the plan (step 2 of 12). fabrig is to name
nothing of whoever runs its development. Today `tools/citations/citations.py` excuses citations of
another repository's documents ("hearth's", and the lab's repository, phrased "agents'" or "the
agents repository's"), and so hides broken citations. One plan; one or two direct tasks.

## Changes
1. **Drop the lab exception.** `citations.py` `EXTERNAL` (~l.38-40) keeps only hearth's branch.
   A citation of another repository's document is refused, except hearth's; say so explicitly in
   the code, since resolution falls back by suffix (~l.169) and would otherwise match a local file
   of the same name. Flip `test_citations.py` (~l.274, which asserts such a citation passes), and
   reword `tools/citations/README.md` (~l.26-29), whose example names the lab's design document.
2. **Read what it skips.** The checker reads the root `DESIGN.md` and `tools/**/*.md`, scoping out
   their teaching examples (`tools/citations/README.md` l.4, 19: `x.md`, `../../a.md`) by syntax or a
   scoped `allow.txt` entry with its reason. It skips `docs/plan/` for **every** extension (today it
   skips the `.md` files there but checks the comments of every `plan.toml`; the README already says
   `docs/plan/` is left out).
3. **Fix what it then shows.** At least these, each rephrased in fabrig's own words or dropped,
   never re-pointed into the other repository:
   - `skein/scripts/browser.sh:2`, `jig/scripts/browser.sh:2` (a landing-flow document);
   - `jig/scripts/live-sign-in.sh:8`, `temper/scripts/proof-live.sh`, `proof-live-*.sh` (a
     build-infrastructure document);
   - `temper/docs/design/draft/dashboards.md` l.10, 58, 197, 370 (a status-page document): where it
     is the draft's example or test input, say what the example is in the draft's own words;
   - any other the stricter checker finds.
4. **The allow list.** Drop `tools/citations/allow.txt` entries for records kept outside the
   repository (the "Evidence outside the repository" block: the lab's backlog, ladder-walk reports,
   a QA brief): state the observed fact where it is used, with its date, instead
   (`skein/crates/skein-fake-forge-protocol/src/answers.rs` ~l.368,
   `temper/crates/temper/tests/support/csv_repair.rs` ~l.21-22). Keep the other entries.

## Bounds
- `tools/citations/citations.py --all` reads 0 at the end; its tests pass.
- Only wording changes outside `tools/citations/`: no behaviour change in any script.
- If a test outward check (`tests/boundary/tests/outward.rs`, a parallel plan) exists at the base,
  keep within its inventory (counts may only fall).
