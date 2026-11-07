# json-strings: the rules every task names

The request is `skein/docs/plan/json-strings/inputs/request.md`: read *What's wanted*, *Limits* and *For every task of this plan*, which bind. `skein/docs/plan/json-strings/notes/contracts.md` holds the shapes that cross the two tasks and what the check of the writing pass's callers found.

## What stays the same

- **No build setting changes,** and no public interface or behaviour of `skein-json` changes: the same bytes written, the same refusals, the same errors at the same bytes (the request). A test that holds today keeps holding, unedited, but where your spec names it.
- **The subset holds** (`skein/docs/foundation/programming-model.md`, 10): no indexing, no `as`, no unchecked arithmetic, every `match` lists every arm. `core::str::from_utf8` and `skein_lib::bytes::is_utf8` stay out of `skein-json` (`skein/docs/design/json.md`, 7). A test crate may use the standard library's check as its reference.
- **What a plain loop may hold:** a slice pattern and a `match` on the byte, and no call. A call per run is fine; a call per byte in a plain run is what this plan removes (`skein/docs/design/json.md`, 7). A non-plain byte may cost what it costs today.

## What stays out

- Only `skein/crates/skein-json`, its tests (`skein/tests/json`, `skein/testing/skein-json-kit`), and your lines of `skein/docs/design/json.status.md`.
- `skein/testing/skein-heap` is another plan's. `skein_llm::wire::json` and each tree's decoders are out, even if they look wasteful: name what you see in your report.
- `json_text` and `number` keep their checks on both passes (`skein/docs/design/json.md`, 4), and `string::next` stays per byte for borrowed spelling.

## Departures from the implementer's text

- **Status lines:** the design for this plan is written, in the sections your spec cites. You change no other design text, but your task removes the **Not yet** lines your spec names from `skein/docs/design/json.status.md` once its code holds.
- **Direct dependents:** 59 packages read `skein-json`, in every tree, and no signature changes. So you don't run `heavy cargo check` on dependents, and of their tests you run only those your spec lists. The trunk's end check runs the wider set.

## Before you call a failure outside your change

Run it once on the pristine base. If main passes and your branch fails, it's yours (the request).

## Runs

Only the focused and fuzzy suites run: never `tools/tiers/run`, a browser, process or live tier, one of their stories, or a tier's CPU figure. No acceptance figure is asked of this plan. A profile of your own, if you want one to choose inside a sketch, is throwaway and stays out of the branch.
