+++
id = "tab-surface"
part = "tab-root"
kind = "interface"
needs = ["web-tab"]
files = ["temper/crates/temper-client-domain/src/boundary.rs", "temper/crates/temper-client-domain/src/domain.rs", "temper/crates/temper-client-domain/src/route.rs", "temper/crates/temper-client-domain/src/lib.rs", "temper/crates/temper-client-view/src/root.rs", "temper/crates/temper-client-view/src/words.rs", "temper/crates/temper-client-shell-web/src/host.rs", "temper/crates/temper-client-shell-web/src/lib.rs", "temper/crates/temper-client-shell-web/src/journey/line.rs", "temper/crates/temper-client-shell-web/src/journey/record.rs", "temper/crates/temper-client-shell-web/src/journey/replay.rs", "temper/crates/temper-client-shell-web/src/journey/tests.rs"]
effort = "high"
minutes = 45
+++

## Goal

The root can ask its shell for a sign-in's tab, the host and the browser's shell carry the asking in the press's own turn, a refusal comes back as a report, and a recording keeps all of it. Nothing asks yet: the task `sign-in-holds` makes the root ask, and removes the departure. This task is the surface, and changes no behaviour.

Acceptance:

- `Tab`, `Request::Tab`, `Event::TabRefused`, `TabAct`, `Duties::tab`, `Report::TabRefused`, `opening_tab` and the four `Line`s, as `temper/docs/plan/sign-in-new-tab/notes/contracts.md` has them.
- A native host test gives the host `Report::TabRefused` and sees no duty and no panic.
- A view test holds `opening_tab`'s words, "Opening ChatGPT’s sign-in…".
- `journey`'s line round trip holds all four lines, and a replay of a recording whose `tab-open` line differs is stale, naming the turn and line.
- `heavy cargo check -p temper-client-shell-web --target wasm32-unknown-unknown` passes, with and without its `record` feature.

## Inputs

- `temper/docs/plan/sign-in-new-tab/notes/contracts.md`, *The root*, *The host*, *The view*, *The recording*.
- `temper/docs/plan/sign-in-new-tab/notes/common.md`.
- `temper/docs/design/client/README.md`, 3.5, `ConnectModel`, *The sign-in's tab*.
- `temper/docs/design/client/browser.md`, 2.1, `host`.
- `temper/docs/design/client/journeys.md`, 3.1 and 4.2.
- `skein/docs/design/ui-web.md`, section 10, *A tab of the page's own*; section 11, *A person's gesture*.

## Owns and may change

- Owns the files in the header.
- Compiler-forced arms for `Request`, `Event`, `Report` and `Line` are allowed in any temper file, each edited only for that. A shell or harness that opens no tab drops `Request::Tab`: that covers `temper-client-shell-terminal`'s `route.rs`, `temper/tools/gallery/src/driver.rs`, and `temper/tests/client/src/app.rs`. In `route.rs`, map `Tab::Point { to }` to `Notice::Leave { to }`, and drop `Open` and `Close`: the process tier's person follows that notice, and the terminal draws nothing of it.
- `Duties` is built with `..`-less literals in `temper-client-shell-web`'s tests: add the field where the compiler asks.
- If `temper-client-shell-web/src/journey/fields.rs` exists on your base (the draft `real-journeys`), add `Line::TabOpen(bytes) | Line::TabPoint(bytes)` to its text fields and `Line::TabClose | Line::TabRefused` to those with none. If `journey/scan.rs` or `journey/redact.rs` match on `Line`, do the same there.
- `Word::WelcomeTabOpening` goes beside `WelcomeAwayAgain` in `words.rs`'s lists, with the table's next free number. If a test or golden lists the words, regenerate it with its tool.
- Keep the hunks in `words.rs` and `temper-client-domain/src/route.rs` to these lines: other drafts are changing them.

## Steps

1. Domain: `boundary.rs` as the note has it; `lib.rs` exports `Tab`. In `domain.rs`'s `step`, `Event::TabRefused` does nothing yet, with a comment naming the task that feeds it.
2. View: `Word::WelcomeTabOpening`; `opening_tab` in `root.rs`. `welcome_model::said` takes the plan's name from the page's form; here the provider is given, so find its name from `accounts::current_me`'s `providers` by that number, as `connect_model::names` does, and fall back to the word with an empty name when `me` isn't in: the tab's words aren't what it's for.
3. Host: `TabAct`, `Duties::tab`, `Report::TabRefused` as the note has them. `Host::duties` maps `Request::Tab` in its loop, in order. `Report::TabRefused` gives `Event::TabRefused` to the root.
4. Shell (`lib.rs`): `enum Report` gains `TabRefused`; `host_report` maps it; `carry` carries `duties.tab` before `transport::carry`, its first act. A false `open_tab` raises `Report::TabRefused` on the inbox. Comment the order with `skein/docs/design/ui-web.md`, section 11. `Shell<P>` is generic over the platform, so the worker's defaults refuse.
5. Journey: the four `Line`s, their words `tab-open`, `tab-point`, `tab-close`, `tab-refused`. `asked` writes `duties.tab` first. `is_ask` holds the first three. `Replay::report` reads `tab-refused` as `Report::TabRefused`.
6. Tests as the acceptance has them, failing first. Break the rule by hand once for the stale test (let `asked` skip the tab) and see it fail.
7. `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `heavy cargo clippy -p temper-client-domain -p temper-client-view -p temper-client-shell-web -p temper-client-shell-terminal --all-targets -- -D warnings`; `heavy cargo nextest run -p temper-client-domain -p temper-client-view -p temper-client-shell-web -p temper-client-shell-terminal`; `heavy cargo check -p temper-client-world -p temper-client-gallery --tests`. Commit and push.

## Report

As the implementer's text asks. List the compiler-forced arms by file.
