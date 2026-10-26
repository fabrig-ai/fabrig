# Session 11: the tools' forge parts

The forge's words move from `temper-client-view` to
`temper-client-domain::forge::words`, with no behaviour change (11.1).
Then the tools run `adopt` through the form's checks and the forge's ask
and answer, and print forge parts in reads (11.2). Needs at start:
**the tools' root**. This session provides **the tools' adopt**, needed
by the next plan's adopt story, its landing through adopt and its process
tier. It replaces the former temper-03.5 and temper-03.6; the
steps are `temper-11.1` and `temper-11.2`.

## Read first

- temper `docs/design/client/tools.md` at `fe0ec316`: 4, 5.2 (forge parts),
  5.3 (`adopt` and its defaults), 5.6, 6.1 to 6.3, 7, 8.2, 11.1 to 11.3, 12;
  `docs/design/client/README.md`: 4.1, 4.2, 4.4 to 4.7, 5.2.
- jig `docs/design/client/domain/client.md`: 4, 6, 11;
  `docs/design/client/domain/root.md`: 3, 5, 15;
  `docs/design/client/domain/contract.md`: 3.1, 3.3.
- This plan: `temper/03-tools-root.md` (Root, RUNNER, Parts, Printed,
  kept results, client-refusal exits); `temper/02-tools-table.md`
  (`adopt::ROW`, Call, Name, limits); `findings/temper-area-1.md` 3 and 4.
- `decisions.md`: the plan's own decisions 10, 14, 15 and "For the user" G;
  design questions 21 to 34 whole, especially 26, 27, 29 CORRECTED, 30
  and 33 with Gaps closed.
  Corrected 29 supersedes the siblings' name-bearing kept lines and
  durable-name gap; 33 limits recovery to the signed-in party.
- Code: `crates/temper-client-domain/src/forge.rs` (`act_on`, `told`),
  `src/forge/form.rs` (`form`, `write`, `broken`, `adoption`),
  `src/forge/family.rs` (`request`, `answer`);
  `crates/temper-client-view/src/forge/words.rs`, `src/forge.rs`,
  `src/forge/bindings.rs`, `src/forge/page.rs`, `src/words.rs`.
- Seams only, no work from these drafts: skein `docs/design/draft/forge.md`
  3 and 7; temper `docs/design/draft/forge-kit.md` 2 and 4. The forge kit
  may move this work's files; the exact hotspot lists below are its rebase list.
- skein foundation `programming-model.md`: 3, 5.1, 6.3, 10;
  `testing-strategy.md`: 2.1, 8, 10; temper `docs/development/workflow.md` 2.

## What the design asks

- **client/tools.md 4:** reuse the forge's adoption ask, owner and words, with
  no page or view in the tools. jig's words say jig's refusals; the
  forge's words say client/README.md 4.1's resources and 4.7's outcomes.
- **5.3, client/README.md 4.4:** the form's fields and defaults: forge 1, owned,
  prefix `temper/`, home and CI on; owner, name and landing required.
  Context turns home off. Without CI there must be at least one check;
  checks are positive, distinct gate numbers within the client's bound.
  Every form check runs before sending; a bad call exits 1, nothing sent.
- **5.2, client/README.md 4.1 and 4.2:** resources, patterns, connector and
  kinds in the forge's words; an unserved part says “Shown on the forge.”
  No read or watch is sent for the forge family.
- **6.1 to 6.3:** adopted exits 0, including protection unknown with its
  sentence; unknown past retention and all client-own failures exit 1;
  other engine outcomes of 4.7 exit 2. A kept line has only operation
  and outcome, never a name or either key, in text and JSON.
- **4, 7:** one asker per owner; the root allocates call prefixes above
  all Start's kept keys. Adopt's ask comes from the call and live `me`
  alone, under the same numbering rule as the runner's keys.
- **5.6, 8.2:** only `me`'s party's restored requests settle before the
  new call; another party's stay parked, reported once. Unread records
  are counted, not decoded or listed. **11.3:** the real loop proves both.

## Choices this plan makes

- **Two landings:** 11.1 is a move only, with unchanged view and terminal
  goldens. 11.2 consumes the shared words and extracts ask/answer from
  the existing side. The view keeps its form state and rendering.
- **One phrase owner and number per phrase.** The 48 adoption rows keep
  their numbers from 1000; resource/kind rows keep their existing forge
  numbers. `Word` remains the adoption vocabulary; `ResourceWord` is
  the resource/kind vocabulary moved out of the view's larger `Word`.
  UI wrappers render these shared words, without duplicate English rows.
- **The adopter** is `temper-client-tools/src/forge.rs`, under
  `Owner::Forge.asker()`. It holds per-call adoption data and pending
  keys, no `forge::Side`. temper 03 owns Root, RUNNER (`Owner::Runner`,
  3), Parts and Printed/Kept. Root allocates adopt's call prefix as it
  does the runner's: high 32 bits in every key/handle, local numbers
  below, above all kept prefixes at Start, starting at one if empty;
  zero stays the me watch's. Prefixes are never reused; caller `number`
  is only correlation. No per-call asker or Side's random serial; overflow exits 1.
- **Form checks:** build `forge::form(project)`, write typed fields
  through the shared `write`, set role/home/CI, then `broken` and
  `adoption`. Do not assign private `Field.typed` or duplicate checks.
  Expose `form::write` as `forge::write`; the form module stays private.
- **Request identity:** after live `me`, resolve the call's project,
  fill the form's fixed defaults and send `forge::ask` with
  `name: Some(crate::key::engine(name))`, temper 02's derivation. The
  encoded ask depends on the call and `me` alone, under the shipped
  limits, never clock, seed, call prefix or DIRECTORY. No local wall
  conversion: temper 03.1 already provides **temper on jig's named send,
  turn read and spans**. Changed inputs from `me` can change a named ask.
- **Exits distinguish client and family outcomes.** Intercept jig's
  Held, Limit, Unconfirmed, NotKept and Unknown before `forge::answer`.
  Each exits 1; Unknown uses the adoption's 4.7 sentence. Unconfirmed
  and a detected client key conflict say “tools' bug; report it as a
  finding”; keep jig's conflict assertion, checking keys before Send.
  Local form/encoding failures exit 1, unsent; a decoding failure also
  exits 1. A forge refusal
  named TooLarge or Unreachable is an engine outcome, exit 2, as is
  its entrance key conflict. Never infer the exit from the word alone.
- **Kept output:** `kept: adopt <outcome as the act's>`, or the restored
  jig operation (`stop`, for example) through temper 03's renderer.
  JSON has exactly `operation` and `outcome`, one object per line on
  stderr; no `name`, even null, and no application or engine key.
  Read adoption context from its kept ask; keep no name map or records
  of the tools' own. `--name N` remains the repository name in the ask.
- **Recovery belongs to Root and the next plan's host.** Use `client.pending(key)`'s
  party to classify restored adopts after `me` is live. `Root::kept()`
  counts only that party's unanswered restores; the host also awaits
  their erases' Stored. Parked parties do not block the call, but their
  names can cause Held, table-wide, exit 1. The shared root reports
  `kept: N for party P, unsent while it is not signed in here` (JSON
  `party`, `unsent`) and `dropped: N kept records these tools cannot
  read` (JSON `dropped`); the adopter adds neither a counter nor a print.
- **Pending is the next plan's host's:** its forge row says `adopt`, `party N`,
  first-sent wall time or `not sent`; JSON rows use `operation`, `party`,
  `first_sent`, with `unread` beside `pending`. Unknown versions are
  counted only, exit 0; an unreadable small-store file exits 1 (5.6).
- **Whole paths:** the moved `joined` keeps the view's truncation
  behaviour. The tools pass a bound computed from all bounded segments,
  so it never truncates their resource paths (client/tools.md 6.4, 12).
- **Proofs:** unchanged terminal goldens prove the move; a unit step
  proof drives adopt through the real jig client. The new tools' joint
  driver is the next plan's, so it is not a start dependency. That plan's
  adopt story, its landing through adopt and its process tier each
  need **the tools' adopt** as well as their own prerequisites. Its
  real-loop stories must carry 33's unread-store and parked-party checks, including
  whether the native shell retains dropped bytes; a repeat is a jig finding.

## Rules that bind it

- **Forge hotspot:** change the exact files named below, behind the
  existing family interface. No fake forge, forge component, engine
  forge client, fake LLM or provider change belongs here.
- **Boundary:** the adopter and forge words stay outside
  `crates/temper-client-tools/src/jig/`. No dependency on the view;
  reuse temper 03's Parts seam and temper 02's shipped limits.
- **No design or STATUS edits.** Step code follows the subset; no I/O,
  clock read or new persistent store. Unit tests stay under 20 ms,
  unoptimized; no profile changes. Extend the shared fixtures.

## Increments

### 11.1 The forge's words in `temper-client-domain`
Scope: move client/README.md 4.1, 4.4 and 4.7's words to the domain, as client/tools.md
4 places them; keep phrase IDs, bounds, text and view behaviour; step `temper-11.1`.
Needs: nothing beyond the session's start (the tools' root).
Files: forge hotspots, every path:
- `crates/temper-client-domain/Cargo.toml` (skein-ui);
- `crates/temper-client-domain/src/forge.rs` (the words module);
- `crates/temper-client-domain/src/forge/words.rs` (new);
- `crates/temper-client-view/src/forge.rs`;
- `crates/temper-client-view/src/forge/words.rs`;
- `crates/temper-client-view/src/forge/bindings.rs`;
- `crates/temper-client-view/src/forge/page.rs`;
- `crates/temper-client-view/src/words.rs`;
- `crates/temper-client-view/src/root.rs`;
- `tests/client/src/app.rs`.
Also `Cargo.lock`, regenerated for the domain's added dependency; no repin.
Shape:
- Move adoption `Word`, `ROWS`, `id`, `extend`, `name_bytes` from the
  view's `forge/words.rs`; `owner` from `forge/bindings.rs`; `Resource`,
  `resource`, `joined`, `full_name` from `forge.rs`, into domain `forge::words`.
- Move the forge resource/kind variants, their IDs and English from
  view `words.rs`: Connector, Repository, Branch, Pull, Issue, Adopted,
  Path, Read, Push, OpenPull, MergePull, Review, Status, Comment,
  CreateIssue, CreateBranch, EditPull, UpdatePull, ClosePull, EditIssue,
  Reviewers, DeleteBranch, OtherKind, Shown; move `kind` with them.
  The view's unrelated words and overrides remain its own.
- Keep `say`, `text`, `capacity`, `path_text`, `drawn`, `owner_part` in
  the view; they call the domain's phrase IDs and resource helpers.
  `Words::english` composes the domain's rows. No old-path aliases.
- Move the pure mappings to `Word` into domain `forge::words` too, each a
  `#[must_use]` function of domain values, with no `Builder` or view
  `Words`: a `Broken` to its Word, `Long`'s bound its Number hole
  (`page.rs`'s `broken`); `Kinds` to its kind Words in 4.7's order, read,
  push, open, land, review, status, comment, issue, branch (`kinds`); a
  `Role` to Owned, AdoptedRole, Fork or Context (`outcome`'s); a path to
  its `Resource`'s Word and holes, the bound passed as `most`
  (`path_text`'s). The view's `broken`, `kinds`, `outcome` and `path_text`
  keep their names and output and call them; 11.2's Parts and outcomes
  call the same, so the tools copy none. Their lines count as moved.
- `Unshowable` (id 24, "Cannot show this part.") stays in the view's
  words: it is the view's word for any part it cannot show, not a forge
  word, though `ALL` lists it after `Shown`.
- Update `root.rs` and `tests/client/src/app.rs`, the external callers
  of `forge::bindings::owner`, to domain `forge::words::owner`;
  bindings and words use that same function. No phrase is renumbered.
Test shape: no new file or test kind. Existing view forge tests and
terminal adoption goldens keep their names and bytes; reuse their
fixtures and shipped limits. The move's checks include resource paths,
kinds, validation marks and outcome sentences, each under 20 ms.
Proof: unit, once: `cargo nextest run -p temper-client-shell-terminal
-p temper-client-view -E 'test(/adopt|forge/)'` (under 1 s); unchanged
adoption goldens show the view still composes the same phrases.
The words move behind the same view; 11.2 connects their tools consumer.
Don't: change a phrase, ID, hole or bound; copy rows; change the form's
rules, the side's behaviour or the family encoding.
Size: at most 100 new production lines; moved lines counted separately
by file, with no behaviour hidden in the move.
Removes: the view's moved rows and helpers, including bindings' owner.

### 11.2 `adopt`, and the forge's parts in reads
Scope: client/tools.md 5.3's checked adopt, named send and outcomes; 5.2's
forge parts through Parts; client/README.md 4.7's words and client/tools.md 6.3's exits;
6.1, 6.2 and 8.2's restored adopts; step `temper-11.2`.
Needs: nothing beyond the session's start; follows 11.1's word move.
Files: forge hotspots, every path:
- `crates/temper-client-domain/src/forge.rs` (ask, answer, write export);
- `crates/temper-client-domain/src/forge/form.rs` (write's visibility);
- `crates/temper-client-tools/src/forge.rs` (new adopter and part rendering);
- `crates/temper-client-tools/src/root.rs`;
- `crates/temper-client-tools/src/lib.rs` (module declaration);
- `crates/temper-client-tools/src/tests.rs` (test module declaration);
- `crates/temper-client-tools/src/tests/fixtures.rs` (extend temper 02/03's);
- `crates/temper-client-tools/src/tests/adopt.rs` (new);
- `crates/temper-client-tools/src/tests/golden/adopt_adopted.txt` (new);
- `crates/temper-client-tools/src/tests/golden/adopt_refused.txt` (new);
- `crates/temper-client-tools/src/tests/golden/forge_part.txt` (new).
The tools' `skein-ui` dependency is temper 03.2's (its Files); 11.2 adds none.
Shape:
- Fill the form from Call, each text through `forge::write`: Forge with
  `Typed::Forge`, Checks with `Typed::Checks`, the others `Typed::Bytes`.
  Use the form's defaults for absent flags; Context always clears home;
  `--no-home` and `--no-ci` clear their switches. `broken` marks print
  once each in the form's words, exit 1 without sending; `adoption`
  supplies the valid record. Long-field marks must not send a retained default.
- Extract `forge::ask` from `act_on`'s family encoding and project scope,
  and `forge::answer` from `told`'s Answered arms. The side calls them;
  its behaviour stays unchanged. Family records continue through
  `family::{request,answer}`; no wire change or second codec.
- Root routes Call to the adopter after `me` is live, and Told by
  `Owner::Forge.asker()`. Send with its call-prefixed key and the named
  engine key, as Choices says. Keep adoption data until its outcome,
  then emit `Printed { number, printed }`; preserve the caller's number.
- For `Told::Kept`, decode the current forge request with the existing
  generated `v1::Adopt::decode`, retaining its outcome context by key;
  no second codec. On answer, emit `Kept { line }` once through the same
  renderer, in `Root::kept_json`'s mode, with operation/outcome only.
  Pass party accounting and Dropped to the shared root as Choices says;
  the host's `pending` uses `adopt` for the same declared family.
- Adopted exits 0, with home and kinds in 4.7's order and protection's
  extra sentence when unknown. CoreRefused, ForgeRefused and
  EntranceRefused exit 2. Unknown exits 1 with “It is not known whether
  O/N was adopted.” Client-own outcomes follow Choices before decoding.
  JSON is `{"adopted":{...}}`, or the outcome's snake-case name and fields;
  this is also the kept object's `outcome`, without inventing a name.
- Parts uses the shared resource, pattern and kind words, through 11.1's
  mappings in domain `forge::words`, as do the marks and outcomes above. Other forge
  parts say “Shown on the forge.” No family read/watch; paths stay whole.
Test shape: `src/tests/adopt.rs`, on `src/tests/fixtures.rs`'s root
harness and shipped limits. Extend `adoption_answer`, `sent_name`,
`sent_adoption`, `nothing_sent` and `part_read` there. The reference:
```rust
#[test]
fn adopt_prints_adopted_in_the_forges_words_and_exits_0() {
    let mut root = fixtures::signed_in(fixtures::PARTY);
    let call = root.submit(&["adopt", "--project", "7", "--owner", "acme",
        "--name", "shop", "--landing", "main", "--key", "adopt-shop"]);
    root.adoption_answer(call, fixtures::adopted_with_unknown_protection());
    let printed = root.answer(call).expect("answered once durable");
    assert_eq!((printed.status, &*printed.stdout), (0, fixtures::golden("adopt_adopted")));
}
```
Each under 20 ms: `each_broken_mark_exits_1_with_nothing_sent`,
`adopts_absent_flags_use_the_forms_defaults_and_context_clears_home`,
`each_refusal_of_4_7_exits_2_in_its_words`, `unknown_past_the_retention_exits_1`,
`adopts_client_failures_exit_1_and_impossible_answers_name_the_tools_bug`,
`a_named_adoption_has_identical_request_bytes_under_two_clocks_and_seeds`,
`adopt_uses_the_roots_call_prefix_above_all_kept_keys`,
`a_restored_adoption_prints_only_operation_and_outcome_once`,
`another_partys_adoption_stays_parked_and_its_name_is_held`,
`a_task_reads_forge_part_prints_shown_on_the_forge_with_its_resource`.
The restore cases use only jig's records, named and drawn keys, text and
JSON: exactly the two wrapper fields, no key/name there; outcome fields
keep their own names. Parked work does not prevent an unrelated call.
Compare asks without envelope keys, varying call prefixes and DIRECTORY too.
Proof: unit, once: `cargo nextest run -p temper-client-tools -E
 'test(=adopt_prints_adopted_in_the_forges_words_and_exits_0)'` (under 20 ms).
It proves the form, family encoder/decoder, jig client and rendered answer
connect. Existing domain/view form tests keep passing; the next plan's adopt
story connects the engine, and its landing through adopt the real loop and forge host.
Don't: hold Side; copy family codecs or form checks; touch a fake forge,
forge component, provider or LLM; cut a path to the view's text window.
Size: at most 300 production lines.
Removes: the root's forge-part fallback from temper 03; inline ask/answer
bodies in the side, replaced by the shared functions.

## Interface sketches

The domain owns the shared words and pure conversions. The tools own
per-call state and rendering. The adopter uses temper 03's Root events,
Parts and Printed unchanged; no forge type crosses into `src/jig/`.
Use `Root::client`, `Root::kept_json` and `Request::{Kept, Printed}`
from temper 03. Its `kept()` is the current party's recovery count
(corrected 29 and 33), never all parked requests. The adopter's
`Key.asker` is Forge's; its number uses Root's allocation rule alongside
RUNNER's. The engine name is a separate Send field, not that number.

```rust
// crates/temper-client-domain/src/forge.rs (11.2)
/// The validated adoption as the existing project-scoped family ask.
pub fn ask(adoption: &Adoption, limits: &Limits) -> Option<jig_client::Ask>;
/// A family or entrance answer, as `told` maps it: jig's `Done`, and each
/// outcome `told` takes as unknown, give `Some(Answer::Unknown)`; `None`
/// only when the family record or the limits cannot be decoded.
pub fn answer(limits: &crate::Limits, answered: &jig_client::Answered) -> Option<Answer>;
// crates/temper-client-domain/src/forge/form.rs (11.2), exported as forge::write
/// The form's bounded write; a long value leaves its mark and retained text.
pub fn write(form: &mut Form, text: Text, value: Box<[u8]>, typed: Typed, bound: u32, most: u32);

// crates/temper-client-domain/src/forge/words.rs (11.1)
/// Shared resource vocabulary; fields unchanged from the view's Resource.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Resource<'a> {
    /// Repository path, decoded without another read.
    Repository { host: &'a [u8], owner: &'a [u8], name: &'a [u8] },
    /// A branch's remaining path segments.
    Branch { host: &'a [u8], owner: &'a [u8], name: &'a [u8], branch: &'a [Box<[u8]>] },
    /// Pull number as the resource path spells it.
    Pull { host: &'a [u8], owner: &'a [u8], name: &'a [u8], number: &'a [u8] },
    /// Issue number as the resource path spells it.
    Issue { host: &'a [u8], owner: &'a [u8], name: &'a [u8], number: &'a [u8] },
    /// Two binary path segments, forge then repository, big-endian.
    Adopted { forge: u16, repository: u32 },
    /// Unrecognised path: render its segments joined by slash.
    Other,
}
#[must_use]
pub fn resource(path: &[Box<[u8]>]) -> Resource<'_>;
#[must_use]
pub fn owner(connector: u16) -> skein_ui::Owner;
#[must_use]
pub fn joined(segments: &[Box<[u8]>], most: u32) -> Box<[u8]>;
#[must_use]
pub fn full_name(owner: &[u8], name: &[u8]) -> Box<[u8]>;
```

## Questions to raise when reached

- **11.1:** a changed golden, phrase ID or hole bound in the move: raise
  it with the before/after, rather than recording a new golden.
- **11.2:** the landed table lacks a form field or supplies incompatible
  typed data: give the call and the failed check before changing either.
- **11.2:** a whole bounded resource path exceeds the tools' window:
  report both bounds; do not inherit the view's truncation silently.

## Done when

In `~/src/rust/temper`, against fetched `origin/main`. These read the
files that remain after this plan's sessions; proofs run at their steps.

- `git show origin/main:crates/temper-client-domain/Cargo.toml | grep -qx 'skein-ui.workspace = true'`
- `git grep -q -F 'Branch protection could not be read.' origin/main -- crates/temper-client-domain/src/forge`
- `git grep -q -F 'Shown on the forge.' origin/main -- crates/temper-client-domain/src/forge`
- `test -z "$(git grep -n -F 'It is not known whether' origin/main -- crates/temper-client-view/src ':!*/tests/*')"`
- `git grep -q -E 'pub fn ask\(' origin/main -- crates/temper-client-domain/src/forge.rs`
- `git grep -q -E 'pub fn answer\(' origin/main -- crates/temper-client-domain/src/forge.rs`
- `git show origin/main:crates/temper-client-domain/src/forge/form.rs | grep -q '^pub fn write'`
- `git grep -q -E 'forge::(write|broken|adoption)' origin/main -- crates/temper-client-tools/src`
- `git grep -q -E 'forge::(ask|answer)' origin/main -- crates/temper-client-tools/src`
- `git grep -q -F 'Owner::Forge.asker()' origin/main -- crates/temper-client-tools/src`
- `git grep -q -F 'key::engine' origin/main -- crates/temper-client-tools/src/forge.rs`
- `git grep -q -F 'Outcome::Held' origin/main -- crates/temper-client-tools/src/forge.rs`
- `git grep -q -F 'operation' origin/main -- crates/temper-client-tools/src/forge.rs`
- `git grep -q -F 'outcome' origin/main -- crates/temper-client-tools/src/forge.rs`
- `! git show origin/main:crates/temper-client-tools/Cargo.toml | grep -q 'temper-client-view'`
