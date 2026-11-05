+++
id = "to-begin-story"
part = "to-begin-story"
kind = "feature"
needs = ["to-begin-facts"]
files = [
  "temper/tests/client/tests/suite/to_begin.rs",
  "temper/tests/client/tests/suite/main.rs",
  "temper/tests/client/src/front/dismissed.rs",
]
effort = "high"
minutes = 45
+++

# "Skip" across the real engine

## Goal

The meeting point of the draft's section 5 and the view's facts: `Act::SkipToBegin`, pressed on a real
`Home`, through jig's client and the real engine, in temper's clients' world. Every commit carries
`Look-request: project-home-onboarding`.

Acceptance, stories in `temper/tests/client/tests/suite/to_begin.rs`, on the shape of
`suite/dismissed.rs`:

- `skip_on_home_hides_to_begin_on_every_device`: two clients of one person on `Home { project }`, To
  begin showing on both (`pages::home::facts(..).to_begin()`); one presses; once settled neither
  shows it, and a third client signing in then never does.
- `skip_holds_across_a_restart`: the engine restarts; after the watches reopen, still hidden.
- `skip_leaves_the_others_to_begin`: another member of the project still sees it, and the person's
  other project too.
- `a_double_press_sends_one_request`, and one row in the store.
- jig's check holds in every story, as the suite's others.

## Inputs

- `temper/docs/plan/look-round-4/notes/common.md`, `notes/contracts.md` (What To begin reads).
- `temper/docs/design/client/README.md`, 3.5, *To begin*, and section 10.
- `temper/tests/client/tests/suite/dismissed.rs`, `temper/tests/client/src/front/dismissed.rs`.

## Owns and may change

Owns, as `files` lists them. May also change `temper/tests/client/src/front.rs` and
`temper/tests/client/src/component.rs`: a helper the stories share.

Options, each pre-approved:

- If `to_begin()` can't go true in this world because a fact isn't delivered there (`settings`, the
  chats' read), deliver it through the world's fixtures; if that isn't possible, assert on
  `Facts.skip` and say so in the report.
- Don't change `temper-client-domain`: a fact missing there is a `Question:`.

## Steps

1. The stories, then each bites: drop the `Dismiss` send, restore.
2. **Check,** `notes/common.md`: `heavy cargo nextest run -p temper-client-world -E
   'test(to_begin)'`, then `-E 'test(dismissed)'`, clippy on `temper-client-world`, push. Say the
   world's budget (`temper/docs/design/domain/testing.md`, 2.1) still holds.

## Report

As `notes/common.md`, Your report.
