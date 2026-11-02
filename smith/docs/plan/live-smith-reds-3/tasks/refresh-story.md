+++
id = "refresh-story"
part = "refresh-story"
kind = "feature"
needs = []
files = ["smith/tests/end-to-end/live.rs", "smith/tests/end-to-end/support/live.rs", "smith/tests/end-to-end/live_configuration.rs"]
effort = "high"
minutes = 50
+++
# The refresh story checks the issuer's rule as its record stands

## Goal

`smith::live a_run_refreshes_its_grant_at_the_issuer` fails at the after-landing pair of main
df41529d3 (hetzner8), on both runs, at `smith/tests/end-to-end/live.rs:47`, after an accepted
run:

> the issuer refresh was kept; record before: generation=9 refreshable=true expires_in=2800s
> earliest_in=2438s; after: generation=9 refreshable=true expires_in=2800s earliest_in=2438s

The cause is the story, not the product: OpenAI's issuer sets an earliest refresh time on every
token (here about six minutes before expiry), and no refresh is asked before it, whatever the lead
(`skein/docs/design/oauth.md`, 2 and 6.3). Pairs run minutes apart and a ChatGPT run refreshes
the record once it expires, so the pair's record is usually fresh. The product path is already
pinned offline and green: `smith/tests/end-to-end/credentials.rs`,
`an_exec_before_the_earliest_refresh_time_keeps_its_record` (a raised lead, an earliest refresh
time ahead: no refresh, generation kept) beside `an_exec_on_a_raised_lead_refreshes_its_record_at_once`.

The design now says what the story checks (`smith/docs/design/testing.md`, 2.3, the story "a run
refreshes its grant when the issuer allows"): it reads the record's refresh facts before its run
and checks the rule as the record stands. With no earliest refresh time, or one past, the run
refreshes at once and the generation rose; with one more than five minutes ahead (longer than the
story's check and exec bounds together), the run completes on the held token and the generation
is unchanged; with one nearer, the story waits until it has passed, then expects the refresh; a
record without a refresh token fails by name. Its failure message names the case it checked.

Acceptance:

- `support/live.rs` chooses the story's case from a record's refresh facts and the story's wall
  time (sketch below); `live_configuration.rs` pins every case and the margin offline.
- The story is renamed `a_run_refreshes_its_grant_when_the_issuer_allows` and checks its case;
  `heavy cargo check -p smith --tests` is green. It is never run here: the after-landing pair runs
  it on main, and its green there is this plan's acceptance, not yours.
- `an_exec_before_the_earliest_refresh_time_keeps_its_record` is green on your base (the offline
  reproduction of the live result), run once before your change.
- fmt, clippy and the focused tests below are green.

## Inputs

- `smith/docs/plan/live-smith-reds-3/notes/common.md` (read it first).
- `smith/docs/design/testing.md`, 2.3: the story "a run refreshes its grant when the issuer
  allows", and "A failure says why, in typed facts" (the refresh story's facts and its case).
- `skein/docs/design/oauth.md`, 2 (the earliest refresh time) and 6.3 ("Refreshed while held",
  "Never back to back").
- `smith/docs/design/shell.md`, 6.5 (`refresh_lead`).

## Owns and may change

- `smith/tests/end-to-end/support/live.rs`: the new constants, `RefreshCase` and `refresh_case`,
  beside `REFRESH_LEAD_SECONDS` and `refresh_report` (both stay as they are).
- `smith/tests/end-to-end/live.rs`: the refresh story (its name, its `Leg`'s `story` and its
  body), and in `run` only the two bound literals `30` and `260`, which become the constants
  below. Nothing else.
- `smith/tests/end-to-end/live_configuration.rs`: one new test.
- Pre-approved:
  - Names and shapes, sketched:

    ```rust
    // support/live.rs
    /// The live check's hard bound, in seconds.
    pub const CHECK_SECONDS: u64 = 30;
    /// The live exec's hard bound, in seconds.
    pub const EXEC_SECONDS: u64 = 260;
    /// How far ahead an earliest refresh time must be for the refresh story to expect its record
    /// kept: longer than its check and exec bounds together (testing.md, 2.3).
    pub const EARLIEST_MARGIN_SECONDS: u64 = 300;
    pub const NOT_REFRESHABLE: &str = "the record in `$SMITH_LIVE/tokens/chatgpt` holds no refresh token: run `XDG_CONFIG_HOME=$SMITH_LIVE/config smith login chatgpt`";

    /// The refresh story's case, from its record's refresh facts (testing.md, 2.3).
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub enum RefreshCase {
        /// No earliest refresh time, or one at or before now: the run refreshes at once.
        Refreshed,
        /// An earliest refresh time more than the margin ahead: the run keeps the record.
        Kept,
        /// An earliest refresh time ahead within the margin: wait this many whole seconds,
        /// which pass it, then choose again.
        Wait { seconds: u64 },
    }

    /// The story's case for `facts` at `now`; a record without a refresh token is
    /// `Err(NOT_REFRESHABLE)`, whatever its earliest refresh time.
    pub fn refresh_case(
        facts: &smith_local_shell::credentials::RefreshFacts,
        now: skein_lib::Wall,
    ) -> Result<RefreshCase, &'static str>;
    ```

    `Wait`'s seconds are the time to the earliest refresh time rounded up to whole seconds, plus
    one: 300 s ahead waits 301, 120.5 s ahead waits 122. Exactly the margin ahead is `Wait`;
    a nanosecond more is `Kept`.
  - The story, sketched (`live.rs`):

    ```rust
    #[test]
    fn a_run_refreshes_its_grant_when_the_issuer_allows() {
        // load chatgpt, raise the lead to REFRESH_LEAD_SECONDS, as today
        // before = refresh_facts(); case = refresh_case(&before, clock.now().wall), panicking
        //   with its message on Err
        // on Wait { seconds }: std::thread::sleep(seconds), read `before` again, choose again;
        //   the case must now be Refreshed, else panic with "case=wait: ..." and refresh_report
        // change(...) as today, then after = refresh_facts(), report = refresh_report(...)
        // Refreshed: assert after.generation > before.generation,
        //   "case=refreshed: the issuer refresh was kept; {report}\n{completed}"
        //   (after a wait: "case=refreshed waited={seconds}s: ...")
        // Kept: assert_eq after.generation, before.generation,
        //   "case=kept: no refresh before the issuer's earliest refresh time; {report}\n{completed}"
    }
    ```
  - If a lint objects to `std::thread::sleep` in the story, add a scoped
    `#[expect(..., reason = "the live story waits for the issuer's earliest refresh time")]` and
    say so.
  - If `refresh_case` needs the signed seconds arithmetic `refresh_report` uses, share a private
    helper in `support/live.rs`; `refresh_report`'s output stays byte for byte.

## Steps

1. Read the inputs, the story, `run` and `change` in `live.rs`, `support/live.rs`
   (`Environment::refresh_facts`, `refresh_report`) and `live_configuration.rs`
   (`the_refresh_report_names_facts_without_token_bytes`).
2. Run the offline reproduction once, before any change, and note its result:
   `heavy cargo nextest run -p smith --features end-to-end --test end_to_end -E 'test(/earliest_refresh/)'`
   (one test, green). If it is red, stop: commit nothing and ask with its log's assertion.
3. `live_configuration.rs`, test first: `the_refresh_story_chooses_its_case_from_the_record`:
   synthetic `RefreshFacts` (the struct's fields are public; no record file) at a fixed `now`:
   refreshable with no earliest refresh time, one a second past and one at `now`: `Refreshed`;
   2336 s and 301 s ahead: `Kept`; 300 s ahead: `Wait { seconds: 301 }`; 120.5 s ahead:
   `Wait { seconds: 122 }`; not refreshable, with none and with one 2336 s ahead:
   `Err(live::NOT_REFRESHABLE)`; and `EARLIEST_MARGIN_SECONDS > CHECK_SECONDS + EXEC_SECONDS`.
   Run it red (it does not compile, or fails), implement `support/live.rs`, run it green:
   `heavy cargo nextest run -p smith --test live_configuration -E 'test(/refresh/)'`
   (this test and the two existing refresh tests).
4. `live.rs`: the two bound literals in `run`, then the story per "Pre-approved";
   `heavy cargo check -p smith --tests`. Never run the `live` binary.
5. `cargo fmt`; `heavy cargo clippy -p smith --all-targets -- -D warnings`, then
   `heavy cargo clippy -p smith --features end-to-end --all-targets -- -D warnings` (the
   `support/` module is compiled into the feature's binaries too).
6. `heavy cargo nextest run -p smith --test live_configuration`.
7. Commit (the case and its test, then the story), push.

## Report

Step 2's result, step 3's red, the files changed, the story's final messages, the test runs
(selected, passed), and implementation and check minutes apart.
