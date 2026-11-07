+++
id = "email-close"
part = "look"
kind = "docs"
needs = ["fake-says-email"]
files = ["jig/docs/design/domain/accounts.status.md", "jig/docs/design/protocol/store.status.md", "jig/docs/design/protocol/channel.status.md", "jig/docs/design/protocol/llm.status.md", "jig/docs/design/client/domain/contract.status.md", "jig/docs/design/STATUS.md", "skein/docs/design/oauth.status.md", "smith/docs/design/protocol/programs.status.md", "smith/docs/design/protocol/hosts.status.md", "temper/docs/design/client/README.status.md", "temper/docs/design/STATUS.md"]
effort = "medium"
minutes = 20
+++

# The status files say what's built

## Goal

Each status file this plan's cut gave a **Not yet** entry says what stands now, and the look request's landing gets its account. This task's departure from the implementer's text: its files are all under `docs/design/`, and it runs nothing but the citations' check.

Acceptance:

- Each entry below is gone, or says what's still not built, as the merged code has it. Read the code, by `rg`, before removing one:
  - `jig/docs/design/domain/accounts.status.md`, 3.7.
  - `jig/docs/design/protocol/store.status.md`, 4.4, 7.2, 15.9.
  - `jig/docs/design/protocol/channel.status.md`, 5.1.
  - `jig/docs/design/protocol/llm.status.md`, 5.2.
  - `jig/docs/design/client/domain/contract.status.md`, 5.1. A file left with no gap says so.
  - `skein/docs/design/oauth.status.md`, 2.2, 6.3.
  - `smith/docs/design/protocol/programs.status.md`, 3, and `hosts.status.md`, 2.
  - `temper/docs/design/client/README.status.md`, 3.5: Ready's and the Model step's email. It takes in its place a **Not yet**: 3.5: the render check of `ready-account-email` on a real sign-in and a real Claude Code login follows landing.
  - `temper/docs/design/STATUS.md`: the sentence saying Ready and the Model step name no email goes.
- Each status file's date is the day you edit it, by `date -u`.
- `jig/docs/design/STATUS.md`'s paragraph on its drafts says what `draft/account-facts.md` still holds: its third step alone.
- `jig/docs/design/draft/account-facts.status.md` stays as the cut wrote it, unless a task's report proved a line of it wrong; then that line changes, and you say so.
- `python3 -I -B tools/citations/citations.py --base origin/main` reports nothing new broken.
- This is the last commit of `ready-account-email`'s landing, so it carries the trailer `Look-request: ready-account-email` and the account's table (`temper/docs/design/draft/look-requests.md`, 5): a row each for Ready's line on a sign-in, Ready's line on a program's login, the terminal's, the Model step's status, `model-check-pane`'s who it is, the picker's heading, the accounts page and the gallery; each landed, with its task. Its render-check row says the check follows landing, and that temper/ui owns it. Nothing is deferred but that.

## Inputs

- `DESIGN.md`, 2 and 4.
- `temper/docs/design/draft/look-requests.md`, 5.
- `temper/docs/plan/account-email/notes/common.md`.

## Owns and may change

Owns the header's files, and no code. A status file a merge of `model-catalogue-3` changed keeps that plan's entries as they are.

## Steps

Edit, check the citations, commit and push.

## Report

As the implementer's text says.
