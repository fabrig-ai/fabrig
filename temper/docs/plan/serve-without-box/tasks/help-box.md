+++
id = "help-box"
part = "document"
kind = "docs"
needs = ["init-box"]
files = ["temper/docs/help/box.md", "temper/docs/help/playground.md", "temper/docs/help/account.md", "temper/docs/help/forge.md", "temper/docs/help/deployment.md", "temper/docs/help/index.md", "temper/docs/help/party.md", "temper/docs/help/project.md", "temper/README.md", "temper/docs/design/draft/serve-without-box.status.md"]
effort = "medium"
minutes = 30
+++

# The help says what `temper init` and `--box` write

## Goal

The help pages follow `temper init`'s two forms. A departure from the implementer's text: this task
owns these `docs/help/` pages and the one draft status line below.

Acceptance:

- `temper/docs/help/box.md`, step 1: `temper init --box --config CONFIG --data DATA --cache CACHE`,
  and its lines at 61-72 and 102-104 follow: plain `temper init` then `temper box` has no playground,
  and `--fake-backends` refuses it. Step 1's heading and the words `ports` and `init` stay, as
  `temper/crates/temper/src/tests/help.rs:509` needs them.
- `playground.md:12`, `account.md:10` and `forge.md:12` say `temper init --box` where they name
  the fake forge, the scripted provider or the machine's places.
- `deployment.md:12,26,102,170`, `index.md:10`, `party.md:28,96`, `project.md:29`: read each; where
  the sentence is the box's, `temper init --box`; where it holds of both, leave it.
- `temper/README.md:78`: the same.
- `heavy cargo nextest run -p temper --lib -E 'test(help)'` and `-p temper-docs` pass.
- `temper/docs/design/draft/serve-without-box.status.md` loses the line "2: `../../help/box.md` and
  the `commands` page's init entry follow `temper init` before `--box`."; if **Design lags** is then
  empty, the heading goes with it.

## Inputs

- `temper/docs/plan/serve-without-box/notes/common.md`.
- `temper/docs/design/protocol/setup.md`, 8.2 and 8.3.
- `temper/docs/design/draft/serve-without-box.md`, 2.

## Owns and may change

- The files of the header. The prose stays the page's own voice: `temper-docs`' rules test holds
  head, voice, Expect and placeholders; a page's bound holds.
- `temper/docs/help/server.md` starts from `docs/examples/owned.json`: no change. What `temper
  init` alone says beyond this is later (the draft, 6): add no walk for it.
- `temper/docs/help/reference/**` are generated, `init-box`'s.

## Steps

1. Each page; the tests; commit and push.

## Report

As the implementer's text asks; list each sentence left as it is, and why.
