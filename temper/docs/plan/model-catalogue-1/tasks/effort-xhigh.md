+++
id = "effort-xhigh"
part = "effort-xhigh"
kind = "feature"
needs = []
files = [
  "skein/crates/skein-llm/src/anthropic/request.rs",
  "skein/crates/skein-llm/src/anthropic/request_tests.rs",
  "skein/crates/skein-llm/README.md",
]
effort = "medium"
minutes = 15
+++

# Anthropic's dialect takes the effort `xhigh`

## Goal

temper's stock is to declare the efforts `low`, `medium`, `high`, `xhigh` and `max` on every model
(`temper/docs/design/draft/model-catalogue.md`, section 2). Anthropic's dialect refuses `xhigh` as
`Error::Unsupported` before a request leaves, so a person picking it on a Claude model with a key
would fail. Let it through as the dialect's other adaptive-thinking efforts go.

Acceptance:

1. `encode_request` with `reasoning_effort` `xhigh` gives `"thinking":{"type":"adaptive"}` and
   `"output_config":{"effort":"xhigh"}`, as `high` does.
2. An unknown effort is still `Error::Unsupported`, and `off` still disables thinking.

## Inputs

- `temper/docs/plan/model-catalogue-1/notes/common.md`.
- `skein/docs/design/llm.md`, 4.7.

## Owns and may change

- Owns `skein/crates/skein-llm/src/anthropic/request.rs`, its `request_tests.rs`, and the effort list
  in `skein/crates/skein-llm/README.md` (the line naming `low`, `medium`, `high`, `max`).
- Nothing else. `smith-protocol-program`'s `--effort` carries the name untouched, and OpenAI's dialect
  validates none: neither needs a change.

## Steps

1. `request.rs`, the `match effort.as_ref()` in the prompt's measure: add `b"xhigh"`.
2. `request_tests.rs`, `effort_is_explicit_and_unsupported_fields_are_rejected`: add `b"xhigh"` to the
   list, and assert the wire holds `"effort":"xhigh"` for it. Break the arm once to see it bite.
3. `heavy cargo nextest run -p skein-llm --lib -E 'test(=anthropic::request_tests::effort_is_explicit_and_unsupported_fields_are_rejected)'`
   (find the test's exact path with `heavy cargo nextest list -p skein-llm --lib` if it differs), then the
   implementer's text's closing checks. Commit and push.

## Report

As the implementer's text says; the test run, and the failure seen when the arm was removed.
