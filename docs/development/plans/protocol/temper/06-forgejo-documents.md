# Session 06: Forgejo's documents, both sides

`temper-forge-forgejo` is Forgejo v16.0.5's API as documents: each
request of forge.md 3.2 and 3.5 as a value both sides share, the
client's encoder of it (method, target, JSON body) and the server's
decoder of it, and each answer as a document the client decodes from
skein-json's tokens as it streams and the server encodes with skein-json's
sized writer; Forgejo's error bodies and refusal messages; the headers
the reads need. It is step code linking skein-lib and skein-json only,
and names no temper domain type: the forge component (13) translates
these documents to the forge client's vocabulary, and the fake forge's
protocol layer (14.3) serves them. Each increment adds both sides of
its documents and tests them against the real Forgejo's transcripts
that session 05 committed: each transcript's body decodes to its
document, each request encodes to what the tool sent, each request the
tool sent routes on the server's side to the same value, and each
document the server encodes decodes back unchanged, with keys and types
the transcript has.

## Read first

- forge.md, sections 2, 3 (all), 9, 10 ("Step tests for
  `temper-forge-forgejo`") and 13.
- skein's `docs/design/json.md` at the pin, sections 3 (the tokenizer:
  `Next`, `Text(max)`, `Skip`, `Long`), 4 (the writer's two passes) and
  5 (decoding is the application's); `crates/skein-json/src/tokenizer.rs:180-300`
  and `writer.rs:1-160` at the pin; findings/skein-smith.md, B.
- `temper/05-forgejo-tools.md`, "Choices this plan makes" (the
  transcripts' bytes, the masking, targets and bodies as temper's client
  writes them) and 05.2's case list; `tools/forgejo-conformance/README.md`
  as 05.2 lands it.
- `temper/03-wire-codecs.md`, 03.1 (the `tests/codecs` world this
  session adds to).
- `crates/temper-engine-domain-forge-client/src/api.rs:1-420`: the
  vocabulary these documents serve (not linked here).
- code-organization.md, sections 2 (`temper-forge-forgejo`'s row), 3,
  5 and 8.

## What the design asks

- **The crate** (forge.md, 2): "`temper-forge-forgejo` depends on
  skein-lib and skein-json. It holds Forgejo's documents and nothing of
  temper's domains: each request's method, path, query and JSON body as a
  sized encoder; each answer's decoder, a small state machine over JSON
  tokens; error bodies ... Both sides live there, the client's for the
  engine and the server's for the fake forge (section 9), so the two are
  also fed transcripts of the real Forgejo (section 10)."
- **Conventions** (3.1): names in paths "percent-encoded, each as one
  path segment; a slash inside a branch's name is `%2F`. A file's path
  keeps its slashes"; times "read as the forge's clock, in nanoseconds
  since the epoch, at its resolution of a second. `since` is written in
  RFC 3339, in UTC, ... up to the largest (`u64::MAX` nanoseconds, in
  2554)"; commits "40 or 64 hexadecimal digits; anything else is a
  malformed answer ... written in lowercase hexadecimal"; paging "always
  explicit"; `X-HasMore`, `total_count`.
- **Decoding** (3.3): "Each answer's decoder is a state machine over
  skein-json's tokens with a bounded stack ... It keeps only the fields
  its answer needs, as plain values; every other field, nested object and
  array is skipped as it passes." "A name, a title, a context, a
  description, a link or a path is kept within `string_bytes`; a longer
  one makes the answer malformed"; "a body ... is kept whole within
  `body_bytes`. A longer one is carried empty"; malformed: "a document
  that does not decode, a field missing, a number out of range, a commit
  id of the wrong length, a string past its cap".
- **The threads** (3.2, `Read::Item`): "The call reads the whole thread,
  keeps the comments with ids above `after` as the answer holds them,
  and skips the rest as they stream, bodies unread."
- **The server's side** (9): "Each request is decoded, translated to the
  fake's `Read` or `Write` ..., and its answer encoded as Forgejo encodes
  it." The routes of 3.2 "and those of a forge's start and of
  resolution: `/api/v1/user`, `/api/v1/settings/api`,
  `/api/v1/repositories/{id}`, `/api/v1/repos/{owner}/{name}`,
  `/api/v1/users/search`."
- **Tests** (10): "each decoder against transcripts of a throwaway
  Forgejo v16.0.5 ..., against answers generated from the seed, and
  against mutations of both; golden bytes for each encoder; bounds at
  their limit and one past; a fuzz target for each decoder. Server
  encoders meet the client decoders, and the client encoders meet the
  server decoders." "Transcripts are format-faithful, content-irrelevant
  ... keys, nesting, types and status lines must match."

## Choices this plan makes

- **Module layout** (06.1, the shared interface 13 and 14 use):
  `limits`, `id`, `time`, `path`, `document`, `request`, `decode` (the
  pattern, with `decode/walk.rs`), `client` (with one file per family
  under `client/`: `start.rs`, `items.rs`, `reviews.rs`, `branches.rs`,
  `files.rs`, `actions.rs`, `writes.rs`), `server` (the same files under
  `server/`), `header` (06.4), `failure` (06.5). `mod_module_files` is
  denied: `client.rs` beside `client/`.
- **Documents carry what Forgejo says, typed, not translated:** times as
  `skein_lib::Wall`, ids as `id::ObjectId` with the format their digits
  had, bodies as `document::Body::{Kept, Long}`, Forgejo's enums as its
  own words. Revisions, provenance, keys' markers, the CI and permission
  mappings and the object format's check against a repository's row are
  temper's reading of these words: the forge component's (13.3).
- **The decoder pattern** (06.1, `decode` and `decode/walk.rs`): each
  answer kind is a state of `client::Decoder`, made by
  `client::decoder(&request, &limits)`; the caller feeds it the
  tokenizer's events one at a time and makes the demand each `Step`
  names. Inside, a walk
  (a `skein_lib::Stack` of open containers, at most `limits.json.depth`)
  and a static table per object (`&'static [Field]`: a key, how it is
  read, a bit of a `u64` mask of the fields seen). At a key: a key the
  table names is read with `Next` (a number, a bool, `null`, a nested
  object the table walks into) or `Text(cap)` (a string: `cap` is
  `string_bytes`, or `body_bytes` for a body); any other key's value
  gets `Skip`. A key named twice in one object is `Malformed::Shape`.
  `null` is an absent value; at an object's end, each required field not
  seen is `Malformed::Missing`. A row of a listing goes up as
  `Step::Row` at its object's end, so a listing is never held here; a
  document's own fields go up with `Step::Done`, after the tokenizer's
  `Done`.
- **Why not skein-json's selective collector** (json.md, 5.1): it hands
  up a document whole, and an unpaged thread must be cut as it streams
  and its rows handed up one at a time (3.2); one pattern serves every
  answer.
- **Requests are one value for both sides** (`request::Request`, with
  paging as `Page { page, limit }` of options and the listing's
  `state`, `type`, `since` and `sort` as options), since the server must
  see what a client left out (forge.md, 9: "a size without a page is
  ignored where Forgejo ignores it"). The client always sets them;
  `client::encode` writes the parameters in forge.md 3.2's order, each
  value with `path::segment`, and bodies with keys in 3.2's order. The
  methods are the crate's own `request::Method` (no skein-http here).
- **The server encodes what the documents carry** under Forgejo's key
  names, and `"pull_request": null` for an issue that is not a pull, an
  object for one: no other field. The fake's quirks (order, page cap,
  unpaged threads, the merge's empty body) are 14.3's, not here.
- **Where tests go.** Unit tests in `crates/temper-forge-forgejo/src/tests.rs`,
  fixtures in `src/tests/fixtures.rs` (`limits()`, `small()` with
  `body_bytes: 16` and `string_bytes: 8`, and `decode(decoder, body,
  &limits) -> Decoded { rows, document }`, which drives skein-json's
  tokenizer over the bytes: each demand met from the body, `Up::End`
  after it, the rows in a `skein_lib::List`, the document a
  `Result<Document, Malformed>`).
  The transcripts are files, and step code may not use `env!` or
  `include_bytes!` (the workspace's `clippy.toml`), so each test that
  reads one is a world test in `tests/codecs` (03.1's world): helpers in
  `tests/codecs/src/forgejo.rs` (`transcript(case) -> Transcript` reading
  `tools/forgejo-conformance/transcripts/<case>/{request,response}` from
  `env!("CARGO_MANIFEST_DIR")`, the body after the blank line;
  `decode(&request, body) -> Decoded { rows, document, steps }` with
  `limits()` as above; `route(&transcript) -> Request`;
  `shape(json) -> BTreeSet<(String, Kind)>`, key paths with `[]` for an
  array's elements, by `serde_json`; `specimen()`, the `Repo`
  `fixture/specimen`; `sha(n)`, the masked id `n` in 40 digits;
  `page(page, limit)`; and, in 06.5, `json_cases()` and `STEP_BOUND`;
  03.1's `mutated(rng, golden)` in `tests/codecs/src/lib.rs` mutates a
  body), and one test file per increment.
  Expected values are literals copied from the committed transcripts
  (ids as the fresh instance gave them; shas and times as masked).
- **Test limits:** `limits()` is `json: { depth: 16, string: 65_536,
  number: 32, chunk: 4_096, length: 4_194_304 }`, `writer: { depth: 16,
  length: 4_194_304 }`, `string_bytes: 1_024`, `body_bytes: 65_536`,
  `inner: 64`, `target_bytes: 4_096`. The profile's values are 13.1's.

## Rules that bind it

- "**`temper-forge-forgejo`** links `skein-lib` and `skein-json` only:
  no temper domain type (forge.md, section 2)" (code-organization.md,
  3); and it is step code: no traits, generic items, closures, `Vec`,
  `String`, `format!`, `loop` or `while` (programming-model.md, 10; the
  workspace's `clippy.toml`).
- "**Transcripts are read as files** from the tools' directories by path
  from `env!("CARGO_MANIFEST_DIR")`, raw bytes, never re-recorded by a
  test" (code-organization.md, 5): a failing transcript test is fixed in
  the code, or raised; it never edits a transcript.
- "**An increment:** ... It lands within twice its Size without asking; a
  codec or document increment ..., whose code is a schema, translations
  and decoders, within 2.5 times it (decisions.md, ruling 106). The parts
  of a split increment count together" (code-organization.md, 8).
  Every increment here is a document increment.

## Increments

### 06.1 The crate, its conventions and the decoder pattern; the start and resolution documents

Scope: `crates/temper-forge-forgejo` (forge.md, 2, 3.1, 3.3, 3.5): the
modules `limits` (`Limits`, `check`), `id` (object ids both ways, a JSON
number as `u64`), `time` (RFC 3339 both ways), `path` (a segment, a
file's path, the server's `read`), `decode` (`Step`, `Malformed`, the
walk), `request` (`Method`, `Repo`, `Page`, and the variants `User`,
`ApiSettings`, `RepositoryById`, `Repository`, `UserSearch`),
`document` (`Body`, `User`, `ApiSettings`, `MergeStyle`, `Repository`,
`Row::User`, `Document::{User, ApiSettings, Repository, Search}`),
`client` (`encode`, `Encoded`, `Refusal`, `Decoder`, `decoder`,
`first`, `step`), `server` (`route`, `Route`, `Unrouted`, `Answer::{User,
ApiSettings, Repository, Search}`, `encode`); both sides of the five
start and resolution requests.
Needs: 05.2's transcripts (`user`, `user-fixture`, `settings-api`,
`repository-by-id`, `repository-by-path`, `users-search-uid`); 03.1's
`tests/codecs` world (if it is not on `origin/main` when 06.1 starts,
the session waits for it).
Files: `crates/temper-forge-forgejo/{Cargo.toml, src/{lib.rs, limits.rs,
id.rs, time.rs, path.rs, document.rs, request.rs, decode.rs,
decode/walk.rs, client.rs, client/start.rs, server.rs, server/start.rs,
tests.rs, tests/fixtures.rs}}`; the workspace `Cargo.toml` (member,
`[workspace.dependencies]`); `tests/codecs/{Cargo.toml, src/lib.rs,
src/forgejo.rs, tests/forgejo_start.rs}`. Shared hotspots: the
workspace `Cargo.toml`; `tests/codecs/Cargo.toml` and `src/lib.rs`
(03's and 04's).
Shape: as the interface sketch's `limits`, `id`, `time` (but
`http_date`: 06.2), `path`, `decode`, `client` (but `keep_after` and
`cut`: 06.2; `error_decoder`: 06.5), and those parts of `document`,
`request` and `server` named in Scope; `server::Route` has `Ready` only
until 06.5 adds `Body` with `BodyDecoder`, `BodyStep`, `body_first` and
`body`. The crate's `Cargo.toml` is
`crates/temper-engine-domain-forge-change/Cargo.toml`'s with
`skein-lib` and `skein-json`; `lib.rs` has `#![cfg_attr(not(test),
no_std)]`, `#![forbid(unsafe_code)]`, `extern crate alloc`.
`tests/codecs` gains `temper-forge-forgejo`, `skein-json`, `skein-lib`
and `serde_json` (workspace) as dependencies.
Test shape: unit tests in `src/tests.rs`, fixtures in
`src/tests/fixtures.rs`:
`names_in_paths_are_percent_encoded_one_segment_each` (rows: `main`,
`feature/slash` → `feature%2Fslash`, `a b`, `é`, a file path
`docs/a b.md` → `docs/a%20b.md`),
`rfc3339_times_read_with_their_offset_and_write_in_utc` (rows: `Z`,
`+05:30`, `-07:00`, a fraction dropped, `1969-12-31T23:59:59Z` refused,
`u64::MAX` ns written `2554-07-21T23:34:33Z`),
`object_ids_read_40_or_64_lowercase_digits_only` (rows: 40, 64, 39, 41,
uppercase),
`a_start_document_missing_a_field_is_malformed_naming_it` (rows: a
user without `login`, a repository without `object_format_name`),
`a_string_past_its_cap_is_malformed_and_a_body_past_its_cap_is_long`
(with `small()`). World tests in `tests/codecs/tests/forgejo_start.rs`:
`start_transcripts_decode_to_their_documents`,
`start_requests_encode_as_the_tool_sent_them` (method, target and body
against the transcript's request line and body),
`start_requests_route_on_the_servers_side_to_the_same_request`,
`start_answers_the_server_encodes_decode_unchanged_with_the_transcripts_shape`.
Each a table over the six cases. Unit tests under 20 ms each; world
tests under 50 ms each. Reference tests, in full:

```rust
// crates/temper-forge-forgejo/src/tests.rs
#[test]
fn a_start_document_missing_a_field_is_malformed_naming_it() {
    let limits = fixtures::limits();
    let rows: [(&str, Request, &[u8], Malformed); 2] = [
        ("user without login", Request::User, br#"{"id":2,"is_admin":false}"#, Malformed::Missing { field: b"login" }),
        (
            "repository without its format",
            Request::RepositoryById { id: 1 },
            br#"{"id":1,"owner":{"id":1,"login":"fixture"},"name":"specimen","default_branch":"main","default_merge_style":"merge","allow_merge_commits":true,"allow_squash_merge":true,"allow_rebase":true,"html_url":"http://127.0.0.1:3000/fixture/specimen"}"#,
            Malformed::Missing { field: b"object_format_name" },
        ),
    ];
    for (name, request, body, expected) in rows {
        let decoder = client::decoder(&request, &limits).expect("a JSON answer");
        let decoded = fixtures::decode(decoder, body, &limits);
        assert_eq!(decoded.document, Err(expected), "{name}");
    }
}
```

```rust
// tests/codecs/tests/forgejo_start.rs
#[test]
fn start_transcripts_decode_to_their_documents() {
    // Values as the committed transcripts show them (ids from the fresh instance).
    let rows = [
        ("user", Request::User, Document::User(User { id: 2, login: Box::from(*b"writer") })),
        ("user-fixture", Request::User, Document::User(User { id: 1, login: Box::from(*b"fixture") })),
    ];
    for (case, request, expected) in rows {
        let transcript = forgejo::transcript(case);
        assert_eq!(transcript.status, 200, "{case}");
        let decoded = forgejo::decode(&request, &transcript.body);
        assert_eq!(decoded.document, Ok(expected), "{case}");
        assert!(decoded.rows.is_empty(), "{case}: a single document has no rows");
    }
}
```

Proof: world (the joint layer needs 13's component),
`cargo nextest run -p temper-forge-forgejo -p temper-codecs-world`,
run once: the six start and resolution transcripts decode to their
documents, their requests encode as the tool sent them and route back
to the same request, and the server's answers decode unchanged with
the transcripts' keys and types (the four world tests); under 2 s.
Don't: link any crate but `skein-lib` and `skein-json`; keep a field a
document does not carry; parse a time with a float; add skein-http.
Size: at most 600 lines.
Removes: nothing.

### 06.2 Items, threads, pull requests and combined statuses

Scope: both sides of `Request::{Issues, Issue, Comments, Pull, PullFor,
CombinedStatus}` (forge.md, 3.2: `Read::Items`, `Read::Item`,
`Read::Pull`, `Read::PullFor`, and a pull's CI); the documents `Issue`,
`IssueState`, `Comment`, `PullBranch`, `PullRequest`, `StatusState`,
`CombinedStatus`, `CommitStatus`, `Row::{Issue, Comment, Status}`,
`Document::{Listed, Issue, Pull, CombinedStatus}`; `client::keep_after`
and `client::cut` for the thread; `time::http_date` (a listing's `now`
is its `Date`).
Needs: nothing beyond 06.1.
Files: `crates/temper-forge-forgejo/src/{document.rs, request.rs,
time.rs, client.rs, client/items.rs, server.rs, server/items.rs,
tests.rs, tests/fixtures.rs}`; `tests/codecs/{src/forgejo.rs,
tests/forgejo_items.rs}`.
Shape: as the interface sketch. An issue's `pull` is a `pull_request`
value that is an object; `null` or absent is not a pull. `labels` keeps
`name` of each element, at most `inner`; `requested_reviewers` likewise.
`base.sha` empty is `None`. The thread: `keep_after(after)` before the
first step; an element whose `id` (read first, as Forgejo writes it) is
at most `after` gets `Skip` for each later key, so its body is never
kept; after `cut`, each remaining element gets one `Skip` and is
counted; `Document::Listed { skipped }` at the array's end.
Test shape: unit tests in `src/tests.rs`:
`a_comment_at_or_below_after_is_skipped_body_unread` (its body over
`small()`'s `body_bytes` would be `Long`: the row is not there at all),
`a_cut_thread_counts_the_comments_it_skipped`,
`a_pull_request_object_makes_an_issue_a_pull_and_null_does_not`,
`http_dates_read_imf_fixdate_and_nothing_else`. World tests in
`tests/codecs/tests/forgejo_items.rs`, as 06.1's four over the cases
`issues-page-1`, `issues-page-2`, `issues-type-issues`,
`issues-type-pulls`, `issues-since-2554`, `issue`, `issue-of-pull`,
`issue-comments`, `pull`, `pull-merged`, `pull-for-encoded-head`,
`combined-status`, `combined-status-empty` (named
`item_transcripts_decode_to_their_documents_and_rows`,
`item_requests_encode_as_the_tool_sent_them`,
`item_requests_route_on_the_servers_side_to_the_same_request`,
`item_answers_the_server_encodes_decode_unchanged_with_the_transcripts_shape`),
and `item_transcript_mutations_are_malformed_naming_the_field` (rows,
each a transcript's body with one edit: `number` removed, `"state":
"merged"`, a `head.sha` of 39 digits, `updated_at` without its offset,
`inner + 1` labels). Ceilings as 06.1. Reference test, in full (the
new kind: rows and a document's own fields together):

```rust
#[test]
fn item_transcripts_decode_to_their_documents_and_rows() {
    let transcript = forgejo::transcript("combined-status");
    let request = Request::CombinedStatus { at: forgejo::specimen(), commit: forgejo::sha(1), page: forgejo::page(1, 2) };
    let decoded = forgejo::decode(&request, &transcript.body);
    assert_eq!(decoded.document, Ok(Document::CombinedStatus(CombinedStatus { state: Some(StatusState::Success), total_count: 1 })));
    assert_eq!(decoded.rows.len(), 1, "one status on the head");
    let Row::Status(status) = &decoded.rows[0] else { panic!("a status row") };
    assert_eq!(&*status.context, b"fixture/test");
    assert_eq!(status.status, StatusState::Success);
}
```

(`forgejo::sha(n)` is the masked id `n` as 40 digits; the values above
stand for what the transcript shows.)
Proof: world, `cargo nextest run -p temper-forge-forgejo -p
temper-codecs-world`, run once: the thirteen item transcripts decode to
their documents and rows, both sides meet the transcripts' requests, and
each mutation is malformed naming its field (the five world tests);
under 2 s.
Don't: hold a listing's rows in the decoder; read a body of a comment
at or below `after`.
Size: at most 550 lines.
Removes: nothing.

### 06.3 Reviews, statuses' pages, a review's comments, branches, protection, collaborators

Scope: both sides of `Request::{Reviews, ReviewComments, Branch,
Branches, Protection, Protections, Collaborators, Permission, Pulls}`,
and `CombinedStatus` read past its first page (`Read::Statuses`); the
documents `ReviewState`, `Review`, `ReviewComment`, `Branch`,
`BranchProtection`, `Permission`, `CollaboratorPermission`,
`Row::{Review, ReviewComment, Branch, Protection, Pull}`,
`Document::{Branch, Protection, Permission}` (forge.md, 3.2);
`PullBranch` gains `repo_id: u64` (`head.repo_id`, `base.repo_id`).
`Request::Pulls` is `GET R/pulls?state=open&sort=oldest&page&limit`,
the open pull requests `Read::PullFor` reads when its lookup names a
closed one (forge.md, 3.2, "A pull request for a head and base"); each
element goes up as `Row::Pull(PullRequest)`, decoded as 06.2's pull.
Needs: nothing beyond 06.2.
Files: `crates/temper-forge-forgejo/src/{document.rs, request.rs,
client.rs, client/reviews.rs, client/branches.rs, server.rs,
server/reviews.rs, server/branches.rs, tests.rs}`;
`tests/codecs/tests/forgejo_reviews.rs`.
Shape: as the interface sketch. `effective_branch_protection_name` empty
is `None`. A review's comments are a thread: `cut` applies to them as to
06.2's.
Test shape: unit tests in `src/tests.rs`:
`each_review_state_reads_from_forgejos_word` (a table of the five),
`a_branch_naming_no_rule_has_no_protection_name`,
`a_protection_with_inner_plus_one_contexts_is_past_its_bound`,
`open_pulls_rows_are_the_pull_transcripts_documents` (no case records
the listing: the body of `pull`'s transcript and of `pull-merged`'s, in
one JSON array, decode to two `Row::Pull`s equal to those cases'
`Document::Pull`s, `repo_id` read on both sides; the server's encoding
of the two decodes back unchanged; the request encodes as
`GET /api/v1/repos/fixture/specimen/pulls?state=open&sort=oldest&page=1&limit=2`
and routes back to itself; 14.5's `component-read-pull-for-open-after-closed`
meets the real listing). World
tests in `tests/codecs/tests/forgejo_reviews.rs`, as 06.2's five
(`review_transcripts_decode_to_their_documents_and_rows`,
`review_requests_encode_as_the_tool_sent_them`,
`review_requests_route_on_the_servers_side_to_the_same_request`,
`review_answers_the_server_encodes_decode_unchanged_with_the_transcripts_shape`,
`review_transcript_mutations_are_malformed_naming_the_field`), over the
cases `reviews`, `statuses-page-2`, `review-comments`, `branch`,
`branch-with-slash`, `branches-page-1`, `branches-page-2`,
`branch-protection`, `branch-protections`, `collaborators`,
`collaborator-permission`, `owner-permission` (a `403` with
`{message, url}`: its rows assert the status, and its error body is
06.5's; `Permission::Owner` is read from a literal unit document;
ruling 109). Ceilings as 06.1.
Proof: world, `cargo nextest run -p temper-forge-forgejo -p
temper-codecs-world`, run once: eleven transcripts decode, and
`owner-permission`'s answers `403` (its error body is 06.5's; ruling
109), both sides meet their requests, mutations are malformed naming their
fields, and the open pulls' listing decodes its rows as pulls; under 2 s.
Don't: map `owner` to `admin` (13.3's mapping); read a review's
comments by page (Forgejo does not page them); filter the pulls'
listing by head or base in the request (Forgejo has no such filter;
13.3 compares each row).
Size: at most 550 lines.
Removes: nothing.

### 06.4 Pull files, comparisons, raw files and Actions

Scope: both sides of `Request::{PullFiles, Compare, Raw, Runs, Run,
Jobs, JobLog}`; the documents `FileStatus`, `ChangedFile`, `Compare`,
`Run`, `Job`, `Row::{File, Commit, Run, Job}`, `Document::{Compare,
Runs, Run}`; the `header` module (`has_more`, `content_range`,
`retry_after`, `job_link`) (forge.md, 3.2: `Read::PullFiles`,
`Read::Compare`, `Read::File`, `Read::Checks`, `Read::Job`; 3.4: the
`429`'s `Retry-After`). `Raw` and `JobLog` answer bytes: their requests
only (both sides), and the `Content-Range` of a `206`.
Needs: 05.3's Actions transcripts (`actions-combined-status`,
`actions-runs`, `actions-run`, `actions-jobs`, `actions-job-log`,
`actions-job-log-range`).
Files: `crates/temper-forge-forgejo/src/{lib.rs, document.rs,
request.rs, header.rs, client.rs, client/files.rs, client/actions.rs,
server.rs, server/files.rs, server/actions.rs, tests.rs}`;
`tests/codecs/tests/forgejo_files.rs`.
Shape: as the interface sketch. A `Raw` target keeps the file's slashes
(`path::file`); its `ref` is the id's hex. `job_link` reads
`<prefix>/actions/runs/<n>/jobs/<i>` and nothing longer, its prefix
either `html_url` whole or `html_url`'s path (from the first `/` after
`://` and the host; `/fixture/specimen`), as v16.0.5 writes the link
relative to the origin (forge.md, 3.2, "Job attempts"); a link that
starts `//`, or names another scheme or host, and any other shape is
`None`. `retry_after` reads decimal
seconds, or an HTTP date less `date` (a date before it: zero).
Test shape: unit tests in `src/tests.rs`:
`x_hasmore_reads_true_or_false_only`,
`a_content_range_reads_its_first_last_and_total`,
`retry_after_reads_seconds_or_a_date_from_the_answers_date`,
`a_job_link_of_another_shape_names_no_job` (its table holds, for
`html_url` `http://127.0.0.1:3000/fixture/specimen`:
`/fixture/specimen/actions/runs/1/jobs/0` and
`http://127.0.0.1:3000/fixture/specimen/actions/runs/1/jobs/0` read as
`(1, 0)`; `//127.0.0.1:3000/fixture/specimen/actions/runs/1/jobs/0`,
`http://other:3000/fixture/specimen/actions/runs/1/jobs/0`,
`/other/specimen/actions/runs/1/jobs/0` and
`/fixture/specimen/actions/runs/1/jobs/0/x` read as `None`), each a
table. World tests in
`tests/codecs/tests/forgejo_files.rs`, as 06.2's five, named
`file_...` in place of `item_...`, over
`pull-files-page-1`, `pull-files-page-2`, `compare`, `compare-reverse`,
`raw-file`, `raw-file-range`, `actions-combined-status`, `actions-runs`,
`actions-run`, `actions-jobs` (decoding), and `actions-job-log`,
`actions-job-log-range`, `raw-file` (requests only), plus
`a_failing_actions_status_links_its_run_and_job` (the `fail` job's
status in `actions-combined-status`, read by `job_link` against the
repository's `html_url`, names the run number of `actions-runs` and the
job's index in `actions-jobs`). Ceilings as 06.1.
Proof: world, `cargo nextest run -p temper-forge-forgejo -p
temper-codecs-world`, run once: the transcripts decode, both sides meet
their requests, and a failing Actions status links its run and job;
under 2 s.
Don't: read a raw file or a log as JSON; follow a link of another host.
Size: at most 500 lines.
Removes: nothing.

### 06.5 Writes, error bodies and refusals; the decoders' fuzz test

Scope: both sides of the writes (forge.md, 3.2): `Request::{CreateIssue,
OpenPull, PostComment, PostReview, EditIssue, RequestReviewers,
RemoveReviewers, Merge, Update, PostStatus, CreateBranch,
DeleteBranch}`, their bodies (the client's encoder, keys in 3.2's
order; the server's `BodyDecoder`), and their answers that 3.2 reads
(`Issue` for a creation's `number`, `Comment`'s `id`, `Review`'s `id`,
`PullRequest`'s `number`, `Branch`); `ReviewEvent`, `UpdateStyle`;
`ErrorBody`, `Document::{Comment, Status, Review, Error}`,
`client::error_decoder`, `server::Answer::Error`; the `failure` module
(3.4's messages, as the failure transcripts give them); the fuzz test.
Needs: nothing beyond 06.4.
Files: `crates/temper-forge-forgejo/src/{lib.rs, document.rs,
request.rs, failure.rs, client.rs, client/writes.rs, server.rs,
server/writes.rs, tests.rs}`; `tests/codecs/tests/{forgejo_writes.rs,
fuzzy_forgejo.rs}`.
Shape: as the interface sketch. `EditIssue` writes only the fields
given (`Write::Close` and `Write::Reopen` are `state`); `Merge`'s `Do`
is the style's word. A `401`'s message holds the token the request sent
(v16.0.5 answered a wrong token with `access token does not exist [sha:
<the token>]`, W3's run of 2026-10-14): its doc says so, and the
server's `401` answers carry `token is required` only. An error body that is empty or not JSON (a `405`
may come so) decodes to no message: the component reads its status
alone. `failure::refusal` compares a message byte for byte with each
known one, as the transcripts
`merge-stale-head`, `update-conflict`, `merge-closed`, `merge-conflict`,
`merge-protected`, `merge-style-refused` and
`create-issue-empty-title` spell them; one no transcript shows is
`None`. `open-pull-no-changes` is no refusal: v16.0.5 answered it `201`
with the pull request opened (forge.md, 13), and it decodes as
`Document::Pull` among the write cases; there is no `Refusal::NoChanges`.
Test shape: unit tests in `src/tests.rs`:
`a_write_whose_text_is_not_utf8_is_refused_before_encoding`,
`an_edit_writes_only_the_fields_it_is_given`,
`each_refusal_message_reads_back_from_what_the_server_answers`. World
tests in `tests/codecs/tests/forgejo_writes.rs`, as 06.2's five, named
`write_...` in place of `item_...`, over the write cases of 05.2 (`create-issue` to `delete-branch`), plus
`failure_transcripts_name_their_refusal_by_status_and_message` (a table
over the failure cases of 05.2, and `owner-permission`, a `403` on a
read; ruling 109). Fuzzy, in
`tests/codecs/tests/fuzzy_forgejo.rs`:
`forgejo_decoders_answer_every_mutated_transcript_without_panicking`
(each JSON transcript body, mutated 200 times by 03.1's `mutated` with
a seeded `skein_lib::Rng`; each decode ends `Done` or `Malformed`
within `STEP_BOUND` steps) and
`forgejo_server_routes_answer_every_mutated_request_without_panicking`;
about 2 s each, one seed per test. Ceilings as 06.1. Reference test, in
full (the new kind: a fuzz test):

```rust
#[test]
fn forgejo_decoders_answer_every_mutated_transcript_without_panicking() {
    let mut rng = skein_lib::Rng::new(0x06_05);
    for (case, request) in forgejo::json_cases() {
        let body = forgejo::transcript(case).body;
        for _ in 0..200 {
            let mutated = temper_codecs_world::mutated(&mut rng, &body);
            let decoded = forgejo::decode(&request, &mutated);
            assert!(decoded.steps <= forgejo::STEP_BOUND, "{case}: a decode ends within its bound");
        }
    }
}
```

Proof: world, `cargo nextest run -p temper-forge-forgejo -p
temper-codecs-world` and `cargo nextest run -p temper-codecs-world
--profile fuzzy -E 'binary(fuzzy_forgejo)'`, each run once: the writes
meet their transcripts on both sides, each failure transcript names its
refusal, and no mutated transcript or request panics a decoder; under 6
s together.
Don't: guess a refusal from part of a message; send a body for a write
that 3.2 sends none.
Size: at most 550 lines.
Removes: nothing.

## Interface sketches

The crate `temper-forge-forgejo`, which no session has made yet, as the
sketch declares it: each `pub mod` below is the crate's module of that
name. `scratch/plan-sketches/src/s06.rs` holds the same text and
compiles under the workspace's lints. Module paths are named in the
comment above each; the increment that adds each item is in its doc.

- **Who owns what:** the crate owns Forgejo's words and their bounds;
  the forge component (13) owns the tokenizer, the HTTP exchange, the
  headers it writes (`Host`, `Authorization`, `User-Agent`,
  `Content-Type`, `Range`) and the translation to the forge client's
  vocabulary; the fake forge's protocol layer (14.3) owns the fake's
  answers and quirks.
- **What ends what:** a `client::Decoder` ends at its first
  `Step::Done` or `Step::Malformed`; a `server::BodyDecoder` at
  `Decoded` or `Malformed`. Nothing here outlives one answer.

```rust
//! Sketches of session 06: `temper-forge-forgejo`, Forgejo v16.0.5's documents, both sides
//! (forge.md, sections 2, 3 and 9). The crate does not exist yet: each module below is the
//! crate's module of the same name (`temper_forge_forgejo::<module>`), declared as the sketch.
//! It links skein-lib and skein-json only, and names no temper domain type.
#![expect(unused_variables, reason = "a sketch's bodies are unreachable; its parameters are its signature")]

// temper_forge_forgejo::limits (06.1)
pub mod limits {
    use skein_json::{tokenizer, writer};

    /// The caps every decoder and encoder of the crate runs under; the forge component (13.1) and
    /// the fake forge's protocol layer (14.3) derive them from their own limits at start.
    #[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
    pub struct Limits {
        /// The tokenizer's: `string` at least `body_bytes` and `string_bytes`, `depth` about 8.
        pub json: tokenizer::Limits,
        /// The writer's, for request bodies (client) and answers (server).
        pub writer: writer::Limits,
        /// The longest name, title, context, description, link or path kept (forge.md, 3.3).
        pub string_bytes: u32,
        /// The longest body kept whole; a longer one is `Body::Long` (forge.md, 3.3).
        pub body_bytes: u32,
        /// The most elements kept of a list inside one document (labels, requested reviewers,
        /// a protection's contexts); one more is `Malformed::Bound`.
        pub inner: u32,
        /// The longest request target written or read, in bytes.
        pub target_bytes: u32,
    }

    /// A limit that does not fit the others: refused at start, naming it.
    #[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
    pub enum Problem {
        /// `json.string` is below `string_bytes` or `body_bytes`, so `Text(cap)` could not keep them.
        TokenizerString,
        /// `json.depth` below the deepest document of section 3.2 (8).
        Depth,
        /// A cap is 0.
        Zero,
    }

    /// The start's check of `limits` (forge.md, 7, "Checked at start").
    pub fn check(limits: &Limits) -> Result<(), Problem> {
        unreachable!("sketch")
    }
}

// temper_forge_forgejo::id (06.1)
pub mod id {
    use alloc::boxed::Box;

    /// A repository's object format (`object_format_name`): how many hex digits its ids have.
    #[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
    pub enum ObjectFormat {
        /// `sha1`: 40 digits.
        Sha1,
        /// `sha256`: 64 digits.
        Sha256,
    }

    /// An object id as Forgejo writes it: lowercase hex of 40 or 64 digits, held as 32 bytes
    /// (a SHA-1 id zero-padded, as `api::Commit`), with the format its digits said.
    #[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
    pub struct ObjectId {
        pub format: ObjectFormat,
        pub bytes: [u8; 32],
    }

    /// Reads 40 or 64 lowercase hex digits; anything else is `None` (a malformed answer).
    #[must_use]
    pub fn read(hex: &[u8]) -> Option<ObjectId> {
        unreachable!("sketch")
    }

    /// Writes the id's 40 or 64 lowercase hex digits, as its format says.
    #[must_use]
    pub fn write(id: &ObjectId) -> Box<[u8]> {
        unreachable!("sketch")
    }

    /// A JSON number's text as a `u64`: digits only, no sign, fraction or exponent, no overflow.
    #[must_use]
    pub fn unsigned(text: &[u8]) -> Option<u64> {
        unreachable!("sketch")
    }
}

// temper_forge_forgejo::time (06.1; `http_date` 06.2)
pub mod time {
    use skein_lib::Wall;

    /// The longest RFC 3339 time this crate writes: `2554-07-21T23:34:33Z`.
    pub const RFC3339_BYTES: u32 = 20;

    /// Reads an RFC 3339 time with an offset (`Z`, `+05:30`, `-07:00`), fractions of a second
    /// dropped, as nanoseconds since the epoch; before the epoch or past `u64::MAX` ns: `None`.
    #[must_use]
    pub fn read(text: &[u8]) -> Option<Wall> {
        unreachable!("sketch")
    }

    /// Writes `wall` in UTC as `YYYY-MM-DDTHH:MM:SSZ`, its nanoseconds dropped (forge.md, 3.1:
    /// `since`, up to `u64::MAX` ns, in 2554).
    #[must_use]
    pub fn write(wall: Wall) -> [u8; 20] {
        unreachable!("sketch")
    }

    /// Reads an HTTP date in IMF-fixdate (`Wed, 14 Oct 2026 01:31:28 GMT`): a `Date`, or a
    /// `Retry-After` given as a date (06.2).
    #[must_use]
    pub fn http_date(text: &[u8]) -> Option<Wall> {
        unreachable!("sketch")
    }
}

// temper_forge_forgejo::path (06.1)
pub mod path {
    use alloc::boxed::Box;

    /// A request target split on the server's side: the path's segments, each percent-decoded
    /// after the split (so `%2F` stays inside its segment), and the query's pairs, decoded.
    #[derive(Clone, PartialEq, Eq, Hash, Debug)]
    pub struct Target {
        pub segments: Box<[Box<[u8]>]>,
        pub query: Box<[Pair]>,
    }

    /// One `name=value` of a target's query, each percent-decoded.
    #[derive(Clone, PartialEq, Eq, Hash, Debug)]
    pub struct Pair {
        pub name: Box<[u8]>,
        pub value: Box<[u8]>,
    }

    /// Why a target was not read.
    #[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
    pub enum Unread {
        /// A `%` not followed by two hex digits.
        Escape,
        /// Past `Limits::target_bytes`, or more segments or pairs than a route has.
        TooLong,
        /// Not in origin form (`/...`).
        Form,
    }

    /// Appends `name` as one path segment: RFC 3986's unreserved bytes kept, every other byte
    /// `%XX` in uppercase hex (a slash in a branch is `%2F`). The query's values use it too.
    pub fn segment(name: &[u8], out: &mut skein_lib::Writer) -> Result<(), skein_lib::Overflow> {
        unreachable!("sketch")
    }

    /// Appends a file's path: each segment as `segment` writes it, the slashes between kept.
    pub fn file(path: &[u8], out: &mut skein_lib::Writer) -> Result<(), skein_lib::Overflow> {
        unreachable!("sketch")
    }

    /// Splits and decodes a target as it came (the server's side); `+` in the query is a space.
    pub fn read(target: &[u8], limits: &super::limits::Limits) -> Result<Target, Unread> {
        unreachable!("sketch")
    }
}

// temper_forge_forgejo::document (06.1 to 06.5): what Forgejo's answers carry that temper reads,
// field by field; both sides use them (the client decodes them, the server encodes them).
pub mod document {
    use super::id::{ObjectFormat, ObjectId};
    use alloc::boxed::Box;
    use skein_lib::Wall;

    /// A body: kept whole within `body_bytes`, or past it its length alone (forge.md, 3.3).
    #[derive(Clone, PartialEq, Eq, Hash, Debug)]
    pub enum Body {
        /// The body's bytes, UTF-8, unescaped.
        Kept(Box<[u8]>),
        /// A body past `body_bytes`: its length once unescaped (skein-json's `Long`).
        Long(u64),
    }

    /// A user object, wherever an answer carries one (`user`, `owner`, `creator`, a
    /// requested reviewer, a collaborator): the component's users table is filled from these.
    #[derive(Clone, PartialEq, Eq, Hash, Debug)]
    pub struct User {
        pub id: u64,
        pub login: Box<[u8]>,
    }

    /// `GET /api/v1/settings/api` (06.1).
    #[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
    pub struct ApiSettings {
        pub max_response_items: u32,
        pub default_paging_num: u32,
    }

    /// `default_merge_style` (forge.md, 13: the merge's `Do` values).
    #[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
    pub enum MergeStyle {
        /// `merge`.
        Merge,
        /// `rebase`.
        Rebase,
        /// `rebase-merge`.
        RebaseMerge,
        /// `squash`: how the fake merges (forge.md, 9).
        Squash,
        /// `fast-forward-only`.
        FastForwardOnly,
        /// `manually-merged`.
        ManuallyMerged,
    }

    /// A repository, by id or by path (06.1): its row of forge.md, 3.5 and `Read::Settings`.
    #[derive(Clone, PartialEq, Eq, Hash, Debug)]
    pub struct Repository {
        pub id: u64,
        pub owner: User,
        pub name: Box<[u8]>,
        pub object_format: ObjectFormat,
        pub default_branch: Box<[u8]>,
        pub default_merge_style: MergeStyle,
        pub allow_merge_commits: bool,
        pub allow_squash_merge: bool,
        pub allow_rebase: bool,
        pub html_url: Box<[u8]>,
    }

    /// An issue's or pull request's `state`.
    #[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
    pub enum IssueState {
        /// `open`.
        Open,
        /// `closed`.
        Closed,
    }

    /// An item of the issue listing, or one issue (06.2).
    #[derive(Clone, PartialEq, Eq, Hash, Debug)]
    pub struct Issue {
        pub number: u64,
        /// A `pull_request` object that is not `null`.
        pub pull: bool,
        pub state: IssueState,
        pub user: User,
        /// `labels[].name`, at most `Limits::inner`.
        pub labels: Box<[Box<[u8]>]>,
        pub updated_at: Wall,
        pub title: Box<[u8]>,
        pub body: Body,
    }

    /// One comment of an issue's thread (06.2). A time missing is `None` (provenance unknown).
    #[derive(Clone, PartialEq, Eq, Hash, Debug)]
    pub struct Comment {
        pub id: u64,
        pub user: User,
        pub created_at: Option<Wall>,
        pub updated_at: Option<Wall>,
        pub body: Body,
    }

    /// A pull request's `head` or `base`.
    #[derive(Clone, PartialEq, Eq, Hash, Debug)]
    pub struct PullBranch {
        /// `ref`.
        pub name: Box<[u8]>,
        /// `repo_id`: the repository the branch is in (06.3).
        pub repo_id: u64,
        /// `sha`; an empty string is `None`.
        pub sha: Option<ObjectId>,
    }

    /// A pull request (06.2).
    #[derive(Clone, PartialEq, Eq, Hash, Debug)]
    pub struct PullRequest {
        pub number: u64,
        pub state: IssueState,
        pub head: PullBranch,
        pub base: PullBranch,
        pub merged: bool,
        /// `null` or absent: `None`.
        pub merge_commit_sha: Option<ObjectId>,
        pub merge_base: Option<ObjectId>,
        pub mergeable: bool,
        /// At most `Limits::inner`.
        pub requested_reviewers: Box<[User]>,
    }

    /// A commit status's `state`, and a combined status's (forge.md, 13).
    #[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
    pub enum StatusState {
        /// `pending`.
        Pending,
        /// `success`.
        Success,
        /// `error`.
        Error,
        /// `failure`.
        Failure,
        /// `warning`.
        Warning,
        /// `skipped`.
        Skipped,
    }

    /// A combined status's own fields; its `statuses` come as `Row::Status` (06.2).
    /// `state` is none when Forgejo sends `""`, as it does for a commit with no
    /// status (ruling 104).
    #[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
    pub struct CombinedStatus {
        pub state: Option<StatusState>,
        pub total_count: u64,
    }

    /// One commit status (06.2, and `Read::Statuses`, `Read::Checks`).
    #[derive(Clone, PartialEq, Eq, Hash, Debug)]
    pub struct CommitStatus {
        /// None when Forgejo writes `null`, as for an Actions job's status (ruling 111).
        pub creator: Option<User>,
        pub updated_at: Wall,
        pub context: Box<[u8]>,
        pub status: StatusState,
        pub description: Box<[u8]>,
        pub target_url: Box<[u8]>,
    }

    /// A review's `state` (06.3).
    #[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
    pub enum ReviewState {
        /// `APPROVED`.
        Approved,
        /// `REQUEST_CHANGES`.
        RequestChanges,
        /// `COMMENT`.
        Comment,
        /// `PENDING`: not a review yet; 13 skips it.
        Pending,
        /// `REQUEST_REVIEW`: a request, not a review; 13 skips it.
        RequestReview,
    }

    /// A pull request's review (06.3).
    #[derive(Clone, PartialEq, Eq, Hash, Debug)]
    pub struct Review {
        pub id: u64,
        pub user: User,
        pub state: ReviewState,
        pub commit_id: ObjectId,
        pub body: Body,
        pub submitted_at: Option<Wall>,
        pub updated_at: Option<Wall>,
        pub official: bool,
        pub dismissed: bool,
    }

    /// One comment of a review: a remark (06.3).
    #[derive(Clone, PartialEq, Eq, Hash, Debug)]
    pub struct ReviewComment {
        pub id: u64,
        pub user: User,
        pub path: Box<[u8]>,
        pub position: u32,
        pub body: Body,
    }

    /// A branch, alone or as a row of the branches' listing (06.3).
    #[derive(Clone, PartialEq, Eq, Hash, Debug)]
    pub struct Branch {
        pub name: Box<[u8]>,
        /// `commit.id`.
        pub commit: ObjectId,
        /// An empty name is `None`: no rule applies.
        pub effective_branch_protection_name: Option<Box<[u8]>>,
    }

    /// A branch protection rule (06.3).
    #[derive(Clone, PartialEq, Eq, Hash, Debug)]
    pub struct BranchProtection {
        pub rule_name: Box<[u8]>,
        pub enable_status_check: bool,
        /// At most `Limits::inner`.
        pub status_check_contexts: Box<[Box<[u8]>]>,
        pub required_approvals: u32,
        pub dismiss_stale_approvals: bool,
    }

    /// A collaborator's `permission` (06.3); `owner` is its own value, read as admin by 13.
    #[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
    pub enum Permission {
        /// `none`.
        None,
        /// `read`.
        Read,
        /// `write`.
        Write,
        /// `admin`.
        Admin,
        /// `owner`.
        Owner,
    }

    /// `GET R/collaborators/{login}/permission` (06.3).
    #[derive(Clone, PartialEq, Eq, Hash, Debug)]
    pub struct CollaboratorPermission {
        pub permission: Permission,
        pub user: User,
    }

    /// A changed file's `status` (06.4), as the transcripts spell them.
    #[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
    pub enum FileStatus {
        /// `added`: no side before.
        Added,
        /// A deletion, as the transcripts spell it: no side after.
        Deleted,
        /// `modified`.
        Modified,
        /// `renamed`: the side before at `previous_filename`.
        Renamed,
        /// `copied`.
        Copied,
    }

    /// A pull request's or a comparison's changed file (06.4).
    #[derive(Clone, PartialEq, Eq, Hash, Debug)]
    pub struct ChangedFile {
        pub filename: Box<[u8]>,
        pub previous_filename: Option<Box<[u8]>>,
        pub status: FileStatus,
    }

    /// A comparison's own field; its `commits[].sha` and `files` come as rows (06.4).
    #[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
    pub struct Compare {
        pub total_commits: u64,
    }

    /// An Actions run (06.4).
    #[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
    pub struct Run {
        pub id: u64,
        pub index_in_repo: u64,
        pub commit_sha: ObjectId,
    }

    /// An Actions job of a run (06.4).
    #[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
    pub struct Job {
        pub id: u64,
        pub run_id: u64,
        pub attempt: u32,
    }

    /// An error answer's body (06.5): `message` within `string_bytes`, or its length.
    #[derive(Clone, PartialEq, Eq, Hash, Debug)]
    pub struct ErrorBody {
        pub message: Body,
    }

    /// A row of a listing, handed up as it is decoded, so a listing is never held whole here.
    #[derive(Clone, PartialEq, Eq, Hash, Debug)]
    pub enum Row {
        /// A user: `/users/search`'s `data`, or a collaborator.
        User(User),
        /// An item of the issue listing.
        Issue(Issue),
        /// A comment of an issue's thread.
        Comment(Comment),
        /// A commit status of a combined status.
        Status(CommitStatus),
        /// A pull request's review.
        Review(Review),
        /// A review's comment.
        ReviewComment(ReviewComment),
        /// A branch of the branches' listing.
        Branch(Branch),
        /// A rule of the protections' listing.
        Protection(BranchProtection),
        /// A changed file of a pull request or a comparison.
        File(ChangedFile),
        /// A comparison's `commits[].sha`.
        Commit(ObjectId),
        /// An Actions run of `workflow_runs`.
        Run(Run),
        /// An Actions job of a run's jobs.
        Job(Job),
        /// A pull request of the open pull requests' listing (06.3).
        Pull(PullRequest),
    }

    /// A decoded document: the whole of a single one, or a listing's own fields once its rows
    /// went up. Each request's answer is one variant (section 3.2's rows; `Listed` for a bare
    /// array, with how many elements the decoder skipped after a cut).
    #[derive(Clone, PartialEq, Eq, Hash, Debug)]
    pub enum Document {
        /// `/user`, or a user search's single answer.
        User(User),
        /// `/settings/api`.
        ApiSettings(ApiSettings),
        /// `/repositories/{id}`, `/repos/{owner}/{name}`.
        Repository(Repository),
        /// `/users/search`: its `ok`; its `data` came as `Row::User`.
        Search { ok: bool },
        /// A bare array's end: its rows went up, `skipped` after a cut.
        Listed { skipped: u32 },
        /// `R/issues/{number}`, and the answer of a creation or an edit.
        Issue(Issue),
        /// A posted comment.
        Comment(Comment),
        /// `R/pulls/{number}`, `R/pulls/{base}/{head}`, an opened pull request.
        Pull(PullRequest),
        /// `R/commits/{sha}/status`: its statuses went up as rows.
        CombinedStatus(CombinedStatus),
        /// A posted status.
        Status(CommitStatus),
        /// A posted review.
        Review(Review),
        /// `R/branches/{branch}`, a created branch.
        Branch(Branch),
        /// `R/branch_protections/{name}`.
        Protection(BranchProtection),
        /// `R/collaborators/{login}/permission`.
        Permission(CollaboratorPermission),
        /// `R/compare/{before}...{after}`: its commits and files went up as rows.
        Compare(Compare),
        /// `actions/runs`: its `total_count`; `workflow_runs` came as `Row::Run`.
        Runs { total_count: u64 },
        /// `R/actions/runs/{run}`.
        Run(Run),
        /// An error answer's body.
        Error(ErrorBody),
    }
}

// temper_forge_forgejo::request (06.1 to 06.5): each request of forge.md 3.2, 3.5 and the
// start, as both sides see it: the client encodes it, the server decodes it.
pub mod request {
    use super::document::{IssueState, MergeStyle, StatusState};
    use super::id::ObjectId;
    use alloc::boxed::Box;
    use skein_lib::Wall;

    /// The methods section 3.2 uses (skein-http's own is the component's to map to).
    #[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
    pub enum Method {
        /// `GET`.
        Get,
        /// `POST`.
        Post,
        /// `PUT`.
        Put,
        /// `PATCH`.
        Patch,
        /// `DELETE`.
        Delete,
    }

    /// A repository's path, R's `{owner}/{name}`.
    #[derive(Clone, PartialEq, Eq, Hash, Debug)]
    pub struct Repo {
        pub owner: Box<[u8]>,
        pub name: Box<[u8]>,
    }

    /// `type=` of the issue listing.
    #[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
    pub enum IssueKind {
        /// `issues`.
        Issues,
        /// `pulls`.
        Pulls,
    }

    /// `sort=` of the issue listing: `leastupdate`, or any other value (Forgejo's default order).
    #[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
    pub enum Sort {
        /// `leastupdate`: least recently updated first.
        LeastUpdate,
        /// Any other value, or one Forgejo ignores: newest created first.
        Other,
    }

    /// `state=` of the issue listing.
    #[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
    pub enum StateFilter {
        /// `open`.
        Open,
        /// `closed`.
        Closed,
        /// `all`, which the client always sends.
        All,
    }

    /// `page` and `limit`: the client always sends both (forge.md, 3.1); the server keeps
    /// either one missing, since the fake answers as Forgejo does then (forge.md, 9).
    #[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
    pub struct Page {
        pub page: Option<u32>,
        pub limit: Option<u32>,
    }

    /// A review's `event` (forge.md, 3.2, `Write::Review`).
    #[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
    pub enum ReviewEvent {
        /// `APPROVED`.
        Approved,
        /// `REQUEST_CHANGES`.
        RequestChanges,
        /// `COMMENT`.
        Comment,
    }

    /// `style=` of an update.
    #[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
    pub enum UpdateStyle {
        /// `merge`, which the client always sends.
        Merge,
        /// `rebase`.
        Rebase,
    }

    /// One request, by route. Paths are under `/api/v1`; `at` is R.
    #[derive(Clone, PartialEq, Eq, Hash, Debug)]
    pub enum Request {
        /// `GET /user` (06.1).
        User,
        /// `GET /settings/api` (06.1).
        ApiSettings,
        /// `GET /repositories/{id}` (06.1).
        RepositoryById { id: u64 },
        /// `GET /repos/{owner}/{name}`: a locate, or a call's own path (06.1).
        Repository { at: Repo },
        /// `GET /users/search?uid=&limit=` (06.1).
        UserSearch { uid: u64, limit: Option<u32> },
        /// `GET R/issues?state&type&since&sort&page&limit` (06.2).
        Issues {
            at: Repo,
            state: Option<StateFilter>,
            kind: Option<IssueKind>,
            since: Option<Wall>,
            sort: Option<Sort>,
            page: Page,
        },
        /// `GET R/issues/{number}` (06.2).
        Issue { at: Repo, number: u64 },
        /// `GET R/issues/{number}/comments`, unpaged (06.2).
        Comments { at: Repo, number: u64 },
        /// `GET R/pulls/{number}` (06.2).
        Pull { at: Repo, number: u64 },
        /// `GET R/pulls/{base}/{head}` (06.2).
        PullFor { at: Repo, base: Box<[u8]>, head: Box<[u8]> },
        /// `GET R/pulls?state=open&sort=oldest&page&limit`: the open pull requests (06.3).
        Pulls { at: Repo, page: Page },
        /// `GET R/commits/{sha}/status?page&limit` (06.2).
        CombinedStatus { at: Repo, commit: ObjectId, page: Page },
        /// `GET R/pulls/{number}/reviews?page&limit` (06.3).
        Reviews { at: Repo, number: u64, page: Page },
        /// `GET R/pulls/{number}/reviews/{review}/comments`, unpaged (06.3).
        ReviewComments { at: Repo, number: u64, review: u64 },
        /// `GET R/branches/{branch}` (06.3).
        Branch { at: Repo, branch: Box<[u8]> },
        /// `GET R/branches?page&limit` (06.3).
        Branches { at: Repo, page: Page },
        /// `GET R/branch_protections/{name}` (06.3).
        Protection { at: Repo, name: Box<[u8]> },
        /// `GET R/branch_protections` (06.3).
        Protections { at: Repo },
        /// `GET R/collaborators?page&limit` (06.3).
        Collaborators { at: Repo, page: Page },
        /// `GET R/collaborators/{login}/permission` (06.3).
        Permission { at: Repo, login: Box<[u8]> },
        /// `GET R/pulls/{number}/files?page&limit` (06.4).
        PullFiles { at: Repo, number: u64, page: Page },
        /// `GET R/compare/{before}...{after}` (06.4).
        Compare { at: Repo, before: ObjectId, after: ObjectId },
        /// `GET R/raw/{path}?ref={commit}`: bytes, not JSON (06.4).
        Raw { at: Repo, path: Box<[u8]>, reference: ObjectId },
        /// `GET R/actions/runs?head_sha&run_number&page&limit` (06.4).
        Runs { at: Repo, head_sha: ObjectId, run_number: u64, page: Page },
        /// `GET R/actions/runs/{run}` (06.4).
        Run { at: Repo, run: u64 },
        /// `GET R/actions/runs/{run}/jobs` (06.4).
        Jobs { at: Repo, run: u64 },
        /// `GET R/actions/jobs/{job}/logs?attempt`: bytes, not JSON (06.4).
        JobLog { at: Repo, job: u64, attempt: u32 },
        /// `POST R/issues {title, body}` (06.5).
        CreateIssue { at: Repo, title: Box<[u8]>, body: Box<[u8]> },
        /// `POST R/pulls {head, base, title, body}` (06.5).
        OpenPull { at: Repo, head: Box<[u8]>, base: Box<[u8]>, title: Box<[u8]>, body: Box<[u8]> },
        /// `POST R/issues/{number}/comments {body}` (06.5).
        PostComment { at: Repo, number: u64, body: Box<[u8]> },
        /// `POST R/pulls/{number}/reviews {event, body}` (06.5).
        PostReview { at: Repo, number: u64, event: ReviewEvent, body: Box<[u8]> },
        /// `PATCH R/issues/{number} {title, body, state}`, each only when given (06.5).
        EditIssue {
            at: Repo,
            number: u64,
            title: Option<Box<[u8]>>,
            body: Option<Box<[u8]>>,
            state: Option<IssueState>,
        },
        /// `POST R/pulls/{number}/requested_reviewers {reviewers}` (06.5).
        RequestReviewers { at: Repo, number: u64, reviewers: Box<[Box<[u8]>]> },
        /// `DELETE R/pulls/{number}/requested_reviewers {reviewers}` (06.5).
        RemoveReviewers { at: Repo, number: u64, reviewers: Box<[Box<[u8]>]> },
        /// `POST R/pulls/{number}/merge {Do, head_commit_id}` (06.5).
        Merge { at: Repo, number: u64, style: MergeStyle, head: ObjectId },
        /// `POST R/pulls/{number}/update?style` (06.5).
        Update { at: Repo, number: u64, style: Option<UpdateStyle> },
        /// `POST R/statuses/{sha} {state, context}` (06.5).
        PostStatus { at: Repo, commit: ObjectId, state: StatusState, context: Box<[u8]> },
        /// `POST R/branches {new_branch_name, old_ref_name}` (06.5).
        CreateBranch { at: Repo, branch: Box<[u8]>, from: ObjectId },
        /// `DELETE R/branches/{branch}` (06.5).
        DeleteBranch { at: Repo, branch: Box<[u8]> },
    }
}

// temper_forge_forgejo::decode (06.1): the decoder pattern, shared by both sides' decoders.
pub mod decode {
    use super::document::{Document, Row};
    use skein_json::tokenizer;

    /// What made a document malformed: the forge component reads it as forge.md 3.3 says.
    #[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
    pub enum Malformed {
        /// The tokenizer failed: not JSON, past its limits, cut short, or the stream failed.
        Json(tokenizer::Error),
        /// A token of another kind than the document has there (a string for a number).
        Shape { field: &'static [u8] },
        /// A field the document needs is absent at its object's end.
        Missing { field: &'static [u8] },
        /// A number out of its field's range, negative, or not an integer.
        Range { field: &'static [u8] },
        /// A string past `string_bytes` where only whole strings are kept.
        TooLong { field: &'static [u8] },
        /// An object id that is not 40 or 64 lowercase hex digits.
        Id { field: &'static [u8] },
        /// A time that is not RFC 3339 with an offset, or outside `Wall`.
        Time { field: &'static [u8] },
        /// A string value the field's enum does not know (`"state": "merged"`).
        Value { field: &'static [u8] },
        /// A list inside the document past `Limits::inner`: an answer past the component's
        /// bounds (forge.md, 3.4, `TooLarge`).
        Bound { field: &'static [u8] },
    }

    /// One answer of a decoder to one tokenizer event: what to demand next, or a result.
    #[derive(Clone, PartialEq, Eq, Hash, Debug)]
    pub enum Step {
        /// Demand this of the tokenizer next (`Next`, `Text(cap)` or `Skip`).
        Demand(tokenizer::Request),
        /// A listing's row, decoded; then this demand.
        Row(Row, tokenizer::Request),
        /// The document is whole and the tokenizer said `Done`. Terminal.
        Done(Document),
        /// Terminal: the answer is malformed (forge.md, 3.3); the connection closes.
        Malformed(Malformed),
    }
}

// temper_forge_forgejo::client (06.1 to 06.5): the engine's side.
pub mod client {
    use super::decode::Step;
    use super::limits::Limits;
    use super::request::{Method, Request};
    use alloc::boxed::Box;
    use skein_json::tokenizer;

    /// A request as the forge component sends it, but for the fields it writes itself
    /// (`Host`, `Authorization`, `User-Agent`, `Content-Type`, `Range`, `Content-Length`).
    #[derive(Clone, PartialEq, Eq, Hash, Debug)]
    pub struct Encoded {
        pub method: Method,
        /// `/api/v1/...`, its names percent-encoded (06.1's `path`).
        pub target: Box<[u8]>,
        /// The JSON body, compact, keys in section 3.2's order; `None` for a request with none.
        pub body: Option<Box<[u8]>>,
    }

    /// Why a request was not encoded; nothing is sent.
    #[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
    pub enum Refusal {
        /// Text that is not UTF-8, which JSON cannot carry (forge.md, 3.4: `Refused`).
        Text,
        /// The target past `target_bytes`, or the body past the writer's `length`.
        TooLong,
    }

    /// Encodes `request` (both passes of skein-json's writer for its body).
    pub fn encode(request: &Request, limits: &Limits) -> Result<Encoded, Refusal> {
        unreachable!("sketch")
    }

    /// An answer's decoder, chosen by its request (each kind its own state, one enum).
    #[derive(Debug)]
    pub struct Decoder {
        state: DecoderState,
    }

    #[derive(Debug)]
    enum DecoderState {}

    /// The decoder of the JSON answer `request` gets (with a `2xx`); `None` for `Raw` and
    /// `JobLog`, whose answers are bytes, and for writes answered with no body.
    #[must_use]
    pub fn decoder(request: &Request, limits: &Limits) -> Option<Decoder> {
        unreachable!("sketch")
    }

    /// The decoder of an error answer's body (`{"message", "url"}`, 06.5).
    #[must_use]
    pub fn error_decoder(limits: &Limits) -> Decoder {
        unreachable!("sketch")
    }

    /// The first demand to make of a fresh tokenizer.
    #[must_use]
    pub fn first(decoder: &Decoder) -> tokenizer::Request {
        match decoder.state {}
    }

    /// One tokenizer event in; exactly one `Step` out.
    pub fn step(decoder: &mut Decoder, limits: &Limits, event: &tokenizer::Event) -> Step {
        match decoder.state {}
    }

    /// Comments only: a comment whose `id` is at most `after` is skipped, its body unread
    /// (forge.md, 3.2, `Read::Item`). Set before the first step.
    pub fn keep_after(decoder: &mut Decoder, after: u64) {
        match decoder.state {}
    }

    /// An unpaged listing (comments, a review's comments): no further row goes up; each
    /// remaining element is skipped and counted into `Listed { skipped }`. Returns the demand
    /// to make in place of the one the last `Row` named.
    pub fn cut(decoder: &mut Decoder) -> tokenizer::Request {
        match decoder.state {}
    }
}

// temper_forge_forgejo::server (06.1 to 06.5): the fake forge's side (forge.md, 9).
pub mod server {
    use super::decode::Malformed;
    use super::document::{
        ApiSettings, Branch, BranchProtection, ChangedFile, CollaboratorPermission, CombinedStatus, Comment,
        CommitStatus, Compare, ErrorBody, Issue, Job, PullRequest, Repository, Review, ReviewComment, Run, User,
    };
    use super::id::ObjectId;
    use super::limits::Limits;
    use super::request::{Method, Request};
    use alloc::boxed::Box;
    use skein_json::{tokenizer, writer};

    /// Why a request has no route here: the fake answers as Forgejo would.
    #[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
    pub enum Unrouted {
        /// No route of section 3.2 or 3.5 has this path: `404`.
        NotFound,
        /// The path is a route, the method is not: `405`.
        Method,
        /// A path or query value that does not read (an id that is no number, a bad escape).
        Malformed,
    }

    /// A request's route: whole already, or waiting for its JSON body.
    #[derive(Debug)]
    pub enum Route {
        /// A request with no body.
        Ready(Request),
        /// A write: step its body's tokens through `body` until it is `Decoded`.
        Body(BodyDecoder),
    }

    /// A write's body decoder (06.5), the decoder pattern on the server's side.
    #[derive(Debug)]
    pub struct BodyDecoder {
        state: BodyState,
    }

    #[derive(Debug)]
    enum BodyState {}

    /// One answer of a body decoder to one tokenizer event.
    #[derive(Clone, PartialEq, Eq, Hash, Debug)]
    pub enum BodyStep {
        /// Demand this of the tokenizer next.
        Demand(tokenizer::Request),
        /// The body is whole: the request it completes. Terminal.
        Decoded(Request),
        /// Terminal: Forgejo answers such a body `422`.
        Malformed(Malformed),
    }

    /// Routes `method` and `target` (06.1's `path::read`, then the route table).
    pub fn route(method: Method, target: &[u8], limits: &Limits) -> Result<Route, Unrouted> {
        unreachable!("sketch")
    }

    /// The first demand of a body decoder.
    #[must_use]
    pub fn body_first(decoder: &BodyDecoder) -> tokenizer::Request {
        match decoder.state {}
    }

    /// One tokenizer event of a write's body in; one `BodyStep` out.
    pub fn body(decoder: &mut BodyDecoder, limits: &Limits, event: &tokenizer::Event) -> BodyStep {
        match decoder.state {}
    }

    /// An answer as Forgejo writes it, whole: what the fake hands the encoder.
    #[derive(Clone, PartialEq, Eq, Hash, Debug)]
    pub enum Answer {
        /// `/user`.
        User(User),
        /// `/settings/api`.
        ApiSettings(ApiSettings),
        /// A repository, by id or by path.
        Repository(Repository),
        /// `/users/search`.
        Search { ok: bool, data: Box<[User]> },
        /// The issue listing's page.
        Issues(Box<[Issue]>),
        /// An issue; a created or edited one.
        Issue(Issue),
        /// An issue's thread, unpaged.
        Comments(Box<[Comment]>),
        /// A posted comment.
        Comment(Comment),
        /// A pull request; an opened one.
        Pull(PullRequest),
        /// A combined status's page; `sha` none and no statuses are written
        /// `"sha":""` and `"statuses":null`, as Forgejo does (ruling 104).
        CombinedStatus { state: CombinedStatus, sha: Option<ObjectId>, statuses: Box<[CommitStatus]> },
        /// A posted status.
        Status(CommitStatus),
        /// A pull request's reviews' page.
        Reviews(Box<[Review]>),
        /// A posted review.
        Review(Review),
        /// A review's comments, unpaged.
        ReviewComments(Box<[ReviewComment]>),
        /// A branch; a created one.
        Branch(Branch),
        /// The branches' page.
        Branches(Box<[Branch]>),
        /// A protection rule.
        Protection(BranchProtection),
        /// The protections' listing, unpaged.
        Protections(Box<[BranchProtection]>),
        /// The collaborators' page.
        Collaborators(Box<[User]>),
        /// A collaborator's permission.
        Permission(CollaboratorPermission),
        /// A pull request's files' page (its `X-HasMore` is the fake's header).
        Files(Box<[ChangedFile]>),
        /// A comparison, unpaged.
        Compare { compare: Compare, commits: Box<[ObjectId]>, files: Box<[ChangedFile]> },
        /// The runs' page.
        Runs { total_count: u64, runs: Box<[Run]> },
        /// One run.
        Run(Run),
        /// A run's jobs, unpaged.
        Jobs(Box<[Job]>),
        /// An error body, with the status the fake chooses.
        Error(ErrorBody),
    }

    /// Encodes `answer` as Forgejo's JSON (both passes of skein-json's writer). The keys it
    /// writes are among the transcript's for the same answer, each with the same JSON type.
    pub fn encode(answer: &Answer, limits: &Limits) -> Result<Box<[u8]>, writer::Refusal> {
        unreachable!("sketch")
    }
}

// temper_forge_forgejo::header (06.4): the answer fields the decoders of 06.4 read beside a body.
pub mod header {
    use skein_lib::{Duration, Wall};

    /// `Content-Range: bytes <first>-<last>/<total>`; `*` as the total is `None`.
    #[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
    pub struct ContentRange {
        pub first: u64,
        pub last: u64,
        pub total: Option<u64>,
    }

    /// `X-HasMore`: `true` or `false`; anything else `None`.
    #[must_use]
    pub fn has_more(value: &[u8]) -> Option<bool> {
        unreachable!("sketch")
    }

    /// `Content-Range` of a `206`.
    #[must_use]
    pub fn content_range(value: &[u8]) -> Option<ContentRange> {
        unreachable!("sketch")
    }

    /// `Retry-After`, in seconds or as an HTTP date taken from `date` (the answer's `Date`).
    #[must_use]
    pub fn retry_after(value: &[u8], date: Wall) -> Option<Duration> {
        unreachable!("sketch")
    }

    /// An Actions status's link, `<html_url>/actions/runs/<run number>/jobs/<job index>`, or
    /// the same after `html_url`'s path alone, relative to the origin as v16.0.5 writes it
    /// (forge.md, 3.2, "Job attempts"): the two numbers, or `None` for a link of another shape,
    /// one that starts `//`, or one of another scheme or host.
    #[must_use]
    pub fn job_link(target_url: &[u8], html_url: &[u8]) -> Option<(u64, u32)> {
        unreachable!("sketch")
    }
}

// temper_forge_forgejo::failure (06.5): Forgejo's refusal messages, pinned by the transcripts.
pub mod failure {
    /// A refusal one status cannot name alone, told by its `message` (forge.md, 3.4).
    #[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
    pub enum Refusal {
        /// `405` on a merge: the pull request is merged or closed.
        MergeClosed,
        /// `405` on a merge: it conflicts.
        MergeConflict,
        /// `405` on a merge: the base's protection refuses it.
        MergeProtected,
        /// `405` on a merge: a style the repository does not allow.
        MergeStyle,
        /// `409` on a merge: `head out of date`.
        HeadOutOfDate,
        /// `409` on an update: `merge failed because of conflict`.
        UpdateConflict,
        /// `422` on a write whose title or body is empty.
        Empty,
    }

    /// The refusal `status` and `message` name, or `None` for a message no transcript shows.
    #[must_use]
    pub fn refusal(status: u16, message: &[u8]) -> Option<Refusal> {
        unreachable!("sketch")
    }

    /// What the fake answers for `refusal`: its status and Forgejo's message.
    #[must_use]
    pub fn answer(refusal: Refusal) -> (u16, &'static [u8]) {
        unreachable!("sketch")
    }
}
```

## Questions to raise when reached

- **A changed file's status no variant names** (Forgejo may write
  `changed` or `unchanged` beside the five): `Malformed::Value`, and a
  question with the transcript. Default: the five, as the transcripts
  spell them.
- **A list inside a document past `Limits::inner`** (labels, requested
  reviewers, a protection's contexts): the design bounds the answer but
  names no cap per list. Default: `Malformed::Bound`, which 13 reads as
  `TooLarge` (forge.md, 3.4, "an answer past the component's bounds");
  `inner`'s profile value is 13.1's.
- **A refusal message two failures share,** or one that differs between
  two failure transcripts of one kind: no guess (forge.md, 3.4). Default:
  `failure::refusal` answers `None`, and the component's status rule
  applies.
- **A `401`'s message echoes the token sent** (above): the design says
  the API token is "never logged" (forge.md, 3.1) but not that an error
  message may hold it. Default: 13 reads a `401` by its status alone and
  never makes its error decoder; raise it if 13.4 needs the message.
- **A key Forgejo writes twice in one object:** none is expected.
  Default: `Malformed::Shape`, as skein-json's collector fails a
  duplicate.
- **A transcript that does not decode** for a reason the design does not
  cover (a field the design reads is absent from Forgejo's answer): a
  block, kind design gap, with the case and the field. Default: the
  decoder keeps the design's field required; the transcript is not
  edited.

## Done when

- The crate and its transcript tests pass: `cargo nextest run -p
  temper-forge-forgejo -p temper-codecs-world` passes, and `cargo
  nextest run -p temper-codecs-world --profile fuzzy -E
  'binary(fuzzy_forgejo)'` passes.
- It links skein-lib and skein-json only: `cargo tree --locked -p
  temper-forge-forgejo -e normal --depth 1 --prefix none | sort -u`
  (it compiles nothing) prints the crate's own line, a
  `skein-json` line and a `skein-lib` line, and nothing else.
- The gate passed on this session's last landing: `git -C
  ~/src/rust/temper log --format='%(trailers:key=Gate-Result,valueonly)'
  --grep='^Step: temper-06\.' origin/main | head -1` names
  `scripts/check.sh passed`.
