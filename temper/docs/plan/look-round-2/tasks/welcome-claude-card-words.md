+++
id = "welcome-claude-card-words"
part = "welcome-claude-card-words"
kind = "feature"
needs = ["welcome-fetch-percentage"]
files = [
  "temper/crates/temper-client-view/src/words.rs",
  "temper/crates/temper-client-view/src/tests/welcome.rs",
  "temper/crates/temper-client-view/src/tests/model_forge.rs",
  "temper/docs/design/client/journeys/welcome-set-up/07.html",
  "temper/docs/design/client/journeys/welcome-set-up/08.html",
]
effort = "medium"
minutes = 45
+++

# Land welcome-claude-card-words: the Claude card says this machine and its own config

## Goal

Land the look request `welcome-claude-card-words`, commit `7bbb7ec2d7`, whole: the Model step's
Claude card's description and note. The card missing from a real box is not this request's: it's
`temper/docs/design/draft/claude-provider.md`'s, on `main`, a deferred row of the account. Leave
that draft and its status to it.

The request's message, whole:

> Look request (temporary): the Claude card says this machine and its own config
>
> The Model step's Claude card said Claude Code ran "on each host
> temper's agents run on", which reads as a fleet's set up. On temper
> box, the host is this machine, and what a person wants to know is
> that temper's Claude Code will not touch theirs (2026-11-04).
>
> - Its description: "Your Claude subscription, through Claude Code on
>   this machine, with a config of its own, apart from your own Claude
>   Code." (in place of "... on each host temper's agents run on.").
> - Its note: "One command in a terminal, on this machine" (in place of
>   "A command in a terminal, on each host").
> - Everything else on the card stays.
>
> Not the look's: the person's own box drew no Claude card at all. Init
> ships no Anthropic provider (temper/docs/design/protocol/configuration.md,
> the fake backends' rewrite, adds one for the fake box alone), so the
> Model step has no Anthropic offer to draw. That is a draft of its own,
> not this request.
>
> Look-request: welcome-claude-card-words
> Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>

Acceptance:

- `welcome-set-up/07.html` and `08.html`, reprinted, equal the request's pages as text, but for what
  `welcome-fetch-percentage` deferred: the request's pages stack on that request's dummy 42%. Each
  such line is an account row.
- The card says both sentences in the browser and the terminal.
- Every commit carries `Look-request: welcome-claude-card-words`; the last holds the account.

## Inputs

- `temper/docs/plan/look-round-2/notes/common.md`: the standing rules. Read it first.
- `temper/docs/design/draft/look-requests.md`, sections 3 to 6.
- `temper/docs/design/client/README.md`, 3.5, The welcome: the Model step's two cards.
- `git show 7bbb7ec2d7`.

## Owns and may change

Owns, as `files` lists them.

May also change:

- `temper/docs/design/client/README.md`, 3.5: the two quoted sentences of the "Claude" card, near
  941.

Options, each pre-approved:

- **Only the card changes.** `Word::ModelProgramDescription` and the words near
  `accounts/words.rs`, 256, are a program's Details and the accounts page: they keep "each host",
  as "Everything else on the card stays" leaves them. The account has a row saying so.
- `mockups/STATUS.md`, 1.1, already says "the words of the cards and leads are the client's. The
  mockup is not edited.": it needs no row. If you find otherwise, add one and say so.
- If a gallery test reads either sentence, change it to the new words and name it in the report.

## Steps

1. **Read** `notes/common.md`, then `git show 7bbb7ec2d7` whole.
2. **The words,** in `words.rs`, as the message has them to the byte, the curly apostrophe of the
   old one gone with it:
   - `Word::WelcomeModelSubscription`: `Your `, its hole, ` subscription, through Claude Code on this
     machine, with a config of its own, apart from your own Claude Code.`
   - `Word::WelcomeModelProgramTime`: `One command in a terminal, on this machine`
3. **The tests:** in `tests/welcome.rs`, the note's expected words, near 575, only. In
   `tests/model_forge.rs`, `the_claude_card_says_this_machine`: both sentences in the card, in
   `browser_page_view` and through `skein_ui_text::render` on `page_view`.
4. **Reprint,** then diff `07.html` and `08.html` against the commit's: expect the card's lines the
   same, and the meter's, the status's and "Each step"'s 42% lines as the last landing left them.
5. **Render check** (`notes/common.md`) at 1440, 1000 and 420 with "Each step" open: the card's box
   grows with its longer words; the rest of the differences are `welcome-fetch-percentage`'s.
6. **The design text:** `README.md`, 3.5.
7. **Account and push.** Rows: the description, the note, the rest of the card unchanged, the Details
   and accounts pages' words (the request names the card alone), the terminal, each line that
   differs from the request's page, and the card missing from a real box (deferred, to
   `temper/docs/design/draft/claude-provider.md`, tracked in its status).

## Report

As `notes/common.md`, Your report. This is the round's last landing: say so, and list the four
earlier requests' deferred rows beside your own, with where each is tracked, from what the trunk
holds.
