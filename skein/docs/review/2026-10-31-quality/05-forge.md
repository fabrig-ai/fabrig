**Code-quality review: skein-forge (crates/skein-forge and tests/forge/github)**

The crate's size is mostly earned, but about 10% of production code (2.5–3.5k lines) and about 1k test lines could go. Most of it is copy-and-adapt between the two dialects and read plans that don't follow the house state-machine rule. I made no edits and ran no builds or tests.

Paths: `S` = `skein (at 3371027)/crates/skein-forge/src`, `T` = `skein (at 3371027)/tests/forge/github/src`.

## 1. Scope read
- **Closely:**
  - `api.rs`, `measure.rs`, both `decode/walk.rs`, `github/decode.rs` and `decode/issues.rs`, `forgejo/client.rs`.
  - `forgejo/server.rs`, `github/server.rs`, `server/{people,issues,writes}.rs`.
  - In both dialects: `plans/{plan,tables,failure,reading}.rs`, `plans/reads.rs`, plus `github/plans/reads/done.rs` and `forgejo/plans/reads/size.rs`.
  - `github/header.rs`, both `id.rs` and both `credential.rs`.
  - Tests: `github/tests/{fixtures,reads,pulls,writes,job}.rs`, `forgejo/tests/fixtures.rs`, `forgejo/plans/tests{,/edges}.rs`, `src/tests.rs`, `tests/common`, `tests/suite/github_plans.rs` (top part).
  - In `T`: `cases.rs` (part), `runner.rs` (`plan_as`, `decode`), the top of `tests.rs`.
- **Skimmed:** both write plans, deliveries, setup documents (orgs, hooks), forgejo `turn`/`actions`/`contents`, github app/log/protection decoders, the remaining tests.

## 2. Scores
1. **Cleanliness 3**: little dead code. Noise from 39 placeholder docs, about 220 lines of lint waivers in tests, and derived state.
2. **Organization 2**: the two dialects are not laid out in parallel, and GitHub imports Forgejo as its shared library.
3. **Taste 3**: careful, bounded and exhaustive; but decoders branch on `==` chains and decide by elimination.
4. **Naming 3**: mostly follows the design, with a few swaps against it (`Entry`/`TreeEntry`, `url`/`link`, `client`, `Drop`).
5. **Primitives 2**: four JSON walkers, the LRU hand-rolled four times, `Failure`/`Body`/`Heard` defined per dialect and again in the connection.
6. **Elegance 2**: read plans are bags of fields with a stage tag, against programming-model §5.4.
7. **Economy 2**: about 3k production and about 1k test lines are recoverable.
8. **Complexity 3**: complexity is earned by the breadth; still, some layers only forward, and some state could be derived.

## 3. Strengths
- Forgejo's test harness is the model to copy. `forgejo/tests/fixtures.rs:254-374` has named wire fixtures (`fixture`/`specimen`), sizes with their derivations and one `Run` driver. `edges.rs` job_failures rows carry only data.
- Forgejo shows the compact shape the rules allow: one `server::Answer` enum with one `encode` (`forgejo/server.rs:83-160`), and one `DecoderState` enum (`forgejo/client.rs:121-135`).
- Neutral shared modules exist where they were made: `directory.rs`, `marker.rs`, and `measure.rs` with compile-time width asserts.
- `github/failure.rs:5-40` names each refusal text and cites the conformance case that checks it.

## Area questions, in brief
- **Is the size earned?** Mostly. Production is 29.5k lines:
  - the wire layer is about 17.3k (59%), of which the fake's server side is 5.7k, as the design intends;
  - plans are about 8.9k (30%).
- **Per-field decode boilerplate.** Each field appears in 4–5 places: a table row, a setter arm, a zero-value initialiser, a document struct field, and the fake's writer (`key` plus value). There are about 440 table rows and about 430 writer `key()` calls. Total is about 2.8–3.2k lines, roughly a fifth of the wire layer and 10% of production. By hand that cannot shrink much. A generator would, but skein-codegen targets binary codecs (unsure whether it could cover JSON).
- **Is the vocabulary neutral?** Mostly; finding 8 has the leaks.
- **Module granularity:** inconsistent (finding 6).
- **In-crate tests:** Forgejo's are good, GitHub's are not (finding 9).

## 4. Findings (ranked by payoff)

**1. Read plans are field bags with a stage tag that keep re-matching `api::Read`.** Elegance, Primitives. Severity: style, highest payoff.
- **Evidence, GitHub:**
  - `github/plans/reads.rs:27-48` is a private copy of `api::Read`.
  - `:49-76` is a flat 26-variant `Stage`.
  - `:85-110` holds 25 fields (issues, pulls, checks, collaborators, remarks, reviews, directory, files, protection…), each meaningful for some reads only.
  - `reads/done.rs:26-160` re-matches all 19 variants of `self.read`. One list appears twice, the second time under a guard.
- **Evidence, Forgejo:** `forgejo/plans/reads.rs:50-52` holds three mutually exclusive `Option` sub-plans (`turn`, `contents`, `actions`), probed one after another at `:262-296`.
- **Evidence, across both dialects' plans:**
  - 81 `self.stage =` and 138 `self.stage ==`/`!=`, but no `mem::replace` transitions.
  - 29 exhaustive `api::Read` lists.
  - About 67 `if let`/`let…else` on the crate's own enums, e.g. `reads/people.rs:6,23,47` and `reads/job.rs:8-52`. Rules §10.2/10.3 exclude these.
- **Why it matters:** §5.4 says "no field is meaningful in some states only". Adding a read touches about a dozen sites. `let Read::X = self.read else { InvalidAnswer }` turns a plan bug into what looks like a peer error.
- **Fix:**
  - GitHub: `enum Plan { Issues(issues::Plan), Pulls(..), … }`, each with a `Stage` enum that holds its own rows.
  - Forgejo: `enum Read { Json(..), Contents(..), Actions(..), Turn(..) }`.
  - Repository resolution becomes one shared prefix state.
- **Size:** about −800 to −1,200 lines (the upper figure is uncertain).

**2. Four hand-written JSON field walkers.** Primitives, Economy. Severity: style.
- **Evidence:**
  - `forgejo/decode/walk.rs` (559 lines) and `github/decode/walk.rs` (1,250 lines; its header says "independent of Forgejo's tables") have the same parts: `Field{key,read,required}`, `fields`/`element` tables, `Frame{touched,seen}` masks, a `Want` machine, and the required-field check (`:473` vs `:1113`).
  - A third walker: `github/server/writes.rs:113-375` (`Parser`).
  - A fourth: `server/queries.rs:115-149` tokenizes the whole body into a `List<Token>`.
  - Per-node rules are hard-coded in the walkers instead of in the tables: `forgejo walk.rs:318-338` chooses key names by kind, `:450-454` treats null as seen for "statuses"/"creator"/"tree", and `github walk.rs:739-790` hard-codes AppEvents, Contexts, GraphqlErrors and LogRows bounds.
  - skein-json already has a static-path walker, `crates/skein-json/src/collector.rs:25-90`, which skein-llm uses.
- **Fix:** one crate-level walker over a union `Node { Forgejo(..), Github(..), GithubBody(..) }`. Move the per-node rules into the table (`Read::Strings{cap}`, `null_seen`). Alternatively, evaluate `collector` (unsure whether it handles streamed rows).
- **Size:** about −600 to −900 lines.

**3. GitHub uses Forgejo as its common library, and copies the rest.** Organization. Severity: style.
- **Evidence:**
  - 44 references in 26 GitHub files to `forgejo::{id::unsigned, time, path, document::Body, decode::Malformed}`, e.g. `github/decode.rs:7` and `github/request.rs:455`.
  - Copied instead of shared:
    - hex digit helpers: `github/id.rs:15-24` = `forgejo/id.rs:28-34,62-68`;
    - body/revision/provenance: `github/plans/reading.rs:6-54` ≈ `forgejo/plans/reading.rs:63-116`;
    - `Failure` plus `transport()`: `forgejo/plans.rs:39-47` = `github/plans.rs:25-31`;
    - the forwarding `Plan{Read,Write}`: `forgejo/plans/plan.rs:27-98` vs `github/plans/plan.rs:42-142`;
    - the credential prefix (the GitHub copy uses a bare `7usize`).
- **Fix:** move these to neutral crate-level modules beside `directory.rs`.
- **Size:** about −200 lines, and github no longer imports forgejo.

**4. The LRU table is hand-rolled four times, and the copies have drifted.** Primitives. Severity: risk.
- **Evidence:**
  - `forgejo/plans/tables.rs:47-89` and `github/plans/tables.rs:57-106` hold four copies of the eviction loop.
  - Forgejo's clock `expect`s at `u64::MAX` (`:37`) and `expect`s room even at capacity 0. GitHub saturates (`:47`) and returns early at capacity 0.
  - §10.2: "what is missing goes into lib".
- **Fix:** `skein_lib::Lru<K,V>`.
- **Size:** about −100 lines, and one behaviour.

**5. Two measures of the same answer.** Economy. Severity: risk.
- **Evidence:**
  - `measure.rs:151-254` and `forgejo/plans/reads/size.rs:28-144` both size `api::Answer`, and `tests.rs:100-103` asserts that they agree.
  - `size.rs:107-121` already calls `measure::answer` after a `rows(...)?` whose value it discards.
  - It also leaves out `dequeued`, which `measure::pull` counts.
- **Fix:** `measure::answer` plus one check of the longest list against the limit.
- **Size:** about −130 lines.

**6. The two dialects took opposite shapes for the same jobs.** Organization, Naming. Severity: style.
- **The engine side:** Forgejo calls it `forgejo/client/` (15 files), which collides with the skein-forge-client crate. GitHub calls it `request.rs` plus `decode/`.
- **The decoder:**
  - Forgejo has a `DecoderState` enum.
  - GitHub has a struct holding every sub-state at once (`github/decode.rs:35-58`), although its doc says "Retains only the selected document".
- **The fake's answers:**
  - Forgejo has one `encode`. GitHub repeats the 9-line measure-then-write in 19 places (`github/server.rs:263,302,326`).
  - `server/people.rs:15-25` merges two unrelated documents behind an `Option` just to share that code.
  - Six types carry lifetime parameters (`Answer<'a>` in `issues.rs:11`, `pulls.rs:8`, `app.rs:108`); §10.3 excludes these.
- **Same name, different job:**
  - `reads/size.rs` is branch admission in GitHub but the answer measure in Forgejo.
  - Each dialect has both `failure.rs` and `plans/failure.rs`, with `classify` on opposite sides.
  - The hint mapping is in `forgejo/plans/hint.rs` but in `github/delivery.rs:121`.
  - Two enums are named `Drop`, which shadows the prelude trait.
- **Granularity:** `github/pin.rs` (7 lines), `decode/people.rs` (17), `writes/merge.rs` (23) sit beside `request.rs` (1,319) and `walk.rs` (1,250).
- **Design:** forge-github.md §2 says GitHub "holds the same" list as forge.md §4.1.
- **Fix:** one module map for both dialects, Forgejo's enum shapes for the decoder and the server, and files under 50 lines folded into their callers.
- **Size:** about −150 lines.

**7. The GitHub decoder routes each node twice and parses numbers twice.** Taste, Economy. Severity: style.
- **Evidence:**
  - `github/decode.rs:225-298` and `:437-525` both list the same five node groups (54 nodes). `Protection` and `PullGraphql` are already nested enums; the other groups are not.
  - The walker passes the raw `Token` on, so setters call `unsigned()` about 34 times with `expect("walker checked integer")`.
  - `decode/issues.rs:96-125` decides by elimination: any unmatched string is a time, and any `True` sets `merged`.
  - About 100 `node ==`/`kind ==` comparisons on own enums across the decoders.
- **Fix:** nest every node group, emit a typed `Value` as Forgejo's walker does, and match exhaustively.
- **Size:** about −120 lines; a new field then fails the build where it is not handled.

**8. Leaks and naming slips in the neutral vocabulary.** Naming. Severity: style.
- `api.rs:463` documents `JobAttempt` as "A Forgejo Actions job attempt".
- `Entry`/`EntryKind` (`:197-215`, a directory entry) and `TreeEntry`/`TreeEntryKind` (`:590-604`) are swapped relative to forge.md §3.6, where `Entry` is the tree's entry.
- `Status.url` (`:457`) vs `Write::Status.link` (`:149`); the design says "link".
- The `Commit` alias is used for tree object ids (`:130`, `:595`).
- 39 placeholder docs that restate the name ("/// Read item.", "/// Return done.").
- In `github/header.rs`:
  - `RateFields` duplicates `api::Rate`.
  - `Head.rate` is derived from three other fields (`:200-203`), so `remaining()`/`reset()` at `:120-133` branch to the same value either way.
  - `Cursor` and `Etag` duplicate `api::Validator`, with a bare 128.

**9. GitHub in-crate tests retype the harness, branch on rows, and test several behaviours at once.** Tests. Severity: style.
- **Retyped harness:**
  - `struct Call{plan,tables,bounds}` and its helpers appear in `github/tests/{reads.rs:40,job.rs:39,pulls.rs:78,checks.rs:54}` and `tests/suite/github_items.rs:80`.
  - `const KEY` is retyped 6 times and `bounds()` 11 times.
  - `tests/suite/github_plans.rs:23-40` retypes the limits in `fixtures.rs:13-34`.
- **Whole-body decode loop written 9 times:**
  - twice in one file (`fixtures.rs:60-104` and `:145-185`);
  - the production copy at `github/decode/delivery.rs:274-322` hand-rolls `Intake::meet`.
- **Row branching:**
  - 40 loops over `[false, true]`, many branching on the flag (`reads.rs:112-124`, `pulls.rs:643-654,688-708`, `writes.rs:924-960`).
  - `pulls.rs:645` computes the expected value with the code under test.
- **Several behaviours per test:** `github/tests.rs:167` holds five table tests. `src/tests.rs:443-485` uses bare sizes 408/201/90.
- **Names:**
  - 190 of 224 GitHub tests are prefixed `github_`.
  - `forgejo/plans/tests.rs:37` `github_rate_and_validator_are_none_in_forgejo_plans` asserts neither.
  - `reads.rs:1` refers to "Task 18".
- **Lint waivers:** 65 `#![expect]` in 20 files under `crates/skein-forge/tests`; one set in `suite/main.rs` (the crate root) would do.
- **Size:** about −600 to −900 lines.

**10. The GitHub conformance retypes what it should be checking.** Tests. Severity: risk.
- `T/cases.rs:439,443,451,499,566,1084,1089` retype about 13 refusal texts that `github/failure.rs` already names. If a constant drifts, the fact still passes. Assert against the constants.
- `T/tests.rs:25-45` reinvents `skein_scratch::Scratch` under a hard-coded `/tmp`, although skein-scratch is already a dev-dependency.
- `T/runner.rs:681-700` puts test-only fault injection, keyed on strings, inside the plan driver.

## 5. Cross-area suspicions (unverified outside my area)
- `crates/skein-forge-connection/src/connection.rs:55-80,815-840` wraps every per-dialect type (`Request`, `Decoder`, `Body`, `Heard`, a third `Failure`, `JsonStep`). That is a direct cost of findings 3 and 6.
- The loop that drives a plan over HTTP exists in the connection and in `T/runner.rs:668`; probably also in tests/forge/machine.
- The whole-body tokenizer loop also appears in `tests/forge/setup/src/decode.rs`, `machine/src/{memory,pipe}.rs`. A shared helper could live in skein-json.
- The same `[lints]` block is copied into each `tests/forge/*/Cargo.toml`.
- skein-llm decodes JSON with skein-json's `collector`; the forge kit does not.
- skein-http has `fn decimal` twice (`server/head.rs:380`, `client/head.rs:281`); forge's `id::unsigned` is a third.

## 6. Rule costs
- **No closures:** the measure-then-write pass must be written once per document. Forgejo's single-enum shape shows the minimum is one site per dialect; GitHub has 19.
- **No generics:** a shared walker needs a crate-wide `Node` union, about 30 lines of dispatch.
- **Exhaustive matches:** the 29 `api::Read` lists (about 580 lines) look forced by the rule, but they come from the plan design (finding 1), not the rule.