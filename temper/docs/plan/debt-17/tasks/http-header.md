+++
id = "http-header"
part = "http-header"
kind = "feature"
needs = []
files = ["skein/crates/skein-http/src/header.rs", "skein/crates/skein-http/src/lib.rs", "skein/crates/skein-http/src/client.rs", "skein/crates/skein-http/src/client/head.rs", "skein/crates/skein-http/src/client/request.rs", "skein/crates/skein-http/src/server/head.rs", "skein/crates/skein-http/src/tests/**", "skein/crates/skein-llm/src/client.rs", "skein/crates/skein-llm/src/client/tests.rs", "skein/crates/skein-fake-llm-protocol/src/provider.rs", "skein/docs/design/http.status.md", "skein/docs/development/debt/D008-http-head-parsing.md"]
effort = "high"
minutes = 70
+++

# skein: one reading of a head, for both of skein-http's machines and for skein-llm

## Goal

Pay `skein/docs/development/debt/D008-http-head-parsing.md`. `skein-http/src/header.rs` exists, private, with the predicates;
each machine still has its own `field`, `content_length`, `persistent` and transfer-coding walk. They move into it, and
its predicates and the head measure go public so that skein-llm stops copying them.

Acceptance:

- `rg -n 'fn (field|content_length|persistent|transfer_encoding|codings)\b' skein/crates/skein-http/src/client/head.rs
  skein/crates/skein-http/src/server/head.rs` finds nothing.
- `rg -n 'fn (header_name|header_value|trim|head_length)\b' skein/crates/skein-llm/src/client.rs` finds nothing but a
  `header_value` that is one line over `is_field_byte`, or a `head_length` that keeps skein-llm's own target and field checks
  and takes its sum from `request_head_len`.
- `rg -n "b\"!#\\$%&" skein/crates/skein-llm skein/crates/skein-fake-llm-protocol` finds nothing.
- Every test and transcript of `skein-http`, `skein-http-world`, `skein-llm`, `skein-llm-world` and
  `skein-fake-llm-protocol` passes unchanged.
- Production lines fall by at least 60 across these crates.
- The item is deleted, and `http.status.md`'s `2:` entry leaves.

## Inputs

- `temper/docs/plan/debt-17/notes/common.md`
- `skein/docs/development/debt/D008-http-head-parsing.md`: `header.rs` already exists, whatever it says.
- `skein/docs/design/http.md`, 2 (this cut wrote the module), 3.2, 5.2, 7.

## Owns and may change

- **Owns:** the files in this spec's header.
- **The head measure sits in `client`:** it needs `client::Body`. `client::request_head_len` beside the `header` module is what
  the item's "its predicates and head measure" comes to.
- **Stay as they are:** the world oracles (`skein/tests/http/src/generate.rs`, `requests.rs`, `reference.rs`), which are
  independent on purpose; `skein-forge-setup/src/host/limits.rs` and `skein-forge-connection/src/limits.rs`, which are outside the
  item. `Header` keeps its `skein_http::Header` path; `pub(crate)` readings stay crate-private, so nothing public widens but
  what's below.
- `skein-llm-connection` only calls `llm::client::request_head`, and nothing outside skein-http reads `head`. If a reader in
  smith, jig or temper turns up, stop and ask.

## Steps

1. `skein-http/src/header.rs`, with `pub mod header;` in `lib.rs`:

   ```rust
   // public
   #[must_use] pub fn is_tchar(byte: u8) -> bool;
   #[must_use] pub fn is_field_byte(byte: u8) -> bool;
   /// Not empty, and every byte a tchar (RFC 9110, 5.6.2).
   #[must_use] pub fn is_token(bytes: &[u8]) -> bool;
   #[must_use] pub fn trim(bytes: &[u8]) -> &[u8];
   /// The bytes a field takes in a head: its name, `: `, its value and a line's end.
   #[must_use] pub const fn field_len(name: usize, value: usize) -> Option<usize>;

   // crate-private
   pub(crate) enum FieldError { Malformed, Full }
   pub(crate) fn field(line: &[u8], headers: &mut List<Header>) -> Result<(), FieldError>;
   pub(crate) enum Length { Absent, Valid(u64), Invalid }
   pub(crate) fn content_length(headers: &[Header]) -> Length;
   pub(crate) enum Coding { Absent, Chunked, Coded, Faulty }
   pub(crate) fn transfer_coding(headers: &[Header]) -> Coding;
   pub(crate) fn persistent(version: Version, headers: &[Header]) -> bool;
   ```

2. Both machines read through them, each keeping its own policy:

   | Input | `Coding` | Client | Server |
   |---|---|---|---|
   | no field | `Absent` | by its length, or until the end | by its length, or none |
   | `chunked` alone | `Chunked` | chunks | chunks |
   | `gzip, chunked` | `Coded` | `Framing` | `Coding` (501) |
   | `gzip`; `chunked, gzip`; `chunked, chunked`; an empty list | `Faulty` | `Framing` | `Framing` (400) |

   These stay per side: the client's `fold` and its fold before any field; the server's `Rejection::Header` for a line
   starting with whitespace; the client's `Empty` for HEAD, 204 and 304; the server's refusal of a coding in HTTP/1.0 or beside
   a length before the walk; its later check of a length past its limit. `FieldError` maps to `Error::Header` or
   `TooManyHeaders` on one side and `Rejection`'s on the other. Keep each side's error order: a transcript pins it.

   Add the row the server lacks to `tests/server/head.rs`'s fold test: a line of whitespace alone is `Rejection::Header`.

3. `client/request.rs`, exported from `client` and used by `request::write`, so that the measure can't drift:

   ```rust
   /// The exact bytes of a request head whose fields take `fields` bytes in all (`field_len`).
   #[must_use] pub fn request_head_len(method: Method, target: usize, fields: usize, body: Body, close: bool) -> Option<usize>;
   ```

   `usize`, so that `skein-llm`'s `request_head` keeps mapping past-`u32` to `Error::Limit`. `Measure` stays private.

4. `skein-llm/src/client.rs`: `header_name` is `header::is_token`, `trim` is `header::trim`, and `head_length`'s sum is
   `field_len` and `request_head_len`. skein-llm keeps its target, authority, reserved-name and duplicate checks. Its
   value check is **stricter** than `is_field_byte`, and that stays: `value.iter().all(|&b| b.is_ascii() && header::is_field_byte(b))`.
   Add a test that a non-ASCII endpoint or credential field value is `Error::Invalid`, next to `tests/llm/tests/suite/client.rs`'s
   `\r\n` one or in `skein-llm/src/client/tests.rs`, and break it by hand by dropping `is_ascii`.
   `skein-fake-llm-protocol/src/provider.rs` takes the same predicates, with the same strictness, for its inline token, value and
   sum.

5. `heavy cargo nextest run -p skein-http -p skein-http-world -p skein-llm -p skein-fake-llm-protocol -p skein-llm-world`, then
   `-p skein-llm-connection`. `heavy cargo check --tests -p skein-fake-peers`.

6. The status line, then delete the item. `cargo fmt`; `python3 -I -B skein/scripts/old_forms.py`; `heavy cargo clippy -p
   skein-http -p skein-llm -p skein-fake-llm-protocol --all-targets -- -D warnings`; the citations check. Commit and push.

## Report

The common note's items, what you broke by hand and what failed, and production lines by crate.
