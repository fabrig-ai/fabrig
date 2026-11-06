+++
id = "browser-story"
part = "tab-browser"
kind = "feature"
needs = ["sign-in-holds", "moves-on", "view-tab", "kit-tabs", "kit-behind-tab"]
files = ["temper/tests/client-browser/tests/browser_all/browser_accounts.rs", "temper/tests/client-browser/src/lib.rs"]
effort = "high"
minutes = 50
+++

## Goal

Chromium runs a sign-in in a new tab against skein's fake issuer: the press opens the tab within its gesture, the answer points it, the provider's redirect reaches the engine's listener there, and temper's page, which stayed, moves on.

Acceptance, as `temper/docs/design/client/browser.md`, 4.2, *`ConnectModel`'s sign-in, in a new tab* has it:

- `the_sign_in_leaves_the_page_and_comes_back_to_ready` becomes `the_sign_in_opens_a_tab_and_the_page_moves_on_to_ready`. Its `Step::Tabs` holds one tab, whose address holds the listener's redirect path (`callback`) and whose `opener` is false. Its `Step::Address` holds the source, in a history entry that replaced Choose's. It sees "Signed in to Test provider", and a reload shows it again, with one `Authorize` and one `Post` observed.
- A story on the welcome's Model step: the same press, and the page moves on to `/welcome?at=ready` by itself.
- Remove `temper/docs/design/client/browser.status.md`'s **Not yet** line for 2.1, 2.3 and 4.2. This task may make that one edit, against the implementer's text.

## Inputs

- `temper/docs/plan/sign-in-new-tab/notes/common.md`.
- `temper/docs/design/client/browser.md`, 4.2 and 4.3.
- `skein/docs/design/ui-web.md`, 14.2, *Its tabs*; 14.3, *A tab*.
- `temper/docs/design/client/README.md`, 3.5, `ConnectModel`, *The sign-in's tab* and *Its end*.

## Owns and may change

- Owns the files in the header. May add a helper or a `RoundTrip` change in `browser_all/browser_accounts.rs`, and rename `RoundTrip` if the name no longer holds.
- The tier's budget is the user's: `--profile browser-temper`, 120 s. If the welcome's story takes it past that, ask the questions the implementer's text has for over budget, and report; don't move it out of the tier.
- If the new tab's listener page raises trouble in the harness, which attaches to no tab of the page's: stop and ask.

## Steps

1. Build the tier's bundle, then run with `temper/scripts/browser.sh`'s steps, one `heavy` command at a time: `heavy cargo nextest run --profile browser-temper -p temper-client-browser-world -E 'test(=browser_accounts::NAME)'`, taking the binary and the test's name from `--list`, and reading the count. If the builder has no Chromium, say so plainly, and don't count it as run.
2. Break the story once: ask `Step::Tabs` for two tabs, see it fail naming what was there, and restore it.
3. `cargo fmt`; `heavy cargo clippy -p temper-client-browser-world --all-targets -- -D warnings`. Commit and push.

## Report

As the implementer's text asks. Say plainly what ran on Chromium, its version, and that Firefox and Safari were read and not run. Give the tier's wall time.

## Answers

2026-11-06. Should a separate skein task take the Chromium queries that fail once the sign-in tab has opened, or this task? A separate one, `kit-behind-tab`, which this task now needs. `kit-tabs` and `web-tab-story` met the same fault and went round it, so it is skein's, reproduced in skein (`AGENTS.md`, 5), and this task's acceptance can't hold without it: a story must see "Signed in to Test provider" on the page that stayed.

Take the partial stories on from commit `6eae35ee87` on `sign-in-new-tab/browser-story`, changing no expectation for them. What stays outstanding is the task's: the standalone story passing, the welcome's story run, the two-tab break seen failing, the tier's wall time, and the status line. If they still fail with `kit-behind-tab` landed, stop and ask again, with the log.
