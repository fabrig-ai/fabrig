+++
id = "welcome-fetch-percentage"
part = "welcome-fetch-percentage"
kind = "feature"
needs = ["welcome-forge-busy-bar"]
files = [
  "temper/crates/temper-client-view/src/look.rs",
  "temper/crates/temper-client-view/src/tests/look.rs",
  "temper/crates/temper-client-shell-web/temper.css",
  "temper/docs/design/draft/fetch-progress.status.md",
]
effort = "medium"
minutes = 50
+++

# Land welcome-fetch-percentage: the meter's token and rule, ready for a value

## Goal

Land what the view can of the look request `welcome-fetch-percentage`, commit `218a9b9080`: the
style token `temper-flow-meter` and its rule, ready for a value, with no node drawing it yet. The
percentage and the fill need data the client does not have.
`temper/docs/design/draft/fetch-progress.md`, on `main`, carries the percentage; the fill needs a
skein draft not written yet. Both are deferred rows of the account. **The status line, the view and
the pages do not change:** no reprint moves.

The request's message, whole:

> Look request (temporary): the fetch's percentage, with its bar filled
>
> While the forge's set up fetches its binaries, the person wants to see
> how far it is (2026-11-04). Drawn with a dummy 42%: the values are
> written in by hand, since the fake box fetches nothing a walk can see.
>
> - The status line names the binary: "Setting up: fetching Forgejo",
>   then "Setting up: fetching its runner".
> - Under it, in place of the moving bar, a bar filled to the percentage,
>   4px, ink over the hairline colour, and the percentage at its end,
>   muted, in tabular figures ("42%"). It is the progress node (skein's
>   Role::Progress), named "Fetching Forgejo", its text the percentage,
>   which is also its `aria-valuetext`, styled temper-flow-meter.
> - "Each step"'s first step says it too: "Fetching Forgejo and its
>   runner · 42%".
> - With no size known, the bytes in place of the percentage ("41 MB")
>   over the moving bar of welcome-forge-busy-bar; the steps after the
>   fetch keep the moving bar.
>
> Not the look's alone: the percentage needs the fetch to report it and
> the engine to carry it, temper/docs/design/draft/fetch-progress.md
> (queued). The landing takes it in two parts:
>
> 1. Once that draft is built, the percentage in words beside the moving
>    bar, which this request's look already places.
> 2. The fill: skein's progress node carries text alone
>    (skein/docs/design/ui.md, 3.1), so the bar's width needs a value of
>    skein's, in a draft of skein's own; until it lands, the moving bar
>    stays and the fill rule waits.
>
> Look-request: welcome-fetch-percentage
> Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>

Acceptance:

- `look::FLOW_METER` is named `temper-flow-meter`, and `temper.css` holds both of the commit's rules,
  with the fill's hand-written 42% replaced by a value the rule is ready for, 0% by default.
- A hand-made copy of the request's `07.html`, given that value at 42%, renders under the landed
  stylesheets as the request's does at 1440, 1000 and 420.
- `welcome-set-up/07.html` and `08.html` do not change, and the journeys' tests pass.
- `fetch-progress.status.md` carries every deferred row, naming its owner.
- Every commit carries `Look-request: welcome-fetch-percentage`; the last holds the account.

## Inputs

- `temper/docs/plan/look-round-2/notes/common.md`: the standing rules. Read it first.
- `temper/docs/design/draft/look-requests.md`, sections 3 to 6, 5's deferred row and 6's engine's
  fact most.
- `temper/docs/design/draft/fetch-progress.md`, sections 5 and 6, and its status.
- `skein/docs/design/ui.md`, 3.1.
- `git show 218a9b9080`.

## Owns and may change

Owns, as `files` lists them. In `fetch-progress.status.md`, this task adds its deferred rows. The
draft's own text stays as it is.

May also change: nothing else. Options, each pre-approved:

- **The message says "the fill rule waits", the round's request says to land the rule ready for a
  value.** The round's request is the later word, and the two agree in effect: the fill waits, its
  width at 0%, and no node carries the token. The account says so.
- If another plan has taken token 174 on the trunk, take the next free one.

## Steps

1. **Read** `notes/common.md`, then `git show 218a9b9080` whole.
2. **The token:** `look::FLOW_METER`, token 174, `b"temper-flow-meter"` after `temper-flow-busy` in
   `NAMES`; `tests/look.rs`'s `FLOW_NAMES` gains it. Its doc comment says what it is, cites
   `temper/docs/design/draft/fetch-progress.md`, 5 and 6, and says no node carries it yet.
3. **The rules:** after the busy bar's in `temper.css`, the commit's two overlay rules, selectors
   and declarations as they stand, but the fill:

   ```css
   background: linear-gradient(to right, var(--gf-ink-2) 0 var(--temper-flow-meter-fill, 0%), var(--gf-line) var(--temper-flow-meter-fill, 0%) 100%);
   ```

   Its comment says the width is the progress node's value once skein's carries one, in the words
   of the draft's section 6, with no dummy.
4. **Nothing to reprint:** the journeys' tests pass, and `git status` shows no page changed.
5. **Render check** (`notes/common.md`), of this shape, since no reprint carries the request's page:
   - the request's `07.html` and `08.html`, under `request/`, as they stand;
   - on the landed side, the pages reprinted, compared with the request's for the account's rows,
     each difference a deferred row;
   - a hand-made copy of the request's `07.html` in `landed/`, with
     `style="--temper-flow-meter-fill: 42%"` on its `.temper-flow-meter`, compared with the request's
     element by element at the three widths. Its meter and its `::before` are identical, or the
     account says why. Keep its screenshots as `meter-<width>-landed.png`.
6. **The deferred rows:** in `fetch-progress.status.md`, under **Design lags**:
   - in the entry for 5, that the look request `welcome-fetch-percentage` waits on it for the status
     line naming the binary, the percentage in words beside the moving bar, the progress node's name
     and `aria-valuetext`, "Each step"'s first step, and the bytes with no size; `temper-flow-meter`'s
     token and rule are landed, and no node carries them;
   - a new entry for 6: skein's draft of a progress value is not written; the bar's fill waits on it,
     the rule's `--temper-flow-meter-fill` at 0% until then.
7. **Account and push.** Rows: each of the commit's two overlay rules (landed; the second differs by
   its fill's value, and why); the status line's binary, the node, its name and text, "Each step"'s
   first step and the bytes (deferred, to `temper/docs/design/draft/fetch-progress.md`, tracked in
   its status, 5); the fill (deferred, to skein's draft, tracked there, 6); the moving bar staying;
   the terminal (nothing changes; the percentage's words are the draft's section 5).

## Report

As `notes/common.md`, Your report.
