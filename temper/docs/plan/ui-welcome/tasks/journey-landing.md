+++
id = "journey-landing"
part = "journey-landing"
kind = "feature"
needs = ["home-add", "box-terminal"]
files = [
  "temper/docs/design/client/journeys/**",
  "temper/docs/design/client/journeys.md",
  "temper/docs/design/client/journeys.status.md",
  "temper/tools/gallery/**",
]
effort = "high"
minutes = 100
+++

# The journeys

## Goal

`project-landing` is recorded at last, on Home, from a walk; `welcome-set-up`'s prints are current; every state of the request's
list is in the gallery. Request, section 10.

Acceptance:

- **`project-landing`:** its recording is a fresh walk's as `temper/docs/design/client/journeys.md`, 9 has it: a fresh `temper box
  --fake-backends`; Forgejo with an empty `work/hello`; the ChatGPT plan signed in; Ready's "Hello" created and its repository
  adopted; once Home says "work/hello is in Hello", `/projects/1` reloaded, where the recording starts; `01.html` prints Home.
  `temper-client-journeys` passes on it.
- **Its `README.md`, the table in 9 and `journeys.status.md`** say what's true: the **Not yet** on 9 goes.
- **`welcome-set-up`** adds nothing past Model, stays within `again`'s 40 seconds, and its prints are current, refreshed only as
  these changes reached them.
- **The gallery holds** all of `welcome-go-on`, `welcome-forge-under-way`, `welcome-forge-connected`, `welcome-forge-github`,
  `welcome-model-connected`, `welcome-model-claude`, `welcome-model-claude-checking`, `welcome-model-claude-not-yet`,
  `welcome-model-away`, `welcome-ready-none`, `welcome-ready-signing`, `welcome-ready-copies`, `welcome-sheet-copies`,
  `welcome-sheet-copies-refused`, `welcome-sheet-nothing`, `home-copying`, `home-copied`, `home-add-repository`,
  `home-add-repository-sheet`. A missing one is added here.

## Inputs

- `temper/docs/plan/ui-welcome/notes/common.md`.
- `temper/docs/plan/ui-welcome/inputs/request.md`, section 10.
- `temper/docs/design/client/journeys.md`, 3.5, 4.3, 8 and 9. `temper/docs/design/client/journeys/project-landing/README.md`.
- `temper/crates/temper/tests/fake_backends/walk/sign_in.rs`: a walk of `ui-signin`'s kind.

## Owns and may change

Owns the files in `files`.

May also change: a line in `temper/crates/temper-client-view/src/**` or `temper-client-shell-web/temper.css` for a fault the walk
finds, named in the report.

Options, pre-approved:

- **The walk's driver** is a throwaway script: headless Chromium through CDP, on the recording bundle (`scripts/bundle.sh
  --loopback --record`), `SKEIN_TEST_CHROMIUM` for the browser. Answer `Page.javascriptDialogOpening` in the driver. Save with
  Alt+Shift+S; `heavy cargo run -p temper-client-journeys -- add project-landing FILE`.
- **`ui-signin` must have landed** for ChatGPT to sign in on a fresh box in a browser: it lands first. Check your base holds
  `temper/crates/temper-fake/src/oauth.rs`'s issuer connections (`git log -1 --format=%h -S'issuer' -- temper/crates/temper-fake/src/oauth.rs`,
  and its walk passing). If it's not on your base, that truly blocks: `Question:`, with its state.
- A journey gone stale by the exchange: `again NAME` while its acts hold; if a walk changed, walk it anew.

Not this task's: `first-run-walks`.

## Steps

1. Walk, save, `add`. A first replay refused is a defect in the seam or the replay: fix it, never record around it
   (`journeys.md`, 4.3).
2. `README.md`: the walk, as it now holds; the stop it recorded goes. The table's row in 9. `journeys.status.md`.
3. `heavy cargo run -p temper-client-journeys`; `heavy cargo nextest run -p temper-client-journeys`.
4. **Check,** `notes/common.md`; commit and push.

## Report

The walk's steps and times; what the recording's print shows; the builder.
