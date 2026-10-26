# Session 01: Forgejo 17.0.0 and forgejo-runner 13.2.0 pinned; the conformance's and git's transcripts and facts recorded on them

temper moves its one pinned Forgejo from 16.0.5 to 17.0.0 and its
runner from 12.13.2 to 13.2.0. Both pins become constants in
`testing/temper-throwaway-forgejo/src/binary.rs`, `VERSION` and
`RUNNER_VERSION`, and each binary's first `--version` line is checked
against its constant there: the throwaway instance, both recording tools
and the process tier share the one check. The conformance tool then
records its plain and component modes again on v17. The codecs world's
literals follow the fresh transcripts, and the process tier runs the
real `temper` binary on v17 once. In 01.2, `tools/git-transcripts`
records git again on v17. Its `facts.txt` header names Forgejo's version
and digest, and the redirect's fact reads a recorded fetch of the
renamed repository's old path, appended as the last case,
`fetch-renamed`. No decoder, component or fake changes in this session.

## Read first

- `docs/design/forge.md`, sections 1 (the last bullet, "Facts are
  checked against a real Forgejo v17.0"), 10 ("Transcripts are
  format-faithful", "The real Forgejo check", "The process tier") and 13
  ("Observed on Forgejo v17.0", and the git item of "To check on
  v17.0"); `docs/design/git.md`, sections 4.8 (the push's table), 9
  ("Transcripts", "The fake git program") and 10 ("A renamed
  repository's remote"); `docs/design/domain/testing.md`, 2.6 ("The throwaway
  Forgejo", "No CI there", "Outside the suites").
- The plan: `decisions.md` 2, 5 and 10; `code-organization.md` 2
  ("Versions"), 5 and 6; `findings/tools.md`, sections 1 to 4, read
  in full (149 lines): the pin's places, the two tools' commands and masking, the
  codecs world's literals, git's readers.
- skein's `docs/foundation/testing-strategy.md`, sections 4.1 (peers'
  transcripts, "captured from real clients and servers"), 2.9 (end to
  end: the process tier) and 8 (the two suites, which the recordings
  stay outside). `programming-model.md`: none; the tools and the
  throwaway crate are ordinary programs outside the loop.
- Code: `testing/temper-throwaway-forgejo/src/binary.rs:8-47`;
  `tools/forgejo-conformance/src/lib.rs:17-42`, `runner.rs:190-197`,
  `component.rs:16-23`; `tools/git-transcripts/src/lib.rs:72-93`,
  `cases.rs:150-198`; `tests/codecs/src/forgejo.rs:24-45,379-388`.

## What the design asks

- **One pinned release** (forge.md, 1): "temper requires that release:
  one patch release of it is pinned, and no other version is served."
- **The check and the digest** (forge.md, 10): the tool "refuses a
  binary whose `--version` is not the pinned release, naming it; it
  records the binary's digest with what it captured. The pin is one
  patch release of v17.0, a constant the throwaway Forgejo and the tool
  share; moving it to a later patch is a change of that constant and of
  the transcripts and facts the tool records, with no change to this
  document." The runner: "`forgejo-runner` v13.2, the program
  `TEMPER_FORGEJO_RUNNER` names, pinned as Forgejo is and refused at
  another version, registered with the throwaway instance alone; without
  it the tool reports those facts unchecked."
- **What a re-recording may change** (forge.md, 10): "Ids, times, counts
  and text vary; keys, nesting, types and status lines must match."
- **The throwaway Forgejo** (domain/testing.md, 2.6): "v17.0, the binary
  `TEMPER_FORGEJO` names ... It refuses to run without `TEMPER_FORGEJO`,
  or with a binary other than the pinned patch release (forge.md,
  section 10), naming it. ... `tools/forgejo-conformance` and
  `tools/git-transcripts` start theirs the same way, from the same
  variable."
- **The process tier** (forge.md, 10): "the real binary against the
  throwaway Forgejo, a repository adopted and a change landed, the forge
  component's start and polling among what it runs."
- **git's facts** (forge.md, 13, "To check"): "by `tools/git-transcripts`
  (git.md, section 9), for the git component: that git follows Forgejo's
  redirect for a renamed repository's old path (git.md, section 10), and
  that a push a protected branch refuses prints a `[remote rejected]`
  whose reason ends in `declined` (git.md, 4.8)".
- **git's transcripts** (git.md, 9): "Each keeps the argument vector,
  exit status, stdout and stderr, masked: object ids, ports, paths and
  times. No credential is ever written." git.md, 10: "Default: git
  follows the forge's redirect for its first request (git's
  `http.followRedirects`, whose default is `initial`) ... Whether
  Forgejo v17.0 redirects git's requests for a renamed repository is
  checked by `tools/git-transcripts`."

## Choices this plan makes

- **The two checks** live beside the constants, in `binary.rs`
  (decisions.md, 5). A Forgejo line names the pin when it is `forgejo
  version `, then `VERSION`, then nothing, `+` or a space. 17.0.0
  prints `forgejo version 17.0.0+gitea-1.22.0 (release name 17.0.0)
  built with GNU Make 4.4.1, go1.26.9 : bindata, timetzdata, sqlite,
  sqlite_unlock_notify`, whose build part varies by build. A runner line
  names the pin when it equals `forgejo-runner version v` followed by
  `RUNNER_VERSION`; 13.2.0 prints `forgejo-runner version v13.2.0`. Both
  lines were printed by the two binaries on x1 on 2026-10-16. The
  base's runner check, `contains("12.13.2")`, accepts `v112.13.2` and
  `v12.13.20`; the new one refuses them.
- **Prose names no number.** Doc comments say "the pinned Forgejo"
  (code-organization.md, 2). The two tools' READMEs name the constants
  and their file, and write the binaries in their commands as
  `<absolute path to the pinned Forgejo>`, so that the next pin changes
  only the constants, the transcripts and the facts (forge.md, 10).
- **Tests that need the pin's line build it from `VERSION` or
  `RUNNER_VERSION`** with `format!`. These lines are inputs. The
  expected values stay literals (`true`, `false`, a `Refusal`). The
  refusal test's `15.0.0` stays: it is a refused version, not the pin.
- **git's `facts.txt` has two header lines:** `# forgejo <Forgejo's
  first --version line> sha256 <digest>`, as
  `tools/forgejo-conformance/src/lib.rs:35` writes its own, then `#
  <git's version line>`, then the two facts.
- **The redirect's fact reads the recorded case,** as the protected
  push's fact reads `push-protected/stdout` (`cases.rs:183`). It holds
  when `transcripts/fetch-renamed/exit` is `0` and a line of its
  `stderr` starts with `warning: redirecting to ` and holds
  `/fixture/renamed`. The name is checked, not the path's end: whether
  Forgejo's redirect names `renamed.git/` or `renamed/` is not recorded
  yet.
- **git's wording, checked by a run.** On 2026-10-16, x1's git 2.53.0
  (`/usr/bin/git`) ran with `LC_ALL=C`, `protocol.version=2` and the
  tool's fetch arguments (`fetch --porcelain --verbose --no-tags
  --no-write-fetch-head --no-recurse-submodules`). It fetched from a
  loopback HTTP server that answered `301` for the old path, exited 0,
  and wrote to stderr `warning: redirecting to
  http://127.0.0.1:40417/new.git/`. Its porcelain line went to stdout.
  The format string `redirecting to %s` is in
  `/usr/lib/git-core/git-remote-http`. git's sources were not read.
- **The case is named `fetch-renamed`** and recorded last. Masking
  numbers ids by first appearance (`findings/tools.md`, 4), so the 41
  cases before it keep their ids.

## Rules that bind it

- **The recording tools run only where a Shape or Proof below says,**
  once. They run once more after each fix aimed
  at a cause their output names. The third failure with the same
  symptom is a question (development.md, 5.5).  The conformance's two
  modes run in one `sh -c '…'`. Transcripts, `facts.txt`
  and `binary.txt` are written by the tools only, never by hand. A
  `fails` line still exits 0: read `facts.txt` after the run (README,
  Rules).
- **Only these two binaries** (README, Rules, "The binaries"):
  `TEMPER_FORGEJO=$HOME/src/rust/tmp/temper/forgejo/v17.0.0/forgejo-17.0.0-linux-amd64`
  and
  `TEMPER_FORGEJO_RUNNER=$HOME/src/rust/tmp/temper/forgejo/runner-v13.2.0/forgejo-runner-13.2.0-linux-amd64`.
  None is downloaded.
- **No version in code but `VERSION` and `RUNNER_VERSION`; a
  transcript's literal is read from the transcript, by hand, never
  from the decoder** (code-organization.md, 5). Checked by "Done when",
  items 2 and 3.

## Increments

### 01.1 The pin, and the conformance recorded on v17

Scope: `VERSION` 17.0.0 and `RUNNER_VERSION` 13.2.0, each checked
against its binary's first `--version` line by one function in the
throwaway crate. The conformance tool's runner check uses
`RUNNER_VERSION`. The fixtures, doc comments and READMEs of
`findings/tools.md`, 1 follow. The conformance's plain and component
modes are recorded again on v17, and the codecs world's literals follow
them (forge.md, 1, 10 and 13; domain/testing.md, 2.6).
Needs: nothing.
Files:
- `testing/temper-throwaway-forgejo/src/binary.rs`: `:8` (`VERSION`),
  `RUNNER_VERSION` beside it, `names_forgejo` and `names_runner`, and
  `:37-38`, where the literal check becomes
  `names_forgejo(&version)`. `:5` of `src/lib.rs` exports
  `RUNNER_VERSION` and `names_runner`.
- `testing/temper-throwaway-forgejo/src/tests.rs`: the new unit tests.
  `tests/throwaway.rs:25` (the fake script prints the pin's line from
  `VERSION`) and `:36` (the message becomes "TEMPER_FORGEJO names the
  pinned Forgejo").
- `tools/forgejo-conformance/src/runner.rs:193-197`: the first stdout
  line, checked by `temper_throwaway_forgejo::names_runner`. Its error
  text stays `runner <path>: <line>`.
- `tools/forgejo-conformance/src/tests.rs:93-94`: the sample header
  becomes `# forgejo pinned\n# runner none\n` in both strings.
- Messages and doc comments naming a version:
  `crates/temper/tests/support/forgejo.rs:17` ("TEMPER_FORGEJO must name
  the pinned Forgejo");
  `crates/temper-engine-domain-forge-client/src/boundary.rs:29`;
  `crates/temper-engine-protocol-forge/src/settings.rs:33`;
  `crates/temper-forge-forgejo/src/lib.rs:1` and `src/failure.rs:1`;
  `testing/temper-fake-forge-domain/src/api.rs:92` ("Forgejo 15 ignores
  these page arguments" becomes "Forgejo ignores …", the observed fact
  of forge.md, 13; `findings/tools.md` misses this line) and `:104`.
- READMEs: `tools/forgejo-conformance/README.md:3,8,30` and
  `tools/git-transcripts/README.md:6`.
- Written by the tool: `tools/forgejo-conformance/facts.txt` and
  `transcripts/**` (`binary.txt`, and the 83 case directories it
  rewrites in place).
- The codecs world's literals that moved: `tests/codecs/tests/forgejo_files.rs`,
  `forgejo_items.rs`, `forgejo_reviews.rs`, `forgejo_writes.rs`.

Shared hotspots: `tools/forgejo-conformance/facts.txt`, `transcripts/`
and `tests/codecs/tests/forgejo_files.rs`, which temper 02 appends to
once this lands.
Shape: the sketch below. `Forgejo::from_path` keeps its signature and
its `Refusal::Version { path, printed }`. `start` in `runner.rs` takes
`String::from_utf8_lossy(&output.stdout).lines().next().unwrap_or_default()`
in place of the trimmed stdout, and `facts.txt`'s `# runner` line
carries that line, as at the base. The order of work: (1) commit the
code and its tests; (2) record, as the Proof's first command;
(3) read `facts.txt` (Proof); (4) run the codecs world in the build
lane, `cargo nextest run -p temper-codecs-world --no-fail-fast`,
after each round of literal fixes; (5) commit the recordings and the
literals; (6) run the process tier.
Test shape:
- `testing/temper-throwaway-forgejo/src/tests.rs`, unit, each under 20
  ms, importing `crate::{RUNNER_VERSION, VERSION, binary::names_forgejo,
  names_runner}`:
  - `forgejo_s_line_with_its_build_details_names_the_pin`: true for
    `forgejo version {VERSION}+gitea-1.22.0 (release name {VERSION})
    built with GNU Make 4.4.1, go1.26.9 : bindata, timetzdata, sqlite,
    sqlite_unlock_notify` and for `forgejo version {VERSION}`;
  - `a_forgejo_version_extending_the_pin_is_refused`: false for
    `forgejo version 1{VERSION}+gitea-1.22.0`, `forgejo version
    {VERSION}0+gitea-1.22.0`, `forgejo version {VERSION}-rc1` and
    `gitea version {VERSION}+gitea-1.22.0`;
  - `the_runner_s_version_line_names_the_runner_pin`: true for
    `forgejo-runner version v{RUNNER_VERSION}`;
  - `a_runner_version_containing_the_pin_is_refused`, the reference:

    ```rust
    #[test]
    fn a_runner_version_containing_the_pin_is_refused() {
        for line in [
            format!("forgejo-runner version v1{RUNNER_VERSION}"),
            format!("forgejo-runner version v{RUNNER_VERSION}0"),
            format!("forgejo-runner version v{RUNNER_VERSION}-dev"),
            format!("forgejo-runner version {RUNNER_VERSION}"),
        ] {
            assert!(!names_runner(&line), "{line}");
        }
    }
    ```
- `testing/temper-throwaway-forgejo/tests/throwaway.rs`:
  `a_scratch_directory_that_is_not_empty_is_refused` keeps its name and
  assertions. Its script prints `format!("#!/bin/sh\nprintf 'forgejo
  version {VERSION}+gitea-1.22.0 (release name {VERSION}) built with GNU
  Make 4.4.1, go1.26.9\\n'\n")`, so `from_path` accepts the v17 line's
  shape. `a_binary_printing_another_version_is_refused_naming_it`
  stays as it is.
- The codecs world (`tests/codecs/tests/`): no test is added or
  renamed. Each literal a fresh transcript moved is read from that
  transcript by hand. The spike's re-recording moved 5 tests' literals,
  at the lines `findings/tools.md`, 3 lists (`forgejo_files.rs:58,87,94`;
  `forgejo_items.rs:26,35,77,82,98,99,105,106,126`;
  `forgejo_reviews.rs:20-21,86`; `forgejo_writes.rs:28,72,244,250`).
  Masking numbers ids and times afresh on each run, so this run's
  values are the ones to write (`sha(n)` is masked id n; a masked time
  `…00:00:0n+05:30` is `Wall` 1_577_817_00n s). Their ceilings stay:
  focused under 0.5 s.

Proof: process layer, the real Forgejo 17.0.0 and runner 13.2.0. Each
command runs once, from the worktree's root:
1. The recordings, after the code's commit:

   ```sh
   TEMPER_FORGEJO=$HOME/src/rust/tmp/temper/forgejo/v17.0.0/forgejo-17.0.0-linux-amd64 \
   TEMPER_FORGEJO_RUNNER=$HOME/src/rust/tmp/temper/forgejo/runner-v13.2.0/forgejo-runner-13.2.0-linux-amd64 \
   sh -c 'cargo run -p forgejo-conformance -- --scratch target/forgejo-conformance --out tools/forgejo-conformance && cargo run -p forgejo-conformance -- --scratch target/forgejo-component --out tools/forgejo-conformance --component'
   ```

   It exits 0. What counts, read after it:
   - `head -2 tools/forgejo-conformance/facts.txt` prints `# forgejo
     forgejo version 17.0.0+gitea-1.22.0 (release name 17.0.0) …
     sha256 5a3b07e124ed70b4e46ee3939515f003bd3cc85352257a9a4607a527251042ca`
     and `# runner forgejo-runner version v13.2.0`;
   - `grep -c '^holds ' tools/forgejo-conformance/facts.txt` prints
     97, and `grep -c '^holds component-' …` prints 39;
   - `grep -E '^(fails|unchecked) ' tools/forgejo-conformance/facts.txt
     | cut -d: -f1` prints `fails pull-for-names-the-newest-of-several`,
     `fails open-pull-no-changes-is-422`, `fails
     delete-missing-branch-is-404`, `fails
     effective-protection-name-for-a-writer` and `unchecked
     webhook-deliveries`, and nothing else;
   - `git status --short tools/forgejo-conformance/transcripts | grep
     -c '^??'` prints 0 (no case added or renamed).

   Bound: 10 minutes for the task, the build included; the spike's
   recordings took 38 s and 24 s.
2. The process tier, after the recordings and the literals are
   committed:

   ```sh
   TEMPER_FORGEJO=$HOME/src/rust/tmp/temper/forgejo/v17.0.0/forgejo-17.0.0-linux-amd64 sh -c 'export NEXTEST_DOUBLE_SPAWN=0; cargo build -p temper --bin temper && cargo nextest run -p temper --profile process --no-fail-fast -E "not test(/^temper_open_/)"'
   ```

   It exits 0, and its log's nextest `Summary` line reads `4 tests
   run: 4 passed`. The two `temper open` tests are left out: on
   origin/main they panic at
   `crates/temper-client-shell-terminal/src/terminal.rs:72`, "print
   bounds", without this plan's changes (ruling 101; the designer's
   question of 2026-10-16T14:19:38Z). The full
   `scripts/proof-one-process.sh`, `6 tests run: 6 passed`, is the proof
   of the step that lands the fix. This shows the real `temper` binary
   on the throwaway Forgejo 17.0.0: it lands a change pushed by real
   git, found after a restart (domain/testing.md, 2.6). It aims at 3
   minutes (domain/testing.md, 2.6); the `process` profile's `global-timeout`
   stops it at 10 minutes (`.config/nextest.toml:32`). The landing's
   report gives the log's path.

Don't: edit a transcript, `facts.txt` or `binary.txt` by hand, or keep
a hand-made copy of one. Change no decoder, no message of
`crates/temper-forge-forgejo/src/failure.rs:32-42`, and no expected
value but the literals the fresh transcripts moved: a moved refusal or
a fact that changes outcome is a question (below). Add no capture or
fact (temper 02's). Do not touch `tools/git-transcripts`' recordings
(01.2's). Leave the runner's configuration (`runner.rs:203-218`) and
`Settings::small()` as they are.
Size: at most 40 lines of production code (binary.rs about 15, the
doc comments 6, runner.rs 2, the READMEs about 5).
Removes: the literal check of `binary.rs:38` and the runner's
`contains("12.13.2")` (`runner.rs:195`).

### 01.2 git's transcripts recorded on v17

Scope: `tools/git-transcripts` recorded again on Forgejo 17.0.0 (git.md,
9). Its `facts.txt` header names Forgejo's version and digest, as the
conformance's does. The redirect's fact reads a recorded fetch of the
renamed repository's old path, `fetch-renamed`, appended as the last
case (git.md, 10; forge.md, 13, the git item). The tests reading git's
transcripts follow any literal that moved, and the fetch parser's test
reads the new case (git.md, 9, "the parsers").
Needs: 01.1 (the pin: `Forgejo::from_env` accepts 17.0.0 and refuses
16.0.5, `findings/tools.md`, 1). It runs beside temper 02, which changes
no file of this increment.
Files:
- `tools/git-transcripts/src/lib.rs:92`: `cases::record(&forgejo,
  instance, &mut capture)`.
- `tools/git-transcripts/src/cases.rs`: `record` (`:150`) takes
  `&Forgejo`. `:182` becomes the recorded `fetch-renamed`. The fact's
  check is `followed`, and the header goes at `:187-194`.
- `tools/git-transcripts/src/tests.rs`: the new unit tests.
- `tools/git-transcripts/README.md:10` ("42 invocations") and `:14-15`
  (what `facts.txt` names).
- Written by the tool: `tools/git-transcripts/facts.txt` and
  `transcripts/**` (41 cases rewritten, `fetch-renamed` added).
- Readers that follow a moved literal (`findings/tools.md`, 4):
  `tests/worker/protocol-git/tests/transcripts.rs` (`FETCHES` gains
  the new row); `tests/worker/protocol-git/src/fixtures.rs:16-23`;
  `testing/temper-fake-git/tests/transcripts.rs:5-58`;
  `crates/temper-service/src/tests/fixtures/worker_git.rs:18` and
  `src/tests/worker_git.rs:47`.
Shape: the sketch below. In `record`, after the rename (`:181`), the
fetch of `url`, the old path `f.remote("specimen")` (`:173`), is
`fetch(c, Some("fetch-renamed"), &f.repo, &url, "refs/heads/main",
identity)?`, the run's last recorded case. The fact reads
`c.out.join("transcripts/fetch-renamed/exit")` and `…/stderr`. When
`followed` is false, its `fails` reason is `exit <exit, trimmed>;
<stderr, CR and LF as spaces>`. `facts.txt`:

```
# forgejo <forgejo.version()> sha256 <forgejo.sha256()>
# <git's version line, trimmed>
holds git-follows-a-renamed-repository-redirect
holds protected-push-is-remote-rejected-declined
```

Each fact line is `fails <name>: <reason>` when it does not hold, as
at the base. `c.scan("facts.txt", …)` (`:195`) still runs before the
write. If a fixture call of `Fixture::new` is refused on v17
(`fixture <target>: HTTP <status>`, `cases.rs:16`), the call follows
v17's API description (`~/src/rust/tmp/forgejo-17-swagger.json`; it
still lists `branch_name` beside `rule_name` for branch protections),
one change per cause the message names, counted in Size.
Test shape:
- `tools/git-transcripts/src/tests.rs`, unit, each under 20 ms, calling
  `crate::cases::followed`:
  - `a_renamed_fetch_warned_of_its_redirect_holds_the_fact`: true for
    exit `b"0\n"` and stderr `b"warning: redirecting to
    http://127.0.0.1:3000/fixture/renamed.git/\nPOST git-upload-pack
    (285 bytes)\n"`;
  - `a_renamed_fetch_without_git_s_redirect_warning_fails_the_fact`, the
    reference:

    ```rust
    #[test]
    fn a_renamed_fetch_without_git_s_redirect_warning_fails_the_fact() {
        assert!(!crate::cases::followed(b"0\n", b"POST git-upload-pack (285 bytes)\n"));
    }
    ```
  - `a_renamed_fetch_exiting_non_zero_fails_the_redirect_fact`: false
    for `b"128\n"` and the first test's stderr;
  - `a_redirect_to_the_old_name_fails_the_redirect_fact`: false for
    `b"0\n"` and `b"warning: redirecting to
    http://127.0.0.1:3000/fixture/specimen.git/\n"`.
- `tests/worker/protocol-git/tests/transcripts.rs:15-21`: `FETCHES`
  gains `("fetch-renamed", <the commit of the second id on its stdout's
  line>)`, read from `transcripts/fetch-renamed/stdout` by hand. Use
  `temper_worker_protocol_git_world::fixtures::SIX`
  (`tests/worker/protocol-git/src/fixtures.rs:20`) when that id is 6;
  another id gets a constant beside `SIX` there. Then
  `each_fetch_transcript_parses_to_its_fetched_commit` and
  `mutated_outputs_parse_to_none_never_to_another_value` read the case,
  with their names unchanged.
- The other readers keep their names. A literal a fresh transcript
  moved (masked ids, argv, push statuses, the classifier rows,
  `conflict.txt` paths) is read from that transcript by hand.
  `fetch-renamed` is not added to the fake git's `CASES`: the fake has
  no rename.

Proof: process layer, git 2.53.0 against the throwaway Forgejo 17.0.0.
Each command runs once, from the worktree's root:
1. The recording, after the code's commit:

   ```sh
   TEMPER_FORGEJO=$HOME/src/rust/tmp/temper/forgejo/v17.0.0/forgejo-17.0.0-linux-amd64 cargo run -p git-transcripts -- --git /usr/bin/git --scratch target/git-transcripts --out tools/git-transcripts
   ```

   It exits 0. What counts, read after it:
   `cat tools/git-transcripts/facts.txt` prints 4 lines. The first
   starts `# forgejo forgejo version 17.0.0+` and ends `sha256
   5a3b07e124ed70b4e46ee3939515f003bd3cc85352257a9a4607a527251042ca`.
   Then `# git version 2.53.0`, `holds
   git-follows-a-renamed-repository-redirect` and `holds
   protected-push-is-remote-rejected-declined`. Also `git status
   --short tools/git-transcripts/transcripts | grep '^??'` prints only
   `?? tools/git-transcripts/transcripts/fetch-renamed/`. Bound: 5
   minutes for the task, the build included.
2. The tests over the new transcripts, once, after the literals follow:
   `cargo nextest run -p temper-worker-protocol-git-world -p
   temper-fake-git -p temper-service` passes. the gate runs them
   again; nothing else is wanted. Each keeps its ceiling (unit under 20
   ms, focused world under 0.5 s).

Don't: insert a case before an existing one, or rename one. Change no
parser or classifier of `crates/temper-worker-protocol-git` to fit a
transcript: a moved classification is a question (below). Do not read
`facts.txt` from a test. Touch nothing of `tools/forgejo-conformance`.
Size: at most 40 lines of production code (cases.rs about 18, lib.rs
1, the README about 4).
Removes: the unrecorded fetch at `cases.rs:182` and the fact's test of
its exit alone (`:190-191`).

## Interface sketches

`temper_throwaway_forgejo` owns the pin and its checks. The
conformance tool calls `names_runner` and `Forgejo::from_env`, which
calls `names_forgejo`. `git-transcripts` calls `Forgejo::from_env`. The
bodies are written out: a check by prefix or substring is easy to write by mistake.

```rust
// testing/temper-throwaway-forgejo/src/binary.rs
/// The pinned Forgejo release: the throwaway instance, both tools and the process tier refuse another.
pub const VERSION: &str = "17.0.0";
/// The pinned forgejo-runner release: the conformance tool refuses another.
pub const RUNNER_VERSION: &str = "13.2.0";
/// Whether Forgejo's first `--version` line names `VERSION`: `forgejo version `, the version, then nothing, `+` or a space.
#[must_use]
pub(crate) fn names_forgejo(line: &str) -> bool {
    line.strip_prefix("forgejo version ").and_then(|rest| rest.split(['+', ' ']).next()) == Some(VERSION)
}
/// Whether forgejo-runner's first `--version` line is `forgejo-runner version v` and `RUNNER_VERSION`, nothing more.
#[must_use]
pub fn names_runner(line: &str) -> bool {
    line.strip_prefix("forgejo-runner version v") == Some(RUNNER_VERSION)
}
```

```rust
// tools/git-transcripts/src/cases.rs
use temper_throwaway_forgejo::Forgejo;
/// Records the cases, `fetch-renamed` last, then `facts.txt` headed by Forgejo's version line and digest.
pub(crate) fn record(forgejo: &Forgejo, instance: Instance, c: &mut Capture) -> Result<()>;
/// Whether the recorded `fetch-renamed` followed Forgejo's redirect: exit `0`, and git warned it was redirected to the new name.
pub(crate) fn followed(exit: &[u8], stderr: &[u8]) -> bool {
    exit == b"0\n"
        && stderr.split(|byte| *byte == b'\n').any(|line| {
            line.starts_with(b"warning: redirecting to ") && line.windows(16).any(|part| part == b"/fixture/renamed")
        })
}
```

## Questions to raise when reached

- **A conformance fact that changes outcome on v17.** This is a
  plain-mode `holds` at the base that fails, one of the four `fails`
  that holds, any `component-` line that is not `holds`, or a message of
  `failure::answer` (`crates/temper-forge-forgejo/src/failure.rs:32-42`)
  that moved. The design states each as "Observed on Forgejo v17.0 by
  `tools/forgejo-conformance`" (forge.md, 13), and the decoders and the
  component rest on them. Raise it with the line at the base and the
  line recorded, and land 01.1 only on the answer.
- **The redirect's fact fails.** `fetch-renamed` exits non-zero, or
  exits 0 with no `redirecting to` line, so Forgejo served the old path
  itself. git.md, 10's default rests on "git follows the forge's
  redirect for its first request". Raise it with the case's `exit` and
  `stderr`.
- **The protected push's fact fails, or a git transcript classifies
  otherwise than at the base** (a failure row of
  `tests/worker/protocol-git/tests/transcripts.rs:62-95`, or a push
  status of `:22-34`). git.md, 4.8 and section 6 rest on these
  classifications. Raise it with the transcript.

## Done when

Each command runs once, after 01.2 has landed, in `~/src/rust/temper`
after `git fetch origin`.

1. Both steps landed: `git log origin/main --grep='^Plan: forgejo-17$'
   --format='%(trailers:key=Step,valueonly,separator=)' | sort -u |
   grep -c -x -E 'temper-01\.[12]'` prints 2.
2. The pin: `git show
   origin/main:testing/temper-throwaway-forgejo/src/binary.rs | grep -c
   -E 'VERSION: &str = "(17\.0\.0|13\.2\.0)"'` prints 2; `git show
   origin/main:tools/forgejo-conformance/src/runner.rs | grep -c -F
   'names_runner('` prints 1.
3. No other version in code or the tools' prose: `git grep -n -e
   '16\.0\.5' -e '12\.13\.2' origin/main -- crates testing tools tests`
   prints nothing, and `git grep -n -E 'Forgejo v?[0-9]|forgejo-runner
   v?[0-9]|runner v[0-9]' origin/main -- crates testing tools tests
   ':!tools/*/transcripts/*' ':!tools/*/facts.txt'` prints nothing.
4. The conformance recorded on v17, read at 01.1's landing (temper 02
   appends facts after it): with `c=$(git log origin/main --all-match
   --grep='^Plan: forgejo-17$' --grep='^Step: temper-01.1$' --format=%H
   -1)`, `git show $c:tools/forgejo-conformance/facts.txt | head -2`
   names `forgejo version 17.0.0+` and `forgejo-runner version
   v13.2.0`. `git show $c:tools/forgejo-conformance/facts.txt | grep -c
   '^holds '` prints 97. `git show $c:tools/forgejo-conformance/facts.txt
   | grep -E '^(fails|unchecked) ' | cut -d: -f1` prints the five lines
   of 01.1's Proof, and nothing else.
5. git's facts on v17: `git show origin/main:tools/git-transcripts/facts.txt`
   prints 4 lines. The first starts `# forgejo forgejo version 17.0.0+`
   and ends `sha256
   5a3b07e124ed70b4e46ee3939515f003bd3cc85352257a9a4607a527251042ca`.
   The second is `# git version 2.53.0`, then `holds
   git-follows-a-renamed-repository-redirect` and `holds
   protected-push-is-remote-rejected-declined`.
6. The redirect is recorded: `git show
   origin/main:tools/git-transcripts/transcripts/fetch-renamed/exit`
   prints `0`, and `git show
   origin/main:tools/git-transcripts/transcripts/fetch-renamed/stderr |
   grep -c '^warning: redirecting to .*/fixture/renamed'` prints 1.
7. The process tier passed on v17: 01.1's landing report names the log
   of its process tier run (01.1's Proof, item 2, ruling 101), and `grep -c
   '4 tests run: 4 passed' <that log>` prints 1.
