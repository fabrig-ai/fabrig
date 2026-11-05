+++
id = "more-accounts"
part = "more-accounts"
kind = "feature"
needs = ["account-onboarding"]
files = [
  "temper/crates/temper-client-view/src/pages/home.rs",
  "temper/crates/temper-client-view/src/tests/home.rs",
  "temper/crates/temper-client-view/src/look.rs",
  "temper/crates/temper-client-shell-web/temper.css",
]
effort = "high"
minutes = 90
+++

# "Add Claude or another plan", and its account

## Goal

Land `to-begin-more-accounts` whole: the shorter lede and the link to Accounts while To begin shows,
then its render check and account. Every commit carries `Look-request: to-begin-more-accounts`; the
last holds the table.

Acceptance:

- Every rule of `git diff 9fcbec1d32 0e816f6749 -- temper/docs/design/client/journeys/overlay.css` is
  in `temper.css`; token `temper-home-ledes`.
- **The markup:** `git show 0e816f6749:temper/docs/design/client/journeys/project-landing/01.html`'s
  intro: `div.temper-home-ledes` holding `p.temper-home-lede` and the link, while To begin shows.
- **The lede,** `temper/docs/design/client/README.md`, 3.5, *To begin*, *The intro*: "Your agents run
  on your ACCOUNT, and Forgejo on this machine."; ", and FORGE holds your code."; each half left out
  with no such fact, a forge alone keeping "Forgejo runs on this machine." or "FORGE holds your
  code."; with a key, "Your agents run on an API key" and the same ends. `HomeAgents` and its
  neighbours go where nothing reads them.
- **The link,** *More accounts*, to `Address::Accounts`, by the usable accounts the person holds,
  playground's left out: ChatGPT plan alone, "Add Claude or another plan"; Claude subscription alone,
  "Add ChatGPT or another plan"; both, a key, or none usable, "Add another account". Its words
  promise no capacity: no other wording.
- **The terminal:** the lede, then the link, numbered as links are.
- **Gallery:** `home-accounts-claude`, `home-accounts-both`, `home-accounts-key`.
- **The account,** by `temper/docs/plan/look-round-4/notes/account.md`: `project-landing/01.html` from
  `0e816f6749` against `home-two-left` at 1440, 1000, 420 and 1366 by 630, its cost in height at
  1366 by 630 said; each state in words; `~/src/fabrig/look-round-4/to-begin-more-accounts/`.
- `README.status.md`: the link goes from its *3.5, Home* entry.

## Inputs

- `temper/docs/plan/look-round-4/notes/common.md`, `notes/contracts.md`, `notes/account.md`.
- `temper/docs/design/client/README.md`, 3.5, *To begin*; `temper/docs/design/draft/to-begin.md`, 2.
- `git show --format=%B -s 0e816f6749`.

## Owns and may change

Owns, as `files` lists them.

May also change:

- `temper/crates/temper-client-view/src/{words.rs,limits.rs}`; `src/tests/{fixtures.rs,look.rs}`.
- `temper/crates/temper-client-view/src/pages/accounts.rs`: a function that says what the person
  holds, beside `first_usable`, from `accounts::document`; `first_usable`'s readers stay as they are.
- `temper/tools/gallery/src/{samples.rs,tests.rs,worlds.rs}`, `names.txt`; `temper/docs/design/client/browser.md`, 7.2.
- `temper/docs/design/client/README.status.md`, `temper/docs/design/client/mockups/STATUS.md`.

Options, each pre-approved:

- A provider that is neither OpenAI's plan nor Claude's counts as "another account": "Add another
  account".
- Two ChatGPT plans alone are still a ChatGPT plan alone.

## Steps

1. Rules, tokens, words, `intro`.
2. Tests, `tests/home.rs`: each of the link's cases; each lede's; the node order. Each bites.
3. The render check, the account.
4. **Check,** `notes/common.md`: `heavy cargo nextest run -p temper-client-view -p
   temper-client-shell-terminal -p temper-client-gallery`, clippy, the citations, then the last
   commit, its table in its message, and push.

## Report

The table, the render check's result, the directory and the builder.
