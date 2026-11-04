+++
id = "ready-account-email"
part = "ready-account-email"
kind = "feature"
needs = ["welcome-model-forge-pane"]
files = [
  "temper/tools/gallery/src/worlds.rs",
  "temper/tools/gallery/src/tests.rs",
  "temper/docs/design/client/README.status.md",
]
effort = "medium"
minutes = 45
+++

# Land ready-account-email: the gallery's Ready says an email; the email itself is deferred

## Goal

Land what the look can of the look request `ready-account-email`, commit `0a94e5f1fb`: the gallery's
Ready world gives the ChatGPT account an email as its display, so that the gallery draws
"Model ChatGPT plan · sam@example.com". **The view needs no change, and the email itself is not
built:** it's the engine's. The user chose way B, and jig's account-facts draft, under the user's
review and not on `main` yet, designs it. It is a deferred row of the account, to that draft.

The request's message, whole:

> Look request (temporary): Ready names the ChatGPT account's email
>
> At Ready, the person's own box said "Model ChatGPT plan" and nothing
> more: which ChatGPT account temper's agents use was not said. Wanted:
> "Model ChatGPT plan · you@example.com", the signed-in account's email
> after the plan, as `welcome-ready/01.html` draws it with the value
> written in by hand.
>
> That page is hand-made (its README says so): no journey reaches Ready
> yet, so a landing cannot reprint it, and nothing in the overlay
> changes. Done is the person's own box saying the email at Ready, in
> the browser and the terminal.
>
> The fix is not the view's. `first_usable_short`
> (temper-client-view/src/pages/accounts.rs) already joins the account's
> display after "ChatGPT plan"; for a ChatGPT sign-in the display is
> empty. The engine asks for the `email` scope and validates the ID
> token (jig/docs/design/domain/providers.md, 2.2), but keeps no email
> for the account, whose display is "a display name its holder gives"
> (jig/docs/design/domain/accounts.md, 3.1).
>
> Two ways were weighed with the user (2026-11-04), who leans to the
> second, more work now and better later:
>
> - A: the sign-in's verified email fills the display when its holder
>   gave none. No format changes, but the display stops being the
>   holder's alone, a rename hides the email, and a later sign-in does
>   not refresh it.
> - B: the account carries the provider's verified email in a field of
>   its own, beside the display, refreshed at each sign-in. It changes
>   a stored format and the protocol, so it goes to the user as a draft
>   before any code; with no deployment to carry over
>   (temper/docs/design/draft/serve-without-box.md, 2), now is its
>   cheapest time. Who may see it (the holder, or everyone who sees the
>   account) is part of that draft.
>
> Either way the account stays the validated `sub`, never the email.
>
> Beside it, the gallery's Ready world gives the account "ChatGPT" as its
> display ("ChatGPT plan · ChatGPT"); it should give an email, so that
> the gallery draws this case.
>
> Look-request: ready-account-email
> Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>

Acceptance:

- The gallery's `welcome-ready-adopted` holds "Model ChatGPT plan · sam@example.com", a test says so,
  and nothing of `temper-client-view` changes.
- The render check compares that state with the request's `welcome-ready/01.html` at 1440, 1000 and
  420; its result is in the account.
- `README.status.md` carries the deferred row, naming its owner.
- Every commit carries `Look-request: ready-account-email`; the last holds the account.

## Inputs

- `temper/docs/plan/look-round-2/notes/common.md`: the standing rules. Read it first.
- `temper/docs/design/draft/look-requests.md`, sections 3 to 6, 5's deferred row and 6's engine's
  fact most.
- `temper/docs/design/client/browser.md`, 7.2: the worlds' values are the mockups'.
- `git show 0a94e5f1fb`.

## Owns and may change

Owns, as `files` lists them.

May also change: nothing else. Options, each pre-approved:

- The value is `sam@example.com`, the first-run world's person in the mockups
  (`mockups/world-first-run-real.js`), as `browser.md`, 7.2, has the worlds' values. The request's
  `you@example.com` is a value written in by hand; the account says "differs, and why".
- If an assertion in `temper/tools/gallery/src/tests.rs` reads the old "ChatGPT" display, change it
  to the new value and name it in the report.
- Only the `FirstRun { step: 5..=9 }` arm of `display` changes (`worlds.rs`, the second account's,
  near 1984). The "ChatGPT" of the earlier arm, near 1900, is the Model samples', not Ready's: leave
  it.

## Steps

1. **Read** `notes/common.md`, then `git show 0a94e5f1fb` whole.
2. **The world:** the display, as above.
3. **The test:** in `tests.rs`, `welcome-ready-adopted`'s `Holds` gains
   `Part::Named { role: Role::Paragraph, name: "Model ChatGPT plan · sam@example.com" }`. Break it
   once and see it fail.
4. **Nothing to reprint:** no journey reaches Ready, and `welcome-ready/` stays on the person's
   branch. The journeys' tests still pass.
5. **Render check** (`notes/common.md`): `request/` as there; the landed side is the gallery's
   `welcome-ready-adopted`, printed by `heavy cargo run -p temper-client-gallery --
   ~/src/fabrig/look-round-2/ready-account-email/gallery`. The two pages link their stylesheets
   differently, so compare from `#app` down. Open nothing on this page but what it has.
6. **The terminal:** the view is unchanged; `temper/tests/client/tests/suite/connect_model.rs`, 248,
   is the print's line. Run it, and say so in the account.
7. **The deferred row:** in `temper/docs/design/client/README.status.md`, under **Not yet**, among
   its 3.5 entries, one sentence: Ready's model line names no email for a ChatGPT sign-in, whose
   account's display is empty, since the engine keeps no email; jig's account-facts draft designs
   it. Name that draft in words, with no path (`notes/common.md`, Your checks).
8. **Account and push.** Rows: the page's model line, landed in the gallery alone, differing by its
   hand-written value; the person's own box saying the email in the browser and in the terminal,
   deferred to jig's account-facts draft, tracked in `README.status.md`, 3.5; the overlay, nothing;
   the gallery's display, landed; way A, not taken, the user having chosen B.

## Report

As `notes/common.md`, Your report.
