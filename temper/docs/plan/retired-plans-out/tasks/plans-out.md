+++
id = "plans-out"
part = "plans-out"
kind = "docs"
needs = ["keep-decisions"]
files = [
  "jig/docs/plan/wasm32-jig/**",
  "skein/docs/plan/playground-skein/**",
  "skein/docs/plan/wasm32-skein/**",
  "smith/docs/plan/live-smith-manual/**",
  "smith/docs/plan/live-smith-reds/**",
  "smith/docs/plan/live-smith-reds-2/**",
  "smith/docs/plan/reference-m4/**",
  "temper/docs/plan/citation-burndown/**",
  "temper/docs/plan/collector-starved/**",
  "temper/docs/plan/configuring-p1/**",
  "temper/docs/plan/configuring-p2/**",
  "temper/docs/plan/configuring-p2-build/**",
  "temper/docs/plan/configuring-p3/**",
  "temper/docs/plan/fab3-fixes/**",
  "temper/docs/plan/first-run-projects/**",
  "temper/docs/plan/forgejo-connect/**",
  "temper/docs/plan/github-repair-race/**",
  "temper/docs/plan/interrupted-calls/**",
  "temper/docs/plan/live-temper/**",
  "temper/docs/plan/live-temper-stories/**",
  "temper/docs/plan/live-temper-stories-2/**",
  "temper/docs/plan/look-welcome-chats/**",
  "temper/docs/plan/me-retention/**",
  "temper/docs/plan/open-default/**",
  "temper/docs/plan/openai-manual/**",
  "temper/docs/plan/phase-case/**",
  "temper/docs/plan/playground-decisions/**",
  "temper/docs/plan/playground-temper/**",
  "temper/docs/plan/public-reading/**",
  "temper/docs/plan/readiness-eintr/**",
  "temper/docs/plan/reference-m1/**",
  "temper/docs/plan/reference-m2/**",
  "temper/docs/plan/reference-m3/**",
  "temper/docs/plan/reference-m5/**",
  "temper/docs/plan/reference-m6/**",
  "temper/docs/plan/reference-m7a/**",
  "temper/docs/plan/reference-m7b/**",
  "temper/docs/plan/reference-m8/**",
  "temper/docs/plan/reference-m9/**",
  "temper/docs/plan/reference-m10/**",
  "temper/docs/plan/repo-links/**",
  "temper/docs/plan/status-gaps/**",
  "temper/docs/plan/status-lines/**",
  "temper/docs/plan/wasm32-list/**",
  "temper/docs/plan/wasm32-more/**",
  "temper/docs/plan/wasm32-rustls/**",
  "temper/docs/plan/wasm32-temper/**",
  "temper/docs/plan/wasm32-temper-2/**",
  "temper/docs/plan/wasm32-temper-3/**",
]
effort = "medium"
minutes = 20
+++

# The 49 retired plan directories leave main

## Goal

The retired plans are no longer read from fabrig (their runner keeps its own copies), and what
only they recorded now stands in the design. This task deletes the 49 directories of
`notes/common.md` that exist at its base, and nothing else (the request's Task 2).

Acceptance:

- Each of the 49 present at your base is gone, every file under it; each absent one is named
  in the report. No other path changes: `git diff --stat <base>..HEAD` lists only files under
  the 49.
- Every other `docs/plan/` directory is still there (`## Steps`, 3).
- `python3 -I -B tools/citations/citations.py --all` prints `citations: 0 broken in HEAD`.
- `heavy cargo nextest run -p fabrig-boundary` passes: `outward`'s 10 tests and `direction`'s
  3, 13 in all at the cut. `outward.rs` leaves out every `docs/plan/` directory and stays as
  it is; so does `tests/boundary/outward.txt`.
- Each tree's old-forms count is unchanged: `python3 -I -B scripts/old_forms.py` from `skein/`,
  `smith/`, `jig/` and `temper/` exits 0, without `--write` (at the cut, deleting the 49 left
  every count as it was).
- One commit, pushed.

## Inputs

- `temper/docs/plan/retired-plans-out/notes/common.md`: the standing rules and the list of the 49.
- `temper/docs/plan/retired-plans-out/inputs/request.md`, "Task 2" and "Bounds".

## Owns and may change

- **The 49 directories,** deleted whole. Nothing else.
- **Departure from the implementer's text:** no fmt, clippy or package tests beyond the checks
  above; the branch deletes documentation only.
- **A directory absent at your base:** skip it; the report names it.
- **A check red because of the deletion** (a file outside `docs/plan/` citing a file under the
  49 that the earlier tasks did not rewrite): rewrite that citation as the fact it stood for,
  with its date or commit, under `notes/common.md`'s rules, in a commit of its own before the
  deletion's, and report it. If the file is outside `notes/common.md`'s allowed targets, or
  the fact is not plain from the cited file, stop: commit nothing of the deletion, and the
  report's first line is `Question:` with the file, the line and what it cites.

## Steps

1. **Delete,** from the repository root:

   ```sh
   list() {
     sed -n '/^## The 49 directories/,/^Every other/p' temper/docs/plan/retired-plans-out/notes/common.md \
       | grep -E '^(jig|skein|smith|temper)/docs/plan/'
   }
   list | wc -l    # 49
   list | while read -r d; do
     if [ -d "$d" ]; then git rm -r -q -- "$d"; else echo "absent: $d"; fi
   done
   ```

2. **Commit** (`retired-plans-out: the retired plans' directories leave main`, saying how many
   were present, that each plan's directory now leaves main when it retires, and naming any
   absent), before the checks: the citations checker reads committed trees.

3. **What remains.** `git ls-files '*/docs/plan/*' | cut -d/ -f1-4 | sort -u` lists only live
   plans: at the cut, `smith/docs/plan/live-smith-reds-3`, `smith/docs/plan/sign-in-wait`,
   `temper/docs/plan/outward-rule`, `temper/docs/plan/public-reading-build`,
   `temper/docs/plan/reference-walks`, `temper/docs/plan/retired-plans-out`,
   `temper/docs/plan/v0.1.0-ladder`, `temper/docs/plan/withdrawn-forms`, and any plan started
   since (none of the 49).

4. **The checks** of `## Goal`, then the search of `tasks/citations-out.md`, `## Steps`, 10: it
   prints only the lines that spec lists as staying (and none of those it rewrote).

5. **Push.**

## Report

`Done:` with the number deleted and the files removed. Then any absent directory; the list of
`## Steps`, 3; each check's result (citations' line, the boundary tests' count, the four
old-forms exits); the search's output; your minutes.
