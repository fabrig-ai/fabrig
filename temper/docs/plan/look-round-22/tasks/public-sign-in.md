+++
id = "public-sign-in"
part = "public"
kind = "feature"
needs = []
files = [
  "temper/crates/temper-client-view/src/public.rs",
  "temper/crates/temper-client-view/src/tests/public.rs",
  "temper/docs/design/client/browser.status.md",
]
effort = "medium"
minutes = 30
+++

# "Sign in" on the public pages' top bar, with or without Google

## Goal

The request's task 4 (`temper/docs/plan/look-round-22/inputs/request.md`). The public pages' top bar offers "Sign in" at its right only where the deployment offers Google's way (`build_public` in `temper/crates/temper-client-view/src/public.rs`, about line 97: `if view.ways.google && let Some(origin) = &view.ways.origin`). The request has it unqualified: it shows with or without Google, and leads to the page's origin, `/`, whose signed-out page is the sign-in page with Google and otherwise the column "Signed out" with the printed link's words.

Acceptance, by `temper/docs/design/client/browser.md`, 8.2 (its last bullet, *Anyone, signed in or not*) and 2.5, *Outside a project*:

1. **`build_public` draws the `TOP_END` slot with the "Sign in" link to `ORIGIN/` whenever `view.ways.origin` is `Some`,** whatever `view.ways.google` says; with no origin, no slot, as now. The link, its words, its slot and the address are as today. `PublicView::new` and `Ways` keep their signatures; nothing else of the public pages changes.
2. **The rows:** the existing table test `projects_are_named_and_the_banner_only_offers_a_known_sign_in` (`temper/crates/temper-client-view/src/tests/public.rs`, about line 220) keeps its rows' shape `(ways, sign_in)`, with these rows:

   ```rust
   (Ways::NONE, None),
   (Ways::parse("", b"https://temper.example"), Some(b"https://temper.example/".as_slice())),
   (Ways::parse("unknown", b"https://temper.example"), Some(b"https://temper.example/".as_slice())),
   (Ways { google: true, origin: None }, None),
   (Ways::parse("google", b"https://temper.example"), Some(b"https://temper.example/".as_slice())),
   ```

   and is renamed `projects_are_named_and_the_top_bar_offers_sign_in_from_the_pages_origin`. No new test function.
3. **Status:** delete the **Not yet** entry naming `public-sign-in` in `temper/docs/design/client/browser.status.md` (the one under section 8.2). No other design line.
4. Runs through `heavy`, counts in the report: `cargo nextest run -p temper-client-view` and `-p temper-client-shell-web` (its public page builds this view); `cargo clippy -p temper-client-view --all-targets -- -D warnings`.

One commit or more, each with the trailer `Look-request: outside-pages`; the last carries the account (below).

## Inputs

- `temper/docs/plan/look-round-22/notes/common.md`.
- `temper/docs/plan/look-round-22/inputs/request.md`, task 4.
- `temper/docs/design/client/browser.md`, 8.2, *Anyone, signed in or not*; 2.5, *Outside a project* (the top bar).
- `temper/docs/design/client/README.md`, 5.1, *The pages not about the work* (the signed-out column a deployment without Google shows at `/`).
- The view's ways: `temper/crates/temper-client-view/src/sign_in.rs`, `Ways` and `Ways::parse` (an origin is kept for any `http` or `https` page, Google or not).

## Owns and may change

Owns the condition in `build_public`, the table's rows and name, and the status entry. Not `sign_in.rs`, the shells, `outside.rs` or `root.rs` (`look-round-22`'s view task changes those), or any other test.

Options, each decided:

- **`PublicView`'s `ways` keeps its `google` field unread:** keep the field and `Ways` as they are (the shells pass it, and the sign-in page reads it); if clippy flags a field never read, read `origin` alone through the struct as now and keep the field. Never change `Ways`.
- **Another test asserts no "Sign in" on a public page without Google** (`rg -n "Sign in" temper/crates/temper-client-view/src/tests temper/crates/temper-client-shell-web`): change it to expect the link, keeping what else it checks, and name it.

## Steps

1. Read the request's task 4 and the Inputs; change the condition (acceptance 1).
2. The rows and the name (acceptance 2); bite it: restore `view.ways.google &&`, see the `""` and `"unknown"` rows fail on the sign-in slot, and restore.
3. The status entry (acceptance 3); the runs of acceptance 4; `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `python3 -I -B tools/citations/citations.py --base origin/main`.
4. Commit with the trailer and the account; push with `git push origin HEAD:refs/heads/<your branch>`.

The account's rows, at least:

| Item | Result |
|---|---|
| Public pages: "Sign in" at the top bar's right, with or without Google | landed (`public.rs`, `build_public`) |
| Without Google, it leads to `/`, the signed-out column's printed-link words | landed: the same link to the origin; the column is `README.md`, 5.1's, unchanged |
| Render check | follows landing, temper/ui's (`temper/docs/design/draft/look-requests.md`, 4): focused tests establish no rendered equality |

## Report

The implementer's report: the condition as written, the rows, the bite and its failure, the runs' counts, and the account as committed.
