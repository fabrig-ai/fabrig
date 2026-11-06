+++
id = "link-hours"
part = "link-hours"
kind = "feature"
needs = []
files = ["jig/crates/jig-protocol-config/src/profile/small.rs", "temper/crates/temper-box/src/terminal.rs", "temper/crates/temper-box/src/tests/terminal.rs", "temper/crates/temper-box/src/tests/host.rs", "temper/crates/temper-box/tests/real_loop.rs", "temper/crates/temper/tests/fake_backends/box_link.rs", "temper/crates/temper/tests/fake_backends/walk.rs", "temper/crates/temper/tests/process.rs", "temper/docs/help/box.md", "temper/docs/help/party.md"]
effort = "medium"
minutes = 25
+++

## Goal

A sign-in link lives 2 hours, and the box's message says so.

Acceptance:

- `link_span` in `jig/crates/jig-protocol-config/src/profile/small.rs` is `Duration::from_secs(7200)`. Nothing else in jig changes; fixtures keep their own 300 s.
- `temper-box`'s `FOOTER` is exactly these bytes, and every copy in tests is too:

  ```
  \nIt signs you in as this temper's owner, once, within 2 hours.\nOpened after that, it prints a fresh one here.\n
  ```

- `temper/docs/help/box.md`'s block has the same two lines. `temper/docs/help/party.md`, its *Signed in* line, reads "once, within 2 hours", and the sentence after it stays.
- One test, in `temper/crates/temper/src/tests.rs` or beside it, holds the footer's hours to the profile's: `link_span` of `jig_protocol_config::profile::profile(Profile::Small).clients` in whole hours, as `within {hours} hours`, is found in `temper_box::terminal::FOOTER`. Break it by hand (change the value to 3600) and see it fail for that reason.
- Remove the **Not yet** lines naming `link_span` and the footer: `jig/docs/design/protocol/client.status.md`'s 8, *How long it lives*, and `temper/docs/design/protocol/setup.status.md`'s 7.2.

## Inputs

- `jig/docs/design/protocol/client.md`, section 8, *How long it lives*.
- `temper/docs/design/protocol/setup.md`, 7.2, its block.
- `temper/docs/plan/box-small/notes/common.md`.

## Owns and may change

- Owns the files in the header. This task departs from the implementer's text, in these two ways: `docs/help/box.md` and `docs/help/party.md` are yours, both read by `temper-docs`' tests; and this branch crosses from temper into jig for one value, `link_span`, its only reader outside jig's fixtures being this footer.
- The design's words are already written: `jig/docs/design/protocol/client.md`, `temper/docs/design/protocol/setup.md` and `temper/docs/design/draft/box-link-first.status.md`. `temper/docs/design/client/greenfield/app.js` and the archived drafts keep theirs: `box-link-first.status.md` already says the mock lags.
- If a test reads the old words outside your files (`rg 'short while'` and `rg 'once spent'` over `temper/crates` and `temper/tests`), edit that literal alone and say so.
- If a test on the small profile counts on the link expiring within 300 s and fails, it may wait on the fake clock for `link_span` instead of a number; a test that would take real time is a question.

## Steps

1. The footer's bytes in `temper/crates/temper-box/src/tests/terminal.rs`, failing first; then `FOOTER` and the copies.
2. The value; the test holding hours to the profile.
3. Help words.
4. `FOOTER` keeps within `FIXED_BYTES`. `box_link.rs`'s `FOOTER` and `process.rs`'s lines follow.
5. `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `python3 -I -B jig/scripts/old_forms.py`; `heavy cargo clippy -p jig-protocol-config -p temper-box -p temper --all-targets -- -D warnings`.
6. The value changes what a shared item means, so its readers' tests run: `heavy cargo nextest run -p jig-protocol-config`, `-p temper-box`, `-p temper-docs`, `-p temper-service`, `-p temper`, one command at a time. Citations. Commit and push.

## Report

As the implementer's text asks.

## Answers

2026-11-06 (fabrig/temper). An addition from the user, after the rest of your task: "the sign-in URL printed by temper box should be clickable if the terminal supports it". Wrap the link in OSC 8 when the box's stdout is a terminal and TERM isn't dumb, the first link and every fresh one: ESC ] 8 ; ; URL ESC \\ URL ESC ] 8 ; ; ESC \\ . The visible text stays the whole URL. Piped or sent to a file, stdout stays exactly as it is, the bare URL alone on its line, since scripts and tests read it (temper/docs/design/protocol/setup.md, the box's stdout). The line's bound then has to hold the URL twice plus the markers ("sign-in link too long" in temper/crates/temper-box/src/terminal.rs). An OSC 8 link stays whole across a wrapped line, where autodetection stops at the wrap and cuts the secret. Say so in setup.md, 7.2. If your run can't take this, finish without it and say so: it'll follow.
