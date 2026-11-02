+++
id = "refresh-facts"
part = "refresh"
kind = "feature"
needs = []
files = ["smith/crates/smith-local-shell/src/credentials/sign_in.rs", "smith/crates/smith-local-shell/src/credentials.rs", "smith/tests/end-to-end/support/live.rs", "smith/tests/end-to-end/live_configuration.rs", "smith/tests/end-to-end/live.rs"]
effort = "high"
minutes = 75
+++
# The refresh story reports its record's refresh facts

## Goal

`smith::live a_run_refreshes_its_grant_at_the_issuer` has failed at every after-landing run since
11-02 10:49 with "the issuer refresh was kept; generation before=N after=N", while N itself rose
about once an hour between runs (3 at 11:51, 4 at 13:02, 5 at 14:12, 6 at 15:28, 7 at 16:30):
the record is refreshed, but not by the story's run, whose lead is 400 days. Read offline, the
product refreshes at once on that lead unless the record holds an earliest refresh time still
ahead (skein/docs/design/oauth.md, 2: "no refresh is asked before it"), or holds no refresh
token; which one the live record holds the report cannot say, and nobody may read the record.
`refresh-twin` pins the product path offline; this task makes the live story say which.

smith/docs/design/testing.md, 2.3 now says: "The refresh story also reports the record's refresh
facts before and after its run, read through smith's own record reader: whether it holds a
refresh token, and its access token's expiry and its earliest refresh time, each in whole seconds
from the story's clock, or none."

Acceptance:

- `smith_local_shell::credentials` exports a reader of a saved record's refresh facts that never
  returns a token's bytes, unit-tested in `sign_in.rs`.
- `support/live.rs` reads them for the story's ChatGPT record and formats them in one line of
  fixed words and numbers; `live_configuration.rs` checks the line offline on a synthetic record,
  and that it holds none of the record's token bytes.
- The refresh story's failure message is that line for before and after its run, then the run's
  report, as today.
- fmt, clippy and the focused tests below are green.

## Inputs

- `smith/docs/plan/live-smith-reds-2/notes/common.md` (read it first). **Departure from it:**
  this task edits `live.rs` while `claude-tools` and `crossing-paths` do: you change only the body
  of `a_run_refreshes_its_grant_at_the_issuer`, which neither touches.
- `smith/docs/design/testing.md`, 2.3 ("A failure says why, in typed facts", the refresh story)
  and 8 ("The refresh story and an earliest refresh time").
- `smith/docs/design/shell.md`, 7.3 (records across processes).
- `skein/docs/design/oauth.md`, 2 (the record's earliest refresh time).

## Owns and may change

- `smith/crates/smith-local-shell/src/credentials/sign_in.rs`: the reader and its unit test, beside
  `record`. Not `credentials/product.rs` (plan `sign-in-wait`'s; `record_generation` stays there).
- `smith/crates/smith-local-shell/src/credentials.rs`: the re-export, in the `pub use sign_in::{...}`
  line.
- `smith/tests/end-to-end/support/live.rs`: `Environment::refresh_facts` and the line's formatter,
  beside `generation` (which stays).
- `smith/tests/end-to-end/live_configuration.rs`: one new test.
- `smith/tests/end-to-end/live.rs`: the body of `a_run_refreshes_its_grant_at_the_issuer` only.
- Pre-approved:
  - Names and shape, sketched:

    ```rust
    // smith-local-shell, credentials/sign_in.rs
    /// A saved sign-in record's refresh facts, without its tokens (shell.md, 7.3).
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub struct RefreshFacts {
        pub generation: u64,
        /// Whether the record holds a refresh token.
        pub refreshable: bool,
        /// When its access token expires.
        pub expires_at: skein_lib::Wall,
        /// The issuer's earliest refresh time, when it set one.
        pub earliest_refresh_at: Option<skein_lib::Wall>,
    }

    /// Read a saved record's refresh facts; `None` when there is no record.
    pub fn refresh_facts(file: &std::path::Path) -> Result<Option<RefreshFacts>, String>;

    // support/live.rs
    impl Environment {
        /// The ChatGPT record's refresh facts, through smith's own reader.
        pub fn refresh_facts(&self) -> smith_local_shell::credentials::RefreshFacts;
    }

    /// One line of the refresh facts before and after a run, in whole seconds from `now`.
    pub fn refresh_report(
        before: &smith_local_shell::credentials::RefreshFacts,
        after: &smith_local_shell::credentials::RefreshFacts,
        now: skein_lib::Wall,
    ) -> String;
    ```

    e.g. `record before: generation=7 refreshable=true expires_in=3127s earliest_in=none; after:
    generation=7 refreshable=true expires_in=2911s earliest_in=1840s`, a time already past as a
    negative number of seconds. The error string of `refresh_facts` is `RecordFailure::line`'s,
    as `record_generation`'s is.
  - The story's message: `the issuer refresh was kept; {refresh_report}\n{completed}`, reading the
    facts before the exec and after it, with the wall time from `skein_shell::Clock`.
  - If `RecordFailure` or `record` cannot serve the reader without a change to their shape, add a
    private helper beside them and say so.

## Steps

1. Read the inputs, `sign_in.rs` (`record`, `RecordFailure`, its `tests` module),
   `product.rs`'s `record_generation`, `skein-oauth`'s `SavedToken` (`refresh_token`,
   `expires_at`, `earliest_refresh_at`), `support/live.rs` and the story.
2. `sign_in.rs`, test first: `refresh_facts_read_no_token`: a synthetic record encoded with
   `skein_oauth::encode_record` and kept 0600 in a scratch directory (as the crate's tests keep
   records; find one with `rg keep_record\|encode_record smith/crates/smith-local-shell`), with a
   refresh token and an earliest refresh time, then one without either; the facts match; an
   absent file is `Ok(None)`; a malformed file is an error. Run red, implement, run green:
   `heavy cargo nextest run -p smith-local-shell --lib -E 'test(/refresh_facts/)'`.
3. `support/live.rs`: `Environment::refresh_facts`, `refresh_report`.
4. `live_configuration.rs`: `the_refresh_report_names_facts_without_token_bytes`: write a synthetic
   record (distinctive access and refresh token bytes, generation 7, expiry and earliest refresh
   time set) under a scratch directory, read it with `smith_local_shell::credentials::refresh_facts`,
   format `refresh_report` for it before and after (generation 8), and assert the line's numbers
   and that it contains neither token's bytes.
   `heavy cargo nextest run -p smith --test live_configuration -E 'test(/refresh/)'`.
5. `live.rs`: the story's body per "Pre-approved"; `heavy cargo check -p smith --tests`; never run
   the `live` binary.
6. `cargo fmt`; `heavy cargo clippy -p smith-local-shell -p smith --all-targets -- -D warnings`.
7. `heavy cargo nextest run -p smith-local-shell`, then
   `heavy cargo nextest run -p smith --test live_configuration`.
8. Commit (the reader, then the story), push.

## Report

The red run of step 2, the files changed, the line's final form, the test runs (selected,
passed), and implementation and check minutes apart.
