+++
id = "list-help"
part = "on"
kind = "docs"
needs = ["list-on", "models-read"]
files = ["temper/docs/help/account.md", "temper/docs/help/deployment.md", "temper/docs/help/settings.md"]
effort = "medium"
minutes = 40
+++

# The help says it reads lists

## Goal

The help tells a person what now happens by itself. This task changes `docs/help/`, by its spec: that departs from the implementer's text.

Acceptance:

- `temper/docs/help/settings.md:163`, Models: temper reads each provider's list where it can, as an account becomes usable, once a day and on "Read again"; a new model comes in off the picker, marked "new" for a week, and its prices are estimated until the configuring chat sets them; a read that fails says so and changes nothing; a Claude subscription has no list, so a new model there is added by name.
- `temper/docs/help/deployment.md:77`, "Offering another model": a provider's `keep` and `drop` come first; adding by name remains for what a list doesn't show.
- `temper/docs/help/account.md:26,99`: a line that an account's list decides which of its provider's models it's offered.
- `temper/crates/temper/src/tests/help.rs:383,505` and `temper-docs`' rules pass. A page past its bound is shortened, losing no fact.

## Inputs

- `temper/docs/plan/model-catalogue-3/notes/common.md`.
- `temper/docs/design/draft/model-catalogue.md`, section 5; `temper/docs/design/client/README.md`, 3.5.

## Owns and may change

Owns the header's files. Each page's voice is its own: read its siblings' lines first. No configuration member's name goes in `settings.md`, a person's page; `deployment.md` is an operator's.

## Steps

Then `heavy cargo nextest run -p temper --lib -E 'test(/help/)'`; `heavy cargo nextest run -p temper-docs`. Commit and push.

## Report

As the implementer's text says.
