# ui-n3a-only: plan

Milestone N3a of `temper/docs/design/draft/whole-story.execution.md`,
Only you (`temper/docs/design/draft/what-reaches-you.md`, section 8, as
the mockups' audit words it in `surfaces.md`, section 4): what only the
person can do, listed apart on Needs you. Its design is
`temper/docs/design/client/README.md`, 3.5, **Only you**, written by this
cut, with 5.5's expiring rows and `browser.md`, 7.2's four states.

## What was decided

- **What Only you lists:** each of the person's own LLM accounts whose
  deciding source, the first supplied, is a sign-in or a program's login
  that is signed out (`SignInAgain`) or that the `accounts` document reads
  `expiring`. That is the audit's "a sign-in or a login its account reads
  as expiring", with no deadline, since jig's `ends` fact is said by no
  source (jig's `domain/accounts.md`, 3.7). The deciding source alone,
  because it is the one whose state holds the account's work, the one the
  rail's mark and Accounts' notice read.
- **Draw what the engine holds:** jig's `accounts` document names no task
  waiting on a person's own account, so the card says nothing of what
  waits ("Two agents on vsr wait for this login."); a forge's token to
  replace and a person task asking an act have no read either. All three
  are section 11's later parts in the client's design, and the draft's
  status says so. No jig change: the request draws what the engine holds.
- **Its act** is one link, "Sign in again", to Accounts, as the run
  wait's sign-in link is. Accounts therefore gains the renewal for an
  expiring deciding source ("Expiring: sign in again before it ends." and
  "Continue with ChatGPT", or a program's command and "Check again"),
  which it lacked: part `accounts`.
- **Where:** after the cards' *Answered*, or Done's "Home", before Decided
  for you, once the inbox has answered; counted nowhere a card is (the
  rail, the head, Home's sentence, Done's rule). The browser's alone: the
  terminal's inbox is temper/tui's, a later part.
- **Its tests:** the domain's table of the rule and the parts' table; the
  view's tables of the region and of Accounts' expiring rows; the joint
  world's story through temper's engine, a signed-in plan made expiring,
  then signed out, then signed in again; the density fuzz gains Only
  you's fact; the gallery's four states.

## Parts and tasks

| Part | Task | Needs | Crates | Minutes |
|---|---|---|---|---|
| `read` | `only-read` | | `temper-client-domain` | 70 |
| `accounts` | `accounts-expiring` | | `temper-client-view` (Accounts' rows) | 70 |
| `story` | `only-story` | `only-read` | `temper-client-world` | 100 |
| `region` | `only-region` | `only-read`, `accounts-expiring` | `temper-client-view`, `temper-client-shell-web`'s CSS | 140 |
| `gallery` | `only-gallery` | `only-region`, `accounts-expiring` | `temper-client-gallery` | 60 |

Slices: 1, `read` and `accounts` (released first); 2, `story` and
`region`; 3, `gallery`. `only-region` waits for `accounts-expiring`
only because both write `temper-client-view` (one writer a crate); it
uses none of its code.

## Critical path

`only-read` or `accounts-expiring` (70 min, side by side) →
`only-region` (140) → `only-gallery` (60): about 270 implementer-minutes.
`only-story` (100) runs beside `only-region` once `only-read` merges.

## Who hears first

- **`temper-client-domain`, for the TUI's owner (temper/tui):** a new
  module `pages::only_you` (`Why`, `Item`, `OnlyYou`, `items`, `read`),
  and `pages::parts`' `NeedsFacts` and `NeedsParts` each gain a field,
  `only` (`notes/contracts.md`, sections 1 and 2). No existing fact or
  part changes meaning; the terminal's inbox draws nothing new.
- **`temper-engine-domain` and `forge_route.rs`, for the owner of coding
  runs:** no change.

## Overlaps

- `ui-p3b-reaches` (cut in flight) is How work runs' What reaches you and
  may draw Needs you's intro lede: it may touch
  `temper/docs/design/client/README.md`, 3.5, `README.status.md`,
  `temper-client-view`'s `pages/needs.rs`, `words.rs` and `temper.css`'s
  `ui: needs` block. This plan keeps off the intro: its region is a
  module of its own called by one line of `needs::main`, its words and
  rules are added after their neighbours, and its design is a bullet of
  its own before **Decided for you**. `surfaces.status.md`, 4's lag lost
  "no Only you" in one line, which P3b may also edit.
