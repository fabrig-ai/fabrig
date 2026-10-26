# Session 13: the forge component and its machine world

`temper-engine-protocol-forge` is the forge component (forge.md): a
child of the engine's protocol layer, one per deployment, that answers
each call of the forge client (`Read`, `Write`, and 02.1's `Locate`) with
exactly one `Answered { call, cost, result }`, made of one or a few HTTP
exchanges with Forgejo v16.0.5 over skein's TLS, HTTP and JSON machines.
It holds the calls in flight, the forges' connections, each forge's
start (the API token's user checked, then the page cap read), the
repositories' paths and the users' logins, and the API tokens, and it
speaks session 06's documents (`temper-forge-forgejo`). It is step code
with no retry: a call goes out once, `Unavailable` only when its request
provably never left, `Timeout` otherwise. The session builds it in four
increments that each land on their own (the boundary and tables; the
connections and the start; the JSON reads; the contents, Actions reads,
writes and failures), then its machine world in
`tests/engine/protocol-forge`, where a scripted HTTP peer answers with
the throwaway Forgejo's recorded responses (session 05). Nothing wires
it into `temper-service` here: that is 14.1.

## Read first

- forge.md, sections 1 to 14; protocol.md, section 5 ("What temper's
  components share") and section 6 (the forge component, "A forge's
  start check"); credentials.md, section 5; performance.md, section 3;
  configuration.md, sections 3 (`forges`) and 7.
- skein at temper's pin (`~/.cargo/git/checkouts/skein-336d5ae5eecfa0f9/b5d3a2c`):
  `crates/skein-http/src/client.rs:1-420` and `client/request.rs:1-60`
  (`Call`, `Body`, `Request`, `Event`, `Error::{Closed, Truncated,
  Stream}`); `crates/skein-tls/src/client.rs:80-130` and
  `config.rs:36-180`; `crates/skein-json/src/tokenizer.rs:40-300`;
  `crates/skein-io/src/records.rs:1-170` (`Connect`, `Connecting`,
  `Connected`, `Stream`, `Failed`, `Close`, `Abort`) and `digest.rs`
  (SHA-256); the precedent of a component that dials out over TLS and
  HTTP, `crates/skein-llm-connection/src/{lib.rs, boundary.rs,
  endpoint.rs, limits.rs}` and `component.rs:60-100, 340-445`;
  `testing/skein-world/src/stream.rs` (`Wire`) and
  `tests/llm-connection/src/world.rs:400-480` (a machine world playing
  io for a component); findings/skein-smith.md, B.
- `temper/06-forgejo-documents.md`: "Choices this plan makes", 06.1's
  Shape, the interface sketches (the documents this component speaks:
  `request::Request`, `client::{encode, decoder, step, keep_after,
  cut, error_decoder}`, `document::*`, `header::*`, `failure::*`) and
  its "Questions to raise when reached".
- `temper/05-forgejo-tools.md`: "Choices this plan makes" ("Transcripts'
  bytes", "The masking") and 05.2's case list, which 13.5 serves.
- `temper/02-adoption.md`, 02.1 (`api::Read::Locate`,
  `api::Answer::Located`).
- temper: `crates/temper-engine-domain-forge-client/src/api.rs:1-420`
  (the vocabulary carried), `boundary.rs:240-295`
  (`Event::Answered`), `limits.rs` (the client's `Limits`), `bounds.rs`
  (how the client measures an answer), `identity.rs` (`effect_key`);
  `crates/temper-engine-domain/src/admission.rs:13-58` (`try_step`) and
  `boundary.rs:229, 393-406` (`Event::Forge`, `Request::Forge`,
  `Request::Busy`); `tests/engine/forge/client/src/translate.rs:1-70`
  (the worlds' marker, which this component's matches byte for byte);
  `tests/engine/forge/src/world.rs:8-31` (the client limits the
  fixtures reuse).
- decisions.md 1, 15, 21 and 22; code-organization.md, sections 2 to 8.

## What the design asks

- **Translation only, no retries** (forge.md, 1): "Each call of the
  forge client (a `Read` or a `Write`) is answered by exactly one
  `Answer` or `Error`, made of one or a few HTTP exchanges on one
  connection." "A call goes out once. Its answer says how many requests
  it cost ... the requests that filled a cache included, and nothing for
  a call that wrote no request."
- **The component** (forge.md, 2): "One component serves every
  configured forge. It holds the calls in flight, the forges'
  connections, each forge's start (section 3.5), the repositories' paths
  and the users, and the translation between the forge client's
  vocabulary and the documents." "An answer ... the root's admission
  cannot take yet ... comes back to the component, which holds it and
  offers it again ...; it holds at most one per call. One the root
  returns undecided during its drain (its `Request::Busy`) stays with the
  component and is never offered again."
- **Conventions** (3.1): `/api/v1` over HTTP/1.1, TLS or loopback
  plaintext; `Authorization: token <t>`; `Host` "the TLS server name, or
  the configured address's host; either with `:port` when the port is not
  the scheme's default"; `User-Agent` "names temper and its version";
  repositories by id, resolved by `GET /api/v1/repositories/{id}`,
  "counts in its cost"; paging "always explicit", the size "the smallest
  of the client's `rows`, the forge's `max_response_items` ..., and the
  number of that listing's largest rows the client's `answer_bytes`
  holds"; users by id, "a table of the two".
- **Each operation** (3.2): the table of requests per call, and its
  notes (keep-up listing, `sort=leastupdate`, unpaged comments, reviews
  mapped by state, CI, job attempts, contents read file by file, merges
  by the repository's `default_merge_style` and `head_commit_id`, "If
  that read fails, the call answers `Timeout`"). "A pull request for a
  head and base": "`PullFor` answers the open one: the lookup's, or, when
  the lookup names one closed or merged, the row of the open pull
  requests' listing whose `head.ref`, `head.repo_id` and `base.ref` are
  the call's head, its repository and its base ... With none open, it
  answers the one the lookup named." "Opening compares first":
  "`OpenPull` reads both branches and compares their commits before it
  writes: a head with no commit its base lacks (`total_commits` 0 ...)
  fails the call `NothingToMerge`, and nothing is written."
- **Decoding** (3.3): bodies "kept whole within `body_bytes`. A longer
  one is carried empty, its key none, its provenance `Unknown`, and its
  revision made from its length"; revisions "the first eight bytes,
  big-endian, of the SHA-256 of its whole body"; provenance; markers;
  "A malformed answer ... fails a write as `Timeout` ... It fails a read
  as `InvalidAnswer`. Either way the connection is closed."
- **Failures** (3.4): the table, and "Unavailable is provable only
  before the first byte"; "an idle connection is closed before the
  forge's keep-alive timeout would close it"; "A call still waiting at
  its deadline is `Unavailable`; one in flight is a `Timeout`, and its
  connection is aborted"; "Never `Busy`"; "Reads before a write":
  "`OpenPull`'s reads come before its `POST` ...: what would be a
  `Timeout` or an `InvalidAnswer` is `Unavailable`"; a branch deleted
  that is not there answers `500`: "a `Timeout`, after which the
  client's find reads the branch"
- **What it keeps** (3.5): each forge's start ("`GET /api/v1/user`. An
  `id` other than the configured `user`, or a `401` or `403`, stops the
  engine ... Then `GET /api/v1/settings/api` ... asked again after
  `start_backoff`, doubling to `start_backoff_max`"); repositories by
  forge and id ("dropped when a call made by its path is answered `404`
  or a redirect, or a merge is refused its style"; "when the table is
  full, the row used longest ago goes"); users ("a user not found fails
  the call as `Missing`"); page caps; API tokens, "never logged, granted
  or stored".
- **The marks** (4): "`<!-- temper:key <the key in lowercase
  hexadecimal> -->` and a line feed, then the text"; "Decoded from a
  body whose first bytes are `<!-- temper:key `, then an even count of
  lowercase hexadecimal digits, then ` -->` and a line feed"; "the
  marker twice its length and 21 bytes more".
- **Limits, the worst case, the checks** (7): the list of limits;
  "Checked at start ...: `connections` equals the client's `calls`;
  `deadline` is at most the client's `lifetime`; `body_bytes` holds the
  largest body temper writes; and, for each listing, at least one of its
  largest rows fits the client's `answer_bytes`".
- **The machines** (8): a connection's states (Closed, Connecting,
  Handshaking, Sending, Reading, Idle), a call's, a forge's start's
  (Checking, Settings, Waiting, Ready, Stopped), and ending.
- **Bytes of unknown length** (performance.md, 3): "A capped buffer that
  grows by doubling ... a write past the cap is refused whole ...
  Finishing trims it to a box of exactly its length", a module of the
  component until skein's lib has it.
- **Tests** (forge.md, 10): step tests of "each call's plan of section
  3.2 against canned answers; the marker's encoder and decoder; the page
  size; names percent-encoded; times both ways; the failures of 3.4";
  "The machine world ... the forge component against a scripted HTTP
  peer, over in-memory streams: every state of section 8's machines,
  faults at every byte, deadlines, a wrong user or another `4xx`
  stopping the start, a path dropped on `404` or a redirect and resolved
  again, a body past `body_bytes`."

## Choices this plan makes

- **The component's shape is skein's LLM connection's**
  (`skein-llm-connection` at the pin): `Component::new(settings,
  &limits, &shared) -> Result<Component, Unusable>`, then `begin`,
  `down` (the routing's `Request`), `up` (an `skein_io::Event` for one of
  its tokens), `fire`, `next_deadline`, `reclaim`, `diagnostics`; two
  output queues, events above and `skein_io::Request`s below, `MAX_OUT`
  reserved in each before each entry point. Its io tokens are its own,
  from 1 below `1 << 60`; the routing puts `tokens::FORGE` (`5 << 60`,
  08.1's) above them (14.1).
- **Its vocabulary** is the sketch's `boundary`: `Request::{Call,
  Refused, Returned, Offer, Close}` and `Event::{Answered, Stopped,
  Closed}`. `Answered` mirrors the forge client's `Event::Answered`
  field for field; the routing wraps it into the root's `Event::Forge`
  and unwraps one handed back (14.1). `Event::Hint` waits for webhooks
  (later.md): no variant nothing emits. A held answer goes up again at
  each `Offer`; a returned one goes when the component closes.
- **What it is told** (`settings::Forge`): the forge's number, kind,
  resolved socket address, transport (a built `skein_tls::Config` and
  `Name`, as skein's LLM connection takes them), `Host`'s value, the API
  token as a `skein_lib::Secret`, and the user. `conventions::host` makes
  `Host`'s value (14.1 calls it); `conventions::USER_AGENT` is
  the literal `temper/0.0.0`, the workspace's `version` (clippy.toml bans
  `env!` in step code), changed with the workspace's version.
- **Limits** are forge.md 7's names, plus what HTTP, JSON and TLS take
  as their own (`headers`, `chunk`, `tls`) and 06's two (`inner`,
  `target_bytes`); `limits::http` and `limits::documents` derive
  skein-http's and 06's limits from them, so no limit has two fields.
  `body_bytes_in` is the tokenizer's `length`. The profile `small`'s
  values are decided here and written by 14.1 into `temper-service`'s
  `limits` module (configuration.md, 7: "Its numbers are constants in
  `temper-service`'s limits module"):

  | Limit | `small` |
  |---|---|
  | `forges` | 4 |
  | `token_bytes` | 512 |
  | `connections` | `forge.client.calls` |
  | `head_bytes` | 16,384 |
  | `headers` | 64 |
  | `body_bytes_in` | 16,777,216 |
  | `depth` | 16 |
  | `chunk` | 4,096 |
  | `string_bytes` | the larger of 512 and `forge.name_bytes` (ruling 131) |
  | `body_bytes` | the larger of 16,384 and `2 × forge.client.op_bytes + 21` (ruling 131) |
  | `inner` | 32 (ruling 131) |
  | `target_bytes` | 4,096 |
  | `repositories` | `forge.repositories` |
  | `users` | 256 |
  | `deadline` | `forge.client.lifetime` |
  | `idle` | 4 s |
  | `start_backoff`, `start_backoff_max` | 1 s, 60 s |
  | `tls` | `read` 16,384, `send` 16,384, `records` `skein_tls::client::MAX_RECORD` |

  `body_bytes` holds the largest body temper writes: a key of up to
  `op_bytes` gives a marker of `2 × op_bytes + 21` (forge.md, 4).
  `idle` is 4 s: forge.md, 7 asks for it "below the keep-alive timeout of
  whatever serves the forge's API (a reverse proxy before Forgejo, when
  there is one) by more than a request's round trip", and 5 s, the
  default of Apache httpd's `KeepAliveTimeout`, is the shortest such
  default a deployment is likely to sit behind. A deployment behind a
  shorter one sets `forge_component.idle_ms`. No fact of the
  conformance tool checks it: the throwaway Forgejo has no proxy, and a
  deployment's proxy is not the tool's to read.
- **The start is a pure machine** (`start::heard`, the sketch's body),
  driven by 13.2 on a connection of each forge's own. A start read's
  deadline is the component's `deadline` from when the read is asked
  (forge.md, 8, "Deadlines are the holder's"), and one that passes is
  heard as unreachable: Waiting, asked again after the backoff.
- **Tables:** repositories by `(forge, id)` (path, object format,
  `default_merge_style`, `html_url`), users by `(forge, id)` (login),
  each a `skein_lib::Map` with a use counter: the entry with the smallest
  counter goes when the table is full. Page caps and tokens per forge, in
  the order of `settings.forges`.
- **The answer's running size** follows the client's own measure
  (`bounds.rs`: a row's `size_of` and its payloads' lengths), so that the
  component stops adding rows where the client would refuse the answer; a
  unit test (13.3) checks the running size against
  `temper_engine_domain_forge_client::answer_bytes_unbounded` on the
  answers it builds.
- **Raw bodies** (a file's side, a job's log) go through
  `grow::Grow` (performance.md, 3), capped at the smaller of
  `max_bytes + 1` and what is left of `answer_bytes`; the `Range` asked
  is `bytes=0-<max_bytes>`, so a byte past `max_bytes` says `truncated`.
- **Revisions** are `skein_io::digest::digest` of the kept body's bytes as
  skein-json hands them (unescaped), its first `u64`; a body not kept
  digests its length then `updated_at`'s nanoseconds, each eight bytes
  big-endian, `0` for a missing time.
- **Object ids** in an answer are checked against the repository row's
  `object_format` (`reading::commit`): another format is an
  `InvalidAnswer` on a read, a `Timeout` on a write.
- **A `401`** is read by its status alone: the component never decodes
  its body (06's question on a message echoing the token).
- **Increments** follow the planner's skeleton, with one move: the
  contents and Actions reads (`PullFiles`, `Compare`, `File`, `Checks`,
  `Job`) go from 13.3 to 13.4, beside the writes, so that neither passes
  600 lines (13.3's JSON reads are fifteen plans; 13.4's writes are
  thirteen short ones).
- **Until 13.3 lands, a call waits for its forge until its deadline**
  and is answered `Unavailable` at cost 0: 13.1 has no connection, 13.2
  connects only for the start. 13.3 removes that.
- **Tests:** step tests in `crates/temper-engine-protocol-forge/src/tests.rs`
  (and `src/tests/<topic>.rs` past 800 lines), fixtures in
  `src/tests/fixtures.rs`; canned answers are literals (step code reads
  no file). The machine world (13.5) serves 05's recorded responses from
  `tools/forgejo-conformance/transcripts/<case>/response`.

## Rules that bind it

- "**`temper-engine-protocol-forge`** links `skein-lib`, `skein-io`,
  `skein-http`, `skein-tls`, `skein-json`, `temper-forge-forgejo` and
  `temper-engine-domain-forge-client`" (code-organization.md, 3); its
  tests may add `skein-world` as a dev-dependency (for `stream::Wire`).
- No increment "adds a retry below the domain (protocol.md, section 5)"
  and none "puts a secret's value in a domain type, a fact, a store
  record, a log or a `Debug` output but skein-lib's `Secret`'s length"
  (code-organization.md, 6): the API token leaves its `Secret` only into
  the `Authorization` field of a request head.
- "**Ceilings, per test** ...: a unit test 20 ms; a focused world test
  0.5 s ...; a fuzzy test about 5 s" and the world's share,
  "`tests/engine/protocol-forge` 1.0 s and 8 s" (code-organization.md,
  5), which 13.5 shares with 14.4: 13.5's tests aim at 0.3 s focused and
  3 s fuzzy, summed.

## Increments

### 13.1 The crate: boundary, settings, limits, tables, markers, the start

Scope: `crates/temper-engine-protocol-forge` (forge.md, 2, 3.1, 3.5, 4,
7 and 8): the modules `boundary`, `settings`, `limits` (`Limits`,
`Shared`, `Listing`, `Unusable`, `check`, `worst_case`, `page_size`,
`documents`, `http`), `marker`, `conventions`, `start` (the pure
machine), `tables` (repositories, users, page caps, tokens), and
`component` with the entry points: `new` runs `limits::check` and
allocates; `begin` sets each forge's start to `Checking`; a `Call` waits
for its forge's start until the component's `deadline` from its arrival,
then `Answered { cost: 0, result: Err(Unavailable) }`; `Refused` holds an
answer and `Offer` emits held answers, at most `MAX_OUT.above` per
entry; `Returned` keeps one until `Close`; `Close` answers `Closed`
(no connection exists yet). `Stopped`'s values exist; nothing emits them
until 13.2.
Needs: nothing beyond the session's start (06.5's crate).
Files: `crates/temper-engine-protocol-forge/{Cargo.toml, src/{lib.rs,
boundary.rs, settings.rs, limits.rs, marker.rs, conventions.rs,
start.rs, tables.rs, component.rs, tests.rs, tests/fixtures.rs}}`
(`Cargo.toml` after `crates/temper-engine-domain-forge-change/Cargo.toml`,
its dependencies code-organization.md 3's, dev-dependency `skein-world`);
the workspace `Cargo.toml` (member and `[workspace.dependencies]`).
Shared hotspot: the workspace `Cargo.toml`.
Shape: the interface sketch's `boundary`, `settings`, `limits`,
`marker`, `conventions`, `start` and `component` (13.1's entry points
as Scope says). `limits::check` checks in this order: each limit non-zero
(`Zero`); `connections == shared.client.calls`; `deadline <=
shared.client.lifetime`; `body_bytes >= 2 × op_bytes + 21`;
`string_bytes >= shared.name_bytes`; `repositories <=
shared.repositories`; 06's `limits::check(&documents(limits))`; each
listing's `page_size(listing, u32::MAX, ..) >= 1`; the stacked
demands (`skein_http::client::largest_read` and `largest_room` against
TLS's `read`/`send` and io's `intake`/`output`, TLS's `LARGEST_READ` and
`largest_room` against io's, the tokenizer's `largest_demand` against
HTTP's `read`); then each forge: at most `forges`, numbers unique, token
1 to `token_bytes` bytes, user above 0, plaintext only to a loopback
`Addr::Ip`, and the longest request head (method, the longest target,
`Host`, `Authorization`, `User-Agent`, `Content-Type`, `Range`) within
`http(..).request`. `worst_case` is the sum of forge.md 7's: per
connection skein-http's, TLS's (when a forge is TLS), the tokenizer's,
the decoder's and one `Grow` at its cap; per call one answer of
`answer_bytes`; the tables at their limits; the held answers (`calls`
of `answer_bytes`). The tables evict by the smallest use counter.
Test shape: unit tests in `src/tests.rs`; fixtures in
`src/tests/fixtures.rs`: `FORGE: u16 = 1`, `WRITER: u64 = 2` (the
transcripts' `writer`), `REPO = api::Repository { forge: 1, repository:
1 }` (`fixture/specimen`), `TOKEN` (`b"0123456789abcdef0123456789abcdef01234567"`),
`limits()` (`forges: 2, token_bytes: 64, connections: 2, head_bytes:
4_096, headers: 32, body_bytes_in: 1_048_576, depth: 16, chunk: 1_024,
string_bytes: 128, body_bytes: 1_045, inner: 8, target_bytes: 1_024,
repositories: 2, users: 8, deadline: 10 s, idle: 5 s, start_backoff: 1
s, start_backoff_max: 8 s, tls: { read: 16_384, send: 16_384, records:
MAX_RECORD }`), `shared()` (`client`: `tests/engine/forge/src/world.rs:8-31`'s
`CLIENT` with `answer_bytes: 65_536`; `name_bytes: 128`, `repositories:
2`, `io`: `sockets 8, names 0, refusals 2, intake 32_768, receive 4_096,
output 32_768, sends 4, accepts 1, backlog 1, close_timeout 1 s, retry
10 ms`), `settings()` (one forge `FORGE`, `Forgejo`, plaintext,
`127.0.0.1:3000`, host `127.0.0.1:3000`, `TOKEN`, `WRITER`), and
`Harness` (`new(settings)` makes the component with `limits()` and
`shared()` and runs `begin`; `call(call: u64, repository, op)`;
`run(span)` advances the clock and fires; `answered(call) ->
Option<&Answered>`; `start(forge) -> Start`). Tests:
`a_marker_round_trips_its_key_and_keeps_the_text` (rows: an empty key,
1 byte, 64 bytes; a text that itself holds a marker),
`a_body_without_a_well_formed_marker_has_no_key` (rows: uppercase hex,
an odd count, no line feed, a space before the marker, a marker after
text), `limits_that_disagree_with_the_client_name_both_limits` (rows:
`connections` 3, `deadline` 11 s, `body_bytes` 1_044, `string_bytes`
127, `repositories` 3, `answer_bytes` 64),
`a_page_is_the_smallest_of_rows_cap_and_answer_room` (rows: cap 2 → 2,
cap 50 → `rows` 8, `answer_bytes` 4_096 → 1 for `Items`),
`the_start_stops_on_another_user_or_a_4xx_and_waits_otherwise`,
`a_call_waiting_for_its_forge_is_unavailable_at_its_deadline`,
`a_full_table_drops_the_entry_used_longest_ago`. Each under 20 ms.
Reference test, in full:

```rust
#[test]
fn the_start_stops_on_another_user_or_a_4xx_and_waits_otherwise() {
    let limits = fixtures::limits();
    let now = Time::from_nanos(1_000_000_000);
    let checking = Start::Checking { next: limits.start_backoff };
    let settings = Start::Settings { next: limits.start_backoff };
    let again = Start::Waiting { until: Time::from_nanos(2_000_000_000), next: Duration::from_secs(2) };
    let rows = [
        ("the configured user", checking, Heard::User { id: fixtures::WRITER }, settings),
        ("another user", checking, Heard::User { id: 1 }, Start::Stopped(Stopped::OtherUser { user: 1 })),
        (
            "a refused token",
            checking,
            Heard::Status { status: 401, retry: None },
            Start::Stopped(Stopped::Refused { read: StartRead::User, status: 401 }),
        ),
        (
            "no Forgejo API at the address",
            settings,
            Heard::Status { status: 404, retry: None },
            Start::Stopped(Stopped::Refused { read: StartRead::ApiSettings, status: 404 }),
        ),
        ("the page cap", settings, Heard::ApiSettings { max_response_items: 2 }, Start::Ready { cap: 2 }),
        ("a 5xx", checking, Heard::Status { status: 503, retry: None }, again),
        (
            "a 429 waits its Retry-After",
            checking,
            Heard::Status { status: 429, retry: Some(Duration::from_secs(5)) },
            Start::Waiting { until: Time::from_nanos(6_000_000_000), next: Duration::from_secs(2) },
        ),
        ("unreachable", checking, Heard::Unanswered, again),
    ];
    for (name, start, heard, expected) in rows {
        assert_eq!(start::heard(start, heard, fixtures::WRITER, now, &limits), expected, "{name}");
    }
}
```

Proof: unit, `cargo nextest run -p temper-engine-protocol-forge`,
run once: a start stops on another user or a `4xx` and waits on the
rest, a marker round-trips and a malformed one has no key, disagreeing
limits are refused naming both, and a waiting call is answered
`Unavailable` at no cost at its deadline (the seven tests); under 0.3 s.
The world layer opens at 13.5, once calls run.
Don't: link `temper-engine-domain-forge` or any root; read `Secret::expose`
outside the code that writes a request head; give `Settings` or a table
a `Debug` that prints a token (`Secret`'s prints its length).
Size: at most 550 lines.
Removes: nothing.

### 13.2 Connections and each forge's start

Scope: forge.md 8's connection machine (`connection.rs`, the sketch's
`connection` module, crate-visible) and the capped buffer (`grow.rs`):
per connection a skein-tls client when the forge is TLS, a skein-http
client, and per answer a skein-json tokenizer with 06's decoder (or a
`Grow` for a raw body); io's `Connect`, then `Connecting`/`Connected`
or `Failed`, its stream requests and events routed by the connection's
token; deadlines (the holder's deadline while Connecting, Handshaking,
Sending, Reading: the call's, or a start read's, the component's
`deadline` from when that read is asked, forge.md, 8, "Deadlines are the
holder's"; no connect or handshake limit of its own; `idle` while Idle),
`Connection: close` and a raw body
past its cap going to Closed; failures as `connection::Failure`
(`Unseen`: nothing written, or skein-http's `Error::Closed`; `Seen`:
anything after). `begin` opens a connection per forge and runs its start
on it (`GET /api/v1/user`, then `GET /api/v1/settings/api`, through
`start::heard`); Waiting's deadline fires Checking again on a new
connection; a `Stopped` start emits `Event::Stopped` once and leaves the
forge's calls waiting until their deadlines. The start's two reads count
in no call's cost. `Close` closes each connection (io's `Close`, or
`Abort` for one in flight) and answers `Closed` once io's last `Closed`
came; no `Answered` goes up after `Close`.
Needs: nothing beyond 13.1.
Files: `crates/temper-engine-protocol-forge/src/{connection.rs, grow.rs,
component.rs, lib.rs, tests.rs, tests/fixtures.rs, tests/connection.rs}`.
Shape: the sketch's `connection` and `grow`; `Component::{begin, up,
fire, next_deadline}` as 13.2 grows them. Request heads: `GET <target>`
with `Host`, `Authorization: token <t>`, `User-Agent`, and for a body
`Content-Type: application/json` (skein-http writes `Content-Length`);
targets and bodies from 06's `client::encode`. An answer's `Date`,
`Retry-After` (06's `header::retry_after`), `X-HasMore`,
`Content-Range` and `Connection: close` are read into `Fields`; a `401`
or `403`'s body is discarded unread; another error's body goes through
06's `client::error_decoder`.
Test shape: unit tests in `src/tests/connection.rs` (`mod connection;`
in `tests.rs`); fixtures grow: `Harness::connected(forge)` (io's
`Connecting` and `Connected` for the forge's open connect, its stream a
`skein_world::stream::Wire` seeded from the harness), `refuse(forge)`
(`Failed { error: Refused }` then `Closed`), `answer(forge, bytes)`
(writes the bytes on the forge's newest connection), `reset(forge)`
(`Stream { up: Failed(Fault::Reset) }`), `request_lines(forge) ->
Vec<String>` (each request line the connection sent), `connections_opened()`,
`stopped() -> Option<(u16, Stopped)>`, `ok(body) -> Vec<u8>` (`HTTP/1.1
200 OK`, `Content-Length` of `body`, `Date: Wed, 01 Jan 2020 00:00:00
GMT`), `status(code) -> Vec<u8>` (no body), `user_ok()` (`ok` of
`{"id":2,"login":"writer"}`), `settings_ok()` (`ok` of
`{"max_response_items":2,"default_paging_num":30}`), and
`Harness::ready()` (`new`, connected, the two answered). Tests:
`the_start_reads_user_then_settings_on_one_connection`,
`a_refused_token_emits_stopped_once_naming_the_forge`,
`a_refused_connect_waits_its_backoff_and_connects_again`,
`an_idle_connection_closes_at_idle`,
`a_start_read_unanswered_at_its_deadline_waits_and_asks_again` (the
connect answered, `GET /api/v1/user` sent and never answered: at the
fixture's `deadline` the connection is aborted, the start is Waiting,
and after `start_backoff` a new connection sends `GET /api/v1/user`),
`a_connection_close_answer_ends_its_connection_after_it`,
`a_reset_before_the_first_byte_is_unseen_and_after_it_seen` (rows: a
reset before `Room` was granted for the head; a reset after the head was
sent),
`the_capped_buffer_doubles_and_refuses_a_write_past_its_cap` (rows:
first 4, writes of 3, 3, 4 with cap 10 → capacities 4, 8, 10, read by a
crate-visible `Grow::capacity`; a write of
1 more refused, the bytes unchanged). Each under 20 ms. Reference test,
in full:

```rust
#[test]
fn the_start_reads_user_then_settings_on_one_connection() {
    let mut h = fixtures::Harness::new(fixtures::settings());
    h.connected(fixtures::FORGE);
    h.answer(fixtures::FORGE, &fixtures::user_ok());
    h.answer(fixtures::FORGE, &fixtures::settings_ok());
    h.run(Duration::from_millis(10));
    assert_eq!(
        h.request_lines(fixtures::FORGE),
        ["GET /api/v1/user HTTP/1.1", "GET /api/v1/settings/api HTTP/1.1"]
    );
    assert_eq!(h.start(fixtures::FORGE), Start::Ready { cap: 2 });
    assert_eq!(h.connections_opened(), 1);
    assert_eq!(h.stopped(), None);
}
```

Proof: unit, `cargo nextest run -p temper-engine-protocol-forge -E
'test(/connection::/)'`, run once: the start reads the user then the
page cap on one connection, a refused token stops it once, a refused
connect waits its backoff, an idle connection closes, a start read
unanswered at its deadline waits and asks again, and a reset tells
`Unseen` from `Seen` by whether a byte went (the eight tests); under 0.3
s. The world layer opens at 13.5.
Don't: retry a start read before its backoff; treat skein-http's
`Truncated { answered: false }` on a reused connection as `Unseen`
(forge.md, 3.4: "it is a `Timeout`"); keep an answer's body past the
decoder's demand.
Size: at most 550 lines.
Removes: nothing.

### 13.3 The JSON reads' plans

Scope: forge.md 3.2's reads whose answers are JSON documents, each a
plan of exchanges (`plan.rs`, `reads.rs`), and temper's reading of the
documents (`reading.rs`): `Items`, `Item` (06's `keep_after`, the
comments kept until the next would pass `answer_bytes` or `rows`, then
06's `cut`), `Pull` (the pull, then `commits/<head.sha>/status?page=1&limit=1`),
`PullFor` (the lookup `GET R/pulls/<base>/<head>`; when the pull it
names is not open, 06.3's `Request::Pulls`, `GET
R/pulls?state=open&sort=oldest&page=<n>&limit=<size>` from page 1,
until a row's `head.ref`, `head.repo_id` and `base.ref` are the call's
head, the repository's id and its base, or a page comes short, the
other rows skipped; then the status read of the pull answered: the open
one, else the lookup's; each page in the cost), `Reviews` (`PENDING`
and `REQUEST_REVIEW` skipped, counted
towards a page coming full), `Statuses` (`more` while the pages read
fall short of `total_count`; a status's `url` its `target_url`, one
that starts with a single `/` given the origin of the repository's
`html_url` before it, forge.md, 3.2, "Links are absolute"; no author:
`creator`, `null` on every Actions status, only fills the users' table
when present), `Remarks` (the `page`th slice of the page
size, cut as it streams), `Branch`, `Branches` (pages from 1 until one
comes short, past `answer_bytes` `TooLarge`), `Protection`
(`effective_branch_protection_name`, then the rule, or the listing when
it names none; `403` `Forbidden`), `Settings` (the row refreshed),
`Collaborators` (each user's permission read by login), `Permission`
(`owner` as `Admin`), `Locate` (`GET /api/v1/repos/<owner>/<name>`, the
row kept, `Located { repository }`; an id past `u32` `TooLarge`); the
resolution before a call whose repository has no row (`GET
/api/v1/repositories/<id>`) or whose user has no login (`GET
/api/v1/users/search?uid=<id>&limit=1`, a user not found `Missing`),
each in the call's cost; the users' table filled from each user object;
page sizes by `limits::page_size` with the forge's cap; the answer's
running size. A call no longer waits past its forge's start: it runs
when its forge is `Ready` and a connection is free (an idle one of its
forge, a new one, or an idle one of another forge closed). A malformed
answer fails the read `InvalidAnswer` and closes the connection; a
call's failure classifies by 13.4's rules for reads (`404` `Missing`
and the path dropped; `3xx` `Unavailable` and the path dropped; `429`
`RateLimited`; `401`, `403`, `423` `Forbidden`; `5xx` `Timeout`;
another `4xx` `InvalidAnswer`), written in `failure.rs` for reads only.
Needs: 02.1's `api::Read::Locate` and `api::Answer::Located` on the
forge client (if they are not on `origin/main` when 13.3 starts, the
session waits for them); 06.3's `Request::Pulls` and `PullBranch::repo_id`.
Files: `crates/temper-engine-protocol-forge/src/{plan.rs, reads.rs,
reading.rs, failure.rs, component.rs, tables.rs, lib.rs, tests.rs,
tests/fixtures.rs, tests/reads.rs}`;
`crates/temper-engine-domain-forge-client/src/api.rs:64`, whose doc
comment `/// Newest matching pull, open or closed.` becomes `/// The
open matching pull, else the oldest.` (that line only). `api::Status`
loses `author` (forge.md, 3.2, "Who posted a status is not carried"),
unless a step before 13.3 removed it: `api.rs:366`, its construction
in `crates/temper-engine-domain-forge-client/src/tests.rs:523`, the
forge world's `tests/engine/forge/client/src/translate.rs:459-469`, and
the store's record `Status` (`schema/store-v1.schema:103`, the generated
code, `src/records/api.rs:409-436`, `src/tests/fixtures.rs:562-572`),
by the README's rule that a durable record's codec follows its record
(ruling 110); nothing else names the field.
Shape: the sketch's `plan` and `reading`. A call's cost is the number of
exchanges whose first byte was written. `Item`'s summary comes from
`GET R/issues/<number>`, its comments from `GET R/issues/<number>/comments`;
`more` when a comment followed the last kept. `Pull`'s `commit` is
`head.sha`, `base_commit` `base.sha` (none when empty), `merged`
`merge_commit_sha` when `merged`, `ci` by `reading::ci`. A summary's
`author` is `user.id`, its `key` `marker::decode(body)`, a `Long` body
carried empty with `key` `None`.
Test shape: unit tests in `src/tests/reads.rs`; fixtures grow:
`Harness::known(repository, row)` (a row kept, so no resolution),
`specimen()` (the row of `fixture/specimen`: owner `fixture`, `sha1`,
`merge`, `http://127.0.0.1:3000/fixture/specimen`), `ok_dated(body)`
(as `ok`). Tests:
`items_list_since_with_leastupdate_and_an_explicit_page`,
`an_item_keeps_comments_above_after_until_the_answer_is_full`,
`a_call_without_a_kept_path_resolves_by_id_and_counts_it`,
`a_pending_review_is_skipped_but_fills_the_page`,
`statuses_say_more_while_pages_fall_short_of_total_count`,
`an_actions_status_reads_with_no_creator_and_an_absolute_link` (a
status with `"creator":null` and `"target_url":"/fixture/specimen/actions/runs/1/jobs/0"`,
as 05.3's `actions-combined-status` has them: its `url` is
`http://127.0.0.1:3000/fixture/specimen/actions/runs/1/jobs/0`),
`revisions_and_provenance_follow_the_body_and_its_times` (rows: equal
times, a later update, a missing `updated_at`, a `Long` body),
`a_malformed_read_answer_is_invalid_and_closes_its_connection`,
`pull_for_lists_open_pulls_when_the_lookup_names_a_closed_one` (the
lookup answers pull 12 closed; page 1 of the open pulls holds pull 9
of another head and pull 13 of this head and base, and the match ends
the listing; then the status read at 13's `head.sha`: `Answer::Pull`
13, open, cost 3),
`pull_for_answers_the_lookups_closed_pull_when_none_is_open` (pull 12
closed; page 1 holds pull 9 alone, of another head, and comes short;
the status read at 12's `head.sha`: `Answer::Pull` 12, closed, cost 3),
`the_running_answer_size_agrees_with_the_clients_measure` (the answers of
the other tests, each measured by `answer_bytes_unbounded`). Each under
20 ms. Reference test, in full:

```rust
#[test]
fn items_list_since_with_leastupdate_and_an_explicit_page() {
    let mut h = fixtures::Harness::ready();
    h.known(fixtures::REPO, fixtures::specimen());
    h.call(1, fixtures::REPO, Op::Read(Read::Items { since: Time::ZERO, page: 1, kind: None }));
    h.answer(
        fixtures::FORGE,
        &fixtures::ok_dated(br#"[{"number":1,"pull_request":null,"state":"open","user":{"id":2,"login":"writer"},"labels":[],"updated_at":"2020-01-01T00:00:01+05:30","title":"t","body":"b"}]"#),
    );
    h.run(Duration::from_millis(10));
    assert_eq!(
        h.request_lines(fixtures::FORGE).last(),
        Some(&String::from(
            "GET /api/v1/repos/fixture/specimen/issues?state=all&since=1970-01-01T00:00:00Z&sort=leastupdate&page=1&limit=2 HTTP/1.1"
        ))
    );
    let answered = h.answered(1).expect("one answer");
    assert_eq!(answered.cost, 1);
    let summary = Summary {
        labels: Box::new([]),
        number: 1,
        kind: Kind::Issue,
        state: State::Open,
        author: 2,
        key: None,
        title: Box::from(*b"t"),
        body: Box::from(*b"b"),
        updated: Time::from_nanos(1_577_817_001_000_000_000),
    };
    assert_eq!(
        answered.result,
        Ok(Answer::Items { items: Box::new([summary]), more: false, now: Time::from_nanos(1_577_836_800_000_000_000) })
    );
}
```

Proof: unit, `cargo nextest run -p temper-engine-protocol-forge -E
'test(/reads::/)'`, run once: the keep-up listing goes out with
`state=all`, `since`, `sort=leastupdate`, `page` and `limit`, a thread
stops at the answer's bound, a missing path is resolved and counted, a
malformed answer is `InvalidAnswer` with its connection closed, and
`PullFor` answers the open pull when the lookup names a closed one, and
an Actions status reads with an absolute link (the eleven tests); under
0.3 s. The world layer opens at 13.5.
Don't: list without `page` and `limit`; read a body the plan does not
carry (skip it); keep a row of a listing past the page.
Size: at most 600 lines; its allowance 2100, 13.3a and 13.3b summed
(decisions.md, ruling 126).
Removes: 13.1's rule that a call waits for its deadline once its forge is
`Ready` (the waiting path stays for a forge still starting).

### 13.4 Contents and Actions reads, the writes, and the failures

Scope: the reads whose answers carry contents or Actions' records:
`PullFiles` (the pull; `head.sha` other than `head` fails `Stale` before
any file; the files' page; per file `GET R/raw/<path>?ref=<merge_base>`
but for `added`, at `previous_filename` for `renamed`, and `GET
R/raw/<filename>?ref=<head>` but for `deleted`; a side's `404` absent;
`more` from `X-HasMore`; a side past what is left of `answer_bytes`
`TooLarge`), `Compare` (both comparisons; per file its sides at `before`
and `after`; `contains_before` when the second's `total_commits` is 0;
the commits from the first's rows), `File` (one raw read with `Range`),
`Checks` (the combined status's pages until one comes short; each
status's `url` as 13.3's `Statuses`; per failing status whose
`target_url` is a job link of the repository's `html_url`, 06's
`header::job_link` (the link with `html_url`, or with its path alone,
as v16.0.5 writes it), the run by `head_sha` and `run_number`, then its
jobs; the job at the link's index, its `attempt`; any miss leaves `job`
none), `Job` (the run, whose `commit_sha` must be the attempt's head;
its jobs, one of which must have the attempt's id and attempt; the log
with `Range`; any miss or `404` `MissingJob`); the writes of 3.2
(`CreateIssue`, `Post` and `Review` behind the key's marker;
`OpenPull` (`GET R/branches/<head>`, `GET R/branches/<base>`, `GET
R/compare/<base's commit.id>...<head's commit.id>`, its
`total_commits` alone kept: 0 fails `NothingToMerge` with no `POST`
written; else `POST R/pulls`; a failure of the three reads that 3.4
makes `Timeout` or `InvalidAnswer` is `Unavailable`, by
`Asked::BeforeWrite`); `Edit`, `Close`, `Reopen` by `PATCH`; `SetReviewers` (the
pull, then `POST` the missing logins and `DELETE` the extra ones);
`Merge` (`Do` the row's `default_merge_style`, `head_commit_id`, then
the pull's `merge_commit_sha`, a failed read after it `Timeout`);
`Update` (`style=merge`); `Status`; `CreateBranch` (a `409`
`Branch(Exists)`); `DeleteBranch` (a `404` `Missing`; v16.0.5's `500`
for a branch not there is a `Timeout`, as every `5xx`)); text that is not
UTF-8 `Refused` with nothing written (06's `client::Refusal::Text`);
`failure::classify` for each row of 3.4, a `4xx` message read by 06's
`failure::refusal`, an unknown message `Refused` on a write and
`InvalidAnswer` on a read; a malformed write answer `Timeout`; a merge
refused its style drops the row.
Needs: nothing beyond 13.3.
Files: `crates/temper-engine-protocol-forge/src/{contents.rs, actions.rs,
writes.rs, failure.rs, plan.rs, lib.rs, tests.rs, tests/fixtures.rs,
tests/contents.rs, tests/writes.rs}`.
Shape: the sketch's `failure` and `grow`'s use for raw bodies. The
merge's style needs the row: a call without one resolves first (cost
counted). `SetReviewers` with nothing missing and nothing extra writes
only the pull's read.
Test shape: unit tests in `src/tests/contents.rs` and
`src/tests/writes.rs`: `pull_files_at_another_head_fail_stale_reading_no_file`,
`a_files_missing_side_is_absent_and_its_read_counted`,
`checks_find_a_failing_jobs_run_and_attempt_from_its_link` (the
failing status's link relative to the origin, as v16.0.5 writes it),
`a_job_log_at_another_head_is_missing_job`,
`creations_carry_the_key_marker_at_the_head_of_their_body`,
`a_merge_sends_the_rows_style_and_reads_its_commit`,
`open_pull_with_nothing_to_merge_compares_and_writes_nothing` (the two
branches at one commit; the comparison answers `total_commits` 0:
`Err(NothingToMerge)`, cost 3, no `POST` among the request lines),
`failures_classify_by_status_call_and_message` (with one row more,
`("a 5xx before opening's POST", Asked::BeforeWrite, 500, None, None,
Error::Unavailable, Drop::Nothing)`; its array 12 rows),
`text_that_is_not_utf8_is_refused_with_nothing_written`. Each under 20
ms. Reference tests, in full:

```rust
// crates/temper-engine-protocol-forge/src/tests/contents.rs
#[test]
fn pull_files_at_another_head_fail_stale_reading_no_file() {
    let mut h = fixtures::Harness::ready();
    h.known(fixtures::REPO, fixtures::specimen());
    let mut head = [0_u8; 32];
    head[0] = 1;
    h.call(1, fixtures::REPO, Op::Read(Read::PullFiles { number: 1, head, page: 1 }));
    h.answer(
        fixtures::FORGE,
        &fixtures::ok(br#"{"number":1,"state":"open","head":{"ref":"work","sha":"0200000000000000000000000000000000000000"},"base":{"ref":"main","sha":"0300000000000000000000000000000000000000"},"merged":false,"merge_commit_sha":null,"merge_base":"0300000000000000000000000000000000000000","mergeable":true,"requested_reviewers":[]}"#),
    );
    h.run(Duration::from_millis(10));
    assert_eq!(
        h.request_lines(fixtures::FORGE).last(),
        Some(&String::from("GET /api/v1/repos/fixture/specimen/pulls/1 HTTP/1.1"))
    );
    let answered = h.answered(1).expect("one answer");
    assert_eq!((answered.cost, answered.result.clone()), (1, Err(Error::Stale)));
}
```

```rust
// crates/temper-engine-protocol-forge/src/tests/writes.rs
#[test]
fn failures_classify_by_status_call_and_message() {
    let five = Some(Duration::from_secs(5));
    let rows: [(&str, Asked, u16, Option<Refusal>, Option<Duration>, Error, Drop); 11] = [
        ("a redirect", Asked::Read, 301, None, None, Error::Unavailable, Drop::Path),
        ("a missing object", Asked::Read, 404, None, None, Error::Missing, Drop::Path),
        ("a job's read not found", Asked::Job, 404, None, None, Error::MissingJob, Drop::Path),
        ("a rate limit", Asked::Read, 429, None, five, Error::RateLimited { after: Duration::from_secs(5) }, Drop::Nothing),
        ("the quota", Asked::Write, 413, None, None, Error::Full, Drop::Nothing),
        ("an archived repository", Asked::Write, 423, None, None, Error::Forbidden, Drop::Nothing),
        ("a merge refused its style", Asked::Merge, 405, Some(Refusal::MergeStyle), None, Error::Refused, Drop::Row),
        ("a merge at an old head", Asked::Merge, 409, Some(Refusal::HeadOutOfDate), None, Error::Stale, Drop::Nothing),
        ("a protected branch deleted", Asked::DeleteBranch, 403, None, None, Error::Protected, Drop::Nothing),
        ("an unknown merge message", Asked::Merge, 405, None, None, Error::Refused, Drop::Nothing),
        ("another 4xx on a read", Asked::Read, 418, None, None, Error::InvalidAnswer, Drop::Nothing),
    ];
    for (name, asked, status, refusal, retry, error, drop) in rows {
        assert_eq!(failure::classify(asked, status, refusal, retry), Classified { error, drop }, "{name}");
    }
}
```

Proof: unit, `cargo nextest run -p temper-engine-protocol-forge -E
'test(/contents::/) | test(/writes::/)'`, run once: a pull's files at
another head read no file, a failing job's attempt is found from its
link, creations carry their marker, a merge names the row's style and
reads its commit, an `OpenPull` with nothing to merge writes nothing,
and each row of forge.md 3.4 above classifies as the table says (the
nine tests); under 0.3 s. The world layer opens at
13.5.
Don't: guess a failure from a message no transcript pinned (06's
`failure::refusal` answers `None`: the status rule applies); retry a
write; send `Update` with another style than `merge`; read a `422` on
`OpenPull`'s `POST` as `NothingToMerge` (v16.0.5 opens such a pull
request; the comparison is the only path to `NothingToMerge`).
Size: at most 1000 lines, so its allowance is 2000 (decisions.md,
ruling 126).
Removes: nothing.

### 13.5 The machine world

Scope: `tests/engine/protocol-forge` (package
`temper-engine-protocol-forge-world`, domain/testing.md 7; forge.md 10, "The
machine world"): the component against a scripted HTTP peer over
`skein_world::stream::Wire`s, the world playing io (connects made or
refused, stream demands and sends, resets, closes, after skein's
`tests/llm-connection/src/world.rs:400-480`); the peer reads each
request head (to `\r\n\r\n`, then `Content-Length` bytes), checks its
`METHOD target` against the script's next, and writes the script's
response: a case's recorded bytes from 05's transcripts, literal bytes,
a reset, the case's bytes cut at a byte, or silence. The stories:
section 8's states (a start through to Ready; Waiting and its backoff;
Stopped by another user and by a `404` to `/settings/api`; a call
through Connecting, Sending, Reading, Idle; Idle closed at `idle`;
`Connection: close`), deadlines (a call waiting for a start; one in
flight, aborted), a path dropped on `404` and on a redirect, resolved
again, a body past `body_bytes` carried empty, and faults at each byte
of one exchange (fuzzy).
Needs: nothing beyond 13.4 (05.2's and 05.3's transcripts landed before
06, which this session needs).
Files: `tests/engine/protocol-forge/{Cargo.toml, src/{lib.rs,
fixtures.rs, machine.rs, peer.rs}, tests/{machine.rs,
fuzzy_machine.rs}}` (`Cargo.toml`'s lints after
`tests/engine/forge/Cargo.toml`); the workspace `Cargo.toml` (member).
Shared hotspot: the workspace `Cargo.toml`.
Shape: ordinary Rust, sketched below ("The machine world"): `Machine`,
`Answer`, `Response`, `Connect`. `Machine::ready(seed, script)` puts
the cases `user` and `settings-api` before `script`.
Test shape: focused tests in `tests/machine.rs`, fixtures in
`src/fixtures.rs` (the crate's `fixtures` of 13.1 copied as the world's
own constants: `FORGE`, `WRITER`, `REPO`, `TOKEN`, `limits()`,
`shared()`, `settings()`; the transcripts read from
`env!("CARGO_MANIFEST_DIR")`'s `../../../tools/forgejo-conformance/transcripts/<case>/response`).
Tests: `the_start_reaches_ready_from_the_recorded_answers`,
`another_user_stops_the_start_once_naming_the_forge`,
`settings_answered_404_stops_the_start_with_its_status`,
`an_unreachable_forge_is_checked_again_after_its_backoff`,
`a_call_waiting_past_its_deadline_is_unavailable_at_no_cost`,
`a_call_silent_past_its_deadline_is_a_timeout_and_aborts`,
`a_path_dropped_on_404_is_resolved_again_by_id`,
`a_redirect_drops_the_path_and_answers_unavailable`,
`a_body_past_body_bytes_is_carried_empty_without_its_key`,
`an_idle_connection_closes_and_the_next_call_connects`. Fuzzy, in
`tests/fuzzy_machine.rs`:
`a_response_cut_at_each_byte_ends_its_call_once` (the `branch` case's
bytes cut at each offset, one machine per offset, seeds 0 to 3 for the
fragments: each call answered once, `Timeout`, cost 2 (the resolution
by id, then the read)). Ceilings: each focused test under 50 ms, the fuzzy test
about 3 s. Reference tests, in full:

```rust
#[test]
fn a_path_dropped_on_404_is_resolved_again_by_id() {
    let mut world = Machine::ready(
        7,
        vec![
            Answer::case("GET /api/v1/repositories/1", "repository-by-id"),
            Answer::case("GET /api/v1/repos/fixture/specimen/branches/gone", "missing-issue"),
            Answer::case("GET /api/v1/repositories/1", "repository-by-id"),
            Answer::case("GET /api/v1/repos/fixture/specimen/branches/main", "branch"),
        ],
    );
    world.call(1, fixtures::REPO, Op::Read(Read::Branch { branch: Box::from(*b"gone") }));
    world.run_for(Duration::from_secs(1));
    world.call(2, fixtures::REPO, Op::Read(Read::Branch { branch: Box::from(*b"main") }));
    world.run_for(Duration::from_secs(1));
    let first = world.answered(1).expect("the first call answered");
    assert_eq!((first.cost, first.result.clone()), (2, Err(Error::Missing)));
    let second = world.answered(2).expect("the second call answered");
    assert_eq!(second.cost, 2, "the path was resolved again, and counted");
    let Ok(Answer::Commit(_)) = &second.result else { panic!("a branch's commit: {:?}", second.result) };
    assert!(world.script_done());
}
```

```rust
// tests/engine/protocol-forge/tests/fuzzy_machine.rs
#[test]
fn a_response_cut_at_each_byte_ends_its_call_once() {
    let length = machine::case_bytes("branch").len();
    for seed in 0..4 {
        for at in 0..length {
            let mut world = Machine::ready(
                seed,
                vec![
                    Answer::case("GET /api/v1/repositories/1", "repository-by-id"),
                    Answer {
                        request: "GET /api/v1/repos/fixture/specimen/branches/main",
                        response: Response::Cut { case: "branch", at },
                    },
                ],
            );
            world.call(1, fixtures::REPO, Op::Read(Read::Branch { branch: Box::from(*b"main") }));
            world.run_for(Duration::from_secs(1));
            let answered = world.answered(1).unwrap_or_else(|| panic!("seed {seed}, cut at {at}: no answer"));
            assert_eq!((answered.cost, answered.result.clone()), (2, Err(Error::Timeout)), "seed {seed}, cut at {at}");
        }
    }
}
```

Proof: world, `cargo nextest run -p temper-engine-protocol-forge-world`
and `cargo nextest run -p temper-engine-protocol-forge-world
--profile fuzzy -E 'binary(fuzzy_machine)'`, each run once: the
component reaches Ready from Forgejo v16.0.5's recorded answers, stops
on another user or a `404`, resolves a dropped path again and counts it,
and ends each call once whatever byte its answer is cut at (the ten
focused tests and the fuzzy one); focused under 0.3 s summed, fuzzy
under 3 s. The joint layer (the fake forge's protocol) opens at 14.3.
Don't: edit a transcript; re-record one; spawn a thread; read any
variable but `CARGO_MANIFEST_DIR`.
Size: world code only (production 0).
Removes: nothing.

## Interface sketches

The same text as `scratch/plan-sketches/src/s13.rs`, which compiles
under the workspace's lints with `crate::s06` standing for
`temper_forge_forgejo` (session 06's sketch).

```rust
//! Sketches of session 13: `temper-engine-protocol-forge`, the forge component (forge.md,
//! sections 2, 3, 4, 7 and 8). The crate does not exist yet: `temper_engine_protocol_forge`
//! below is the crate, each module its module of the same name. It links skein-lib, skein-io,
//! skein-http, skein-tls, skein-json, `temper-forge-forgejo` (session 06's sketch, `crate::s06`)
//! and `temper-engine-domain-forge-client`.
//!
//! Who owns what: the component owns every forge connection, each forge's start, the calls in
//! flight, the repositories' and users' tables, the page caps and the API tokens. The routing
//! (`temper-service`, 14.1) owns its tokens' prefix and carries what passes between the
//! component, io and the root; it decides nothing. What ends what: a `Request::Call` ends with
//! exactly one `Event::Answered`; `Request::Close` ends with `Event::Closed`, the last event; a
//! forge's refused start emits one `Event::Stopped`, and the engine stops.
#![expect(unused_variables, reason = "a sketch's bodies are unreachable; its parameters are its signature")]

pub mod temper_engine_protocol_forge {
    // temper_engine_protocol_forge::boundary (13.1)
    pub mod boundary {
        use skein_lib::Token;
        use temper_engine_domain_forge_client::api;

        /// One call's terminal, the forge client's `Event::Answered` field for field. The
        /// component makes it; the routing hands it to the root as `Event::Forge`, and hands
        /// back one the root did not take (`Request::Refused`, `Request::Returned`).
        #[derive(Clone, PartialEq, Eq, Hash, Debug)]
        pub struct Answered {
            /// The forge client's call token, echoed.
            pub call: Token,
            /// The requests the call wrote, cache fills included; 0 for a call that wrote none.
            pub cost: u32,
            pub result: Result<api::Answer, api::Error>,
        }

        /// From the routing.
        #[derive(Debug)]
        pub enum Request {
            /// The root's `Request::Forge`: ended by exactly one `Event::Answered`.
            Call { call: Token, repository: api::Repository, op: api::Op },
            /// An answer the root's admission did not take (`try_step`'s `Err`): held, offered
            /// again at the next `Offer`; at most one per call.
            Refused(Answered),
            /// An answer the root returned undecided in its drain (`Request::Busy`): kept until
            /// `Close`, never offered again (forge.md, 2).
            Returned(Answered),
            /// The root admits again: held answers go up again, at most `MAX_OUT.above`.
            Offer,
            /// After the root's drain: calls in flight end with their connections and no answer
            /// goes up; `Event::Closed` answers it.
            Close,
        }

        /// To the routing; io's requests go in a queue of their own.
        #[derive(Debug)]
        pub enum Event {
            /// A call's one terminal.
            Answered(Answered),
            /// A forge's start check refused: the engine stops, naming the forge, and `temper
            /// serve` exits 1. At most once per forge.
            Stopped { forge: u16, why: Stopped },
            /// The terminal of `Close`: every connection closed.
            Closed,
        }

        /// Why a forge's start stopped the engine (forge.md, 3.5 and 8).
        #[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
        pub enum Stopped {
            /// `GET /api/v1/user` answered with another user's id.
            OtherUser { user: u64 },
            /// A `4xx` other than `429` to one of the start's reads: `401` or `403` refused the
            /// token; any other, the address names no Forgejo API.
            Refused { read: StartRead, status: u16 },
        }

        /// The start's two reads.
        #[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
        pub enum StartRead {
            /// `GET /api/v1/user`.
            User,
            /// `GET /api/v1/settings/api`.
            ApiSettings,
        }
    }

    // temper_engine_protocol_forge::settings (13.1)
    pub mod settings {
        use alloc::boxed::Box;
        use skein_io::Addr;
        use skein_lib::{List, Secret};

        /// What the component is told at start (forge.md, 2): one entry per configured forge.
        #[derive(Debug)]
        pub struct Settings {
            /// At most `Limits::forges`, numbers unique.
            pub forges: List<Forge>,
        }

        /// One forge, from `application.forges` (configuration.md, 3) and what the binary resolved.
        #[derive(Debug)]
        pub struct Forge {
            /// The domain's forge number: a repository's `forge`.
            pub number: u16,
            pub kind: Kind,
            /// The API's socket address, resolved before the loop; never an in-process name.
            pub address: Addr,
            pub transport: Transport,
            /// `Host`'s value, made by `conventions::host`.
            pub host: Box<[u8]>,
            /// The API token, in `Authorization: token <t>`: never logged, granted or stored.
            pub token: Secret,
            /// The forge user the token belongs to, checked by the start's `GET /api/v1/user`.
            pub user: u64,
        }

        /// The documents a forge speaks.
        #[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
        pub enum Kind {
            /// Forgejo v16.0.5's API under `/api/v1`.
            Forgejo,
        }

        /// A forge's connections.
        #[derive(Debug)]
        pub enum Transport {
            /// Only to a socket address on loopback.
            Plaintext,
            /// TLS with the configured server name and trust.
            Tls { server_name: skein_tls::Name, trust: skein_tls::Config },
        }
    }

    // temper_engine_protocol_forge::limits (13.1)
    pub mod limits {
        use super::settings::Settings;
        use crate::s06::limits as documents;
        use skein_lib::Duration;
        use temper_engine_domain_forge_client as client;

        /// The component's limits (forge.md, 7), in temper's `limits` member as
        /// `forge_component.<field>`; the profile `small`'s values are 14.1's.
        #[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
        pub struct Limits {
            /// The forges served.
            pub forges: u32,
            /// The longest API token.
            pub token_bytes: u32,
            /// Forge connections in all: the client's `calls`.
            pub connections: u32,
            /// The largest response head read (HTTP's `head`).
            pub head_bytes: u32,
            /// The most fields of a response head (HTTP's own).
            pub headers: u32,
            /// The largest response body read (JSON's `length`): time and bytes, not memory.
            pub body_bytes_in: u32,
            /// The JSON nesting followed.
            pub depth: u32,
            /// The most bytes of a string's text demanded at once (JSON's own).
            pub chunk: u32,
            /// The longest name, title, context, description, link or path kept.
            pub string_bytes: u32,
            /// The longest body kept whole: at least twice the client's `op_bytes` and 21.
            pub body_bytes: u32,
            /// The most elements kept of a list inside one document (06's `inner`).
            pub inner: u32,
            /// The longest request target (06's `target_bytes`).
            pub target_bytes: u32,
            /// The repositories' table: at most the forge's `repositories`.
            pub repositories: u32,
            /// The users' table.
            pub users: u32,
            /// A call's, at most the client's `lifetime`; also each start read's.
            pub deadline: Duration,
            /// How long an idle connection is kept, below the forge's keep-alive timeout.
            pub idle: Duration,
            /// A forge's start asked again after this, doubling to `start_backoff_max`.
            pub start_backoff: Duration,
            pub start_backoff_max: Duration,
            /// TLS's own.
            pub tls: skein_tls::client::Limits,
        }

        /// What the component is given of its neighbours' limits (forge.md, 12, "Shared limits").
        #[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
        pub struct Shared {
            /// The forge client's: `calls`, `rows`, `answer_bytes`, `op_bytes`, `lifetime`.
            pub client: client::Limits,
            /// The forge's `name_bytes` (`temper_engine_domain_forge::Limits`).
            pub name_bytes: u32,
            /// The forge's `repositories`.
            pub repositories: u32,
            /// io's caps, which the stacked machines' demands must fit.
            pub io: skein_io::Limits,
        }

        /// The listings whose page size the component chooses (forge.md, 3.1).
        #[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
        pub enum Listing {
            Items,
            Reviews,
            Statuses,
            Remarks,
            Branches,
            Collaborators,
            PullFiles,
        }

        /// A machine stacked on another, whose demand must fit the other's cap.
        #[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
        pub enum Stacked {
            HttpOverTls,
            HttpOverIo,
            TlsOverIo,
            JsonOverHttp,
        }

        /// Why the component cannot start (configuration.md, 7: "each failure naming both limits").
        #[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
        pub enum Unusable {
            /// A limit is 0, or the worst case passes a `u64`.
            Zero,
            /// `connections` is not the client's `calls`.
            Connections { connections: u32, calls: u32 },
            /// `deadline` passes the client's `lifetime`.
            Deadline { deadline: Duration, lifetime: Duration },
            /// `body_bytes` below the largest body temper writes, its marker included.
            BodyBytes { body_bytes: u32, needed: u64 },
            /// `string_bytes` below the forge's `name_bytes`.
            StringBytes { string_bytes: u32, name_bytes: u32 },
            /// `repositories` above the forge's.
            Repositories { repositories: u32, forge: u32 },
            /// One of the listing's largest rows does not fit the client's `answer_bytes`.
            PageSize { listing: Listing },
            /// The documents' limits refused (06's `limits::check`).
            Documents(documents::Problem),
            /// A stacked machine's demand past the cap of the stream below.
            Stack { stacked: Stacked, demand: u32, cap: u32 },
            /// More forges than `forges`.
            Forges { forges: u32 },
            /// Two forges with one number.
            Duplicate { forge: u16 },
            /// An API token empty or past `token_bytes`.
            Token { forge: u16 },
            /// A user of 0.
            User { forge: u16 },
            /// Plaintext to an address not on loopback, or an in-process name.
            Address { forge: u16 },
            /// The longest request head (target, `Host`, token, agent) past HTTP's `request`.
            Request { forge: u16 },
        }

        /// The start's checks (forge.md, 7, "Checked at start"), in this order.
        pub fn check(limits: &Limits, shared: &Shared, settings: &Settings) -> Result<(), Unusable> {
            unreachable!("sketch")
        }

        /// The component's heap at once (forge.md, 7, "The worst case"), or `None`.
        #[must_use]
        pub fn worst_case(limits: &Limits, shared: &Shared, forges: u32) -> Option<u64> {
            unreachable!("sketch")
        }

        /// A listing's page size: the smallest of the client's `rows`, the forge's `cap`, and
        /// how many of the listing's largest rows `answer_bytes` holds; at least 1 once checked.
        #[must_use]
        pub fn page_size(listing: Listing, cap: u32, limits: &Limits, shared: &Shared) -> u32 {
            unreachable!("sketch")
        }

        /// The documents' limits (06.1's `Limits`) these limits make.
        #[must_use]
        pub fn documents(limits: &Limits) -> documents::Limits {
            unreachable!("sketch")
        }

        /// skein-http's client limits for a connection over TLS or over io directly.
        #[must_use]
        pub fn http(limits: &Limits, tls: bool, io: &skein_io::Limits) -> skein_http::client::Limits {
            unreachable!("sketch")
        }
    }

    // temper_engine_protocol_forge::marker (13.1): the keys' markers (forge.md, 4).
    pub mod marker {
        use alloc::boxed::Box;

        /// A marker's head: `<!-- temper:key `.
        pub const HEAD: &[u8] = b"<!-- temper:key ";
        /// A marker's tail: ` -->` and a line feed.
        pub const TAIL: &[u8] = b" -->\n";

        /// A marker's length for a key of `key` bytes: twice it, and 21.
        #[must_use]
        pub fn length(key: u32) -> Option<u32> {
            unreachable!("sketch")
        }

        /// `HEAD`, the key in lowercase hexadecimal, `TAIL`, then `text`; none past `limit`.
        #[must_use]
        pub fn encode(key: &[u8], text: &[u8], limit: u32) -> Option<Box<[u8]>> {
            unreachable!("sketch")
        }

        /// The key at the head of `body`: `HEAD`, an even count of lowercase hexadecimal digits,
        /// `TAIL`; any other body has none. Whether to believe it is the domain's.
        #[must_use]
        pub fn decode(body: &[u8]) -> Option<Box<[u8]>> {
            unreachable!("sketch")
        }
    }

    // temper_engine_protocol_forge::conventions (13.1): forge.md, 3.1's fields.
    pub mod conventions {
        use alloc::boxed::Box;
        use core::net::SocketAddr;

        /// `User-Agent`: temper and its version, the workspace's `version` (a literal: step code
        /// reads no build environment); a version bump changes it with the workspace's.
        pub const USER_AGENT: &str = "temper/0.0.0";

        /// `Host`: the TLS server name, or the address's IP (an IPv6 one in brackets), with
        /// `:port` unless the port is the scheme's default (443 with TLS, 80 without).
        #[must_use]
        pub fn host(address: SocketAddr, server_name: Option<&[u8]>) -> Option<Box<[u8]>> {
            unreachable!("sketch")
        }
    }

    // temper_engine_protocol_forge::start (13.1): a forge's start (forge.md, 3.5 and 8).
    pub mod start {
        use super::boundary::{StartRead, Stopped};
        use super::limits::Limits;
        use skein_lib::{Duration, Time};

        /// Where a forge's start is; `next` is the wait the next unanswered read takes.
        #[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
        pub enum Start {
            /// `GET /api/v1/user` sent or to send.
            Checking { next: Duration },
            /// `GET /api/v1/settings/api` sent or to send.
            Settings { next: Duration },
            /// Asked again at `until`, from `Checking`.
            Waiting { until: Time, next: Duration },
            /// Serves calls; `cap` is the forge's `max_response_items`.
            Ready { cap: u32 },
            /// The engine stops, naming the forge. Terminal.
            Stopped(Stopped),
        }

        /// What one start read heard.
        #[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
        pub enum Heard {
            /// `/user`'s `id`.
            User { id: u64 },
            /// `/settings/api`'s `max_response_items`.
            ApiSettings { max_response_items: u32 },
            /// A status that is no `2xx`, with a `429`'s `Retry-After` wait.
            Status { status: u16, retry: Option<Duration> },
            /// Nothing that decodes: unreachable, cut, past its deadline, malformed.
            Unanswered,
        }

        /// The start's next state after `heard` (forge.md, 8, "A forge's start").
        #[must_use]
        pub fn heard(start: Start, heard: Heard, user: u64, now: Time, limits: &Limits) -> Start {
            let (read, next) = match start {
                Start::Checking { next } => (StartRead::User, next),
                Start::Settings { next } => (StartRead::ApiSettings, next),
                Start::Waiting { .. } | Start::Ready { .. } | Start::Stopped(_) => return start,
            };
            let doubled = next.saturating_mul(2).min(limits.start_backoff_max);
            match heard {
                Heard::User { id } if read == StartRead::User && id == user => Start::Settings { next },
                Heard::User { id } if read == StartRead::User => Start::Stopped(Stopped::OtherUser { user: id }),
                Heard::ApiSettings { max_response_items } if read == StartRead::ApiSettings && max_response_items > 0 => {
                    Start::Ready { cap: max_response_items }
                }
                Heard::Status { status, .. } if status != 429 && (400..500).contains(&status) => {
                    Start::Stopped(Stopped::Refused { read, status })
                }
                Heard::Status { retry: Some(after), .. } if after > next => {
                    Start::Waiting { until: now.saturating_add(after), next: doubled }
                }
                Heard::User { .. } | Heard::ApiSettings { .. } | Heard::Status { .. } | Heard::Unanswered => {
                    Start::Waiting { until: now.saturating_add(next), next: doubled }
                }
            }
        }
    }

    // temper_engine_protocol_forge::component (13.1; connections 13.2; plans 13.3 and 13.4)
    pub mod component {
        use super::boundary::{Event, Request};
        use super::limits::{Limits, Shared, Unusable};
        use super::settings::Settings;
        use skein_lib::{Env, Queue, Time};

        /// The most one entry point emits into each queue; reserve it first.
        #[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
        pub struct MaxOut {
            /// Events for the routing.
            pub above: u32,
            /// io requests.
            pub below: u32,
        }

        /// Each entry point's most; the step tests count against it.
        pub const MAX_OUT: MaxOut = MaxOut { above: 16, below: 16 };

        /// What the component counted that no call's answer says (forge.md, 3.3).
        #[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
        pub struct Diagnostics {
            /// Bodies past `body_bytes`, carried empty.
            pub long_bodies: u64,
            /// Connections closed because an answer was malformed.
            pub malformed: u64,
        }

        /// The forge component: one per deployment, serving every configured forge.
        #[derive(Debug)]
        pub struct Component {
            _owned: (),
        }

        impl Component {
            /// Checks `settings` and `limits` against `shared` (`limits::check`) and allocates by them.
            pub fn new(settings: Settings, limits: &Limits, shared: &Shared) -> Result<Component, Unusable> {
                unreachable!("sketch")
            }

            /// Begins each forge's start: its first connection, then `GET /api/v1/user`.
            pub fn begin(&mut self, env: &Env<Limits>, up: &mut Queue<Event>, io: &mut Queue<skein_io::Request>) {
                unreachable!("sketch")
            }

            /// A request from the routing. Reserve `MAX_OUT` first.
            pub fn down(
                &mut self,
                env: &Env<Limits>,
                request: Request,
                up: &mut Queue<Event>,
                io: &mut Queue<skein_io::Request>,
            ) {
                unreachable!("sketch")
            }

            /// An io event for one of the component's tokens. Reserve `MAX_OUT` first.
            pub fn up(
                &mut self,
                env: &Env<Limits>,
                event: skein_io::Event,
                up: &mut Queue<Event>,
                io: &mut Queue<skein_io::Request>,
            ) {
                unreachable!("sketch")
            }

            /// Every deadline due at `env.now`: calls, connections, idle, the starts' backoffs.
            pub fn fire(&mut self, env: &Env<Limits>, up: &mut Queue<Event>, io: &mut Queue<skein_io::Request>) {
                unreachable!("sketch")
            }

            /// The earliest deadline it holds.
            #[must_use]
            pub fn next_deadline(&self) -> Option<Time> {
                unreachable!("sketch")
            }

            /// What it counted.
            #[must_use]
            pub fn diagnostics(&self) -> Diagnostics {
                unreachable!("sketch")
            }

            /// The owner's reclaim point.
            pub fn reclaim(&mut self) {
                unreachable!("sketch")
            }
        }
    }

    // temper_engine_protocol_forge::grow (13.2): performance.md, 3's capped buffer that grows by
    // doubling, for a raw body (a file's side, a job's log); moves to skein's lib unchanged.
    pub mod grow {
        use alloc::boxed::Box;
        use skein_lib::List;

        /// A write past the cap, refused whole: nothing written.
        #[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
        pub struct Full;

        /// Bytes of unknown length, at most `cap`.
        #[derive(Debug)]
        pub struct Grow {
            bytes: List<u8>,
            cap: u32,
        }

        impl Grow {
            /// Starts at `first` bytes (at most `cap`).
            #[must_use]
            pub fn new(first: u32, cap: u32) -> Grow {
                Grow { bytes: List::with_capacity(first.min(cap)), cap }
            }

            /// Appends `piece` whole, growing to twice the capacity or what it needs, never past `cap`.
            pub fn write(&mut self, piece: &[u8]) -> Result<(), Full> {
                let Ok(length) = u32::try_from(piece.len()) else { return Err(Full) };
                let Some(need) = self.bytes.len().checked_add(length) else { return Err(Full) };
                if need > self.cap {
                    return Err(Full);
                }
                if need > self.bytes.capacity() {
                    let mut grown = List::with_capacity(need.max(self.bytes.capacity().saturating_mul(2)).min(self.cap));
                    for byte in self.bytes.as_slice() {
                        grown.push(*byte).expect("grown past the length");
                    }
                    self.bytes = grown;
                }
                for byte in piece {
                    self.bytes.push(*byte).expect("room checked above");
                }
                Ok(())
            }

            /// The bytes, in a box of exactly their length.
            #[must_use]
            pub fn finish(self) -> Box<[u8]> {
                self.bytes.into_boxed()
            }
        }
    }

    // temper_engine_protocol_forge::connection (13.2): one forge connection's machine (forge.md, 8);
    // `pub(crate)` in the crate: what a connection tells the call that holds it.
    pub mod connection {
        use crate::s06::{decode, document, header};
        use alloc::boxed::Box;
        use skein_lib::{Duration, Wall};

        /// A forge connection's state; the component arms its deadline.
        #[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
        pub enum Phase {
            /// Nothing held.
            Closed,
            /// io's connect, until the call's deadline (or a start read's).
            Connecting,
            /// TLS's handshake, until the same deadline.
            Handshaking,
            /// The request going down; `written` once its first byte went.
            Sending { written: bool },
            /// The head, then the body as the decoder demands.
            Reading,
            /// Kept for its forge until `idle`.
            Idle,
        }

        /// The answer's fields the plans read beside its body.
        #[derive(Clone, PartialEq, Eq, Hash, Debug)]
        pub struct Fields {
            /// `Date`: the time the answer was made.
            pub date: Option<Wall>,
            /// `Retry-After` of a `429`, from `Date` when it is a date.
            pub retry_after: Option<Duration>,
            /// `X-HasMore`.
            pub has_more: Option<bool>,
            /// `Content-Range` of a `206`.
            pub content_range: Option<header::ContentRange>,
            /// `Connection: close`: Closed after this answer, not Idle.
            pub close: bool,
        }

        /// What the body was.
        #[derive(Clone, PartialEq, Eq, Hash, Debug)]
        pub enum Body {
            /// A JSON answer, whole; its rows went up before it.
            Document(document::Document),
            /// A raw answer within its cap; `more` when bytes followed the cap.
            Raw { bytes: Box<[u8]>, more: bool },
            /// No body, or one not read (a failure's status needs no message).
            Empty,
            /// Not the document its request gets: the connection closes.
            Malformed(decode::Malformed),
        }

        /// Why an exchange failed (forge.md, 3.4, "Unavailable is provable only before the first byte").
        #[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
        pub enum Failure {
            /// The server never saw the request: no byte written (a refused connect, a failed
            /// handshake, the deadline while waiting), or skein-http's `Closed`: `Unavailable`.
            Unseen,
            /// It may have been read: a reset, the stream's end, `Truncated`, `Stream`, the
            /// deadline in flight, an answer HTTP refuses: `Timeout`.
            Seen,
        }

        /// What a connection tells the call it serves, in order: `Head`, `Row`s, `Done`; or `Failed`.
        #[derive(Clone, PartialEq, Eq, Hash, Debug)]
        pub enum Heard {
            Head { status: u16, fields: Fields },
            Row(document::Row),
            Done(Body),
            Failed(Failure),
        }
    }

    // temper_engine_protocol_forge::plan (13.3, 13.4): a call as one or a few exchanges.
    pub mod plan {
        use crate::s06::request;
        use temper_engine_domain_forge_client::api;

        /// One exchange of a call: the request, and a raw read's `Range` of `0-<max>`.
        #[derive(Clone, PartialEq, Eq, Hash, Debug)]
        pub struct Exchange {
            pub request: request::Request,
            pub range: Option<u32>,
        }

        /// What a call does next.
        #[derive(Clone, PartialEq, Eq, Hash, Debug)]
        pub enum Step {
            /// Send this on the call's connection; its cost grows by one once a byte is written.
            Send(Exchange),
            /// The call's result; with its cost, it goes up as `Event::Answered`.
            Done(Result<api::Answer, api::Error>),
        }
    }

    // temper_engine_protocol_forge::reading (13.3): temper's reading of Forgejo's words.
    pub mod reading {
        use crate::s06::document::{Body, Permission, ReviewState, StatusState};
        use crate::s06::id::{ObjectFormat, ObjectId};
        use skein_lib::{Time, Wall};
        use temper_engine_domain_forge_client::api;

        /// A combined status: `total_count` 0 is `None` whatever the state says (forge.md, 3.2, "CI").
        #[must_use]
        pub fn ci(state: StatusState, total_count: u64) -> api::Ci {
            if total_count == 0 {
                return api::Ci::None;
            }
            match state {
                StatusState::Failure | StatusState::Error => api::Ci::Failed,
                StatusState::Pending => api::Ci::Pending,
                StatusState::Success | StatusState::Warning | StatusState::Skipped => api::Ci::Passed,
            }
        }

        /// A status's own state, mapped as `ci` maps a combined one.
        #[must_use]
        pub fn check(state: StatusState) -> api::Check {
            unreachable!("sketch")
        }

        /// A review's verdict; `None` for `PENDING` and `REQUEST_REVIEW`, which are skipped; a
        /// dismissed review is `Comment`.
        #[must_use]
        pub fn verdict(state: ReviewState, dismissed: bool) -> Option<api::Verdict> {
            unreachable!("sketch")
        }

        /// `owner` reads as `Admin`.
        #[must_use]
        pub fn permission(permission: Permission) -> api::Permission {
            unreachable!("sketch")
        }

        /// `Original` when `updated` equals `created`, `Revised` when later, `Unknown` when either
        /// is missing or the body was not kept (forge.md, 3.3).
        #[must_use]
        pub fn provenance(created: Option<Wall>, updated: Option<Wall>, body: &Body) -> api::Provenance {
            unreachable!("sketch")
        }

        /// The first eight bytes, big-endian, of the SHA-256 (`skein_io::digest`) of a kept body;
        /// of a body not kept, of its length then `updated`'s nanoseconds, each eight bytes big-endian.
        #[must_use]
        pub fn revision(body: &Body, updated: Option<Wall>) -> u64 {
            unreachable!("sketch")
        }

        /// An id in the domain's 32 bytes, if its format is the repository's.
        #[must_use]
        pub fn commit(id: ObjectId, format: ObjectFormat) -> Option<api::Commit> {
            unreachable!("sketch")
        }

        /// The forge's clock as the client reads it: nanoseconds since the epoch.
        #[must_use]
        pub fn time(wall: Wall) -> Time {
            Time::from_nanos(wall.as_nanos())
        }
    }

    // temper_engine_protocol_forge::failure (13.4): forge.md, 3.4's table.
    pub mod failure {
        use crate::s06::failure::Refusal;
        use skein_lib::Duration;
        use temper_engine_domain_forge_client::api;

        /// The call a failure's status answers, where 3.4's rows name one.
        #[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
        pub enum Asked {
            /// A read but `Job`'s.
            Read,
            /// One of `Read::Job`'s reads.
            Job,
            /// A write 3.4 names no row for.
            Write,
            OpenPull,
            /// One of `OpenPull`'s reads before its `POST` (forge.md, 3.4, "Reads before a write").
            BeforeWrite,
            Merge,
            Update,
            DeleteBranch,
        }

        /// What the failure drops of the component's tables (forge.md, 3.5).
        #[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
        pub enum Drop {
            Nothing,
            /// The repository's path: the next call resolves its id again.
            Path,
            /// The repository's whole row (a merge refused its style).
            Row,
        }

        /// A failure as the call's `Error`, and what it drops.
        #[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
        pub struct Classified {
            pub error: api::Error,
            pub drop: Drop,
        }

        /// A `3xx`, `4xx` or `5xx` answer to `asked`, its message read into `refusal` (06.5), a
        /// `429`'s wait in `retry`. A `409` on `CreateBranch` is `Branch(Exists)`, an answer the
        /// plan reads before this.
        #[must_use]
        pub fn classify(asked: Asked, status: u16, refusal: Option<Refusal>, retry: Option<Duration>) -> Classified {
            unreachable!("sketch")
        }
    }
}
```

The machine world, ordinary Rust, not compiled:

```rust
// temper_engine_protocol_forge_world::machine (tests/engine/protocol-forge/src/machine.rs), ordinary Rust, not compiled.

/// One scripted exchange: the request's `METHOD target` it expects, and what the peer answers.
pub struct Answer { pub request: &'static str, pub response: Response }

impl Answer {
    /// A recorded case's response (`tools/forgejo-conformance/transcripts/<case>/response`).
    pub fn case(request: &'static str, case: &'static str) -> Answer;
}

/// What the peer writes for one request.
pub enum Response {
    /// The case's recorded bytes: status line, headers, body.
    Case(&'static str),
    /// Literal bytes.
    Bytes(Vec<u8>),
    /// The stream reset before any byte of the answer.
    Reset,
    /// The case's first `at` bytes, then the stream's end.
    Cut { case: &'static str, at: usize },
    /// Nothing: the call's deadline decides.
    Silent,
}

/// What io does with the component's next connect.
pub enum Connect { Made, Refused }

/// The component, io played by the world, and one scripted peer per connection.
pub struct Machine { /* the component and its env, the events and io queues, the peers by
                       token (each a `skein_world::stream::Wire`), the script, the connects,
                       the request heads read */ }

impl Machine {
    /// Made with the fixtures' limits and settings, `begin` run; the script answers in order.
    pub fn new(seed: u64, script: Vec<Answer>) -> Machine;
    /// As `new`, with the cases `user` and `settings-api` first.
    pub fn ready(seed: u64, script: Vec<Answer>) -> Machine;
    /// As `new`, with other limits or settings.
    pub fn with(seed: u64, limits: Limits, settings: Settings, script: Vec<Answer>) -> Machine;
    /// The next connects' outcomes (default: made).
    pub fn connects(&mut self, connects: &[Connect]);
    pub fn call(&mut self, call: u64, repository: api::Repository, op: api::Op);
    /// Ticks io, the peers and `fire` until `span` has passed on the clock.
    pub fn run_for(&mut self, span: Duration);
    /// The call's answer; a second answer for one call fails the test where it comes.
    pub fn answered(&self, call: u64) -> Option<&Answered>;
    pub fn stopped(&self) -> Option<(u16, Stopped)>;
    /// Each request head the peers read, in order.
    pub fn requests(&self) -> &[Vec<u8>];
    pub fn connections_opened(&self) -> u32;
    /// Whether the script was answered to its end.
    pub fn script_done(&self) -> bool;
    /// `Close`, then ticks until `Event::Closed`; whether it came within 1,000 ticks.
    pub fn close(&mut self) -> bool;
}

/// A case's recorded response bytes, as `Response::Case` serves them.
pub fn case_bytes(case: &str) -> Vec<u8>;
```

## Questions to raise when reached

- **A `1xx` left over, a `2xx` other than the one a route answers, or a
  `3xx` to a start read:** forge.md 8 names `4xx`, `429`, `5xx`,
  unreachable and malformed only. Default: Waiting, asked again.
- **A `429` with no `Retry-After`:** forge.md 3.4 writes
  `RateLimited { after }` from it. Default: `after` 0 for a call; the
  backoff alone for a start read.
- **A list inside a document past `inner`** (06's question: labels,
  requested reviewers, contexts): 06's `Malformed::Bound`, read as
  `TooLarge`, so an issue with more labels than `inner` fails the
  keep-up listing at each pass. Default: `TooLarge`, `inner` 64 in the
  profile; raise it with the case if a story meets it.
- **The job index in a status link:** forge.md 3.2 says "the one at that
  index of the run's jobs"; whether the index counts from 0 is 05.3's
  fact `actions-status-link-is-runs-n-jobs-i`. Default: from 0, the
  position in the jobs listing; follow the fact if it says otherwise.
- **`idle` against Forgejo's keep-alive timeout:** forge.md 7 says
  "below the forge's keep-alive timeout", whose value no fact records.
  Default: 5 s.
- **A revision's bytes:** "the SHA-256 of its whole body as the forge
  returns it" (forge.md, 3.3), and skein-json hands the string
  unescaped. Default: the unescaped bytes.

## Done when

- The crate and its machine world pass: `cargo nextest run -p
  temper-engine-protocol-forge -p temper-engine-protocol-forge-world`
  passes, and `cargo nextest run -p
  temper-engine-protocol-forge-world --profile fuzzy -E
  'binary(fuzzy_machine)'` passes.
- It links what code-organization.md 3 names: `cargo tree --locked -p
  temper-engine-protocol-forge -e normal --depth 1 --prefix none | sort
  -u` prints the crate's own line and lines for `skein-http`,
  `skein-io`, `skein-json`, `skein-lib`, `skein-tls`,
  `temper-engine-domain-forge-client` and `temper-forge-forgejo`, and
  nothing else.
- The token leaves its `Secret` in one module: `git grep -n 'expose()'
  -- crates/temper-engine-protocol-forge/src ':!*tests*'` prints lines
  of one file only.
- The pins held: `grep -oE 'forgetic/(jig|smith|skein)\.git#[0-9a-f]+'
  Cargo.lock | sort -u` prints the three pinned revisions and no other line.
- The gate passed on this session's last landing: `git -C
  ~/src/rust/temper log --format='%(trailers:key=Gate-Result,valueonly)'
  --grep='^Step: temper-13\.' origin/main | head -1` names
  `scripts/check.sh passed`.
