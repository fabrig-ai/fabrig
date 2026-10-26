# The pin and the two recording tools, at the base

The code audit of temper at `118f7fb7` (`origin/main`), 2026-10-16, by
two read-only sub-agents, against the throwaway spike of the same day
(`~/src/rust/tmp/temper-forgejo-17-spike.md`, not versioned: the
retarget on 17.0.0 and runner 13.2.0 kept every fact, and broke five
codecs tests on re-recorded literals). Paths are temper's.

## 1. Where the pin lives

- `testing/temper-throwaway-forgejo/src/binary.rs:8`, `pub const
  VERSION: &str = "16.0.5"`, re-exported at `src/lib.rs:5`, used by
  nothing. The check at `binary.rs:38` repeats the literal (`"forgejo
  version 16.0.5"` and `"…16.0.5+"`). forge.md, section 10 asks for "a
  constant the throwaway Forgejo and the tool share": not met.
- The runner's pin is a looser substring check of its own,
  `tools/forgejo-conformance/src/runner.rs:195`, `contains("12.13.2")`.
- No digest is checked: `binary.rs:42-45` computes the SHA-256, which
  the tool records (`tools/forgejo-conformance/src/lib.rs:35,39`), as
  forge.md, section 10 asks.
- Fixtures: `testing/temper-throwaway-forgejo/tests/throwaway.rs:25`
  (a fake script printing the pin; it must follow it, or `:27` fails),
  `:36` (message text); `tools/forgejo-conformance/src/tests.rs:93-94`
  (a sample header); `crates/temper/tests/support/forgejo.rs:17`
  (message text).
- Doc comments: `crates/temper-engine-domain-forge-client/src/boundary.rs:29`,
  `crates/temper-engine-protocol-forge/src/settings.rs:33`,
  `crates/temper-forge-forgejo/src/lib.rs:1`, `failure.rs:1`,
  `testing/temper-fake-forge-domain/src/api.rs:104`.
- Prose: `tools/forgejo-conformance/README.md:3,8,30`,
  `tools/git-transcripts/README.md:6` (and `:10`, "41 invocations").
- Re-recording alone changes `tools/forgejo-conformance/facts.txt:1-2`,
  `transcripts/binary.txt:1-2`, `transcripts/actions-job-log/response:12`.
- `tools/git-transcripts/src/lib.rs:76` uses the same
  `Forgejo::from_env`: once the pin moves, it refuses v16.

## 2. `tools/forgejo-conformance`

- **Plain mode:** `TEMPER_FORGEJO=<17.0.0> TEMPER_FORGEJO_RUNNER=<13.2.0>
  cargo run -p forgejo-conformance -- --scratch
  target/forgejo-conformance --out tools/forgejo-conformance`; 38 s in
  the spike. It rewrites `facts.txt` whole (`lib.rs:33-36`).
- **Component mode:** the same with `--scratch target/forgejo-component
  … --component`; 24 s. It replaces only the `component-` lines
  (`component.rs:19-21`), so it runs after the plain mode.
- Scratch must be absent or empty, and not under `/tmp`
  (`testing/temper-throwaway-forgejo/src/instance.rs:248-256`).
- Without the runner, 6 Actions facts and 2 component facts are
  unchecked, and the six `actions-*` transcripts stay as they were.
- **A plain fact:** `facts::add(facts, name, holds, seen)`
  (`facts.rs:35-37`), inside `facts::evaluate` (`facts.rs:59-94`),
  reading a capture by `case(out, "<case>")` (`facts.rs:44-51`). The
  shape, `facts.rs:471`: `add(facts, "raw-range-answers-206", case(out,
  "raw-file-range")?.status == 206, "raw Range did not answer 206");`.
  Actions facts come from `runner.rs`'s `NAMES` (`:15-22`), added at
  `:107-135`; Actions captures at `:77-105`.
- **A component fact:** `Hosted::observe(name, repo, op, check)`
  (`component.rs:62-71`), pushed in `component/reads.rs` or `writes.rs`.
  The shape, `component/reads.rs:49-54`: `facts.push(h.observe(
  "component-read-branch", repo, Read::Branch { branch:
  bytes("feature/slash") }.operation(), |a| a ==
  &Answer::Commit(head)));`. A name must start with `component-`:
  `replace_block` (`facts.rs:503-526`) matches on it, and duplicates any
  other name each run. `writes.rs:97-98` inserts the cost-tally fact two
  places from the end.
- **Captures** are declared in `cases.rs` (`start` :54-63, `writes`
  :66-157, `reads` :167-199, `failures` :206-292), each through
  `Recorder::send` (:17-32): secrets checked, masked, written
  (`write.rs:65-75`) to `transcripts/<case>/{request,response}`.
- **Masking numbers ids and times by first appearance,** once per run
  (`mask.rs:49-50,68-69`): a capture inserted early renumbers everything
  after it, so new captures go last. Plain-text log timestamps and the
  runner's UUID are not masked (`mask.rs:39-47`). `write::case` replaces
  only its own directory: a renamed case leaves a stale one, which the
  fuzzy tests and the rate-limit scan (`facts.rs:80-92`) still read.
- The runner's configuration (`runner.rs:203-218`) works on v13
  unchanged (the spike, section 2).

## 3. The codecs world

- `transcript_cases()` (`tests/codecs/src/forgejo.rs:379-388`) lists
  every case directory, for `json_cases` and `fuzzy_forgejo.rs:6,23`. A
  new 2xx JSON case on a route the decoder does not route panics at
  `forgejo.rs:96-97`: `GET /actions/jobs/2` is not routed
  (`crates/temper-forge-forgejo/src/server/actions.rs:24-45`). So a new
  capture lands with its route, and the spike's 88 `v17-*` captures
  would have broken the fuzzy suite.
- A second reader, `tests/engine/protocol-forge/src/machine.rs:311-319`,
  loads named cases; it passed in the spike.
- **Literals a re-recording changes,** read from the spike's
  re-recorded transcripts (`sha(n)` is masked id n; a masked time
  `…00:00:0n+05:30` is `Wall` 1_577_817_00n s), all in
  `tests/codecs/tests/`:
  - `forgejo_files.rs` :58 `sha(10)`→`sha(11)`; :87 …011→…010; :94
    …012→…011;
  - `forgejo_items.rs` :26 `edited()` …005→…006; :35 `writer()`
    …008→…006; :77 `[writer(), feature()]`; :82 `[conflict(),
    writer()]`; :98, :99, :105, :106, :126 …005→…004;
  - `forgejo_reviews.rs` :20-21 …005→…004; :86 `branch(b"main",3)`→
    `branch(b"conflict",10)`;
  - `forgejo_writes.rs` :28 splits (create-issue and edit-issue-title
    …004, close-issue and reopen-issue …006); :72 …005→…004; :244
    `000c`→`000d` (twice); :250 `"IsErrIssueNotExist"`.

  A fresh recording masks afresh: the run's author reads the values
  from its own transcripts, as above, never from the decoder.
- Transcript headers use CRLF; `.gitattributes:1-6` does not cover
  them, and `git diff --check` flags them. Nothing in the gate runs it.

## 4. `tools/git-transcripts`

- **Command:** `TEMPER_FORGEJO=<binary> cargo run -p
  git-transcripts -- --git /usr/bin/git --scratch target/git-transcripts
  --out tools/git-transcripts` (`README.md:6`; `src/main.rs:7-20`).
  Forgejo serves with the recorded git (`lib.rs:79`); Actions is off
  (`lib.rs:78`). It overwrites cases and never removes one.
- **Its facts** (`facts.txt:1-3`): `# git version 2.53.0`, `holds
  git-follows-a-renamed-repository-redirect`, `holds
  protected-push-is-remote-rejected-declined`: both of forge.md,
  section 13's git facts exist and held on v16.0.5.
  - The redirect's fact holds when an unrecorded fetch of the old path
    exits 0 (`cases.rs:181-182,190`): nothing shows that a redirect was
    followed, and nothing is kept.
  - The header names git's version only (`cases.rs:188`): a v17 record
    cannot be told from a v16 one. The conformance tool's header names
    Forgejo's version and digest (`tools/forgejo-conformance/src/lib.rs:35`).
  - A `fails` line still exits 0 (`cases.rs:187-197`).
- **Readers:** `tests/worker/protocol-git/src/transcripts.rs:14` alone
  opens the directory. Literals: `tests/worker/protocol-git/tests/transcripts.rs`
  (masked ids :9-21, argv :41, push statuses :22-34, classifier rows
  :63-71 and :79-94, `version == 2.53.0` :100, `conflict.txt` paths
  :125-145); `tests/worker/protocol-git/src/fixtures.rs:16-23`;
  `testing/temper-fake-git/tests/transcripts.rs:5-58` (41 cases' exit
  and stdout equal to the fake's, `answer.rs:40,264,308`);
  `crates/temper-service/src/tests/fixtures/worker_git.rs:18`,
  `src/tests/worker_git.rs:47`. No time and no Forgejo version is a
  literal (`mask.rs:29-36`).
- **git's version is recorded, not pinned:** the tool takes 2.41 or
  later (`cases.rs:163`), the tests 2.53.0. x1 has Debian's git 2.53.0.
- Ids are numbered in first-seen order over the run (`mask.rs:66-72`):
  a case inserted before the pushes shifts them. New cases go after
  `cases.rs:182`.
- The fixture's API calls on v17 are unchecked: `branch_name` in branch
  protections (deprecated for `rule_name`, still accepted per the v17
  API description), contents writes, the rename, `object_format_name:
  sha256`, token scopes `["all"]`. A non-2xx aborts the run
  (`cases.rs:15-17`).
