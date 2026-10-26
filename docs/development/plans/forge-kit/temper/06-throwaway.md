# Session 06: temper on skein's throwaway Forgejo

Put temper's process tier, its git transcripts tool and (until temper 07
deletes it) its conformance tool on skein's `testing/skein-throwaway-forgejo`,
then delete temper's copy, `testing/temper-throwaway-forgejo`. The tier keeps
its own variables, its binaries and every value it asserts; the kit reads
no environment, and its `attach` names no login. The conformance also goes on
skein-json (decision 27), no parallel value tree: the `json` module and the
byte-level leaves (06.2), the reads (06.3), the writes (06.4); findings
`base-temper-03.md` hold their base runs.

**Needs at start:** skein 06.1 (`skein_throwaway_forgejo` on origin/main, with
`Forgejo::from_path` and `Instance::attach(child, port, scratch, admin,
&Settings)` reading the admin's login and id from the forge). The sessions
table's order has temper 01.2 landed before this session: its pin imports
(`skein_forge::forgejo::pin`) are already in the files this session edits.
Each capability must be on origin/main.
06.2 to 06.4 need nothing beyond 06.1: `skein-json` is a workspace dependency,
and `serde_json` stays one (five other packages once 06.1 deletes temper's throwaway). Skein 06.2 copies the
conformance after temper 03.1 and 06.4; 03.1 changes the conformance's manifest
line, `component.rs` and `component/hosted.rs`, and may land before or after
06.2 to 06.4, beside them: a rebase meets it.

## Read first

- skein `docs/design/fake-forge.md` 6; temper `docs/design/domain/testing.md` 2.6
  and `protocol/git.md` 9 (the transcripts tool).
- skein `docs/foundation/testing-strategy.md` 2.1, 8, 10 and
  `development.md` 5.5-5.7.
- README (sessions, Rules, Live services, Done when); code-organization 1, 6,
  7; decisions 27, Design questions 30 and 31;

- `skein/06-conformance.md` 06.1 ("temper's side", its Removes; 06.2 copies the
  conformance, 06.4 is the stable-fact check) and `temper/01-forge.md` 01.2;
  findings `survey-05-fakes.md`, `trace-fakes.md`, `base-runs.md`,
  `base-temper-06.md`, `base-temper-03.md`.

## What the design asks

- fake-forge 6: `Forgejo::from_path` reads no environment; `Instance::attach`
  takes a Forgejo its caller started and reads the admin's login and id with
  `GET /api/v1/user`, naming no login of its own; its `stop` ends the child
  and leaves the caller's scratch.
- testing 2.6: temper's tests give the throwaway the binaries
  `TEMPER_FORGEJO` and `TEMPER_FORGEJO_RUNNER` name and refuse to run without
  `TEMPER_FORGEJO`, naming it; `tools/git-transcripts` starts its Forgejo with
  the throwaway; the process tier is `scripts/proof-one-process.sh`.
- Decision 27: the conformance reads and writes with skein-json's `Document`
  and `writer::Encoder`, never re-serializing a captured body.

## Choices this plan makes

Counts and Files below describe the base, dd12b8e; this step edits origin/main
as it stands at the step's start, where temper 01.2 has repointed the pin lines.

- The process tier hands the throwaway no path and no login. Its two
  variables feed temper's own `forge prepare` and `forge reconcile`
  (`owned::binaries`, `prepared_processes`, `reconcile_configuration`);
  `owned::attach(directory, port, running)` keeps its arguments and its token
  file. The change in `support/` is the crate path; the login now comes from the
  API (skein 06.1, "temper's side").
- The pin's names come from `skein_forge::forgejo::pin` since temper 01.2,
  `registration.rs`'s `RUNNER_VERSION` and `owned.rs`'s included; nothing in
  temper uses the kit's designed re-export of them (skein 06.1, "temper's side").
- domain/testing.md 2.6's refusal without `TEMPER_FORGEJO` is the process tier's own:
  `owned.rs`'s `expect`s naming the variable (in `prepared_processes`,
  `reconcile_configuration` and `binaries`) stay unchanged and keep meeting it; this step adds the tools' refusal.
- `tools/git-transcripts` reads `TEMPER_FORGEJO` itself and gives the path to
  `Forgejo::from_path`, naming the variable when it is unset.
- `tools/forgejo-conformance` is temper 07's to delete, but it builds in
  between and imports the deleted crate in 18 files: this step repoints it
  and gives it the same two lines. It keeps its own `TEMPER_FORGEJO_RUNNER`
  reads. The README's row for temper 06 names this repoint.
- The committed transcripts and facts are not re-recorded. Their git (2.53.0)
  is not this machine's (2.47.3); the Proof compares two recordings made on
  one machine.
- **Size of the conformance:** its rewritten `json!` and read sites are new
  code in substance (about 450 lines on top of the module's: 242 production and 57 test lines in the
  scratch, findings/base-temper-03.md, which 06.2 to 06.4's Sizes round up to about 290 and 97). 06.3 counts
  them as moved in place, as 02.1 counted callers whose paths changed, and says
  so; counted as new they pass the aim (06.3, Size).

## Rules that bind it

- The increment's **first commit** is its repin: `cargo update -p
  skein-lib`, then `cargo check --workspace --all-targets`. Repair in
  that commit what skein's steps since temper's last repin broke. A lock
  conflict takes origin/main's Cargo.lock and repeats the update.
- No alias, no `pub use` of a kit item, no second copy: every `use` names
  `skein_throwaway_forgejo`, or `skein_forge::forgejo::pin` for the pin.
- Hotspot: Cargo.toml and Cargo.lock (every temper session). Test builds stay
  unoptimized; the process tier runs once, never in the gate, with the pinned
  binaries README names (`$WS` as the README's Rules define it).
- 06.2 to 06.4: no behaviour change, no raised ordinary writer or production bound, no loosened assertion;
  only06.4's explicitly bounded existing large-fact requests use their own output room;
  moved tests keep names; units under 20 ms, focused worlds under 0.5 s. Proofs
  state gate counts as deltas from the step's start; the gate lane takes a
  committed tree only, so recordings run committed. Don't rewrite a moved part,
  raise a budget, filter, ignore or split a test, or edit a transcript, a
  delivery or `facts.txt`.

## Increments

### 06.1 The repin, the process tier and the tools on the throwaway

Scope: fake-forge 6, testing 2.6; repin, repoint the users, delete temper's copy.
Needs: skein 06.1. (temper 01.2 has repointed the pin lines; temper 07, which
  needs this step, deletes the conformance, so it is repointed here.)
Files (changed; at the base):
- `Cargo.lock`: the repin commit, then the entries `cargo check` writes in the swap
  (the kit's package in, the copy's out).
- `Cargo.toml`: delete member `"testing/temper-throwaway-forgejo"`; the
  dependency line becomes `skein-throwaway-forgejo = { git =
  "https://github.com/fabrig-dev/skein.git" }`.
- `crates/temper/Cargo.toml` (dev-dependencies), `tools/git-transcripts/Cargo.toml`,
  `tools/forgejo-conformance/Cargo.toml`: `temper-throwaway-forgejo` becomes
  `skein-throwaway-forgejo` (one line each).
- `crates/temper/tests/support/owned.rs` (`attach`, `assert_converged`: the
  crate path, 4 lines besides the pin import), `support/forgejo.rs` (the `use`
  and `Exchange` in `get`, 2 lines); `process.rs` has only 01.2's pin lines.
- `tools/git-transcripts/src/{lib,cases}.rs`: imports and `run`'s start
  (temper 01.2 has already repointed both READMEs' pin sentence).
- `tools/forgejo-conformance/src/*.rs`: 18 files, 30 lines at the base (the
  crate path), `lib.rs` (the helper) and `component.rs::record` (calls it);
  also `src/component/hosted.rs::Hosted::new`, whose `Token` parameter becomes
  `skein_throwaway_forgejo::Token`. Nothing in its README.
- New: `tools/git-transcripts/tests/refusal.rs`,
  `tools/forgejo-conformance/tests/refusal.rs`.
- Deleted: `testing/temper-throwaway-forgejo/` (Removes).
Shape (compiled at the base against temper's crate, which has the same
  `from_path`; the swap of the crate name is mechanical):
```rust
// tools/git-transcripts/src/lib.rs, in run(git, scratch, out), for from_env:
let binary = std::env::var_os("TEMPER_FORGEJO").ok_or("TEMPER_FORGEJO must name the pinned Forgejo")?;
let forgejo = Forgejo::from_path(Path::new(&binary)).map_err(|e| format!("{e:?}"))?;
// tools/forgejo-conformance/src/lib.rs; run and component::record call it:
pub(crate) fn forgejo() -> Result<Forgejo> {
    let binary = std::env::var_os("TEMPER_FORGEJO").ok_or("TEMPER_FORGEJO must name the pinned Forgejo")?;
    Forgejo::from_path(Path::new(&binary)).map_err(|e| format!("{e:?}").into())
}
// crates/temper/tests/support/owned.rs: the signature and body stay,
pub fn attach(directory: &Path, port: u16, running: Running) -> skein_throwaway_forgejo::Instance;
```
  Nothing else is handed: no word, user agent or login (`temper-admin` was
  the copy's own); `Settings::small()` is the kit's. A repin that breaks a
  name (for example a kit field) is repaired in the repin commit.
Test shape: two new tests, one per tool, the same name
  `an_unset_forgejo_variable_is_refused_naming_it`. World: the built tool run as a
  process, `TEMPER_FORGEJO` removed from its environment, no Forgejo started
  (`env!("CARGO_BIN_EXE_git-transcripts")`, `"forgejo-conformance"`). Each
  asserts exit code `Some(1)` and stderr exactly `Error: "TEMPER_FORGEJO must
  name the pinned Forgejo"` and a newline. The reference test, in full (the
  conformance's is the same with its binary and without `--git`; both tools
  refuse a missing `--scratch` or `--out` before they read the variable):
```rust
// tools/git-transcripts/tests/refusal.rs
#[test]
fn an_unset_forgejo_variable_is_refused_naming_it() {
    let tmp = std::path::Path::new(env!("CARGO_TARGET_TMPDIR"));
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_git-transcripts"))
        .args(["--git", "/usr/bin/git", "--scratch"])
        .arg(tmp.join("refusal-scratch"))
        .arg("--out")
        .arg(tmp.join("refusal-out"))
        .env_remove("TEMPER_FORGEJO")
        .output()
        .expect("the tool runs");
    assert_eq!(output.status.code(), Some(1));
    assert_eq!(output.stderr, b"Error: \"TEMPER_FORGEJO must name the pinned Forgejo\"\n");
}
```
  Measured at the base against
  temper's crate: 0.020 s and 0.016 s (focused world < 0.5 s). No test moves
  into temper: the copy's 13 running tests and 1 ignored (`a_throwaway_forgejo_starts_serves_its_admin_and_is_removed`)
  leave with it and live on, names unchanged, as skein 06.1's. Unchanged:
  git-transcripts 7, conformance 25, `-p temper` 58 (default profile), process 19.
Proof:
- world: `cargo nextest run -p git-transcripts -p forgejo-conformance`: the
  count at the step's start plus 2 passed, 0 skipped (32 at the base, 7 + 25,
  in 0.016 s and 0.116 s), the two new < 0.1 s.
- world: `cargo nextest run -p temper`: the count at the step's start, as
  before it (58 passed, 0.369 s at the base); its five test binaries compile
  `support/{owned,forgejo}.rs`.
- joint: `cargo check --workspace --all-targets` Finished (0.28 s at the base)
  after the repin; then `scripts/check.sh` exits 0: clippy clean,
  focused count = origin/main's at the start minus 13 plus 2, and 0 skipped (the
  one skipped test at the base is the copy's only `#[ignore]`: 1879 passed,
  1 skipped, 4.9 s), fuzzy count at the step's start, unchanged (50 at the base, about 23 s; one slow,
  `fuzzy_walking_commit_cuts_latency_replay_and_observation_pressure`, 23.4 s).
- process: once, `TEMPER_FORGEJO=$WS/tmp/temper/forgejo/v17.0.0/forgejo-17.0.0-linux-amd64
  TEMPER_FORGEJO_RUNNER=$WS/tmp/temper/forgejo/runner-v13.2.0/forgejo-runner-13.2.0-linux-amd64
  scripts/proof-one-process.sh`: its count at the step's start passed (19 at
  the base), exit 0, about 146 s (145.2 s here, 145.7 s in base-runs.md), inside domain/testing.md 2.6's aim of 3
  minutes; a failure, or a run past it, is reported with its test and never
  retried silently (another base run failed at test 6, `base-temper-01.md`).
- process: the recording, before the repin commit and after the last, on one
  machine: `TEMPER_FORGEJO=$WS/tmp/temper/forgejo/v17.0.0/forgejo-17.0.0-linux-amd64
  TEMPER_FORGEJO_RUNNER=$WS/tmp/temper/forgejo/runner-v13.2.0/forgejo-runner-13.2.0-linux-amd64
  cargo run -p git-transcripts -- --git /usr/bin/git
  --scratch target/gt-<before|after> --out target/gt-<before|after>-out`
  exits 0 in about 16 s each, and `diff -r target/gt-before-out target/gt-after-out`
  prints nothing (the same tree twice prints nothing at the base).
Don't: change a value the process tier asserts or the arguments of `owned::attach`;
  add a path or login parameter, an alias or a `pub use`; make temper read
  `SKEIN_FORGEJO*`; touch the conformance's `TEMPER_FORGEJO_RUNNER` reads or
  its corpus; re-record the committed transcripts; delete `tools/forgejo-conformance`
  (temper 07); split a file.
Size: about 40 lines moved (45 at the base; 5 of them 01.2's pin lines), 31 new
  (4 for the three replaced `from_env` calls, 7 in the conformance's helper, 20
  in the two tests).
Removes: `testing/temper-throwaway-forgejo/`, 934 lines in 9 files (Cargo.toml,
  clippy.toml, `src/{binary,cli,http,instance,lib,tests}.rs`,
  `tests/throwaway.rs`) with `Forgejo::from_env` and `Refusal::Unset` inside it,
  and its workspace member and dependency lines; nothing else.

### 06.2 The `json` module and the leaves that read or write bytes

Scope: decision 27, first part: the module the reads and writes use, and the three leaves that parse
  or write bytes of their own. Behaviour unchanged.
Needs: 06.1 (the conformance on skein's throwaway); nothing outside (skein-json is a workspace
  dependency).
Files: new `tools/forgejo-conformance/src/{json.rs,json/tests.rs}`; changed `Cargo.toml`
  (`skein-json.workspace = true`), `src/lib.rs` (`pub mod json;`), `src/mask.rs::sorted_units`,
  `src/hooks.rs::body` (`serde_json::to_string(events)`), `src/registration.rs` (the runner-file read,
  `from_slice(&fs::read(file)?)`).
Shape: `Json` parses one document under the tokenizer's limits (depth 32, lengths the input's) into a
  retained `Document`; `At` reads by member, item and kind, and a missing member or the wrong kind
  reads `None`, no items or a non-match; `write` runs the builder twice (measure, then write) over
  `Encoder` (`Writing { depth: 8, length: 65_536 }`). `sorted_units` parses the array, sorts its
  strings and writes them back through `write`; a non-string item stops the rewrite as the base's
  failed parse did. `body` writes the event names as `["a","b"]`.
Test shape: four new unit tests in `json/tests.rs`, each <20 ms, over `BODY =
  {"id":7,"name":"specimen","private":false,"labels":["a","b"],"owner":{"id":3,"login":"fixture"}}`:
  - `a_parsed_body_reads_members_items_and_scalars_by_name` (reference): `id` 7, `name` "specimen",
    `private` false, `owner.login` "fixture", two labels with the second "b", and `same` true for one
    position, false for two:
    ```rust
    let json = Json::parse(BODY).expect("body");
    let root = json.root();
    assert_eq!((root.get("id").u64(), root.get("name").str(), root.get("private").bool()),
        (Some(7), Some("specimen"), Some(false)));
    assert!(!root.get("id").same(root.get("owner").get("id")));
    ```
  - `a_missing_member_or_the_wrong_kind_reads_as_none`: `absent`, `absent.deeper` and `name` as a number
    read `(None, None, None)`; `id` as a string or bool, and `name`'s items: `(None, None, 0)`.
  - `a_body_that_is_not_one_document_is_refused`: `""`, `{`, `{"a":}`, `[1,2` and `{} {}` are errors.
  - `an_array_of_strings_is_written_compact_and_escaped`: `["a","b\"c"]` writes the bytes
    `["a","b\"c"]` with its quote escaped, and no space or newline.
  Existing tests keep names: `masking_sorts_the_deprecated_units_set_in_every_team` proves
  `sorted_units`. Conformance tests: 26 at the step's start (the base's 25, 0.118 s, and 06.1's
  refusal test), 30 after.
Proof: unit; `cargo nextest run -p forgejo-conformance`: 30 passed, 26 at the step's start (the
  base's 25 and 06.1's refusal test) plus 4.
Proof: process; both recordings, with `T` a fresh directory under `$WS/tmp` and the forge variables of
  06.1's process line: `cargo run -p forgejo-conformance -- --scratch $T/instance
  --out $T/out`, exit 0, about 100 s (98 s alone at the base), then the same with `--component
  --scratch $T/component`, exit 0, about 30 s (28 s). Then the stable-fact line (the recording's checks) on
  `$T/out/facts.txt` prints the counts and the digest, and the transcripts diff (the same) names
  the four framing files and optional merge-ID variants in The recording's checks below, with only the
  four prescribed empty-body header additions beyond the original two. The leaves `hooks::body` and the
  registration read run only live.
Apply README's compatible smith source and Settled smith adapters rules. live-text/smith-02.1 is landed; shared compatibility now waits for jig01.s2 and temper02.1. Rebase their adapter landing before this repin's all-target check and both required recordings; retain the completed JSON checkpoint and every facts/transcript proof.
Upstream reset cause/repair belongs to skein06.s1; the bounded diagnostic exception is complete. Recording proof waits for its evidenced repair to land, then a compatible skein repin with the existing all-target check. Independent JSON work/checkpoint stays; diagnostic probes do not replace required recording/facts/transcript proof.
Proof: joint; `scripts/check.sh`: origin/main's focused count at the step's start plus 4,
  fuzzy unchanged, clippy and format clean (the gate runs both).
Don't: add a value tree, a `serde_json::Value` bridge or a second parser; re-serialize a captured body;
  change a transcript, a delivery or `facts.txt`; touch the `Fixture` family (06.3).
Size: about 15 lines moved (the three leaves, in place), about 215 new production lines (the module's
  reading half and `write` about 190, the leaves about 25) and 45 of tests.
Removes: nothing (`serde_json` still serves the rest).

### 06.3 The reads

Scope: decision 27, second part: every read of a captured or fetched body goes to `Json`.
Needs: 06.2 (the module); nothing outside.
Files: `tools/forgejo-conformance/src/json.rs` (`has_key`, its test); `src/lib.rs::json` and
  `src/fixture.rs` (`api_answer`, `api`, `get`, `branch`, `pull`, `put` return `Json`); read sites in
  `cases.rs`, `cli.rs::tokens` (all three captured-body `crate::json` reads),
  `facts.rs` (`Case.body: Json`, `matches`, `updated`), `registration.rs::listed`,
  `runner.rs` (`rows` and the helper returning a `Value`), `deliveries.rs` (its helpers, `act`'s
  answer), `hooks.rs` (`no_secret` through `Json::has_key("secret")`, `event_set`), `orgs.rs`
  (`write_team`), `tokens.rs`, `users.rs`, `component/{reads,writes}.rs`, `job_clone.rs`, `setup.rs`;
  `src/tests.rs` (the runner row `a_listed_runner_is_found_by_its_uuid` parses).
Shape: the family cannot split by module (every file calls `Fixture::api`), so it splits by direction:
  a body goes in as `&Value` for now and the answer comes out as `Json`. Each read keeps its meaning,
  `v["a"]["b"].as_str()` becoming `v.root().get("a").get("b").str()`; at the base 347 lines at most, in 18
  files (facts 75, deliveries 51, runner 45, hooks 28, component/writes 25, orgs 19, component/reads
  17, registration 15, cases 15, tokens 14, users 13, fixture 11, six files 19), and 146 lines of the
  `api` family in 14 files. `has_key` walks the document for a member at any depth and never matches
  a value that spells the key.
  Diagnostics (the designer's answer, as the decision states): a whole body in a failure's text
  (`facts.rs`: `initial(&defaults.to_string())`, `listing(&since.to_string())`, and `flags`'
  `format!("Forward {}, reverse {}, verification {}", ..)`) is printed as the response's own bytes,
  as received, the `body` its call site already holds: `facts.rs::diagnostic(body: &[u8]) -> String`
  cuts them at `DIAGNOSTIC_BYTES` (4096, a failure a person reads), decodes them lossily as UTF-8,
  and marks a cut with `… (N more bytes)`. JSON order and spacing may differ from today's
  `Display`: a diagnostic is for a person reading a failure, not a fact. A scalar's diagnostic
  prints the decoded value as Rust's escaped `{:?}`. `Json` keeps no bytes and gets no serializer:
  the seam is the conformance's fact code, not skein-json. Fact names, outcomes, statuses, their
  order and every assertion stay.
Test shape: two new unit tests. `a_diagnostic_is_the_bodys_bytes_cut_at_its_bound_and_marked`: a
  body of `DIAGNOSTIC_BYTES` bytes prints whole, and one byte more prints its first
  `DIAGNOSTIC_BYTES` and `… (1 more bytes)`. `a_key_is_found_at_any_depth_and_a_value_is_not_a_key`: over
  `{"a":{"secret":1},"b":["x",{"c":"secret"}]}`, `has_key("secret")` is true and `has_key("c")` true,
  `has_key("x")` false (an item, not a key). `a_listed_runner_is_found_by_its_uuid` and
  `a_fact_read_from_a_case_compares_its_status_and_message` change their fixtures, not their names or
  expected values. Conformance tests: 32.
Proof: unit; `cargo nextest run -p forgejo-conformance`: 32 passed, <1 s.
Proof: process; 06.2's two recordings, exit 0, the stable-fact line and the transcripts diff: the same
  counts and digest, and only the four framing differences plus optional masked
  merge-ID variants in The recording's checks. Any other difference, or a changed digest, is a question: the
  diagnostics print only on a failure.
Proof: joint; `scripts/check.sh`: the focused count plus 2, fuzzy unchanged.
Don't: add a `Value`-shaped accessor (`Index`), a macro, or a second tree; change a fact's text, its
  order or an expected value; turn a failing read into a `None` default that hides it; give `Json`
  retained bytes, a `Display` or a serializer, or print a body through a second parser or a
  normalizer; change ruling 192's six-path recording proof or a bound.
Size: about 350 lines moved (changed in place: each read keeps its meaning with another accessor),
  about 80 new (`has_key` about 35, its test 12, the `Json` returns about 8, `diagnostic` and its
  test about 25). Counted as new it is
  about 400, and stays whole: a half-converted family holds two trees, which decision 27 forbids.
Removes: nothing.

### 06.4 The writes

Scope: decision 27, last part: every body sent is written by the encoder; `serde_json` leaves.
Needs: 06.3.
Files: `tools/forgejo-conformance/src/json.rs` (`Put`, `object`, `write_bounded`),
  `facts.rs::bodies` and its `LARGE_FACT_REQUEST_BYTES`; `Cargo.toml` (`serde_json` out),
  `fixture.rs` (`api(&self, method: &str, target: &str, body: &[u8])`) and its 146 callers; the 69
  `json!` sites in 14 files (deliveries 23, facts 14, fixture 9, tokens 6, component/reads 4, hooks 3,
  orgs 2, job_clone 2, one each in users, setup, runner, component/writes, registration, cases); the
  `from_str(TEAM)` of `orgs.rs`.
Shape: a flat object becomes `json::object(&[("key", &value), ..])` with its keys sorted, which is the
  order `serde_json` wrote them in (no `preserve_order`), so the bytes sent do not change; `object`
  refuses unsorted or repeated keys. A nested one becomes `json::write(|out| ..)`. `TEAM` is sent as
  its own bytes, unparsed (its keys are unsorted): the two recorded `r.send` POSTs already send them
  so, and the PATCH goes through `Fixture::api`, which records nothing.
  Existing large-fact requests: extract the same two-pass implementation in `json.rs` as
  `pub(crate) fn write_bounded(length: u32, build: impl Fn(&mut Encoder))
  -> Result<Box<[u8]>>`. Ordinary `write` delegates with65_536 and `object`
  continues through ordinary `write`; depth8, error handling and measure-before-allocation
  stay. Only `facts::bodies` uses the explicit helper for its existing two rows,
  replaying the sorted `body`, `title` object builder with unchanged text and title.
  Name `LARGE_FACT_REQUEST_BYTES: u32 = 1_048_607` there: existing maximum
 1_048_576 ASCII `x` bytes plus31bytes for `{"body":"","title":"body fact"}`.
  The two complete requests are65_567and1_048_607bytes. Use identical limits and
  stable data on both passes; measurement still counts all escaping and refuses
  a longer document before output allocation. This is only room for those unchanged
  fixtures, never a replacement-text promise or a shared cap increase. Keep both
  fact names, GET equality assertions, request bytes and recording proofs. Carry
  the helper/caller into Skein06.2's whole-file move and refresh further reached
  copies or proof scripts, listing users in the commit body.
Test shape: three new unit tests, <20 ms (conformance tests: 35):
  - `an_object_is_written_compact_in_sorted_key_order`: `object(&[("a", &1u64), ("b", &"x"), ("c",
    &true)])` writes `{"a":1,"b":"x","c":true}`.
  - `members_out_of_order_or_repeated_are_refused`: `("b", ..), ("a", ..)` and `("a", ..), ("a", ..)`
    are errors.
  - `a_string_is_escaped_to_the_bytes_the_base_wrote`: `"` `\` newline, tab and a control byte `\u0001`
    write `\"`, `\\`, `\n`, `\t` and `\u0001`; a non-ASCII letter is written as it is.
Proof: unit; `cargo nextest run -p forgejo-conformance`: 35 passed (the base's 25, 06.1's refusal
  test and the 9 of 06.2 to 06.4), <1 s; `rg -n 'serde_json|json!' tools/forgejo-conformance
  -g '*.rs' -g Cargo.toml` prints nothing (96 lines in 18 files at the base).
Proof: process; 06.2's two recordings, the stable-fact line and the transcripts diff, as 06.3: `api`
  builds the state the facts record, so equal counts, digest and files show the bodies unchanged.
Proof: joint; `scripts/check.sh`: the focused count plus 3, fuzzy unchanged.
Don't: raise ordinary writer or production bounds, enlarge the explicit large-fact room beyond its named existing-input derivation, write a body by `format!`, or edit a transcript to match.
Size: about 100 lines changed in place (counted as moved), about 105 new (`Put` and `object` about
  66, the three tests about 40); counted as new, about 205.
Removes: `serde_json` from the conformance's manifest and its 96 lines at the base in 18 files.

## Interface sketches

Compiled in the scratch (clippy `-D warnings`, format clean): the `json` module
against skein-json (8a51e10).

```rust
// tools/forgejo-conformance/src/json.rs
/// One parsed document, retained whole; reads borrow from it.
pub struct Json(Document);
impl Json { pub fn parse(input: &[u8]) -> Result<Json>; pub fn root(&self) -> At<'_>;
    pub fn has_key(&self, name: &str) -> bool; }
/// A position in a `Json`; a wrong path or kind reads as `None`.
#[derive(Clone, Copy)] pub struct At<'a> { .. }
impl<'a> At<'a> { pub fn get(self, name: &str) -> At<'a>; pub fn items(self) -> Vec<At<'a>>;
    pub fn nth(self, n: usize) -> At<'a>; pub fn str(self) -> Option<&'a str>;
    pub fn u64(self) -> Option<u64>; pub fn bool(self) -> Option<bool>;
    pub fn is(self, text: &str) -> bool; pub fn same(self, other: At<'_>) -> bool; }
/// A value the encoder can place.
pub trait Put { fn put(&self, out: &mut Encoder); }
pub fn write(build: impl Fn(&mut Encoder)) -> Result<Box<[u8]>>;
pub fn object(members: &[(&str, &dyn Put)]) -> Result<Box<[u8]>>;
```

The conformance owns its `json` module until skein 06.2 copies it.

## Questions to raise

- A recording that differs beyond the four required framing changes or the two exact optional
  masked merge-ID variants in The recording's checks, or a changed digest (06.2 to 06.4):
  never edit a transcript.

## The recording's checks

06.2 to 06.4's process Proofs run these on a recording, `$T/out`. They are not Done when items:
temper 07 deletes the conformance, and skein 06.4 keeps the stable-fact line.

- The stable-fact line, on a fresh recording's `facts.txt` (`$T/out/facts.txt`) or the committed
  `tools/forgejo-conformance/facts.txt`, prints the five timing outcomes, then `facts=218 stable=213
  holds=207 fails=6 timing=5 unchecked=0` and the digest `d1f7ba2bb323...ec18347d`, exit 0:
  ```sh
  bash -o pipefail -c 'set -eu; digest=$(awk '\''BEGIN { split("merge-old-head-is-409-head-out-of-date delivery-pull-reads-not-mergeable-while-checked delivery-pull-is-settled-when-its-push-is-delivered delivery-pull-is-checking-when-its-push-is-delivered webhook-deliveries", a); for (i in a) timing[a[i]]=1 } /^#/ || NF==0 { next } { name=$2; sub(/:.*/, "", name); if (seen[name]++ || $1 !~ /^(holds|fails)$/) exit 1; n++;   if (name in timing) { t++; print "timing " $0 > "/dev/stderr" }   else { s++; h+=($1=="holds"); f+=($1=="fails"); print $1, name } } END { if (n!=218 || s!=213 || h!=207 || f!=6 || t!=5) exit 1;   print "facts=218 stable=213 holds=207 fails=6 timing=5 unchecked=0" > "/dev/stderr" }'\'' "$1" | LC_ALL=C sort | sha256sum); printf "%s\n" "$digest"; test "$digest" = "d1f7ba2bb323b8944e05d2e05bc1c83396fb7bfffadbcefea4faad64ec18347d  -"' sh tools/forgejo-conformance/facts.txt
  ```
- The transcripts diff, `sh -c 'cd tools/forgejo-conformance && diff -rq --exclude=response
  transcripts "$1"/transcripts; echo "diff exit $?"' sh $T/out`, prints four required
  `Files ... differ` lines: `setup-team-member-added/request`,
  `setup-team-member-missing-user/request`, `update-conflict/request` and `update-pull/request`.
  `merge-conflict/request` and `merge-stale-head/request` may each be byte-identical to
  corpus, as in06.4's retained recording; if different, only the prior06.3masked
  `head_commit_id` replacement is allowed:40hex digits with39leading zeros, conflict
  ending `e` to `a`, stale ending `d` to `e`, and every other byte unchanged.
  Thus4to6paths differ and `diff exit 1`. For each required framing file,
  inspect the full byte diff: the sole addition is `Content-Length: 0`
  before `Connection: close`, with no body or other byte change. These empty PUT
  (team member) and POST (pull update) requests require the header under skein
  `http.md`3.1. Do not strip headers, normalize bodies, change captured bytes,
  re-record the corpus or accept another difference. Refresh every reached proof
  script/copy within its owning increment and list users in the commit body.


## Done when

Each command runs once in a worktree of temper at `origin/main`.

- `cargo check --workspace --all-targets` prints Finished, exits 0;
  the session's gate Proof is `scripts/check.sh` (land owns it).
- `rg -n 'temper_throwaway_forgejo|temper-throwaway-forgejo' Cargo.toml crates testing tests tools -g '*.rs' -g Cargo.toml`
  prints nothing (exit 1); at the base it prints 48 lines (46 outside the crate).
- `python3 -I -c 'from pathlib import Path; print("throwaway_copy_absent=" + str(not Path("testing/temper-throwaway-forgejo").exists()))'`
  prints `throwaway_copy_absent=True` (False at the base).
- `rg -c 'skein-throwaway-forgejo' Cargo.toml crates/temper/Cargo.toml tools/git-transcripts/Cargo.toml`
  prints `1` for each of the three files, with the file names.
- `rg -n 'Forgejo::from_env|SKEIN_FORGEJO|skein_throwaway_forgejo::(VERSION|RUNNER_VERSION|names_runner)' crates tools -g '*.rs'`
  prints nothing (exit 1); the pin is read from `skein_forge::forgejo::pin`
  (at the base it prints the three `from_env` calls).
- `cargo nextest run -p git-transcripts --test refusal` prints `1 test run: 1 passed, 0 skipped`
  (the conformance's refusal test leaves with it in temper 07).
- `rg -n 'pub use skein_' crates tools -g '*.rs'` prints nothing (exit 1): no
  alias or re-export of a kit item.
- The process tier: `TEMPER_FORGEJO=$WS/tmp/temper/forgejo/v17.0.0/forgejo-17.0.0-linux-amd64
  TEMPER_FORGEJO_RUNNER=$WS/tmp/temper/forgejo/runner-v13.2.0/forgejo-runner-13.2.0-linux-amd64
  scripts/proof-one-process.sh` exits 0 (README, Done when 6).
