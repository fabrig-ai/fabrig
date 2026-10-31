+++
id = "stage-forge"
part = "stage-sites"
kind = "migration"
repo = "fabrig"
needs = ["ratchets", "kit-io", "shell-surfaces", "echo-stage"]
files = ["skein/tests/forge/conformance/src/component/hosted.rs", "skein/tests/forge/setup/src/closing.rs", "skein/testing/skein-throwaway-forgejo/src/http/host.rs", "skein/scripts/old-forms.txt"]
effort = "medium"
minutes = 35
+++
# Stage sites in the forge worlds and the throwaway Forgejo

Read first: `skein/docs/plan/component-kit/notes/common.md`, `skein/docs/plan/component-kit/notes/stage-common.md`, `skein/docs/plan/component-kit/notes/decisions.md`.

Draft, 3.3.

## Sites

- `skein/tests/forge/conformance/src/component/hosted.rs:254` and `:259`
  (two calls in one stage: one loop's resume and a second pass; keep the
  order its tests pin).
- `skein/tests/forge/setup/src/closing.rs:67`.
- `skein/testing/skein-throwaway-forgejo/src/http/host.rs:81`.

## Pre-approved defaults

- The throwaway Forgejo and the conformance runs against it are slow
  tiers: compile them and say so.

## Acceptance

- `io_resume` falls by four, less named sites.
- `heavy cargo nextest run -p <tests/forge/setup's package>`, and the
  compile of the others, with counts.
