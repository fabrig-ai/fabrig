+++
id = "say-what-failed"
part = "browser-says"
kind = "interface"
needs = []
files = [
  "skein/testing/skein-browser/src/boundary.rs",
  "skein/testing/skein-browser/src/browser.rs",
  "skein/testing/skein-browser/src/browser/**",
  "skein/testing/skein-fake-person-browser/src/face.rs",
  "skein/testing/skein-fake-person-browser/src/harness.rs",
  "skein/testing/skein-fake-person-browser/src/harness/**",
  "skein/testing/skein-fake-person-browser/src/tests/**",
  "skein/tests/browser/**",
  "skein/docs/design/browser.status.md",
]
effort = "high"
minutes = 45
+++

# A browser failure says what failed

## Goal

The browser tier's cookie and Sign out test fails in two ways nobody can read: `unexpected Log: Failed to load resource: the server responded with a status of 503` names no URL and always says step 0, and `browser refused act: Protocol` doesn't say what Chromium answered. Make both say. This task changes what a failure says and nothing about what passes: the next task uses it to catch the cause.

Acceptance:

1. An unexpected log entry's failure reads `unexpected Log: <text> (<url>)`, and `unexpected Log: <text>` where the entry names no URL.
2. A `Setup`'s or a script's expected words are read in the text alone: an entry whose URL holds `401` and whose text doesn't still fails a test expecting `401`.
3. A failure on an unexpected `Trouble` names the step its person is at, where it now always says 0.
4. A `Protocol` refusal's failure reads `browser refused act: Protocol (DOM.getContentQuads: Could not compute content quads.)`. Every other refusal's reads as it does now (`browser refused act: Gone`). A refused read and a refused keep carry it the same way.
5. Which errors the kit refuses with which `Refusal` doesn't change here, and neither does any wait.

## Inputs

- `skein/docs/design/browser.md`, 5 (`Refused`'s `detail`, `Trouble`'s `url`, and the rule "A refusal says what refused it"), 7 (the `url`), 10 (the harness).
- `temper/docs/plan/ui-browser-red/notes/common.md`.
- `skein/testing/skein-browser/src/browser.rs`: `command_error` (the generic `PendingKind::Step` path) and the `finish_refused(token, Refusal::Protocol)` sites in `step_reply` and `browser/attribute.rs`; the `Log.entryAdded` arm of `notification`.
- `skein/testing/skein-fake-person-browser/src/harness.rs`: `event`'s `Event::Trouble` arm, `fail`, `step_failure`.

## Owns and may change

Owns the `files` above.

Readers, for the new fields only:

- `Event::Refused { op, why }` and `Event::Trouble { page, trouble, text }` patterns and literals that the compiler stops: `skein/testing/skein-fake-person-browser/src/face.rs`, `harness.rs`, `harness/failure_tests.rs`, `harness/script/tests.rs`; `skein/tests/browser/src/tests.rs`, `src/referee.rs`, `tests/support/real.rs`, `tests/support/real/dialogs.rs`, `tests/support/real/downloads.rs`; `skein/testing/skein-browser/src/browser/*_tests.rs`. A pattern takes `..` where the test doesn't read the field.
- `temper/tools/journeys/src/again.rs`, `skein/tests/ui-web/tests/**`, `temper/tests/client-browser/tests/browser_all/**`, if the compiler stops them: the same one-line edit.

Options, each decided:

- Where the failing method comes from: if `Pending` can hold the `&'static [u8]` method its command was issued with, do that; if a site can't reach it, name the `Step` there and say so in the report.
- If `event` can't reach a person's step directly, pass what it needs in or let the loop that holds `people` fill the step; the first failure still wins, as `fail` has it.
- A failure text reader outside this list that pinned the old words: update it to the new words and name it in the report.

Remove this task's `Not yet` entries from `skein/docs/design/browser.status.md`, leaving `No gap.`, if nothing else stands there by then. The design text is written; if the code teaches otherwise, say so in the report.

Don't change `temper/tests/client-browser/tests/browser_all/browser_sign_in.rs` or its `expected` list, or any product crate: the next task owns those.

## Steps

### boundary.rs and browser.rs

```rust
pub enum Event {
    Refused { op: Token, why: Refusal, detail: Box<[u8]> },
    Trouble { page: Token, trouble: Trouble, text: Box<[u8]>, url: Box<[u8]> },
    // the rest unchanged
}
```

- `Refusal` stays `Copy` and unchanged; `Face::refusal` keeps returning it.
- `finish_refused(op, why)` keeps its signature and sends an empty `detail`, so that the non-`Protocol` sites don't move. A new `finish_protocol(op, detail: Box<[u8]>)` serves every `Refusal::Protocol` site:
  - `command_error`: `<method>: <message>`, the CDP method and Chromium's `error.message`;
  - a reply without its field in `step_reply`, `mouse` and `attribute.rs`: `<method>: malformed reply`;
  - the kit's own refusals of a request (`browser.rs` `Request::Key`, a letter not lowercase; `issue_character`): their own words.
- Both cut to `Limits::text`, as `trouble` does.
- `Log.entryAdded`: `url` is `entry.url`, empty if absent. Every other trouble's is empty. `trouble`'s signature takes it.

### the face and the harness

- `face.rs`: the act's and the keep's failure text appends ` (<detail>)` when `detail` isn't empty, lossily decoded as the other texts are.
- `harness.rs`, the `Event::Trouble` arm: a small pure function for the words, so that they can be tested without a browser:

  ```rust
  fn trouble_report(trouble: Trouble, text: &[u8], url: &[u8]) -> String
  ```

  The expected words go on being matched against `text` alone; say so in a test by pulling the match out as `fn expected_trouble(expected: &[Box<[u8]>], text: &[u8]) -> bool` if that's the plainest way. The step is the person's `play.at()`.

### Tests

In `skein/tests/browser/src/tests.rs`, the kit's world, which replays CDP frames with no browser:

- `trouble_is_reported_without_judgment`: the `Log.entryAdded` message carries a `url`, and the event's `url` holds it; the exception's and the console's are empty.
- A new `a_log_entrys_url_is_cut_to_the_text_limit`, beside the test near line 444 that cuts text.
- A new `a_protocol_refusal_names_its_method_and_error`: a press whose `DOM.getContentQuads` answers an error with a message the kit doesn't recognise is `Refused` with `Protocol` and `detail` `DOM.getContentQuads: <message>`.
- A new `a_malformed_reply_is_a_protocol_refusal_that_says_so`: `Page.getNavigationHistory` answered without `entries`.
- A new `other_refusals_carry_no_detail`: an existing `Gone` or `Covered` path's `detail` is empty.

In `skein/testing/skein-fake-person-browser/src/harness/failure_tests.rs`:

- `a_failed_keep_reports_its_own_step`'s `Protocol` row carries a detail and reads it.
- A new test for `trouble_report` with and without a URL, and for `expected_trouble`: `401` in the URL alone is not expected.
- A new test that a refused act's failure carries the detail, and that one with an empty detail reads as it does now.

Each new test bites: break its rule once, see it fail, restore, and name it in the report.

### Checks

`cargo fmt`; `python3 -I -B skein/scripts/old_forms.py`; `heavy cargo clippy -p skein-browser -p skein-fake-person-browser -p skein-browser-world --all-targets -- -D warnings`; `heavy cargo nextest run -p skein-browser -p skein-fake-person-browser -p skein-browser-world`. Direct dependents, by check: `heavy cargo check --tests -p skein-ui-web-tests -p jig-client-fake-person-browser -p jig-client-browser-world -p temper-client-journeys -p temper-client-browser-world`. `python3 -I -B tools/citations/citations.py --base origin/main`.

Commit and push. Keep the commits small: the kit's fields, then the harness's words.

## Report

Per the implementer's text, and the exact failure text each acceptance item now gives, as a test or a run shows it. If `Log.entryAdded` turned out not to carry a URL for a failed `fetch` in any real run you saw, say so: the next task relies on it.
