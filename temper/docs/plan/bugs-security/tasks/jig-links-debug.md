+++
id = "jig-links-debug"
part = "jig-holders"
kind = "feature"
needs = ["jig-accounts-check"]
files = [
  "jig/testing/jig-test-service/src/clients.rs",
  "jig/testing/jig-test-service/src/tests/clients.rs",
]
effort = "medium"
minutes = 35
+++

# jig-test-service's printed sign-in links print as a count

## Goal

`jig/testing/jig-test-service/src/clients.rs`'s `Clients` (~16) derives `Debug`, and its `links:
Queue<Box<[u8]>>` holds each printed sign-in link, the link's secret in it (`print_link`, ~201),
while the same struct keeps that secret as `secret: Option<Secret>`. So a debug print of
`Clients` shows the secret. Give `Clients` a hand-written `Debug` that prints every field as
derived but `links`, which prints as its count.

Acceptance:

- `Clients` prints as derived, the same fields in the same order, but `links_entries: N` in
  `links`' place; no byte of a link.
- A `Clients` holding a printed link of the canary formats without it in either form
  (`notes/debug.md`), and with `links_entries: 1`.
- The new test fails on the old code and passes on the new.
- clippy clean; the crate's lib tests pass.

## Inputs

- `temper/docs/plan/bugs-security/notes/common.md` and `notes/debug.md`.
- `jig/AGENTS.md`.
- `temper/docs/plan/bugs-security/inputs/request.md`, item 4.
- `skein/docs/design/lib.md`, 6, `Secret`.
- Readers, surveyed: no test or file reads `Clients`' `Debug` text; the `Service` that holds it
  has no `Debug` (`lib.rs` ~53).

## Owns and may change

- `jig/testing/jig-test-service/src/clients.rs`: `Clients`' derive and its impl; nothing else.
- `jig/testing/jig-test-service/src/tests/clients.rs`: the test below, appended, and a fixture
  it needs there.

Nothing else.

Pre-approved options:

- If a `Clients` cannot be built in a test without a whole service, reach it through the
  service fixture the file already uses (`tests/clients.rs` ~198 drives sign-in links), and
  format the `Clients` it holds; if that needs a `pub(crate)` accessor, add one in `clients.rs`
  or `lib.rs` and say so.
- If clippy's `missing_fields_in_debug` fires, a scoped expect citing lib.md, 6.
- Formatting in the test: `alloc::fmt::format(format_args!(..))` or a scoped
  `#[expect(clippy::disallowed_macros, reason = "...")]`.

## Steps

1. The test first, appended to `src/tests/clients.rs` (the crate's lib tests,
   `tests::clients`):

   ```rust
   #[test]
   fn debug_of_the_clients_shows_the_links_count_never_a_link() {
       // a Clients with one printed link holding the canary (by print_link, or pushed onto
       // `links`): formatted, "links_entries: 1" present, the canary absent in both forms
   }
   ```

   Run `heavy cargo nextest run -p jig-test-service --lib -E 'test(/debug_of_the_clients/)'` on
   the old code: it fails. Keep the output.
2. The impl, a line above it saying what it hides.
3. Narrow tests, one at a time:
   - `heavy cargo nextest run -p jig-test-service --lib -E 'test(/debug_of_the_clients/)'`: 1 passed.
   - `heavy cargo nextest run -p jig-test-service --lib`.
4. `cargo fmt`; `python3 -I -B scripts/old_forms.py` from `jig/`;
   `heavy cargo clippy -p jig-test-service --all-targets -- -D warnings`.
5. One commit, the impl and its test, saying why; push.

## Report

As the implementer's text says, and: the failing run on the old code, the passing runs and their
counts, and how the test reached a `Clients`.
