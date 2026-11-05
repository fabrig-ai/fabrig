+++
id = "ready-step"
part = "ready-step"
kind = "feature"
needs = ["claude-way"]
files = [
  "temper/crates/temper-client-view/src/pages/welcome.rs",
  "temper/crates/temper-client-view/src/pages/accounts.rs",
  "temper/crates/temper-client-view/src/forge/connect.rs",
  "temper/crates/temper-client-view/src/look.rs",
  "temper/crates/temper-client-view/src/words.rs",
  "temper/crates/temper-client-view/src/tests/welcome.rs",
  "temper/crates/temper-client-shell-web/temper.css",
  "temper/tools/gallery/**",
  "temper/docs/design/client/journeys/**",
]
effort = "high"
minutes = 110
+++

# Ready

## Goal

Ready's checklist, words, note and foot as the mock draws them. Its form stays: it matches to the pixel. Request,
section 6.

Acceptance, each in the browser and the terminal:

- **Forge, then Model, always.**
- **Forge done:** `Forge Forgejo on this machine · connected as temper · work/hello made`, the last part while a first
  repository was made. "Open Forgejo ↗" stays at the row's end.
- **The line:** 15 px, what follows the strong name muted, as `.checklist`.
- **Under way, a line's ring spins:** Forge while it sets up, Model while signing in, by `.checklist .current .mark`.
- **`Forge none yet · Connect a forge`:** one line, the link after " · ", as Model's is.
- **Model's words:** signing in `Model ChatGPT plan · signing in…`; a program `Claude subscription · claude-owner, signed
  in on this machine`; a key `OpenAI API key · billed to the key’s organization`.
- **Under the form,** while a forge is under way and not ready, "Forgejo is still setting up; this page moves on by
  itself."
- **The foot:** "‹ Back" to Model, and "Try the playground instead" while offered.
- **"Set up again"** keeps its place, at its natural width.
- A render beside the mock's `welcome.html?at=ready` at 1440, 1000 and 420: each state of `Holds` below.

## Inputs

- `temper/docs/plan/ui-welcome/notes/common.md`, `notes/contracts.md`.
- `temper/docs/plan/ui-welcome/inputs/request.md`, section 6.
- `temper/docs/design/client/README.md`, 3.5, *Ready*; 5.5, *An account*.
- The mock: `temper/docs/design/client/greenfield/app.js`, `welcomePage()`'s last branch; `mock.css`'s `.checklist`.

## Owns and may change

Owns the files in `files`.

May also change: `temper/crates/temper-client-view/src/tests/{model_forge,connect_forge}.rs`, `temper/tests/client/tests/suite/{welcome,welcome_m3,welcome_m3_owned,welcome_m4,welcome_edges}.rs`,
`temper/crates/temper/tests/fake_backends/walk/{sign_in,open_default}.rs` and `temper/crates/temper/tests/support/person.rs`:
a line's words or order, changed with it. `temper/docs/design/client/README.status.md`: delete "3.5, *Ready*".

Options, pre-approved:

- **The email:** "ChatGPT plan · owner@example.com" needs jig's `account-facts`; until it lands the line says "ChatGPT plan"
  alone. Not this plan's.
- `first_usable(root, w, true)`'s words change for program and key on the short form only: `first_usable_kind`, Home's
  lede, stays. If another reader takes the short form, it takes the new words; say so.
- `own_signing_name` gives the plan's or brand's word: "PLAN plan · signing in…" with a plan, "BRAND · signing in…" with
  none. Its other readers keep theirs.
- The terminal's ring has no form: a line under way prints its words.

Not this task's: copies' words and act (`ready-copies`).

## Steps

1. **The order.** `ready`: the forge's `Item`, then `model_line`, whatever `done` says.
2. **Forge's words.** Connected: reuse `WelcomeForgeDoneDetail` and `WelcomeForgeDoneRepository`, as `forge-step`'s status
   does. `labelled` draws the rest muted. None yet: as `model_line`'s none: a `Slot` styled `ACCOUNT_LINE`, the text's
   runs ending " · ", then `go` to Forge.
3. **`WELCOME_CURRENT`** on an `Item` under way (`connect::underway(row)`, `own_signing_name`), by `notes/contracts.md`.
4. **Model's words,** as above: `first_usable_short`; `WelcomeModelSigningIn`.
5. **The note and foot.** `Word::WelcomeForgeStillSettingUp`, `FLOW_FACTS`, under the form while the row is `underway`
   and not `forge_connected`. `main`'s foot arm takes `Step::Ready`.
6. **The look,** under the marker: the rules that select `.temper-welcome-checklist`: 15 px, its paragraph muted and its
   `strong` ink; `.temper-welcome-current`'s ring and `animation: spin .9s linear infinite`, with a no-motion rule as
   `temper-flow-busy` has one. "Set up again": natural width.
7. **Tests,** `temper-client-view/src/tests/welcome.rs`, browser and terminal: the order at every combination of done;
   each Forge and Model line; the note; the foot, with and without the playground; `WELCOME_CURRENT`. Gallery:
   `welcome-ready-none`, `welcome-ready-signing`, in `names.txt`, `samples.rs`, `tests.rs` and `browser.md`'s 7.2 table.
   `welcome-ready-under-way`, `welcome-ready-adopted`: their `Holds`.
8. **Check,** `notes/common.md`; reprint journeys: the terminal's goldens move for the order and the new lines, each
   with a reason in your report; render proof; commit and push.

## Report

The lines as printed in the terminal; goldens moved and why; the builder and directory.
