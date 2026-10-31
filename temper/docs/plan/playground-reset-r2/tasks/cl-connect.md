+++
id = "cl-connect"
part = "client"
kind = "feature"
needs = ["cl-surface"]
files = [
  "temper/crates/temper-client-domain/src/address.rs",
  "temper/crates/temper-client-domain/src/forge.rs",
  "temper/crates/temper-client-domain/src/forge/{connect,forges,admin,github,picker,family,form,start,words}.rs",
  "temper/crates/temper-client-domain/src/tests/{connect,forges,admin,github_app,address,form}.rs",
  "temper/crates/temper-client-view/src/forge/bindings.rs",
  "temper/crates/temper-client-tools/**",
  "temper/crates/temper-client-shell-cli/src/tests/**",
  "temper/docs/design/client/journeys/welcome-set-up/**",
]
effort = "high"
minutes = 75
+++

## Goal

The client's forge domain on records (draft 3, "Its client"; 6.4): the
Forge step's and Connect a forge's choices are kinds to make (GitHub,
Forgejo on this machine), never configured slots; `SetUp` and a new
GitHub's `Connect` name no forge and follow the number their answer
names; a kept forge not finished is shown as such, to start over (a
failed set up starts over, never retries in place) or, for GitHub, to go
on; the next set up after a start over reuses the welcome's `Default`
project (draft 9's default). The view's pages are `cl-connect-view`'s:
here the view changes only where the domain's names force it.

Acceptance (`temper-client-domain`, `src/tests/`):

- `connect.rs`: `the_choices_are_kinds_not_slots` (no rows: `[New(GitHub),
  New(Here)]`; a kept GitHub row adds no second GitHub choice while it is
  not finished); `an_unfinished_forge_is_shown_not_finished`
  (`Shown::Unfinished(n)` for a kept real row not `finished` and not
  underway; a playground's forge never); `an_underway_set_up_is_followed`
  (as today: `Shown::SettingUp(n)`).
- `forges.rs`: `a_set_up_names_no_forge_and_follows_the_answered_number`
  (the sent `SetUp` has no forge; `SettingUp(5)` makes `set_up().forge ==
  Some(5)`; `SettingUp(0)`, the bridge's, leaves it `None` and the row
  followed is the newest owned row not finished);
  `a_failed_set_up_starts_over_rather_than_retrying` (`Act::SetUp` with a
  failed kept owned row sends nothing; the step reads `Unfinished`);
  `a_set_up_after_start_over_reuses_the_default_project` (a project named
  `Default` in `Me`'s projects: the at-once set up sends
  `SetUpProject::Existing(it)`).
- `github_app.rs`: `a_new_github_connects_without_a_forge` and
  `go_on_connects_the_kept_github` (`Connect { forge: Some(n) }`), and
  the `Link`'s number followed.
- `address.rs`: `elsewhere_is_no_choice` (a `forge=elsewhere` query
  parses as no choice).
- The focused tests of every package in Steps 6.

## Inputs

- `temper/docs/plan/playground-reset-r2/notes/common.md` and
  `temper/docs/plan/playground-reset-r2/notes/scope.md`.
- `temper/docs/plan/playground-reset-r2/notes/contracts.md`, 5.
- `temper/docs/plan/playground-reset-r2/tasks/cl-wire.md` and
  `cl-surface.md` (what merged: `Choice::New`, `Row.finished`, the
  bridges' zero numbers).
- `temper/docs/design/draft/playground-reset.md`, 3, 4 ("On the forge
  itself"), 6.3, 6.4, 9 (the welcome's Default project).
- `temper/docs/design/client/forge.md`, 5.4 (connecting a forge).
- `temper/docs/design/client/README.md`, 3.5 (the welcome's Forge step).

## Owns and may change

Owns the files in the header. Pre-approved:

- `Choice::Elsewhere` goes, with its parse and print (a document edit no
  longer adds a forge: draft 3); its view (`V/forge/connect.rs`'s
  `elsewhere`, the `next_number` instructions) goes with it as forced
  edits. A Forgejo connected elsewhere, made from the client, is later:
  the wire's `ForgejoAdmin` carries no address. A kept connected or
  managed Forgejo (made by other means) still connects with
  `ConnectAdmin { forge }` as today.
- `connect::next_number` goes.
- Forced edits (scope.md) in `temper-client-view` (any file: the acts'
  and `Meaning`s' new payloads, `Shown::Unfinished` drawn as `Choose`
  until `cl-connect-view`), `temper-client-tools` (its `SetUp`,
  `Connect` and their answers: a ctl set up names no forge and prints the
  answered number), `temper-client-shell-*`, `temper/tests/client`,
  `temper/tools/gallery`.
- If `Offer` on the wire row no longer says whether "Forgejo on this
  machine" is offered (no row exists before it is made): offer `Here`
  always, and show the `NotUnderTheBox` refusal on its answer as the
  view shows a set up refusal today.

Not yours: `D/forge/{demo,listed,remove,vocabulary}.rs`,
`D/{domain,route,pages}.rs`, `D/pages/**` (other tasks hold them now);
call their public items as they are.

## Steps

Paths: D = `temper/crates/temper-client-domain/src`, V =
`temper/crates/temper-client-view/src`.

1. **Choices (D/forge/connect.rs).** `choices(rows, demo)` returns
   `New(GitHub)` unless a kept real GitHub row exists, then `New(Here)`
   unless a kept owned Forgejo row exists; a kept row is reached by
   `Choice::Forge(n)` (its details, as today). `Shown` gains
   `Unfinished(u16)`; `step` and `welcome_step` give it for a kept real
   row (`mode != Fake`, not the playground's forge) that is not
   `finished`, not `underway`, and not offered for a connection today
   (an admin Forgejo waiting for its token stays `Details`). `pub fn
   unfinished<'a>(rows: &'a State, demo: Option<&Demo>) -> Option<&'a
   Row>`: the first such row, for the welcome (draft 6.3).
2. **Acts and payloads (D/forge/form.rs, D/forge.rs).**
   `SetUp`, `SetUpAtOnce` lose `forge`; `ConnectApp { forge:
   Option<u16> }`, `ConnectManifest { forge: Option<u16>, back }`
   (`None`: a new GitHub; `Some(n)`: go on with kept `n`, draft 6.4);
   `ConnectAdmin`, `Install`, `StartIn`, `Picker` keep `u16`. `StartIn`
   for a failed first repository stays (it retries the adoption, not the
   set up).
3. **Set up (D/forge/forges.rs).** `SetUp.forge: Option<u16>`, set from
   the answer; `SetUpOutcome::SettingUp { forge: Option<u16> }`
   (`None` for the bridge's 0); `ask(org, repository, project,
   playground, limits)` without a forge; `retry_fields` goes; `set_up`
   refuses (sends nothing) while an unfinished kept owned row exists, so
   that its Start over (`cl-remove`) comes first. The forge a set up
   follows before its number: the newest (highest number) owned row not
   finished. `set_up_at_once` takes `SetUpProject::Existing(p)` when
   `Me`'s projects hold one named `AT_ONCE_PROJECT` (the read
   `start::same_project_name` uses), else `New` as today.
4. **GitHub (D/forge/github.rs) and admin (D/forge/admin.rs).**
   `github::send` and `manifest` take `Option<u16>`; their pending state
   records the number the answer names (`Link { forge }`,
   `Connecting(forge)`), and `None` until then; `available` and the
   installation flow use the recorded number. `admin::send` is
   unchanged but for `Connect { forge: Some(forge) }`.
5. **Family (D/forge/family.rs)** decodes the answers' numbers into the
   vocabulary above (0 → `None`).
6. **Checks:** the tests above; `cargo fmt`; `heavy cargo clippy -p
   temper-client-domain -p temper-client-view -p temper-client-tools -p
   temper-client-shell-cli -p temper-client-shell-terminal -p
   temper-client-shell-web -p temper-client-world -p
   temper-client-gallery --all-targets -- -D warnings`; one `heavy cargo
   nextest run` over the same packages and `temper-client-journeys`.
7. **Journeys:** `welcome-set-up` records `Forge(SetUpAtOnce { forge: 3
   })`; it is now `Forge(SetUpAtOnce)`. Put the recording's `act` line's
   event text to the new Debug print, then `heavy cargo run -p
   temper-client-journeys -- again welcome-set-up` (journeys.md, 8.1),
   and reprint (`heavy cargo run -p temper-client-journeys`). If `again`
   refuses, stop and ask (`Question:`, with its refusal).
8. Old forms from `temper/`; commit and push.

## Report

- The final public items of steps 1-4 (`cl-connect-view`, `cl-welcome`
  and `cl-tools` read them).
- Whether `Here` is offered always or by an `Offer`, and why.
- Each forced edit outside the domain, by file.
