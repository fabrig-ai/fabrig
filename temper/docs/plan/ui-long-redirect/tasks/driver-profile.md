+++
id = "driver-profile"
part = "profile"
kind = "feature"
needs = ["redirect-bound"]
files = ["jig/crates/jig-protocol-config/src/profile/small.rs", "jig/crates/jig-protocol-config/src/overrides.rs", "jig/crates/jig-protocol-config/src/domain.rs", "jig/crates/jig-protocol-config/src/tests/providers.rs", "jig/crates/jig-protocol-config/src/tests/profile.rs", "jig/docs/design/protocol/configuration.status.md"]
effort = "high"
minutes = 60
+++

# ui-long-redirect: jig's profiles size the driver's redirect and its listener's head

## Goal

The small profile's accounts driver holds a browser's request for a returning redirect
at its bound: its `server.head` is 4,096, which a redirect near 16 KiB with a browser's
fields and cookies for `127.0.0.1` cannot fit. Size it from skein's parts
(`listener_head`), and refuse a driver whose redirect bound or head holds less than its
parts, in `driver_fits` and in the whole check, at the driver's member.

Acceptance:

- `small.rs` as `contracts.md`, section 2 gives it: `REDIRECT: u32 = 16_384`, read by
  `DRIVER.redirect_bytes` (its comment from `redirect-bound` kept) and by
  `DRIVER.server.head`, which is `skein_oauth_accounts::listener_head(REDIRECT)` in the
  const, commented with its parts (the request line at the bound, `BROWSER_FIELDS` for a
  browser's fields and its cookies for the loopback host, which are not port-scoped).
  Standard inherits both through `..small::DRIVER`.
- `overrides.rs`' `driver_fits` and `domain.rs`' `whole` check both bounds as
  `contracts.md`, section 2 gives them; `driver_fits`' doc comment names them.
- Rows, below, pass and bite. Every other test of `jig-protocol-config` passes as before.
- The readers of the profile's driver pass their focused tests: `jig-protocol-accounts`,
  `temper-service`, `temper-fake` (a change of the profile's meaning runs its readers'
  tests).
- Delete from `jig/docs/design/protocol/configuration.status.md` the bullet "**Not yet:**
  6: the driver's returning redirect and its listener's head: ..." and nothing else (a
  stated departure from the implementer's text, which closes `docs/design/`).

## Inputs

- `temper/docs/plan/ui-long-redirect/notes/common.md`: the standing rules.
- `temper/docs/plan/ui-long-redirect/notes/contracts.md`, sections 1, 2 and 4.
- `jig/docs/design/protocol/configuration.md`, 6 (*LLM accounts' bounds*: the paragraph
  on the driver's returning redirect, its listener's head and `ADDRESS`).
- `skein/docs/design/oauth.md`, 6.7 (*The redirect bound holds a full answer*; *The
  listener's head holds a browser's request*): what `redirect_room` and `listener_head`
  count.

## Owns and may change

- Owns: `jig-protocol-config`'s `profile/small.rs` (the driver's `server.head` and the
  `REDIRECT` const; nothing else of the profile), `overrides.rs` (`driver_fits`),
  `domain.rs` (`whole`), and its tests `tests/providers.rs` and `tests/profile.rs`.
- Options, pre-approved:
  - `driver_fits` becomes `pub(crate)` so the rows can call it; say so.
  - If `largest_read` or another skein check of the driver's server refuses the larger
    head against small's io limits, raise nothing: it is a `Question:` with the figures.
  - If a reader's test (temper's fake issuers copy `profile.driver.server`) moves with
    the larger head, by a buffer or a memory figure, repair its oracle to the new head,
    keeping what it checks, and name it.

## Steps

1. **The failing rows first**, in `tests/providers.rs`: turn
   `a_profile_whose_driver_could_make_a_longer_account_visit_than_the_address_bound_is_refused`
   into a table test of the driver's bounds, each row one mutation of
   `fixtures::engine(fixtures::ENGINE_SMALL)`'s `limits.driver`, alone:
   - `url_bytes` = `ADDRESS + 1` → `What::Bound` at `limits` → `driver.url_bytes` (today's
     row);
   - `redirect_bytes` = `redirect_room(&driver) - 1` → `What::Bound` at `limits` →
     `driver.redirect_bytes`;
   - `server.head` = `listener_head(driver.redirect_bytes) - 1` → `What::Bound` at
     `limits` → `driver.server.head`;
   and for each, `driver_fits(&engine.limits.driver)` is false; for the unmutated
   engine, no driver problem and `driver_fits` true. Run them: the last two rows fail
   today (no problem; `driver_fits` true). In `tests/profile.rs`'
   `supplied_profiles_have_valid_domain_limits`, for both profiles: `redirect_bytes`
   16,384, at least `redirect_room` (13,407 in small); `server.head` 24,593, at least
   `listener_head(redirect_bytes)`. Quote the failures seen.
2. Size `small.rs`' head; add both checks to `driver_fits` and `whole`.
3. Make each row bite: drop each new check in turn, see its row fail, restore.
4. Delete the status bullet. `cargo fmt`; `python3 -I -B jig/scripts/old_forms.py`;
   `heavy cargo clippy -p jig-protocol-config --all-targets -- -D warnings`;
   `heavy cargo nextest run -p jig-protocol-config -p jig-protocol-accounts`; then
   `heavy cargo nextest run -p temper-service -p temper-fake`. Commit, push.

## Report

The implementer's report, with: the failures seen before the change; the head's const
form; any reader's oracle repaired.
