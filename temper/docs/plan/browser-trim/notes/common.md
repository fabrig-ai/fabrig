# browser-trim: what every task holds

The request is `temper/docs/plan/browser-trim/inputs/request.md`; its section *For every task of this plan* holds for every task here.

- **Never raised:** a budget, a timeout or a limit: `tools/tiers/budgets.toml`, `.config/nextest.toml`'s `browser-temper`, the harness's run deadline, a step's `within`. No `#[ignore]`, no retries. Moving a test to escape a budget is raising it (`skein/docs/foundation/testing-strategy.md`, 8.1).
- **Kept whole:** every cookie, sign-out, attack and focus assertion.
- **Run one test of the tier:** `heavy python3 -I -B tools/tiers/run temper browser -- -E 'test(=NAME)'`, whose prerequisites build the bundles. Once they're built, `heavy cargo nextest run --profile browser-temper -p temper-client-browser-world --test browser_all -E 'test(=NAME)'` with `TEMPER_TEST_BUNDLE` and `TEMPER_TEST_PLAYGROUND` set as `tools/tiers/design.md`, section 1 has them.
- **Measure the tier:** `heavy python3 -I -B tools/tiers/run --cpu DIR temper browser`, with `DIR` fresh, private and untracked, then `tools/tiers/cpu-summary DIR --budgets tools/tiers/budgets.toml --pair browser-temper` (`tools/tiers/design.md`, 7).
- **Beside this plan:** `chat-reading-c/reading-browser` adds stories beside `latest_returns_a_chat_to_its_end_after_new_words_while_reading_back` in `browser_pages.rs`, and `sign-in-new-tab/browser-story` rewrites `the_sign_in_leaves_the_page_and_comes_back_to_ready` and owns `temper/tests/client-browser/src/lib.rs`. Leave those two tests alone. `ui-walk-3` changes these pages: a catch-up is expected, and whatever trunk you start from is the one you measure.
- **Throwaway instrumentation** (a timing `Play`, a CDP script, `perf` data) stays untracked or uncommitted and is gone before the push.
