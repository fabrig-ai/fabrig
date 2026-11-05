+++
id = "to-begin-card"
part = "to-begin-card"
kind = "feature"
needs = ["home-everyday", "to-begin-facts"]
files = [
  "temper/crates/temper-client-view/src/pages/home.rs",
  "temper/crates/temper-client-view/src/tests/home.rs",
  "temper/crates/temper-client-view/src/look.rs",
  "temper/crates/temper-client-shell-web/temper.css",
  "temper/tools/gallery/src/samples.rs",
  "temper/tools/gallery/src/tests.rs",
  "temper/tools/gallery/names.txt",
]
effort = "high"
minutes = 150
+++

# "To begin", a card, with "Skip"

## Goal

Home's To begin becomes the request `project-home-onboarding`'s card, drawn from the facts
`to-begin-facts` gives, shown as the draft says and with "Skip" wired. Every commit carries
`Look-request: project-home-onboarding`.

Acceptance:

- Every rule of `git diff 8bbc1bc964 9fcbec1d32 -- temper/docs/design/client/journeys/overlay.css` is
  in `temper.css`, as `notes/common.md` says, and the rules of Home's steps it replaces go. Tokens
  `temper-home-tag`, `temper-home-skip`, `temper-home-note`.
- The markup is `git show 9fcbec1d32:temper/docs/design/client/journeys/project-landing/01.html`'s
  `main`: names, classes and order. Words and states are
  `temper/docs/design/client/README.md`, 3.5, *To begin*.
- **Showing:** To begin draws while `facts.to_begin()`, not while `left() > 0`.
- **The intro while it shows:** the kicker, "PROJECT is ready.", the lede with no "PROJECT has no
  repository yet.": "Your agents run on your ACCOUNT. Forgejo runs on this machine.", as the request
  draws it; `more-accounts` shortens it. `HomeThree`, `HomeTwo`, `HomeOne`, `HomeReady` and
  `HomeNoRepository` go, and the headline with To begin gone is `home-everyday`'s.
- **The card:** `section.temper-home-begin`; `.temper-home-head` with `h2` "To begin",
  `span.temper-home-tag` "Optional", `button.temper-home-skip` "Skip" (`Act::SkipToBegin`);
  `p.temper-home-note` by what is left, its five wordings; `ul.temper-home-steps`, each row one
  `li.temper-home-step`, `.temper-home-done` once done, with no `.temper-home-acts` then.
- **The steps:** the design's, each state of it. Steps 1 and 3 done draw their title alone. Step 2:
  in, adopting ("Adopting ORG/NAME into PROJECT…"), offered with its refusal's words, GitHub, none:
  one act, "Adopt" (`HomeAdoptLink`), `.temper-home-primary`, `Act::AdoptFirst` for a first
  repository, a `Go` to `Adopt { project }` otherwise. "Adopt another…" and `HomeAdopt` go. "New
  chat" has no extra class, as drawn.
- **Refused:** `skip_refused` draws a `Role::Alert` under the note, "temper couldn't keep that. Try
  Skip again."
- **Empty sections:** while To begin shows, Needs you and Chats are left out when empty; Connected is
  left out while it shows, and the landing's line isn't drawn at all.
- **The terminal:** "To begin" keeps its heading, "Optional" and "Skip" read in line; a step reads
  "1. Tell temper how you work", "Done. " before a done one, as today.
- **Gallery:** the `home-*` states draw the card; `home-three-left`, `home-two-left`, `home-one-left`,
  `home-adopting`, `home-refused`, `home-no-repository`, `home-github` hold what their names say with
  the new words. New: `home-after-skip`, the request's `project-after-skip/01.html`; `home-skip-refused`;
  `home-needs-under-begin` (something needs the person, a chat runs, To begin showing);
  `home-begin-settings-left` (the first and second left); `home-begin-one-chats` (the third alone).
- `home_title_keeps_the_page_name_when_the_headline_counts_steps` is renamed for what it holds.

## Inputs

- `temper/docs/plan/look-round-4/notes/common.md`, `notes/contracts.md`.
- `temper/docs/design/client/README.md`, 3.5, *To begin*; `temper/docs/design/draft/to-begin.md`, 2.
- `git show --format=%B -s 9fcbec1d32`; `project-landing/01.html` and `project-after-skip/01.html` at
  that commit.

## Owns and may change

Owns, as `files` lists them.

May also change:

- `temper/crates/temper-client-view/src/words.rs`, `limits.rs`: as `home-everyday`.
- `temper/crates/temper-client-view/src/tests/{fixtures.rs,look.rs}`: fixtures, tokens.
- `temper/tools/gallery/src/worlds.rs`, and `samples.rs`'s `Watchable::Family` arm, which answers
  every family watch with the forges' bytes: it must tell the `dismissed` watch from `forges` and
  answer it with a `Dismissals` document, or no sample reaches `to_begin()`.
- `temper/tools/journeys/src/print.rs`: the test's name.
- `temper/docs/design/client/browser.md`, 7.2: the gallery's table.
- `temper/docs/design/client/README.status.md`: from its *3.5, Home* entry, all but Connected, the
  link to `Accounts` and what `account-everyday` adds.
- `temper/docs/design/client/journeys/project-landing/README.md`: the states' list, if it names them.

Options, each pre-approved:

- The refusal's words after step 2's line stay `forge::page::outcome`'s.
- If `HomeGitHub`'s and `HomeNone`'s words share a sentence, keep one word for it; the design's lines
  are what binds.
- A world that needs `to_begin()` to go true and can't deliver `settings`: fix the sample.

## Steps

1. The rules and tokens, then `pages/home.rs`: `to_begin` and `repository_step` as above.
2. **Tests,** `tests/home.rs`: every state above in both shells where they differ; each act's event;
   the node order against the request's; the note's five wordings; a project skipped; a project with
   every step done; To begin left out until `delivered`. Each rule-pinning test bites.
3. **Gallery,** and render each at 1440, 1000 and 420, and 1366 by 630 for `home-two-left`: To begin
   and the intro fit it, the design's *Small*.
4. **Check,** `notes/common.md`: `heavy cargo nextest run -p temper-client-domain -p
   temper-client-view -p temper-client-shell-terminal -p temper-client-gallery -p
   temper-client-journeys -p temper-client-world`, clippy on the same, push.

## Report

As `notes/common.md`, Your report, every state with where it renders, and what 1366 by 630 showed.

## Rebase (2026-11-05, step merge, foreman look-round-4/to-begin-card/foreman-1)

rebase and combine both Home status updates in README.status.md
The merge conflicts only in `temper/docs/design/client/README.status.md`.
Integration commit `df6b13f213` adds the outstanding Home chat-row “landed” reader gap beside the onboarding entry that `e0e2f435a1` replaces.
Resolve by retaining the branch’s replacement:
  3.5, Home: no link to `Accounts` after the lede; Connected and its
  rename are still drawn once To begin is gone.
Immediately after it, preserve `df6b13f213`’s complete four-line entry about “landed”, the chat-row reader, and supplied chat state including “closed”.
Keep the date `2026-11-05` and all other status entries.
Do not restore the obsolete gaps for the card, Optional, Skip, or delivery/skip visibility.
Preserve both sides’ changes in the automatically merged `temper/docs/design/client/journeys/project-landing/README.md` and `temper/tools/gallery/src/samples.rs`.
This refusal requires a status-document conflict resolution; it reports no build or test failure.
