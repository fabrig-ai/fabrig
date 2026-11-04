# outward-names: standing rules

Every task of this plan names this note, and `notes/contracts.md` beside it.

- Read `AGENTS.md` and the `AGENTS.md` of the tree your task works in.
- **The lab's names** are the ones `tests/boundary/tests/outward.rs` counts (its module doc and
  its `FIXED` list): a build tool's command form (`heavy cargo`, `heavy env`, `heavy --`, a quoted
  `"heavy"` or `'heavy'`), machine names (`hel8`, `hetzner` followed by digits, `debian-16gb`),
  `/home/free/`, `~/src/fabrig/`, `~/src/rust/tmp/`, the pipeline's name (`fab`, `FAB_HOME`),
  `status.fabrig.ai`, the agents repository and its paths. A file your task cleans ends holding
  none of them. Write no new one anywhere, nor a bare `heavy` meaning the build tool, nor "the
  queue", "the director", "a slot" or a machine's name: those are the lab's too, though the check
  does not count them.
- **The wording** for what replaces each name is `notes/contracts.md`, 3. Every tree says it the
  same way.
- **Behaviour stays as it is** (`inputs/request.md`, Bounds), except the scripts' build wrapper
  (`notes/contracts.md`, 1 and 2). A test hint, a panic message, a comment or a README line
  changes its words only; a test asserts what it asserted before.
- **Dated files keep their findings.** A review under `*/docs/development/debt/reviews/`, a debt
  item, a dated benchmark or measurement report changes only the lab's names in it: a sentence
  keeps its claim, its date, its commit and its counts.
- **Not this plan's, never edited by a task:** `tests/boundary/outward.txt` (the last task,
  `outward-inventory`, alone writes it: parallel removals of neighbouring lines conflict in git);
  `tests/boundary/tests/outward.rs`; anything under `tools/tiers/` (a parallel step);
  `temper/docs/development/qa.md`, `v0.1.0-ladder.md` and `reference-walks.md`;
  `temper/docs/design/draft/notes.md`, `reference-case.execution.md`, `configuring.execution.md`
  and `dashboards.md` (later steps); anything under a `docs/plan/` directory; any `docs/design/`
  file except where a spec names one and says why.
- **Checking your files are clean.** The check reads the working tree, so you check before you
  commit the inventory you never commit: delete your task's entries (the spec lists them) from
  `tests/boundary/outward.txt` in the working tree only, run
  `heavy cargo nextest run -p fabrig-boundary --test outward -E 'test(=no_file_names_the_lab_beyond_the_inventory)'`
  (1 selected; it passes only if each of those files holds no name), then restore the inventory
  with `git checkout -- tests/boundary/outward.txt`. `git status` shows it unchanged before you
  commit. A failure prints each remaining name with its file and line.
- **Citations:** after committing, `python3 -I -B tools/citations/citations.py --all` from the
  repository root prints `citations: 0 broken in HEAD`. A broken citation your branch did not
  add goes under `## Left`.
- **Old forms:** `python3 -I -B <tree>/scripts/old_forms.py` for your tree; no count rises.
- **Shell scripts** you change pass `sh -n` (or `bash -n` for a bash script) and, where
  `shellcheck` is installed, `shellcheck` with no new finding; say in the report whether it was.
- Every build and test through `heavy`, one command at a time (`AGENTS.md`, 5); the Python checks
  and `sh -n` run directly. The commands you show in fabrig's files are the plain ones.
- One commit per coherent step, each saying why. Push your branch.
