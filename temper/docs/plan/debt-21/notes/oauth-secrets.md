# debt-21: the oauth crates' credentials (skein D009)

Binding on the `oauth-*`, `accounts-secrets` and `fake-oauth-secrets` tasks. Line numbers are at the plan's base.

## The rule

`skein/docs/design/oauth.md`, 4: every type carrying a token, authorization code, verifier, state, nonce, ID token or client secret omits its bytes from `Debug`. `skein/docs/design/lib.md`, 6: a `Secret` prints its length, and a carrier prints lengths. So:

- **An owned credential field is a `skein_lib::Secret`,** public or private, in an item's type, a struct or an enum's variant. That covers tokens, codes, verifiers, `state`, `nonce`, client secrets and a record's bytes.
- **A carrier stays a byte box, with a hand `Debug` on its type printing the length:** a body, a URL or a pasted URI holding credentials among other bytes. The carriers are `oauth::HttpRequest.body`, `oauth::HttpResponse.body`, `oauth::Request::Visit.url`, `oauth::Exchange.visit`, `accounts::Request::Redirected.uri`, `accounts::Event::Visit.url`, `fake::Body::Raw`, `fake::Event::Authorize.url`, fake's `Pending.body`.
- **A public component gets a hand `Debug` that stops at its edge,** so that its private state needs none: `oauth::Client`, `accounts::Component`, `fake::Issuer`. Use `finish_non_exhaustive`, and print a phase or a count only where that type already has one to hand. `missing_debug_implementations` looks at public types alone, so private types stay as they are.
- **A borrow stays a borrow:** a function taking `&[u8]` (`read_claims`, `challenge`, `token_body`, encoders) keeps it, and its caller passes `expose()`. A `Secret` isn't threaded through a helper for its own sake.

## Traps

- **Deriving `Debug` through accounts' `Web` leaks:** its `document: List<u8>` and `Queue<http::Event>` print raw bytes (`Up::Bytes` derives `Debug`, `skein/crates/skein-lib/src/stream.rs`). `Component`'s `Debug` doesn't reach them.
- **`Secret` has no `Deref`, `AsRef` or `From`.** `.as_ref()` and `.as_deref()` stop compiling; `.len()`, `.is_empty()` and `.clone()` keep working. `skein-lib` doesn't change: `Secret::new` builds one, `expose` reads it, `into_exposed` gives the box back, none of them copying. No closures in a step crate: an `Option<&Secret>` to `Option<&[u8]>` is a small `match` helper, as `skein-oauth/src/record.rs` has.
- **A comparison stays what it is:** `sha256::equal(a.expose(), b.expose())` where it was constant time, `==` where it wasn't.
- **Nothing stored or on the wire changes.** `skein-oauth/schema/oauth-v3.schema` already says `secret`, and `src/generated/v3.rs` is already `Secret`. `git diff --stat` shows nothing under `schema/`, `generated/` or `golden/`. `FINGERPRINT`, `suite/drift.rs`, `record_memory` and `fuzzy/record` pass unchanged. If one doesn't, stop and ask.
- **Heap:** a `Secret` is a box's size and adds no copy. The memory tests' ceilings stay (`CLIENT_REFRESH_HEAP_CEILING`); a test that goes over means an added `clone`, which you remove.
- **What crosses to another tree stays that tree's type.** smith's `Ready.value` stays `Box<[u8]>`, with `into_exposed()` at that line; jig's `Secret::new(token)` in `translate.rs` becomes `token`.

## Readers

Compile-forced edits, each only in a field's value, its spelling or a type path (`AGENTS.md`'s "required field"): `Secret::new(..)`, `.expose()`, `.into_exposed()`. A test may have one local `fn secret(bytes: &[u8]) -> Secret`. An assertion's spelling may change (`.as_ref()` to `.expose()`); what it compares doesn't. No other refactor.

The packages that depend on these crates:

- **skein:** `skein-oauth-accounts`, `skein-fake-oauth`, `skein-fake-peers`, `skein-fake-peers-tests`, `skein-oauth-world`, `skein-oauth-accounts-world`, `skein-oauth-issuer-world`
- **jig:** `jig-protocol-accounts`, `jig-protocol-accounts-world`, `jig-protocol-config`, `jig-protocol-engine-clients`, `jig-protocol-store`, `jig-store-codecs-world`, `jig-engine-clients-world`, `jig-test`, `jig-test-service`, `jig-one-process-world`
- **smith:** `smith`, `smith-bench`, `smith-local-protocol`, `smith-local-service`, `smith-local-shell`, `smith-local-process-world`
- **temper:** `temper`, `temper-service`, `temper-fake`, `temper-client-browser-world`, `temper-real-loop-world`

Each task gets its readers to compile with `heavy cargo check -p <package> --tests`, naming them, and runs the tests its spec lists. The meaning of no value changes, so the other trees' tests are the trunk's end check.

Stay off `skein/docs/development/debt/D022-oauth-states.md`'s work: `Phase`, `Exchange`'s shape and `AccountEntry` keep their shapes here, and only their credential fields change type.
