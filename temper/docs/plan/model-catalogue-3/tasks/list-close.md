+++
id = "list-close"
part = "on"
kind = "docs"
needs = ["list-story", "list-help", "models-read", "core-reads", "offer-narrows", "temper-worker", "list-on"]
files = ["temper/docs/design/draft/model-catalogue.status.md", "temper/docs/design/client/README.status.md", "temper/docs/design/STATUS.md", "jig/docs/design/STATUS.md", "smith/docs/design/STATUS.md", "skein/docs/design/STATUS.md", "**/*.status.md"]
effort = "medium"
minutes = 50
+++

# The status files, and the look's account

## Goal

Every status line this plan wrote is gone or true, and the look request's account for what this step landed is on record. No walk or render check runs here: they follow landing.

Acceptance:

- `rg -n 'list' ` over the status files this plan's cut changed (skein's `llm`, `llm-connection`, `fake-llm`; smith's `protocol/agent`, `protocol/hosts`; jig's `domain/accounts`, `domain/engine`, `domain/hosts`, `protocol/channel`, `protocol/store`, `protocol/configuration`, `client/domain/contract`; temper's `client/README`, `protocol/configuration`) shows no **Not yet** line for what is now built. A task that left one is finished here; what's truly unbuilt stays, truly worded, with its reason.
- `temper/docs/design/draft/model-catalogue.status.md`: *Design departs* stays as the cut wrote it, unless a report says it went otherwise, then it's corrected to what's built. Its *Open* is unchanged.
- Each tree's `docs/design/STATUS.md` index line for a status file whose gaps changed is true.
- **The account,** in this task's last commit's message, with the trailer `Look-request: model-picker`, directly above the other trailers: a row per state of step 3 the message and 07 name, `temper/docs/design/draft/look-requests.md`, 5's table: "Read again"; "Read from OpenAI 2 hours ago"; "Couldn't read OpenAI's list · 2 hours ago"; "new" on Settings; "new" on All models; 07's "the default without Claude", and its wording "Claude Code has no list to read: these come with temper." beside the design's "These come with temper.", each landed, or differs and why, or deferred, to whom. The render-check row says it follows landing and is its owner's. `models-read`'s report names what else `07.html` draws.
- `python3 -I -B tools/citations/citations.py --base origin/main` lists nothing new.

## Inputs

- `temper/docs/plan/model-catalogue-3/notes/common.md`.
- `DESIGN.md`, section 2.
- `temper/docs/design/draft/look-requests.md`, 3 and 5.
- `temper/docs/design/client/README.status.md`, its `model-picker` lines.

## Owns and may change

Owns the header's files, each only as above. Any `.md` this acceptance names. A design line a report proved wrong may be corrected; a design question it opens goes to `temper/docs/design/draft/model-catalogue.status.md`'s *Open*, saying so in the report. This departs from the implementer's text: `docs/design/` is yours here.

Options, each pre-approved:

- A task's report isn't to hand: read `git log` and the code, and check the line against what's built.
- If an implemented state differs from `README.md`, 3.5, change the design only where the code is right by the request; where the code is wrong, say so under `## Left`.

## Steps

Read each status file named. Edit. Run the citations tool, then `heavy cargo nextest run -p jig-protocol-store --lib -E 'test(words)'` and `heavy cargo nextest run -p temper-client-journeys -E 'test(/readme/)'` if those files moved. Commit and push.

## Report

As the implementer's text says, with the account's table copied.
