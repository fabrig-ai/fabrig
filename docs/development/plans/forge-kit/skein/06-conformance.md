# Session 06: the throwaway, conformance and dialect document tests

Move the Forgejo harness, recorder and corpus, then the dialect's document and mutation tests. Needs
at start: temper 01.2's dialect users on skein-forge.

## Read first

- skein `docs/design/fake-forge.md` 6–8, `forge.md` 4.1, 11, `shell.md` 12;
  `docs/foundation/programming-model.md` 3, 9, 10 and `testing-strategy.md` 2, 8, 10.
- README's sessions, Rules and Done when; `code-organization.md` 1, 3, 5–8; `decisions.md` 2, 3,
  10–18 and settled Design questions 30, 31.
- `findings/{survey-05-fakes,survey-01-forge,trace-fakes,trace-forge,base-runs,base-skein-06}.md`:
  source users, design trace, test names/times and base commands.

## What the design asks

- fake-forge 6: explicit paths, bounded skein-http exchanges under drive, attachment.
- fake-forge 7: transcripts, deliveries, setup/runner/component facts and masks.
- fake-forge 8: cheap focused checks; binary tests opt in.
- forge 11: recorded/generated/mutated documents, golden bytes and boundary refusals.
- testing-strategy 8: keep moved tests' costs understood and visible.

## Choices this plan makes

- Keep the package/binary name `forgejo-conformance`. Counts below are at the base: 151 transcript
  cases plus binary.txt (303 files), 24 deliveries (48 files).
- 06.2 checks the copied bytes, then records neutral values once. Its existing mask clears four
  stale UUID occurrences; 06.4 repairs nine unquoted log times and records once more. 06.3 keeps the
  README's document-test step number.

## Rules that bind it

- Copy temper origin/main at the step's start; name the source commit. Only Shape's home-forced
  changes accompany moves. Data is written only by recording.
- No system names, serde or moved-type aliases/re-exports; tests use kit's values (code-organization
  5). fake-forge 6 explicitly requires the runner pin re-exports.
- Use code-organization 6's per-pair numstat. Size counts production, a tests-only increment its tests
  (code-organization 6); moved tests keep names, workloads and times. No optimized tests or raised budgets.

Commands run at skein's root. Binary paths are under `$WS/tmp/`, `$WS` the workspace (README's
Rules). Process proofs stay outside the gate. Base forms are in the
findings; commands naming future code await readiness.

## Increments

### 06.1 The throwaway on skein's HTTP client
Scope: fake-forge 6, 8; the harness and its tests, with the required host seam.
Needs: the session's start capability.
Files: temper `testing/temper-throwaway-forgejo/{Cargo.toml,clippy.toml,
  src/{lib,binary,cli,http,instance,tests}.rs,tests/throwaway.rs}` -> skein
  `testing/skein-throwaway-forgejo/` (new); new private `src/http/host.rs`, `tests/support/mod.rs` and
  a Rust scripted-peer helper `tests/support/attached.rs` (test fixture binary declared in
  Cargo.toml). Shared: root `Cargo.toml` member and workspace dependency, `Cargo.lock`.
Shape:
- Keep `Forgejo`, `Settings`, `Account`, `Token`, `Init`, `Made`, `Range`, `Outgoing`, `Exchange`,
  `Failure`, `Output`, `WorkPath`; provisioning, scratch refusal, environment clearing and cleanup
  stay. Decision 3 replaces `binary.rs::Forgejo::from_path`'s `skein_io::digest::digest(&bytes).0`
  with `skein_lib::sha256::digest(&bytes)`, rendering each byte as `{byte:02x}`; the hex is identical.
- Remove `Forgejo::from_env` and its `Refusal::Unset`; `from_path(&Path) -> Result<Forgejo,
  Refusal>` reads no environment. Use the moved dialect pin. Add the designed `pub use
  skein_forge::forgejo::pin::{RUNNER_VERSION, names_runner}`. `tests/throwaway.rs` imports VERSION
  from `skein_forge::forgejo::pin`, and binary.rs names_forgejo comes from that pin; remove root
  VERSION. APP_NAME becomes `kit throwaway`; retain the existing neutral `fixture` login. Only the
  ignored test reads `SKEIN_FORGEJO`; any runner test reads `SKEIN_FORGEJO_RUNNER` there, never in
  the library. Unset errors name `SKEIN_FORGEJO must name the pinned Forgejo` or
  `SKEIN_FORGEJO_RUNNER must name the pinned runner` (answer 31).
- Home-forced by skein-http and the no-serde rule: replace provisioning's serde values with
  skein-json documents/writer calls. Use skein-lib's
  standard padded base64 encoder in the existing `basic` helper; delete its private
  alphabet/encoding loop. `http::exchange(port, &Outgoing, auth, cap) -> Result<Exchange, Failure>`
  keeps its signature. Capture receive bytes through the first CRLFCRLF, retaining the head through
  its last CRLF, before feeding skein-http: `Response` loses the reason phrase, so never synthesize
  a status line. The HTTP decoder alone supplies status and unframed body. Build the client's Call
  with ordered Host, Authorization, Content-Type, Range. In `http.rs::request`, use
  `Body::None` when the body is absent and `Body::Length(n)` for a present body;
  `Length(0)` adds Content-Length and changes the absent-body GET golden. Keep
  close=true for Connection. Capture its sends, replacing only credentials for `sent`.
  No credential reaches the retained capture. A private one-shot Host drives
  skein-http over skein-io using `skein_shell::drive`: close admission at the answer/error, drain
  all completions, then return. Keep 100 ms connect, 10 s exchange and the 16 MiB
  `Settings::small().answer_bytes` cap; cap before retaining bytes.
- `ready` keeps `ready_polls` (200), 50 ms spacing and child-exit checks, with drive's
  deadline/wait, no `thread::sleep` or busy polling. The existing request/framing tests exercise
  skein-http through their existing helper entry points; retain no second HTTP/chunk decoder.
- Home-forced by answer 30: attach drops the service's hard-coded login and reads both login and
  id from the already-required user lookup; the owner supplies the token. Signatures:
```rust
// testing/skein-throwaway-forgejo/src/instance.rs
pub fn attach(child: Child, port: u16, scratch: &Path,
              admin: Token, settings: &Settings) -> Result<Instance, Failure>;
fn value(response: &Exchange) -> Result<skein_json::Document, Failure>;
```
  temper's side: temper 06.1 or the next repin repairs `tools/git-transcripts/src/lib.rs::run`'s
  from_env; temper reads TEMPER_FORGEJO then calls from_path.
  `crates/temper/tests/support/forgejo.rs::Throwaway::start` reaches owned::attach/binaries; keep
  those environment reads in temper. `support/owned.rs::attach` keeps its arguments; its admin login
  now comes from the API. `support/owned.rs`'s VERSION import (scripted, finished, dump) and
  `process.rs::{a_prepared_forge_runs_forgejo_and_its_runner_once_each,
  an_older_recorded_forgejo_is_dumped_then_upgraded}` import VERSION from the pin directly.
  owned.rs's RUNNER_VERSION is already the pin's (temper 01.2); temper uses no re-export.
Test shape: 13 focused tests and 1 ignored move; add three focused tests in `tests/throwaway.rs` (16
  focused total). Reuse Settings::small and the scratch helper. A Rust child serves scripted
  version/user responses on loopback, then waits for termination; no Forgejo or runner is needed.
  The fixture retains the PID and a `b"mine"` file; under 0.5 s per test.
- `attach_reads_the_admin_login_and_id`: GET user returns login `kit-owner`, id 37; admin() must
  expose precisely that pair.
- `stopping_an_attached_instance_reaps_the_child_and_keeps_scratch`: stop succeeds, the PID is
  gone/reaped, file bytes remain `b"mine"`.
- `an_attached_instance_refuses_cli`: cli([]) returns `Failure::Io("attached instance has no offline
  work path")`. Reference (support::attached is the new fixture described above):
```rust
// testing/skein-throwaway-forgejo/tests/throwaway.rs
mod support;
#[test]
fn attach_reads_the_admin_login_and_id() {
    let fixture = support::attached("kit-owner", 37);
    assert_eq!(fixture.instance.admin().login, "kit-owner");
    assert_eq!(fixture.instance.admin().id, 37);
}
```
  Reuse `Settings::small`, `forgejo_running`, `Outgoing`, `http::request` and `http::parse`'s test
  entry point with its skein-http implementation. Update only neutral expected bytes. Existing
  assertions retain `Basic MASKED`, `token MASKED`, chunked body bytes, page cap 2, refusal
  preserving `b"mine"`, and removal of the stopped scratch. Extend
  `a_recorded_request_holds_no_token` with literal sent/head bytes copied from base
  `transcripts/user` (head through last CRLF; body excluded), fed through the Host capture/decoder
  entry point; expect byte equality, including `HTTP/1.1 200 OK`. Keep this golden inline before
  06.2 exists. Base 13 tests: 0.010 s, <20 ms each; three new fixture tests <0.5 s each.
Proof: unit; `cargo nextest run -p skein-throwaway-forgejo`
  prints 16 passed, 1 skipped; base 13 in 0.010 s, new tests <0.5 s each.
Proof: process; `SKEIN_FORGEJO=$WS/tmp/temper/forgejo/v17.0.0/forgejo-17.0.0-linux-amd64 cargo nextest run -p skein-throwaway-forgejo --run-ignored only`
  prints 1 passed, 16 skipped: starts, serves its admin and removes scratch. Base 7.898 s; under 15
  s; no runner needed by this existing test.
Don't: move runner startup here; change the process tier's owner values.
Size: about 715 lines moved, 200 new: the measured 7, the 121-line http.rs on skein-http (about
  90), instance.rs's five serde lines (about 15) and the new tests (about 90), estimated; 168 test
  lines move (findings, Re-check sizing, for the measured part).
Removes: the copied environment constructor and raw TCP/chunk-parser implementation. Nothing in
  temper: temper 06.1 repins and updates `tools/git-transcripts`,
  `crates/temper/tests/{process.rs,support/forgejo.rs, support/owned.rs}`, including the sole
  `Instance::attach` caller.

### 06.2 The conformance and its recording script
Scope: fake-forge 7, 8; the recorder on the kit and its neutral corpus.
Needs: 06.1's throwaway; temper 03.1 and temper 06.4 (README's table): after them the
  conformance is on skein-forge's vocabulary, skein-forge-connection and skein-json (decision 27).
Files: temper `tools/forgejo-conformance/` -> skein `tests/forge/conformance/`, including
  Cargo.toml, clippy.toml, README, every `src/` file, `transcripts/`, `deliveries/`, `facts.txt`.
  New `scripts/proof-forgejo-conformance.sh`; shared root Cargo.toml (explicit member), Cargo.lock.
  The script is a thin shell invocation of Rust.
Shape:
- The whole-file move also carries Temper06.4's `json::write_bounded` and
  `facts::bodies`/`LARGE_FACT_REQUEST_BYTES`: ordinary writes retain65_536,
  only the two unchanged large-fact requests use their explicit1_048_607byte
  room. Preserve both fact names/equality assertions, request bytes and all
  recording/digest proof copies; refresh further reached copies and list users.
- Set `license.workspace = true` in the destination Cargo.toml's `[package]`, beside its other
  workspace-inherited fields (README, Workspace license inheritance).
- Keep `Fixture`, `Mask`, `Secrets`, `Fact`, `Outcome`, `Hosted` and the cases, provisioning order,
  leak scans and fact names. Preserve each measured outcome; the comparison below separates five
  known races. The source already names the kit after Needs; no client/component edge is added. Do
  not move a downstream profile into a dependency.
- The copy has no serde: temper 06.2 to 06.4 put the conformance's 17 source files on skein-json's
  `Document` and `writer::Encoder` (decision 27), so the move renames only. If serde is back on
  temper's main at the start, stop and ask; never replace it in the move.
- Neutralize these source values/assertions; data changes by recording: `temper / label` -> `kit /
  label`, `temper:bot` -> `kit:bot`, `temper-api`/`temper-git` -> `kit-api`/`kit-git`,
  `temper-admin` (including -a/-b) -> `kit-admin` (same suffixes), `temper-host`/`temper-held` ->
  `kit-host`/`kit-held`, `temper-conformance` -> `kit-conformance`, `temper-conformance-hook-secret`
  -> `kit-conformance-hook-secret`, `TEMPER_CONFORMANCE_FAILED_JOB` -> `KIT_CONFORMANCE_FAILED_JOB`,
  all `temper:key` markers -> `kit:key` (13 source and 29 corpus occurrences;
  49 corpus files use the old word); the component
  receives the owner's `kit` word. Switch `registration.rs`'s RUNNER_VERSION, `runner.rs::version`'s
  names_runner and `tests.rs::a_listed_runner_is_found_by_its_uuid`'s version reference from
  the dialect pin to `skein_throwaway_forgejo`'s designed re-exports, added by 06.1. `--forgejo` is
  required; `--runner` optional; `--scratch`, `--out` and `--component` retain meaning (designer
  answer 31).
```rust
// tests/forge/conformance/src/lib.rs
pub fn run(binary: &Path, runner: Option<&Path>,
           scratch: &Path, out: &Path) -> Result<()>;
fn json(exchange: &Exchange) -> Result<skein_json::Document>;
// tests/forge/conformance/src/component.rs
pub fn record(binary: &Path, runner: Option<&Path>,
              scratch: &Path, out: &Path) -> Result<()>;
pub fn run(forgejo: &Forgejo, runner: Option<&Path>,
           scratch: &Path) -> Result<Vec<Fact>>;
```
  `Result` is the existing boxed-error result. Thread `Option<&Path>` as first argument of
  `runner::{run,component,run_with}`, `deliveries::record` and `component::reads::{run,actions}`;
  update callers in lib.rs, component.rs and reads.rs, with other parameters/returns unchanged.
  Replace lib.rs::run/component.rs::record's from_env (or, once temper 06.1 has landed, its
  `forgejo()` helper's read of TEMPER_FORGEJO) with from_path(binary); remove both runner.rs
  environment reads, deliveries.rs::record's read and both reads.rs reads (run/actions). Only
  main.rs reads CLI arguments; absent --forgejo fails `--forgejo required`. None runner keeps
  the base runner/component groups unchecked (`--runner not supplied`), but `deliveries::record` still refuses at its existing
  runner lookup, now `--runner required for delivery CI`. Never skip delivery CI, here or later.
  No conformance code reads a variable.
  temper's side: no temper caller uses the conformance's flags or runner arguments; its own tool
  is deleted by temper 07. `tools/git-transcripts` uses only 06.1's throwaway interface.
- The script passes both paths, runs ordinary recording then `--component` with separate empty
  scratches under the given scratch parent, and preserves every failure exit. It adds no gate entry
  and runs directly. The only 06.2 recording is its process Proof, after this copy check. Set
  `FORGE_COPY` to the source checkout at the recorded source commit;
  `sh -c 'for p in transcripts deliveries facts.txt; do diff -qr "$1/$p" "$2/$p" || exit; done' sh "$FORGE_COPY/tools/forgejo-conformance" tests/forge/conformance`
  prints nothing and exits 0 before recording. This compares all data bytes, including binary
  provenance, without a fixture rewrite. Document every existing mask: ids/UUIDs by first
  appearance, quoted times, fixed HTTP dates and ports, sorted units, git pkt-line prefixes;
  document the outstanding unquoted log-time repair owned by 06.4. The base mask's UUID rule clears
  four stale UUID occurrences during 06.2's recording; nine unquoted log times remain for 06.4.
Test shape: move all 34 library tests unchanged in name (the base's25 plus9from Temper06.2–06.4, including the bounded-diagnostic test;
  temper 06.1's refusal test stays temper's, deleted by temper 07: answer 31); retain `Mask::new`, `Secrets::of`, `write::check`,
  `Fact::line`. No new test kind. Token/tail/password leaks retain the literal `Leak` case/kind;
  masked bytes and content lengths retain their asserted values. Base suite 0.114 s; listener world
  0.109 s (<0.5 s), other tests <20 ms.
Proof: unit; `cargo nextest run -p forgejo-conformance`
  prints34passed,0skipped (source-verified library identity count; the historical base's25took0.114s), preserving that workload/time.
Proof: process; `scripts/proof-forgejo-conformance.sh --forgejo "$WS/tmp/temper/forgejo/v17.0.0/forgejo-17.0.0-linux-amd64" --runner "$WS/tmp/temper/forgejo/runner-v13.2.0/forgejo-runner-13.2.0-linux-amd64" --scratch target/forgejo-conformance --out tests/forge/conformance`
  once: exits 0; 218 facts, all 55 setup and 40 component facts hold. Then compare, as part of this
  Proof (no second recording):
  `bash -o pipefail -c 'set -eu; digest=$(awk '\''BEGIN { split("merge-old-head-is-409-head-out-of-date delivery-pull-reads-not-mergeable-while-checked delivery-pull-is-settled-when-its-push-is-delivered delivery-pull-is-checking-when-its-push-is-delivered webhook-deliveries", a); for (i in a) timing[a[i]]=1 } /^#/ || NF==0 { next } { name=$2; sub(/:.*/, "", name); if (seen[name]++ || $1 !~ /^(holds|fails)$/) exit 1; n++;   if (name in timing) { t++; print "timing " $0 > "/dev/stderr" }   else { s++; h+=($1=="holds"); f+=($1=="fails"); print $1, name } } END { if (n!=218 || s!=213 || h!=207 || f!=6 || t!=5) exit 1;   print "facts=218 stable=213 holds=207 fails=6 timing=5 unchecked=0" > "/dev/stderr" }'\'' "$1" | LC_ALL=C sort | sha256sum); printf "%s\n" "$digest"; test "$digest" = "d1f7ba2bb323b8944e05d2e05bc1c83396fb7bfffadbcefea4faad64ec18347d  -"' sh tests/forge/conformance/facts.txt`
  prints five `timing <holds|fails> <name>[: reason]` lines, then `facts=218 stable=213 holds=207
  fails=6 timing=5 unchecked=0` and
  `d1f7ba2bb323b8944e05d2e05bc1c83396fb7bfffadbcefea4faad64ec18347d  -`. Base recordings: 106 s + 22
  s; comparison <1 s; both modes within 5 min. The five timing observations (committed/fresh on
  identical pins):
- `merge-old-head-is-409-head-out-of-date` (holds/fails): stale-head merge races mergeability work;
  405 Please try again later precedes the 409 check.
- `delivery-pull-reads-not-mergeable-while-checked` (holds/fails): initial create/push probes may
  miss the short mergeable=false interval.
- `delivery-pull-is-settled-when-its-push-is-delivered` (fails/holds): both first pull reads after
  matching push delivery may follow check completion.
- `delivery-pull-is-checking-when-its-push-is-delivered` (holds/fails): one first read must catch
  checking and the reread must settle within 30 s.
- `webhook-deliveries` (holds/fails): roll_up inherits checking detection and the exclusive
  settled/checking pair; it is no independent race.
Don't: make known base `fails` facts pass by weakening them; hand-edit data; fix masking here; treat
  exit 0 alone as conformance. Making the five races deterministic goes to later.md; retain their
  outcomes/reasons verbatim.
Size: about 4,050 lines moved, 170 new: the measured 119 (paths, owner values and flags) and the
  script, about 50, estimated; the 324 test lines (and temper 06.2 to 06.4's) and the corpus move. The JSON replacement is
  temper 06.2 to 06.4's (decision 27).
Removes: serde and service-named environment reads in the copied recorder. Nothing in temper; temper
  07 removes its copy after all readers move.

### 06.3 The dialect's document tests and missing boundary coverage
Scope: forge 11's documents, encoders and decoders; decision 10.
Needs: 06.2's neutral transcripts/deliveries, not just its crate.
Files: temper `tests/codecs/tests/{forgejo_*.rs,fuzzy_forgejo.rs,fixtures.rs}` -> `crates/skein-forge/tests/`;
  `tests/codecs/src/forgejo.rs` -> `crates/skein-forge/tests/common/forgejo.rs`; copy only `mutated`
  and `mutated_into` from codecs' src/lib.rs into `tests/common/mod.rs`. New
  `tests/forgejo_bounds.rs`; changed skein-forge Cargo.toml if a test-only dependency is needed;
  shared case tables extracted into `tests/common/cases/`. Reuse `src/forgejo/tests/fixtures.rs`'s
  admitted limits and values by test-source inclusion, adjusting its imports for both harnesses if
  needed (changed). No production fixture export.
Shape:
- The moved integration targets inherit skein's step-code lints. Add `#![cfg(test)]`
  to test targets and use scoped, reasoned test-harness `#[expect]` attributes where
  the copied harness needs them. Expose `common` within each test target so shared
  helpers do not trigger unused-helper errors; keep production lints intact.
  Shorten redundant qualifications in `common/forgejo.rs::{decode_refusal,decode_body,
  transcript_cases}` and `forgejo_setup.rs::OrgAnswer::request` for their new imports.
- Keep helpers' public signatures and all 58 moved test names. Import `skein_forge::forgejo`; the
  transcript helper reads `../../tests/forge/conformance/{transcripts,deliveries}` relative to
  CARGO_MANIFEST_DIR. Production dialect code still performs no io.
- Convert shape inspection, JSON validation and mutations in the helper and five test files from
  serde to skein-json; discard the serde-only `walk` and value-builder helpers. Use literal
  malformed bytes where a test changes one field; preserve the same fault and expected `Malformed`.
  No loader substitution of old words: the on-disk corpus already uses kit; the moved files' own
  values (`temper-api`, `temper:bot`, ...) become kit ones, the copy rule's forced rename.
- Keep generic `limits()` as copied. `profile_limits() -> Limits` becomes a fixture in the shared
  test source, reached by the existing helper: plain values from the shipped Small profile:
  tokenizer depth 16, string 16,384, number 32, chunk 4,096, length 16,777,216; writer depth 16,
  length 16,777,216; string_bytes 512, body_bytes 16,384, inner 32, target_bytes 4,096. The delivery
  fuzzy test uses this same helper. `forgejo_deliveries.rs::shipped` also calls
  `forgejo::profile_limits()` through its existing `use common::forgejo`, replacing its
  temper-service profile lookup and avoiding a redundant qualification.
  Refresh further copies and derived values reached by these repairs, and list
  further users and lint adaptations in the commit body.
  Required fresh-corpus expectations: in `tests/common/cases/files.rs`, the
  `actions-combined-status` row keeps both complete expected statuses and every
  assertion; only their captured `updated_at` literals follow06.2's recording.
  The old13/14second masked values become12/12:
  `1_577_817_013_000_000_000` and `1_577_817_014_000_000_000` each become
  `1_577_817_012_000_000_000`, independently converted from the raw
  `2020-01-01T00:00:12+05:30` fields. Follow the README Recorded fixture
  expectations rule for further reached masked identities/times/derived values
  in the moved case tables and after06.4's required recording; list changed users.
  The same pure-decode rule includes captured page membership: in
  `tests/common/cases/reviews.rs`, `branches-page-2` retains two ordered rows
  and every exact equality. Keep `writer-head`/sha9first; the recorded second
  row is now `conflict`/sha10rather than `main`/sha3, independently verified
  from the old/new raw responses. These are fixture input values, not a
  guarantee of stable API page selection. Apply the README rule to all further
  reached files/items/reviews/writes table literals; do not adjust semantic
  error/status/scenario guarantees, remove a case or derive expected values
  from the decoder under test.
  This does not change58inherited test names/cases/assertions or61focused/4fuzzy
  final counts. Preserve every malformed mutation and its expected error;
  repair the independent JSON mutation adaptation within its existing scope,
  never by changing an expected error or a production decoder.
Test shape: 55 focused + 3 fuzzy move. Add six table tests in `forgejo_bounds.rs`, reusing
  common::forgejo's limits/decoders and the moved seven case tables (share the tables, not copies):
- `limits_accept_the_profile_and_reject_each_problem`: reference below; all eight zero fields, both
  tokenizer-string inequalities, both depths.
- `document_bounds_hold_at_the_limit_and_one_past`: string_bytes 8, body_bytes 16 and inner 2 beside
  the admitted profile; 8-byte login accepted/9 gives TooLong(login), body 16 Kept/17 Long, 2 scopes
  accepted/ 3 gives Bound(scopes), with named expected values.
- `stack_depth_accepts_the_limit_and_refuses_one_past`: unknown nested arrays in a valid user
  document reach json.depth, then depth+1; identical User at the limit, Json(TooDeep) one past;
  writer depth 8 admits the boundary fixture and 7 is Problem::Depth.
- `strings_cut_at_every_byte_decode_once`: split a valid string containing UTF-8, escaped quote and
  a surrogate pair at each byte (including 0/end); decoded `kité"𝄞` bytes and one terminal at every
  split; feed each strict prefix of the complete JSON document an End and expect Json(Truncated).
- `each_encoder_has_golden_bytes_and_a_measured_bound`: rows cover every client request, server
  answer and delivery encoder, exact literal bytes; target/writer room at measured size succeeds,
  size-1 is TooLong.
- `seeded_documents_round_trip_on_both_sides`: seeds 0, 7, 31 generate valid rows of every decoder
  family; client->server and server->client equality against the generated value, including delivery
  bodies and error documents. Extend `fuzzy_forgejo.rs` with
  `generated_documents_and_their_mutations_end_within_the_step_bound`: seeds 0..32, eight mutations
  per generated case, no panic, at most `STEP_BOUND`, valid cases equal their generated value. Keep
  the three original sweeps unchanged (3.079–4.347 s at the earlier base). New unit tables <20 ms
  each; new fuzzy sweep <=1 s. No new world.
```rust
// crates/skein-forge/tests/forgejo_bounds.rs
mod common;
use common::forgejo;
use skein_forge::forgejo::limits::{self, Limits, Problem};
#[test]
fn limits_accept_the_profile_and_reject_each_problem() {
    let good = forgejo::profile_limits();
    assert_eq!(limits::check(&good), Ok(()));
    let zeros: [fn(&mut Limits); 8] = [
        |l| l.string_bytes = 0, |l| l.body_bytes = 0,
        |l| l.inner = 0, |l| l.target_bytes = 0,
        |l| l.json.chunk = 0, |l| l.json.number = 0,
        |l| l.json.length = 0, |l| l.writer.length = 0,
    ];
    for zero in zeros {
        let mut value = good;
        zero(&mut value);
        assert_eq!(limits::check(&value), Err(Problem::Zero));
    }
    for field in [good.string_bytes, good.body_bytes] {
        let mut value = good;
        value.json.string = field - 1;
        assert_eq!(limits::check(&value), Err(Problem::TokenizerString));
    }
    for writer in [false, true] {
        let mut value = good;
        if writer { value.writer.depth = 7; } else { value.json.depth = 7; }
        assert_eq!(limits::check(&value), Err(Problem::Depth));
    }
}
```
Proof: unit; `cargo nextest run -p skein-forge --test 'forgejo_*'`
  prints 61 passed (55 moved + 6 new), preserving the moved tests' times;
  `cargo nextest run -p skein-forge --test fuzzy_forgejo --profile fuzzy`
  prints 4 passed (3 moved + 1 new). Base forms use temper-codecs-world: 55 passed in 0.134 s and 3
  passed in 2.524 s; new tables <20 ms each, new fuzzy <=1 s, the older moved fuzzy measurements
  stay as recorded. Only pure document tests change here; 06.2 connected the recorder, and 06.4
  records again only after its mask change. No process Proof here.
Don't: import temper-service, the component or serde into skein-forge's tests; weaken mutations;
  rename a moved test; change production decoders to accommodate a failing new test without
  recording the design discrepancy.
Size: about 2,560 lines moved (fixtures.rs's 122 included), 300 new, all tests (code-organization.md 6).
Removes: nothing in temper; temper 07 removes these tests/helpers only after their remaining readers
  have moved. Other codecs tests remain temper's.

### 06.4 Complete the masks and re-record
Scope: fake-forge 7 and forge 11's volatile fields; decision 16, split from the move so a behavioral
  change has its own tests.
Needs: 06.2's recorder and 06.3's document regression tests.
Files: `tests/forge/conformance/src/{mask,tests}.rs`, README, `transcripts/`, `deliveries/`,
  `facts.txt` (recorder output only).
Shape: retain `Mask::bytes(&mut self, &[u8]) -> Vec<u8>`. Its existing UUID rule is unchanged; 06.2
  already cleared the four stale UUIDs. This step clears the nine unquoted log times. Recognize
  unquoted RFC3339, including the seven fractional digits: reuse the time map, retain fraction width
  and offset, replace volatile fraction digits with zeros. Quoted timestamps keep the base rule:
  fractions are dropped. Both forms key the shared map by the original timestamp without quotes:
  same ordinal, different rendered bytes. Do not mask numeric resource ids, git pkt lengths or
  unrelated text. Update the mask contract.
  temper's side: no caller changes; `Mask::bytes` and 06.2's flags/runner arguments stay the same.
Test shape: preserve the34moved library identities. Add two mask tests and split three existing success/refusal pairs in src/tests.rs (five additional test identities in total):
  `unquoted_log_times_keep_fraction_width_offset_and_identity` checks duplicate and distinct
  quoted/unquoted times, literal fractions/offsets;
  `recorded_corpus_has_no_unmasked_uuid_or_log_time` scans every committed request/response
  independently of Mask, accepting UUIDs only as `00000000-0000-0000-0000-<12 hex digits>` and
  volatile log dates only in their canonical 2020 form with zero fractions. Expected bad counts:
  0/0. Also check Content-Length against body bytes after masking, using the existing framing test.
  The two mask tests and three separated negative cases each remain below20ms.
  Split only these existing pairs under testing-strategy.md10, One behaviour per test:
  keep the original positive test names and move their existing negative assertions
  into `a_fact_read_from_a_case_rejects_a_different_status`,
  `a_runner_absent_from_the_list_is_not_found`, and
  `a_signature_refuses_a_changed_body`. Original positives are
  `a_fact_read_from_a_case_compares_its_status_and_message`,
  `a_listed_runner_is_found_by_its_uuid`, and
  `a_signature_holds_only_for_the_body_it_signs`. Preserve all six assertions,
  every input and expected value; factor their repeated setup into private helpers.
  The fact's negative case changes only Exchange.status to405, retaining its409head
  and message; the runner's negative lookup stays `absent`; the signature's negative
  body stays `body!` under the original signature for `body`. No new case or world.
  This belongs only to the already-changing06.4, never the completed06.2move or06.3.

```rust
// tests/forge/conformance/src/tests.rs
#[test]
fn unquoted_log_times_keep_fraction_width_offset_and_identity() {
    let mut mask = Mask::new(3000);
    let raw = b"2026-10-18T12:34:56.1234567+05:30";
    assert_eq!(mask.bytes(b"\"2026-10-18T12:34:56.1234567+05:30\""), b"\"2020-01-01T00:00:01+05:30\"");
    assert_eq!(mask.bytes(raw), b"2020-01-01T00:00:01.0000000+05:30");
    assert_eq!(mask.bytes(b"2026-10-18T12:34:57.89Z"), b"2020-01-01T00:00:02.00Z");
    assert_eq!(mask.bytes(b"\"2026-10-18T12:34:57.89Z\""), b"\"2020-01-01T00:00:02Z\"");
    assert_eq!(mask.bytes(raw), b"2020-01-01T00:00:01.0000000+05:30");
}
```
Proof: process; `scripts/proof-forgejo-conformance.sh --forgejo "$WS/tmp/temper/forgejo/v17.0.0/forgejo-17.0.0-linux-amd64" --runner "$WS/tmp/temper/forgejo/runner-v13.2.0/forgejo-runner-13.2.0-linux-amd64" --scratch target/forgejo-conformance --out tests/forge/conformance`
  records again on both pinned binaries, once, in <=5 minutes of execution. Run 06.2's exact
  comparison after recording: 213 stable outcomes match digest d1f7ba2b…18347d; five timing
  outcomes/reasons print, each holds or fails. Total 218; no unchecked; all 55 setup and 40
  component facts hold.
Proof: unit; `cargo nextest run -p forgejo-conformance` prints
  the step-start library count plus5 (39from the checked34: two mask tests and three separated negative cases), each new unit<20ms. Historical measurements remain evidence, not a new pass.
Proof: unit; `cargo nextest run -p skein-forge --test 'forgejo_*'`
  and `cargo nextest run -p skein-forge --test fuzzy_forgejo --profile fuzzy`
  still print 61 and 4 passed, with 06.3's times and ceilings.
Don't: fix the base's Forgejo quirks; mask by replacing the whole body; accept arbitrary UUIDs
  merely because decoding ignores them.
Size: about0lines moved,80new production lines, plus about35test/helper lines for the three preserved assertion-pair splits.
Removes: the gap for unquoted times, through recording; quoted behavior stays.

## Done when

- `cargo nextest run -p skein-throwaway-forgejo` prints
  16 passed, 1 skipped; `cargo nextest run -p forgejo-conformance`
  prints39passed (checked34library baseline plus5; retain any further landed identities). Tests needing the pinned Forgejo/runner stay outside the default suites.
- `cargo nextest run -p skein-forge --test 'forgejo_*'` prints
  61 passed; `cargo nextest run -p skein-forge --test fuzzy_forgejo --profile fuzzy`
  prints 4 passed. Moved names and costs match the findings' inventory.
- `sh -c 'rg -n -i "temper|serde" testing/skein-throwaway-forgejo tests/forge/conformance crates/skein-forge/tests | wc -l'`
  prints 0 (README Done when 5, extended to serde).
- `sh -c 'rg -n "from_env|TcpStream|thread::sleep" testing/skein-throwaway-forgejo/src | wc -l'`
  prints 0; environment reads occur only in the opt-in test file.
- `sh -c 'rg -n "env::var|TEMPER_FORGEJO" tests/forge/conformance/src testing/skein-throwaway-forgejo/src | wc -l'`
  prints 0; the opt-in test file alone reads SKEIN_FORGEJO(_RUNNER).
- `bash -o pipefail -c 'set -eu; digest=$(awk '\''BEGIN { split("merge-old-head-is-409-head-out-of-date delivery-pull-reads-not-mergeable-while-checked delivery-pull-is-settled-when-its-push-is-delivered delivery-pull-is-checking-when-its-push-is-delivered webhook-deliveries", a); for (i in a) timing[a[i]]=1 } /^#/ || NF==0 { next } { name=$2; sub(/:.*/, "", name); if (seen[name]++ || $1 !~ /^(holds|fails)$/) exit 1; n++;   if (name in timing) { t++; print "timing " $0 > "/dev/stderr" }   else { s++; h+=($1=="holds"); f+=($1=="fails"); print $1, name } } END { if (n!=218 || s!=213 || h!=207 || f!=6 || t!=5) exit 1;   print "facts=218 stable=213 holds=207 fails=6 timing=5 unchecked=0" > "/dev/stderr" }'\'' "$1" | LC_ALL=C sort | sha256sum); printf "%s\n" "$digest"; test "$digest" = "d1f7ba2bb323b8944e05d2e05bc1c83396fb7bfffadbcefea4faad64ec18347d  -"' sh tests/forge/conformance/facts.txt`
  prints the five timing outcomes/reasons and `facts=218 stable=213 holds=207 fails=6 timing=5
  unchecked=0`, then `d1f7ba2bb323b8944e05d2e05bc1c83396fb7bfffadbcefea4faad64ec18347d  -`. It reads
  committed facts; no final recording. All 55 setup and 40 component facts hold, covered by the
  digest.

