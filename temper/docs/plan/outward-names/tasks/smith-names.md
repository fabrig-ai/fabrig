+++
id = "smith-names"
part = "smith"
kind = "docs"
needs = []
files = [
  "smith/benchmarks/README.md",
  "smith/benchmarks/tests/recorded/codex/**",
  "smith/benchmarks/src/arm.rs",
  "smith/benchmarks/src/fix.rs",
  "smith/docs/development/benchmarks/smith-codex-2026-10-14.md",
  "smith/docs/development/debt/reviews/2026-10-31-quality-areas.md",
  "smith/docs/development/debt/reviews/2026-10-31-quality.md",
]
effort = "high"
minutes = 50
+++

# smith's files stop naming the lab

## Goal

fabrig names nothing of whoever runs its development (`AGENTS.md`, 1). smith's benchmark README
shows the lab's build commands, lanes and machine; two reports and two reviews name its machine
or scratch checkout; seven recorded Codex files and their manifest hold a home directory. This
task says each generically and, where the recordings and their tests allow it, neutralises the
recorded home directory; behaviour stays as it is.

Acceptance:

- `smith/benchmarks/README.md`, `smith-codex-2026-10-14.md` and the two reviews hold none of the
  lab's names. The recordings hold none either (option A below), or, under option B, are left
  as they are and the report says why.
- The check of `notes/common.md` ("Checking your files are clean"), with the entries of
  `## Steps`, 4 deleted, passes (1 selected); the inventory is then restored and not committed.
- The tests of `## Steps`, 5 pass with counts above zero, among them the recordings' digest
  check (`smith/benchmarks/tests/suite/codex.rs`, the loop over `manifest["records"]`).
- `heavy cargo clippy -p smith-bench --all-targets -- -D warnings` passes;
  `python3 -I -B smith/scripts/old_forms.py` raises no count; `python3 -I -B
  tools/citations/citations.py --all` prints `citations: 0 broken in HEAD`.

## Inputs

- `temper/docs/plan/outward-names/notes/common.md`: the standing rules.
- `temper/docs/plan/outward-names/notes/contracts.md`, 3: the wording.
- `temper/docs/plan/outward-names/inputs/request.md`: the request; its "Recorded fixtures" item
  is the choice below.
- `tests/boundary/tests/outward.rs`: the check.
- `smith/benchmarks/README.md` and `smith/benchmarks/tests/recorded/**` are read by tests
  (`.config/gate/read-by-tests.txt`): `smith/benchmarks/src/credential/logins.rs` (~l.128)
  asserts the README holds `cat benchmarks/agents/smith/accounts.toml >>`; the suite includes the
  recordings by `include_str!`/`include_bytes!` (`tests/suite/codex.rs`, `codex_adapter.rs`,
  `events.rs`, `fuzzy/parsers.rs`) and checks each manifest record's `sha256` against its file
  with `smith_bench::arm::binary_digest`.

## Owns and may change

- **`smith/benchmarks/README.md`**, the lines that hold a name and the sentence that frames them
  (`notes/contracts.md`, 3):
  - ~l.7-16: "Build each named smith commit in the build lane before running it in the gate
    lane" becomes what it asks: build each named commit first; run probes, calibrations, fixes
    and suites on an otherwise idle machine, one at a time. The six commands lose `heavy ` and
    `heavy --gate `: `cargo run -p smith-bench -- build HEAD`, and so on.
  - ~l.167-168: the two `heavy --gate cargo run` commands → `cargo run`.
  - ~l.193: "on `debian-16gb-hel1-1`" → "on the benchmark machine".
  - ~l.199 and ~l.206: `heavy cargo vendor` → `cargo vendor`, `heavy cargo build` → `cargo build`.
  - Nothing else; the `cat benchmarks/agents/smith/accounts.toml >>` line stays.
- **`smith/docs/development/benchmarks/smith-codex-2026-10-14.md`** ~l.11: "on
  debian-16gb-hel1-1" → "on the benchmark machine"; the sentence keeps its claim.
- **The two reviews** under `smith/docs/development/debt/reviews/`: `2026-10-31-quality.md` ~l.5
  "a fresh clone at `~/src/fabrig/leads/smith-review/survey`" → "a fresh clone";
  `2026-10-31-quality-areas.md` ~l.9 "Tree: `~/src/fabrig/leads/smith-review/survey` (a clone of
  ..." → "Tree: a clone of ..." and ~l.13 "path under `~/src/fabrig/`" → "path outside that
  clone". Their findings stay.
- **`smith/benchmarks/src/arm.rs`** ~l.342 ("called only inside a `heavy` task") and
  **`smith/benchmarks/src/fix.rs`** ~l.2 ("an explicit heavy preparation step"): the doc
  comments say it generically ("called only inside a build step", "an explicit build step");
  the check does not count these, but they name the tool. Comment words only.
- **The recordings, `smith/benchmarks/tests/recorded/codex/`:** the choice the request leaves
  to this task, pre-approved:
  - **Option A (default):** replace every `/home/free/` with `/home/user/` (same length) in
    `children.jsonl`, `completed.jsonl`, `failed-rollout.jsonl`, `live-child-rollout.jsonl`,
    `live-root-rollout.jsonl`, `repeated-rollout.jsonl` and `manifest.json`, as a byte
    substitution (`sed -i 's#/home/free/#/home/user/#g'`), nothing else in those files. In
    `manifest.json`, set each changed file's `sha256` to its new digest (the digest
    `binary_digest` computes; `sha256sum` gives it if it is SHA-256 of the bytes: check
    `smith/benchmarks/src/arm.rs`); `source_sha256` stays (it pins the original), and each
    changed record's `change` text gains "; home directory in recorded paths written
    /home/user/". In `README.md` beside them, the sentence "Only records were removed; retained
    content is unchanged." gains: "except that the recording machine's home directory reads
    `/home/user/` in recorded paths; `source_sha256` pins each original". How you checked goes
    in the report: the tests of `## Steps`, 5, and a search showing no test or source reads
    `/home/free/` or `/home/user/` from these files.
  - **Option B:** if a test or the digest check cannot hold under A (a recorded byte count or
    digest something else pins, a parser that reads the path), restore the recordings as they
    were, change none of them, and say in the report which test or reader stopped A, with its
    file and line. The inventory's reason line is the last task's (`outward-inventory`); you
    never edit `tests/boundary/outward.txt`.
- **If a line moved** at your base, the name is changed where it stands; a name this spec does
  not list is changed by `notes/contracts.md`, 3, and the report says where.
- Nothing else: not `tests/boundary/outward.txt`, not `smith/docs/design/` (read by tests).

## Steps

1. The README, the report, the reviews and the two doc comments, as `## Owns and may change`.
2. The recordings: option A, then step 5's tests; on a failure that A causes, option B.
3. `git diff --stat` shows only `files`; under A, `git diff --word-diff` of each recording shows
   only `/home/free/` → `/home/user/` (and the manifest's `sha256` and `change` fields).
4. Check the files clean, as `notes/common.md` says, deleting these entries of
   `tests/boundary/outward.txt` in the working tree: `smith/benchmarks/README.md`,
   `smith/docs/development/benchmarks/smith-codex-2026-10-14.md`, the two reviews and, under
   option A, the seven `smith/benchmarks/tests/recorded/codex/` entries. Restore it.
5. Tests: `heavy cargo nextest run -p smith-bench --lib -E 'test(/logins/)'` (the README
   assertion), `heavy cargo nextest run -p smith-bench --test suite -E 'test(/codex/)'` (the
   digest check and the adapters that parse the recordings) and the fuzzy parsers over the
   recordings, `heavy cargo nextest run -p smith-bench --profile fuzzy --test suite -E
   'test(/parsers/)'`. Read each filter's selected count against `rg` of the test names; a
   filter that selects nothing is replaced by the right one and the report says so. If
   `events.rs`'s recording tests are not among them, add them.
6. `cargo fmt`; clippy as in Acceptance; old forms; commit; citations; push.

## Report

As the implementer's text says, and: which option, and how you checked (the tests, their
counts, the searches); each recording's old and new `sha256`; each prose line before and after;
the check of step 4 and its selected count; anything under `## Left`.
