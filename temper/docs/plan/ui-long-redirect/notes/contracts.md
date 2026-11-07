# ui-long-redirect: the binding shapes

The run's calls on what crosses a task or a tree. The design they follow is
`skein/docs/design/oauth.md`, 6.3, 6.5, 6.7 and 6.8, and
`jig/docs/design/protocol/configuration.md`, 6.

## 1. The driver's redirect bound (task `redirect-bound`)

In `skein/crates/skein-oauth-accounts/src/limits.rs`, beside `Limits`:

```rust
pub struct Limits {
    // ... as today, then after `listeners`:
    /// The longest returning redirect the listener or a paste reads, the whole
    /// address with its query (oauth.md, 6.5 and 6.7).
    pub redirect_bytes: u32,
    // ... as today
}

/// A browser's own request fields beside its request line, with their line ends:
/// its navigation fields and the cookies it holds for the loopback host (oauth.md, 6.7).
pub const BROWSER_FIELDS: u32 = 8_192;

/// The shortest `redirect_bytes` that holds a full answer at the client's bounds:
/// the registered URI at `uri_bytes`, `?code=` and `code_bytes`, `&scope=` and
/// `scope_bytes`, `&state=` and `state_bytes`, `&client_id=` and
/// `issued_client_bytes`, unescaped; none past `u32` (oauth.md, 6.7).
#[must_use]
pub const fn redirect_room(limits: &Limits) -> Option<u32>;

/// The least `server.head` that holds a browser's request for a redirect of
/// `redirect_bytes`: `GET `, the target, ` HTTP/1.1\r\n`, `BROWSER_FIELDS` and the
/// blank line; none past `u32` (oauth.md, 6.7).
#[must_use]
pub const fn listener_head(redirect_bytes: u32) -> Option<u32>;
```

- `redirect_room` is the five bounds plus 31 (`?code=` 6, `&scope=` 7, `&state=` 7,
  `&client_id=` 11). Under the small profile's client: 4,096 + 4,096 + 4,096 + 64 +
  1,024 + 31 = **13,407**.
- `listener_head(n)` is `n + BROWSER_FIELDS + 17`. `listener_head(16_384)` = **24,593**.
- `lib.rs` exports `pub use limits::{BROWSER_FIELDS, Limits, listener_head, redirect_room, valid};`.
- `valid` checks neither (oauth.md, 6.7): the owner's profile does (section 2).
- `redirect.rs`' `parse` and `from_head` bound the address by `redirect_bytes`, never
  by the client's `url_bytes`; their callers pass it (the signatures inside the crate
  are the task's). Each field the client reads keeps its client bound.
- The field's value in every literal outside the crate's own tests keeps today's
  behaviour, the literal's own client `url_bytes`, but the small profile's:

  | literal | `redirect_bytes` |
  |---|---|
  | `jig/crates/jig-protocol-config/src/profile/small.rs`, `DRIVER` | `16_384` (standard inherits it through `..small::DRIVER`) |
  | `jig/crates/jig-protocol-accounts/src/tests/fixtures.rs` | its client's `url_bytes`, 1,024 |
  | `jig/testing/jig-protocol-accounts-kit/src/fixtures.rs` | its client's `url_bytes`, 1,024 |
  | `smith/crates/smith-local-shell/src/credentials/product.rs`, `limits` | its client's `url_bytes`, 8,192 |
  | `smith/crates/smith-local-service/src/credentials/tests.rs` | its client's `url_bytes`, 1,024 |
  | `skein/tests/oauth-accounts` (`src/world.rs`, `tests/suite/memory.rs`) | each its client's `url_bytes` |
  | `skein/crates/skein-oauth-accounts/src/tests.rs`, `limits()` | 1,024, its client's `url_bytes` |

  Small's line carries the comment: `// A full answer at the client's bounds, 13,407
  bytes (redirect_room), with room to grow (jig's configuration.md, 6).`

## 2. jig's profiles (task `driver-profile`)

- `small.rs`' `DRIVER.server.head` is `skein_oauth_accounts::listener_head(16_384)`,
  computed in the const (`.expect("a browser's redirect at the bound")`), with a
  comment naming its parts; `headers` stays 32. The `16_384` becomes one named const
  of the module, `REDIRECT: u32`, which `DRIVER.redirect_bytes` and `head` both read.
- `overrides.rs`' `driver_fits` keeps `url_bytes <= ADDRESS` and adds
  `redirect_room(driver)` at most `driver.redirect_bytes` and `listener_head
  (driver.redirect_bytes)` at most `driver.server.head` (a `None` from either is
  false). Its doc comment says so.
- `domain.rs`' `whole` keeps its `driver.url_bytes` problem and adds, in this order
  after it, `What::Bound` at `limits` → `driver.redirect_bytes` when the redirect bound
  is under its room, and at `limits` → `driver.server.head` when the head is under
  `listener_head`.

## 3. The over-long redirect (task `long-redirect`)

In `skein/crates/skein-oauth-accounts`:

```rust
// boundary.rs, Failure, after AnotherSubject:
/// A public sign-in's listener read its own redirect, its state the sign-in's,
/// past the redirect bound: the address's `length` and the `bound`,
/// `redirect_bytes`; the sign-in ends at once (oauth.md, 6.5).
LongRedirect { length: u32, bound: u32 },

// lib.rs
/// The longest application name the listener's long page names (oauth.md, 6.7).
pub const APPLICATION_BYTES: usize = 64;

// component.rs; `check` is unchanged.
/// `application` is the name the listener's long page gives: 1 to
/// `APPLICATION_BYTES` bytes of visible ASCII and spaces, or `Unusable::Limits`
/// (oauth.md, 6.7).
pub fn new(
    accounts: List<Account>,
    limits: &Limits,
    crypto: CryptoRng,
    application: &[u8],
) -> Result<Component, Unusable>;
```

In `skein/crates/skein-oauth/src/machine.rs`, `impl Client`:

```rust
/// Whether a sign-in waits for its redirect, its deadline not reached, and
/// `state` is its own, compared in constant time (oauth.md, 3 and 6.5).
#[must_use]
pub fn awaits_state(&self, state: &[u8], now: Time) -> bool;
```

- **The pages** (`listener.rs`): `PAGE` and `REFUSAL` keep their bytes. The long page
  is `This sign-in's answer is longer than {application} accepts. Go back to
  {application} and start the sign-in again.\n`, ASCII apostrophe, built once when the
  component is made. Its answer is `414`, `content-type: text/plain; charset=utf-8`,
  no `location`, closed as the others are.
- **Which request is over-long** (`redirect.rs`): a `GET` with one Host equal to the
  registered authority, its address (`http://` + Host + target, as `from_head` builds
  it) starting with the bound URI and then its separator (`?`, or `&` where the URI
  holds a `?`), with no `#`, longer than `redirect_bytes`, holding exactly one `state`
  field that decodes within `state_bytes`. Only then does the listener hand it up as
  long (its length, as `u32`, and its state as a `Secret`); anything else past the bound
  is refused as today. The paste path refuses past the bound as `Refusal::Redirect`,
  unchanged.
- **The judgement** (component): a long read whose state `awaits_state` accepts is
  answered with the long page, and in the same step the sign-in fails
  `Event::Failed { account, ends: Ends::SignIn, failure: Failure::LongRedirect { length,
  bound: limits.redirect_bytes } }`; the listener closes after its page is sent. One it
  refuses gets the `REFUSAL` page and the sign-in waits.
- **Readers outside skein** (each only this):
  - `jig/crates/jig-protocol-accounts/src/translate.rs`, `classify`:
    `driver::Failure::LongRedirect { .. } => accounts::Failure::SignInAgain`, in the arm
    with `NotGranted | AnotherSubject | Expired`.
  - `jig/crates/jig-protocol-accounts/src/moves.rs`: `pub fn application(&self) -> &[u8]`
    on `Moves`, and `component.rs`' `opened` passes it to `driver::Component::new`.
  - `smith/crates/smith-local-service/src/credentials.rs`, `configured_driver`: passes
    `b"smith"`, the name smith's registration already sends (`agent_name_hint`).
  - `smith/crates/smith-local-shell/src/credentials/login/provider/driver.rs`,
    `failure_line`: `Failure::LongRedirect { .. }` joins the "not signed in" arm.

## 4. What changes in place

No stored format, client wire or refusal vocabulary changes. Public Rust interfaces
change in place, with no compatibility kept:

- `skein_oauth_accounts::Limits` gains `redirect_bytes`; the crate exports
  `BROWSER_FIELDS`, `redirect_room`, `listener_head` and `APPLICATION_BYTES`.
- `skein_oauth_accounts::Component::new` takes the application's name.
- `skein_oauth_accounts::Failure` gains `LongRedirect { length, bound }`; jig carries it
  as `SignInAgain`, so the client wire and temper's pages are unchanged.
- `skein_oauth::Client` gains `awaits_state`.
- jig's small profile (and standard, through it): the driver's `redirect_bytes` 16,384
  and `server.head` 24,593, from 4,096.

## 5. Synthetic values for the rows

In `skein/crates/skein-oauth-accounts/src/paste_tests.rs` (task `redirect-bound`
writes them; `long-redirect` reuses them):

- `fn synthetic(prefix: &[u8], length: usize) -> Box<[u8]>`: the prefix, then
  `0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ-_` repeated, cut to
  `length` bytes in all. A code is `synthetic(b"ac_", n)`; the issued client id
  `synthetic(b"app_", 28)`; a cookie value `synthetic(b"c_", n)`.
- An issuer's answer, its fields in this order:
  `?code=<code>&scope=openid+profile+email+offline_access&state=<the visit's>&client_id=<id>`.
- **A browser's request** at the listener, one field per line, `\r\n` line ends:

  ```text
  GET <target> HTTP/1.1
  Host: 127.0.0.1:41000
  Connection: keep-alive
  sec-ch-ua: "Chromium";v="142", "Google Chrome";v="142", "Not_A Brand";v="99"
  sec-ch-ua-mobile: ?0
  sec-ch-ua-platform: "Linux"
  Upgrade-Insecure-Requests: 1
  User-Agent: Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/142.0.0.0 Safari/537.36
  Accept: text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,image/apng,*/*;q=0.8,application/signed-exchange;v=b3;q=0.7
  Sec-Fetch-Site: cross-site
  Sec-Fetch-Mode: navigate
  Sec-Fetch-User: ?1
  Sec-Fetch-Dest: document
  Referer: https://issuer.example/
  Accept-Encoding: gzip, deflate, br, zstd
  Accept-Language: en-GB,en-US;q=0.9,en;q=0.8
  Cookie: session=<synthetic(b"c_", 4_000)>
  ```

  then the blank line.
- **A profile that holds them** (`fn answer_profile() -> Limits`): the crate's
  `limits()` with the small profile's client bounds, `uri_bytes`, `code_bytes`,
  `scope_bytes` 4,096, `state_bytes` 64, `url_bytes` 2,048, `issued_client_bytes`
  1,024; `redirect_bytes` 16,384; `server.head` `listener_head(16_384)`,
  `server.headers` 32; and what else the code's exchange needs to carry a 3,900-byte
  code (the HTTP client's `request`, TLS `send`, io), named in the report.
